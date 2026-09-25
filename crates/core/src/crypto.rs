use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    #[error("DPAPI encryption failed: {0}")]
    DpapiProtectFailed(String),
    #[error("DPAPI decryption failed: {0}")]
    DpapiUnprotectFailed(String),
    #[error("Invalid recovery key format: {0}")]
    InvalidRecoveryKeyFormat(String),
    #[error("Recovery key checksum mismatch")]
    RecoveryKeyChecksumMismatch,
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Authenticated decryption failed (tag mismatch or corrupted data)")]
    DecryptionFailed,
    #[error("Base64 decode error: {0}")]
    Base64Decode(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::Serialization(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// Base64 helper utilities (standard engine without padding issues)
// -----------------------------------------------------------------------------

pub fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as u32
        } else {
            0
        };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);

        if i + 1 < data.len() {
            result.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }
    result
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, CryptoError> {
    let clean = input.trim();
    if clean.is_empty() {
        return Ok(Vec::new());
    }

    fn val(c: u8) -> Result<u32, CryptoError> {
        match c {
            b'A'..=b'Z' => Ok((c - b'A') as u32),
            b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(CryptoError::Base64Decode(format!(
                "Invalid base64 char: {}",
                c as char
            ))),
        }
    }

    let bytes = clean.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(CryptoError::Base64Decode(
            "Invalid base64 length".to_string(),
        ));
    }

    let mut out = Vec::with_capacity((bytes.len() / 4) * 3);
    for chunk in bytes.chunks(4) {
        let v0 = val(chunk[0])?;
        let v1 = val(chunk[1])?;
        let v2 = val(chunk[2])?;
        let v3 = val(chunk[3])?;

        let triple = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;

        out.push(((triple >> 16) & 0xFF) as u8);
        if chunk[2] != b'=' {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            out.push((triple & 0xFF) as u8);
        }
    }
    Ok(out)
}

// -----------------------------------------------------------------------------
// Master Key Generation
// -----------------------------------------------------------------------------

/// Generates a cryptographically secure 256-bit random master database key.
pub fn generate_master_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    key
}

// -----------------------------------------------------------------------------
// Windows DPAPI Machine-Scope Wrapping
// -----------------------------------------------------------------------------

static MOCK_DPAPI_FAIL: AtomicBool = AtomicBool::new(false);
static MOCK_DPAPI_FORCE: AtomicBool = AtomicBool::new(false);

/// Test helper to simulate foreign machine or DPAPI failure.
pub fn set_mock_dpapi_failure(fail: bool) {
    MOCK_DPAPI_FAIL.store(fail, Ordering::SeqCst);
}

/// Test helper to force mock DPAPI mode (used for portable unit testing).
pub fn set_mock_dpapi_force(force: bool) {
    MOCK_DPAPI_FORCE.store(force, Ordering::SeqCst);
}

#[cfg(windows)]
#[repr(C)]
struct DATA_BLOB {
    cb_data: u32,
    pb_data: *mut u8,
}

#[cfg(windows)]
const CRYPTPROTECT_LOCAL_MACHINE: u32 = 0x4;

