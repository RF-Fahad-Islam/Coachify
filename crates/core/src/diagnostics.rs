use crate::db;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum DiagnosticsError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error("Serialization error: {0}")]
    Serde(String),
}

impl DiagnosticsError {
    pub fn code(&self) -> &'static str {
        match self {
            DiagnosticsError::Sqlite(_) => "diagnostics_database_error",
            DiagnosticsError::Io(_) => "diagnostics_io_error",
            DiagnosticsError::Zip(_) => "diagnostics_zip_error",
            DiagnosticsError::Serde(_) => "diagnostics_serde_error",
        }
    }
}

impl From<zip::result::ZipError> for DiagnosticsError {
    fn from(err: zip::result::ZipError) -> Self {
        DiagnosticsError::Zip(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Data Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiagnosticsInfo {
    pub app_version: String,
    pub schema_version: u32,
    pub database_path: String,
    pub database_size_bytes: u64,
    pub wal_size_bytes: u64,
    pub freelist_pages: u32,
    pub integrity_status: String,
    pub backup_status: BackupStatusSummary,
    pub data_drive_free_bytes: u64,
    pub backup_drive_free_bytes: Option<u64>,
    pub os_version: String,
    pub table_stats: Vec<TableStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupStatusSummary {
    pub last_created_at: Option<String>,
    pub last_verified_at: Option<String>,
    pub destination_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TableStat {
    pub name: String,
    pub row_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IntegrityCheckResult {
    pub passed: bool,
    pub details: Vec<String>,
}

// ---------------------------------------------------------------------------
// Diagnostics Collection
// ---------------------------------------------------------------------------

pub fn collect_diagnostics_info(conn: &Connection) -> Result<DiagnosticsInfo, DiagnosticsError> {
    let db_path = db::resolve_active_db_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Schema version
    let schema_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Database file size
    let database_size_bytes = db::resolve_active_db_path()
        .ok()
        .and_then(|p| std::fs::metadata(&p).ok())
        .map(|m| m.len())
        .unwrap_or(0);

    // WAL file size
    let wal_size_bytes = db::resolve_active_db_path()
        .ok()
        .map(|p| {
            let wal = p.with_extension("db-wal");
            std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0)
        })
        .unwrap_or(0);

    // Freelist pages
    let freelist_pages: u32 = conn
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .unwrap_or(0);

    // Quick integrity check
    let integrity_status = match db::verify_integrity(conn) {
        Ok(()) => "ok".to_string(),
        Err(e) => format!("failed: {e}"),
    };

    // Backup status
    let backup_status = collect_backup_status(conn);

    // Disk space
    let data_drive_free_bytes =
        get_drive_free_bytes(&db::resolve_active_db_path().unwrap_or_else(|_| PathBuf::from(".")));

    // Backup drive free bytes
    let backup_drive_free_bytes: Option<u64> = conn
        .query_row(
            "SELECT path FROM backup_destinations WHERE is_default = 1 LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|path| get_drive_free_bytes(Path::new(&path)));

    // OS version
    let os_version = format!(
        "{} {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::consts::FAMILY,
    );

    // Table stats
    let table_stats = collect_table_stats(conn)?;

    Ok(DiagnosticsInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        database_path: db_path,
        database_size_bytes,
        wal_size_bytes,
        freelist_pages,
        integrity_status,
        backup_status,
        data_drive_free_bytes,
        backup_drive_free_bytes,
        os_version,
        table_stats,
    })
}

fn collect_backup_status(conn: &Connection) -> BackupStatusSummary {
    let last_created_at: Option<String> = conn
        .query_row(
            "SELECT created_at FROM backup_history WHERE status IN ('completed', 'verified')
             ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let last_verified_at: Option<String> = conn
        .query_row(
            "SELECT verified_at FROM backup_history WHERE status = 'verified'
             ORDER BY verified_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let destination_healthy = conn
        .query_row(
            "SELECT COUNT(*) FROM backup_destinations WHERE last_error IS NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        > 0;

    BackupStatusSummary {
        last_created_at,
        last_verified_at,
        destination_healthy,
    }
}

fn collect_table_stats(conn: &Connection) -> Result<Vec<TableStat>, DiagnosticsError> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master
         WHERE type = 'table'
           AND name NOT LIKE 'sqlite_%'
           AND SUBSTR(name, 1, 1) != '_'
         ORDER BY name",
    )?;

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let mut stats = Vec::with_capacity(table_names.len());
    for name in &table_names {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        stats.push(TableStat {
            name: name.clone(),
            row_count: count,
        });
    }

    Ok(stats)
}

// ---------------------------------------------------------------------------
// Full Integrity Check
// ---------------------------------------------------------------------------

pub fn run_full_integrity_check(
    conn: &Connection,
) -> Result<IntegrityCheckResult, DiagnosticsError> {
    let mut stmt = conn.prepare("PRAGMA integrity_check")?;
    let mut rows = stmt.query([])?;

    let mut details = Vec::new();
    let mut passed = true;

    while let Some(row) = rows.next()? {
        let result: String = row.get(0)?;
        if result != "ok" {
            passed = false;
        }
        details.push(result);
    }

    Ok(IntegrityCheckResult { passed, details })
}

// ---------------------------------------------------------------------------
// Diagnostics Bundle Export
// ---------------------------------------------------------------------------

pub fn export_diagnostics_bundle(
    conn: &Connection,
    data_root: &Path,
    destination_path: &str,
) -> Result<String, DiagnosticsError> {
    let dest = Path::new(destination_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let now = db::now_utc_iso();
    let safe_ts = now.replace([':', '.'], "-");
    let bundle_name = format!("coachify-diagnostics-{safe_ts}.zip");
    let bundle_path = dest.join(&bundle_name);

    let file = std::fs::File::create(&bundle_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. info.json
    let info = collect_diagnostics_info(conn)?;
    let info_json = serde_json::to_string_pretty(&serde_json::json!({
        "app_version": info.app_version,
        "schema_version": info.schema_version,
        "os_version": info.os_version,
        "generated_at": now,
        "database_path": info.database_path,
    }))
    .map_err(|e| DiagnosticsError::Serde(e.to_string()))?;
    zip.start_file("info.json", options)?;
    zip.write_all(info_json.as_bytes())?;

    // 2. database_stats.json
    let stats_json = serde_json::to_string_pretty(&serde_json::json!({
        "database_size_bytes": info.database_size_bytes,
        "wal_size_bytes": info.wal_size_bytes,
        "freelist_pages": info.freelist_pages,
        "data_drive_free_bytes": info.data_drive_free_bytes,
        "backup_drive_free_bytes": info.backup_drive_free_bytes,
        "table_stats": info.table_stats,
    }))
    .map_err(|e| DiagnosticsError::Serde(e.to_string()))?;
    zip.start_file("database_stats.json", options)?;
    zip.write_all(stats_json.as_bytes())?;

    // 3. backup_history.json (last 10 backups)
    let backup_history = collect_recent_backup_history(conn, 10)?;
    let backup_json = serde_json::to_string_pretty(&backup_history)
        .map_err(|e| DiagnosticsError::Serde(e.to_string()))?;
    zip.start_file("backup_history.json", options)?;
    zip.write_all(backup_json.as_bytes())?;

    // 4. settings.json (redacted)
    let settings = collect_redacted_settings(conn)?;
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| DiagnosticsError::Serde(e.to_string()))?;
    zip.start_file("settings.json", options)?;
    zip.write_all(settings_json.as_bytes())?;

    // 5. integrity_check.txt
    let integrity = run_full_integrity_check(conn)?;
    let integrity_text = integrity.details.join("\n");
    zip.start_file("integrity_check.txt", options)?;
    zip.write_all(integrity_text.as_bytes())?;

    // 6. recent_logs.txt (redacted, from data_root/logs/)
    let logs_dir = data_root.join("logs");
    let log_content = read_recent_log_files(&logs_dir, 3)?;
    let redacted = redact_pii(&log_content);
    zip.start_file("recent_logs.txt", options)?;
    zip.write_all(redacted.as_bytes())?;

    zip.finish()?;

    Ok(bundle_path.display().to_string())
}

// ---------------------------------------------------------------------------
// PII & Secret Redaction
// ---------------------------------------------------------------------------

/// Redacts API keys (`sk-...`), Bearer tokens, Owner Recovery Keys (`CKREC-...`),
/// DPAPI blobs (`DPAPI_V1:...`, `MOCK_DPAPI_WRAP:...`), and credential key-value assignments.
pub fn redact_secrets(input: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    static RE_API_KEY: OnceLock<Regex> = OnceLock::new();
    static RE_BEARER: OnceLock<Regex> = OnceLock::new();
    static RE_RECOVERY: OnceLock<Regex> = OnceLock::new();
    static RE_DPAPI: OnceLock<Regex> = OnceLock::new();
    static RE_KV_SECRET: OnceLock<Regex> = OnceLock::new();

    let re_api = RE_API_KEY
        .get_or_init(|| Regex::new(r"\b(?:sk|pk|rk)-[A-Za-z0-9_\-]{8,}\b").expect("valid regex"));
    let re_bearer = RE_BEARER
        .get_or_init(|| Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9._\-+/=]{8,}").expect("valid regex"));
    let re_rec = RE_RECOVERY
        .get_or_init(|| Regex::new(r"\bCKREC(?:-[A-Z0-9]{4}){1,6}\b").expect("valid regex"));
    let re_dpapi = RE_DPAPI.get_or_init(|| {
        Regex::new(r"(?:DPAPI_V1:|MOCK_DPAPI_WRAP:)[^\s\x22\x27,}]+").expect("valid regex")
    });
    let re_kv = RE_KV_SECRET.get_or_init(|| {
        Regex::new(r"(?i)\b(api_key|apikey|secret|secret_ref|password|passwd|token|auth_token|smtp_pass|private_key)\s*([:=])\s*([\x22\x27]?)([^\s\x22\x27,}]+)([\x22\x27]?)").expect("valid regex")
    });

    let s = re_api.replace_all(input, "[REDACTED_API_KEY]");
    let s = re_bearer.replace_all(&s, "Bearer [REDACTED_SECRET]");
    let s = re_rec.replace_all(&s, "[REDACTED_RECOVERY_KEY]");
    let s = re_dpapi.replace_all(&s, "[REDACTED_DPAPI_BLOB]");
    let s = re_kv.replace_all(&s, "$1$2$3[REDACTED_SECRET]$5");
    s.into_owned()
}

/// Redacts phone numbers and sensitive secrets from text.
/// Bangladesh phone patterns: 01X-XXXX-XXXX or 8801XXXXXXXXX
pub fn redact_pii(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Check for international Bangladesh phone: 8801XXXXXXXXX (13 digits)
        if i + 13 <= len
            && chars[i] == '8'
            && chars[i + 1] == '8'
            && chars[i + 2] == '0'
            && chars[i + 3] == '1'
            && is_bd_operator(chars[i + 4])
            && all_digits(&chars[i + 5..i + 13])
        {
            result.push_str("8801X***XXXX");
            i += 13;
            continue;
        }

        // Check for local Bangladesh phone: 01XXXXXXXXX (11 digits)
        if i + 11 <= len
            && chars[i] == '0'
            && chars[i + 1] == '1'
            && is_bd_operator(chars[i + 2])
            && all_digits(&chars[i + 3..i + 11])
        {
            result.push_str("01X***XXXX");
            i += 11;
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    redact_secrets(&result)
}

fn is_bd_operator(c: char) -> bool {
    matches!(c, '3' | '4' | '5' | '6' | '7' | '8' | '9')
}

fn all_digits(chars: &[char]) -> bool {
    chars.iter().all(|c| c.is_ascii_digit())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_recent_backup_history(
    conn: &Connection,
    limit: u32,
) -> Result<Vec<serde_json::Value>, DiagnosticsError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, status, file_size_bytes, started_at, completed_at,
                verified_at, error_message, created_at
         FROM backup_history
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let items: Vec<serde_json::Value> = stmt
        .query_map([limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "kind": row.get::<_, String>(1)?,
                "status": row.get::<_, String>(2)?,
                "file_size_bytes": row.get::<_, i64>(3)?,
                "started_at": row.get::<_, Option<String>>(4)?,
                "completed_at": row.get::<_, Option<String>>(5)?,
                "verified_at": row.get::<_, Option<String>>(6)?,
                "error_message": row.get::<_, Option<String>>(7)?.map(|e| redact_pii(&e)),
                "created_at": row.get::<_, String>(8)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

fn redact_json_value(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            let mut clean = serde_json::Map::new();
            for (k, v) in map {
                if crate::crypto::is_sensitive_key_name(&k) {
                    clean.insert(k, serde_json::Value::String("[REDACTED]".to_string()));
                } else {
                    clean.insert(k, redact_json_value(v));
                }
            }
            serde_json::Value::Object(clean)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(redact_json_value).collect())
        }
        serde_json::Value::String(s) => serde_json::Value::String(redact_pii(&s)),
        other => other,
    }
}

fn collect_redacted_settings(conn: &Connection) -> Result<serde_json::Value, DiagnosticsError> {
    let mut stmt = conn.prepare("SELECT key, value_json FROM settings ORDER BY key")?;

    let mut settings = serde_json::Map::new();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (key, value) in rows {
        let redacted_value = if crate::crypto::is_sensitive_key_name(&key) || key == "sms_provider"
        {
            serde_json::Value::String("[REDACTED]".to_string())
        } else {
            let parsed = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
            redact_json_value(parsed)
        };
        settings.insert(key, redacted_value);
    }

    // Include redacted provider_configs summary so diagnostics confirms provider state without leaking secrets
    if let Ok(mut p_stmt) = conn.prepare(
        "SELECT channel, adapter, is_active, config_json, secret_ref FROM provider_configs ORDER BY channel, adapter",
    ) {
        if let Ok(p_rows) = p_stmt.query_map([], |r| {
            let channel: String = r.get(0)?;
            let adapter: String = r.get(1)?;
            let is_active: i64 = r.get(2)?;
            let cfg_str: String = r.get(3)?;
            let has_secret: bool = r.get::<_, Option<String>>(4)?.is_some();
            let cfg_val = serde_json::from_str(&cfg_str).unwrap_or(serde_json::Value::String(cfg_str));
            Ok(serde_json::json!({
                "channel": channel,
                "adapter": adapter,
                "is_active": is_active == 1,
                "config": redact_json_value(cfg_val),
                "secret_ref": if has_secret { "[REDACTED]" } else { "[UNSET]" }
            }))
        }) {
            let list: Vec<serde_json::Value> = p_rows.flatten().collect();
            settings.insert("_provider_configs".to_string(), serde_json::Value::Array(list));
        }
    }

    Ok(serde_json::Value::Object(settings))
}

fn read_recent_log_files(logs_dir: &Path, max_days: u32) -> Result<String, DiagnosticsError> {
    if !logs_dir.exists() {
        return Ok("(no log directory found)".to_string());
    }

    let mut content = String::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in std::fs::read_dir(logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "log").unwrap_or(false) {
            files.push(path);
        }
    }

    // Sort by name (which includes timestamp) descending
    files.sort();
    files.reverse();

    // Read up to max_days worth of files, capped at 1MB total
    let max_bytes = 1024 * 1024;
    let mut total_bytes = 0usize;

    for (i, path) in files.iter().enumerate() {
        if i >= max_days as usize || total_bytes >= max_bytes {
            break;
        }
        let mut file = std::fs::File::open(path)?;
        let mut buf = String::new();
        let n = file.read_to_string(&mut buf)?;
        total_bytes += n;
        content.push_str(&format!(
            "=== {} ===\n",
            path.file_name().unwrap_or_default().to_string_lossy()
        ));
        content.push_str(&buf);
        content.push('\n');
    }

    if content.is_empty() {
        content = "(no log files found)".to_string();
    }

    Ok(content)
}

fn get_drive_free_bytes(path: &Path) -> u64 {
    // On Windows, we can use the GetDiskFreeSpaceEx API through std
    // For now, return 0 as a fallback — the Tauri shell layer will provide real values
    #[cfg(windows)]
    {
        let wide_path: Vec<u16> = path
            .to_str()
            .unwrap_or("C:\\")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        unsafe {
            // winapi GetDiskFreeSpaceExW
            extern "system" {
                fn GetDiskFreeSpaceExW(
                    lpDirectoryName: *const u16,
                    lpFreeBytesAvailableToCaller: *mut u64,
                    lpTotalNumberOfBytes: *mut u64,
                    lpTotalNumberOfFreeBytes: *mut u64,
                ) -> i32;
            }

            let result = GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes,
                &mut total_bytes,
                &mut total_free_bytes,
            );

            if result != 0 {
                return free_bytes;
            }
        }
    }

    0
}

// ---------------------------------------------------------------------------
// Log Reading
// ---------------------------------------------------------------------------

pub fn get_recent_logs(
    data_root: &Path,
    level_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<LogEntry>, DiagnosticsError> {
    let logs_dir = data_root.join("logs");
    let raw = read_recent_log_files(&logs_dir, 3)?;
    let redacted = redact_pii(&raw);

    let mut entries = Vec::new();

    for line in redacted.lines() {
        if line.starts_with("===") || line.is_empty() {
            continue;
        }

        // Parse tracing log format: "TIMESTAMP LEVEL target: message"
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() >= 3 {
            let timestamp = parts[0].to_string();
            let level = parts[1].to_string();
            let rest = if parts.len() >= 4 {
                parts[2..].join(" ")
            } else {
                parts[2].to_string()
            };

            let (target, message) = if let Some(idx) = rest.find(": ") {
                (Some(rest[..idx].to_string()), rest[idx + 2..].to_string())
            } else {
                (None, rest)
            };

            if let Some(filter) = level_filter {
                if !level.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            entries.push(LogEntry {
                timestamp,
                level,
                message,
                target,
            });
        }

        if entries.len() >= limit {
            break;
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("diag_test.db");
        let conn = db::open_connection_at(&db_path).expect("open test db");
        db::apply_all_migrations(&conn).expect("apply migrations");
        (dir, conn)
    }

    #[test]
    fn test_pii_redaction_phone_numbers() {
        let input = "Parent contact: 01712345678, emergency: 8801812345678, other: 123456.";
        let redacted = redact_pii(input);
        assert!(redacted.contains("01X***XXXX"));
        assert!(redacted.contains("8801X***XXXX"));
        assert!(redacted.contains("123456"));
        assert!(!redacted.contains("01712345678"));
        assert!(!redacted.contains("8801812345678"));
    }

    #[test]
    fn test_collect_diagnostics_info() {
        let (_dir, conn) = setup_test_db();
        let info = collect_diagnostics_info(&conn).expect("collect info");
        assert_eq!(
            info.schema_version,
            crate::migrations::TARGET_SCHEMA_VERSION
        );
        assert_eq!(info.integrity_status, "ok");
        assert!(!info.table_stats.is_empty());
        assert!(info.table_stats.iter().any(|t| t.name == "students"));
        assert!(info.table_stats.iter().any(|t| t.name == "backup_history"));
    }

    #[test]
    fn test_full_integrity_check() {
        let (_dir, conn) = setup_test_db();
        let result = run_full_integrity_check(&conn).expect("integrity check");
        assert!(result.passed);
        assert_eq!(result.details, vec!["ok"]);
    }

    #[test]
    fn test_export_diagnostics_bundle() {
        let (db_dir, conn) = setup_test_db();
        let dest_dir = tempfile::tempdir().expect("dest dir");

        let zip_path_str =
            export_diagnostics_bundle(&conn, db_dir.path(), &dest_dir.path().display().to_string())
                .expect("export bundle");

        let zip_path = Path::new(&zip_path_str);
        assert!(zip_path.exists());

        // Verify it can be opened as a ZIP archive
        let zip_file = std::fs::File::open(zip_path).expect("open zip");
        let mut archive = zip::ZipArchive::new(zip_file).expect("read zip archive");

        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        assert!(names.contains(&"info.json".to_string()));
        assert!(names.contains(&"database_stats.json".to_string()));
        assert!(names.contains(&"backup_history.json".to_string()));
        assert!(names.contains(&"settings.json".to_string()));
        assert!(names.contains(&"integrity_check.txt".to_string()));
        assert!(names.contains(&"recent_logs.txt".to_string()));
    }
}
