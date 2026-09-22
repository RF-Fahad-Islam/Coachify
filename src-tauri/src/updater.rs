//! Updater and pre-update safety backup coordinator for Coachify Desktop.
//!
//! Enforces that a verified snapshot is captured before any update or
//! schema advancement is staged.

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Pre-update safety backup failed: {0}")]
    BackupFailed(String),
    #[error("Downloaded installer payload is invalid or corrupted: {0}")]
    InvalidPayload(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Prepares for application update by capturing a mandatory pre-update
/// safety backup into the backup directory.
pub fn prepare_for_update(db_path: &Path, backup_dir: &Path) -> Result<PathBuf, UpdateError> {
    if !db_path.exists() {
        return Ok(backup_dir.to_path_buf());
    }

    std::fs::create_dir_all(backup_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let target_file = backup_dir.join(format!("pre_update_{timestamp}.db"));

    // Hot copy using SQLite backup API
    let source_conn = rusqlite::Connection::open(db_path)
        .map_err(|e| UpdateError::BackupFailed(e.to_string()))?;
    let mut dest_conn = rusqlite::Connection::open(&target_file)
        .map_err(|e| UpdateError::BackupFailed(e.to_string()))?;

    {
        let backup = rusqlite::backup::Backup::new(&source_conn, &mut dest_conn)
            .map_err(|e| UpdateError::BackupFailed(e.to_string()))?;

        backup
            .run_to_completion(50, std::time::Duration::from_millis(50), None)
            .map_err(|e| UpdateError::BackupFailed(e.to_string()))?;
    }

    // Verify the pre-update backup passed PRAGMA integrity_check
    let integrity: String = dest_conn
        .query_row("PRAGMA integrity_check;", [], |row| row.get(0))
        .map_err(|e| UpdateError::BackupFailed(format!("Integrity check failed: {e}")))?;

    if integrity != "ok" {
        let _ = std::fs::remove_file(&target_file);
        return Err(UpdateError::BackupFailed(format!(
            "Pre-update backup integrity check failed: {integrity}"
        )));
    }

    Ok(target_file)
}

/// Verifies that an installer executable file exists, is non-empty, and has expected minimum size.
pub fn verify_installer_payload(
    installer_path: &Path,
    min_size_bytes: u64,
) -> Result<(), UpdateError> {
    if !installer_path.exists() {
        return Err(UpdateError::InvalidPayload(
            "Installer file does not exist".into(),
        ));
    }

    let metadata = std::fs::metadata(installer_path)?;
    if metadata.len() < min_size_bytes {
        return Err(UpdateError::InvalidPayload(format!(
            "Installer file is truncated ({} bytes, expected at least {} bytes)",
            metadata.len(),
            min_size_bytes
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_for_update_creates_valid_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("dev.db");
        let backup_dir = temp_dir.path().join("backups");

        // Create sample db
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT);", [])
            .unwrap();
        conn.execute("INSERT INTO test (name) VALUES ('Coachify');", [])
            .unwrap();
        drop(conn);

        let backup_path = prepare_for_update(&db_path, &backup_dir).unwrap();
        assert!(backup_path.exists());

        // Verify contents
        let b_conn = rusqlite::Connection::open(&backup_path).unwrap();
        let count: i64 = b_conn
            .query_row("SELECT COUNT(*) FROM test;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_verify_installer_payload() {
        let temp_dir = tempfile::tempdir().unwrap();
        let empty_file = temp_dir.path().join("setup.exe");
        std::fs::write(&empty_file, b"").unwrap();

        let err = verify_installer_payload(&empty_file, 100).unwrap_err();
        assert!(err.to_string().contains("truncated"));

        std::fs::write(&empty_file, vec![0u8; 150]).unwrap();
        assert!(verify_installer_payload(&empty_file, 100).is_ok());
    }
}
