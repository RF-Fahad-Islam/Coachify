use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_exams.db");
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

fn setup_academic_environment(
    conn: &mut rusqlite::Connection,
    token: &str,
) -> (String, String, Vec<String>) {
    // 1. Insert academic session
    let now = "2026-01-01T00:00:00Z";
    conn.execute(
        "INSERT OR IGNORE INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
         VALUES ('sess-2026', 'Session 2026', '2026-01-01', '2026-12-31', 1, ?1, ?1)",
        params![now],
    ).unwrap();

    // 2. Insert subjects
    let subjects = [
        ("sub-101", "Bangla", "101"),
        ("sub-102", "English", "102"),
        ("sub-103", "Mathematics", "103"),
        ("sub-104", "Higher Mathematics", "104"),
    ];
    for (id, name, code) in subjects {
        conn.execute(
            "INSERT OR IGNORE INTO subjects (id, name, code, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, name, code, now],
        )
        .unwrap();
    }

    // 3. Create program and batch
    let (p_res, _) = dispatch_cmd(
        conn,
        "programs.create",
        json!({ "program": { "name": "Class 10 SSC" } }),
        Some(token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": &program_id,
                "name": "Batch Alpha",
                "code": "ALPHA-10",
                "capacity": 50
            }
        }),
        Some(token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    (
        "sess-2026".to_string(),
        batch_id,
        vec![
            "sub-101".to_string(),
            "sub-102".to_string(),
            "sub-103".to_string(),
            "sub-104".to_string(),
        ],
    )
}

