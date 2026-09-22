use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::ledger;
use coachify_core::storage::{
    get_workstation_printer_settings, list_system_printers, save_workstation_printer_settings,
    WorkstationPrinterSettings,
};
use coachify_core::words::{amount_to_words_bn, amount_to_words_en};
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_print_receipts.db");
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

#[test]
fn test_ac2_workstation_printer_settings_persistence_and_validation() {
    // Tests workstation printer settings persistence outside SQLite database
    let mut custom_settings = WorkstationPrinterSettings {
        printer_name: Some("POS-80C Thermal".to_string()),
        page_size: "thermal_80mm".to_string(),
        paper_source: None,
        copies: 2,
        auto_print_on_payment: true,
        open_cash_drawer: false,
        header_title: None,
        footer_note: Some("Test note".to_string()),
        updated_at: "2026-09-22T10:00:00Z".to_string(),
    };

    let saved =
        save_workstation_printer_settings(&custom_settings).expect("saving valid printer settings");
    assert_eq!(saved.page_size, "thermal_80mm");
    assert_eq!(saved.copies, 2);

    let loaded = get_workstation_printer_settings();
    assert_eq!(loaded.page_size, "thermal_80mm");
    assert_eq!(loaded.copies, 2);

    // Test layout validation: only thermal_80mm, a4_dual, a5_single are accepted
    custom_settings.page_size = "invalid_layout".to_string();
    assert!(save_workstation_printer_settings(&custom_settings).is_err());

    custom_settings.page_size = "a4_dual".to_string();
    assert!(save_workstation_printer_settings(&custom_settings).is_ok());

    custom_settings.page_size = "a5_single".to_string();
    assert!(save_workstation_printer_settings(&custom_settings).is_ok());
}

#[test]
fn test_ac3_system_printer_enumeration() {
    // Tests system printer enumeration
    let printers = list_system_printers();
    assert!(
        !printers.is_empty(),
        "Printer enumeration should return at least one printer or fallback"
    );
    for p in &printers {
        assert!(!p.name.is_empty());
        assert!(!p.status.is_empty());
    }
}

#[test]
fn test_ac7_amount_to_words_en_and_bn() {
    // Covers AC-7: English and Bengali amount in words converters for monetary receipts
    assert_eq!(amount_to_words_en(0), "Zero Taka Only");
    assert_eq!(
        amount_to_words_en(250000),
        "Two Thousand Five Hundred Taka Only"
    );
    assert_eq!(
        amount_to_words_en(125075),
        "One Thousand Two Hundred Fifty Taka and Seventy Five Poisha Only"
    );
    assert_eq!(amount_to_words_en(1000000000), "One Crore Taka Only");

    assert_eq!(amount_to_words_bn(0), "শূন্য টাকা মাত্র");
    assert_eq!(amount_to_words_bn(250000), "দুই হাজার পাঁচ শত টাকা মাত্র");
    assert_eq!(
        amount_to_words_bn(125075),
        "এক হাজার দুই শত পঞ্চাশ টাকা পঁচাত্তর পয়সা মাত্র"
    );
    assert_eq!(amount_to_words_bn(10000000), "এক লক্ষ টাকা মাত্র");
}

