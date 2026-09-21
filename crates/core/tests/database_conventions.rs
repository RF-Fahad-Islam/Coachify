use coachify_core::db::{self, DbError};
use rusqlite::params;
use tempfile::tempdir;
use uuid::Uuid;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_foundation_migrations(&conn).expect("apply migrations");
    (dir, conn)
}

// covers: AC-1, AC-2
#[test]
fn test_pragma_settings_configured() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("pragma.db");
    let conn = db::open_connection_at(&db_path).expect("open connection");

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .expect("query journal_mode");
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let synchronous: i32 = conn
        .query_row("PRAGMA synchronous;", [], |row| row.get(0))
        .expect("query synchronous");
    assert_eq!(synchronous, 2, "synchronous must be 2 (FULL)");

    let foreign_keys: i32 = conn
        .query_row("PRAGMA foreign_keys;", [], |row| row.get(0))
        .expect("query foreign_keys");
    assert_eq!(foreign_keys, 1, "foreign_keys must be 1 (ON)");

    let busy_timeout: i32 = conn
        .query_row("PRAGMA busy_timeout;", [], |row| row.get(0))
        .expect("query busy_timeout");
    assert_eq!(busy_timeout, 5000, "busy_timeout must be 5000 ms");

    let integrity = db::verify_integrity(&conn);
    assert!(integrity.is_ok(), "quick check passes on fresh database");
}

// covers: AC-3
#[test]
fn test_foundation_migrations_create_all_tables() {
    let (_dir, conn) = setup_test_db();

    let expected_tables = vec![
        "institution",
        "academic_sessions",
        "clock_guard",
        "sequences",
        "settings",
        "students",
        "guardians",
        "student_guardians",
        "staff",
        "programs",
        "subjects",
        "batches",
        "batch_subjects",
        "enrollments",
        "documents",
        "fee_heads",
        "fee_plans",
        "concessions",
        "charges",
        "payments",
        "payment_allocations",
        "adjustments",
        "refunds",
        "cash_days",
        "expense_categories",
        "expenses",
        "student_balances",
        "audit_log",
        "_schema_version",
    ];

    for table in expected_tables {
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                params![table],
                |row| row.get(0),
            )
            .unwrap_or_else(|e| panic!("failed to check table {}: {}", table, e));
        assert_eq!(count, 1, "Table '{}' must exist in schema", table);
    }

    let max_version: i32 = conn
        .query_row("SELECT max(version) FROM _schema_version", [], |row| {
            row.get(0)
        })
        .expect("query schema version");
    assert_eq!(
        max_version, 6,
        "Schema version must be 6 after applying all 6 foundation migrations"
    );
}

// covers: AC-3
#[test]
fn test_foreign_key_enforcement() {
    let (_dir, conn) = setup_test_db();

    let fake_session_id = Uuid::now_v7().to_string();
    let program_id = Uuid::now_v7().to_string();

    let err = conn.execute(
        "INSERT INTO programs (id, session_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'HSC Science', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![program_id, fake_session_id],
    );
    assert!(
        err.is_err(),
        "Inserting program with invalid session_id must fail foreign key constraint"
    );

    let session_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
         VALUES (?1, '2026', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![session_id],
    ).expect("insert valid session");

    conn.execute(
        "INSERT INTO programs (id, session_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'HSC Science', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![program_id, session_id],
    )
    .expect("insert program with valid session");

    let del_err = conn.execute(
        "DELETE FROM academic_sessions WHERE id = ?1",
        params![session_id],
    );
    assert!(
        del_err.is_err(),
        "Deleting session referenced by program must fail with RESTRICT"
    );
}

