use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::{auth, db};
use rusqlite::params;
use serde_json::json;
use tempfile::tempdir;
use uuid::Uuid;

fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_accounts.db");
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

// =============================================================================
// AC-1: Centralized typed rpc command dispatcher
// =============================================================================

#[test]
fn test_ac1_command_dispatcher_basics() {
    let (_dir, mut conn) = setup_test_db();

    // 1. system.ping (Public, Read)
    let (res, events) = dispatch_cmd(
        &mut conn,
        "system.ping",
        json!({ "message": "Test Ping" }),
        None,
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["status"], "ok");
    assert!(res["data"]["echo"].as_str().unwrap().contains("Test Ping"));
    assert!(events.is_empty(), "Read commands emit no events");

    // 2. system.get_status before bootstrap
    let (res, _) = dispatch_cmd(&mut conn, "system.get_status", json!({}), None, None);
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["needs_setup"], true);

    // 3. Unknown command
    let (res, _) = dispatch_cmd(&mut conn, "unknown.command", json!({}), None, None);
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "unknown_command");

    // 4. Unauthenticated access to protected command
    let (res, _) = dispatch_cmd(&mut conn, "users.list", json!({}), None, None);
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "unauthenticated");
}

// =============================================================================
// AC-2: Argon2id password hashing, lockout, idle lock, 24h cap, clock tampering
// =============================================================================

#[test]
fn test_ac2_password_hashing_argon2id() {
    let password = "SuperSecretPassword123!";
    let hash = auth::hash_password(password).expect("hash password with argon2id");

    // PHC string format check: starts with $argon2id$v=19$m=65536,t=3,p=4$
    assert!(
        hash.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"),
        "Hash must be Argon2id with m=65536, t=3, p=4. Got: {}",
        hash
    );

    // Verification check
    let valid = auth::verify_password(password, &hash).expect("verify password");
    assert!(valid, "Valid password must verify successfully");

    let invalid = auth::verify_password("WrongPassword!", &hash).expect("verify password");
    assert!(!invalid, "Invalid password must fail verification");
}

#[test]
fn test_ac2_failed_attempt_lockout_and_decay() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap initial owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_user",
            "display_name": "Institute Owner",
            "password": "ValidPassword123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true);

    // 2. Perform 4 consecutive failed login attempts
    for attempt in 1..=4 {
        let (res, _) = dispatch_cmd(
            &mut conn,
            "auth.login",
            json!({
                "username": "owner_user",
                "password": "WrongPassword!"
            }),
            None,
            None,
        );
        assert_eq!(res["ok"], false, "Attempt {} should fail", attempt);
        assert_eq!(res["error"]["code"], "invalid_credentials");
    }

    // Verify failed_login_attempts is 4 in database
    let failed_count: i64 = conn
        .query_row(
            "SELECT failed_login_attempts FROM users WHERE username = 'owner_user'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(failed_count, 4);

    // 3. 5th consecutive failed login attempt locks the account
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "owner_user",
            "password": "WrongPassword!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "invalid_credentials");

    // Locked until must now be set
    let locked_until: Option<String> = conn
        .query_row(
            "SELECT locked_until FROM users WHERE username = 'owner_user'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        locked_until.is_some(),
        "locked_until must be set after 5 failed attempts"
    );

    // Next login attempt (even with correct password) returns account_locked
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "owner_user",
            "password": "ValidPassword123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "account_locked");
    assert_eq!(res["error"]["message"], "Account is locked for 5 minutes");

    // 4. Test 24h decay: simulate last failed attempt occurred 25 hours ago
    conn.execute(
        "UPDATE users
         SET failed_login_attempts = 3,
             last_failed_attempt_at = '2026-09-19T10:00:00Z',
             locked_until = NULL
         WHERE username = 'owner_user'",
        [],
    )
    .unwrap();

    // Login with correct password now resets counter and succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "owner_user",
            "password": "ValidPassword123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true);

    let reset_count: i64 = conn
        .query_row(
            "SELECT failed_login_attempts FROM users WHERE username = 'owner_user'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        reset_count, 0,
        "Counter should be reset after successful login"
    );
}

#[test]
fn test_ac2_idle_lock_and_unlock() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap initial owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_idle",
            "display_name": "Idle Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    // Command works normally while active
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.get_current_session",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 2. Simulate 16 minutes of inactivity by updating last_seen_at in DB
    conn.execute(
        "UPDATE sessions
         SET last_seen_at = '2026-09-20T10:00:00Z',
             expires_at = '2026-09-20T10:15:00Z'
         WHERE user_id IN (SELECT id FROM users WHERE username = 'owner_idle')",
        [],
    )
    .unwrap();

    // Protected commands reject with session_idle_locked
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.get_current_session",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "session_idle_locked");

    // 3. Unlock with wrong password fails
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.unlock",
        json!({ "password": "WrongPassword!" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "invalid_credentials");

    // 4. Unlock with correct password succeeds and restores session
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.unlock",
        json!({ "password": "Password123!" }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["success"], true);

    // Subsequent command succeeds
    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.get_current_session",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
}

