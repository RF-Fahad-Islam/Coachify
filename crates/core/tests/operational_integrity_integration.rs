use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::integrity::set_safety_read_only;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    set_safety_read_only(false);
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_operational_integrity.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply all migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut rusqlite::Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
    idem_key: Option<&str>,
) -> (serde_json::Value, Vec<ChangeEvent>) {
    let payload_str = serde_json::to_string(&payload).unwrap();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, idem_key);
    let val: serde_json::Value = serde_json::from_str(&res_str).unwrap();
    (val, events)
}

fn bootstrap_and_login(conn: &mut rusqlite::Connection) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner",
            "display_name": "Integrity Director",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "bootstrap failed: {:?}", res);

    let (res, _) = dispatch_cmd(
        conn,
        "auth.login",
        json!({
            "username": "owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(80000);

fn create_student_helper(
    conn: &mut rusqlite::Connection,
    token: &str,
    name: &str,
    guardian_phone: &str,
) -> (String, String) {
    let seq = STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let student_phone = format!("0171{:07}", seq);
    let (res, _) = dispatch_cmd(
        conn,
        "students.create",
        json!({
            "student": {
                "name": name,
                "gender": "female",
                "dob": "2007-05-15",
                "present_address": "Chittagong, Bangladesh",
                "phone": student_phone,
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": format!("Guardian of {}", name),
                        "relationship": "mother",
                        "phone": guardian_phone
                    }
                }]
            },
            "allow_duplicate_phone": true
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create student failed: {:?}", res);
    (
        res["data"]["id"].as_str().unwrap().to_string(),
        res["data"]["student_code"].as_str().unwrap().to_string(),
    )
}

// -----------------------------------------------------------------------------
// Test 1: Receipt HMAC Cryptographic Signature & Verification (AC-1, AC-2, AC-3)
// -----------------------------------------------------------------------------

#[test]
fn test_receipt_hmac_offline_verification_and_audit() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (student_id, student_code) =
        create_student_helper(&mut conn, &token, "Nusrat Jahan", "01712345678");

    // 1. Post payment using payments.create
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 350000,
                "method": "bkash",
                "reference": "TRX-778899",
                "idempotency_key": "idem-rcp-verify-1",
                "duplicate_override_reason": null
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true, "create payment failed: {:?}", pay_res);
    let receipt_no = pay_res["data"]["receipt_no"].as_str().unwrap();
    let v_code = pay_res["data"]["verification_code"].as_str().unwrap();
    let qr_payload = pay_res["data"]["verification_qr_payload"].as_str().unwrap();

    assert_eq!(
        v_code.len(),
        12,
        "Verification code must be exactly 12 characters"
    );
    assert!(
        qr_payload.starts_with("RCP|"),
        "QR payload must begin with RCP prefix"
    );
    assert!(
        qr_payload.contains(receipt_no),
        "QR payload must contain receipt number"
    );

    // 2. Verify with valid QR payload (AC-2)
    let (verify_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.verify_receipt",
        json!({ "code_or_payload": qr_payload }),
        Some(&token),
        None,
    );
    assert_eq!(verify_res["ok"], true);
    assert_eq!(verify_res["data"]["is_authentic"], true);
    assert_eq!(verify_res["data"]["status"], "valid");
    assert_eq!(verify_res["data"]["receipt_number"], receipt_no);
    assert_eq!(verify_res["data"]["student_code"], student_code);
    assert_eq!(verify_res["data"]["amount_poisha"], 350000);

    // 3. Verify with 12-char alphanumeric code (AC-2)
    let (code_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.verify_receipt",
        json!({ "code_or_payload": v_code }),
        Some(&token),
        None,
    );
    assert_eq!(code_res["ok"], true);
    assert_eq!(code_res["data"]["is_authentic"], true);
    assert_eq!(code_res["data"]["status"], "valid");

    // 4. Verify tampered QR payload (altered amount)
    let tampered_qr = qr_payload.replace("350000", "999999");
    let (tamper_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.verify_receipt",
        json!({ "code_or_payload": tampered_qr }),
        Some(&token),
        None,
    );
    assert_eq!(tamper_res["ok"], true);
    assert_eq!(tamper_res["data"]["is_authentic"], false);
    assert_eq!(tamper_res["data"]["status"], "signature_mismatch");

    // 5. Verify non-existent code
    let (fake_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.verify_receipt",
        json!({ "code_or_payload": "NONEXISTENT1" }),
        Some(&token),
        None,
    );
    assert_eq!(fake_res["ok"], true);
    assert_eq!(fake_res["data"]["is_authentic"], false);
    assert_eq!(fake_res["data"]["status"], "not_found");

    // 6. Check verification audit logs recorded in table (AC-3)
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM receipt_verifications", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        count, 4,
        "All 4 verification checks must be recorded in receipt_verifications table"
    );
}

