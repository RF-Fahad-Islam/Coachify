use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::lan::{
    ensure_crypto_provider_installed, parse_qr_pairing_uri, race_connect_candidate_ips,
    set_test_db_conn, FingerprintPinnedCertVerifier, MobileClientState,
};
use rusqlite::{params, Connection};
use rustls::pki_types::ServerName;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

static SERIAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_test_env() -> (Arc<Mutex<Connection>>, String, String) {
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let mgr = coachify_core::lan::lan_manager();
    if mgr.is_running() {
        let dummy_conn = Connection::open_in_memory().unwrap();
        let _ = coachify_core::lan::stop_host(&dummy_conn, now);
        std::thread::sleep(Duration::from_millis(250));
    }
    set_test_db_conn(None);

    let mut raw_conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&raw_conn).expect("apply all migrations");

    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    raw_conn
        .execute(
            "INSERT INTO institution (id, name, address, phones, currency, timezone, created_at, updated_at)
             VALUES (1, 'Apex Coaching Center', 'Dhaka', '01700000000', 'BDT', 'Asia/Dhaka', ?1, ?1)",
            [now],
        )
        .unwrap();

    let boot_payload = json!({
        "username": "owner",
        "display_name": "Principal Owner",
        "password": "OwnerPassword123!"
    })
    .to_string();

    let (res_str, _) = dispatch_at(
        &mut raw_conn,
        "system.bootstrap_owner",
        &boot_payload,
        None,
        None,
        now,
    );
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(res["ok"].as_bool().unwrap(), "bootstrap failed: {res}");
    let token = res["data"]["session_token"].as_str().unwrap().to_string();
    let user_id = res["data"]["user"]["id"].as_str().unwrap().to_string();

    let shared = Arc::new(Mutex::new(raw_conn));
    set_test_db_conn(Some(shared.clone()));

    (shared, token, user_id)
}

fn cleanup_test_env(conn: &Arc<Mutex<Connection>>, token: &str, now: &str) {
    let _ = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.stop_host",
        "{}",
        Some(token),
        None,
        now,
    );
    set_test_db_conn(None);
    std::thread::sleep(Duration::from_millis(250));
}

fn run_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(f)
    })
    .join()
    .unwrap()
}