#[test]
fn test_ac2_absolute_session_lifespan_and_clock_tampering() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_expiry",
            "display_name": "Expiry Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    // 1. Simulate absolute expiry (> 24 hours)
    conn.execute(
        "UPDATE sessions
         SET absolute_expires_at = '2026-09-19T00:00:00Z'
         WHERE user_id IN (SELECT id FROM users WHERE username = 'owner_expiry')",
        [],
    )
    .unwrap();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.get_current_session",
        json!({}),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "session_expired");

    // 2. Simulate backwards clock tampering: created_at in the future
    let (login_res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "owner_expiry",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let new_token = login_res["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    conn.execute(
        "UPDATE sessions
         SET created_at = '2099-01-01T00:00:00Z'
         WHERE user_id IN (SELECT id FROM users WHERE username = 'owner_expiry')",
        [],
    )
    .unwrap();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "auth.get_current_session",
        json!({}),
        Some(&new_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "clock_tampering_revoked");

    // Verify session revoked_at is now set in the database for this session
    let token_hash = auth::hash_session_token(&new_token);
    let revoked_at: Option<String> = conn
        .query_row(
            "SELECT revoked_at FROM sessions WHERE token_hash = ?1",
            params![token_hash],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        revoked_at.is_some(),
        "Session must be revoked on clock tampering"
    );
}

// =============================================================================
// AC-3: Built-in system roles, custom roles, role protection triggers
// =============================================================================

#[test]
fn test_ac3_system_roles_and_custom_roles() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Check seeded system roles in database
    let system_roles: Vec<(String, i64)> = {
        let mut stmt = conn
            .prepare("SELECT name, is_system FROM roles ORDER BY name")
            .unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        rows.map(|r| r.unwrap()).collect()
    };
    let names: Vec<String> = system_roles.iter().map(|(n, _)| n.clone()).collect();
    assert!(names.contains(&"Owner".to_string()));
    assert!(names.contains(&"Cashier".to_string()));
    assert!(names.contains(&"Teacher".to_string()));

    for (name, is_sys) in &system_roles {
        if name == "Owner" || name == "Cashier" || name == "Teacher" {
            assert_eq!(*is_sys, 1, "Role '{}' must have is_system = 1", name);
        }
    }

    // 2. Check 17 seeded permissions (including audit.view, settings.view, and students.archive)
    let perm_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM permissions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        perm_count, 43,
        "Must have seeded granular permissions including financial ledger, backup/diagnostics, attendance/comms, and exam permissions"
    );

    // 3. Test trigger trg_roles_protect_system: system roles cannot be deleted
    let del_res = conn.execute("DELETE FROM roles WHERE name = 'Owner'", []);
    assert!(
        del_res.is_err(),
        "Deleting system role Owner must fail via trigger"
    );

    let del_cashier = conn.execute("DELETE FROM roles WHERE name = 'Cashier'", []);
    assert!(
        del_cashier.is_err(),
        "Deleting system role Cashier must fail via trigger"
    );

    // 4. Test trigger trg_roles_protect_owner_name: Owner cannot be renamed
    let rename_res = conn.execute(
        "UPDATE roles SET name = 'SuperAdmin' WHERE name = 'Owner'",
        [],
    );
    assert!(
        rename_res.is_err(),
        "Renaming Owner role must fail via trigger"
    );

    // 5. Bootstrap owner and test custom role creation
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "owner_roles",
            "display_name": "Roles Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap();

    let (res, events) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Academic Coordinator",
            "description": "Manages courses and batches",
            "permission_codes": ["academics.view", "academics.manage", "students.view"]
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["name"], "Academic Coordinator");
    assert_eq!(res["data"]["is_system"], false);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "roles:changed");

    let role_id = res["data"]["id"].as_str().unwrap().to_string();

    // 6. Update custom role
    let (res, _) = dispatch_cmd(
        &mut conn,
        "roles.update",
        json!({
            "id": role_id,
            "name": "Senior Academic Coordinator",
            "description": "Updated description",
            "permission_codes": ["academics.view", "academics.manage", "students.view", "reports.view"]
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["name"], "Senior Academic Coordinator");
    let perms: Vec<String> = res["data"]["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(perms.contains(&"reports.view".to_string()));
}

// =============================================================================
// AC-4: At least one active Owner invariant enforcement
// =============================================================================

#[test]
fn test_ac4_last_active_owner_invariant() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap initial owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "sole_owner",
            "display_name": "Sole Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();
    let owner_id = res["data"]["user"]["id"].as_str().unwrap().to_string();

    let owner_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Owner'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let teacher_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Teacher'", [], |r| {
            r.get(0)
        })
        .unwrap();

    // 1. Attempt to disable the only active Owner via users.update
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": owner_id,
            "display_name": "Sole Owner",
            "status": "disabled",
            "role_ids": [owner_role_id.clone()]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "last_owner_required");

    // 2. Attempt to remove Owner role from the only active Owner via users.update
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": owner_id,
            "display_name": "Sole Owner",
            "status": "active",
            "role_ids": [teacher_role_id.clone()]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "last_owner_required");

    // 3. Test SQLite trigger trg_users_protect_last_owner_delete directly
    let del_res = conn.execute("DELETE FROM users WHERE id = ?1", params![owner_id]);
    assert!(
        del_res.is_err(),
        "Direct SQL delete of last active Owner must be aborted by trigger"
    );

    // 4. Create a second Owner user
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "second_owner",
            "display_name": "Second Owner",
            "password": "Password123!",
            "role_ids": [owner_role_id.clone()]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    let second_owner_id = res["data"]["id"].as_str().unwrap().to_string();

    // Now disabling the first Owner succeeds because second Owner is active
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": owner_id,
            "display_name": "Sole Owner",
            "status": "disabled",
            "role_ids": [owner_role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["status"], "disabled");

    // Attempting to disable the second Owner now fails because it's the last active one
    let (login_res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "second_owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let second_token = login_res["data"]["session_token"].as_str().unwrap();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": second_owner_id,
            "display_name": "Second Owner",
            "status": "disabled",
            "role_ids": [teacher_role_id]
        }),
        Some(second_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "last_owner_required");
}