// -----------------------------------------------------------------------------
// Test 2: Duplicate Payment Detection & Override Validation (AC-4, AC-5, AC-6)
// -----------------------------------------------------------------------------

#[test]
fn test_duplicate_payment_detection_and_override() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (student_id, _) = create_student_helper(&mut conn, &token, "Tanvir Ahmed", "01811223344");

    // Initial legitimate payment
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "nagad",
                "reference": "NAGAD-9988",
                "idempotency_key": "idem-dup-legit-1",
                "duplicate_override_reason": null
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        pay_res["ok"], true,
        "first payment should succeed: {:?}",
        pay_res
    );

    // 1. Reference collision within 24 hours (AC-4)
    let (check_ref, _) = dispatch_cmd(
        &mut conn,
        "integrity.check_duplicate_payment",
        json!({
            "student_id": student_id,
            "amount_poisha": 50000,
            "reference": "NAGAD-9988"
        }),
        Some(&token),
        None,
    );
    assert_eq!(check_ref["ok"], true);
    assert_eq!(check_ref["data"]["is_duplicate"], true);
    assert_eq!(check_ref["data"]["warning_type"], "reference_collision");

    // 2. Same student + daily amount collision (AC-5)
    let (check_amt, _) = dispatch_cmd(
        &mut conn,
        "integrity.check_duplicate_payment",
        json!({
            "student_id": student_id,
            "amount_poisha": 200000,
            "reference": "DIFFERENT-REF"
        }),
        Some(&token),
        None,
    );
    assert_eq!(check_amt["ok"], true);
    assert_eq!(check_amt["data"]["is_duplicate"], true);
    assert_eq!(
        check_amt["data"]["warning_type"],
        "same_student_daily_amount"
    );

    // 3. Attempt posting duplicate payment without override reason: must be rejected (AC-6)
    let (dup_fail, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "reference": null,
                "idempotency_key": "idem-dup-attempt-1",
                "duplicate_override_reason": null
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(dup_fail["ok"], false);
    assert_eq!(dup_fail["error"]["code"], "validation_error");

    // 4. Attempt with invalid override reason (single word or too short)
    let (short_fail, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "reference": null,
                "idempotency_key": "idem-dup-attempt-2",
                "duplicate_override_reason": "ok"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(short_fail["ok"], false);

    // 5. Post with valid 2-word override reason: must succeed and record audit entry (AC-6)
    let (override_ok, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "reference": null,
                "idempotency_key": "idem-dup-override-ok-1",
                "duplicate_override_reason": "Second installment paid separately by uncle"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        override_ok["ok"], true,
        "Payment with valid override reason must succeed: {:?}",
        override_ok
    );

    // Verify audit entry logged
    let audit_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'payment.duplicate_override'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        audit_count, 1,
        "Duplicate override must create an audit log entry"
    );
}

// -----------------------------------------------------------------------------
// Test 3: Guardian Contact Health & Bulk Sanitization (AC-7, AC-8)
// -----------------------------------------------------------------------------

