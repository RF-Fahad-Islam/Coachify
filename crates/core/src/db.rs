use rusqlite::{Connection, OptionalExtension};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Database integrity corruption detected: {0}")]
    Integrity(String),
    #[error("Clock guard violation: system time cannot move backwards ({0})")]
    ClockDriftBackwards(String),
    #[error("Clock guard violation: unexpected forward jump of {0} hours without authorization")]
    ClockJumpForward(u32),
    #[error("Migration error: {0}")]
    Migration(String),
}

#[cfg(debug_assertions)]
pub const DEV_KEY: &str = "coachify_dev_debug_key_local_only";

#[cfg(not(debug_assertions))]
pub const DEV_KEY: &str = "";

pub const MIGRATION_0001_SQL: &str = include_str!("../../../migrations/0001_scaffold.sql");
pub const MIGRATION_0002_SQL: &str = include_str!("../../../migrations/0002_system_config.sql");
pub const MIGRATION_0003_SQL: &str = include_str!("../../../migrations/0003_people_directory.sql");
pub const MIGRATION_0004_SQL: &str =
    include_str!("../../../migrations/0004_academic_structure.sql");
pub const MIGRATION_0005_SQL: &str = include_str!("../../../migrations/0005_financial_ledger.sql");
pub const MIGRATION_0006_SQL: &str =
    include_str!("../../../migrations/0006_audit_and_triggers.sql");
pub const MIGRATION_0007_SQL: &str =
    include_str!("../../../migrations/0007_accounts_and_dispatcher.sql");
pub const MIGRATION_0008_SQL: &str =
    include_str!("../../../migrations/0008_audit_indexes_and_permissions.sql");
pub const MIGRATION_0009_SQL: &str = include_str!("../../../migrations/0009_migration_engine.sql");
pub const MIGRATION_0010_SQL: &str =
    include_str!("../../../migrations/0010_student_search_and_triggers.sql");
pub const MIGRATION_0011_SQL: &str =
    include_str!("../../../migrations/0011_academic_operations_and_triggers.sql");
pub const MIGRATION_0012_SQL: &str =
    include_str!("../../../migrations/0012_financial_ledger_refinements.sql");
pub const MIGRATION_0013_SQL: &str =
    include_str!("../../../migrations/0013_backup_infrastructure.sql");
pub const MIGRATION_0014_SQL: &str =
    include_str!("../../../migrations/0014_attendance_and_messaging.sql");
pub const MIGRATION_0015_SQL: &str = include_str!("../../../migrations/0015_exams_and_results.sql");
pub const MIGRATION_0016_SQL: &str =
    include_str!("../../../migrations/0016_role_dashboards_and_fast_search.sql");
pub const MIGRATION_0017_SQL: &str =
    include_str!("../../../migrations/0017_owner_intelligence.sql");
pub const MIGRATION_0018_SQL: &str =
    include_str!("../../../migrations/0018_operational_integrity.sql");
pub const MIGRATION_0019_SQL: &str =
    include_str!("../../../migrations/0019_professional_printing.sql");
pub const MIGRATION_0020_SQL: &str = include_str!("../../../migrations/0020_ai_foundation.sql");
pub const MIGRATION_0021_SQL: &str = include_str!("../../../migrations/0021_desktop_licensing.sql");
pub const MIGRATION_0022_SQL: &str =
    include_str!("../../../migrations/0022_license_refresh_state.sql");

pub fn resolve_dev_db_path() -> Result<PathBuf, DbError> {
    if let Ok(Some(cfg)) = crate::storage::load_global_config() {
        let p = PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("db")
            .join("main.db");
        if p.exists() {
            return Ok(p);
        }
    }

    let base = match env::var("LOCALAPPDATA") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            let home = env::var("USERPROFILE")
                .or_else(|_| env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".coachify")
        }
    };
    let dir = base.join("Coachify").join("dev");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("dev.db"))
}