// =============================================================================
// AC-5: Core permission verification rejecting unauthorized requests
// =============================================================================

#[test]
fn test_ac5_permission_verification() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap initial owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "perm_owner",
            "display_name": "Perm Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let owner_token = res["data"]["session_token"].as_str().unwrap().to_string();

    let teacher_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Teacher'", [], |r| {
            r.get(0)
        })
        .unwrap();

    // 2. Create teacher account
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "teacher_user",
            "display_name": "Mr. Karim",
            "password": "TeacherPassword123!",
            "role_ids": [teacher_role_id]
        }),
        Some(&owner_token),
        None,
    );
    assert_eq!(res["ok"], true);

    // 3. Login as Teacher
    let (login_res, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "teacher_user",
            "password": "TeacherPassword123!"
        }),
        None,
        None,
    );
    assert_eq!(login_res["ok"], true);
    let teacher_token = login_res["data"]["session_token"].as_str().unwrap();

    // 4. Teacher tries to call users.list (requires users.manage permission) -> Rejected with forbidden
    let (res, _) = dispatch_cmd(
        &mut conn,
        "users.list",
        json!({}),
        Some(teacher_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");
    assert!(
        res["error"]["message"]
            .as_str()
            .unwrap()
            .contains("users.manage"),
        "Forbidden error message must identify missing permission code"
    );

    // 5. Teacher tries to call roles.create -> Rejected with forbidden
    let (res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Hacker Role",
            "permission_codes": ["users.manage"]
        }),
        Some(teacher_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");

    // 6. Owner can execute users.list successfully
    let (res, _) = dispatch_cmd(&mut conn, "users.list", json!({}), Some(&owner_token), None);
    assert_eq!(res["ok"], true);
    assert_eq!(res["data"]["total_count"], 2);
}

// =============================================================================
// AC-6: Idempotency cache, replay, mismatch error, 72h cleanup
// =============================================================================

#[test]
fn test_ac6_idempotency_cache_and_cleanup() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "idem_owner",
            "display_name": "Idem Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();
    let cashier_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Cashier'", [], |r| {
            r.get(0)
        })
        .unwrap();

    let key = Uuid::now_v7().to_string();
    let payload = json!({
        "username": "cashier_idem",
        "display_name": "Cashier Idem",
        "password": "Password123!",
        "role_ids": [cashier_role_id]
    });

    // 1. Initial execution with idempotency key
    let (first_res, first_events) = dispatch_cmd(
        &mut conn,
        "users.create",
        payload.clone(),
        Some(&token),
        Some(&key),
    );
    assert_eq!(first_res["ok"], true);
    let created_user_id = first_res["data"]["id"].as_str().unwrap();
    assert_eq!(first_events.len(), 1);

    // 2. Replay with IDENTICAL key and payload -> returns exact cached response
    let (replay_res, replay_events) =
        dispatch_cmd(&mut conn, "users.create", payload, Some(&token), Some(&key));
    assert_eq!(replay_res["ok"], true);
    assert_eq!(replay_res["data"]["id"], created_user_id);
    assert!(
        replay_events.is_empty(),
        "Cached replay must not re-emit change events"
    );

    // Verify only 1 user was actually created in the DB
    let user_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE username = 'cashier_idem'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(user_count, 1);

    // 3. Replay with SAME key but DIFFERENT payload -> idempotency_key_mismatch
    let diff_payload = json!({
        "username": "altered_user",
        "display_name": "Altered",
        "password": "Password123!",
        "role_ids": []
    });
    let (mismatch_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        diff_payload,
        Some(&token),
        Some(&key),
    );
    assert_eq!(mismatch_res["ok"], false);
    assert_eq!(mismatch_res["error"]["code"], "idempotency_key_mismatch");

    // 4. Invalid UUID format key -> invalid_idempotency_key
    let (invalid_key_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "user_xyz",
            "display_name": "XYZ",
            "password": "Password123!",
            "role_ids": []
        }),
        Some(&token),
        Some("not-a-uuid"),
    );
    assert_eq!(invalid_key_res["ok"], false);
    assert_eq!(invalid_key_res["error"]["code"], "invalid_idempotency_key");

    // 5. 72h cleanup test: insert an expired idempotency record and verify pruning
    let old_key = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO command_idempotency (idempotency_key, command_name, payload_hash, user_id, response_json, created_at)
         VALUES (?1, 'test.cmd', 'dummy_hash', (SELECT id FROM users LIMIT 1), '{}', '2026-09-01T00:00:00Z')",
        params![old_key],
    )
    .unwrap();

    let fresh_key = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO command_idempotency (idempotency_key, command_name, payload_hash, user_id, response_json, created_at)
         VALUES (?1, 'test.cmd', 'dummy_hash', (SELECT id FROM users LIMIT 1), '{}', '2026-09-20T10:00:00Z')",
        params![fresh_key],
    )
    .unwrap();

    // Dispatching any command triggers purge_expired_idempotency
    let (ping_res, _) = dispatch_cmd(&mut conn, "system.ping", json!({}), None, None);
    assert_eq!(ping_res["ok"], true);

    let old_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_idempotency WHERE idempotency_key = ?1",
            params![old_key],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        old_exists, 0,
        "Old idempotency records (> 72h) must be purged"
    );

    let fresh_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_idempotency WHERE idempotency_key = ?1",
            params![fresh_key],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        fresh_exists, 1,
        "Recent idempotency records (< 72h) must be preserved"
    );
}

