use crate::db;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{Duration, Instant};
use thiserror::Error;

pub const MIGRATION_0009_SQL: &str = include_str!("../../../migrations/0009_migration_engine.sql");

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    #[error("Insufficient disk storage: available {available} bytes, required {required} bytes (2.5x database size)")]
    InsufficientStorage { available: u64, required: u64 },
    #[error("Database migration lock held by another process: {0}")]
    LockConflict(String),
    #[error("Validation gate failed: {0}")]
    ValidationFailed(String),
    #[error("Database error during migration v{version} ({name}): {error}")]
    ExecutionFailed {
        version: u32,
        name: String,
        error: String,
    },
    #[error("Database schema version {current} is newer than binary supported version {target}. Update Coachify.")]
    DowngradePrevented { current: u32, target: u32 },
    #[error("File operation error: {0}")]
    Io(String),
    #[error("Database error: {0}")]
    Sql(String),
}

impl MigrationError {
    /// Stable client-facing error code, distinct per variant, for RPC error mapping.
    pub fn code(&self) -> &'static str {
        match self {
            MigrationError::InsufficientStorage { .. } => "migration_insufficient_storage",
            MigrationError::LockConflict(_) => "migration_lock_conflict",
            MigrationError::ValidationFailed(_) => "migration_validation_failed",
            MigrationError::ExecutionFailed { .. } => "migration_execution_failed",
            MigrationError::DowngradePrevented { .. } => "migration_downgrade_prevented",
            MigrationError::Io(_) => "migration_io_error",
            MigrationError::Sql(_) => "migration_sql_error",
        }
    }
}

impl From<rusqlite::Error> for MigrationError {
    fn from(err: rusqlite::Error) -> Self {
        MigrationError::Sql(err.to_string())
    }
}

impl From<std::io::Error> for MigrationError {
    fn from(err: std::io::Error) -> Self {
        MigrationError::Io(err.to_string())
    }
}

