//! LAN Host Update Mirror Storage and Operations Engine (Feature 72, Spec 0048).
//!
//! Provides local caching, staging, validation, and metadata serving of Coachify
//! installer releases so secondary counter workstations can update across the LAN
//! without internet connectivity.

use crate::lan::protocol::{
    ClearMirrorUpdateInput, ConfigureMirrorInput, HostUpdateManifest, LanError, LanStagedUpdate,
    LanUpdateMirrorConfig, LanUpdateMirrorStatus, StageMirrorUpdateInput,
};
use ring::signature::{Ed25519KeyPair, UnparsedPublicKey, ED25519};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Resolves the base directory for storing staged update releases.
pub fn resolve_mirror_storage_dir(
    conn: &Connection,
    current_utc: &str,
    base_dir_override: Option<&Path>,
) -> Result<PathBuf, LanError> {
    if let Some(dir) = base_dir_override {
        return Ok(dir.to_path_buf());
    }

    let config = get_or_create_mirror_config(conn, current_utc)?;
    if let Some(ref custom_path) = config.storage_path {
        let p = PathBuf::from(custom_path);
        if p.is_absolute() {
            return Ok(p);
        }
    }

    if let Ok(Some(cfg)) = crate::storage::load_global_config() {
        Ok(PathBuf::from(&cfg.data_dir).join("update_mirror"))
    } else {
        Ok(std::env::temp_dir().join("coachify_update_mirror"))
    }
}

/// Retrieves or creates the default mirror configuration singleton.
pub fn get_or_create_mirror_config(
    conn: &Connection,
    current_utc: &str,
) -> Result<LanUpdateMirrorConfig, LanError> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, mirror_enabled, storage_path, max_cached_releases, auto_stage_on_host_update, created_at, updated_at
         FROM lan_update_mirror_config WHERE id = 'default'",
    )?;

    let existing = stmt
        .query_row([], |row| {
            Ok(LanUpdateMirrorConfig {
                id: row.get(0)?,
                mirror_enabled: row.get::<_, i64>(1)? == 1,
                storage_path: row.get(2)?,
                max_cached_releases: row.get(3)?,
                auto_stage_on_host_update: row.get::<_, i64>(4)? == 1,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .optional()?;

    if let Some(cfg) = existing {
        return Ok(cfg);
    }

    conn.execute(
        "INSERT INTO lan_update_mirror_config (id, mirror_enabled, storage_path, max_cached_releases, auto_stage_on_host_update, created_at, updated_at)
         VALUES ('default', 1, NULL, 2, 1, ?1, ?2)",
        params![current_utc, current_utc],
    )?;

    Ok(LanUpdateMirrorConfig {
        id: "default".to_string(),
        mirror_enabled: true,
        storage_path: None,
        max_cached_releases: 2,
        auto_stage_on_host_update: true,
        created_at: current_utc.to_string(),
        updated_at: current_utc.to_string(),
    })
}

/// Updates mirror configuration settings (enable/disable, etc.).
pub fn configure_mirror(
    conn: &Connection,
    input: &ConfigureMirrorInput,
    current_utc: &str,
) -> Result<LanUpdateMirrorConfig, LanError> {
    get_or_create_mirror_config(conn, current_utc)?;

    let enabled_int = if input.mirror_enabled { 1 } else { 0 };
    conn.execute(
        "UPDATE lan_update_mirror_config SET mirror_enabled = ?1, updated_at = ?2 WHERE id = 'default'",
        params![enabled_int, current_utc],
    )?;

    get_or_create_mirror_config(conn, current_utc)
}

/// Retrieves the comprehensive mirror status and current active staged release.
pub fn get_mirror_status(
    conn: &Connection,
    current_utc: &str,
    base_dir_override: Option<&Path>,
) -> Result<LanUpdateMirrorStatus, LanError> {
    let config = get_or_create_mirror_config(conn, current_utc)?;
    let storage_dir = resolve_mirror_storage_dir(conn, current_utc, base_dir_override)?;

    let mut stmt = conn.prepare_cached(
        "SELECT id, version, channel, artifact_filename, file_size_bytes, sha256,
                updater_signature, updater_key_id, notes, min_upgrade_from, staged_at,
                downloads_count, is_active
         FROM lan_staged_updates
         WHERE is_active = 1
         ORDER BY staged_at DESC
         LIMIT 1",
    )?;

    let active_update = stmt
        .query_row([], |row| {
            Ok(LanStagedUpdate {
                id: row.get(0)?,
                version: row.get(1)?,
                channel: row.get(2)?,
                artifact_filename: row.get(3)?,
                file_size_bytes: row.get(4)?,
                sha256: row.get(5)?,
                updater_signature: row.get(6)?,
                updater_key_id: row.get(7)?,
                notes: row.get(8)?,
                min_upgrade_from: row.get(9)?,
                staged_at: row.get(10)?,
                downloads_count: row.get(11)?,
                is_active: row.get::<_, i64>(12)? == 1,
            })
        })
        .optional()?;

    let (total_cached, total_downloads): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(downloads_count), 0) FROM lan_staged_updates",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(LanUpdateMirrorStatus {
        mirror_enabled: config.mirror_enabled,
        storage_dir: storage_dir.to_string_lossy().to_string(),
        active_update,
        total_cached_releases: total_cached,
        total_downloads_served: total_downloads,
    })
}

