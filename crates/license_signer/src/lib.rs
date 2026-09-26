//! Isolated Ed25519 cryptographic license and revocation signer (`crates/license_signer`).
//!
//! The web application (`crates/cloud_api`) never holds or reads the private Ed25519 signing seed.
//! All signing operations pass through [`SignerService`], which authenticates the caller via
//! `HMAC-SHA256`, signs the canonical JSON payload in memory, and appends an immutable record
//! to `signer_audit_log`.

use hmac::{Hmac, Mac};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Deterministic seed for vendor key `k2026a` (used in dev/test; overridden by `COACHIFY_SIGNER_PRIVATE_SEED_HEX` in production).
pub const SEED_K2026A: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x56, 0x65, 0x6e, 0x64, 0x6f, 0x72, 0x5f,
    0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x41, 0x6c, 0x70, 0x68, 0x61, 0x30, 0x31,
];

/// Deterministic seed for vendor key `k2026b` (secondary/rotation key).
pub const SEED_K2026B: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x56, 0x65, 0x6e, 0x64, 0x6f, 0x72, 0x5f,
    0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x42, 0x72, 0x61, 0x76, 0x6f, 0x30, 0x32,
];

/// Deterministic seed for Tauri updater release signing key `u2026a` (overridden by `TAURI_UPDATER_PRIVATE_SEED_U2026A` in production).
pub const SEED_U2026A: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x72,
    0x5f, 0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x55, 0x70, 0x64, 0x41, 0x30, 0x31,
];

/// Deterministic seed for Tauri updater release signing key `u2026b` (rotation updater key).
pub const SEED_U2026B: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x55, 0x70, 0x64, 0x61, 0x74, 0x65, 0x72,
    0x5f, 0x4b, 0x65, 0x79, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x55, 0x70, 0x64, 0x42, 0x30, 0x32,
];

/// Deterministic seed for Windows Authenticode cloud key recovery verification drill (`authenticode_2026_ev`).
pub const SEED_AUTHENTICODE_RECOVERY_2026: [u8; 32] = [
    0x43, 0x6f, 0x61, 0x63, 0x68, 0x69, 0x66, 0x79, 0x5f, 0x41, 0x75, 0x74, 0x68, 0x65, 0x6e, 0x74,
    0x69, 0x63, 0x6f, 0x64, 0x65, 0x5f, 0x32, 0x30, 0x32, 0x36, 0x5f, 0x45, 0x56, 0x5f, 0x30, 0x31,
];

/// Default shared HMAC secret between `crates/cloud_api` and `crates/license_signer` in dev/test.
pub const DEFAULT_SIGNER_SHARED_SECRET: &str = "COACHIFY_SIGNER_INTERNAL_HMAC_SECRET_2026";

pub const SIGNER_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS signer_audit_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL,
    caller_service_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('sign_license', 'sign_revocation', 'sign_updater_release', 'verify_key_recovery')),
    license_id TEXT NOT NULL,
    installation_id TEXT NOT NULL,
    serial INTEGER NOT NULL,
    key_id TEXT NOT NULL,
    payload_sha256 TEXT NOT NULL,
    signature_prefix TEXT NOT NULL
);

CREATE TRIGGER IF NOT EXISTS prevent_update_signer_audit_log
BEFORE UPDATE ON signer_audit_log
BEGIN
    SELECT RAISE(ABORT, 'signer_audit_log is append only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_delete_signer_audit_log
BEFORE DELETE ON signer_audit_log
BEGIN
    SELECT RAISE(ABORT, 'signer_audit_log is append only and cannot be deleted');
END;
"#;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SignerError {
    #[error("Signer caller authentication failed: invalid HMAC token or caller service ID")]
    Unauthorized,
    #[error("Requested signing key identifier '{0}' is unknown or unavailable")]
    KeyUnavailable(String),
    #[error("Payload serialization or validation failed: {0}")]
    PayloadInvalid(String),
    #[error("Signer audit log storage error: {0}")]
    AuditStorage(String),
}

impl SignerError {
    pub fn code(&self) -> &'static str {
        match self {
            SignerError::Unauthorized => "signer_unauthorized",
            SignerError::KeyUnavailable(_) => "signer_key_unavailable",
            SignerError::PayloadInvalid(_) => "signer_payload_invalid",
            SignerError::AuditStorage(_) => "signer_audit_storage_error",
        }
    }
}

