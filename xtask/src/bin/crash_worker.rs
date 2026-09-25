use coachify_core::db::{self, DbError};
use coachify_core::ledger::{self, CreatePaymentInput};
use rusqlite::{params, Connection, OptionalExtension};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut scenario = "payments".to_string();
    let mut action = "run".to_string();
    let mut db_path = PathBuf::from("crash_test.db");
    let mut backup_dir = PathBuf::from("backups");
    let mut iteration = 0usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scenario" => {
                i += 1;
                if i < args.len() {
                    scenario = args[i].clone();
                }
            }
            "--action" => {
                i += 1;
                if i < args.len() {
                    action = args[i].clone();
                }
            }
            "--db" => {
                i += 1;
                if i < args.len() {
                    db_path = PathBuf::from(&args[i]);
                }
            }
            "--backup-dir" => {
                i += 1;
                if i < args.len() {
                    backup_dir = PathBuf::from(&args[i]);
                }
            }
            "--iteration" => {
                i += 1;
                if i < args.len() {
                    iteration = args[i].parse().unwrap_or(0);
                }
            }
            _ => {}
        }
        i += 1;
    }

    match scenario.as_str() {
        "payments" => handle_payments_scenario(&action, &db_path, iteration),
        "backups" => handle_backups_scenario(&action, &db_path, &backup_dir, iteration),
        "migrations" => handle_migrations_scenario(&action, &db_path, iteration),
        other => {
            eprintln!("Unknown scenario: {}", other);
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Payments Crash & Replay
// ---------------------------------------------------------------------------

fn seed_payment_fixtures(conn: &Connection) -> Result<(), DbError> {
    // Check if test student already exists
    let student_exists: bool = conn
        .query_row(
            "SELECT 1 FROM students WHERE id = 'student-crash-test'",
            [],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !student_exists {
        let now = db::now_utc_iso();
        // Insert student
        conn.execute(
            "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, admission_date, status, created_at, updated_at)
             VALUES ('student-crash-test', 'CTS-001', 'Crash Test Student', 'male', '2005-01-01', '01711000000', 'Dhaka', '2026-09-01', 'active', ?1, ?1)",
            params![now],
        )?;

        // Insert fee head
        conn.execute(
            "INSERT OR IGNORE INTO fee_heads (id, name, kind, created_at, updated_at)
             VALUES ('fh-crash', 'Monthly Tuition', 'tuition', ?1, ?1)",
            params![now],
        )?;

        let b_date = db::current_business_date()?;

        // Insert open charge with 50,000 poisha (500 BDT)
        conn.execute(
            "INSERT INTO charges (id, student_id, fee_head_id, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
             VALUES ('charge-crash-test', 'student-crash-test', 'fh-crash', 50000, 0, 50000, ?1, 'manual', 'open', ?2, ?2)",
            params![b_date, now],
        )?;

        // Recalculate balance
        coachify_core::fees::recalculate_student_balance(conn, "student-crash-test")
            .map_err(|e| DbError::Config(e.to_string()))?;
    }

    Ok(())
}

fn handle_payments_scenario(action: &str, db_path: &Path, iteration: usize) {
    let idempotency_key = format!("crash-idem-{}", iteration);

    if action == "run" {
        let conn = db::open_connection_at(db_path).expect("open database");
        db::apply_all_migrations(&conn).expect("apply migrations");
        seed_payment_fixtures(&conn).expect("seed payment fixtures");

        // Execute payment creation inside an immediate transaction
        let mut tx_conn = db::open_connection_at(db_path).expect("open tx conn");
        let result: Result<(), DbError> = db::run_in_transaction(&mut tx_conn, |tx| {
            let input = CreatePaymentInput {
                student_id: "student-crash-test".to_string(),
                amount_poisha: 20000, // 200 BDT
                method: "cash".to_string(),
                reference: None,
                business_date: None,
                target_charge_ids: None,
                idempotency_key: idempotency_key.clone(),
                duplicate_override_reason: None,
                expected_balance_poisha: None,
            };

            let receipt = ledger::create_payment(tx, input, Some("cashier-1"), Some("device-1"))
                .map_err(|e| DbError::Config(e.to_string()))?;

            let audit_entry = coachify_core::audit::AuditEntry {
                action: "payments.create".to_string(),
                entity: "payment".to_string(),
                entity_id: receipt.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&receipt).unwrap()),
            };

            coachify_core::audit::record_audit_entry_with_user(
                tx,
                audit_entry,
                Some("cashier-1"),
                Some("device-1"),
                &receipt.created_at,
            )
            .map_err(|e| DbError::Config(e.to_string()))?;

            Ok(())
        });

        if let Err(e) = result {
            eprintln!("Payment run error: {}", e);
            std::process::exit(1);
        }
    } else if action == "verify" {
        let conn = db::open_connection_at(db_path).expect("open database on restart");
        db::verify_integrity(&conn).expect("PRAGMA integrity_check must pass after crash");

        // Check for partial rows
        let orphan_allocations: i64 = conn
            .query_row(
                "SELECT count(*) FROM payment_allocations WHERE payment_id NOT IN (SELECT id FROM payments)",
                [],
                |r| r.get(0),
            )
            .expect("check orphan allocations");
        assert_eq!(
            orphan_allocations, 0,
            "AC-1: Zero orphan payment allocations must exist after crash"
        );

        let existing_payment: Option<(String, String, i64)> = conn
            .query_row(
                "SELECT id, receipt_no, amount_poisha FROM payments WHERE idempotency_key = ?1",
                params![idempotency_key],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .expect("query payment by idempotency key");

        // Replay command with the same idempotency key
        let mut tx_conn = db::open_connection_at(db_path).expect("open replay conn");
        let replay_receipt = db::run_in_transaction(&mut tx_conn, |tx| {
            let input = CreatePaymentInput {
                student_id: "student-crash-test".to_string(),
                amount_poisha: 20000,
                method: "cash".to_string(),
                reference: None,
                business_date: None,
                target_charge_ids: None,
                idempotency_key: idempotency_key.clone(),
                duplicate_override_reason: None,
                expected_balance_poisha: None,
            };

            ledger::create_payment(tx, input, Some("cashier-1"), Some("device-1"))
                .map_err(|e| DbError::Config(e.to_string()))
        })
        .expect("replay payment must succeed");

        if let Some((_, original_receipt_no, _)) = existing_payment {
            assert_eq!(
                replay_receipt.receipt_no, original_receipt_no,
                "AC-1: Replaying already committed payment must return identical receipt number"
            );
        }

        // Verify there is exactly ONE payment with this idempotency key
        let payment_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM payments WHERE idempotency_key = ?1",
                params![idempotency_key],
                |r| r.get(0),
            )
            .expect("count payments with idempotency key");
        assert_eq!(
            payment_count, 1,
            "AC-1: Exactly one payment must exist after replay, zero duplicate charges allowed"
        );

        // Verify database integrity and audit chain
        db::verify_integrity(&conn).expect("final integrity check");
        coachify_core::audit::verify_audit_log_chain(&conn)
            .expect("AC-1: Audit chain must remain unbroken");

        println!("VERIFY_PAYMENTS_OK");
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Backups Crash & Sweep
// ---------------------------------------------------------------------------

fn handle_backups_scenario(action: &str, db_path: &Path, backup_dir: &Path, _iteration: usize) {
    if action == "run" {
        let conn = db::open_connection_at(db_path).expect("open database");
        db::apply_all_migrations(&conn).expect("apply migrations");
        seed_payment_fixtures(&conn).expect("seed fixtures");

        // Execute snapshot creation targeting backup_dir
        match coachify_core::backup::create_snapshot(&conn, backup_dir, "crash_test", "inst-1") {
            Ok(_) => println!("BACKUP_COMPLETED_UNEXPECTEDLY"),
            Err(e) => {
                eprintln!("Backup run error: {}", e);
                std::process::exit(1);
            }
        }
    } else if action == "verify" {
        // Startup sweep must clean up partial .tmp-backup-*.db files
        let swept_count = coachify_core::backup::sweep_temporary_backups(backup_dir)
            .expect("sweep temporary backups");
        println!("Swept {} temporary backup files", swept_count);

        // Verify that no .tmp-backup-* files remain
        if backup_dir.exists() {
            for entry in fs::read_dir(backup_dir).expect("read backup dir") {
                let entry = entry.expect("dir entry");
                let name = entry.file_name().to_string_lossy().to_string();
                assert!(
                    !name.starts_with(".tmp-backup-"),
                    "AC-2: Temporary backup file {:?} was not purged by startup sweep",
                    name
                );
            }
        }

        // Run a subsequent backup to completion and verify it succeeds
        let conn = db::open_connection_at(db_path).expect("open database for subsequent backup");
        let (final_path, counts, totals) = coachify_core::backup::create_snapshot(
            &conn,
            backup_dir,
            "post_crash_verified",
            "inst-1",
        )
        .expect("AC-2: Subsequent backup after crash must succeed");

        assert!(
            final_path.exists(),
            "Completed backup file must exist on disk"
        );
        assert!(
            counts.students > 0 || counts.charges > 0,
            "Backup record count must be greater than zero"
        );
        let _ = totals;

        println!("VERIFY_BACKUPS_OK");
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Migrations Crash & Sandbox Cleanup
// ---------------------------------------------------------------------------

fn handle_migrations_scenario(action: &str, db_path: &Path, _iteration: usize) {
    if action == "run" {
        // Prepare database at earlier migration version
        let conn = db::open_connection_at(db_path).expect("open database");
        db::apply_foundation_migrations(&conn).expect("apply foundation migrations");
        db::apply_accounts_migrations(&conn).expect("apply accounts migrations");
        db::apply_audit_migrations(&conn).expect("apply audit migrations");
        drop(conn);

        // Run migration staging engine with fault checkpoint
        match coachify_core::migrations::run_migrations_with_staging(db_path, None) {
            Ok(_) => println!("MIGRATIONS_COMPLETED_UNEXPECTEDLY"),
            Err(e) => {
                eprintln!("Migration run error: {}", e);
                std::process::exit(1);
            }
        }
    } else if action == "verify" {
        // Verify active database is intact and valid
        let conn = db::open_connection_at(db_path).expect("open database");
        db::verify_integrity(&conn).expect("AC-3: Target database must be intact after crash");
        drop(conn);

        // Run aborted migration recovery
        coachify_core::migrations::check_and_recover_interrupted_swap(db_path)
            .expect("recover aborted migration");

        // Verify staging file was discarded
        if let Some(parent) = db_path.parent() {
            let staging_path = parent.join("staging.db");
            assert!(
                !staging_path.exists(),
                "AC-3: Staging database must be purged on restart"
            );
        }

        // Re-run migrations to completion
        coachify_core::migrations::run_migrations_with_staging(db_path, None)
            .expect("AC-3: Migration re-run to completion must succeed");

        let final_conn = db::open_connection_at(db_path).expect("open final db");
        db::verify_integrity(&final_conn).expect("final migration integrity check");

        let status = coachify_core::migrations::get_migration_status(&final_conn)
            .expect("get migration status");
        assert_eq!(
            status.pending_migrations.len(),
            0,
            "AC-3: All migrations must be applied cleanly"
        );

        println!("VERIFY_MIGRATIONS_OK");
    }
}
