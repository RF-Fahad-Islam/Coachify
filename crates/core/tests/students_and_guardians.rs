use coachify_core::crypto;
use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::phone::normalize_phone;
use rusqlite::params;
use serde_json::json;
use std::io::Cursor;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_students.db");
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
            "username": "admin",
            "display_name": "Admin Owner",
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
            "username": "admin",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

fn sample_student_json(name: &str, phone: &str, guardians: serde_json::Value) -> serde_json::Value {
    json!({
        "name": name,
        "gender": "male",
        "dob": "2008-01-01",
        "phone": phone,
        "present_address": "Dhaka, Bangladesh",
        "guardians": guardians
    })
}

fn sample_guardian_link(
    name: &str,
    phone: &str,
    is_primary: bool,
    receives_sms: bool,
) -> serde_json::Value {
    json!({
        "is_primary": is_primary,
        "receives_sms": receives_sms,
        "new_guardian": {
            "name": name,
            "relationship": "father",
            "phone": phone
        }
    })
}

fn create_sample_png_base64() -> String {
    use image::{ImageBuffer, Rgba};
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(50, 50);
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .expect("write png");
    crypto::base64_encode(&buf)
}

// =============================================================================
// AC-1: Student Registration & Auto-generated Code
// =============================================================================

#[test]
fn test_ac1_student_registration_and_code_generation() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Register student 1 without code -> auto-generates S{YEAR}-0001
    let s1 = sample_student_json(
        "Tanvir Ahmed",
        "01711-000001",
        json!([sample_guardian_link(
            "Kamal Ahmed",
            "01711-000002",
            true,
            true
        )]),
    );

    let (res1, events1) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s1 }),
        Some(&token),
        None,
    );
    assert_eq!(res1["ok"], true, "student 1 creation failed: {:?}", res1);
    let code1 = res1["data"]["student_code"].as_str().unwrap();
    assert!(
        code1.starts_with("S"),
        "code should start with S: {}",
        code1
    );
    assert!(
        code1.ends_with("-0001"),
        "first student should be -0001: {}",
        code1
    );
    assert_eq!(events1.len(), 1);
    assert_eq!(events1[0].topic, "students");

    // 2. Register student 2 without code -> auto-generates S{YEAR}-0002
    let s2 = sample_student_json(
        "Nafisa Kamal",
        "01711-000003",
        json!([sample_guardian_link(
            "Salma Begum",
            "01711-000004",
            true,
            true
        )]),
    );

    let (res2, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s2 }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], true);
    let code2 = res2["data"]["student_code"].as_str().unwrap();
    assert!(
        code2.ends_with("-0002"),
        "second student should be -0002: {}",
        code2
    );

    // 3. Register student with custom code
    let mut s3 = sample_student_json(
        "Rahat Kabir",
        "01711-000005",
        json!([sample_guardian_link(
            "Kabir Hossain",
            "01711-000006",
            true,
            true
        )]),
    );
    s3["student_code"] = json!("CUSTOM-9999");

    let (res3, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s3 }),
        Some(&token),
        None,
    );
    assert_eq!(res3["ok"], true);
    assert_eq!(res3["data"]["student_code"], "CUSTOM-9999");

    // 4. Duplicate custom code fails with student_code_exists
    let mut s4 = sample_student_json(
        "Duplicate Person",
        "01711-000007",
        json!([sample_guardian_link(
            "Parent Person",
            "01711-000008",
            true,
            true
        )]),
    );
    s4["student_code"] = json!("CUSTOM-9999");

    let (res4, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s4 }),
        Some(&token),
        None,
    );
    assert_eq!(res4["ok"], false);
    assert_eq!(res4["error"]["code"], "student_code_exists");
}

// =============================================================================
// AC-2: Primary Guardian & SMS Enforcement
// =============================================================================