// covers: AC-3
#[test]
fn test_unique_constraints_and_indexes() {
    let (_dir, conn) = setup_test_db();

    let session1_id = Uuid::now_v7().to_string();
    let session2_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
         VALUES (?1, '2026-SESSION', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![session1_id],
    ).expect("insert session 1");

    let dup_err = conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
         VALUES (?1, '2026-SESSION', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![session2_id],
    );
    assert!(
        dup_err.is_err(),
        "Duplicate academic session name must violate unique constraint"
    );

    let subj1_id = Uuid::now_v7().to_string();
    let subj2_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO subjects (id, name, code, created_at, updated_at)
         VALUES (?1, 'Physics 1st Paper', 'PHY-101', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![subj1_id],
    ).expect("insert subject 1");

    let dup_subj = conn.execute(
        "INSERT INTO subjects (id, name, code, created_at, updated_at)
         VALUES (?1, 'Physics 2nd Paper', 'PHY-101', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![subj2_id],
    );
    assert!(
        dup_subj.is_err(),
        "Duplicate subject code must violate unique constraint"
    );
}

// covers: AC-5
#[test]
fn test_sequence_allocation_atomic_formatting() {
    let (_dir, mut conn) = setup_test_db();

    let seq1 = db::run_in_transaction(&mut conn, |tx| {
        db::allocate_sequence(tx, "receipt:2026", "REC-2026-")
    })
    .expect("allocate seq1");
    assert_eq!(seq1, "REC-2026-0001");

    let seq2 = db::run_in_transaction(&mut conn, |tx| {
        db::allocate_sequence(tx, "receipt:2026", "REC-2026-")
    })
    .expect("allocate seq2");
    assert_eq!(seq2, "REC-2026-0002");

    let student_code = db::run_in_transaction(&mut conn, |tx| {
        db::allocate_sequence(tx, "student:2026", "S26-")
    })
    .expect("allocate student code");
    assert_eq!(student_code, "S26-0001");
}

// covers: AC-4
#[test]
fn test_clock_guard_backwards_and_forward_jumps() {
    let (_dir, mut conn) = setup_test_db();

    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-20T10:00:00Z", Some(24))
    })
    .expect("initial clock verification");

    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-20T11:00:00Z", Some(24))
    })
    .expect("advancing clock forward by 1 hour");

    let backwards_res = db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-20T09:00:00Z", Some(24))
    });
    assert!(
        matches!(backwards_res, Err(DbError::ClockDriftBackwards(_))),
        "Moving system time backwards must return ClockDriftBackwards error"
    );

    let direct_backwards_err = conn.execute(
        "UPDATE clock_guard SET high_water_at = '2026-09-20T08:00:00Z' WHERE id = 1",
        [],
    );
    assert!(
        direct_backwards_err.is_err(),
        "Direct backwards update on clock_guard must be aborted by SQLite trigger"
    );

    let jump_res = db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-22T12:00:00Z", Some(24))
    });
    assert!(
        matches!(jump_res, Err(DbError::ClockJumpForward(_))),
        "Clock jump of more than 24 hours without ack must return ClockJumpForward error"
    );
}