#[test]
fn test_nctb_gpa5_with_fourth_subject_bonus() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    // Enroll 3 students
    let s1 = create_student_helper(&mut conn, &token, "Rahim Chowdhury");
    let s2 = create_student_helper(&mut conn, &token, "Karim Ahmed");
    let s3 = create_student_helper(&mut conn, &token, "Farhana Yasmin");

    for (roll, s_id) in [(1, &s1), (2, &s2), (3, &s3)] {
        let (enr, _) = dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({
                "enrollment": {
                    "student_id": s_id,
                    "batch_id": &batch_id,
                    "roll": roll,
                    "start_date": "2026-01-01"
                }
            }),
            Some(&token),
            None,
        );
        assert_eq!(enr["ok"], true);
    }

    // Create exam
    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "First Term Examination 2026",
            "name_bn": "প্রথম সাময়িক পরীক্ষা ২০২৬",
            "session_id": session_id,
            "exam_type": "term",
            "start_date": "2026-04-01",
            "end_date": "2026-04-10",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[1], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[2], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[3], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": true }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(exam_res["ok"], true, "create exam failed: {:?}", exam_res);
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let exam_subs = exam_res["data"]["subjects"].as_array().unwrap();
    let es_bangla = exam_subs[0]["id"].as_str().unwrap().to_string();
    let es_english = exam_subs[1]["id"].as_str().unwrap().to_string();
    let es_math = exam_subs[2]["id"].as_str().unwrap().to_string();
    let es_hmath = exam_subs[3]["id"].as_str().unwrap().to_string();

    // 1. Enter marks for Bangla
    let (m1_res, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_bangla,
            "marks": [
                { "student_id": &s1, "marks": 85, "is_absent": false, "is_optional": false }, // A+ (5.00)
                { "student_id": &s2, "marks": 80, "is_absent": false, "is_optional": false }, // A+ (5.00)
                { "student_id": &s3, "marks": 25, "is_absent": false, "is_optional": false }  // F  (0.00, FAIL)
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(m1_res["ok"], true);

    // 2. Enter marks for English
    let (m2_res, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_english,
            "marks": [
                { "student_id": &s1, "marks": 75, "is_absent": false, "is_optional": false }, // A (4.00)
                { "student_id": &s2, "marks": 82, "is_absent": false, "is_optional": false }, // A+ (5.00)
                { "student_id": &s3, "marks": 90, "is_absent": false, "is_optional": false }  // A+ (5.00)
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(m2_res["ok"], true);

    // 3. Enter marks for Math
    let (m3_res, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_math,
            "marks": [
                { "student_id": &s1, "marks": 65, "is_absent": false, "is_optional": false }, // A- (3.50)
                { "student_id": &s2, "marks": 85, "is_absent": false, "is_optional": false }, // A+ (5.00)
                { "student_id": &s3, "marks": 92, "is_absent": false, "is_optional": false }  // A+ (5.00)
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(m3_res["ok"], true);

    // 4. Enter marks for Higher Math (4th subject / optional)
    let (m4_res, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_hmath,
            "marks": [
                { "student_id": &s1, "marks": 72, "is_absent": false, "is_optional": true }, // A (4.00) -> bonus = 2.00
                { "student_id": &s2, "marks": 88, "is_absent": false, "is_optional": true }, // A+ (5.00) -> bonus = 3.00
                { "student_id": &s3, "marks": 95, "is_absent": false, "is_optional": true }  // A+ (5.00)
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(m4_res["ok"], true);

    // Verify all subjects
    for es_id in [&es_bangla, &es_english, &es_math, &es_hmath] {
        let (v_res, _) = dispatch_cmd(
            &mut conn,
            "exams.verify_subject",
            json!({ "exam_subject_id": es_id }),
            Some(&token),
            None,
        );
        assert_eq!(v_res["ok"], true, "verify subject failed: {:?}", v_res);
    }

    // Preview results
    let (prev_res, _) = dispatch_cmd(
        &mut conn,
        "exams.preview_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(prev_res["ok"], true, "preview failed: {:?}", prev_res);

    let students = prev_res["data"]["students"].as_array().unwrap();
    assert_eq!(students.len(), 3);

    // Find student 1 (Rahim): Bangla 85 (5.00), English 75 (4.00), Math 65 (3.50), HMath 72 (4.00 -> bonus 2.00)
    // Mandatory sum: 5.00 + 4.00 + 3.50 = 12.50. Bonus = 2.00. Total GP = 14.50.
    // Divided by 3 mandatory subjects = 14.50 / 3 = 4.8333... -> 483 (4.83), Grade = A, passed = true.
    let r1 = students.iter().find(|s| s["student_id"] == s1).unwrap();
    assert_eq!(r1["gpa"], 483);
    assert_eq!(r1["grade"], "A");
    assert_eq!(r1["passed"], true);
    assert_eq!(r1["total_marks"], 85 + 75 + 65 + 72); // 297

    // Find student 2 (Karim): All 4 subjects 80+ -> 5.00 + 5.00 + 5.00 + bonus(3.00) = 18.00 / 3 = 6.00 -> capped at 500 (5.00).
    // Grade = A+, passed = true, rank = 1.
    let r2 = students.iter().find(|s| s["student_id"] == s2).unwrap();
    assert_eq!(r2["gpa"], 500);
    assert_eq!(r2["grade"], "A+");
    assert_eq!(r2["passed"], true);
    assert_eq!(r2["rank"], 1);

    // Find student 3 (Farhana): Failed Bangla (25/100) -> mandatory fail caps GPA to 0 (0.00), Grade = F, passed = false!
    let r3 = students.iter().find(|s| s["student_id"] == s3).unwrap();
    assert_eq!(r3["gpa"], 0);
    assert_eq!(r3["grade"], "F");
    assert_eq!(r3["passed"], false);

    // Publish results
    let (pub_res, _) = dispatch_cmd(
        &mut conn,
        "exams.publish_results",
        json!({
            "exam_id": &exam_id,
            "send_notifications": false
        }),
        Some(&token),
        None,
    );
    assert_eq!(pub_res["ok"], true, "publish failed: {:?}", pub_res);
    assert_eq!(pub_res["data"]["version"], 1);

    // Query student report card for Student 1
    let (rc_res, _) = dispatch_cmd(
        &mut conn,
        "exams.get_student_report_card",
        json!({
            "exam_id": &exam_id,
            "student_id": &s1
        }),
        Some(&token),
        None,
    );
    assert_eq!(rc_res["ok"], true, "get report card failed: {:?}", rc_res);
    assert_eq!(rc_res["data"]["gpa"], 483);
    assert_eq!(rc_res["data"]["grade"], "A");
    assert_eq!(rc_res["data"]["subjects"].as_array().unwrap().len(), 4);

    // Query tabulation sheet
    let (tab_res, _) = dispatch_cmd(
        &mut conn,
        "exams.get_tabulation_sheet",
        json!({
            "exam_id": &exam_id,
            "batch_id": &batch_id
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        tab_res["ok"], true,
        "get tabulation sheet failed: {:?}",
        tab_res
    );
    assert_eq!(tab_res["data"]["rows"].as_array().unwrap().len(), 3);
    assert_eq!(tab_res["data"]["subjects"].as_array().unwrap().len(), 4);
}

#[test]
fn test_fourth_subject_fail_does_not_fail_student() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Tanvir Hasan");
    dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": &s1, "batch_id": &batch_id, "roll": 10, "start_date": "2026-01-01" } }),
        Some(&token),
        None,
    );

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Mid Term 2026",
            "session_id": session_id,
            "exam_type": "term",
            "start_date": "2026-05-01",
            "end_date": "2026-05-10",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[1], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[3], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": true }
            ]
        }),
        Some(&token),
        None,
    );
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let exam_subs = exam_res["data"]["subjects"].as_array().unwrap();
    let es_m1 = exam_subs[0]["id"].as_str().unwrap().to_string();
    let es_m2 = exam_subs[1]["id"].as_str().unwrap().to_string();
    let es_opt = exam_subs[2]["id"].as_str().unwrap().to_string();

    // Mandatory subjects passed: 60/100 (A-, 3.50) and 60/100 (A-, 3.50)
    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_m1,
            "marks": [{ "student_id": &s1, "marks": 60, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );

    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_m2,
            "marks": [{ "student_id": &s1, "marks": 60, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );

    // Optional 4th subject FAILED: 20/100 (Fail, 0.00)
    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_opt,
            "marks": [{ "student_id": &s1, "marks": 20, "is_absent": false, "is_optional": true }]
        }),
        Some(&token),
        None,
    );

    for es in [&es_m1, &es_m2, &es_opt] {
        dispatch_cmd(
            &mut conn,
            "exams.verify_subject",
            json!({ "exam_subject_id": es }),
            Some(&token),
            None,
        );
    }

    let (prev_res, _) = dispatch_cmd(
        &mut conn,
        "exams.preview_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    let st = &prev_res["data"]["students"][0];

    // Student MUST pass! GPA = (3.50 + 3.50 + 0) / 2 = 3.50 (350), Grade = A-
    assert_eq!(st["passed"], true);
    assert_eq!(st["gpa"], 350);
    assert_eq!(st["grade"], "A-");
}

#[test]
fn test_complete_roster_verification_requirement() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Student 2");

    for s_id in [&s1, &s2] {
        dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({ "enrollment": { "student_id": s_id, "batch_id": &batch_id, "start_date": "2026-01-01" } }),
            Some(&token),
            None,
        );
    }

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Class Test 1",
            "session_id": session_id,
            "exam_type": "class_test",
            "start_date": "2026-02-01",
            "end_date": "2026-02-02",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 50, "pass_marks": 17, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    let es_id = exam_res["data"]["subjects"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Only enter marks for s1, s2 is left empty
    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": 40, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );

    // Verify should fail because roster is incomplete
    let (v_res, _) = dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_id }),
        Some(&token),
        None,
    );
    assert_eq!(v_res["ok"], false);
    assert_eq!(v_res["error"]["code"], "marks_incomplete");

    // Mark s2 as absent
    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s2, "marks": null, "is_absent": true, "is_optional": false }]
        }),
        Some(&token),
        None,
    );

    // Now verification succeeds
    let (v2_res, _) = dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_id }),
        Some(&token),
        None,
    );
    assert_eq!(v2_res["ok"], true);
}

