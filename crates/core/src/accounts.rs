use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AccountsError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Account is locked for 5 minutes")]
    AccountLocked,
    #[error("Account is disabled")]
    AccountDisabled,
    #[error("Cannot remove or disable the last active Owner")]
    LastOwnerRequired,
    #[error("System roles cannot be modified or deleted")]
    SystemRoleProtected,
    #[error("Role name '{0}' is already taken")]
    RoleNameTaken(String),
    #[error("Username '{0}' is already taken")]
    UsernameTaken(String),
    #[error("Invalid permission code: {0}")]
    InvalidPermission(String),
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Session has expired")]
    SessionExpired,
    #[error("Session is idle locked")]
    SessionIdleLocked,
    #[error("Clock tampering detected: system time moved backwards")]
    ClockTampering,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub status: String,
    pub must_change_password: bool,
    pub roles: Vec<String>,
    pub staff_profile_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RoleWithPermissions {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub permissions: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PermissionItem {
    pub code: String,
    pub module: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeviceSummary {
    pub id: String,
    pub name: String,
    pub windows_name: String,
    pub is_host: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SessionUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResolvedSession {
    pub session_id: String,
    pub user: SessionUser,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub device: DeviceSummary,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
    pub absolute_expires_at: String,
    pub is_idle_locked: bool,
    pub is_expired: bool,
    pub is_revoked: bool,
}

pub struct SessionRecord {
    pub id: String,
    pub token_hash: String,
    pub user_id: String,
    pub device_id: String,
    pub created_at: String,
    pub last_seen_at: String,
    pub expires_at: String,
    pub absolute_expires_at: String,
    pub revoked_at: Option<String>,
}

pub const IDLE_TIMEOUT_SECONDS: i64 = 900; // 15 minutes
pub const ABSOLUTE_SESSION_TIMEOUT_SECONDS: i64 = 86400; // 24 hours
pub const LOCKOUT_DURATION_SECONDS: i64 = 300; // 5 minutes
pub const FAILED_ATTEMPTS_DECAY_SECONDS: i64 = 86400; // 24 hours
pub const IDEMPOTENCY_PURGE_SECONDS: i64 = 72 * 3600; // 72 hours

pub fn add_seconds_to_iso(iso: &str, seconds: i64) -> Result<String, String> {
    let ts: jiff::Timestamp = iso.parse().map_err(|e: jiff::Error| e.to_string())?;
    let dur = jiff::SignedDuration::from_secs(seconds);
    let new_ts = ts.checked_add(dur).map_err(|e| e.to_string())?;
    Ok(new_ts.to_string())
}

pub fn is_after(a_iso: &str, b_iso: &str) -> bool {
    match (
        a_iso.parse::<jiff::Timestamp>(),
        b_iso.parse::<jiff::Timestamp>(),
    ) {
        (Ok(a), Ok(b)) => a.as_second() > b.as_second(),
        _ => a_iso > b_iso,
    }
}

pub fn seconds_between(a_iso: &str, b_iso: &str) -> Option<i64> {
    match (
        a_iso.parse::<jiff::Timestamp>(),
        b_iso.parse::<jiff::Timestamp>(),
    ) {
        (Ok(a), Ok(b)) => Some(b.as_second() - a.as_second()),
        _ => None,
    }
}

// -----------------------------------------------------------------------------
// Active Owner Invariant
// -----------------------------------------------------------------------------

pub fn count_active_owners(conn: &Connection) -> Result<u32, AccountsError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT u.id)
         FROM users u
         JOIN user_roles ur ON u.id = ur.user_id
         JOIN roles r ON ur.role_id = r.id
         WHERE r.name = 'Owner' AND u.status = 'active'",
        [],
        |row| row.get(0),
    )?;
    Ok(count as u32)
}

pub fn count_total_users(conn: &Connection) -> Result<u32, AccountsError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    Ok(count as u32)
}

// -----------------------------------------------------------------------------
// User Directory & Credentials
// -----------------------------------------------------------------------------

pub fn get_user_roles(conn: &Connection, user_id: &str) -> Result<Vec<String>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT r.name
         FROM roles r
         JOIN user_roles ur ON r.id = ur.role_id
         WHERE ur.user_id = ?1
         ORDER BY r.name ASC",
    )?;
    let roles = stmt
        .query_map([user_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(roles)
}

pub fn get_user_role_ids(conn: &Connection, user_id: &str) -> Result<Vec<String>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT role_id
         FROM user_roles
         WHERE user_id = ?1
         ORDER BY role_id ASC",
    )?;
    let ids = stmt
        .query_map([user_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(ids)
}

pub fn find_user_by_id(
    conn: &Connection,
    user_id: &str,
) -> Result<Option<UserSummary>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT id, username, display_name, status, must_change_password,
                staff_profile_id, created_at, updated_at
         FROM users
         WHERE id = ?1",
    )?;

    let maybe_user = stmt
        .query_row([user_id], |row| {
            let id: String = row.get(0)?;
            let username: String = row.get(1)?;
            let display_name: String = row.get(2)?;
            let status: String = row.get(3)?;
            let must_change_password: i32 = row.get(4)?;
            let staff_profile_id: Option<String> = row.get(5)?;
            let created_at: String = row.get(6)?;
            let updated_at: String = row.get(7)?;

            Ok((
                id,
                username,
                display_name,
                status,
                must_change_password != 0,
                staff_profile_id,
                created_at,
                updated_at,
            ))
        })
        .optional()?;

    match maybe_user {
        Some((
            id,
            username,
            display_name,
            status,
            must_change_password,
            staff_profile_id,
            created_at,
            updated_at,
        )) => {
            let roles = get_user_roles(conn, &id)?;
            Ok(Some(UserSummary {
                id,
                username,
                display_name,
                status,
                must_change_password,
                roles,
                staff_profile_id,
                created_at,
                updated_at,
            }))
        }
        None => Ok(None),
    }
}

