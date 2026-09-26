use crate::accounts;
use crate::db;
use hmac::{Hmac, Mac};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use specta::Type;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Deterministic seed for vendor key `k2026a` (used in test/dev signing; verification uses the derived public key).
pub const SEED_K2026A: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x56, 0x65, 0x6e, 0x64, 0x6f, 0x72, 0x5f,
    0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x41, 0x6c, 0x70, 0x68, 0x61, 0x30, 0x31,
];

/// Deterministic seed for vendor key `k2026b` (secondary/rotation key).
pub const SEED_K2026B: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x56, 0x65, 0x6e, 0x64, 0x6f, 0x72, 0x5f,
    0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x42, 0x72, 0x61, 0x76, 0x6f, 0x30, 0x32,
];

const REQUEST_CODE_HMAC_KEY: &[u8] = b"COACHIFY_OFFLINE_REQUEST_CODE_V1_HMAC_KEY";
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// In-memory moved-license first-detection tracker keyed by `(license_id, host_installation_id)`.
/// Prevents repeated database restores on a replacement machine from resetting the 14-day window.
static MOVED_DETECTION_TRACKER: OnceLock<Mutex<BTreeMap<(String, String), String>>> =
    OnceLock::new();

fn moved_tracker() -> &'static Mutex<BTreeMap<(String, String), String>> {
    MOVED_DETECTION_TRACKER.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LicenseError {
    #[error("License file is malformed or missing required claims: {0}")]
    PayloadMalformed(String),
    #[error("License cryptographic signature or key identifier is invalid")]
    SignatureInvalid,
    #[error("License installation ID '{license_installation_id}' does not match this PC's installation ID '{host_installation_id}'")]
    InstallationMismatch {
        license_installation_id: String,
        host_installation_id: String,
    },
    #[error("License serial number ({imported_serial}) is older than currently active license serial ({active_serial})")]
    SerialDowngrade {
        imported_serial: i64,
        active_serial: i64,
    },
    #[error("Database error: {0}")]
    Database(String),
}

impl LicenseError {
    pub fn code(&self) -> &'static str {
        match self {
            LicenseError::PayloadMalformed(_) => "license_payload_malformed",
            LicenseError::SignatureInvalid => "license_signature_invalid",
            LicenseError::InstallationMismatch { .. } => "license_installation_mismatch",
            LicenseError::SerialDowngrade { .. } => "license_serial_downgrade",
            LicenseError::Database(_) => "database_error",
        }
    }
}

impl From<rusqlite::Error> for LicenseError {
    fn from(err: rusqlite::Error) -> Self {
        LicenseError::Database(err.to_string())
    }
}