/// Computes SHA-256 lowercase hex checksum of arbitrary data bytes.
pub fn compute_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Signs a SHA-256 checksum string using the embedded Ed25519 updater private seed (`u2026a` or `u2026b`).
pub fn sign_update_checksum_ed25519(sha256: &str, key_id: &str) -> Result<String, LanError> {
    let seed = match key_id {
        "u2026a" => &coachify_license_signer::SEED_U2026A,
        "u2026b" => &coachify_license_signer::SEED_U2026B,
        _ => {
            return Err(LanError::UpdateSignatureInvalid(format!(
                "Unknown updater key_id '{key_id}'"
            )))
        }
    };
    let kp = Ed25519KeyPair::from_seed_unchecked(seed)
        .map_err(|e| LanError::UpdateSignatureInvalid(e.to_string()))?;
    let sig = kp.sign(sha256.as_bytes());
    Ok(hex::encode(sig.as_ref()))
}

/// Cryptographically verifies an Ed25519 updater signature against the trusted vendor public key.
pub fn verify_staged_update_signature(
    file_bytes: &[u8],
    sha256: &str,
    signature_hex: &str,
    key_id: &str,
) -> Result<(), LanError> {
    let pub_key_bytes = coachify_license_signer::get_public_key_bytes(key_id)
        .ok_or_else(|| LanError::UpdateSignatureInvalid(format!("Unknown key_id '{key_id}'")))?;
    let sig_bytes = hex::decode(signature_hex.trim())
        .map_err(|e| LanError::UpdateSignatureInvalid(format!("Invalid signature hex: {e}")))?;
    if sig_bytes.len() != 64 {
        return Err(LanError::UpdateSignatureInvalid(format!(
            "Expected 64-byte Ed25519 signature, got {}",
            sig_bytes.len()
        )));
    }
    let pub_key = UnparsedPublicKey::new(&ED25519, &pub_key_bytes);
    if pub_key.verify(sha256.as_bytes(), &sig_bytes).is_ok() {
        return Ok(());
    }
    if pub_key.verify(file_bytes, &sig_bytes).is_ok() {
        return Ok(());
    }
    Err(LanError::UpdateSignatureInvalid(
        "Ed25519 signature verification failed for update package".to_string(),
    ))
}

/// Resolves the local path where a workstation writes its downloaded update installer.
pub fn resolve_workstation_staged_update_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("COACHIFY_TEST_WORKSTATION_UPDATE_PATH") {
        if !override_path.trim().is_empty() {
            let path = PathBuf::from(override_path);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            return path;
        }
    }
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let dir = PathBuf::from(local_app).join("Coachify");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("staged_update.exe");
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".coachify");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("staged_update.exe")
}