pub type UserLoginRecord = (
    UserSummary,
    String,         // password_hash
    i32,            // failed_login_attempts
    Option<String>, // last_failed_attempt_at
    Option<String>, // locked_until
);

pub fn find_user_for_login(
    conn: &Connection,
    username: &str,
) -> Result<Option<UserLoginRecord>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT id, username, display_name, password_hash, status, must_change_password,
                failed_login_attempts, last_failed_attempt_at, locked_until,
                staff_profile_id, created_at, updated_at
         FROM users
         WHERE username = ?1 COLLATE NOCASE",
    )?;

    let row_opt = stmt
        .query_row([username], |row| {
            let id: String = row.get(0)?;
            let username: String = row.get(1)?;
            let display_name: String = row.get(2)?;
            let password_hash: String = row.get(3)?;
            let status: String = row.get(4)?;
            let must_change_password: i32 = row.get(5)?;
            let failed_login_attempts: i32 = row.get(6)?;
            let last_failed_attempt_at: Option<String> = row.get(7)?;
            let locked_until: Option<String> = row.get(8)?;
            let staff_profile_id: Option<String> = row.get(9)?;
            let created_at: String = row.get(10)?;
            let updated_at: String = row.get(11)?;

            Ok((
                id,
                username,
                display_name,
                password_hash,
                status,
                must_change_password != 0,
                failed_login_attempts,
                last_failed_attempt_at,
                locked_until,
                staff_profile_id,
                created_at,
                updated_at,
            ))
        })
        .optional()?;

    match row_opt {
        Some((
            id,
            uname,
            display_name,
            password_hash,
            status,
            must_change_password,
            failed_login_attempts,
            last_failed_attempt_at,
            locked_until,
            staff_profile_id,
            created_at,
            updated_at,
        )) => {
            let roles = get_user_roles(conn, &id)?;
            let summary = UserSummary {
                id,
                username: uname,
                display_name,
                status,
                must_change_password,
                roles,
                staff_profile_id,
                created_at,
                updated_at,
            };
            Ok(Some((
                summary,
                password_hash,
                failed_login_attempts,
                last_failed_attempt_at,
                locked_until,
            )))
        }
        None => Ok(None),
    }
}

pub fn list_users(
    conn: &Connection,
    status_filter: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<(Vec<UserSummary>, u32), AccountsError> {
    let limit_val = limit.unwrap_or(50);
    let offset_val = offset.unwrap_or(0);

    let (total_count, mut stmt) = match status_filter {
        Some(st) => {
            let total: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE status = ?1",
                [st],
                |row| row.get(0),
            )?;
            let stmt = conn.prepare(
                "SELECT id, username, display_name, status, must_change_password,
                        staff_profile_id, created_at, updated_at
                 FROM users
                 WHERE status = ?1
                 ORDER BY username ASC
                 LIMIT ?2 OFFSET ?3",
            )?;
            (total as u32, stmt)
        }
        None => {
            let total: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
            let stmt = conn.prepare(
                "SELECT id, username, display_name, status, must_change_password,
                        staff_profile_id, created_at, updated_at
                 FROM users
                 ORDER BY username ASC
                 LIMIT ?1 OFFSET ?2",
            )?;
            (total as u32, stmt)
        }
    };

    type RawUserRow = (
        String,
        String,
        String,
        String,
        bool,
        Option<String>,
        String,
        String,
    );

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<RawUserRow> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get::<_, i32>(4)? != 0,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    };

    let user_tuples: Vec<RawUserRow> = match status_filter {
        Some(st) => stmt
            .query_map(params![st, limit_val, offset_val], map_row)?
            .collect::<Result<Vec<_>, _>>()?,
        None => stmt
            .query_map(params![limit_val, offset_val], map_row)?
            .collect::<Result<Vec<_>, _>>()?,
    };

    let mut users = Vec::with_capacity(user_tuples.len());
    for (
        id,
        username,
        display_name,
        status,
        must_change_password,
        staff_profile_id,
        created_at,
        updated_at,
    ) in user_tuples
    {
        let roles = get_user_roles(conn, &id)?;
        users.push(UserSummary {
            id,
            username,
            display_name,
            status,
            must_change_password,
            roles,
            staff_profile_id,
            created_at,
            updated_at,
        });
    }

    Ok((users, total_count))
}

