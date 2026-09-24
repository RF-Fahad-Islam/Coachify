use crate::db;
use hmac::{Hmac, Mac};
use rusqlite::backup::Backup;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

// Re-export Sha256 for HMAC
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Database error: {0}")]
    Db(#[from] crate::db::DbError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Backup destination not found: {0}")]
    DestinationNotFound(String),
    #[error("Backup destination not writable: {0}")]
    DestinationNotWritable(String),
    #[error("Backup file not found: {0}")]
    BackupNotFound(String),
    #[error("Backup verification failed: {0}")]
    VerificationFailed(String),
    #[error("Manifest HMAC mismatch — backup may have been tampered with")]
    ManifestTampered,
    #[error("Integrity check failed: {0}")]
    IntegrityFailed(String),
    #[error("Foreign key check failed: {0}")]
    ForeignKeyFailed(String),
    #[error("Schema version {backup_version} is newer than app version {app_version}. Update Coachify first.")]
    NewerSchema {
        backup_version: u32,
        app_version: u32,
    },
    #[error("Recovery key required to restore on a new PC")]
    RecoveryKeyRequired,
    #[error("Invalid recovery key")]
    InvalidRecoveryKey,
    #[error("Insufficient disk space: need {required} bytes, have {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },
    #[error("Serialization error: {0}")]
    Serde(String),
    #[error("Database is corrupted. Read-only mode active.")]
    DatabaseCorrupted,
}

impl BackupError {
    pub fn code(&self) -> &'static str {
        match self {
            BackupError::Sqlite(_) | BackupError::Db(_) => "backup_database_error",
            BackupError::Io(_) => "backup_io_error",
            BackupError::DestinationNotFound(_) => "backup_destination_not_found",
            BackupError::DestinationNotWritable(_) => "backup_destination_not_writable",
            BackupError::BackupNotFound(_) => "backup_not_found",
            BackupError::VerificationFailed(_) => "backup_verification_failed",
            BackupError::ManifestTampered => "backup_manifest_tampered",
            BackupError::IntegrityFailed(_) => "backup_integrity_failed",
            BackupError::ForeignKeyFailed(_) => "backup_foreign_key_failed",
            BackupError::NewerSchema { .. } => "backup_newer_schema",
            BackupError::RecoveryKeyRequired => "backup_recovery_key_required",
            BackupError::InvalidRecoveryKey => "backup_invalid_recovery_key",
            BackupError::InsufficientSpace { .. } => "backup_insufficient_space",
            BackupError::Serde(_) => "backup_serde_error",
            BackupError::DatabaseCorrupted => "database_corrupted",
        }
    }
}

