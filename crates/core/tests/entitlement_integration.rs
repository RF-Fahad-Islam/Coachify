use coachify_core::db;
use coachify_core::dispatcher::{dispatch_at, find_command_meta, LicenseGate};
use coachify_core::entitlement::{
    authorize_command, get_status, get_workstation_quota, guard_workstation_seat, is_read_only,
    EntitlementError,
};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

fn setup_test_db() -> (Connection, String) {
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
fn test_entitlement_seam_initial_trial_and_quota() {
    let (conn, _token) = setup_test_db();
    let now = "2026-09-26T10:00:00Z";

    // 1. Initial trial license status through Entitlement seam
    let status = get_status(&conn, now).expect("get_status succeeds");
    assert_eq!(status.tier, "trial");
    assert_eq!(status.workstation_limit, Some(1));
    assert!(!status.is_read_only);
    assert!(!is_read_only(&conn, now));

    // 2. Initial workstation quota
    let quota = get_workstation_quota(&conn, now).expect("quota lookup succeeds");
    assert_eq!(quota.active_workstations, 0);
    assert_eq!(quota.workstation_limit, Some(1));
    assert_eq!(quota.seats_remaining, Some(1));
    assert_eq!(quota.license_tier, "trial");

    // Guard seat should pass
    guard_workstation_seat(&conn, now).expect("seat guard passes when seats available");

    // 3. Command authorization for write commands
    let write_meta = find_command_meta("students.create").expect("students.create exists");
    assert_eq!(write_meta.gate, LicenseGate::Write);
    authorize_command(&conn, write_meta, now).expect("write authorized during active trial");

    // 4. Command authorization for LAN commands
    let lan_meta = find_command_meta("lan.start_host").expect("lan.start_host exists");
    assert_eq!(lan_meta.gate, LicenseGate::Lan);
    authorize_command(&conn, lan_meta, now).expect("lan authorized during active trial");
}

#[test]
fn test_entitlement_workstation_seat_quota_enforcement() {
    let (conn, _token) = setup_test_db();
    let now = "2026-09-26T10:00:00Z";

    // Insert 1 active secondary workstation
    let dev1 = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version)
         VALUES (?1, 'Workstation 1', 'WIN-WS1', 'hash1', 'active', 0, ?2, ?2, '1.0.0')",
        params![dev1, now],
    )
    .unwrap();

    // Now active_workstations = 1, limit = 1 -> seats_remaining = 0
    let quota = get_workstation_quota(&conn, now).expect("quota lookup succeeds");
    assert_eq!(quota.active_workstations, 1);
    assert_eq!(quota.workstation_limit, Some(1));
    assert_eq!(quota.seats_remaining, Some(0));

    // Guarding next seat must fail with WorkstationLimitReached
    let err =
        guard_workstation_seat(&conn, now).expect_err("seat guard must fail when limit reached");
    assert_eq!(err, EntitlementError::WorkstationLimitReached);
    assert_eq!(err.code(), "LAN_WORKSTATION_LIMIT_REACHED");

    // Expand limit to 2
    conn.execute(
        "UPDATE licenses SET limits_json = '{\"workstations\": 2}' WHERE is_active = 1",
        [],
    )
    .unwrap();

    let quota2 = get_workstation_quota(&conn, now).expect("quota lookup succeeds");
    assert_eq!(quota2.workstation_limit, Some(2));
    assert_eq!(quota2.seats_remaining, Some(1));
    guard_workstation_seat(&conn, now).expect("seat guard passes after quota expansion");
}

#[test]
fn test_entitlement_tier_feature_gating() {
    let (conn, _token) = setup_test_db();
    let now = "2026-09-26T10:00:00Z";

    let lan_meta = find_command_meta("lan.start_host").expect("lan.start_host exists");

    // 1. Switch active license to 'local' (single PC tier without LAN capability)
    conn.execute(
        "UPDATE licenses SET tier = 'local', capabilities_json = '[\"core\"]' WHERE is_active = 1",
        [],
    )
    .unwrap();

    let err = authorize_command(&conn, lan_meta, now)
        .expect_err("LAN command must fail on single-PC tier");
    assert_eq!(err.code(), "LICENSE_FEATURE_NOT_ENTITLED");

    // 2. Upgrade active license to 'local_pro'
    conn.execute(
        "UPDATE licenses SET tier = 'local_pro', capabilities_json = '[\"core\", \"lan_multi_device\"]' WHERE is_active = 1",
        [],
    )
    .unwrap();

    authorize_command(&conn, lan_meta, now).expect("LAN command must succeed on local_pro tier");
}

