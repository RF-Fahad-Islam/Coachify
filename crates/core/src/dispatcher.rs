use crate::accounts::{
    self, find_user_by_id, find_user_for_login, get_or_create_host_device, AccountsError,
    DeviceSummary, PermissionItem, ResolvedSession, RoleWithPermissions, UserSummary,
};
use crate::auth;
use crate::db;
use crate::migrations;
pub use crate::migrations::{
    MigrationHistoryItem, MigrationHistoryResponse, MigrationStatusResponse,
    OpenDbDirectoryResponse, PendingMigrationInfo, RecoveryInfoResponse,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

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
    OwnerOrDebug,
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
    CommandMeta {
        name: "audit.list",
        auth: AuthRequirement::Permission("audit.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "audit.verify_chain",
        auth: AuthRequirement::Permission("audit.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "audit.get_head",
        auth: AuthRequirement::Permission("audit.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.migration_status",
        auth: AuthRequirement::Permission("settings.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.migration_history",
        auth: AuthRequirement::Permission("settings.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.recovery_info",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.open_db_directory",
        auth: AuthRequirement::Permission("settings.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "recovery.reprint_sheet",
        auth: AuthRequirement::Permission("settings.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "students.create",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "students.update",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "students.get",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "students.list",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "students.search",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "students.archive",
        auth: AuthRequirement::Permission("students.archive"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "guardians.search",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "guardians.get",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "guardians.update",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "vault.upload_file",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "vault.get_file",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Import commands (Spec 0010)
    CommandMeta {
        name: "import.inspect_file",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "import.preview_validation",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "import.execute",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    // Academic & Staff commands
    CommandMeta {
        name: "programs.create",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "programs.update",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "programs.get",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "programs.list",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "subjects.create",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "subjects.list",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "batches.create",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "batches.update",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "batches.get",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "batches.list",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "batches.check_conflicts",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "batches.set_subjects",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "staff.create",
        auth: AuthRequirement::Permission("staff.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "staff.update",
        auth: AuthRequirement::Permission("staff.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "staff.get",
        auth: AuthRequirement::Permission("staff.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "staff.list",
        auth: AuthRequirement::Permission("staff.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "staff.search",
        auth: AuthRequirement::Permission("staff.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "enrollments.create",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "enrollments.transfer",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "enrollments.end",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "enrollments.bulk_create",
        auth: AuthRequirement::Permission("academic.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "enrollments.list_by_batch",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "enrollments.list_by_student",
        auth: AuthRequirement::Permission("academic.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "generator.generate_dataset",
        auth: AuthRequirement::OwnerOrDebug,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "generator.get_status",
        auth: AuthRequirement::Public,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "generator.switch_database",
        auth: AuthRequirement::OwnerOrDebug,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "generator.switch_to_production_database",
        auth: AuthRequirement::OwnerOrDebug,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateStudentPayload {
    pub student: crate::students::CreateStudentInput,
    pub allow_duplicate_phone: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateStudentPayload {
    pub id: String,
    pub expected_version: u32,
    pub student: crate::students::UpdateStudentInput,
    pub allow_duplicate_phone: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStudentPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchStudentsPayload {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ArchiveStudentPayload {
    pub id: String,
    pub reason: String,
    pub expected_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchGuardiansPayload {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetGuardianPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateGuardianPayload {
    pub id: String,
    pub guardian: crate::students::UpdateGuardianInput,
    pub split_from_siblings: Option<bool>,
    pub student_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetFilePayload {
    pub id: String,
}

// Academic & Staff Payloads
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateProgramPayload {
    pub program: crate::academic::CreateProgramInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateProgramPayload {
    pub id: String,
    pub program: crate::academic::UpdateProgramInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetProgramPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListProgramsPayload {
    pub session_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateSubjectPayload {
    pub subject: crate::academic::CreateSubjectInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateBatchPayload {
    pub batch: crate::academic::CreateBatchInput,
    pub allow_conflict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateBatchPayload {
    pub id: String,
    pub batch: crate::academic::UpdateBatchInput,
    pub allow_conflict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListBatchesPayload {
    pub program_id: Option<String>,
    pub teacher_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CheckConflictsPayload {
    pub batch_id: Option<String>,
    pub teacher_staff_id: Option<String>,
    pub room_number: Option<String>,
    pub schedule: Vec<crate::schedule::ScheduleSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetBatchSubjectsPayload {
    pub batch_id: String,
    pub subjects: Vec<crate::academic::BatchSubjectInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateStaffPayload {
    pub staff: crate::academic::CreateStaffInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateStaffPayload {
    pub id: String,
    pub staff: crate::academic::UpdateStaffInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStaffPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStaffPayload {
    pub status: Option<String>,
    pub department: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchStaffPayload {
    pub query: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateEnrollmentPayload {
    pub enrollment: crate::academic::CreateEnrollmentInput,
    pub allow_overcapacity: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransferEnrollmentPayload {
    pub transfer: crate::academic::TransferEnrollmentInput,
    pub allow_overcapacity: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EndEnrollmentPayload {
    pub enrollment: crate::academic::EndEnrollmentInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BulkEnrollPayload {
    pub bulk: crate::academic::BulkEnrollInput,
    pub allow_overcapacity: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListBatchRosterPayload {
    pub batch_id: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStudentEnrollmentsPayload {
    pub student_id: String,
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

    // AC-8: In recovery mode, only recovery diagnostic and explorer commands are allowed
    if crate::migrations::is_in_recovery()
        && command != "system.recovery_info"
        && command != "system.open_db_directory"
        && command != "system.ping"
    {
        let env: RpcEnvelope<()> = RpcEnvelope {
            ok: false,
            data: None,
            error: Some(RpcError {
                code: "database_recovery_mode".to_string(),
                message: "Coachify is in migration recovery mode. Normal operations are suspended until the database upgrade is resolved.".to_string(),
                field: None,
            }),
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

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
        AuthRequirement::Public | AuthRequirement::OwnerOrDebug if cfg!(debug_assertions) => {
            // Optional session resolution for public or debug generator commands
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
        AuthRequirement::Public => {
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
        AuthRequirement::Authenticated
        | AuthRequirement::Permission(_)
        | AuthRequirement::OwnerOrDebug => {
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

    // 5b. Owner or Debug check (Spec 0011 AC-13)
    if meta.auth == AuthRequirement::OwnerOrDebug {
        #[cfg(not(debug_assertions))]
        {
            let Some(s) = &resolved_session else {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "unauthenticated".to_string(),
                        message: "Authentication required for generator commands".to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            };

            let is_owner = s.roles.iter().any(|r| r == "Owner");
            if !is_owner {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "forbidden".to_string(),
                        message: "Generator commands require the Owner role in production"
                            .to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
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
        CommandKind::Read => {
            let (val, events, _) =
                run_command_handler(conn, command, payload, session, current_utc)?;
            Ok((val, events))
        }
        CommandKind::Mutation => {
            if command == "auth.login" || command == "auth.unlock" {
                let (val, events, _) =
                    run_command_handler(conn, command, payload, session, current_utc)?;
                return Ok((val, events));
            }

            // Run inside immediate transaction
            let tx = conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let (val, events, audit_entries) =
                run_command_handler(&tx, command, payload, session, current_utc)?;

            let cli_session;
            let audit_session = if session.is_none() && command.starts_with("generator.") {
                cli_session = accounts::ResolvedSession {
                    session_id: "cli-session".to_string(),
                    user: accounts::SessionUser {
                        id: crate::generator::SYSTEM_CLI_USER_ID.to_string(),
                        username: "SYSTEM_CLI".to_string(),
                        display_name: "System CLI Operator".to_string(),
                        status: "active".to_string(),
                    },
                    roles: vec!["Owner".to_string()],
                    permissions: vec![],
                    device: accounts::DeviceSummary {
                        id: "cli-device".to_string(),
                        name: "CLI Console".to_string(),
                        windows_name: "CLI".to_string(),
                        is_host: true,
                    },
                    created_at: current_utc.to_string(),
                    last_seen_at: current_utc.to_string(),
                    expires_at: current_utc.to_string(),
                    absolute_expires_at: current_utc.to_string(),
                    is_idle_locked: false,
                    is_expired: false,
                    is_revoked: false,
                };
                Some(&cli_session)
            } else {
                session
            };

            for entry in audit_entries {
                crate::audit::record_audit_entry(&tx, entry, audit_session, current_utc).map_err(
                    |e| RpcError {
                        code: "audit_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    },
                )?;
            }

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
) -> Result<
    (
        serde_json::Value,
        Vec<ChangeEvent>,
        Vec<crate::audit::AuditEntry>,
    ),
    RpcError,
> {
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
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "system.migration_status" => {
            let resp = migrations::get_migration_status(conn).map_err(|e| RpcError {
                code: e.code().to_string(),
                message: e.to_string(),
                field: None,
            })?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "system.migration_history" => {
            let resp = migrations::get_migration_history(conn).map_err(|e| RpcError {
                code: e.code().to_string(),
                message: e.to_string(),
                field: None,
            })?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "system.recovery_info" => {
            let db_path = db::resolve_dev_db_path().unwrap_or_else(|_| PathBuf::from("dev.db"));
            let resp = migrations::get_recovery_info(&db_path);
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "system.open_db_directory" => {
            let db_path = db::resolve_dev_db_path().unwrap_or_else(|_| PathBuf::from("dev.db"));
            let res = migrations::open_directory_in_explorer(&db_path);
            let resp = OpenDbDirectoryResponse {
                success: res.is_ok(),
                path: db_path.display().to_string(),
                error: res.err().map(|e| e.to_string()),
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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
                    code: "role_not_found".to_string(),
                    message: "Owner system role missing".to_string(),
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
                entity_ids: vec![user.id.clone()],
            }];

            let audit = vec![crate::audit::AuditEntry {
                action: "system.bootstrap_owner".to_string(),
                entity: "user".to_string(),
                entity_id: user.id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "username": user.username,
                    "display_name": user.display_name,
                    "roles": ["Owner"],
                })),
            }];

            Ok((serde_json::to_value(resp).unwrap(), events, audit))
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
                        message: "Account is locked for 5 minutes".to_string(),
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
                let _ = crate::audit::record_audit_entry(
                    conn,
                    crate::audit::AuditEntry {
                        action: "auth.login_failed".to_string(),
                        entity: "user".to_string(),
                        entity_id: summary.id.clone(),
                        before_json: None,
                        after_json: Some(serde_json::json!({
                            "username": p.username,
                            "reason": "invalid_password",
                        })),
                    },
                    None,
                    current_utc,
                );
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

            let _ = crate::audit::record_audit_entry(
                conn,
                crate::audit::AuditEntry {
                    action: "auth.login".to_string(),
                    entity: "session".to_string(),
                    entity_id: session_rec.id.clone(),
                    before_json: None,
                    after_json: Some(serde_json::json!({
                        "user_id": summary.id,
                        "username": summary.username,
                        "device_id": device.id,
                    })),
                },
                None,
                current_utc,
            );

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

            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "auth.logout" => {
            let mut audit = Vec::new();
            if let Some(s) = session {
                accounts::revoke_session(conn, &s.session_id, current_utc).map_err(|e| {
                    RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }
                })?;

                audit.push(crate::audit::AuditEntry {
                    action: "auth.logout".to_string(),
                    entity: "session".to_string(),
                    entity_id: s.session_id.clone(),
                    before_json: None,
                    after_json: Some(serde_json::json!({
                        "user_id": s.user.id,
                        "status": "logged_out",
                    })),
                });
            }
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                vec![],
                audit,
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

            let (summary, password_hash, _, _, locked_until) = match user_record {
                Some(u) => u,
                None => {
                    return Err(RpcError {
                        code: "invalid_credentials".to_string(),
                        message: "Invalid password".to_string(),
                        field: None,
                    });
                }
            };

            if let Some(lock_time) = locked_until {
                if accounts::is_after(&lock_time, current_utc) {
                    return Err(RpcError {
                        code: "account_locked".to_string(),
                        message: "Account is locked for 5 minutes".to_string(),
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
                let _ = crate::audit::record_audit_entry(
                    conn,
                    crate::audit::AuditEntry {
                        action: "auth.unlock_failed".to_string(),
                        entity: "user".to_string(),
                        entity_id: summary.id.clone(),
                        before_json: None,
                        after_json: Some(serde_json::json!({
                            "username": s.user.username,
                            "reason": "invalid_password",
                        })),
                    },
                    Some(s),
                    current_utc,
                );
                return Err(RpcError {
                    code: "invalid_credentials".to_string(),
                    message: "Invalid password".to_string(),
                    field: None,
                });
            }

            // On successful unlock, clear failed attempts
            let _ = accounts::record_login_success(conn, &summary.id);

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

            let _ = crate::audit::record_audit_entry(
                conn,
                crate::audit::AuditEntry {
                    action: "auth.unlock".to_string(),
                    entity: "session".to_string(),
                    entity_id: s.session_id.clone(),
                    before_json: None,
                    after_json: Some(serde_json::json!({
                        "user_id": s.user.id,
                        "status": "unlocked",
                    })),
                },
                Some(s),
                current_utc,
            );

            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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

            let audit = vec![crate::audit::AuditEntry {
                action: "auth.change_password".to_string(),
                entity: "user".to_string(),
                entity_id: s.user.id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "user_id": s.user.id,
                    "password_changed": true,
                })),
            }];

            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                vec![],
                audit,
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
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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

            let audit = vec![crate::audit::AuditEntry {
                action: "users.create".to_string(),
                entity: "user".to_string(),
                entity_id: user.id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "username": user.username,
                    "display_name": user.display_name,
                    "status": user.status,
                    "roles": user.roles,
                    "staff_profile_id": user.staff_profile_id,
                })),
            }];

            Ok((serde_json::to_value(user).unwrap(), events, audit))
        }

        "users.update" => {
            let p: UpdateUserPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update user payload: {}", e),
                    field: None,
                })?;

            let existing_user = find_user_by_id(conn, &p.id).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let before_json = existing_user.as_ref().map(|u| {
                serde_json::json!({
                    "display_name": u.display_name,
                    "status": u.status,
                    "roles": u.roles,
                    "staff_profile_id": u.staff_profile_id,
                })
            });

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

            let audit = vec![crate::audit::AuditEntry {
                action: "users.update".to_string(),
                entity: "user".to_string(),
                entity_id: user.id.clone(),
                before_json,
                after_json: Some(serde_json::json!({
                    "display_name": user.display_name,
                    "status": user.status,
                    "roles": user.roles,
                    "staff_profile_id": user.staff_profile_id,
                })),
            }];

            Ok((serde_json::to_value(user).unwrap(), events, audit))
        }

        "roles.list" => {
            let roles = accounts::list_roles(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let resp = ListRolesResponse { roles };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
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

            let audit = vec![crate::audit::AuditEntry {
                action: "roles.create".to_string(),
                entity: "role".to_string(),
                entity_id: role.id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "name": role.name,
                    "description": role.description,
                    "permissions": role.permissions,
                })),
            }];

            Ok((serde_json::to_value(role).unwrap(), events, audit))
        }

        "roles.update" => {
            let p: UpdateRolePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update role payload: {}", e),
                    field: None,
                })?;

            let existing_roles = accounts::list_roles(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;
            let existing_role = existing_roles.into_iter().find(|r| r.id == p.id);
            let before_json = existing_role.as_ref().map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "description": r.description,
                    "permissions": r.permissions,
                })
            });

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

            let audit = vec![crate::audit::AuditEntry {
                action: "roles.update".to_string(),
                entity: "role".to_string(),
                entity_id: role.id.clone(),
                before_json,
                after_json: Some(serde_json::json!({
                    "name": role.name,
                    "description": role.description,
                    "permissions": role.permissions,
                })),
            }];

            Ok((serde_json::to_value(role).unwrap(), events, audit))
        }

        "permissions.list" => {
            let permissions = accounts::list_permissions(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let resp = ListPermissionsResponse { permissions };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "audit.list" => {
            let p: crate::audit::ListAuditLogsPayload = serde_json::from_value(payload.clone())
                .unwrap_or(crate::audit::ListAuditLogsPayload {
                    entity: None,
                    entity_id: None,
                    action: None,
                    user_id: None,
                    date_from: None,
                    date_to: None,
                    limit: None,
                    offset: None,
                });

            let resp = crate::audit::list_audit_logs(conn, &p).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "audit.verify_chain" => {
            let resp = crate::audit::verify_audit_log_chain(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "audit.get_head" => {
            let resp = crate::audit::get_audit_head(conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "recovery.reprint_sheet" => {
            let p: crate::setup::ReprintSheetPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid reprint payload: {}", e),
                    field: None,
                })?;

            let s = session.ok_or_else(|| RpcError {
                code: "unauthenticated".to_string(),
                message: "Authentication required".to_string(),
                field: None,
            })?;

            if !s.roles.contains(&"Owner".to_string()) {
                return Err(RpcError {
                    code: "forbidden".to_string(),
                    message: "Only the Institute Owner may reprint the Recovery Sheet".to_string(),
                    field: None,
                });
            }

            let pwd_hash: String = conn
                .query_row(
                    "SELECT password_hash FROM users WHERE id = ?1",
                    rusqlite::params![s.user.id],
                    |r| r.get(0),
                )
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            match auth::verify_password(&p.owner_password, &pwd_hash) {
                Ok(true) => {}
                _ => {
                    return Err(RpcError {
                        code: "invalid_password".to_string(),
                        message: "Incorrect owner password".to_string(),
                        field: Some("owner_password".to_string()),
                    });
                }
            }

            let inst: (String, String) = conn
                .query_row(
                    "SELECT name, created_at FROM institution LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|e| RpcError {
                    code: "database_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;

            let (_, envelope_path) = if let Ok(Some(cfg)) = crate::storage::load_global_config() {
                let root = std::path::PathBuf::from(&cfg.data_dir)
                    .join(&cfg.institution_id)
                    .join("db");
                (root.join("main.db"), root.join("key.envelope"))
            } else {
                let dev_path = db::resolve_dev_db_path()
                    .unwrap_or_else(|_| std::path::PathBuf::from("dev.db"));
                let dev_env = dev_path.parent().unwrap().join("key.envelope");
                (dev_path, dev_env)
            };

            let checksum = if envelope_path.exists() {
                if let Ok(env) = crate::crypto::KeyEnvelope::load_from_file(&envelope_path) {
                    env.recovery.checksum
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let resp = crate::setup::ReprintSheetResponse {
                institution_name: inst.0.clone(),
                owner_username: s.user.username.clone(),
                created_at: inst.1,
                recovery_key_checksum: checksum.clone(),
                instructions: "Keep this sheet in a safe physical vault. If hardware fails, this sheet is required to recover your data.".to_string(),
            };

            let audit_entry = crate::audit::AuditEntry {
                action: "recovery.reprint_sheet".to_string(),
                entity: "institution".to_string(),
                entity_id: inst.0,
                before_json: None,
                after_json: Some(serde_json::json!({
                    "owner_username": s.user.username,
                    "recovery_key_checksum": checksum,
                })),
            };

            Ok((
                serde_json::to_value(resp).unwrap(),
                vec![],
                vec![audit_entry],
            ))
        }

        "students.create" => {
            let p: CreateStudentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create student payload: {}", e),
                    field: None,
                })?;

            let user_id = session.map(|s| s.user.id.as_str());
            let student = crate::students::create_student(
                conn,
                &p.student,
                p.allow_duplicate_phone.unwrap_or(false),
                user_id,
                current_utc,
            )
            .map_err(map_students_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "students.create".to_string(),
                entity: "student".to_string(),
                entity_id: student.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&student).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "students".to_string(),
                entity_ids: vec![student.id.clone()],
            }];

            Ok((serde_json::to_value(student).unwrap(), events, audit))
        }

        "students.update" => {
            let p: UpdateStudentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update student payload: {}", e),
                    field: None,
                })?;

            let user_id = session.map(|s| s.user.id.as_str());
            let student = crate::students::update_student(
                conn,
                &p.id,
                p.expected_version,
                &p.student,
                p.allow_duplicate_phone.unwrap_or(false),
                user_id,
                current_utc,
            )
            .map_err(map_students_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "students.update".to_string(),
                entity: "student".to_string(),
                entity_id: student.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&student).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "students".to_string(),
                entity_ids: vec![student.id.clone()],
            }];

            Ok((serde_json::to_value(student).unwrap(), events, audit))
        }

        "students.get" => {
            let p: GetStudentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get student payload: {}", e),
                    field: None,
                })?;

            let resp =
                crate::students::get_student_view(conn, &p.id).map_err(map_students_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "students.list" => {
            let p: crate::students::ListStudentsPayload = serde_json::from_value(payload.clone())
                .unwrap_or(crate::students::ListStudentsPayload {
                    status: None,
                    limit: None,
                    offset: None,
                });

            let resp = crate::students::list_students(conn, &p).map_err(map_students_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "students.search" => {
            let p: SearchStudentsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid search students payload: {}", e),
                    field: None,
                })?;

            let matches = crate::students::search_students(conn, &p.query, p.limit)
                .map_err(map_students_error)?;
            let resp = crate::students::SearchStudentsResponse { matches };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "students.archive" => {
            let p: ArchiveStudentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid archive student payload: {}", e),
                    field: None,
                })?;

            let user_id = session.map(|s| s.user.id.as_str());
            let student = crate::students::archive_student(
                conn,
                &p.id,
                &p.reason,
                p.expected_version,
                user_id,
                current_utc,
            )
            .map_err(map_students_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "students.archive".to_string(),
                entity: "student".to_string(),
                entity_id: student.id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "status": "archived",
                    "reason": p.reason,
                })),
            }];

            let events = vec![ChangeEvent {
                topic: "students".to_string(),
                entity_ids: vec![student.id.clone()],
            }];

            Ok((serde_json::to_value(student).unwrap(), events, audit))
        }

        "guardians.search" => {
            let p: SearchGuardiansPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid search guardians payload: {}", e),
                    field: None,
                })?;

            let matches = crate::students::search_guardians(conn, &p.query, p.limit)
                .map_err(map_students_error)?;
            let resp = crate::students::SearchGuardiansResponse { matches };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "guardians.get" => {
            let p: GetGuardianPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get guardian payload: {}", e),
                    field: None,
                })?;

            let resp = crate::students::get_guardian(conn, &p.id).map_err(map_students_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "guardians.update" => {
            let p: UpdateGuardianPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update guardian payload: {}", e),
                    field: None,
                })?;

            let guardian = crate::students::update_guardian(
                conn,
                &p.id,
                &p.guardian,
                p.split_from_siblings.unwrap_or(false),
                p.student_id.as_deref(),
                current_utc,
            )
            .map_err(map_students_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "guardians.update".to_string(),
                entity: "guardian".to_string(),
                entity_id: guardian.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&guardian).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "guardians".to_string(),
                entity_ids: vec![guardian.id.clone()],
            }];

            Ok((serde_json::to_value(guardian).unwrap(), events, audit))
        }

        "vault.upload_file" => {
            let p: crate::vault::UploadFilePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid upload payload: {}", e),
                    field: None,
                })?;

            let vault_dir = crate::vault::resolve_vault_dir();
            let user_id = session.map(|s| s.user.id.as_str());
            let doc = crate::vault::upload_file(conn, &vault_dir, &p, user_id, current_utc)
                .map_err(map_vault_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "vault.upload".to_string(),
                entity: "document".to_string(),
                entity_id: doc.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&doc).unwrap()),
            }];

            Ok((serde_json::to_value(doc).unwrap(), vec![], audit))
        }

        "vault.get_file" => {
            let p: GetFilePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get file payload: {}", e),
                    field: None,
                })?;

            let vault_dir = crate::vault::resolve_vault_dir();
            let resp = crate::vault::get_file(conn, &vault_dir, &p.id).map_err(map_vault_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        // Programs
        "programs.create" => {
            let p: CreateProgramPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create program payload: {}", e),
                    field: None,
                })?;

            let program = crate::academic::create_program(conn, &p.program, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "programs.create".to_string(),
                entity: "program".to_string(),
                entity_id: program.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&program).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "programs".to_string(),
                entity_ids: vec![program.id.clone()],
            }];

            Ok((serde_json::to_value(program).unwrap(), events, audit))
        }

        "programs.update" => {
            let p: UpdateProgramPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update program payload: {}", e),
                    field: None,
                })?;

            let program = crate::academic::update_program(conn, &p.id, &p.program, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "programs.update".to_string(),
                entity: "program".to_string(),
                entity_id: program.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&program).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "programs".to_string(),
                entity_ids: vec![program.id.clone()],
            }];

            Ok((serde_json::to_value(program).unwrap(), events, audit))
        }

        "programs.get" => {
            let p: GetProgramPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get program payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::get_program(conn, &p.id).map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "programs.list" => {
            let p: ListProgramsPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListProgramsPayload {
                    session_id: None,
                    status: None,
                });

            let resp =
                crate::academic::list_programs(conn, p.session_id.as_deref(), p.status.as_deref())
                    .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        // Subjects
        "subjects.create" => {
            let p: CreateSubjectPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create subject payload: {}", e),
                    field: None,
                })?;

            let subject = crate::academic::create_subject(conn, &p.subject, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "subjects.create".to_string(),
                entity: "subject".to_string(),
                entity_id: subject.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&subject).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "subjects".to_string(),
                entity_ids: vec![subject.id.clone()],
            }];

            Ok((serde_json::to_value(subject).unwrap(), events, audit))
        }

        "subjects.list" => {
            let resp = crate::academic::list_subjects(conn).map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        // Batches
        "batches.create" => {
            let p: CreateBatchPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create batch payload: {}", e),
                    field: None,
                })?;

            let allow_conflict = p.allow_conflict.unwrap_or(false);
            let batch = crate::academic::create_batch(conn, &p.batch, allow_conflict, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "batches.create".to_string(),
                entity: "batch".to_string(),
                entity_id: batch.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&batch).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "batches".to_string(),
                entity_ids: vec![batch.id.clone()],
            }];

            Ok((serde_json::to_value(batch).unwrap(), events, audit))
        }

        "batches.update" => {
            let p: UpdateBatchPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update batch payload: {}", e),
                    field: None,
                })?;

            let allow_conflict = p.allow_conflict.unwrap_or(false);
            let batch =
                crate::academic::update_batch(conn, &p.id, &p.batch, allow_conflict, current_utc)
                    .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "batches.update".to_string(),
                entity: "batch".to_string(),
                entity_id: batch.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&batch).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "batches".to_string(),
                entity_ids: vec![batch.id.clone()],
            }];

            Ok((serde_json::to_value(batch).unwrap(), events, audit))
        }

        "batches.get" => {
            let p: GetBatchPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get batch payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::get_batch(conn, &p.id).map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "batches.list" => {
            let p: ListBatchesPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListBatchesPayload {
                    program_id: None,
                    teacher_id: None,
                    status: None,
                });

            let resp = crate::academic::list_batches(
                conn,
                p.program_id.as_deref(),
                p.teacher_id.as_deref(),
                p.status.as_deref(),
            )
            .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "batches.check_conflicts" => {
            let p: CheckConflictsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid check conflicts payload: {}", e),
                    field: None,
                })?;

            let conflicts = crate::schedule::detect_conflicts(
                conn,
                p.batch_id.as_deref(),
                p.teacher_staff_id.as_deref(),
                p.room_number.as_deref(),
                &p.schedule,
            )
            .map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            Ok((serde_json::to_value(conflicts).unwrap(), vec![], vec![]))
        }

        "batches.set_subjects" => {
            let p: SetBatchSubjectsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid set batch subjects payload: {}", e),
                    field: None,
                })?;

            crate::academic::set_batch_subjects(conn, &p.batch_id, &p.subjects)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "batches.set_subjects".to_string(),
                entity: "batch".to_string(),
                entity_id: p.batch_id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&p.subjects).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "batches".to_string(),
                entity_ids: vec![p.batch_id.clone()],
            }];

            Ok((serde_json::json!({ "success": true }), events, audit))
        }

        // Staff
        "staff.create" => {
            let p: CreateStaffPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create staff payload: {}", e),
                    field: None,
                })?;

            let staff = crate::academic::create_staff(conn, &p.staff, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "staff.create".to_string(),
                entity: "staff".to_string(),
                entity_id: staff.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&staff).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "staff".to_string(),
                entity_ids: vec![staff.id.clone()],
            }];

            Ok((serde_json::to_value(staff).unwrap(), events, audit))
        }

        "staff.update" => {
            let p: UpdateStaffPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update staff payload: {}", e),
                    field: None,
                })?;

            let staff = crate::academic::update_staff(conn, &p.id, &p.staff, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "staff.update".to_string(),
                entity: "staff".to_string(),
                entity_id: staff.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&staff).unwrap()),
            }];

            let events = vec![ChangeEvent {
                topic: "staff".to_string(),
                entity_ids: vec![staff.id.clone()],
            }];

            Ok((serde_json::to_value(staff).unwrap(), events, audit))
        }

        "staff.get" => {
            let p: GetStaffPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get staff payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::get_staff(conn, &p.id).map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "staff.list" => {
            let p: ListStaffPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListStaffPayload {
                    status: None,
                    department: None,
                    limit: None,
                    offset: None,
                });

            let resp = crate::academic::list_staff(
                conn,
                p.status.as_deref(),
                p.department.as_deref(),
                p.limit,
                p.offset,
            )
            .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "staff.search" => {
            let p: SearchStaffPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid search staff payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::search_staff(conn, &p.query, p.limit)
                .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        // Enrollments
        "enrollments.create" => {
            let p: CreateEnrollmentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create enrollment payload: {}", e),
                    field: None,
                })?;

            let allow_overcapacity = p.allow_overcapacity.unwrap_or(false);
            let enrollment = crate::academic::create_enrollment(
                conn,
                &p.enrollment,
                allow_overcapacity,
                current_utc,
            )
            .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "enrollments.create".to_string(),
                entity: "enrollment".to_string(),
                entity_id: enrollment.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&enrollment).unwrap()),
            }];

            let events = vec![
                ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: vec![enrollment.id.clone()],
                },
                ChangeEvent {
                    topic: "batches".to_string(),
                    entity_ids: vec![enrollment.batch_id.clone()],
                },
                ChangeEvent {
                    topic: "students".to_string(),
                    entity_ids: vec![enrollment.student_id.clone()],
                },
            ];

            Ok((serde_json::to_value(enrollment).unwrap(), events, audit))
        }

        "enrollments.transfer" => {
            let p: TransferEnrollmentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid transfer enrollment payload: {}", e),
                    field: None,
                })?;

            let allow_overcapacity = p.allow_overcapacity.unwrap_or(false);
            let enrollment = crate::academic::transfer_enrollment(
                conn,
                &p.transfer,
                allow_overcapacity,
                current_utc,
            )
            .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "enrollments.transfer".to_string(),
                entity: "enrollment".to_string(),
                entity_id: enrollment.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&enrollment).unwrap()),
            }];

            let events = vec![
                ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: vec![enrollment.id.clone()],
                },
                ChangeEvent {
                    topic: "batches".to_string(),
                    entity_ids: vec![
                        p.transfer.from_batch_id.clone(),
                        enrollment.batch_id.clone(),
                    ],
                },
                ChangeEvent {
                    topic: "students".to_string(),
                    entity_ids: vec![enrollment.student_id.clone()],
                },
            ];

            Ok((serde_json::to_value(enrollment).unwrap(), events, audit))
        }

        "enrollments.end" => {
            let p: EndEnrollmentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid end enrollment payload: {}", e),
                    field: None,
                })?;

            let enrollment = crate::academic::end_enrollment(conn, &p.enrollment, current_utc)
                .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "enrollments.end".to_string(),
                entity: "enrollment".to_string(),
                entity_id: enrollment.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&enrollment).unwrap()),
            }];

            let events = vec![
                ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: vec![enrollment.id.clone()],
                },
                ChangeEvent {
                    topic: "batches".to_string(),
                    entity_ids: vec![enrollment.batch_id.clone()],
                },
                ChangeEvent {
                    topic: "students".to_string(),
                    entity_ids: vec![enrollment.student_id.clone()],
                },
            ];

            Ok((serde_json::to_value(enrollment).unwrap(), events, audit))
        }

        "enrollments.bulk_create" => {
            let p: BulkEnrollPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid bulk enroll payload: {}", e),
                    field: None,
                })?;

            let allow_overcapacity = p.allow_overcapacity.unwrap_or(false);
            let enrollments = crate::academic::bulk_create_enrollments(
                conn,
                &p.bulk,
                allow_overcapacity,
                current_utc,
            )
            .map_err(map_academic_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "enrollments.bulk_create".to_string(),
                entity: "batch".to_string(),
                entity_id: p.bulk.batch_id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "count": enrollments.len(),
                    "batch_id": p.bulk.batch_id,
                })),
            }];

            let events = vec![
                ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: enrollments.iter().map(|e| e.id.clone()).collect(),
                },
                ChangeEvent {
                    topic: "batches".to_string(),
                    entity_ids: vec![p.bulk.batch_id.clone()],
                },
            ];

            Ok((serde_json::to_value(enrollments).unwrap(), events, audit))
        }

        "enrollments.list_by_batch" => {
            let p: ListBatchRosterPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid list batch roster payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::list_batch_roster(conn, &p.batch_id, p.status.as_deref())
                .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "enrollments.list_by_student" => {
            let p: ListStudentEnrollmentsPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid list student enrollments payload: {}", e),
                    field: None,
                })?;

            let resp = crate::academic::list_student_enrollments(conn, &p.student_id)
                .map_err(map_academic_error)?;
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "import.inspect_file" => {
            let p: crate::import::InspectFilePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid inspect file payload: {}", e),
                    field: None,
                })?;

            let res = crate::import::inspect_file(std::path::Path::new(&p.file_path))
                .map_err(map_import_error)?;
            Ok((serde_json::to_value(res).unwrap(), vec![], vec![]))
        }

        "import.preview_validation" => {
            let p: crate::import::PreviewValidationPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid preview validation payload: {}", e),
                    field: None,
                })?;

            let res = crate::import::preview_validation(conn, &p).map_err(map_import_error)?;
            Ok((serde_json::to_value(res).unwrap(), vec![], vec![]))
        }

        "import.execute" => {
            let p: crate::import::ExecuteImportInput = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid execute import payload: {}", e),
                    field: None,
                })?;

            let user_id = session.map(|s| s.user.id.as_str());
            let res = crate::import::execute_import(conn, &p, user_id, current_utc)
                .map_err(map_import_error)?;

            let mut events = vec![
                ChangeEvent {
                    topic: "students".to_string(),
                    entity_ids: vec![],
                },
                ChangeEvent {
                    topic: "guardians".to_string(),
                    entity_ids: vec![],
                },
            ];
            if p.target_batch_id.is_some() {
                events.push(ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: vec![],
                });
            }

            Ok((serde_json::to_value(res).unwrap(), events, vec![]))
        }

        "generator.generate_dataset" => {
            let p: crate::generator::GenerateDatasetPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid generate dataset payload: {}", e),
                    field: None,
                })?;

            let mut gen =
                crate::generator::DatasetGenerator::new(p.profile, p.seed, p.out_filename)
                    .map_err(|e| RpcError {
                        code: match &e {
                            db::DbError::Config(msg) if msg.contains("disk space") => {
                                "low_disk_space".to_string()
                            }
                            db::DbError::Config(msg)
                                if msg.contains("active production database") =>
                            {
                                "conflict".to_string()
                            }
                            _ => "generator_error".to_string(),
                        },
                        message: e.to_string(),
                        field: None,
                    })?;

            let summary = gen.generate().map_err(|e| RpcError {
                code: "generator_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let audit = vec![crate::audit::AuditEntry {
                action: "generator.generate_dataset".to_string(),
                entity: "demo_dataset".to_string(),
                entity_id: summary.profile.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&summary).unwrap_or_default()),
            }];

            let events = vec![ChangeEvent {
                topic: "generator:status".to_string(),
                entity_ids: vec![summary.profile.clone()],
            }];

            Ok((serde_json::to_value(summary).unwrap(), events, audit))
        }

        "generator.get_status" => {
            let status = crate::generator::get_current_status();
            Ok((serde_json::to_value(status).unwrap(), vec![], vec![]))
        }

        "generator.switch_database" => {
            let p: crate::generator::SwitchDatabasePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid switch database payload: {}", e),
                    field: None,
                })?;

            if p.target_filename.contains("..")
                || p.target_filename.contains('/')
                || p.target_filename.contains('\\')
            {
                return Err(RpcError {
                    code: "validation_error".to_string(),
                    message: "Invalid target database filename".to_string(),
                    field: Some("target_filename".to_string()),
                });
            }

            let target_path = crate::generator::resolve_demo_dir().join(&p.target_filename);
            let active_path = db::switch_to_demo_database(&target_path).map_err(|e| RpcError {
                code: match &e {
                    db::DbError::Integrity(_) => "integrity_check_failed".to_string(),
                    db::DbError::Config(msg) if msg.contains("does not exist") => {
                        "not_found".to_string()
                    }
                    db::DbError::Config(msg) if msg.contains("active production database") => {
                        "conflict".to_string()
                    }
                    _ => "database_error".to_string(),
                },
                message: e.to_string(),
                field: None,
            })?;

            let audit = vec![crate::audit::AuditEntry {
                action: "generator.switch_database".to_string(),
                entity: "database".to_string(),
                entity_id: p.target_filename.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "active_path": active_path,
                })),
            }];

            let events = vec![ChangeEvent {
                topic: "database:switched".to_string(),
                entity_ids: vec![active_path.clone()],
            }];

            let resp = crate::generator::SwitchDatabaseResponse {
                success: true,
                active_path,
            };
            Ok((serde_json::to_value(resp).unwrap(), events, audit))
        }

        "generator.switch_to_production_database" => {
            let prod_path = db::switch_to_production_database().map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let audit = vec![crate::audit::AuditEntry {
                action: "generator.switch_to_production_database".to_string(),
                entity: "database".to_string(),
                entity_id: "production".to_string(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "production_path": prod_path,
                })),
            }];

            let events = vec![ChangeEvent {
                topic: "database:switched".to_string(),
                entity_ids: vec![prod_path.clone()],
            }];

            let resp = crate::generator::SwitchToProductionResponse {
                success: true,
                production_path: prod_path,
            };
            Ok((serde_json::to_value(resp).unwrap(), events, audit))
        }

        _ => Err(RpcError {
            code: "unknown_command".to_string(),
            message: format!("Unhandled command: {}", command),
            field: None,
        }),
    }
}

