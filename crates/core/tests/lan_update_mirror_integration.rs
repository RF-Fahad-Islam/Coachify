use axum::body::Body;
use axum::http::{Request, StatusCode};
use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::lan::client::{InMemoryClientTransport, TransportClient};
use coachify_core::lan::mirror::{
    compute_sha256, get_active_manifest, get_staged_file_path, sign_update_checksum_ed25519,
    stage_update,
};
use coachify_core::lan::protocol::{
    HostUpdateManifest, LanError, StageMirrorUpdateInput, WorkstationCredentials,
};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use tower::util::ServiceExt;

static SERIAL_TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn setup_test_db() -> (Connection, String) {
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&conn).expect("apply all migrations");

    let now = "2026-09-28T12:00:00Z";
    conn.execute(
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
        &mut conn,
        "system.bootstrap_owner",
        &boot_payload,
        None,
        None,
        now,
    );
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(res["ok"].as_bool().unwrap(), "bootstrap failed: {res}");
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    (conn, token)
}

fn register_test_device(conn: &Connection, device_id: &str, token: &str, status: &str, now: &str) {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    conn.execute(
        "INSERT INTO devices (
            id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version
         ) VALUES (?1, 'Counter 1', 'counter1-pc', ?2, ?3, 0, ?4, ?4, '1.1.0')",
        params![device_id, token_hash, status, now],
    )
    .unwrap();
}

fn create_test_router(conn: Arc<Mutex<Connection>>) -> axum::Router {
    let (event_tx, _) = tokio::sync::broadcast::channel(16);
    let state = coachify_core::lan::host::HostServerState {
        host_id: "host_apex".to_string(),
        institution_name: "Apex Coaching".to_string(),
        app_version: "1.1.0".to_string(),
        fingerprint: "SHA256:00".to_string(),
        db_path: None,
        db_conn: Some(conn),
        started_at: std::time::Instant::now(),
        port: 50443,
        rate_limiter: coachify_core::lan::RateLimiter::new(),
        control_sender: event_tx,
    };
    coachify_core::lan::HostController::build_router_state(state)
}

