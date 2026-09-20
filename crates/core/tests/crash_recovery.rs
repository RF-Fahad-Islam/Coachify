use coachify_core::db::{self, DbError};
use rusqlite::params;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use tempfile::tempdir;
use uuid::Uuid;

// covers: AC-1, AC-2
#[test]
fn test_integrity_check_valid_database() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("clean.db");
    let conn = db::open_connection_at(&db_path).expect("open clean db");
    db::apply_foundation_migrations(&conn).expect("apply migrations");

    let result = db::verify_integrity(&conn);
    assert!(result.is_ok(), "Clean database must pass verify_integrity");
}

// covers: AC-2
#[test]
fn test_integrity_check_detects_corruption() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("corrupted.db");

    // 1. Create and populate a database
    {
        let conn = db::open_connection_at(&db_path).expect("open initial db");
        db::apply_foundation_migrations(&conn).expect("apply migrations");

        // Insert some data so pages are allocated
        for i in 1..=50 {
            let id = Uuid::now_v7().to_string();
            conn.execute(
                "INSERT INTO subjects (id, name, code, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
                params![id, format!("Subject {}", i), format!("CODE-{}", i)],
            )
            .expect("insert subject");
        }
    }

    // 2. Checkpoint and close database cleanly so WAL is flushed into the main db file
    {
        let conn = rusqlite::Connection::open(&db_path).expect("open raw");
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .expect("checkpoint");
    }

    // 3. Corrupt internal pages of the sqlite file (beyond the 100 byte SQLite header)
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&db_path)
            .expect("open db file for corruption");

        let len = file.metadata().expect("metadata").len();
        assert!(len > 512, "db file must be large enough to corrupt pages");

        // Seek past header to corrupt B-tree page content
        file.seek(SeekFrom::Start(120)).expect("seek");
        file.write_all(&[0xFF; 256]).expect("corrupt page bytes");
        file.flush().expect("flush");
    }

    // 4. verify_integrity, the check run_startup_checks performs at startup, must catch the corruption
    let raw_conn = rusqlite::Connection::open(&db_path).expect("open raw conn");
    let check_result = db::verify_integrity(&raw_conn);
    assert!(
        check_result.is_err(),
        "Corrupted database pages must be detected by verify_integrity"
    );
    if let Err(DbError::Integrity(msg)) = check_result {
        assert!(
            !msg.is_empty() && msg != "ok",
            "Corruption report must contain error details: {}",
            msg
        );
    } else {
        panic!("Expected DbError::Integrity on corrupted database");
    }
}

// covers: AC-2
#[test]
fn test_startup_check_passes_and_repeats_cheaply() {
    db::run_startup_checks().expect("startup integrity check passes on the dev database");
    // The scan is cached after the first success, so repeat calls stay cheap
    db::run_startup_checks().expect("repeat startup check must stay successful");
}

// covers: AC-1
#[test]
fn test_transaction_rollback_safety_leaves_zero_trace() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().join("rollback.db");
    let mut conn = db::open_connection_at(&db_path).expect("open db");
    db::apply_foundation_migrations(&conn).expect("apply migrations");

    let session_id = Uuid::now_v7().to_string();

    let fail_result: Result<(), DbError> = db::run_in_transaction(&mut conn, |tx| {
        tx.execute(
            "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
             VALUES (?1, '2099-ROLLBACK', '2099-01-01', '2099-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
            params![session_id],
        )?;

        // Simulate crash or error mid transaction
        Err(DbError::Config("simulated crash during write".to_string()))
    });

    assert!(fail_result.is_err(), "Transaction must fail as intended");

    // Verify uncommitted row was rolled back and left zero trace
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM academic_sessions WHERE name = '2099-ROLLBACK'",
            [],
            |row| row.get(0),
        )
        .expect("query rolled back session");
    assert_eq!(count, 0, "Uncommitted transaction must leave zero trace");

    // Subsequent transaction must succeed normally
    let success_result: Result<(), DbError> = db::run_in_transaction(&mut conn, |tx| {
        tx.execute(
            "INSERT INTO academic_sessions (id, name, start_date, end_date, created_at, updated_at)
             VALUES (?1, '2026-COMMITTED', '2026-01-01', '2026-12-31', '2026-09-20T10:00:00Z', '2026-09-20T10:00:00Z')",
            params![session_id],
        )?;
        Ok(())
    });
    assert!(
        success_result.is_ok(),
        "Next transaction must commit cleanly"
    );

    let count_committed: i64 = conn
        .query_row(
            "SELECT count(*) FROM academic_sessions WHERE name = '2026-COMMITTED'",
            [],
            |row| row.get(0),
        )
        .expect("query committed session");
    assert_eq!(count_committed, 1, "Committed session must persist");
}