pub fn verify_generator_permission(
    session: Option<&accounts::ResolvedSession>,
    enforce_production_rule: bool,
) -> Result<(), RpcError> {
    if !enforce_production_rule {
        return Ok(());
    }
    let Some(s) = session else {
        return Err(RpcError {
            code: "unauthenticated".to_string(),
            message: "Authentication required for generator commands".to_string(),
            field: None,
        });
    };
    if !s.roles.iter().any(|r| r == "Owner") {
        return Err(RpcError {
            code: "forbidden".to_string(),
            message: "Generator commands require the Owner role in production".to_string(),
            field: None,
        });
    }
    Ok(())
}

fn map_students_error(err: crate::students::StudentsError) -> RpcError {
    match err {
        crate::students::StudentsError::StudentCodeExists(c) => RpcError {
            code: "student_code_exists".to_string(),
            message: format!("Student code '{}' already exists", c),
            field: Some("student_code".to_string()),
        },
        crate::students::StudentsError::InvalidPhone(msg) => RpcError {
            code: "invalid_phone".to_string(),
            message: msg,
            field: Some("phone".to_string()),
        },
        crate::students::StudentsError::PhoneConflictWarning(conflict) => RpcError {
            code: "phone_conflict_warning".to_string(),
            message: conflict.to_string(),
            field: Some("phone".to_string()),
        },
        crate::students::StudentsError::PrimaryGuardianMissing => RpcError {
            code: "primary_guardian_missing".to_string(),
            message: "A student record must have exactly one primary guardian".to_string(),
            field: Some("guardians".to_string()),
        },
        crate::students::StudentsError::MultiplePrimaryGuardians => RpcError {
            code: "multiple_primary_guardians".to_string(),
            message: "A student record cannot have multiple primary guardians".to_string(),
            field: Some("guardians".to_string()),
        },
        crate::students::StudentsError::SmsGuardianMissing => RpcError {
            code: "sms_guardian_missing".to_string(),
            message: "At least one guardian must be designated to receive SMS".to_string(),
            field: Some("guardians".to_string()),
        },
        crate::students::StudentsError::StudentNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Student with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::students::StudentsError::GuardianNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Guardian with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::students::StudentsError::ConcurrencyConflict { expected, actual } => RpcError {
            code: "concurrency_conflict".to_string(),
            message: format!(
                "Record was modified by another user (expected version {}, found {})",
                expected, actual
            ),
            field: Some("expected_version".to_string()),
        },
        crate::students::StudentsError::InvalidStatusTransition { from, to } => RpcError {
            code: "invalid_status_transition".to_string(),
            message: format!("Invalid status transition from '{}' to '{}'", from, to),
            field: Some("status".to_string()),
        },
        crate::students::StudentsError::Sql(msg) => RpcError {
            code: "database_error".to_string(),
            message: msg,
            field: None,
        },
    }
}

