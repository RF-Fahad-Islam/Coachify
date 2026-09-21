use coachify_core::audit::{
    get_audit_head, list_audit_logs, record_audit_entry, verify_audit_log_chain, AuditEntry,
    ListAuditLogsPayload, GENESIS_PREV_HASH,
};
use coachify_core::db;
use coachify_core::dispatcher::{self, RpcEnvelope};
use rusqlite::Connection;
use serde_json::json;
use tempfile::TempDir;

fn setup_test_db() -> (TempDir, Connection) {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("test_audit.db");
    let conn = db::open_connection_at(&db_path).expect("open connection");
    db::apply_all_migrations(&conn).expect("apply migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
) -> (serde_json::Value, Vec<dispatcher::ChangeEvent>) {
    let payload_str = payload.to_string();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, None);
    let envelope: RpcEnvelope<serde_json::Value> =
        serde_json::from_str(&res_str).expect("parse envelope");
    let val = serde_json::to_value(envelope).expect("to value");
    (val, events)
}

#[test]
fn test_genesis_record_and_hash_chaining() {
    let (_dir, conn) = setup_test_db();

    // 1. Record genesis entry
    let entry1 = AuditEntry {
        action: "users.create".to_string(),
        entity: "user".to_string(),
        entity_id: "user-1".to_string(),
        before_json: None,
        after_json: Some(json!({ "username": "alice", "role": "Teacher" })),
    };
    record_audit_entry(&conn, entry1, None, "2026-09-20T10:00:00Z").expect("record entry 1");

    // 2. Record second entry
    let entry2 = AuditEntry {
        action: "users.update".to_string(),
        entity: "user".to_string(),
        entity_id: "user-1".to_string(),
        before_json: Some(json!({ "display_name": "Alice" })),
        after_json: Some(json!({ "display_name": "Alice Smith" })),
    };
    record_audit_entry(&conn, entry2, None, "2026-09-20T10:05:00Z").expect("record entry 2");

    // 3. Record third entry
    let entry3 = AuditEntry {
        action: "roles.create".to_string(),
        entity: "role".to_string(),
        entity_id: "role-1".to_string(),
        before_json: None,
        after_json: Some(json!({ "name": "Counselor" })),
    };
    record_audit_entry(&conn, entry3, None, "2026-09-20T10:10:00Z").expect("record entry 3");

    // Verify row 1 prev_hash is GENESIS_PREV_HASH (64 zeros)
    let (seq1, prev1, hash1): (i64, String, String) = conn
        .query_row(
            "SELECT seq, prev_hash, hash FROM audit_log WHERE seq = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("query row 1");
    assert_eq!(seq1, 1);
    assert_eq!(prev1, GENESIS_PREV_HASH);
    assert_eq!(prev1.len(), 64);
    assert_eq!(hash1.len(), 64);

    // Verify row 2 links to row 1
    let (seq2, prev2, hash2): (i64, String, String) = conn
        .query_row(
            "SELECT seq, prev_hash, hash FROM audit_log WHERE seq = 2",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("query row 2");
    assert_eq!(seq2, 2);
    assert_eq!(prev2, hash1, "Row 2 prev_hash must match Row 1 hash");

    // Verify row 3 links to row 2
    let (seq3, prev3, hash3): (i64, String, String) = conn
        .query_row(
            "SELECT seq, prev_hash, hash FROM audit_log WHERE seq = 3",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .expect("query row 3");
    assert_eq!(seq3, 3);
    assert_eq!(prev3, hash2, "Row 3 prev_hash must match Row 2 hash");

    // Verify chain diagnostics reports valid
    let report = verify_audit_log_chain(&conn).expect("verify chain");
    assert!(report.is_valid);
    assert_eq!(report.total_records, 3);
    assert_eq!(report.head_seq, Some(3));
    assert_eq!(report.head_hash, Some(hash3.clone()));
    assert!(report.broken_seq.is_none());

    // Verify get_audit_head matches
    let head = get_audit_head(&conn).expect("get head");
    assert_eq!(head.total_records, 3);
    assert_eq!(head.head_seq, Some(3));
    assert_eq!(head.head_hash, Some(hash3));
}

#[test]
fn test_sensitive_field_redaction_in_audit_log() {
    let (_dir, conn) = setup_test_db();

    let entry = AuditEntry {
        action: "users.create".to_string(),
        entity: "user".to_string(),
        entity_id: "user-secret".to_string(),
        before_json: None,
        after_json: Some(json!({
            "username": "secret_user",
            "password": "RawPlainPassword123!",
            "password_hash": "$argon2id$v=19$m=65536,t=3,p=4$...",
            "session_token": "secret_session_bearer_token_value",
            "token_hash": "hashed_token_abc123",
            "secret_ref": "private_secret_key",
            "display_name": "Secret User",
            "safe_metadata": {
                "department": "IT",
                "nested_password": "AnotherSecretPassword!"
            }
        })),
    };

    record_audit_entry(&conn, entry, None, "2026-09-20T10:00:00Z").expect("record entry");

    let after_json_str: String = conn
        .query_row("SELECT after_json FROM audit_log WHERE seq = 1", [], |r| {
            r.get(0)
        })
        .expect("query row");

    let parsed: serde_json::Value = serde_json::from_str(&after_json_str).expect("parse json");

    // Sensitive keys must be redacted
    assert_eq!(parsed["password"], "[REDACTED]");
    assert_eq!(parsed["password_hash"], "[REDACTED]");
    assert_eq!(parsed["session_token"], "[REDACTED]");
    assert_eq!(parsed["token_hash"], "[REDACTED]");
    assert_eq!(parsed["secret_ref"], "[REDACTED]");
    assert_eq!(parsed["safe_metadata"]["nested_password"], "[REDACTED]");

    // Non-sensitive keys must be preserved
    assert_eq!(parsed["username"], "secret_user");
    assert_eq!(parsed["display_name"], "Secret User");
    assert_eq!(parsed["safe_metadata"]["department"], "IT");

    // Chain must still be valid
    let report = verify_audit_log_chain(&conn).expect("verify chain");
    assert!(report.is_valid);
}

#[test]
fn test_sqlite_triggers_reject_audit_log_modifications() {
    let (_dir, conn) = setup_test_db();

    let entry = AuditEntry {
        action: "users.create".to_string(),
        entity: "user".to_string(),
        entity_id: "user-trigger".to_string(),
        before_json: None,
        after_json: Some(json!({ "name": "Trigger Test" })),
    };
    record_audit_entry(&conn, entry, None, "2026-09-20T10:00:00Z").expect("record entry");

    // Attempt UPDATE -> must be aborted by trg_audit_log_no_update
    let upd_res = conn.execute("UPDATE audit_log SET action = 'tampered' WHERE seq = 1", []);
    assert!(upd_res.is_err());
    let upd_err = upd_res.unwrap_err().to_string();
    assert!(upd_err.contains("Audit log records are immutable and cannot be updated"));

    // Attempt DELETE -> must be aborted by trg_audit_log_no_delete
    let del_res = conn.execute("DELETE FROM audit_log WHERE seq = 1", []);
    assert!(del_res.is_err());
    let del_err = del_res.unwrap_err().to_string();
    assert!(del_err.contains("Audit log records are immutable and cannot be deleted"));
}

#[test]
fn test_diagnostics_detects_tampered_payload() {
    let (_dir, conn) = setup_test_db();

    for i in 1..=3 {
        let entry = AuditEntry {
            action: format!("action_{}", i),
            entity: "user".to_string(),
            entity_id: format!("user-{}", i),
            before_json: None,
            after_json: Some(json!({ "val": i })),
        };
        record_audit_entry(&conn, entry, None, &format!("2026-09-20T10:0{}:00Z", i))
            .expect("record entry");
    }

    // Verify valid before tampering
    assert!(verify_audit_log_chain(&conn).unwrap().is_valid);

    // Bypass trigger to simulate raw file modification by an external tool
    conn.execute("DROP TRIGGER trg_audit_log_no_update", [])
        .unwrap();
    conn.execute(
        "UPDATE audit_log SET after_json = '{\"val\":999}' WHERE seq = 2",
        [],
    )
    .unwrap();

    // Verify diagnostics flags tampered payload
    let report = verify_audit_log_chain(&conn).expect("verify chain");
    assert!(!report.is_valid, "Tampered payload must be detected");
    assert_eq!(report.broken_seq, Some(2));
    assert!(report
        .error_message
        .unwrap()
        .contains("Audit record payload tampered at seq 2"));
}

#[test]
fn test_diagnostics_detects_deleted_row_sequence_gap() {
    let (_dir, conn) = setup_test_db();

    for i in 1..=3 {
        let entry = AuditEntry {
            action: format!("action_{}", i),
            entity: "user".to_string(),
            entity_id: format!("user-{}", i),
            before_json: None,
            after_json: Some(json!({ "val": i })),
        };
        record_audit_entry(&conn, entry, None, &format!("2026-09-20T10:0{}:00Z", i))
            .expect("record entry");
    }

    // Bypass trigger and delete row 2
    conn.execute("DROP TRIGGER trg_audit_log_no_delete", [])
        .unwrap();
    conn.execute("DELETE FROM audit_log WHERE seq = 2", [])
        .unwrap();

    let report = verify_audit_log_chain(&conn).expect("verify chain");
    assert!(!report.is_valid, "Deleted row gap must be detected");
    assert_eq!(report.broken_seq, Some(3));
    assert!(report
        .error_message
        .unwrap()
        .contains("Sequence gap detected: expected seq 2, found 3"));
}

#[test]
fn test_dispatcher_mutation_commands_write_audit_rows() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap Owner -> must write audit row
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_audit",
            "display_name": "Audit Owner",
            "password": "Password123!"
        }),
        None,
    );
    assert_eq!(res["ok"], true);
    let owner_token = res["data"]["session_token"].as_str().unwrap().to_string();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'system.bootstrap_owner'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "system.bootstrap_owner must write an audit row");

    // 2. Create User -> must write audit row
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "teacher_bob",
            "display_name": "Bob Teacher",
            "password": "Password123!",
            "role_ids": ["role-teacher-00000000000000000000"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);
    let bob_id = res["data"]["id"].as_str().unwrap().to_string();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'users.create'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "users.create must write an audit row");

    // 3. Update User -> must write audit row with before and after JSON
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": bob_id,
            "display_name": "Robert Teacher",
            "status": "active",
            "role_ids": ["role-teacher-00000000000000000000"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);

    let (before_json, after_json): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT before_json, after_json FROM audit_log WHERE action = 'users.update' ORDER BY seq DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert!(before_json.is_some());
    assert!(after_json.is_some());
    assert!(before_json.unwrap().contains("Bob Teacher"));
    assert!(after_json.unwrap().contains("Robert Teacher"));

    // 4. Change Password -> must write audit row with password NOT in JSON
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.change_password",
        json!({
            "current_password": "Password123!",
            "new_password": "NewSecretPassword123!"
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);

    let after_json: String = conn
        .query_row(
            "SELECT after_json FROM audit_log WHERE action = 'auth.change_password' ORDER BY seq DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(!after_json.contains("NewSecretPassword123!"));
    assert!(!after_json.contains("Password123!"));

    // Password change revokes all active sessions for the user. Re-login with new password.
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "owner_audit",
            "password": "NewSecretPassword123!"
        }),
        None,
    );
    assert_eq!(res["ok"], true);
    let owner_token = res["data"]["session_token"].as_str().unwrap().to_string();

    // 5. Create Role -> must write audit row
    let (res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Staff Coordinator",
            "description": "Coordinates teaching schedules",
            "permission_codes": ["academics.view"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'roles.create'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // 6. Verify full chain from real commands is 100% cryptographically valid
    let report = verify_audit_log_chain(&conn).expect("verify chain");
    assert!(
        report.is_valid,
        "Full chain from dispatcher commands must be valid"
    );
    assert!(report.total_records >= 5);
}

#[test]
fn test_transaction_rollback_prevents_orphan_audit_rows() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_tx",
            "display_name": "Tx Owner",
            "password": "Password123!"
        }),
        None,
    );
    let owner_token = res["data"]["session_token"].as_str().unwrap().to_string();

    let initial_count = get_audit_head(&conn).unwrap().total_records;

    // Trigger a failed mutation (e.g. creating user with invalid/non-existent role)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "failed_user",
            "display_name": "Failed",
            "password": "Password123!",
            "role_ids": ["non_existent_role_id_xyz"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], false);

    // Assert audit count did NOT increase (no orphan rows)
    let after_count = get_audit_head(&conn).unwrap().total_records;
    assert_eq!(
        initial_count, after_count,
        "Failed transaction must not leave orphan audit row"
    );

    // Subsequent successful command works with unbroken chain
    let (res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Valid Role",
            "permission_codes": ["students.view"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);

    let report = verify_audit_log_chain(&conn).unwrap();
    assert!(report.is_valid);
}

