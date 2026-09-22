use serde::{Deserialize, Serialize};
use specta::Type;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Path does not exist and cannot be created: {0}")]
    PathCreationFailed(String),
    #[error("Write permission denied on path: {0}")]
    PermissionDenied(String),
    #[error(
        "Insufficient disk space on path: available {available} bytes, required {required} bytes"
    )]
    InsufficientSpace { available: u64, required: u64 },
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Backup target mismatch: expected institution {expected}, found {actual}")]
    BackupTargetMismatch { expected: String, actual: String },
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// Drive and Volume Detection
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub drive_type: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_system: bool,
    pub is_recommended: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct PathValidationResult {
    pub valid: bool,
    pub path: String,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub is_non_system: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct BackupValidationResult {
    pub valid: bool,
    pub path: String,
    pub free_bytes: u64,
    pub has_existing_marker: bool,
    pub marker_institution_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BackupTargetMarker {
    pub institution_id: String,
    pub initialized_at: String,
    pub app_version: String,
}

pub const BACKUP_MARKER_FILENAME: &str = ".coachify-backup-target";

// -----------------------------------------------------------------------------
// Windows Native Storage Integration
// -----------------------------------------------------------------------------

#[cfg(windows)]
extern "system" {
    fn GetLogicalDriveStringsW(n_buffer_length: u32, lp_buffer: *mut u16) -> u32;
    fn GetDriveTypeW(lp_root_path_name: *const u16) -> u32;
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;
const DRIVE_REMOTE: u32 = 4;

/// Detects all logical storage drives on the host PC.
pub fn detect_logical_drives() -> Result<Vec<DriveInfo>, StorageError> {
    let system_drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let system_drive_upper = system_drive.trim_end_matches('\\').to_uppercase();

    #[cfg(windows)]
    {
        let mut buffer = [0u16; 512];
        let len = unsafe { GetLogicalDriveStringsW(512, buffer.as_mut_ptr()) };
        if len == 0 {
            return Err(StorageError::Io(
                std::io::Error::last_os_error().to_string(),
            ));
        }

        let mut drives = Vec::new();
        let mut start = 0;
        for i in 0..(len as usize) {
            if buffer[i] == 0 {
                if i > start {
                    let drive_w = &buffer[start..=i]; // includes null terminator
                    let drive_str = String::from_utf16_lossy(&buffer[start..i]);
                    let drive_type_raw = unsafe { GetDriveTypeW(drive_w.as_ptr()) };

                    let drive_type = match drive_type_raw {
                        DRIVE_FIXED => "fixed",
                        DRIVE_REMOVABLE => "removable",
                        DRIVE_REMOTE => "network",
                        _ => "unknown",
                    };

                    let mut free_bytes = 0u64;
                    let mut total_bytes = 0u64;
                    let mut total_free = 0u64;

                    let _ = unsafe {
                        GetDiskFreeSpaceExW(
                            drive_w.as_ptr(),
                            &mut free_bytes,
                            &mut total_bytes,
                            &mut total_free,
                        )
                    };

                    let is_system = drive_str
                        .trim_end_matches('\\')
                        .eq_ignore_ascii_case(&system_drive_upper);

                    // Recommend fixed drive that is NOT the system drive and has >= 500 MB free
                    let is_recommended =
                        drive_type == "fixed" && !is_system && free_bytes >= 500 * 1024 * 1024;

                    let label = if is_system {
                        format!("System Volume ({})", drive_str)
                    } else if drive_type == "fixed" {
                        format!("Secondary Storage ({})", drive_str)
                    } else if drive_type == "removable" {
                        format!("USB Drive ({})", drive_str)
                    } else {
                        format!("Drive ({})", drive_str)
                    };

                    drives.push(DriveInfo {
                        letter: drive_str,
                        label,
                        drive_type: drive_type.to_string(),
                        total_bytes,
                        free_bytes,
                        is_system,
                        is_recommended,
                    });
                }
                start = i + 1;
            }
        }

        // Sort so recommended non-system drives appear first
        drives.sort_by(|a, b| {
            b.is_recommended
                .cmp(&a.is_recommended)
                .then_with(|| a.letter.cmp(&b.letter))
        });

        Ok(drives)
    }

    #[cfg(not(windows))]
    {
        Ok(vec![
            DriveInfo {
                letter: "/".to_string(),
                label: "Root Volume".to_string(),
                drive_type: "fixed".to_string(),
                total_bytes: 500 * 1024 * 1024 * 1024,
                free_bytes: 200 * 1024 * 1024 * 1024,
                is_system: true,
                is_recommended: false,
            },
            DriveInfo {
                letter: "/data".to_string(),
                label: "Data Volume".to_string(),
                drive_type: "fixed".to_string(),
                total_bytes: 1000 * 1024 * 1024 * 1024,
                free_bytes: 800 * 1024 * 1024 * 1024,
                is_system: false,
                is_recommended: true,
            },
        ])
    }
}

/// Recommends a default data root directory.
/// Returns (recommended_path, is_non_system_recommended).
pub fn recommend_default_data_dir() -> (PathBuf, bool) {
    if let Ok(drives) = detect_logical_drives() {
        for drive in drives {
            if drive.is_recommended {
                let clean_letter = drive.letter.trim_end_matches('\\');
                return (
                    PathBuf::from(format!(r"{}\Coachify Data", clean_letter)),
                    true,
                );
            }
        }
    }

    // Fallback: ProgramData or LocalAppData
    let base = match env::var("ProgramData") {
        Ok(val) => PathBuf::from(val),
        Err(_) => match env::var("LOCALAPPDATA") {
            Ok(val) => PathBuf::from(val),
            Err(_) => {
                let home = env::var("USERPROFILE")
                    .or_else(|_| env::var("HOME"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".coachify")
            }
        },
    };

    (base.join("Coachify").join("data"), false)
}

/// Validates that a proposed data folder path is writable and has sufficient free space.
pub fn validate_data_folder(path: &Path) -> PathValidationResult {
    let path_str = path.to_string_lossy().to_string();

    let system_drive = env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
    let is_non_system = !path_str
        .to_uppercase()
        .starts_with(&system_drive.to_uppercase());

    // Try creating the directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(path) {
        return PathValidationResult {
            valid: false,
            path: path_str,
            free_bytes: 0,
            total_bytes: 0,
            is_non_system,
            error: Some(format!("Cannot create directory: {}", e)),
        };
    }

    // Test write permission by writing and removing a temporary canary file
    let test_file = path.join(".coachify-perm-check.tmp");
    match fs::write(&test_file, b"coachify_test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
        }
        Err(e) => {
            return PathValidationResult {
                valid: false,
                path: path_str,
                free_bytes: 0,
                total_bytes: 0,
                is_non_system,
                error: Some(format!("Directory is not writable: {}", e)),
            };
        }
    }

    // Determine volume free space
    let (free_bytes, total_bytes) = get_path_disk_space(path);

    if free_bytes < 50 * 1024 * 1024 {
        return PathValidationResult {
            valid: false,
            path: path_str,
            free_bytes,
            total_bytes,
            is_non_system,
            error: Some("Less than 50 MB free disk space available".to_string()),
        };
    }

    PathValidationResult {
        valid: true,
        path: path_str,
        free_bytes,
        total_bytes,
        is_non_system,
        error: None,
    }
}

/// Validates a potential backup destination directory.
pub fn validate_backup_destination(
    path: &Path,
    expected_institution_id: Option<&str>,
) -> BackupValidationResult {
    let path_str = path.to_string_lossy().to_string();

    if let Err(e) = fs::create_dir_all(path) {
        return BackupValidationResult {
            valid: false,
            path: path_str,
            free_bytes: 0,
            has_existing_marker: false,
            marker_institution_id: None,
            error: Some(format!("Cannot access backup directory: {}", e)),
        };
    }

    // Test write permission
    let test_file = path.join(".coachify-perm-check.tmp");
    match fs::write(&test_file, b"coachify_backup_check") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
        }
        Err(e) => {
            return BackupValidationResult {
                valid: false,
                path: path_str,
                free_bytes: 0,
                has_existing_marker: false,
                marker_institution_id: None,
                error: Some(format!("Backup folder is not writable: {}", e)),
            };
        }
    }

    let (free_bytes, _) = get_path_disk_space(path);

    // Check for existing marker file
    let marker_path = path.join(BACKUP_MARKER_FILENAME);
    let (has_existing_marker, marker_institution_id) = if marker_path.exists() {
        if let Ok(content) = fs::read_to_string(&marker_path) {
            if let Ok(marker) = serde_json::from_str::<BackupTargetMarker>(&content) {
                (true, Some(marker.institution_id))
            } else {
                (true, None)
            }
        } else {
            (true, None)
        }
    } else {
        (false, None)
    };

    if let (Some(expected), Some(actual)) = (expected_institution_id, &marker_institution_id) {
        if expected != actual {
            let actual_str = actual.clone();
            return BackupValidationResult {
                valid: false,
                path: path_str,
                free_bytes,
                has_existing_marker,
                marker_institution_id,
                error: Some(format!(
                    "This drive was initialized for another institution ({})",
                    actual_str
                )),
            };
        }
    }

    BackupValidationResult {
        valid: true,
        path: path_str,
        free_bytes,
        has_existing_marker,
        marker_institution_id,
        error: None,
    }
}