fn map_vault_error(err: crate::vault::VaultError) -> RpcError {
    match err {
        crate::vault::VaultError::FileTooLarge { size } => RpcError {
            code: "file_too_large".to_string(),
            message: format!("File size ({} bytes) exceeds maximum limit of 20MB", size),
            field: None,
        },
        crate::vault::VaultError::DimensionsExceeded { width, height } => RpcError {
            code: "image_decode_failed".to_string(),
            message: format!(
                "Image dimensions ({}x{}) exceed maximum limit of 6000x6000",
                width, height
            ),
            field: None,
        },
        crate::vault::VaultError::ImageDecodeFailed(msg) => RpcError {
            code: "image_decode_failed".to_string(),
            message: msg,
            field: None,
        },
        crate::vault::VaultError::NotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Document with ID '{}' not found", id),
            field: None,
        },
        crate::vault::VaultError::Base64Decode(msg) => RpcError {
            code: "invalid_base64".to_string(),
            message: msg,
            field: None,
        },
        crate::vault::VaultError::UnsupportedFormat(fmt) => RpcError {
            code: "unsupported_format".to_string(),
            message: format!("Unsupported file format: {}", fmt),
            field: None,
        },
        other => RpcError {
            code: "io_error".to_string(),
            message: other.to_string(),
            field: None,
        },
    }
}