pub fn verify_integrity(conn: &Connection) -> Result<(), DbError> {
    let mut stmt = conn
        .prepare("PRAGMA quick_check;")
        .map_err(|e| DbError::Integrity(e.to_string()))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| DbError::Integrity(e.to_string()))?;
    if let Some(row) = rows.next().map_err(|e| DbError::Integrity(e.to_string()))? {
        let result: String = row.get(0).map_err(|e| DbError::Integrity(e.to_string()))?;
        if result == "ok" {
            Ok(())
        } else {
            Err(DbError::Integrity(result))
        }
    } else {
        Err(DbError::Integrity(
            "No output from PRAGMA quick_check".to_string(),
        ))
    }
}

static STARTUP_INTEGRITY_OK: OnceLock<()> = OnceLock::new();
static SCHEMA_MIGRATED: OnceLock<Result<(), String>> = OnceLock::new();

/// Runs the startup integrity check for the application database.
///
/// The first successful call opens the database and executes `PRAGMA quick_check`.
/// Later calls in the same process return immediately without rescanning, so the
/// full database scan stays on the startup path and never runs per command.
/// A failure is not cached: the caller is expected to stop the application.
pub fn run_startup_checks() -> Result<(), DbError> {
    if STARTUP_INTEGRITY_OK.get().is_some() {
        return Ok(());
    }

    let db_path = resolve_dev_db_path()?;
    let conn = open_connection_at(&db_path)?;
    verify_integrity(&conn)?;

    if let Some(parent) = db_path.parent() {
        let backups_dir = parent.join("backups");
        if backups_dir.exists() {
            let _ = crate::backup::sweep_temporary_backups(&backups_dir);
        }
    }

    let _ = STARTUP_INTEGRITY_OK.set(());
    Ok(())
}

pub fn configure_connection(conn: &Connection) -> Result<(), DbError> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    Ok(())
}

pub fn open_connection_at(path: &Path) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    configure_connection(&conn)?;
    Ok(conn)
}

pub fn apply_migration_sql(conn: &Connection, sql: &str) -> Result<(), DbError> {
    conn.execute_batch(sql)?;
    Ok(())
}

pub fn apply_foundation_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0001_SQL)?;
    conn.execute_batch(MIGRATION_0002_SQL)?;
    conn.execute_batch(MIGRATION_0003_SQL)?;
    conn.execute_batch(MIGRATION_0004_SQL)?;
    conn.execute_batch(MIGRATION_0005_SQL)?;
    conn.execute_batch(MIGRATION_0006_SQL)?;
    Ok(())
}

pub fn apply_accounts_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0007_SQL)?;
    Ok(())
}

pub fn apply_audit_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0008_SQL)?;
    Ok(())
}

pub fn apply_migration_engine_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0009_SQL)?;
    let now = now_utc_iso();
    for def in crate::migrations::MIGRATIONS {
        conn.execute(
            "INSERT OR IGNORE INTO _migrations (version, name, sha256, applied_at, execution_time_ms)
             VALUES (?1, ?2, ?3, ?4, 0)",
            rusqlite::params![def.version, def.name, def.sha256, now],
        )?;
    }
    Ok(())
}

pub fn apply_student_search_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0010_SQL)?;
    Ok(())
}

pub fn apply_academic_operations_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0011_SQL)?;
    Ok(())
}

pub fn apply_financial_ledger_refinements_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0012_SQL)?;
    Ok(())
}

pub fn apply_backup_infrastructure_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0013_SQL)?;
    Ok(())
}

pub fn apply_attendance_and_messaging_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0014_SQL)?;
    Ok(())
}

pub fn apply_exams_and_results_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0015_SQL)?;
    Ok(())
}

pub fn apply_role_dashboards_and_fast_search_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0016_SQL)?;
    Ok(())
}

pub fn apply_owner_intelligence_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0017_SQL)?;
    Ok(())
}

pub fn apply_operational_integrity_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0018_SQL)?;
    Ok(())
}

pub fn apply_professional_printing_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0019_SQL)?;
    Ok(())
}

pub fn apply_ai_foundation_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0020_SQL)?;
    Ok(())
}

pub fn apply_desktop_licensing_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(MIGRATION_0021_SQL)?;
    conn.execute_batch(MIGRATION_0022_SQL)?;
    Ok(())
}