#[test]
fn test_out_of_range_mark_rejected() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Student Range");
    dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": &s1, "batch_id": &batch_id, "start_date": "2026-01-01" } }),
        Some(&token),
        None,
    );

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Quiz 1",
            "session_id": session_id,
            "exam_type": "quiz",
            "start_date": "2026-02-01",
            "end_date": "2026-02-02",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 50, "pass_marks": 17, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    let es_id = exam_res["data"]["subjects"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Mark > full_marks
    let (err1, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": 55, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(err1["ok"], false);
    assert_eq!(err1["error"]["code"], "mark_out_of_range");

    // Negative mark
    let (err2, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": -5, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(err2["ok"], false);
    assert_eq!(err2["error"]["code"], "mark_negative");
}

#[test]
fn test_immutability_and_republish_workflow() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Republish Student");
    dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": &s1, "batch_id": &batch_id, "start_date": "2026-01-01" } }),
        Some(&token),
        None,
    );

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Annual Exam 2026",
            "session_id": session_id,
            "exam_type": "final",
            "start_date": "2026-11-01",
            "end_date": "2026-11-15",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let es_id = exam_res["data"]["subjects"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": 70, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );

    dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_id }),
        Some(&token),
        None,
    );

    // Publish v1
    let (pub1, _) = dispatch_cmd(
        &mut conn,
        "exams.publish_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(pub1["ok"], true);
    assert_eq!(pub1["data"]["version"], 1);

    // Direct save_marks now fails because exam is published
    let (save_err, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": 85, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save_err["ok"], false);
    assert_eq!(save_err["error"]["code"], "exam_already_published");

    // Revert exam to verified
    let (rev, _) = dispatch_cmd(
        &mut conn,
        "exams.revert_to_verified",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(rev["ok"], true);

    // Unverify subject marks
    let (unv, _) = dispatch_cmd(
        &mut conn,
        "exams.unverify_subject",
        json!({ "exam_subject_id": &es_id }),
        Some(&token),
        None,
    );
    assert_eq!(unv["ok"], true);

    // Now save corrected marks
    let (save2, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_id,
            "marks": [{ "student_id": &s1, "marks": 85, "is_absent": false, "is_optional": false }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save2["ok"], true);

    // Re-verify
    dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_id }),
        Some(&token),
        None,
    );

    // Re-publish as version 2
    let (pub2, _) = dispatch_cmd(
        &mut conn,
        "exams.publish_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(pub2["ok"], true);
    assert_eq!(pub2["data"]["version"], 2);

    // Check that previous snapshot has is_current = 0
    let (v1_current, v2_current): (i64, i64) = conn
        .query_row(
            "SELECT
            (SELECT is_current FROM result_snapshots WHERE exam_id = ?1 AND version = 1),
            (SELECT is_current FROM result_snapshots WHERE exam_id = ?1 AND version = 2)",
            params![exam_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(v1_current, 0);
    assert_eq!(v2_current, 1);
}

#[test]
fn test_deterministic_5_tier_tie_breaking() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Student Alpha");
    let s2 = create_student_helper(&mut conn, &token, "Student Beta");

    // Student Alpha: Roll 1
    // Student Beta: Roll 2
    dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": { "student_id": &s1, "batch_id": &batch_id, "roll": 1, "start_date": "2026-01-01" }
        }),
        Some(&token),
        None,
    );

    dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": { "student_id": &s2, "batch_id": &batch_id, "roll": 2, "start_date": "2026-01-01" }
        }),
        Some(&token),
        None,
    );

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Tie Break Exam",
            "session_id": session_id,
            "exam_type": "term",
            "start_date": "2026-03-01",
            "end_date": "2026-03-05",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[1], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let es_1 = exam_res["data"]["subjects"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let es_2 = exam_res["data"]["subjects"][1]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Both get Grade A (4.00, 70-79%) on both subjects:
    // Student Alpha: 78 in Sub 1, 78 in Sub 2 -> Total = 156, GPA = 4.00
    // Student Beta:  72 in Sub 1, 72 in Sub 2 -> Total = 144, GPA = 4.00
    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_1,
            "marks": [
                { "student_id": &s1, "marks": 78, "is_absent": false, "is_optional": false },
                { "student_id": &s2, "marks": 72, "is_absent": false, "is_optional": false }
            ]
        }),
        Some(&token),
        None,
    );

    dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_2,
            "marks": [
                { "student_id": &s1, "marks": 78, "is_absent": false, "is_optional": false },
                { "student_id": &s2, "marks": 72, "is_absent": false, "is_optional": false }
            ]
        }),
        Some(&token),
        None,
    );

    for es in [&es_1, &es_2] {
        dispatch_cmd(
            &mut conn,
            "exams.verify_subject",
            json!({ "exam_subject_id": es }),
            Some(&token),
            None,
        );
    }

    let (prev, _) = dispatch_cmd(
        &mut conn,
        "exams.preview_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    let st_list = prev["data"]["students"].as_array().unwrap();

    let st_alpha = st_list.iter().find(|s| s["student_id"] == s1).unwrap();
    let st_beta = st_list.iter().find(|s| s["student_id"] == s2).unwrap();

    assert_eq!(st_alpha["gpa"], 400);
    assert_eq!(st_beta["gpa"], 400);

    // Alpha has total marks 156 vs Beta's 144 -> Alpha is Rank 1, Beta is Rank 2
    assert_eq!(st_alpha["rank"], 1);
    assert_eq!(st_beta["rank"], 2);
}