pub fn create_user(
    conn: &Connection,
    username: &str,
    display_name: &str,
    password_hash: &str,
    role_ids: &[String],
    staff_profile_id: Option<&str>,
    current_utc: &str,
) -> Result<UserSummary, AccountsError> {
    let clean_username = username.trim();
    if clean_username.is_empty() {
        return Err(AccountsError::Validation(
            "Username cannot be empty".to_string(),
        ));
    }
    let clean_display_name = display_name.trim();
    if clean_display_name.is_empty() {
        return Err(AccountsError::Validation(
            "Display name cannot be empty".to_string(),
        ));
    }

    // Check username uniqueness
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?1 COLLATE NOCASE)",
        [clean_username],
        |row| row.get(0),
    )?;
    if exists {
        return Err(AccountsError::UsernameTaken(clean_username.to_string()));
    }

    // Validate role_ids
    for r_id in role_ids {
        let role_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE id = ?1)",
            [r_id],
            |row| row.get(0),
        )?;
        if !role_exists {
            return Err(AccountsError::RoleNotFound(r_id.clone()));
        }
    }

    let user_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO users (id, username, display_name, password_hash, status,
                            must_change_password, failed_login_attempts, staff_profile_id,
                            created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'active', 0, 0, ?5, ?6, ?6)",
        params![
            user_id,
            clean_username,
            clean_display_name,
            password_hash,
            staff_profile_id,
            current_utc
        ],
    )?;

    for r_id in role_ids {
        conn.execute(
            "INSERT INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
            params![user_id, r_id],
        )?;
    }

    let summary =
        find_user_by_id(conn, &user_id)?.ok_or_else(|| AccountsError::NotFound(user_id.clone()))?;

    Ok(summary)
}

pub fn update_user(
    conn: &Connection,
    user_id: &str,
    display_name: &str,
    status: &str,
    role_ids: &[String],
    staff_profile_id: Option<&str>,
    current_utc: &str,
) -> Result<UserSummary, AccountsError> {
    let clean_display_name = display_name.trim();
    if clean_display_name.is_empty() {
        return Err(AccountsError::Validation(
            "Display name cannot be empty".to_string(),
        ));
    }

    if status != "active" && status != "disabled" {
        return Err(AccountsError::Validation(format!(
            "Invalid status '{}', must be 'active' or 'disabled'",
            status
        )));
    }

    let existing = find_user_by_id(conn, user_id)?
        .ok_or_else(|| AccountsError::NotFound(user_id.to_string()))?;

    // Validate role_ids
    for r_id in role_ids {
        let role_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE id = ?1)",
            [r_id],
            |row| row.get(0),
        )?;
        if !role_exists {
            return Err(AccountsError::RoleNotFound(r_id.clone()));
        }
    }

    // AC-4 invariant check: At least one active user holding Owner role must always exist
    let is_currently_active_owner =
        existing.status == "active" && existing.roles.iter().any(|r| r == "Owner");
    let will_be_active_owner = status == "active"
        && role_ids.iter().any(|id| {
            conn.query_row("SELECT name FROM roles WHERE id = ?1", [id], |r| {
                r.get::<_, String>(0)
            })
            .unwrap_or_default()
                == "Owner"
        });

    if is_currently_active_owner && !will_be_active_owner {
        let remaining_active_owners: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT u.id)
             FROM users u
             JOIN user_roles ur ON u.id = ur.user_id
             JOIN roles r ON ur.role_id = r.id
             WHERE r.name = 'Owner' AND u.status = 'active' AND u.id != ?1",
            [user_id],
            |row| row.get(0),
        )?;
        if remaining_active_owners == 0 {
            return Err(AccountsError::LastOwnerRequired);
        }
    }

    // Update user record
    conn.execute(
        "UPDATE users
         SET display_name = ?1, status = ?2, staff_profile_id = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            clean_display_name,
            status,
            staff_profile_id,
            current_utc,
            user_id
        ],
    )?;

    // If status became disabled, revoke all active sessions for this user
    if status == "disabled" && existing.status == "active" {
        revoke_user_sessions(conn, user_id, current_utc)?;
    }

    // Replace role mappings by diffing so sole active Owner is never deleted
    let existing_role_ids = get_user_role_ids(conn, user_id)?;
    for old_r_id in &existing_role_ids {
        if !role_ids.contains(old_r_id) {
            conn.execute(
                "DELETE FROM user_roles WHERE user_id = ?1 AND role_id = ?2",
                params![user_id, old_r_id],
            )?;
        }
    }
    for r_id in role_ids {
        if !existing_role_ids.contains(r_id) {
            conn.execute(
                "INSERT INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
                params![user_id, r_id],
            )?;
        }
    }

    // Post-check just in case
    let active_owners = count_active_owners(conn)?;
    if active_owners == 0 {
        return Err(AccountsError::LastOwnerRequired);
    }

    let summary = find_user_by_id(conn, user_id)?
        .ok_or_else(|| AccountsError::NotFound(user_id.to_string()))?;

    Ok(summary)
}