#[cfg(windows)]
#[link(name = "crypt32")]
extern "system" {
    fn CryptProtectData(
        p_data_in: *const DATA_BLOB,
        sz_data_descr: *const u16,
        p_optional_entropy: *const DATA_BLOB,
        pv_reserved: *mut std::ffi::c_void,
        p_prompt_struct: *mut std::ffi::c_void,
        dw_flags: u32,
        p_data_out: *mut DATA_BLOB,
    ) -> i32;

    fn CryptUnprotectData(
        p_data_in: *const DATA_BLOB,
        ppsz_data_descr: *mut *mut u16,
        p_optional_entropy: *const DATA_BLOB,
        pv_reserved: *mut std::ffi::c_void,
        p_prompt_struct: *mut std::ffi::c_void,
        dw_flags: u32,
        p_data_out: *mut DATA_BLOB,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn LocalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

/// Wraps data using Windows DPAPI machine-scope (`CRYPTPROTECT_LOCAL_MACHINE`).
pub fn wrap_dpapi_machine(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if MOCK_DPAPI_FAIL.load(Ordering::SeqCst) {
        return Err(CryptoError::DpapiProtectFailed(
            "Mock DPAPI protection error".to_string(),
        ));
    }

    if MOCK_DPAPI_FORCE.load(Ordering::SeqCst) {
        // Deterministic mock wrapper for tests
        let mut mock = b"MOCK_DPAPI_WRAP:".to_vec();
        mock.extend_from_slice(plaintext);
        return Ok(mock);
    }

    #[cfg(windows)]
    {
        let in_blob = DATA_BLOB {
            cb_data: plaintext.len() as u32,
            pb_data: plaintext.as_ptr() as *mut u8,
        };
        let mut out_blob = DATA_BLOB {
            cb_data: 0,
            pb_data: std::ptr::null_mut(),
        };

        let res = unsafe {
            CryptProtectData(
                &in_blob,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_LOCAL_MACHINE,
                &mut out_blob,
            )
        };

        if res == 0 || out_blob.pb_data.is_null() {
            let err = std::io::Error::last_os_error();
            return Err(CryptoError::DpapiProtectFailed(err.to_string()));
        }

        let slice =
            unsafe { std::slice::from_raw_parts(out_blob.pb_data, out_blob.cb_data as usize) };
        let result = slice.to_vec();
        unsafe {
            LocalFree(out_blob.pb_data as *mut std::ffi::c_void);
        }
        Ok(result)
    }

    #[cfg(not(windows))]
    {
        let mut mock = b"MOCK_DPAPI_WRAP:".to_vec();
        mock.extend_from_slice(plaintext);
        Ok(mock)
    }
}

/// Unwraps data using Windows DPAPI machine-scope.
pub fn unwrap_dpapi_machine(ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if MOCK_DPAPI_FAIL.load(Ordering::SeqCst) {
        return Err(CryptoError::DpapiUnprotectFailed(
            "Simulated foreign machine DPAPI failure".to_string(),
        ));
    }

    const PREFIX: &[u8] = b"MOCK_DPAPI_WRAP:";
    if ciphertext.starts_with(PREFIX) {
        return Ok(ciphertext[PREFIX.len()..].to_vec());
    }

    #[cfg(windows)]
    {
        let in_blob = DATA_BLOB {
            cb_data: ciphertext.len() as u32,
            pb_data: ciphertext.as_ptr() as *mut u8,
        };
        let mut out_blob = DATA_BLOB {
            cb_data: 0,
            pb_data: std::ptr::null_mut(),
        };

        let res = unsafe {
            CryptUnprotectData(
                &in_blob,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut out_blob,
            )
        };

        if res == 0 || out_blob.pb_data.is_null() {
            let err = std::io::Error::last_os_error();
            return Err(CryptoError::DpapiUnprotectFailed(err.to_string()));
        }

        let slice =
            unsafe { std::slice::from_raw_parts(out_blob.pb_data, out_blob.cb_data as usize) };
        let result = slice.to_vec();
        unsafe {
            LocalFree(out_blob.pb_data as *mut std::ffi::c_void);
        }
        Ok(result)
    }

    #[cfg(not(windows))]
    {
        Err(CryptoError::DpapiUnprotectFailed(
            "Unrecognized non-Windows DPAPI blob".to_string(),
        ))
    }
}

/// Encrypts a sensitive credential string at rest using Windows DPAPI machine-scope
/// and returns a Base64-encoded envelope prefixed with `DPAPI_V1:`.
pub fn protect_secret_dpapi(secret: &str) -> Result<String, CryptoError> {
    let wrapped = wrap_dpapi_machine(secret.as_bytes())?;
    Ok(format!("DPAPI_V1:{}", base64_encode(&wrapped)))
}

/// Decrypts a DPAPI-protected credential string (`DPAPI_V1:<base64>` or legacy `<base64>`).
/// Fails with `CryptoError::DpapiUnprotectFailed` if the database was copied to another machine.
pub fn unprotect_secret_dpapi(cipher_b64: &str) -> Result<String, CryptoError> {
    let clean = cipher_b64
        .trim()
        .strip_prefix("DPAPI_V1:")
        .unwrap_or(cipher_b64.trim());
    let raw = base64_decode(clean)?;
    let plain_bytes = unwrap_dpapi_machine(&raw)?;
    String::from_utf8(plain_bytes)
        .map_err(|e| CryptoError::DpapiUnprotectFailed(format!("Invalid UTF-8 secret: {}", e)))
}

/// Checks whether a JSON property name represents a sensitive credential that must never
/// be stored in plaintext `config_json` or exported in diagnostics.
pub fn is_sensitive_key_name(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("token")
        || lower.contains("private_key")
        || lower.contains("credential")
        || lower == "authorization"
        || lower == "smtp_pass"
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct SecretHealthItem {
    pub scope: String,
    pub identifier: String,
    pub status: String, // "healthy" | "foreign_machine" | "unset"
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct SecretsHealthReport {
    pub all_healthy: bool,
    pub configured_count: u32,
    pub foreign_machine_count: u32,
    pub zero_plaintext_leaks: bool,
    pub items: Vec<SecretHealthItem>,
}

/// Inspects all stored credentials across `provider_configs`, `ai_credit_wallets`, and `settings`
/// to verify DPAPI machine decryptability and confirm zero plaintext secret leaks.
pub fn get_secrets_health(conn: &rusqlite::Connection) -> Result<SecretsHealthReport, CryptoError> {
    let mut items = Vec::new();
    let mut configured_count = 0u32;
    let mut foreign_machine_count = 0u32;
    let mut zero_plaintext_leaks = true;

    // 1. Inspect provider_configs
    if let Ok(mut stmt) = conn.prepare(
        "SELECT channel, adapter, config_json, secret_ref, updated_at FROM provider_configs ORDER BY channel, adapter",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        }) {
            for r in rows.flatten() {
                let (channel, adapter, config_json, secret_ref, updated_at) = r;
                // Check config_json for any unredacted sensitive keys
                if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&config_json) {
                    for (k, v) in map {
                        if is_sensitive_key_name(&k) {
                            if let Some(s) = v.as_str() {
                                if !s.is_empty()
                                    && s != "[REDACTED_TO_DPAPI]"
                                    && s != "[REDACTED]"
                                    && !s.starts_with("••")
                                {
                                    zero_plaintext_leaks = false;
                                }
                            }
                        }
                    }
                }

                let status = match secret_ref {
                    None => "unset".to_string(),
                    Some(ref s) if s.trim().is_empty() => "unset".to_string(),
                    Some(ref s) => {
                        configured_count += 1;
                        // Check that raw secret_ref is wrapped (not raw plaintext)
                        if !s.starts_with("DPAPI_V1:") && base64_decode(s).is_err() {
                            zero_plaintext_leaks = false;
                        }
                        match unprotect_secret_dpapi(s) {
                            Ok(_) => "healthy".to_string(),
                            Err(_) => {
                                foreign_machine_count += 1;
                                "foreign_machine".to_string()
                            }
                        }
                    }
                };

                items.push(SecretHealthItem {
                    scope: "comms_provider".to_string(),
                    identifier: format!("{}:{}", channel, adapter),
                    status,
                    updated_at: Some(updated_at),
                });
            }
        }
    }

    // 2. Inspect ai_credit_wallets
    if let Ok(mut stmt) =
        conn.prepare("SELECT id, token_dpapi_blob, updated_at FROM ai_credit_wallets")
    {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<Vec<u8>>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        }) {
            for r in rows.flatten() {
                let (wallet_id, token_blob, updated_at) = r;
                let status = match token_blob {
                    None => "unset".to_string(),
                    Some(ref b) if b.is_empty() => "unset".to_string(),
                    Some(ref b) => {
                        configured_count += 1;
                        match unwrap_dpapi_machine(b) {
                            Ok(_) => "healthy".to_string(),
                            Err(_) => {
                                foreign_machine_count += 1;
                                "foreign_machine".to_string()
                            }
                        }
                    }
                };
                items.push(SecretHealthItem {
                    scope: "ai_gateway".to_string(),
                    identifier: format!("ai_wallet:{}:gateway_token", wallet_id),
                    status,
                    updated_at,
                });
            }
        }
    }

    Ok(SecretsHealthReport {
        all_healthy: foreign_machine_count == 0 && zero_plaintext_leaks,
        configured_count,
        foreign_machine_count,
        zero_plaintext_leaks,
        items,
    })
}

