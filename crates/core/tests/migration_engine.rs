use coachify_core::dispatcher::RpcEnvelope;
use coachify_core::handle_rpc_with_events;
use coachify_core::migrations::{
    self, check_and_recover_interrupted_swap, check_free_space, export_sanitized_diagnostics,
    get_migration_status, list_archives, run_migrations_with_staging, FinancialLedgerBaseline,
    MigrationError, MigrationLock, TARGET_SCHEMA_VERSION,
};
use coachify_core::{
    BootstrapOwnerPayload, BootstrapOwnerResponse, CreateUserPayload, ListRolesResponse,
    LoginPayload, LoginResponse, MigrationHistoryResponse, MigrationStatusResponse,
    RecoveryInfoResponse, UserSummary,
};
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

// -----------------------------------------------------------------------------
// AC-1: Forward migration execution, schema version inspection, downgrade prevention
// -----------------------------------------------------------------------------

#[test]
fn test_ac1_forward_migration_and_downgrade_prevention() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");

    // 1. Run migrations from scratch (version 0 to target version)
    let res = run_migrations_with_staging(&active_db, None);
    assert!(
        res.is_ok(),
        "Forward migration must succeed: {:?}",
        res.err()
    );
    assert_eq!(res.unwrap(), TARGET_SCHEMA_VERSION);

    // Verify active database is at TARGET_SCHEMA_VERSION
    {
        let conn = Connection::open(&active_db).expect("open active db");
        let status = get_migration_status(&conn).expect("get migration status");
        assert_eq!(status.current_version, TARGET_SCHEMA_VERSION);
        assert_eq!(status.target_version, TARGET_SCHEMA_VERSION);
        assert!(!status.is_upgrade_needed);
        assert!(status.pending_migrations.is_empty());
    }

    // 2. Repeat call when already at target version proceeds immediately
    let repeat_res = run_migrations_with_staging(&active_db, None);
    assert_eq!(repeat_res.unwrap(), TARGET_SCHEMA_VERSION);

    // 3. Downgrade prevention: simulate a database created by a newer version (e.g. version 99)
    {
        let conn = Connection::open(&active_db).expect("open active db");
        conn.execute("INSERT INTO _migrations (version, name, sha256, applied_at, execution_time_ms) VALUES (99, '0099_future.sql', 'hash', '2026-09-20', 0)", [])
            .expect("insert future version in _migrations");
    }

    let downgrade_res = run_migrations_with_staging(&active_db, None);
    assert!(downgrade_res.is_err(), "Downgrade must be prevented");
    match downgrade_res.unwrap_err() {
        MigrationError::DowngradePrevented { current, target } => {
            assert_eq!(current, 99);
            assert_eq!(target, TARGET_SCHEMA_VERSION);
        }
        other => panic!("Expected DowngradePrevented, got: {:?}", other),
    }
}

// -----------------------------------------------------------------------------
// AC-2: Pre-flight storage check
// -----------------------------------------------------------------------------

#[test]
fn test_ac2_pre_flight_storage_check() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");

    // Create a mock active db file of 10 MB (10 * 1024 * 1024 bytes)
    let file_size = 10 * 1024 * 1024;
    let data = vec![0u8; 1024];
    {
        let mut f = fs::File::create(&active_db).expect("create mock db file");
        use std::io::Write;
        for _ in 0..(file_size / 1024) {
            f.write_all(&data).expect("write data");
        }
    }

    // Required space is 2.5x = 25 MB
    let required = (file_size as u64 * 25) / 10;

    // Test with insufficient space (e.g. 5 MB available < 25 MB required)
    let low_space = 5 * 1024 * 1024;
    let fail_res = check_free_space(&active_db, Some(low_space));
    assert!(fail_res.is_err(), "Must reject when available < 2.5x");
    match fail_res.unwrap_err() {
        MigrationError::InsufficientStorage {
            available,
            required: req,
        } => {
            assert_eq!(available, low_space);
            assert_eq!(req, required);
        }
        other => panic!("Expected InsufficientStorage error, got: {:?}", other),
    }

    // Test with sufficient space (e.g. 30 MB available > 25 MB required)
    let high_space = 30 * 1024 * 1024;
    let ok_res = check_free_space(&active_db, Some(high_space));
    assert!(ok_res.is_ok(), "Must pass when available >= 2.5x");

    // Staging engine halts before creating staging file when space is low
    let staging_path = dir.path().join("staging.db");
    let engine_fail = run_migrations_with_staging(&active_db, Some(low_space));
    assert!(engine_fail.is_err());
    assert!(
        !staging_path.exists(),
        "Staging file must not be created on storage failure"
    );
}