#[test]
fn test_ac2_primary_guardian_and_sms_enforcement() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Zero guardians -> primary_guardian_missing
    let s1 = sample_student_json("No Guardian Student", "01711-111111", json!([]));
    let (res1, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s1 }),
        Some(&token),
        None,
    );
    assert_eq!(res1["ok"], false);
    assert_eq!(res1["error"]["code"], "primary_guardian_missing");

    // 2. Two guardians, none primary -> primary_guardian_missing
    let s2 = sample_student_json(
        "No Primary Guardian",
        "01711-111112",
        json!([
            sample_guardian_link("G1", "01711-111113", false, true),
            sample_guardian_link("G2", "01711-111114", false, false)
        ]),
    );
    let (res2, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s2 }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], false);
    assert_eq!(res2["error"]["code"], "primary_guardian_missing");

    // 3. Two guardians, both primary -> multiple_primary_guardians
    let s3 = sample_student_json(
        "Two Primary Guardians",
        "01711-111115",
        json!([
            sample_guardian_link("G1", "01711-111116", true, true),
            sample_guardian_link("G2", "01711-111117", true, false)
        ]),
    );
    let (res3, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s3 }),
        Some(&token),
        None,
    );
    assert_eq!(res3["ok"], false);
    assert_eq!(res3["error"]["code"], "multiple_primary_guardians");

    // 4. Exactly one primary guardian, but neither receives SMS -> sms_guardian_missing
    let s4 = sample_student_json(
        "No SMS Guardian",
        "01711-111118",
        json!([sample_guardian_link("G1", "01711-111119", true, false)]),
    );
    let (res4, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s4 }),
        Some(&token),
        None,
    );
    assert_eq!(res4["ok"], false);
    assert_eq!(res4["error"]["code"], "sms_guardian_missing");
}

// =============================================================================
// AC-3: Sibling Guardian Linking & Shared Guardian Updates
// =============================================================================