fn map_academic_error(err: crate::academic::AcademicError) -> RpcError {
    match err {
        crate::academic::AcademicError::ProgramNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Program with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::academic::AcademicError::ProgramCodeExists(code) => RpcError {
            code: "program_code_exists".to_string(),
            message: format!("Program code '{}' already exists in this session", code),
            field: Some("code".to_string()),
        },
        crate::academic::AcademicError::SessionNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Academic session with ID '{}' not found", id),
            field: Some("session_id".to_string()),
        },
        crate::academic::AcademicError::BatchNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Batch with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::academic::AcademicError::BatchCodeExists(code) => RpcError {
            code: "batch_code_exists".to_string(),
            message: format!("Batch code '{}' already exists in this program", code),
            field: Some("code".to_string()),
        },
        crate::academic::AcademicError::SubjectNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Subject with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::academic::AcademicError::SubjectCodeExists(code) => RpcError {
            code: "subject_code_exists".to_string(),
            message: format!("Subject code '{}' already exists", code),
            field: Some("code".to_string()),
        },
        crate::academic::AcademicError::StaffNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Staff member with ID '{}' not found", id),
            field: Some("id".to_string()),
        },
        crate::academic::AcademicError::StudentNotFound(id) => RpcError {
            code: "not_found".to_string(),
            message: format!("Student with ID '{}' not found", id),
            field: Some("student_id".to_string()),
        },
        crate::academic::AcademicError::StudentNotActive(id) => RpcError {
            code: "student_not_active".to_string(),
            message: format!("Student '{}' is not active", id),
            field: Some("student_id".to_string()),
        },
        crate::academic::AcademicError::AlreadyEnrolled => RpcError {
            code: "already_enrolled".to_string(),
            message: "Student is already actively enrolled in this batch".to_string(),
            field: Some("batch_id".to_string()),
        },
        crate::academic::AcademicError::NotEnrolledInBatch(id) => RpcError {
            code: "not_enrolled".to_string(),
            message: format!("Student is not actively enrolled in source batch '{}'", id),
            field: Some("from_batch_id".to_string()),
        },
        crate::academic::AcademicError::RollTaken { roll } => RpcError {
            code: "roll_taken".to_string(),
            message: format!(
                "Roll number {} is already assigned to an active student in this batch",
                roll
            ),
            field: Some("roll".to_string()),
        },
        crate::academic::AcademicError::BatchCapacityReached { capacity, enrolled } => RpcError {
            code: "batch_capacity_reached".to_string(),
            message: format!(
                "Batch has reached its maximum capacity of {} students ({} enrolled)",
                capacity, enrolled
            ),
            field: Some("capacity".to_string()),
        },
        crate::academic::AcademicError::ScheduleConflict(conflicts) => RpcError {
            code: "schedule_conflict".to_string(),
            message: serde_json::to_string(&conflicts)
                .unwrap_or_else(|_| "Schedule conflict detected".to_string()),
            field: Some("schedule".to_string()),
        },
        crate::academic::AcademicError::InvalidSchedule(msg) => RpcError {
            code: "invalid_schedule".to_string(),
            message: msg,
            field: Some("schedule".to_string()),
        },
        crate::academic::AcademicError::InvalidPhone(msg) => RpcError {
            code: "invalid_phone".to_string(),
            message: msg,
            field: Some("phone".to_string()),
        },
        crate::academic::AcademicError::CannotDelete(msg) => RpcError {
            code: "cannot_delete".to_string(),
            message: msg,
            field: None,
        },
        crate::academic::AcademicError::InvalidStatusTransition { from, to } => RpcError {
            code: "invalid_status_transition".to_string(),
            message: format!("Invalid status transition from '{}' to '{}'", from, to),
            field: Some("status".to_string()),
        },
        crate::academic::AcademicError::Sql(msg) => RpcError {
            code: "database_error".to_string(),
            message: msg,
            field: None,
        },
    }
}

