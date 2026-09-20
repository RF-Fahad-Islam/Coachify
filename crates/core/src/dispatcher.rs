use crate::accounts::{
    self, find_user_by_id, find_user_for_login, get_or_create_host_device, AccountsError,
    DeviceSummary, PermissionItem, ResolvedSession, RoleWithPermissions, UserSummary,
};
use crate::auth;
use crate::db;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RpcError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RpcEnvelope<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChangeEvent {
    pub topic: String,
    pub entity_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthRequirement {
    Public,
    Authenticated,
    Permission(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseGate {
    None,
    Write,
    Sms,
    Lan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Read,
    Mutation,
}

pub struct CommandMeta {
    pub name: &'static str,
    pub auth: AuthRequirement,
    pub gate: LicenseGate,
    pub kind: CommandKind,
}

pub const COMMANDS: &[CommandMeta] = &[
    CommandMeta {
        name: "system.ping",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.get_status",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.bootstrap_owner",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "auth.login",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "auth.logout",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "auth.unlock",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "auth.change_password",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "auth.get_current_session",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "auth.heartbeat",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "users.list",
        auth: AuthRequirement::Permission("users.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "users.create",
        auth: AuthRequirement::Permission("users.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "users.update",
        auth: AuthRequirement::Permission("users.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "roles.list",
        auth: AuthRequirement::Permission("roles.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "roles.create",
        auth: AuthRequirement::Permission("roles.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "roles.update",
        auth: AuthRequirement::Permission("roles.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "permissions.list",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
];

fn find_command_meta(name: &str) -> Option<&'static CommandMeta> {
    COMMANDS.iter().find(|c| c.name == name)
}

// -----------------------------------------------------------------------------
// Request / Response Payload Structs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PingPayload {
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PingResponse {
    pub status: String,
    pub database_status: String,
    pub database_path: String,
    pub roundtrip_timestamp: String,
    pub echo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStatusResponse {
    pub needs_setup: bool,
    pub institution_name: Option<String>,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BootstrapOwnerPayload {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BootstrapOwnerResponse {
    pub session_token: String,
    pub user: UserSummary,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LoginResponse {
    pub session_token: String,
    pub user: UserSummary,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UnlockPayload {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UnlockResponse {
    pub success: bool,
    pub extended_expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CurrentSessionResponse {
    pub user: UserSummary,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub expires_at: String,
    pub absolute_expires_at: String,
    pub device: DeviceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HeartbeatResponse {
    pub extended_expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListUsersPayload {
    pub status_filter: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListUsersResponse {
    pub users: Vec<UserSummary>,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateUserPayload {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub role_ids: Vec<String>,
    pub staff_profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateUserPayload {
    pub id: String,
    pub display_name: String,
    pub status: String,
    pub role_ids: Vec<String>,
    pub staff_profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListRolesResponse {
    pub roles: Vec<RoleWithPermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
    pub permission_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateRolePayload {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permission_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListPermissionsResponse {
    pub permissions: Vec<PermissionItem>,
}

// -----------------------------------------------------------------------------
// License Verification (AC-7)
// -----------------------------------------------------------------------------

pub fn is_license_expired_past_grace(conn: &Connection) -> bool {
    use rusqlite::OptionalExtension;
    let val_opt: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'license'",
            [],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    if let Some(json_str) = val_opt {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if val
                .get("expired_past_grace")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || val.get("status").and_then(|v| v.as_str()) == Some("expired_past_grace")
            {
                return true;
            }
        }
    }
    false
}

// -----------------------------------------------------------------------------
// Core Dispatch Pipeline
// -----------------------------------------------------------------------------

pub fn dispatch(
    conn: &mut Connection,
    command: &str,
    payload_json: &str,
    session_token: Option<&str>,
    idempotency_key: Option<&str>,
) -> (String, Vec<ChangeEvent>) {
    let current_utc = db::now_utc_iso();

    // 1. Command lookup
    let meta = match find_command_meta(command) {
        Some(m) => m,
        None => {
            let env: RpcEnvelope<()> = RpcEnvelope {
                ok: false,
                data: None,
                error: Some(RpcError {
                    code: "unknown_command".to_string(),
                    message: format!("Unknown command: {}", command),
                    field: None,
                }),
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        }
    };

    // 2. Periodic maintenance: prune expired idempotency rows (> 72 hours)
    let _ = accounts::purge_expired_idempotency(conn, &current_utc);

    // 3. Idempotency pre-check for mutation commands
    let mut canonical_payload_hash = String::new();
    if let Some(raw_key) = idempotency_key {
        let key = raw_key.trim();
        if !key.is_empty() {
            // Validate UUID
            if uuid::Uuid::parse_str(key).is_err() {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "invalid_idempotency_key".to_string(),
                        message: "Idempotency key must be a valid UUID".to_string(),
                        field: Some("idempotency_key".to_string()),
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }

            // Compute hash of payload
            let parsed_payload = serde_json::from_str::<serde_json::Value>(payload_json)
                .unwrap_or(serde_json::Value::Null);
            let canonical_json = serde_json::to_string(&parsed_payload).unwrap_or_default();
            canonical_payload_hash = auth::hash_session_token(&canonical_json);

            // Check if record exists
            if let Ok(Some((stored_hash, stored_response))) =
                accounts::get_idempotency_record(conn, key)
            {
                if stored_hash == canonical_payload_hash {
                    // Cached replay: return stored response directly!
                    return (stored_response, vec![]);
                } else {
                    let env: RpcEnvelope<()> = RpcEnvelope {
                        ok: false,
                        data: None,
                        error: Some(RpcError {
                            code: "idempotency_key_mismatch".to_string(),
                            message:
                                "Payload does not match original request for this idempotency key"
                                    .to_string(),
                            field: Some("idempotency_key".to_string()),
                        }),
                    };
                    return (serde_json::to_string(&env).unwrap(), vec![]);
                }
            }
        }
    }

    // 4. Session resolution
    let resolved_session: Option<ResolvedSession> = match meta.auth {
        AuthRequirement::Public => {
            // Optional session resolution for public commands
            if let Some(token) = session_token {
                if !token.is_empty() {
                    let hash = auth::hash_session_token(token);
                    accounts::resolve_session_by_token_hash(conn, &hash, &current_utc)
                        .unwrap_or(None)
                } else {
                    None
                }
            } else {
                None
            }
        }
        AuthRequirement::Authenticated | AuthRequirement::Permission(_) => {
            let Some(token) = session_token else {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "unauthenticated".to_string(),
                        message: "Authentication required for this command".to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            };

            let token_hash = auth::hash_session_token(token);
            let session_res =
                accounts::resolve_session_by_token_hash(conn, &token_hash, &current_utc);

            match session_res {
                Ok(Some(s)) => {
                    if s.is_revoked {
                        let env: RpcEnvelope<()> = RpcEnvelope {
                            ok: false,
                            data: None,
                            error: Some(RpcError {
                                code: "unauthenticated".to_string(),
                                message: "Session is revoked or account disabled".to_string(),
                                field: None,
                            }),
                        };
                        return (serde_json::to_string(&env).unwrap(), vec![]);
                    }

                    if s.is_expired {
                        let env: RpcEnvelope<()> = RpcEnvelope {
                            ok: false,
                            data: None,
                            error: Some(RpcError {
                                code: "session_expired".to_string(),
                                message: "Session has exceeded maximum lifespan (24h)".to_string(),
                                field: None,
                            }),
                        };
                        return (serde_json::to_string(&env).unwrap(), vec![]);
                    }

                    if s.is_idle_locked && command != "auth.unlock" && command != "auth.logout" {
                        let env: RpcEnvelope<()> = RpcEnvelope {
                            ok: false,
                            data: None,
                            error: Some(RpcError {
                                code: "session_idle_locked".to_string(),
                                message: "Session is idle locked due to inactivity. Password unlock required.".to_string(),
                                field: None,
                            }),
                        };
                        return (serde_json::to_string(&env).unwrap(), vec![]);
                    }

                    Some(s)
                }
                Ok(None) => {
                    let env: RpcEnvelope<()> = RpcEnvelope {
                        ok: false,
                        data: None,
                        error: Some(RpcError {
                            code: "unauthenticated".to_string(),
                            message: "Invalid session token".to_string(),
                            field: None,
                        }),
                    };
                    return (serde_json::to_string(&env).unwrap(), vec![]);
                }
                Err(AccountsError::ClockTampering) => {
                    let env: RpcEnvelope<()> = RpcEnvelope {
                        ok: false,
                        data: None,
                        error: Some(RpcError {
                            code: "clock_tampering_revoked".to_string(),
                            message: "Clock tampering detected: system time moved backwards. Session revoked.".to_string(),
                            field: None,
                        }),
                    };
                    return (serde_json::to_string(&env).unwrap(), vec![]);
                }
                Err(err) => {
                    let env: RpcEnvelope<()> = RpcEnvelope {
                        ok: false,
                        data: None,
                        error: Some(RpcError {
                            code: "database_error".to_string(),
                            message: err.to_string(),
                            field: None,
                        }),
                    };
                    return (serde_json::to_string(&env).unwrap(), vec![]);
                }
            }
        }
    };

    // 5. Permission check (AC-5)
    if let AuthRequirement::Permission(perm_code) = meta.auth {
        let Some(s) = &resolved_session else {
            let env: RpcEnvelope<()> = RpcEnvelope {
                ok: false,
                data: None,
                error: Some(RpcError {
                    code: "unauthenticated".to_string(),
                    message: "Authentication required for this command".to_string(),
                    field: None,
                }),
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        };

        let is_owner = s.roles.iter().any(|r| r == "Owner");
        let has_perm = s.permissions.iter().any(|p| p == perm_code);
        let roles_manage_fallback = command == "roles.list"
            && (s.permissions.iter().any(|p| p == "users.manage")
                || s.permissions.iter().any(|p| p == "roles.manage"));

        if !is_owner && !has_perm && !roles_manage_fallback {
            let env: RpcEnvelope<()> = RpcEnvelope {
                ok: false,
                data: None,
                error: Some(RpcError {
                    code: "forbidden".to_string(),
                    message: format!("User lacks required permission '{}'", perm_code),
                    field: Some(perm_code.to_string()),
                }),
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        }
    }

    // 6. License check (AC-7)
    if matches!(
        meta.gate,
        LicenseGate::Write | LicenseGate::Sms | LicenseGate::Lan
    ) && is_license_expired_past_grace(conn)
    {
        let env: RpcEnvelope<()> = RpcEnvelope {
            ok: false,
            data: None,
            error: Some(RpcError {
                code: "license_expired_read_only".to_string(),
                message: "Institution license has expired past the 14-day grace period. Database is in read-only mode.".to_string(),
                field: None,
            }),
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

    // Parse payload JSON into serde_json::Value
    let payload_val: serde_json::Value = match serde_json::from_str(payload_json) {
        Ok(v) => v,
        Err(_) => serde_json::Value::Object(serde_json::Map::new()),
    };

    // 7. Execute Command Handler
    let result = execute_command(
        conn,
        command,
        meta.kind,
        &payload_val,
        resolved_session.as_ref(),
        &current_utc,
    );

    match result {
        Ok((data_val, events)) => {
            let env = RpcEnvelope {
                ok: true,
                data: Some(data_val),
                error: None,
            };
            let response_json = serde_json::to_string(&env).unwrap();

            // Save idempotency record if key provided
            if let Some(raw_key) = idempotency_key {
                let key = raw_key.trim();
                if !key.is_empty() && meta.kind == CommandKind::Mutation {
                    let user_id = resolved_session
                        .as_ref()
                        .map(|s| s.user.id.as_str())
                        .unwrap_or("system");
                    let _ = accounts::save_idempotency_record(
                        conn,
                        key,
                        command,
                        &canonical_payload_hash,
                        user_id,
                        &response_json,
                        &current_utc,
                    );
                }
            }

            (response_json, events)
        }
        Err(err) => {
            let env: RpcEnvelope<()> = RpcEnvelope {
                ok: false,
                data: None,
                error: Some(err),
            };
            (serde_json::to_string(&env).unwrap(), vec![])
        }
    }
}

fn execute_command(
    conn: &mut Connection,
    command: &str,
    kind: CommandKind,
    payload: &serde_json::Value,
    session: Option<&ResolvedSession>,
    current_utc: &str,
) -> Result<(serde_json::Value, Vec<ChangeEvent>), RpcError> {
    match kind {
        CommandKind::Read => run_command_handler(conn, command, payload, session, current_utc),
        CommandKind::Mutation => {
            if command == "auth.login" {
                return run_command_handler(conn, command, payload, session, current_utc);
            }

            // Run inside immediate transaction
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let (val, events) = run_command_handler(&tx, command, payload, session, current_utc)?;

            tx.commit().map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            Ok((val, events))
        }
    }
}

fn run_command_handler(
    conn: &Connection,
    command: &str,
    payload: &serde_json::Value,
    session: Option<&ResolvedSession>,
    current_utc: &str,
) -> Result<(serde_json::Value, Vec<ChangeEvent>), RpcError> {
    match command {
        "system.ping" => {
            let ping_payload: PingPayload =
                serde_json::from_value(payload.clone()).unwrap_or(PingPayload { message: None });
            let db_path = db::resolve_dev_db_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            let echo_msg = ping_payload.message.unwrap_or_else(|| "ping".to_string());
            let resp = PingResponse {
                status: "ok".to_string(),
                database_status: "active".to_string(),
                database_path: db_path.clone(),
                roundtrip_timestamp: current_utc.to_string(),
                echo: Some(format!("echo: {} [db: {}]", echo_msg, db_path)),
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "system.get_status" => {
            use rusqlite::OptionalExtension;
            let total_users = accounts::count_total_users(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let inst_name: Option<String> = conn
                .query_row("SELECT name FROM institution WHERE id = 1", [], |row| {
                    row.get(0)
                })
                .optional()
                .unwrap_or(None);

            let resp = GetStatusResponse {
                needs_setup: total_users == 0,
                institution_name: inst_name,
                app_version: "1.0.0".to_string(),
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "system.bootstrap_owner" => {
            let p: BootstrapOwnerPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid bootstrap payload: {}", e),
                    field: None,
                })?;

            let total_users = accounts::count_total_users(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            if total_users > 0 {
                return Err(RpcError {
                    code: "setup_already_completed".to_string(),
                    message: "Initial setup has already been completed".to_string(),
                    field: None,
                });
            }

            let hash = auth::hash_password(&p.password).map_err(|e| RpcError {
                code: "weak_password".to_string(),
                message: e.to_string(),
                field: Some("password".to_string()),
            })?;

            let owner_role = accounts::list_roles(conn)
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?
                .into_iter()
                .find(|r| r.name == "Owner")
                .ok_or_else(|| RpcError {
                    code: "database_error".to_string(),
                    message: "Owner role is missing from system roles".to_string(),
                    field: None,
                })?;

            let user = accounts::create_user(
                conn,
                &p.username,
                &p.display_name,
                &hash,
                &[owner_role.id],
                None,
                current_utc,
            )
            .map_err(|e| match e {
                AccountsError::UsernameTaken(u) => RpcError {
                    code: "username_taken".to_string(),
                    message: format!("Username '{}' is already taken", u),
                    field: Some("username".to_string()),
                },
                other => RpcError {
                    code: "database_error".to_string(),
                    message: other.to_string(),
                    field: None,
                },
            })?;

            let device = get_or_create_host_device(conn, current_utc).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let raw_token = auth::generate_session_token();
            let token_hash = auth::hash_session_token(&raw_token);
            let session_rec =
                accounts::create_session(conn, &user.id, &device.id, &token_hash, current_utc)
                    .map_err(|e| RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;

            let resp = BootstrapOwnerResponse {
                session_token: raw_token,
                user: user.clone(),
                expires_at: session_rec.expires_at,
            };

            let events = vec![ChangeEvent {
                topic: "users:changed".to_string(),
                entity_ids: vec![user.id],
            }];

            Ok((serde_json::to_value(resp).unwrap(), events))
        }

        "auth.login" => {
            let p: LoginPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid login payload: {}", e),
                    field: None,
                })?;

            let user_record = find_user_for_login(conn, &p.username).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let (summary, password_hash, _, _, locked_until) = match user_record {
                Some(u) => u,
                None => {
                    return Err(RpcError {
                        code: "invalid_credentials".to_string(),
                        message: "Invalid username or password".to_string(),
                        field: None,
                    });
                }
            };

            if summary.status == "disabled" {
                return Err(RpcError {
                    code: "account_disabled".to_string(),
                    message: "Account has been disabled".to_string(),
                    field: None,
                });
            }

            if let Some(lock_time) = locked_until {
                if accounts::is_after(&lock_time, current_utc) {
                    return Err(RpcError {
                        code: "account_locked".to_string(),
                        message: format!("Account is temporarily locked until {}", lock_time),
                        field: None,
                    });
                }
            }

            let valid =
                auth::verify_password(&p.password, &password_hash).map_err(|e| RpcError {
                    code: "verification_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            if !valid {
                let _ = accounts::record_login_failure(conn, &summary.id, current_utc);
                return Err(RpcError {
                    code: "invalid_credentials".to_string(),
                    message: "Invalid username or password".to_string(),
                    field: None,
                });
            }

            // Success: clear failed attempts
            let _ = accounts::record_login_success(conn, &summary.id);

            let device = get_or_create_host_device(conn, current_utc).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let raw_token = auth::generate_session_token();
            let token_hash = auth::hash_session_token(&raw_token);
            let session_rec =
                accounts::create_session(conn, &summary.id, &device.id, &token_hash, current_utc)
                    .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let mut perm_stmt = conn
                .prepare(
                    "SELECT DISTINCT rp.permission_code
                 FROM role_permissions rp
                 JOIN user_roles ur ON rp.role_id = ur.role_id
                 WHERE ur.user_id = ?1
                 ORDER BY rp.permission_code ASC",
                )
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let permissions = perm_stmt
                .query_map([&summary.id], |row| row.get(0))
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?
                .collect::<Result<Vec<String>, _>>()
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let resp = LoginResponse {
                session_token: raw_token,
                user: summary.clone(),
                roles: summary.roles,
                permissions,
                expires_at: session_rec.expires_at,
            };

            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "auth.logout" => {
            if let Some(s) = session {
                accounts::revoke_session(conn, &s.session_id, current_utc).map_err(|e| {
                    RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }
                })?;
            }
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                vec![],
            ))
        }

        "auth.unlock" => {
            let p: UnlockPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid unlock payload: {}", e),
                    field: None,
                })?;

            let s = session.ok_or_else(|| RpcError {
                code: "unauthenticated".to_string(),
                message: "Session token required to unlock".to_string(),
                field: None,
            })?;

            let user_record =
                find_user_for_login(conn, &s.user.username).map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let (_, password_hash, _, _, _) = match user_record {
                Some(u) => u,
                None => {
                    return Err(RpcError {
                        code: "invalid_credentials".to_string(),
                        message: "Invalid password".to_string(),
                        field: None,
                    });
                }
            };

            let valid =
                auth::verify_password(&p.password, &password_hash).map_err(|e| RpcError {
                    code: "verification_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            if !valid {
                return Err(RpcError {
                    code: "invalid_credentials".to_string(),
                    message: "Invalid password".to_string(),
                    field: None,
                });
            }

            let extended =
                accounts::unlock_session(conn, &s.session_id, current_utc).map_err(|e| {
                    RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }
                })?;

            let resp = UnlockResponse {
                success: true,
                extended_expires_at: extended,
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "auth.change_password" => {
            let p: ChangePasswordPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid change password payload: {}", e),
                    field: None,
                })?;

            let s = session.ok_or_else(|| RpcError {
                code: "unauthenticated".to_string(),
                message: "Authentication required".to_string(),
                field: None,
            })?;

            let user_rec = find_user_for_login(conn, &s.user.username).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let (_, password_hash, _, _, _) = match user_rec {
                Some(u) => u,
                None => {
                    return Err(RpcError {
                        code: "invalid_credentials".to_string(),
                        message: "Invalid credentials".to_string(),
                        field: None,
                    });
                }
            };

            let valid =
                auth::verify_password(&p.current_password, &password_hash).map_err(|e| {
                    RpcError {
                        code: "verification_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }
                })?;

            if !valid {
                return Err(RpcError {
                    code: "invalid_credentials".to_string(),
                    message: "Current password does not match".to_string(),
                    field: Some("current_password".to_string()),
                });
            }

            let new_hash = auth::hash_password(&p.new_password).map_err(|e| RpcError {
                code: "weak_password".to_string(),
                message: e.to_string(),
                field: Some("new_password".to_string()),
            })?;

            accounts::change_user_password(conn, &s.user.id, &new_hash, current_utc).map_err(
                |e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                },
            )?;

            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                vec![],
            ))
        }

        "auth.get_current_session" => {
            let s = session.ok_or_else(|| RpcError {
                code: "unauthenticated".to_string(),
                message: "Authentication required".to_string(),
                field: None,
            })?;

            let user_summary = find_user_by_id(conn, &s.user.id)
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?
                .ok_or_else(|| RpcError {
                    code: "unauthenticated".to_string(),
                    message: "User record not found".to_string(),
                    field: None,
                })?;

            let resp = CurrentSessionResponse {
                user: user_summary,
                roles: s.roles.clone(),
                permissions: s.permissions.clone(),
                expires_at: s.expires_at.clone(),
                absolute_expires_at: s.absolute_expires_at.clone(),
                device: s.device.clone(),
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "auth.heartbeat" => {
            let s = session.ok_or_else(|| RpcError {
                code: "unauthenticated".to_string(),
                message: "Authentication required".to_string(),
                field: None,
            })?;

            let extended =
                accounts::touch_session(conn, &s.session_id, current_utc).map_err(|e| {
                    RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }
                })?;

            let resp = HeartbeatResponse {
                extended_expires_at: extended,
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "users.list" => {
            let p: ListUsersPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListUsersPayload {
                    status_filter: None,
                    limit: None,
                    offset: None,
                });

            let (users, total_count) =
                accounts::list_users(conn, p.status_filter.as_deref(), p.limit, p.offset).map_err(
                    |e| RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    },
                )?;

            let resp = ListUsersResponse { users, total_count };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "users.create" => {
            let p: CreateUserPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create user payload: {}", e),
                    field: None,
                })?;

            let hash = auth::hash_password(&p.password).map_err(|e| RpcError {
                code: "weak_password".to_string(),
                message: e.to_string(),
                field: Some("password".to_string()),
            })?;

            let user = accounts::create_user(
                conn,
                &p.username,
                &p.display_name,
                &hash,
                &p.role_ids,
                p.staff_profile_id.as_deref(),
                current_utc,
            )
            .map_err(|e| match e {
                AccountsError::UsernameTaken(u) => RpcError {
                    code: "username_taken".to_string(),
                    message: format!("Username '{}' is already taken", u),
                    field: Some("username".to_string()),
                },
                AccountsError::RoleNotFound(r) => RpcError {
                    code: "role_not_found".to_string(),
                    message: format!("Role '{}' not found", r),
                    field: Some("role_ids".to_string()),
                },
                other => RpcError {
                    code: "validation_error".to_string(),
                    message: other.to_string(),
                    field: None,
                },
            })?;

            let events = vec![ChangeEvent {
                topic: "users:changed".to_string(),
                entity_ids: vec![user.id.clone()],
            }];

            Ok((serde_json::to_value(user).unwrap(), events))
        }

        "users.update" => {
            let p: UpdateUserPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update user payload: {}", e),
                    field: None,
                })?;

            let user = accounts::update_user(
                conn,
                &p.id,
                &p.display_name,
                &p.status,
                &p.role_ids,
                p.staff_profile_id.as_deref(),
                current_utc,
            )
            .map_err(|e| match e {
                AccountsError::LastOwnerRequired => RpcError {
                    code: "last_owner_required".to_string(),
                    message: "Cannot disable or remove Owner role from the last active Owner"
                        .to_string(),
                    field: Some("status".to_string()),
                },
                AccountsError::NotFound(id) => RpcError {
                    code: "user_not_found".to_string(),
                    message: format!("User '{}' not found", id),
                    field: Some("id".to_string()),
                },
                AccountsError::RoleNotFound(r) => RpcError {
                    code: "role_not_found".to_string(),
                    message: format!("Role '{}' not found", r),
                    field: Some("role_ids".to_string()),
                },
                other => RpcError {
                    code: "validation_error".to_string(),
                    message: other.to_string(),
                    field: None,
                },
            })?;

            let events = vec![ChangeEvent {
                topic: "users:changed".to_string(),
                entity_ids: vec![user.id.clone()],
            }];

            Ok((serde_json::to_value(user).unwrap(), events))
        }

        "roles.list" => {
            let roles = accounts::list_roles(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let resp = ListRolesResponse { roles };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        "roles.create" => {
            let p: CreateRolePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create role payload: {}", e),
                    field: None,
                })?;

            let role = accounts::create_role(
                conn,
                &p.name,
                p.description.as_deref(),
                &p.permission_codes,
                current_utc,
            )
            .map_err(|e| match e {
                AccountsError::RoleNameTaken(name) => RpcError {
                    code: "role_name_taken".to_string(),
                    message: format!("Role name '{}' is already taken", name),
                    field: Some("name".to_string()),
                },
                AccountsError::InvalidPermission(perm) => RpcError {
                    code: "invalid_permission".to_string(),
                    message: format!("Permission code '{}' does not exist", perm),
                    field: Some("permission_codes".to_string()),
                },
                other => RpcError {
                    code: "validation_error".to_string(),
                    message: other.to_string(),
                    field: None,
                },
            })?;

            let events = vec![ChangeEvent {
                topic: "roles:changed".to_string(),
                entity_ids: vec![role.id.clone()],
            }];

            Ok((serde_json::to_value(role).unwrap(), events))
        }

        "roles.update" => {
            let p: UpdateRolePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update role payload: {}", e),
                    field: None,
                })?;

            let role = accounts::update_role(
                conn,
                &p.id,
                &p.name,
                p.description.as_deref(),
                &p.permission_codes,
                current_utc,
            )
            .map_err(|e| match e {
                AccountsError::SystemRoleProtected => RpcError {
                    code: "system_role_protected".to_string(),
                    message: "The Owner system role cannot be renamed".to_string(),
                    field: Some("name".to_string()),
                },
                AccountsError::RoleNotFound(id) => RpcError {
                    code: "role_not_found".to_string(),
                    message: format!("Role '{}' not found", id),
                    field: Some("id".to_string()),
                },
                AccountsError::RoleNameTaken(name) => RpcError {
                    code: "role_name_taken".to_string(),
                    message: format!("Role name '{}' is already taken", name),
                    field: Some("name".to_string()),
                },
                AccountsError::InvalidPermission(perm) => RpcError {
                    code: "invalid_permission".to_string(),
                    message: format!("Permission code '{}' does not exist", perm),
                    field: Some("permission_codes".to_string()),
                },
                other => RpcError {
                    code: "validation_error".to_string(),
                    message: other.to_string(),
                    field: None,
                },
            })?;

            let events = vec![ChangeEvent {
                topic: "roles:changed".to_string(),
                entity_ids: vec![role.id.clone()],
            }];

            Ok((serde_json::to_value(role).unwrap(), events))
        }

        "permissions.list" => {
            let permissions = accounts::list_permissions(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let resp = ListPermissionsResponse { permissions };
            Ok((serde_json::to_value(resp).unwrap(), vec![]))
        }

        _ => Err(RpcError {
            code: "unknown_command".to_string(),
            message: format!("Unhandled command: {}", command),
            field: None,
        }),
    }
}