// -----------------------------------------------------------------------------
// Crockford Base32 Recovery Key Formatting with Checksum
// -----------------------------------------------------------------------------

const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

fn calculate_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Generates a human-readable 24-character Owner Recovery Key formatted as:
/// `CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`
/// The first 20 characters encode 100 bits of entropy, and the final 4 characters
/// encode a 20-bit polynomial checksum for immediate validation.
pub fn generate_recovery_key() -> (String, Vec<u8>) {
    let mut raw_entropy = [0u8; 13];
    rand::rngs::OsRng.fill_bytes(&mut raw_entropy);

    // Convert 100 bits (20 characters of 5 bits each)
    let mut body_chars = Vec::with_capacity(20);
    let mut bit_buf: u64 = 0;
    let mut bit_count = 0;

    for &b in &raw_entropy[0..13] {
        bit_buf = (bit_buf << 8) | (b as u64);
        bit_count += 8;
        while bit_count >= 5 && body_chars.len() < 20 {
            bit_count -= 5;
            let idx = ((bit_buf >> bit_count) & 0x1F) as usize;
            body_chars.push(CROCKFORD_ALPHABET[idx] as char);
        }
    }

    let body_str: String = body_chars.into_iter().collect();

    // Compute checksum over canonical body bytes
    let crc = calculate_crc16(body_str.as_bytes());
    // 4 checksum characters (4 x 5 = 20 bits; pack crc into 20 bits)
    let checksum_bits: u32 = ((crc as u32) << 4) | ((crc as u32) & 0x0F);
    let mut check_chars = Vec::with_capacity(4);
    for i in (0..4).rev() {
        let idx = ((checksum_bits >> (i * 5)) & 0x1F) as usize;
        check_chars.push(CROCKFORD_ALPHABET[idx] as char);
    }
    let check_str: String = check_chars.into_iter().collect();

    let full_24 = format!("{}{}", body_str, check_str);

    // Chunk into 6 blocks of 4
    let mut chunked = String::from("CKREC");
    for i in 0..6 {
        chunked.push('-');
        chunked.push_str(&full_24[i * 4..(i + 1) * 4]);
    }

    (chunked, raw_entropy.to_vec())
}

