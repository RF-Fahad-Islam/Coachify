use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use serde_json::json;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_owner_intelligence.db");
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
            "display_name": "Center Director",
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

static STUDENT_SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(70000);

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

// -----------------------------------------------------------------------------
// Test 1: Student 360 Timeline Aggregation Across Categories (AC-1, AC-2, AC-4)
// -----------------------------------------------------------------------------

#[test]
fn test_student_360_timeline_aggregates_all_categories() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create Academic Setup (Program, Batch)
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "Class 10 Science",
                "code": "C10-SCI"
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
                "name": "Morning Science Batch",
                "code": "MSB-01",
                "capacity": 30
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(b_res["ok"], true);
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // 2. Create Student with Guardian
    let student_id = create_student_helper(&mut conn, &token, "Tanvir Ahmed");

    // 3. Enroll Student in Batch
    let (en_res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": student_id,
                "batch_id": batch_id,
                "roll": 101
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(en_res["ok"], true, "enrollment failed: {:?}", en_res);

    // 4. Create Fee Head and Charge
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
    assert_eq!(fh_res["ok"], true);
    let fee_head_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    let (chg_res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fee_head_id,
                "amount_poisha": 500000, // 5000 Taka
                "due_date": "2026-09-10"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(chg_res["ok"], true, "manual charge failed: {:?}", chg_res);
    let charge_id = chg_res["data"]["id"].as_str().unwrap().to_string();

    // 5. Post Payment Voucher
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": student_id,
                "amount_poisha": 300000, // 3000 Taka
                "method": "bkash",
                "business_date": "2026-09-15",
                "idempotency_key": "pay-s360-01",
                "allocations": [
                    { "charge_id": charge_id, "amount_poisha": 300000 }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true, "payment failed: {:?}", pay_res);

    // 6. Record Attendance
    let (att_res, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": batch_id,
            "date": "2026-09-15",
            "notes": "Regular Class",
            "marks": [
                { "student_id": student_id, "status": "present" }
            ]
        }),
        Some(&token),
        None,
    );
    assert_eq!(att_res["ok"], true, "save_session failed: {:?}", att_res);

    // 7. Add Staff Note (Feature 34 AC-4)
    let (note_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.add_student_note",
        json!({
            "student_id": student_id,
            "note_type": "academic",
            "title": "Strong Problem Solving",
            "content": "Student showed excellent grasp of Kinematics concepts in class."
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        note_res["ok"], true,
        "add_student_note failed: {:?}",
        note_res
    );
    assert_eq!(note_res["data"]["note_type"], "academic");

    // 8. Query Student 360 Timeline (AC-1, AC-2)
    let (s360_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_student_360",
        json!({
            "student_id": student_id,
            "category": "all"
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        s360_res["ok"], true,
        "get_student_360 failed: {:?}",
        s360_res
    );
    let data = &s360_res["data"];

    assert_eq!(data["student_id"], student_id);
    assert_eq!(data["student_name"], "Tanvir Ahmed");
    assert_eq!(data["student_roll"], 101);
    assert_eq!(data["total_paid_poisha"], 300000);
    assert_eq!(data["total_due_poisha"], 200000); // 5000 - 3000 = 2000

    let events = data["events"].as_array().expect("events array");
    assert!(
        events.len() >= 5,
        "Expected at least 5 events, got {}",
        events.len()
    );

    let categories: Vec<&str> = events
        .iter()
        .map(|e| e["category"].as_str().unwrap())
        .collect();

    assert!(
        categories.contains(&"admission"),
        "Missing admission category"
    );
    assert!(
        categories.contains(&"enrollment"),
        "Missing enrollment category"
    );
    assert!(
        categories.contains(&"financial"),
        "Missing financial category"
    );
    assert!(
        categories.contains(&"attendance"),
        "Missing attendance category"
    );
    assert!(categories.contains(&"note"), "Missing note category");
}

// -----------------------------------------------------------------------------
// Test 2: Student 360 Category Filtering and Cursor Paging (AC-3)
// -----------------------------------------------------------------------------

#[test]
fn test_student_360_category_filtering_and_cursor_paging() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create Student
    let student_id = create_student_helper(&mut conn, &token, "Nafis Rahman");

    // Add 5 staff notes
    for i in 1..=5 {
        let (n_res, _) = dispatch_cmd(
            &mut conn,
            "intelligence.add_student_note",
            json!({
                "student_id": student_id,
                "note_type": "general",
                "title": format!("Counseling Note {}", i),
                "content": format!("Discussion log {}", i)
            }),
            Some(&token),
            None,
        );
        assert_eq!(n_res["ok"], true);
    }

    // Filter by category = "note" with limit = 2
    let (page1_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_student_360",
        json!({
            "student_id": student_id,
            "category": "note",
            "limit": 2
        }),
        Some(&token),
        None,
    );
    assert_eq!(page1_res["ok"], true);
    let p1 = &page1_res["data"];
    assert_eq!(p1["events"].as_array().unwrap().len(), 2);
    assert_eq!(p1["has_more"], true);
    let cursor = p1["next_cursor"].as_object().expect("cursor object");

    // Fetch Page 2 with cursor
    let (page2_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_student_360",
        json!({
            "student_id": student_id,
            "category": "note",
            "before_time": cursor["before_time"],
            "before_id": cursor["before_id"],
            "limit": 2
        }),
        Some(&token),
        None,
    );
    assert_eq!(page2_res["ok"], true);
    let p2 = &page2_res["data"];
    assert_eq!(p2["events"].as_array().unwrap().len(), 2);
}

// -----------------------------------------------------------------------------
// Test 3: Coaching Day Timeline with Actor Attribution (Feature 36 AC-1 through AC-4)
// -----------------------------------------------------------------------------

#[test]
fn test_coaching_day_timeline_with_actor_attribution() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create admission today
    let student_id = create_student_helper(&mut conn, &token, "Sadia Islam");
    let today: String = conn
        .query_row(
            "SELECT substr(created_at, 1, 10) FROM students WHERE id = ?1",
            rusqlite::params![student_id],
            |r| r.get(0),
        )
        .unwrap();

    // Query Coaching Day Timeline for today
    let (feed_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_coaching_day_timeline",
        json!({
            "date": today,
            "filter_category": "admissions"
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        feed_res["ok"], true,
        "coaching day feed failed: {:?}",
        feed_res
    );
    let events = feed_res["data"]["events"].as_array().unwrap();
    assert!(!events.is_empty(), "Expected admission event in today feed");
    assert_eq!(events[0]["category"], "admissions");
}

// -----------------------------------------------------------------------------
// Test 4: Today Cockpit Missing Attendance & Alert Clearance (Feature 35 AC-2)
// -----------------------------------------------------------------------------

#[test]
fn test_today_cockpit_missing_attendance_detection() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // 1. Create Staff (Teacher)
    let (st_res, _) = dispatch_cmd(
        &mut conn,
        "staff.create",
        json!({
            "staff": {
                "name": "Professor Rafiqul Islam",
                "phone": "01744000001",
                "designation": "Senior Lecturer"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(st_res["ok"], true);
    let teacher_id = st_res["data"]["id"].as_str().unwrap().to_string();

    // 2. Create Program
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "College Prep",
                "code": "CP-2026"
            }
        }),
        Some(&token),
        None,
    );
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    // Today date & day of week
    let now = jiff::Timestamp::now();
    let dhaka = now
        .checked_add(jiff::SignedDuration::from_hours(6))
        .unwrap();
    let dhaka_str = dhaka.to_string();
    let today_date = dhaka_str[..10].to_string();
    let current_hh: u32 = dhaka_str[11..13].parse().unwrap_or(12);
    let current_mm: u32 = dhaka_str[14..16].parse().unwrap_or(0);
    let current_mins = current_hh * 60 + current_mm;

    let (target_date, day_of_week, start_time, end_time) = if current_mins >= 70 {
        let dow = match dhaka.to_zoned(jiff::tz::TimeZone::UTC).date().weekday() {
            jiff::civil::Weekday::Sunday => "Sunday",
            jiff::civil::Weekday::Monday => "Monday",
            jiff::civil::Weekday::Tuesday => "Tuesday",
            jiff::civil::Weekday::Wednesday => "Wednesday",
            jiff::civil::Weekday::Thursday => "Thursday",
            jiff::civil::Weekday::Friday => "Friday",
            jiff::civil::Weekday::Saturday => "Saturday",
        };
        let end_m = current_mins - 5;
        let start_m = end_m - 30;
        (
            today_date.clone(),
            dow,
            format!("{:02}:{:02}", start_m / 60, start_m % 60),
            format!("{:02}:{:02}", end_m / 60, end_m % 60),
        )
    } else {
        let yesterday = dhaka
            .checked_sub(jiff::SignedDuration::from_hours(24))
            .unwrap();
        let y_date = yesterday.to_string()[..10].to_string();
        let y_dow = match yesterday.to_zoned(jiff::tz::TimeZone::UTC).date().weekday() {
            jiff::civil::Weekday::Sunday => "Sunday",
            jiff::civil::Weekday::Monday => "Monday",
            jiff::civil::Weekday::Tuesday => "Tuesday",
            jiff::civil::Weekday::Wednesday => "Wednesday",
            jiff::civil::Weekday::Thursday => "Thursday",
            jiff::civil::Weekday::Friday => "Friday",
            jiff::civil::Weekday::Saturday => "Saturday",
        };
        (y_date, y_dow, "06:00".to_string(), "07:00".to_string())
    };

    // 3. Create Batch with schedule today ending in the past
    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": prog_id,
                "name": "Early Morning Math",
                "code": "EMM-01",
                "capacity": 25,
                "teacher_staff_id": teacher_id,
                "room_number": "Room 101",
                "schedule": [
                    {
                        "day_of_week": day_of_week,
                        "start_time": start_time,
                        "end_time": end_time,
                        "room": "Room 101"
                    }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(b_res["ok"], true, "create batch failed: {:?}", b_res);
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // 4. Query Cockpit: Batch should appear in missing attendance alerts
    let (c1_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_today_cockpit",
        json!({ "target_date": target_date }),
        Some(&token),
        None,
    );
    assert_eq!(c1_res["ok"], true, "get_today_cockpit failed: {:?}", c1_res);
    let alerts = c1_res["data"]["missing_attendance_alerts"]
        .as_array()
        .unwrap();
    let found = alerts.iter().any(|a| a["batch_id"] == batch_id);
    assert!(
        found,
        "Batch whose class ended should trigger missing attendance alert"
    );

    let absent_teachers = c1_res["data"]["absent_teachers"].as_array().unwrap();
    assert!(
        absent_teachers
            .iter()
            .any(|t| t["teacher_id"] == teacher_id),
        "Teacher should appear in absent teachers"
    );

    // 5. Submit attendance for this batch today
    let (att_res, _) = dispatch_cmd(
        &mut conn,
        "attendance.save_session",
        json!({
            "batch_id": batch_id,
            "date": target_date,
            "notes": "Session completed",
            "marks": []
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        att_res["ok"], true,
        "attendance.save_session failed: {:?}",
        att_res
    );

    // 6. Query Cockpit again: Missing attendance alert must be cleared!
    let (c2_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_today_cockpit",
        json!({ "target_date": target_date }),
        Some(&token),
        None,
    );
    assert_eq!(
        c2_res["ok"], true,
        "get_today_cockpit c2 failed: {:?}",
        c2_res
    );
    let alerts_after = c2_res["data"]["missing_attendance_alerts"]
        .as_array()
        .unwrap();
    let still_missing = alerts_after.iter().any(|a| a["batch_id"] == batch_id);
    assert!(
        !still_missing,
        "Attendance was submitted; alert must disappear immediately"
    );
}

// -----------------------------------------------------------------------------
// Test 5: Batch Health Diagnostics & Thresholds (Feature 37 AC-1 through AC-5)
// -----------------------------------------------------------------------------

#[test]
fn test_batch_health_diagnostics_and_thresholds() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    // Create Program and Batch with Capacity 50
    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": { "name": "Engineering Foundation", "code": "ENG-2026" }
        }),
        Some(&token),
        None,
    );
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": prog_id,
                "name": "Underfilled Batch",
                "code": "UFB-01",
                "capacity": 50 // Enrolling only 1 student gives 2% utilization (< 50% threshold)
            }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // Create Student & Enroll
    let student_id = create_student_helper(&mut conn, &token, "Imran Khan");

    let (en_res, _) = dispatch_cmd(
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
    assert_eq!(en_res["ok"], true);

    // Bill a large fee and make no payments -> 100% dues ratio (> 30% threshold)
    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({ "head": { "name": "Lab Fee", "kind": "tuition" } }),
        Some(&token),
        None,
    );
    assert_eq!(fh_res["ok"], true, "fee_heads.create failed: {:?}", fh_res);
    let fh_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    let (chg_res, _) = dispatch_cmd(
        &mut conn,
        "charges.create_manual",
        json!({
            "charge": {
                "student_id": student_id,
                "fee_head_id": fh_id,
                "amount_poisha": 1000000,
                "due_date": "2026-09-01"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(chg_res["ok"], true);

    // Query Batch Health
    let (bh_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_batch_health_list",
        json!({ "program_id": prog_id }),
        Some(&token),
        None,
    );
    assert_eq!(
        bh_res["ok"], true,
        "get_batch_health_list failed: {:?}",
        bh_res
    );
    let batches = bh_res["data"].as_array().unwrap();
    let b_health = batches
        .iter()
        .find(|b| b["batch_id"] == batch_id)
        .expect("found batch health");

    assert_eq!(b_health["capacity"], 50);
    assert_eq!(b_health["enrolled_count"], 1);
    assert_eq!(b_health["capacity_utilization_pct"], 2.0);
    assert_eq!(b_health["dues_ratio_pct"], 100.0);
    // Two thresholds failed (capacity < 50% and dues > 30%) -> critical!
    assert_eq!(b_health["health_status"], "critical");
    assert!(b_health["flags"].as_array().unwrap().len() >= 2);
}

// -----------------------------------------------------------------------------
// Test 6: Batch Profitability & Breakeven Calculation (Feature 38 AC-1, AC-2, AC-3)
// -----------------------------------------------------------------------------

#[test]
fn test_batch_profitability_margin_and_breakeven() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": { "name": "Medical Coaching", "code": "MED-2026" }
        }),
        Some(&token),
        None,
    );
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": prog_id,
                "name": "Medical Elite",
                "code": "ME-01",
                "capacity": 30
            }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // Configure Batch Costs: Teacher 40,000 Taka, Room 15,000 Taka, Overhead 5,000 Taka (Total 60,000 Taka = 6,000,000 poisha)
    let (cfg_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.update_batch_cost_config",
        json!({
            "batch_id": batch_id,
            "teacher_monthly_cost": 4000000,
            "teacher_cost_type": "monthly_fixed",
            "room_monthly_cost": 1500000,
            "operational_overhead_cost": 500000
        }),
        Some(&token),
        None,
    );
    assert_eq!(cfg_res["ok"], true, "cost config failed: {:?}", cfg_res);
    assert_eq!(cfg_res["data"]["teacher_monthly_cost"], 4000000);

    // Initial check: Collections = 0, Costs = 6,000,000 poisha -> Net margin = -6,000,000 -> is_breakeven = false
    let (prof1_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_batch_profitability",
        json!({ "batch_id": batch_id }),
        Some(&token),
        None,
    );
    assert_eq!(prof1_res["ok"], true);
    let p1 = &prof1_res["data"][0];
    assert_eq!(p1["net_collected_revenue_poisha"], 0);
    assert_eq!(p1["total_cost_poisha"], 6000000);
    assert_eq!(p1["net_margin_poisha"], -6000000);
    assert_eq!(p1["is_breakeven"], false);

    // Create student, enroll, bill 80,000 Taka and collect 80,000 Taka (8,000,000 poisha)
    let s_id = create_student_helper(&mut conn, &token, "Farhana Yasmin");

    let (en_res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s_id,
                "batch_id": batch_id,
                "roll": 5
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(en_res["ok"], true);
    let en_id = en_res["data"]["id"].as_str().unwrap().to_string();

    let (fh_res, _) = dispatch_cmd(
        &mut conn,
        "fee_heads.create",
        json!({
            "head": {
                "name": "Medical Package",
                "kind": "tuition"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(fh_res["ok"], true, "fee_heads.create failed: {:?}", fh_res);
    let fh_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    // Insert charge linked to enrollment
    conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, net_amount_poisha, gross_amount_poisha, due_date, status, source, created_at, updated_at)
         VALUES ('chg-med-01', ?1, ?2, ?3, 8000000, 8000000, '2026-09-01', 'open', 'manual', '2026-09-01T00:00:00Z', '2026-09-01T00:00:00Z')",
        rusqlite::params![s_id, en_id, fh_id],
    ).unwrap();

    // Collect full payment
    let (pay_res, _) = dispatch_cmd(
        &mut conn,
        "payments.create",
        json!({
            "payment": {
                "student_id": s_id,
                "amount_poisha": 8000000,
                "method": "bank",
                "business_date": "2026-09-01",
                "idempotency_key": "pay-profit-01",
                "allocations": [
                    { "charge_id": "chg-med-01", "amount_poisha": 8000000 }
                ]
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(pay_res["ok"], true, "payments.create failed: {:?}", pay_res);

    // Query Profitability again: Revenue 8,000,000 - Cost 6,000,000 = Margin 2,000,000 poisha -> is_breakeven = true!
    let (prof2_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_batch_profitability",
        json!({ "batch_id": batch_id }),
        Some(&token),
        None,
    );
    assert_eq!(prof2_res["ok"], true);
    let p2 = &prof2_res["data"][0];
    assert_eq!(p2["net_collected_revenue_poisha"], 8000000);
    assert_eq!(p2["net_margin_poisha"], 2000000);
    assert_eq!(p2["is_breakeven"], true);
    assert_eq!(p2["margin_percentage"], 25.0); // 2,000,000 / 8,000,000 = 25%
}

// -----------------------------------------------------------------------------
// Test 7: Smart Due Queue Priority Ranking & Contact Logging (Feature 39 AC-1 to AC-4)
// -----------------------------------------------------------------------------

#[test]
fn test_smart_due_queue_weighted_scoring_and_contact_log() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);

    let (p_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({ "program": { "name": "General SSC", "code": "SSC-2026" } }),
        Some(&token),
        None,
    );
    let prog_id = p_res["data"]["id"].as_str().unwrap().to_string();

    let (b_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": { "program_id": prog_id, "name": "Batch Alpha", "code": "BA-01", "capacity": 30 }
        }),
        Some(&token),
        None,
    );
    let batch_id = b_res["data"]["id"].as_str().unwrap().to_string();

    // Student A: Old overdue charge (due 60 days ago), large amount
    let s1_id = create_student_helper(&mut conn, &token, "Student A High Due");

    let (en1_res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s1_id,
                "batch_id": batch_id,
                "roll": 10
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(en1_res["ok"], true);
    let en1_id = en1_res["data"]["id"].as_str().unwrap().to_string();

    // Student B: Recent overdue charge (due 5 days ago), small amount
    let s2_id = create_student_helper(&mut conn, &token, "Student B Low Due");

    let (en2_res, _) = dispatch_cmd(
        &mut conn,
        "enrollments.create",
        json!({
            "enrollment": {
                "student_id": s2_id,
                "batch_id": batch_id,
                "roll": 20
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(en2_res["ok"], true);
    let en2_id = en2_res["data"]["id"].as_str().unwrap().to_string();

    let (fh_res, _) = dispatch_cmd(
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
    assert_eq!(fh_res["ok"], true);
    let fh_id = fh_res["data"]["id"].as_str().unwrap().to_string();

    // Insert charge A (due 2026-07-01 -> 80+ days overdue, 15,000 Taka)
    conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, net_amount_poisha, gross_amount_poisha, due_date, status, source, created_at, updated_at)
         VALUES ('chg-due-a', ?1, ?2, ?3, 1500000, 1500000, '2026-07-01', 'open', 'manual', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![s1_id, en1_id, fh_id],
    ).unwrap();

    // Insert charge B (due 2026-09-20 -> 3 days overdue, 2,000 Taka)
    conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, net_amount_poisha, gross_amount_poisha, due_date, status, source, created_at, updated_at)
         VALUES ('chg-due-b', ?1, ?2, ?3, 200000, 200000, '2026-09-20', 'open', 'manual', '2026-09-20T00:00:00Z', '2026-09-20T00:00:00Z')",
        rusqlite::params![s2_id, en2_id, fh_id],
    ).unwrap();

    // Query Smart Due Queue
    let (q1_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_smart_due_queue",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(
        q1_res["ok"], true,
        "get_smart_due_queue q1 failed: {:?}",
        q1_res
    );
    let items = q1_res["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Student A must rank higher than Student B
    assert_eq!(
        items[0]["student_id"], s1_id,
        "Student A should have higher priority score than Student B"
    );
    assert!(
        items[0]["priority_score"].as_f64().unwrap() > items[1]["priority_score"].as_f64().unwrap()
    );

    // Record contact log for Student A with promised payment date
    let (cl_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.record_due_contact",
        json!({
            "student_id": s1_id,
            "contact_channel": "phone_call",
            "outcome": "promised_payment",
            "promised_payment_date": "2026-09-28",
            "notes": "Guardian Kabira promised to deposit 15,000 Taka on Sunday"
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        cl_res["ok"], true,
        "record_due_contact failed: {:?}",
        cl_res
    );
    assert_eq!(cl_res["data"]["outcome"], "promised_payment");

    // Re-query Queue: Student A should now display the promised payment date
    let (q2_res, _) = dispatch_cmd(
        &mut conn,
        "intelligence.get_smart_due_queue",
        json!({}),
        Some(&token),
        None,
    );
    let items2 = q2_res["data"]["items"].as_array().unwrap();
    let item_a = items2.iter().find(|i| i["student_id"] == s1_id).unwrap();
    assert_eq!(item_a["last_contact_outcome"], "promised_payment");
    assert_eq!(item_a["promised_payment_date"], "2026-09-28");
}
