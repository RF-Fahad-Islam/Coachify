use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::handle_rpc_with_events;
use coachify_core::lan::{
    block_workstation, clear_workstation_credentials, disconnect_workstation_client,
    get_workstation_client_status, is_workstation_client_mode, pair_with_host,
    reload_workstation_credentials, revoke_workstation, save_workstation_credentials,
    set_test_db_conn, test_workstation_connection, ClientPairPayload, WorkstationConnectionState,
    WorkstationCredentials,
};
use coachify_core::storage::{save_workstation_printer_settings, WorkstationPrinterSettings};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

static SERIAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_host_env(port: u16) -> (Arc<Mutex<Connection>>, String, String, String) {
    let now_ts = jiff::Timestamp::now().to_string();
    let now = now_ts.as_str();

    let mgr = coachify_core::lan::lan_manager();
    if mgr.is_running() {
        let dummy_conn = Connection::open_in_memory().unwrap();
        let _ = coachify_core::lan::stop_host(&dummy_conn, now);
        std::thread::sleep(Duration::from_millis(150));
    }
    set_test_db_conn(None);

    let mut raw_conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&raw_conn).expect("apply all migrations");

    raw_conn.execute(
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

    let shared = Arc::new(Mutex::new(raw_conn));
    set_test_db_conn(Some(shared.clone()));

    // Start host
    let start_payload = json!({
        "port": port,
        "bind_address": "127.0.0.1"
    })
    .to_string();

    let (start_res_str, _) = dispatch_at(
        &mut shared.lock().unwrap(),
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(
        start_res["ok"].as_bool().unwrap(),
        "start host failed: {start_res}"
    );
    let fingerprint = start_res["data"]["fingerprint"]
        .as_str()
        .unwrap()
        .to_string();

    // Open pairing window
    let (pair_win_str, _) = dispatch_at(
        &mut shared.lock().unwrap(),
        "lan.open_pairing_window",
        &json!({ "duration_seconds": 600 }).to_string(),
        Some(&token),
        None,
        now,
    );
    let pair_win_res: Value = serde_json::from_str(&pair_win_str).unwrap();
    assert!(
        pair_win_res["ok"].as_bool().unwrap(),
        "open pairing window failed: {pair_win_res}"
    );
    let code = pair_win_res["data"]["code"].as_str().unwrap().to_string();

    (shared, token, fingerprint, code)
}

fn cleanup_host_env(shared: &Arc<Mutex<Connection>>, token: &str) {
    let now = jiff::Timestamp::now().to_string();
    let _ = dispatch_at(
        &mut shared.lock().unwrap(),
        "lan.stop_host",
        "{}",
        Some(token),
        None,
        &now,
    );
    set_test_db_conn(None);
    let _ = disconnect_workstation_client();
    let _ = clear_workstation_credentials();
    let _ = reload_workstation_credentials();
    std::thread::sleep(Duration::from_millis(150));
}

struct TestCleanup {
    _temp_file: NamedTempFile,
}

impl TestCleanup {
    fn new() -> Self {
        let temp_file = NamedTempFile::new().expect("create temp file");
        std::env::set_var(
            "COACHIFY_TEST_WORKSTATION_CREDS_PATH",
            temp_file.path().to_str().unwrap(),
        );
        Self {
            _temp_file: temp_file,
        }
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        let _ = disconnect_workstation_client();
        let _ = clear_workstation_credentials();
        let _ = reload_workstation_credentials();
        set_test_db_conn(None);
        std::env::remove_var("COACHIFY_TEST_WORKSTATION_CREDS_PATH");
    }
}

#[test]
fn test_transparent_dual_mode_rpc_dispatch() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48101;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    // 1. Initially workstation is in standalone mode
    assert!(!is_workstation_client_mode());

    // 2. Pair as workstation client
    let pair_res = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("workstation pairing must succeed");
    assert!(!pair_res.device_token.is_empty());

    assert!(
        is_workstation_client_mode(),
        "must now be in workstation client mode"
    );

    // 3. Local command execution: printers.get_settings
    let (printer_res_str, _) = handle_rpc_with_events("printers.get_settings", "{}", None, None);
    let printer_res: Value = serde_json::from_str(&printer_res_str).unwrap();
    assert!(printer_res["ok"].as_bool().unwrap());
    assert!(printer_res["data"]["page_size"].is_string());

    // 4. Local command execution: lan.get_client_status
    let (status_res_str, _) = handle_rpc_with_events("lan.get_client_status", "{}", None, None);
    let status_res: Value = serde_json::from_str(&status_res_str).unwrap();
    assert!(status_res["ok"].as_bool().unwrap());
    assert_eq!(status_res["data"]["mode"], "workstation_client");
    assert_eq!(status_res["data"]["host_port"], port);

    // 5. Local command execution: system.ping
    let (ping_res_str, _) =
        handle_rpc_with_events("system.ping", "{\"message\":\"hello\"}", None, None);
    let ping_res: Value = serde_json::from_str(&ping_res_str).unwrap();
    assert!(ping_res["ok"].as_bool().unwrap());
    assert_eq!(ping_res["data"]["database_status"], "workstation_remote");
    assert_eq!(ping_res["data"]["echo"], "hello");

    // 6. Local command execution: system.get_client_info
    let (info_res_str, _) = handle_rpc_with_events("system.get_client_info", "{}", None, None);
    let info_res: Value = serde_json::from_str(&info_res_str).unwrap();
    assert!(info_res["ok"].as_bool().unwrap());
    assert_eq!(info_res["data"]["mode"], "workstation_client");

    // 7. Remote command forwarding: students.list forwards over TLS to host
    let (list_res_str, _) = handle_rpc_with_events(
        "students.list",
        "{\"limit\":10,\"offset\":0}",
        Some(&owner_token),
        None,
    );
    let list_res: Value = serde_json::from_str(&list_res_str).unwrap();
    assert!(
        list_res["ok"].as_bool().unwrap(),
        "remote students.list failed: {list_res}"
    );
    assert!(list_res["data"]["items"].is_array());

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_pinned_tls_transport_and_mitm_rejection() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48102;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    // 1. Legitimate pairing with correct pinned fingerprint
    let pair_res = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code: pairing_code.clone(),
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint.clone()),
    });
    assert!(
        pair_res.is_ok(),
        "connection with valid pinned fingerprint must succeed"
    );

    // 2. MITM / Rogue host attempt: Alter fingerprint in creds and attempt RPC
    let bad_creds = pair_res.unwrap();
    let fake_fingerprint = "SHA256:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF".to_string();
    let altered_creds = WorkstationCredentials {
        device_id: bad_creds.device_id,
        device_token: bad_creds.device_token,
        host_id: bad_creds.host_device_id,
        host_fingerprint: fake_fingerprint,
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        paired_at: jiff::Timestamp::now().to_string(),
        host_name: Some("Apex Coaching Center".to_string()),
        workstation_name: Some("Front Desk PC".to_string()),
    };
    save_workstation_credentials(&altered_creds).expect("save altered creds");
    reload_workstation_credentials();

    let (mitm_res_str, _) = handle_rpc_with_events("system.ping", "{}", None, None);
    let _mitm_res: Value = serde_json::from_str(&mitm_res_str).unwrap();
    // Local ping still works, but remote forwarded command should fail TLS handshake:
    let (remote_res_str, _) = handle_rpc_with_events("students.list", "{}", None, None);
    let remote_res: Value = serde_json::from_str(&remote_res_str).unwrap();
    assert!(!remote_res["ok"].as_bool().unwrap());
    assert_eq!(remote_res["error"]["code"], "FINGERPRINT_MISMATCH");

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_device_token_authentication_and_session_layering() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48103;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);
    let now = "2026-09-26T10:05:00Z";

    let pair_res = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");
    let dev_id = pair_res.device_id.clone();

    // 1. Valid request without staff session token returns UNAUTHENTICATED
    let (no_auth_str, _) = handle_rpc_with_events("students.list", "{}", None, None);
    let no_auth_res: Value = serde_json::from_str(&no_auth_str).unwrap();
    assert!(!no_auth_res["ok"].as_bool().unwrap());
    assert_eq!(
        no_auth_res["error"]["code"]
            .as_str()
            .unwrap()
            .to_lowercase(),
        "unauthenticated"
    );

    // 2. Valid request WITH staff session token succeeds and updates last_seen_at
    let (auth_str, _) = handle_rpc_with_events(
        "students.list",
        "{\"limit\":10,\"offset\":0}",
        Some(&owner_token),
        None,
    );
    let auth_res: Value = serde_json::from_str(&auth_str).unwrap();
    assert!(auth_res["ok"].as_bool().unwrap());

    // Verify last_seen_at was updated on host
    let last_seen: Option<String> = conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT last_seen_at FROM devices WHERE id = ?1",
            params![dev_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        last_seen.is_some(),
        "devices.last_seen_at must be populated on active RPC"
    );

    // 3. Block device on host
    let user_id: String = conn
        .lock()
        .unwrap()
        .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let _ = block_workstation(
        &conn.lock().unwrap(),
        &dev_id,
        Some("Audit"),
        Some(&user_id),
        now,
    );

    let (blocked_str, _) = handle_rpc_with_events(
        "students.list",
        "{\"limit\":10,\"offset\":0}",
        Some(&owner_token),
        None,
    );
    let blocked_res: Value = serde_json::from_str(&blocked_str).unwrap();
    assert!(!blocked_res["ok"].as_bool().unwrap());
    assert_eq!(blocked_res["error"]["code"], "DEVICE_BLOCKED");

    // 4. Revoke device on host
    let _ = revoke_workstation(
        &conn.lock().unwrap(),
        &dev_id,
        Some("Retired"),
        Some(&user_id),
        now,
    );

    let (revoked_str, _) = handle_rpc_with_events(
        "students.list",
        "{\"limit\":10,\"offset\":0}",
        Some(&owner_token),
        None,
    );
    let revoked_res: Value = serde_json::from_str(&revoked_str).unwrap();
    assert!(!revoked_res["ok"].as_bool().unwrap());
    assert_eq!(revoked_res["error"]["code"], "DEVICE_REVOKED");

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_strictly_local_printing_isolation() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48104;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);
    let now = "2026-09-26T10:05:00Z";

    let _ = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Cashier Desk 2".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    // 1. Save local printer settings on workstation
    let local_settings = WorkstationPrinterSettings {
        printer_name: Some("POS-80 Counter 2".to_string()),
        page_size: "thermal_80mm".to_string(),
        paper_source: None,
        copies: 1,
        auto_print_on_payment: true,
        open_cash_drawer: true,
        header_title: Some("Apex Coaching".to_string()),
        footer_note: Some("Thank you".to_string()),
        updated_at: now.to_string(),
    };
    let saved = save_workstation_printer_settings(&local_settings).expect("save local settings");
    assert_eq!(saved.printer_name.as_deref(), Some("POS-80 Counter 2"));

    // 2. Verify reading via handle_rpc_with_events executes locally
    let (get_res_str, _) = handle_rpc_with_events("printers.get_settings", "{}", None, None);
    let get_res: Value = serde_json::from_str(&get_res_str).unwrap();
    assert!(get_res["ok"].as_bool().unwrap());
    assert_eq!(get_res["data"]["printer_name"], "POS-80 Counter 2");

    // 3. Verify host database has no record of this workstation's printer settings
    let host_tables: Vec<String> = conn
        .lock()
        .unwrap()
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%printer%'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(
        host_tables.is_empty(),
        "host SQLite should have no workstation printer tables"
    );

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_realtime_multi_counter_sse_cache_sync() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48105;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    let _ = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    // Broadcast a change event on host and verify broadcast helper
    let test_events = vec![coachify_core::dispatcher::ChangeEvent {
        topic: "student:created".to_string(),
        entity_ids: vec!["stu_test_123".to_string()],
    }];

    coachify_core::lan::broadcast_change_events(test_events);

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_in_flight_mutation_idempotency_recovery() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48106;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    let _ = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    let idem_key = uuid::Uuid::now_v7().to_string();

    let student_payload = json!({
        "student": {
            "name": "Tariqul Islam",
            "gender": "male",
            "dob": "2010-01-01",
            "phone": "01711111111",
            "present_address": "Dhaka",
            "guardians": [
                {
                    "is_primary": true,
                    "receives_sms": true,
                    "relationship": "Father",
                    "new_guardian": {
                        "name": "Rafiqul Islam",
                        "relationship": "Father",
                        "phone": "01811111111"
                    }
                }
            ]
        }
    })
    .to_string();

    // First attempt with idempotency key
    let (res1_str, _) = handle_rpc_with_events(
        "students.create",
        &student_payload,
        Some(&owner_token),
        Some(&idem_key),
    );
    let res1: Value = serde_json::from_str(&res1_str).unwrap();
    assert!(
        res1["ok"].as_bool().unwrap(),
        "first attempt failed: {res1}"
    );
    let student_id = res1["data"]["id"].as_str().unwrap().to_string();

    // Second retry with the EXACT same idempotency key
    let (res2_str, _) = handle_rpc_with_events(
        "students.create",
        &student_payload,
        Some(&owner_token),
        Some(&idem_key),
    );
    let res2: Value = serde_json::from_str(&res2_str).unwrap();
    assert!(
        res2["ok"].as_bool().unwrap(),
        "idempotent retry failed: {res2}"
    );
    let retry_student_id = res2["data"]["id"].as_str().unwrap().to_string();

    // Must return the exact same student ID
    assert_eq!(
        student_id, retry_student_id,
        "idempotent replay must return identical record"
    );

    // Verify only ONE student was inserted
    let count: i64 = conn
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        count, 1,
        "exactly 1 student record must exist despite retry"
    );

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_dynamic_mdns_host_ip_shift_recovery() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48107;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    let _pair_res = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    // Workstation client status shows connected
    let status = get_workstation_client_status();
    assert_eq!(
        status.connection_state,
        WorkstationConnectionState::Connected
    );

    // Call reconnect
    let reconnect_res = coachify_core::lan::reconnect_workstation_client();
    assert!(
        reconnect_res.is_ok(),
        "reconnect must succeed against running host"
    );

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_instant_remote_revocation_lockout_latency() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48108;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);
    let now = "2026-09-26T10:05:00Z";

    let pair_res = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");
    let dev_id = pair_res.device_id.clone();

    // Revoke workstation on host
    let start = Instant::now();
    let user_id: String = conn
        .lock()
        .unwrap()
        .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let _ = revoke_workstation(
        &conn.lock().unwrap(),
        &dev_id,
        Some("Decommissioned"),
        Some(&user_id),
        now,
    );

    // Forward an RPC call - must return DEVICE_REVOKED and wipe local creds
    let (rev_str, _) = handle_rpc_with_events("students.list", "{}", Some(&owner_token), None);
    let elapsed = start.elapsed();
    let rev_res: Value = serde_json::from_str(&rev_str).unwrap();

    assert!(!rev_res["ok"].as_bool().unwrap());
    assert_eq!(rev_res["error"]["code"], "DEVICE_REVOKED");

    // Assert SLA: < 500ms
    assert!(
        elapsed < Duration::from_millis(500),
        "revocation response must complete in <500ms, took {:?}",
        elapsed
    );

    // Credentials file must be wiped and client state must be Revoked
    let status = get_workstation_client_status();
    assert_eq!(status.connection_state, WorkstationConnectionState::Revoked);

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_multi_counter_concurrency_stress() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48109;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    let _ = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Counter 1".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    // Perform 10 sequential mutations through client RPC
    for i in 1..=10 {
        let payload = json!({
            "student": {
                "name": format!("Student {:03}", i),
                "gender": "female",
                "dob": "2010-01-01",
                "phone": format!("01700000{:03}", i),
                "present_address": "Dhaka",
                "guardians": [
                    {
                        "is_primary": true,
                        "receives_sms": true,
                        "relationship": "Mother",
                        "new_guardian": {
                            "name": format!("Mother {:03}", i),
                            "relationship": "Mother",
                            "phone": format!("01800000{:03}", i)
                        }
                    }
                ]
            }
        })
        .to_string();

        let (res_str, _) =
            handle_rpc_with_events("students.create", &payload, Some(&owner_token), None);
        let res: Value = serde_json::from_str(&res_str).unwrap();
        assert!(
            res["ok"].as_bool().unwrap(),
            "create student {i} failed: {res}"
        );
    }

    let count: i64 = conn
        .lock()
        .unwrap()
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 10);

    // Cleanup host
    cleanup_host_env(&conn, &owner_token);
}

#[test]
fn test_workstation_health_and_latency_monitoring() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _cleanup = TestCleanup::new();
    let port = 48110;
    let (conn, owner_token, fingerprint, pairing_code) = setup_host_env(port);

    let _ = pair_with_host(ClientPairPayload {
        host_ip: "127.0.0.1".to_string(),
        host_port: port,
        pairing_code,
        workstation_name: "Front Desk PC".to_string(),
        expected_fingerprint: Some(fingerprint),
    })
    .expect("pair workstation");

    // 1. Host running -> ping /health returns success: true with latency
    let test_res = test_workstation_connection().expect("test connection");
    assert!(test_res.success);
    assert!(test_res.latency_ms.is_some());

    // 2. Stop host -> test_connection returns success: false
    let now = jiff::Timestamp::now().to_string();
    let _ = dispatch_at(
        &mut conn.lock().unwrap(),
        "lan.stop_host",
        "{}",
        Some(&owner_token),
        None,
        &now,
    );
    std::thread::sleep(Duration::from_millis(150));

    let test_res_stopped = test_workstation_connection().expect("test connection when stopped");
    assert!(!test_res_stopped.success);

    // Full cleanup
    cleanup_host_env(&conn, &owner_token);
}
