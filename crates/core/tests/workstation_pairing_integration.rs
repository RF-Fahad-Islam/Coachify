use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::lan::{
    block_workstation, broadcast_control_event, clear_workstation_credentials, control_sender,
    list_device_events, list_workstations, load_workstation_credentials, rename_workstation,
    revoke_workstation, save_workstation_credentials, unblock_workstation, DeviceControlEvent,
    WorkstationCredentials,
};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

static SERIAL_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_test_env() -> (Connection, String) {
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&conn).expect("apply all migrations");

    let now = "2026-09-26T10:00:00Z";
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

#[test]
fn test_pairing_window_creation_and_lockout_ac1() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap();
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-26T10:05:00Z";

    // 1. Host must be running to open pairing window
    let (res_fail_str, _) = dispatch_at(
        &mut conn,
        "lan.open_pairing_window",
        "{}",
        Some(&token),
        None,
        now,
    );
    let res_fail: Value = serde_json::from_str(&res_fail_str).unwrap();
    assert_eq!(
        res_fail["error"]["code"], "LAN_HOST_NOT_RUNNING",
        "should fail when host server is not started"
    );

    // 2. Start LAN host on a test port
    let start_payload = json!({
        "port": 50495,
        "bind_address": "127.0.0.1"
    })
    .to_string();
    let (start_res_str, _) = dispatch_at(
        &mut conn,
        "lan.start_host",
        &start_payload,
        Some(&token),
        None,
        now,
    );
    let start_res: Value = serde_json::from_str(&start_res_str).unwrap();
    assert!(start_res["ok"].as_bool().unwrap());

    // 3. Open pairing window
    let (pair_res_str, _) = dispatch_at(
        &mut conn,
        "lan.open_pairing_window",
        &json!({ "duration_seconds": 600 }).to_string(),
        Some(&token),
        None,
        now,
    );
    let pair_res: Value = serde_json::from_str(&pair_res_str).unwrap();
    assert!(pair_res["ok"].as_bool().unwrap(), "failed: {pair_res}");

    let code = pair_res["data"]["code"].as_str().unwrap();
    assert_eq!(code.len(), 6, "pairing code must be exactly 6 digits");
    assert!(
        code.chars().all(|c| c.is_ascii_digit()),
        "pairing code must be numeric digits"
    );
    assert_eq!(pair_res["data"]["expires_in_seconds"], 600);

    // 4. Verify lockout: simulate 5 incorrect attempts
    let session_id = pair_res["data"]["session_id"].as_str().unwrap();
    conn.execute(
        "UPDATE lan_pairing_sessions SET failed_attempts = 5, status = 'locked', expires_at = '2026-09-26T10:20:00Z' WHERE id = ?1",
        params![session_id],
    )
    .unwrap();

    // Trying to open a new pairing window while lockout cooldown is active should return LAN_PAIRING_COOLDOWN_ACTIVE
    let (cooldown_str, _) = dispatch_at(
        &mut conn,
        "lan.open_pairing_window",
        "{}",
        Some(&token),
        None,
        "2026-09-26T10:10:00Z",
    );
    let cooldown_res: Value = serde_json::from_str(&cooldown_str).unwrap();
    assert_eq!(
        cooldown_res["error"]["code"], "LAN_PAIRING_COOLDOWN_ACTIVE",
        "must be locked out during cooldown window"
    );

    // After cooldown expires, opening pairing window succeeds (keep test session active)
    conn.execute(
        "UPDATE sessions SET expires_at = '2026-09-26T10:40:00Z', last_seen_at = '2026-09-26T10:24:59Z'",
        [],
    )
    .unwrap();
    let (reopen_str, _) = dispatch_at(
        &mut conn,
        "lan.open_pairing_window",
        "{}",
        Some(&token),
        None,
        "2026-09-26T10:25:00Z",
    );
    let reopen_res: Value = serde_json::from_str(&reopen_str).unwrap();
    assert!(
        reopen_res["ok"].as_bool().unwrap(),
        "reopen failed: {reopen_res}"
    );

    // 5. Test cancel pairing window
    let (close_str, _) = dispatch_at(
        &mut conn,
        "lan.close_pairing_window",
        "{}",
        Some(&token),
        None,
        "2026-09-26T10:26:00Z",
    );
    let close_res: Value = serde_json::from_str(&close_str).unwrap();
    assert!(close_res["ok"].as_bool().unwrap());

    let status: String = conn
        .query_row(
            "SELECT status FROM lan_pairing_sessions ORDER BY created_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "cancelled");

    // Clean up host
    let _ = dispatch_at(&mut conn, "lan.stop_host", "{}", Some(&token), None, now);
}

