use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::ledger::current_business_date;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_slice2_features.db");
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(30000);

fn create_student_helper(conn: &mut rusqlite::Connection, token: &str, name: &str) -> String {
    let seq = STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (res, _) = dispatch_cmd(
        conn,
        "students.create",
        json!({
            "student": {
                "name": name,
                "gender": "male",
                "dob": "2008-01-01",
                "phone": format!("01711{:06}", seq % 1000000),
                "present_address": "Dhaka, Bangladesh",
                "guardians": [
                    {
                        "is_primary": true,
                        "receives_sms": true,
                        "new_guardian": {
                            "name": format!("Guardian of {}", name),
                            "relationship": "father",
                            "phone": format!("01811{:06}", seq % 1000000)
                        }
                    }
                ]
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create student failed: {:?}", res);
    res["data"]["id"].as_str().unwrap().to_string()
}

fn create_fee_head_helper(
    conn: &mut rusqlite::Connection,
    token: &str,
    name: &str,
    kind: &str,
) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": name,
                "name_bn": name,
                "kind": kind
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create fee head failed: {:?}", res);
    res["data"]["id"].as_str().unwrap().to_string()
}

#[test]
fn test_feature_17_voids_refunds_and_adjustments() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let student_id = create_student_helper(&mut conn, &token, "Rahim Ahmed");
    let fee_head_id = create_fee_head_helper(&mut conn, &token, "Monthly Tuition", "tuition");

    let today = current_business_date();

    // 1. Create a manual charge of ৳2,000 (200,000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": 200000,
                "billing_month": "2026-09",
                "due_date": today,
                "description": "Tuition for Sep 2026"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create charge failed: {:?}", res);
    let charge_id = res["data"]["id"].as_str().unwrap().to_string();

    // Verify balance is 200000
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["balance_poisha"], 200000);

    // 2. Pay ৳2,000 in cash
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "business_date": today,
                "reference": "COUNTER-01",
                "idempotency_key": "pay-test-17-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "payment failed: {:?}", res);
    let payment_id = res["data"]["id"].as_str().unwrap().to_string();

    // Verify balance is 0
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 0);

    // 3. Void payment on open same day should succeed
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.void",
        json!({
            "payment_id": payment_id,
            "reason": "Mistaken cash entry"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "void payment failed: {:?}", res);

    // Verify balance is restored to 200000 and charge is restored
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 200000);

    // 4. Test adjustment waiver ceiling
    // Charge balance is 200000. Try waiver of 250000 -> must fail ceiling!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "adjustments.create",
        json!({
            "adjustment": {
                "student_id": student_id,
                "charge_id": charge_id,
                "type": "waiver",
                "amount_poisha": 250000,
                "reason": "Exorbitant waiver"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        res["ok"], false,
        "waiver exceeding charge must fail: {:?}",
        res
    );

    // Valid waiver of 50000 (৳500)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "adjustments.create",
        json!({
            "adjustment": {
                "student_id": student_id,
                "charge_id": charge_id,
                "type": "waiver",
                "amount_poisha": 50000,
                "reason": "Merit discount"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "valid waiver failed: {:?}", res);

    // Verify balance is now 150000
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 150000);

    // 5. Pay ৳2,000 (200000 poisha) -> covers 150000 due and leaves 50000 advance credit
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "business_date": today,
                "reference": "COUNTER-02",
                "idempotency_key": "pay-test-17-02"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let payment2_id = res["data"]["id"].as_str().unwrap().to_string();

    // 6. Test refund ceiling on payment2: available unallocated is 50000
    // Requesting 60000 must fail ceiling!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "refunds.create",
        json!({
            "refund": {
                "payment_id": payment2_id,
                "amount_poisha": 60000,
                "method": "cash",
                "reason": "Over-refund attempt"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        res["ok"], false,
        "refund exceeding unallocated credit must fail"
    );

    // Valid refund of 30000 (৳300)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "refunds.create",
        json!({
            "refund": {
                "payment_id": payment2_id,
                "amount_poisha": 30000,
                "method": "cash",
                "reason": "Advance credit return"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "valid refund failed: {:?}", res);
}

#[test]
fn test_feature_18_direct_admission_pipeline() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Set up Program, Batch, Fee Head, and Fee Plan
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "HSC Science 2026",
                "code": "HSC-SCI-26",
                "class_level": "HSC"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create program failed: {:?}", res);
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Physics Batch A",
                "code": "PHY-A-26",
                "max_capacity": 40,
                "start_date": "2026-09-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create batch failed: {:?}", res);
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    let head_id = create_fee_head_helper(&mut conn, &token, "Admission Fee", "admission");

    // Create fee plan for this batch: ৳5,000 (500000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "fee_head_id": head_id,
                "amount_poisha": 500000,
                "currency": "BDT",
                "batch_id": batch_id,
                "frequency": "once",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create fee plan failed: {:?}", res);

    // 2. Perform direct admission with concession and initial payment
    let admit_payload = json!({
        "input": {
            "student": {
                "name": "Sadia Islam",
                "gender": "female",
                "dob": "2008-05-15",
                "phone": "01755123456",
                "present_address": "Mirpur, Dhaka",
                "guardians": [
                    {
                        "is_primary": true,
                        "receives_sms": true,
                        "new_guardian": {
                            "name": "Mohammad Islam",
                            "relationship": "father",
                            "phone": "01955123456"
                        }
                    }
                ]
            },
            "guardian": {
                "is_primary": true,
                "receives_sms": true,
                "new_guardian": {
                    "name": "Mohammad Islam",
                    "relationship": "father",
                    "phone": "01955123456"
                }
            },
            "batch_ids": [batch_id],
            "concessions": [
                {
                    "fee_head_id": head_id,
                    "batch_id": batch_id,
                    "type": "percent",
                    "value": 1000,
                    "reason": "Early bird 10% discount"
                }
            ],
            "payment": {
                "amount_poisha": 450000,
                "method": "bkash",
                "reference": "TRX998877"
            },
            "idempotency_key": "admit-test-001"
        }
    });

    let (res, events) = dispatch_cmd(
        &mut conn,
        "students.admit_with_payment",
        admit_payload,
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "admit student failed: {:?}", res);

    let data = &res["data"];
    let student_id = data["student_id"].as_str().unwrap();
    assert!(data["student_code"].as_str().unwrap().starts_with('S'));
    assert_eq!(data["enrollment_ids"].as_array().unwrap().len(), 1);
    assert_eq!(data["charge_ids"].as_array().unwrap().len(), 1);
    assert!(data["payment_receipt"].is_object());
    let receipt = &data["payment_receipt"];
    assert!(receipt["receipt_no"].as_str().unwrap().starts_with("REC-"));
    assert_eq!(receipt["amount_poisha"], 450000);

    // Balance should be zero because 500000 - 50000 concession - 450000 payment = 0
    assert_eq!(data["balance_poisha"], 0);

    // Verify events were emitted for students, enrollments, and payments
    assert!(events.iter().any(|e| e.topic == "students"));
    assert!(events.iter().any(|e| e.topic == "enrollments"));
    assert!(events.iter().any(|e| e.topic == "payments"));

    // Verify student profile exists in DB
    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.get",
        json!({ "id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["student"]["name"], "Sadia Islam");
}

#[test]
fn test_feature_19_dues_directory_and_bilingual_statement() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let student_id = create_student_helper(&mut conn, &token, "Tanvir Hasan");
    let head_id = create_fee_head_helper(&mut conn, &token, "Coaching Fee", "tuition");

    let today = current_business_date();

    // Create a current charge of ৳1,500
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": head_id,
                "amount_poisha": 150000,
                "billing_month": "2026-09",
                "due_date": today,
                "description": "Current Month Fee"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Create an overdue charge from last month of ৳1,000
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": head_id,
                "amount_poisha": 100000,
                "billing_month": "2026-08",
                "due_date": "2026-08-10",
                "description": "Previous Month Fee"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 1. Query dues directory via ledger.list_dues
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.list_dues",
        json!({
            "filter": {
                "search": "Tanvir"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "list_dues failed: {:?}", res);
    let items = res["data"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["name"], "Tanvir Hasan");
    assert_eq!(item["total_due_poisha"], 250000);
    assert_eq!(item["current_month_due_poisha"], 150000);
    assert_eq!(item["overdue_poisha"], 100000);

    // Verify mathematical reconciliation with student_balances
    let total_in_dues: i64 = items
        .iter()
        .map(|it| it["total_due_poisha"].as_i64().unwrap())
        .sum();
    let cache_total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(balance_poisha), 0) FROM student_balances WHERE balance_poisha > 0",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(
        total_in_dues, cache_total,
        "Dues list must strictly equal student_balances cache!"
    );

    // 2. Query bilingual due statement via ledger.get_due_statement
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_due_statement",
        json!({
            "student_id": student_id
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get_due_statement failed: {:?}", res);
    let stmt = &res["data"];
    assert_eq!(stmt["student_name"], "Tanvir Hasan");
    assert_eq!(stmt["total_charged_poisha"], 250000);
    assert_eq!(stmt["net_due_poisha"], 250000);
    assert_eq!(stmt["itemized_charges"].as_array().unwrap().len(), 2);
    // Currency in words
    assert!(!stmt["amount_in_words_bn"].as_str().unwrap().is_empty());
    assert!(!stmt["amount_in_words_en"].as_str().unwrap().is_empty());
}

#[test]
fn test_feature_20_expenses_and_daily_cash_close() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let today = current_business_date();

    // 1. List expense categories (auto-seeds defaults)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "expense_categories.list",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let cats = res["data"].as_array().unwrap();
    assert!(
        cats.len() >= 9,
        "Should auto seed at least 9 default categories"
    );
    let printing_cat = cats
        .iter()
        .find(|c| c["name"] == "Printing & Stationery")
        .unwrap();
    let cat_id = printing_cat["id"].as_str().unwrap().to_string();

    // 2. Open cash day for today
    let (res, _) = dispatch_cmd(
        &mut conn,
        "cash_days.get_or_open_today",
        json!({ "business_date": today }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["opening_cash_poisha"], 0);

    // 3. Receive student cash payment of ৳3,000 (300000 poisha)
    let student_id = create_student_helper(&mut conn, &token, "Nafis Fuad");
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 300000,
                "method": "cash",
                "business_date": today,
                "reference": "CASH-REC-1",
                "idempotency_key": "pay-nafis-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Also receive a digital payment via bKash of ৳1,500 (150000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 150000,
                "method": "bkash",
                "business_date": today,
                "reference": "BKASH-TX-99",
                "idempotency_key": "pay-nafis-02"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 4. Record petty expense of ৳500 (50000 poisha) for whiteboard markers in cash
    let (res, _) = dispatch_cmd(
        &mut conn,
        "expenses.create",
        json!({
            "expense": {
                "category_id": cat_id,
                "amount_poisha": 50000,
                "method": "cash",
                "payee": "Bismillah Stationery",
                "reference": "BILL-104",
                "business_date": today
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create expense failed: {:?}", res);

    // 5. Inspect cash day summary
    let (res, _) = dispatch_cmd(
        &mut conn,
        "cash_days.get_summary",
        json!({ "business_date": today }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get cash day summary failed: {:?}", res);
    let summary = &res["data"];
    assert_eq!(summary["opening_cash_poisha"], 0);
    assert_eq!(summary["cash_in_poisha"], 300000);
    assert_eq!(summary["cash_out_poisha"], 50000);
    assert_eq!(summary["expected_closing_poisha"], 250000); // 0 + 300000 - 50000 = 250000
    assert_eq!(summary["is_closed"], false);

    // Digital breakdown should show bKash
    let dig = summary["digital_collections"].as_array().unwrap();
    assert_eq!(dig.len(), 1);
    assert_eq!(dig[0]["method"], "bkash");
    assert_eq!(dig[0]["total_amount_poisha"], 150000);

    // Collector breakdown
    let collectors = summary["collector_breakdown"].as_array().unwrap();
    assert!(!collectors.is_empty());

    // 6. Close cash day: count drawer has exactly ৳2,500 (250000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "cash_days.close",
        json!({
            "input": {
                "business_date": today,
                "counted_closing_poisha": 250000,
                "note": "Evening shift close - all reconciled"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "close cash day failed: {:?}", res);
    assert_eq!(res["data"]["is_closed"], true);
    assert_eq!(res["data"]["difference_poisha"], 0);

    // 7. Verify Invariant 5 (Retroactive Mutation Lock):
    // Trying to add an expense on closed date must fail!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "expenses.create",
        json!({
            "expense": {
                "category_id": cat_id,
                "amount_poisha": 20000,
                "method": "cash",
                "payee": "Late Bill",
                "business_date": today
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        res["ok"], false,
        "mutation on closed cash day must be rejected: {:?}",
        res
    );

    // 8. Next business day balance carry-forward
    let tomorrow = {
        let d: jiff::civil::Date = today.parse().expect("valid date");
        d.checked_add(jiff::Span::new().days(1))
            .expect("valid tomorrow")
            .to_string()
    };
    let (res, _) = dispatch_cmd(
        &mut conn,
        "cash_days.get_or_open_today",
        json!({ "business_date": tomorrow }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(
        res["data"]["opening_cash_poisha"], 250000,
        "Tomorrow must carry forward today's counted closing balance!"
    );
}