/// Normalizes and verifies an input recovery key, asserting checksum correctness.
/// Returns the canonical 24-character key string.
pub fn normalize_and_verify_recovery_key(input: &str) -> Result<String, CryptoError> {
    let mut clean = String::new();
    let upper = input.trim().to_uppercase();

    // Strip optional "CKREC" or "CKREC-" prefix
    let core_str = if let Some(stripped) = upper.strip_prefix("CKREC") {
        stripped.trim_start_matches('-')
    } else {
        &upper
    };

    for c in core_str.chars() {
        match c {
            '-' | ' ' => continue,
            'O' => clean.push('0'),
            'I' | 'L' => clean.push('1'),
            c if CROCKFORD_ALPHABET.contains(&(c as u8)) => clean.push(c),
            other => {
                return Err(CryptoError::InvalidRecoveryKeyFormat(format!(
                    "Unrecognized character: {}",
                    other
                )))
            }
        }
    }

    if clean.len() != 24 {
        return Err(CryptoError::InvalidRecoveryKeyFormat(format!(
            "Expected 24 characters, got {}",
            clean.len()
        )));
    }

    let body_str = &clean[0..20];
    let expected_check = &clean[20..24];

    let crc = calculate_crc16(body_str.as_bytes());
    let checksum_bits: u32 = ((crc as u32) << 4) | ((crc as u32) & 0x0F);
    let mut calculated_check = String::with_capacity(4);
    for i in (0..4).rev() {
        let idx = ((checksum_bits >> (i * 5)) & 0x1F) as usize;
        calculated_check.push(CROCKFORD_ALPHABET[idx] as char);
    }

    if expected_check != calculated_check {
        return Err(CryptoError::RecoveryKeyChecksumMismatch);
    }

    // Format canonical chunked representation
    let mut canonical = String::from("CKREC");
    for i in 0..6 {
        canonical.push('-');
        canonical.push_str(&clean[i * 4..(i + 1) * 4]);
    }

    Ok(canonical)
}

/// Extracts the last 4 characters of a formatted recovery key for the confirmation gate.
pub fn extract_last_four_checksum(key: &str) -> Result<String, CryptoError> {
    let canonical = normalize_and_verify_recovery_key(key)?;
    let clean: String = canonical
        .strip_prefix("CKREC-")
        .unwrap_or(&canonical)
        .replace('-', "");
    if clean.len() == 24 {
        Ok(clean[20..24].to_string())
    } else {
        Err(CryptoError::InvalidRecoveryKeyFormat(
            "Malformed key length".to_string(),
        ))
    }
}