#[test]
fn test_workstation_credentials_dpapi_storage_ac3() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap();

    let creds = WorkstationCredentials {
        device_id: "dev-test-123".to_string(),
        device_token: "dt_test_token_secret_1234567890abcdef".to_string(),
        host_id: "host-456".to_string(),
        host_fingerprint: "SHA256:11:22:33:44:55:66".to_string(),
        host_ip: "192.168.1.50".to_string(),
        host_port: 50443,
        paired_at: "2026-09-26T10:00:00Z".to_string(),
        host_name: Some("Apex Primary Server".to_string()),
        workstation_name: Some("Front Desk PC".to_string()),
    };

    // 1. Save credentials
    save_workstation_credentials(&creds).expect("save credentials");

    // 2. Load credentials
    let loaded = load_workstation_credentials()
        .expect("load credentials")
        .expect("credentials must exist");
    assert_eq!(loaded.device_id, creds.device_id);
    assert_eq!(loaded.device_token, creds.device_token);
    assert_eq!(loaded.host_fingerprint, creds.host_fingerprint);
    assert_eq!(loaded.host_ip, creds.host_ip);
    assert_eq!(loaded.host_port, creds.host_port);

    // 3. Clear credentials
    clear_workstation_credentials().expect("clear credentials");
    let after_clear = load_workstation_credentials().expect("load after clear");
    assert!(
        after_clear.is_none(),
        "credentials must be removed after clear"
    );
}

#[test]
fn test_workstation_license_quota_enforcement_ac4() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap();
    let (conn, _token) = setup_test_env();
    let now = "2026-09-26T10:00:00Z";

    // Set workstation limit = 2 in active license limits_json
    conn.execute(
        "UPDATE licenses SET limits_json = '{\"workstations\": 2}' WHERE is_active = 1",
        [],
    )
    .unwrap();

    // Insert 2 active workstations
    let dev1_id = uuid::Uuid::now_v7().to_string();
    let dev2_id = uuid::Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version)
         VALUES (?1, 'PC 1', 'WIN-PC1', 'hash1', 'active', 0, ?2, ?2, '0.1.0')",
        params![dev1_id, now],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version)
         VALUES (?1, 'PC 2', 'WIN-PC2', 'hash2', 'active', 0, ?2, ?2, '0.1.0')",
        params![dev2_id, now],
    )
    .unwrap();

    // Query list
    let list = list_workstations(&conn, now).unwrap();
    assert_eq!(list.quota.active_workstations, 2);
    assert_eq!(list.quota.workstation_limit, Some(2));
    assert_eq!(list.quota.seats_remaining, Some(0));

    // Insert a third device in blocked state
    let dev3_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version)
         VALUES (?1, 'PC 3', 'WIN-PC3', 'hash3', 'blocked', 0, ?2, ?2, '0.1.0')",
        params![dev3_id, now],
    )
    .unwrap();

    let user_id: String = conn
        .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    // Trying to unblock dev3 when seats are full (2/2) must fail with WorkstationLimitReached
    let unblock_err = unblock_workstation(&conn, &dev3_id, Some(&user_id), now).unwrap_err();
    assert_eq!(unblock_err.code(), "LAN_WORKSTATION_LIMIT_REACHED");

    // Block dev1 to free up a seat
    block_workstation(&conn, &dev1_id, Some("freeing seat"), Some(&user_id), now).unwrap();

    // Now unblocking dev3 succeeds because 1/2 seats are used
    let unblocked = unblock_workstation(&conn, &dev3_id, Some(&user_id), now).unwrap();
    assert_eq!(unblocked.status, "active");
}

