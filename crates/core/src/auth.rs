use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Password hashing failed: {0}")]
    Hash(String),
    #[error("Password verification failed: {0}")]
    Verify(String),
    #[error("Password does not meet complexity requirements: {0}")]
    WeakPassword(String),
}

/// Argon2id parameters specified in spec 0004 AC-2:
/// - 64 MB memory (65536 KiB)
/// - 3 iterations
/// - 4 parallel lanes
pub const ARGON2_MEMORY_KIB: u32 = 65536;
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Hashes a plaintext password using Argon2id with 64 MB memory, 3 iterations,
/// 4 parallel lanes, and a 16-byte random salt.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AuthError::WeakPassword(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        )));
    }

    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        None,
    )
    .map_err(|e| AuthError::Hash(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::Hash(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verifies a plaintext password against an Argon2 PHC formatted hash string.
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| AuthError::Verify(e.to_string()))?;

    // Argon2::default() inspects the algorithm and parameters in parsed_hash to verify
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(err) => Err(AuthError::Verify(err.to_string())),
    }
}

/// Generates a cryptographically secure 256-bit bearer token formatted as a 64-character hex string.
pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut token = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut token, "{:02x}", b);
    }
    token
}

/// Computes the SHA256 hash of a session bearer token for database storage.
pub fn hash_session_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password_success() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password).expect("hashing should succeed");

        assert!(hash.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"));
        let is_valid = verify_password(password, &hash).expect("verification should succeed");
        assert!(is_valid);
    }

    #[test]
    fn test_verify_incorrect_password_fails() {
        let password = "CorrectPassword123!";
        let hash = hash_password(password).expect("hashing should succeed");

        let is_valid =
            verify_password("WrongPassword!", &hash).expect("verification should evaluate");
        assert!(!is_valid);
    }

    #[test]
    fn test_password_too_short_rejected() {
        let short = "short";
        let result = hash_password(short);
        assert!(result.is_err());
        if let Err(AuthError::WeakPassword(msg)) = result {
            assert!(msg.contains("at least 8 characters"));
        } else {
            panic!("Expected WeakPassword error");
        }
    }

    #[test]
    fn test_session_token_generation_and_hashing() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        assert_eq!(token1.len(), 64);
        assert_eq!(token2.len(), 64);
        assert_ne!(token1, token2);

        let hash1 = hash_session_token(&token1);
        let hash2 = hash_session_token(&token1);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }
}
