use coachify_core::db;
use coachify_core::dispatcher::dispatch_at;
use coachify_core::licensing::{
    create_signed_license_file, generate_request_code_for_installation, LicenseLimits,
    LicensePayload, LicenseStatusResponse, RequestCodeResponse,
};
use rusqlite::Connection;
use serde_json::{json, Value};

fn setup_test_env() -> (Connection, String) {
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    db::apply_all_migrations(&conn).expect("apply all migrations");

    let now = "2026-09-25T09:55:00Z";
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
fn test_automatic_14_day_trial_generation_ac4() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (res_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let env: Value = serde_json::from_str(&res_str).unwrap();
    assert!(env["ok"].as_bool().unwrap());

    let status: LicenseStatusResponse = serde_json::from_value(env["data"].clone()).unwrap();
    assert_eq!(status.tier, "trial");
    assert_eq!(status.status, "active");
    assert!(!status.is_read_only);
    assert!(!status.is_moved);
    assert_eq!(status.days_remaining, 14);
    assert_eq!(status.valid_until, "2026-10-09");

    // Verify sentinel signature and key_id in database
    let (key_id, sig, is_active): (String, String, i32) = conn
        .query_row(
            "SELECT key_id, signature, is_active FROM active_license",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(key_id, "trial");
    assert_eq!(sig, "LOCAL_TRIAL");
    assert_eq!(is_active, 1);
}

#[test]
fn test_authentic_ed25519_signature_verification_and_import_ac1_ac9() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let status_env: Value = serde_json::from_str(&status_str).unwrap();
    let host_id = status_env["data"]["installation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let payload = LicensePayload {
        license_id: "LIC-2026-9001".to_string(),
        customer_id: "CUST-APEX-01".to_string(),
        customer_name: "Apex Coaching Center".to_string(),
        installation_id: host_id.clone(),
        tier: "local_pro".to_string(),
        kind: "paid".to_string(),
        serial: 1,
        paid_through: "2027-09-25".to_string(),
        grace_days: 14,
        valid_until: "2027-10-09".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits {
            active_students: Some(1000),
            workstations: Some(5),
        },
        capabilities: vec!["core".to_string(), "lan".to_string(), "sms".to_string()],
        key_id: "k2026a".to_string(),
    };

    let signed_file = create_signed_license_file(&payload, "k2026a").unwrap();
    let import_req = json!({ "license_content": signed_file }).to_string();

    let (import_str, events) = dispatch_at(
        &mut conn,
        "license.import_file",
        &import_req,
        Some(&token),
        None,
        now,
    );
    let import_env: Value = serde_json::from_str(&import_str).unwrap();
    assert!(
        import_env["ok"].as_bool().unwrap(),
        "Expected valid license import to succeed: {import_env}"
    );
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "license:changed");

    let updated: LicenseStatusResponse =
        serde_json::from_value(import_env["data"].clone()).unwrap();
    assert_eq!(updated.tier, "local_pro");
    assert_eq!(updated.status, "active");
    assert_eq!(updated.license_id, "LIC-2026-9001");
    assert_eq!(updated.student_limit, Some(1000));
    assert_eq!(updated.workstation_limit, Some(5));
    assert!(!updated.is_read_only);
}