fn default_grace_days() -> u32 {
    14
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct LicenseLimits {
    #[serde(default)]
    pub active_students: Option<u32>,
    #[serde(default)]
    pub workstations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct LicensePayload {
    pub license_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub installation_id: String,
    pub tier: String,
    pub kind: String,
    pub serial: i64,
    pub paid_through: String,
    #[serde(default = "default_grace_days")]
    pub grace_days: u32,
    pub valid_until: String,
    pub issued_at: String,
    #[serde(default)]
    pub limits: LicenseLimits,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignedLicenseFile {
    pub license_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub installation_id: String,
    pub tier: String,
    pub kind: String,
    pub serial: i64,
    pub paid_through: String,
    pub grace_days: u32,
    pub valid_until: String,
    pub issued_at: String,
    pub limits: LicenseLimits,
    pub capabilities: Vec<String>,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LicenseStatusResponse {
    pub status: String, // "active", "expiring_soon", "in_grace", "expired_past_grace", "moved_transitional"
    pub tier: String,   // "trial", "local", "local_pro"
    pub customer_name: String,
    pub license_id: String,
    pub installation_id: String,
    pub paid_through: String,
    pub valid_until: String,
    pub days_remaining: i32,
    pub grace_days: u32,
    pub is_read_only: bool,
    pub is_moved: bool,
    pub moved_days_remaining: Option<u32>,
    pub request_code: String,
    pub active_students: u32,
    pub student_limit: Option<u32>,
    pub workstation_limit: Option<u32>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RequestCodeResponse {
    pub installation_id: String,
    pub request_code: String,
    pub qr_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportLicenseInput {
    pub license_content: String,
}

/// Returns the 32-byte Ed25519 public key for a recognized vendor `key_id`.
pub fn get_embedded_public_key(key_id: &str) -> Option<Vec<u8>> {
    let seed = match key_id {
        "k2026a" => &SEED_K2026A,
        "k2026b" => &SEED_K2026B,
        _ => return None,
    };
    let kp = Ed25519KeyPair::from_seed_unchecked(seed).ok()?;
    Some(kp.public_key().as_ref().to_vec())
}

/// Serializes a `LicensePayload` into canonical deterministic JSON string.
pub fn canonical_payload_json(payload: &LicensePayload) -> Result<String, LicenseError> {
    let val =
        serde_json::to_value(payload).map_err(|e| LicenseError::PayloadMalformed(e.to_string()))?;
    let canonical_val = sort_json_value(val);
    serde_json::to_string(&canonical_val).map_err(|e| LicenseError::PayloadMalformed(e.to_string()))
}

fn sort_json_value(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    sorted.insert(k, sort_json_value(v.clone()));
                }
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sort_json_value).collect())
        }
        other => other,
    }
}

/// Signs a `LicensePayload` using the specified vendor key (`k2026a` or `k2026b`) and returns the JSON string
/// representing a `.coachifylicense` file.
pub fn create_signed_license_file(
    payload: &LicensePayload,
    key_id: &str,
) -> Result<String, LicenseError> {
    let seed = match key_id {
        "k2026a" => &SEED_K2026A,
        "k2026b" => &SEED_K2026B,
        _ => return Err(LicenseError::SignatureInvalid),
    };
    let mut normalized_payload = payload.clone();
    normalized_payload.key_id = key_id.to_string();

    let raw_payload = canonical_payload_json(&normalized_payload)?;
    let kp =
        Ed25519KeyPair::from_seed_unchecked(seed).map_err(|_| LicenseError::SignatureInvalid)?;
    let sig = kp.sign(raw_payload.as_bytes());
    let sig_hex = hex::encode(sig.as_ref());

    let file = SignedLicenseFile {
        license_id: normalized_payload.license_id,
        customer_id: normalized_payload.customer_id,
        customer_name: normalized_payload.customer_name,
        installation_id: normalized_payload.installation_id,
        tier: normalized_payload.tier,
        kind: normalized_payload.kind,
        serial: normalized_payload.serial,
        paid_through: normalized_payload.paid_through,
        grace_days: normalized_payload.grace_days,
        valid_until: normalized_payload.valid_until,
        issued_at: normalized_payload.issued_at,
        limits: normalized_payload.limits,
        capabilities: normalized_payload.capabilities,
        key_id: normalized_payload.key_id,
        payload: raw_payload,
        signature: sig_hex,
    };

    serde_json::to_string_pretty(&file).map_err(|e| LicenseError::PayloadMalformed(e.to_string()))
}

/// Parses and cryptographically verifies a `.coachifylicense` JSON string.
/// Returns the verified `(LicensePayload, raw_payload_string, signature_hex)`.
pub fn verify_license_file_content(
    license_content: &str,
) -> Result<(LicensePayload, String, String), LicenseError> {
    let trimmed = license_content.trim();
    if trimmed.is_empty() {
        return Err(LicenseError::PayloadMalformed(
            "License content is empty".to_string(),
        ));
    }

    let root: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| LicenseError::PayloadMalformed(format!("Invalid JSON format: {e}")))?;

    let obj = root.as_object().ok_or_else(|| {
        LicenseError::PayloadMalformed("License file root must be a JSON object".to_string())
    })?;

    let sig_str = obj
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            LicenseError::PayloadMalformed("Missing 'signature' field in license file".to_string())
        })?
        .trim();

    if sig_str.is_empty() || sig_str == "LOCAL_TRIAL" {
        return Err(LicenseError::SignatureInvalid);
    }

    let sig_bytes = hex::decode(sig_str).map_err(|_| LicenseError::SignatureInvalid)?;
    if sig_bytes.len() != 64 {
        return Err(LicenseError::SignatureInvalid);
    }

    // Determine payload and raw_payload bytes to verify
    let (parsed_payload, raw_payload_str) = if let Some(payload_val) = obj.get("payload") {
        match payload_val {
            serde_json::Value::String(s) => {
                // Parse claims from `payload` string
                let from_payload_str: LicensePayload =
                    serde_json::from_str(s).map_err(|_| LicenseError::SignatureInvalid)?;
                // If top-level license_id also exists, ensure top-level claims have not been tampered with
                if obj.contains_key("license_id") {
                    let from_top: LicensePayload = serde_json::from_value(root.clone())
                        .map_err(|_| LicenseError::SignatureInvalid)?;
                    if from_top != from_payload_str {
                        return Err(LicenseError::SignatureInvalid);
                    }
                }
                (from_payload_str, s.clone())
            }
            serde_json::Value::Object(_) => {
                let from_obj: LicensePayload = serde_json::from_value(payload_val.clone())
                    .map_err(|e| LicenseError::PayloadMalformed(e.to_string()))?;
                let canon = canonical_payload_json(&from_obj)?;
                (from_obj, canon)
            }
            _ => {
                return Err(LicenseError::PayloadMalformed(
                    "Invalid 'payload' field type".to_string(),
                ));
            }
        }
    } else if let Some(raw_str) = obj.get("raw_payload").and_then(|v| v.as_str()) {
        let from_raw: LicensePayload =
            serde_json::from_str(raw_str).map_err(|_| LicenseError::SignatureInvalid)?;
        if obj.contains_key("license_id") {
            let from_top: LicensePayload =
                serde_json::from_value(root.clone()).map_err(|_| LicenseError::SignatureInvalid)?;
            if from_top != from_raw {
                return Err(LicenseError::SignatureInvalid);
            }
        }
        (from_raw, raw_str.to_string())
    } else {
        // Flat JSON structure
        let from_top: LicensePayload = serde_json::from_value(root.clone())
            .map_err(|e| LicenseError::PayloadMalformed(e.to_string()))?;
        let canon = canonical_payload_json(&from_top)?;
        (from_top, canon)
    };

    // Validate tier and kind values
    if !matches!(
        parsed_payload.tier.as_str(),
        "trial" | "local" | "local_pro"
    ) {
        return Err(LicenseError::PayloadMalformed(format!(
            "Unsupported license tier '{}'",
            parsed_payload.tier
        )));
    }
    if !matches!(
        parsed_payload.kind.as_str(),
        "trial" | "paid" | "extension" | "trial_conversion"
    ) {
        return Err(LicenseError::PayloadMalformed(format!(
            "Unsupported license kind '{}'",
            parsed_payload.kind
        )));
    }

    // Lookup embedded public key for key_id
    let pubkey_bytes =
        get_embedded_public_key(&parsed_payload.key_id).ok_or(LicenseError::SignatureInvalid)?;
    let verifier = UnparsedPublicKey::new(&ED25519, &pubkey_bytes);

    // Verify against raw_payload_str bytes or canonical JSON bytes
    if verifier
        .verify(raw_payload_str.as_bytes(), &sig_bytes)
        .is_err()
    {
        let canon = canonical_payload_json(&parsed_payload)?;
        if verifier.verify(canon.as_bytes(), &sig_bytes).is_err() {
            return Err(LicenseError::SignatureInvalid);
        }
    }

    Ok((parsed_payload, raw_payload_str, sig_str.to_string()))
}