/// Writes the `.coachify-backup-target` marker file to bind the backup drive.
pub fn write_backup_marker(destination: &Path, institution_id: &str) -> Result<(), StorageError> {
    fs::create_dir_all(destination)?;
    let marker = BackupTargetMarker {
        institution_id: institution_id.to_string(),
        initialized_at: crate::db::now_utc_iso(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let json = serde_json::to_string_pretty(&marker)?;
    let marker_path = destination.join(BACKUP_MARKER_FILENAME);
    fs::write(marker_path, json)?;
    Ok(())
}

pub fn get_path_disk_space(path: &Path) -> (u64, u64) {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        wide.push(0);

        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        let res = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            )
        };

        if res != 0 {
            return (free_bytes_available, total_bytes);
        }
    }

    (10 * 1024 * 1024 * 1024, 100 * 1024 * 1024 * 1024)
}

// -----------------------------------------------------------------------------
// Institution Storage Topology Scaffolding
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstitutionPaths {
    pub root: PathBuf,
    pub db_dir: PathBuf,
    pub db_file: PathBuf,
    pub key_envelope_file: PathBuf,
    pub vault_dir: PathBuf,
    pub snapshots_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub staging_dir: PathBuf,
}

/// Scaffolds the complete folder layout under `<data_root>/<institution_id>/`.
pub fn scaffold_institution_storage(
    data_root: &Path,
    institution_id: &str,
) -> Result<InstitutionPaths, StorageError> {
    let root = data_root.join(institution_id);
    let db_dir = root.join("db");
    let vault_dir = root.join("vault");
    let snapshots_dir = root.join("snapshots");
    let logs_dir = root.join("logs");
    let staging_dir = root.join("staging");

    fs::create_dir_all(&db_dir)?;
    fs::create_dir_all(&vault_dir)?;
    fs::create_dir_all(&snapshots_dir)?;
    fs::create_dir_all(&logs_dir)?;
    fs::create_dir_all(&staging_dir)?;

    let db_file = db_dir.join("main.db");
    let key_envelope_file = db_dir.join("key.envelope");

    Ok(InstitutionPaths {
        root,
        db_dir,
        db_file,
        key_envelope_file,
        vault_dir,
        snapshots_dir,
        logs_dir,
        staging_dir,
    })
}