#[test]
fn test_tampered_payload_rejection_ac1() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let status_env: Value = serde_json::from_str(&status_str).unwrap();
    let host_id = status_env["data"]["installation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let payload = LicensePayload {
        license_id: "LIC-2026-9002".to_string(),
        customer_id: "CUST-APEX-01".to_string(),
        customer_name: "Apex Coaching Center".to_string(),
        installation_id: host_id,
        tier: "local".to_string(),
        kind: "paid".to_string(),
        serial: 1,
        paid_through: "2027-09-25".to_string(),
        grace_days: 14,
        valid_until: "2027-10-09".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits {
            active_students: Some(300),
            workstations: Some(1),
        },
        capabilities: vec!["core".to_string()],
        key_id: "k2026b".to_string(),
    };

    let signed_file = create_signed_license_file(&payload, "k2026b").unwrap();
    // Tamper with the tier in the signed file
    let tampered_file = signed_file.replace("\"local\"", "\"local_pro\"");
    let import_req = json!({ "license_content": tampered_file }).to_string();

    let (import_str, _) = dispatch_at(
        &mut conn,
        "license.import_file",
        &import_req,
        Some(&token),
        None,
        now,
    );
    let import_env: Value = serde_json::from_str(&import_str).unwrap();
    assert!(!import_env["ok"].as_bool().unwrap());
    assert_eq!(
        import_env["error"]["code"].as_str().unwrap(),
        "license_signature_invalid"
    );
}

#[test]
fn test_installation_hardware_binding_mismatch_ac2() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let foreign_payload = LicensePayload {
        license_id: "LIC-2026-9003".to_string(),
        customer_id: "CUST-OTHER-99".to_string(),
        customer_name: "Other Coaching Center".to_string(),
        installation_id: "foreign-pc-installation-uuid-9999".to_string(),
        tier: "local_pro".to_string(),
        kind: "paid".to_string(),
        serial: 1,
        paid_through: "2027-09-25".to_string(),
        grace_days: 14,
        valid_until: "2027-10-09".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits::default(),
        capabilities: vec!["core".to_string()],
        key_id: "k2026a".to_string(),
    };

    let signed_file = create_signed_license_file(&foreign_payload, "k2026a").unwrap();
    let import_req = json!({ "license_content": signed_file }).to_string();

    let (import_str, _) = dispatch_at(
        &mut conn,
        "license.import_file",
        &import_req,
        Some(&token),
        None,
        now,
    );
    let import_env: Value = serde_json::from_str(&import_str).unwrap();
    assert!(!import_env["ok"].as_bool().unwrap());
    assert_eq!(
        import_env["error"]["code"].as_str().unwrap(),
        "license_installation_mismatch"
    );
}

#[test]
fn test_monotonic_serial_progression_downgrade_rejected_ac3() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let status_env: Value = serde_json::from_str(&status_str).unwrap();
    let host_id = status_env["data"]["installation_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Import serial 5 first
    let payload_v5 = LicensePayload {
        license_id: "LIC-2026-9004".to_string(),
        customer_id: "CUST-APEX-01".to_string(),
        customer_name: "Apex Coaching Center".to_string(),
        installation_id: host_id.clone(),
        tier: "local_pro".to_string(),
        kind: "paid".to_string(),
        serial: 5,
        paid_through: "2027-09-25".to_string(),
        grace_days: 14,
        valid_until: "2027-10-09".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits::default(),
        capabilities: vec![],
        key_id: "k2026a".to_string(),
    };
    let file_v5 = create_signed_license_file(&payload_v5, "k2026a").unwrap();
    let (res_v5, _) = dispatch_at(
        &mut conn,
        "license.import_file",
        &json!({ "license_content": file_v5 }).to_string(),
        Some(&token),
        None,
        now,
    );
    let env_v5: Value = serde_json::from_str(&res_v5).unwrap();
    assert!(env_v5["ok"].as_bool().unwrap());

    // Attempt downgrade to serial 4
    let mut payload_v4 = payload_v5.clone();
    payload_v4.serial = 4;
    let file_v4 = create_signed_license_file(&payload_v4, "k2026a").unwrap();
    let (res_v4, _) = dispatch_at(
        &mut conn,
        "license.import_file",
        &json!({ "license_content": file_v4 }).to_string(),
        Some(&token),
        None,
        now,
    );
    let env_v4: Value = serde_json::from_str(&res_v4).unwrap();
    assert!(!env_v4["ok"].as_bool().unwrap());
    assert_eq!(
        env_v4["error"]["code"].as_str().unwrap(),
        "license_serial_downgrade"
    );
}