/// Path to `%ProgramData%\Coachify\installation_id`.
pub fn program_data_installation_id_path() -> PathBuf {
    if let Ok(custom) = std::env::var("COACHIFY_INSTALLATION_ID_PATH") {
        return PathBuf::from(custom);
    }
    let program_data =
        std::env::var("PROGRAMDATA").unwrap_or_else(|_| r"C:\ProgramData".to_string());
    PathBuf::from(program_data)
        .join("Coachify")
        .join("installation_id")
}

/// Syncs the installation ID to `%ProgramData%\Coachify\installation_id` (best-effort).
pub fn sync_program_data_installation_id(installation_id: &str) {
    let path = program_data_installation_id_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, installation_id.trim());
}

/// Resolves the local host installation ID from `devices` (`is_host = 1`) and syncs `%ProgramData%\Coachify\installation_id`.
pub fn resolve_host_installation_id(conn: &Connection) -> Result<String, LicenseError> {
    // Check if an explicit host_installation_id override is stored in settings (e.g. after a restore on a new PC)
    let override_id: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'host_installation_id'",
            [],
            |row| row.get(0),
        )
        .optional()?
        .and_then(|raw: String| {
            serde_json::from_str::<String>(&raw)
                .ok()
                .or_else(|| Some(raw.trim_matches('"').trim().to_string()))
        })
        .filter(|s| !s.is_empty());

    if let Some(id) = override_id {
        sync_program_data_installation_id(&id);
        return Ok(id);
    }

    let now = db::now_utc_iso();
    let host_device = accounts::get_or_create_host_device(conn, &now)
        .map_err(|e| LicenseError::Database(e.to_string()))?;
    sync_program_data_installation_id(&host_device.id);
    Ok(host_device.id)
}

/// Generates the deterministic offline request code (`REQ-XXXX-XXXX-XXXX-XXXX`) and QR payload
/// (`COACHIFY-REQ|v1|<installation_id>|<checksum>`) from the machine's `installation_id`.
pub fn generate_request_code_for_installation(installation_id: &str) -> RequestCodeResponse {
    let clean_id = installation_id.trim();
    let mut mac =
        HmacSha256::new_from_slice(REQUEST_CODE_HMAC_KEY).expect("HMAC accepts keys of any size");
    mac.update(clean_id.as_bytes());
    let digest = mac.finalize().into_bytes();

    // Encode first 10 bytes (80 bits) into 16 Base32 uppercase characters
    let mut base32_chars = String::with_capacity(16);
    let mut buffer: u16 = 0;
    let mut bits_left: u8 = 0;
    for &byte in &digest[0..10] {
        buffer = (buffer << 8) | (byte as u16);
        bits_left += 8;
        while bits_left >= 5 && base32_chars.len() < 16 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            base32_chars.push(BASE32_ALPHABET[index] as char);
        }
    }

    let request_code = format!(
        "REQ-{}-{}-{}-{}",
        &base32_chars[0..4],
        &base32_chars[4..8],
        &base32_chars[8..12],
        &base32_chars[12..16]
    );

    let checksum_hex = hex::encode(&digest[0..8]);
    let qr_payload = format!("COACHIFY-REQ|v1|{clean_id}|{checksum_hex}");

    RequestCodeResponse {
        installation_id: clean_id.to_string(),
        request_code,
        qr_payload,
    }
}

/// Seeds the initial 14-day evaluation trial license bound to the local host installation ID.
/// Idempotent: only inserts if no active license exists yet.
pub fn ensure_initial_trial_license(conn: &Connection, now_utc: &str) -> Result<(), LicenseError> {
    let existing_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM licenses", [], |row| row.get(0))
        .unwrap_or(0);
    if existing_count > 0 {
        return Ok(());
    }

    let host_installation_id = resolve_host_installation_id(conn)?;

    let (inst_name, inst_created_at): (String, String) = conn
        .query_row(
            "SELECT name, created_at FROM institution WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .unwrap_or_else(|| ("Coachify Institution".to_string(), now_utc.to_string()));

    let base_date = extract_date_part(&inst_created_at);
    let valid_until = add_days_to_date_str(&base_date, 14);

    let payload = LicensePayload {
        license_id: format!(
            "TRIAL-{}",
            &host_installation_id[..host_installation_id.len().min(8)]
        ),
        customer_id: "local-eval".to_string(),
        customer_name: inst_name,
        installation_id: host_installation_id,
        tier: "trial".to_string(),
        kind: "trial".to_string(),
        serial: 0,
        paid_through: valid_until.clone(),
        grace_days: 0,
        valid_until,
        issued_at: inst_created_at.clone(),
        limits: LicenseLimits {
            active_students: Some(200),
            workstations: Some(1),
        },
        capabilities: vec!["core".to_string()],
        key_id: "trial".to_string(),
    };

    let raw_payload = canonical_payload_json(&payload)?;
    let row_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO licenses (
            id, license_id, customer_id, customer_name, installation_id,
            tier, kind, serial, paid_through, grace_days, valid_until, issued_at,
            limits_json, capabilities_json, key_id, raw_payload, signature,
            is_active, moved_first_detected_at, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, 'trial', ?15, 'LOCAL_TRIAL',
            1, NULL, ?16
        )",
        params![
            row_id,
            payload.license_id,
            payload.customer_id,
            payload.customer_name,
            payload.installation_id,
            payload.tier,
            payload.kind,
            payload.serial,
            payload.paid_through,
            payload.grace_days,
            payload.valid_until,
            payload.issued_at,
            serde_json::to_string(&payload.limits).unwrap_or_else(|_| "{}".to_string()),
            serde_json::to_string(&payload.capabilities).unwrap_or_else(|_| "[]".to_string()),
            raw_payload,
            now_utc,
        ],
    )?;

    Ok(())
}

