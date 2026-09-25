use crate::accounts::{self, UserSummary};
use crate::audit;
use crate::crypto::{
    self, extract_last_four_checksum, normalize_and_verify_recovery_key, KeyEnvelope,
};
use crate::db::{self, DbError};
use crate::storage::{
    self, detect_logical_drives, recommend_default_data_dir, scaffold_institution_storage,
    validate_data_folder, write_backup_marker, DriveInfo, GlobalConfig,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SetupError {
    #[error("Institution has already been initialized")]
    AlreadyInitialized,
    #[error("Institution setup is required")]
    SetupRequired,
    #[error("Invalid field '{field}': {message}")]
    Validation { field: String, message: String },
    #[error("Recovery key confirmation mismatch: expected '{expected}', got '{actual}'")]
    RecoveryConfirmationMismatch { expected: String, actual: String },
    #[error("Database error during setup: {0}")]
    Database(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Recovery error: {0}")]
    Recovery(String),
}

impl From<DbError> for SetupError {
    fn from(err: DbError) -> Self {
        SetupError::Database(err.to_string())
    }
}

impl From<storage::StorageError> for SetupError {
    fn from(err: storage::StorageError) -> Self {
        SetupError::Storage(err.to_string())
    }
}

impl From<crypto::CryptoError> for SetupError {
    fn from(err: crypto::CryptoError) -> Self {
        SetupError::Crypto(err.to_string())
    }
}

impl From<rusqlite::Error> for SetupError {
    fn from(err: rusqlite::Error) -> Self {
        SetupError::Database(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// Request / Response Payloads
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
    pub institution_id: Option<String>,
    pub institution_name: Option<String>,
    pub app_version: String,
    pub default_data_dir: String,
    pub is_non_system_recommended: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct DetectDrivesResponse {
    pub drives: Vec<DriveInfo>,
    pub recommended_path: String,
    pub is_non_system_recommended: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct ValidatePathPayload {
    pub path: String,
    pub kind: String, // "data" or "backup"
    pub expected_institution_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct GenerateRecoveryPreviewResponse {
    pub recovery_key: String,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct InitializeInstitutionPayload {
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub academic_session_name: String,
    pub student_id_prefix: Option<String>,
    pub receipt_prefix: Option<String>,
    pub data_dir: String,
    pub backup_destination: Option<String>,
    pub owner_display_name: String,
    pub owner_username: String,
    pub owner_password: String,
    pub recovery_key: String,
    pub recovery_key_confirmation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct InitializeInstitutionResponse {
    pub session_token: String,
    pub user: UserSummary,
    pub expires_at: String,
    pub recovery_key: String,
    pub institution_id: String,
    pub data_dir: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct InspectDatabasePayload {
    pub db_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct InspectDatabaseResponse {
    pub database_exists: bool,
    pub has_key_envelope: bool,
    pub can_unwrap_dpapi: bool,
    pub needs_recovery: bool,
    pub institution_id: Option<String>,
    pub institution_name: Option<String>,
    pub schema_version: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct UnlockWithKeyPayload {
    pub db_dir: Option<String>,
    pub recovery_key: String,
    pub reset_owner_password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct UnlockWithKeyResponse {
    pub success: bool,
    pub institution_id: String,
    pub institution_name: String,
    pub owner_password_reset: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct ReprintSheetPayload {
    pub owner_password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct ReprintSheetResponse {
    pub institution_name: String,
    pub owner_username: String,
    pub created_at: String,
    pub recovery_key_checksum: String,
    pub instructions: String,
}

// -----------------------------------------------------------------------------
// Core Setup Functions
// -----------------------------------------------------------------------------

/// Determines the current initialization status of the application.
pub fn get_setup_status() -> Result<SetupStatusResponse, SetupError> {
    let global_config = storage::load_global_config()?;
    let (default_dir, is_rec) = recommend_default_data_dir();

    if let Some(config) = global_config {
        let db_path = PathBuf::from(&config.data_dir)
            .join(&config.institution_id)
            .join("db")
            .join("main.db");

        if db_path.exists() {
            // Attempt to read institution name if database is accessible
            let inst_name = match db::open_connection_at(&db_path) {
                Ok(conn) => conn
                    .query_row("SELECT name FROM institution LIMIT 1", [], |r| {
                        r.get::<_, String>(0)
                    })
                    .ok(),
                Err(_) => None,
            };

            return Ok(SetupStatusResponse {
                needs_setup: false,
                institution_id: Some(config.institution_id),
                institution_name: inst_name,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                default_data_dir: default_dir.to_string_lossy().to_string(),
                is_non_system_recommended: is_rec,
            });
        }
    }

    Ok(SetupStatusResponse {
        needs_setup: true,
        institution_id: None,
        institution_name: None,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        default_data_dir: default_dir.to_string_lossy().to_string(),
        is_non_system_recommended: is_rec,
    })
}

/// Discovers mounted logical drives and marks non-system recommendations.
pub fn detect_drives() -> Result<DetectDrivesResponse, SetupError> {
    let drives = detect_logical_drives()?;
    let (rec_path, is_rec) = recommend_default_data_dir();
    Ok(DetectDrivesResponse {
        drives,
        recommended_path: rec_path.to_string_lossy().to_string(),
        is_non_system_recommended: is_rec,
    })
}

/// Generates a preview recovery key for rendering on the Setup Wizard sheet.
pub fn generate_recovery_preview() -> Result<GenerateRecoveryPreviewResponse, SetupError> {
    let (key, _) = crypto::generate_recovery_key();
    let checksum = extract_last_four_checksum(&key)?;
    Ok(GenerateRecoveryPreviewResponse {
        recovery_key: key,
        checksum,
    })
}

/// Executes the full atomic institutional bootstrap transaction.
pub fn initialize_institution(
    payload: &InitializeInstitutionPayload,
) -> Result<InitializeInstitutionResponse, SetupError> {
    // 1. Validation of required fields
    if payload.institution_name.trim().is_empty() {
        return Err(SetupError::Validation {
            field: "institution_name".to_string(),
            message: "Institution name is required".to_string(),
        });
    }

    if payload.academic_session_name.trim().is_empty() {
        return Err(SetupError::Validation {
            field: "academic_session_name".to_string(),
            message: "Academic session name (e.g. 2026) is required".to_string(),
        });
    }

    if payload.owner_display_name.trim().is_empty() {
        return Err(SetupError::Validation {
            field: "owner_display_name".to_string(),
            message: "Owner display name is required".to_string(),
        });
    }

    if payload.owner_username.trim().is_empty() {
        return Err(SetupError::Validation {
            field: "owner_username".to_string(),
            message: "Owner username is required".to_string(),
        });
    }

    if payload.owner_password.len() < 8 {
        return Err(SetupError::Validation {
            field: "owner_password".to_string(),
            message: "Owner password must be at least 8 characters long".to_string(),
        });
    }

    // 2. Validate Recovery Key format & Confirmation Gate
    let canonical_recovery_key = normalize_and_verify_recovery_key(&payload.recovery_key)?;
    let expected_checksum = extract_last_four_checksum(&canonical_recovery_key)?;

    let clean_confirm = payload.recovery_key_confirmation.trim().to_uppercase();
    if clean_confirm != expected_checksum {
        return Err(SetupError::RecoveryConfirmationMismatch {
            expected: expected_checksum,
            actual: clean_confirm,
        });
    }

    // 3. Check and validate data directory
    let data_root = PathBuf::from(&payload.data_dir);
    let val_res = validate_data_folder(&data_root);
    if !val_res.valid {
        return Err(SetupError::Storage(
            val_res
                .error
                .unwrap_or_else(|| "Invalid data folder".to_string()),
        ));
    }

    // 4. Generate Institution ID (UUIDv7)
    let institution_id = Uuid::now_v7().to_string();

    // 5. Scaffold storage layout
    let paths = scaffold_institution_storage(&data_root, &institution_id)?;

    // 6. Generate 256-bit CSPRNG Master Key and Key Envelope
    let master_key = crypto::generate_master_key();
    let envelope = KeyEnvelope::create_with_recovery_key(
        &institution_id,
        &master_key,
        &canonical_recovery_key,
    )?;
    envelope.save_to_file(&paths.key_envelope_file)?;

    // 7. Initialize Database and execute migrations
    let mut conn = db::open_connection_at(&paths.db_file)?;
    db::apply_all_migrations(&conn)?;

    // 8. Atomic institutional bootstrap transaction
    let receipt_pfx = payload.receipt_prefix.as_deref().unwrap_or("REC-").trim();
    let student_pfx = payload.student_id_prefix.as_deref().unwrap_or("S").trim();

    let (user_summary, session_token, expires_at) = db::run_in_transaction(&mut conn, |tx| {
        let now = db::now_utc_iso();

        // 8a. Insert institution metadata
        let addr = payload.address.as_deref().unwrap_or("").trim();
        let phone = payload.phone.as_deref().unwrap_or("").trim();
        tx.execute(
            "INSERT INTO institution (
                id, name, name_bn, address, phones, email, currency, timezone,
                fiscal_year_start, created_at, updated_at
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, 'BDT', 'Asia/Dhaka', 1, ?6, ?6)
            ON CONFLICT(id) DO UPDATE SET
                name = ?1, name_bn = ?2, address = ?3, phones = ?4, email = ?5, updated_at = ?6",
            rusqlite::params![
                payload.institution_name.trim(),
                payload.institution_name_bn.as_deref().map(|s| s.trim()),
                addr,
                phone,
                payload.email.as_deref().map(|s| s.trim()),
                now,
            ],
        )?;

        // 8b. Insert initial academic session
        let session_id = Uuid::now_v7().to_string();
        let session_year = payload.academic_session_name.trim();
        let start_date = format!("{}-01-01", session_year);
        let end_date = format!("{}-12-31", session_year);

        tx.execute(
            "INSERT INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)
             ON CONFLICT(name) DO UPDATE SET start_date = ?3, end_date = ?4, updated_at = ?5",
            rusqlite::params![session_id, session_year, start_date, end_date, now],
        )?;

        // 8c. Initialize sequences for student IDs and receipt numbers
        tx.execute(
            "INSERT INTO sequences (name, prefix, next_value) VALUES ('student', ?1, 1)
             ON CONFLICT(name) DO UPDATE SET prefix = ?1",
            rusqlite::params![student_pfx],
        )?;

        let seq_name = format!("receipt:{}", session_year);
        tx.execute(
            "INSERT INTO sequences (name, prefix, next_value) VALUES (?1, ?2, 1)
             ON CONFLICT(name) DO UPDATE SET prefix = ?2",
            rusqlite::params![seq_name, receipt_pfx],
        )?;

        // 8d. Seed initial owner account and active session
        let (user, token, exp) = accounts::create_initial_owner_in_conn(
            tx,
            payload.owner_username.trim(),
            payload.owner_display_name.trim(),
            &payload.owner_password,
            &now,
        )
        .map_err(|e| DbError::Config(format!("Failed to create owner account: {}", e)))?;

        // 8e. Seed 14-day evaluation trial license in licenses table (AC-4) and settings
        crate::licensing::ensure_initial_trial_license(tx, &now).map_err(|e| {
            DbError::Config(format!("Failed to seed evaluation trial license: {e}"))
        })?;
        let trial_until = crate::licensing::add_days_to_date_str(&now[0..10], 14);
        let license_blob = serde_json::json!({
            "status": "active",
            "tier": "trial",
            "institution_id": institution_id,
            "valid_until": trial_until,
        });
        tx.execute(
            "INSERT INTO settings (key, value_json, updated_by, updated_at)
             VALUES ('license', ?1, 'bootstrap', ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = ?1, updated_at = ?2",
            rusqlite::params![license_blob.to_string(), now],
        )?;

        // 8f. Seed clock_guard high-water mark
        tx.execute(
            "INSERT INTO clock_guard (high_water_at, last_ack_by, last_ack_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET high_water_at = ?1",
            rusqlite::params![now, "bootstrap", now],
        )?;

        // 8g. Record genesis audit log entry
        let before_json = None;
        let after_json = Some(format!(
            "{{\"institution_id\":\"{}\",\"name\":\"{}\",\"owner\":\"{}\"}}",
            institution_id,
            payload.institution_name.trim(),
            payload.owner_username.trim()
        ));

        let genesis_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let genesis_hash = audit::compute_audit_hash(
            genesis_prev_hash,
            &now,
            &now[0..10],
            Some(&user.id),
            Some("host_primary"),
            "system.bootstrap",
            "institution",
            &institution_id,
            before_json,
            after_json.as_deref(),
        );

        tx.execute(
            "INSERT INTO audit_log (
                seq, at, business_date, user_id, device_id, action, entity, entity_id,
                before_json, after_json, prev_hash, hash
            ) VALUES (1, ?1, ?2, ?3, 'host_primary', 'system.bootstrap', 'institution', ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                now,
                &now[0..10],
                user.id,
                institution_id,
                before_json,
                after_json,
                genesis_prev_hash,
                genesis_hash,
            ],
        )?;

        Ok((user, token, exp))
    })?;

    // 9. Write backup destination marker if destination provided
    if let Some(ref dest) = payload.backup_destination {
        if !dest.trim().is_empty() {
            let dest_path = PathBuf::from(dest.trim());
            let _ = write_backup_marker(&dest_path, &institution_id);
        }
    }

    // 10. Persist global configuration
    let global_config = GlobalConfig {
        data_dir: data_root.to_string_lossy().to_string(),
        institution_id: institution_id.clone(),
        app_mode: "standalone".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        backup_destination: payload.backup_destination.clone(),
    };
    storage::save_global_config(&global_config)?;

    Ok(InitializeInstitutionResponse {
        session_token,
        user: user_summary,
        expires_at,
        recovery_key: canonical_recovery_key,
        institution_id,
        data_dir: data_root.to_string_lossy().to_string(),
    })
}

// -----------------------------------------------------------------------------
// Disaster Recovery & Password Reset
// -----------------------------------------------------------------------------

/// Inspects a database directory to determine if disaster recovery is needed.
pub fn inspect_database(custom_dir: Option<&str>) -> Result<InspectDatabaseResponse, SetupError> {
    let (db_path, envelope_path) = if let Some(dir) = custom_dir {
        let p = PathBuf::from(dir);
        (p.join("main.db"), p.join("key.envelope"))
    } else if let Ok(Some(cfg)) = storage::load_global_config() {
        let root = PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("db");
        (root.join("main.db"), root.join("key.envelope"))
    } else {
        let dev_path = db::resolve_dev_db_path()?;
        let dev_env = dev_path.parent().unwrap().join("key.envelope");
        (dev_path, dev_env)
    };

    if !db_path.exists() {
        return Ok(InspectDatabaseResponse {
            database_exists: false,
            has_key_envelope: false,
            can_unwrap_dpapi: false,
            needs_recovery: false,
            institution_id: None,
            institution_name: None,
            schema_version: None,
        });
    }

    let has_key_envelope = envelope_path.exists();
    let can_unwrap_dpapi = if has_key_envelope {
        if let Ok(env) = KeyEnvelope::load_from_file(&envelope_path) {
            env.unwrap_with_dpapi().is_ok()
        } else {
            false
        }
    } else {
        true
    };

    let needs_recovery = has_key_envelope && !can_unwrap_dpapi;

    let (inst_id, inst_name, schema_version) = if can_unwrap_dpapi {
        if let Ok(conn) = db::open_connection_at(&db_path) {
            let inst_name = conn
                .query_row("SELECT name FROM institution LIMIT 1", [], |r| {
                    r.get::<_, String>(0)
                })
                .ok();
            let inst_id = conn
                .query_row(
                    "SELECT value_json FROM settings WHERE key = 'license'",
                    [],
                    |r| {
                        let v: String = r.get(0)?;
                        let parsed: serde_json::Value =
                            serde_json::from_str(&v).unwrap_or_default();
                        Ok(parsed
                            .get("institution_id")
                            .and_then(|id| id.as_str())
                            .map(String::from))
                    },
                )
                .ok()
                .flatten();
            let ver: Option<u32> = conn
                .query_row("SELECT MAX(version) FROM _migrations", [], |r| r.get(0))
                .ok();
            (inst_id, inst_name, ver)
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    Ok(InspectDatabaseResponse {
        database_exists: true,
        has_key_envelope,
        can_unwrap_dpapi,
        needs_recovery,
        institution_id: inst_id,
        institution_name: inst_name,
        schema_version,
    })
}

/// Unlocks an encrypted database on a new PC using the Owner Recovery Key,
/// re-wraps with local DPAPI, and optionally resets the Owner password.
pub fn unlock_database_with_key(
    payload: &UnlockWithKeyPayload,
) -> Result<UnlockWithKeyResponse, SetupError> {
    let (db_path, envelope_path) = if let Some(ref dir) = payload.db_dir {
        let p = PathBuf::from(dir);
        (p.join("main.db"), p.join("key.envelope"))
    } else if let Ok(Some(cfg)) = storage::load_global_config() {
        let root = PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("db");
        (root.join("main.db"), root.join("key.envelope"))
    } else {
        let dev_path = db::resolve_dev_db_path()?;
        let dev_env = dev_path.parent().unwrap().join("key.envelope");
        (dev_path, dev_env)
    };

    if !envelope_path.exists() {
        return Err(SetupError::Recovery(
            "key.envelope not found in database directory".to_string(),
        ));
    }

    let mut envelope = KeyEnvelope::load_from_file(&envelope_path)?;

    // 1. Decrypt master key using recovery key
    let master_key = envelope.unwrap_with_recovery_key(&payload.recovery_key)?;

    // 2. Re-wrap with current machine's DPAPI
    envelope.rewrap_dpapi(&master_key)?;
    envelope.save_to_file(&envelope_path)?;

    // 3. Open connection to database
    let mut conn = db::open_connection_at(&db_path)?;

    let inst_name: String =
        conn.query_row("SELECT name FROM institution LIMIT 1", [], |r| r.get(0))?;
    let inst_id = envelope.institution_id.clone();

    let mut password_reset = false;

    // 4. Reset owner password if requested
    if let Some(ref new_pwd) = payload.reset_owner_password {
        if new_pwd.len() < 8 {
            return Err(SetupError::Validation {
                field: "reset_owner_password".to_string(),
                message: "New password must be at least 8 characters long".to_string(),
            });
        }

        let new_hash =
            crate::auth::hash_password(new_pwd).map_err(|e| SetupError::Auth(e.to_string()))?;

        db::run_in_transaction(&mut conn, |tx| {
            // Find primary Owner user
            let owner_id: String = tx.query_row(
                "SELECT u.id FROM users u
                 JOIN user_roles ur ON u.id = ur.user_id
                 JOIN roles r ON ur.role_id = r.id
                 WHERE r.name = 'Owner' AND u.status = 'active'
                 LIMIT 1",
                [],
                |r| r.get(0),
            )?;

            tx.execute(
                "UPDATE users SET password_hash = ?1, failed_login_attempts = 0, last_failed_attempt_at = NULL, locked_until = NULL, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![new_hash, db::now_utc_iso(), owner_id],
            )?;

            Ok(())
        })?;

        password_reset = true;
    }

    Ok(UnlockWithKeyResponse {
        success: true,
        institution_id: inst_id,
        institution_name: inst_name,
        owner_password_reset: password_reset,
        message: "Database unlocked and successfully bound to this computer.".to_string(),
    })
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_institution_and_reopen() {
        crypto::set_mock_dpapi_force(true);
        let temp_dir = tempfile::tempdir().unwrap();
        let data_dir = temp_dir.path().join("coachify_test_data");

        let preview = generate_recovery_preview().unwrap();

        let payload = InitializeInstitutionPayload {
            institution_name: "Dhaka Science Academy".to_string(),
            institution_name_bn: Some("ঢাকা সায়েন্স একাডেমি".to_string()),
            address: Some("Mirpur 10, Dhaka".to_string()),
            phone: Some("01711000000".to_string()),
            email: Some("info@dhakascience.com".to_string()),
            academic_session_name: "2026".to_string(),
            student_id_prefix: Some("DSA-".to_string()),
            receipt_prefix: Some("REC-".to_string()),
            data_dir: data_dir.to_string_lossy().to_string(),
            backup_destination: None,
            owner_display_name: "Dr. Masud Rana".to_string(),
            owner_username: "director".to_string(),
            owner_password: "director_secure_pwd_123".to_string(),
            recovery_key: preview.recovery_key.clone(),
            recovery_key_confirmation: preview.checksum.clone(),
        };

        let init_res = initialize_institution(&payload).expect("initialization should succeed");
        assert_eq!(init_res.user.username, "director");
        assert_eq!(init_res.user.display_name, "Dr. Masud Rana");
        assert!(!init_res.session_token.is_empty());

        // Validate database inspection
        let db_root = data_dir.join(&init_res.institution_id).join("db");
        let inspect = inspect_database(Some(&db_root.to_string_lossy())).unwrap();
        assert!(inspect.database_exists);
        assert!(inspect.has_key_envelope);
        assert!(inspect.can_unwrap_dpapi);
        assert!(!inspect.needs_recovery);
        assert_eq!(
            inspect.institution_name,
            Some("Dhaka Science Academy".to_string())
        );

        // Test Recovery Screen Unlock
        let unlock_payload = UnlockWithKeyPayload {
            db_dir: Some(db_root.to_string_lossy().to_string()),
            recovery_key: preview.recovery_key,
            reset_owner_password: Some("new_owner_password_456".to_string()),
        };

        let unlock_res = unlock_database_with_key(&unlock_payload).unwrap();
        assert!(unlock_res.success);
        assert!(unlock_res.owner_password_reset);
        crypto::set_mock_dpapi_force(false);
    }

    #[test]
    fn test_confirmation_mismatch_fails() {
        crypto::set_mock_dpapi_force(true);
        let temp_dir = tempfile::tempdir().unwrap();
        let preview = generate_recovery_preview().unwrap();

        let payload = InitializeInstitutionPayload {
            institution_name: "Test Academy".to_string(),
            institution_name_bn: None,
            address: None,
            phone: None,
            email: None,
            academic_session_name: "2026".to_string(),
            student_id_prefix: None,
            receipt_prefix: None,
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            backup_destination: None,
            owner_display_name: "Test Owner".to_string(),
            owner_username: "owner".to_string(),
            owner_password: "password123".to_string(),
            recovery_key: preview.recovery_key,
            recovery_key_confirmation: "ZZZZ".to_string(), // mismatched
        };

        let res = initialize_institution(&payload);
        assert!(matches!(
            res,
            Err(SetupError::RecoveryConfirmationMismatch { .. })
        ));
        crypto::set_mock_dpapi_force(false);
    }
}