// covers: AC-4
#[test]
fn test_immutability_triggers_abort_modifications() {
    let (_dir, conn) = setup_test_db();

    // 1. audit_log immutability
    conn.execute(
        "INSERT INTO audit_log (at, business_date, action, entity, entity_id, prev_hash, hash)
         VALUES ('2026-09-20T10:00:00Z', '2026-09-20', 'create', 'student', 's1', '0000000000000000000000000000000000000000000000000000000000000000', 'mockhash')",
        [],
    ).expect("insert audit log");

    let audit_upd_err = conn.execute("UPDATE audit_log SET action = 'tampered' WHERE seq = 1", []);
    assert!(
        audit_upd_err.is_err(),
        "Updating audit_log must be aborted by trigger"
    );
    let audit_err_msg = audit_upd_err.unwrap_err().to_string();
    assert!(audit_err_msg.contains("Audit log records are immutable and cannot be updated"));

    let audit_del_err = conn.execute("DELETE FROM audit_log WHERE seq = 1", []);
    assert!(
        audit_del_err.is_err(),
        "Deleting audit_log must be aborted by trigger"
    );

    // 2. students row_version trigger
    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-001', 'Tanvir Ahmed', 'male', '2008-05-14', '8801711000000', 'Dhaka, Bangladesh', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let initial_version: i64 = conn
        .query_row(
            "SELECT row_version FROM students WHERE id = ?1",
            params![student_id],
            |row| row.get(0),
        )
        .expect("get initial row_version");
    assert_eq!(initial_version, 1);

    conn.execute(
        "UPDATE students SET name = 'Tanvir Ahmed Chowdhury', updated_at = '2026-09-20T11:00:00Z' WHERE id = ?1",
        params![student_id],
    ).expect("update student");

    let bumped_version: i64 = conn
        .query_row(
            "SELECT row_version FROM students WHERE id = ?1",
            params![student_id],
            |row| row.get(0),
        )
        .expect("get bumped row_version");
    assert_eq!(
        bumped_version, 2,
        "row_version must be automatically incremented by trigger"
    );

    // 3. payments immutability
    let payment_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-001', ?2, 250000, 'cash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-1', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![payment_id, student_id],
    ).expect("insert payment");

    let pay_del_err = conn.execute("DELETE FROM payments WHERE id = ?1", params![payment_id]);
    assert!(
        pay_del_err.is_err(),
        "Deleting posted payment must be aborted by trigger"
    );
    let pay_del_msg = pay_del_err.unwrap_err().to_string();
    assert!(pay_del_msg.contains("Posted payments cannot be deleted; void the payment instead"));

    let pay_upd_err = conn.execute(
        "UPDATE payments SET amount_poisha = 999999 WHERE id = ?1",
        params![payment_id],
    );
    assert!(
        pay_upd_err.is_err(),
        "Modifying amount of posted payment must be aborted by trigger"
    );

    conn.execute(
        "UPDATE payments SET status = 'voided', void_reason = 'Cashier mistake' WHERE id = ?1",
        params![payment_id],
    )
    .expect("voiding payment must be permitted");

    // 4. cash_days closed immutability
    conn.execute(
        "INSERT INTO cash_days (business_date, opening_cash_poisha, created_at, updated_at)
         VALUES ('2026-09-20', 100000, '2026-09-20T08:00:00Z', '2026-09-20T08:00:00Z')",
        [],
    )
    .expect("insert cash day");

    conn.execute(
        "UPDATE cash_days SET cash_in_poisha = 250000 WHERE business_date = '2026-09-20'",
        [],
    )
    .expect("open cash day update allowed");

    conn.execute(
        "UPDATE cash_days SET closed_at = '2026-09-20T20:00:00Z', closed_by = 'staff-1' WHERE business_date = '2026-09-20'",
        [],
    ).expect("close cash day");

    let closed_upd_err = conn.execute(
        "UPDATE cash_days SET opening_cash_poisha = 50000 WHERE business_date = '2026-09-20'",
        [],
    );
    assert!(closed_upd_err.is_err(), "Closed cash day cannot be updated");
    let closed_msg = closed_upd_err.unwrap_err().to_string();
    assert!(closed_msg.contains("Closed cash days are locked and cannot be modified"));

    // 5. charges amount immutability
    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Tuition', 'tuition', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    )
    .expect("insert fee head");

    let charge_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-09', 200000, 0, 200000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge_id, student_id, fee_head_id],
    )
    .expect("insert charge");

    let charge_upd_err = conn.execute(
        "UPDATE charges SET gross_amount_poisha = 300000 WHERE id = ?1",
        params![charge_id],
    );
    assert!(
        charge_upd_err.is_err(),
        "Modifying charge amounts must be aborted by trigger"
    );
    let charge_upd_msg = charge_upd_err.unwrap_err().to_string();
    assert!(charge_upd_msg.contains("Charge amounts and key attributes cannot be altered"));

    conn.execute(
        "UPDATE charges SET status = 'voided', void_reason = 'Wrong student' WHERE id = ?1",
        params![charge_id],
    )
    .expect("voiding charge is permitted");

    // 6. adjustments immutability
    let adj_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO adjustments (id, student_id, charge_id, type, amount_poisha, reason, business_date, created_at)
         VALUES (?1, ?2, ?3, 'waiver', -50000, 'Hardship', '2026-09-20', '2026-09-20T10:00:00Z')",
        params![adj_id, student_id, charge_id],
    )
    .expect("insert adjustment");

    let adj_upd_err = conn.execute(
        "UPDATE adjustments SET amount_poisha = -60000 WHERE id = ?1",
        params![adj_id],
    );
    assert!(
        adj_upd_err.is_err(),
        "Updating adjustment must be aborted by trigger"
    );
    let adj_upd_msg = adj_upd_err.unwrap_err().to_string();
    assert!(adj_upd_msg.contains("Adjustments are immutable and cannot be updated"));

    let adj_del_err = conn.execute("DELETE FROM adjustments WHERE id = ?1", params![adj_id]);
    assert!(
        adj_del_err.is_err(),
        "Deleting adjustment must be aborted by trigger"
    );
    let adj_del_msg = adj_del_err.unwrap_err().to_string();
    assert!(adj_del_msg.contains("Adjustments are immutable and cannot be deleted"));

    // 7. refunds immutability
    let refund_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO refunds (id, payment_id, amount_poisha, method, reason, business_date, created_at)
         VALUES (?1, ?2, 50000, 'cash', 'Overpayment refund', '2026-09-20', '2026-09-20T10:00:00Z')",
        params![refund_id, payment_id],
    )
    .expect("insert refund");

    let ref_upd_err = conn.execute(
        "UPDATE refunds SET amount_poisha = 60000 WHERE id = ?1",
        params![refund_id],
    );
    assert!(
        ref_upd_err.is_err(),
        "Updating refund must be aborted by trigger"
    );
    let ref_upd_msg = ref_upd_err.unwrap_err().to_string();
    assert!(ref_upd_msg.contains("Refunds are immutable and cannot be updated"));

    let ref_del_err = conn.execute("DELETE FROM refunds WHERE id = ?1", params![refund_id]);
    assert!(
        ref_del_err.is_err(),
        "Deleting refund must be aborted by trigger"
    );
    let ref_del_msg = ref_del_err.unwrap_err().to_string();
    assert!(ref_del_msg.contains("Refunds are immutable and cannot be deleted"));
}