#[test]
fn test_multi_subject_verification_and_unauthorized_edit_gate() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s1 = create_student_helper(&mut conn, &token, "Multi Subject S1");
    let s2 = create_student_helper(&mut conn, &token, "Multi Subject S2");

    for s_id in [&s1, &s2] {
        dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({ "enrollment": { "student_id": s_id, "batch_id": &batch_id, "start_date": "2026-01-01" } }),
            Some(&token),
            None,
        );
    }

    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Term Final 3 Subjects",
            "session_id": session_id,
            "exam_type": "term",
            "start_date": "2026-03-01",
            "end_date": "2026-03-10",
            "grading_scheme_id": "scheme_nctb_gpa5",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[1], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false },
                { "subject_id": &subjects[2], "full_marks": 100, "pass_marks": 33, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(exam_res["ok"], true);
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let es_list = exam_res["data"]["subjects"].as_array().unwrap();
    let es_1 = es_list[0]["id"].as_str().unwrap().to_string();
    let es_2 = es_list[1]["id"].as_str().unwrap().to_string();
    let es_3 = es_list[2]["id"].as_str().unwrap().to_string();

    // Enter marks for all 3 subjects
    for es in [&es_1, &es_2, &es_3] {
        let (save_res, _) = dispatch_cmd(
            &mut conn,
            "exams.save_marks",
            json!({
                "exam_subject_id": es,
                "marks": [
                    { "student_id": &s1, "marks": 85, "is_absent": false, "is_optional": false },
                    { "student_id": &s2, "marks": 90, "is_absent": false, "is_optional": false }
                ]
            }),
            Some(&token),
            None,
        );
        assert_eq!(save_res["ok"], true);
    }

    // Verify Subject 1 -> Exam should still be in 'entry'
    let (v1_res, _) = dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_1 }),
        Some(&token),
        None,
    );
    assert_eq!(v1_res["ok"], true);
    let (get1, _) = dispatch_cmd(
        &mut conn,
        "exams.get",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(get1["data"]["exam"]["status"], "entry");

    // Verify Subject 2 -> Exam should still be in 'entry'
    let (v2_res, _) = dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_2 }),
        Some(&token),
        None,
    );
    assert_eq!(v2_res["ok"], true);
    let (get2, _) = dispatch_cmd(
        &mut conn,
        "exams.get",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(get2["data"]["exam"]["status"], "entry");

    // Attempt to modify Subject 1 marks directly while verified -> must return unauthorized_verified_edit
    let (save_blocked, _) = dispatch_cmd(
        &mut conn,
        "exams.save_marks",
        json!({
            "exam_subject_id": &es_1,
            "marks": [
                { "student_id": &s1, "marks": 88, "is_absent": false, "is_optional": false },
                { "student_id": &s2, "marks": 90, "is_absent": false, "is_optional": false }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save_blocked["ok"], false);
    assert_eq!(save_blocked["error"]["code"], "unauthorized_verified_edit");

    // Verify Subject 3 -> Now ALL 3 subjects are verified, exam status must transition to 'verified'
    let (v3_res, _) = dispatch_cmd(
        &mut conn,
        "exams.verify_subject",
        json!({ "exam_subject_id": &es_3 }),
        Some(&token),
        None,
    );
    assert_eq!(v3_res["ok"], true);
    let (get3, _) = dispatch_cmd(
        &mut conn,
        "exams.get",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(get3["data"]["exam"]["status"], "verified");

    // Unverify Subject 1 -> Exam reverts to 'entry' and records audit log
    let (unv_res, _) = dispatch_cmd(
        &mut conn,
        "exams.unverify_subject",
        json!({ "exam_subject_id": &es_1 }),
        Some(&token),
        None,
    );
    assert_eq!(unv_res["ok"], true);
    let (get4, _) = dispatch_cmd(
        &mut conn,
        "exams.get",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(get4["data"]["exam"]["status"], "entry");

    // Verify audit log has entry for exam.unverify_subject
    let (audit_res, _) = dispatch_cmd(&mut conn, "audit.list", json!({}), Some(&token), None);
    let logs = audit_res["data"]["items"].as_array().unwrap();
    let unv_log = logs.iter().find(|l| l["action"] == "exam.unverify_subject");
    assert!(
        unv_log.is_some(),
        "Audit log must contain exam.unverify_subject action"
    );
    assert_eq!(unv_log.unwrap()["entity_id"], es_1);
}

#[test]
fn test_publishing_gate_and_british_curriculum() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let (session_id, batch_id, subjects) = setup_academic_environment(&mut conn, &token);

    let s_top = create_student_helper(&mut conn, &token, "British Star Student");
    let s_good = create_student_helper(&mut conn, &token, "British Good Student");

    for s_id in [&s_top, &s_good] {
        dispatch_cmd(
            &mut conn,
            "enrollments.create",
            json!({ "enrollment": { "student_id": s_id, "batch_id": &batch_id, "start_date": "2026-01-01" } }),
            Some(&token),
            None,
        );
    }

    // Create British Curriculum Exam (GPA 4.0 scale)
    let (exam_res, _) = dispatch_cmd(
        &mut conn,
        "exams.create",
        json!({
            "name": "Cambridge O-Level Mock",
            "session_id": session_id,
            "exam_type": "model_test",
            "start_date": "2026-04-01",
            "end_date": "2026-04-05",
            "grading_scheme_id": "scheme_british_gpa4",
            "batch_ids": [&batch_id],
            "subjects": [
                { "subject_id": &subjects[0], "full_marks": 100, "pass_marks": 40, "is_optional_allowed": false },
                { "subject_id": &subjects[1], "full_marks": 100, "pass_marks": 40, "is_optional_allowed": false }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(exam_res["ok"], true);
    let exam_id = exam_res["data"]["exam"]["id"].as_str().unwrap().to_string();
    let es_list = exam_res["data"]["subjects"].as_array().unwrap();
    let es_1 = es_list[0]["id"].as_str().unwrap().to_string();
    let es_2 = es_list[1]["id"].as_str().unwrap().to_string();

    // S_Top scores 95 in both (95% -> A*)
    // S_Good scores 85 in both (85% -> A)
    for es in [&es_1, &es_2] {
        dispatch_cmd(
            &mut conn,
            "exams.save_marks",
            json!({
                "exam_subject_id": es,
                "marks": [
                    { "student_id": &s_top, "marks": 95, "is_absent": false, "is_optional": false },
                    { "student_id": &s_good, "marks": 85, "is_absent": false, "is_optional": false }
                ]
            }),
            Some(&token),
            None,
        );
    }

    // Attempt to publish before subjects are verified -> must fail with exam_not_verified
    let (pub_blocked, _) = dispatch_cmd(
        &mut conn,
        "exams.publish_results",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(pub_blocked["ok"], false);
    assert_eq!(pub_blocked["error"]["code"], "exam_not_verified");

    // Verify both subjects
    for es in [&es_1, &es_2] {
        let (v_res, _) = dispatch_cmd(
            &mut conn,
            "exams.verify_subject",
            json!({ "exam_subject_id": es }),
            Some(&token),
            None,
        );
        assert_eq!(v_res["ok"], true);
    }

    // Now publish exam
    let (pub_ok, _) = dispatch_cmd(
        &mut conn,
        "exams.publish_results",
        json!({ "exam_id": &exam_id, "send_notifications": false }),
        Some(&token),
        None,
    );
    assert_eq!(pub_ok["ok"], true);

    // Fetch report cards and verify grades
    let (rc_top, _) = dispatch_cmd(
        &mut conn,
        "exams.get_student_report_card",
        json!({ "exam_id": &exam_id, "student_id": &s_top }),
        Some(&token),
        None,
    );
    assert_eq!(rc_top["ok"], true);
    assert_eq!(rc_top["data"]["gpa"], 400); // 4.00
    assert_eq!(rc_top["data"]["grade"], "A*"); // Cambridge 95% is A*!
    assert_eq!(rc_top["data"]["rank"], 1);

    let (rc_good, _) = dispatch_cmd(
        &mut conn,
        "exams.get_student_report_card",
        json!({ "exam_id": &exam_id, "student_id": &s_good }),
        Some(&token),
        None,
    );
    assert_eq!(rc_good["ok"], true);
    assert_eq!(rc_good["data"]["gpa"], 400); // 4.00
    assert_eq!(rc_good["data"]["grade"], "A"); // Cambridge 85% is A!
    assert_eq!(rc_good["data"]["rank"], 2);

    // Test revert to verified and audit log
    let (rev_res, _) = dispatch_cmd(
        &mut conn,
        "exams.revert_to_verified",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(rev_res["ok"], true);

    let (get_exam, _) = dispatch_cmd(
        &mut conn,
        "exams.get",
        json!({ "exam_id": &exam_id }),
        Some(&token),
        None,
    );
    assert_eq!(get_exam["data"]["exam"]["status"], "verified");

    // Verify audit log has entry for exam.revert_published
    let (audit_res, _) = dispatch_cmd(&mut conn, "audit.list", json!({}), Some(&token), None);
    let logs = audit_res["data"]["items"].as_array().unwrap();
    let rev_log = logs.iter().find(|l| l["action"] == "exam.revert_published");
    assert!(
        rev_log.is_some(),
        "Audit log must contain exam.revert_published action"
    );
    assert_eq!(rev_log.unwrap()["entity_id"], exam_id);
}
