use std::path::PathBuf;

#[test]
fn test_released_migrations_guard() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let result = coachify_core::migrations::verify_released_migrations(workspace_root);
    assert!(
        result.is_ok(),
        "Released migrations check failed: {:?}",
        result.err()
    );
}

#[test]
fn test_detects_tampered_migration() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    // Create a temporary mock directory with tampered content
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let temp_path = temp_dir.path();
    let temp_migrations = temp_path.join("migrations");
    std::fs::create_dir_all(&temp_migrations).expect("create_dir_all");

    // Copy released.json from real migrations
    let real_released = workspace_root.join("migrations").join("released.json");
    std::fs::copy(&real_released, temp_migrations.join("released.json"))
        .expect("copy released.json");

    // Write tampered migration file
    std::fs::write(
        temp_migrations.join("0001_scaffold.sql"),
        "-- TAMPERED CONTENT\nSELECT 1;",
    )
    .expect("write tampered file");

    let result = coachify_core::migrations::verify_released_migrations(temp_path);
    assert!(
        result.is_err(),
        "Expected error when migration is tampered with"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("has been modified"),
        "Error message should explain modification: {}",
        err
    );
}