// covers: AC-5
#[test]
fn test_financial_poisha_and_net_charge_constraints() {
    let (_dir, conn) = setup_test_db();

    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-002', 'Fatima Begum', 'female', '2009-02-18', '8801811000000', 'Dhaka, Bangladesh', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Monthly Tuition Fee', 'tuition', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    ).expect("insert fee head");

    let charge_id = Uuid::now_v7().to_string();
    let invalid_net_charge = conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-09', 300000, 50000, 200000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge_id, student_id, fee_head_id],
    );
    assert!(
        invalid_net_charge.is_err(),
        "net_amount_poisha must equal gross_amount_poisha - concession_amount_poisha"
    );

    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-09', 300000, 50000, 250000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge_id, student_id, fee_head_id],
    ).expect("insert valid charge with 2500 BDT (250000 poisha)");

    let charge_del_err = conn.execute("DELETE FROM charges WHERE id = ?1", params![charge_id]);
    assert!(
        charge_del_err.is_err(),
        "Deleting charge must be aborted by trg_charges_no_delete trigger"
    );
}

// covers: AC-6
#[test]
fn test_dual_time_and_business_date() {
    let utc_str = db::now_utc_iso();
    assert!(
        utc_str.contains('T'),
        "now_utc_iso must produce ISO 8601 timestamp with T"
    );

    let b_date = db::current_business_date().expect("current business date");
    assert_eq!(
        b_date.len(),
        10,
        "business_date must be YYYY-MM-DD formatted"
    );
    assert_eq!(&b_date[4..5], "-");
    assert_eq!(&b_date[7..8], "-");
}