#[derive(Clone, Copy)]
pub enum MigrationStep {
    Sql(&'static str),
    Data(fn(&Transaction) -> Result<(), MigrationError>),
}

#[derive(Clone, Copy)]
pub struct MigrationDefinition {
    pub version: u32,
    pub name: &'static str,
    pub sha256: &'static str,
    pub step: MigrationStep,
}

pub const MIGRATIONS: &[MigrationDefinition] = &[
    MigrationDefinition {
        version: 1,
        name: "0001_scaffold.sql",
        sha256: "55a376fd4078b504e436601eadf4aa9949aaa86f969d6e67972886e1b9359fdd",
        step: MigrationStep::Sql(db::MIGRATION_0001_SQL),
    },
    MigrationDefinition {
        version: 2,
        name: "0002_system_config.sql",
        sha256: "62d6eeaa52510434621227cb2d85147d56c0dc019c0c35d5f2ac2e85db63698f",
        step: MigrationStep::Sql(db::MIGRATION_0002_SQL),
    },
    MigrationDefinition {
        version: 3,
        name: "0003_people_directory.sql",
        sha256: "d91729ce6cb912d28badc7618aaa54b3284ed5521bee0a9d86cbc1e75aa20736",
        step: MigrationStep::Sql(db::MIGRATION_0003_SQL),
    },
    MigrationDefinition {
        version: 4,
        name: "0004_academic_structure.sql",
        sha256: "b912ff4af5f5c5b732698d6d2598b910a47c0e8e5ab7d04db7bda4c170a4bfea",
        step: MigrationStep::Sql(db::MIGRATION_0004_SQL),
    },
    MigrationDefinition {
        version: 5,
        name: "0005_financial_ledger.sql",
        sha256: "8f3a3e1231748b4872379a86c89b7001bbed308ba5579d50fe60df2d1f1d7554",
        step: MigrationStep::Sql(db::MIGRATION_0005_SQL),
    },
    MigrationDefinition {
        version: 6,
        name: "0006_audit_and_triggers.sql",
        sha256: "4ec233d90c3bd5771266925b9c431e638fa9692b898a1f6c9d728c7567a3527d",
        step: MigrationStep::Sql(db::MIGRATION_0006_SQL),
    },
    MigrationDefinition {
        version: 7,
        name: "0007_accounts_and_dispatcher.sql",
        sha256: "056b9389bb03e0ac23b5397e6491da6bec2277ee56b6be166b695647595a5c20",
        step: MigrationStep::Sql(db::MIGRATION_0007_SQL),
    },
    MigrationDefinition {
        version: 8,
        name: "0008_audit_indexes_and_permissions.sql",
        sha256: "313675133efd58cb30aabbe3d3fac49a32497afbd9ea2d9beb081f65ae4e047a",
        step: MigrationStep::Sql(db::MIGRATION_0008_SQL),
    },
    MigrationDefinition {
        version: 9,
        name: "0009_migration_engine.sql",
        sha256: "569c127e3facbd1291c62ca7fe766be3d84167660dd94d2d5f10f778384056eb",
        step: MigrationStep::Sql(MIGRATION_0009_SQL),
    },
    MigrationDefinition {
        version: 10,
        name: "0010_student_search_and_triggers.sql",
        sha256: "62ac4e571ae140c53878ba5e5fcefee3f7efc80e1777d9af338a2fe8d6d2af03",
        step: MigrationStep::Sql(db::MIGRATION_0010_SQL),
    },
    MigrationDefinition {
        version: 11,
        name: "0011_academic_operations_and_triggers.sql",
        sha256: "c7143885b62bb040031e6ada0fb34b757fe71e849e7792a8dc0f1a858a5efdaf",
        step: MigrationStep::Sql(db::MIGRATION_0011_SQL),
    },
    MigrationDefinition {
        version: 12,
        name: "0012_financial_ledger_refinements.sql",
        sha256: "91ef0d984fa7ef4e41e2aa35606fac410628285aeec541cd28f178d5d5c8369d",
        step: MigrationStep::Sql(db::MIGRATION_0012_SQL),
    },
    MigrationDefinition {
        version: 13,
        name: "0013_backup_infrastructure.sql",
        sha256: "04408114f696398f4891baed3d57952bcc606f415a55fa65b4a5c09330b70705",
        step: MigrationStep::Sql(db::MIGRATION_0013_SQL),
    },
    MigrationDefinition {
        version: 14,
        name: "0014_attendance_and_messaging.sql",
        sha256: "99d38f074d4e6c6c39ba142e0b21c9dc751e1c9c43c804e6105c3c991b8a1534",
        step: MigrationStep::Sql(db::MIGRATION_0014_SQL),
    },
    MigrationDefinition {
        version: 15,
        name: "0015_exams_and_results.sql",
        sha256: "d69f4208523e5e48e57a256b2541cb0d9fd069926c5b1189c46568d96a190c0c",
        step: MigrationStep::Sql(db::MIGRATION_0015_SQL),
    },
    MigrationDefinition {
        version: 16,
        name: "0016_role_dashboards_and_fast_search.sql",
        sha256: "1886c143bba854e53563ae59c67d18151af367a85e5ecbe3d3f5151d9a2bfe97",
        step: MigrationStep::Sql(db::MIGRATION_0016_SQL),
    },
    MigrationDefinition {
        version: 17,
        name: "0017_owner_intelligence.sql",
        sha256: "d027d357e1bbcbf755f689ef6e66aea0ce93b08e6a0de5830684688cc969547a",
        step: MigrationStep::Sql(db::MIGRATION_0017_SQL),
    },
    MigrationDefinition {
        version: 18,
        name: "0018_operational_integrity.sql",
        sha256: "b061e09b24c6a649bc21a8196d4ec65472fadcecaa415ae9c09f8b96453a331a",
        step: MigrationStep::Sql(db::MIGRATION_0018_SQL),
    },
    MigrationDefinition {
        version: 19,
        name: "0019_professional_printing.sql",
        sha256: "db1fedc61f55469d471802843a210405e57270e88bfbca881bbd4f5492ec5a14",
        step: MigrationStep::Sql(db::MIGRATION_0019_SQL),
    },
    MigrationDefinition {
        version: 20,
        name: "0020_ai_foundation.sql",
        sha256: "98c82309fc03eda867705c1279037446dbeb4334a0b0b0b7e70932074a3ed072",
        step: MigrationStep::Sql(db::MIGRATION_0020_SQL),
    },
];

/// Derived from the last entry in the compiled catalog rather than hardcoded,
/// so the two can never drift apart.
pub const TARGET_SCHEMA_VERSION: u32 = MIGRATIONS[MIGRATIONS.len() - 1].version;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PendingMigrationInfo {
    pub version: u32,
    pub name: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MigrationStatusResponse {
    pub current_version: u32,
    pub target_version: u32,
    pub is_upgrade_needed: bool,
    pub pending_migrations: Vec<PendingMigrationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MigrationHistoryItem {
    pub version: u32,
    pub name: String,
    pub sha256: String,
    pub applied_at: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MigrationHistoryResponse {
    pub history: Vec<MigrationHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecoveryInfoResponse {
    pub is_in_recovery: bool,
    pub failed_version: Option<u32>,
    pub error_message: Option<String>,
    pub active_db_path: String,
    pub archive_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OpenDbDirectoryResponse {
    pub success: bool,
    pub path: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportDiagnosticsResponse {
    pub json: String,
}

// -----------------------------------------------------------------------------
// Global Recovery State
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GlobalRecoveryState {
    pub is_in_recovery: bool,
    pub failed_version: Option<u32>,
    pub error_message: Option<String>,
    pub active_db_path: PathBuf,
    pub archive_paths: Vec<PathBuf>,
}

static RECOVERY_STATE: RwLock<Option<GlobalRecoveryState>> = RwLock::new(None);

pub fn set_recovery_state(
    failed_version: Option<u32>,
    error_message: String,
    active_db_path: PathBuf,
) {
    let archive_paths = list_archives(&active_db_path);
    let mut lock = RECOVERY_STATE.write().unwrap();
    *lock = Some(GlobalRecoveryState {
        is_in_recovery: true,
        failed_version,
        error_message: Some(error_message),
        active_db_path,
        archive_paths,
    });
}

pub fn clear_recovery_state() {
    let mut lock = RECOVERY_STATE.write().unwrap();
    *lock = None;
}

pub fn is_in_recovery() -> bool {
    let lock = RECOVERY_STATE.read().unwrap();
    lock.as_ref().map(|s| s.is_in_recovery).unwrap_or(false)
}

pub fn get_recovery_info(default_db_path: &Path) -> RecoveryInfoResponse {
    let lock = RECOVERY_STATE.read().unwrap();
    if let Some(state) = lock.as_ref() {
        RecoveryInfoResponse {
            is_in_recovery: state.is_in_recovery,
            failed_version: state.failed_version,
            error_message: state.error_message.as_deref().map(sanitize_error_string),
            active_db_path: state.active_db_path.display().to_string(),
            archive_paths: state
                .archive_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        }
    } else {
        let archives = list_archives(default_db_path);
        RecoveryInfoResponse {
            is_in_recovery: false,
            failed_version: None,
            error_message: None,
            active_db_path: default_db_path.display().to_string(),
            archive_paths: archives.iter().map(|p| p.display().to_string()).collect(),
        }
    }
}

// -----------------------------------------------------------------------------
// Operating System File Locking
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct MigrationLock {
    #[allow(dead_code)]
    file: fs::File,
    pub path: PathBuf,
}

#[cfg(windows)]
impl MigrationLock {
    pub fn acquire(dir: &Path) -> Result<Self, MigrationError> {
        use std::os::windows::io::AsRawHandle;
        let lock_path = dir.join("migration.active.lock");
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| {
                MigrationError::Io(format!("Failed to open lock file {:?}: {}", lock_path, e))
            })?;

        let raw_handle = file.as_raw_handle();
        // LOCKFILE_EXCLUSIVE_LOCK = 0x02, LOCKFILE_FAIL_IMMEDIATELY = 0x01
        let flags: u32 = 0x00000002 | 0x00000001;
        let mut overlapped = [0u8; 32];

        extern "system" {
            fn LockFileEx(
                hFile: *mut std::ffi::c_void,
                dwFlags: u32,
                dwReserved: u32,
                nNumberOfBytesToLockLow: u32,
                nNumberOfBytesToLockHigh: u32,
                lpOverlapped: *mut std::ffi::c_void,
            ) -> i32;
        }

        let ret = unsafe {
            LockFileEx(
                raw_handle,
                flags,
                0,
                1,
                0,
                overlapped.as_mut_ptr() as *mut std::ffi::c_void,
            )
        };

        if ret == 0 {
            let err = std::io::Error::last_os_error();
            return Err(MigrationError::LockConflict(format!(
                "Exclusive lock on {:?} could not be acquired (another migration is running): {}",
                lock_path, err
            )));
        }

        Ok(Self {
            file,
            path: lock_path,
        })
    }
}

#[cfg(windows)]
impl Drop for MigrationLock {
    fn drop(&mut self) {
        use std::os::windows::io::AsRawHandle;
        extern "system" {
            fn UnlockFileEx(
                hFile: *mut std::ffi::c_void,
                dwReserved: u32,
                nNumberOfBytesToUnlockLow: u32,
                nNumberOfBytesToUnlockHigh: u32,
                lpOverlapped: *mut std::ffi::c_void,
            ) -> i32;
        }
        let mut overlapped = [0u8; 32];
        unsafe {
            UnlockFileEx(
                self.file.as_raw_handle(),
                0,
                1,
                0,
                overlapped.as_mut_ptr() as *mut std::ffi::c_void,
            );
        }
    }
}

#[cfg(not(windows))]
impl MigrationLock {
    pub fn acquire(dir: &Path) -> Result<Self, MigrationError> {
        let lock_path = dir.join("migration.active.lock");
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| MigrationError::Io(e.to_string()))?;
        Ok(Self {
            file,
            path: lock_path,
        })
    }
}

// -----------------------------------------------------------------------------
// Pre-flight Storage Check
// -----------------------------------------------------------------------------

/// Queries free disk space for the volume containing `dir` via
/// `GetDiskFreeSpaceExW`. Returns 0 on non-Windows targets or on query failure.
pub fn get_free_disk_bytes(dir: &Path) -> u64 {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = dir.as_os_str().encode_wide().collect();
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

        if success == 0 {
            return 0;
        }

        free_bytes_available
    }

    #[cfg(not(windows))]
    {
        let _ = dir;
        0
    }
}

pub fn check_free_space(
    db_path: &Path,
    mock_free_bytes: Option<u64>,
) -> Result<(), MigrationError> {
    let db_size = if db_path.exists() {
        fs::metadata(db_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let required = (db_size.max(4096) * 25) / 10;

    if let Some(free) = mock_free_bytes {
        if free < required {
            return Err(MigrationError::InsufficientStorage {
                available: free,
                required,
            });
        }
        return Ok(());
    }

    #[cfg(windows)]
    {
        let dir = db_path.parent().unwrap_or_else(|| Path::new("."));
        let free_bytes_available = get_free_disk_bytes(dir);

        if free_bytes_available < required {
            return Err(MigrationError::InsufficientStorage {
                available: free_bytes_available,
                required,
            });
        }
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Financial Ledger Invariant Baseline
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableBaseline {
    pub count: i64,
    pub sum_1: i64,
    pub sum_2: i64,
    pub sum_3: i64,
    pub sum_4: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialLedgerBaseline {
    pub payments: Option<TableBaseline>,
    pub charges: Option<TableBaseline>,
    pub payment_allocations: Option<TableBaseline>,
    pub adjustments: Option<TableBaseline>,
    pub refunds: Option<TableBaseline>,
    pub cash_days: Option<TableBaseline>,
}

fn table_exists(conn: &Connection, table_name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table_name],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

impl FinancialLedgerBaseline {
    pub fn capture(conn: &Connection) -> Result<Self, MigrationError> {
        let payments = if table_exists(conn, "payments") {
            let (count, sum): (i64, i64) = conn.query_row(
                "SELECT count(*), coalesce(sum(amount_poisha), 0) FROM payments",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Some(TableBaseline {
                count,
                sum_1: sum,
                sum_2: 0,
                sum_3: 0,
                sum_4: 0,
            })
        } else {
            None
        };

        let charges = if table_exists(conn, "charges") {
            let (count, net_sum, gross_sum): (i64, i64, i64) = conn.query_row(
                "SELECT count(*), coalesce(sum(net_amount_poisha), 0), coalesce(sum(gross_amount_poisha), 0) FROM charges",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            Some(TableBaseline {
                count,
                sum_1: net_sum,
                sum_2: gross_sum,
                sum_3: 0,
                sum_4: 0,
            })
        } else {
            None
        };

        let payment_allocations = if table_exists(conn, "payment_allocations") {
            let (count, sum): (i64, i64) = conn.query_row(
                "SELECT count(*), coalesce(sum(amount_poisha), 0) FROM payment_allocations",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Some(TableBaseline {
                count,
                sum_1: sum,
                sum_2: 0,
                sum_3: 0,
                sum_4: 0,
            })
        } else {
            None
        };

        let adjustments = if table_exists(conn, "adjustments") {
            let (count, sum): (i64, i64) = conn.query_row(
                "SELECT count(*), coalesce(sum(amount_poisha), 0) FROM adjustments",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Some(TableBaseline {
                count,
                sum_1: sum,
                sum_2: 0,
                sum_3: 0,
                sum_4: 0,
            })
        } else {
            None
        };

        let refunds = if table_exists(conn, "refunds") {
            let (count, sum): (i64, i64) = conn.query_row(
                "SELECT count(*), coalesce(sum(amount_poisha), 0) FROM refunds",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Some(TableBaseline {
                count,
                sum_1: sum,
                sum_2: 0,
                sum_3: 0,
                sum_4: 0,
            })
        } else {
            None
        };

        let cash_days = if table_exists(conn, "cash_days") {
            let (count, s1, s2, s3, s4): (i64, i64, i64, i64, i64) = conn.query_row(
                "SELECT count(*),
                        coalesce(sum(opening_cash_poisha), 0),
                        coalesce(sum(cash_in_poisha), 0),
                        coalesce(sum(cash_out_poisha), 0),
                        coalesce(sum(expected_closing_poisha), 0)
                 FROM cash_days",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )?;
            Some(TableBaseline {
                count,
                sum_1: s1,
                sum_2: s2,
                sum_3: s3,
                sum_4: s4,
            })
        } else {
            None
        };

        Ok(Self {
            payments,
            charges,
            payment_allocations,
            adjustments,
            refunds,
            cash_days,
        })
    }

    pub fn assert_matches(&self, conn: &Connection) -> Result<(), MigrationError> {
        let current = Self::capture(conn)?;

        let check = |name: &str, before: &Option<TableBaseline>, after: &Option<TableBaseline>| {
            match (before, after) {
                (Some(b), Some(a)) => {
                    if b != a {
                        return Err(MigrationError::ValidationFailed(format!(
                            "Financial invariant altered in table {}: before={:?}, after={:?}",
                            name, b, a
                        )));
                    }
                }
                (Some(_), None) => {
                    return Err(MigrationError::ValidationFailed(format!(
                        "Financial invariant violated: ledger table {} existed before the migration but is missing afterward",
                        name
                    )));
                }
                (None, Some(_)) | (None, None) => {}
            }
            Ok(())
        };

        check("payments", &self.payments, &current.payments)?;
        check("charges", &self.charges, &current.charges)?;
        check(
            "payment_allocations",
            &self.payment_allocations,
            &current.payment_allocations,
        )?;
        check("adjustments", &self.adjustments, &current.adjustments)?;
        check("refunds", &self.refunds, &current.refunds)?;
        check("cash_days", &self.cash_days, &current.cash_days)?;

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Validation Gates
// -----------------------------------------------------------------------------

pub fn run_validation_gates(
    conn: &Connection,
    baseline: &FinancialLedgerBaseline,
) -> Result<(), MigrationError> {
    // Gate 1: Integrity check
    let mut stmt = conn.prepare("PRAGMA integrity_check;").map_err(|e| {
        MigrationError::ValidationFailed(format!("Integrity prepare failed: {}", e))
    })?;
    let mut rows = stmt
        .query([])
        .map_err(|e| MigrationError::ValidationFailed(format!("Integrity query failed: {}", e)))?;

    if let Some(row) = rows.next()? {
        let status: String = row.get(0)?;
        if status != "ok" {
            return Err(MigrationError::ValidationFailed(format!(
                "PRAGMA integrity_check reported corruption: {}",
                status
            )));
        }
    } else {
        return Err(MigrationError::ValidationFailed(
            "No output from PRAGMA integrity_check".to_string(),
        ));
    }

    // Gate 2: Foreign key check
    let mut fk_stmt = conn
        .prepare("PRAGMA foreign_key_check;")
        .map_err(|e| MigrationError::ValidationFailed(format!("FK check prepare failed: {}", e)))?;
    let mut fk_rows = fk_stmt
        .query([])
        .map_err(|e| MigrationError::ValidationFailed(format!("FK check query failed: {}", e)))?;

    if let Some(row) = fk_rows.next()? {
        let table: String = row.get(0)?;
        let rowid: i64 = row.get(1)?;
        let target: String = row.get(2)?;
        let fkid: i64 = row.get(3)?;
        return Err(MigrationError::ValidationFailed(format!(
            "Foreign key violation detected: table={}, rowid={}, target={}, fkid={}",
            table, rowid, target, fkid
        )));
    }

    // Gate 3: Financial ledger invariant audit
    baseline.assert_matches(conn)?;

    Ok(())
}

// -----------------------------------------------------------------------------
// Schema Version Query and Inspection
// -----------------------------------------------------------------------------

pub fn inspect_schema_version(conn: &Connection) -> Result<u32, MigrationError> {
    let mut ver_m = 0;
    if table_exists(conn, "_migrations") {
        let max_ver: Option<u32> = conn
            .query_row("SELECT max(version) FROM _migrations", [], |row| row.get(0))
            .map_err(|e| MigrationError::Sql(format!("Failed to read _migrations version: {e}")))?;
        if let Some(v) = max_ver {
            ver_m = v;
        }
    }

    let mut ver_s = 0;
    if table_exists(conn, "_schema_version") {
        let max_ver: Option<u32> = conn
            .query_row("SELECT max(version) FROM _schema_version", [], |row| {
                row.get(0)
            })
            .map_err(|e| {
                MigrationError::Sql(format!("Failed to read _schema_version version: {e}"))
            })?;
        if let Some(v) = max_ver {
            ver_s = v;
        }
    }

    Ok(ver_m.max(ver_s))
}

pub fn get_migration_status(conn: &Connection) -> Result<MigrationStatusResponse, MigrationError> {
    let current_version = inspect_schema_version(conn)?;
    let target_version = TARGET_SCHEMA_VERSION;
    let is_upgrade_needed = current_version < target_version;

    let pending_migrations = MIGRATIONS
        .iter()
        .filter(|m| m.version > current_version)
        .map(|m| PendingMigrationInfo {
            version: m.version,
            name: m.name.to_string(),
            sha256: m.sha256.to_string(),
        })
        .collect();

    Ok(MigrationStatusResponse {
        current_version,
        target_version,
        is_upgrade_needed,
        pending_migrations,
    })
}

pub fn get_migration_history(
    conn: &Connection,
) -> Result<MigrationHistoryResponse, MigrationError> {
    if !table_exists(conn, "_migrations") {
        return Ok(MigrationHistoryResponse { history: vec![] });
    }

    let mut stmt = conn.prepare(
        "SELECT version, name, sha256, applied_at, execution_time_ms
         FROM _migrations
         ORDER BY version ASC",
    )?;

    let history = stmt
        .query_map([], |row| {
            Ok(MigrationHistoryItem {
                version: row.get(0)?,
                name: row.get(1)?,
                sha256: row.get(2)?,
                applied_at: row.get(3)?,
                execution_time_ms: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(MigrationHistoryResponse { history })
}

// -----------------------------------------------------------------------------
// Rename and Cleanup Helpers
// -----------------------------------------------------------------------------

pub fn rename_with_retry(from: &Path, to: &Path) -> Result<(), MigrationError> {
    let mut delay = Duration::from_millis(50);
    for attempt in 0..15 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) if attempt == 14 => {
                return Err(MigrationError::Io(format!(
                    "Failed to rename {:?} to {:?} after 15 attempts: {}",
                    from, to, e
                )));
            }
            Err(_) => {
                std::thread::sleep(delay);
                delay = (delay * 2).min(Duration::from_millis(1000));
            }
        }
    }
    unreachable!("loop always returns via Ok or the attempt == 14 error branch");
}

pub fn purge_sidecars(db_path: &Path) {
    let wal = db_path.with_extension(format!(
        "{}-wal",
        db_path.extension().and_then(|s| s.to_str()).unwrap_or("db")
    ));
    let shm = db_path.with_extension(format!(
        "{}-shm",
        db_path.extension().and_then(|s| s.to_str()).unwrap_or("db")
    ));
    let _ = fs::remove_file(wal);
    let _ = fs::remove_file(shm);
}

/// Sort key for an archive file: the fixed-width `%Y%m%d%H%M%S` timestamp
/// segment of `pre-migrate-v{version}-{timestamp}.db`, which sorts correctly
/// as text on its own. The version segment must never be used for ordering:
/// it is variable-width, so `"v10"` sorts below `"v9"` lexicographically.
/// Falls back to file modified time when the name does not parse.
fn archive_sort_key(path: &Path) -> String {
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        if let Some(stem) = file_name.strip_suffix(".db") {
            if let Some(idx) = stem.rfind('-') {
                let ts = &stem[idx + 1..];
                if ts.len() == 14 && ts.chars().all(|c| c.is_ascii_digit()) {
                    return ts.to_string();
                }
            }
        }
    }

    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| format!("{:020}", d.as_millis()))
        .unwrap_or_default()
}

pub fn list_archives(db_path: &Path) -> Vec<PathBuf> {
    let parent = db_path.parent().unwrap_or_else(|| Path::new("."));
    let mut archives = Vec::new();

    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.starts_with("pre-migrate-v") && file_name.ends_with(".db") {
                    archives.push(path);
                }
            }
        }
    }

    archives.sort_by_key(|a| std::cmp::Reverse(archive_sort_key(a)));
    archives
}

pub fn prune_old_archives(db_path: &Path) {
    let archives = list_archives(db_path);
    let thirty_days = Duration::from_secs(30 * 24 * 3600);
    let now = std::time::SystemTime::now();

    for path in archives {
        if let Ok(meta) = fs::metadata(&path) {
            let created = meta.created().or_else(|_| meta.modified());
            if let Ok(created) = created {
                if let Ok(age) = now.duration_since(created) {
                    if age > thirty_days {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Interrupted Swap Recovery
// -----------------------------------------------------------------------------

pub fn check_and_recover_interrupted_swap(active_path: &Path) -> Result<(), MigrationError> {
    let parent = active_path.parent().unwrap_or_else(|| Path::new("."));
    let staging_path = parent.join("staging.db");

    if !active_path.exists() {
        if staging_path.exists() {
            // Check if staging is valid and intact
            let is_valid = match Connection::open(&staging_path) {
                Ok(conn) => {
                    let quick_check: Result<String, _> =
                        conn.query_row("PRAGMA quick_check;", [], |r| r.get(0));
                    quick_check.map(|s| s == "ok").unwrap_or(false)
                }
                Err(_) => false,
            };

            if is_valid {
                rename_with_retry(&staging_path, active_path)?;
                purge_sidecars(&staging_path);
                return Ok(());
            } else {
                let _ = fs::remove_file(&staging_path);
                purge_sidecars(&staging_path);
            }
        }

        // Active path missing and staging not valid; restore from newest pre-migrate archive
        let archives = list_archives(active_path);
        if let Some(newest_archive) = archives.first() {
            fs::copy(newest_archive, active_path).map_err(|e| {
                MigrationError::Io(format!(
                    "Failed to restore archive {:?} to active {:?}: {}",
                    newest_archive, active_path, e
                ))
            })?;
            return Ok(());
        }
    } else if staging_path.exists() {
        // Active database exists; leftover staging from an aborted previous run must be purged
        let _ = fs::remove_file(&staging_path);
        purge_sidecars(&staging_path);
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Staging Copy Migration Engine
// -----------------------------------------------------------------------------

pub fn run_migrations_with_staging(
    active_path: &Path,
    mock_free_bytes: Option<u64>,
) -> Result<u32, MigrationError> {
    run_migrations_with_staging_using(
        active_path,
        mock_free_bytes,
        MIGRATIONS,
        TARGET_SCHEMA_VERSION,
    )
}

/// Verifies that every already-applied migration recorded in `_migrations`
/// still matches the checksum the compiled catalog expects for that version.
/// A mismatch means the migration file was edited after being applied to this
/// database, which `INSERT OR IGNORE` alone would otherwise hide forever.
fn verify_applied_migration_checksums(
    conn: &Connection,
    definitions: &[MigrationDefinition],
) -> Result<(), MigrationError> {
    if !table_exists(conn, "_migrations") {
        return Ok(());
    }

    let mut stmt = conn.prepare("SELECT version, sha256 FROM _migrations")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (version, recorded_hash) = row?;
        if let Some(def) = definitions.iter().find(|m| m.version == version) {
            if !recorded_hash.eq_ignore_ascii_case(def.sha256) {
                return Err(MigrationError::ValidationFailed(format!(
                    "Recorded checksum for migration v{} ({}) no longer matches the compiled catalog (recorded={}, expected={}); the migration file may have been edited after release",
                    version, def.name, recorded_hash, def.sha256
                )));
            }
        }
    }

    Ok(())
}

/// Runs the staging-copy migration engine against an explicit set of
/// migration definitions and target version. `run_migrations_with_staging`
/// always calls this with the real compiled catalog; tests use it directly
/// to inject definitions that deliberately fail, since the engine must be
/// provably safe against a broken migration, not just the ones we shipped.
pub fn run_migrations_with_staging_using(
    active_path: &Path,
    mock_free_bytes: Option<u64>,
    definitions: &[MigrationDefinition],
    target_version: u32,
) -> Result<u32, MigrationError> {
    let parent = active_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    // AC-3: Exclusive OS file lock, held for the entire operation including
    // interrupted-swap recovery below. Recovery deletes and renames files, so
    // it must never run concurrently with another process's migration either.
    let _lock = MigrationLock::acquire(parent)?;

    // 1. Recover any interrupted swap before proceeding
    check_and_recover_interrupted_swap(active_path)?;

    // 2. Inspect active database version if it exists
    let current_version = if active_path.exists() {
        let conn = Connection::open(active_path)?;
        let ver = inspect_schema_version(&conn)?;
        verify_applied_migration_checksums(&conn, definitions)?;
        drop(conn);
        ver
    } else {
        0
    };

    // AC-1: Downgrade prevention
    if current_version > target_version {
        return Err(MigrationError::DowngradePrevented {
            current: current_version,
            target: target_version,
        });
    }

    // AC-6: Archives are pruned on every startup, not only when an upgrade runs.
    prune_old_archives(active_path);

    if current_version == target_version {
        return Ok(current_version);
    }

    // 3. AC-2: Pre-flight storage check
    check_free_space(active_path, mock_free_bytes)?;

    // 5. AC-3: Checkpoint active WAL and duplicate to staging
    let staging_path = parent.join("staging.db");
    if staging_path.exists() {
        let _ = fs::remove_file(&staging_path);
        purge_sidecars(&staging_path);
    }

    if active_path.exists() {
        {
            let conn = Connection::open(active_path)?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        purge_sidecars(active_path);
        fs::copy(active_path, &staging_path).map_err(|e| {
            MigrationError::Io(format!(
                "Failed to copy active db {:?} to staging {:?}: {}",
                active_path, staging_path, e
            ))
        })?;
    }

    // 6. Open staging connection, capture baseline, execute pending migrations
    let run_res: Result<(), MigrationError> = (|| {
        let mut staging_conn = db::open_connection_at(&staging_path)
            .map_err(|e| MigrationError::Sql(e.to_string()))?;

        // AC-5: Capture pre-migration financial baseline from staging
        let baseline = FinancialLedgerBaseline::capture(&staging_conn)?;

        // AC-4: Apply pending migrations in strict numeric order
        let pending: Vec<&MigrationDefinition> = definitions
            .iter()
            .filter(|m| m.version > current_version)
            .collect();

        for def in pending {
            let start = Instant::now();
            let tx = staging_conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|e| MigrationError::Sql(e.to_string()))?;

            match def.step {
                MigrationStep::Sql(sql) => {
                    tx.execute_batch(sql)
                        .map_err(|e| MigrationError::ExecutionFailed {
                            version: def.version,
                            name: def.name.to_string(),
                            error: e.to_string(),
                        })?;
                }
                MigrationStep::Data(func) => {
                    func(&tx)?;
                }
            }

            // AC-10: Record in _migrations if table exists
            if table_exists(&tx, "_migrations") {
                let applied_at = jiff::Timestamp::now().to_string();
                let elapsed_ms = start.elapsed().as_millis() as u64;
                tx.execute(
                    "INSERT OR IGNORE INTO _migrations (version, name, sha256, applied_at, execution_time_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![def.version, def.name, def.sha256, applied_at, elapsed_ms],
                )?;

                // Sync legacy table if present
                if table_exists(&tx, "_schema_version") {
                    tx.execute(
                        "INSERT OR IGNORE INTO _schema_version (version, applied_at) VALUES (?1, ?2)",
                        rusqlite::params![def.version, applied_at],
                    )?;
                }
            }

            tx.commit()
                .map_err(|e| MigrationError::Sql(e.to_string()))?;
        }

        // Backfill earlier migrations in _migrations if migration 9 just created it
        if table_exists(&staging_conn, "_migrations") {
            for def in definitions.iter().filter(|m| m.version <= target_version) {
                let count: i64 = staging_conn.query_row(
                    "SELECT count(*) FROM _migrations WHERE version = ?1",
                    [def.version],
                    |r| r.get(0),
                )?;
                if count == 0 {
                    let applied_at = jiff::Timestamp::now().to_string();
                    staging_conn.execute(
                        "INSERT OR IGNORE INTO _migrations (version, name, sha256, applied_at, execution_time_ms)
                         VALUES (?1, ?2, ?3, ?4, 0)",
                        rusqlite::params![def.version, def.name, def.sha256, applied_at],
                    )?;
                }
            }
        }

        // 7. AC-5: Run validation gates
        run_validation_gates(&staging_conn, &baseline)?;

        // Checkpoint staging WAL and close
        staging_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    })();

    if let Err(err) = run_res {
        // Abandon staging copy and leave live database unmodified
        let _ = fs::remove_file(&staging_path);
        purge_sidecars(&staging_path);
        return Err(err);
    }

    purge_sidecars(&staging_path);

    // 8. AC-6: Atomic swap sequence with retry loop
    if active_path.exists() {
        let now_ts = jiff::Timestamp::now().strftime("%Y%m%d%H%M%S").to_string();
        let archive_name = format!("pre-migrate-v{}-{}.db", current_version, now_ts);
        let archive_path = parent.join(archive_name);

        rename_with_retry(active_path, &archive_path)?;
        purge_sidecars(active_path);
    }

    rename_with_retry(&staging_path, active_path)?;
    purge_sidecars(&staging_path);

    Ok(target_version)
}

// -----------------------------------------------------------------------------
// Startup Application Lifecycle Integration
// -----------------------------------------------------------------------------

pub fn run_startup_lifecycle() -> Result<(), MigrationError> {
    let db_path = match db::resolve_dev_db_path() {
        Ok(p) => p,
        Err(e) => {
            let msg = format!("Failed to resolve database path: {}", e);
            set_recovery_state(None, msg.clone(), PathBuf::from("dev.db"));
            return Err(MigrationError::Io(msg));
        }
    };

    match run_migrations_with_staging(&db_path, None) {
        Ok(_) => {
            clear_recovery_state();
            // Verify integrity of the active database through the same
            // once-per-process gate documented for the whole application:
            // this is what primes the OnceLock so no later command re-scans
            // the file.
            if let Err(e) = db::run_startup_checks() {
                let msg = format!("Integrity check failed on active database: {}", e);
                set_recovery_state(None, msg.clone(), db_path);
                return Err(MigrationError::ValidationFailed(msg));
            }
            Ok(())
        }
        Err(e) => {
            let failed_version = match &e {
                MigrationError::ExecutionFailed { version, .. } => Some(*version),
                MigrationError::DowngradePrevented { current, .. } => Some(*current),
                _ => None,
            };
            set_recovery_state(failed_version, e.to_string(), db_path);
            Err(e)
        }
    }
}

// -----------------------------------------------------------------------------
// Sanitized Diagnostics Export
// -----------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SanitizedDiagnostics {
    pub app_version: &'static str,
    pub target_schema_version: u32,
    pub failed_version: Option<u32>,
    pub error_message: String,
    pub active_db_path: String,
    pub database_size_bytes: u64,
    pub free_disk_bytes: u64,
    pub archive_count: usize,
    pub timestamp: String,
}

pub fn export_sanitized_diagnostics(
    active_path: &Path,
    error_msg: &str,
    failed_version: Option<u32>,
) -> String {
    let size = if active_path.exists() {
        fs::metadata(active_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let archives = list_archives(active_path);
    let parent = active_path.parent().unwrap_or_else(|| Path::new("."));

    let diag = SanitizedDiagnostics {
        app_version: env!("CARGO_PKG_VERSION"),
        target_schema_version: TARGET_SCHEMA_VERSION,
        failed_version,
        error_message: sanitize_error_string(error_msg),
        active_db_path: active_path.display().to_string(),
        database_size_bytes: size,
        free_disk_bytes: get_free_disk_bytes(parent),
        archive_count: archives.len(),
        timestamp: jiff::Timestamp::now().to_string(),
    };

    serde_json::to_string_pretty(&diag).unwrap_or_else(|_| "{}".to_string())
}

/// Builds the sanitized diagnostics JSON for the current global recovery
/// state (if any), falling back to a healthy/no-failure snapshot of
/// `default_db_path` when the process is not in recovery.
pub fn export_sanitized_diagnostics_for_state(default_db_path: &Path) -> String {
    let state = RECOVERY_STATE.read().ok().and_then(|g| g.clone());
    match state {
        Some(s) if s.is_in_recovery => export_sanitized_diagnostics(
            &s.active_db_path,
            s.error_message.as_deref().unwrap_or(""),
            s.failed_version,
        ),
        _ => export_sanitized_diagnostics(default_db_path, "", None),
    }
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn sanitize_error_string(input: &str) -> String {
    let mut s = input.to_string();
    let keywords = [
        "password", "secret", "token", "hash", "argon2", "key", "guardian", "phone", "student",
    ];

    for kw in keywords {
        let search_eq = format!("{}=", kw);
        let mut search_from = 0;
        while search_from < s.len() {
            if let Some(offset) = find_ascii_case_insensitive(&s[search_from..], &search_eq) {
                let pos = search_from + offset;
                let val_start = pos + search_eq.len();
                let val_end = s[val_start..]
                    .find(['&', ';', ' ', ',', '\n', '}'])
                    .map(|offset| val_start + offset)
                    .unwrap_or(s.len());
                let replacement = format!("{}=[REDACTED]", kw);
                s.replace_range(pos..val_end, &replacement);
                search_from = pos + replacement.len();
            } else {
                break;
            }
        }

        let search_col = format!("{}:", kw);
        let mut search_from = 0;
        while search_from < s.len() {
            if let Some(offset) = find_ascii_case_insensitive(&s[search_from..], &search_col) {
                let pos = search_from + offset;
                let val_start = pos + search_col.len();
                let trimmed_offset = s[val_start..]
                    .find(|c: char| !c.is_whitespace())
                    .unwrap_or(0);
                let act_start = val_start + trimmed_offset;
                let val_end = s[act_start..]
                    .find(['&', ';', ',', '\n', '}'])
                    .map(|offset| act_start + offset)
                    .unwrap_or(s.len());
                let replacement = format!("{}: [REDACTED]", kw);
                s.replace_range(pos..val_end, &replacement);
                search_from = pos + replacement.len();
            } else {
                break;
            }
        }
    }

    s
}

pub fn open_directory_in_explorer(path: &Path) -> Result<(), MigrationError> {
    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or_else(|| Path::new("."))
    };

    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(dir)
            .spawn()
            .map_err(|e| MigrationError::Io(e.to_string()))?;
    }

    #[cfg(not(windows))]
    {
        let _ = dir;
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Immutability Check for Released Migrations
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
struct ReleasedMigrationEntry {
    version: u32,
    filename: String,
    sha256: String,
}

#[derive(Deserialize)]
struct ReleasedManifest {
    released: Vec<ReleasedMigrationEntry>,
}

/// Verifies that all released migrations recorded in `migrations/released.json`
/// exist and match their SHA256 checksums exactly. Released migrations are immutable.
pub fn verify_released_migrations(workspace_root: &Path) -> Result<(), String> {
    let manifest_path = workspace_root.join("migrations").join("released.json");
    if !manifest_path.exists() {
        return Err(format!(
            "Missing released migrations manifest at {:?}",
            manifest_path
        ));
    }

    let manifest_bytes =
        fs::read(&manifest_path).map_err(|e| format!("Failed to read released.json: {}", e))?;
    let manifest: ReleasedManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("Failed to parse released.json: {}", e))?;

    for entry in manifest.released {
        let migration_file = workspace_root.join("migrations").join(&entry.filename);
        if !migration_file.exists() {
            return Err(format!(
                "Released migration file {:?} is missing from disk",
                entry.filename
            ));
        }

        let raw_content = fs::read_to_string(&migration_file)
            .map_err(|e| format!("Failed to read {:?}: {}", migration_file, e))?;

        // Normalize line endings to \n to ensure cross-platform hash stability
        let normalized = raw_content.replace("\r\n", "\n");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());

        if !computed_hash.eq_ignore_ascii_case(&entry.sha256) {
            return Err(format!(
                "Released migration '{}' (version {}) has been modified! Expected SHA256: {}, found: {}. Released migrations are immutable.",
                entry.filename, entry.version, entry.sha256, computed_hash
            ));
        }
    }

    Ok(())
}

/// Verifies that migration 0009 adds the `_migrations` tracking table with immutability triggers per AC-10.
pub fn verify_migration_0009_schema(workspace_root: &Path) -> Result<(), String> {
    let migration_file = workspace_root
        .join("migrations")
        .join("0009_migration_engine.sql");
    if !migration_file.exists() {
        return Err("Migration 0009_migration_engine.sql is missing from disk".to_string());
    }

    let content = fs::read_to_string(&migration_file)
        .map_err(|e| format!("Failed to read 0009_migration_engine.sql: {}", e))?;

    if !content.contains("CREATE TABLE IF NOT EXISTS _migrations") {
        return Err("Migration 0009 does not create _migrations table".to_string());
    }

    if !content.contains("CREATE TRIGGER IF NOT EXISTS trg_migrations_no_update") {
        return Err("Migration 0009 missing trg_migrations_no_update trigger".to_string());
    }

    if !content.contains("CREATE TRIGGER IF NOT EXISTS trg_migrations_no_delete") {
        return Err("Migration 0009 missing trg_migrations_no_delete trigger".to_string());
    }

    if !content.contains("settings.view") {
        return Err("Migration 0009 missing settings.view permission seed".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_released_migrations_are_intact() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root");

        let result = verify_released_migrations(workspace_root);
        assert!(
            result.is_ok(),
            "Released migrations check failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_migration_0009_schema_verified() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root");

        let result = verify_migration_0009_schema(workspace_root);
        assert!(
            result.is_ok(),
            "Migration 0009 check failed: {:?}",
            result.err()
        );
    }
}
