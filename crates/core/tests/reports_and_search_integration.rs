use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_reports_search.db");
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

#[test]
fn test_global_search_multi_entity_and_reverse_lookup() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "Higher Secondary",
                "code": "HSC-2026"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(p_res["ok"], true, "create program failed: {:?}", p_res);
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": prog_id,
                "name": "Physics Champions Batch",
                "code": "PHY-CHAMP-01",
                "capacity": 40
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(b_res["ok"], true, "create batch failed: {:?}", b_res);
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // 2. Create Student with English & Bengali name, and Guardian with unique phone
    let (stu_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({
            "student": {
                "name": "Rahim Uddin",
                "name_bn": "রহিম উদ্দিন",
                "gender": "male",
                "dob": "2008-01-01",
                "phone": "01711000111",
                "present_address": "Dhaka, Bangladesh",
                "class_level": "HSC-1st",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Abdul Malek",
                        "name_bn": "আব্দুল মালেক",
                        "relationship": "Father",
                        "phone": "01711998877"
                    }
                }]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        stu_res["ok"], true,
        "student creation failed: {:?}",
        stu_res
    );
    let student_id = stu_res["data"]["id"].as_str().unwrap().to_string();
    let student_code = stu_res["data"]["student_code"]
        .as_str()
        .unwrap()
        .to_string();

    // Enroll student in batch
    let (enr_res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": student_id,
                "batch_id": batch_id,
                "roll": 1
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(enr_res["ok"], true, "enrollment failed: {:?}", enr_res);

    // 3. Create Staff Member
    let (stf_res, _) = dispatch_cmd(
        &mut conn,
        "staff.create",
        json!({
            "staff": {
                "name": "Farhana Akter",
                "phone": "01811223344",
                "designation": "Senior Physics Lecturer"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(stf_res["ok"], true, "staff create failed: {:?}", stf_res);

    // 4. Create Fee Head, Plan, Charge, and Payment
    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Monthly Tuition",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(fh_res["ok"], true, "fee_heads.create failed: {:?}", fh_res);
    let fee_head_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    let (chg_res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": 500000,
                "due_date": "2026-09-10"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        chg_res["ok"], true,
        "charges.create_manual failed: {:?}",
        chg_res
    );

    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 500000,
                "method": "bkash",
                "business_date": "2026-09-12",
                "idempotency_key": "pay-global-search-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true, "payments.create failed: {:?}", pay_res);
    let receipt_no = pay_res["data"]["receipt_no"].as_str().unwrap().to_string();

    // -------------------------------------------------------------------------
    // Execution AC Tests: Global Search
    // -------------------------------------------------------------------------

    // AC-6: Short / empty query (< 2 characters) returns 0 hits in < 5ms without DB query
    let (q_short, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": " " }),
        Some(&token),
        None,
    );
    assert_eq!(q_short["ok"], true);
    assert_eq!(q_short["data"]["total_hits"], 0);

    // AC-1: Search student by English name
    let (q_stu, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "Rahim" }),
        Some(&token),
        None,
    );
    assert_eq!(q_stu["ok"], true);
    let stu_hits = q_stu["data"]["students"].as_array().unwrap();
    assert!(!stu_hits.is_empty());
    assert_eq!(stu_hits[0]["student_code"], student_code);
    assert_eq!(stu_hits[0]["batch_name"], "Physics Champions Batch");

    // AC-4: Search by Bengali Unicode name
    let (q_bn, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "রহিম" }),
        Some(&token),
        None,
    );
    assert_eq!(q_bn["ok"], true);
    let bn_hits = q_bn["data"]["students"].as_array().unwrap();
    assert!(!bn_hits.is_empty(), "Bengali query should match student");
    assert_eq!(bn_hits[0]["name"], "Rahim Uddin");

    // AC-2: Guardian Phone Reverse Lookup (6+ digits)
    let (q_phone, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "01711998877" }),
        Some(&token),
        None,
    );
    assert_eq!(q_phone["ok"], true);
    let phone_hits = q_phone["data"]["students"].as_array().unwrap();
    assert!(!phone_hits.is_empty());
    assert_eq!(phone_hits[0]["name"], "Rahim Uddin");
    let badge = phone_hits[0]["matched_guardian_badge"].as_str().unwrap();
    assert!(badge.contains("Abdul Malek"));
    assert!(badge.contains("01711998877"));

    // AC-3: Receipt Number Fast Matching via idx_payments_search_composite
    let receipt_prefix = &receipt_no[..receipt_no.len().min(8)];
    let (q_rcp, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": receipt_prefix }),
        Some(&token),
        None,
    );
    assert_eq!(q_rcp["ok"], true);
    let rcp_hits = q_rcp["data"]["receipts"].as_array().unwrap();
    assert!(!rcp_hits.is_empty());
    assert_eq!(rcp_hits[0]["receipt_no"], receipt_no);
    assert_eq!(rcp_hits[0]["amount_formatted"], "5000.00");
    assert_eq!(rcp_hits[0]["student_name"], "Rahim Uddin");

    // Batch and Staff search matching
    let (q_batch, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "Champions" }),
        Some(&token),
        None,
    );
    let b_hits = q_batch["data"]["batches"].as_array().unwrap();
    assert!(!b_hits.is_empty());
    assert_eq!(b_hits[0]["name"], "Physics Champions Batch");
    assert_eq!(b_hits[0]["student_count"], 1);

    let (q_staff, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "Farhana" }),
        Some(&token),
        None,
    );
    let stf_hits = q_staff["data"]["staff"].as_array().unwrap();
    assert!(!stf_hits.is_empty());
    assert_eq!(stf_hits[0]["name"], "Farhana Akter");
}

