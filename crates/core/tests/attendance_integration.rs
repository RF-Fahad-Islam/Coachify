use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_attendance.db");
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

#[test]
fn test_attendance_fast_batch_marking_and_querying() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1: Create Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Class 10 Science" } }),
        Some(&token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": &program_id,
                "name": "Batch Morning A",
                "code": "MOR-A",
                "capacity": 40
            }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // 2: Enroll 4 students
    let s1 = create_student_helper(&mut conn, &token, "Student One");
    let s2 = create_student_helper(&mut conn, &token, "Student Two");
    let s3 = create_student_helper(&mut conn, &token, "Student Three");
    let s4 = create_student_helper(&mut conn, &token, "Student Four");

    for s in [&s1, &s2, &s3, &s4] {
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

    // 3: Fetch initial session for 2026-03-05 (session does not exist yet)
    let (sess_init, _) = dispatch_cmd(
        &mut conn,
        "attendance.get_session",
        json!({ "batch_id": &batch_id, "date": "2026-03-05" }),
        Some(&token),
        None,
    );
    assert_eq!(sess_init["ok"], true);
    assert_eq!(sess_init["data"]["session_exists"], false);
    assert_eq!(sess_init["data"]["locked"], false);
    let marks_init = sess_init["data"]["marks"].as_array().unwrap();
    assert_eq!(marks_init.len(), 4);
    assert_eq!(marks_init[0]["status"], "present");

    // 4: Save session marks (batch marking with varied statuses)
    let (save_res, events) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-05",
            "notes": "Physics lecture on mechanics",
            "marks": [
                { "student_id": &s1, "status": "present" },
                { "student_id": &s2, "status": "absent", "note": "Sick leave" },
                { "student_id": &s3, "status": "late", "note": "Traffic delay" },
                { "student_id": &s4, "status": "excused", "note": "Sports event" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save_res["ok"], true, "save session failed: {:?}", save_res);
    assert_eq!(save_res["data"]["session_exists"], true);
    assert_eq!(save_res["data"]["notes"], "Physics lecture on mechanics");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "attendance");

    // 5: Fetch session again and verify saved marks
    let (sess_saved, _) = dispatch_cmd(
        &mut conn,
        "attendance.get_session",
        json!({ "batch_id": &batch_id, "date": "2026-03-05" }),
        Some(&token),
        None,
    );
    assert_eq!(sess_saved["ok"], true);
    assert_eq!(sess_saved["data"]["session_exists"], true);
    let marks = sess_saved["data"]["marks"].as_array().unwrap();
    assert_eq!(marks.len(), 4);

    let m1 = marks.iter().find(|m| m["student_id"] == s1).unwrap();
    assert_eq!(m1["status"], "present");

    let m2 = marks.iter().find(|m| m["student_id"] == s2).unwrap();
    assert_eq!(m2["status"], "absent");
    assert_eq!(m2["note"], "Sick leave");

    let m3 = marks.iter().find(|m| m["student_id"] == s3).unwrap();
    assert_eq!(m3["status"], "late");

    let m4 = marks.iter().find(|m| m["student_id"] == s4).unwrap();
    assert_eq!(m4["status"], "excused");
}