async fn send_pinned_tls_request(
    ip: &str,
    port: u16,
    fingerprint: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Result<(u16, Value), String> {
    ensure_crypto_provider_installed();
    let verifier = Arc::new(FingerprintPinnedCertVerifier::new(fingerprint));
    let client_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

    let tcp_stream = tokio::net::TcpStream::connect((ip, port))
        .await
        .map_err(|e| format!("TCP connect failed: {e}"))?;

    let server_name =
        ServerName::try_from("coachify.lan").map_err(|e| format!("ServerName error: {e}"))?;

    let mut tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .map_err(|e| format!("TLS connect failed: {e}"))?;

    let mut req_str = format!(
        "{} {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nContent-Length: {}\r\n",
        method,
        path,
        ip,
        port,
        body.len()
    );
    for (k, v) in headers {
        req_str.push_str(&format!("{}: {}\r\n", k, v));
    }
    req_str.push_str("\r\n");

    tls_stream
        .write_all(req_str.as_bytes())
        .await
        .map_err(|e| format!("Write headers failed: {e}"))?;
    if !body.is_empty() {
        tls_stream
            .write_all(body.as_bytes())
            .await
            .map_err(|e| format!("Write body failed: {e}"))?;
    }

    let mut resp_bytes = Vec::new();
    tls_stream
        .read_to_end(&mut resp_bytes)
        .await
        .map_err(|e| format!("Read response failed: {e}"))?;

    let resp_str = String::from_utf8_lossy(&resp_bytes);
    let first_line = resp_str.lines().next().unwrap_or("");
    let status_code: u16 = first_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let body_part = if let Some(pos) = resp_str.find("\r\n\r\n") {
        &resp_str[pos + 4..]
    } else {
        ""
    };

    let json_val: Value =
        serde_json::from_str(body_part).unwrap_or_else(|_| json!({ "raw": body_part }));

    Ok((status_code, json_val))
}

// -----------------------------------------------------------------------------
// Test 1: test_open_mobile_pairing_window_success (AC-1)
// -----------------------------------------------------------------------------
#[test]
fn test_open_mobile_pairing_window_success() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();

    // 1. Start host on safe non-excluded port
    let start_payload = json!({ "port": 48101, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );

    std::thread::sleep(Duration::from_millis(50));

    // 2. Open mobile pairing window
    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    assert!(
        open_res["ok"].as_bool().unwrap(),
        "open_mobile_pairing_window failed: {open_res}"
    );

    let data = &open_res["data"];
    let session_id = data["session_id"].as_str().unwrap();
    let qr_uri = data["qr_uri"].as_str().unwrap();
    let fp = data["host_fingerprint"].as_str().unwrap();
    let pairing_code = data["code"].as_str().unwrap();

    assert!(
        qr_uri.starts_with("coachify://pair?v=1"),
        "QR URI must have coachify scheme"
    );
    assert_eq!(pairing_code.len(), 6, "Pairing code must be 6 digits");
    assert!(fp.starts_with("SHA256:"));

    // 3. Verify lan_pairing_sessions has active session with qr_secret_hash
    let (status, qr_hash_opt): (String, Option<String>) = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT status, qr_secret_hash FROM lan_pairing_sessions WHERE id = ?1",
            params![session_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(status, "pending");
    assert!(qr_hash_opt.is_some(), "qr_secret_hash must be stored");

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 2: test_pairing_brute_force_lockout (AC-2)
// -----------------------------------------------------------------------------
#[test]
fn test_pairing_brute_force_lockout() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48102;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let fp = open_res["data"]["host_fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    let invalid_payload = json!({
        "qr_secret": "wrong_secret_1234567890abcdef",
        "device_name": "Test Attacker Phone",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce123",
        "app_version": "0.1.0"
    })
    .to_string();

    run_async(async move {
        // First 4 attempts return HTTP 403 PAIRING_SECRET_INVALID
        for i in 1..=4 {
            let (status, resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &invalid_payload,
            )
            .await
            .expect("attempt failed");

            assert_eq!(status, 403, "attempt {i} status should be 403");
            assert_eq!(
                resp["code"], "PAIRING_SECRET_INVALID",
                "attempt {i} code mismatch: {resp}"
            );
        }

        // 5th attempt triggers lockout and returns HTTP 403 PAIRING_LOCKED
        let (status5, resp5) = send_pinned_tls_request(
            "127.0.0.1",
            port,
            &fp,
            "POST",
            "/pair/qr",
            &[("Content-Type", "application/json")],
            &invalid_payload,
        )
        .await
        .expect("attempt 5 failed");

        assert_eq!(status5, 403);
        assert_eq!(
            resp5["code"], "PAIRING_LOCKED",
            "5th attempt must return PAIRING_LOCKED: {resp5}"
        );

        // 6th attempt within cooldown window returns PAIRING_COOLDOWN_ACTIVE
        let (status6, resp6) = send_pinned_tls_request(
            "127.0.0.1",
            port,
            &fp,
            "POST",
            "/pair/qr",
            &[("Content-Type", "application/json")],
            &invalid_payload,
        )
        .await
        .expect("attempt 6 failed");

        assert_eq!(status6, 403);
        assert_eq!(
            resp6["code"], "PAIRING_COOLDOWN_ACTIVE",
            "6th attempt must return PAIRING_COOLDOWN_ACTIVE: {resp6}"
        );
    });

    // Verify audit log has security.pairing_brute_force_locked
    let audit_count: i64 = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'security.pairing_brute_force_locked'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(audit_count >= 1, "Must record security audit event");

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 3: test_qr_pairing_handshake_flow (AC-4)
// -----------------------------------------------------------------------------
#[test]
fn test_qr_pairing_handshake_flow() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48103;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let qr_uri = open_res["data"]["qr_uri"].as_str().unwrap();

    let parsed = parse_qr_pairing_uri(qr_uri).expect("parse qr uri");
    let valid_secret = parsed.secret;
    let fp = parsed.fingerprint;

    let pair_payload = json!({
        "qr_secret": valid_secret,
        "device_name": "Pixel 8",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce_xyz_987",
        "app_version": "0.1.0"
    })
    .to_string();

    let (device_id, device_token) = run_async({
        let fp = fp.clone();
        let pair_payload = pair_payload.clone();
        async move {
            let (status, resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair request failed");

            assert_eq!(status, 200, "pairing should succeed: {resp}");
            let dt = resp["device_token"].as_str().unwrap().to_string();
            let did = resp["device_id"].as_str().unwrap().to_string();
            assert!(
                dt.starts_with("dt_"),
                "device token must be prefixed with dt_"
            );
            (did, dt)
        }
    });

    // Verify devices table has android / mobile_phone / qr / active
    let (platform, dev_type, paired_via, dev_status): (String, String, String, String) = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT platform, device_type, paired_via, status FROM devices WHERE id = ?1",
            params![device_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();

    assert_eq!(platform, "android");
    assert_eq!(dev_type, "mobile_phone");
    assert_eq!(paired_via, "qr");
    assert_eq!(dev_status, "active");

    // Verify session status is 'used'
    let session_status: String = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT status FROM lan_pairing_sessions WHERE paired_device_id = ?1",
            params![device_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(session_status, "used");

    // Replay attempt with same qr_secret should return 409 CONFLICT PAIRING_SESSION_ALREADY_USED
    run_async({
        let fp = fp.clone();
        let pair_payload = pair_payload.clone();
        async move {
            let (replay_status, replay_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("replay request failed");

            assert_eq!(replay_status, 409, "Replay must return 409 Conflict");
            assert_eq!(replay_resp["code"], "PAIRING_SESSION_ALREADY_USED");
        }
    });

    let _ = device_token;
    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 4: test_tls_fingerprint_pinning_rejection (AC-3)
// -----------------------------------------------------------------------------
#[test]
fn test_tls_fingerprint_pinning_rejection() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48104;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    // Corrupted fingerprint
    let corrupted_fp = "SHA256:00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff";

    run_async(async move {
        let result =
            send_pinned_tls_request("127.0.0.1", port, corrupted_fp, "GET", "/hello", &[], "")
                .await;

        assert!(
            result.is_err(),
            "TLS handshake must be rejected with mismatched fingerprint"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("TLS connect failed") || err_msg.contains("InvalidCertificate"),
            "Handshake must fail at transport: {err_msg}"
        );
    });

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 5: test_mobile_pairing_respects_seat_limit (AC-5)
// -----------------------------------------------------------------------------
#[test]
fn test_mobile_pairing_respects_seat_limit() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48105;

    // Set license workstation limit = 1 in limits_json
    conn.lock().unwrap().execute(
        "UPDATE licenses SET limits_json = '{\"workstations\": 1}', tier = 'local_pro' WHERE is_active = 1",
        [],
    )
    .unwrap();

    // Add 1 active desktop workstation
    let desktop_id = "desktop-pc-1";
    conn.lock().unwrap().execute(
        "INSERT INTO devices (id, name, windows_name, ip_address, app_version, status, is_host, paired_at, last_seen_at, token_hash)
         VALUES (?1, 'Front Desk PC', 'DESKTOP-FRONT', '192.168.1.5', '0.1.0', 'active', 0, ?2, ?2, 'dummy_token_hash_desktop')",
        params![desktop_id, now],
    )
    .unwrap();

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let qr_uri = open_res["data"]["qr_uri"].as_str().unwrap();
    let parsed = parse_qr_pairing_uri(qr_uri).expect("parse qr");

    let pair_payload = json!({
        "qr_secret": parsed.secret,
        "device_name": "Teacher Tablet",
        "platform": "ios",
        "device_type": "tablet",
        "client_nonce": "nonce_tablet",
        "app_version": "0.1.0"
    })
    .to_string();

    // 1. Attempt pairing while quota is full
    run_async({
        let fp = parsed.fingerprint.clone();
        let pair_payload = pair_payload.clone();
        async move {
            let (status_limit, resp_limit) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair request failed");

            assert_eq!(status_limit, 403);
            assert_eq!(resp_limit["code"], "LAN_WORKSTATION_LIMIT_REACHED");
        }
    });

    // 2. Revoke the desktop workstation to free up a seat
    let revoke_payload = json!({
        "device_id": desktop_id,
        "reason": "Replaced with mobile tablet"
    })
    .to_string();
    let (revoke_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.revoke_workstation",
        &revoke_payload,
        Some(&token),
        None,
        now,
    );
    let revoke_res: Value = serde_json::from_str(&revoke_res_str).unwrap();
    assert!(
        revoke_res["ok"].as_bool().unwrap(),
        "revoke failed: {revoke_res}"
    );

    // 3. Retry mobile pairing - now succeeds
    run_async({
        let fp = parsed.fingerprint.clone();
        let pair_payload = pair_payload.clone();
        async move {
            let (status_ok, resp_ok) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair retry failed");

            assert_eq!(
                status_ok, 200,
                "pairing must succeed after seat freed: {resp_ok}"
            );
            assert!(resp_ok["device_token"].as_str().unwrap().starts_with("dt_"));
        }
    });

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 6: test_mobile_instant_revocation_sla (AC-9)
// -----------------------------------------------------------------------------
#[test]
fn test_mobile_instant_revocation_sla() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48106;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let qr_uri = open_res["data"]["qr_uri"].as_str().unwrap();
    let parsed = parse_qr_pairing_uri(qr_uri).expect("parse qr");

    // Pair device
    let pair_payload = json!({
        "qr_secret": parsed.secret,
        "device_name": "Field Phone",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce_field",
        "app_version": "0.1.0"
    })
    .to_string();

    let (device_id, device_token) = run_async({
        let fp = parsed.fingerprint.clone();
        let pair_payload = pair_payload.clone();
        async move {
            let (pair_status, pair_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair failed");

            assert_eq!(pair_status, 200);
            (
                pair_resp["device_id"].as_str().unwrap().to_string(),
                pair_resp["device_token"].as_str().unwrap().to_string(),
            )
        }
    });

    // Verify active RPC works
    let rpc_payload = json!({
        "method": "system.get_status",
        "params": {}
    })
    .to_string();

    run_async({
        let fp = parsed.fingerprint.clone();
        let dt = device_token.clone();
        let rpc_payload = rpc_payload.clone();
        async move {
            let (rpc_status, rpc_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/rpc",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", &dt),
                ],
                &rpc_payload,
            )
            .await
            .expect("initial rpc failed");

            assert_eq!(rpc_status, 200);
            assert!(rpc_resp["ok"].as_bool().unwrap());
        }
    });

    // Host administrator revokes device
    let revoke_payload = json!({
        "device_id": device_id,
        "reason": "Security protocol testing"
    })
    .to_string();

    let start_revoke = std::time::Instant::now();
    let (revoke_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.revoke_workstation",
        &revoke_payload,
        Some(&token),
        None,
        now,
    );
    let revoke_res: Value = serde_json::from_str(&revoke_res_str).unwrap();
    assert!(revoke_res["ok"].as_bool().unwrap());
    let revoke_elapsed = start_revoke.elapsed();
    assert!(
        revoke_elapsed < Duration::from_millis(500),
        "Revocation SLA must be under 500ms"
    );

    // Subsequent RPC call with device token must fail immediately with 403 DEVICE_REVOKED
    run_async({
        let fp = parsed.fingerprint.clone();
        let dt = device_token.clone();
        let rpc_payload = rpc_payload.clone();
        async move {
            let (post_status, post_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/rpc",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", &dt),
                ],
                &rpc_payload,
            )
            .await
            .expect("post revoke rpc failed");

            assert_eq!(
                post_status, 403,
                "Revoked device must be rejected with 403: {post_resp}"
            );
            assert_eq!(post_resp["error"]["code"], "DEVICE_REVOKED");
        }
    });

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 7: test_ip_candidate_racing_fast_connect (AC-8)
// -----------------------------------------------------------------------------
#[test]
fn test_ip_candidate_racing_fast_connect() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48107;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let fp = open_res["data"]["host_fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    // One dead/unroutable candidate IP and one valid local candidate IP
    let candidates = vec![
        "198.51.100.1".to_string(), // RFC 5737 TEST-NET-2 (unroutable black hole)
        "127.0.0.1".to_string(),    // valid local host
    ];

    let (winning_ip, elapsed) = run_async(async move {
        let start_race = std::time::Instant::now();
        let winning =
            race_connect_candidate_ips(&candidates, port, &fp, Duration::from_millis(2500))
                .await
                .expect("race connection failed");
        (winning, start_race.elapsed())
    });

    assert_eq!(winning_ip, "127.0.0.1", "Valid candidate must win the race");
    assert!(
        elapsed < Duration::from_millis(1500),
        "Fast connect should resolve well under 1.5s: {:?}",
        elapsed
    );

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 8: test_staff_pin_authentication_and_session_timeout
// -----------------------------------------------------------------------------
#[test]
fn test_staff_pin_authentication_and_session_timeout() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48108;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    // Set staff PIN for owner user
    let pin_payload = json!({
        "user_id": user_id,
        "staff_pin": "5678"
    })
    .to_string();
    let (pin_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.set_staff_pin",
        &pin_payload,
        Some(&token),
        None,
        now,
    );
    let pin_res: Value = serde_json::from_str(&pin_res_str).unwrap();
    assert!(
        pin_res["ok"].as_bool().unwrap(),
        "set staff pin failed: {pin_res}"
    );

    // Pair device
    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let parsed = parse_qr_pairing_uri(open_res["data"]["qr_uri"].as_str().unwrap()).unwrap();

    let pair_payload = json!({
        "qr_secret": parsed.secret,
        "device_name": "Cashier Phone",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce_cashier",
        "app_version": "0.1.0"
    })
    .to_string();

    let (device_token, staff_session_token) = run_async({
        let fp = parsed.fingerprint.clone();
        async move {
            let (pair_status, pair_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair failed");

            assert_eq!(pair_status, 200);
            let dt = pair_resp["device_token"].as_str().unwrap().to_string();

            // Authenticate with PIN
            let login_payload = json!({
                "staff_pin": "5678",
                "device_token": dt
            })
            .to_string();

            let (login_status, login_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/auth/staff-login",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", &dt),
                ],
                &login_payload,
            )
            .await
            .expect("staff login failed");

            assert_eq!(login_status, 200, "staff login must succeed: {login_resp}");
            let st = login_resp["session_token"].as_str().unwrap().to_string();
            assert!(st.starts_with("st_"));

            // Execute RPC call using session token
            let rpc_payload = json!({
                "method": "system.get_status",
                "params": {}
            })
            .to_string();

            let (rpc_status, rpc_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/rpc",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", &dt),
                    ("X-Coachify-Session-Token", &st),
                ],
                &rpc_payload,
            )
            .await
            .expect("rpc call failed");

            assert_eq!(rpc_status, 200);
            assert!(rpc_resp["ok"].as_bool().unwrap());

            (dt, st)
        }
    });

    // Simulate 15-minute background eviction or expiry
    conn.lock().unwrap().execute(
        "UPDATE sessions SET expires_at = '2026-09-26T09:00:00Z', last_seen_at = '2026-09-26T09:00:00Z' WHERE id = ?1",
        params![staff_session_token],
    )
    .unwrap();

    // Subsequent RPC call must be rejected with SESSION_LOCKED_PIN_REQUIRED
    let rpc_payload = json!({
        "method": "system.get_status",
        "params": {}
    })
    .to_string();

    run_async({
        let fp = parsed.fingerprint.clone();
        let dt = device_token.clone();
        let st = staff_session_token.clone();
        async move {
            let (lock_status, lock_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/rpc",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", &dt),
                    ("X-Coachify-Session-Token", &st),
                ],
                &rpc_payload,
            )
            .await
            .expect("locked rpc failed");

            assert_eq!(
                lock_status, 401,
                "Expired/locked session must return 401: {lock_resp}"
            );
            assert_eq!(lock_resp["error"]["code"], "SESSION_LOCKED_PIN_REQUIRED");
        }
    });

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 9: test_designated_receipt_printer_dispatch (AC-7)
// -----------------------------------------------------------------------------
#[test]
fn test_designated_receipt_printer_dispatch() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port = 48109;

    let start_payload = json!({ "port": port, "bind_address": "127.0.0.1" }).to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start_host failed: {start_res}"
    );
    std::thread::sleep(Duration::from_millis(50));

    // Configure receipt_printer_name = 'POS-80C' in lan_host_config
    let printer_payload = json!({ "printer_name": "POS-80C" }).to_string();
    let (pr_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.update_host_receipt_printer",
        &printer_payload,
        Some(&token),
        None,
        now,
    );
    let pr_res: Value = serde_json::from_str(&pr_res_str).unwrap();
    assert!(
        pr_res["ok"].as_bool().unwrap(),
        "update_host_receipt_printer failed: {pr_res}"
    );

    // Pair mobile device
    let (open_res_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res: Value = serde_json::from_str(&open_res_str).unwrap();
    let parsed = parse_qr_pairing_uri(open_res["data"]["qr_uri"].as_str().unwrap()).unwrap();

    let pair_payload = json!({
        "qr_secret": parsed.secret,
        "device_name": "Counter POS Phone",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce_print",
        "app_version": "0.1.0"
    })
    .to_string();

    run_async({
        let fp = parsed.fingerprint.clone();
        async move {
            let (pair_status, pair_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair failed");

            assert_eq!(pair_status, 200);
            let device_token = pair_resp["device_token"].as_str().unwrap();

            // Print receipt request
            let print_payload = json!({
                "payment_id": "pay_test_001",
                "device_token": device_token,
                "receipt_template": "thermal_80mm",
                "copies": 1
            })
            .to_string();

            let (print_status, print_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port,
                &fp,
                "POST",
                "/print/host-receipt",
                &[
                    ("Content-Type", "application/json"),
                    ("X-Coachify-Device-Token", device_token),
                ],
                &print_payload,
            )
            .await
            .expect("print failed");

            assert_eq!(
                print_status, 200,
                "Receipt print must succeed: {print_resp}"
            );
            assert_eq!(
                print_resp["printer_name"], "POS-80C",
                "Must route specifically to designated receipt printer POS-80C"
            );
            assert_eq!(print_resp["status"], "dispatched");
            assert!(print_resp["print_job_id"]
                .as_str()
                .unwrap()
                .starts_with("pj_"));
        }
    });

    cleanup_test_env(&conn, &token, now);
}

// -----------------------------------------------------------------------------
// Test 10: test_quick_scan_ip_update_without_repairing (AC-8)
// -----------------------------------------------------------------------------
#[test]
fn test_quick_scan_ip_update_without_repairing() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (conn, token, _user_id) = setup_test_env();
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();
    let port1 = 48110;
    let port2 = 48111;

    // 1. Initial host start on port1
    let start_payload1 = json!({ "port": port1, "bind_address": "127.0.0.1" }).to_string();
    let (start_res1_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload1,
        Some(&token),
        None,
        now,
    );
    let start_res1: Value = serde_json::from_str(&start_res1_str).unwrap();
    assert!(
        start_res1["ok"].as_bool().unwrap(),
        "start_host failed: {start_res1}"
    );
    std::thread::sleep(Duration::from_millis(50));

    // Pair mobile device
    let (open_res1_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res1: Value = serde_json::from_str(&open_res1_str).unwrap();
    let parsed1 = parse_qr_pairing_uri(open_res1["data"]["qr_uri"].as_str().unwrap()).unwrap();

    let pair_payload = json!({
        "qr_secret": parsed1.secret,
        "device_name": "Roaming Mobile",
        "platform": "android",
        "device_type": "mobile_phone",
        "client_nonce": "nonce_roam",
        "app_version": "0.1.0"
    })
    .to_string();

    let (device_id, device_token, host_id) = run_async({
        let fp = parsed1.fingerprint.clone();
        async move {
            let (pair_status, pair_resp) = send_pinned_tls_request(
                "127.0.0.1",
                port1,
                &fp,
                "POST",
                "/pair/qr",
                &[("Content-Type", "application/json")],
                &pair_payload,
            )
            .await
            .expect("pair failed");

            assert_eq!(pair_status, 200);
            (
                pair_resp["device_id"].as_str().unwrap().to_string(),
                pair_resp["device_token"].as_str().unwrap().to_string(),
                pair_resp["host_id"].as_str().unwrap().to_string(),
            )
        }
    });

    // Create MobileClientState on mobile device
    let mut client_state = MobileClientState::new(
        device_id.clone(),
        device_token.clone(),
        host_id.clone(),
        parsed1.fingerprint.clone(),
        "127.0.0.1".to_string(),
        port1,
    );

    // Initial RPC works on port1
    let init_rpc = run_async({
        let cs = client_state.clone();
        async move { cs.execute_rpc("system.get_status", json!({})).await }
    });
    assert!(
        init_rpc.is_ok(),
        "Initial RPC on port1 must succeed: {:?}",
        init_rpc
    );

    // 2. Host changes IP/port (simulating DHCP renewal / network rebind)
    let _ = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.stop_host",
        "{}",
        Some(&token),
        None,
        now,
    );
    std::thread::sleep(Duration::from_millis(250));

    let start_payload2 = json!({ "port": port2, "bind_address": "127.0.0.1" }).to_string();
    let (start_res2_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.start_host",
        &start_payload2,
        Some(&token),
        None,
        now,
    );
    let start_res2: Value = serde_json::from_str(&start_res2_str).unwrap();
    assert!(
        start_res2["ok"].as_bool().unwrap(),
        "start_host 2 failed: {start_res2}"
    );
    std::thread::sleep(Duration::from_millis(50));

    // Mobile client fails to reach port1
    let fail_rpc = run_async({
        let cs = client_state.clone();
        async move { cs.execute_rpc("system.get_status", json!({})).await }
    });
    assert!(
        fail_rpc.is_err(),
        "Must fail to reach old port: {:?}",
        fail_rpc
    );

    // 3. User scans updated QR code from host
    let (open_res2_str, _) = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.open_mobile_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let open_res2: Value = serde_json::from_str(&open_res2_str).unwrap();
    let new_qr_uri = open_res2["data"]["qr_uri"].as_str().unwrap();

    // Mobile client updates connection coordinates from scanned QR
    let update_res = client_state.update_from_scanned_qr(new_qr_uri);
    assert!(
        update_res.is_ok(),
        "update_from_scanned_qr failed: {:?}",
        update_res
    );
    assert_eq!(client_state.host_port, port2);

    // 4. Mobile client immediately resumes RPC operations using existing device token
    let resumed_rpc = run_async({
        let cs = client_state.clone();
        async move { cs.execute_rpc("system.get_status", json!({})).await }
    });
    assert!(
        resumed_rpc.is_ok(),
        "Resumed RPC on port2 must succeed: {:?}",
        resumed_rpc
    );

    // Verify no duplicate device record was created
    let count_mobile_devices: i64 = conn
        .lock()
        .unwrap()
        .query_row("SELECT COUNT(*) FROM devices WHERE is_host = 0", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        count_mobile_devices, 1,
        "Must maintain single device record without duplicates"
    );

    cleanup_test_env(&conn, &token, now);
}