#[test]
fn test_reports_collection_collector_dues_and_expenses() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup student, charge, and payment
    let (s_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({
            "student": {
                "name": "Tamim Iqbal",
                "gender": "male",
                "dob": "2008-01-01",
                "phone": "01711222333",
                "present_address": "Dhaka, Bangladesh",
                "class_level": "Class 10",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Iqbal Hossain",
                        "relationship": "Father",
                        "phone": "01711122334"
                    }
                }]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(s_res["ok"], true, "create student failed: {:?}", s_res);
    let student_id = s_res["data"]["id"].as_str().unwrap().to_string();

    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Coaching Fee",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(fh_res["ok"], true, "fee_heads.create failed: {:?}", fh_res);
    let fh_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    // 1. Charge 1: Overdue by 45 days (31-60 days bucket)
    let _ = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fh_id,
                "amount_poisha": 300000,
                "due_date": "2026-08-01"
            }
        }),
        Some(&token),
        None,
    );

    // 2. Charge 2: Current (0-30 days bucket) - partially paid
    let (chg2_res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fh_id,
                "amount_poisha": 400000,
                "due_date": "2026-09-15"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(chg2_res["ok"], true);

    // Payment of 2000 Taka for Charge 2
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 200000,
                "method": "cash",
                "business_date": "2026-09-16",
                "idempotency_key": "pay-test2-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true, "payments.create failed: {:?}", pay_res);

    // Open cash drawer and record expense
    let (c_res, _) = dispatch_cmd(
        &mut conn,
        "cash_days.get_or_open_today",
        json!({ "business_date": "2026-09-16" }),
        Some(&token),
        None,
    );
    assert_eq!(c_res["ok"], true);

    // Record center expense using default category
    let (cats_res, _) = dispatch_cmd(
        &mut conn,
        "expense_categories.list",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(cats_res["ok"], true);
    let cats = cats_res["data"].as_array().unwrap();
    let cat_id = cats[0]["id"].as_str().unwrap().to_string();
    let cat_name = cats[0]["name"].as_str().unwrap().to_string();

    let (exp_res, _) = dispatch_cmd(
        &mut conn,
        "expenses.create",
        json!({
            "expense": {
                "category_id": cat_id,
                "amount_poisha": 50000,
                "method": "cash",
                "payee": "Paper House",
                "business_date": "2026-09-16"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(exp_res["ok"], true, "expenses.create failed: {:?}", exp_res);

    // -------------------------------------------------------------------------
    // AC-1: Collection Summary
    // -------------------------------------------------------------------------
    let (col_res, _) = dispatch_cmd(
        &mut conn,
        "reports.get_collection_summary",
        json!({
            "start_date": "2026-09-01",
            "end_date": "2026-09-30"
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        col_res["ok"], true,
        "get_collection_summary failed: {:?}",
        col_res
    );
    assert_eq!(col_res["data"]["total_collected_poisha"], 200000);
    assert_eq!(col_res["data"]["total_collected_formatted"], "2000.00");
    assert_eq!(col_res["data"]["payment_count"], 1);

    // AC-2: Collector Summary
    let (coll_res, _) = dispatch_cmd(
        &mut conn,
        "reports.get_collector_summary",
        json!({
            "start_date": "2026-09-01",
            "end_date": "2026-09-30"
        }),
        Some(&token),
        None,
    );
    assert_eq!(coll_res["ok"], true);
    let collectors = coll_res["data"]["collectors"].as_array().unwrap();
    assert_eq!(collectors.len(), 1);
    assert_eq!(collectors[0]["total_collected_formatted"], "2000.00");
    assert_eq!(collectors[0]["cash_collected_formatted"], "2000.00");

    // AC-3: Dues Aging Report with Buckets
    let (due_res, _) = dispatch_cmd(
        &mut conn,
        "reports.get_dues_aging_report",
        json!({
            "as_of_date": "2026-09-23"
        }),
        Some(&token),
        None,
    );
    assert_eq!(due_res["ok"], true);
    let items = due_res["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    let tamim_due = &items[0];
    assert_eq!(tamim_due["name"], "Tamim Iqbal");
    // Charge 1: 3000.00 - 2000.00 paid via FIFO = 1000.00 remaining overdue ~53 days -> days_31_60_poisha
    assert_eq!(tamim_due["days_31_60_poisha"], 100000);
    // Charge 2: 4000.00 remaining overdue ~8 days -> current_bucket_poisha
    assert_eq!(tamim_due["current_bucket_poisha"], 400000);
    assert_eq!(tamim_due["total_due_poisha"], 500000);
    assert_eq!(tamim_due["primary_guardian_phone"], "8801711122334");

    // AC-4: Expenses & Drawer Reconciliation
    let (exp_res, _) = dispatch_cmd(
        &mut conn,
        "reports.get_expenses_summary",
        json!({
            "start_date": "2026-09-01",
            "end_date": "2026-09-30"
        }),
        Some(&token),
        None,
    );
    assert_eq!(exp_res["ok"], true);
    assert_eq!(exp_res["data"]["total_expense_poisha"], 50000);
    assert_eq!(exp_res["data"]["total_expense_formatted"], "500.00");
    let cats = exp_res["data"]["categories"].as_array().unwrap();
    assert_eq!(cats[0]["category_name"], cat_name);
    assert_eq!(cats[0]["amount_formatted"], "500.00");

    // AC-7: UTF-8 BOM CSV Export Engine
    let (csv_res, _) = dispatch_cmd(
        &mut conn,
        "reports.export_csv",
        json!({
            "report_type": "dues_aging",
            "params": {
                "as_of_date": "2026-09-23"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(csv_res["ok"], true);
    let csv_content = csv_res["data"]["csv_content"].as_str().unwrap();
    // Must start with UTF-8 BOM \u{FEFF}
    assert!(
        csv_content.starts_with('\u{FEFF}'),
        "CSV must begin with UTF-8 BOM"
    );
    assert!(csv_content.contains("Tamim Iqbal"));
    assert!(csv_content.contains("5000.00"));
}

#[test]
fn test_attendance_chronic_absenteeism_and_rbac() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "SSC Science",
                "code": "SSC-SCI"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(p_res["ok"], true);
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": prog_id,
                "name": "Batch Alpha",
                "code": "ALPHA",
                "capacity": 40
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(b_res["ok"], true);
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // Create student
    let (stu_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({
            "student": {
                "name": "Nusrat Jahan",
                "gender": "female",
                "dob": "2009-01-01",
                "phone": "01722000111",
                "present_address": "Dhaka, Bangladesh",
                "class_level": "Class 9",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Jahanara Begum",
                        "relationship": "Mother",
                        "phone": "01722334455"
                    }
                }]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(stu_res["ok"], true);
    let student_id = stu_res["data"]["id"].as_str().unwrap().to_string();

    let enr_res = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": student_id,
                "batch_id": batch_id,
                "roll": 5
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(enr_res.0["ok"], true);

    // Record 4 sessions in month 2026-09: 1 present, 3 absent (attendance rate = 25% < 75%)
    let dates = ["2026-09-01", "2026-09-02", "2026-09-03", "2026-09-04"];
    for (i, d) in dates.iter().enumerate() {
        let status = if i == 0 { "present" } else { "absent" };
        let (save_res, _) = dispatch_cmd(
            &mut conn,
            "attendance.save_session",
            json!({
                "batch_id": batch_id,
                "date": d,
                "marks": [{
                    "student_id": student_id,
                    "status": status,
                    "note": null
                }]
            }),
            Some(&token),
            None,
        );
        assert_eq!(save_res["ok"], true);
    }

    // AC-5: Attendance summary & chronic absenteeism
    let (att_res, _) = dispatch_cmd(
        &mut conn,
        "reports.get_attendance_summary",
        json!({
            "batch_id": batch_id,
            "month": "2026-09"
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        att_res["ok"], true,
        "get_attendance_summary failed: {:?}",
        att_res
    );
    assert_eq!(att_res["data"]["chronic_absentee_count"], 1);
    let items = att_res["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "Nusrat Jahan");
    assert_eq!(items[0]["total_sessions"], 4);
    assert_eq!(items[0]["present_count"], 1);
    assert_eq!(items[0]["absent_count"], 3);
    assert_eq!(items[0]["attendance_percentage"], 25.0);
    assert_eq!(items[0]["is_chronic_absentee"], true);

    // -------------------------------------------------------------------------
    // RBAC: Roles & Zero Training Mode Permission Testing
    // -------------------------------------------------------------------------

    let rec_role_id: String = conn
        .query_row(
            "SELECT id FROM roles WHERE name = 'Receptionist'",
            [],
            |r| r.get(0),
        )
        .expect("role must exist");
    let csh_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Cashier'", [], |r| {
            r.get(0)
        })
        .expect("role must exist");
    let adm_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Admin'", [], |r| {
            r.get(0)
        })
        .expect("role must exist");

    // 1. Receptionist User
    let (rec_user, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "receptionist1",
            "display_name": "Front Desk Receptionist",
            "password": "Password123!",
            "role_ids": [rec_role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(rec_user["ok"], true);

    let (rec_login, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({ "username": "receptionist1", "password": "Password123!" }),
        None,
        None,
    );
    assert_eq!(rec_login["ok"], true);
    let rec_token = rec_login["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Receptionist CAN search globally
    let (rec_search, _) = dispatch_cmd(
        &mut conn,
        "search.global",
        json!({ "query": "Nusrat" }),
        Some(&rec_token),
        None,
    );
    assert_eq!(
        rec_search["ok"], true,
        "Receptionist must have search.global permission"
    );

    // Receptionist CANNOT view financial collection summary (reports.financial required)
    let (rec_fin, _) = dispatch_cmd(
        &mut conn,
        "reports.get_collection_summary",
        json!({ "start_date": "2026-09-01", "end_date": "2026-09-30" }),
        Some(&rec_token),
        None,
    );
    assert_eq!(rec_fin["ok"], false);
    assert_eq!(rec_fin["error"]["code"], "forbidden");

    // 2. Cashier User
    let (csh_user, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "cashier1",
            "display_name": "Main Cashier",
            "password": "Password123!",
            "role_ids": [csh_role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(csh_user["ok"], true);

    let (csh_login, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({ "username": "cashier1", "password": "Password123!" }),
        None,
        None,
    );
    let csh_token = csh_login["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Cashier CAN view financial reports
    let (csh_fin, _) = dispatch_cmd(
        &mut conn,
        "reports.get_collection_summary",
        json!({ "start_date": "2026-09-01", "end_date": "2026-09-30" }),
        Some(&csh_token),
        None,
    );
    assert_eq!(
        csh_fin["ok"], true,
        "Cashier must have reports.financial permission"
    );

    // Cashier CANNOT take attendance
    let (csh_att, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": batch_id,
            "date": "2026-09-10",
            "marks": []
        }),
        Some(&csh_token),
        None,
    );
    assert_eq!(csh_att["ok"], false);
    assert_eq!(csh_att["error"]["code"], "forbidden");

    // 3. Admin User
    let (adm_user, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "admin1",
            "display_name": "Center Administrator",
            "password": "Password123!",
            "role_ids": [adm_role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(adm_user["ok"], true);

    let (adm_login, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({ "username": "admin1", "password": "Password123!" }),
        None,
        None,
    );
    let adm_token = adm_login["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Admin CAN view reports and manage students
    let (adm_rep, _) = dispatch_cmd(
        &mut conn,
        "reports.get_attendance_summary",
        json!({ "batch_id": batch_id, "month": "2026-09" }),
        Some(&adm_token),
        None,
    );
    assert_eq!(adm_rep["ok"], true);

    // Admin CANNOT trigger database backups (backup.manage reserved for Owner)
    let (adm_backup, _) = dispatch_cmd(
        &mut conn,
        "backup.create_snapshot",
        json!({ "kind": "manual", "notes": "test" }),
        Some(&adm_token),
        None,
    );
    assert_eq!(adm_backup["ok"], false);
    assert_eq!(adm_backup["error"]["code"], "forbidden");
}
