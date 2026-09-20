use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize)]
struct ReleasedMigrationEntry {
    version: u32,
    filename: String,
    sha256: String,
}

#[derive(serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_released_migrations_are_intact() {
        // Cargo tests run with CARGO_MANIFEST_DIR = crates/core
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
}