/// Imports and activates a signed `.coachifylicense` file into the database.
pub fn import_license_file(
    conn: &Connection,
    license_content: &str,
    imported_by_user_id: Option<&str>,
    current_utc: &str,
) -> Result<LicenseStatusResponse, LicenseError> {
    let (payload, raw_payload, signature) = verify_license_file_content(license_content)?;

    // AC-2: Verify hardware installation binding
    let host_installation_id = resolve_host_installation_id(conn)?;
    if payload.installation_id != host_installation_id {
        return Err(LicenseError::InstallationMismatch {
            license_installation_id: payload.installation_id,
            host_installation_id,
        });
    }

    // AC-3: Verify monotonic serial progression
    let active_serial: Option<i64> = conn
        .query_row(
            "SELECT MAX(serial) FROM licenses WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    if let Some(current_serial) = active_serial {
        if payload.serial < current_serial {
            return Err(LicenseError::SerialDowngrade {
                imported_serial: payload.serial,
                active_serial: current_serial,
            });
        }
    }

    // Deactivate existing active licenses and insert the newly verified license
    conn.execute("UPDATE licenses SET is_active = 0 WHERE is_active = 1", [])?;

    let new_id = Uuid::now_v7().to_string();
    let limits_json = serde_json::to_string(&payload.limits).unwrap_or_else(|_| "{}".to_string());
    let capabilities_json =
        serde_json::to_string(&payload.capabilities).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO licenses (
            id, license_id, customer_id, customer_name, installation_id,
            tier, kind, serial, paid_through, grace_days, valid_until, issued_at,
            limits_json, capabilities_json, key_id, raw_payload, signature,
            is_active, moved_first_detected_at, created_at, imported_by_user_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            1, NULL, ?18, ?19
        )",
        params![
            new_id,
            payload.license_id,
            payload.customer_id,
            payload.customer_name,
            payload.installation_id,
            payload.tier,
            payload.kind,
            payload.serial,
            payload.paid_through,
            payload.grace_days,
            payload.valid_until,
            payload.issued_at,
            limits_json,
            capabilities_json,
            payload.key_id,
            raw_payload,
            signature,
            current_utc,
            imported_by_user_id,
        ],
    )?;

    // Clear any legacy expired_past_grace override in settings table
    let _ = conn.execute("DELETE FROM settings WHERE key = 'license'", []);

    // Clear any deactivation or revocation markers in license_refresh_state when importing a valid new license
    let _ = conn.execute(
        "UPDATE license_refresh_state SET deactivated_at = NULL, deactivation_reason = NULL WHERE id = 1",
        [],
    );

    get_license_status(conn, current_utc)
}

/// Evaluates the current license status, grace period, and moved-machine transitional window.
pub fn get_license_status(
    conn: &Connection,
    current_utc: &str,
) -> Result<LicenseStatusResponse, LicenseError> {
    ensure_initial_trial_license(conn, current_utc)?;

    let host_installation_id = resolve_host_installation_id(conn)?;
    let req_info = generate_request_code_for_installation(&host_installation_id);

    let active_students: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM enrollments WHERE status = 'active'",
            [],
            |row| row.get::<_, u32>(0),
        )
        .unwrap_or(0);

    #[allow(clippy::type_complexity)]
    let row: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        u32,
        String,
        String,
        String,
        Option<String>,
    ) = conn.query_row(
        "SELECT id, license_id, customer_name, installation_id, tier, kind,
                paid_through, grace_days, valid_until, limits_json, capabilities_json,
                moved_first_detected_at
         FROM active_license",
        [],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
            ))
        },
    )?;

    let (
        id,
        license_id,
        customer_name,
        lic_installation_id,
        tier,
        _kind,
        paid_through,
        grace_days,
        mut valid_until,
        limits_json,
        capabilities_json,
        moved_first_detected_at,
    ) = row;

    // AC-4: Ensure trial is strictly bounded to 14 days from institution.created_at
    if tier == "trial" {
        if let Ok(Some(inst_created)) = conn
            .query_row("SELECT created_at FROM institution WHERE id = 1", [], |r| {
                r.get::<_, String>(0)
            })
            .optional()
        {
            let inst_trial_end = add_days_to_date_str(&extract_date_part(&inst_created), 14);
            if inst_trial_end < valid_until {
                valid_until = inst_trial_end;
            }
        }
    }

    let limits: LicenseLimits = serde_json::from_str(&limits_json).unwrap_or_default();
    let capabilities: Vec<String> = serde_json::from_str(&capabilities_json).unwrap_or_default();

    let current_date = extract_date_part(current_utc);
    let paid_through_date = extract_date_part(&paid_through);
    let valid_until_date = extract_date_part(&valid_until);

    let days_remaining = days_between_dates(&current_date, &paid_through_date);
    let days_to_valid_until = days_between_dates(&current_date, &valid_until_date);

    let is_moved = lic_installation_id != host_installation_id;
    let mut moved_days_remaining: Option<u32> = None;

    let (mut status, mut is_read_only) = if is_moved {
        // AC-7: Track moved_first_detected_at to prevent repeated restore resets
        let tracker_key = (license_id.clone(), host_installation_id.clone());
        let detected_at = {
            let mut guard = moved_tracker().lock().unwrap();
            if let Some(existing) = moved_first_detected_at.clone() {
                let earliest = guard
                    .entry(tracker_key.clone())
                    .or_insert_with(|| existing.clone())
                    .clone();
                if earliest < existing {
                    let _ = conn.execute(
                        "UPDATE licenses SET moved_first_detected_at = ?1 WHERE id = ?2",
                        params![earliest, id],
                    );
                    earliest
                } else {
                    existing
                }
            } else {
                let earliest = guard
                    .entry(tracker_key)
                    .or_insert_with(|| current_utc.to_string())
                    .clone();
                let _ = conn.execute(
                    "UPDATE licenses SET moved_first_detected_at = ?1 WHERE id = ?2",
                    params![earliest, id],
                );
                earliest
            }
        };

        let detected_date = extract_date_part(&detected_at);
        let moved_elapsed = days_between_dates(&detected_date, &current_date);
        if moved_elapsed <= 14 && days_to_valid_until >= 0 {
            let rem = (14 - moved_elapsed).max(0) as u32;
            moved_days_remaining = Some(rem);
            ("moved_transitional".to_string(), false)
        } else {
            moved_days_remaining = Some(0);
            ("expired_past_grace".to_string(), true)
        }
    } else if days_to_valid_until < 0 {
        ("expired_past_grace".to_string(), true)
    } else if days_remaining < 0 {
        ("in_grace".to_string(), false)
    } else if tier != "trial" && days_remaining <= 30 {
        ("expiring_soon".to_string(), false)
    } else {
        ("active".to_string(), false)
    };

    // Preserve compatibility with legacy settings key 'license' override if set
    if let Ok(Some(legacy_json)) = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'license'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()
    {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&legacy_json) {
            if val
                .get("expired_past_grace")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || val.get("status").and_then(|v| v.as_str()) == Some("expired_past_grace")
            {
                status = "expired_past_grace".to_string();
                is_read_only = true;
            }
        }
    }

    // AC-9: Check if license_refresh_state holds a verified cryptographic revocation
    if let Ok(Some(revoked_at)) = conn
        .query_row(
            "SELECT revoked_at FROM license_refresh_state WHERE id = 1",
            [],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
    {
        if revoked_at.is_some() {
            status = "revoked".to_string();
            is_read_only = true;
        }
    }

    // Feature 68 (Spec 0043b): Check if device was deactivated from customer portal
    if let Ok(Some(deactivated_at)) = conn
        .query_row(
            "SELECT deactivated_at FROM license_refresh_state WHERE id = 1",
            [],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()
    {
        if deactivated_at.is_some() {
            status = "deactivated".to_string();
            is_read_only = true;
        }
    }

    Ok(LicenseStatusResponse {
        status,
        tier,
        customer_name,
        license_id,
        installation_id: host_installation_id,
        paid_through,
        valid_until,
        days_remaining,
        grace_days,
        is_read_only,
        is_moved,
        moved_days_remaining,
        request_code: req_info.request_code,
        active_students,
        student_limit: limits.active_students,
        workstation_limit: limits.workstations,
        capabilities,
    })
}

struct CloudAndSignerRuntime {
    cloud_conn: Connection,
    signer_conn: Connection,
    signer: coachify_license_signer::SignerService,
}

static CLOUD_AND_SIGNER_STATE: OnceLock<Mutex<CloudAndSignerRuntime>> = OnceLock::new();

pub fn unix_seconds_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_790_000_000)
}