/// Stages an installer file into the LAN host update mirror cache.
pub fn stage_update(
    conn: &Connection,
    input: &StageMirrorUpdateInput,
    current_utc: &str,
    base_dir_override: Option<&Path>,
) -> Result<HostUpdateManifest, LanError> {
    let source_path = Path::new(&input.file_path);
    if !source_path.exists() {
        return Err(LanError::UpdateFileNotFound(input.file_path.clone()));
    }

    let file_bytes = std::fs::read(source_path)
        .map_err(|e| LanError::UpdateMirrorError(format!("Failed to read source update: {e}")))?;

    if file_bytes.is_empty() {
        return Err(LanError::UpdateMirrorError(
            "Source update file is empty (0 bytes)".to_string(),
        ));
    }

    let computed_sha256 = compute_sha256(&file_bytes);
    if let Some(ref expected) = input.expected_sha256 {
        if !expected.trim().eq_ignore_ascii_case(&computed_sha256) {
            return Err(LanError::UpdateChecksumMismatch {
                expected: expected.trim().to_lowercase(),
                computed: computed_sha256,
            });
        }
    }

    let key_id = input
        .updater_key_id
        .clone()
        .unwrap_or_else(|| "u2026a".to_string());

    let signature = match input.updater_signature {
        Some(ref sig) => sig.clone(),
        None => sign_update_checksum_ed25519(&computed_sha256, &key_id)
            .unwrap_or_else(|_| "unverified_local_stage".to_string()),
    };
    let channel = input
        .channel
        .clone()
        .unwrap_or_else(|| "stable".to_string());
    let notes = input.notes.clone().unwrap_or_default();

    let storage_dir = resolve_mirror_storage_dir(conn, current_utc, base_dir_override)?;
    let version_dir = storage_dir.join(&input.version);
    std::fs::create_dir_all(&version_dir).map_err(|e| {
        LanError::UpdateMirrorError(format!("Failed to create mirror version directory: {e}"))
    })?;

    let filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Coachify-Setup.exe")
        .to_string();

    let dest_file_path = version_dir.join(&filename);
    std::fs::write(&dest_file_path, &file_bytes).map_err(|e| {
        LanError::UpdateMirrorError(format!("Failed to write mirror target file: {e}"))
    })?;

    let file_size_bytes = file_bytes.len() as i64;

    // Deactivate previous active records
    conn.execute(
        "UPDATE lan_staged_updates SET is_active = 0 WHERE is_active = 1",
        [],
    )?;

    // Insert new active record
    conn.execute(
        "INSERT INTO lan_staged_updates (
            id, version, channel, artifact_filename, file_size_bytes, sha256,
            updater_signature, updater_key_id, notes, min_upgrade_from, staged_at, downloads_count, is_active
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, 0, 1)
         ON CONFLICT(id) DO UPDATE SET
            version = excluded.version,
            channel = excluded.channel,
            artifact_filename = excluded.artifact_filename,
            file_size_bytes = excluded.file_size_bytes,
            sha256 = excluded.sha256,
            updater_signature = excluded.updater_signature,
            updater_key_id = excluded.updater_key_id,
            notes = excluded.notes,
            staged_at = excluded.staged_at,
            is_active = 1",
        params![
            input.version,
            input.version,
            channel,
            filename,
            file_size_bytes,
            computed_sha256,
            signature,
            key_id,
            notes,
            current_utc,
        ],
    )?;

    let manifest = HostUpdateManifest {
        version: input.version.clone(),
        channel,
        artifact_filename: filename,
        file_size_bytes,
        sha256: computed_sha256,
        updater_signature: signature,
        updater_key_id: key_id,
        notes,
        min_upgrade_from: None,
        download_path: format!("/updates/download/{}", input.version),
    };

    // Save metadata json alongside file for offline inspection
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap_or_default();
    let _ = std::fs::write(version_dir.join("manifest.json"), manifest_json);

    Ok(manifest)
}