#[test]
fn test_grace_warnings_and_dispatcher_read_only_enforcement_ac5_ac6() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let status_env: Value = serde_json::from_str(&status_str).unwrap();
    let host_id = status_env["data"]["installation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let payload = LicensePayload {
        license_id: "LIC-2026-9005".to_string(),
        customer_id: "CUST-APEX-01".to_string(),
        customer_name: "Apex Coaching Center".to_string(),
        installation_id: host_id,
        tier: "local".to_string(),
        kind: "paid".to_string(),
        serial: 2,
        paid_through: "2026-10-15".to_string(),
        grace_days: 14,
        valid_until: "2026-10-29".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits::default(),
        capabilities: vec![],
        key_id: "k2026a".to_string(),
    };

    let signed = create_signed_license_file(&payload, "k2026a").unwrap();
    let _ = dispatch_at(
        &mut conn,
        "license.import_file",
        &json!({ "license_content": signed }).to_string(),
        Some(&token),
        None,
        now,
    );

    // 1. 20 days before paid_through -> expiring_soon, write still allowed
    let (exp_soon_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let exp_soon: Value = serde_json::from_str(&exp_soon_str).unwrap();
    assert_eq!(
        exp_soon["data"]["status"].as_str().unwrap(),
        "expiring_soon"
    );
    assert!(!exp_soon["data"]["is_read_only"].as_bool().unwrap());

    // 2. Past paid_through (2026-10-20) but before valid_until (2026-10-29) -> in_grace, write still allowed
    let in_grace_time = "2026-10-20T10:00:00Z";
    // Re-login to get active session at in_grace_time
    let (login_str, _) = dispatch_at(
        &mut conn,
        "auth.login",
        &json!({ "username": "owner", "password": "OwnerPassword123!" }).to_string(),
        None,
        None,
        in_grace_time,
    );
    let grace_token = serde_json::from_str::<Value>(&login_str).unwrap()["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (grace_status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&grace_token),
        None,
        in_grace_time,
    );
    let grace_status: Value = serde_json::from_str(&grace_status_str).unwrap();
    assert_eq!(grace_status["data"]["status"].as_str().unwrap(), "in_grace");
    assert!(!grace_status["data"]["is_read_only"].as_bool().unwrap());

    // Write succeeds during grace period
    let (subj_ok_str, _) = dispatch_at(
        &mut conn,
        "subjects.create",
        &json!({ "subject": { "name": "Physics", "code": "PHY1" } }).to_string(),
        Some(&grace_token),
        None,
        in_grace_time,
    );
    let subj_ok: Value = serde_json::from_str(&subj_ok_str).unwrap();
    assert!(
        subj_ok["ok"].as_bool().unwrap(),
        "Expected write to succeed during grace: {subj_ok}"
    );

    // 3. Past valid_until (2026-11-01) -> expired_past_grace, writes rejected with license_expired_read_only, reads succeed
    let expired_time = "2026-11-01T10:00:00Z";
    let (login_exp_str, _) = dispatch_at(
        &mut conn,
        "auth.login",
        &json!({ "username": "owner", "password": "OwnerPassword123!" }).to_string(),
        None,
        None,
        expired_time,
    );
    let exp_token = serde_json::from_str::<Value>(&login_exp_str).unwrap()["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (write_blocked_str, _) = dispatch_at(
        &mut conn,
        "subjects.create",
        &json!({ "subject": { "name": "Chemistry", "code": "CHEM1" } }).to_string(),
        Some(&exp_token),
        None,
        expired_time,
    );
    let write_blocked: Value = serde_json::from_str(&write_blocked_str).unwrap();
    assert!(!write_blocked["ok"].as_bool().unwrap());
    assert_eq!(
        write_blocked["error"]["code"].as_str().unwrap(),
        "license_expired_read_only"
    );

    // Read commands still succeed in read-only mode
    let (read_ok_str, _) = dispatch_at(
        &mut conn,
        "subjects.list",
        "{}",
        Some(&exp_token),
        None,
        expired_time,
    );
    let read_ok: Value = serde_json::from_str(&read_ok_str).unwrap();
    assert!(read_ok["ok"].as_bool().unwrap());
}

