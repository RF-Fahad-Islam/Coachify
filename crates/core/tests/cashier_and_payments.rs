use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::ledger;
use serde_json::json;
use std::time::Instant;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_cashier_payments.db");
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
            "display_name": "Institution Owner",
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(40000);

fn create_student_helper(
    conn: &mut rusqlite::Connection,
    token: &str,
    name: &str,
    name_bn: Option<&str>,
) -> String {
    let seq = STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let student_phone = format!("0171{:07}", seq);
    let guardian_phone = format!("0181{:07}", seq);
    let (res, _) = dispatch_cmd(
        conn,
        "students.create",
        json!({
            "student": {
                "name": name,
                "name_bn": name_bn,
                "gender": "male",
                "dob": "2008-01-01",
                "phone": student_phone,
                "present_address": "Dhaka, Bangladesh",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Primary Guardian",
                        "relationship": "father",
                        "phone": guardian_phone
                    }
                }]
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create student failed: {:?}", res);
    res["data"]["id"].as_str().unwrap().to_string()
}

fn create_fee_head(conn: &mut rusqlite::Connection, token: &str, name: &str, kind: &str) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": name,
                "kind": kind
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true);
    res["data"]["id"].as_str().unwrap().to_string()
}

fn create_charge(
    conn: &mut rusqlite::Connection,
    token: &str,
    student_id: &str,
    fee_head_id: &str,
    amount_poisha: i64,
    due_date: &str,
) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": amount_poisha,
                "due_date": due_date
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true);
    res["data"]["id"].as_str().unwrap().to_string()
}

#[test]
fn test_ac2_cashier_student_overview_aggregation() {
    // AC-2: Cashier student overview query returning student identity, open charges, advance credit, and payment history
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    let stu_id =
        create_student_helper(&mut conn, &token, "Zubair Al Mahmud", Some("জুবায়ের আল মাহমুদ"));

    let head_id = create_fee_head(&mut conn, &token, "Monthly Tuition", "tuition");
    let _c1 = create_charge(&mut conn, &token, &stu_id, &head_id, 200000, today);
    let _c2 = create_charge(&mut conn, &token, &stu_id, &head_id, 150000, today);

    // Query cashier student overview via dispatcher
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_cashier_student_overview",
        json!({ "student_id": stu_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get overview failed: {:?}", res);
    let data = &res["data"];

    assert_eq!(data["name"], "Zubair Al Mahmud");
    assert_eq!(data["name_bn"], "জুবায়ের আল মাহমুদ");
    assert_eq!(data["total_due_poisha"], 350000);
    assert_eq!(data["advance_credit_poisha"], 0);
    assert_eq!(data["net_balance_poisha"], 350000);
    assert_eq!(data["guardian_name"], "Primary Guardian");

    let open_charges = data["open_charges"].as_array().unwrap();
    assert_eq!(open_charges.len(), 2);
    assert_eq!(data["recent_payments"].as_array().unwrap().len(), 0);

    // Record partial payment of 2000 Taka (200,000 poisha)
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 200000,
                "method": "cash",
                "reference": "CTR-01",
                "business_date": today,
                "idempotency_key": "pay-cashier-ov-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true);

    // Refresh overview
    let (res2, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_cashier_student_overview",
        json!({ "student_id": stu_id }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], true);
    let data2 = &res2["data"];
    assert_eq!(data2["total_due_poisha"], 150000); // 3500 - 2000 = 1500
    assert_eq!(data2["net_balance_poisha"], 150000);
    assert_eq!(data2["recent_payments"].as_array().unwrap().len(), 1);
}

#[test]
fn test_ac4_oldest_first_and_manual_allocation() {
    // AC-4: Oldest-first allocation hierarchy vs manual targeting
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let stu_id = create_student_helper(&mut conn, &token, "Fahim Shahriar", None);
    let head_id = create_fee_head(&mut conn, &token, "Course Fee", "tuition");

    let c1 = create_charge(&mut conn, &token, &stu_id, &head_id, 100000, "2026-07-01");
    let c2 = create_charge(&mut conn, &token, &stu_id, &head_id, 100000, "2026-08-01");
    let c3 = create_charge(&mut conn, &token, &stu_id, &head_id, 100000, "2026-09-01");

    // Pay 1500 Taka oldest-first: should pay c1 fully (1000), c2 half (500)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 150000,
                "method": "bkash",
                "idempotency_key": "pay-alloc-oldest"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["allocated_amount_poisha"], 150000);
    assert_eq!(res["data"]["unallocated_amount_poisha"], 0);

    // Verify c1 is fully paid, c2 has 500 left, c3 has 1000 left
    let charges = ledger::list_charges_by_student(&conn, &stu_id).unwrap();
    let charge1 = charges.iter().find(|c| c.id == c1).unwrap();
    let charge2 = charges.iter().find(|c| c.id == c2).unwrap();
    let charge3 = charges.iter().find(|c| c.id == c3).unwrap();
    assert_eq!(charge1.remaining_due_poisha, 0);
    assert_eq!(charge1.allocated_amount_poisha, 100000);
    assert_eq!(charge2.remaining_due_poisha, 50000);
    assert_eq!(charge2.allocated_amount_poisha, 50000);
    assert_eq!(charge3.remaining_due_poisha, 100000);
    assert_eq!(charge3.allocated_amount_poisha, 0);

    // Test manual allocation: target c3 specifically (skip c2)
    let (res_man, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 80000,
                "method": "nagad",
                "target_charge_ids": [c3],
                "idempotency_key": "pay-alloc-manual"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res_man["ok"], true);

    let charges_after = ledger::list_charges_by_student(&conn, &stu_id).unwrap();
    let charge3_after = charges_after.iter().find(|c| c.id == c3).unwrap();
    assert_eq!(charge3_after.remaining_due_poisha, 20000);
    assert_eq!(charge3_after.allocated_amount_poisha, 80000);
}

#[test]
fn test_ac7_concurrency_stale_due_guard() {
    // AC-7: Concurrency collision detection with expected_balance_poisha
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    let stu_id = create_student_helper(&mut conn, &token, "Nafis Fuad", None);
    let head_id = create_fee_head(&mut conn, &token, "Coaching Package", "tuition");
    create_charge(&mut conn, &token, &stu_id, &head_id, 500000, today);

    // Initial balance is 500,000 poisha (5,000 Taka)
    let summary = ledger::get_student_ledger_summary(&conn, &stu_id).unwrap();
    assert_eq!(summary.balance_poisha, 500000);

    // Payment with correct expected balance succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 100000,
                "method": "cash",
                "expected_balance_poisha": 500000,
                "idempotency_key": "pay-concurrent-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Balance is now 400,000 poisha.
    // Window 2 attempts payment assuming balance was still 500,000: MUST FAIL with due_changed
    let (res2, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 100000,
                "method": "cash",
                "expected_balance_poisha": 500000,
                "idempotency_key": "pay-concurrent-02"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], false);
    assert_eq!(res2["error"]["code"], "due_changed");
}