/// Clears staged releases from the mirror cache.
pub fn clear_staged_update(
    conn: &Connection,
    input: &ClearMirrorUpdateInput,
    current_utc: &str,
    base_dir_override: Option<&Path>,
) -> Result<LanUpdateMirrorStatus, LanError> {
    if let Some(ref version) = input.version {
        conn.execute(
            "DELETE FROM lan_staged_updates WHERE version = ?1",
            params![version],
        )?;

        let storage_dir = resolve_mirror_storage_dir(conn, current_utc, base_dir_override)?;
        let version_dir = storage_dir.join(version);
        if version_dir.exists() {
            let _ = std::fs::remove_dir_all(version_dir);
        }
    } else {
        conn.execute("DELETE FROM lan_staged_updates", [])?;

        let storage_dir = resolve_mirror_storage_dir(conn, current_utc, base_dir_override)?;
        if storage_dir.exists() {
            let _ = std::fs::remove_dir_all(&storage_dir);
        }
    }

    get_mirror_status(conn, current_utc, base_dir_override)
}

/// Returns the currently active release manifest, or None if no update is staged.
pub fn get_active_manifest(
    conn: &Connection,
    current_utc: &str,
) -> Result<Option<HostUpdateManifest>, LanError> {
    let config = get_or_create_mirror_config(conn, current_utc)?;
    if !config.mirror_enabled {
        return Err(LanError::UpdateMirrorDisabled);
    }

    let mut stmt = conn.prepare_cached(
        "SELECT version, channel, artifact_filename, file_size_bytes, sha256,
                updater_signature, updater_key_id, notes, min_upgrade_from
         FROM lan_staged_updates
         WHERE is_active = 1
         ORDER BY staged_at DESC
         LIMIT 1",
    )?;

    let manifest = stmt
        .query_row([], |row| {
            let version: String = row.get(0)?;
            Ok(HostUpdateManifest {
                download_path: format!("/updates/download/{version}"),
                version,
                channel: row.get(1)?,
                artifact_filename: row.get(2)?,
                file_size_bytes: row.get(3)?,
                sha256: row.get(4)?,
                updater_signature: row.get(5)?,
                updater_key_id: row.get(6)?,
                notes: row.get(7)?,
                min_upgrade_from: row.get(8)?,
            })
        })
        .optional()?;

    Ok(manifest)
}