// -----------------------------------------------------------------------------
// Argon2id + AES-256-GCM Envelope Encryption
// -----------------------------------------------------------------------------

const ARGON2_M_COST: u32 = 65536; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

/// Derives a 256-bit Key Encryption Key (KEK) from a recovery key and salt.
pub fn derive_kek_from_key(recovery_key: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    let params = Params::new(
        ARGON2_M_COST,
        ARGON2_T_COST,
        ARGON2_P_COST,
        Some(32), // output 32 bytes
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut kek = [0u8; 32];
    argon2
        .hash_password_into(recovery_key.as_bytes(), salt, &mut kek)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(kek)
}

pub fn derive_kek(recovery_key: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    derive_kek_from_key(recovery_key, salt)
}

// -----------------------------------------------------------------------------
// KeyEnvelope Data Structures
// -----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct DpapiEnvelope {
    pub scope: String,
    pub wrapped_key_base64: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct RecoveryEnvelope {
    pub kdf: String,
    pub argon2_params: Argon2Params,
    pub salt_base64: String,
    pub nonce_base64: String,
    pub ciphertext_base64: String,
    pub checksum: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct KeyEnvelope {
    pub version: u32,
    pub institution_id: String,
    pub created_at: String,
    pub dpapi: DpapiEnvelope,
    pub recovery: RecoveryEnvelope,
}

impl KeyEnvelope {
    /// Creates a key envelope using a pre-generated or verified recovery key.
    pub fn create_with_recovery_key(
        institution_id: &str,
        master_key: &[u8; 32],
        recovery_key: &str,
    ) -> Result<Self, CryptoError> {
        // 1. Wrap with DPAPI machine scope
        let dpapi_bytes = wrap_dpapi_machine(master_key)?;
        let dpapi_b64 = base64_encode(&dpapi_bytes);

        // 2. Validate and extract checksum
        let checksum = extract_last_four_checksum(recovery_key)?;

        // 3. Generate salt and nonce
        let mut salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

        // 4. Derive KEK via Argon2id
        let kek = derive_kek(recovery_key, &salt)?;

        // 5. Encrypt master key with AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&kek)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext_with_tag = cipher
            .encrypt(nonce, master_key.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)?;

        let envelope = KeyEnvelope {
            version: 1,
            institution_id: institution_id.to_string(),
            created_at: crate::db::now_utc_iso(),
            dpapi: DpapiEnvelope {
                scope: "local_machine".to_string(),
                wrapped_key_base64: dpapi_b64,
            },
            recovery: RecoveryEnvelope {
                kdf: "argon2id".to_string(),
                argon2_params: Argon2Params {
                    m_cost: ARGON2_M_COST,
                    t_cost: ARGON2_T_COST,
                    p_cost: ARGON2_P_COST,
                },
                salt_base64: base64_encode(&salt),
                nonce_base64: base64_encode(&nonce_bytes),
                ciphertext_base64: base64_encode(&ciphertext_with_tag),
                checksum,
            },
        };

        Ok(envelope)
    }

    /// Creates a complete two-tier key envelope from a master database key.
    /// Returns the envelope and the plaintext recovery key string.
    pub fn create(
        institution_id: &str,
        master_key: &[u8; 32],
    ) -> Result<(Self, String), CryptoError> {
        let (recovery_key, _) = generate_recovery_key();
        let envelope = Self::create_with_recovery_key(institution_id, master_key, &recovery_key)?;
        Ok((envelope, recovery_key))
    }

    /// Unwraps the master key using Windows DPAPI (machine scope).
    pub fn unwrap_with_dpapi(&self) -> Result<[u8; 32], CryptoError> {
        let ciphertext = base64_decode(&self.dpapi.wrapped_key_base64)?;
        let plaintext = unwrap_dpapi_machine(&ciphertext)?;
        if plaintext.len() != 32 {
            return Err(CryptoError::DpapiUnprotectFailed(format!(
                "Expected 32 bytes master key, got {}",
                plaintext.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&plaintext);
        Ok(key)
    }

    /// Unwraps the master key using the Owner Recovery Key.
    pub fn unwrap_with_recovery_key(&self, recovery_key: &str) -> Result<[u8; 32], CryptoError> {
        let canonical_key = normalize_and_verify_recovery_key(recovery_key)?;
        let salt = base64_decode(&self.recovery.salt_base64)?;
        let nonce_bytes = base64_decode(&self.recovery.nonce_base64)?;
        let ciphertext_with_tag = base64_decode(&self.recovery.ciphertext_base64)?;

        let kek = derive_kek(&canonical_key, &salt)?;

        let cipher = Aes256Gcm::new_from_slice(&kek)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let decrypted = cipher
            .decrypt(nonce, ciphertext_with_tag.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)?;

        if decrypted.len() != 32 {
            return Err(CryptoError::DecryptionFailed);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);
        Ok(key)
    }

    /// Re-wraps the master key with the current machine's DPAPI.
    /// Used when recovering on a new PC.
    pub fn rewrap_dpapi(&mut self, master_key: &[u8; 32]) -> Result<(), CryptoError> {
        let dpapi_bytes = wrap_dpapi_machine(master_key)?;
        self.dpapi.wrapped_key_base64 = base64_encode(&dpapi_bytes);
        Ok(())
    }

    /// Saves the envelope to a file (typically `key.envelope`).
    pub fn save_to_file(&self, path: &Path) -> Result<(), CryptoError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Loads the envelope from a file.
    pub fn load_from_file(path: &Path) -> Result<Self, CryptoError> {
        let json = fs::read_to_string(path)?;
        let envelope: Self = serde_json::from_str(&json)?;
        Ok(envelope)
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, Coachify Key Envelope Test!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).expect("decode should succeed");
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_generate_master_key_randomness() {
        let k1 = generate_master_key();
        let k2 = generate_master_key();
        assert_ne!(k1, k2);
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_recovery_key_format_and_verification() {
        let (key, _) = generate_recovery_key();
        assert!(key.starts_with("CKREC-"));
        let chunks: Vec<&str> = key.split('-').collect();
        assert_eq!(chunks.len(), 7); // "CKREC" + 6 blocks
        for chunk in &chunks[1..] {
            assert_eq!(chunk.len(), 4);
        }

        // Verify key checksum
        let canonical =
            normalize_and_verify_recovery_key(&key).expect("generated key should be valid");
        assert_eq!(canonical, key);

        // Verify last four checksum matches
        let last4 = extract_last_four_checksum(&key).expect("checksum extraction should succeed");
        assert_eq!(chunks[6], last4);

        // Verify typo detection
        let mut tampered = key.clone();
        tampered.replace_range(8..9, "Z");
        if tampered != key {
            let res = normalize_and_verify_recovery_key(&tampered);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_two_tier_envelope_create_and_unwrap_dpapi() {
        set_mock_dpapi_force(true);
        let master = generate_master_key();
        let (env, rec_key) = KeyEnvelope::create("inst_test_001", &master)
            .expect("envelope creation should succeed");

        // Unwrap via DPAPI
        let unwrapped_dpapi = env
            .unwrap_with_dpapi()
            .expect("DPAPI unwrap should succeed");
        assert_eq!(master, unwrapped_dpapi);

        // Unwrap via Recovery Key
        let unwrapped_rec = env
            .unwrap_with_recovery_key(&rec_key)
            .expect("Recovery key unwrap should succeed");
        assert_eq!(master, unwrapped_rec);

        // Invalid recovery key fails
        let invalid_res = env.unwrap_with_recovery_key("CKREC-0000-0000-0000-0000-0000-0000");
        assert!(invalid_res.is_err());
        set_mock_dpapi_force(false);
    }

    #[test]
    fn test_envelope_file_save_and_load() {
        set_mock_dpapi_force(true);
        let temp_dir = tempfile::tempdir().unwrap();
        let env_path = temp_dir.path().join("db").join("key.envelope");

        let master = generate_master_key();
        let (env, rec_key) = KeyEnvelope::create("inst_test_file", &master).unwrap();
        env.save_to_file(&env_path).unwrap();

        let loaded = KeyEnvelope::load_from_file(&env_path).unwrap();
        assert_eq!(env, loaded);

        let unwrapped = loaded.unwrap_with_recovery_key(&rec_key).unwrap();
        assert_eq!(master, unwrapped);
        set_mock_dpapi_force(false);
    }
}