// covers: AC-4
#[test]
fn test_payment_allocations_immutability_triggers() {
    let (_dir, conn) = setup_test_db();

    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-ALLOC-1', 'Rafiqul Islam', 'male', '2008-01-01', '8801911000000', 'Dhaka', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Tuition Fee', 'tuition', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    )
    .expect("insert fee head");

    let charge_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-09', 200000, 0, 200000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge_id, student_id, fee_head_id],
    ).expect("insert charge");

    let payment_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-ALLOC-1', ?2, 200000, 'bkash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-alloc-1', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![payment_id, student_id],
    ).expect("insert payment");

    let alloc_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO payment_allocations (id, payment_id, charge_id, amount_poisha, created_at)
         VALUES (?1, ?2, ?3, 200000, '2026-09-20T10:00:00Z')",
        params![alloc_id, payment_id, charge_id],
    )
    .expect("insert payment allocation");

    let upd_res = conn.execute(
        "UPDATE payment_allocations SET amount_poisha = 150000 WHERE id = ?1",
        params![alloc_id],
    );
    assert!(
        upd_res.is_err(),
        "Updating payment allocations must be rejected by trigger"
    );
    let upd_msg = upd_res.unwrap_err().to_string();
    assert!(upd_msg.contains("Payment allocations are immutable"));

    let del_res = conn.execute(
        "DELETE FROM payment_allocations WHERE id = ?1",
        params![alloc_id],
    );
    assert!(
        del_res.is_err(),
        "Deleting payment allocations must be rejected by trigger"
    );
    let del_msg = del_res.unwrap_err().to_string();
    assert!(del_msg.contains("Payment allocations cannot be deleted"));
}

// covers: AC-3
#[test]
fn test_single_row_tables_enforcement() {
    let (_dir, conn) = setup_test_db();

    // 1. Institution table CHECK (id = 1)
    conn.execute(
        "INSERT INTO institution (id, name, address, phones, created_at, updated_at)
         VALUES (1, 'Coachify Academy', 'Dhaka', '01700000000', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        [],
    ).expect("insert valid institution with id 1");

    let invalid_inst = conn.execute(
        "INSERT INTO institution (id, name, address, phones, created_at, updated_at)
         VALUES (2, 'Second Academy', 'Chittagong', '01800000000', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        [],
    );
    assert!(
        invalid_inst.is_err(),
        "Inserting institution with id other than 1 must fail check constraint"
    );

    // 2. Clock guard table CHECK (id = 1)
    let invalid_clock = conn.execute(
        "INSERT INTO clock_guard (id, high_water_at)
         VALUES (2, '2026-09-20T10:00:00Z')",
        [],
    );
    assert!(
        invalid_clock.is_err(),
        "Inserting clock guard with id other than 1 must fail check constraint"
    );
}

// covers: AC-3
#[test]
fn test_partial_unique_indexes_and_enrollment_lifecycle() {
    let (_dir, conn) = setup_test_db();

    let session_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
         VALUES (?1, '2026-ACTIVE-TEST', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![session_id],
    ).expect("insert session");

    let program_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO programs (id, session_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'Class 10 Science', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![program_id, session_id],
    )
    .expect("insert program");

    let batch_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO batches (id, program_id, name, code, start_date, created_at, updated_at)
         VALUES (?1, ?2, 'Morning Batch', 'MB-01', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![batch_id, program_id],
    ).expect("insert batch");

    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-ENROLL-1', 'Nusrat Jahan', 'female', '2008-04-12', '8801700000001', 'Dhaka', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let enroll1_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO enrollments (id, student_id, batch_id, start_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-01-10', 'active', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![enroll1_id, student_id, batch_id],
    ).expect("insert first active enrollment");

    let enroll2_id = Uuid::now_v7().to_string();
    let dup_active = conn.execute(
        "INSERT INTO enrollments (id, student_id, batch_id, start_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-02-01', 'active', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![enroll2_id, student_id, batch_id],
    );
    assert!(
        dup_active.is_err(),
        "Second active enrollment for the same student and batch must violate partial unique index"
    );

    conn.execute(
        "UPDATE enrollments SET status = 'ended', end_reason = 'completed' WHERE id = ?1",
        params![enroll1_id],
    )
    .expect("end first enrollment");

    conn.execute(
        "INSERT INTO enrollments (id, student_id, batch_id, start_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-02-01', 'active', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![enroll2_id, student_id, batch_id],
    ).expect("inserting active enrollment succeeds once earlier enrollment is ended");

    // Test generated charge unique period index
    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Exam Fee', 'exam', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    )
    .expect("insert fee head");

    let charge1_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, '2026-10', 50000, 0, 50000, '2026-10-15', 'generated', 'open', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge1_id, student_id, enroll2_id, fee_head_id],
    ).expect("insert generated open charge");

    let charge2_id = Uuid::now_v7().to_string();
    let dup_gen_charge = conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, '2026-10', 50000, 0, 50000, '2026-10-15', 'generated', 'open', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![charge2_id, student_id, enroll2_id, fee_head_id],
    );
    assert!(
        dup_gen_charge.is_err(),
        "Duplicate generated open charge for same enrollment and period must violate unique index"
    );

    let manual_charge_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, '2026-10', 25000, 0, 25000, '2026-10-15', 'manual', 'open', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![manual_charge_id, student_id, enroll2_id, fee_head_id],
    ).expect("manual charge with same period is permitted");
}