#[test]
fn test_moved_machine_transitional_grace_window_ac7() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (status_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let status_env: Value = serde_json::from_str(&status_str).unwrap();
    let original_host_id = status_env["data"]["installation_id"]
        .as_str()
        .unwrap()
        .to_string();

    let payload = LicensePayload {
        license_id: "LIC-2026-MOVED-01".to_string(),
        customer_id: "CUST-APEX-01".to_string(),
        customer_name: "Apex Coaching Center".to_string(),
        installation_id: original_host_id,
        tier: "local_pro".to_string(),
        kind: "paid".to_string(),
        serial: 1,
        paid_through: "2027-09-25".to_string(),
        grace_days: 14,
        valid_until: "2027-10-09".to_string(),
        issued_at: now.to_string(),
        limits: LicenseLimits::default(),
        capabilities: vec![],
        key_id: "k2026a".to_string(),
    };
    let signed = create_signed_license_file(&payload, "k2026a").unwrap();
    let _ = dispatch_at(
        &mut conn,
        "license.import_file",
        &json!({ "license_content": signed }).to_string(),
        Some(&token),
        None,
        now,
    );

    // Simulate restoring database onto a new replacement computer with a different host_installation_id
    conn.execute(
        "INSERT INTO settings (key, value_json, updated_at) VALUES ('host_installation_id', '\"replacement-pc-uuid-2026\"', ?1)",
        [now],
    )
    .unwrap();

    let (moved_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token),
        None,
        now,
    );
    let moved_env: Value = serde_json::from_str(&moved_str).unwrap();
    let moved_status: LicenseStatusResponse =
        serde_json::from_value(moved_env["data"].clone()).unwrap();

    assert!(moved_status.is_moved);
    assert_eq!(moved_status.status, "moved_transitional");
    assert!(!moved_status.is_read_only);
    assert_eq!(moved_status.moved_days_remaining, Some(14));

    // Attempting to clear moved_first_detected_at in SQLite (simulating repeated restore) does not reset the 14-day window
    conn.execute("UPDATE licenses SET moved_first_detected_at = NULL", [])
        .unwrap();
    let day_5 = "2026-09-30T10:00:00Z";
    let (login_d5_str, _) = dispatch_at(
        &mut conn,
        "auth.login",
        &json!({ "username": "owner", "password": "OwnerPassword123!" }).to_string(),
        None,
        None,
        day_5,
    );
    let token_d5 = serde_json::from_str::<Value>(&login_d5_str).unwrap()["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (d5_str, _) = dispatch_at(
        &mut conn,
        "license.get_status",
        "{}",
        Some(&token_d5),
        None,
        day_5,
    );
    let d5_env: Value = serde_json::from_str(&d5_str).unwrap();
    let d5_status: LicenseStatusResponse = serde_json::from_value(d5_env["data"].clone()).unwrap();
    assert_eq!(d5_status.moved_days_remaining, Some(9));
}

#[test]
fn test_deterministic_offline_request_code_ac8() {
    let (mut conn, token) = setup_test_env();
    let now = "2026-09-25T10:00:00Z";

    let (req_str, _) = dispatch_at(
        &mut conn,
        "license.get_request_code",
        "{}",
        Some(&token),
        None,
        now,
    );
    let req_env: Value = serde_json::from_str(&req_str).unwrap();
    assert!(req_env["ok"].as_bool().unwrap());

    let code_resp: RequestCodeResponse = serde_json::from_value(req_env["data"].clone()).unwrap();
    assert!(code_resp.request_code.starts_with("REQ-"));
    assert_eq!(code_resp.request_code.len(), 23); // REQ-XXXX-XXXX-XXXX-XXXX
    assert!(code_resp.qr_payload.starts_with("COACHIFY-REQ|v1|"));

    // Determinism check
    let second = generate_request_code_for_installation(&code_resp.installation_id);
    assert_eq!(code_resp.request_code, second.request_code);
    assert_eq!(code_resp.qr_payload, second.qr_payload);
}