// =============================================================================
// AC-7: License gate rejecting write commands when expired past grace
// =============================================================================

#[test]
fn test_ac7_license_gate_enforcement() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap owner
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "license_owner",
            "display_name": "License Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    // 1. Set license in settings table to 'expired_past_grace'
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value_json, updated_at)
         VALUES ('license', '{\"status\": \"expired_past_grace\"}', '2026-09-20T10:00:00Z')",
        [],
    )
    .unwrap();

    // 2. Read commands still succeed
    let (ping_res, _) = dispatch_cmd(&mut conn, "system.ping", json!({}), None, None);
    assert_eq!(ping_res["ok"], true);

    let (list_res, _) = dispatch_cmd(&mut conn, "users.list", json!({}), Some(&token), None);
    assert_eq!(list_res["ok"], true);

    // 3. Write commands are rejected with license_expired_read_only
    let (write_res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "New Role During Expired",
            "permission_codes": ["students.view"]
        }),
        Some(&token),
        None,
    );
    assert_eq!(write_res["ok"], false);
    assert_eq!(write_res["error"]["code"], "license_expired_read_only");

    // 4. Restore license to active: write command succeeds
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value_json, updated_at)
         VALUES ('license', '{\"status\": \"active\"}', '2026-09-20T10:00:00Z')",
        [],
    )
    .unwrap();

    let (write_res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "New Role During Active",
            "permission_codes": ["students.view"]
        }),
        Some(&token),
        None,
    );
    assert_eq!(write_res["ok"], true);
    assert_eq!(write_res["data"]["name"], "New Role During Active");
}

// =============================================================================
// AC-8: Change event emission over channel
// =============================================================================

