//! Command Seam: Centralized contract specification for Coachify UI <-> Core RPC.
//!
//! This module defines the typed command manifest and registers all domain
//! transfer objects with Specta for compile-time TypeScript type reflection.

use specta::Types;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PortalTokenPayload {
    pub session_token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CheckoutGetStatusPayload {
    pub order_number: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ClearedResponse {
    pub cleared: bool,
}

#[derive(Debug, Clone)]
pub struct CommandContractDescriptor {
    pub name: &'static str,
    pub request_type: &'static str,
    pub response_type: &'static str,
    pub is_empty_req: bool,
}

/// Registers all Command Seam types with Specta's type collector.
pub fn register_all_types(types: &mut Types) {
    // Auxiliary seam types
    types.register_mut::<PortalTokenPayload>();
    types.register_mut::<CheckoutGetStatusPayload>();
    types.register_mut::<ClearedResponse>();
    types.register_mut::<crate::dispatcher::RpcError>();
    types.register_mut::<crate::dispatcher::ChangeEvent>();

    // Core domain types
    types.register_mut::<crate::AcknowledgeClockJumpPayload>();
    types.register_mut::<crate::ActivateOnlineInput>();
    types.register_mut::<crate::ActivateOnlineResponse>();
    types.register_mut::<crate::AddStudentNotePayload>();
    types.register_mut::<crate::AdjustmentItem>();
    types.register_mut::<crate::AdmitCardBatchData>();
    types.register_mut::<crate::students::AdmitStudentResponse>();
    types.register_mut::<crate::dispatcher::AdmitStudentWithPaymentPayload>();
    types.register_mut::<crate::AiAuditLogItem>();
    types.register_mut::<crate::AiGatewayConfig>();
    types.register_mut::<crate::AiWalletBalance>();
    types.register_mut::<crate::AnalyzeTextPayload>();
    types.register_mut::<crate::dispatcher::ArchiveStudentPayload>();
    types.register_mut::<crate::dispatcher::AskCopilotPayload>();
    types.register_mut::<crate::AttendanceSessionInfo>();
    types.register_mut::<crate::AttendanceSummaryPayload>();
    types.register_mut::<crate::AttendanceSummaryResponse>();
    types.register_mut::<crate::AutomationRuleItem>();
    types.register_mut::<crate::backup::BackupDashboard>();
    types.register_mut::<crate::backup::BackupDestinationItem>();
    types.register_mut::<crate::backup::BackupHistoryItem>();
    types.register_mut::<crate::backup::BackupPreview>();
    types.register_mut::<crate::backup::BackupScheduleItem>();
    types.register_mut::<crate::BackupValidationResult>();
    types.register_mut::<crate::BatchAttendanceMatrixResponse>();
    types.register_mut::<crate::BatchCostConfig>();
    types.register_mut::<crate::BatchDetail>();
    types.register_mut::<crate::BatchHealthItem>();
    types.register_mut::<crate::BatchHealthPayload>();
    types.register_mut::<crate::BatchProfitabilityItem>();
    types.register_mut::<crate::BatchProfitabilityPayload>();
    types.register_mut::<crate::BatchRosterItem>();
    types.register_mut::<crate::BatchSummary>();
    types.register_mut::<crate::BatchTabulationSheetData>();
    types.register_mut::<crate::BlockWorkstationPayload>();
    types.register_mut::<crate::BootstrapOwnerPayload>();
    types.register_mut::<crate::BootstrapOwnerResponse>();
    types.register_mut::<crate::BulkEnrollPayload>();
    types.register_mut::<crate::dispatcher::BulkUpdateGuardianContactsPayload>();
    types.register_mut::<crate::BulkUpdateResult>();
    types.register_mut::<crate::CancelMessagePayload>();
    types.register_mut::<crate::CashDayItem>();
    types.register_mut::<crate::CashDaySummary>();
    types.register_mut::<crate::CashierStudentOverview>();
    types.register_mut::<crate::ChangePasswordPayload>();
    types.register_mut::<crate::ChargeItem>();
    types.register_mut::<crate::CheckConflictsPayload>();
    types.register_mut::<crate::dispatcher::CheckDuplicatePaymentPayload>();
    types.register_mut::<crate::dispatcher::ClearConversationPayload>();
    types.register_mut::<crate::ClientInfoResponse>();
    types.register_mut::<crate::ClientPairPayload>();
    types.register_mut::<crate::dispatcher::CloseCashDayPayload>();
    types.register_mut::<crate::CoachingDayTimelinePayload>();
    types.register_mut::<crate::CoachingDayTimelineResponse>();
    types.register_mut::<crate::CollectionSummaryPayload>();
    types.register_mut::<crate::CollectionSummaryResponse>();
    types.register_mut::<crate::CollectorSummaryPayload>();
    types.register_mut::<crate::CollectorSummaryResponse>();
    types.register_mut::<crate::ConcessionItem>();
    types.register_mut::<crate::ContactHealthReport>();
    types.register_mut::<crate::CopilotConversationDetails>();
    types.register_mut::<crate::CopilotResponse>();
    types.register_mut::<crate::CreateAdjustmentPayload>();
    types.register_mut::<crate::CreateBatchPayload>();
    types.register_mut::<crate::CreateConcessionPayload>();
    types.register_mut::<crate::CreateEnrollmentPayload>();
    types.register_mut::<crate::CreateExamPayload>();
    types.register_mut::<crate::dispatcher::CreateExpenseCategoryPayload>();
    types.register_mut::<crate::dispatcher::CreateExpensePayload>();
    types.register_mut::<crate::CreateFeeHeadPayload>();
    types.register_mut::<crate::CreateFeePlanPayload>();
    types.register_mut::<crate::CreateManualChargePayload>();
    types.register_mut::<crate::CreatePaymentPayload>();
    types.register_mut::<crate::CreateProgramPayload>();
    types.register_mut::<crate::CreateRefundPayload>();
    types.register_mut::<crate::CreateRolePayload>();
    types.register_mut::<crate::CreateSnapshotPayload>();
    types.register_mut::<crate::CreateStaffPayload>();
    types.register_mut::<crate::dispatcher::CreateStudentPayload>();
    types.register_mut::<crate::CreateSubjectPayload>();
    types.register_mut::<crate::CreateUserPayload>();
    types.register_mut::<crate::CreditTransactionItem>();
    types.register_mut::<crate::CurrentSessionResponse>();
    types.register_mut::<crate::DbIntegrityResult>();
    types.register_mut::<crate::DeleteConcessionPayload>();
    types.register_mut::<crate::DeleteExamPayload>();
    types.register_mut::<crate::DeleteFeeHeadPayload>();
    types.register_mut::<crate::DeleteFeePlanPayload>();
    types.register_mut::<crate::DeleteTemplatePayload>();
    types.register_mut::<crate::DetectDrivesResponse>();
    types.register_mut::<crate::DeviceEventItem>();
    types.register_mut::<crate::DiagnosticsBundleResponse>();
    types.register_mut::<crate::diagnostics::DiagnosticsInfo>();
    types.register_mut::<crate::diagnostics::IntegrityCheckResult>();
    types.register_mut::<crate::DiscoverHostsPayload>();
    types.register_mut::<crate::DiscoveredHostList>();
    types.register_mut::<crate::vault::DocumentSummary>();
    types.register_mut::<crate::DrillResult>();
    types.register_mut::<crate::DueContactLogItem>();
    types.register_mut::<crate::DueReminderCandidate>();
    types.register_mut::<crate::DueStatementData>();
    types.register_mut::<crate::DuesAgingPayload>();
    types.register_mut::<crate::DuesAgingResponse>();
    types.register_mut::<crate::DuplicateWarningResult>();
    types.register_mut::<crate::EndEnrollmentPayload>();
    types.register_mut::<crate::EnqueueDueRemindersPayload>();
    types.register_mut::<crate::EnqueueDueRemindersResponse>();
    types.register_mut::<crate::EnrollmentDetail>();
    types.register_mut::<crate::ExamDetails>();
    types.register_mut::<crate::ExamPerformancePayload>();
    types.register_mut::<crate::ExamPerformanceResponse>();
    types.register_mut::<crate::ExamResultPreviewData>();
    types.register_mut::<crate::ExamSummary>();
    types.register_mut::<crate::ExecuteImportInput>();
    types.register_mut::<crate::ExpenseCategoryItem>();
    types.register_mut::<crate::ExpenseItem>();
    types.register_mut::<crate::ExpensesSummaryPayload>();
    types.register_mut::<crate::ExpensesSummaryResponse>();
    types.register_mut::<crate::ExportCsvPayload>();
    types.register_mut::<crate::ExportCsvResponse>();
    types.register_mut::<crate::ExportDiagnosticsBundlePayload>();
    types.register_mut::<crate::migrations::ExportDiagnosticsResponse>();
    types.register_mut::<crate::dispatcher::ExportMigrationBundlePayload>();
    types.register_mut::<crate::FeeHeadItem>();
    types.register_mut::<crate::FeePlanItem>();
    types.register_mut::<crate::FileInspectionResult>();
    types.register_mut::<crate::FirewallGuidance>();
    types.register_mut::<crate::FlushQueuePayload>();
    types.register_mut::<crate::FlushQueueResponse>();
    types.register_mut::<crate::FlushResult>();
    types.register_mut::<crate::GatewayTestResult>();
    types.register_mut::<crate::GenerateDatasetPayload>();
    types.register_mut::<crate::GenerateMonthlyChargesPayload>();
    types.register_mut::<crate::GenerateRecoveryPreviewResponse>();
    types.register_mut::<crate::GenerationResult>();
    types.register_mut::<crate::GenerationSummary>();
    types.register_mut::<crate::GeneratorStatusResponse>();
    types.register_mut::<crate::dispatcher::GetAdmitCardDataPayload>();
    types.register_mut::<crate::dispatcher::GetAiAuditLogPayload>();
    types.register_mut::<crate::GetAttendanceSessionPayload>();
    types.register_mut::<crate::GetAuditHeadResponse>();
    types.register_mut::<crate::GetBatchAttendanceMatrixPayload>();
    types.register_mut::<crate::dispatcher::GetBatchDueStatementsPayload>();
    types.register_mut::<crate::GetBatchPayload>();
    types.register_mut::<crate::dispatcher::GetBatchReportCardsPayload>();
    types.register_mut::<crate::GetBatchTabulationSheetPayload>();
    types.register_mut::<crate::dispatcher::GetCashDaySummaryPayload>();
    types.register_mut::<crate::GetCashierStudentOverviewPayload>();
    types.register_mut::<crate::dispatcher::GetConversationPayload>();
    types.register_mut::<crate::dispatcher::GetCreditLedgerPayload>();
    types.register_mut::<crate::GetDueReminderCandidatesQuery>();
    types.register_mut::<crate::dispatcher::GetDueStatementPayload>();
    types.register_mut::<crate::GetExamPayload>();
    types.register_mut::<crate::dispatcher::GetFilePayload>();
    types.register_mut::<crate::vault::GetFileResponse>();
    types.register_mut::<crate::dispatcher::GetGuardianContactHealthPayload>();
    types.register_mut::<crate::dispatcher::GetGuardianPayload>();
    types.register_mut::<crate::students::GetGuardianResponse>();
    types.register_mut::<crate::dispatcher::GetIdCardDataPayload>();
    types.register_mut::<crate::GetMarkSheetPayload>();
    types.register_mut::<crate::dispatcher::GetOrOpenTodayCashDayPayload>();
    types.register_mut::<crate::GetOutboxDetailPayload>();
    types.register_mut::<crate::dispatcher::GetOutboxLifecyclePayload>();
    types.register_mut::<crate::GetPrintableReceiptPayload>();
    types.register_mut::<crate::GetProgramPayload>();
    types.register_mut::<crate::GetStaffPayload>();
    types.register_mut::<crate::GetStatusResponse>();
    types.register_mut::<crate::GetStudentAttendanceSummaryPayload>();
    types.register_mut::<crate::GetStudentLedgerSummaryPayload>();
    types.register_mut::<crate::dispatcher::GetStudentPayload>();
    types.register_mut::<crate::GetStudentReportCardPayload>();
    types.register_mut::<crate::students::GetStudentResponse>();
    types.register_mut::<crate::GetWorkstationPayload>();
    types.register_mut::<crate::GlobalSearchPayload>();
    types.register_mut::<crate::GlobalSearchResponse>();
    types.register_mut::<crate::GradingSchemeSummary>();
    types.register_mut::<crate::students::GuardianDetail>();
    types.register_mut::<crate::backup::HealthCheckResult>();
    types.register_mut::<crate::HeartbeatResponse>();
    types.register_mut::<crate::HostPingResult>();
    types.register_mut::<crate::ImportLicenseInput>();
    types.register_mut::<crate::dispatcher::ImportMigrationBundlePayload>();
    types.register_mut::<crate::ImportSummaryResult>();
    types.register_mut::<crate::InitializeInstitutionPayload>();
    types.register_mut::<crate::InitializeInstitutionResponse>();
    types.register_mut::<crate::InspectDatabasePayload>();
    types.register_mut::<crate::InspectDatabaseResponse>();
    types.register_mut::<crate::InspectFilePayload>();
    types.register_mut::<crate::LanHostStatus>();
    types.register_mut::<crate::LanUpdateMirrorConfig>();
    types.register_mut::<crate::LanStagedUpdate>();
    types.register_mut::<crate::LanUpdateMirrorStatus>();
    types.register_mut::<crate::HostUpdateManifest>();
    types.register_mut::<crate::StageMirrorUpdateInput>();
    types.register_mut::<crate::ConfigureMirrorInput>();
    types.register_mut::<crate::ClearMirrorUpdateInput>();
    types.register_mut::<crate::WorkstationUpdateCheckResult>();
    types.register_mut::<crate::WorkstationUpdateDownloadResult>();
    types.register_mut::<crate::DownloadWorkstationUpdatePayload>();
    types.register_mut::<crate::LicenseStatusResponse>();
    types.register_mut::<crate::ListAuditLogsPayload>();
    types.register_mut::<crate::ListAuditLogsResponse>();
    types.register_mut::<crate::ListBackupHistoryPayload>();
    types.register_mut::<crate::ListBatchRosterPayload>();
    types.register_mut::<crate::ListBatchesPayload>();
    types.register_mut::<crate::ListConcessionsPayload>();
    types.register_mut::<crate::ListDeviceEventsPayload>();
    types.register_mut::<crate::dispatcher::ListDuesPayload>();
    types.register_mut::<crate::dispatcher::ListExpensesPayload>();
    types.register_mut::<crate::ListFeePlansPayload>();
    types.register_mut::<crate::ListOutboxQuery>();
    types.register_mut::<crate::ListOutboxResponse>();
    types.register_mut::<crate::ListPermissionsResponse>();
    types.register_mut::<crate::dispatcher::ListPrintJobsPayload>();
    types.register_mut::<crate::ListProgramsPayload>();
    types.register_mut::<crate::ListRolesResponse>();
    types.register_mut::<crate::ListStaffPayload>();
    types.register_mut::<crate::ListStudentChargesPayload>();
    types.register_mut::<crate::ListStudentEnrollmentsPayload>();
    types.register_mut::<crate::ListStudentPaymentsPayload>();
    types.register_mut::<crate::students::ListStudentsPayload>();
    types.register_mut::<crate::students::ListStudentsResponse>();
    types.register_mut::<crate::ListTemplatesPayload>();
    types.register_mut::<crate::ListUsersPayload>();
    types.register_mut::<crate::ListUsersResponse>();
    types.register_mut::<crate::LockAttendanceSessionPayload>();
    types.register_mut::<crate::LoginPayload>();
    types.register_mut::<crate::LoginResponse>();
    types.register_mut::<crate::MarkSheetData>();
    types.register_mut::<crate::MessageTemplateItem>();
    types.register_mut::<crate::MigrationExportResult>();
    types.register_mut::<crate::MigrationHistoryResponse>();
    types.register_mut::<crate::MigrationImportResult>();
    types.register_mut::<crate::MigrationStatusResponse>();
    types.register_mut::<crate::MobilePairingWindowInfo>();
    types.register_mut::<crate::OpenDbDirectoryResponse>();
    types.register_mut::<crate::OpenPairingWindowPayload>();
    types.register_mut::<crate::OutboxDetailResponse>();
    types.register_mut::<crate::OutboxLifecycleSummary>();
    types.register_mut::<crate::OutboxStatsResponse>();
    types.register_mut::<crate::PairSuccessResponse>();
    types.register_mut::<crate::PairingWindowInfo>();
    types.register_mut::<crate::PathValidationResult>();
    types.register_mut::<crate::PaymentReceipt>();
    types.register_mut::<crate::PingHostPayload>();
    types.register_mut::<crate::PingPayload>();
    types.register_mut::<crate::PingResponse>();
    types.register_mut::<crate::PreviewBatchBroadcastPayload>();
    types.register_mut::<crate::PreviewBatchBroadcastResponse>();
    types.register_mut::<crate::PreviewResultsPayload>();
    types.register_mut::<crate::PreviewValidationPayload>();
    types.register_mut::<crate::PrintJobListResult>();
    types.register_mut::<crate::PrintJobRun>();
    types.register_mut::<crate::PrintTemplateConfig>();
    types.register_mut::<crate::PrintableReceiptData>();
    types.register_mut::<crate::PrivacySettings>();
    types.register_mut::<crate::ProgramDetail>();
    types.register_mut::<crate::ProgramSummary>();
    types.register_mut::<crate::ProviderConfigItem>();
    types.register_mut::<crate::PublishExamPayload>();
    types.register_mut::<crate::PublishExamResponse>();
    types.register_mut::<crate::ReceiptVerificationResult>();
    types.register_mut::<crate::ReconnectWorkstationResponse>();
    types.register_mut::<crate::dispatcher::RecordCreditGrantPayload>();
    types.register_mut::<crate::RecordDueContactPayload>();
    types.register_mut::<crate::dispatcher::RecordPrintJobPayload>();
    types.register_mut::<crate::RecoveryInfoResponse>();
    types.register_mut::<crate::RecoveryScoreResult>();
    types.register_mut::<crate::RefreshLicenseNowInput>();
    types.register_mut::<crate::RefreshLicenseNowResponse>();
    types.register_mut::<crate::RefreshStateResponse>();
    types.register_mut::<crate::RefundItem>();
    types.register_mut::<crate::RemoveDestinationPayload>();
    types.register_mut::<crate::RenameWorkstationPayload>();
    types.register_mut::<crate::ReprintSheetPayload>();
    types.register_mut::<crate::ReprintSheetResponse>();
    types.register_mut::<crate::RequestCodeResponse>();
    types.register_mut::<crate::ResendMessagePayload>();
    types.register_mut::<crate::ResetTransferCountInput>();
    types.register_mut::<crate::ResetTransferCountResponse>();
    types.register_mut::<crate::ResilienceStatus>();
    types.register_mut::<crate::RestoreExecutePayload>();
    types.register_mut::<crate::RestorePreviewPayload>();
    types.register_mut::<crate::backup::RestoreResult>();
    types.register_mut::<crate::dispatcher::RetryOutboxMessagesPayload>();
    types.register_mut::<crate::RetryResult>();
    types.register_mut::<crate::RevertPublishedPayload>();
    types.register_mut::<crate::RevokeWorkstationPayload>();
    types.register_mut::<crate::RoleWithPermissions>();
    types.register_mut::<crate::dispatcher::RunDatabaseIntegrityChecksPayload>();
    types.register_mut::<crate::dispatcher::RunVerificationDrillPayload>();
    types.register_mut::<crate::SaveAttendanceSessionPayload>();
    types.register_mut::<crate::SaveDestinationPayload>();
    types.register_mut::<crate::SaveMarksPayload>();
    types.register_mut::<crate::SaveMessageTemplatePayload>();
    types.register_mut::<crate::SavePrinterSettingsPayload>();
    types.register_mut::<crate::SaveProviderConfigPayload>();
    types.register_mut::<crate::SaveSchedulePayload>();
    types.register_mut::<crate::ScheduleConflictItem>();
    types.register_mut::<crate::dispatcher::SearchGuardiansPayload>();
    types.register_mut::<crate::students::SearchGuardiansResponse>();
    types.register_mut::<crate::SearchStaffPayload>();
    types.register_mut::<crate::dispatcher::SearchStudentsPayload>();
    types.register_mut::<crate::students::SearchStudentsResponse>();
    types.register_mut::<crate::crypto::SecretsHealthReport>();
    types.register_mut::<crate::SendBatchBroadcastPayload>();
    types.register_mut::<crate::SendBatchBroadcastResponse>();
    types.register_mut::<crate::SendTestMessagePayload>();
    types.register_mut::<crate::SendTestMessageResponse>();
    types.register_mut::<crate::SetBatchSubjectsPayload>();
    types.register_mut::<crate::SetStaffPinPayload>();
    types.register_mut::<crate::SetupStatusResponse>();
    types.register_mut::<crate::SmartDueQueuePayload>();
    types.register_mut::<crate::SmartDueQueueResponse>();
    types.register_mut::<crate::StaffDetail>();
    types.register_mut::<crate::StaffSearchHit>();
    types.register_mut::<crate::StaffSummary>();
    types.register_mut::<crate::StartHostPayload>();
    types.register_mut::<crate::Student360Payload>();
    types.register_mut::<crate::Student360Response>();
    types.register_mut::<crate::lan::StaffLoginResponse>();
    types.register_mut::<crate::StudentAttendanceSummary>();
    types.register_mut::<crate::students::StudentDetail>();
    types.register_mut::<crate::StudentDueSummaryItem>();
    types.register_mut::<crate::StudentEnrollmentHistoryItem>();
    types.register_mut::<crate::StudentIdCardData>();
    types.register_mut::<crate::StudentLedgerSummary>();
    types.register_mut::<crate::StudentReportCardData>();
    types.register_mut::<crate::StudentTimelineNote>();
    types.register_mut::<crate::SubjectItem>();
    types.register_mut::<crate::SuccessResponse>();
    types.register_mut::<crate::SwitchDatabasePayload>();
    types.register_mut::<crate::SwitchDatabaseResponse>();
    types.register_mut::<crate::SwitchToProductionResponse>();
    types.register_mut::<crate::SystemPrinterInfo>();
    types.register_mut::<crate::SystemStatusResponse>();
    types.register_mut::<crate::TestConnectionResponse>();
    types.register_mut::<crate::TextAnalysisResponse>();
    types.register_mut::<crate::TodayCockpitPayload>();
    types.register_mut::<crate::TodayCockpitResponse>();
    types.register_mut::<crate::TransferEnrollmentPayload>();
    types.register_mut::<crate::UnblockWorkstationPayload>();
    types.register_mut::<crate::UnlockPayload>();
    types.register_mut::<crate::UnlockResponse>();
    types.register_mut::<crate::UnlockWithKeyPayload>();
    types.register_mut::<crate::UnlockWithKeyResponse>();
    types.register_mut::<crate::UnverifySubjectMarksPayload>();
    types.register_mut::<crate::UpdateAutomationRulePayload>();
    types.register_mut::<crate::UpdateBatchCostConfigPayload>();
    types.register_mut::<crate::UpdateBatchPayload>();
    types.register_mut::<crate::UpdateExamPayload>();
    types.register_mut::<crate::UpdateFeeHeadPayload>();
    types.register_mut::<crate::UpdateFeePlanPayload>();
    types.register_mut::<crate::dispatcher::UpdateGatewayConfigPayload>();
    types.register_mut::<crate::dispatcher::UpdateGuardianPayload>();
    types.register_mut::<crate::UpdateHostConfigPayload>();
    types.register_mut::<crate::UpdateHostReceiptPrinterPayload>();
    types.register_mut::<crate::dispatcher::UpdatePrintTemplateConfigPayload>();
    types.register_mut::<crate::dispatcher::UpdatePrivacySettingsPayload>();
    types.register_mut::<crate::UpdateProgramPayload>();
    types.register_mut::<crate::UpdateRolePayload>();
    types.register_mut::<crate::UpdateStaffPayload>();
    types.register_mut::<crate::dispatcher::UpdateStudentPayload>();
    types.register_mut::<crate::UpdateUserPayload>();
    types.register_mut::<crate::vault::UploadFilePayload>();
    types.register_mut::<crate::UserSummary>();
    types.register_mut::<crate::ValidatePathPayload>();
    types.register_mut::<crate::ValidationPreviewResult>();
    types.register_mut::<crate::backup::VerificationResult>();
    types.register_mut::<crate::VerifyAuditChainResponse>();
    types.register_mut::<crate::VerifyBackupPayload>();
    types.register_mut::<crate::dispatcher::VerifyReceiptPayload>();
    types.register_mut::<crate::VerifySubjectMarksPayload>();
    types.register_mut::<crate::VoidChargePayload>();
    types.register_mut::<crate::dispatcher::VoidExpensePayload>();
    types.register_mut::<crate::VoidPaymentPayload>();
    types.register_mut::<crate::WhatsAppQrResponse>();
    types.register_mut::<crate::WhatsAppStatusResponse>();
    types.register_mut::<crate::WorkstationClientStatus>();
    types.register_mut::<crate::WorkstationCredentials>();
    types.register_mut::<crate::WorkstationDetail>();
    types.register_mut::<crate::WorkstationListResponse>();
    types.register_mut::<crate::WorkstationPrinterSettings>();
    types.register_mut::<crate::WorkstationSummary>();

    // Cloud API domain types
    types.register_mut::<coachify_cloud_api::AdminLoginInput>();
    types.register_mut::<coachify_cloud_api::AdminLoginResponse>();
    types.register_mut::<coachify_cloud_api::AdminResetTransferCountInput>();
    types.register_mut::<coachify_cloud_api::AdminUpdatePriceTierInput>();
    types.register_mut::<coachify_cloud_api::ApproveBankTransferInput>();
    types.register_mut::<coachify_cloud_api::CheckoutOrderStatusView>();
    types.register_mut::<coachify_cloud_api::CheckoutSessionResponse>();
    types.register_mut::<coachify_cloud_api::CreateCheckoutSessionInput>();
    types.register_mut::<coachify_cloud_api::CreateVendorCustomerInput>();
    types.register_mut::<coachify_cloud_api::DeviceActionResult>();
    types.register_mut::<coachify_cloud_api::IssueOfflineLicenseInput>();
    types.register_mut::<coachify_cloud_api::IssueOfflineLicenseResult>();
    types.register_mut::<coachify_cloud_api::KeyRecoveryDrillRow>();
    types.register_mut::<coachify_cloud_api::PaymentWebhookInput>();
    types.register_mut::<coachify_cloud_api::PendingBankTransferOrderView>();
    types.register_mut::<coachify_cloud_api::PortalDeactivateDeviceInput>();
    types.register_mut::<coachify_cloud_api::PortalDeviceItem>();
    types.register_mut::<coachify_cloud_api::PortalDeviceListView>();
    types.register_mut::<coachify_cloud_api::PortalDeviceOtpResponse>();
    types.register_mut::<coachify_cloud_api::PortalLoginInput>();
    types.register_mut::<coachify_cloud_api::PortalLoginResponse>();
    types.register_mut::<coachify_cloud_api::PortalOverviewResponse>();
    types.register_mut::<coachify_cloud_api::PortalRenameDeviceInput>();
    types.register_mut::<coachify_cloud_api::PortalReplaceDeviceInput>();
    types.register_mut::<coachify_cloud_api::PortalRequestDeviceOtpInput>();
    types.register_mut::<coachify_cloud_api::PortalRequestTransferResetInput>();
    types.register_mut::<coachify_cloud_api::PortalVerifyOtpInput>();
    types.register_mut::<coachify_cloud_api::PortalVerifyOtpResponse>();
    types.register_mut::<coachify_cloud_api::PricingCatalogResponse>();
    types.register_mut::<coachify_cloud_api::PromoteReleaseChannelInput>();
    types.register_mut::<coachify_cloud_api::PublishReleaseInput>();
    types.register_mut::<coachify_cloud_api::RecordKeyRecoveryDrillInput>();
    types.register_mut::<coachify_cloud_api::RecordManualPaymentInput>();
    types.register_mut::<coachify_cloud_api::RecordManualPaymentResult>();
    types.register_mut::<coachify_cloud_api::ReleaseChannelHeadRow>();
    types.register_mut::<coachify_cloud_api::RevokeLicenseInput>();
    types.register_mut::<coachify_cloud_api::SubmitBankTransferInput>();
    types.register_mut::<coachify_cloud_api::UpdateReleaseStatusInput>();
    types.register_mut::<coachify_cloud_api::UpdatedPriceTierResult>();
    types.register_mut::<coachify_cloud_api::VendorCustomerRow>();
    types.register_mut::<coachify_cloud_api::VendorDashboardOverview>();
    types.register_mut::<coachify_cloud_api::VendorReleaseRow>();
    types.register_mut::<coachify_cloud_api::VendorRevocationRow>();
    types.register_mut::<coachify_cloud_api::WebhookAckResponse>();
}

/// Returns the complete list of Command Seam command descriptors.
pub fn command_descriptors() -> Vec<CommandContractDescriptor> {
    vec![
        CommandContractDescriptor {
            name: "system.ping",
            request_type: "PingPayload",
            response_type: "PingResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "system.get_status",
            request_type: "EmptyPayload",
            response_type: "GetStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.get_system_status",
            request_type: "EmptyPayload",
            response_type: "SystemStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.acknowledge_clock_jump",
            request_type: "AcknowledgeClockJumpPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "setup.get_status",
            request_type: "EmptyPayload",
            response_type: "SetupStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "setup.detect_drives",
            request_type: "EmptyPayload",
            response_type: "DetectDrivesResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "setup.validate_path",
            request_type: "ValidatePathPayload",
            response_type: "ValidatePathResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "setup.generate_recovery_preview",
            request_type: "EmptyPayload",
            response_type: "GenerateRecoveryPreviewResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "setup.initialize_institution",
            request_type: "InitializeInstitutionPayload",
            response_type: "InitializeInstitutionResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "recovery.inspect_database",
            request_type: "InspectDatabasePayload",
            response_type: "InspectDatabaseResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "recovery.unlock_with_key",
            request_type: "UnlockWithKeyPayload",
            response_type: "UnlockWithKeyResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "recovery.reprint_sheet",
            request_type: "ReprintSheetPayload",
            response_type: "ReprintSheetResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "system.bootstrap_owner",
            request_type: "BootstrapOwnerPayload",
            response_type: "BootstrapOwnerResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "auth.login",
            request_type: "LoginPayload",
            response_type: "LoginResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "auth.logout",
            request_type: "EmptyPayload",
            response_type: "SuccessResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "auth.unlock",
            request_type: "UnlockPayload",
            response_type: "UnlockResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "auth.change_password",
            request_type: "ChangePasswordPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "auth.get_current_session",
            request_type: "EmptyPayload",
            response_type: "CurrentSessionResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "auth.heartbeat",
            request_type: "EmptyPayload",
            response_type: "HeartbeatResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "users.list",
            request_type: "ListUsersPayload",
            response_type: "ListUsersResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "users.create",
            request_type: "CreateUserPayload",
            response_type: "UserSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "users.update",
            request_type: "UpdateUserPayload",
            response_type: "UserSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "roles.list",
            request_type: "EmptyPayload",
            response_type: "ListRolesResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "roles.create",
            request_type: "CreateRolePayload",
            response_type: "RoleWithPermissions",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "roles.update",
            request_type: "UpdateRolePayload",
            response_type: "RoleWithPermissions",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "permissions.list",
            request_type: "EmptyPayload",
            response_type: "ListPermissionsResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "audit.list",
            request_type: "ListAuditLogsPayload",
            response_type: "ListAuditLogsResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "audit.verify_chain",
            request_type: "EmptyPayload",
            response_type: "VerifyAuditChainResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "audit.get_head",
            request_type: "EmptyPayload",
            response_type: "GetAuditHeadResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.migration_status",
            request_type: "EmptyPayload",
            response_type: "MigrationStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.migration_history",
            request_type: "EmptyPayload",
            response_type: "MigrationHistoryResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.recovery_info",
            request_type: "EmptyPayload",
            response_type: "RecoveryInfoResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.open_db_directory",
            request_type: "EmptyPayload",
            response_type: "OpenDbDirectoryResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.retry_migration",
            request_type: "EmptyPayload",
            response_type: "RecoveryInfoResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "system.export_diagnostics",
            request_type: "EmptyPayload",
            response_type: "ExportDiagnosticsResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "students.create",
            request_type: "CreateStudentPayload",
            response_type: "StudentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.update",
            request_type: "UpdateStudentPayload",
            response_type: "StudentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.get",
            request_type: "GetStudentPayload",
            response_type: "GetStudentResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.list",
            request_type: "ListStudentsPayload",
            response_type: "ListStudentsResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.search",
            request_type: "SearchStudentsPayload",
            response_type: "SearchStudentsResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.archive",
            request_type: "ArchiveStudentPayload",
            response_type: "StudentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "guardians.search",
            request_type: "SearchGuardiansPayload",
            response_type: "SearchGuardiansResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "guardians.get",
            request_type: "GetGuardianPayload",
            response_type: "GetGuardianResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "guardians.update",
            request_type: "UpdateGuardianPayload",
            response_type: "GuardianDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vault.upload_file",
            request_type: "UploadFilePayload",
            response_type: "DocumentSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vault.get_file",
            request_type: "GetFilePayload",
            response_type: "GetFileResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "programs.create",
            request_type: "CreateProgramPayload",
            response_type: "ProgramDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "programs.update",
            request_type: "UpdateProgramPayload",
            response_type: "ProgramDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "programs.get",
            request_type: "GetProgramPayload",
            response_type: "ProgramDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "programs.list",
            request_type: "ListProgramsPayload",
            response_type: "ProgramSummary[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "subjects.create",
            request_type: "CreateSubjectPayload",
            response_type: "SubjectItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "subjects.list",
            request_type: "Record<string, never>",
            response_type: "SubjectItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.create",
            request_type: "CreateBatchPayload",
            response_type: "BatchDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.update",
            request_type: "UpdateBatchPayload",
            response_type: "BatchDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.get",
            request_type: "GetBatchPayload",
            response_type: "BatchDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.list",
            request_type: "ListBatchesPayload",
            response_type: "BatchSummary[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.check_conflicts",
            request_type: "CheckConflictsPayload",
            response_type: "ScheduleConflictItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "batches.set_subjects",
            request_type: "SetBatchSubjectsPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "staff.create",
            request_type: "CreateStaffPayload",
            response_type: "StaffDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "staff.update",
            request_type: "UpdateStaffPayload",
            response_type: "StaffDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "staff.get",
            request_type: "GetStaffPayload",
            response_type: "StaffDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "staff.list",
            request_type: "ListStaffPayload",
            response_type: "StaffSummary[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "staff.search",
            request_type: "SearchStaffPayload",
            response_type: "StaffSearchHit[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.create",
            request_type: "CreateEnrollmentPayload",
            response_type: "EnrollmentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.transfer",
            request_type: "TransferEnrollmentPayload",
            response_type: "EnrollmentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.end",
            request_type: "EndEnrollmentPayload",
            response_type: "EnrollmentDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.bulk_create",
            request_type: "BulkEnrollPayload",
            response_type: "EnrollmentDetail[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.list_by_batch",
            request_type: "ListBatchRosterPayload",
            response_type: "BatchRosterItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "enrollments.list_by_student",
            request_type: "ListStudentEnrollmentsPayload",
            response_type: "StudentEnrollmentHistoryItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "import.inspect_file",
            request_type: "InspectFilePayload",
            response_type: "FileInspectionResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "import.preview_validation",
            request_type: "PreviewValidationPayload",
            response_type: "ValidationPreviewResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "import.execute",
            request_type: "ExecuteImportInput",
            response_type: "ImportSummaryResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "generator.generate_dataset",
            request_type: "GenerateDatasetPayload",
            response_type: "GenerationSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "generator.get_status",
            request_type: "EmptyPayload",
            response_type: "GeneratorStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "generator.switch_database",
            request_type: "SwitchDatabasePayload",
            response_type: "SwitchDatabaseResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "generator.switch_to_production_database",
            request_type: "EmptyPayload",
            response_type: "SwitchToProductionResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "fee_heads.list",
            request_type: "EmptyPayload",
            response_type: "FeeHeadItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "fee_heads.create",
            request_type: "CreateFeeHeadPayload",
            response_type: "FeeHeadItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_heads.update",
            request_type: "UpdateFeeHeadPayload",
            response_type: "FeeHeadItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_heads.delete",
            request_type: "DeleteFeeHeadPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_plans.list",
            request_type: "ListFeePlansPayload",
            response_type: "FeePlanItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_plans.create",
            request_type: "CreateFeePlanPayload",
            response_type: "FeePlanItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_plans.update",
            request_type: "UpdateFeePlanPayload",
            response_type: "FeePlanItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "fee_plans.delete",
            request_type: "DeleteFeePlanPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "concessions.list",
            request_type: "ListConcessionsPayload",
            response_type: "ConcessionItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "concessions.create",
            request_type: "CreateConcessionPayload",
            response_type: "ConcessionItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "concessions.delete",
            request_type: "DeleteConcessionPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "charges.generate_monthly",
            request_type: "GenerateMonthlyChargesPayload",
            response_type: "GenerationResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "charges.create_manual",
            request_type: "CreateManualChargePayload",
            response_type: "ChargeItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "charges.void",
            request_type: "VoidChargePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "charges.list_by_student",
            request_type: "ListStudentChargesPayload",
            response_type: "ChargeItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "payments.create",
            request_type: "CreatePaymentPayload",
            response_type: "PaymentReceipt",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "payments.void",
            request_type: "VoidPaymentPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "payments.list_by_student",
            request_type: "ListStudentPaymentsPayload",
            response_type: "PaymentReceipt[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "adjustments.create",
            request_type: "CreateAdjustmentPayload",
            response_type: "AdjustmentItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "refunds.create",
            request_type: "CreateRefundPayload",
            response_type: "RefundItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ledger.get_student_summary",
            request_type: "GetStudentLedgerSummaryPayload",
            response_type: "StudentLedgerSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "printers.list",
            request_type: "EmptyPayload",
            response_type: "SystemPrinterInfo[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "printers.get_settings",
            request_type: "EmptyPayload",
            response_type: "WorkstationPrinterSettings",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "printers.save_settings",
            request_type: "SavePrinterSettingsPayload",
            response_type: "WorkstationPrinterSettings",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "receipts.get_printable",
            request_type: "GetPrintableReceiptPayload",
            response_type: "PrintableReceiptData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ledger.get_cashier_student_overview",
            request_type: "GetCashierStudentOverviewPayload",
            response_type: "CashierStudentOverview",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.admit_with_payment",
            request_type: "AdmitStudentWithPaymentPayload",
            response_type: "AdmitStudentResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ledger.list_dues",
            request_type: "ListDuesPayload",
            response_type: "StudentDueSummaryItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ledger.get_due_statement",
            request_type: "GetDueStatementPayload",
            response_type: "DueStatementData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "expense_categories.list",
            request_type: "EmptyPayload",
            response_type: "ExpenseCategoryItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "expense_categories.create",
            request_type: "CreateExpenseCategoryPayload",
            response_type: "ExpenseCategoryItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "expenses.list",
            request_type: "ListExpensesPayload",
            response_type: "ExpenseItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "expenses.create",
            request_type: "CreateExpensePayload",
            response_type: "ExpenseItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "expenses.void",
            request_type: "VoidExpensePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "cash_days.get_or_open_today",
            request_type: "GetOrOpenTodayCashDayPayload",
            response_type: "CashDayItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "cash_days.get_summary",
            request_type: "GetCashDaySummaryPayload",
            response_type: "CashDaySummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "cash_days.close",
            request_type: "CloseCashDayPayload",
            response_type: "CashDaySummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.get_dashboard",
            request_type: "EmptyPayload",
            response_type: "BackupDashboard",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "backup.create_snapshot",
            request_type: "CreateSnapshotPayload",
            response_type: "BackupHistoryItem | null",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.verify",
            request_type: "VerifyBackupPayload",
            response_type: "VerificationResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.list_history",
            request_type: "ListBackupHistoryPayload",
            response_type: "BackupHistoryItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.list_destinations",
            request_type: "EmptyPayload",
            response_type: "BackupDestinationItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "backup.save_destination",
            request_type: "SaveDestinationPayload",
            response_type: "BackupDestinationItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.remove_destination",
            request_type: "RemoveDestinationPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "backup.get_schedules",
            request_type: "EmptyPayload",
            response_type: "BackupScheduleItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "backup.save_schedule",
            request_type: "SaveSchedulePayload",
            response_type: "BackupScheduleItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "restore.preview_backup",
            request_type: "RestorePreviewPayload",
            response_type: "BackupPreview",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "restore.execute",
            request_type: "RestoreExecutePayload",
            response_type: "RestoreResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "system.health_check",
            request_type: "EmptyPayload",
            response_type: "HealthCheckResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "diagnostics.get_info",
            request_type: "EmptyPayload",
            response_type: "DiagnosticsInfo",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "diagnostics.export_bundle",
            request_type: "ExportDiagnosticsBundlePayload",
            response_type: "DiagnosticsBundleResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "diagnostics.run_integrity_check",
            request_type: "EmptyPayload",
            response_type: "IntegrityCheckResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "attendance.get_session",
            request_type: "GetAttendanceSessionPayload",
            response_type: "AttendanceSessionInfo",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "attendance.save_session",
            request_type: "SaveAttendanceSessionPayload",
            response_type: "AttendanceSessionInfo",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "attendance.lock_session",
            request_type: "LockAttendanceSessionPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "attendance.get_student_summary",
            request_type: "GetStudentAttendanceSummaryPayload",
            response_type: "StudentAttendanceSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "attendance.get_batch_matrix",
            request_type: "GetBatchAttendanceMatrixPayload",
            response_type: "BatchAttendanceMatrixResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.list_outbox",
            request_type: "ListOutboxQuery",
            response_type: "ListOutboxResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.get_outbox_stats",
            request_type: "EmptyPayload",
            response_type: "OutboxStatsResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "comms.get_outbox_detail",
            request_type: "GetOutboxDetailPayload",
            response_type: "OutboxDetailResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.resend_message",
            request_type: "ResendMessagePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.cancel_message",
            request_type: "CancelMessagePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.flush_queue",
            request_type: "FlushQueuePayload",
            response_type: "FlushQueueResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.get_provider_configs",
            request_type: "EmptyPayload",
            response_type: "ProviderConfigItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "comms.save_provider_config",
            request_type: "SaveProviderConfigPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.send_test_message",
            request_type: "SendTestMessagePayload",
            response_type: "SendTestMessageResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "whatsapp.get_status",
            request_type: "EmptyPayload",
            response_type: "WhatsAppStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "whatsapp.get_qr",
            request_type: "EmptyPayload",
            response_type: "WhatsAppQrResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "comms.list_templates",
            request_type: "ListTemplatesPayload",
            response_type: "MessageTemplateItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.save_template",
            request_type: "SaveMessageTemplatePayload",
            response_type: "MessageTemplateItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.delete_template",
            request_type: "DeleteTemplatePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.list_automation_rules",
            request_type: "EmptyPayload",
            response_type: "AutomationRuleItem[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "comms.update_automation_rule",
            request_type: "UpdateAutomationRulePayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.analyze_text",
            request_type: "AnalyzeTextPayload",
            response_type: "TextAnalysisResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.get_due_reminder_candidates",
            request_type: "GetDueReminderCandidatesQuery",
            response_type: "DueReminderCandidate[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.enqueue_due_reminders",
            request_type: "EnqueueDueRemindersPayload",
            response_type: "EnqueueDueRemindersResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.preview_batch_broadcast",
            request_type: "PreviewBatchBroadcastPayload",
            response_type: "PreviewBatchBroadcastResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "comms.send_batch_broadcast",
            request_type: "SendBatchBroadcastPayload",
            response_type: "SendBatchBroadcastResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "grading_schemes.list",
            request_type: "EmptyPayload",
            response_type: "GradingSchemeSummary[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "exams.list",
            request_type: "EmptyPayload",
            response_type: "ExamSummary[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "exams.get",
            request_type: "GetExamPayload",
            response_type: "ExamDetails",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.create",
            request_type: "CreateExamPayload",
            response_type: "ExamDetails",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.update",
            request_type: "UpdateExamPayload",
            response_type: "ExamDetails",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.delete",
            request_type: "DeleteExamPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.get_mark_sheet",
            request_type: "GetMarkSheetPayload",
            response_type: "MarkSheetData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.save_marks",
            request_type: "SaveMarksPayload",
            response_type: "MarkSheetData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.verify_subject",
            request_type: "VerifySubjectMarksPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.unverify_subject",
            request_type: "UnverifySubjectMarksPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.preview_results",
            request_type: "PreviewResultsPayload",
            response_type: "ExamResultPreviewData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.publish_results",
            request_type: "PublishExamPayload",
            response_type: "PublishExamResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.revert_to_verified",
            request_type: "RevertPublishedPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.get_student_report_card",
            request_type: "GetStudentReportCardPayload",
            response_type: "StudentReportCardData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.get_tabulation_sheet",
            request_type: "GetBatchTabulationSheetPayload",
            response_type: "BatchTabulationSheetData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "search.global",
            request_type: "GlobalSearchPayload",
            response_type: "GlobalSearchResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_collection_summary",
            request_type: "CollectionSummaryPayload",
            response_type: "CollectionSummaryResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_collector_summary",
            request_type: "CollectorSummaryPayload",
            response_type: "CollectorSummaryResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_dues_aging_report",
            request_type: "DuesAgingPayload",
            response_type: "DuesAgingResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_expenses_summary",
            request_type: "ExpensesSummaryPayload",
            response_type: "ExpensesSummaryResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_attendance_summary",
            request_type: "AttendanceSummaryPayload",
            response_type: "AttendanceSummaryResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.get_exam_performance",
            request_type: "ExamPerformancePayload",
            response_type: "ExamPerformanceResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "reports.export_csv",
            request_type: "ExportCsvPayload",
            response_type: "ExportCsvResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_student_360",
            request_type: "Student360Payload",
            response_type: "Student360Response",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.add_student_note",
            request_type: "AddStudentNotePayload",
            response_type: "StudentTimelineNote",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_today_cockpit",
            request_type: "TodayCockpitPayload",
            response_type: "TodayCockpitResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_coaching_day_timeline",
            request_type: "CoachingDayTimelinePayload",
            response_type: "CoachingDayTimelineResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_batch_health_list",
            request_type: "BatchHealthPayload",
            response_type: "BatchHealthItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_batch_profitability",
            request_type: "BatchProfitabilityPayload",
            response_type: "BatchProfitabilityItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.update_batch_cost_config",
            request_type: "UpdateBatchCostConfigPayload",
            response_type: "BatchCostConfig",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.get_smart_due_queue",
            request_type: "SmartDueQueuePayload",
            response_type: "SmartDueQueueResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "intelligence.record_due_contact",
            request_type: "RecordDueContactPayload",
            response_type: "DueContactLogItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.verify_receipt",
            request_type: "VerifyReceiptPayload",
            response_type: "ReceiptVerificationResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.check_duplicate_payment",
            request_type: "CheckDuplicatePaymentPayload",
            response_type: "DuplicateWarningResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.get_guardian_contact_health",
            request_type: "GetGuardianContactHealthPayload",
            response_type: "ContactHealthReport",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.bulk_update_guardian_contacts",
            request_type: "BulkUpdateGuardianContactsPayload",
            response_type: "BulkUpdateResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.get_outbox_lifecycle",
            request_type: "GetOutboxLifecyclePayload",
            response_type: "OutboxLifecycleSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.retry_outbox_messages",
            request_type: "RetryOutboxMessagesPayload",
            response_type: "RetryResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.flush_offline_queue",
            request_type: "EmptyPayload",
            response_type: "FlushResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "integrity.get_resilience_status",
            request_type: "EmptyPayload",
            response_type: "ResilienceStatus",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "integrity.get_disaster_recovery_score",
            request_type: "EmptyPayload",
            response_type: "RecoveryScoreResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "integrity.run_verification_drill",
            request_type: "RunVerificationDrillPayload",
            response_type: "DrillResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.export_migration_bundle",
            request_type: "ExportMigrationBundlePayload",
            response_type: "MigrationExportResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.import_migration_bundle",
            request_type: "ImportMigrationBundlePayload",
            response_type: "MigrationImportResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "integrity.run_database_integrity_checks",
            request_type: "RunDatabaseIntegrityChecksPayload",
            response_type: "DbIntegrityResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.ask_copilot",
            request_type: "AskCopilotPayload",
            response_type: "CopilotResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.get_conversation",
            request_type: "GetConversationPayload",
            response_type: "CopilotConversationDetails",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.clear_conversation",
            request_type: "ClearConversationPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.get_wallet_balance",
            request_type: "EmptyPayload",
            response_type: "AiWalletBalance",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "ai.get_credit_ledger",
            request_type: "GetCreditLedgerPayload",
            response_type: "CreditTransactionItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.sync_credits",
            request_type: "EmptyPayload",
            response_type: "AiWalletBalance",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "ai.record_credit_grant",
            request_type: "RecordCreditGrantPayload",
            response_type: "CreditTransactionItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.get_privacy_settings",
            request_type: "EmptyPayload",
            response_type: "PrivacySettings",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "ai.update_privacy_settings",
            request_type: "UpdatePrivacySettingsPayload",
            response_type: "PrivacySettings",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.get_gateway_config",
            request_type: "EmptyPayload",
            response_type: "AiGatewayConfig",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "ai.update_gateway_config",
            request_type: "UpdateGatewayConfigPayload",
            response_type: "AiGatewayConfig",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "ai.test_gateway_connection",
            request_type: "EmptyPayload",
            response_type: "GatewayTestResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "ai.get_audit_log",
            request_type: "GetAiAuditLogPayload",
            response_type: "AiAuditLogItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "printers.get_template_configs",
            request_type: "EmptyPayload",
            response_type: "PrintTemplateConfig[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "printers.update_template_config",
            request_type: "UpdatePrintTemplateConfigPayload",
            response_type: "PrintTemplateConfig",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "printers.record_print_job",
            request_type: "RecordPrintJobPayload",
            response_type: "PrintJobRun",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "printers.list_print_jobs",
            request_type: "ListPrintJobsPayload",
            response_type: "PrintJobListResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "students.get_id_card_data",
            request_type: "GetIdCardDataPayload",
            response_type: "StudentIdCardData[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.get_admit_card_data",
            request_type: "GetAdmitCardDataPayload",
            response_type: "AdmitCardBatchData",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "exams.get_batch_report_cards",
            request_type: "GetBatchReportCardsPayload",
            response_type: "StudentReportCardData[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "finances.get_batch_due_statements",
            request_type: "GetBatchDueStatementsPayload",
            response_type: "DueStatementData[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "license.get_status",
            request_type: "EmptyPayload",
            response_type: "LicenseStatusResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "license.get_request_code",
            request_type: "EmptyPayload",
            response_type: "RequestCodeResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "license.import_file",
            request_type: "ImportLicenseInput",
            response_type: "LicenseStatusResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "license.get_refresh_state",
            request_type: "EmptyPayload",
            response_type: "RefreshStateResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "license.activate_online",
            request_type: "ActivateOnlineInput",
            response_type: "ActivateOnlineResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "license.refresh_now",
            request_type: "RefreshLicenseNowInput",
            response_type: "RefreshLicenseNowResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.get_overview",
            request_type: "EmptyPayload",
            response_type: "VendorDashboardOverview",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "vendor.login_totp",
            request_type: "AdminLoginInput",
            response_type: "AdminLoginResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.record_payment",
            request_type: "RecordManualPaymentInput",
            response_type: "RecordManualPaymentResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.issue_offline_license",
            request_type: "IssueOfflineLicenseInput",
            response_type: "IssueOfflineLicenseResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.revoke_license",
            request_type: "RevokeLicenseInput",
            response_type: "VendorRevocationRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.update_release_status",
            request_type: "UpdateReleaseStatusInput",
            response_type: "VendorReleaseRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.reset_transfer_count",
            request_type: "ResetTransferCountInput",
            response_type: "ResetTransferCountResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.create_customer",
            request_type: "CreateVendorCustomerInput",
            response_type: "VendorCustomerRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.publish_release",
            request_type: "PublishReleaseInput",
            response_type: "VendorReleaseRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.promote_release",
            request_type: "PromoteReleaseInput",
            response_type: "ReleaseChannelHeadRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.record_key_recovery_drill",
            request_type: "RecordKeyRecoveryDrillInput",
            response_type: "KeyRecoveryDrillRow",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.login",
            request_type: "PortalLoginInput",
            response_type: "PortalLoginResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.verify_otp",
            request_type: "PortalVerifyOtpInput",
            response_type: "PortalVerifyOtpResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.get_overview",
            request_type: "PortalTokenPayload",
            response_type: "PortalOverviewResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "pricing.get_catalog",
            request_type: "EmptyPayload",
            response_type: "PricingCatalogResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "vendor.update_price_tier",
            request_type: "AdminUpdatePriceTierInput",
            response_type: "UpdatedPriceTierResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "checkout.create_session",
            request_type: "CreateCheckoutSessionInput",
            response_type: "CheckoutSessionResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "checkout.submit_bank_transfer",
            request_type: "SubmitBankTransferInput",
            response_type: "CheckoutOrderStatusView",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "checkout.get_status",
            request_type: "CheckoutGetStatusPayload",
            response_type: "CheckoutOrderStatusView",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "checkout.handle_payment_webhook",
            request_type: "PaymentWebhookInput",
            response_type: "WebhookAckResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.list_pending_bank_transfers",
            request_type: "EmptyPayload",
            response_type: "PendingBankTransferOrderView[]",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "vendor.approve_bank_transfer",
            request_type: "ApproveBankTransferInput",
            response_type: "CheckoutOrderStatusView",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.list_devices",
            request_type: "PortalTokenPayload",
            response_type: "PortalDeviceListView",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.rename_device",
            request_type: "PortalRenameDeviceInput",
            response_type: "PortalDeviceItem",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.request_device_otp",
            request_type: "PortalRequestDeviceOtpInput",
            response_type: "PortalDeviceOtpResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.deactivate_device",
            request_type: "PortalDeactivateDeviceInput",
            response_type: "DeviceActionResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.replace_device",
            request_type: "PortalReplaceDeviceInput",
            response_type: "DeviceActionResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "portal.request_transfer_reset",
            request_type: "PortalRequestTransferResetInput",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "vendor.admin_reset_transfer_count",
            request_type: "AdminResetTransferCountInput",
            response_type: "ResetTransferCountResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "security.get_secrets_health",
            request_type: "EmptyPayload",
            response_type: "SecretsHealthReport",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.get_host_status",
            request_type: "EmptyPayload",
            response_type: "LanHostStatus",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.start_host",
            request_type: "StartHostPayload",
            response_type: "LanHostStatus",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.stop_host",
            request_type: "EmptyPayload",
            response_type: "LanHostStatus",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.update_host_config",
            request_type: "UpdateHostConfigPayload",
            response_type: "LanHostStatus",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.get_firewall_guidance",
            request_type: "EmptyPayload",
            response_type: "FirewallGuidance",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.discover_hosts",
            request_type: "DiscoverHostsPayload",
            response_type: "DiscoveredHostList",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.ping_host",
            request_type: "PingHostPayload",
            response_type: "HostPingResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.open_pairing_window",
            request_type: "OpenPairingWindowPayload",
            response_type: "PairingWindowInfo",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.close_pairing_window",
            request_type: "EmptyPayload",
            response_type: "SuccessResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.list_workstations",
            request_type: "EmptyPayload",
            response_type: "WorkstationListResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.get_workstation",
            request_type: "GetWorkstationPayload",
            response_type: "WorkstationDetail",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.rename_workstation",
            request_type: "RenameWorkstationPayload",
            response_type: "WorkstationSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.block_workstation",
            request_type: "BlockWorkstationPayload",
            response_type: "WorkstationSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.unblock_workstation",
            request_type: "UnblockWorkstationPayload",
            response_type: "WorkstationSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.revoke_workstation",
            request_type: "RevokeWorkstationPayload",
            response_type: "WorkstationSummary",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.list_device_events",
            request_type: "ListDeviceEventsPayload",
            response_type: "DeviceEventItem[]",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.pair_with_host",
            request_type: "ClientPairPayload",
            response_type: "PairSuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.get_workstation_credentials",
            request_type: "EmptyPayload",
            response_type: "WorkstationCredentials | null",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.clear_workstation_credentials",
            request_type: "EmptyPayload",
            response_type: "ClearedResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.open_mobile_pairing_window",
            request_type: "OpenPairingWindowPayload",
            response_type: "MobilePairingWindowInfo",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.update_host_receipt_printer",
            request_type: "UpdateHostReceiptPrinterPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.set_staff_pin",
            request_type: "SetStaffPinPayload",
            response_type: "SuccessResponse",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.get_client_status",
            request_type: "EmptyPayload",
            response_type: "WorkstationClientStatus",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.reconnect",
            request_type: "EmptyPayload",
            response_type: "ReconnectWorkstationResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.disconnect_workstation",
            request_type: "EmptyPayload",
            response_type: "SuccessResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.test_connection",
            request_type: "EmptyPayload",
            response_type: "TestConnectionResponse",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.get_update_mirror_status",
            request_type: "EmptyPayload",
            response_type: "LanUpdateMirrorStatus",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.configure_update_mirror",
            request_type: "ConfigureMirrorInput",
            response_type: "LanUpdateMirrorStatus",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.stage_mirror_update",
            request_type: "StageMirrorUpdateInput",
            response_type: "LanUpdateMirrorStatus",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.clear_mirror_update",
            request_type: "ClearMirrorUpdateInput",
            response_type: "LanUpdateMirrorStatus",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "lan.check_workstation_update",
            request_type: "EmptyPayload",
            response_type: "WorkstationUpdateCheckResult",
            is_empty_req: true,
        },
        CommandContractDescriptor {
            name: "lan.download_workstation_update",
            request_type: "DownloadWorkstationUpdatePayload",
            response_type: "WorkstationUpdateDownloadResult",
            is_empty_req: false,
        },
        CommandContractDescriptor {
            name: "system.get_client_info",
            request_type: "EmptyPayload",
            response_type: "ClientInfoResponse",
            is_empty_req: true,
        },
    ]
}