#[test]
fn test_entitlement_read_only_grace_period_expiration() {
    let (mut conn, token) = setup_test_db();
    let now = "2026-09-26T10:00:00Z";

    // Set license valid_until in the past
    conn.execute(
        "UPDATE licenses SET paid_through = '2026-08-01', valid_until = '2026-08-15' WHERE is_active = 1",
        [],
    )
    .unwrap();

    assert!(is_read_only(&conn, now));

    let write_meta = find_command_meta("students.create").expect("students.create exists");
    let err = authorize_command(&conn, write_meta, now)
        .expect_err("Write must fail when expired past grace");
    assert_eq!(err, EntitlementError::ExpiredReadOnly);
    assert_eq!(err.code(), "license_expired_read_only");

    // Verify end-to-end dispatcher RPC rejection
    let student_payload = json!({
        "full_name": "Test Student",
        "phone": "01711111111",
        "date_of_birth": "2010-01-01",
        "gender": "male"
    })
    .to_string();

    let (res_str, _) = dispatch_at(
        &mut conn,
        "students.create",
        &student_payload,
        Some(&token),
        None,
        now,
    );
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(!res["ok"].as_bool().unwrap());
    assert_eq!(
        res["error"]["code"].as_str().unwrap(),
        "license_expired_read_only"
    );

    // Read commands must still succeed
    let (read_res_str, _) = dispatch_at(&mut conn, "students.list", "{}", Some(&token), None, now);
    let read_res: Value = serde_json::from_str(&read_res_str).unwrap();
    assert!(
        read_res["ok"].as_bool().unwrap(),
        "Read commands remain functional in read-only mode"
    );
}

#[test]
fn test_entitlement_device_deactivation() {
    let (mut conn, token) = setup_test_db();
    let now = "2026-09-26T10:00:00Z";

    // Simulate device deactivation in license_refresh_state
    conn.execute(
        "UPDATE license_refresh_state SET deactivated_at = ?1, deactivation_reason = 'Revoked from portal' WHERE id = 1",
        params![now],
    )
    .unwrap();

    let write_meta = find_command_meta("students.create").expect("students.create exists");
    let err = authorize_command(&conn, write_meta, now)
        .expect_err("Write must fail when device deactivated");
    assert_eq!(err, EntitlementError::DeviceDeactivated);
    assert_eq!(err.code(), "LICENSE_DEVICE_DEACTIVATED");

    // Verify end-to-end dispatcher RPC rejection
    let (res_str, _) = dispatch_at(&mut conn, "students.create", "{}", Some(&token), None, now);
    let res: Value = serde_json::from_str(&res_str).unwrap();
    assert!(!res["ok"].as_bool().unwrap());
    assert_eq!(
        res["error"]["code"].as_str().unwrap(),
        "LICENSE_DEVICE_DEACTIVATED"
    );
}

#[test]
fn test_entitlement_clock_tampering_guard() {
    let (mut conn, token) = setup_test_db();

    // Establish clock watermark at 2026-09-26T10:00:00Z
    let (res1, _) = dispatch_at(
        &mut conn,
        "system.get_system_status",
        "{}",
        Some(&token),
        None,
        "2026-09-26T10:00:00Z",
    );
    let env1: Value = serde_json::from_str(&res1).unwrap();
    assert!(env1["ok"].as_bool().unwrap());

    // Jump time backwards to 2026-09-20T10:00:00Z
    let write_meta = find_command_meta("students.create").expect("students.create exists");
    let err = authorize_command(&conn, write_meta, "2026-09-20T10:00:00Z")
        .expect_err("Clock rollback must fail authorization");
    assert_eq!(err.code(), "clock_out_of_sync");

    // Adjust session timestamps backwards so session verification passes, but clock watermark fails
    conn.execute(
        "UPDATE sessions SET created_at = '2026-09-19T10:00:00Z', last_seen_at = '2026-09-19T10:00:00Z'",
        [],
    )
    .unwrap();

    // Dispatcher RPC must return clock_out_of_sync from the entitlement check
    let (res2, _) = dispatch_at(
        &mut conn,
        "students.create",
        "{}",
        Some(&token),
        None,
        "2026-09-20T10:00:00Z",
    );
    let env2: Value = serde_json::from_str(&res2).unwrap();
    assert!(!env2["ok"].as_bool().unwrap());
    assert_eq!(env2["error"]["code"].as_str().unwrap(), "clock_out_of_sync");
}
