use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_templates.db");
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
                "present_address": "Dhaka Bangladesh",
                "phone": student_phone,
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Guardian Name",
                        "relationship": "father",
                        "phone": guardian_phone,
                        "email": format!("guardian{}@example.com", seq)
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
fn test_text_analysis_gsm_vs_unicode() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1: Short English message (GSM single segment)
    let short_en = "Hello guardian, your child has arrived safely at Coachify Academy.";
    let (res1, _) = dispatch_cmd(
        &mut conn,
        "comms.analyze_text",
        json!({ "text": short_en, "channel": "sms" }),
        Some(&token),
        None,
    );
    assert_eq!(res1["ok"], true);
    assert_eq!(res1["data"]["encoding"], "gsm");
    assert_eq!(res1["data"]["segment_count"], 1);
    assert_eq!(res1["data"]["units_per_segment"], 160);

    // 2: Long English message (> 160 chars, GSM multipart 153 chars per segment)
    let long_en = "This is a detailed announcement for all guardians regarding upcoming final semester examinations scheduled for next month. Please check the portal for full details and schedules.";
    assert!(long_en.len() > 160);
    let (res2, _) = dispatch_cmd(
        &mut conn,
        "comms.analyze_text",
        json!({ "text": long_en, "channel": "sms" }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], true);
    assert_eq!(res2["data"]["encoding"], "gsm");
    assert_eq!(res2["data"]["segment_count"], 2);
    assert_eq!(res2["data"]["units_per_segment"], 153);

    // 3: Bengali message (Unicode single segment <= 70 chars)
    let short_bn = "সম্মানিত অভিভাবক, আপনার সন্তান ক্লাসে উপস্থিত হয়েছে।";
    let (res3, _) = dispatch_cmd(
        &mut conn,
        "comms.analyze_text",
        json!({ "text": short_bn, "channel": "sms" }),
        Some(&token),
        None,
    );
    assert_eq!(res3["ok"], true);
    assert_eq!(res3["data"]["encoding"], "unicode");
    assert_eq!(res3["data"]["segment_count"], 1);
    assert_eq!(res3["data"]["units_per_segment"], 70);

    // 4: Long Bengali message (> 70 chars, Unicode multipart 67 chars per segment)
    let long_bn = "সম্মানিত অভিভাবক, আপনার সন্তানের বকেয়া ফি আগামী সপ্তাহের মধ্যে পরিশোধ করার জন্য বিনীত অনুরোধ করা হচ্ছে। বিস্তারিত জানতে অফিসে যোগাযোগ করুন। কোচিং কর্তৃপক্ষ।";
    assert!(long_bn.chars().count() > 70);
    let (res4, _) = dispatch_cmd(
        &mut conn,
        "comms.analyze_text",
        json!({ "text": long_bn, "channel": "sms" }),
        Some(&token),
        None,
    );
    assert_eq!(res4["ok"], true);
    assert_eq!(res4["data"]["encoding"], "unicode");
    assert!(res4["data"]["segment_count"].as_u64().unwrap() >= 2);
    assert_eq!(res4["data"]["units_per_segment"], 67);

    // 5: WhatsApp message (UTF8, single segment regardless of length)
    let (res5, _) = dispatch_cmd(
        &mut conn,
        "comms.analyze_text",
        json!({ "text": long_bn, "channel": "whatsapp" }),
        Some(&token),
        None,
    );
    assert_eq!(res5["ok"], true);
    assert_eq!(res5["data"]["encoding"], "utf8");
    assert_eq!(res5["data"]["segment_count"], 1);
}