#[test]
fn test_ac3_sibling_linking_and_shared_guardian_split() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create first student with Father
    let s1 = sample_student_json(
        "Sibling Older",
        "01722-000001",
        json!([{
            "is_primary": true,
            "receives_sms": true,
            "new_guardian": {
                "name": "Tariqul Islam",
                "name_bn": "তারিকুল ইসলাম",
                "relationship": "father",
                "phone": "01722-000002",
                "present_address": "Dhanmondi, Dhaka"
            }
        }]),
    );

    let (res1, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s1 }),
        Some(&token),
        None,
    );
    assert_eq!(res1["ok"], true, "student 1 creation failed: {:?}", res1);
    let s1_id = res1["data"]["id"].as_str().unwrap().to_string();
    let g_id = res1["data"]["guardians"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Search guardians -> returns Tariqul Islam with student_count = 1
    let (g_search, _) = dispatch_cmd(
        &mut conn,
        "guardians.search",
        json!({ "query": "Tariqul" }),
        Some(&token),
        None,
    );
    assert_eq!(g_search["ok"], true);
    let matches = g_search["data"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["name"], "Tariqul Islam");
    assert_eq!(matches[0]["student_count"], 1);

    // 3. Create second student linking existing guardian ID
    let s2 = sample_student_json(
        "Sibling Younger",
        "01722-000003",
        json!([{
            "guardian_id": g_id,
            "relationship": "father",
            "is_primary": true,
            "receives_sms": true
        }]),
    );

    let (res2, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s2 }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], true, "student 2 creation failed: {:?}", res2);
    let s2_id = res2["data"]["id"].as_str().unwrap().to_string();

    // 4. Getting guardian details shows both siblings
    let (g_detail, _) = dispatch_cmd(
        &mut conn,
        "guardians.get",
        json!({ "id": g_id }),
        Some(&token),
        None,
    );
    assert_eq!(g_detail["ok"], true);
    let siblings = g_detail["data"]["siblings"].as_array().unwrap();
    assert_eq!(siblings.len(), 2);

    // 5. Update shared guardian without split -> updates both
    let (up_res1, _) = dispatch_cmd(
        &mut conn,
        "guardians.update",
        json!({
            "id": g_id,
            "guardian": {
                "name": "Tariqul Islam Updated",
                "relationship": "father",
                "phone": "01722-000002",
                "present_address": "Gulshan, Dhaka"
            },
            "split_from_siblings": false
        }),
        Some(&token),
        None,
    );
    assert_eq!(up_res1["ok"], true);

    // Check student 1 profile shows updated address
    let (s1_get, _) = dispatch_cmd(
        &mut conn,
        "students.get",
        json!({ "id": s1_id }),
        Some(&token),
        None,
    );
    assert_eq!(
        s1_get["data"]["student"]["guardians"][0]["name"],
        "Tariqul Islam Updated"
    );
    assert_eq!(
        s1_get["data"]["student"]["guardians"][0]["present_address"],
        "Gulshan, Dhaka"
    );

    // 6. Update shared guardian with split_from_siblings = true for student 2
    let (split_res, _) = dispatch_cmd(
        &mut conn,
        "guardians.update",
        json!({
            "id": g_id,
            "guardian": {
                "name": "Tariqul (Split for Younger)",
                "relationship": "father",
                "phone": "01722-999999",
                "present_address": "Banani, Dhaka"
            },
            "split_from_siblings": true,
            "student_id": s2_id
        }),
        Some(&token),
        None,
    );
    assert_eq!(split_res["ok"], true);

    // Student 1 still has the original guardian
    let (s1_check, _) = dispatch_cmd(
        &mut conn,
        "students.get",
        json!({ "id": s1_id }),
        Some(&token),
        None,
    );
    assert_eq!(s1_check["data"]["student"]["guardians"][0]["id"], g_id);
    assert_eq!(
        s1_check["data"]["student"]["guardians"][0]["name"],
        "Tariqul Islam Updated"
    );

    // Student 2 now has the new split guardian
    let (s2_check, _) = dispatch_cmd(
        &mut conn,
        "students.get",
        json!({ "id": s2_id }),
        Some(&token),
        None,
    );
    let s2_gid = s2_check["data"]["student"]["guardians"][0]["id"]
        .as_str()
        .unwrap();
    assert_ne!(
        s2_gid, g_id,
        "student 2 guardian should be detached into new record"
    );
    assert_eq!(
        s2_check["data"]["student"]["guardians"][0]["name"],
        "Tariqul (Split for Younger)"
    );
}

// =============================================================================
// AC-4: Phone Normalization & Duplicate Detection
// =============================================================================

#[test]
fn test_ac4_phone_normalization_and_duplicate_warning() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Test normalization: 01812-345678 -> 8801812345678
    assert_eq!(normalize_phone("01812-345678").unwrap(), "8801812345678");
    assert_eq!(
        normalize_phone("+880 1812-345678").unwrap(),
        "8801812345678"
    );

    // 2. Register first student with phone 01812-345678
    let s1 = sample_student_json(
        "First Phone Owner",
        "01812-345678",
        json!([sample_guardian_link("G1", "01812-999999", true, true)]),
    );

    let (res1, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s1 }),
        Some(&token),
        None,
    );
    assert_eq!(res1["ok"], true);

    // 3. Register second student with same normalized phone without bypass -> phone_conflict_warning
    let s2 = sample_student_json(
        "Second Phone Owner",
        "+880 1812 345678",
        json!([sample_guardian_link("G2", "01812-888888", true, true)]),
    );

    let (res2, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s2 }),
        Some(&token),
        None,
    );
    assert_eq!(res2["ok"], false);
    assert_eq!(res2["error"]["code"], "phone_conflict_warning");
    assert!(res2["error"]["message"]
        .as_str()
        .unwrap()
        .contains("First Phone Owner"));

    // 4. Register with allow_duplicate_phone = true -> succeeds
    let s3 = sample_student_json(
        "Second Phone Owner Allowed",
        "+880 1812 345678",
        json!([sample_guardian_link("G2", "01812-888888", true, true)]),
    );

    let (res3, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({
            "student": s3,
            "allow_duplicate_phone": true
        }),
        Some(&token),
        None,
    );
    assert_eq!(res3["ok"], true);
}