#[test]
fn test_mirror_cache_staging_and_db_tracking() {
    let _lock = SERIAL_TEST_MUTEX.blocking_lock();
    let (mut conn, token) = setup_test_db();
    let temp_dir = tempfile::tempdir().unwrap();
    let now = "2026-09-28T12:00:00Z";

    // Create simulated v1.1.0 update binary
    let file_v110 = temp_dir.path().join("Coachify-Setup-1.1.0.exe");
    let payload_v110 = b"COACHIFY-V1.1.0-BINARY-PAYLOAD-SIMULATED-FOR-TESTING";
    std::fs::write(&file_v110, payload_v110).unwrap();
    let hash_v110 = compute_sha256(payload_v110);
    let sig_v110 = sign_update_checksum_ed25519(&hash_v110, "u2026a").unwrap();

    let stage_payload = json!({
        "file_path": file_v110.to_string_lossy().to_string(),
        "version": "1.1.0",
        "channel": "stable",
        "notes": "LAN update mirror engine enhancements",
        "expected_sha256": hash_v110,
        "updater_signature": sig_v110,
        "updater_key_id": "u2026a"
    })
    .to_string();

    let (res_str, events) = dispatch_at(
        &mut conn,
        "lan.stage_mirror_update",
        &stage_payload,
        Some(&token),
        None,
        now,
    );
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(res["ok"].as_bool().unwrap(), "staging failed: {res}");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "lan:changed");

    // Verify database row
    let active_manifest = get_active_manifest(&conn, now)
        .unwrap()
        .expect("active manifest");
    assert_eq!(active_manifest.version, "1.1.0");
    assert_eq!(active_manifest.sha256, hash_v110);
    assert_eq!(active_manifest.file_size_bytes, payload_v110.len() as i64);

    // Stage second version v1.2.0
    let file_v120 = temp_dir.path().join("Coachify-Setup-1.2.0.exe");
    let payload_v120 = b"COACHIFY-V1.2.0-NEXT-GEN-UPDATE-BINARY";
    std::fs::write(&file_v120, payload_v120).unwrap();
    let hash_v120 = compute_sha256(payload_v120);
    let sig_v120 = sign_update_checksum_ed25519(&hash_v120, "u2026a").unwrap();

    let stage_v120_payload = json!({
        "file_path": file_v120.to_string_lossy().to_string(),
        "version": "1.2.0",
        "channel": "stable",
        "notes": "Version 1.2.0 active release",
        "expected_sha256": hash_v120,
        "updater_signature": sig_v120,
        "updater_key_id": "u2026a"
    })
    .to_string();

    let (res_str_2, _) = dispatch_at(
        &mut conn,
        "lan.stage_mirror_update",
        &stage_v120_payload,
        Some(&token),
        None,
        now,
    );
    let res_2: Value = serde_json::from_str(&res_str_2).unwrap();
    assert!(res_2["ok"].as_bool().unwrap());

    // v1.2.0 is now active, v1.1.0 is inactive
    let active_2 = get_active_manifest(&conn, now)
        .unwrap()
        .expect("active manifest 2");
    assert_eq!(active_2.version, "1.2.0");

    let is_v110_active: i64 = conn
        .query_row(
            "SELECT is_active FROM lan_staged_updates WHERE version = '1.1.0'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(is_v110_active, 0);

    let is_v120_active: i64 = conn
        .query_row(
            "SELECT is_active FROM lan_staged_updates WHERE version = '1.2.0'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(is_v120_active, 1);
}

#[tokio::test]
async fn test_host_update_endpoints_and_auth() {
    let _lock = SERIAL_TEST_MUTEX.lock().await;
    let (conn, _token) = setup_test_db();
    let now = "2026-09-28T12:00:00Z";
    let temp_dir = tempfile::tempdir().unwrap();

    // Register active, blocked, revoked devices
    register_test_device(&conn, "dev_active", "tok_active_123", "active", now);
    register_test_device(&conn, "dev_blocked", "tok_blocked_456", "blocked", now);
    register_test_device(&conn, "dev_revoked", "tok_revoked_789", "revoked", now);

    // Stage update
    let installer_file = temp_dir.path().join("Coachify-Setup-1.1.0.exe");
    let payload = b"HOST-ENDPOINTS-TEST-BINARY-CONTENT";
    std::fs::write(&installer_file, payload).unwrap();
    let hash = compute_sha256(payload);
    let sig = sign_update_checksum_ed25519(&hash, "u2026a").unwrap();

    let input = StageMirrorUpdateInput {
        file_path: installer_file.to_string_lossy().to_string(),
        version: "1.1.0".to_string(),
        channel: Some("stable".to_string()),
        notes: Some("Endpoint test release".to_string()),
        expected_sha256: Some(hash.clone()),
        updater_signature: Some(sig),
        updater_key_id: Some("u2026a".to_string()),
    };
    stage_update(&conn, &input, now, None).unwrap();

    let shared_db = Arc::new(Mutex::new(conn));
    let router = create_test_router(shared_db.clone());

    // 1. Missing token -> 401 Unauthorized
    let req_no_auth = Request::builder()
        .method("GET")
        .uri("/updates/manifest")
        .body(Body::empty())
        .unwrap();
    let res = router.clone().oneshot(req_no_auth).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Blocked device token -> 403 Forbidden
    let req_blocked = Request::builder()
        .method("GET")
        .uri("/updates/manifest")
        .header("X-Coachify-Device-Token", "tok_blocked_456")
        .body(Body::empty())
        .unwrap();
    let res_blocked = router.clone().oneshot(req_blocked).await.unwrap();
    assert_eq!(res_blocked.status(), StatusCode::FORBIDDEN);

    // 3. Revoked device token -> 403 Forbidden
    let req_revoked = Request::builder()
        .method("GET")
        .uri("/updates/manifest")
        .header("X-Coachify-Device-Token", "tok_revoked_789")
        .body(Body::empty())
        .unwrap();
    let res_revoked = router.clone().oneshot(req_revoked).await.unwrap();
    assert_eq!(res_revoked.status(), StatusCode::FORBIDDEN);

    // 4. Valid active token -> 200 OK with manifest JSON
    let req_active = Request::builder()
        .method("GET")
        .uri("/updates/manifest")
        .header("X-Coachify-Device-Token", "tok_active_123")
        .body(Body::empty())
        .unwrap();
    let res_active = router.clone().oneshot(req_active).await.unwrap();
    assert_eq!(res_active.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(res_active.into_body(), usize::MAX)
        .await
        .unwrap();
    let manifest: HostUpdateManifest = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(manifest.version, "1.1.0");
    assert_eq!(manifest.sha256, hash);

    // 5. Download binary stream -> 200 OK with exact bytes
    let req_download = Request::builder()
        .method("GET")
        .uri("/updates/download/1.1.0")
        .header("X-Coachify-Device-Token", "tok_active_123")
        .body(Body::empty())
        .unwrap();
    let res_download = router.clone().oneshot(req_download).await.unwrap();
    assert_eq!(res_download.status(), StatusCode::OK);
    let streamed_bytes = axum::body::to_bytes(res_download.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(streamed_bytes.as_ref(), payload);
}

#[test]
fn test_download_cryptographic_verification_and_tamper_rejection() {
    let _lock = SERIAL_TEST_MUTEX.blocking_lock();
    let (conn, _token) = setup_test_db();
    let now = "2026-09-28T12:00:00Z";
    let temp_dir = tempfile::tempdir().unwrap();

    register_test_device(&conn, "dev_counter", "tok_counter_123", "active", now);

    let installer_file = temp_dir.path().join("Coachify-Setup-1.1.0.exe");
    let payload = b"AUTHENTIC-UNTOUCHED-COACHIFY-RELEASE-PACKAGE";
    std::fs::write(&installer_file, payload).unwrap();
    let hash = compute_sha256(payload);
    let valid_sig = sign_update_checksum_ed25519(&hash, "u2026a").unwrap();

    let input = StageMirrorUpdateInput {
        file_path: installer_file.to_string_lossy().to_string(),
        version: "1.1.0".to_string(),
        channel: Some("stable".to_string()),
        notes: Some("Tamper rejection test release".to_string()),
        expected_sha256: Some(hash.clone()),
        updater_signature: Some(valid_sig.clone()),
        updater_key_id: Some("u2026a".to_string()),
    };
    stage_update(&conn, &input, now, None).unwrap();

    let shared_db = Arc::new(Mutex::new(conn));
    let router = create_test_router(shared_db.clone());

    let creds = WorkstationCredentials {
        device_id: "dev_counter".to_string(),
        device_token: "tok_counter_123".to_string(),
        host_id: "host_apex".to_string(),
        host_fingerprint: "SHA256:00".to_string(),
        host_ip: "127.0.0.1".to_string(),
        host_port: 50443,
        paired_at: now.to_string(),
        host_name: Some("Apex Host".to_string()),
        workstation_name: Some("Counter 1".to_string()),
    };

    let client = InMemoryClientTransport::with_credentials(router.clone(), creds);

    // Drill A: Valid download succeeds and verifies
    let target_dest = temp_dir.path().join("workstation_staged.exe");
    let downloaded_path = client
        .download_host_update_sync(Some("1.1.0"), &target_dest)
        .expect("authentic download should succeed");
    assert!(downloaded_path.exists());
    assert_eq!(std::fs::read(&downloaded_path).unwrap(), payload);

    // Drill B: Corrupted/tampered bits in physical storage -> Checksum Mismatch
    let staged_path = get_staged_file_path(&shared_db.lock().unwrap(), "1.1.0", now, None).unwrap();
    let mut tampered = payload.to_vec();
    tampered[0] ^= 0xFF; // Invert first byte
    std::fs::write(&staged_path, &tampered).unwrap();

    let tampered_dest = temp_dir.path().join("workstation_tampered.exe");
    let tamper_res = client.download_host_update_sync(Some("1.1.0"), &tampered_dest);
    match tamper_res {
        Err(LanError::UpdateChecksumMismatch { expected, computed }) => {
            assert_eq!(expected, hash);
            assert_ne!(computed, hash);
        }
        other => panic!("Expected UpdateChecksumMismatch, got {:?}", other),
    }
    assert!(
        !tampered_dest.exists(),
        "Tampered destination file must be deleted upon checksum failure"
    );

    // Restore authentic file, but inject invalid Ed25519 signature in database
    std::fs::write(&staged_path, payload).unwrap();
    let bogus_sig = hex::encode([0xAAu8; 64]);
    shared_db
        .lock()
        .unwrap()
        .execute(
            "UPDATE lan_staged_updates SET updater_signature = ?1 WHERE version = '1.1.0'",
            params![bogus_sig],
        )
        .unwrap();

    let invalid_sig_dest = temp_dir.path().join("workstation_bad_sig.exe");
    let sig_res = client.download_host_update_sync(Some("1.1.0"), &invalid_sig_dest);
    match sig_res {
        Err(LanError::UpdateSignatureInvalid(_)) => {}
        other => panic!("Expected UpdateSignatureInvalid, got {:?}", other),
    }
    assert!(
        !invalid_sig_dest.exists(),
        "Unverified binary must be deleted upon invalid signature"
    );
}

#[tokio::test]
async fn test_concurrent_workstation_downloads() {
    let _lock = SERIAL_TEST_MUTEX.lock().await;
    let (conn, _token) = setup_test_db();
    let now = "2026-09-28T12:00:00Z";
    let temp_dir = tempfile::tempdir().unwrap();

    register_test_device(&conn, "dev_ws1", "tok_ws1", "active", now);
    register_test_device(&conn, "dev_ws2", "tok_ws2", "active", now);
    register_test_device(&conn, "dev_ws3", "tok_ws3", "active", now);

    let installer_file = temp_dir.path().join("Coachify-Setup-1.1.0.exe");
    let payload = b"CONCURRENT-STREAMING-DOWNLOAD-TEST-CONTENT-0123456789";
    std::fs::write(&installer_file, payload).unwrap();
    let hash = compute_sha256(payload);
    let valid_sig = sign_update_checksum_ed25519(&hash, "u2026a").unwrap();

    let input = StageMirrorUpdateInput {
        file_path: installer_file.to_string_lossy().to_string(),
        version: "1.1.0".to_string(),
        channel: Some("stable".to_string()),
        notes: Some("Concurrent download release".to_string()),
        expected_sha256: Some(hash.clone()),
        updater_signature: Some(valid_sig),
        updater_key_id: Some("u2026a".to_string()),
    };
    stage_update(&conn, &input, now, None).unwrap();

    let shared_db = Arc::new(Mutex::new(conn));
    let router = create_test_router(shared_db.clone());

    // Launch 3 concurrent downloads
    let mut handles = Vec::new();
    for i in 1..=3 {
        let r = router.clone();
        let tok = format!("tok_ws{}", i);
        let h = tokio::spawn(async move {
            let req = Request::builder()
                .method("GET")
                .uri("/updates/download/1.1.0")
                .header("X-Coachify-Device-Token", tok)
                .body(Body::empty())
                .unwrap();
            let res = r.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            bytes
        });
        handles.push(h);
    }

    for h in handles {
        let downloaded = h.await.unwrap();
        assert_eq!(downloaded.as_ref(), payload);
    }

    // Verify downloads_count on database record
    let downloads_count: i64 = shared_db
        .lock()
        .unwrap()
        .query_row(
            "SELECT downloads_count FROM lan_staged_updates WHERE version = '1.1.0'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(downloads_count, 3);
}

#[test]
fn test_update_mirror_audit_trail_logging() {
    let _lock = SERIAL_TEST_MUTEX.blocking_lock();
    let (mut conn, token) = setup_test_db();
    let now = "2026-09-28T12:00:00Z";
    let temp_dir = tempfile::tempdir().unwrap();

    let installer_file = temp_dir.path().join("Coachify-Setup-1.1.0.exe");
    let payload = b"AUDIT-TRAIL-TEST-PAYLOAD";
    std::fs::write(&installer_file, payload).unwrap();
    let hash = compute_sha256(payload);

    let stage_payload = json!({
        "file_path": installer_file.to_string_lossy().to_string(),
        "version": "1.1.0",
        "channel": "stable",
        "expected_sha256": hash,
    })
    .to_string();

    // 1. Stage update through dispatcher
    let (stage_res, _) = dispatch_at(
        &mut conn,
        "lan.stage_mirror_update",
        &stage_payload,
        Some(&token),
        None,
        now,
    );
    let stage_v: Value = serde_json::from_str(&stage_res).unwrap();
    assert!(stage_v["ok"].as_bool().unwrap());

    // Check audit_log table for lan_update_mirror.staged
    let staged_audit_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'lan_update_mirror.staged' AND entity_id = '1.1.0'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(staged_audit_count, 1);

    // 2. Clear staged update through dispatcher
    let clear_payload = json!({ "version": "1.1.0" }).to_string();
    let (clear_res, _) = dispatch_at(
        &mut conn,
        "lan.clear_mirror_update",
        &clear_payload,
        Some(&token),
        None,
        now,
    );
    let clear_v: Value = serde_json::from_str(&clear_res).unwrap();
    assert!(clear_v["ok"].as_bool().unwrap());

    // Check audit_log table for lan_update_mirror.cleared
    let cleared_audit_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'lan_update_mirror.cleared' AND entity_id = '1.1.0'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cleared_audit_count, 1);
}
