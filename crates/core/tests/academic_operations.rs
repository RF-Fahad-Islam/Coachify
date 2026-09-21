use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_academic.db");
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(10000);

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
                "present_address": "Dhaka, Bangladesh",
                "phone": student_phone,
                "guardians": [{
                    "is_primary": true,
                    "receives_sms": true,
                    "new_guardian": {
                        "name": "Father",
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
fn test_program_crud_and_delete_prevention() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create Program
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "HSC Science 2026",
                "code": "HSC-SCI-26",
                "description": "Two-year intermediate program"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create program failed: {:?}", res);
    let program_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["name"], "HSC Science 2026");
    assert_eq!(res["data"]["status"], "active");

    // 2. List Programs
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.list",
        json!({ "status": "active" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], program_id);

    // 3. Update Program
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.update",
        json!({
            "id": program_id,
            "program": {
                "name": "HSC Science 2026 Revised",
                "status": "archived"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["name"], "HSC Science 2026 Revised");
    assert_eq!(res["data"]["status"], "archived");

    // 4. Create batch linked to program
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Morning Batch A",
                "code": "MOR-A",
                "capacity": 40
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create batch failed: {:?}", res);

    // 5. Test deletion prevention trigger on programs
    let err = conn.execute("DELETE FROM programs WHERE id = ?", [&program_id]);
    assert!(
        err.is_err(),
        "Direct SQL delete of program with batches should be blocked"
    );
    let err_msg = err.unwrap_err().to_string();
    assert!(
        err_msg.contains("Cannot delete program with existing batches")
            || err_msg.contains("FOREIGN KEY constraint failed"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[test]
fn test_batch_creation_and_schedule_conflict_detection() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create Program
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": { "name": "SSC Crash Course 2026" }
        }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    // Create Staff (Teacher)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.create",
        json!({
            "staff": {
                "name": "Dr. Kamal Hossain",
                "phone": "01711-998877",
                "designation": "Head of Physics"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let teacher_id = res["data"]["id"].as_str().unwrap().to_string();

    // 1. Create Batch 1: Monday & Wednesday 10:00 - 11:30 in Room 101 with Dr. Kamal
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Batch Physics 1",
                "code": "PHY-1",
                "capacity": 30,
                "teacher_staff_id": teacher_id,
                "schedule": [
                    { "day_of_week": "Monday", "start_time": "10:00", "end_time": "11:30", "room": "Room 101" },
                    { "day_of_week": "Wednesday", "start_time": "10:00", "end_time": "11:30", "room": "Room 101" }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "Batch 1 create failed: {:?}", res);

    // 2. Attempt Batch 2 with Room conflict (Monday 11:00 - 12:00 in Room 101)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Batch Chemistry",
                "code": "CHEM-1",
                "capacity": 30,
                "schedule": [
                    { "day_of_week": "Monday", "start_time": "11:00", "end_time": "12:00", "room": "Room 101" }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "schedule_conflict");
    assert!(res["error"]["message"].as_str().unwrap().contains("Room"));

    // 3. Attempt Batch 3 with Teacher conflict (Monday 10:30 - 11:30 in Room 202 with Dr. Kamal)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Batch Physics 2",
                "code": "PHY-2",
                "capacity": 25,
                "teacher_staff_id": teacher_id,
                "schedule": [
                    { "day_of_week": "Monday", "start_time": "10:30", "end_time": "11:30", "room": "Room 202" }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "schedule_conflict");
    assert!(res["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Teacher"));

    // 4. Override conflict with allow_conflict: true
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Batch Chemistry (Overridden)",
                "code": "CHEM-OVER",
                "capacity": 30,
                "schedule": [
                    { "day_of_week": "Monday", "start_time": "11:00", "end_time": "12:00", "room": "Room 101" }
                ]
            },
            "allow_conflict": true
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "Override batch creation should succeed");

    // 5. Test pre-check conflict RPC directly
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.check_conflicts",
        json!({
            "teacher_staff_id": teacher_id,
            "schedule": [
                { "day_of_week": "Wednesday", "start_time": "11:00", "end_time": "12:00" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let conflicts = res["data"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0]["category"], "teacher");
}

#[test]
fn test_staff_management_phone_normalization_and_fts5_search() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create Staff with unformatted phone
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.create",
        json!({
            "staff": {
                "name": "Farhana Yasmin",
                "phone": "01799 123 456",
                "email": "farhana@example.com",
                "designation": "Biology Senior Instructor"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "Staff create failed: {:?}", res);
    let staff_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["phone_norm"], "8801799123456");

    // 2. FTS5 Search by Name
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.search",
        json!({ "query": "Farhana" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], staff_id);

    // 3. FTS5 Search by partial phone
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.search",
        json!({ "query": "8801799" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);

    // 4. FTS5 Search by designation
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.search",
        json!({ "query": "Biology" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);

    // 5. Update Staff and re-search
    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.update",
        json!({
            "id": staff_id,
            "staff": {
                "designation": "Zoology Lecturer"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.search",
        json!({ "query": "Zoology" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn test_staff_deletion_prevention_trigger() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create Program & Staff
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "General Science" } }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "staff.create",
        json!({
            "staff": {
                "name": "Monirul Islam",
                "phone": "01711-224466"
            }
        }),
        Some(&token),
        None,
    );
    let staff_id = res["data"]["id"].as_str().unwrap().to_string();

    // Assign staff as lead teacher in batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Sec A",
                "code": "SEC-A-MON",
                "capacity": 20,
                "teacher_staff_id": staff_id
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Direct SQL delete on staff must fail via trigger
    let err = conn.execute("DELETE FROM staff WHERE id = ?", [&staff_id]);
    assert!(err.is_err());
    let err_msg = err.unwrap_err().to_string();
    assert!(
        err_msg.contains("Cannot delete staff member assigned as lead teacher of a batch"),
        "Unexpected error message: {}",
        err_msg
    );
}

#[test]
fn test_enrollment_auto_roll_and_duplicate_roll_rejection() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create program & batch
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Commerce 2026" } }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Accounting Morning",
                "code": "ACC-MORN",
                "capacity": 50
            }
        }),
        Some(&token),
        None,
    );
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    // Create students
    let s1 = create_student_helper(&mut conn, &token, "Student One");
    let s2 = create_student_helper(&mut conn, &token, "Student Two");
    let s3 = create_student_helper(&mut conn, &token, "Student Three");
    let s4 = create_student_helper(&mut conn, &token, "Student Four");

    // 1. Enroll S1 without roll -> gets Roll 1
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s1,
                "batch_id": batch_id
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "enrollment 1 failed: {:?}", res);
    assert_eq!(res["data"]["roll"], 1);

    // 2. Enroll S2 without roll -> gets Roll 2
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s2,
                "batch_id": batch_id
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["roll"], 2);

    // 3. Attempt manual duplicate roll 1 for S3 -> rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s3,
                "batch_id": batch_id,
                "roll": 1
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "roll_taken");

    // 4. Enroll S3 with manual roll 15
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s3,
                "batch_id": batch_id,
                "roll": 15
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["roll"], 15);

    // 5. Enroll S4 without roll -> gets max(roll) + 1 = 16
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s4,
                "batch_id": batch_id
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["roll"], 16);

    // 6. Verify roster query
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.list_by_batch",
        json!({ "batch_id": batch_id }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let roster = res["data"].as_array().unwrap();
    assert_eq!(roster.len(), 4);
    assert_eq!(roster[0]["roll"], 1);
    assert_eq!(roster[3]["roll"], 16);
}

#[test]
fn test_batch_capacity_guard_and_overcapacity_override() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Small Batch Program" } }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Max 2 Batch",
                "code": "CAP-2",
                "capacity": 2
            }
        }),
        Some(&token),
        None,
    );
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Student 2");
    let s3 = create_student_helper(&mut conn, &token, "Student 3");

    // Enroll S1, S2 (fills capacity)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": s1, "batch_id": batch_id } }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": s2, "batch_id": batch_id } }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Attempt S3 without allow_overcapacity -> rejected
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": s3, "batch_id": batch_id } }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "batch_capacity_reached");

    // S3 with allow_overcapacity: true -> succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": { "student_id": s3, "batch_id": batch_id },
            "allow_overcapacity": true
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
}