#[test]
fn test_workstation_crud_lifecycle_ac5_ac6_ac7_ac8_ac10() {
    let _guard = SERIAL_TEST_MUTEX.lock().unwrap();
    let (conn, _token) = setup_test_env();
    let now = "2026-09-26T10:00:00Z";

    let user_id: String = conn
        .query_row("SELECT id FROM users LIMIT 1", [], |r| r.get(0))
        .unwrap();

    let dev_id = uuid::Uuid::now_v7().to_string();
    let mut hasher = Sha256::new();
    hasher.update(b"dt_test_bearer_token");
    let token_hash = hex::encode(hasher.finalize());

    // 1. Register workstation (AC-5)
    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, ip_address, app_version, paired_at, last_seen_at)
         VALUES (?1, 'Reception PC', 'RECEPTION-01', ?2, 'active', 0, '192.168.1.150', '0.1.0', ?3, ?3)",
        params![dev_id, token_hash, now],
    )
    .unwrap();

    // Record pairing event (AC-10)
    let ev_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
         VALUES (?1, ?2, 'paired', ?3, '192.168.1.150', '{}', ?4)",
        params![ev_id, dev_id, user_id, now],
    )
    .unwrap();

    // 2. Rename Workstation (AC-6)
    let renamed = rename_workstation(
        &conn,
        &dev_id,
        "Front Desk Counter 1",
        Some(&user_id),
        "2026-09-26T10:05:00Z",
    )
    .expect("rename workstation");
    assert_eq!(renamed.name, "Front Desk Counter 1");

    // Host cannot be renamed
    let host_id: String = conn
        .query_row("SELECT id FROM devices WHERE is_host = 1", [], |r| r.get(0))
        .unwrap();
    let host_rename_err = rename_workstation(&conn, &host_id, "New Host", None, now).unwrap_err();
    assert_eq!(host_rename_err.code(), "CANNOT_RENAME_HOST");

    // 3. Block Workstation and verify session termination (AC-7)
    // Create a mock active session on this device
    let sess_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO sessions (id, user_id, device_id, token_hash, created_at, last_seen_at, expires_at, absolute_expires_at)
         VALUES (?1, (SELECT id FROM users LIMIT 1), ?2, 'sess_hash', ?3, ?3, '2026-09-27T10:00:00Z', '2026-09-27T10:00:00Z')",
        params![sess_id, dev_id, now],
    )
    .unwrap();

    let blocked = block_workstation(
        &conn,
        &dev_id,
        Some("Security audit check"),
        Some(&user_id),
        "2026-09-26T10:10:00Z",
    )
    .expect("block workstation");
    assert_eq!(blocked.status, "blocked");
    assert!(blocked.blocked_at.is_some());

    // Verify session revoked
    let revoked_at: Option<String> = conn
        .query_row(
            "SELECT revoked_at FROM sessions WHERE id = ?1",
            params![sess_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        revoked_at.is_some(),
        "session on blocked device must be immediately purged"
    );

    // Cannot block host
    let host_block_err = block_workstation(&conn, &host_id, None, None, now).unwrap_err();
    assert_eq!(host_block_err.code(), "CANNOT_BLOCK_HOST");

    // 4. Unblock Workstation (AC-8)
    let unblocked = unblock_workstation(&conn, &dev_id, Some(&user_id), "2026-09-26T10:15:00Z")
        .expect("unblock workstation");
    assert_eq!(unblocked.status, "active");
    assert!(unblocked.blocked_at.is_none());

    // 5. Revoke Workstation (AC-8)
    let revoked = revoke_workstation(
        &conn,
        &dev_id,
        Some("Decommissioned hardware"),
        Some(&user_id),
        "2026-09-26T10:20:00Z",
    )
    .expect("revoke workstation");
    assert_eq!(revoked.status, "revoked");
    assert!(revoked.revoked_at.is_some());
    assert_eq!(
        revoked.revocation_reason.as_deref(),
        Some("Decommissioned hardware")
    );

    // Verify token hash is tombstoned
    let current_token_hash: String = conn
        .query_row(
            "SELECT token_hash FROM devices WHERE id = ?1",
            params![dev_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        current_token_hash.starts_with("revoked:"),
        "token hash must be tombstoned"
    );

    // Revoked device cannot be blocked, renamed, or unblocked
    let re_rename = rename_workstation(&conn, &dev_id, "Attempt", None, now).unwrap_err();
    assert_eq!(re_rename.code(), "DEVICE_ALREADY_REVOKED");

    let re_block = block_workstation(&conn, &dev_id, None, None, now).unwrap_err();
    assert_eq!(re_block.code(), "DEVICE_ALREADY_REVOKED");

    let re_revoke = revoke_workstation(&conn, &dev_id, None, None, now).unwrap_err();
    assert_eq!(re_revoke.code(), "DEVICE_ALREADY_REVOKED");

    // 6. Check Audit Trail in device_events (AC-10)
    let events = list_device_events(&conn, Some(&dev_id), None).expect("list device events");
    assert_eq!(events.len(), 5); // paired, renamed, blocked, unblocked, revoked

    let event_types: Vec<&str> = events.iter().map(|e| e.event_type.as_str()).collect();
    assert_eq!(
        event_types,
        vec!["revoked", "unblocked", "blocked", "renamed", "paired"]
    );

    // 7. Test Immutability triggers on device_events (AC-10)
    let first_ev_id = &events[0].id;
    let update_res = conn.execute(
        "UPDATE device_events SET event_type = 'tampered' WHERE id = ?1",
        params![first_ev_id],
    );
    assert!(
        update_res.is_err(),
        "device_events must be append-only: UPDATE must be rejected by trigger"
    );

    let delete_res = conn.execute(
        "DELETE FROM device_events WHERE id = ?1",
        params![first_ev_id],
    );
    assert!(
        delete_res.is_err(),
        "device_events must be append-only: DELETE must be rejected by trigger"
    );
}

#[tokio::test]
async fn test_realtime_control_broadcast_ac9() {
    let mut rx = control_sender().subscribe();

    // Broadcast a revocation event
    broadcast_control_event(DeviceControlEvent::Revoked {
        device_id: "dev-999".to_string(),
        reason: "Admin revoked".to_string(),
    });

    let received = rx.recv().await.expect("receive broadcast event");
    match received {
        DeviceControlEvent::Revoked { device_id, reason } => {
            assert_eq!(device_id, "dev-999");
            assert_eq!(reason, "Admin revoked");
        }
        _ => panic!("unexpected event variant"),
    }

    // Broadcast a blocked event
    broadcast_control_event(DeviceControlEvent::Blocked {
        device_id: "dev-888".to_string(),
        reason: "Suspicious activity".to_string(),
    });

    let received_blocked = rx.recv().await.expect("receive blocked broadcast");
    match received_blocked {
        DeviceControlEvent::Blocked { device_id, reason } => {
            assert_eq!(device_id, "dev-888");
            assert_eq!(reason, "Suspicious activity");
        }
        _ => panic!("unexpected event variant"),
    }
}