#[test]
fn test_ac8_double_click_idempotency_protection() {
    // AC-8: Double-click idempotency guard creates exactly one payment row and receipt
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    let stu_id = create_student_helper(&mut conn, &token, "Sadman Sakib", None);
    let head_id = create_fee_head(&mut conn, &token, "Model Test", "tuition");
    create_charge(&mut conn, &token, &stu_id, &head_id, 300000, today);

    let idem_key = uuid::Uuid::now_v7().to_string();

    // First payment submission
    let (res1, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 150000,
                "method": "cash",
                "idempotency_key": idem_key
            }
        }),
        Some(&token),
        Some(&idem_key),
    );
    assert_eq!(res1["ok"], true);
    let receipt1 = res1["data"]["receipt_no"].as_str().unwrap().to_string();
    let payment_id1 = res1["data"]["id"].as_str().unwrap().to_string();

    // Rapid second click with identical idempotency key
    let (res2, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 150000,
                "method": "cash",
                "idempotency_key": idem_key
            }
        }),
        Some(&token),
        Some(&idem_key),
    );
    assert_eq!(res2["ok"], true);
    let receipt2 = res2["data"]["receipt_no"].as_str().unwrap().to_string();
    let payment_id2 = res2["data"]["id"].as_str().unwrap().to_string();

    // Both calls must return identical receipt number and ID
    assert_eq!(receipt1, receipt2);
    assert_eq!(payment_id1, payment_id2);

    // Verify SQLite table only contains 1 payment row
    let payment_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM payments WHERE student_id = ?1",
            rusqlite::params![stu_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(payment_count, 1, "Only 1 payment record must exist");

    // Balance must only be deducted once (3000 - 1500 = 1500)
    let summary = ledger::get_student_ledger_summary(&conn, &stu_id).unwrap();
    assert_eq!(summary.balance_poisha, 150000);
}

#[test]
fn test_ac6_sub_second_latency_benchmark() {
    // AC-6: Cashier payment collection and overview must complete in under 800 ms
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    let stu_id = create_student_helper(&mut conn, &token, "Latency Test Student", None);
    let head_id = create_fee_head(&mut conn, &token, "Fast Pay Head", "tuition");
    create_charge(&mut conn, &token, &stu_id, &head_id, 100000, today);

    let start = Instant::now();

    // 1. Overview fetch
    let (res_ov, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_cashier_student_overview",
        json!({ "student_id": stu_id }),
        Some(&token),
        None,
    );
    assert_eq!(res_ov["ok"], true);

    // 2. Payment submission
    let (res_pay, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 100000,
                "method": "cash",
                "idempotency_key": "benchmark-pay-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res_pay["ok"], true);

    // 3. Receipt fetch
    let payment_id = res_pay["data"]["id"].as_str().unwrap();
    let (res_rec, _) = dispatch_cmd(
        &mut conn,
        "receipts.get_printable",
        json!({ "payment_id": payment_id }),
        Some(&token),
        None,
    );
    assert_eq!(res_rec["ok"], true);

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 800,
        "End-to-end cashier workflow took {:?}, must be under 800ms",
        duration
    );
}