pub fn apply_all_migrations(conn: &Connection) -> Result<(), DbError> {
    apply_foundation_migrations(conn)?;
    apply_accounts_migrations(conn)?;
    apply_audit_migrations(conn)?;
    apply_migration_engine_migrations(conn)?;
    apply_student_search_migrations(conn)?;
    apply_academic_operations_migrations(conn)?;
    apply_financial_ledger_refinements_migrations(conn)?;
    apply_backup_infrastructure_migrations(conn)?;
    apply_attendance_and_messaging_migrations(conn)?;
    apply_exams_and_results_migrations(conn)?;
    apply_role_dashboards_and_fast_search_migrations(conn)?;
    apply_owner_intelligence_migrations(conn)?;
    apply_operational_integrity_migrations(conn)?;
    apply_professional_printing_migrations(conn)?;
    apply_ai_foundation_migrations(conn)?;
    apply_desktop_licensing_migrations(conn)?;
    Ok(())
}

use std::sync::RwLock;

static ACTIVE_DB_OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);

pub fn resolve_active_db_path() -> Result<PathBuf, DbError> {
    if let Ok(guard) = ACTIVE_DB_OVERRIDE.read() {
        if let Some(ref path) = *guard {
            return Ok(path.clone());
        }
    }
    resolve_dev_db_path()
}

pub fn get_active_db_path() -> Result<PathBuf, DbError> {
    resolve_active_db_path()
}

pub fn set_active_database_override(path: Option<PathBuf>) {
    if let Ok(mut guard) = ACTIVE_DB_OVERRIDE.write() {
        *guard = path;
    }
}

pub fn is_demo_database_active() -> bool {
    if let Ok(guard) = ACTIVE_DB_OVERRIDE.read() {
        if guard.is_some() {
            return true;
        }
    }
    if let Ok(active_path) = resolve_active_db_path() {
        if active_path.exists() {
            if let Ok(conn) = open_connection_at(&active_path) {
                let has_table: Result<i64, _> = conn.query_row(
                    "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'demo_metadata'",
                    [],
                    |row| row.get(0),
                );
                return has_table.is_ok();
            }
        }
    }
    false
}

pub fn switch_to_demo_database(target_path: &Path) -> Result<String, DbError> {
    if !target_path.exists() {
        return Err(DbError::Config(format!(
            "Target demo database file does not exist: {}",
            target_path.display()
        )));
    }

    let canonical_target = target_path
        .canonicalize()
        .map_err(|e| DbError::Config(format!("Failed to canonicalize target path: {}", e)))?;

    // Protection guard against targeting active production database (AC-9)
    let prod_path = resolve_dev_db_path()?;
    if let Ok(canonical_prod) = prod_path.canonicalize() {
        if canonical_target == canonical_prod {
            return Err(DbError::Config(
                "Cannot target active production database as demo database".to_string(),
            ));
        }
    }

    // Verify integrity of the new database (AC-11)
    let conn = open_connection_at(&canonical_target)?;
    verify_integrity(&conn)?;

    set_active_database_override(Some(canonical_target.clone()));
    Ok(canonical_target.display().to_string())
}

pub fn switch_to_production_database() -> Result<String, DbError> {
    set_active_database_override(None);
    let prod_path = resolve_dev_db_path()?;
    if prod_path.exists() {
        let conn = open_connection_at(&prod_path)?;
        verify_integrity(&conn)?;
    }
    Ok(prod_path.display().to_string())
}