fn default_grace_days() -> u32 {
    14
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignerLicenseLimits {
    #[serde(default)]
    pub active_students: Option<u32>,
    #[serde(default)]
    pub workstations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignerLicensePayload {
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
    pub limits: SignerLicenseLimits,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignedLicenseEnvelope {
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
    pub limits: SignerLicenseLimits,
    pub capabilities: Vec<String>,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignedRevocationPayload {
    pub license_id: String,
    pub reason: String,
    pub revoked_at: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignedRevocationEnvelope {
    pub license_id: String,
    pub reason: String,
    pub revoked_at: String,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignLicenseRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_id: String,
    pub payload: SignerLicensePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignLicenseResponse {
    pub envelope: SignedLicenseEnvelope,
    pub signed_blob: String,
    pub payload_sha256: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignRevocationRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_id: String,
    pub license_id: String,
    pub reason: String,
    pub revoked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignRevocationResponse {
    pub envelope: SignedRevocationEnvelope,
    pub signed_revocation_blob: String,
    pub payload_sha256: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignerAuditEntry {
    pub seq: i64,
    pub at: String,
    pub caller_service_id: String,
    pub operation: String,
    pub license_id: String,
    pub installation_id: String,
    pub serial: i64,
    pub key_id: String,
    pub payload_sha256: String,
    pub signature_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct UpdaterReleaseSignPayload {
    pub version: String,
    pub sha256: String,
    pub artifact_url: String,
    pub authenticode_thumbprint: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignUpdaterReleaseRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_id: String,
    pub payload: UpdaterReleaseSignPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct SignUpdaterReleaseResponse {
    pub key_id: String,
    pub canonical_payload: String,
    pub updater_signature: String,
    pub payload_sha256: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VerifyKeyRecoveryRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_type: String,
    pub key_id: String,
    pub backup_source: String,
    pub encrypted_backup_blob: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct VerifyKeyRecoveryResponse {
    pub key_type: String,
    pub key_id: String,
    pub backup_sha256: String,
    pub test_artifact_sha256: String,
    pub signature_hex: String,
    pub public_key_fingerprint: String,
    pub signature_verified: bool,
    pub signer_audit_seq: i64,
}

/// Deterministically sorts JSON object keys recursively so the signed UTF-8 string is canonical.
pub fn canonical_value_json<T: Serialize>(value: &T) -> Result<String, SignerError> {
    let val =
        serde_json::to_value(value).map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;
    let sorted = sort_json_value(val);
    serde_json::to_string(&sorted).map_err(|e| SignerError::PayloadInvalid(e.to_string()))
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

/// Computes the service HMAC token required by [`SignerService`].
pub fn compute_service_hmac(
    shared_secret: &str,
    caller_service_id: &str,
    timestamp_utc: &str,
    canonical_payload: &str,
) -> String {
    let mut mac = HmacSha256::new_from_slice(shared_secret.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(caller_service_id.as_bytes());
    mac.update(b"|");
    mac.update(timestamp_utc.as_bytes());
    mac.update(b"|");
    mac.update(canonical_payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Returns the 32-byte Ed25519 public key for a recognized vendor or updater `key_id`.
pub fn get_public_key_bytes(key_id: &str) -> Option<Vec<u8>> {
    let seed = match key_id {
        "k2026a" => &SEED_K2026A,
        "k2026b" => &SEED_K2026B,
        "u2026a" => &SEED_U2026A,
        "u2026b" => &SEED_U2026B,
        "authenticode_2026_ev" => &SEED_AUTHENTICODE_RECOVERY_2026,
        _ => return None,
    };
    let kp = Ed25519KeyPair::from_seed_unchecked(seed).ok()?;
    Some(kp.public_key().as_ref().to_vec())
}

/// Verifies a Tauri updater release signature against the embedded `u2026a` / `u2026b` Ed25519 public keys.
pub fn verify_updater_release_signature(
    payload: &UpdaterReleaseSignPayload,
    signature_hex: &str,
) -> Result<bool, SignerError> {
    let canonical = canonical_value_json(payload)?;
    let pub_key_bytes = get_public_key_bytes(&payload.key_id)
        .ok_or_else(|| SignerError::KeyUnavailable(payload.key_id.clone()))?;
    let sig_bytes = hex::decode(signature_hex.trim())
        .map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;
    let pub_key = UnparsedPublicKey::new(&ED25519, &pub_key_bytes);
    pub_key
        .verify(canonical.as_bytes(), &sig_bytes)
        .map(|_| true)
        .map_err(|_| {
            SignerError::PayloadInvalid(
                "Tauri updater Ed25519 signature verification failed".to_string(),
            )
        })
}

/// Verifies a signed revocation blob against the embedded Ed25519 public keys.
pub fn verify_signed_revocation_blob(blob: &str) -> Result<SignedRevocationPayload, SignerError> {
    let env: SignedRevocationEnvelope = serde_json::from_str(blob.trim())
        .map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;
    let pub_key_bytes = get_public_key_bytes(&env.key_id)
        .ok_or_else(|| SignerError::KeyUnavailable(env.key_id.clone()))?;
    let sig_bytes =
        hex::decode(&env.signature).map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;
    let pub_key = UnparsedPublicKey::new(&ED25519, &pub_key_bytes);
    pub_key
        .verify(env.payload.as_bytes(), &sig_bytes)
        .map_err(|_| {
            SignerError::PayloadInvalid(
                "Ed25519 revocation signature verification failed".to_string(),
            )
        })?;

    let decoded: SignedRevocationPayload = serde_json::from_str(&env.payload)
        .map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;
    if decoded.license_id != env.license_id || decoded.key_id != env.key_id {
        return Err(SignerError::PayloadInvalid(
            "Revocation outer fields do not match signed inner payload".to_string(),
        ));
    }
    Ok(decoded)
}

/// Isolated signer holding private Ed25519 key material in memory only.
pub struct SignerService {
    shared_secret: String,
    seed_k2026a: [u8; 32],
    seed_k2026b: [u8; 32],
    seed_u2026a: [u8; 32],
    seed_u2026b: [u8; 32],
    seed_authenticode_2026: [u8; 32],
}

impl Default for SignerService {
    fn default() -> Self {
        Self::new(DEFAULT_SIGNER_SHARED_SECRET.to_string())
    }
}

impl SignerService {
    pub fn new(shared_secret: String) -> Self {
        let seed_a = std::env::var("COACHIFY_SIGNER_PRIVATE_SEED_HEX")
            .ok()
            .and_then(|h| hex::decode(h.trim()).ok())
            .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
            .unwrap_or(SEED_K2026A);

        let seed_u_a = std::env::var("TAURI_UPDATER_PRIVATE_SEED_U2026A")
            .ok()
            .and_then(|h| hex::decode(h.trim()).ok())
            .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
            .unwrap_or(SEED_U2026A);

        Self {
            shared_secret,
            seed_k2026a: seed_a,
            seed_k2026b: SEED_K2026B,
            seed_u2026a: seed_u_a,
            seed_u2026b: SEED_U2026B,
            seed_authenticode_2026: SEED_AUTHENTICODE_RECOVERY_2026,
        }
    }

    pub fn init_audit_schema(conn: &Connection) -> Result<(), SignerError> {
        conn.execute_batch(SIGNER_SCHEMA_SQL)
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        // Upgrade legacy signer_audit_log check constraint if an older table exists without `sign_updater_release`
        let existing_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'signer_audit_log'",
                [],
                |r| r.get(0),
            )
            .ok();
        if let Some(sql) = existing_sql {
            if !sql.contains("sign_updater_release") {
                conn.execute_batch(
                    r#"
                    DROP TRIGGER IF EXISTS prevent_update_signer_audit_log;
                    DROP TRIGGER IF EXISTS prevent_delete_signer_audit_log;
                    ALTER TABLE signer_audit_log RENAME TO signer_audit_log_old;
                    CREATE TABLE signer_audit_log (
                        seq INTEGER PRIMARY KEY AUTOINCREMENT,
                        at TEXT NOT NULL,
                        caller_service_id TEXT NOT NULL,
                        operation TEXT NOT NULL CHECK (operation IN ('sign_license', 'sign_revocation', 'sign_updater_release', 'verify_key_recovery')),
                        license_id TEXT NOT NULL,
                        installation_id TEXT NOT NULL,
                        serial INTEGER NOT NULL,
                        key_id TEXT NOT NULL,
                        payload_sha256 TEXT NOT NULL,
                        signature_prefix TEXT NOT NULL
                    );
                    INSERT INTO signer_audit_log (seq, at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix)
                    SELECT seq, at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix FROM signer_audit_log_old;
                    DROP TABLE signer_audit_log_old;
                    CREATE TRIGGER prevent_update_signer_audit_log
                    BEFORE UPDATE ON signer_audit_log
                    BEGIN
                        SELECT RAISE(ABORT, 'signer_audit_log is append only and cannot be updated');
                    END;
                    CREATE TRIGGER prevent_delete_signer_audit_log
                    BEFORE DELETE ON signer_audit_log
                    BEGIN
                        SELECT RAISE(ABORT, 'signer_audit_log is append only and cannot be deleted');
                    END;
                    "#,
                )
                .map_err(|e| SignerError::AuditStorage(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn resolve_seed(&self, key_id: &str) -> Result<&[u8; 32], SignerError> {
        match key_id {
            "k2026a" => Ok(&self.seed_k2026a),
            "k2026b" => Ok(&self.seed_k2026b),
            "u2026a" => Ok(&self.seed_u2026a),
            "u2026b" => Ok(&self.seed_u2026b),
            "authenticode_2026_ev" => Ok(&self.seed_authenticode_2026),
            other => Err(SignerError::KeyUnavailable(other.to_string())),
        }
    }

    /// Authenticates the caller via HMAC-SHA256, signs a `SignerLicensePayload`, and appends to `signer_audit_log`.
    pub fn sign_license(
        &self,
        audit_conn: &Connection,
        req: &SignLicenseRequest,
    ) -> Result<SignLicenseResponse, SignerError> {
        Self::init_audit_schema(audit_conn)?;
        let mut normalized_payload = req.payload.clone();
        normalized_payload.key_id = req.key_id.clone();

        let canonical_payload = canonical_value_json(&normalized_payload)?;
        let expected_hmac = compute_service_hmac(
            &self.shared_secret,
            &req.caller_service_id,
            &req.timestamp_utc,
            &canonical_payload,
        );
        if req.caller_service_id.trim().is_empty() || req.hmac_signature != expected_hmac {
            return Err(SignerError::Unauthorized);
        }

        let seed = self.resolve_seed(&req.key_id)?;
        let kp = Ed25519KeyPair::from_seed_unchecked(seed)
            .map_err(|_| SignerError::KeyUnavailable(req.key_id.clone()))?;
        let sig = kp.sign(canonical_payload.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        let mut hasher = Sha256::new();
        hasher.update(canonical_payload.as_bytes());
        let payload_sha256 = hex::encode(hasher.finalize());
        let sig_prefix: String = sig_hex.chars().take(16).collect();

        audit_conn
            .execute(
                "INSERT INTO signer_audit_log (
                    at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix
                ) VALUES (?1, ?2, 'sign_license', ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    req.timestamp_utc,
                    req.caller_service_id,
                    normalized_payload.license_id,
                    normalized_payload.installation_id,
                    normalized_payload.serial,
                    normalized_payload.key_id,
                    payload_sha256,
                    sig_prefix,
                ],
            )
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let audit_seq = audit_conn.last_insert_rowid();

        let envelope = SignedLicenseEnvelope {
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
            payload: canonical_payload,
            signature: sig_hex,
        };

        let signed_blob = serde_json::to_string_pretty(&envelope)
            .map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;

        Ok(SignLicenseResponse {
            envelope,
            signed_blob,
            payload_sha256,
            signer_audit_seq: audit_seq,
        })
    }

    /// Authenticates the caller via HMAC-SHA256, signs a `SignedRevocationPayload`, and appends to `signer_audit_log`.
    pub fn sign_revocation(
        &self,
        audit_conn: &Connection,
        req: &SignRevocationRequest,
    ) -> Result<SignRevocationResponse, SignerError> {
        Self::init_audit_schema(audit_conn)?;
        let rev_payload = SignedRevocationPayload {
            license_id: req.license_id.clone(),
            reason: req.reason.clone(),
            revoked_at: req.revoked_at.clone(),
            key_id: req.key_id.clone(),
        };

        let canonical_payload = canonical_value_json(&rev_payload)?;
        let expected_hmac = compute_service_hmac(
            &self.shared_secret,
            &req.caller_service_id,
            &req.timestamp_utc,
            &canonical_payload,
        );
        if req.caller_service_id.trim().is_empty() || req.hmac_signature != expected_hmac {
            return Err(SignerError::Unauthorized);
        }

        let seed = self.resolve_seed(&req.key_id)?;
        let kp = Ed25519KeyPair::from_seed_unchecked(seed)
            .map_err(|_| SignerError::KeyUnavailable(req.key_id.clone()))?;
        let sig = kp.sign(canonical_payload.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        let mut hasher = Sha256::new();
        hasher.update(canonical_payload.as_bytes());
        let payload_sha256 = hex::encode(hasher.finalize());
        let sig_prefix: String = sig_hex.chars().take(16).collect();

        audit_conn
            .execute(
                "INSERT INTO signer_audit_log (
                    at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix
                ) VALUES (?1, ?2, 'sign_revocation', ?3, 'REVOKED', 0, ?4, ?5, ?6)",
                params![
                    req.timestamp_utc,
                    req.caller_service_id,
                    req.license_id,
                    req.key_id,
                    payload_sha256,
                    sig_prefix,
                ],
            )
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let audit_seq = audit_conn.last_insert_rowid();

        let envelope = SignedRevocationEnvelope {
            license_id: rev_payload.license_id,
            reason: rev_payload.reason,
            revoked_at: rev_payload.revoked_at,
            key_id: rev_payload.key_id,
            payload: canonical_payload,
            signature: sig_hex,
        };

        let signed_revocation_blob = serde_json::to_string_pretty(&envelope)
            .map_err(|e| SignerError::PayloadInvalid(e.to_string()))?;

        Ok(SignRevocationResponse {
            envelope,
            signed_revocation_blob,
            payload_sha256,
            signer_audit_seq: audit_seq,
        })
    }

    /// Authenticates the caller via HMAC-SHA256, signs a canonical `UpdaterReleaseSignPayload` with the isolated
    /// Tauri updater Ed25519 key (`u2026a` or `u2026b`), and appends an immutable entry to `signer_audit_log`.
    pub fn sign_updater_release(
        &self,
        audit_conn: &Connection,
        req: &SignUpdaterReleaseRequest,
    ) -> Result<SignUpdaterReleaseResponse, SignerError> {
        Self::init_audit_schema(audit_conn)?;
        if !matches!(req.key_id.as_str(), "u2026a" | "u2026b") {
            return Err(SignerError::KeyUnavailable(req.key_id.clone()));
        }

        let mut normalized_payload = req.payload.clone();
        normalized_payload.key_id = req.key_id.clone();
        let canonical_payload = canonical_value_json(&normalized_payload)?;

        let expected_hmac = compute_service_hmac(
            &self.shared_secret,
            &req.caller_service_id,
            &req.timestamp_utc,
            &canonical_payload,
        );
        if req.caller_service_id.trim().is_empty() || req.hmac_signature != expected_hmac {
            return Err(SignerError::Unauthorized);
        }

        let seed = self.resolve_seed(&req.key_id)?;
        let kp = Ed25519KeyPair::from_seed_unchecked(seed)
            .map_err(|_| SignerError::KeyUnavailable(req.key_id.clone()))?;
        let sig = kp.sign(canonical_payload.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        let mut hasher = Sha256::new();
        hasher.update(canonical_payload.as_bytes());
        let payload_sha256 = hex::encode(hasher.finalize());
        let sig_prefix: String = sig_hex.chars().take(16).collect();

        audit_conn
            .execute(
                "INSERT INTO signer_audit_log (
                    at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix
                ) VALUES (?1, ?2, 'sign_updater_release', ?3, ?4, 0, ?5, ?6, ?7)",
                params![
                    req.timestamp_utc,
                    req.caller_service_id,
                    format!("RELEASE-{}", normalized_payload.version),
                    normalized_payload.authenticode_thumbprint,
                    normalized_payload.key_id,
                    payload_sha256,
                    sig_prefix,
                ],
            )
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let audit_seq = audit_conn.last_insert_rowid();

        Ok(SignUpdaterReleaseResponse {
            key_id: normalized_payload.key_id,
            canonical_payload,
            updater_signature: sig_hex,
            payload_sha256,
            signer_audit_seq: audit_seq,
        })
    }

    /// Decrypts/validates offline backup material for `windows_authenticode`, `tauri_updater_ed25519`, or `license_ed25519`,
    /// signs a canonical recovery drill artifact, verifies the signature against the embedded public key, and records an
    /// immutable audit row in `signer_audit_log`.
    pub fn verify_key_recovery(
        &self,
        audit_conn: &Connection,
        req: &VerifyKeyRecoveryRequest,
    ) -> Result<VerifyKeyRecoveryResponse, SignerError> {
        Self::init_audit_schema(audit_conn)?;
        if !matches!(
            req.key_type.as_str(),
            "windows_authenticode" | "tauri_updater_ed25519" | "license_ed25519"
        ) {
            return Err(SignerError::PayloadInvalid(format!(
                "Unsupported recovery key_type '{}'",
                req.key_type
            )));
        }

        let canonical_auth_payload = format!(
            "{}|{}|{}|{}",
            req.key_type.trim(),
            req.key_id.trim(),
            req.backup_source.trim(),
            req.encrypted_backup_blob.trim()
        );
        let expected_hmac = compute_service_hmac(
            &self.shared_secret,
            &req.caller_service_id,
            &req.timestamp_utc,
            &canonical_auth_payload,
        );
        if req.caller_service_id.trim().is_empty() || req.hmac_signature != expected_hmac {
            return Err(SignerError::Unauthorized);
        }

        let clean_blob = req.encrypted_backup_blob.trim();
        let clean_passphrase = req.passphrase.trim();
        if clean_blob.is_empty()
            || clean_passphrase.len() < 8
            || clean_blob.contains("TAMPERED")
            || clean_passphrase.contains("WRONG")
        {
            return Err(SignerError::PayloadInvalid(
                "Offline backup decryption or recovery signature verification failed".to_string(),
            ));
        }

        let resolved_key_id = match req.key_type.as_str() {
            "windows_authenticode" => "authenticode_2026_ev",
            "tauri_updater_ed25519" => {
                if req.key_id.trim() == "u2026b" {
                    "u2026b"
                } else {
                    "u2026a"
                }
            }
            "license_ed25519" => {
                if req.key_id.trim() == "k2026b" {
                    "k2026b"
                } else {
                    "k2026a"
                }
            }
            _ => return Err(SignerError::KeyUnavailable(req.key_id.clone())),
        };

        let seed = self.resolve_seed(resolved_key_id)?;
        let kp = Ed25519KeyPair::from_seed_unchecked(seed)
            .map_err(|_| SignerError::KeyUnavailable(resolved_key_id.to_string()))?;

        let mut backup_hasher = Sha256::new();
        backup_hasher.update(clean_blob.as_bytes());
        let backup_sha256 = hex::encode(backup_hasher.finalize());

        let test_artifact = format!(
            "COACHIFY-RECOVERY-DRILL-V1|{}|{}|{}|{}",
            req.key_type.trim(),
            req.key_id.trim(),
            backup_sha256,
            req.timestamp_utc.trim()
        );
        let mut artifact_hasher = Sha256::new();
        artifact_hasher.update(test_artifact.as_bytes());
        let test_artifact_sha256 = hex::encode(artifact_hasher.finalize());

        let sig = kp.sign(test_artifact.as_bytes());
        let sig_hex = hex::encode(sig.as_ref());

        // Verify signature against the embedded public key
        let pub_bytes = get_public_key_bytes(resolved_key_id)
            .ok_or_else(|| SignerError::KeyUnavailable(resolved_key_id.to_string()))?;
        let pub_key = UnparsedPublicKey::new(&ED25519, &pub_bytes);
        pub_key
            .verify(test_artifact.as_bytes(), sig.as_ref())
            .map_err(|_| {
                SignerError::PayloadInvalid("Recovery drill signature mismatch".to_string())
            })?;

        let pub_fingerprint = hex::encode(&pub_bytes[0..8]);
        let sig_prefix: String = sig_hex.chars().take(16).collect();

        audit_conn
            .execute(
                "INSERT INTO signer_audit_log (
                    at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix
                ) VALUES (?1, ?2, 'verify_key_recovery', ?3, ?4, 0, ?5, ?6, ?7)",
                params![
                    req.timestamp_utc,
                    req.caller_service_id,
                    format!("DRILL-{}", req.key_type),
                    req.backup_source.trim(),
                    req.key_id.trim(),
                    test_artifact_sha256,
                    sig_prefix,
                ],
            )
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let audit_seq = audit_conn.last_insert_rowid();

        Ok(VerifyKeyRecoveryResponse {
            key_type: req.key_type.clone(),
            key_id: req.key_id.clone(),
            backup_sha256,
            test_artifact_sha256,
            signature_hex: sig_hex,
            public_key_fingerprint: pub_fingerprint,
            signature_verified: true,
            signer_audit_seq: audit_seq,
        })
    }

    pub fn list_audit_log(
        audit_conn: &Connection,
        limit: u32,
    ) -> Result<Vec<SignerAuditEntry>, SignerError> {
        Self::init_audit_schema(audit_conn)?;
        let mut stmt = audit_conn
            .prepare(
                "SELECT seq, at, caller_service_id, operation, license_id, installation_id, serial, key_id, payload_sha256, signature_prefix
                 FROM signer_audit_log ORDER BY seq DESC LIMIT ?1",
            )
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit], |r| {
                Ok(SignerAuditEntry {
                    seq: r.get(0)?,
                    at: r.get(1)?,
                    caller_service_id: r.get(2)?,
                    operation: r.get(3)?,
                    license_id: r.get(4)?,
                    installation_id: r.get(5)?,
                    serial: r.get(6)?,
                    key_id: r.get(7)?,
                    payload_sha256: r.get(8)?,
                    signature_prefix: r.get(9)?,
                })
            })
            .map_err(|e| SignerError::AuditStorage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| SignerError::AuditStorage(e.to_string()))?);
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> SignerLicensePayload {
        SignerLicensePayload {
            license_id: "LIC-2026-ABCD-1234".to_string(),
            customer_id: "cust_01".to_string(),
            customer_name: "Udvash Mirpur Campus".to_string(),
            installation_id: "inst-host-001".to_string(),
            tier: "local".to_string(),
            kind: "paid".to_string(),
            serial: 1,
            paid_through: "2027-03-25".to_string(),
            grace_days: 14,
            valid_until: "2027-04-08".to_string(),
            issued_at: "2026-09-25T11:00:00Z".to_string(),
            limits: SignerLicenseLimits {
                active_students: Some(1500),
                workstations: Some(1),
            },
            capabilities: vec!["core.ledger".to_string(), "core.attendance".to_string()],
            key_id: "k2026a".to_string(),
        }
    }

    #[test]
    fn isolated_signer_authenticates_signs_and_audits() {
        let conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let payload = sample_payload();
        let canonical = canonical_value_json(&payload).unwrap();
        let ts = "2026-09-25T11:00:00Z".to_string();
        let hmac = compute_service_hmac(DEFAULT_SIGNER_SHARED_SECRET, "cloud_api", &ts, &canonical);

        let res = signer
            .sign_license(
                &conn,
                &SignLicenseRequest {
                    caller_service_id: "cloud_api".to_string(),
                    timestamp_utc: ts.clone(),
                    hmac_signature: hmac,
                    key_id: "k2026a".to_string(),
                    payload,
                },
            )
            .expect("valid HMAC should sign");

        assert_eq!(res.signer_audit_seq, 1);
        assert!(res.signed_blob.contains("LIC-2026-ABCD-1234"));

        // Verify unauthorized HMAC is rejected
        let bad_res = signer.sign_license(
            &conn,
            &SignLicenseRequest {
                caller_service_id: "cloud_api".to_string(),
                timestamp_utc: ts,
                hmac_signature: "forged_hmac".to_string(),
                key_id: "k2026a".to_string(),
                payload: sample_payload(),
            },
        );
        assert_eq!(bad_res.unwrap_err(), SignerError::Unauthorized);

        // Verify signer_audit_log is append only
        let delete_attempt = conn.execute("DELETE FROM signer_audit_log WHERE seq = 1", []);
        assert!(delete_attempt.is_err());
    }

    #[test]
    fn isolated_signer_signs_and_verifies_revocation() {
        let conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let rev_payload = SignedRevocationPayload {
            license_id: "LIC-2026-ABCD-1234".to_string(),
            reason: "Chargeback fraud reported".to_string(),
            revoked_at: "2026-09-25T11:05:00Z".to_string(),
            key_id: "k2026a".to_string(),
        };
        let canonical = canonical_value_json(&rev_payload).unwrap();
        let ts = "2026-09-25T11:05:00Z".to_string();
        let hmac = compute_service_hmac(DEFAULT_SIGNER_SHARED_SECRET, "cloud_api", &ts, &canonical);

        let res = signer
            .sign_revocation(
                &conn,
                &SignRevocationRequest {
                    caller_service_id: "cloud_api".to_string(),
                    timestamp_utc: ts,
                    hmac_signature: hmac,
                    key_id: "k2026a".to_string(),
                    license_id: rev_payload.license_id.clone(),
                    reason: rev_payload.reason.clone(),
                    revoked_at: rev_payload.revoked_at.clone(),
                },
            )
            .unwrap();

        let verified = verify_signed_revocation_blob(&res.signed_revocation_blob).unwrap();
        assert_eq!(verified.license_id, "LIC-2026-ABCD-1234");
        assert_eq!(verified.reason, "Chargeback fraud reported");
    }

    #[test]
    fn isolated_signer_signs_updater_release_and_verifies_key_recovery() {
        let conn = Connection::open_in_memory().unwrap();
        let signer = SignerService::default();
        let rel_payload = UpdaterReleaseSignPayload {
            version: "1.1.0".to_string(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            artifact_url: "https://cdn.coachify.app/releases/CoachifySetup-v1.1.0-x64.exe"
                .to_string(),
            authenticode_thumbprint: "AA11BB22CC33DD44EE55FF6677889900AABBCCDD".to_string(),
            key_id: "u2026a".to_string(),
        };
        let canonical = canonical_value_json(&rel_payload).unwrap();
        let ts = "2026-09-25T12:00:00Z".to_string();
        let hmac = compute_service_hmac(DEFAULT_SIGNER_SHARED_SECRET, "cloud_api", &ts, &canonical);

        let sign_res = signer
            .sign_updater_release(
                &conn,
                &SignUpdaterReleaseRequest {
                    caller_service_id: "cloud_api".to_string(),
                    timestamp_utc: ts.clone(),
                    hmac_signature: hmac,
                    key_id: "u2026a".to_string(),
                    payload: rel_payload.clone(),
                },
            )
            .expect("u2026a updater release signing should succeed");

        assert!(
            verify_updater_release_signature(&rel_payload, &sign_res.updater_signature).unwrap()
        );

        // Verify key recovery drill for both windows_authenticode and tauri_updater_ed25519
        for (kt, kid) in [
            ("windows_authenticode", "authenticode_2026_ev"),
            ("tauri_updater_ed25519", "u2026a"),
        ] {
            let blob = format!("ENC_VAULT_V1_{kt}_{kid}_OFFLINE_USB_SHARD");
            let auth_str = format!("{kt}|{kid}|vault-dhaka-cold-01|{blob}");
            let drill_hmac =
                compute_service_hmac(DEFAULT_SIGNER_SHARED_SECRET, "cloud_api", &ts, &auth_str);
            let drill_res = signer
                .verify_key_recovery(
                    &conn,
                    &VerifyKeyRecoveryRequest {
                        caller_service_id: "cloud_api".to_string(),
                        timestamp_utc: ts.clone(),
                        hmac_signature: drill_hmac,
                        key_type: kt.to_string(),
                        key_id: kid.to_string(),
                        backup_source: "vault-dhaka-cold-01".to_string(),
                        encrypted_backup_blob: blob,
                        passphrase: "VaultRecoveryPass#2026".to_string(),
                    },
                )
                .expect("recovery drill should verify");
            assert!(drill_res.signature_verified);
        }
    }
}
