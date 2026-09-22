use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use serde_json::json;
use std::path::Path;
use tempfile::tempdir;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_backup_and_safety.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply all migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut rusqlite::Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
    idem_key: Option<&str>,
) -> (serde_json::Value, Vec<ChangeEvent>) {
    let payload_str = serde_json::to_string(&payload).unwrap();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, idem_key);
    let val: serde_json::Value = serde_json::from_str(&res_str).unwrap();
    (val, events)
}

fn bootstrap_and_login(conn: &mut rusqlite::Connection) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner",
            "display_name": "Institution Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "bootstrap failed: {:?}", res);

    let (res, _) = dispatch_cmd(
        conn,
        "auth.login",
        json!({
            "username": "owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

#[test]
fn test_slice3_backup_and_diagnostics_integration() {
    let (_dir, mut conn) = setup_test_db();
    let owner_token = bootstrap_and_login(&mut conn);
    let dest_dir = tempdir().expect("dest dir");

    // 1. Initial dashboard
    let (dash, _) = dispatch_cmd(
        &mut conn,
        "backup.get_dashboard",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(dash["ok"], true);
    assert!(dash["data"]["last_backup"].is_null());
    assert!(!dash["data"]["warnings"].as_array().unwrap().is_empty());

    // 2. Save destination
    let (save_dest, _) = dispatch_cmd(
        &mut conn,
        "backup.save_destination",
        json!({
            "path": dest_dir.path().display().to_string(),
            "kind": "local",
            "label": "Primary Local Backup"
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(save_dest["ok"], true);
    let dest_id = save_dest["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(save_dest["data"]["kind"], "local");
    assert_eq!(save_dest["data"]["label"], "Primary Local Backup");

    // Marker verification
    assert!(dest_dir.path().join(".coachify-backup-target").exists());

    // 3. List destinations
    let (list_dest, _) = dispatch_cmd(
        &mut conn,
        "backup.list_destinations",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(list_dest["ok"], true);
    assert_eq!(list_dest["data"].as_array().unwrap().len(), 1);

    // 4. Create snapshot
    let (snap, _) = dispatch_cmd(
        &mut conn,
        "backup.create_snapshot",
        json!({
            "destination_id": dest_id,
            "kind": "manual"
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(snap["ok"], true, "create_snapshot failed: {:?}", snap);
    let backup_id = snap["data"]["id"].as_str().unwrap().to_string();
    let file_path = snap["data"]["file_path"].as_str().unwrap().to_string();
    assert_eq!(snap["data"]["status"], "completed");
    assert!(snap["data"]["file_size_bytes"].as_u64().unwrap() > 0);
    assert_eq!(
        snap["data"]["schema_version"],
        coachify_core::migrations::TARGET_SCHEMA_VERSION
    );
    assert!(Path::new(&file_path).exists());

    // 5. Verify snapshot
    let (verify, _) = dispatch_cmd(
        &mut conn,
        "backup.verify",
        json!({
            "backup_id": backup_id
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(verify["ok"], true);
    assert_eq!(verify["data"]["passed"], true);

    // 6. Preview backup
    let (preview, _) = dispatch_cmd(
        &mut conn,
        "restore.preview_backup",
        json!({
            "file_path": file_path
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(preview["ok"], true);
    assert_eq!(
        preview["data"]["schema_version"],
        coachify_core::migrations::TARGET_SCHEMA_VERSION
    );
    assert_eq!(preview["data"]["integrity_ok"], true);

    // 7. List history
    let (hist, _) = dispatch_cmd(
        &mut conn,
        "backup.list_history",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(hist["ok"], true);
    let history = hist["data"].as_array().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["status"], "verified");

    // 8. Schedules
    let (save_sched, _) = dispatch_cmd(
        &mut conn,
        "backup.save_schedule",
        json!({
            "kind": "daily",
            "destination_id": dest_id,
            "time_of_day": "23:00",
            "enabled": true
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(save_sched["ok"], true);

    let (get_sched, _) = dispatch_cmd(
        &mut conn,
        "backup.get_schedules",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(get_sched["ok"], true);
    assert_eq!(get_sched["data"].as_array().unwrap().len(), 1);

    // 9. System health check
    let (health, _) = dispatch_cmd(
        &mut conn,
        "system.health_check",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(health["ok"], true);
    assert_eq!(health["data"]["healthy"], true);

    // 10. Diagnostics get_info
    let (diag, _) = dispatch_cmd(
        &mut conn,
        "diagnostics.get_info",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(diag["ok"], true);
    assert_eq!(
        diag["data"]["schema_version"],
        coachify_core::migrations::TARGET_SCHEMA_VERSION
    );
    assert_eq!(diag["data"]["integrity_status"], "ok");
    assert!(!diag["data"]["table_stats"].as_array().unwrap().is_empty());

    // 11. Diagnostics export bundle
    let bundle_dest = tempdir().expect("bundle dest");
    let (export, _) = dispatch_cmd(
        &mut conn,
        "diagnostics.export_bundle",
        json!({
            "destination_path": bundle_dest.path().display().to_string()
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(export["ok"], true);
    let bundle_file = export["data"]["bundle_path"].as_str().unwrap();
    assert!(Path::new(bundle_file).exists());

    // 12. Execute restore from backup snapshot
    let (restore_res, _) = dispatch_cmd(
        &mut conn,
        "restore.execute",
        json!({
            "file_path": file_path,
            "recovery_key": null
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(restore_res["ok"], true, "restore failed: {:?}", restore_res);
    assert_eq!(restore_res["data"]["success"], true);
    assert!(restore_res["data"]["safety_backup_id"].is_string());

    // Verify restore drill was recorded
    let (dash_post_restore, _) = dispatch_cmd(
        &mut conn,
        "backup.get_dashboard",
        json!({}),
        Some(&owner_token),
        None,
    );
    assert_eq!(dash_post_restore["ok"], true);
    assert!(dash_post_restore["data"]["last_restore_drill"].is_object());
}

#[test]
fn test_slice3_permission_enforcement() {
    let (_dir, mut conn) = setup_test_db();
    let owner_token = bootstrap_and_login(&mut conn);

    // Find Cashier role
    let (roles_res, _) = dispatch_cmd(&mut conn, "roles.list", json!({}), Some(&owner_token), None);
    let cashier_role_id = roles_res["data"]["roles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "Cashier")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create a cashier user
    let (create_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "cashier1",
            "display_name": "Counter Cashier",
            "password": "CashierPassword123!",
            "role_ids": [cashier_role_id]
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(create_res["ok"], true);

    // Login as cashier
    let (login_res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "cashier1",
            "password": "CashierPassword123!"
        }),
        None,
        None,
    );
    assert_eq!(login_res["ok"], true);
    let cashier_token = login_res["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Cashier attempting to create backup -> forbidden
    let (snap_res, _) = dispatch_cmd(
        &mut conn,
        "backup.create_snapshot",
        json!({
            "destination_id": "some-dest"
        }),
        Some(&cashier_token),
        None,
    );
    assert_eq!(snap_res["ok"], false);
    assert_eq!(snap_res["error"]["code"], "forbidden");

    // Cashier attempting to export diagnostics -> forbidden
    let (export_res, _) = dispatch_cmd(
        &mut conn,
        "diagnostics.export_bundle",
        json!({
            "destination_path": "C:\\tmp"
        }),
        Some(&cashier_token),
        None,
    );
    assert_eq!(export_res["ok"], false);
    assert_eq!(export_res["error"]["code"], "forbidden");

    // Unauthenticated user attempting backup -> unauthenticated
    let (unauth_res, _) = dispatch_cmd(
        &mut conn,
        "backup.create_snapshot",
        json!({
            "destination_id": "some-dest"
        }),
        None,
        None,
    );
    assert_eq!(unauth_res["ok"], false);
    assert_eq!(unauth_res["error"]["code"], "unauthenticated");
}
