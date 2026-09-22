pub mod academic;
pub mod accounts;
pub mod audit;
pub mod auth;
pub mod crypto;
pub mod db;
pub mod dispatcher;
pub mod fees;
pub mod generator;
pub mod import;
pub mod ledger;
pub mod migrations;
pub mod phone;
pub mod schedule;
pub mod setup;
pub mod storage;
pub mod students;
pub mod vault;
pub mod words;

pub use academic::*;
pub use fees::*;
pub use generator::*;
pub use import::*;
pub use ledger::*;
pub use schedule::*;
pub use words::*;

pub use dispatcher::{
    BootstrapOwnerPayload, BootstrapOwnerResponse, BulkEnrollPayload, ChangeEvent,
    ChangePasswordPayload, CheckConflictsPayload, CreateAdjustmentPayload, CreateBatchPayload,
    CreateConcessionPayload, CreateEnrollmentPayload, CreateFeeHeadPayload, CreateFeePlanPayload,
    CreateManualChargePayload, CreatePaymentPayload, CreateProgramPayload, CreateRefundPayload,
    CreateRolePayload, CreateStaffPayload, CreateSubjectPayload, CreateUserPayload,
    CurrentSessionResponse, DeleteConcessionPayload, DeleteFeeHeadPayload, DeleteFeePlanPayload,
    EndEnrollmentPayload, GenerateMonthlyChargesPayload, GetBatchPayload,
    GetCashierStudentOverviewPayload, GetPrintableReceiptPayload, GetProgramPayload,
    GetStaffPayload, GetStatusResponse, GetStudentLedgerSummaryPayload, HeartbeatResponse,
    ListBatchRosterPayload, ListBatchesPayload, ListConcessionsPayload, ListFeePlansPayload,
    ListPermissionsResponse, ListProgramsPayload, ListRolesResponse, ListStaffPayload,
    ListStudentChargesPayload, ListStudentEnrollmentsPayload, ListStudentPaymentsPayload,
    ListUsersPayload, ListUsersResponse, LoginPayload, LoginResponse, MigrationHistoryItem,
    MigrationHistoryResponse, MigrationStatusResponse, OpenDbDirectoryResponse,
    PendingMigrationInfo, PingPayload, PingResponse, RecoveryInfoResponse, RpcEnvelope, RpcError,
    SavePrinterSettingsPayload, SearchStaffPayload, SetBatchSubjectsPayload, SuccessResponse,
    TransferEnrollmentPayload, UnlockPayload, UnlockResponse, UpdateBatchPayload,
    UpdateFeeHeadPayload, UpdateFeePlanPayload, UpdateProgramPayload, UpdateRolePayload,
    UpdateStaffPayload, UpdateUserPayload, VoidChargePayload, VoidPaymentPayload,
};

pub use accounts::{DeviceSummary, PermissionItem, RoleWithPermissions, UserSummary};

pub use audit::{
    AuditEntry, AuditError, AuditLogItem, GetAuditHeadResponse, ListAuditLogsPayload,
    ListAuditLogsResponse, VerifyAuditChainResponse,
};

pub use crypto::KeyEnvelope;
pub use setup::{
    DetectDrivesResponse, GenerateRecoveryPreviewResponse, InitializeInstitutionPayload,
    InitializeInstitutionResponse, InspectDatabasePayload, InspectDatabaseResponse,
    ReprintSheetPayload, ReprintSheetResponse, SetupStatusResponse, UnlockWithKeyPayload,
    UnlockWithKeyResponse, ValidatePathPayload,
};
pub use storage::{
    BackupValidationResult, DriveInfo, PathValidationResult, SystemPrinterInfo,
    WorkstationPrinterSettings,
};