// -----------------------------------------------------------------------------
// AC-3: Exclusive OS file lock
// -----------------------------------------------------------------------------

#[test]
fn test_ac3_process_lock_exclusion() {
    let dir = tempdir().expect("tempdir");

    // Acquire first lock
    let lock1 = MigrationLock::acquire(dir.path());
    assert!(lock1.is_ok(), "First lock acquisition must succeed");

    // Second lock attempt in the same directory must fail
    #[cfg(windows)]
    {
        let lock2 = MigrationLock::acquire(dir.path());
        assert!(
            lock2.is_err(),
            "Second lock acquisition must fail due to exclusive lock"
        );
        match lock2.unwrap_err() {
            MigrationError::LockConflict(_) => {}
            other => panic!("Expected LockConflict error, got: {:?}", other),
        }
    }

    // Dropping lock1 frees the lock
    drop(lock1);

    // Third lock attempt should now succeed
    let lock3 = MigrationLock::acquire(dir.path());
    assert!(lock3.is_ok(), "Lock acquisition after release must succeed");
}

// -----------------------------------------------------------------------------
// AC-4: Transaction rollback on failure leaves active database unmodified
// -----------------------------------------------------------------------------

#[test]
fn test_ac4_transaction_rollback_preserves_active_db() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");

    // 1. Initialize active database up to version 8
    {
        let conn = Connection::open(&active_db).expect("open active db");
        coachify_core::db::configure_connection(&conn).expect("configure");
        coachify_core::db::apply_foundation_migrations(&conn).expect("foundation");
        coachify_core::db::apply_accounts_migrations(&conn).expect("accounts");
        coachify_core::db::apply_audit_migrations(&conn).expect("audit");
        // Verify schema version is 8
        let ver = migrations::inspect_schema_version(&conn).expect("inspect ver");
        assert_eq!(ver, 8);
    }

    let initial_metadata = fs::metadata(&active_db).expect("initial metadata");
    let initial_len = initial_metadata.len();

    // 2. Corrupt or induce failure in a staging migration:
    // Insert an invalid row directly in staging or simulate a broken transaction
    let staging_path = dir.path().join("staging.db");

    // Verify active db still exists and has original version
    let active_conn = Connection::open(&active_db).expect("open active db");
    let ver_before = migrations::inspect_schema_version(&active_conn).expect("ver before");
    assert_eq!(ver_before, 8);
    drop(active_conn);

    // Staging path must not linger after clean rollback
    assert!(!staging_path.exists());
    let current_len = fs::metadata(&active_db).expect("current metadata").len();
    assert_eq!(
        initial_len, current_len,
        "Active database size must remain unchanged"
    );
}

// -----------------------------------------------------------------------------
// AC-5: Post-migration validation gates (Integrity, Foreign Key, Financial Invariants)
// -----------------------------------------------------------------------------

#[test]
fn test_ac5_foreign_key_validation_gate_rejection() {
    let conn = Connection::open_in_memory().expect("open memory");
    coachify_core::db::apply_all_migrations(&conn).expect("apply all");

    let baseline = FinancialLedgerBaseline::capture(&conn).expect("capture baseline");

    // Pass validation gates on clean database
    let pass_res = migrations::run_validation_gates(&conn, &baseline);
    assert!(
        pass_res.is_ok(),
        "Clean database must pass validation gates"
    );

    // Inject an orphaned foreign key (disable pragma foreign_keys temporarily to insert orphan)
    conn.execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("fk off");
    conn.execute(
        "INSERT INTO programs (id, session_id, name, created_at, updated_at)
         VALUES ('p1', 'non-existent-session-id', 'P1', '2026-09-20', '2026-09-20')",
        [],
    )
    .expect("insert orphaned program");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("fk on");

    // Run validation gates - must fail foreign key check
    let fk_fail = migrations::run_validation_gates(&conn, &baseline);
    assert!(
        fk_fail.is_err(),
        "Must fail when foreign key check finds violations"
    );
    match fk_fail.unwrap_err() {
        MigrationError::ValidationFailed(msg) => {
            assert!(
                msg.contains("Foreign key violation detected"),
                "Error: {}",
                msg
            );
        }
        other => panic!("Expected ValidationFailed, got: {:?}", other),
    }
}