// covers: AC-3, AC-5
#[test]
fn test_fee_plans_exclusive_scope_and_concessions() {
    let (_dir, conn) = setup_test_db();

    let session_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
         VALUES (?1, '2026-PLAN-TEST', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![session_id],
    ).expect("insert session");

    let program_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO programs (id, session_id, name, created_at, updated_at)
         VALUES (?1, ?2, 'Class 9 General', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![program_id, session_id],
    )
    .expect("insert program");

    let batch_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO batches (id, program_id, name, code, start_date, created_at, updated_at)
         VALUES (?1, ?2, 'Batch A', 'BA-01', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![batch_id, program_id],
    ).expect("insert batch");

    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Admission Fee', 'admission', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    )
    .expect("insert fee head");

    // 1. Fee plan with both program_id and batch_id must fail XOR check
    let plan_both_id = Uuid::now_v7().to_string();
    let both_err = conn.execute(
        "INSERT INTO fee_plans (id, program_id, batch_id, fee_head_id, amount_poisha, frequency, effective_from, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 500000, 'once', '2026-01-01', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![plan_both_id, program_id, batch_id, fee_head_id],
    );
    assert!(
        both_err.is_err(),
        "Fee plan with both program and batch must fail check"
    );

    // 2. Fee plan with neither must fail XOR check
    let plan_neither_id = Uuid::now_v7().to_string();
    let neither_err = conn.execute(
        "INSERT INTO fee_plans (id, program_id, batch_id, fee_head_id, amount_poisha, frequency, effective_from, created_at, updated_at)
         VALUES (?1, NULL, NULL, ?2, 500000, 'once', '2026-01-01', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![plan_neither_id, fee_head_id],
    );
    assert!(
        neither_err.is_err(),
        "Fee plan with neither program nor batch must fail check"
    );

    // 3. Fee plan with negative amount must fail check
    let plan_neg_id = Uuid::now_v7().to_string();
    let neg_err = conn.execute(
        "INSERT INTO fee_plans (id, program_id, batch_id, fee_head_id, amount_poisha, frequency, effective_from, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, -100, 'once', '2026-01-01', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![plan_neg_id, program_id, fee_head_id],
    );
    assert!(
        neg_err.is_err(),
        "Fee plan with negative amount must fail check"
    );

    // 4. Valid fee plan with program only
    let plan_valid_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_plans (id, program_id, batch_id, fee_head_id, amount_poisha, frequency, effective_from, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, 500000, 'once', '2026-01-01', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![plan_valid_id, program_id, fee_head_id],
    ).expect("insert valid program fee plan");

    // 5. Concession check constraints (percent basis points vs fixed poisha)
    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-CONC-1', 'Amina Begum', 'female', '2008-07-21', '8801700000002', 'Dhaka', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let conc_valid_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO concessions (id, student_id, fee_head_id, type, value, reason, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'percent', 2500, 'Merit scholarship 25 percent', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![conc_valid_id, student_id, fee_head_id],
    ).expect("insert valid concession with 2500 basis points");

    let conc_zero_id = Uuid::now_v7().to_string();
    let zero_conc_err = conn.execute(
        "INSERT INTO concessions (id, student_id, fee_head_id, type, value, reason, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'percent', 0, 'Zero concession', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![conc_zero_id, student_id, fee_head_id],
    );
    assert!(
        zero_conc_err.is_err(),
        "Concession with zero value must fail check"
    );

    let conc_invalid_type_id = Uuid::now_v7().to_string();
    let type_err = conn.execute(
        "INSERT INTO concessions (id, student_id, fee_head_id, type, value, reason, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'discount', 1000, 'Invalid type', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![conc_invalid_type_id, student_id, fee_head_id],
    );
    assert!(
        type_err.is_err(),
        "Concession with invalid type must fail check"
    );
}