/// Executes a typed command through the centralized command dispatcher, returning
/// the serialized JSON envelope and any broadcast change events.
pub fn handle_rpc_with_events(
    command: &str,
    payload_json: &str,
    session_token: Option<&str>,
    idempotency_key: Option<&str>,
) -> (String, Vec<ChangeEvent>) {
    if command == "system.recovery_info" {
        let db_path =
            db::resolve_dev_db_path().unwrap_or_else(|_| std::path::PathBuf::from("dev.db"));
        let resp = migrations::get_recovery_info(&db_path);
        let env: RpcEnvelope<migrations::RecoveryInfoResponse> = RpcEnvelope {
            ok: true,
            data: Some(resp),
            error: None,
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

    // Outside recovery, opening the db directory goes through the normal
    // permission-gated dispatch path below; during recovery, the UI needs it
    // before a session exists, so it is served here unauthenticated.
    if command == "system.open_db_directory" && migrations::is_in_recovery() {
        let db_path =
            db::resolve_dev_db_path().unwrap_or_else(|_| std::path::PathBuf::from("dev.db"));
        let res = migrations::open_directory_in_explorer(&db_path);
        let resp = migrations::OpenDbDirectoryResponse {
            success: res.is_ok(),
            path: db_path.display().to_string(),
            error: res.err().map(|e| e.to_string()),
        };
        let env: RpcEnvelope<migrations::OpenDbDirectoryResponse> = RpcEnvelope {
            ok: true,
            data: Some(resp),
            error: None,
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

    if command == "system.retry_migration" {
        let db_path =
            db::resolve_dev_db_path().unwrap_or_else(|_| std::path::PathBuf::from("dev.db"));
        let _ = migrations::run_startup_lifecycle();
        let resp = migrations::get_recovery_info(&db_path);
        let env: RpcEnvelope<migrations::RecoveryInfoResponse> = RpcEnvelope {
            ok: true,
            data: Some(resp),
            error: None,
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

    if command == "system.export_diagnostics" {
        let db_path =
            db::resolve_dev_db_path().unwrap_or_else(|_| std::path::PathBuf::from("dev.db"));
        let json = migrations::export_sanitized_diagnostics_for_state(&db_path);
        let env: RpcEnvelope<migrations::ExportDiagnosticsResponse> = RpcEnvelope {
            ok: true,
            data: Some(migrations::ExportDiagnosticsResponse { json }),
            error: None,
        };
        return (serde_json::to_string(&env).unwrap(), vec![]);
    }

    if command == "setup.get_status" {
        match setup::get_setup_status() {
            Ok(resp) => {
                let env: RpcEnvelope<setup::SetupStatusResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "setup_error".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if command == "setup.detect_drives" {
        match setup::detect_drives() {
            Ok(resp) => {
                let env: RpcEnvelope<setup::DetectDrivesResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "drive_detection_error".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if command == "setup.validate_path" {
        let p: setup::ValidatePathPayload = match serde_json::from_str(payload_json) {
            Ok(val) => val,
            Err(e) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        };

        if p.kind == "backup" {
            let res = storage::validate_backup_destination(
                std::path::Path::new(&p.path),
                p.expected_institution_id.as_deref(),
            );
            let env: RpcEnvelope<storage::BackupValidationResult> = RpcEnvelope {
                ok: true,
                data: Some(res),
                error: None,
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        } else {
            let res = storage::validate_data_folder(std::path::Path::new(&p.path));
            let env: RpcEnvelope<storage::PathValidationResult> = RpcEnvelope {
                ok: true,
                data: Some(res),
                error: None,
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        }
    }

    if command == "setup.generate_recovery_preview" {
        match setup::generate_recovery_preview() {
            Ok(resp) => {
                let env: RpcEnvelope<setup::GenerateRecoveryPreviewResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "crypto_error".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if command == "setup.initialize_institution" {
        let p: setup::InitializeInstitutionPayload = match serde_json::from_str(payload_json) {
            Ok(val) => val,
            Err(e) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        };

        match setup::initialize_institution(&p) {
            Ok(resp) => {
                let env: RpcEnvelope<setup::InitializeInstitutionResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "setup_failed".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if command == "recovery.inspect_database" {
        let p: setup::InspectDatabasePayload = serde_json::from_str(payload_json)
            .unwrap_or(setup::InspectDatabasePayload { db_dir: None });
        match setup::inspect_database(p.db_dir.as_deref()) {
            Ok(resp) => {
                let env: RpcEnvelope<setup::InspectDatabaseResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "inspect_error".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if command == "recovery.unlock_with_key" {
        let p: setup::UnlockWithKeyPayload = match serde_json::from_str(payload_json) {
            Ok(val) => val,
            Err(e) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        };

        match setup::unlock_database_with_key(&p) {
            Ok(resp) => {
                let env: RpcEnvelope<setup::UnlockWithKeyResponse> = RpcEnvelope {
                    ok: true,
                    data: Some(resp),
                    error: None,
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
            Err(err) => {
                let env: RpcEnvelope<()> = RpcEnvelope {
                    ok: false,
                    data: None,
                    error: Some(RpcError {
                        code: "unlock_failed".to_string(),
                        message: err.to_string(),
                        field: None,
                    }),
                };
                return (serde_json::to_string(&env).unwrap(), vec![]);
            }
        }
    }

    if migrations::is_in_recovery() && command != "system.ping" {
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

    let mut conn = match db::open_database() {
        Ok(c) => c,
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
    };

    dispatcher::dispatch(
        &mut conn,
        command,
        payload_json,
        session_token,
        idempotency_key,
    )
}

/// Executes a typed command through the centralized command dispatcher without session or idempotency keys.
pub fn handle_rpc(command: &str, payload_json: &str) -> String {
    let (response, _) = handle_rpc_with_events(command, payload_json, None, None);
    response
}

pub fn generate_ts_definitions() -> String {
    r#"/**
 * Auto-generated by Coachify xtask export-types.
 * UI <-> Core boundary contract. Do not edit manually.
 */

import { invoke } from "@tauri-apps/api/core";

export interface PingPayload {
  message?: string | null;
}

export interface PingResponse {
  status: string;
  database_status: string;
  database_path: string;
  roundtrip_timestamp: string;
  echo?: string | null;
}

export interface GetStatusResponse {
  needs_setup: boolean;
  institution_name?: string | null;
  app_version: string;
}

export interface BootstrapOwnerPayload {
  username: string;
  display_name: string;
  password: string;
}

export interface BootstrapOwnerResponse {
  session_token: string;
  user: UserSummary;
  expires_at: string;
}

export interface LoginPayload {
  username: string;
  password: string;
  device_name?: string | null;
}

export interface LoginResponse {
  session_token: string;
  user: UserSummary;
  roles: string[];
  permissions: string[];
  expires_at: string;
}

export interface UnlockPayload {
  password: string;
}

export interface UnlockResponse {
  success: boolean;
  extended_expires_at: string;
}

export interface ChangePasswordPayload {
  current_password: string;
  new_password: string;
}

export interface SuccessResponse {
  success: boolean;
}

export interface CurrentSessionResponse {
  user: UserSummary;
  roles: string[];
  permissions: string[];
  expires_at: string;
  absolute_expires_at: string;
  device: DeviceSummary;
}

export interface HeartbeatResponse {
  extended_expires_at: string;
}

export interface ListUsersPayload {
  status_filter?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface ListUsersResponse {
  users: UserSummary[];
  total_count: number;
}

export interface CreateUserPayload {
  username: string;
  display_name: string;
  password: string;
  role_ids: string[];
  staff_profile_id?: string | null;
}

export interface UpdateUserPayload {
  id: string;
  display_name: string;
  status: string;
  role_ids: string[];
  staff_profile_id?: string | null;
}

export interface ListRolesResponse {
  roles: RoleWithPermissions[];
}

export interface CreateRolePayload {
  name: string;
  description?: string | null;
  permission_codes: string[];
}

export interface UpdateRolePayload {
  id: string;
  name: string;
  description?: string | null;
  permission_codes: string[];
}

export interface ListPermissionsResponse {
  permissions: PermissionItem[];
}

export interface UserSummary {
  id: string;
  username: string;
  display_name: string;
  status: string;
  must_change_password: boolean;
  roles: string[];
  staff_profile_id?: string | null;
  created_at: string;
  updated_at: string;
}

export interface RoleWithPermissions {
  id: string;
  name: string;
  description?: string | null;
  is_system: boolean;
  permissions: string[];
  created_at: string;
  updated_at: string;
}

export interface PermissionItem {
  code: string;
  module: string;
  description: string;
}

export interface ListAuditLogsPayload {
  entity?: string | null;
  entity_id?: string | null;
  action?: string | null;
  user_id?: string | null;
  date_from?: string | null;
  date_to?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface AuditLogItem {
  seq: number;
  at: string;
  business_date: string;
  user_id?: string | null;
  user_display_name?: string | null;
  device_id?: string | null;
  device_name?: string | null;
  action: string;
  entity: string;
  entity_id: string;
  before_json?: string | null;
  after_json?: string | null;
  prev_hash: string;
  hash: string;
}

export interface ListAuditLogsResponse {
  items: AuditLogItem[];
  total_count: number;
  head_seq?: number | null;
  head_hash?: string | null;
}

export interface VerifyAuditChainResponse {
  is_valid: boolean;
  total_records: number;
  head_seq?: number | null;
  head_hash?: string | null;
  broken_seq?: number | null;
  error_message?: string | null;
  verified_at: string;
}

export interface GetAuditHeadResponse {
  head_seq?: number | null;
  head_hash?: string | null;
  total_records: number;
}

export interface DeviceSummary {
  id: string;
  name: string;
  windows_name: string;
  is_host: boolean;
}

export interface DriveInfo {
  letter: string;
  label: string;
  drive_type: string;
  total_bytes: number;
  free_bytes: number;
  is_system: boolean;
  is_recommended: boolean;
}

export interface SetupStatusResponse {
  needs_setup: boolean;
  institution_id?: string | null;
  institution_name?: string | null;
  app_version: string;
  default_data_dir: string;
  is_non_system_recommended: boolean;
}

export interface DetectDrivesResponse {
  drives: DriveInfo[];
  recommended_path: string;
  is_non_system_recommended: boolean;
}

export interface ValidatePathPayload {
  path: string;
  kind: string;
  expected_institution_id?: string | null;
}

export interface PathValidationResult {
  valid: boolean;
  path: string;
  free_bytes: number;
  total_bytes: number;
  is_non_system: boolean;
  error?: string | null;
}

export interface BackupValidationResult {
  valid: boolean;
  path: string;
  free_bytes: number;
  has_existing_marker: boolean;
  marker_institution_id?: string | null;
  error?: string | null;
}

export type ValidatePathResponse = PathValidationResult | BackupValidationResult;

export interface GenerateRecoveryPreviewResponse {
  recovery_key: string;
  checksum: string;
}

export interface InitializeInstitutionPayload {
  institution_name: string;
  institution_name_bn?: string | null;
  address?: string | null;
  phone?: string | null;
  email?: string | null;
  academic_session_name: string;
  student_id_prefix?: string | null;
  receipt_prefix?: string | null;
  data_dir: string;
  backup_destination?: string | null;
  owner_display_name: string;
  owner_username: string;
  owner_password: string;
  recovery_key: string;
  recovery_key_confirmation: string;
}

export interface InitializeInstitutionResponse {
  session_token: string;
  user: UserSummary;
  expires_at: string;
  recovery_key: string;
  institution_id: string;
  data_dir: string;
}

export interface InspectDatabasePayload {
  db_dir?: string | null;
}

export interface InspectDatabaseResponse {
  database_exists: boolean;
  has_key_envelope: boolean;
  can_unwrap_dpapi: boolean;
  needs_recovery: boolean;
  institution_id?: string | null;
  institution_name?: string | null;
  schema_version?: number | null;
}

export interface UnlockWithKeyPayload {
  db_dir?: string | null;
  recovery_key: string;
  reset_owner_password?: string | null;
}

export interface UnlockWithKeyResponse {
  success: boolean;
  institution_id: string;
  institution_name: string;
  owner_password_reset: boolean;
  message: string;
}

export interface ReprintSheetPayload {
  owner_password: string;
}

export interface ReprintSheetResponse {
  institution_name: string;
  owner_username: string;
  created_at: string;
  recovery_key_checksum: string;
  instructions: string;
}

export interface PendingMigrationInfo {
  version: number;
  name: string;
  sha256: string;
}

export interface MigrationStatusResponse {
  current_version: number;
  target_version: number;
  is_upgrade_needed: boolean;
  pending_migrations: PendingMigrationInfo[];
}

export interface MigrationHistoryItem {
  version: number;
  name: string;
  sha256: string;
  applied_at: string;
  execution_time_ms: number;
}

export interface MigrationHistoryResponse {
  history: MigrationHistoryItem[];
}

export interface RecoveryInfoResponse {
  is_in_recovery: boolean;
  failed_version?: number | null;
  error_message?: string | null;
  active_db_path: string;
  archive_paths: string[];
}

export interface OpenDbDirectoryResponse {
  success: boolean;
  path: string;
  error?: string | null;
}

export interface ExportDiagnosticsResponse {
  json: string;
}

export interface ChangeEvent {
  topic: string;
  entity_ids: string[];
}

export interface CreateGuardianInput {
  name: string;
  name_bn?: string | null;
  relationship: string;
  occupation?: string | null;
  monthly_income_poisha?: number | null;
  nid?: string | null;
  phone: string;
  phone2?: string | null;
  whatsapp_number?: string | null;
  email?: string | null;
  present_address?: string | null;
  permanent_address?: string | null;
  notes?: string | null;
}

export interface UpdateGuardianInput {
  name: string;
  name_bn?: string | null;
  relationship: string;
  occupation?: string | null;
  monthly_income_poisha?: number | null;
  nid?: string | null;
  phone: string;
  phone2?: string | null;
  whatsapp_number?: string | null;
  email?: string | null;
  present_address?: string | null;
  permanent_address?: string | null;
  notes?: string | null;
}

export interface GuardianLinkInput {
  guardian_id?: string | null;
  is_primary: boolean;
  receives_sms: boolean;
  relationship?: string | null;
  new_guardian?: CreateGuardianInput | null;
}

export interface CreateStudentInput {
  student_code?: string | null;
  name: string;
  name_bn?: string | null;
  gender: string;
  dob: string;
  blood_group?: string | null;
  religion?: string | null;
  nid_or_birth_cert?: string | null;
  phone: string;
  emergency_phone?: string | null;
  email?: string | null;
  present_address: string;
  permanent_address?: string | null;
  district?: string | null;
  thana?: string | null;
  previous_school?: string | null;
  current_school_roll?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  photo_vault_id?: string | null;
  admission_date?: string | null;
  notes?: string | null;
  guardians: GuardianLinkInput[];
}

export interface UpdateStudentInput {
  name: string;
  name_bn?: string | null;
  gender: string;
  dob: string;
  blood_group?: string | null;
  religion?: string | null;
  nid_or_birth_cert?: string | null;
  phone: string;
  emergency_phone?: string | null;
  email?: string | null;
  present_address: string;
  permanent_address?: string | null;
  district?: string | null;
  thana?: string | null;
  previous_school?: string | null;
  current_school_roll?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  photo_vault_id?: string | null;
  admission_date?: string | null;
  status?: string | null;
  notes?: string | null;
  guardians?: GuardianLinkInput[] | null;
}

export interface GuardianDetail {
  id: string;
  name: string;
  name_bn?: string | null;
  relationship: string;
  occupation?: string | null;
  monthly_income_poisha?: number | null;
  nid?: string | null;
  phone_norm: string;
  phone2_norm?: string | null;
  whatsapp_number?: string | null;
  email?: string | null;
  present_address?: string | null;
  permanent_address?: string | null;
  notes?: string | null;
  is_primary: boolean;
  receives_sms: boolean;
  created_at: string;
  updated_at: string;
}

export interface StudentDetail {
  id: string;
  student_code: string;
  name: string;
  name_bn?: string | null;
  gender: string;
  dob: string;
  blood_group?: string | null;
  religion?: string | null;
  nid_or_birth_cert?: string | null;
  phone_norm: string;
  emergency_phone_norm?: string | null;
  email?: string | null;
  present_address: string;
  permanent_address?: string | null;
  district?: string | null;
  thana?: string | null;
  previous_school?: string | null;
  current_school_roll?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  photo_vault_id?: string | null;
  status: string;
  admission_date: string;
  notes?: string | null;
  row_version: number;
  created_at: string;
  updated_at: string;
  guardians: GuardianDetail[];
}

export interface StudentSummary {
  id: string;
  student_code: string;
  name: string;
  name_bn?: string | null;
  gender: string;
  class_level?: string | null;
  current_school_roll?: string | null;
  phone_norm: string;
  emergency_phone_norm?: string | null;
  status: string;
  photo_vault_id?: string | null;
  admission_date: string;
  row_version: number;
  guardian_names?: string | null;
  guardian_phones?: string | null;
}

export interface StudentSearchHit {
  id: string;
  student_code: string;
  name: string;
  name_bn?: string | null;
  gender: string;
  class_level?: string | null;
  current_school_roll?: string | null;
  phone_norm: string;
  status: string;
  photo_vault_id?: string | null;
  guardian_names?: string | null;
  guardian_phones?: string | null;
}

export interface GuardianSearchHit {
  id: string;
  name: string;
  name_bn?: string | null;
  relationship: string;
  phone_norm: string;
  email?: string | null;
  present_address?: string | null;
  student_count: number;
}

export interface DocumentSummary {
  id: string;
  owner_type: string;
  owner_id: string;
  kind: string;
  original_name: string;
  mime: string;
  size: number;
  sha256: string;
  created_at: string;
}

export interface CreateStudentPayload {
  student: CreateStudentInput;
  allow_duplicate_phone?: boolean | null;
}

export interface UpdateStudentPayload {
  id: string;
  expected_version: number;
  student: UpdateStudentInput;
  allow_duplicate_phone?: boolean | null;
}

export interface GetStudentPayload {
  id: string;
}

export interface ListStudentsPayload {
  status?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface ListStudentsResponse {
  items: StudentSummary[];
  total: number;
}

export interface SearchStudentsPayload {
  query: string;
  limit?: number | null;
}

export interface SearchStudentsResponse {
  matches: StudentSearchHit[];
}

export interface ArchiveStudentPayload {
  id: string;
  reason: string;
  expected_version: number;
}

export interface SearchGuardiansPayload {
  query: string;
  limit?: number | null;
}

export interface SearchGuardiansResponse {
  matches: GuardianSearchHit[];
}

export interface GetGuardianPayload {
  id: string;
}

export interface GetGuardianResponse {
  guardian: GuardianDetail;
  siblings: StudentSummary[];
}

export interface UpdateGuardianPayload {
  id: string;
  guardian: UpdateGuardianInput;
  split_from_siblings?: boolean | null;
  student_id?: string | null;
}

export interface GetStudentResponse {
  student: StudentDetail;
  siblings: StudentSummary[];
  documents: DocumentSummary[];
}

export interface UploadFilePayload {
  owner_type: string;
  owner_id: string;
  kind: string;
  file_name: string;
  mime_type: string;
  data_base64: string;
}

export interface GetFilePayload {
  id: string;
}

export interface GetFileResponse {
  data_base64: string;
  mime_type: string;
  file_name: string;
}

export interface ScheduleSlot {
  day_of_week: string;
  start_time: string;
  end_time: string;
  room?: string | null;
}

export interface ScheduleConflictItem {
  category: string;
  message: string;
  conflicting_batch_id: string;
  conflicting_batch_name: string;
  day_of_week: string;
  existing_time_slot: string;
  candidate_time_slot: string;
  resource_name: string;
}

export interface CreateProgramInput {
  session_id?: string | null;
  name: string;
  name_bn?: string | null;
  code?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  description?: string | null;
}

export interface UpdateProgramInput {
  name: string;
  name_bn?: string | null;
  code?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  description?: string | null;
  status?: string | null;
}

export interface ProgramSummary {
  id: string;
  session_id: string;
  session_name: string;
  name: string;
  name_bn?: string | null;
  code?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  status: string;
  batch_count: number;
  active_student_count: number;
}

export interface ProgramDetail {
  id: string;
  session_id: string;
  session_name: string;
  name: string;
  name_bn?: string | null;
  code?: string | null;
  class_level?: string | null;
  group_major?: string | null;
  description?: string | null;
  status: string;
  batch_count: number;
  active_student_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateProgramPayload {
  program: CreateProgramInput;
}

export interface UpdateProgramPayload {
  id: string;
  program: UpdateProgramInput;
}

export interface GetProgramPayload {
  id: string;
}

export interface ListProgramsPayload {
  session_id?: string | null;
  status?: string | null;
}

export interface CreateSubjectInput {
  name: string;
  name_bn?: string | null;
  code: string;
}

export interface SubjectItem {
  id: string;
  name: string;
  name_bn?: string | null;
  code: string;
  created_at: string;
  updated_at: string;
}

export interface BatchSubjectInput {
  subject_id: string;
  teacher_staff_id?: string | null;
}

export interface BatchSubjectItem {
  subject_id: string;
  subject_name: string;
  subject_code: string;
  teacher_staff_id?: string | null;
  teacher_name?: string | null;
}

export interface CreateSubjectPayload {
  subject: CreateSubjectInput;
}

export interface CreateBatchInput {
  program_id: string;
  name: string;
  code: string;
  teacher_staff_id?: string | null;
  room_number?: string | null;
  shift?: string | null;
  schedule: ScheduleSlot[];
  capacity?: number | null;
  start_date: string;
  end_date?: string | null;
}

export interface UpdateBatchInput {
  name: string;
  code: string;
  teacher_staff_id?: string | null;
  room_number?: string | null;
  shift?: string | null;
  schedule: ScheduleSlot[];
  capacity?: number | null;
  start_date: string;
  end_date?: string | null;
  status?: string | null;
}

export interface BatchSummary {
  id: string;
  program_id: string;
  program_name: string;
  name: string;
  code: string;
  teacher_staff_id?: string | null;
  teacher_name?: string | null;
  room_number?: string | null;
  shift: string;
  schedule: ScheduleSlot[];
  capacity: number;
  enrolled_count: number;
  available_seats: number;
  status: string;
  start_date: string;
  end_date?: string | null;
}

export interface BatchDetail {
  id: string;
  program_id: string;
  program_name: string;
  name: string;
  code: string;
  teacher_staff_id?: string | null;
  teacher_name?: string | null;
  room_number?: string | null;
  shift: string;
  schedule: ScheduleSlot[];
  capacity: number;
  enrolled_count: number;
  available_seats: number;
  status: string;
  start_date: string;
  end_date?: string | null;
  subjects: BatchSubjectItem[];
  created_at: string;
  updated_at: string;
}

export interface CreateBatchPayload {
  batch: CreateBatchInput;
  allow_conflict?: boolean | null;
}

export interface UpdateBatchPayload {
  id: string;
  batch: UpdateBatchInput;
  allow_conflict?: boolean | null;
}

export interface GetBatchPayload {
  id: string;
}

export interface ListBatchesPayload {
  program_id?: string | null;
  teacher_id?: string | null;
  status?: string | null;
}

export interface CheckConflictsPayload {
  batch_id?: string | null;
  teacher_staff_id?: string | null;
  room_number?: string | null;
  schedule: ScheduleSlot[];
}

export interface SetBatchSubjectsPayload {
  batch_id: string;
  subjects: BatchSubjectInput[];
}

export interface CreateStaffInput {
  name: string;
  name_bn?: string | null;
  phone: string;
  email?: string | null;
  designation: string;
  department?: string | null;
  educational_qualifications?: string | null;
  nid?: string | null;
  blood_group?: string | null;
  gender?: string | null;
  dob?: string | null;
  present_address?: string | null;
  joining_date: string;
  resignation_date?: string | null;
  salary_type?: string | null;
  notes?: string | null;
}

export interface UpdateStaffInput {
  name: string;
  name_bn?: string | null;
  phone: string;
  email?: string | null;
  designation: string;
  department?: string | null;
  educational_qualifications?: string | null;
  nid?: string | null;
  blood_group?: string | null;
  gender?: string | null;
  dob?: string | null;
  present_address?: string | null;
  joining_date: string;
  resignation_date?: string | null;
  salary_type?: string | null;
  status?: string | null;
  notes?: string | null;
}

export interface StaffSummary {
  id: string;
  name: string;
  name_bn?: string | null;
  phone_norm: string;
  email?: string | null;
  designation: string;
  department?: string | null;
  status: string;
  salary_type: string;
  joining_date: string;
  active_batch_count: number;
}

export interface StaffDetail {
  id: string;
  name: string;
  name_bn?: string | null;
  phone_norm: string;
  email?: string | null;
  designation: string;
  department?: string | null;
  educational_qualifications?: string | null;
  nid?: string | null;
  blood_group?: string | null;
  gender?: string | null;
  dob?: string | null;
  present_address?: string | null;
  joining_date: string;
  resignation_date?: string | null;
  salary_type: string;
  status: string;
  notes?: string | null;
  linked_user_id?: string | null;
  assigned_batches: BatchSummary[];
  created_at: string;
  updated_at: string;
}

export interface StaffSearchHit {
  id: string;
  name: string;
  name_bn?: string | null;
  phone_norm: string;
  designation: string;
  department?: string | null;
  status: string;
}

export interface CreateStaffPayload {
  staff: CreateStaffInput;
}

export interface UpdateStaffPayload {
  id: string;
  staff: UpdateStaffInput;
}

export interface GetStaffPayload {
  id: string;
}

export interface ListStaffPayload {
  status?: string | null;
  department?: string | null;
  limit?: number | null;
  offset?: number | null;
}

export interface SearchStaffPayload {
  query: string;
  limit?: number | null;
}

export interface CreateEnrollmentInput {
  student_id: string;
  batch_id: string;
  roll?: number | null;
  start_date?: string | null;
  notes?: string | null;
}

export interface TransferEnrollmentInput {
  student_id: string;
  from_batch_id: string;
  to_batch_id: string;
  new_roll?: number | null;
  transfer_date?: string | null;
  notes?: string | null;
}

export interface EndEnrollmentInput {
  enrollment_id: string;
  end_reason: string;
  end_date?: string | null;
  notes?: string | null;
}

export interface BulkEnrollInput {
  batch_id: string;
  student_ids: string[];
  start_date?: string | null;
  notes?: string | null;
}

export interface EnrollmentDetail {
  id: string;
  student_id: string;
  student_code: string;
  student_name: string;
  batch_id: string;
  batch_name: string;
  program_name: string;
  roll?: number | null;
  start_date: string;
  end_date?: string | null;
  status: string;
  end_reason?: string | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
}

export interface BatchRosterItem {
  enrollment_id: string;
  student_id: string;
  student_code: string;
  student_name: string;
  student_name_bn?: string | null;
  phone_norm: string;
  guardian_phone?: string | null;
  guardian_name?: string | null;
  roll?: number | null;
  start_date: string;
  status: string;
  admission_date: string;
}

export interface StudentEnrollmentHistoryItem {
  enrollment_id: string;
  batch_id: string;
  batch_name: string;
  batch_code: string;
  program_name: string;
  shift: string;
  room_number?: string | null;
  roll?: number | null;
  start_date: string;
  end_date?: string | null;
  status: string;
  end_reason?: string | null;
  notes?: string | null;
}

export interface CreateEnrollmentPayload {
  enrollment: CreateEnrollmentInput;
  allow_overcapacity?: boolean | null;
}

export interface TransferEnrollmentPayload {
  transfer: TransferEnrollmentInput;
  allow_overcapacity?: boolean | null;
}

export interface EndEnrollmentPayload {
  enrollment: EndEnrollmentInput;
}

export interface BulkEnrollPayload {
  bulk: BulkEnrollInput;
  allow_overcapacity?: boolean | null;
}

export interface ListBatchRosterPayload {
  batch_id: string;
  status?: string | null;
}

export interface ListStudentEnrollmentsPayload {
  student_id: string;
}

export interface InspectFilePayload {
  file_path: string;
}

export interface SheetInspection {
  name: string;
  row_count: number;
  sample_headers: string[];
}

export interface ColumnMappingSuggestion {
  source_column: string;
  target_field?: string | null;
  confidence: number;
  sample_values: string[];
}

export interface FileInspectionResult {
  file_name: string;
  file_size_bytes: number;
  format: string;
  sheets: SheetInspection[];
  suggested_mappings: ColumnMappingSuggestion[];
}

export interface ColumnMappingInput {
  source_column: string;
  target_field: string;
}

export type RowIssueSeverity = "warning" | "error";

export interface RowIssue {
  column?: string | null;
  field?: string | null;
  message: string;
  severity: RowIssueSeverity;
}

export interface ImportPreviewRow {
  row_index: number;
  raw_data: Record<string, string>;
  parsed_name?: string | null;
  parsed_phone?: string | null;
  parsed_guardian_name?: string | null;
  parsed_guardian_phone?: string | null;
  issues: RowIssue[];
  is_valid: boolean;
  has_warnings: boolean;
  has_errors: boolean;
}

export interface ValidationPreviewResult {
  total_rows: number;
  valid_count: number;
  warning_count: number;
  error_count: number;
  rows_sample: ImportPreviewRow[];
}

export interface PreviewValidationPayload {
  file_path: string;
  sheet_name?: string | null;
  mappings: ColumnMappingInput[];
  phone_conflict_mode?: string | null;
  default_present_address?: string | null;
}

export interface ExecuteImportInput {
  file_path: string;
  sheet_name?: string | null;
  mappings: ColumnMappingInput[];
  skip_error_rows: boolean;
  phone_conflict_mode: string;
  target_batch_id?: string | null;
  default_present_address?: string | null;
}

export interface ImportSummaryResult {
  total_rows: number;
  imported_students_count: number;
  imported_guardians_count: number;
  skipped_errors_count: number;
  backup_path: string;
  rejected_rows_csv_path?: string | null;
}

export interface RpcError {
  code: string;
  message: string;
  field?: string | null;
}

export interface RpcEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: RpcError;
}

export interface RpcOptions {
  sessionToken?: string | null;
  idempotencyKey?: string | null;
}

export type DemoProfile = "demo" | "load";

export interface GenerateDatasetPayload {
  profile: DemoProfile;
  seed?: number | null;
  out_filename?: string | null;
}

export interface GenerationSummary {
  profile: string;
  seed: number;
  student_count: number;
  guardian_count: number;
  batch_count: number;
  charge_count: number;
  payment_count: number;
  db_path: string;
  duration_ms: number;
}

export interface GeneratorStatusResponse {
  status: string;
  percent: number;
  stage: string;
  is_demo_active: boolean;
  active_database_path: string;
}

export interface SwitchDatabasePayload {
  target_filename: string;
}

export interface SwitchDatabaseResponse {
  success: boolean;
  active_path: string;
}

export interface SwitchToProductionResponse {
  success: boolean;
  production_path: string;
}

export interface FeeHeadItem {
  id: string;
  name: string;
  name_bn?: string | null;
  kind: string;
  created_at: string;
  updated_at: string;
}

export interface CreateFeeHeadInput {
  name: string;
  name_bn?: string | null;
  kind: string;
}

export interface CreateFeeHeadPayload {
  head: CreateFeeHeadInput;
}

export interface UpdateFeeHeadInput {
  name?: string | null;
  name_bn?: string | null;
  kind?: string | null;
}

export interface UpdateFeeHeadPayload {
  id: string;
  head: UpdateFeeHeadInput;
}

export interface DeleteFeeHeadPayload {
  id: string;
}

export interface FeePlanItem {
  id: string;
  program_id?: string | null;
  program_name?: string | null;
  batch_id?: string | null;
  batch_name?: string | null;
  fee_head_id: string;
  fee_head_name: string;
  fee_head_kind: string;
  amount_poisha: number;
  frequency: string;
  effective_from: string;
  effective_to?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateFeePlanInput {
  program_id?: string | null;
  batch_id?: string | null;
  fee_head_id: string;
  amount_poisha: number;
  frequency: string;
  effective_from: string;
  effective_to?: string | null;
}

export interface CreateFeePlanPayload {
  plan: CreateFeePlanInput;
}

export interface UpdateFeePlanInput {
  amount_poisha?: number | null;
  frequency?: string | null;
  effective_from?: string | null;
  effective_to?: string | null;
}

export interface UpdateFeePlanPayload {
  id: string;
  plan: UpdateFeePlanInput;
}

export interface DeleteFeePlanPayload {
  id: string;
}

export interface ListFeePlansPayload {
  program_id?: string | null;
  batch_id?: string | null;
}

export interface ConcessionItem {
  id: string;
  student_id: string;
  student_name: string;
  student_code: string;
  batch_id?: string | null;
  batch_name?: string | null;
  fee_head_id: string;
  fee_head_name: string;
  type: string;
  value: number;
  from_period?: string | null;
  to_period?: string | null;
  reason: string;
  approved_by?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateConcessionInput {
  student_id: string;
  batch_id?: string | null;
  fee_head_id: string;
  type: string;
  value: number;
  from_period?: string | null;
  to_period?: string | null;
  reason: string;
}

export interface CreateConcessionPayload {
  concession: CreateConcessionInput;
}

export interface DeleteConcessionPayload {
  id: string;
}

export interface ListConcessionsPayload {
  student_id?: string | null;
  batch_id?: string | null;
}

export interface GenerationResult {
  period: string;
  total_students_processed: number;
  generated_count: number;
  skipped_count: number;
  total_gross_poisha: number;
  total_concession_poisha: number;
  total_net_poisha: number;
  allocated_credit_poisha: number;
}

export type MonthlyGenerationResult = GenerationResult;

export interface GenerateMonthlyChargesPayload {
  period: string;
  batch_id?: string | null;
}

export interface CreateManualChargeInput {
  student_id: string;
  fee_head_id: string;
  amount_poisha: number;
  due_date: string;
  period?: string | null;
}

export interface CreateManualChargePayload {
  charge: CreateManualChargeInput;
}

export interface VoidChargePayload {
  charge_id: string;
  reason: string;
}

export interface ListStudentChargesPayload {
  student_id: string;
}

export interface ChargeItem {
  id: string;
  student_id: string;
  enrollment_id?: string | null;
  fee_head_id: string;
  fee_head_name: string;
  fee_head_kind: string;
  period?: string | null;
  gross_amount_poisha: number;
  concession_amount_poisha: number;
  net_amount_poisha: number;
  allocated_amount_poisha: number;
  waiver_amount_poisha: number;
  fine_amount_poisha: number;
  remaining_due_poisha: number;
  due_date: string;
  source: string;
  status: string;
  void_reason?: string | null;
  created_at: string;
}

export interface CreatePaymentInput {
  student_id: string;
  amount_poisha: number;
  method: string;
  reference?: string | null;
  business_date?: string | null;
  idempotency_key: string;
  target_charge_ids?: string[] | null;
  expected_balance_poisha?: number | null;
}

export interface CreatePaymentPayload {
  payment: CreatePaymentInput;
}

export interface VoidPaymentPayload {
  payment_id: string;
  reason: string;
}

export interface ListStudentPaymentsPayload {
  student_id: string;
}

export interface PaymentReceipt {
  id: string;
  receipt_no: string;
  student_id: string;
  student_name: string;
  amount_poisha: number;
  method: string;
  reference?: string | null;
  business_date: string;
  received_at: string;
  collector_user_id?: string | null;
  device_id?: string | null;
  idempotency_key: string;
  status: string;
  void_reason?: string | null;
  allocated_amount_poisha: number;
  unallocated_amount_poisha: number;
  created_at: string;
}

export interface CreateAdjustmentInput {
  student_id: string;
  charge_id?: string | null;
  type: string;
  amount_poisha: number;
  reason: string;
  business_date?: string | null;
}

export interface CreateAdjustmentPayload {
  adjustment: CreateAdjustmentInput;
}

export interface AdjustmentItem {
  id: string;
  student_id: string;
  charge_id?: string | null;
  type: string;
  amount_poisha: number;
  reason: string;
  approved_by?: string | null;
  business_date: string;
  created_at: string;
}

export interface CreateRefundInput {
  payment_id: string;
  amount_poisha: number;
  method: string;
  reason: string;
  business_date?: string | null;
}

export interface CreateRefundPayload {
  refund: CreateRefundInput;
}

export interface RefundItem {
  id: string;
  payment_id: string;
  receipt_no: string;
  amount_poisha: number;
  method: string;
  reason: string;
  business_date: string;
  refunded_by?: string | null;
  created_at: string;
}

export interface GetStudentLedgerSummaryPayload {
  student_id: string;
}

export interface StudentLedgerSummary {
  student_id: string;
  student_code: string;
  student_name: string;
  total_charged_poisha: number;
  total_paid_poisha: number;
  total_adjusted_poisha: number;
  total_refunded_poisha: number;
  balance_poisha: number;
  charges: ChargeItem[];
  payments: PaymentReceipt[];
  adjustments: AdjustmentItem[];
  refunds: RefundItem[];
}

export interface WorkstationPrinterSettings {
  printer_name?: string | null;
  page_size: "thermal_80mm" | "a4_dual" | "a5_single";
  paper_source?: string | null;
  copies: number;
  auto_print_on_payment: boolean;
  open_cash_drawer: boolean;
  header_title?: string | null;
  footer_note?: string | null;
  updated_at: string;
}

export interface SystemPrinterInfo {
  name: string;
  is_default: boolean;
  status: string;
}

export interface SavePrinterSettingsPayload {
  settings: WorkstationPrinterSettings;
}

export interface ReceiptAllocationItem {
  fee_head_name: string;
  fee_head_name_bn?: string | null;
  period?: string | null;
  amount_poisha: number;
}

export interface PrintableReceiptData {
  receipt_no: string;
  business_date: string;
  received_at: string;
  student_id: string;
  student_code: string;
  student_name: string;
  student_name_bn?: string | null;
  guardian_name?: string | null;
  guardian_phone?: string | null;
  batch_name?: string | null;
  program_name?: string | null;
  roll_no?: string | null;
  amount_poisha: number;
  amount_in_words_en: string;
  amount_in_words_bn: string;
  method: string;
  reference?: string | null;
  collector_name: string;
  institution_name: string;
  institution_name_bn?: string | null;
  institution_address?: string | null;
  institution_phone?: string | null;
  institution_email?: string | null;
  institution_website?: string | null;
  balance_after_poisha: number;
  items: ReceiptAllocationItem[];
  is_reprint: boolean;
}

export interface GetPrintableReceiptPayload {
  payment_id: string;
  is_reprint?: boolean | null;
}

export interface CashierStudentOverview {
  student_id: string;
  student_code: string;
  name: string;
  name_bn?: string | null;
  photo_vault_id?: string | null;
  phone: string;
  guardian_name?: string | null;
  guardian_phone?: string | null;
  batch_name?: string | null;
  program_name?: string | null;
  roll_no?: string | null;
  total_due_poisha: number;
  advance_credit_poisha: number;
  net_balance_poisha: number;
  open_charges: ChargeItem[];
  recent_payments: PaymentReceipt[];
}

export interface GetCashierStudentOverviewPayload {
  student_id: string;
}

export interface CommandMap {
  "system.ping": {
    request: PingPayload;
    response: PingResponse;
  };
  "system.get_status": {
    request: Record<string, never> | Record<string, unknown>;
    response: GetStatusResponse;
  };
  "setup.get_status": {
    request: Record<string, never> | Record<string, unknown>;
    response: SetupStatusResponse;
  };
  "setup.detect_drives": {
    request: Record<string, never> | Record<string, unknown>;
    response: DetectDrivesResponse;
  };
  "setup.validate_path": {
    request: ValidatePathPayload;
    response: ValidatePathResponse;
  };
  "setup.generate_recovery_preview": {
    request: Record<string, never> | Record<string, unknown>;
    response: GenerateRecoveryPreviewResponse;
  };
  "setup.initialize_institution": {
    request: InitializeInstitutionPayload;
    response: InitializeInstitutionResponse;
  };
  "recovery.inspect_database": {
    request: InspectDatabasePayload;
    response: InspectDatabaseResponse;
  };
  "recovery.unlock_with_key": {
    request: UnlockWithKeyPayload;
    response: UnlockWithKeyResponse;
  };
  "recovery.reprint_sheet": {
    request: ReprintSheetPayload;
    response: ReprintSheetResponse;
  };
  "system.bootstrap_owner": {
    request: BootstrapOwnerPayload;
    response: BootstrapOwnerResponse;
  };
  "auth.login": {
    request: LoginPayload;
    response: LoginResponse;
  };
  "auth.logout": {
    request: Record<string, never> | Record<string, unknown>;
    response: SuccessResponse;
  };
  "auth.unlock": {
    request: UnlockPayload;
    response: UnlockResponse;
  };
  "auth.change_password": {
    request: ChangePasswordPayload;
    response: SuccessResponse;
  };
  "auth.get_current_session": {
    request: Record<string, never> | Record<string, unknown>;
    response: CurrentSessionResponse;
  };
  "auth.heartbeat": {
    request: Record<string, never> | Record<string, unknown>;
    response: HeartbeatResponse;
  };
  "users.list": {
    request: ListUsersPayload;
    response: ListUsersResponse;
  };
  "users.create": {
    request: CreateUserPayload;
    response: UserSummary;
  };
  "users.update": {
    request: UpdateUserPayload;
    response: UserSummary;
  };
  "roles.list": {
    request: Record<string, never> | Record<string, unknown>;
    response: ListRolesResponse;
  };
  "roles.create": {
    request: CreateRolePayload;
    response: RoleWithPermissions;
  };
  "roles.update": {
    request: UpdateRolePayload;
    response: RoleWithPermissions;
  };
  "permissions.list": {
    request: Record<string, never> | Record<string, unknown>;
    response: ListPermissionsResponse;
  };
  "audit.list": {
    request: ListAuditLogsPayload;
    response: ListAuditLogsResponse;
  };
  "audit.verify_chain": {
    request: Record<string, never> | Record<string, unknown>;
    response: VerifyAuditChainResponse;
  };
  "audit.get_head": {
    request: Record<string, never> | Record<string, unknown>;
    response: GetAuditHeadResponse;
  };
  "system.migration_status": {
    request: Record<string, never> | Record<string, unknown>;
    response: MigrationStatusResponse;
  };
  "system.migration_history": {
    request: Record<string, never> | Record<string, unknown>;
    response: MigrationHistoryResponse;
  };
  "system.recovery_info": {
    request: Record<string, never> | Record<string, unknown>;
    response: RecoveryInfoResponse;
  };
  "system.open_db_directory": {
    request: Record<string, never> | Record<string, unknown>;
    response: OpenDbDirectoryResponse;
  };
  "system.retry_migration": {
    request: Record<string, never> | Record<string, unknown>;
    response: RecoveryInfoResponse;
  };
  "system.export_diagnostics": {
    request: Record<string, never> | Record<string, unknown>;
    response: ExportDiagnosticsResponse;
  };
  "students.create": {
    request: CreateStudentPayload;
    response: StudentDetail;
  };
  "students.update": {
    request: UpdateStudentPayload;
    response: StudentDetail;
  };
  "students.get": {
    request: GetStudentPayload;
    response: GetStudentResponse;
  };
  "students.list": {
    request: ListStudentsPayload;
    response: ListStudentsResponse;
  };
  "students.search": {
    request: SearchStudentsPayload;
    response: SearchStudentsResponse;
  };
  "students.archive": {
    request: ArchiveStudentPayload;
    response: StudentDetail;
  };
  "guardians.search": {
    request: SearchGuardiansPayload;
    response: SearchGuardiansResponse;
  };
  "guardians.get": {
    request: GetGuardianPayload;
    response: GetGuardianResponse;
  };
  "guardians.update": {
    request: UpdateGuardianPayload;
    response: GuardianDetail;
  };
  "vault.upload_file": {
    request: UploadFilePayload;
    response: DocumentSummary;
  };
  "vault.get_file": {
    request: GetFilePayload;
    response: GetFileResponse;
  };
  "programs.create": {
    request: CreateProgramPayload;
    response: ProgramDetail;
  };
  "programs.update": {
    request: UpdateProgramPayload;
    response: ProgramDetail;
  };
  "programs.get": {
    request: GetProgramPayload;
    response: ProgramDetail;
  };
  "programs.list": {
    request: ListProgramsPayload;
    response: ProgramSummary[];
  };
  "subjects.create": {
    request: CreateSubjectPayload;
    response: SubjectItem;
  };
  "subjects.list": {
    request: Record<string, never>;
    response: SubjectItem[];
  };
  "batches.create": {
    request: CreateBatchPayload;
    response: BatchDetail;
  };
  "batches.update": {
    request: UpdateBatchPayload;
    response: BatchDetail;
  };
  "batches.get": {
    request: GetBatchPayload;
    response: BatchDetail;
  };
  "batches.list": {
    request: ListBatchesPayload;
    response: BatchSummary[];
  };
  "batches.check_conflicts": {
    request: CheckConflictsPayload;
    response: ScheduleConflictItem[];
  };
  "batches.set_subjects": {
    request: SetBatchSubjectsPayload;
    response: SuccessResponse;
  };
  "staff.create": {
    request: CreateStaffPayload;
    response: StaffDetail;
  };
  "staff.update": {
    request: UpdateStaffPayload;
    response: StaffDetail;
  };
  "staff.get": {
    request: GetStaffPayload;
    response: StaffDetail;
  };
  "staff.list": {
    request: ListStaffPayload;
    response: StaffSummary[];
  };
  "staff.search": {
    request: SearchStaffPayload;
    response: StaffSearchHit[];
  };
  "enrollments.create": {
    request: CreateEnrollmentPayload;
    response: EnrollmentDetail;
  };
  "enrollments.transfer": {
    request: TransferEnrollmentPayload;
    response: EnrollmentDetail;
  };
  "enrollments.end": {
    request: EndEnrollmentPayload;
    response: EnrollmentDetail;
  };
  "enrollments.bulk_create": {
    request: BulkEnrollPayload;
    response: EnrollmentDetail[];
  };
  "enrollments.list_by_batch": {
    request: ListBatchRosterPayload;
    response: BatchRosterItem[];
  };
  "enrollments.list_by_student": {
    request: ListStudentEnrollmentsPayload;
    response: StudentEnrollmentHistoryItem[];
  };
  "import.inspect_file": {
    request: InspectFilePayload;
    response: FileInspectionResult;
  };
  "import.preview_validation": {
    request: PreviewValidationPayload;
    response: ValidationPreviewResult;
  };
  "import.execute": {
    request: ExecuteImportInput;
    response: ImportSummaryResult;
  };
  "generator.generate_dataset": {
    request: GenerateDatasetPayload;
    response: GenerationSummary;
  };
  "generator.get_status": {
    request: Record<string, never> | Record<string, unknown>;
    response: GeneratorStatusResponse;
  };
  "generator.switch_database": {
    request: SwitchDatabasePayload;
    response: SwitchDatabaseResponse;
  };
  "generator.switch_to_production_database": {
    request: Record<string, never> | Record<string, unknown>;
    response: SwitchToProductionResponse;
  };
  "fee_heads.list": {
    request: Record<string, never> | Record<string, unknown>;
    response: FeeHeadItem[];
  };
  "fee_heads.create": {
    request: CreateFeeHeadPayload;
    response: FeeHeadItem;
  };
  "fee_heads.update": {
    request: UpdateFeeHeadPayload;
    response: FeeHeadItem;
  };
  "fee_heads.delete": {
    request: DeleteFeeHeadPayload;
    response: SuccessResponse;
  };
  "fee_plans.list": {
    request: ListFeePlansPayload;
    response: FeePlanItem[];
  };
  "fee_plans.create": {
    request: CreateFeePlanPayload;
    response: FeePlanItem;
  };
  "fee_plans.update": {
    request: UpdateFeePlanPayload;
    response: FeePlanItem;
  };
  "fee_plans.delete": {
    request: DeleteFeePlanPayload;
    response: SuccessResponse;
  };
  "concessions.list": {
    request: ListConcessionsPayload;
    response: ConcessionItem[];
  };
  "concessions.create": {
    request: CreateConcessionPayload;
    response: ConcessionItem;
  };
  "concessions.delete": {
    request: DeleteConcessionPayload;
    response: SuccessResponse;
  };
  "charges.generate_monthly": {
    request: GenerateMonthlyChargesPayload;
    response: GenerationResult;
  };
  "charges.create_manual": {
    request: CreateManualChargePayload;
    response: ChargeItem;
  };
  "charges.void": {
    request: VoidChargePayload;
    response: SuccessResponse;
  };
  "charges.list_by_student": {
    request: ListStudentChargesPayload;
    response: ChargeItem[];
  };
  "payments.create": {
    request: CreatePaymentPayload;
    response: PaymentReceipt;
  };
  "payments.void": {
    request: VoidPaymentPayload;
    response: SuccessResponse;
  };
  "payments.list_by_student": {
    request: ListStudentPaymentsPayload;
    response: PaymentReceipt[];
  };
  "adjustments.create": {
    request: CreateAdjustmentPayload;
    response: AdjustmentItem;
  };
  "refunds.create": {
    request: CreateRefundPayload;
    response: RefundItem;
  };
  "ledger.get_student_summary": {
    request: GetStudentLedgerSummaryPayload;
    response: StudentLedgerSummary;
  };
  "printers.list": {
    request: Record<string, never> | Record<string, unknown>;
    response: SystemPrinterInfo[];
  };
  "printers.get_settings": {
    request: Record<string, never> | Record<string, unknown>;
    response: WorkstationPrinterSettings;
  };
  "printers.save_settings": {
    request: SavePrinterSettingsPayload;
    response: WorkstationPrinterSettings;
  };
  "receipts.get_printable": {
    request: GetPrintableReceiptPayload;
    response: PrintableReceiptData;
  };
  "ledger.get_cashier_student_overview": {
    request: GetCashierStudentOverviewPayload;
    response: CashierStudentOverview;
  };
}

export type CommandName = keyof CommandMap;

/**
 * Single typed RPC entry point to the Rust core.
 */
export async function rpc<C extends CommandName>(
  command: C,
  payload: CommandMap[C]["request"],
  options?: RpcOptions | string | null
): Promise<RpcEnvelope<CommandMap[C]["response"]>> {
  const isTauri =
    typeof window !== "undefined" &&
    Boolean(
      (window as unknown as { __TAURI_INTERNALS__?: { invoke?: unknown } })
        .__TAURI_INTERNALS__?.invoke
    );

  if (!isTauri) {
    return {
      ok: false,
      error: {
        code: "tauri_unavailable",
        message:
          "Tauri desktop bridge is not available. Please open Coachify using the native desktop app window rather than an external web browser.",
      },
    };
  }

  const sessionToken =
    typeof options === "object" && options !== null ? options.sessionToken : null;
  const idempotencyKey =
    typeof options === "string"
      ? options
      : typeof options === "object" && options !== null
        ? options.idempotencyKey
        : null;

  try {
    const raw = await invoke<string | RpcEnvelope<CommandMap[C]["response"]>>("rpc", {
      command,
      payload,
      sessionToken: sessionToken ?? null,
      idempotencyKey: idempotencyKey ?? null,
    });
    if (typeof raw === "string") {
      return JSON.parse(raw);
    }
    return raw;
  } catch (err: unknown) {
    return {
      ok: false,
      error: {
        code: "transport_error",
        message: err instanceof Error ? err.message : String(err),
      },
    };
  }
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_ping_roundtrip() {
        let payload = serde_json::to_string(&PingPayload {
            message: Some("test_ping".to_string()),
        })
        .unwrap();

        let response_str = handle_rpc("system.ping", &payload);
        let envelope: RpcEnvelope<PingResponse> = serde_json::from_str(&response_str).unwrap();

        assert!(envelope.ok);
        let data = envelope.data.expect("data should be present");
        assert_eq!(data.status, "ok");
        assert_eq!(data.database_status, "active");
        assert!(data.echo.unwrap().contains("test_ping"));
    }

    #[test]
    fn test_unknown_command() {
        let response_str = handle_rpc("nonexistent.command", "{}");
        let envelope: RpcEnvelope<()> = serde_json::from_str(&response_str).unwrap();
        assert!(!envelope.ok);
        let error = envelope.error.expect("error should be present");
        assert_eq!(error.code, "unknown_command");
    }
}