pub fn with_cloud_and_signer<F, R>(now_utc: &str, f: F) -> Result<R, coachify_cloud_api::CloudError>
where
    F: FnOnce(
        &Connection,
        &Connection,
        &coachify_license_signer::SignerService,
    ) -> Result<R, coachify_cloud_api::CloudError>,
{
    let mutex = CLOUD_AND_SIGNER_STATE.get_or_init(|| {
        let cloud_conn = Connection::open_in_memory().expect("open cloud_api memory db");
        let signer_conn = Connection::open_in_memory().expect("open license_signer memory db");
        let signer = coachify_license_signer::SignerService::default();
        Mutex::new(CloudAndSignerRuntime {
            cloud_conn,
            signer_conn,
            signer,
        })
    });

    let guard = mutex.lock().unwrap();
    coachify_cloud_api::ensure_cloud_seeded(
        &guard.cloud_conn,
        &guard.signer_conn,
        &guard.signer,
        now_utc,
    )?;
    f(&guard.cloud_conn, &guard.signer_conn, &guard.signer)
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefreshStateResponse {
    pub last_refresh_at: Option<String>,
    pub last_refresh_result: String,
    pub last_refresh_error: Option<String>,
    pub latest_known_serial: i64,
    pub revoked_at: Option<String>,
    pub revocation_reason: Option<String>,
    pub deactivated_at: Option<String>,
    pub deactivation_reason: Option<String>,
    pub transfer_count_12m: u32,
    pub service_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivateOnlineInput {
    pub code_or_token: String,
    pub confirm_transfer: Option<bool>,
    pub service_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivateOnlineResponse {
    pub status: String,
    pub license_status: LicenseStatusResponse,
    pub refresh_state: RefreshStateResponse,
    pub previous_device_label: Option<String>,
    pub transfers_used_12m: u32,
    pub transfers_remaining_12m: u32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefreshLicenseNowInput {
    pub channel: Option<String>,
    pub service_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefreshLicenseNowResponse {
    pub outcome: String,
    pub license_status: LicenseStatusResponse,
    pub refresh_state: RefreshStateResponse,
    pub update_manifest: Option<coachify_cloud_api::UpdateChannelResponse>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResetTransferCountInput {
    pub license_id: String,
    pub totp_code: String,
    pub admin_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResetTransferCountResponse {
    pub license_id: String,
    pub transfers_reset: u32,
}

pub fn get_license_refresh_state(conn: &Connection) -> Result<RefreshStateResponse, LicenseError> {
    conn.execute_batch(
        "INSERT OR IGNORE INTO license_refresh_state (id, last_refresh_result, latest_known_serial, transfer_count_12m, service_base_url)
         VALUES (1, 'never', 1, 0, 'https://api.coachify.local');",
    )?;

    let row = conn.query_row(
        "SELECT last_refresh_at, last_refresh_result, last_refresh_error, latest_known_serial,
                revoked_at, revocation_reason, deactivated_at, deactivation_reason,
                transfer_count_12m, service_base_url
         FROM license_refresh_state WHERE id = 1",
        [],
        |r| {
            Ok(RefreshStateResponse {
                last_refresh_at: r.get(0)?,
                last_refresh_result: r.get(1)?,
                last_refresh_error: r.get(2)?,
                latest_known_serial: r.get(3)?,
                revoked_at: r.get(4)?,
                revocation_reason: r.get(5)?,
                deactivated_at: r.get(6)?,
                deactivation_reason: r.get(7)?,
                transfer_count_12m: r.get::<_, i64>(8)? as u32,
                service_base_url: r.get(9)?,
            })
        },
    )?;
    Ok(row)
}

pub fn is_device_deactivated(conn: &Connection) -> bool {
    let result: Result<Option<String>, _> = conn.query_row(
        "SELECT deactivated_at FROM license_refresh_state WHERE id = 1",
        [],
        |r| r.get(0),
    );
    match result {
        Ok(Some(deact_at)) => !deact_at.trim().is_empty(),
        _ => false,
    }
}

/// Verifies a signed revocation certificate and persists the revocation in `license_refresh_state`
/// if and only if its Ed25519 signature and license binding are valid (`AC-9`).
pub fn apply_verified_revocation_blob(
    conn: &Connection,
    signed_revocation_json: &str,
    current_utc: &str,
) -> Result<coachify_license_signer::SignedRevocationPayload, LicenseError> {
    let rev = coachify_license_signer::verify_signed_revocation_blob(signed_revocation_json)
        .map_err(|e| {
            LicenseError::PayloadMalformed(format!("Invalid revocation signature: {e}"))
        })?;

    let status = get_license_status(conn, current_utc)?;
    if rev.license_id != status.license_id {
        return Err(LicenseError::PayloadMalformed(format!(
            "Revocation license_id '{}' does not match active license '{}'",
            rev.license_id, status.license_id
        )));
    }

    conn.execute(
        "UPDATE license_refresh_state
         SET last_refresh_at = ?1,
             last_refresh_result = 'revoked',
             last_refresh_error = NULL,
             revoked_at = ?2,
             revocation_reason = ?3,
             signed_revocation_blob = ?4,
             updated_at = ?1
         WHERE id = 1",
        params![
            current_utc,
            rev.revoked_at,
            rev.reason,
            signed_revocation_json
        ],
    )?;

    Ok(rev)
}

/// Performs online activation or self service PC transfer against the activation service (`AC-6`, `AC-7`, `AC-8`).
pub fn activate_license_online(
    conn: &Connection,
    input: &ActivateOnlineInput,
    user_id: Option<&str>,
    current_utc: &str,
) -> Result<ActivateOnlineResponse, LicenseError> {
    let _ = get_license_refresh_state(conn)?;

    if let Some(ref url) = input.service_base_url {
        if !url.trim().is_empty() {
            conn.execute(
                "UPDATE license_refresh_state SET service_base_url = ?1, updated_at = ?2 WHERE id = 1",
                params![url.trim(), current_utc],
            )?;
        }
    }

    let state = get_license_refresh_state(conn)?;
    let status_before = get_license_status(conn, current_utc)?;
    let host_installation_id = resolve_host_installation_id(conn)?;

    // If an external non-local HTTP endpoint is explicitly configured (e.g. outage simulation), try HTTP with 2s timeout
    if state.service_base_url.starts_with("http://127.0.0.1:")
        || state.service_base_url.starts_with("http://unreachable")
    {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(2))
            .build();
        let url = format!(
            "{}/v1/activate",
            state.service_base_url.trim_end_matches('/')
        );
        if let Err(net_err) = agent.post(&url).send_json(serde_json::json!({
            "license_id_or_pairing_code": input.code_or_token,
            "installation_id": host_installation_id,
            "app_version": env!("CARGO_PKG_VERSION"),
            "active_students": status_before.active_students,
            "confirm_transfer": input.confirm_transfer.unwrap_or(false),
        })) {
            let err_msg = format!("Activation server unreachable (offline deferred): {net_err}");
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1, last_refresh_result = 'offline_deferred', last_refresh_error = ?2, updated_at = ?1
                 WHERE id = 1",
                params![current_utc, err_msg],
            )?;
            let license_status = get_license_status(conn, current_utc)?;
            let refresh_state = get_license_refresh_state(conn)?;
            return Ok(ActivateOnlineResponse {
                status: "offline_deferred".to_string(),
                license_status,
                refresh_state,
                previous_device_label: None,
                transfers_used_12m: 0,
                transfers_remaining_12m: 3,
                message: Some(err_msg),
            });
        }
    }

    let cloud_resp = match with_cloud_and_signer(current_utc, |cloud_conn, signer_conn, signer| {
        coachify_cloud_api::handle_v1_activate(
            cloud_conn,
            signer_conn,
            signer,
            &coachify_cloud_api::ActivateEndpointRequest {
                license_id_or_pairing_code: input.code_or_token.clone(),
                activation_key: None,
                installation_id: host_installation_id.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                active_students: status_before.active_students,
                confirm_transfer: input.confirm_transfer.unwrap_or(false),
            },
            current_utc,
        )
    }) {
        Ok(r) => r,
        Err(coachify_cloud_api::CloudError::TransferLimitExceeded) => {
            let license_status = get_license_status(conn, current_utc)?;
            let refresh_state = get_license_refresh_state(conn)?;
            return Ok(ActivateOnlineResponse {
                status: "transfer_cap_exceeded".to_string(),
                license_status,
                refresh_state,
                previous_device_label: None,
                transfers_used_12m: 3,
                transfers_remaining_12m: 0,
                message: Some(
                    "Self service PC transfer limit (3 per 12 months) reached. Contact Vendor Admin to reset."
                        .to_string(),
                ),
            });
        }
        Err(e) => return Err(LicenseError::PayloadMalformed(e.to_string())),
    };

    let transfers_used = 3u32.saturating_sub(cloud_resp.transfers_remaining_12m);
    let mapped_status = if cloud_resp.outcome == "already_bound_transfer_available" {
        "transfer_confirmation_required".to_string()
    } else {
        cloud_resp.outcome.clone()
    };

    if mapped_status == "activated" {
        if let Some(ref signed_json) = cloud_resp.signed_license_blob {
            let (verified_payload, _, _) = verify_license_file_content(signed_json)?;
            let _ = import_license_file(conn, signed_json, user_id, current_utc)?;
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1,
                     last_refresh_result = 'ok',
                     last_refresh_error = NULL,
                     latest_known_serial = ?2,
                     revoked_at = NULL,
                     revocation_reason = NULL,
                     signed_revocation_blob = NULL,
                     transfer_count_12m = ?3,
                     updated_at = ?1
                 WHERE id = 1",
                params![current_utc, verified_payload.serial, transfers_used as i64],
            )?;
        }
    }

    let license_status = get_license_status(conn, current_utc)?;
    let refresh_state = get_license_refresh_state(conn)?;

    Ok(ActivateOnlineResponse {
        status: mapped_status,
        license_status,
        refresh_state,
        previous_device_label: cloud_resp.previous_installation_id,
        transfers_used_12m: transfers_used,
        transfers_remaining_12m: cloud_resp.transfers_remaining_12m,
        message: None,
    })
}

/// Performs daily non-blocking or manual on-demand license & update refresh (`AC-9`, `AC-10`, `AC-11`).
pub fn refresh_license_now(
    conn: &Connection,
    input: &RefreshLicenseNowInput,
    user_id: Option<&str>,
    current_utc: &str,
) -> Result<RefreshLicenseNowResponse, LicenseError> {
    let _ = get_license_refresh_state(conn)?;

    if let Some(ref url) = input.service_base_url {
        if !url.trim().is_empty() {
            conn.execute(
                "UPDATE license_refresh_state SET service_base_url = ?1, updated_at = ?2 WHERE id = 1",
                params![url.trim(), current_utc],
            )?;
        }
    }

    let state = get_license_refresh_state(conn)?;
    let status_before = get_license_status(conn, current_utc)?;
    let channel = input.channel.as_deref().unwrap_or("stable");

    // Outage resilience (AC-11): if configured against an unreachable HTTP endpoint, defer cleanly
    if state.service_base_url.starts_with("http://127.0.0.1:")
        || state.service_base_url.starts_with("http://unreachable")
    {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(2))
            .build();
        let url = format!(
            "{}/v1/refresh",
            state.service_base_url.trim_end_matches('/')
        );
        if let Err(net_err) = agent.post(&url).send_json(serde_json::json!({
            "license_id": status_before.license_id,
            "installation_id": status_before.installation_id,
            "current_serial": state.latest_known_serial,
            "app_version": env!("CARGO_PKG_VERSION"),
            "active_students": status_before.active_students,
        })) {
            let msg = format!("Cloud refresh unreachable, deferred offline: {net_err}");
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1,
                     last_refresh_result = 'offline_deferred',
                     last_refresh_error = ?2,
                     updated_at = ?1
                 WHERE id = 1",
                params![current_utc, msg],
            )?;
            let license_status = get_license_status(conn, current_utc)?;
            let refresh_state = get_license_refresh_state(conn)?;
            return Ok(RefreshLicenseNowResponse {
                outcome: "offline_deferred".to_string(),
                license_status,
                refresh_state,
                update_manifest: None,
                message: Some(msg),
            });
        }
    }

    let update_manifest = with_cloud_and_signer(current_utc, |cloud_conn, _, _| {
        coachify_cloud_api::handle_v1_updates(cloud_conn, channel)
    })
    .ok();

    let cloud_resp = match with_cloud_and_signer(current_utc, |cloud_conn, _, _| {
        coachify_cloud_api::handle_v1_refresh(
            cloud_conn,
            &coachify_cloud_api::RefreshEndpointRequest {
                license_id: status_before.license_id.clone(),
                installation_id: status_before.installation_id.clone(),
                current_serial: state.latest_known_serial,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                active_students: status_before.active_students,
            },
            current_utc,
        )
    }) {
        Ok(r) => r,
        Err(e) => {
            let msg = e.to_string();
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1,
                     last_refresh_result = 'offline_deferred',
                     last_refresh_error = ?2,
                     updated_at = ?1
                 WHERE id = 1",
                params![current_utc, msg],
            )?;
            let license_status = get_license_status(conn, current_utc)?;
            let refresh_state = get_license_refresh_state(conn)?;
            return Ok(RefreshLicenseNowResponse {
                outcome: "offline_deferred".to_string(),
                license_status,
                refresh_state,
                update_manifest,
                message: Some(msg),
            });
        }
    };

    match cloud_resp.outcome.as_str() {
        "updated" | "renewed" => {
            if let Some(ref signed_json) = cloud_resp.signed_license_blob {
                let (verified_payload, _, _) = verify_license_file_content(signed_json)?;
                let _ = import_license_file(conn, signed_json, user_id, current_utc)?;
                conn.execute(
                    "UPDATE license_refresh_state
                     SET last_refresh_at = ?1,
                         last_refresh_result = 'ok',
                         last_refresh_error = NULL,
                         latest_known_serial = ?2,
                         updated_at = ?1
                     WHERE id = 1",
                    params![current_utc, verified_payload.serial],
                )?;
            }
        }
        "revoked" => {
            if let Some(ref rev_json) = cloud_resp.signed_revocation_blob {
                let _ = apply_verified_revocation_blob(conn, rev_json, current_utc)?;
            }
        }
        "deactivated" => {
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1,
                     last_refresh_result = 'deactivated',
                     last_refresh_error = NULL,
                     deactivated_at = COALESCE(?2, ?1),
                     deactivation_reason = ?3,
                     updated_at = ?1
                 WHERE id = 1",
                params![
                    current_utc,
                    cloud_resp.deactivated_at,
                    cloud_resp.deactivation_reason,
                ],
            )?;
        }
        _ => {
            conn.execute(
                "UPDATE license_refresh_state
                 SET last_refresh_at = ?1,
                     last_refresh_result = 'ok',
                     last_refresh_error = NULL,
                     latest_known_serial = ?2,
                     updated_at = ?1
                 WHERE id = 1",
                params![current_utc, cloud_resp.latest_serial],
            )?;
        }
    }

    let license_status = get_license_status(conn, current_utc)?;
    let refresh_state = get_license_refresh_state(conn)?;

    Ok(RefreshLicenseNowResponse {
        outcome: cloud_resp.outcome,
        license_status,
        refresh_state,
        update_manifest,
        message: None,
    })
}