pub fn change_user_password(
    conn: &Connection,
    user_id: &str,
    new_password_hash: &str,
    current_utc: &str,
) -> Result<(), AccountsError> {
    let affected = conn.execute(
        "UPDATE users
         SET password_hash = ?1, must_change_password = 0, updated_at = ?2
         WHERE id = ?3",
        params![new_password_hash, current_utc, user_id],
    )?;

    if affected == 0 {
        return Err(AccountsError::NotFound(user_id.to_string()));
    }

    // Password change revokes existing active sessions
    revoke_user_sessions(conn, user_id, current_utc)?;
    Ok(())
}

pub fn record_login_failure(
    conn: &Connection,
    user_id: &str,
    current_utc: &str,
) -> Result<(), AccountsError> {
    let (current_attempts, last_attempt): (i32, Option<String>) = conn.query_row(
        "SELECT failed_login_attempts, last_failed_attempt_at FROM users WHERE id = ?1",
        [user_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let mut new_attempts = current_attempts + 1;

    // AC-2: If last failed attempt occurred > 24 hours ago, decay and reset count to 1
    if let Some(last_at) = last_attempt {
        if let Some(diff) = seconds_between(&last_at, current_utc) {
            if diff > FAILED_ATTEMPTS_DECAY_SECONDS {
                new_attempts = 1;
            }
        }
    }

    // AC-2: 5 consecutive failed login attempts lock the account for 300 seconds (5 minutes)
    let locked_until = if new_attempts >= 5 {
        Some(
            add_seconds_to_iso(current_utc, LOCKOUT_DURATION_SECONDS)
                .map_err(AccountsError::Validation)?,
        )
    } else {
        None
    };

    conn.execute(
        "UPDATE users
         SET failed_login_attempts = ?1, last_failed_attempt_at = ?2, locked_until = ?3
         WHERE id = ?4",
        params![new_attempts, current_utc, locked_until, user_id],
    )?;

    Ok(())
}

pub fn record_login_success(conn: &Connection, user_id: &str) -> Result<(), AccountsError> {
    conn.execute(
        "UPDATE users
         SET failed_login_attempts = 0, last_failed_attempt_at = NULL, locked_until = NULL
         WHERE id = ?1",
        [user_id],
    )?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Role & Permission Management
// -----------------------------------------------------------------------------

pub fn list_permissions(conn: &Connection) -> Result<Vec<PermissionItem>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT code, module, description
         FROM permissions
         ORDER BY module ASC, code ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PermissionItem {
            code: row.get(0)?,
            module: row.get(1)?,
            description: row.get(2)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn list_roles(conn: &Connection) -> Result<Vec<RoleWithPermissions>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_system, created_at, updated_at
         FROM roles
         ORDER BY is_system DESC, name ASC",
    )?;

    let role_tuples = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let is_system: i32 = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;
            Ok((
                id,
                name,
                description,
                is_system != 0,
                created_at,
                updated_at,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut roles = Vec::new();
    for (id, name, description, is_system, created_at, updated_at) in role_tuples {
        let mut perm_stmt = conn.prepare(
            "SELECT permission_code FROM role_permissions WHERE role_id = ?1 ORDER BY permission_code ASC"
        )?;
        let perms = perm_stmt
            .query_map([&id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        roles.push(RoleWithPermissions {
            id,
            name,
            description,
            is_system,
            permissions: perms,
            created_at,
            updated_at,
        });
    }

    Ok(roles)
}

pub fn find_role_by_id(
    conn: &Connection,
    role_id: &str,
) -> Result<Option<RoleWithPermissions>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, is_system, created_at, updated_at
         FROM roles
         WHERE id = ?1",
    )?;

    let maybe_role = stmt
        .query_row([role_id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let is_system: i32 = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;
            Ok((
                id,
                name,
                description,
                is_system != 0,
                created_at,
                updated_at,
            ))
        })
        .optional()?;

    match maybe_role {
        Some((id, name, description, is_system, created_at, updated_at)) => {
            let mut perm_stmt = conn.prepare(
                "SELECT permission_code FROM role_permissions WHERE role_id = ?1 ORDER BY permission_code ASC"
            )?;
            let perms = perm_stmt
                .query_map([&id], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;

            Ok(Some(RoleWithPermissions {
                id,
                name,
                description,
                is_system,
                permissions: perms,
                created_at,
                updated_at,
            }))
        }
        None => Ok(None),
    }
}

pub fn create_role(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    permission_codes: &[String],
    current_utc: &str,
) -> Result<RoleWithPermissions, AccountsError> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err(AccountsError::Validation(
            "Role name cannot be empty".to_string(),
        ));
    }

    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM roles WHERE name = ?1 COLLATE NOCASE)",
        [clean_name],
        |row| row.get(0),
    )?;
    if exists {
        return Err(AccountsError::RoleNameTaken(clean_name.to_string()));
    }

    // Verify all permission codes exist
    for code in permission_codes {
        let perm_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM permissions WHERE code = ?1)",
            [code],
            |row| row.get(0),
        )?;
        if !perm_exists {
            return Err(AccountsError::InvalidPermission(code.clone()));
        }
    }

    let role_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO roles (id, name, description, is_system, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, ?4)",
        params![role_id, clean_name, description, current_utc],
    )?;

    for code in permission_codes {
        conn.execute(
            "INSERT INTO role_permissions (role_id, permission_code) VALUES (?1, ?2)",
            params![role_id, code],
        )?;
    }

    find_role_by_id(conn, &role_id)?.ok_or_else(|| AccountsError::NotFound(role_id))
}