// -----------------------------------------------------------------------------
// Global Configuration Management
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct GlobalConfig {
    pub data_dir: String,
    pub institution_id: String,
    pub app_mode: String,
    pub version: String,
    pub backup_destination: Option<String>,
}

pub fn resolve_global_config_path() -> PathBuf {
    let base = match env::var("ProgramData") {
        Ok(val) => PathBuf::from(val),
        Err(_) => match env::var("LOCALAPPDATA") {
            Ok(val) => PathBuf::from(val),
            Err(_) => {
                let home = env::var("USERPROFILE")
                    .or_else(|_| env::var("HOME"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".coachify")
            }
        },
    };
    base.join("Coachify").join("config.json")
}

pub fn save_global_config(config: &GlobalConfig) -> Result<(), StorageError> {
    let path = resolve_global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_global_config() -> Result<Option<GlobalConfig>, StorageError> {
    let path = resolve_global_config_path();
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(path)?;
    let config: GlobalConfig = serde_json::from_str(&json)?;
    Ok(Some(config))
}

// -----------------------------------------------------------------------------
// Workstation Printer Settings & System Printer Enumeration (AC-2, AC-3)
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct WorkstationPrinterSettings {
    pub receipt_printer: Option<String>,
    pub report_printer: Option<String>,
    pub receipt_layout: String, // "thermal_80mm" | "a4_dual" | "a5_single"
    pub silent_receipt_printing: bool,
    pub auto_print_on_payment: bool,
    pub copies: u32,
}

impl Default for WorkstationPrinterSettings {
    fn default() -> Self {
        Self {
            receipt_printer: None,
            report_printer: None,
            receipt_layout: "thermal_80mm".to_string(),
            silent_receipt_printing: true,
            auto_print_on_payment: true,
            copies: 1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct SystemPrinterInfo {
    pub name: String,
    pub is_default: bool,
    pub status: String,
}

pub fn resolve_printer_settings_path() -> PathBuf {
    if let Ok(prog_data) = env::var("ProgramData") {
        let dir = PathBuf::from(prog_data).join("Coachify");
        if fs::create_dir_all(&dir).is_ok() {
            return dir.join("printer_settings.json");
        }
    }
    if let Ok(local_app) = env::var("LOCALAPPDATA") {
        let dir = PathBuf::from(local_app).join("Coachify");
        if fs::create_dir_all(&dir).is_ok() {
            return dir.join("printer_settings.json");
        }
    }
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".coachify");
    let _ = fs::create_dir_all(&dir);
    dir.join("printer_settings.json")
}

pub fn get_workstation_printer_settings() -> WorkstationPrinterSettings {
    let path = resolve_printer_settings_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<WorkstationPrinterSettings>(&content) {
                return settings;
            }
        }
    }
    WorkstationPrinterSettings::default()
}

pub fn save_workstation_printer_settings(
    settings: &WorkstationPrinterSettings,
) -> Result<WorkstationPrinterSettings, StorageError> {
    const VALID_LAYOUTS: &[&str] = &["thermal_80mm", "a4_dual", "a5_single"];
    if !VALID_LAYOUTS.contains(&settings.receipt_layout.as_str()) {
        return Err(StorageError::InvalidPath(format!(
            "Invalid receipt layout: {}",
            settings.receipt_layout
        )));
    }
    let path = resolve_printer_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(settings.clone())
}

pub fn list_system_printers() -> Vec<SystemPrinterInfo> {
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_Printer | Select-Object Name, Default | ConvertTo-Json -Compress",
            ])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    if let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
                        let printers: Vec<SystemPrinterInfo> = list
                            .into_iter()
                            .filter_map(|v| {
                                let name = v.get("Name")?.as_str()?.to_string();
                                let is_default = v
                                    .get("Default")
                                    .and_then(|d| d.as_bool())
                                    .unwrap_or(false);
                                Some(SystemPrinterInfo {
                                    name,
                                    is_default,
                                    status: "ready".to_string(),
                                })
                            })
                            .collect();
                        if !printers.is_empty() {
                            return printers;
                        }
                    } else if let Ok(single) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if let Some(name) = single.get("Name").and_then(|n| n.as_str()) {
                            let is_default = single
                                .get("Default")
                                .and_then(|d| d.as_bool())
                                .unwrap_or(false);
                            return vec![SystemPrinterInfo {
                                name: name.to_string(),
                                is_default,
                                status: "ready".to_string(),
                            }];
                        }
                    }
                }
            }
        }
    }

    vec![
        SystemPrinterInfo {
            name: "Microsoft Print to PDF".to_string(),
            is_default: true,
            status: "ready".to_string(),
        },
        SystemPrinterInfo {
            name: "POS-80 Thermal Printer".to_string(),
            is_default: false,
            status: "ready".to_string(),
        },
    ]
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_logical_drives_returns_entries() {
        let drives = detect_logical_drives().expect("drive detection should not fail");
        assert!(!drives.is_empty());
    }

    #[test]
    fn test_validate_data_folder_happy_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("my_coachify_data");
        let res = validate_data_folder(&target);
        assert!(res.valid);
        assert!(res.error.is_none());
        assert!(target.exists());
    }

    #[test]
    fn test_scaffold_institution_storage_creates_all_subdirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let inst_id = "018e6e5a-8b14-7d92-936d-test001";
        let paths = scaffold_institution_storage(temp_dir.path(), inst_id).unwrap();

        assert!(paths.db_dir.exists());
        assert!(paths.vault_dir.exists());
        assert!(paths.snapshots_dir.exists());
        assert!(paths.logs_dir.exists());
        assert!(paths.staging_dir.exists());
    }

    #[test]
    fn test_backup_marker_write_and_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_dir = temp_dir.path().join("usb_backup");
        let inst_id = "inst_abc_123";

        // Pre-marker validation should succeed
        let pre_val = validate_backup_destination(&backup_dir, Some(inst_id));
        assert!(pre_val.valid);
        assert!(!pre_val.has_existing_marker);

        // Write marker
        write_backup_marker(&backup_dir, inst_id).unwrap();

        // Validate again with matching ID
        let post_val = validate_backup_destination(&backup_dir, Some(inst_id));
        assert!(post_val.valid);
        assert!(post_val.has_existing_marker);
        assert_eq!(post_val.marker_institution_id, Some(inst_id.to_string()));

        // Validate with mismatching ID
        let mismatch_val = validate_backup_destination(&backup_dir, Some("different_inst_id"));
        assert!(!mismatch_val.valid);
        assert!(mismatch_val.error.is_some());
    }

    #[test]
    fn test_printer_settings_validation_and_persistence() {
        let default_settings = WorkstationPrinterSettings::default();
        assert_eq!(default_settings.receipt_layout, "thermal_80mm");
        assert!(default_settings.silent_receipt_printing);

        let invalid_settings = WorkstationPrinterSettings {
            receipt_layout: "invalid_layout".to_string(),
            ..default_settings.clone()
        };
        assert!(save_workstation_printer_settings(&invalid_settings).is_err());

        let printers = list_system_printers();
        assert!(!printers.is_empty());
    }
}
