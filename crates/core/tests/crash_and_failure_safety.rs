use coachify_core::audit::{record_audit_entry, AuditEntry};
use coachify_core::db::{self, DbError};
use coachify_core::dispatcher::{self, RpcEnvelope};
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn setup_test_db() -> (TempDir, Connection) {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("test_crash_safety.db");
    let conn = db::open_connection_at(&db_path).expect("open connection");
    db::apply_all_migrations(&conn).expect("apply migrations");
    (dir, conn)
}

fn bootstrap_owner_and_login(conn: &mut Connection) -> String {
    let (res_str, _) = dispatcher::dispatch(
        conn,
        "system.bootstrap_owner",
        &json!({
            "username": "safety_owner",
            "display_name": "Safety Owner",
            "password": "Password123!"
        })
        .to_string(),
        None,
        None,
    );
    let envelope: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&res_str).expect("parse envelope");
    assert!(envelope.ok, "bootstrap owner failed: {:?}", envelope.error);

    let (login_str, _) = dispatcher::dispatch(
        conn,
        "auth.login",
        &json!({
            "username": "safety_owner",
            "password": "Password123!"
        })
        .to_string(),
        None,
        None,
    );
    let login_env: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&login_str).expect("parse login envelope");
    login_env.data.unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn test_clock_watermark_tolerance_and_rollback_guard_ac7() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    coachify_core::fault::reset_all_faults();

    let (_dir, mut conn) = setup_test_db();
    let _initial_token = bootstrap_owner_and_login(&mut conn);

    conn.execute(
        "INSERT INTO expense_categories (id, name, created_at, updated_at) VALUES ('cat-1', 'Utilities', datetime('now'), datetime('now'))",
        [],
    )
    .expect("seed category");

    // 1. Establish watermark
    let audit_entry = AuditEntry {
        action: "institution.update".to_string(),
        entity: "institution".to_string(),
        entity_id: "1".to_string(),
        before_json: None,
        after_json: Some(json!({"name": "Apex Coaching"})),
    };
    record_audit_entry(&conn, audit_entry, None, "2026-09-24T12:00:00Z")
        .expect("record watermark entry");

    let watermark: String = conn
        .query_row("SELECT MAX(at) FROM audit_log", [], |row| row.get(0))
        .expect("watermark");
    let water_ts: jiff::Timestamp = watermark.parse().unwrap();

    let t_minus_30 = water_ts
        .checked_sub(jiff::SignedDuration::from_secs(30))
        .unwrap()
        .to_string();
    let t_minus_60 = water_ts
        .checked_sub(jiff::SignedDuration::from_secs(60))
        .unwrap()
        .to_string();
    let t_minus_61 = water_ts
        .checked_sub(jiff::SignedDuration::from_secs(61))
        .unwrap()
        .to_string();
    let t_minus_2h = water_ts
        .checked_sub(jiff::SignedDuration::from_secs(7200))
        .unwrap()
        .to_string();

    // 2. Tolerance check: 30 seconds backward drift (<= 60s) is accepted
    assert!(db::check_clock_watermark(&conn, &t_minus_30).is_ok());
    assert!(db::is_clock_synchronized(&conn, &t_minus_30));

    // Exactly 60 seconds backward drift is accepted
    assert!(db::check_clock_watermark(&conn, &t_minus_60).is_ok());

    // 3. Rollback violation: 61 seconds and 2 hours backward drift must be rejected
    let err = db::check_clock_watermark(&conn, &t_minus_61).unwrap_err();
    match err {
        DbError::ClockDriftBackwards(msg) => {
            assert!(msg.contains("behind audit watermark by 61 seconds"));
        }
        other => panic!("expected ClockDriftBackwards error, got: {:?}", other),
    }
    assert!(!db::is_clock_synchronized(&conn, &t_minus_2h));

    // 4. Financial commands must be blocked when clock is rolled back beyond 60s
    coachify_core::fault::set_simulated_clock_offset_seconds(Some(-7200)); // 2 hours behind watermark
                                                                           // User logs in after computer restarted with rolled back clock (e.g. dead CMOS battery)
    let (login_str, _) = dispatcher::dispatch(
        &mut conn,
        "auth.login",
        &json!({
            "username": "safety_owner",
            "password": "Password123!"
        })
        .to_string(),
        None,
        None,
    );
    let login_env: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&login_str).expect("parse login envelope");
    let rolled_back_token = login_env.data.unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let expense_payload = json!({
        "expense": {
            "category_id": "cat-1",
            "amount_poisha": 50000,
            "payee": "Dhaka Electric",
            "method": "cash",
            "reference": null,
            "business_date": null,
            "attachment_vault_id": null
        }
    });

    let (res_str, _) = dispatcher::dispatch(
        &mut conn,
        "expenses.create",
        &expense_payload.to_string(),
        Some(&rolled_back_token),
        None,
    );
    let envelope: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&res_str).expect("parse envelope");
    assert!(!envelope.ok);
    let err = envelope.error.expect("expected error envelope");
    assert_eq!(err.code, "clock_out_of_sync");
    assert!(err.message.contains("Please synchronize computer clock"));

    // 5. Non-financial operations (reading students, system status) must proceed normally
    let (status_str, _) = dispatcher::dispatch(
        &mut conn,
        "system.get_system_status",
        "{}",
        Some(&rolled_back_token),
        None,
    );
    let status_env: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&status_str).expect("parse system status");
    assert!(
        status_env.ok,
        "read command must succeed even when clock is rolled back"
    );
    let status_data = status_env.data.unwrap();
    assert_eq!(status_data["clock_ok"], false);

    // 6. Owner unblocks via system.acknowledge_clock_jump
    let (ack_str, _) = dispatcher::dispatch(
        &mut conn,
        "system.acknowledge_clock_jump",
        &json!({
            "reason": "Motherboard CMOS battery replaced"
        })
        .to_string(),
        Some(&rolled_back_token),
        None,
    );
    let ack_env: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&ack_str).expect("parse acknowledge envelope");
    assert!(
        ack_env.ok,
        "Owner must be able to acknowledge clock jump: {:?}",
        ack_env.error
    );

    // 7. Watermark check now accepts the new timeline at or after the acknowledgment
    let curr = db::now_utc_iso();
    assert!(db::check_clock_watermark(&conn, &curr).is_ok());

    // Financial command now succeeds on the acknowledged timeline
    let (res2_str, _) = dispatcher::dispatch(
        &mut conn,
        "expenses.create",
        &expense_payload.to_string(),
        Some(&rolled_back_token),
        None,
    );
    let env2: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&res2_str).expect("parse envelope 2");
    assert!(
        env2.ok,
        "financial transaction must succeed after acknowledgment: {:?}",
        env2.error
    );

    coachify_core::fault::reset_all_faults();
}