#[test]
fn test_template_crud_and_system_template_protection() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1: List system templates: should contain default seeded templates
    let (list_res, _) = dispatch_cmd(
        &mut conn,
        "comms.list_templates",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(list_res["ok"], true);
    let templates = list_res["data"].as_array().unwrap();
    assert!(templates.len() >= 6);

    let sys_tmpl = templates
        .iter()
        .find(|t| t["id"] == "tmpl-payment-sms-en")
        .unwrap();
    assert_eq!(sys_tmpl["is_system"], true);

    // 2: Attempt to delete system template: must be rejected with template_is_system
    let (del_sys, _) = dispatch_cmd(
        &mut conn,
        "comms.delete_template",
        json!({ "id": "tmpl-payment-sms-en" }),
        Some(&token),
        None,
    );
    assert_eq!(del_sys["ok"], false);
    assert_eq!(del_sys["error"]["code"], "template_is_system");

    // 3: Create custom template
    let (save_res, _) = dispatch_cmd(
        &mut conn,
        "comms.save_template",
        json!({
            "key": "exam_reminder",
            "channel": "sms",
            "language": "en",
            "title": "Model Test Notice",
            "body": "Dear Guardian, {student_name} has Model Test on {date}. Best regards, Coachify."
        }),
        Some(&token),
        None,
    );
    assert_eq!(save_res["ok"], true);
    let custom_id = save_res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(save_res["data"]["is_system"], false);

    // 4: Update custom template
    let (update_res, _) = dispatch_cmd(
        &mut conn,
        "comms.save_template",
        json!({
            "id": &custom_id,
            "key": "exam_reminder",
            "channel": "sms",
            "language": "en",
            "title": "Model Test Notice Revised",
            "body": "Updated message body for {student_name}."
        }),
        Some(&token),
        None,
    );
    assert_eq!(update_res["ok"], true);
    assert_eq!(update_res["data"]["title"], "Model Test Notice Revised");

    // 5: Delete custom template: succeeds
    let (del_res, _) = dispatch_cmd(
        &mut conn,
        "comms.delete_template",
        json!({ "id": &custom_id }),
        Some(&token),
        None,
    );
    assert_eq!(del_res["ok"], true);

    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM message_templates WHERE id = ?1",
            params![custom_id],
            |_| Ok(()),
        )
        .is_ok();
    assert!(!exists);
}

#[test]
fn test_payment_received_automation_trigger_hook() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Fee Head and Plan
    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Monthly Tuition Fee",
                "kind": "tuition",
                "default_amount_poisha": 200000
            }
        }),
        Some(&token),
        None,
    );
    let fee_head_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Paying Student");

    // Create manual charge of Tk 2000
    let (ch_res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": &s1,
                "fee_head_id": &fee_head_id,
                "amount_poisha": 200000,
                "due_date": "2026-03-31",
                "period": "2026-03"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(ch_res["ok"], true);

    // Collect payment of Tk 2000
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": &s1,
                "amount_poisha": 200000,
                "method": "cash",
                "idempotency_key": "pay-hook-001"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true);
    let payment_id = pay_res["data"]["id"].as_str().unwrap().to_string();

    // Verify outbox has payment_received receipt queued
    let sms_dedupe = format!("payment:{}:receipt_sms", payment_id);
    let (sms_status, sms_body): (String, String) = conn
        .query_row(
            "SELECT status, body FROM outbox WHERE dedupe_key = ?1",
            params![sms_dedupe],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("receipt sms must be queued");
    assert_eq!(sms_status, "queued");
    assert!(sms_body.contains("Paying Student"));
    assert!(sms_body.contains("2000.00"));

    // Void the payment: verify trigger trg_payments_cancel_outbox automatically cancels queued receipt!
    let (void_res, _) = dispatch_cmd(
        &mut conn,
        "payments.void",
        json!({
            "payment_id": &payment_id,
            "reason": "Wrong amount entered by cashier"
        }),
        Some(&token),
        None,
    );
    assert_eq!(void_res["ok"], true);

    let cancelled_status: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE dedupe_key = ?1",
            params![sms_dedupe],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(cancelled_status, "cancelled");
}