#[test]
fn test_attendance_locking_and_retroactive_edit_rules() {
    let (_dir, mut conn) = setup_test_db();
    let owner_token = bootstrap_and_login(&mut conn);

    // 1: Setup Program and Batch with 1 student
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Commerce Revision" } }),
        Some(&owner_token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Batch B", "capacity": 20 }
        }),
        Some(&owner_token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &owner_token, "Student Lock Test");
    let (enr, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": &s1,
                "batch_id": &batch_id,
                "start_date": "2026-03-01"
            }
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(enr["ok"], true);

    // 2: Save initial session
    let (save1, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-01",
            "marks": [{ "student_id": &s1, "status": "absent" }]
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(save1["ok"], true);
    let session_id = save1["data"]["id"].as_str().unwrap().to_string();

    // 3: Explicitly lock session
    let (lock_res, _) = dispatch_cmd(
        &mut conn,
        "attendance.lock_session",
        json!({ "session_id": &session_id }),
        Some(&owner_token),
        None,
    );
    assert_eq!(lock_res["ok"], true);

    // 4: Create Teacher user who lacks attendance.edit_previous permission
    let (role_res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Teacher Standard",
            "permission_codes": ["attendance.mark", "attendance.view"]
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(role_res["ok"], true);
    let teacher_role_id = role_res["data"]["id"].as_str().unwrap().to_string();

    let (user_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "teacher1",
            "display_name": "Teacher Assistant",
            "password": "Password123!",
            "role_ids": [teacher_role_id]
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(user_res["ok"], true);

    let (login_teacher, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({ "username": "teacher1", "password": "Password123!" }),
        None,
        None,
    );
    assert_eq!(login_teacher["ok"], true);
    let teacher_token = login_teacher["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    // 5: Teacher attempts to edit locked session: rejected with attendance_locked
    let (edit_attempt, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-01",
            "marks": [{ "student_id": &s1, "status": "present" }]
        }),
        Some(&teacher_token),
        None,
    );
    assert_eq!(edit_attempt["ok"], false);
    assert_eq!(edit_attempt["error"]["code"], "attendance_locked");

    // 6: Owner attempts to edit locked session WITHOUT audit reason: rejected with missing_audit_reason
    let (owner_attempt_no_reason, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-01",
            "marks": [{ "student_id": &s1, "status": "present" }],
            "audit_reason": "   "
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(owner_attempt_no_reason["ok"], false);
    assert_eq!(
        owner_attempt_no_reason["error"]["code"],
        "missing_audit_reason"
    );

    // 7: Owner edits locked session WITH valid audit reason: succeeds
    let (owner_edit_ok, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-01",
            "marks": [{ "student_id": &s1, "status": "present" }],
            "audit_reason": "Guardian provided medical certificate"
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(owner_edit_ok["ok"], true);

    // 8: Verify audit log recorded the retroactive edit
    let (audit_list, _) = dispatch_cmd(
        &mut conn,
        "audit.list",
        json!({
            "entity": "attendance_sessions",
            "entity_id": &session_id,
            "action": "attendance.edit_locked"
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(audit_list["ok"], true);
    let items = audit_list["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert!(
        items[0]["user_id"].is_string(),
        "Audit entry must record acting user_id"
    );
    assert!(!items[0]["user_id"].as_str().unwrap().is_empty());
    assert!(items[0]["after_json"]
        .as_str()
        .unwrap()
        .contains("Guardian provided medical certificate"));
}

#[test]
fn test_absence_notification_hook_and_cancellation() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Absence Automation Program" } }),
        Some(&token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Batch Alert", "capacity": 30 }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Absent Student");
    let (enr, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": &s1,
                "batch_id": &batch_id,
                "start_date": "2026-03-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(enr["ok"], true);

    // 1: Mark student absent in session
    let (save1, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-08",
            "marks": [{ "student_id": &s1, "status": "absent" }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save1["ok"], true);
    let session_id = save1["data"]["id"].as_str().unwrap().to_string();

    // 2: Check outbox entries created with status queued
    let sms_dedupe = format!("attendance:{}:{}:absence_sms", session_id, s1);
    let outbox_status: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE dedupe_key = ?1",
            params![sms_dedupe],
            |row| row.get(0),
        )
        .expect("SMS outbox row must exist");
    assert_eq!(outbox_status, "queued");

    // 3: Teacher corrects mark from absent to present
    let (save2, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-08",
            "marks": [{ "student_id": &s1, "status": "present" }]
        }),
        Some(&token),
        None,
    );
    assert_eq!(save2["ok"], true);

    // 4: Verify outbox entry is now cancelled!
    let cancelled_status: String = conn
        .query_row(
            "SELECT status FROM outbox WHERE dedupe_key = ?1",
            params![sms_dedupe],
            |row| row.get(0),
        )
        .expect("Outbox row should still exist");
    assert_eq!(cancelled_status, "cancelled");
}

#[test]
fn test_student_attendance_summary_and_batch_matrix() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Setup Program and Batch
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Summary Program" } }),
        Some(&token),
        None,
    );
    let program_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Section Alpha", "capacity": 20 }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Matrix Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Matrix Student 2");

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

    // Record attendance on 3 dates in March 2026
    // Day 1: s1 present, s2 absent
    let (r1, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-10",
            "marks": [
                { "student_id": &s1, "status": "present" },
                { "student_id": &s2, "status": "absent" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(r1["ok"], true);

    // Day 2: s1 late, s2 present
    let (r2, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-11",
            "marks": [
                { "student_id": &s1, "status": "late" },
                { "student_id": &s2, "status": "present" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(r2["ok"], true);

    // Day 3: s1 excused, s2 absent
    let (r3, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": &batch_id,
            "date": "2026-03-12",
            "marks": [
                { "student_id": &s1, "status": "excused" },
                { "student_id": &s2, "status": "absent" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(r3["ok"], true);

    // 1: Query student summary for s1 (1 present + 1 late = 2 attended out of 3 sessions = 66.7%)
    let (s1_sum, _) = dispatch_cmd(
        &mut conn,
        "attendance.get_student_summary",
        json!({ "student_id": &s1 }),
        Some(&token),
        None,
    );
    assert_eq!(s1_sum["ok"], true);
    let s1_data = &s1_sum["data"];
    assert_eq!(s1_data["total_sessions"], 3);
    assert_eq!(s1_data["present_count"], 1);
    assert_eq!(s1_data["late_count"], 1);
    assert_eq!(s1_data["excused_count"], 1);
    assert_eq!(s1_data["absent_count"], 0);
    assert_eq!(s1_data["attendance_percentage"], 66.7);

    // 2: Query student summary for s2 (1 present out of 3 sessions = 33.3%)
    let (s2_sum, _) = dispatch_cmd(
        &mut conn,
        "attendance.get_student_summary",
        json!({ "student_id": &s2 }),
        Some(&token),
        None,
    );
    assert_eq!(s2_sum["ok"], true);
    let s2_data = &s2_sum["data"];
    assert_eq!(s2_data["total_sessions"], 3);
    assert_eq!(s2_data["present_count"], 1);
    assert_eq!(s2_data["absent_count"], 2);
    assert_eq!(s2_data["attendance_percentage"], 33.3);

    // 3: Query monthly matrix for batch in 2026-03
    let (matrix_res, _) = dispatch_cmd(
        &mut conn,
        "attendance.get_batch_matrix",
        json!({ "batch_id": &batch_id, "month": "2026-03" }),
        Some(&token),
        None,
    );
    assert_eq!(matrix_res["ok"], true);
    let matrix = &matrix_res["data"];
    let dates = matrix["dates"].as_array().unwrap();
    assert_eq!(dates.len(), 3);
    assert_eq!(dates[0], "2026-03-10");
    assert_eq!(dates[1], "2026-03-11");
    assert_eq!(dates[2], "2026-03-12");

    let students = matrix["students"].as_array().unwrap();
    assert_eq!(students.len(), 2);
    let row1 = students.iter().find(|r| r["student_id"] == s1).unwrap();
    assert_eq!(row1["marks"]["2026-03-10"], "present");
    assert_eq!(row1["marks"]["2026-03-11"], "late");
    assert_eq!(row1["marks"]["2026-03-12"], "excused");
}