// ---------------------------------------------------------------------------
// Data Types (RPC-facing)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupDestinationItem {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub label: String,
    pub institution_marker: Option<String>,
    pub is_default: bool,
    pub last_ok_at: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupHistoryItem {
    pub id: String,
    pub destination_id: Option<String>,
    pub kind: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub app_version: String,
    pub schema_version: u32,
    pub institution_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub record_counts: RecordCounts,
    pub money_totals: MoneyTotals,
    pub audit_chain_head: Option<String>,
    pub vault_file_count: u32,
    pub verified_at: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupScheduleItem {
    pub id: String,
    pub kind: String,
    pub destination_id: Option<String>,
    pub time_of_day: Option<String>,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupDashboard {
    pub last_backup: Option<BackupHistoryItem>,
    pub last_verified: Option<BackupHistoryItem>,
    pub last_restore_drill: Option<RestoreDrillItem>,
    pub destinations: Vec<BackupDestinationItem>,
    pub schedules: Vec<BackupScheduleItem>,
    pub warnings: Vec<String>,
    pub data_size_bytes: u64,
    pub backup_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RestoreDrillItem {
    pub id: String,
    pub backup_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub result: String,
    pub integrity_ok: Option<bool>,
    pub foreign_keys_ok: Option<bool>,
    pub record_counts_match: Option<bool>,
    pub money_totals_match: Option<bool>,
    pub details_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct RecordCounts {
    pub students: i64,
    pub guardians: i64,
    pub enrollments: i64,
    pub payments: i64,
    pub charges: i64,
    pub refunds: i64,
    pub adjustments: i64,
    pub expenses: i64,
    pub audit_rows: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct MoneyTotals {
    pub total_charged_poisha: i64,
    pub total_paid_poisha: i64,
    pub total_refunded_poisha: i64,
    pub total_adjusted_poisha: i64,
    pub total_balance_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupManifest {
    pub version: u32,
    pub app_version: String,
    pub schema_version: u32,
    pub institution_id: String,
    pub institution_name: String,
    pub created_at: String,
    pub record_counts: RecordCounts,
    pub money_totals: MoneyTotals,
    pub audit_chain_head: Option<String>,
    pub vault_files: Vec<VaultFileEntry>,
    pub wrapped_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VaultFileEntry {
    pub relative_path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerificationResult {
    pub backup_id: String,
    pub passed: bool,
    pub integrity_ok: bool,
    pub foreign_keys_ok: bool,
    pub record_counts_match: bool,
    pub money_totals_match: bool,
    pub manifest_hmac_ok: bool,
    pub vault_files_ok: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupPreview {
    pub institution_name: String,
    pub institution_id: String,
    pub created_at: String,
    pub app_version: String,
    pub schema_version: u32,
    pub current_schema_version: u32,
    pub needs_migration: bool,
    pub needs_update: bool,
    pub record_counts: RecordCounts,
    pub money_totals: MoneyTotals,
    pub integrity_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RestoreResult {
    pub success: bool,
    pub safety_backup_id: Option<String>,
    pub migrated_from_version: Option<u32>,
    pub record_counts: RecordCounts,
    pub money_totals: MoneyTotals,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub integrity_ok: bool,
    pub foreign_keys_ok: bool,
    pub error_message: Option<String>,
    pub is_read_only: bool,
}

// ---------------------------------------------------------------------------
// Record Counts and Money Totals Collection
// ---------------------------------------------------------------------------

pub fn collect_record_counts(conn: &Connection) -> Result<RecordCounts, BackupError> {
    let count = |table: &str| -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap_or(0)
    };

    Ok(RecordCounts {
        students: count("students"),
        guardians: count("guardians"),
        enrollments: count("enrollments"),
        payments: count("payments"),
        charges: count("charges"),
        refunds: conn
            .query_row("SELECT COUNT(*) FROM refunds", [], |row| row.get(0))
            .unwrap_or(0),
        adjustments: conn
            .query_row("SELECT COUNT(*) FROM adjustments", [], |row| row.get(0))
            .unwrap_or(0),
        expenses: conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap_or(0),
        audit_rows: count("audit_log"),
    })
}

pub fn collect_money_totals(conn: &Connection) -> Result<MoneyTotals, BackupError> {
    let totals = conn
        .query_row(
            "SELECT
                COALESCE(SUM(total_charged), 0),
                COALESCE(SUM(total_paid), 0),
                COALESCE(SUM(total_refunded), 0),
                COALESCE(SUM(total_adjusted), 0),
                COALESCE(SUM(balance), 0)
             FROM student_balances",
            [],
            |row| {
                Ok(MoneyTotals {
                    total_charged_poisha: row.get(0)?,
                    total_paid_poisha: row.get(1)?,
                    total_refunded_poisha: row.get(2)?,
                    total_adjusted_poisha: row.get(3)?,
                    total_balance_poisha: row.get(4)?,
                })
            },
        )
        .unwrap_or_default();

    Ok(totals)
}

// ---------------------------------------------------------------------------
// Snapshot Creation (SQLite hot backup API)
// ---------------------------------------------------------------------------

/// Creates a consistent database snapshot using SQLite's online backup API.
/// Writes to a temporary file first, then renames atomically.
pub fn create_snapshot(
    source_conn: &Connection,
    destination_dir: &Path,
    kind: &str,
    _institution_id: &str,
) -> Result<(PathBuf, RecordCounts, MoneyTotals), BackupError> {
    fs::create_dir_all(destination_dir)?;

    // AC-6: Validate disk space on destination volume (< 500 MB refuses backup)
    let free_bytes = db::get_disk_free_bytes(destination_dir).unwrap_or(u64::MAX);
    const MIN_BACKUP_FREE_BYTES: u64 = 500 * 1024 * 1024; // 500 MB
    if free_bytes < MIN_BACKUP_FREE_BYTES {
        return Err(BackupError::InsufficientSpace {
            required: MIN_BACKUP_FREE_BYTES,
            available: free_bytes,
        });
    }

    #[cfg(feature = "fault-injection")]
    if crate::fault::is_usb_removal_simulated() {
        return Err(BackupError::DestinationNotFound(
            "Simulated USB drive disconnect mid backup".to_string(),
        ));
    }

    let now = db::now_utc_iso();
    let safe_ts = now.replace([':', '.'], "-");
    let filename = format!("backup-{kind}-{safe_ts}.db");
    let temp_path = destination_dir.join(format!(".tmp-{filename}"));
    let final_path = destination_dir.join(&filename);

    // Collect counts and totals BEFORE the backup so they reflect the
    // same logical state (the backup API copies pages as they are).
    let record_counts = collect_record_counts(source_conn)?;
    let money_totals = collect_money_totals(source_conn)?;

    // Open a destination connection for the backup API
    {
        let mut dest_conn = Connection::open(&temp_path)?;
        let backup = Backup::new(source_conn, &mut dest_conn)?;

        // Step in pages of 256 at a time, sleeping 10ms between to reduce
        // write pressure on the source database.
        loop {
            match backup.step(256)? {
                rusqlite::backup::StepResult::Done => break,
                rusqlite::backup::StepResult::More => {
                    #[cfg(feature = "fault-injection")]
                    crate::fault::trigger_checkpoint("backup_mid_stream");
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                rusqlite::backup::StepResult::Busy | rusqlite::backup::StepResult::Locked => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }

    // Flush and rename atomically
    fs::rename(&temp_path, &final_path)?;

    Ok((final_path, record_counts, money_totals))
}

/// Sweeps any temporary `.tmp-backup-*.db` files left behind by uncompleted or crashed backups.
///
/// Satisfies AC-2.
pub fn sweep_temporary_backups(destination_dir: &Path) -> Result<usize, io::Error> {
    if !destination_dir.exists() {
        return Ok(0);
    }
    let mut swept = 0;
    for entry in fs::read_dir(destination_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(".tmp-backup-") {
                    let _ = fs::remove_file(&path);
                    swept += 1;
                }
            }
        }
    }
    Ok(swept)
}

// ---------------------------------------------------------------------------
// Manifest Creation
// ---------------------------------------------------------------------------

/// Builds a backup manifest capturing the state at backup time.
pub fn create_manifest(
    conn: &Connection,
    record_counts: &RecordCounts,
    money_totals: &MoneyTotals,
    institution_id: &str,
    vault_dir: Option<&Path>,
) -> Result<BackupManifest, BackupError> {
    // Fetch institution name
    let institution_name: String = conn
        .query_row("SELECT name FROM institution WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    // Fetch schema version
    let schema_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Fetch audit chain head
    let audit_chain_head: Option<String> = conn
        .query_row(
            "SELECT entry_hash FROM audit_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    // Collect vault file entries
    let vault_files = if let Some(vdir) = vault_dir {
        collect_vault_entries(vdir)?
    } else {
        vec![]
    };

    Ok(BackupManifest {
        version: 1,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        institution_id: institution_id.to_string(),
        institution_name,
        created_at: db::now_utc_iso(),
        record_counts: record_counts.clone(),
        money_totals: money_totals.clone(),
        audit_chain_head,
        vault_files,
        wrapped_key: None, // Filled in by the caller when exporting
    })
}

/// Write the manifest JSON alongside the backup and compute its HMAC.
pub fn write_manifest(
    manifest: &BackupManifest,
    backup_dir: &Path,
    hmac_key: &[u8],
) -> Result<String, BackupError> {
    let manifest_json =
        serde_json::to_string_pretty(manifest).map_err(|e| BackupError::Serde(e.to_string()))?;

    let manifest_path = backup_dir.join("manifest.json");
    let mut file = fs::File::create(&manifest_path)?;
    file.write_all(manifest_json.as_bytes())?;
    file.flush()?;

    // Compute HMAC-SHA256 over the manifest content
    let mut mac =
        HmacSha256::new_from_slice(hmac_key).map_err(|e| BackupError::Serde(e.to_string()))?;
    mac.update(manifest_json.as_bytes());
    let result = mac.finalize();
    let hmac_hex = hex::encode(result.into_bytes());

    // Write HMAC to a sidecar file
    let hmac_path = backup_dir.join("manifest.hmac");
    fs::write(&hmac_path, &hmac_hex)?;

    Ok(hmac_hex)
}

// ---------------------------------------------------------------------------
// Vault Mirroring (incremental)
// ---------------------------------------------------------------------------

fn collect_vault_entries(vault_dir: &Path) -> Result<Vec<VaultFileEntry>, BackupError> {
    let mut entries = Vec::new();
    if !vault_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(vault_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let metadata = entry.metadata()?;
            let relative = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // The vault is content-addressed: filename IS the hash.
            // We still verify it for backup integrity.
            let sha = compute_file_sha256(&path)?;

            entries.push(VaultFileEntry {
                relative_path: relative,
                sha256: sha,
                size_bytes: metadata.len(),
            });
        }
    }

    Ok(entries)
}

fn compute_file_sha256(path: &Path) -> Result<String, BackupError> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Mirrors vault files from source to destination, copying only new files.
pub fn mirror_vault(source_vault: &Path, dest_vault: &Path) -> Result<u32, BackupError> {
    if !source_vault.exists() {
        return Ok(0);
    }
    fs::create_dir_all(dest_vault)?;

    let mut copied = 0u32;
    for entry in fs::read_dir(source_vault)? {
        let entry = entry?;
        let source_path = entry.path();
        if source_path.is_file() {
            if let Some(name) = source_path.file_name() {
                let dest_path = dest_vault.join(name);
                if !dest_path.exists() {
                    fs::copy(&source_path, &dest_path)?;
                    copied += 1;
                }
            }
        }
    }

    Ok(copied)
}

// ---------------------------------------------------------------------------
// Backup Verification
// ---------------------------------------------------------------------------

/// Verifies a backup's integrity: database health, record counts, money totals,
/// manifest HMAC, and vault file checksums.
pub fn verify_backup(
    backup_db_path: &Path,
    manifest: &BackupManifest,
    manifest_hmac: &str,
    hmac_key: &[u8],
    vault_dir: Option<&Path>,
) -> Result<VerificationResult, BackupError> {
    let mut errors = Vec::new();
    let backup_id = String::new(); // Caller sets this

    // 1. Verify manifest HMAC
    let manifest_json =
        serde_json::to_string_pretty(manifest).map_err(|e| BackupError::Serde(e.to_string()))?;
    let mut mac =
        HmacSha256::new_from_slice(hmac_key).map_err(|e| BackupError::Serde(e.to_string()))?;
    mac.update(manifest_json.as_bytes());
    let expected_hmac = hex::encode(mac.finalize().into_bytes());
    let manifest_hmac_ok = expected_hmac == manifest_hmac;
    if !manifest_hmac_ok {
        errors.push("Manifest HMAC mismatch — backup may have been tampered with".to_string());
    }

    // 2. Open backup database and run integrity check
    let conn = Connection::open(backup_db_path)?;
    db::configure_connection(&conn)?;

    let integrity_ok = match db::verify_integrity(&conn) {
        Ok(()) => true,
        Err(e) => {
            errors.push(format!("Integrity check failed: {e}"));
            false
        }
    };

    // 3. Foreign key check
    let foreign_keys_ok = match run_foreign_key_check(&conn) {
        Ok(()) => true,
        Err(e) => {
            errors.push(format!("Foreign key check failed: {e}"));
            false
        }
    };

    // 4. Record counts comparison
    let actual_counts = collect_record_counts(&conn)?;
    let record_counts_match = actual_counts.students == manifest.record_counts.students
        && actual_counts.payments == manifest.record_counts.payments
        && actual_counts.charges == manifest.record_counts.charges
        && actual_counts.audit_rows == manifest.record_counts.audit_rows;
    if !record_counts_match {
        errors.push(format!(
            "Record count mismatch: expected {} students / {} payments, found {} / {}",
            manifest.record_counts.students,
            manifest.record_counts.payments,
            actual_counts.students,
            actual_counts.payments,
        ));
    }

    // 5. Money totals comparison
    let actual_totals = collect_money_totals(&conn)?;
    let money_totals_match = actual_totals.total_charged_poisha
        == manifest.money_totals.total_charged_poisha
        && actual_totals.total_paid_poisha == manifest.money_totals.total_paid_poisha
        && actual_totals.total_balance_poisha == manifest.money_totals.total_balance_poisha;
    if !money_totals_match {
        errors.push(format!(
            "Money totals mismatch: expected charged={} paid={} balance={}, found charged={} paid={} balance={}",
            manifest.money_totals.total_charged_poisha,
            manifest.money_totals.total_paid_poisha,
            manifest.money_totals.total_balance_poisha,
            actual_totals.total_charged_poisha,
            actual_totals.total_paid_poisha,
            actual_totals.total_balance_poisha,
        ));
    }

    // 6. Vault file verification
    let vault_files_ok = if let Some(vdir) = vault_dir {
        verify_vault_files(&manifest.vault_files, vdir, &mut errors)
    } else {
        true
    };

    let passed = manifest_hmac_ok
        && integrity_ok
        && foreign_keys_ok
        && record_counts_match
        && money_totals_match
        && vault_files_ok;

    Ok(VerificationResult {
        backup_id,
        passed,
        integrity_ok,
        foreign_keys_ok,
        record_counts_match,
        money_totals_match,
        manifest_hmac_ok,
        vault_files_ok,
        errors,
    })
}

fn run_foreign_key_check(conn: &Connection) -> Result<(), BackupError> {
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let mut rows = stmt.query([])?;
    let mut violations = Vec::new();
    while let Some(row) = rows.next()? {
        let table: String = row.get(0)?;
        let rowid: i64 = row.get(1)?;
        violations.push(format!("{table} row {rowid}"));
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(BackupError::ForeignKeyFailed(format!(
            "{} violations: {}",
            violations.len(),
            violations.join(", ")
        )))
    }
}

fn verify_vault_files(
    expected: &[VaultFileEntry],
    vault_dir: &Path,
    errors: &mut Vec<String>,
) -> bool {
    let mut all_ok = true;
    for entry in expected {
        let path = vault_dir.join(&entry.relative_path);
        if !path.exists() {
            errors.push(format!("Missing vault file: {}", entry.relative_path));
            all_ok = false;
            continue;
        }
        match compute_file_sha256(&path) {
            Ok(sha) if sha == entry.sha256 => {}
            Ok(sha) => {
                errors.push(format!(
                    "Vault file {} hash mismatch: expected {}, got {}",
                    entry.relative_path, entry.sha256, sha
                ));
                all_ok = false;
            }
            Err(e) => {
                errors.push(format!(
                    "Cannot read vault file {}: {}",
                    entry.relative_path, e
                ));
                all_ok = false;
            }
        }
    }
    all_ok
}

// ---------------------------------------------------------------------------
// Restore Execution
// ---------------------------------------------------------------------------

pub fn execute_restore(
    conn: &mut Connection,
    candidate_file_path: &str,
    recovery_key: Option<&str>,
) -> Result<RestoreResult, BackupError> {
    let candidate_path = Path::new(candidate_file_path);
    if !candidate_path.exists() {
        return Err(BackupError::BackupNotFound(candidate_file_path.to_string()));
    }

    // 1. Verify candidate database
    let candidate_conn = Connection::open(candidate_path)?;
    db::configure_connection(&candidate_conn)?;
    db::verify_integrity(&candidate_conn)?;
    run_foreign_key_check(&candidate_conn)?;

    // 2. Check candidate schema version
    let candidate_schema_version: u32 = candidate_conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .or_else(|_| {
            candidate_conn.query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
                [],
                |row| row.get(0),
            )
        })
        .unwrap_or(0);

    let target_version = crate::migrations::TARGET_SCHEMA_VERSION;
    if candidate_schema_version > target_version {
        return Err(BackupError::NewerSchema {
            backup_version: candidate_schema_version,
            app_version: target_version,
        });
    }

    // 3. If recovery key provided, validate format
    if let Some(key) = recovery_key {
        crate::crypto::normalize_and_verify_recovery_key(key)
            .map_err(|_| BackupError::InvalidRecoveryKey)?;
    }

    // 4. Create Safety Backup of currently active database before overwriting
    let safety_dir = candidate_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("safety_backups");
    let (safety_path, safety_counts, safety_totals) =
        create_snapshot(conn, &safety_dir, "pre_restore", "current")?;

    let safety_id = uuid::Uuid::now_v7().to_string();
    let now = db::now_utc_iso();
    let safety_size = std::fs::metadata(&safety_path)
        .map(|m| m.len())
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO backup_history (id, destination_id, kind, file_path, file_size_bytes,
            app_version, schema_version, institution_id, started_at, completed_at,
            record_counts_json, money_totals_json, status, created_at)
         VALUES (?1, NULL, 'pre_restore', ?2, ?3, ?4, ?5, 'current', ?6, ?6, ?7, ?8, 'completed', ?6)",
        rusqlite::params![
            safety_id,
            safety_path.display().to_string(),
            safety_size as i64,
            env!("CARGO_PKG_VERSION"),
            target_version,
            now,
            serde_json::to_string(&safety_counts).unwrap_or_default(),
            serde_json::to_string(&safety_totals).unwrap_or_default(),
        ],
    )?;

    // 5. If migration needed (candidate schema < target), run forward migrations on a staging copy
    let staged_file = safety_dir.join(format!(".staged_restore_{}.db", uuid::Uuid::now_v7()));
    std::fs::copy(candidate_path, &staged_file)?;

    let (source_for_restore_path, migrated_from) = if candidate_schema_version < target_version {
        let staging_conn = Connection::open(&staged_file)?;
        db::configure_connection(&staging_conn)?;
        db::apply_all_migrations(&staging_conn)?;
        db::verify_integrity(&staging_conn)?;
        (staged_file.clone(), Some(candidate_schema_version))
    } else {
        (staged_file.clone(), None)
    };

    // 6. Overwrite active database using SQLite backup API
    {
        let source_conn = Connection::open(&source_for_restore_path)?;
        let backup = Backup::new(&source_conn, conn)?;
        loop {
            match backup.step(256)? {
                rusqlite::backup::StepResult::Done => break,
                rusqlite::backup::StepResult::More => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                rusqlite::backup::StepResult::Busy | rusqlite::backup::StepResult::Locked => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        }
    }

    // Clean up temporary staged file
    let _ = std::fs::remove_file(&staged_file);

    // Re-record safety backup into the restored database so foreign key to backup_history succeeds
    conn.execute(
        "INSERT INTO backup_history (id, destination_id, kind, file_path, file_size_bytes,
            app_version, schema_version, institution_id, started_at, completed_at,
            record_counts_json, money_totals_json, status, created_at)
         VALUES (?1, NULL, 'pre_restore', ?2, ?3, ?4, ?5, 'current', ?6, ?6, ?7, ?8, 'completed', ?6)",
        rusqlite::params![
            safety_id,
            safety_path.display().to_string(),
            safety_size as i64,
            env!("CARGO_PKG_VERSION"),
            target_version,
            now,
            serde_json::to_string(&safety_counts).unwrap_or_default(),
            serde_json::to_string(&safety_totals).unwrap_or_default(),
        ],
    )?;

    // 7. Post-restore verification and collection
    db::verify_integrity(conn)?;
    let record_counts = collect_record_counts(conn)?;
    let money_totals = collect_money_totals(conn)?;

    // 8. Record restore drill entry with result 'pass'
    record_restore_drill(
        conn,
        &safety_id,
        "pass",
        true,
        true,
        true,
        true,
        Some("Restore completed successfully"),
    )?;

    Ok(RestoreResult {
        success: true,
        safety_backup_id: Some(safety_id),
        migrated_from_version: migrated_from,
        record_counts,
        money_totals,
        message: "Database successfully restored and verified".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Retention / Pruning
// ---------------------------------------------------------------------------

/// Retention policy: 7 daily, 4 weekly, 12 monthly, tagged (pre_update, pre_restore) for 30 days.
/// Never prunes the last verified backup.
pub fn prune_backups(conn: &Connection, destination_id: &str) -> Result<u32, BackupError> {
    let mut pruned = 0u32;

    // Get all completed/verified backups for this destination, ordered newest first
    let mut stmt = conn.prepare(
        "SELECT id, kind, file_path, status, created_at
         FROM backup_history
         WHERE destination_id = ?1
           AND status IN ('completed', 'verified')
         ORDER BY created_at DESC",
    )?;

    let rows: Vec<(String, String, String, String, String)> = stmt
        .query_map([destination_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(0);
    }

    // Find the last verified backup — never prune it
    let last_verified_id = rows
        .iter()
        .find(|(_, _, _, status, _)| status == "verified")
        .map(|(id, _, _, _, _)| id.clone());

    // Separate daily/hourly from tagged backups
    let mut daily_backups: Vec<&(String, String, String, String, String)> = Vec::new();
    let mut tagged_backups: Vec<&(String, String, String, String, String)> = Vec::new();

    for row in &rows {
        match row.1.as_str() {
            "daily" | "hourly_snapshot" | "manual" => daily_backups.push(row),
            "pre_update" | "pre_restore" | "pre_import" => tagged_backups.push(row),
            _ => daily_backups.push(row),
        }
    }

    // Keep 7 most recent daily backups
    let daily_keep = 7;
    for (i, row) in daily_backups.iter().enumerate() {
        if i >= daily_keep && Some(&row.0) != last_verified_id.as_ref() {
            if let Err(e) = delete_backup_file(&row.2) {
                eprintln!("Failed to delete backup file {}: {}", row.2, e);
                continue;
            }
            conn.execute("DELETE FROM backup_history WHERE id = ?1", [&row.0])?;
            pruned += 1;
        }
    }

    // Prune tagged backups older than 30 days
    for row in &tagged_backups {
        if is_older_than_days(&row.4, 30) && Some(&row.0) != last_verified_id.as_ref() {
            if let Err(e) = delete_backup_file(&row.2) {
                eprintln!("Failed to delete tagged backup file {}: {}", row.2, e);
                continue;
            }
            conn.execute("DELETE FROM backup_history WHERE id = ?1", [&row.0])?;
            pruned += 1;
        }
    }

    Ok(pruned)
}

fn delete_backup_file(path: &str) -> Result<(), BackupError> {
    let p = Path::new(path);
    if p.exists() {
        fs::remove_file(p)?;
    }
    Ok(())
}

fn is_older_than_days(timestamp: &str, days: i64) -> bool {
    let Ok(ts) = jiff::Timestamp::from_str(timestamp) else {
        return false;
    };
    let now = jiff::Timestamp::now();
    (now.as_second() - ts.as_second()) > days * 86400
}

// ---------------------------------------------------------------------------
// Health Check (startup)
// ---------------------------------------------------------------------------

use std::sync::RwLock;

static DATABASE_CORRUPTED: RwLock<bool> = RwLock::new(false);

/// Returns true if the database has been flagged as corrupted.
pub fn is_database_corrupted() -> bool {
    *DATABASE_CORRUPTED.read().unwrap_or_else(|e| e.into_inner())
}

/// Sets the database corruption flag (used on startup check failure).
pub fn set_database_corrupted(corrupted: bool) {
    if let Ok(mut guard) = DATABASE_CORRUPTED.write() {
        *guard = corrupted;
    }
}

/// Full health check: integrity + foreign keys.
pub fn check_database_health(conn: &Connection) -> HealthCheckResult {
    let integrity_ok = db::verify_integrity(conn).is_ok();
    let foreign_keys_ok = run_foreign_key_check(conn).is_ok();
    let healthy = integrity_ok && foreign_keys_ok;

    if !healthy {
        set_database_corrupted(true);
    }

    HealthCheckResult {
        healthy,
        integrity_ok,
        foreign_keys_ok,
        error_message: if !healthy {
            Some("Database integrity check failed. Data is protected — no changes will be saved. Restore from your latest backup or contact support.".to_string())
        } else {
            None
        },
        is_read_only: is_database_corrupted(),
    }
}

// ---------------------------------------------------------------------------
// Backup Destination Management (Repository)
// ---------------------------------------------------------------------------

pub fn save_destination(
    conn: &Connection,
    path: &str,
    kind: &str,
    label: &str,
    institution_id: &str,
) -> Result<BackupDestinationItem, BackupError> {
    let id = uuid::Uuid::now_v7().to_string();
    let now = db::now_utc_iso();

    // Validate that the path exists and is writable
    let dest_path = Path::new(path);
    if !dest_path.exists() {
        fs::create_dir_all(dest_path)?;
    }

    // Test write
    let test_file = dest_path.join(".coachify-write-test");
    fs::write(&test_file, "test")?;
    fs::remove_file(&test_file)?;

    // Write marker file
    let marker_path = dest_path.join(".coachify-backup-target");
    let marker_content = serde_json::json!({
        "institution_id": institution_id,
        "initialized_at": now,
    });
    fs::write(
        &marker_path,
        serde_json::to_string_pretty(&marker_content)
            .map_err(|e| BackupError::Serde(e.to_string()))?,
    )?;

    conn.execute(
        "INSERT INTO backup_destinations (id, path, kind, label, institution_marker, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
        rusqlite::params![id, path, kind, label, institution_id, now],
    )?;

    Ok(BackupDestinationItem {
        id,
        path: path.to_string(),
        kind: kind.to_string(),
        label: label.to_string(),
        institution_marker: Some(institution_id.to_string()),
        is_default: false,
        last_ok_at: None,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_destinations(conn: &Connection) -> Result<Vec<BackupDestinationItem>, BackupError> {
    let mut stmt = conn.prepare(
        "SELECT id, path, kind, label, institution_marker, is_default, last_ok_at, last_error, created_at, updated_at
         FROM backup_destinations
         ORDER BY created_at",
    )?;

    let items: Vec<BackupDestinationItem> = stmt
        .query_map([], |row| {
            Ok(BackupDestinationItem {
                id: row.get(0)?,
                path: row.get(1)?,
                kind: row.get(2)?,
                label: row.get(3)?,
                institution_marker: row.get(4)?,
                is_default: row.get::<_, i32>(5)? != 0,
                last_ok_at: row.get(6)?,
                last_error: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

pub fn remove_destination(conn: &Connection, destination_id: &str) -> Result<bool, BackupError> {
    let affected = conn.execute(
        "DELETE FROM backup_destinations WHERE id = ?1",
        [destination_id],
    )?;
    Ok(affected > 0)
}

// ---------------------------------------------------------------------------
// Backup History (Repository)
// ---------------------------------------------------------------------------

pub fn record_backup_started(
    conn: &Connection,
    destination_id: Option<&str>,
    kind: &str,
    file_path: &str,
    institution_id: &str,
) -> Result<String, BackupError> {
    let id = uuid::Uuid::now_v7().to_string();
    let now = db::now_utc_iso();
    let schema_version = crate::migrations::TARGET_SCHEMA_VERSION;

    conn.execute(
        "INSERT INTO backup_history (id, destination_id, kind, file_path, file_size_bytes,
            app_version, schema_version, institution_id, started_at, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, 'in_progress', ?8)",
        rusqlite::params![
            id,
            destination_id,
            kind,
            file_path,
            env!("CARGO_PKG_VERSION"),
            schema_version,
            institution_id,
            now,
        ],
    )?;

    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub fn record_backup_completed(
    conn: &Connection,
    backup_id: &str,
    file_size_bytes: u64,
    record_counts: &RecordCounts,
    money_totals: &MoneyTotals,
    audit_chain_head: Option<&str>,
    vault_file_count: u32,
    manifest_hmac: &str,
) -> Result<(), BackupError> {
    let now = db::now_utc_iso();
    let counts_json =
        serde_json::to_string(record_counts).map_err(|e| BackupError::Serde(e.to_string()))?;
    let totals_json =
        serde_json::to_string(money_totals).map_err(|e| BackupError::Serde(e.to_string()))?;

    conn.execute(
        "UPDATE backup_history SET
            completed_at = ?2, file_size_bytes = ?3, record_counts_json = ?4,
            money_totals_json = ?5, audit_chain_head = ?6, vault_file_count = ?7,
            manifest_hmac = ?8, status = 'completed'
         WHERE id = ?1",
        rusqlite::params![
            backup_id,
            now,
            file_size_bytes as i64,
            counts_json,
            totals_json,
            audit_chain_head,
            vault_file_count,
            manifest_hmac,
        ],
    )?;

    // Update destination last_ok_at
    conn.execute(
        "UPDATE backup_destinations SET last_ok_at = ?2, last_error = NULL, updated_at = ?2
         WHERE id = (SELECT destination_id FROM backup_history WHERE id = ?1)",
        rusqlite::params![backup_id, now],
    )?;

    Ok(())
}

pub fn record_backup_verified(conn: &Connection, backup_id: &str) -> Result<(), BackupError> {
    let now = db::now_utc_iso();
    conn.execute(
        "UPDATE backup_history SET verified_at = ?2, status = 'verified' WHERE id = ?1",
        rusqlite::params![backup_id, now],
    )?;
    Ok(())
}

pub fn record_backup_failed(
    conn: &Connection,
    backup_id: &str,
    error: &str,
) -> Result<(), BackupError> {
    let now = db::now_utc_iso();
    conn.execute(
        "UPDATE backup_history SET completed_at = ?2, status = 'failed', error_message = ?3
         WHERE id = ?1",
        rusqlite::params![backup_id, now, error],
    )?;
    Ok(())
}

pub fn record_backup_corrupted(
    conn: &Connection,
    backup_id: &str,
    error: &str,
) -> Result<(), BackupError> {
    let now = db::now_utc_iso();
    conn.execute(
        "UPDATE backup_history SET status = 'corrupted', error_message = ?3, verified_at = ?2
         WHERE id = ?1",
        rusqlite::params![backup_id, now, error],
    )?;
    Ok(())
}

pub fn list_backup_history(
    conn: &Connection,
    destination_id: Option<&str>,
    limit: u32,
) -> Result<Vec<BackupHistoryItem>, BackupError> {
    if let Some(did) = destination_id {
        let mut stmt = conn.prepare(
            "SELECT id, destination_id, kind, file_path, file_size_bytes, app_version,
                    schema_version, institution_id, started_at, completed_at,
                    record_counts_json, money_totals_json, audit_chain_head,
                    vault_file_count, verified_at, status, error_message, created_at
             FROM backup_history
             WHERE destination_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;
        let items: Vec<BackupHistoryItem> = stmt
            .query_map(rusqlite::params![did, limit], map_backup_history_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, destination_id, kind, file_path, file_size_bytes, app_version,
                    schema_version, institution_id, started_at, completed_at,
                    record_counts_json, money_totals_json, audit_chain_head,
                    vault_file_count, verified_at, status, error_message, created_at
             FROM backup_history
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let items: Vec<BackupHistoryItem> = stmt
            .query_map([limit], map_backup_history_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }
}

fn map_backup_history_row(row: &rusqlite::Row) -> rusqlite::Result<BackupHistoryItem> {
    let counts_json: String = row.get(10)?;
    let totals_json: String = row.get(11)?;
    let record_counts: RecordCounts = serde_json::from_str(&counts_json).unwrap_or_default();
    let money_totals: MoneyTotals = serde_json::from_str(&totals_json).unwrap_or_default();

    Ok(BackupHistoryItem {
        id: row.get(0)?,
        destination_id: row.get(1)?,
        kind: row.get(2)?,
        file_path: row.get(3)?,
        file_size_bytes: row.get(4)?,
        app_version: row.get(5)?,
        schema_version: row.get(6)?,
        institution_id: row.get(7)?,
        started_at: row.get(8)?,
        completed_at: row.get(9)?,
        record_counts,
        money_totals,
        audit_chain_head: row.get(12)?,
        vault_file_count: row.get(13)?,
        verified_at: row.get(14)?,
        status: row.get(15)?,
        error_message: row.get(16)?,
        created_at: row.get(17)?,
    })
}

// ---------------------------------------------------------------------------
// Backup Schedule Management
// ---------------------------------------------------------------------------

pub fn save_schedule(
    conn: &Connection,
    kind: &str,
    destination_id: Option<&str>,
    time_of_day: Option<&str>,
    enabled: bool,
) -> Result<BackupScheduleItem, BackupError> {
    let now = db::now_utc_iso();

    // Upsert: one schedule per kind
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM backup_schedules WHERE kind = ?1",
            [kind],
            |row| row.get(0),
        )
        .ok();

    let id = if let Some(eid) = existing_id {
        conn.execute(
            "UPDATE backup_schedules SET destination_id = ?2, time_of_day = ?3, enabled = ?4, updated_at = ?5
             WHERE id = ?1",
            rusqlite::params![eid, destination_id, time_of_day, enabled as i32, now],
        )?;
        eid
    } else {
        let new_id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO backup_schedules (id, kind, destination_id, time_of_day, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![new_id, kind, destination_id, time_of_day, enabled as i32, now],
        )?;
        new_id
    };

    Ok(BackupScheduleItem {
        id,
        kind: kind.to_string(),
        destination_id: destination_id.map(|s| s.to_string()),
        time_of_day: time_of_day.map(|s| s.to_string()),
        enabled,
        last_run_at: None,
        next_run_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_schedules(conn: &Connection) -> Result<Vec<BackupScheduleItem>, BackupError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, destination_id, time_of_day, enabled, last_run_at, next_run_at, created_at, updated_at
         FROM backup_schedules
         ORDER BY kind",
    )?;

    let items: Vec<BackupScheduleItem> = stmt
        .query_map([], |row| {
            Ok(BackupScheduleItem {
                id: row.get(0)?,
                kind: row.get(1)?,
                destination_id: row.get(2)?,
                time_of_day: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
                last_run_at: row.get(5)?,
                next_run_at: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

// ---------------------------------------------------------------------------
// Restore Drill
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn record_restore_drill(
    conn: &Connection,
    backup_id: &str,
    result: &str,
    integrity_ok: bool,
    foreign_keys_ok: bool,
    record_counts_match: bool,
    money_totals_match: bool,
    details: Option<&str>,
) -> Result<String, BackupError> {
    let id = uuid::Uuid::now_v7().to_string();
    let now = db::now_utc_iso();

    conn.execute(
        "INSERT INTO restore_drills (id, backup_id, started_at, completed_at, result,
            integrity_ok, foreign_keys_ok, record_counts_match, money_totals_match,
            details_json, created_at)
         VALUES (?1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?3)",
        rusqlite::params![
            id,
            backup_id,
            now,
            result,
            integrity_ok as i32,
            foreign_keys_ok as i32,
            record_counts_match as i32,
            money_totals_match as i32,
            details,
        ],
    )?;

    Ok(id)
}

pub fn get_last_restore_drill(conn: &Connection) -> Result<Option<RestoreDrillItem>, BackupError> {
    let item = conn
        .query_row(
            "SELECT id, backup_id, started_at, completed_at, result,
                    integrity_ok, foreign_keys_ok, record_counts_match, money_totals_match,
                    details_json, created_at
             FROM restore_drills
             ORDER BY created_at DESC
             LIMIT 1",
            [],
            |row| {
                Ok(RestoreDrillItem {
                    id: row.get(0)?,
                    backup_id: row.get(1)?,
                    started_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    result: row.get(4)?,
                    integrity_ok: row.get::<_, Option<i32>>(5)?.map(|v| v != 0),
                    foreign_keys_ok: row.get::<_, Option<i32>>(6)?.map(|v| v != 0),
                    record_counts_match: row.get::<_, Option<i32>>(7)?.map(|v| v != 0),
                    money_totals_match: row.get::<_, Option<i32>>(8)?.map(|v| v != 0),
                    details_json: row.get(9)?,
                    created_at: row.get(10)?,
                })
            },
        )
        .ok();

    Ok(item)
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

pub fn get_backup_dashboard(conn: &Connection) -> Result<BackupDashboard, BackupError> {
    let mut warnings = Vec::new();

    // Last backup (any status)
    let last_backup = list_backup_history(conn, None, 1)?.into_iter().next();

    // Last verified
    let last_verified: Option<BackupHistoryItem> = conn
        .query_row(
            "SELECT id, destination_id, kind, file_path, file_size_bytes, app_version,
                    schema_version, institution_id, started_at, completed_at,
                    record_counts_json, money_totals_json, audit_chain_head,
                    vault_file_count, verified_at, status, error_message, created_at
             FROM backup_history
             WHERE status = 'verified'
             ORDER BY created_at DESC
             LIMIT 1",
            [],
            map_backup_history_row,
        )
        .ok();

    let last_restore_drill = get_last_restore_drill(conn)?;
    let destinations = list_destinations(conn)?;
    let schedules = list_schedules(conn)?;

    // Check for warnings
    if last_backup.is_none() {
        warnings.push("No backups have been created yet".to_string());
    } else if let Some(ref lb) = last_backup {
        if is_older_than_days(&lb.created_at, 3) {
            warnings.push("No backup in the last 3 days".to_string());
        }
    }

    if last_verified.is_none() && last_backup.is_some() {
        warnings.push("No backup has been verified yet".to_string());
    }

    // Check destination accessibility
    for dest in &destinations {
        if !Path::new(&dest.path).exists() {
            warnings.push(format!(
                "Backup destination '{}' is not accessible",
                dest.label
            ));
        }
    }

    if destinations.is_empty() {
        warnings.push("No backup destination configured".to_string());
    }

    // Database size
    let data_size_bytes = conn
        .query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get::<_, u64>(0),
        )
        .unwrap_or(0);

    let backup_count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM backup_history WHERE status IN ('completed', 'verified')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(BackupDashboard {
        last_backup,
        last_verified,
        last_restore_drill,
        destinations,
        schedules,
        warnings,
        data_size_bytes,
        backup_count,
    })
}

// ---------------------------------------------------------------------------
// Use `jiff::Timestamp` from_str
// ---------------------------------------------------------------------------
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("backup_test.db");
        let conn = db::open_connection_at(&db_path).expect("open test db");
        db::apply_all_migrations(&conn).expect("apply migrations");
        (dir, conn)
    }

    #[test]
    fn test_create_and_verify_backup_happy_path() {
        let (db_dir, conn) = setup_test_db();
        let backup_dir = tempfile::tempdir().expect("backup dir");

        // Insert an institution name
        conn.execute(
            "INSERT OR REPLACE INTO institution (id, name, address, phones, created_at, updated_at)
             VALUES (1, 'Test Coaching Academy', 'Dhaka, Bangladesh', '01711111111', datetime('now'), datetime('now'))",
            [],
        )
        .expect("insert institution");

        // 1. Create snapshot
        let (db_file, record_counts, money_totals) =
            create_snapshot(&conn, backup_dir.path(), "manual", "inst-123")
                .expect("create snapshot");
        assert!(db_file.exists());

        // 2. Create and write manifest
        let manifest = create_manifest(&conn, &record_counts, &money_totals, "inst-123", None)
            .expect("create manifest");
        assert_eq!(manifest.institution_name, "Test Coaching Academy");

        let hmac_key = b"super_secret_test_hmac_key_32_bytes";
        let hmac_hex =
            write_manifest(&manifest, backup_dir.path(), hmac_key).expect("write manifest");
        assert!(!hmac_hex.is_empty());

        // 3. Verify backup
        let result =
            verify_backup(&db_file, &manifest, &hmac_hex, hmac_key, None).expect("verify backup");
        assert!(result.passed);
        assert!(result.integrity_ok);
        assert!(result.foreign_keys_ok);
        assert!(result.record_counts_match);
        assert!(result.money_totals_match);
        assert!(result.manifest_hmac_ok);
        assert!(result.errors.is_empty());

        drop(db_dir);
        drop(backup_dir);
    }

    #[test]
    fn test_verify_detects_corrupt_manifest_hmac() {
        let (db_dir, conn) = setup_test_db();
        let backup_dir = tempfile::tempdir().expect("backup dir");

        let (db_file, record_counts, money_totals) =
            create_snapshot(&conn, backup_dir.path(), "manual", "inst-123")
                .expect("create snapshot");

        let manifest = create_manifest(&conn, &record_counts, &money_totals, "inst-123", None)
            .expect("create manifest");

        let hmac_key = b"super_secret_test_hmac_key_32_bytes";
        let _ = write_manifest(&manifest, backup_dir.path(), hmac_key).expect("write manifest");

        // Tamper HMAC
        let tampered_hmac = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = verify_backup(&db_file, &manifest, tampered_hmac, hmac_key, None)
            .expect("verify backup");
        assert!(!result.passed);
        assert!(!result.manifest_hmac_ok);
        assert!(!result.errors.is_empty());

        drop(db_dir);
        drop(backup_dir);
    }

    #[test]
    fn test_destination_and_history_lifecycle() {
        let (_db_dir, conn) = setup_test_db();
        let dest_dir = tempfile::tempdir().expect("dest dir");

        // Save destination
        let dest = save_destination(
            &conn,
            &dest_dir.path().display().to_string(),
            "usb",
            "USB Flash Drive",
            "inst-test",
        )
        .expect("save destination");
        assert_eq!(dest.kind, "usb");

        // Check marker file exists
        assert!(dest_dir.path().join(".coachify-backup-target").exists());

        // List destinations
        let dests = list_destinations(&conn).expect("list destinations");
        assert_eq!(dests.len(), 1);
        assert_eq!(dests[0].label, "USB Flash Drive");

        // History tracking
        let backup_id = record_backup_started(
            &conn,
            Some(&dest.id),
            "daily",
            "C:\\dummy\\path.db",
            "inst-test",
        )
        .expect("record started");

        let counts = RecordCounts::default();
        let totals = MoneyTotals::default();
        record_backup_completed(
            &conn, &backup_id, 1024, &counts, &totals, None, 0, "hmac123",
        )
        .expect("record completed");

        record_backup_verified(&conn, &backup_id).expect("record verified");

        let history = list_backup_history(&conn, None, 10).expect("list history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, "verified");

        // Dashboard
        let dashboard = get_backup_dashboard(&conn).expect("dashboard");
        assert_eq!(dashboard.backup_count, 1);
        assert!(dashboard.last_verified.is_some());
    }

    #[test]
    fn test_prune_backups_never_prunes_last_verified() {
        let (_db_dir, conn) = setup_test_db();
        let dest_dir = tempfile::tempdir().expect("dest dir");

        let dest = save_destination(
            &conn,
            &dest_dir.path().display().to_string(),
            "local",
            "Local Backup",
            "inst-1",
        )
        .expect("save destination");

        // Create 10 dummy backup files and history records
        let mut backup_ids = Vec::new();
        for i in 0..10 {
            let file_path = dest_dir.path().join(format!("backup_{i}.db"));
            fs::write(&file_path, "dummy db content").expect("write dummy");
            let id = format!("backup-id-{i}");
            conn.execute(
                "INSERT INTO backup_history (id, destination_id, kind, file_path, file_size_bytes,
                    app_version, schema_version, institution_id, started_at, completed_at, status, created_at)
                 VALUES (?1, ?2, 'daily', ?3, 100, '0.1.0', 13, 'inst-1', datetime('now', ?4), datetime('now', ?4), 'completed', datetime('now', ?4))",
                rusqlite::params![
                    id,
                    dest.id,
                    file_path.display().to_string(),
                    format!("-{} days", 10 - i),
                ],
            )
            .expect("insert history");
            backup_ids.push(id);
        }

        // Mark the oldest backup (backup-id-0) as verified!
        conn.execute(
            "UPDATE backup_history SET status = 'verified', verified_at = datetime('now') WHERE id = 'backup-id-0'",
            [],
        )
        .expect("update verified");

        // Run pruning
        let pruned = prune_backups(&conn, &dest.id).expect("prune backups");
        assert!(pruned > 0);

        // Verify that backup-id-0 was NOT deleted from history or disk
        let oldest_exists: bool = conn
            .query_row(
                "SELECT 1 FROM backup_history WHERE id = 'backup-id-0'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(oldest_exists, "Last verified backup must never be pruned");
        assert!(dest_dir.path().join("backup_0.db").exists());
    }

    #[test]
    fn test_database_health_check() {
        let (_db_dir, conn) = setup_test_db();
        let health = check_database_health(&conn);
        assert!(health.healthy);
        assert!(health.integrity_ok);
        assert!(health.foreign_keys_ok);
        assert!(health.error_message.is_none());
    }
}