// =============================================================================
// AC-5: Vault Storage, Limits, and Retrieval
// =============================================================================

#[test]
fn test_ac5_vault_upload_limits_and_retrieval() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let png_base64 = create_sample_png_base64();

    // 1. Upload valid image
    let (res, _) = dispatch_cmd(
        &mut conn,
        "vault.upload_file",
        json!({
            "owner_type": "student",
            "owner_id": "test-student-id",
            "kind": "photo",
            "file_name": "student_photo.png",
            "mime_type": "image/png",
            "data_base64": png_base64
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "upload failed: {:?}", res);
    let doc_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["mime"], "image/png");

    // 2. Retrieve file via vault.get_file
    let (get_res, _) = dispatch_cmd(
        &mut conn,
        "vault.get_file",
        json!({ "id": doc_id }),
        Some(&token),
        None,
    );
    assert_eq!(get_res["ok"], true);
    assert_eq!(get_res["data"]["mime_type"], "image/png");
    assert!(!get_res["data"]["data_base64"].as_str().unwrap().is_empty());

    // 3. Reject unsupported format
    let exe_base64 = crypto::base64_encode(b"MZ executable header binary");
    let (bad_res, _) = dispatch_cmd(
        &mut conn,
        "vault.upload_file",
        json!({
            "owner_type": "student",
            "owner_id": "test-student-id",
            "kind": "document",
            "file_name": "virus.exe",
            "mime_type": "application/x-msdownload",
            "data_base64": exe_base64
        }),
        Some(&token),
        None,
    );
    assert_eq!(bad_res["ok"], false);
    assert_eq!(bad_res["error"]["code"], "unsupported_format");
}

// =============================================================================
// AC-6 & AC-7: FTS5 Substring Search, Trigram Index, & Sub-50ms Latency
// =============================================================================