pub fn update_role(
    conn: &Connection,
    role_id: &str,
    name: &str,
    description: Option<&str>,
    permission_codes: &[String],
    current_utc: &str,
) -> Result<RoleWithPermissions, AccountsError> {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return Err(AccountsError::Validation(
            "Role name cannot be empty".to_string(),
        ));
    }

    let existing = find_role_by_id(conn, role_id)?
        .ok_or_else(|| AccountsError::RoleNotFound(role_id.to_string()))?;

    // Protect system role: cannot rename or modify Owner role
    if existing.is_system && existing.name == "Owner" {
        return Err(AccountsError::SystemRoleProtected);
    }

    // Verify name uniqueness if changed
    if !clean_name.eq_ignore_ascii_case(&existing.name) {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE name = ?1 COLLATE NOCASE AND id != ?2)",
            params![clean_name, role_id],
            |row| row.get(0),
        )?;
        if exists {
            return Err(AccountsError::RoleNameTaken(clean_name.to_string()));
        }
    }

    // Verify all permission codes exist
    for code in permission_codes {
        let perm_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM permissions WHERE code = ?1)",
            [code],
            |row| row.get(0),
        )?;
        if !perm_exists {
            return Err(AccountsError::InvalidPermission(code.clone()));
        }
    }

    conn.execute(
        "UPDATE roles
         SET name = ?1, description = ?2, updated_at = ?3
         WHERE id = ?4",
        params![clean_name, description, current_utc, role_id],
    )?;

    // Update permissions
    conn.execute(
        "DELETE FROM role_permissions WHERE role_id = ?1",
        params![role_id],
    )?;

    for code in permission_codes {
        conn.execute(
            "INSERT INTO role_permissions (role_id, permission_code) VALUES (?1, ?2)",
            params![role_id, code],
        )?;
    }

    find_role_by_id(conn, role_id)?.ok_or_else(|| AccountsError::NotFound(role_id.to_string()))
}

// -----------------------------------------------------------------------------
// Device Resolution
// -----------------------------------------------------------------------------

pub fn get_or_create_host_device(
    conn: &Connection,
    current_utc: &str,
) -> Result<DeviceSummary, AccountsError> {
    let mut stmt = conn
        .prepare("SELECT id, name, windows_name, is_host FROM devices WHERE is_host = 1 LIMIT 1")?;

    let maybe_device = stmt
        .query_row([], |row| {
            Ok(DeviceSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                windows_name: row.get(2)?,
                is_host: row.get::<_, i32>(3)? != 0,
            })
        })
        .optional()?;

    if let Some(dev) = maybe_device {
        return Ok(dev);
    }

    let dev_id = Uuid::now_v7().to_string();
    let win_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "HOST-PC".to_string());
    let token_hash = format!("host_token_hash_{}", dev_id);

    conn.execute(
        "INSERT INTO devices (id, name, windows_name, token_hash, status, is_host, paired_at, last_seen_at, app_version)
         VALUES (?1, 'Host Workstation', ?2, ?3, 'active', 1, ?4, ?4, '1.0.0')",
        params![dev_id, win_name, token_hash, current_utc],
    )?;

    Ok(DeviceSummary {
        id: dev_id,
        name: "Host Workstation".to_string(),
        windows_name: win_name,
        is_host: true,
    })
}

// -----------------------------------------------------------------------------
// Session Management
// -----------------------------------------------------------------------------

pub fn create_session(
    conn: &Connection,
    user_id: &str,
    device_id: &str,
    token_hash: &str,
    current_utc: &str,
) -> Result<SessionRecord, AccountsError> {
    let session_id = Uuid::now_v7().to_string();
    let expires_at =
        add_seconds_to_iso(current_utc, IDLE_TIMEOUT_SECONDS).map_err(AccountsError::Validation)?;
    let absolute_expires_at = add_seconds_to_iso(current_utc, ABSOLUTE_SESSION_TIMEOUT_SECONDS)
        .map_err(AccountsError::Validation)?;

    conn.execute(
        "INSERT INTO sessions (id, token_hash, user_id, device_id, created_at, last_seen_at, expires_at, absolute_expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
        params![
            session_id,
            token_hash,
            user_id,
            device_id,
            current_utc,
            expires_at,
            absolute_expires_at
        ],
    )?;

    Ok(SessionRecord {
        id: session_id,
        token_hash: token_hash.to_string(),
        user_id: user_id.to_string(),
        device_id: device_id.to_string(),
        created_at: current_utc.to_string(),
        last_seen_at: current_utc.to_string(),
        expires_at,
        absolute_expires_at,
        revoked_at: None,
    })
}