pub fn open_database() -> Result<Connection, DbError> {
    let db_path = resolve_active_db_path()?;
    let is_demo = is_demo_database_active();

    if !is_demo {
        let envelope_path = db_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("key.envelope");

        if envelope_path.exists() {
            let envelope = crate::crypto::KeyEnvelope::load_from_file(&envelope_path)
                .map_err(|e| DbError::Config(format!("Failed to load key.envelope: {}", e)))?;
            let _master_key = envelope.unwrap_with_dpapi().map_err(|e| {
                DbError::Config(format!(
                    "Failed to unwrap database key via DPAPI: {}. Emergency recovery required.",
                    e
                ))
            })?;
        } else {
            #[cfg(not(debug_assertions))]
            {
                if DEV_KEY.is_empty() {
                    return Err(DbError::Config(
                        "Setup required: key envelope not found in production".to_string(),
                    ));
                }
            }
        }

        // Schema arrival goes through the same staging/validate/swap engine used at
        // startup, so a dev db that predates a schema change is brought current
        // with the same safety gates instead of a bare execute_batch. Gated to run
        // once per process (like run_startup_checks) since open_database is called
        // on every command and the engine already re-runs at each app launch.
        let migrated = SCHEMA_MIGRATED.get_or_init(|| {
            crate::migrations::run_migrations_with_staging(&db_path, None)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if let Err(e) = migrated {
            return Err(DbError::Migration(e.clone()));
        }
    }

    let conn = open_connection_at(&db_path)?;

    // Create scaffold health table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _scaffold_health (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            status TEXT NOT NULL,
            pinged_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

pub fn run_in_transaction<F, T>(conn: &mut Connection, f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Transaction) -> Result<T, DbError>,
{
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let result = f(&tx)?;
    tx.commit()?;
    Ok(result)
}

pub fn allocate_sequence(
    conn: &rusqlite::Connection,
    name: &str,
    prefix: &str,
) -> Result<String, DbError> {
    conn.execute(
        "INSERT INTO sequences (name, prefix, next_value)
         VALUES (?1, ?2, 1)
         ON CONFLICT(name) DO NOTHING",
        rusqlite::params![name, prefix],
    )?;

    let (db_prefix, val): (String, i64) = conn.query_row(
        "SELECT prefix, next_value FROM sequences WHERE name = ?1",
        rusqlite::params![name],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    conn.execute(
        "UPDATE sequences SET next_value = next_value + 1 WHERE name = ?1",
        rusqlite::params![name],
    )?;

    let effective_prefix = if db_prefix.is_empty() {
        prefix
    } else {
        &db_prefix
    };

    Ok(format!("{}{:04}", effective_prefix, val))
}

/// Records an operator acknowledgment that authorizes one oversized forward
/// clock jump.
///
/// The acknowledgment names the operator and carries the wall clock time it was
/// given at. [`verify_clock`] honours it once and then clears it, so every
/// further oversized jump needs a fresh acknowledgment.
pub fn acknowledge_clock_jump(
    conn: &rusqlite::Connection,
    actor: &str,
    acknowledged_at: &str,
) -> Result<(), DbError> {
    if actor.trim().is_empty() {
        return Err(DbError::Config(
            "Clock jump acknowledgment requires an operator".to_string(),
        ));
    }

    let _: jiff::Timestamp = acknowledged_at.parse().map_err(|e| {
        DbError::Config(format!(
            "Invalid acknowledged_at timestamp {acknowledged_at}: {e}"
        ))
    })?;

    let updated = conn.execute(
        "UPDATE clock_guard SET high_water_at = ?2, last_ack_by = ?1, last_ack_at = ?2 WHERE id = 1",
        rusqlite::params![actor, acknowledged_at],
    )?;

    if updated == 0 {
        conn.execute(
            "INSERT INTO clock_guard (id, high_water_at, last_ack_by, last_ack_at)
             VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![acknowledged_at, actor, acknowledged_at],
        )?;
    }

    Ok(())
}

/// Checks the monotonic clock guard before a write and advances its high water mark.
///
/// Any timestamp earlier than the recorded high water mark is rejected. A forward
/// jump larger than `max_forward_hours` (24 when `None`) is rejected too, unless an
/// acknowledgment recorded by [`acknowledge_clock_jump`] covers it: the
/// acknowledgment must name an operator and be dated at or after the current high
/// water mark. A covering acknowledgment is consumed once it lets a jump through.
pub fn verify_clock(
    tx: &rusqlite::Transaction,
    current_utc: &str,
    max_forward_hours: Option<u32>,
) -> Result<(), DbError> {
    let guard: Option<(String, Option<String>, Option<String>)> = tx
        .query_row(
            "SELECT high_water_at, last_ack_by, last_ack_at FROM clock_guard WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    let Some((high_water_at, last_ack_by, last_ack_at)) = guard else {
        tx.execute(
            "INSERT INTO clock_guard (id, high_water_at) VALUES (1, ?1)",
            rusqlite::params![current_utc],
        )?;
        return Ok(());
    };

    let curr_ts: jiff::Timestamp = current_utc.parse().map_err(|e| {
        DbError::Config(format!("Invalid current_utc timestamp {current_utc}: {e}"))
    })?;
    let high_ts: jiff::Timestamp = high_water_at.parse().map_err(|e| {
        DbError::Config(format!(
            "Invalid high_water_at timestamp {high_water_at}: {e}"
        ))
    })?;

    let diff_seconds = curr_ts.as_second() - high_ts.as_second();
    if diff_seconds < 0 {
        return Err(DbError::ClockDriftBackwards(format!(
            "Current timestamp {current_utc} is earlier than recorded high water mark {high_water_at}"
        )));
    }

    let max_seconds = (max_forward_hours.unwrap_or(24) as i64) * 3600;
    if diff_seconds > max_seconds {
        if !jump_is_acknowledged(last_ack_by.as_deref(), last_ack_at.as_deref(), high_ts)? {
            return Err(DbError::ClockJumpForward((diff_seconds / 3600) as u32));
        }

        // The acknowledgment authorizes this jump only, so it is cleared here.
        tx.execute(
            "UPDATE clock_guard
             SET high_water_at = ?1, last_ack_by = NULL, last_ack_at = NULL
             WHERE id = 1",
            rusqlite::params![current_utc],
        )?;
        return Ok(());
    }

    tx.execute(
        "UPDATE clock_guard SET high_water_at = ?1 WHERE id = 1",
        rusqlite::params![current_utc],
    )?;

    Ok(())
}

fn jump_is_acknowledged(
    last_ack_by: Option<&str>,
    last_ack_at: Option<&str>,
    high_ts: jiff::Timestamp,
) -> Result<bool, DbError> {
    let (Some(actor), Some(ack_at)) = (last_ack_by, last_ack_at) else {
        return Ok(false);
    };
    if actor.trim().is_empty() {
        return Ok(false);
    }

    let ack_ts: jiff::Timestamp = ack_at
        .parse()
        .map_err(|e| DbError::Config(format!("Invalid last_ack_at timestamp {ack_at}: {e}")))?;

    Ok(ack_ts.as_second() >= high_ts.as_second())
}

pub fn now_utc_iso() -> String {
    #[cfg(feature = "fault-injection")]
    if let Some(offset) = crate::fault::get_simulated_clock_offset_seconds() {
        let ts = jiff::Timestamp::now();
        let modified = ts
            .checked_add(jiff::SignedDuration::from_secs(offset))
            .unwrap_or(ts);
        return modified.to_string();
    }
    jiff::Timestamp::now().to_string()
}

/// Retrieves available free bytes on the volume containing `path`.
pub fn get_disk_free_bytes(path: &Path) -> Result<u64, std::io::Error> {
    #[cfg(feature = "fault-injection")]
    if let Some(bytes) = crate::fault::get_simulated_disk_free() {
        return Ok(bytes);
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut target_dir = path.to_path_buf();
        if target_dir.is_file() || (!target_dir.exists() && target_dir.extension().is_some()) {
            if let Some(parent) = target_dir.parent() {
                target_dir = parent.to_path_buf();
            }
        }
        if !target_dir.exists() {
            let _ = std::fs::create_dir_all(&target_dir);
        }

        let mut wide: Vec<u16> = target_dir.as_os_str().encode_wide().collect();
        wide.push(0);

        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        extern "system" {
            fn GetDiskFreeSpaceExW(
                lpDirectoryName: *const u16,
                lpFreeBytesAvailableToCaller: *mut u64,
                lpTotalNumberOfBytes: *mut u64,
                lpTotalNumberOfFreeBytes: *mut u64,
            ) -> i32;
        }

        let success = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            )
        };

        if success != 0 {
            Ok(free_bytes_available)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    #[cfg(not(windows))]
    {
        // 50 GB default for non-windows platforms in tests
        Ok(50 * 1024 * 1024 * 1024)
    }
}

/// Validates that the current system clock is not drifted backwards beyond 60 seconds
/// relative to the latest audit log watermark (`SELECT MAX(at) FROM audit_log`).
///
/// Satisfies AC-7: Backward clock changes beyond 60 seconds prevent financial entries
/// unless explicitly acknowledged by an operator.
pub fn check_clock_watermark(conn: &Connection, current_utc: &str) -> Result<(), DbError> {
    let curr_ts: jiff::Timestamp = current_utc.parse().map_err(|e| {
        DbError::Config(format!("Invalid current_utc timestamp {current_utc}: {e}"))
    })?;

    let guard_row: Option<(String, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT high_water_at, last_ack_by, last_ack_at FROM clock_guard WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .unwrap_or(None);

    let max_at: Option<String> = conn
        .query_row("SELECT MAX(at) FROM audit_log", [], |row| row.get(0))
        .optional()
        .unwrap_or(None)
        .flatten();

    let watermark_opt = match (&guard_row, &max_at) {
        (Some((hw, _, _)), Some(audit_max)) => {
            if hw >= audit_max {
                Some(hw.clone())
            } else {
                Some(audit_max.clone())
            }
        }
        (Some((hw, _, _)), None) => Some(hw.clone()),
        (None, Some(audit_max)) => Some(audit_max.clone()),
        (None, None) => None,
    };

    let Some(watermark) = watermark_opt else {
        return Ok(());
    };

    let water_ts: jiff::Timestamp = watermark
        .parse()
        .map_err(|e| DbError::Config(format!("Invalid watermark timestamp {watermark}: {e}")))?;

    let diff_seconds = curr_ts.as_second() - water_ts.as_second();
    // 60-second backward drift tolerance
    if diff_seconds < -60 {
        if let Some((_, Some(actor), Some(ack_at))) = &guard_row {
            if !actor.trim().is_empty() {
                if let Ok(ack_ts) = ack_at.parse::<jiff::Timestamp>() {
                    if ack_ts.as_second() >= water_ts.as_second()
                        || curr_ts.as_second() >= ack_ts.as_second() - 60
                    {
                        return Ok(());
                    }
                }
            }
        }

        return Err(DbError::ClockDriftBackwards(format!(
            "System clock is behind high water mark by {} seconds (current: {}, watermark: {}). Please synchronize computer clock before performing write operations.",
            -diff_seconds, current_utc, watermark
        )));
    }

    if diff_seconds > 0 {
        let _ = conn.execute(
            "INSERT INTO clock_guard (id, high_water_at) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET high_water_at = ?1 WHERE high_water_at < ?1",
            rusqlite::params![current_utc],
        );
    }

    Ok(())
}

/// Helper that checks whether the current clock is within acceptable watermark tolerance.
pub fn is_clock_synchronized(conn: &Connection, current_utc: &str) -> bool {
    check_clock_watermark(conn, current_utc).is_ok()
}

pub fn current_business_date() -> Result<String, DbError> {
    let zdt = jiff::Timestamp::now()
        .in_tz("Asia/Dhaka")
        .map_err(|e| DbError::Config(e.to_string()))?;
    Ok(zdt.date().to_string())
}

pub fn run_ping_transaction(message: Option<String>) -> Result<(String, String, String), DbError> {
    let mut conn = open_database()?;
    let db_path = resolve_active_db_path()?;

    let (status, pinged_at) = run_in_transaction(&mut conn, |tx| {
        tx.execute(
            "INSERT INTO _scaffold_health (id, status, pinged_at)
             VALUES (1, 'active', datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                 status = 'active',
                 pinged_at = datetime('now')",
            [],
        )?;

        let mut stmt = tx.prepare("SELECT status, pinged_at FROM _scaffold_health WHERE id = 1")?;
        let result: (String, String) = stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(result)
    })?;

    let echo_msg = message.unwrap_or_else(|| "ping".to_string());
    Ok((
        status,
        pinged_at,
        format!("echo: {} [db: {}]", echo_msg, db_path.display()),
    ))
}