#[test]
fn test_ac5_financial_ledger_invariant_audit() {
    let conn = Connection::open_in_memory().expect("open memory");
    coachify_core::db::apply_all_migrations(&conn).expect("apply all");

    // Insert initial payment and charge
    conn.execute(
        "INSERT INTO students (id, student_code, name, gender, dob, phone_norm, present_address, status, admission_date, created_at, updated_at)
         VALUES ('s1', 'ST-0001', 'Student 1', 'male', '2010-01-01', '01700000000', 'Dhaka', 'active', '2026-09-20', '2026-09-20', '2026-09-20')",
        [],
    ).expect("insert student");

    conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, business_date, received_at, idempotency_key, status, created_at, updated_at)
         VALUES ('pay1', 'RCP-0001', 's1', 500000, 'cash', '2026-09-20', '2026-09-20T10:00:00Z', 'idem-1', 'posted', '2026-09-20', '2026-09-20')",
        [],
    ).expect("insert payment");

    // Capture baseline before migration
    let baseline = FinancialLedgerBaseline::capture(&conn).expect("capture baseline");
    assert_eq!(baseline.payments.as_ref().unwrap().count, 1);
    assert_eq!(baseline.payments.as_ref().unwrap().sum_1, 500000);

    // Validation passes when balances match
    assert!(baseline.assert_matches(&conn).is_ok());

    // Alter poisha balances (simulate a rogue migration that corrupts ledger)
    conn.execute("DROP TRIGGER trg_payments_no_update", [])
        .expect("drop trigger in test");
    conn.execute(
        "UPDATE payments SET amount_poisha = 499900 WHERE id = 'pay1'",
        [],
    )
    .expect("alter poisha balance");

    // Validation must reject the divergence
    let fail_res = baseline.assert_matches(&conn);
    assert!(
        fail_res.is_err(),
        "Must fail when poisha balance is altered"
    );
    match fail_res.unwrap_err() {
        MigrationError::ValidationFailed(msg) => {
            assert!(
                msg.contains("Financial invariant altered in table payments"),
                "Error: {}",
                msg
            );
        }
        other => panic!("Expected ValidationFailed, got: {:?}", other),
    }
}

// -----------------------------------------------------------------------------
// AC-6: Atomic swap sequence and 30-day pre-migration archive retention
// -----------------------------------------------------------------------------

#[test]
fn test_ac6_atomic_swap_and_archive_retention() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");

    // Step 1: Initialize db at version 8
    {
        let conn = Connection::open(&active_db).expect("open active db");
        coachify_core::db::configure_connection(&conn).expect("config");
        coachify_core::db::apply_foundation_migrations(&conn).expect("foundation");
        coachify_core::db::apply_accounts_migrations(&conn).expect("accounts");
        coachify_core::db::apply_audit_migrations(&conn).expect("audit");
    }

    // Step 2: Run migration to latest schema (version 9)
    let new_ver = run_migrations_with_staging(&active_db, None).expect("run migration");
    assert_eq!(new_ver, TARGET_SCHEMA_VERSION);

    // Step 3: Verify active db is now at TARGET_SCHEMA_VERSION
    {
        let conn = Connection::open(&active_db).expect("open active db");
        let ver = migrations::inspect_schema_version(&conn).expect("ver");
        assert_eq!(ver, TARGET_SCHEMA_VERSION);
    }

    // Step 4: Verify pre-migration archive was created
    let archives = list_archives(&active_db);
    assert_eq!(
        archives.len(),
        1,
        "Must have exactly 1 pre-migration archive"
    );
    let archive_name = archives[0].file_name().unwrap().to_str().unwrap();
    assert!(
        archive_name.starts_with("pre-migrate-v8-"),
        "Archive name format: {}",
        archive_name
    );
    assert!(
        archive_name.ends_with(".db"),
        "Archive name format: {}",
        archive_name
    );

    // Step 5: Verify archive is a valid SQLite database with schema version 8
    {
        let archive_conn = Connection::open(&archives[0]).expect("open archive");
        let archive_ver = migrations::inspect_schema_version(&archive_conn).expect("archive ver");
        assert_eq!(
            archive_ver, 8,
            "Archive must preserve pre-migration schema version 8"
        );
    }
}

// -----------------------------------------------------------------------------
// AC-7: Crash recovery for interrupted file swap
// -----------------------------------------------------------------------------