#[test]
fn test_ac6_and_ac7_fts5_search_and_triggers() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create a student with Bengali and English names
    let mut s = sample_student_json(
        "Tahsin Mahmud",
        "01301-234567",
        json!([sample_guardian_link(
            "Mahmudur Rahman",
            "01301-888999",
            true,
            true
        )]),
    );
    s["name_bn"] = json!("তাহসিন মাহমুদ");
    s["class_level"] = json!("Class 10");
    s["current_school_roll"] = json!("R-101");

    let (create_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s }),
        Some(&token),
        None,
    );
    assert_eq!(create_res["ok"], true, "create failed: {:?}", create_res);
    let student_id = create_res["data"]["id"].as_str().unwrap().to_string();
    let student_code = create_res["data"]["student_code"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Search by student code substring
    let (search_code, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": &student_code[student_code.len()-4..] }),
        Some(&token),
        None,
    );
    assert_eq!(search_code["ok"], true);
    assert_eq!(search_code["data"]["matches"].as_array().unwrap().len(), 1);

    // 3. Search by English name substring
    let (search_name, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "Tahsin" }),
        Some(&token),
        None,
    );
    assert_eq!(search_name["ok"], true);
    assert_eq!(search_name["data"]["matches"].as_array().unwrap().len(), 1);

    // 4. Search by Bengali name substring
    let (search_bn, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "তাহসিন" }),
        Some(&token),
        None,
    );
    assert_eq!(search_bn["ok"], true);
    assert_eq!(search_bn["data"]["matches"].as_array().unwrap().len(), 1);

    // 5. Search by phone number substring
    let (search_phone, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "01301" }),
        Some(&token),
        None,
    );
    assert_eq!(search_phone["ok"], true);
    assert_eq!(search_phone["data"]["matches"].as_array().unwrap().len(), 1);

    // 6. Search by guardian name
    let (search_guardian, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "Mahmudur" }),
        Some(&token),
        None,
    );
    assert_eq!(search_guardian["ok"], true);
    assert_eq!(
        search_guardian["data"]["matches"].as_array().unwrap().len(),
        1
    );

    // 7. Verify Trigger Updates: Update student name -> old name should NOT match, new name SHOULD match
    let (up_res, _) = dispatch_cmd(
        &mut conn,
        "students.update",
        json!({
            "id": student_id,
            "expected_version": 1,
            "student": {
                "name": "Zubair Al Mahmud",
                "gender": "male",
                "dob": "2008-01-01",
                "phone": "01301-234567",
                "present_address": "Dhaka, Bangladesh"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(up_res["ok"], true, "update failed: {:?}", up_res);

    let (old_search, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "Tahsin" }),
        Some(&token),
        None,
    );
    assert_eq!(old_search["data"]["matches"].as_array().unwrap().len(), 0);

    let (new_search, _) = dispatch_cmd(
        &mut conn,
        "students.search",
        json!({ "query": "Zubair" }),
        Some(&token),
        None,
    );
    assert_eq!(new_search["data"]["matches"].as_array().unwrap().len(), 1);

    // 8. Performance test: 20 searches in quick succession should each complete well within 50ms
    let start = std::time::Instant::now();
    for _ in 0..20 {
        let (res, _) = dispatch_cmd(
            &mut conn,
            "students.search",
            json!({ "query": "Zubair" }),
            Some(&token),
            None,
        );
        assert_eq!(res["ok"], true);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / 20.0;
    assert!(
        avg_ms < 50.0,
        "Average search latency should be < 50ms, was {:.2}ms",
        avg_ms
    );
}

// =============================================================================
// AC-8: Student Lifecycle Transitions & Archive
// =============================================================================

#[test]
fn test_ac8_lifecycle_transitions_and_archive() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let s = sample_student_json(
        "Lifecycle Student",
        "01911-000001",
        json!([sample_guardian_link("G", "01911-000002", true, true)]),
    );

    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "creation failed: {:?}", res);
    let s_id = res["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(res["data"]["status"], "active");

    // 1. Archive student -> sets status to archived
    let (arch_res, _) = dispatch_cmd(
        &mut conn,
        "students.archive",
        json!({
            "id": s_id,
            "reason": "Family relocated to Chittagong",
            "expected_version": 1
        }),
        Some(&token),
        None,
    );
    assert_eq!(arch_res["ok"], true, "archive failed: {:?}", arch_res);
    assert_eq!(arch_res["data"]["status"], "archived");

    // 2. List students with status = archived returns the student
    let (list_archived, _) = dispatch_cmd(
        &mut conn,
        "students.list",
        json!({ "status": "archived" }),
        Some(&token),
        None,
    );
    assert_eq!(list_archived["ok"], true);
    assert_eq!(list_archived["data"]["total"], 1);

    // List with status = active does NOT return the student
    let (list_active, _) = dispatch_cmd(
        &mut conn,
        "students.list",
        json!({ "status": "active" }),
        Some(&token),
        None,
    );
    assert_eq!(list_active["ok"], true);
    assert_eq!(list_active["data"]["total"], 0);
}

// =============================================================================
// AC-9: Deletion Prevention with History Trigger
// =============================================================================

#[test]
fn test_ac9_deletion_prevention_trigger() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let s = sample_student_json(
        "History Student",
        "01611-000001",
        json!([sample_guardian_link("G", "01611-000002", true, true)]),
    );

    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true, "creation failed: {:?}", res);
    let s_id = res["data"]["id"].as_str().unwrap().to_string();

    // 1. Insert session, program, batch, and enrollment records for this student
    conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
         VALUES ('session-1', '2026', '2026-01-01', '2026-12-31', 1, datetime('now'), datetime('now'))",
        [],
    )
    .expect("insert session");

    conn.execute(
        "INSERT INTO programs (id, session_id, name, code, created_at, updated_at)
         VALUES ('prog-1', 'session-1', 'Test Program', 'TP-01', datetime('now'), datetime('now'))",
        [],
    )
    .expect("insert program");

    conn.execute(
        "INSERT INTO batches (id, program_id, name, code, start_date, created_at, updated_at)
         VALUES ('batch-1', 'prog-1', 'Batch 1', 'B-01', '2026-01-01', datetime('now'), datetime('now'))",
        [],
    )
    .expect("insert batch");

    conn.execute(
        "INSERT INTO enrollments (id, student_id, batch_id, start_date, created_at, updated_at)
         VALUES ('test-enrollment-1', ?1, 'batch-1', '2026-01-01', datetime('now'), datetime('now'))",
        params![s_id],
    )
    .expect("insert enrollment");

    // 2. Attempt direct SQL delete on student -> trigger trg_students_prevent_delete_with_history must abort
    let delete_result = conn.execute("DELETE FROM students WHERE id = ?1", params![s_id]);
    assert!(
        delete_result.is_err(),
        "Trigger must prevent deletion when enrollment history exists"
    );
    let err_msg = delete_result.unwrap_err().to_string();
    assert!(
        err_msg
            .contains("Cannot delete student with enrollment history; archive the student instead"),
        "Unexpected error: {}",
        err_msg
    );

    // 3. A student with zero enrollment/financial history can be deleted
    let s_clean = sample_student_json(
        "Deletable Student",
        "01611-000003",
        json!([sample_guardian_link("G", "01611-000004", true, true)]),
    );
    let (clean_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s_clean }),
        Some(&token),
        None,
    );
    let clean_id = clean_res["data"]["id"].as_str().unwrap();
    let del_ok = conn.execute("DELETE FROM students WHERE id = ?1", params![clean_id]);
    assert!(del_ok.is_ok(), "Student with no history can be deleted");
}