// covers: AC-5
#[test]
fn test_payment_method_and_amount_constraints() {
    let (_dir, conn) = setup_test_db();

    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-PAY-1', 'Sajid Hasan', 'male', '2008-09-15', '8801700000003', 'Dhaka', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    // 1. Payment amount must be strictly positive
    let pay_zero_id = Uuid::now_v7().to_string();
    let zero_pay_err = conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-PAY-0', ?2, 0, 'cash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-pay-0', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![pay_zero_id, student_id],
    );
    assert!(
        zero_pay_err.is_err(),
        "Payment with amount 0 must fail check"
    );

    let pay_neg_id = Uuid::now_v7().to_string();
    let neg_pay_err = conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-PAY-NEG', ?2, -50000, 'cash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-pay-neg', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![pay_neg_id, student_id],
    );
    assert!(
        neg_pay_err.is_err(),
        "Payment with negative amount must fail check"
    );

    // 2. Payment method must be one of the permitted options
    let pay_invalid_method_id = Uuid::now_v7().to_string();
    let method_err = conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-PAY-METH', ?2, 50000, 'crypto', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-pay-meth', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![pay_invalid_method_id, student_id],
    );
    assert!(
        method_err.is_err(),
        "Payment with invalid method must fail check"
    );

    // 3. Valid payment with bkash succeeds
    let pay_valid_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, created_at, updated_at)
         VALUES (?1, 'REC-PAY-OK', ?2, 120000, 'bkash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-pay-ok', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![pay_valid_id, student_id],
    ).expect("valid payment insert succeeds");
}

// covers: AC-4
#[test]
fn test_clock_guard_default_interval_ac4() {
    let (_dir, mut conn) = setup_test_db();

    // Initial record with None defaults to 24 hours
    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-20T10:00:00Z", None)
    })
    .expect("initial clock verification with default interval");

    // Advance 23 hours with None (within default 24h)
    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-21T09:00:00Z", None)
    })
    .expect("advancing within 24 hour default window succeeds");

    // Jump 26 hours ahead with None
    let jump_res = db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-22T12:00:00Z", None)
    });
    assert!(
        matches!(jump_res, Err(DbError::ClockJumpForward(hours)) if hours >= 24),
        "Clock jump of more than 24 hours with None default returns ClockJumpForward error"
    );
}

