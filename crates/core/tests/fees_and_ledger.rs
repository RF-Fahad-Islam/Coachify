use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::fees::calculate_concession;
use coachify_core::ledger;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_fees_ledger.db");
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(20000);

fn create_student_helper(conn: &mut rusqlite::Connection, token: &str, name: &str) -> String {
    let seq = STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let student_phone = format!("0171{:07}", seq);
    let guardian_phone = format!("0181{:07}", seq);
    let (res, _) = dispatch_cmd(
        conn,
        "students.create",
        json!({
            "student": {
                "name": name,
                "gender": "male",
                "dob": "2008-01-01",
                "phone": student_phone,
                "present_address": "Dhaka, Bangladesh",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Guardian",
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

fn create_academic_setup_helper(conn: &mut rusqlite::Connection, token: &str) -> (String, String) {
    let (res, _) = dispatch_cmd(
        conn,
        "programs.create",
        json!({
            "program": {
                "name": "Class 10 Science",
                "code": format!("C10SCI-{}", STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
                "class_level": "Class 10"
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create program failed: {:?}", res);
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Morning Batch A",
                "code": format!("MBA-{}", STUDENT_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
                "max_capacity": 50,
                "start_date": "2026-01-01"
            }
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true, "create batch failed: {:?}", res);
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    (program_id, batch_id)
}

// -----------------------------------------------------------------------------
// AC-1: Fee Heads Catalog CRUD & Referencing Protection
// -----------------------------------------------------------------------------

#[test]
fn test_fee_heads_crud_and_referential_integrity() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create fee head
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Monthly Tuition",
                "name_bn": "মাসিক বেতন",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "failed to create fee head: {:?}", res);
    let fee_head_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["name"], "Monthly Tuition");
    assert_eq!(res["data"]["kind"], "tuition");

    // 2. Case-insensitive duplicate name rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "monthly tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "duplicate_name");

    // 3. Invalid kind rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Random Fee",
                "kind": "invalid_kind_here"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "invalid_kind");

    // 4. Update fee head
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.update",
        json!({
            "id": fee_head_id,
            "head": {
                "name": "Regular Tuition Fee"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["name"], "Regular Tuition Fee");

    // 5. Create fee plan referencing this fee head
    let (prog_id, _) = create_academic_setup_helper(&mut conn, &token);
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": 300000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let plan_id = res["data"]["id"].as_str().unwrap().to_string();

    // 6. Attempt to delete fee head -> rejected because referenced
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.delete",
        json!({ "id": fee_head_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "fee_head_referenced");

    // 7. Delete fee plan -> now fee head deletion succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.delete",
        json!({ "id": plan_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.delete",
        json!({ "id": fee_head_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
}

// -----------------------------------------------------------------------------
// AC-2: Fee Plans Scoping, Date Ranges, & Overlap Prevention
// -----------------------------------------------------------------------------

#[test]
fn test_fee_plans_scoping_and_overlap_prevention() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (prog_id, batch_id) = create_academic_setup_helper(&mut conn, &token);

    // Create fee head
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Coaching Tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    let head_id = res["data"]["id"].as_str().unwrap().to_string();

    // 1. Invalid scope: both program and batch provided
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "batch_id": batch_id,
                "fee_head_id": head_id,
                "amount_poisha": 400000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "invalid_scope");

    // 2. Invalid date range: effective_to < effective_from
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 400000,
                "frequency": "monthly",
                "effective_from": "2026-06-01",
                "effective_to": "2026-05-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "invalid_date_range");

    // 3. Create initial plan: Jan 1 to Jun 30, 2026
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 400000,
                "frequency": "monthly",
                "effective_from": "2026-01-01",
                "effective_to": "2026-06-30"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 4. Overlapping plan for same program & fee head: May 1 to Dec 31 -> rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 450000,
                "frequency": "monthly",
                "effective_from": "2026-05-01",
                "effective_to": "2026-12-31"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "overlapping_plan");

    // 5. Non-overlapping plan: Jul 1 to Dec 31 -> succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 450000,
                "frequency": "monthly",
                "effective_from": "2026-07-01",
                "effective_to": "2026-12-31"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
}

// -----------------------------------------------------------------------------
// AC-3: Concession Math, Half-Up Rounding, Clamping & Hierarchy
// -----------------------------------------------------------------------------

#[test]
fn test_concession_math_and_hierarchy() {
    // 1. Math unit assertions
    // 15% on 10,000 BDT (1,000,000 poisha)
    assert_eq!(calculate_concession(1000000, "percent", 1500), 150000);

    // 33.33% on 1000 poisha: (1000 * 3333 + 5000) / 10000 = (3333000 + 5000) / 10000 = 333 poisha
    assert_eq!(calculate_concession(1000, "percent", 3333), 333);

    // 33.33% on 100 poisha: (100 * 3333 + 5000) / 10000 = 338300 / 10000 = 33 poisha
    assert_eq!(calculate_concession(100, "percent", 3333), 33);

    // Half-up boundary test: 50% on 15 poisha -> (15 * 5000 + 5000) / 10000 = 80000 / 10000 = 8 poisha
    assert_eq!(calculate_concession(15, "percent", 5000), 8);

    // Fixed discount with gross clamping
    assert_eq!(calculate_concession(50000, "fixed", 20000), 20000);
    assert_eq!(calculate_concession(50000, "fixed", 80000), 50000); // clamped to gross

    // 2. Integration: Concession hierarchy (batch override > general)
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (prog_id, batch_id) = create_academic_setup_helper(&mut conn, &token);
    let student_id = create_student_helper(&mut conn, &token, "Rahim Ahmed");

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "General Tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    let head_id = res["data"]["id"].as_str().unwrap().to_string();

    // General concession: 10% (1000 bps) across all batches
    let (res, _) = dispatch_cmd(
        &mut conn,
        "concessions.create",
        json!({
            "concession": {
                "student_id": student_id,
                "fee_head_id": head_id,
                "type": "percent",
                "value": 1000,
                "reason": "General merit scholarship"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Batch-specific override: 25% (2500 bps) for this specific batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "concessions.create",
        json!({
            "concession": {
                "student_id": student_id,
                "batch_id": batch_id,
                "fee_head_id": head_id,
                "type": "percent",
                "value": 2500,
                "reason": "Special morning batch grant"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Duplicate active concession for same batch rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "concessions.create",
        json!({
            "concession": {
                "student_id": student_id,
                "batch_id": batch_id,
                "fee_head_id": head_id,
                "type": "fixed",
                "value": 50000,
                "reason": "Duplicate attempt"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "concession_conflict");

    // Fee plan: 4,000 BDT (400,000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 400000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Enroll student in batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": student_id,
                "batch_id": batch_id,
                "start_date": "2026-09-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Generate monthly charges for 2026-09
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.generate_monthly",
        json!({ "period": "2026-09" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["generated_count"], 1);
    // 25% of 400,000 is 100,000 concession poisha (batch override wins over 10% general)
    assert_eq!(res["data"]["total_gross_poisha"], 400000);
    assert_eq!(res["data"]["total_concession_poisha"], 100000);
    assert_eq!(res["data"]["total_net_poisha"], 300000);
}

// -----------------------------------------------------------------------------
// AC-4 & AC-5: Monthly Fee Generation, Idempotency & Mid-Month Enrollment
// -----------------------------------------------------------------------------

#[test]
fn test_monthly_generation_idempotency_and_mid_month_rule() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (prog_id, batch_id) = create_academic_setup_helper(&mut conn, &token);

    // Student 1: regular start
    let s1 = create_student_helper(&mut conn, &token, "S1 Student");
    // Student 2: mid-month start on 2026-09-18
    let s2 = create_student_helper(&mut conn, &token, "S2 MidMonth");

    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s1,
                "batch_id": batch_id,
                "start_date": "2026-09-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // AC-5: mid-month enrollment on 18th
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s2,
                "batch_id": batch_id,
                "start_date": "2026-09-18"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Fee head & plan
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Standard Tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    let head_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": head_id,
                "amount_poisha": 500000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // First generation run
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.generate_monthly",
        json!({ "period": "2026-09" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["generated_count"], 2);
    // Both students get full 500,000 (AC-5 mid-month full tuition rule)
    assert_eq!(res["data"]["total_net_poisha"], 1000000);

    // Second generation run (AC-4 Idempotency): should generate 0 and skip 2
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.generate_monthly",
        json!({ "period": "2026-09" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["generated_count"], 0);
    assert_eq!(res["data"]["skipped_count"], 2);

    // Void S1's charge
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.list_by_student",
        json!({ "student_id": s1 }),
        Some(&token),
        None,
    );
    let s1_charge_id = res["data"][0]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.void",
        json!({
            "charge_id": s1_charge_id,
            "reason": "Test voiding"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Third generation run: voided charge must NOT be recreated
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.generate_monthly",
        json!({ "period": "2026-09" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["generated_count"], 0);
    assert_eq!(res["data"]["skipped_count"], 2);
}

// -----------------------------------------------------------------------------
// AC-6 & AC-8: Payment Receipts, Atomic Monotonic Sequence & Advance Credit Allocation
// -----------------------------------------------------------------------------

#[test]
fn test_payments_advance_credit_and_receipt_sequence() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (prog_id, batch_id) = create_academic_setup_helper(&mut conn, &token);
    let student_id = create_student_helper(&mut conn, &token, "Karim Ullah");

    // Enroll student
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": student_id,
                "batch_id": batch_id,
                "start_date": "2026-09-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Fee heads: Tuition (priority 1) and Exam (priority 3)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Class Tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    let tuition_head_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Midterm Exam",
                "kind": "exam"
            }
        }),
        Some(&token),
        None,
    );
    let exam_head_id = res["data"]["id"].as_str().unwrap().to_string();

    // 1. Advance payment of 7,000 BDT (700,000 poisha) BEFORE any charges exist
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 700000,
                "method": "bkash",
                "idempotency_key": "pay-adv-001"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "advance payment failed: {:?}", res);
    let receipt1 = res["data"]["receipt_no"].as_str().unwrap().to_string();
    assert_eq!(receipt1, "REC-2026-0001");
    assert_eq!(res["data"]["allocated_amount_poisha"], 0);
    assert_eq!(res["data"]["unallocated_amount_poisha"], 700000);

    // Check balance: advance credit means student is in negative balance (-700,000)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["balance_poisha"], -700000);

    // 2. Set fee plans: Tuition 500,000, Exam 100,000
    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": tuition_head_id,
                "amount_poisha": 500000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_plans.create",
        json!({
            "plan": {
                "program_id": prog_id,
                "fee_head_id": exam_head_id,
                "amount_poisha": 100000,
                "frequency": "monthly",
                "effective_from": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 3. Generate monthly charges for 2026-09
    // Gross charges = 500,000 + 100,000 = 600,000
    // AC-6: Auto-credit allocation engine automatically allocates the 700,000 credit!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.generate_monthly",
        json!({ "period": "2026-09" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["generated_count"], 2);
    assert_eq!(res["data"]["allocated_credit_poisha"], 600000);

    // Check student summary:
    // Total charged: 600,000
    // Total paid: 700,000
    // Balance: -100,000 (still has 100,000 poisha advance credit)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["total_charged_poisha"], 600000);
    assert_eq!(res["data"]["total_paid_poisha"], 700000);
    assert_eq!(res["data"]["balance_poisha"], -100000);

    // Verify Invariants!
    ledger::verify_all_ledger_invariants(&conn).expect("All invariants must hold");
}

// -----------------------------------------------------------------------------
// AC-7: Adjustments, Waivers, Refunds & Trigger Defense-in-Depth
// -----------------------------------------------------------------------------

#[test]
fn test_adjustments_refunds_and_defense_triggers() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (_prog_id, _) = create_academic_setup_helper(&mut conn, &token);
    let student_id = create_student_helper(&mut conn, &token, "Mitu Begum");

    let (res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Lab Fee",
                "kind": "material"
            }
        }),
        Some(&token),
        None,
    );
    let head_id = res["data"]["id"].as_str().unwrap().to_string();

    // 1. Create manual charge: 2,000 BDT (200,000 poisha)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": head_id,
                "amount_poisha": 200000,
                "due_date": "2026-09-15"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let charge_id = res["data"]["id"].as_str().unwrap().to_string();

    // 2. Waiver ceiling check: trying to waive 250,000 on a 200,000 charge -> rejected!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "adjustments.create",
        json!({
            "adjustment": {
                "student_id": student_id,
                "charge_id": charge_id,
                "type": "waiver",
                "amount_poisha": 250000,
                "reason": "Excessive waiver test"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "waiver_exceeds_unallocated");

    // 3. Valid waiver: 50,000 poisha -> succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "adjustments.create",
        json!({
            "adjustment": {
                "student_id": student_id,
                "charge_id": charge_id,
                "type": "waiver",
                "amount_poisha": 50000,
                "reason": "Special discount"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Remaining due is now 150,000 poisha
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 150000);

    // 4. Pay remaining due: 150,000 poisha
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 150000,
                "method": "cash",
                "idempotency_key": "pay-mitu-001"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let payment_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["allocated_amount_poisha"], 150000);
    assert_eq!(res["data"]["unallocated_amount_poisha"], 0);

    // 5. Try to void charge that has active allocations -> rejected!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.void",
        json!({
            "charge_id": charge_id,
            "reason": "Should fail because allocated"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "charge_has_allocations");

    // 6. Try to refund payment that has 0 unallocated funds -> rejected!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "refunds.create",
        json!({
            "refund": {
                "payment_id": payment_id,
                "amount_poisha": 50000,
                "method": "cash",
                "reason": "Should fail because fully allocated"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "refund_exceeds_unallocated");

    // 7. Void payment -> frees charge allocations!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.void",
        json!({
            "payment_id": payment_id,
            "reason": "Mistaken entry"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Charge is now unpaid again! Remaining due = 150,000. Balance = 150,000.
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 150000);

    // Now charge can be voided because allocations are inactive (from voided payment)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.void",
        json!({
            "charge_id": charge_id,
            "reason": "Cancelled fee"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Balance is now -50,000 poisha because the 50,000 waiver remains on the student's ledger
    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], -50000);

    // Apply a fine of 50,000 poisha to balance the ledger back to 0
    let (res, _) = dispatch_cmd(
        &mut conn,
        "adjustments.create",
        json!({
            "adjustment": {
                "student_id": student_id,
                "type": "fine",
                "amount_poisha": 50000,
                "reason": "Reversing waiver on voided charge"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "ledger.get_student_summary",
        json!({ "student_id": student_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["data"]["balance_poisha"], 0);

    ledger::verify_all_ledger_invariants(&conn).expect("All invariants must hold");
}