#[test]
fn test_ac8_change_event_emission() {
    let (_dir, mut conn) = setup_test_db();

    // 1. Bootstrap owner emits users:changed
    let (res, events) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "event_owner",
            "display_name": "Event Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "users:changed");
    let owner_id = res["data"]["user"]["id"].as_str().unwrap();
    assert_eq!(events[0].entity_ids, vec![owner_id.to_string()]);

    let token = res["data"]["session_token"].as_str().unwrap();

    // 2. roles.create emits roles:changed
    let (res, events) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "Supervisor",
            "permission_codes": ["students.view"]
        }),
        Some(token),
        None,
    );
    assert_eq!(res["ok"], true);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "roles:changed");
    let role_id = res["data"]["id"].as_str().unwrap();
    assert_eq!(events[0].entity_ids, vec![role_id.to_string()]);

    // 3. Read command emits no events
    let (_, ping_events) = dispatch_cmd(&mut conn, "system.ping", json!({}), None, None);
    assert!(ping_events.is_empty());
}

#[test]
fn test_sole_active_owner_profile_update_succeeds() {
    let (_dir, mut conn) = setup_test_db();

    // Bootstrap initial owner (only 1 active Owner exists in entire database)
    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "solo_owner",
            "display_name": "Original Name",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true);
    let token = res["data"]["session_token"].as_str().unwrap().to_string();
    let owner_id = res["data"]["user"]["id"].as_str().unwrap().to_string();

    let owner_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Owner'", [], |r| {
            r.get(0)
        })
        .unwrap();

    // Update display name of sole active owner while keeping Owner role
    let (update_res, _) = dispatch_cmd(
        &mut conn,
        "users.update",
        json!({
            "id": owner_id,
            "display_name": "Updated Name",
            "status": "active",
            "role_ids": [owner_role_id]
        }),
        Some(&token),
        None,
    );
    assert_eq!(
        update_res["ok"], true,
        "Sole active owner updating display name must succeed: {:?}",
        update_res["error"]
    );
    assert_eq!(update_res["data"]["display_name"], "Updated Name");
}

#[test]
fn test_auth_unlock_failed_attempts_trigger_lockout_and_audit() {
    let (_dir, mut conn) = setup_test_db();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "lockout_user",
            "display_name": "Lockout User",
            "password": "CorrectPassword123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();

    // Incur 4 failed unlock attempts
    for _ in 1..=4 {
        let (unlock_res, _) = dispatch_cmd(
            &mut conn,
            "auth.unlock",
            json!({ "password": "BadPassword!" }),
            Some(&token),
            None,
        );
        assert_eq!(unlock_res["ok"], false);
        assert_eq!(unlock_res["error"]["code"], "invalid_credentials");
    }

    // 5th failed unlock attempt triggers 5-minute lockout
    let (unlock_res, _) = dispatch_cmd(
        &mut conn,
        "auth.unlock",
        json!({ "password": "BadPassword!" }),
        Some(&token),
        None,
    );
    assert_eq!(unlock_res["ok"], false);
    assert_eq!(unlock_res["error"]["code"], "invalid_credentials");

    // 6th attempt (even with correct password) returns account_locked
    let (unlock_res, _) = dispatch_cmd(
        &mut conn,
        "auth.unlock",
        json!({ "password": "CorrectPassword123!" }),
        Some(&token),
        None,
    );
    assert_eq!(unlock_res["ok"], false);
    assert_eq!(unlock_res["error"]["code"], "account_locked");

    // Verify auth.unlock_failed audit entries are written
    let audit_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'auth.unlock_failed'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(audit_count, 5, "5 failed unlock audit rows must be logged");
}

#[test]
fn test_owner_system_role_cannot_be_modified() {
    let (_dir, mut conn) = setup_test_db();

    let (res, _) = dispatch_cmd(
        &mut conn,
        "system.bootstrap_owner",
        json!({
            "username": "system_owner",
            "display_name": "System Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    let token = res["data"]["session_token"].as_str().unwrap().to_string();
    let owner_role_id: String = conn
        .query_row("SELECT id FROM roles WHERE name = 'Owner'", [], |r| {
            r.get(0)
        })
        .unwrap();

    // Attempting to modify Owner role is rejected with system_role_protected
    let (mod_res, _) = dispatch_cmd(
        &mut conn,
        "roles.update",
        json!({
            "id": owner_role_id,
            "name": "Owner",
            "description": "Trying to strip permissions",
            "permission_codes": ["students.view"]
        }),
        Some(&token),
        None,
    );
    assert_eq!(mod_res["ok"], false);
    assert_eq!(mod_res["error"]["code"], "system_role_protected");
}