// covers: AC-4
#[test]
fn test_clock_guard_forward_jump_with_acknowledgment() {
    let (_dir, mut conn) = setup_test_db();

    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-20T10:00:00Z", Some(24))
    })
    .expect("initial clock verification");

    // Without an acknowledgment the oversized jump stays rejected
    let unacked = db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-25T10:00:00Z", Some(24))
    });
    assert!(
        matches!(unacked, Err(DbError::ClockJumpForward(_))),
        "Forward jump without acknowledgment must return ClockJumpForward error"
    );

    // An operator acknowledges the corrected clock, then the same jump is allowed
    db::run_in_transaction(&mut conn, |tx| {
        db::acknowledge_clock_jump(tx, "staff-1", "2026-09-25T09:00:00Z")
    })
    .expect("record clock jump acknowledgment");

    db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-25T10:00:00Z", Some(24))
    })
    .expect("acknowledged forward jump must be accepted");

    let (high_water, ack_by, ack_at): (String, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT high_water_at, last_ack_by, last_ack_at FROM clock_guard WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read clock guard");
    assert_eq!(
        high_water, "2026-09-25T10:00:00Z",
        "Accepted jump must advance the high water mark"
    );
    assert!(
        ack_by.is_none() && ack_at.is_none(),
        "Acknowledgment must be consumed after it authorizes one jump"
    );

    // The consumed acknowledgment cannot authorize a second jump
    let second_jump = db::run_in_transaction(&mut conn, |tx| {
        db::verify_clock(tx, "2026-09-30T10:00:00Z", Some(24))
    });
    assert!(
        matches!(second_jump, Err(DbError::ClockJumpForward(_))),
        "A consumed acknowledgment must not authorize a further forward jump"
    );

    let empty_actor = db::run_in_transaction(&mut conn, |tx| {
        db::acknowledge_clock_jump(tx, "   ", "2026-09-30T09:00:00Z")
    });
    assert!(
        matches!(empty_actor, Err(DbError::Config(_))),
        "Acknowledgment without an operator must be rejected"
    );
}

// covers: AC-4
#[test]
fn test_charge_period_null_transitions_are_blocked() {
    let (_dir, conn) = setup_test_db();

    let student_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, created_at, updated_at)
         VALUES (?1, 'STU-PERIOD-1', 'Nusrat Jahan', 'female', '2009-07-21', '8801611000000', 'Dhaka', '2026-01-10', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![student_id],
    ).expect("insert student");

    let fee_head_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO fee_heads (id, name, kind, created_at, updated_at)
         VALUES (?1, 'Exam Fee', 'exam', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![fee_head_id],
    )
    .expect("insert fee head");

    // A one time charge carries no period
    let one_time_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, NULL, 150000, 0, 150000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![one_time_id, student_id, fee_head_id],
    ).expect("insert one time charge");

    let null_to_value = conn.execute(
        "UPDATE charges SET period = '2026-09' WHERE id = ?1",
        params![one_time_id],
    );
    assert!(
        null_to_value.is_err(),
        "Setting a period on a one time charge must be aborted by trigger"
    );
    assert!(null_to_value
        .unwrap_err()
        .to_string()
        .contains("Charge amounts and key attributes cannot be altered"));

    // A periodic charge cannot lose its period either
    let periodic_id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, created_at, updated_at)
         VALUES (?1, ?2, ?3, '2026-09', 200000, 0, 200000, '2026-09-25', 'manual', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
        params![periodic_id, student_id, fee_head_id],
    ).expect("insert periodic charge");

    let value_to_null = conn.execute(
        "UPDATE charges SET period = NULL WHERE id = ?1",
        params![periodic_id],
    );
    assert!(
        value_to_null.is_err(),
        "Clearing the period of a periodic charge must be aborted by trigger"
    );

    // Voiding still works, so the trigger did not become a blanket update ban
    conn.execute(
        "UPDATE charges SET status = 'voided', void_reason = 'Duplicate charge' WHERE id = ?1",
        params![periodic_id],
    )
    .expect("voiding a charge must stay permitted");
}

// covers: AC-1
#[test]
fn test_resolve_dev_db_path() {
    let path = db::resolve_dev_db_path().expect("resolve dev db path");
    let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
    assert_eq!(
        file_name, "dev.db",
        "Resolved database path must end with dev.db"
    );
}
