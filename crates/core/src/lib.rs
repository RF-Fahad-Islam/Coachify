pub mod academic;
pub mod accounts;
pub mod ai;
pub mod attendance;
pub mod audit;
pub mod auth;
pub mod backup;
pub mod comms;
pub mod crypto;
pub mod db;
pub mod diagnostics;
pub mod dispatcher;
pub mod entitlement;
pub mod exam;
#[cfg(feature = "fault-injection")]
pub mod fault;
pub mod fees;
pub mod generator;
pub mod import;
pub mod integrity;
pub mod intelligence;
pub mod jobs;
pub mod lan;
pub mod ledger;
pub mod licensing;
pub mod migrations;
pub mod phone;
pub mod printers;
pub mod reports;
pub mod schedule;
pub mod seam;
pub mod search;
pub mod setup;
pub mod storage;
pub mod students;
pub mod vault;
pub mod words;

pub use academic::*;
pub use ai::*;
pub use attendance::*;
pub use comms::*;
pub use entitlement::*;
pub use exam::*;
pub use fees::*;
pub use generator::*;
pub use import::*;
pub use integrity::*;
pub use intelligence::*;
pub use lan::*;
pub use ledger::*;
pub use licensing::*;
pub use printers::*;
pub use reports::*;
pub use schedule::*;
pub use search::*;
pub use words::*;

pub use dispatcher::{
    AcknowledgeClockJumpPayload,
    BootstrapOwnerPayload,
    BootstrapOwnerResponse,
    BulkEnrollPayload,
    ChangeEvent,
    ChangePasswordPayload,
    CheckConflictsPayload,
    CreateAdjustmentPayload,
    CreateBatchPayload,
    CreateConcessionPayload,
    CreateEnrollmentPayload,
    CreateFeeHeadPayload,
    CreateFeePlanPayload,
    CreateManualChargePayload,
    CreatePaymentPayload,
    CreateProgramPayload,
    CreateRefundPayload,
    CreateRolePayload,
    // Backup & Diagnostics
    CreateSnapshotPayload,
    CreateStaffPayload,
    CreateSubjectPayload,
    CreateUserPayload,
    CurrentSessionResponse,
    DeleteConcessionPayload,
    DeleteFeeHeadPayload,
    DeleteFeePlanPayload,
    DiagnosticsBundleResponse,
    EndEnrollmentPayload,
    ExportDiagnosticsBundlePayload,
    GenerateMonthlyChargesPayload,
    GetBatchPayload,
    GetCashierStudentOverviewPayload,
    GetPrintableReceiptPayload,
    GetProgramPayload,
    GetStaffPayload,
    GetStatusResponse,
    GetStudentLedgerSummaryPayload,
    HeartbeatResponse,
    ListBackupHistoryPayload,
    ListBatchRosterPayload,
    ListBatchesPayload,
    ListConcessionsPayload,
    ListFeePlansPayload,
    ListPermissionsResponse,
    ListProgramsPayload,
    ListRolesResponse,
    ListStaffPayload,
    ListStudentChargesPayload,
    ListStudentEnrollmentsPayload,
    ListStudentPaymentsPayload,
    ListUsersPayload,
    ListUsersResponse,
    LoginPayload,
    LoginResponse,
    MigrationHistoryItem,
    MigrationHistoryResponse,
    MigrationStatusResponse,
    OpenDbDirectoryResponse,
    PendingMigrationInfo,
    PingPayload,
    PingResponse,
    RecoveryInfoResponse,
    RemoveDestinationPayload,
    RestoreExecutePayload,
    RestorePreviewPayload,
    RpcEnvelope,
    RpcError,
    SaveDestinationPayload,
    SavePrinterSettingsPayload,
    SaveSchedulePayload,
    SearchStaffPayload,
    SetBatchSubjectsPayload,
    SuccessResponse,
    SystemStatusResponse,
    TransferEnrollmentPayload,
    UnlockPayload,
    UnlockResponse,
    UpdateBatchPayload,
    UpdateFeeHeadPayload,
    UpdateFeePlanPayload,
    UpdateProgramPayload,
    UpdateRolePayload,
    UpdateStaffPayload,
    UpdateUserPayload,
    VerifyBackupPayload,
    VoidChargePayload,
    VoidPaymentPayload,
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
    dispatcher::handle_rpc_with_events(command, payload_json, session_token, idempotency_key)
}

/// Executes a typed command through the centralized command dispatcher without session or idempotency keys.
pub fn handle_rpc(command: &str, payload_json: &str) -> String {
    dispatcher::handle_rpc(command, payload_json)
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