#[test]
fn test_due_reminders_candidates_and_batch_enqueue() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Fee Head
    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Term Fee",
                "kind": "tuition",
                "default_amount_poisha": 150000
            }
        }),
        Some(&token),
        None,
    );
    let fee_head_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    // Setup Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Due Reminder Program" } }),
        Some(&token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Batch Due", "capacity": 20 }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Due Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Due Student 2");

    for s in [&s1, &s2] {
        let (enr, _) = dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({
                "enrollment": {
                    "student_id": s,
                    "batch_id": &batch_id,
                    "start_date": "2026-03-01"
                }
            }),
            Some(&token),
            None,
        );
        assert_eq!(enr["ok"], true);
    }

    // Charge student 1: Tk 1500
    let (c1, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": &s1,
                "fee_head_id": &fee_head_id,
                "amount_poisha": 150000,
                "due_date": "2026-03-31",
                "period": "2026-03"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(c1["ok"], true);

    // Charge student 2: Tk 300 (below threshold)
    let (c2, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": &s2,
                "fee_head_id": &fee_head_id,
                "amount_poisha": 30000,
                "due_date": "2026-03-31",
                "period": "2026-03"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(c2["ok"], true);

    // 1: Query candidates with min_due_poisha: 100000 (Tk 1000)
    let (cand_res, _) = dispatch_cmd(
        &mut conn,
        "comms.get_due_reminder_candidates",
        json!({ "min_due_poisha": 100000, "period": "2026-03" }),
        Some(&token),
        None,
    );
    assert_eq!(cand_res["ok"], true);
    let candidates = cand_res["data"].as_array().unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["student_id"], s1);
    assert_eq!(candidates[0]["due_amount_poisha"], 150000);
    assert_eq!(candidates[0]["already_reminded_sms"], false);

    // 2: Enqueue reminder for s1
    let (enq_res1, _) = dispatch_cmd(
        &mut conn,
        "comms.enqueue_due_reminders",
        json!({
            "student_ids": [s1],
            "channel": "sms",
            "period": "2026-03"
        }),
        Some(&token),
        None,
    );
    assert_eq!(enq_res1["ok"], true);
    assert_eq!(enq_res1["data"]["enqueued_count"], 1);
    assert_eq!(enq_res1["data"]["skipped_count"], 0);

    // 3: Query candidates again: s1 now has already_reminded_sms true
    let (cand_res2, _) = dispatch_cmd(
        &mut conn,
        "comms.get_due_reminder_candidates",
        json!({ "min_due_poisha": 100000, "period": "2026-03" }),
        Some(&token),
        None,
    );
    assert_eq!(cand_res2["ok"], true);
    assert_eq!(cand_res2["data"][0]["already_reminded_sms"], true);

    // 4: Attempt to enqueue reminder again for s1: skipped due to dedupe uniqueness
    let (enq_res2, _) = dispatch_cmd(
        &mut conn,
        "comms.enqueue_due_reminders",
        json!({
            "student_ids": [s1],
            "channel": "sms",
            "period": "2026-03"
        }),
        Some(&token),
        None,
    );
    assert_eq!(enq_res2["ok"], true);
    assert_eq!(enq_res2["data"]["enqueued_count"], 0);
    assert_eq!(enq_res2["data"]["skipped_count"], 1);
}

#[test]
fn test_batch_broadcast_preview_and_send() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Broadcast Program" } }),
        Some(&token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Broadcast Batch", "capacity": 20 }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Broadcast Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Broadcast Student 2");

    for s in [&s1, &s2] {
        let (enr, _) = dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({
                "enrollment": {
                    "student_id": s,
                    "batch_id": &batch_id,
                    "start_date": "2026-03-01"
                }
            }),
            Some(&token),
            None,
        );
        assert_eq!(enr["ok"], true);
    }

    // 1: Preview broadcast
    let (prev_res, _) = dispatch_cmd(
        &mut conn,
        "comms.preview_batch_broadcast",
        json!({
            "batch_id": &batch_id,
            "channel": "sms",
            "custom_body": "Important notice for {student_name}: Class timings rescheduled to 9 AM tomorrow."
        }),
        Some(&token),
        None,
    );
    assert_eq!(prev_res["ok"], true);
    let prev = &prev_res["data"];
    let ready = prev["ready_recipients"].as_array().unwrap();
    assert_eq!(ready.len(), 2);
    assert_eq!(prev["segments_per_message"], 1);
    assert!(prev["estimated_cost"].as_f64().unwrap() > 0.0);

    // 2: Send batch broadcast
    let (send_res, _) = dispatch_cmd(
        &mut conn,
        "comms.send_batch_broadcast",
        json!({
            "batch_id": &batch_id,
            "channel": "sms",
            "recipient_student_ids": [s1, s2],
            "custom_body": "Important notice for {student_name}: Class timings rescheduled to 9 AM tomorrow."
        }),
        Some(&token),
        None,
    );
    assert_eq!(send_res["ok"], true);
    assert_eq!(send_res["data"]["enqueued_count"], 2);

    // Verify messages in outbox
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM outbox WHERE body LIKE '%Class timings rescheduled%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
}