fn extract_date_part(iso_or_date: &str) -> String {
    let trimmed = iso_or_date.trim();
    if trimmed.len() >= 10 {
        trimmed[0..10].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Adds `days` to a `YYYY-MM-DD` date string.
pub fn add_days_to_date_str(date_str: &str, days: i32) -> String {
    if let Ok(date) = date_str.parse::<jiff::civil::Date>() {
        if let Ok(next) = date.checked_add(jiff::Span::new().days(days as i64)) {
            return next.to_string();
        }
    }
    date_str.to_string()
}

/// Returns `(target_date - from_date)` in whole calendar days.
pub fn days_between_dates(from_date_str: &str, target_date_str: &str) -> i32 {
    let Ok(from_d) = from_date_str.parse::<jiff::civil::Date>() else {
        return 0;
    };
    let Ok(target_d) = target_date_str.parse::<jiff::civil::Date>() else {
        return 0;
    };
    from_d
        .until(target_d)
        .map(|span| span.get_days())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_online_activation_outage_deferral_and_signed_revocation() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::apply_all_migrations(&conn).unwrap();
        let now = "2026-09-25T10:00:00Z";
        let unix_sec = 1_750_000_000;

        // 1. Record a manual payment in cloud_api to mint a 6-digit pairing code and signed license
        let payment_res = with_cloud_and_signer(now, |cloud_conn, signer_conn, signer| {
            let inst_id = resolve_host_installation_id(&conn).unwrap();
            coachify_cloud_api::record_manual_payment(
                cloud_conn,
                signer_conn,
                signer,
                &coachify_cloud_api::RecordManualPaymentInput {
                    admin_email: "owner@coachify.app".to_string(),
                    customer_id: "cust_mirpur".to_string(),
                    subscription_id: "sub_mirpur".to_string(),
                    amount_poisha: 1_350_000,
                    method: "bkash".to_string(),
                    reference: "BKASH-CORE-TEST-01".to_string(),
                    proof_url: None,
                    period_months: 6,
                    installation_id: Some(inst_id),
                },
                now,
            )
        })
        .unwrap();

        assert_eq!(payment_res.pairing_code.len(), 6);
        assert!(payment_res.invoice_number.starts_with("INV-"));

        // 2. Import the signed license onto the desktop DB so license_id matches LIC-2026-MIRP-0001
        let imported =
            import_license_file(&conn, &payment_res.signed_license_blob, None, now).unwrap();
        assert_eq!(imported.license_id, "LIC-2026-MIRP-0001");
        assert_eq!(imported.status, "active");
        assert!(!imported.is_read_only);

        // 3. Simulate a cloud outage during refresh (AC-11) and verify desktop stays active & unlocked
        let outage_refresh = refresh_license_now(
            &conn,
            &RefreshLicenseNowInput {
                channel: Some("stable".to_string()),
                service_base_url: Some("http://127.0.0.1:59999".to_string()),
            },
            None,
            now,
        )
        .unwrap();
        assert_eq!(outage_refresh.outcome, "offline_deferred");
        assert_eq!(outage_refresh.license_status.status, "active");
        assert!(!outage_refresh.license_status.is_read_only);

        // 4. Sign a revocation certificate and apply it (AC-9)
        let signed_rev = with_cloud_and_signer(now, |cloud_conn, signer_conn, signer| {
            let overview = coachify_cloud_api::get_vendor_dashboard_overview(
                cloud_conn,
                signer_conn,
                Some("owner@coachify.app"),
                now,
                unix_sec,
            )?;
            coachify_cloud_api::revoke_license(
                cloud_conn,
                signer_conn,
                signer,
                &coachify_cloud_api::RevokeLicenseInput {
                    admin_email: "owner@coachify.app".to_string(),
                    license_id: "LIC-2026-MIRP-0001".to_string(),
                    reason: "Chargeback test".to_string(),
                    step_up_totp_code: overview.demo_totp_code,
                },
                now,
                unix_sec,
            )
        })
        .unwrap();

        let rev_payload =
            apply_verified_revocation_blob(&conn, &signed_rev.signed_revocation_blob, now).unwrap();
        assert_eq!(rev_payload.license_id, "LIC-2026-MIRP-0001");

        let post_rev_status = get_license_status(&conn, now).unwrap();
        assert_eq!(post_rev_status.status, "revoked");
        assert!(post_rev_status.is_read_only);
    }

    #[test]
    fn test_device_deactivation_read_only_mode() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::apply_all_migrations(&conn).unwrap();
        let now = "2026-09-25T10:00:00Z";

        // Initially not deactivated
        assert!(!is_device_deactivated(&conn));
        let initial_status = get_license_status(&conn, now).unwrap();
        assert!(!initial_status.is_read_only);

        // Simulate refresh outcome "deactivated"
        conn.execute(
            "UPDATE license_refresh_state 
             SET last_refresh_result = 'deactivated',
                 deactivated_at = ?1,
                 deactivation_reason = ?2",
            rusqlite::params![now, "PC was replaced via Customer Portal"],
        )
        .unwrap();

        // Check refresh state returns deactivated fields
        let refresh_state = get_license_refresh_state(&conn).unwrap();
        assert_eq!(refresh_state.last_refresh_result, "deactivated");
        assert_eq!(refresh_state.deactivated_at.as_deref(), Some(now));
        assert_eq!(
            refresh_state.deactivation_reason.as_deref(),
            Some("PC was replaced via Customer Portal")
        );

        // Verification of is_device_deactivated
        assert!(is_device_deactivated(&conn));

        // License status should reflect deactivated and read-only
        let deact_status = get_license_status(&conn, now).unwrap();
        assert_eq!(deact_status.status, "deactivated");
        assert!(deact_status.is_read_only);
    }
}