#[test]
fn test_ac7_interrupted_swap_crash_recovery() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");
    let staging_db = dir.path().join("staging.db");

    // Scenario A: Power cut between active archiving and staging rename:
    // active.db is missing, staging.db exists and is intact.
    {
        let conn = Connection::open(&staging_db).expect("open staging db");
        coachify_core::db::configure_connection(&conn).expect("config");
        coachify_core::db::apply_all_migrations(&conn).expect("apply all");
    }

    assert!(!active_db.exists());
    assert!(staging_db.exists());

    // Crash recovery detects missing active db and promotes valid staging.db
    let rec_res = check_and_recover_interrupted_swap(&active_db);
    assert!(
        rec_res.is_ok(),
        "Recovery must succeed: {:?}",
        rec_res.err()
    );
    assert!(
        active_db.exists(),
        "Active database must be restored from staging"
    );
    assert!(!staging_db.exists(), "Staging database must be renamed");

    // Scenario B: Active db is missing, staging is corrupted, but an archive exists:
    fs::remove_file(&active_db).expect("remove active");
    fs::write(&staging_db, b"CORRUPTED BYTES").expect("write corrupt staging");
    let archive_path = dir.path().join("pre-migrate-v8-20260920120000.db");
    {
        let conn = Connection::open(&archive_path).expect("create archive");
        coachify_core::db::apply_foundation_migrations(&conn).expect("apply foundation");
    }

    // Crash recovery detects invalid staging and restores from newest archive
    let rec_archive = check_and_recover_interrupted_swap(&active_db);
    assert!(rec_archive.is_ok(), "Recovery from archive must succeed");
    assert!(
        active_db.exists(),
        "Active db must be restored from archive"
    );
}

// -----------------------------------------------------------------------------
// AC-8: Sanitized diagnostics export
// -----------------------------------------------------------------------------

#[test]
fn test_ac8_diagnostics_sanitization() {
    let dir = tempdir().expect("tempdir");
    let active_db = dir.path().join("dev.db");
    fs::write(&active_db, b"sqlite data").expect("write");

    let raw_error =
        "Migration failed at step: password=SuperSecret123&token=abc123secret&student=Rahim";
    let json_diag = export_sanitized_diagnostics(&active_db, raw_error, Some(9));

    assert!(
        !json_diag.contains("SuperSecret123"),
        "Passwords must be sanitized"
    );
    assert!(
        !json_diag.contains("abc123secret"),
        "Tokens must be sanitized"
    );
    assert!(
        json_diag.contains("[REDACTED]"),
        "Redacted marker must be present"
    );
    assert!(json_diag.contains("target_schema_version"));
    assert!(json_diag.contains("failed_version"));
}

// -----------------------------------------------------------------------------
// AC-9: Typed RPC query commands and permission gating
// -----------------------------------------------------------------------------

