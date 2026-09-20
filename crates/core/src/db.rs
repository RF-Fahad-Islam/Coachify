use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

#[cfg(debug_assertions)]
pub const DEV_KEY: &str = "coachify_dev_debug_key_local_only";

#[cfg(not(debug_assertions))]
pub const DEV_KEY: &str = "";

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
    let conn = Connection::open(&db_path)?;

    // Set performance and integrity PRAGMAs
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

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

pub fn run_ping_transaction(message: Option<String>) -> Result<(String, String, String), DbError> {
    let conn = open_database()?;
    let db_path = resolve_dev_db_path()?;

    conn.execute(
        "INSERT INTO _scaffold_health (id, status, pinged_at)
         VALUES (1, 'active', datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
             status = 'active',
             pinged_at = datetime('now')",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT status, pinged_at FROM _scaffold_health WHERE id = 1")?;
    let (status, pinged_at): (String, String) =
        stmt.query_row([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let echo_msg = message.unwrap_or_else(|| "ping".to_string());
    Ok((
        status,
        pinged_at,
        format!("echo: {} [db: {}]", echo_msg, db_path.display()),
    ))
}
