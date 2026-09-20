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

#[test]
fn test_migration_0002_creates_all_tables() {
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
    assert!(max_version >= 2, "Schema version must be at least 2");
}

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
}

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