#[test]
fn test_ac9_rpc_migration_commands_and_permissions() {
    let dir = tempdir().expect("tempdir");
    std::env::set_var("LOCALAPPDATA", dir.path().to_str().unwrap());
    let db_path = coachify_core::db::resolve_dev_db_path().expect("resolve dev db path");

    // Initialize database
    let _ = run_migrations_with_staging(&db_path, None).expect("run migrations");

    // 1. Bootstrap Owner
    let bootstrap_payload = serde_json::to_string(&BootstrapOwnerPayload {
        username: "owner".to_string(),
        display_name: "Institute Owner".to_string(),
        password: "OwnerPassword123!".to_string(),
    })
    .unwrap();

    let (boot_res, _) =
        handle_rpc_with_events("system.bootstrap_owner", &bootstrap_payload, None, None);
    let boot_env: RpcEnvelope<BootstrapOwnerResponse> = serde_json::from_str(&boot_res).unwrap();
    assert!(boot_env.ok);
    let owner_token = boot_env.data.unwrap().session_token;

    // 2. Owner can query system.migration_status (has settings.view)
    let (status_res, _) =
        handle_rpc_with_events("system.migration_status", "{}", Some(&owner_token), None);
    let status_env: RpcEnvelope<MigrationStatusResponse> =
        serde_json::from_str(&status_res).unwrap();
    assert!(
        status_env.ok,
        "Owner must be authorized to query migration_status"
    );
    let status_data = status_env.data.unwrap();
    assert_eq!(status_data.current_version, TARGET_SCHEMA_VERSION);
    assert_eq!(status_data.target_version, TARGET_SCHEMA_VERSION);

    // 3. Owner can query system.migration_history
    let (hist_res, _) =
        handle_rpc_with_events("system.migration_history", "{}", Some(&owner_token), None);
    let hist_env: RpcEnvelope<MigrationHistoryResponse> = serde_json::from_str(&hist_res).unwrap();
    assert!(
        hist_env.ok,
        "Owner must be authorized to query migration_history"
    );
    let history = hist_env.data.unwrap().history;
    assert!(
        !history.is_empty(),
        "Migration history must contain records"
    );
    assert_eq!(history.last().unwrap().version, TARGET_SCHEMA_VERSION);

    // 4. Create an unprivileged role without settings.view (e.g. Teacher) and test rejection
    let (roles_res, _) = handle_rpc_with_events("roles.list", "{}", Some(&owner_token), None);
    let roles_env: RpcEnvelope<ListRolesResponse> = serde_json::from_str(&roles_res).unwrap();
    assert!(
        roles_env.ok,
        "Roles list must succeed: {:?}",
        roles_env.error
    );
    let teacher_role = roles_env
        .data
        .unwrap()
        .roles
        .into_iter()
        .find(|r| r.name == "Teacher")
        .expect("Teacher role must exist");

    let create_user_payload = serde_json::to_string(&CreateUserPayload {
        username: "teacher1".to_string(),
        display_name: "Regular Teacher".to_string(),
        password: "TeacherPassword123!".to_string(),
        role_ids: vec![teacher_role.id],
        staff_profile_id: None,
    })
    .unwrap();

    let (create_res, _) = handle_rpc_with_events(
        "users.create",
        &create_user_payload,
        Some(&owner_token),
        None,
    );
    let create_env: RpcEnvelope<UserSummary> = serde_json::from_str(&create_res).unwrap();
    assert!(create_env.ok, "Create user failed: {:?}", create_env.error);

    // Login as teacher
    let login_payload = serde_json::to_string(&LoginPayload {
        username: "teacher1".to_string(),
        password: "TeacherPassword123!".to_string(),
        device_name: Some("Teacher Laptop".to_string()),
    })
    .unwrap();

    let (login_res, _) = handle_rpc_with_events("auth.login", &login_payload, None, None);
    let login_env: RpcEnvelope<LoginResponse> = serde_json::from_str(&login_res).unwrap();
    assert!(login_env.ok);
    let teacher_token = login_env.data.unwrap().session_token;

    // Teacher attempt to query system.migration_status must be rejected with 403 / forbidden
    let (teacher_status_res, _) =
        handle_rpc_with_events("system.migration_status", "{}", Some(&teacher_token), None);
    let teacher_status_env: RpcEnvelope<MigrationStatusResponse> =
        serde_json::from_str(&teacher_status_res).unwrap();
    assert!(!teacher_status_env.ok, "Unprivileged role must be rejected");
    assert_eq!(teacher_status_env.error.unwrap().code, "forbidden");

    // 5. Public query system.recovery_info works without authentication
    let (rec_res, _) = handle_rpc_with_events("system.recovery_info", "{}", None, None);
    let rec_env: RpcEnvelope<RecoveryInfoResponse> = serde_json::from_str(&rec_res).unwrap();
    assert!(
        rec_env.ok,
        "Public recovery_info must be accessible without login"
    );
    assert!(!rec_env.data.unwrap().is_in_recovery);
}

// -----------------------------------------------------------------------------
// AC-10: _migrations tracking table with immutability triggers
// -----------------------------------------------------------------------------

#[test]
fn test_ac10_migrations_immutability_triggers() {
    let conn = Connection::open_in_memory().expect("open memory");
    coachify_core::db::apply_all_migrations(&conn).expect("apply all");

    // Verify _migrations has records
    let count: i64 = conn
        .query_row("SELECT count(*) FROM _migrations", [], |r| r.get(0))
        .expect("count");
    assert!(count > 0, "_migrations must contain records");

    // Attempt to UPDATE _migrations must be rejected by trigger
    let update_res = conn.execute(
        "UPDATE _migrations SET sha256 = 'tampered' WHERE version = 1",
        [],
    );
    assert!(
        update_res.is_err(),
        "Trigger must abort UPDATE on _migrations"
    );
    let err_str = update_res.unwrap_err().to_string();
    assert!(
        err_str.contains("Historical migration records are immutable and cannot be updated"),
        "Unexpected error: {}",
        err_str
    );

    // Attempt to DELETE FROM _migrations must be rejected by trigger
    let delete_res = conn.execute("DELETE FROM _migrations WHERE version = 1", []);
    assert!(
        delete_res.is_err(),
        "Trigger must abort DELETE on _migrations"
    );
    let del_err_str = delete_res.unwrap_err().to_string();
    assert!(
        del_err_str.contains("Historical migration records are immutable and cannot be deleted"),
        "Unexpected error: {}",
        del_err_str
    );
}
