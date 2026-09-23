use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::printers::{
    AdmitCardBatchData, PrintJobListResult, PrintJobRun, PrintTemplateConfig, StudentIdCardData,
};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_professional_printing.db");
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
fn test_print_template_configs_defaults_and_update() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Fetch template configs (printers.get_template_configs)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "printers.get_template_configs",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get_template_configs failed: {:?}", res);
    let configs: Vec<PrintTemplateConfig> =
        serde_json::from_value(res["data"].clone()).expect("parse template configs");
    assert_eq!(configs.len(), 11, "expected 11 seeded template configs");

    let id_card_cfg = configs
        .iter()
        .find(|c| c.document_type == "id_card")
        .expect("id_card config present");
    assert_eq!(id_card_cfg.paper_size, "a4");
    assert_eq!(
        id_card_cfg.signature_title.as_deref(),
        Some("Principal / Director")
    );

    // 2. Update a template config (printers.update_template_config)
    let update_payload = json!({
        "config": {
            "document_type": "id_card",
            "paper_size": "id_card_pvc",
            "orientation": "landscape",
            "show_header": true,
            "show_logo": true,
            "show_watermark": true,
            "custom_header_text": "Excellence in Coaching",
            "custom_footer_text": "Card is property of Coachify Academy",
            "signature_title": "Headmaster",
            "options_json": "{\"layout_mode\":\"single_pvc\"}"
        }
    });
    let (res, _) = dispatch_cmd(
        &mut conn,
        "printers.update_template_config",
        update_payload,
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "update_template_config failed: {:?}", res);
    let updated: PrintTemplateConfig =
        serde_json::from_value(res["data"].clone()).expect("parse updated config");
    assert_eq!(updated.paper_size, "id_card_pvc");
    assert_eq!(updated.orientation, "landscape");
    assert!(updated.show_watermark);
    assert_eq!(updated.signature_title.as_deref(), Some("Headmaster"));
}

#[test]
fn test_print_job_run_audit_logging() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Record a print job (printers.record_print_job)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "printers.record_print_job",
        json!({
            "job": {
                "document_type": "admit_card",
                "batch_mode": true,
                "item_count": 45,
                "printer_name": "HP LaserJet Pro P1102",
                "paper_size": "a4",
                "status": "completed",
                "metadata_json": "{\"exam_id\":\"exam-123\"}"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "record_print_job failed: {:?}", res);
    let job: PrintJobRun = serde_json::from_value(res["data"].clone()).expect("parse job");
    assert_eq!(job.document_type, "admit_card");
    assert_eq!(job.item_count, 45);
    assert!(job.batch_mode);

    // 2. List print jobs (printers.list_print_jobs)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "printers.list_print_jobs",
        json!({
            "limit": 10,
            "document_type": "admit_card"
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "list_print_jobs failed: {:?}", res);
    let result: PrintJobListResult =
        serde_json::from_value(res["data"].clone()).expect("parse job list");
    assert_eq!(result.total_count, 1);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id, job.id);
}

#[test]
fn test_student_id_card_and_admit_card_data_queries() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Seed academic session and subjects
    let now = "2026-01-01T00:00:00Z";
    conn.execute(
        "INSERT OR IGNORE INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
         VALUES ('sess-2026', 'Session 2026', '2026-01-01', '2026-12-31', 1, ?1, ?1)",
        params![now],
    ).unwrap();

    conn.execute(
        "INSERT OR IGNORE INTO subjects (id, name, code, created_at, updated_at)
         VALUES ('sub-101', 'Physics', '101', ?1, ?1),
                ('sub-102', 'Chemistry', '102', ?1, ?1)",
        params![now],
    )
    .unwrap();

    // 2. Create program & batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "HSC Science 2026",
                "session_id": "sess-2026",
                "class_level": "HSC",
                "group_major": "Science"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "programs.create: {:?}", res);
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": &program_id,
                "name": "Batch Alpha",
                "capacity": 50
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "batches.create: {:?}", res);
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    // 3. Create student
    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({
            "student": {
                "name": "Tanvir Rahman",
                "name_bn": "তানভীর রহমান",
                "gender": "male",
                "dob": "2008-01-01",
                "blood_group": "B+",
                "phone": "01712345678",
                "present_address": "Dhaka, Bangladesh",
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Rafiqul Islam",
                        "relationship": "father",
                        "phone": "01712345679"
                    }
                }]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "students.create: {:?}", res);
    let student_id = res["data"]["id"].as_str().unwrap().to_string();
    let student_code = res["data"]["student_code"].as_str().unwrap().to_string();

    // 4. Enroll student in batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": &student_id,
                "batch_id": &batch_id,
                "roll": 12,
                "start_date": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "enrollments.create: {:?}", res);

    // 5. Query ID Card Data
    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.get_id_card_data",
        json!({
            "student_ids": [&student_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get_id_card_data: {:?}", res);
    let cards: Vec<StudentIdCardData> =
        serde_json::from_value(res["data"].clone()).expect("parse id cards");
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].student_id, student_id);
    assert_eq!(cards[0].student_code, student_code);
    assert_eq!(cards[0].full_name_en, "Tanvir Rahman");
    assert_eq!(cards[0].full_name_bn.as_deref(), Some("তানভীর রহমান"));
    assert_eq!(cards[0].blood_group.as_deref(), Some("B+"));
    assert_eq!(cards[0].barcode_payload, student_code);
    assert_eq!(cards[0].primary_batch_name.as_deref(), Some("Batch Alpha"));

    // 6. Create Exam & Subjects
    let (res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "HSC Term 1 Final",
            "session_id": "sess-2026",
            "exam_type": "term",
            "start_date": "2026-10-15",
            "end_date": "2026-10-20",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                {
                    "subject_id": "sub-101",
                    "full_marks": 100,
                    "pass_marks": 40,
                    "exam_date": "2026-10-15",
                    "start_time": "10:00 AM",
                    "room_number": "Room 301",
                    "is_optional_allowed": false
                },
                {
                    "subject_id": "sub-102",
                    "full_marks": 100,
                    "pass_marks": 40,
                    "exam_date": "2026-10-17",
                    "start_time": "10:00 AM",
                    "room_number": "Room 302",
                    "is_optional_allowed": false
                }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "exams.create: {:?}", res);
    let exam_id = res["data"]["exam"]["id"].as_str().unwrap().to_string();

    // 7. Query Admit Card Data
    let (res, _) = dispatch_cmd(
        &mut conn,
        "exams.get_admit_card_data",
        json!({
            "exam_id": &exam_id,
            "student_ids": [&student_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "get_admit_card_data: {:?}", res);
    let admit_data: AdmitCardBatchData =
        serde_json::from_value(res["data"].clone()).expect("parse admit data");
    assert_eq!(admit_data.exam_id, exam_id);
    assert_eq!(admit_data.exam_title, "HSC Term 1 Final");
    assert_eq!(admit_data.students.len(), 1);
    assert_eq!(admit_data.students[0].student_id, student_id);
    assert_eq!(admit_data.students[0].subjects.len(), 2);
    assert_eq!(admit_data.students[0].subjects[0].subject_name, "Physics");
    assert_eq!(admit_data.students[0].subjects[1].subject_name, "Chemistry");
    assert!(admit_data.students[0]
        .qr_verification_payload
        .contains(&student_code));
}
