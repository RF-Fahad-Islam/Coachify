use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

pub fn resolve_dev_db_path() -> Result<PathBuf, DbError> {
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

pub fn open_database() -> Result<Connection, DbError> {
    #[cfg(not(debug_assertions))]
    {
        if DEV_KEY.is_empty() {
            return Err(DbError::Config(
                "Release build refuses to start until feature 8 provides real key management"
                    .to_string(),
            ));
        }
    }

    let db_path = resolve_dev_db_path()?;
    let conn = open_connection_at(&db_path)?;
    verify_integrity(&conn)?;

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
    tx: &rusqlite::Transaction,
    name: &str,
    prefix: &str,
) -> Result<String, DbError> {
    tx.execute(
        "INSERT INTO sequences (name, prefix, next_value)
         VALUES (?1, ?2, 1)
         ON CONFLICT(name) DO NOTHING",
        rusqlite::params![name, prefix],
    )?;

    let (db_prefix, val): (String, i64) = tx.query_row(
        "SELECT prefix, next_value FROM sequences WHERE name = ?1",
        rusqlite::params![name],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    tx.execute(
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

pub fn verify_clock(
    tx: &rusqlite::Transaction,
    current_utc: &str,
    max_forward_hours: Option<u32>,
) -> Result<(), DbError> {
    let mut stmt = tx.prepare("SELECT high_water_at, last_ack_at FROM clock_guard WHERE id = 1")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let high_water_at: String = row.get(0)?;

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
            return Err(DbError::ClockJumpForward((diff_seconds / 3600) as u32));
        }

        tx.execute(
            "UPDATE clock_guard SET high_water_at = ?1 WHERE id = 1",
            rusqlite::params![current_utc],
        )?;
    } else {
        tx.execute(
            "INSERT INTO clock_guard (id, high_water_at) VALUES (1, ?1)",
            rusqlite::params![current_utc],
        )?;
    }

    Ok(())
}

pub fn now_utc_iso() -> String {
    jiff::Timestamp::now().to_string()
}

pub fn current_business_date() -> Result<String, DbError> {
    let zdt = jiff::Timestamp::now()
        .in_tz("Asia/Dhaka")
        .map_err(|e| DbError::Config(e.to_string()))?;
    Ok(zdt.date().to_string())
}

pub fn run_ping_transaction(message: Option<String>) -> Result<(String, String, String), DbError> {
    let mut conn = open_database()?;
    let db_path = resolve_dev_db_path()?;

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