#[test]
fn test_guardian_contact_health_and_bulk_update() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create 3 students: one with valid phone, one invalid phone, one missing
    let (_s1, _) = create_student_helper(&mut conn, &token, "Amina", "01711223344");
    let (_s2, _) = create_student_helper(&mut conn, &token, "Bilkis", "01711223344"); // sibling shared phone
    let (s3, _) = create_student_helper(&mut conn, &token, "Chayan", "01719999999"); // valid initially

    // Set invalid format on guardian of s3 directly
    let guardian_id_to_fix: String = conn
        .query_row(
            "SELECT g.id FROM guardians g JOIN student_guardians sg ON g.id = sg.guardian_id WHERE sg.student_id = ?1",
            [&s3],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE guardians SET phone_norm = '12345' WHERE id = ?1",
        [&guardian_id_to_fix],
    )
    .unwrap();

    let (health_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.get_guardian_contact_health",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(health_res["ok"], true);
    let data = &health_res["data"];
    assert!(data["total_guardians"].as_i64().unwrap() >= 3);
    assert!(data["invalid_phone_count"].as_i64().unwrap() >= 1);
    assert!(data["sibling_group_count"].as_i64().unwrap() >= 1);

    // Bulk update the invalid phone (AC-8)
    let (update_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.bulk_update_guardian_contacts",
        json!({
            "updates": [{
                "guardian_id": guardian_id_to_fix,
                "new_phone": "+8801912345678" // should normalize to 01912345678
            }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(update_res["ok"], true);
    assert_eq!(update_res["data"]["updated_count"], 1);

    let updated_phone: String = conn
        .query_row(
            "SELECT phone_norm FROM guardians WHERE id = ?1",
            [&guardian_id_to_fix],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(updated_phone, "01912345678");
}

// -----------------------------------------------------------------------------
// Test 4: Durable Outbox Lifecycle, Flush & Retry (AC-9, AC-10)
// -----------------------------------------------------------------------------

#[test]
fn test_outbox_lifecycle_flush_and_retry() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Insert outbox items directly
    conn.execute(
        "INSERT INTO outbox (id, channel, to_address, body, scheduled_at, status, attempts, last_error, created_at)
         VALUES ('msg-1', 'sms', '01711000001', 'Test message 1', '2026-09-23T10:00:00Z', 'queued', 0, NULL, '2026-09-23T10:00:00Z'),
                ('msg-2', 'sms', '01711000002', 'Test message 2', '2026-09-23T10:00:00Z', 'failed', 3, 'Timeout', '2026-09-23T10:00:00Z')",
        [],
    ).unwrap();

    // 1. Get lifecycle summary (AC-9)
    let (summary_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.get_outbox_lifecycle",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(summary_res["ok"], true);
    assert_eq!(summary_res["data"]["queued_count"], 1);
    assert_eq!(summary_res["data"]["failed_count"], 1);

    // 2. Retry failed message (AC-9)
    let (retry_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.retry_outbox_messages",
        json!({ "message_ids": ["msg-2"] }),
        Some(&token),
        None,
    );
    assert_eq!(retry_res["ok"], true);
    assert_eq!(retry_res["data"]["retried_count"], 1);

    let status_after: String = conn
        .query_row("SELECT status FROM outbox WHERE id = 'msg-2'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(status_after, "queued");

    // 3. Flush offline queue (AC-10)
    let (flush_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.flush_offline_queue",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(flush_res["ok"], true);
    assert_eq!(flush_res["data"]["processed_count"], 2);
    assert_eq!(flush_res["data"]["sent_count"], 2);

    let sent_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM outbox WHERE status = 'sent'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(sent_count, 2);
}

// -----------------------------------------------------------------------------
// Test 5: Disaster Recovery Score & Drill (AC-11, AC-12)
// -----------------------------------------------------------------------------

#[test]
fn test_disaster_recovery_score_and_verification_drill() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Get initial DR score
    let (score_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.get_disaster_recovery_score",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(score_res["ok"], true);
    let total_score = score_res["data"]["total_score"].as_i64().unwrap();
    assert!((0..=100).contains(&total_score));

    // 2. Run verification drill (AC-12)
    let (drill_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.run_verification_drill",
        json!({ "backup_id": "test-backup-001" }),
        Some(&token),
        None,
    );
    assert_eq!(drill_res["ok"], true);
    assert_eq!(drill_res["data"]["outcome"], "passed");
    assert_eq!(drill_res["data"]["pragma_integrity_ok"], true);
    assert_eq!(drill_res["data"]["foreign_keys_ok"], true);
    assert_eq!(drill_res["data"]["balance_reconciled"], true);
}

// -----------------------------------------------------------------------------
// Test 6: One-Click PC Migration Export and Import (AC-13)
// -----------------------------------------------------------------------------

#[test]
fn test_pc_migration_export_and_import() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let recovery_phrase = "correct horse battery staple 2026 bangladesh offline";

    // 1. Export migration bundle
    let (export_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.export_migration_bundle",
        json!({
            "target_hint": "new_reception_pc",
            "recovery_phrase": recovery_phrase
        }),
        Some(&token),
        None,
    );
    assert_eq!(export_res["ok"], true, "Export failed: {:?}", export_res);
    assert!(export_res["data"]["package_size_bytes"].as_u64().unwrap() > 0);
    assert!(!export_res["data"]["checksum_sha256"]
        .as_str()
        .unwrap()
        .is_empty());
    let bundle_path = export_res["data"]["bundle_path"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Import migration bundle
    let (import_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.import_migration_bundle",
        json!({
            "bundle_path": bundle_path,
            "recovery_phrase": recovery_phrase
        }),
        Some(&token),
        None,
    );
    assert_eq!(import_res["ok"], true, "Import failed: {:?}", import_res);
    assert_eq!(import_res["data"]["checksum_verified"], true);

    // 3. Import with invalid recovery phrase must fail
    let (bad_import, _) = dispatch_cmd(
        &mut conn,
        "integrity.import_migration_bundle",
        json!({
            "bundle_path": bundle_path,
            "recovery_phrase": "wrong phrase completely invalid"
        }),
        Some(&token),
        None,
    );
    assert_eq!(bad_import["ok"], false);
}

// -----------------------------------------------------------------------------
// Test 7: 4-Layer Database Integrity & Read-Only Safety Lock (AC-14)
// -----------------------------------------------------------------------------

struct SafetyLockGuard;
impl Drop for SafetyLockGuard {
    fn drop(&mut self) {
        set_safety_read_only(false);
    }
}

#[test]
fn test_database_integrity_and_read_only_safety_lock() {
    let _guard = SafetyLockGuard;
    set_safety_read_only(false);
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Run database integrity checks
    let (check_res, _) = dispatch_cmd(
        &mut conn,
        "integrity.run_database_integrity_checks",
        json!({ "full": true }),
        Some(&token),
        None,
    );
    assert_eq!(
        check_res["ok"], true,
        "Integrity check failed: {:?}",
        check_res
    );
    assert_eq!(check_res["data"]["overall_ok"], true);
    assert_eq!(check_res["data"]["is_read_only_locked"], false);

    // 2. Activate safety read-only lock
    set_safety_read_only(true);

    // 3. Attempt a mutation command while read-only locked: must be rejected with protective_read_only_mode
    let (mutation_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Should Fail Due to Read Only Lock",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(mutation_res["ok"], false);
    assert_eq!(mutation_res["error"]["code"], "protective_read_only_mode");

    // 4. Verify read commands are still permitted during read-only mode
    let (read_res, _) = dispatch_cmd(&mut conn, "fee_heads.list", json!({}), Some(&token), None);
    assert_eq!(
        read_res["ok"], true,
        "Read commands must succeed in safety read-only mode"
    );

    // Reset lock
    set_safety_read_only(false);
}