/// Creates the initial Owner user account and an active session within an existing transaction or connection.
pub fn create_initial_owner_in_conn(
    conn: &Connection,
    username: &str,
    display_name: &str,
    password: &str,
    current_utc: &str,
) -> Result<(UserSummary, String, String), AccountsError> {
    let hash = crate::auth::hash_password(password)
        .map_err(|e| AccountsError::Validation(e.to_string()))?;
    let owner_role = list_roles(conn)?
        .into_iter()
        .find(|r| r.name == "Owner")
        .ok_or_else(|| AccountsError::RoleNotFound("Owner system role missing".to_string()))?;

    let user = create_user(
        conn,
        username,
        display_name,
        &hash,
        &[owner_role.id],
        None,
        current_utc,
    )?;

    let device = get_or_create_host_device(conn, current_utc)?;
    let raw_token = crate::auth::generate_session_token();
    let token_hash = crate::auth::hash_session_token(&raw_token);
    let session = create_session(conn, &user.id, &device.id, &token_hash, current_utc)?;

    Ok((user, raw_token, session.expires_at))
}

pub fn resolve_session_by_token_hash(
    conn: &Connection,
    token_hash: &str,
    current_utc: &str,
) -> Result<Option<ResolvedSession>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.user_id, s.device_id, s.created_at, s.last_seen_at,
                s.expires_at, s.absolute_expires_at, s.revoked_at,
                u.username, u.display_name, u.status,
                d.name, d.windows_name, d.is_host
         FROM sessions s
         JOIN users u ON s.user_id = u.id
         JOIN devices d ON s.device_id = d.id
         WHERE s.token_hash = ?1",
    )?;

    let maybe_record = stmt
        .query_row([token_hash], |row| {
            let session_id: String = row.get(0)?;
            let user_id: String = row.get(1)?;
            let device_id: String = row.get(2)?;
            let created_at: String = row.get(3)?;
            let last_seen_at: String = row.get(4)?;
            let expires_at: String = row.get(5)?;
            let absolute_expires_at: String = row.get(6)?;
            let revoked_at: Option<String> = row.get(7)?;
            let username: String = row.get(8)?;
            let display_name: String = row.get(9)?;
            let user_status: String = row.get(10)?;
            let dev_name: String = row.get(11)?;
            let dev_win_name: String = row.get(12)?;
            let dev_is_host: i32 = row.get(13)?;

            Ok((
                session_id,
                user_id,
                device_id,
                created_at,
                last_seen_at,
                expires_at,
                absolute_expires_at,
                revoked_at,
                username,
                display_name,
                user_status,
                dev_name,
                dev_win_name,
                dev_is_host != 0,
            ))
        })
        .optional()?;

    let Some((
        session_id,
        user_id,
        _device_id,
        created_at,
        last_seen_at,
        expires_at,
        absolute_expires_at,
        revoked_at,
        username,
        display_name,
        user_status,
        dev_name,
        dev_win_name,
        dev_is_host,
    )) = maybe_record
    else {
        return Ok(None);
    };

    // Clock tampering protection: if current_utc < created_at, session is compromised -> revoke!
    if is_after(&created_at, current_utc) {
        let _ = conn.execute(
            "UPDATE sessions SET revoked_at = ?1 WHERE id = ?2",
            params![current_utc, session_id],
        );
        return Err(AccountsError::ClockTampering);
    }

    let is_revoked = revoked_at.is_some() || user_status == "disabled";
    let is_expired = is_after(current_utc, &absolute_expires_at);
    let is_idle_locked = is_after(current_utc, &expires_at) && !is_expired;

    // Resolve user roles and permissions
    let roles = get_user_roles(conn, &user_id)?;

    let mut perm_stmt = conn.prepare(
        "SELECT DISTINCT rp.permission_code
         FROM role_permissions rp
         JOIN user_roles ur ON rp.role_id = ur.role_id
         WHERE ur.user_id = ?1
         ORDER BY rp.permission_code ASC",
    )?;
    let permissions = perm_stmt
        .query_map([&user_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(Some(ResolvedSession {
        session_id,
        user: SessionUser {
            id: user_id,
            username,
            display_name,
            status: user_status,
        },
        roles,
        permissions,
        device: DeviceSummary {
            id: _device_id,
            name: dev_name,
            windows_name: dev_win_name,
            is_host: dev_is_host,
        },
        created_at,
        last_seen_at,
        expires_at,
        absolute_expires_at,
        is_idle_locked,
        is_expired,
        is_revoked,
    }))
}

pub fn touch_session(
    conn: &Connection,
    session_id: &str,
    current_utc: &str,
) -> Result<String, AccountsError> {
    let (abs_expires_at,): (String,) = conn.query_row(
        "SELECT absolute_expires_at FROM sessions WHERE id = ?1",
        [session_id],
        |row| Ok((row.get(0)?,)),
    )?;

    let candidate_expires =
        add_seconds_to_iso(current_utc, IDLE_TIMEOUT_SECONDS).map_err(AccountsError::Validation)?;

    let new_expires_at = if is_after(&candidate_expires, &abs_expires_at) {
        abs_expires_at
    } else {
        candidate_expires
    };

    conn.execute(
        "UPDATE sessions
         SET last_seen_at = ?1, expires_at = ?2
         WHERE id = ?3",
        params![current_utc, new_expires_at, session_id],
    )?;

    Ok(new_expires_at)
}

pub fn unlock_session(
    conn: &Connection,
    session_id: &str,
    current_utc: &str,
) -> Result<String, AccountsError> {
    touch_session(conn, session_id, current_utc)
}

pub fn revoke_session(
    conn: &Connection,
    session_id: &str,
    current_utc: &str,
) -> Result<(), AccountsError> {
    conn.execute(
        "UPDATE sessions SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
        params![current_utc, session_id],
    )?;
    Ok(())
}

pub fn revoke_user_sessions(
    conn: &Connection,
    user_id: &str,
    current_utc: &str,
) -> Result<(), AccountsError> {
    conn.execute(
        "UPDATE sessions SET revoked_at = ?1 WHERE user_id = ?2 AND revoked_at IS NULL",
        params![current_utc, user_id],
    )?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Idempotency Tracking & Pruning
// -----------------------------------------------------------------------------

pub fn get_idempotency_record(
    conn: &Connection,
    key: &str,
) -> Result<Option<(String, String)>, AccountsError> {
    let mut stmt = conn.prepare(
        "SELECT payload_hash, response_json FROM command_idempotency WHERE idempotency_key = ?1",
    )?;

    let row = stmt
        .query_row([key], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()?;

    Ok(row)
}

pub fn save_idempotency_record(
    conn: &Connection,
    key: &str,
    command_name: &str,
    payload_hash: &str,
    user_id: &str,
    response_json: &str,
    current_utc: &str,
) -> Result<(), AccountsError> {
    conn.execute(
        "INSERT INTO command_idempotency (idempotency_key, command_name, payload_hash, user_id, response_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(idempotency_key) DO NOTHING",
        params![key, command_name, payload_hash, user_id, response_json, current_utc],
    )?;
    Ok(())
}

pub fn purge_expired_idempotency(
    conn: &Connection,
    current_utc: &str,
) -> Result<usize, AccountsError> {
    let threshold = add_seconds_to_iso(current_utc, -IDEMPOTENCY_PURGE_SECONDS)
        .map_err(AccountsError::Validation)?;

    let deleted = conn.execute(
        "DELETE FROM command_idempotency WHERE created_at < ?1",
        params![threshold],
    )?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_test_accounts_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("accounts_test.db");
        let conn = db::open_connection_at(&db_path).expect("open test db");
        db::apply_all_migrations(&conn).expect("apply migrations");
        (dir, conn)
    }

    #[test]
    fn test_initial_roles_and_permissions_seeded() {
        let (_dir, conn) = setup_test_accounts_db();

        let perms = list_permissions(&conn).expect("list permissions");
        assert_eq!(perms.len(), 43);

        let roles = list_roles(&conn).expect("list roles");
        assert_eq!(roles.len(), 3);

        let owner_role = roles
            .iter()
            .find(|r| r.name == "Owner")
            .expect("Owner role exists");
        assert!(owner_role.is_system);
        assert_eq!(owner_role.permissions.len(), 43);

        let cashier_role = roles
            .iter()
            .find(|r| r.name == "Cashier")
            .expect("Cashier role exists");
        assert!(cashier_role.is_system);
        assert_eq!(cashier_role.permissions.len(), 11);

        let teacher_role = roles
            .iter()
            .find(|r| r.name == "Teacher")
            .expect("Teacher role exists");
        assert!(teacher_role.is_system);
        assert_eq!(teacher_role.permissions.len(), 11);
    }

    #[test]
    fn test_system_role_triggers_prevent_delete_and_rename() {
        let (_dir, conn) = setup_test_accounts_db();

        // System role deletion aborts via trigger
        let del_result = conn.execute("DELETE FROM roles WHERE name = 'Owner'", []);
        assert!(
            del_result.is_err(),
            "Trigger should abort deletion of system role"
        );

        // Renaming Owner aborts via trigger
        let rename_result = conn.execute("UPDATE roles SET name = 'Boss' WHERE name = 'Owner'", []);
        assert!(
            rename_result.is_err(),
            "Trigger should abort renaming Owner role"
        );
    }

    #[test]
    fn test_user_creation_and_last_owner_protection() {
        let (_dir, conn) = setup_test_accounts_db();
        let now = "2026-09-20T10:00:00Z";

        let owner_role = list_roles(&conn)
            .expect("roles")
            .into_iter()
            .find(|r| r.name == "Owner")
            .unwrap();

        // Create owner user
        let user = create_user(
            &conn,
            "owner1",
            "Initial Owner",
            "$argon2id$v=19$m=65536,t=3,p=4$dummyhash",
            std::slice::from_ref(&owner_role.id),
            None,
            now,
        )
        .expect("create owner");

        assert_eq!(user.username, "owner1");
        assert_eq!(count_active_owners(&conn).unwrap(), 1);

        // Attempt to disable the last owner fails
        let disable_result = update_user(
            &conn,
            &user.id,
            "Initial Owner",
            "disabled",
            std::slice::from_ref(&owner_role.id),
            None,
            now,
        );
        assert!(disable_result.is_err());
        match disable_result {
            Err(AccountsError::LastOwnerRequired) => (),
            other => panic!("Expected LastOwnerRequired, got {:?}", other),
        }

        // Trigger also aborts raw SQL disabling the last owner
        let raw_disable = conn.execute(
            "UPDATE users SET status = 'disabled' WHERE id = ?1",
            [&user.id],
        );
        assert!(
            raw_disable.is_err(),
            "Trigger should abort raw SQL disable of sole owner"
        );
    }

    #[test]
    fn test_failed_attempts_lockout_and_decay() {
        let (_dir, conn) = setup_test_accounts_db();
        let now = "2026-09-20T10:00:00Z";

        let owner_role = list_roles(&conn)
            .unwrap()
            .into_iter()
            .find(|r| r.name == "Owner")
            .unwrap();
        let user = create_user(
            &conn,
            "testuser",
            "Test",
            "dummyhash",
            &[owner_role.id],
            None,
            now,
        )
        .unwrap();

        // 4 failed attempts: not locked
        for _ in 0..4 {
            record_login_failure(&conn, &user.id, now).unwrap();
        }
        let (_, _, attempts, _, locked_until) =
            find_user_for_login(&conn, "testuser").unwrap().unwrap();
        assert_eq!(attempts, 4);
        assert!(locked_until.is_none());

        // 5th failed attempt: locked for 300 seconds
        record_login_failure(&conn, &user.id, now).unwrap();
        let (_, _, attempts5, _, locked_until5) =
            find_user_for_login(&conn, "testuser").unwrap().unwrap();
        assert_eq!(attempts5, 5);
        assert!(locked_until5.is_some());
        assert_eq!(locked_until5.unwrap(), "2026-09-20T10:05:00Z");

        // After 25 hours (decay): a new failure resets attempts to 1
        let later = "2026-09-21T11:00:00Z";
        record_login_failure(&conn, &user.id, later).unwrap();
        let (_, _, attempts_decayed, _, _) =
            find_user_for_login(&conn, "testuser").unwrap().unwrap();
        assert_eq!(attempts_decayed, 1);
    }

    #[test]
    fn test_session_lifecycle_and_idle_locking() {
        let (_dir, conn) = setup_test_accounts_db();
        let now = "2026-09-20T10:00:00Z";

        let owner_role = list_roles(&conn)
            .unwrap()
            .into_iter()
            .find(|r| r.name == "Owner")
            .unwrap();
        let user = create_user(
            &conn,
            "owner2",
            "Owner 2",
            "dummyhash",
            &[owner_role.id],
            None,
            now,
        )
        .unwrap();
        let device = get_or_create_host_device(&conn, now).unwrap();

        let token_hash = "tokenhash12345";
        let session = create_session(&conn, &user.id, &device.id, token_hash, now).unwrap();

        // Session immediately active
        let resolved = resolve_session_by_token_hash(&conn, token_hash, now)
            .unwrap()
            .unwrap();
        assert!(!resolved.is_idle_locked);
        assert!(!resolved.is_expired);
        assert_eq!(resolved.user.username, "owner2");

        // After 16 minutes: idle locked
        let at_16m = "2026-09-20T10:16:00Z";
        let resolved_16m = resolve_session_by_token_hash(&conn, token_hash, at_16m)
            .unwrap()
            .unwrap();
        assert!(resolved_16m.is_idle_locked);
        assert!(!resolved_16m.is_expired);

        // Unlock resets last seen and extends expires_at
        let extended = unlock_session(&conn, &session.id, at_16m).unwrap();
        assert_eq!(extended, "2026-09-20T10:31:00Z");

        let resolved_after_unlock = resolve_session_by_token_hash(&conn, token_hash, at_16m)
            .unwrap()
            .unwrap();
        assert!(!resolved_after_unlock.is_idle_locked);

        // After 25 hours: expired
        let at_25h = "2026-09-21T11:00:00Z";
        let resolved_25h = resolve_session_by_token_hash(&conn, token_hash, at_25h)
            .unwrap()
            .unwrap();
        assert!(resolved_25h.is_expired);

        // Backwards clock drift detects tampering and revokes session
        let backwards = "2026-09-20T09:00:00Z";
        let drift_result = resolve_session_by_token_hash(&conn, token_hash, backwards);
        assert!(matches!(drift_result, Err(AccountsError::ClockTampering)));
    }
}