#[test]
fn test_audit_rpc_commands_and_permissions() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap Owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_admin",
            "display_name": "Owner Admin",
            "password": "Password123!"
        }),
        None,
    );
    let owner_token = res["data"]["session_token"].as_str().unwrap().to_string();

    // 2. Create Teacher user (does NOT have audit.view permission)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "teacher_tina",
            "display_name": "Tina Teacher",
            "password": "Password123!",
            "role_ids": ["role-teacher-00000000000000000000"]
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);

    // Login as Teacher
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "teacher_tina",
            "password": "Password123!"
        }),
        None,
    );
    assert_eq!(res["ok"], true);
    let teacher_token = res["data"]["session_token"].as_str().unwrap().to_string();

    // 3. Teacher invoking audit.list -> must receive forbidden
    let (res, _) = dispatch_cmd(&mut conn, "audit.list", json!({}), Some(&teacher_token));
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");

    // Teacher invoking audit.verify_chain -> must receive forbidden
    let (res, _) = dispatch_cmd(
        &mut conn,
        "audit.verify_chain",
        json!({}),
        Some(&teacher_token),
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");

    // 4. Owner invoking audit.list -> succeeds and returns items with total_count
    let (res, _) = dispatch_cmd(
        &mut conn,
        "audit.list",
        json!({
            "limit": 10,
            "offset": 0
        }),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);
    let items = res["data"]["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(res["data"]["total_count"].as_i64().unwrap() >= 2);

    // 5. Owner invoking audit.verify_chain -> succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "audit.verify_chain",
        json!({}),
        Some(&owner_token),
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["is_valid"], true);

    // 6. Owner invoking audit.get_head -> succeeds
    let (res, _) = dispatch_cmd(&mut conn, "audit.get_head", json!({}), Some(&owner_token));
    assert_eq!(res["ok"], true);
    assert!(res["data"]["head_seq"].as_i64().unwrap() >= 2);
    assert!(res["data"]["head_hash"].is_string());
}