#[test]
fn test_system_status_and_disk_thresholds_ac6() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    coachify_core::fault::reset_all_faults();

    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_owner_and_login(&mut conn);

    let (res_str, _) = dispatcher::dispatch(
        &mut conn,
        "system.get_system_status",
        "{}",
        Some(&token),
        None,
    );
    let envelope: RpcEnvelope<coachify_core::SystemStatusResponse> =
        serde_json::from_str(&res_str).expect("parse system status envelope");
    assert!(
        envelope.ok,
        "get_system_status failed: {:?}",
        envelope.error
    );
    let data = envelope.data.expect("status data");

    // Real system returns valid disk free bytes and synchronized clock
    assert!(data.db_disk_free_bytes > 0);
    assert!(data.backup_disk_free_bytes > 0);
    assert!(data.clock_ok);
}

#[test]
fn test_temporary_backup_sweep_ac2() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TempDir::new().expect("create temp dir");
    let backup_dir = dir.path().join("backups");
    fs::create_dir_all(&backup_dir).expect("create backup dir");

    // Seed temporary and valid backup files
    let tmp1 = backup_dir.join(".tmp-backup-daily-2026-09-24T12-00-00Z.db");
    let tmp1_journal = backup_dir.join(".tmp-backup-daily-2026-09-24T12-00-00Z.db-journal");
    let tmp2 = backup_dir.join(".tmp-backup-manual-2026-09-24T12-05-00Z.db");
    let valid_backup = backup_dir.join("backup-daily-2026-09-24T11-00-00Z.db");

    fs::write(&tmp1, b"partial db").unwrap();
    fs::write(&tmp1_journal, b"partial journal").unwrap();
    fs::write(&tmp2, b"partial db 2").unwrap();
    fs::write(&valid_backup, b"valid complete backup").unwrap();

    let swept = coachify_core::backup::sweep_temporary_backups(&backup_dir).expect("sweep");
    assert_eq!(swept, 3, "must sweep all 3 .tmp-backup-* files");

    assert!(!tmp1.exists());
    assert!(!tmp1_journal.exists());
    assert!(!tmp2.exists());
    assert!(
        valid_backup.exists(),
        "valid non-temporary backup must remain intact"
    );
}

#[cfg(feature = "fault-injection")]
#[test]
fn test_fault_injection_hooks_ac4_ac6() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    coachify_core::fault::reset_all_faults();

    let (_dir, conn) = setup_test_db();
    let temp_backup = TempDir::new().expect("temp backup dir");

    // 1. Simulate USB drive disconnect
    coachify_core::fault::set_simulate_usb_removal(true);
    let err =
        coachify_core::backup::create_snapshot(&conn, temp_backup.path(), "fault_test", "inst-1")
            .unwrap_err();
    match err {
        coachify_core::backup::BackupError::DestinationNotFound(msg) => {
            assert!(msg.contains("Simulated USB drive disconnect"));
        }
        other => panic!("expected DestinationNotFound, got: {:?}", other),
    }
    coachify_core::fault::set_simulate_usb_removal(false);

    // 2. Simulate Low Disk Space (< 500 MB refuses backup)
    coachify_core::fault::set_simulated_disk_free(Some(300 * 1024 * 1024)); // 300 MB
    let err =
        coachify_core::backup::create_snapshot(&conn, temp_backup.path(), "fault_test", "inst-1")
            .unwrap_err();
    match err {
        coachify_core::backup::BackupError::InsufficientSpace {
            required,
            available,
        } => {
            assert_eq!(required, 500 * 1024 * 1024);
            assert_eq!(available, 300 * 1024 * 1024);
        }
        other => panic!("expected InsufficientSpace, got: {:?}", other),
    }

    // Normal queries/transactions still work safely under low disk!
    let counts = coachify_core::backup::collect_record_counts(&conn).expect("counts");
    assert_eq!(counts.students, 0);

    // 3. Reset all faults
    coachify_core::fault::reset_all_faults();
    assert_eq!(coachify_core::fault::get_simulated_disk_free(), None);
    assert!(!coachify_core::fault::is_usb_removal_simulated());
    assert!(!coachify_core::fault::is_sms_timeout_simulated());
}