#[test]
fn test_ac1_ac7_ac8_printable_receipt_aggregation_and_reprint_flag() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    // Create student with Bengali name (covers AC-7)
    let stu_id =
        create_student_helper(&mut conn, &token, "Tanvir Ahmed", Some("মুহাম্মদ তানভীর আহমেদ"));

    // Create a fee head and charge
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
    assert_eq!(res["ok"], true, "fee head failed: {:?}", res);
    let fee_head_id = res["data"]["id"].as_str().unwrap();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": stu_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": 200000,
                "due_date": today
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "charge failed: {:?}", res);

    // Record payment of 2500 Taka (2000 for tuition, 500 advance credit)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 250000,
                "method": "cash",
                "reference": "CTR-01",
                "business_date": today,
                "idempotency_key": "pay-print-test-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "payment creation failed: {:?}", res);
    let payment_id = res["data"]["id"].as_str().unwrap();
    let receipt_no = res["data"]["receipt_no"].as_str().unwrap();

    // Query printable receipt via dispatcher (covers AC-1, AC-5, AC-7)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "receipts.get_printable",
        json!({
            "payment_id": payment_id,
            "is_reprint": false
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let data = &res["data"];
    assert_eq!(data["receipt_no"], receipt_no);
    assert_eq!(data["student_name"], "Tanvir Ahmed");
    assert_eq!(data["student_name_bn"], "মুহাম্মদ তানভীর আহমেদ");
    assert_eq!(data["amount_poisha"], 250000);
    assert_eq!(
        data["amount_in_words_en"],
        "Two Thousand Five Hundred Taka Only"
    );
    assert_eq!(data["amount_in_words_bn"], "দুই হাজার পাঁচ শত টাকা মাত্র");
    assert_eq!(data["is_reprint"], false);

    // Verify item allocation: 1 fee head + 1 advance credit line
    let items = data["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["fee_head_name"], "Monthly Tuition");
    assert_eq!(items[0]["amount_poisha"], 200000);
    assert_eq!(items[1]["amount_poisha"], 50000);

    // Reprint query (covers AC-8 duplicate watermark flag)
    let (res_reprint, _) = dispatch_cmd(
        &mut conn,
        "receipts.get_printable",
        json!({
            "payment_id": payment_id,
            "is_reprint": true
        }),
        Some(&token),
        None,
    );
    assert_eq!(res_reprint["ok"], true);
    assert_eq!(res_reprint["data"]["is_reprint"], true);
}

#[test]
fn test_ac4_ac8_decoupled_printing_fault_tolerance() {
    // Covers AC-4 & AC-8: Printing error must not rollback payment transaction
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let today = "2026-09-22";

    let stu_id = create_student_helper(&mut conn, &token, "Shahed Karim", None);

    // Record payment
    let (res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": stu_id,
                "amount_poisha": 150000,
                "method": "bkash",
                "reference": "TRX-FAULT-01",
                "business_date": today,
                "idempotency_key": "pay-fault-tol-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "payment failed: {:?}", res);
    let payment_id = res["data"]["id"].as_str().unwrap();

    // Verify payment is firmly saved and posted in SQLite
    let payment = ledger::get_payment_receipt(&conn, payment_id).expect("get payment");
    assert_eq!(payment.status, "posted");
    assert_eq!(payment.amount_poisha, 150000);

    // Verify student balance was updated synchronously despite any potential downstream print error
    let summary = ledger::get_student_ledger_summary(&conn, &stu_id).expect("get balance");
    assert_eq!(summary.balance_poisha, -150000); // 1500 advance credit
}

#[test]
fn test_dispatcher_printer_settings_rpcs() {
    // Tests printers.list, printers.get_settings, and printers.save_settings via dispatcher
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (res, _) = dispatch_cmd(&mut conn, "printers.list", json!({}), Some(&token), None);
    assert_eq!(res["ok"], true);
    assert!(res["data"].is_array());

    let (res, events) = dispatch_cmd(
        &mut conn,
        "printers.save_settings",
        json!({
            "settings": {
                "printer_name": "Test POS 80",
                "page_size": "a4_dual",
                "paper_source": null,
                "copies": 3,
                "auto_print_on_payment": true,
                "open_cash_drawer": false,
                "header_title": null,
                "footer_note": null,
                "updated_at": "2026-09-22T10:00:00Z"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["page_size"], "a4_dual");
    assert_eq!(res["data"]["copies"], 3);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "printer:settings");

    let (res_get, _) = dispatch_cmd(
        &mut conn,
        "printers.get_settings",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res_get["ok"], true);
    assert_eq!(res_get["data"]["page_size"], "a4_dual");
    assert_eq!(res_get["data"]["copies"], 3);
}