#[test]
fn test_audit_list_filtering() {
    let (_dir, conn) = setup_test_db();

    // Insert test rows across different entities and actions
    let entry1 = AuditEntry {
        action: "users.create".to_string(),
        entity: "user".to_string(),
        entity_id: "user-abc".to_string(),
        before_json: None,
        after_json: Some(json!({ "username": "abc" })),
    };
    record_audit_entry(&conn, entry1, None, "2026-09-20T10:00:00Z").unwrap();

    let entry2 = AuditEntry {
        action: "roles.create".to_string(),
        entity: "role".to_string(),
        entity_id: "role-xyz".to_string(),
        before_json: None,
        after_json: Some(json!({ "name": "XYZ" })),
    };
    record_audit_entry(&conn, entry2, None, "2026-09-20T10:05:00Z").unwrap();

    // Filter by entity "role"
    let res = list_audit_logs(
        &conn,
        &ListAuditLogsPayload {
            entity: Some("role".to_string()),
            entity_id: None,
            action: None,
            user_id: None,
            date_from: None,
            date_to: None,
            limit: None,
            offset: None,
        },
    )
    .unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].entity, "role");
    assert_eq!(res.items[0].entity_id, "role-xyz");

    // Filter by action "users."
    let res = list_audit_logs(
        &conn,
        &ListAuditLogsPayload {
            entity: None,
            entity_id: None,
            action: Some("users.".to_string()),
            user_id: None,
            date_from: None,
            date_to: None,
            limit: None,
            offset: None,
        },
    )
    .unwrap();

    assert_eq!(res.items.len(), 1);
    assert_eq!(res.items[0].action, "users.create");
}