#[test]
fn test_atomic_batch_transfer_transaction() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Transfer Program" } }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Section A", "code": "SEC-A", "capacity": 10 }
        }),
        Some(&token),
        None,
    );
    let batch_a = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Section B", "code": "SEC-B", "capacity": 1 }
        }),
        Some(&token),
        None,
    );
    let batch_b = res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Transfer Student 1");
    let s2 = create_student_helper(&mut conn, &token, "Transfer Student 2");

    // Enroll S1 into Section A
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": &s1, "batch_id": &batch_a } }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Transfer S1 from Section A to Section B
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.transfer",
        json!({
            "transfer": {
                "student_id": &s1,
                "from_batch_id": &batch_a,
                "to_batch_id": &batch_b,
                "notes": "Shifted due to morning timing"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "transfer failed: {:?}", res);
    assert_eq!(res["data"]["batch_id"], batch_b);
    assert_eq!(res["data"]["status"], "active");

    // Verify S1 in Section A is now 'ended' with end_reason 'transferred'
    let old_status: (String, Option<String>) = conn
        .query_row(
            "SELECT status, end_reason FROM enrollments WHERE student_id = ? AND batch_id = ?",
            params![s1, batch_a],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(old_status.0, "ended");
    assert_eq!(old_status.1.as_deref(), Some("transferred"));

    // Enroll S2 into Section A
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({ "enrollment": { "student_id": &s2, "batch_id": &batch_a } }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // Attempt transferring S2 to Section B (Section B is full! capacity 1)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.transfer",
        json!({
            "transfer": {
                "student_id": &s2,
                "from_batch_id": &batch_a,
                "to_batch_id": &batch_b
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "batch_capacity_reached");

    // Invariant check: S2 must STILL be active in Section A (transaction rollback!)
    let s2_status: String = conn
        .query_row(
            "SELECT status FROM enrollments WHERE student_id = ? AND batch_id = ?",
            params![s2, batch_a],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        s2_status, "active",
        "Failed transfer must leave source enrollment active"
    );
}

#[test]
fn test_bulk_enrollment_sequential_rolls() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Bulk Program" } }),
        Some(&token),
        None,
    );
    let program_id = res["data"]["id"].as_str().unwrap().to_string();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": &program_id, "name": "Bulk Section", "code": "BLK-SEC", "capacity": 20 }
        }),
        Some(&token),
        None,
    );
    let batch_id = res["data"]["id"].as_str().unwrap().to_string();

    let s1 = create_student_helper(&mut conn, &token, "Bulk 1");
    let s2 = create_student_helper(&mut conn, &token, "Bulk 2");
    let s3 = create_student_helper(&mut conn, &token, "Bulk 3");

    let (res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.bulk_create",
        json!({
            "bulk": {
                "batch_id": batch_id,
                "student_ids": [s1, s2, s3],
                "start_date": "2026-01-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "bulk enroll failed: {:?}", res);
    let list = res["data"].as_array().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0]["roll"], 1);
    assert_eq!(list[1]["roll"], 2);
    assert_eq!(list[2]["roll"], 3);
}

#[test]
fn test_rbac_permission_enforcement() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create a restricted role with only academic.view and staff.view
    let (res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Receptionist View Only",
            "permission_codes": ["academic.view", "staff.view"]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create role failed: {:?}", res);
    let role_id = res["data"]["id"].as_str().unwrap().to_string();

    // Create a new user with this restricted role
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "viewer",
            "display_name": "Viewer Receptionist",
            "password": "Password123!",
            "role_ids": [role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "create user failed: {:?}", res);

    // Login as viewer
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "viewer",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    let viewer_token = res["data"]["session_token"].as_str().unwrap().to_string();

    // Attempt programs.create with viewer token -> should be forbidden!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "Illegal Program" } }),
        Some(&viewer_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");

    // Attempt batches.create with viewer token -> should be forbidden!
    let (res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({ "batch": { "program_id": "dummy", "name": "Illegal Batch" } }),
        Some(&viewer_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");
}