#[test]
fn test_clock_rollback_defense_and_forward_jump_recovery_ac10() {
    let (mut conn, _token) = setup_test_env();
    let future_time = "2026-09-26T12:00:00Z";

    // Advance clock watermark by executing a write at future_time
    let (login_fut_str, _) = dispatch_at(
        &mut conn,
        "auth.login",
        &json!({ "username": "owner", "password": "OwnerPassword123!" }).to_string(),
        None,
        None,
        future_time,
    );
    let fut_token = serde_json::from_str::<Value>(&login_fut_str).unwrap()["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (write_fut_str, _) = dispatch_at(
        &mut conn,
        "subjects.create",
        &json!({ "subject": { "name": "Mathematics", "code": "MATH1" } }).to_string(),
        Some(&fut_token),
        None,
        future_time,
    );
    let write_fut: Value = serde_json::from_str(&write_fut_str).unwrap();
    assert!(
        write_fut["ok"].as_bool().unwrap(),
        "Expected write at future_time to succeed: {write_fut}"
    );

    // Roll system clock backwards by 6 hours
    let rolled_back_time = "2026-09-26T06:00:00Z";
    let (login_rb_str, _) = dispatch_at(
        &mut conn,
        "auth.login",
        &json!({ "username": "owner", "password": "OwnerPassword123!" }).to_string(),
        None,
        None,
        rolled_back_time,
    );
    let rb_token = serde_json::from_str::<Value>(&login_rb_str).unwrap()["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (write_rb_str, _) = dispatch_at(
        &mut conn,
        "subjects.create",
        &json!({ "subject": { "name": "Biology", "code": "BIO1" } }).to_string(),
        Some(&rb_token),
        None,
        rolled_back_time,
    );
    let write_rb: Value = serde_json::from_str(&write_rb_str).unwrap();
    assert!(!write_rb["ok"].as_bool().unwrap());
    assert_eq!(
        write_rb["error"]["code"].as_str().unwrap(),
        "clock_out_of_sync"
    );

    // Owner acknowledges clock jump at rolled_back_time
    let (ack_str, _) = dispatch_at(
        &mut conn,
        "system.acknowledge_clock_jump",
        &json!({ "reason": "Fixed accidental date forward shift" }).to_string(),
        Some(&rb_token),
        None,
        rolled_back_time,
    );
    assert!(serde_json::from_str::<Value>(&ack_str).unwrap()["ok"]
        .as_bool()
        .unwrap());

    // Write operation now succeeds at rolled_back_time
    let (write_recovered_str, _) = dispatch_at(
        &mut conn,
        "subjects.create",
        &json!({ "subject": { "name": "Biology", "code": "BIO1" } }).to_string(),
        Some(&rb_token),
        None,
        rolled_back_time,
    );
    let write_recovered: Value = serde_json::from_str(&write_recovered_str).unwrap();
    assert!(
        write_recovered["ok"].as_bool().unwrap(),
        "Write should succeed after clock jump acknowledgment: {write_recovered}"
    );
}

#[test]
fn test_append_only_trigger_prevents_license_deletion_ac11() {
    let (conn, _token) = setup_test_env();
    let err = conn
        .execute("DELETE FROM licenses", [])
        .expect_err("DELETE on licenses table should be blocked by SQLite trigger");
    assert!(
        err.to_string()
            .contains("Historical license records cannot be deleted"),
        "Unexpected trigger error: {err}"
    );
}