fn map_import_error(err: crate::import::ImportError) -> RpcError {
    match err {
        crate::import::ImportError::FileNotFound(msg) => RpcError {
            code: "file_not_found".to_string(),
            message: msg,
            field: Some("file_path".to_string()),
        },
        crate::import::ImportError::UnsupportedFormat(msg) => RpcError {
            code: "unsupported_format".to_string(),
            message: msg,
            field: Some("file_path".to_string()),
        },
        crate::import::ImportError::CorruptFile(msg) => RpcError {
            code: "corrupt_file".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::InvalidMappings(msg) => RpcError {
            code: "invalid_mappings".to_string(),
            message: msg,
            field: Some("mappings".to_string()),
        },
        crate::import::ImportError::ParsingFailed(msg) => RpcError {
            code: "parsing_failed".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::BackupFailed(msg) => RpcError {
            code: "backup_failed".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::TransactionAborted(msg) => RpcError {
            code: "transaction_aborted".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::AllRowsInvalid(msg) => RpcError {
            code: "all_rows_invalid".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::Sql(msg) => RpcError {
            code: "database_error".to_string(),
            message: msg,
            field: None,
        },
        crate::import::ImportError::PermissionDenied(msg) => RpcError {
            code: "permission_denied".to_string(),
            message: msg,
            field: None,
        },
    }
}