/// Locates the physical file path for a staged version.
pub fn get_staged_file_path(
    conn: &Connection,
    version: &str,
    current_utc: &str,
    base_dir_override: Option<&Path>,
) -> Result<PathBuf, LanError> {
    let config = get_or_create_mirror_config(conn, current_utc)?;
    if !config.mirror_enabled {
        return Err(LanError::UpdateMirrorDisabled);
    }

    let filename: String = conn
        .query_row(
            "SELECT artifact_filename FROM lan_staged_updates WHERE version = ?1 AND is_active = 1",
            params![version],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(LanError::NoUpdateMirrored)?;

    let storage_dir = resolve_mirror_storage_dir(conn, current_utc, base_dir_override)?;
    let file_path = storage_dir.join(version).join(filename);

    if !file_path.exists() {
        return Err(LanError::UpdateFileNotFound(
            file_path.to_string_lossy().to_string(),
        ));
    }

    Ok(file_path)
}

/// Increments download served count for a version.
pub fn record_download_completed(conn: &Connection, version: &str) -> Result<i64, LanError> {
    conn.execute(
        "UPDATE lan_staged_updates SET downloads_count = downloads_count + 1 WHERE version = ?1",
        params![version],
    )?;

    let count: i64 = conn
        .query_row(
            "SELECT downloads_count FROM lan_staged_updates WHERE version = ?1",
            params![version],
            |row| row.get(0),
        )
        .unwrap_or(1);

    Ok(count)
}

/// Returns true if `target` version is semantically greater than `current` version.
/// Supports standard `MAJOR.MINOR.PATCH` format (e.g. "1.1.0" > "1.0.0").
pub fn is_newer_version(current: &str, target: &str) -> bool {
    let parse_nums = |v: &str| -> Vec<u64> {
        let clean = v.trim().trim_start_matches('v');
        clean
            .split('.')
            .map(|s| s.split('-').next().unwrap_or(s))
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };
    let cur_parts = parse_nums(current);
    let target_parts = parse_nums(target);

    for (c, t) in cur_parts.iter().zip(target_parts.iter()) {
        if t > c {
            return true;
        }
        if t < c {
            return false;
        }
    }
    target_parts.len() > cur_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::apply_lan_update_mirror_migrations;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("0.9.0", "1.0.0"));
        assert!(is_newer_version("v1.0.0", "v1.0.1"));
        assert!(!is_newer_version("1.1.0", "1.1.0"));
        assert!(!is_newer_version("2.0.0", "1.9.9"));
        assert!(!is_newer_version("1.1.0", "1.0.0"));
    }

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        apply_lan_update_mirror_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_mirror_config_lifecycle() {
        let conn = setup_test_db();
        let now = "2026-09-28T12:00:00Z";

        let cfg = get_or_create_mirror_config(&conn, now).unwrap();
        assert!(cfg.mirror_enabled);
        assert_eq!(cfg.max_cached_releases, 2);

        // Disable mirror
        let updated = configure_mirror(
            &conn,
            &ConfigureMirrorInput {
                mirror_enabled: false,
            },
            now,
        )
        .unwrap();
        assert!(!updated.mirror_enabled);

        let status = get_mirror_status(&conn, now, None).unwrap();
        assert!(!status.mirror_enabled);
        assert_eq!(status.total_cached_releases, 0);
    }

    #[test]
    fn test_stage_and_get_manifest() {
        let conn = setup_test_db();
        let temp_dir = tempfile::tempdir().unwrap();
        let now = "2026-09-28T12:00:00Z";

        // Create dummy installer
        let installer_path = temp_dir.path().join("Coachify-v1.1.0-Setup.exe");
        let fake_payload = b"MZ-DUMMY-WINDOWS-INSTALLER-CONTENT-FOR-COACHIFY";
        std::fs::write(&installer_path, fake_payload).unwrap();
        let expected_hash = compute_sha256(fake_payload);

        let input = StageMirrorUpdateInput {
            file_path: installer_path.to_string_lossy().to_string(),
            version: "1.1.0".to_string(),
            channel: Some("stable".to_string()),
            notes: Some("Fixes and LAN speedups".to_string()),
            expected_sha256: Some(expected_hash.clone()),
            updater_signature: Some("mock_ed25519_signature".to_string()),
            updater_key_id: Some("u2026a".to_string()),
        };

        let manifest = stage_update(&conn, &input, now, Some(temp_dir.path())).unwrap();
        assert_eq!(manifest.version, "1.1.0");
        assert_eq!(manifest.sha256, expected_hash);
        assert_eq!(manifest.file_size_bytes, fake_payload.len() as i64);

        // Fetch active manifest
        let active = get_active_manifest(&conn, now)
            .unwrap()
            .expect("manifest should be active");
        assert_eq!(active.version, "1.1.0");
        assert_eq!(active.download_path, "/updates/download/1.1.0");

        // Verify status
        let status = get_mirror_status(&conn, now, Some(temp_dir.path())).unwrap();
        assert_eq!(status.total_cached_releases, 1);
        assert_eq!(status.total_downloads_served, 0);

        // Verify physical file resolution
        let file_path = get_staged_file_path(&conn, "1.1.0", now, Some(temp_dir.path())).unwrap();
        assert!(file_path.exists());
        assert_eq!(std::fs::read(file_path).unwrap(), fake_payload);

        // Record download
        let count = record_download_completed(&conn, "1.1.0").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_checksum_mismatch_rejected() {
        let conn = setup_test_db();
        let temp_dir = tempfile::tempdir().unwrap();
        let now = "2026-09-28T12:00:00Z";

        let installer_path = temp_dir.path().join("corrupted.exe");
        std::fs::write(&installer_path, b"ORIGINAL_BYTES").unwrap();

        let input = StageMirrorUpdateInput {
            file_path: installer_path.to_string_lossy().to_string(),
            version: "1.1.0".to_string(),
            channel: None,
            notes: None,
            expected_sha256: Some("bad_hash_0000000000000000000000000000000000000000".to_string()),
            updater_signature: None,
            updater_key_id: None,
        };

        let result = stage_update(&conn, &input, now, Some(temp_dir.path()));
        match result {
            Err(LanError::UpdateChecksumMismatch { .. }) => {}
            other => panic!("Expected UpdateChecksumMismatch, got {:?}", other),
        }
    }
}