// =============================================================================
// AC-10: RBAC Permission Enforcement
// =============================================================================

#[test]
fn test_ac10_rbac_permission_enforcement() {
    let (_dir, mut conn) = setup_test_db();
    let admin_token = bootstrap_and_login(&mut conn);

    // 1. Create a role with ONLY students.view permission
    let (role_res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Viewer",
            "description": "Read-only viewer",
            "permission_codes": ["students.view"]
        }),
        Some(&admin_token),
        None,
    );
    assert_eq!(role_res["ok"], true);
    let role_id = role_res["data"]["id"].as_str().unwrap();

    // 2. Create a user with this viewer role
    let (user_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "viewer_user",
            "display_name": "Viewer User",
            "password": "Password123!",
            "role_ids": [role_id]
        }),
        Some(&admin_token),
        None,
    );
    assert_eq!(user_res["ok"], true);

    // 3. Log in as viewer_user
    let (login_res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "viewer_user",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let viewer_token = login_res["data"]["session_token"].as_str().unwrap();

    // 4. Viewer CAN list students
    let (list_res, _) = dispatch_cmd(
        &mut conn,
        "students.list",
        json!({}),
        Some(viewer_token),
        None,
    );
    assert_eq!(list_res["ok"], true);

    // 5. Viewer CANNOT create students -> forbidden
    let s = sample_student_json(
        "Unauthorized Student",
        "01755-000001",
        json!([sample_guardian_link("G", "01755-000002", true, true)]),
    );
    let (create_res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": s }),
        Some(viewer_token),
        None,
    );
    assert_eq!(create_res["ok"], false);
    assert_eq!(create_res["error"]["code"], "forbidden");

    // 6. Viewer CANNOT archive students -> forbidden
    let (arch_res, _) = dispatch_cmd(
        &mut conn,
        "students.archive",
        json!({
            "id": "any-id",
            "reason": "Test",
            "expected_version": 1
        }),
        Some(viewer_token),
        None,
    );
    assert_eq!(arch_res["ok"], false);
    assert_eq!(arch_res["error"]["code"], "forbidden");
}
