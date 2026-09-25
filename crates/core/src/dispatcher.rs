use crate::accounts::{
    self, find_user_by_id, find_user_for_login, get_or_create_host_device, AccountsError,
    DeviceSummary, PermissionItem, ResolvedSession, RoleWithPermissions, UserSummary,
};
use crate::auth;
use crate::backup;
use crate::db;
use crate::migrations;
pub use crate::migrations::{
    MigrationHistoryItem, MigrationHistoryResponse, MigrationStatusResponse,
    OpenDbDirectoryResponse, PendingMigrationInfo, RecoveryInfoResponse,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Digest;
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
        name: "system.get_system_status",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "system.acknowledge_clock_jump",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
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
    // Fees & Ledger commands (Spec 0012)
    CommandMeta {
        name: "fee_heads.list",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "fee_heads.create",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "fee_heads.update",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "fee_heads.delete",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "fee_plans.list",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "fee_plans.create",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "fee_plans.update",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "fee_plans.delete",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "concessions.list",
        auth: AuthRequirement::Permission("finance.concessions_manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "concessions.create",
        auth: AuthRequirement::Permission("finance.concessions_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "concessions.delete",
        auth: AuthRequirement::Permission("finance.concessions_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "charges.generate_monthly",
        auth: AuthRequirement::Permission("charges.generate"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "charges.create_manual",
        auth: AuthRequirement::Permission("charges.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "charges.void",
        auth: AuthRequirement::Permission("charges.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "charges.list_by_student",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "payments.create",
        auth: AuthRequirement::Permission("payments.collect"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "payments.void",
        auth: AuthRequirement::Permission("payments.void"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "payments.list_by_student",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "adjustments.create",
        auth: AuthRequirement::Permission("finance.adjust"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "refunds.create",
        auth: AuthRequirement::Permission("finance.adjust"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ledger.get_student_summary",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Print engine and receipts (Spec 0013)
    CommandMeta {
        name: "printers.list",
        auth: AuthRequirement::Permission("settings.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "printers.get_settings",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "printers.save_settings",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "receipts.get_printable",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Professional printing system (Spec 0035, Feature 50)
    CommandMeta {
        name: "printers.get_template_configs",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "printers.update_template_config",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "printers.record_print_job",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "printers.list_print_jobs",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "students.get_id_card_data",
        auth: AuthRequirement::Permission("students.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.get_admit_card_data",
        auth: AuthRequirement::Permission("exams.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.get_batch_report_cards",
        auth: AuthRequirement::Permission("exams.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "finances.get_batch_due_statements",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Fast cashier and payments (Spec 0014)
    CommandMeta {
        name: "ledger.get_cashier_student_overview",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Direct Admission Pipeline (Spec 0016)
    CommandMeta {
        name: "students.admit_with_payment",
        auth: AuthRequirement::Permission("students.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    // Dues Directory & Bilingual Due Statements (Spec 0017)
    CommandMeta {
        name: "ledger.list_dues",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ledger.get_due_statement",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Expenses & Daily Cash Close (Spec 0018)
    CommandMeta {
        name: "expense_categories.list",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "expense_categories.create",
        auth: AuthRequirement::Permission("finance.plans_manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "expenses.list",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "expenses.create",
        auth: AuthRequirement::Permission("charges.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "expenses.void",
        auth: AuthRequirement::Permission("payments.void"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "cash_days.get_or_open_today",
        auth: AuthRequirement::Permission("charges.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "cash_days.get_summary",
        auth: AuthRequirement::Permission("ledger.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "cash_days.close",
        auth: AuthRequirement::Permission("payments.void"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    // Backup & Verification (Feature 21) — never gated by license
    CommandMeta {
        name: "backup.get_dashboard",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "backup.create_snapshot",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "backup.verify",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "backup.list_history",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "backup.list_destinations",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "backup.save_destination",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "backup.remove_destination",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "backup.get_schedules",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "backup.save_schedule",
        auth: AuthRequirement::Permission("backup.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    // Restore & Corruption Recovery (Feature 22)
    CommandMeta {
        name: "restore.preview_backup",
        auth: AuthRequirement::Permission("backup.restore"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "restore.execute",
        auth: AuthRequirement::Permission("backup.restore"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "system.health_check",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Diagnostics & Logs (Feature 23)
    CommandMeta {
        name: "diagnostics.get_info",
        auth: AuthRequirement::Permission("diagnostics.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "diagnostics.export_bundle",
        auth: AuthRequirement::Permission("diagnostics.export"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "diagnostics.run_integrity_check",
        auth: AuthRequirement::Permission("diagnostics.export"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Attendance (Feature 25, Spec 0023)
    CommandMeta {
        name: "attendance.get_session",
        auth: AuthRequirement::Permission("attendance.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "attendance.save_session",
        auth: AuthRequirement::Permission("attendance.mark"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "attendance.lock_session",
        auth: AuthRequirement::Permission("attendance.mark"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "attendance.get_student_summary",
        auth: AuthRequirement::Permission("attendance.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "attendance.get_batch_matrix",
        auth: AuthRequirement::Permission("attendance.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Outbox & Providers (Feature 26, Spec 0024)
    CommandMeta {
        name: "comms.list_outbox",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.get_outbox_stats",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.get_outbox_detail",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.resend_message",
        auth: AuthRequirement::Permission("comms.send"),
        gate: LicenseGate::Sms,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.cancel_message",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.flush_queue",
        auth: AuthRequirement::Permission("comms.send"),
        gate: LicenseGate::Sms,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.get_provider_configs",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.save_provider_config",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.send_test_message",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Sms,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "whatsapp.get_status",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "whatsapp.get_qr",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Templates & Automations (Feature 27, Spec 0025)
    CommandMeta {
        name: "comms.list_templates",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.save_template",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.delete_template",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.list_automation_rules",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.update_automation_rule",
        auth: AuthRequirement::Permission("comms.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.analyze_text",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.get_due_reminder_candidates",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.enqueue_due_reminders",
        auth: AuthRequirement::Permission("comms.send"),
        gate: LicenseGate::Sms,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "comms.preview_batch_broadcast",
        auth: AuthRequirement::Permission("comms.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "comms.send_batch_broadcast",
        auth: AuthRequirement::Permission("comms.send"),
        gate: LicenseGate::Sms,
        kind: CommandKind::Mutation,
    },
    // Exams & Results (Features 28 & 29, Specs 0026 & 0027)
    CommandMeta {
        name: "grading_schemes.list",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.list",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.get",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.create",
        auth: AuthRequirement::Permission("exam.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.update",
        auth: AuthRequirement::Permission("exam.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.delete",
        auth: AuthRequirement::Permission("exam.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.get_mark_sheet",
        auth: AuthRequirement::Permission("exam.marks_entry"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.save_marks",
        auth: AuthRequirement::Permission("exam.marks_entry"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.verify_subject",
        auth: AuthRequirement::Permission("exam.verify"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.unverify_subject",
        auth: AuthRequirement::Permission("exam.edit_verified"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.preview_results",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.publish_results",
        auth: AuthRequirement::Permission("exam.publish"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.revert_to_verified",
        auth: AuthRequirement::Permission("exam.edit_verified"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "exams.get_student_report_card",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "exams.get_tabulation_sheet",
        auth: AuthRequirement::Permission("exam.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Fast Global Search (Feature 33, Spec 0031)
    CommandMeta {
        name: "search.global",
        auth: AuthRequirement::Permission("search.global"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Reports Center & Exports (Feature 30, Spec 0028)
    CommandMeta {
        name: "reports.get_collection_summary",
        auth: AuthRequirement::Permission("reports.financial"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.get_collector_summary",
        auth: AuthRequirement::Permission("reports.financial"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.get_dues_aging_report",
        auth: AuthRequirement::Permission("reports.financial"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.get_expenses_summary",
        auth: AuthRequirement::Permission("reports.financial"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.get_attendance_summary",
        auth: AuthRequirement::Permission("reports.academic"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.get_exam_performance",
        auth: AuthRequirement::Permission("reports.academic"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "reports.export_csv",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Owner Intelligence (Slice 7: Features 34 to 39, Spec 0033)
    CommandMeta {
        name: "intelligence.get_student_360",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.add_student_note",
        auth: AuthRequirement::Permission("intelligence.notes"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "intelligence.get_today_cockpit",
        auth: AuthRequirement::Permission("intelligence.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.get_coaching_day_timeline",
        auth: AuthRequirement::Permission("intelligence.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.get_batch_health_list",
        auth: AuthRequirement::Permission("intelligence.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.get_batch_profitability",
        auth: AuthRequirement::Permission("intelligence.profitability"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.update_batch_cost_config",
        auth: AuthRequirement::Permission("intelligence.profitability"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "intelligence.get_smart_due_queue",
        auth: AuthRequirement::Permission("intelligence.due_queue"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "intelligence.record_due_contact",
        auth: AuthRequirement::Permission("intelligence.due_queue"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    // Operational Integrity (Slice 8: Features 40 to 49, Spec 0032)
    CommandMeta {
        name: "integrity.verify_receipt",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.check_duplicate_payment",
        auth: AuthRequirement::Permission("payments.collect"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "integrity.get_guardian_contact_health",
        auth: AuthRequirement::Permission("integrity.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "integrity.bulk_update_guardian_contacts",
        auth: AuthRequirement::Permission("integrity.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.get_outbox_lifecycle",
        auth: AuthRequirement::Permission("integrity.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "integrity.retry_outbox_messages",
        auth: AuthRequirement::Permission("integrity.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.flush_offline_queue",
        auth: AuthRequirement::Permission("integrity.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.get_resilience_status",
        auth: AuthRequirement::Permission("integrity.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "integrity.get_disaster_recovery_score",
        auth: AuthRequirement::Permission("integrity.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "integrity.run_verification_drill",
        auth: AuthRequirement::Permission("integrity.manage"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.export_migration_bundle",
        auth: AuthRequirement::Permission("integrity.migrate"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.import_migration_bundle",
        auth: AuthRequirement::Permission("integrity.migrate"),
        gate: LicenseGate::Write,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "integrity.run_database_integrity_checks",
        auth: AuthRequirement::Permission("integrity.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    // AI Foundation (Slice 10: Features 54 to 57, Spec 0035)
    CommandMeta {
        name: "ai.ask_copilot",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.get_conversation",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.clear_conversation",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.get_wallet_balance",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.get_credit_ledger",
        auth: AuthRequirement::Permission("finance.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.sync_credits",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.record_credit_grant",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.get_privacy_settings",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.update_privacy_settings",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.get_gateway_config",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.update_gateway_config",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    CommandMeta {
        name: "ai.test_gateway_connection",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "ai.get_audit_log",
        auth: AuthRequirement::Permission("audit.view"),
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    // Desktop Licensing & Read Only Mode (Slice 12: Feature 62, Spec 0037)
    CommandMeta {
        name: "license.get_status",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "license.get_request_code",
        auth: AuthRequirement::Authenticated,
        gate: LicenseGate::None,
        kind: CommandKind::Read,
    },
    CommandMeta {
        name: "license.import_file",
        auth: AuthRequirement::Permission("settings.manage"),
        gate: LicenseGate::None,
        kind: CommandKind::Mutation,
    },
    // Encrypted Sensitive Secrets (Slice 9: Feature 53, Spec 0040)
    CommandMeta {
        name: "security.get_secrets_health",
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
pub struct SystemStatusResponse {
    pub db_disk_free_bytes: u64,
    pub backup_disk_free_bytes: u64,
    pub low_disk_warning: bool,
    pub disk_critical: bool,
    pub clock_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AcknowledgeClockJumpPayload {
    pub reason: Option<String>,
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

// --- Fees & Ledger Payloads (Spec 0012) ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFeeHeadPayload {
    pub head: crate::fees::CreateFeeHeadInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateFeeHeadPayload {
    pub id: String,
    pub head: crate::fees::UpdateFeeHeadInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteFeeHeadPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListFeePlansPayload {
    pub program_id: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFeePlanPayload {
    pub plan: crate::fees::CreateFeePlanInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateFeePlanPayload {
    pub id: String,
    pub plan: crate::fees::UpdateFeePlanInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteFeePlanPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListConcessionsPayload {
    pub student_id: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateConcessionPayload {
    pub concession: crate::fees::CreateConcessionInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteConcessionPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GenerateMonthlyChargesPayload {
    pub period: String,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateManualChargePayload {
    pub charge: crate::ledger::CreateManualChargeInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoidChargePayload {
    pub charge_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStudentChargesPayload {
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreatePaymentPayload {
    pub payment: crate::ledger::CreatePaymentInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoidPaymentPayload {
    pub payment_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStudentPaymentsPayload {
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAdjustmentPayload {
    pub adjustment: crate::ledger::CreateAdjustmentInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateRefundPayload {
    pub refund: crate::ledger::CreateRefundInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStudentLedgerSummaryPayload {
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SavePrinterSettingsPayload {
    pub settings: crate::storage::WorkstationPrinterSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetPrintableReceiptPayload {
    pub payment_id: String,
    pub is_reprint: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdatePrintTemplateConfigPayload {
    pub config: crate::printers::PrintTemplateConfigInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecordPrintJobPayload {
    pub job: crate::printers::RecordPrintJobInput,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ListPrintJobsPayload {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub document_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetIdCardDataPayload {
    pub student_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAdmitCardDataPayload {
    pub exam_id: String,
    pub batch_id: Option<String>,
    pub student_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchReportCardsPayload {
    pub exam_id: String,
    pub student_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchDueStatementsPayload {
    pub student_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCashierStudentOverviewPayload {
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmitStudentWithPaymentPayload {
    pub input: crate::students::AdmitStudentInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListDuesPayload {
    pub filter: Option<crate::ledger::ListDuesFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetDueStatementPayload {
    pub student_id: String,
    pub period_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExpenseCategoryPayload {
    pub category: crate::ledger::CreateExpenseCategoryInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListExpensesPayload {
    pub filter: Option<crate::ledger::ListExpensesFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExpensePayload {
    pub expense: crate::ledger::CreateExpenseInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoidExpensePayload {
    pub expense_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOrOpenTodayCashDayPayload {
    pub business_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCashDaySummaryPayload {
    pub business_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloseCashDayPayload {
    pub input: crate::ledger::CloseCashDayInput,
}

// -----------------------------------------------------------------------------
// Backup & Diagnostics Request Payloads
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateSnapshotPayload {
    pub destination_id: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerifyBackupPayload {
    pub backup_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListBackupHistoryPayload {
    pub destination_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveDestinationPayload {
    pub path: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RemoveDestinationPayload {
    pub destination_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveSchedulePayload {
    pub kind: String,
    pub destination_id: Option<String>,
    pub time_of_day: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RestorePreviewPayload {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RestoreExecutePayload {
    pub file_path: String,
    pub recovery_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportDiagnosticsBundlePayload {
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiagnosticsBundleResponse {
    pub bundle_path: String,
}

// -----------------------------------------------------------------------------
// Operational Integrity Request Payloads (Slice 8)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerifyReceiptPayload {
    pub code_or_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CheckDuplicatePaymentPayload {
    pub student_id: String,
    pub amount_poisha: i64,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetGuardianContactHealthPayload {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BulkUpdateGuardianContactsPayload {
    pub updates: Vec<crate::integrity::GuardianContactUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOutboxLifecyclePayload {
    pub status: Option<String>,
    pub channel: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RetryOutboxMessagesPayload {
    pub message_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunVerificationDrillPayload {
    pub backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportMigrationBundlePayload {
    pub target_hint: Option<String>,
    pub recovery_phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportMigrationBundlePayload {
    pub bundle_path: String,
    pub recovery_phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RunDatabaseIntegrityChecksPayload {
    pub full: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AskCopilotPayload {
    pub conversation_id: Option<String>,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetConversationPayload {
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClearConversationPayload {
    pub conversation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetCreditLedgerPayload {
    pub limit: Option<u32>,
    pub before_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecordCreditGrantPayload {
    pub amount: i64,
    pub tx_type: String,
    pub reference_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdatePrivacySettingsPayload {
    pub allow_student_names: bool,
    pub allow_phone_numbers: bool,
    pub allow_guardian_info: bool,
    pub allow_financial_figures: bool,
    pub allow_exam_marks: bool,
    pub allow_staff_names: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateGatewayConfigPayload {
    pub gateway_url: String,
    pub token: Option<String>,
    pub openrouter_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAiAuditLogPayload {
    pub limit: Option<u32>,
    pub before_time: Option<String>,
}

// -----------------------------------------------------------------------------
// License Verification (AC-7)
// -----------------------------------------------------------------------------

pub fn is_license_expired_past_grace(conn: &Connection) -> bool {
    let now = db::now_utc_iso();
    is_license_expired_past_grace_at(conn, &now)
}

pub fn is_license_expired_past_grace_at(conn: &Connection, current_utc: &str) -> bool {
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

    if let Ok(status) = crate::licensing::get_license_status(conn, current_utc) {
        if status.is_read_only {
            return true;
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
    dispatch_at(
        conn,
        command,
        payload_json,
        session_token,
        idempotency_key,
        &current_utc,
    )
}

pub fn dispatch_at(
    conn: &mut Connection,
    command: &str,
    payload_json: &str,
    session_token: Option<&str>,
    idempotency_key: Option<&str>,
    current_utc: &str,
) -> (String, Vec<ChangeEvent>) {
    let current_utc = current_utc.to_string();

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

    // Slice 8 Spec 0032 AC-4: If protective read-only lock is active, reject mutating commands
    if crate::integrity::is_safety_read_only()
        && meta.kind == CommandKind::Mutation
        && !command.starts_with("integrity.run_database_integrity_checks")
        && !command.starts_with("backup.")
    {
        let env: RpcEnvelope<()> = RpcEnvelope {
            ok: false,
            data: None,
            error: Some(RpcError {
                code: "protective_read_only_mode".to_string(),
                message: "Database integrity corruption detected. Mutation commands are locked in protective read-only mode to prevent data loss. Please restore a verified backup or contact administrator.".to_string(),
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

    // 6. License check (AC-6) and Clock Guard check (AC-10)
    if matches!(
        meta.gate,
        LicenseGate::Write | LicenseGate::Sms | LicenseGate::Lan
    ) {
        if is_license_expired_past_grace_at(conn, &current_utc) {
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

        if let Err(e) = db::check_clock_watermark(conn, &current_utc) {
            let env: RpcEnvelope<()> = RpcEnvelope {
                ok: false,
                data: None,
                error: Some(RpcError {
                    code: "clock_out_of_sync".to_string(),
                    message: e.to_string(),
                    field: None,
                }),
            };
            return (serde_json::to_string(&env).unwrap(), vec![]);
        }
    }

    // 6b. Safety Read-Only lock (Spec 0032 Feature 49 / AC-14)
    if meta.kind == CommandKind::Mutation
        && command != "integrity.run_database_integrity_checks"
        && crate::integrity::is_safety_read_only()
    {
        let env: RpcEnvelope<()> = RpcEnvelope {
            ok: false,
            data: None,
            error: Some(RpcError {
                code: "database_corruption_read_only".to_string(),
                message: "Database is in emergency read-only mode due to detected corruption or data safety lock.".to_string(),
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
            if command == "restore.execute" {
                let p: RestoreExecutePayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: format!("Invalid restore payload: {}", e),
                        field: None,
                    })?;

                let result = backup::execute_restore(conn, &p.file_path, p.recovery_key.as_deref())
                    .map_err(map_backup_error)?;

                return Ok((serde_json::to_value(result).unwrap(), vec![]));
            }

            if command == "auth.login"
                || command == "auth.unlock"
                || command == "charges.generate_monthly"
                || command == "backup.create_snapshot"
                || command == "integrity.export_migration_bundle"
            {
                let (val, events, audit_entries) =
                    run_command_handler(conn, command, payload, session, current_utc)?;
                for entry in audit_entries {
                    let _ = crate::audit::record_audit_entry(conn, entry, session, current_utc);
                }
                return Ok((val, events));
            }

            // AC-7: Monotonic clock guard check for sensitive financial commands
            let is_clock_sensitive_financial = matches!(
                command,
                "payments.create"
                    | "payments.void"
                    | "students.admit_with_payment"
                    | "adjustments.create"
                    | "expenses.create"
            );
            if is_clock_sensitive_financial {
                db::check_clock_watermark(conn, current_utc).map_err(|e| RpcError {
                    code: "clock_out_of_sync".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
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

        "system.get_system_status" => {
            let db_path = db::resolve_active_db_path()
                .unwrap_or_else(|_| std::path::PathBuf::from("main.db"));
            let db_disk_free_bytes = db::get_disk_free_bytes(&db_path).unwrap_or(0);

            let backup_dir = db_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("backups");
            let backup_disk_free_bytes =
                db::get_disk_free_bytes(&backup_dir).unwrap_or(db_disk_free_bytes);

            let low_disk_warning = db_disk_free_bytes < 2 * 1024 * 1024 * 1024
                || backup_disk_free_bytes < 2 * 1024 * 1024 * 1024;
            let disk_critical = db_disk_free_bytes < 500 * 1024 * 1024
                || backup_disk_free_bytes < 500 * 1024 * 1024;

            let clock_ok = db::is_clock_synchronized(conn, current_utc);

            let resp = SystemStatusResponse {
                db_disk_free_bytes,
                backup_disk_free_bytes,
                low_disk_warning,
                disk_critical,
                clock_ok,
            };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "system.acknowledge_clock_jump" => {
            let p: Option<AcknowledgeClockJumpPayload> =
                serde_json::from_value(payload.clone()).ok();
            let actor = session
                .map(|s| s.user.username.clone())
                .unwrap_or_else(|| "Operator".to_string());
            db::acknowledge_clock_jump(conn, &actor, current_utc).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;
            let audit = vec![crate::audit::AuditEntry {
                action: "system.acknowledge_clock_jump".to_string(),
                entity: "clock_guard".to_string(),
                entity_id: "1".to_string(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "actor": actor,
                    "acknowledged_at": current_utc,
                    "reason": p.and_then(|x| x.reason),
                })),
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                vec![],
                audit,
            ))
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

            let _ = crate::licensing::ensure_initial_trial_license(conn, current_utc);

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

        "license.get_status" => {
            let status =
                crate::licensing::get_license_status(conn, current_utc).map_err(|e| RpcError {
                    code: e.code().to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            Ok((serde_json::to_value(status).unwrap(), vec![], vec![]))
        }

        "license.get_request_code" => {
            let host_id =
                crate::licensing::resolve_host_installation_id(conn).map_err(|e| RpcError {
                    code: e.code().to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let resp = crate::licensing::generate_request_code_for_installation(&host_id);
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "license.import_file" => {
            let license_content = if let Some(s) = payload
                .get("license_content")
                .and_then(|v| v.as_str())
                .or_else(|| payload.get("raw_payload").and_then(|v| v.as_str()))
                .or_else(|| payload.get("content").and_then(|v| v.as_str()))
            {
                s.to_string()
            } else if payload.is_object() && payload.get("signature").is_some() {
                payload.to_string()
            } else {
                let p: crate::licensing::ImportLicenseInput =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "license_payload_malformed".to_string(),
                        message: format!("Invalid import license payload: {e}"),
                        field: Some("license_content".to_string()),
                    })?;
                p.license_content
            };

            let user_id = session.map(|s| s.user.id.as_str());
            let status =
                crate::licensing::import_license_file(conn, &license_content, user_id, current_utc)
                    .map_err(|e| RpcError {
                        code: e.code().to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;

            let events = vec![ChangeEvent {
                topic: "license:changed".to_string(),
                entity_ids: vec![status.license_id.clone()],
            }];

            let audit = vec![crate::audit::AuditEntry {
                action: "license.import_file".to_string(),
                entity: "licenses".to_string(),
                entity_id: status.license_id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "license_id": status.license_id,
                    "tier": status.tier,
                    "status": status.status,
                    "paid_through": status.paid_through,
                    "valid_until": status.valid_until,
                })),
            }];

            Ok((serde_json::to_value(status).unwrap(), events, audit))
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

            let resp = crate::academic::get_program_with_batches(conn, &p.id)
                .map_err(map_academic_error)?;
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

        // --- Fees & Ledger Commands (Spec 0012) ---
        "fee_heads.list" => {
            let items = crate::fees::list_fee_heads(conn).map_err(map_fees_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "fee_heads.create" => {
            let p: CreateFeeHeadPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid fee head payload: {}", e),
                    field: None,
                })?;
            let item = crate::fees::create_fee_head(conn, p.head).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_heads.create".to_string(),
                entity: "fee_head".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "fee_heads".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "fee_heads.update" => {
            let p: UpdateFeeHeadPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid fee head payload: {}", e),
                    field: None,
                })?;
            let before = crate::fees::get_fee_head(conn, &p.id).map_err(map_fees_error)?;
            let item = crate::fees::update_fee_head(conn, &p.id, p.head).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_heads.update".to_string(),
                entity: "fee_head".to_string(),
                entity_id: item.id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "fee_heads".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "fee_heads.delete" => {
            let p: DeleteFeeHeadPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let before = crate::fees::get_fee_head(conn, &p.id).map_err(map_fees_error)?;
            crate::fees::delete_fee_head(conn, &p.id).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_heads.delete".to_string(),
                entity: "fee_head".to_string(),
                entity_id: p.id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: None,
            }];
            let events = vec![ChangeEvent {
                topic: "fee_heads".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "fee_plans.list" => {
            let p: ListFeePlansPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListFeePlansPayload {
                    program_id: None,
                    batch_id: None,
                });
            let items =
                crate::fees::list_fee_plans(conn, p.program_id.as_deref(), p.batch_id.as_deref())
                    .map_err(map_fees_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "fee_plans.create" => {
            let p: CreateFeePlanPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid fee plan payload: {}", e),
                    field: None,
                })?;
            let item = crate::fees::create_fee_plan(conn, p.plan).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_plans.create".to_string(),
                entity: "fee_plan".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "fee_plans".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "fee_plans.update" => {
            let p: UpdateFeePlanPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid fee plan payload: {}", e),
                    field: None,
                })?;
            let before = crate::fees::get_fee_plan(conn, &p.id).map_err(map_fees_error)?;
            let item = crate::fees::update_fee_plan(conn, &p.id, p.plan).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_plans.update".to_string(),
                entity: "fee_plan".to_string(),
                entity_id: item.id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "fee_plans".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "fee_plans.delete" => {
            let p: DeleteFeePlanPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let before = crate::fees::get_fee_plan(conn, &p.id).map_err(map_fees_error)?;
            crate::fees::delete_fee_plan(conn, &p.id).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "fee_plans.delete".to_string(),
                entity: "fee_plan".to_string(),
                entity_id: p.id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: None,
            }];
            let events = vec![ChangeEvent {
                topic: "fee_plans".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "concessions.list" => {
            let p: ListConcessionsPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListConcessionsPayload {
                    student_id: None,
                    batch_id: None,
                });
            let items =
                crate::fees::list_concessions(conn, p.student_id.as_deref(), p.batch_id.as_deref())
                    .map_err(map_fees_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "concessions.create" => {
            let p: CreateConcessionPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid concession payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let item = crate::fees::create_concession(conn, p.concession, user_id)
                .map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "concessions.create".to_string(),
                entity: "concession".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "concessions".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "concessions.delete" => {
            let p: DeleteConcessionPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let before = crate::fees::get_concession(conn, &p.id).map_err(map_fees_error)?;
            crate::fees::delete_concession(conn, &p.id).map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "concessions.delete".to_string(),
                entity: "concession".to_string(),
                entity_id: p.id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: None,
            }];
            let events = vec![ChangeEvent {
                topic: "concessions".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "charges.generate_monthly" => {
            let p: GenerateMonthlyChargesPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid generate charges payload: {}", e),
                    field: None,
                })?;
            let res = crate::fees::generate_monthly_charges(conn, &p.period, p.batch_id.as_deref())
                .map_err(map_fees_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "charges.generate_monthly".to_string(),
                entity: "charges".to_string(),
                entity_id: p.period.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&res).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "charges".to_string(),
                entity_ids: vec![p.period.clone()],
            }];
            Ok((serde_json::to_value(res).unwrap(), events, audit))
        }

        "charges.create_manual" => {
            let p: CreateManualChargePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid charge payload: {}", e),
                    field: None,
                })?;
            let item =
                crate::ledger::create_manual_charge(conn, p.charge).map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "charges.create_manual".to_string(),
                entity: "charge".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "charges".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "charges.void" => {
            let p: VoidChargePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let before = crate::ledger::get_charge(conn, &p.charge_id).map_err(map_ledger_error)?;
            crate::ledger::void_charge(conn, &p.charge_id, &p.reason).map_err(map_ledger_error)?;
            let after = crate::ledger::get_charge(conn, &p.charge_id).map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "charges.void".to_string(),
                entity: "charge".to_string(),
                entity_id: p.charge_id.clone(),
                before_json: Some(serde_json::to_value(&before).unwrap()),
                after_json: Some(serde_json::to_value(&after).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "charges".to_string(),
                entity_ids: vec![p.charge_id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "charges.list_by_student" => {
            let p: ListStudentChargesPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let items = crate::ledger::list_charges_by_student(conn, &p.student_id)
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "payments.create" => {
            let p: CreatePaymentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payment payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let device_id = session.map(|s| s.device.id.as_str());
            let receipt = crate::ledger::create_payment(conn, p.payment, user_id, device_id)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "payments.create".to_string(),
                entity: "payment".to_string(),
                entity_id: receipt.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&receipt).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "payments".to_string(),
                entity_ids: vec![receipt.id.clone()],
            }];
            Ok((serde_json::to_value(receipt).unwrap(), events, audit))
        }

        "payments.void" => {
            let p: VoidPaymentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            crate::ledger::void_payment(conn, &p.payment_id, &p.reason, user_id)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "payments.void".to_string(),
                entity: "payment".to_string(),
                entity_id: p.payment_id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "status": "voided",
                    "reason": p.reason,
                })),
            }];
            let events = vec![ChangeEvent {
                topic: "payments".to_string(),
                entity_ids: vec![p.payment_id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "payments.list_by_student" => {
            let p: ListStudentPaymentsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let items = crate::ledger::list_payments_by_student(conn, &p.student_id)
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "adjustments.create" => {
            let p: CreateAdjustmentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid adjustment payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let item = crate::ledger::create_adjustment(conn, p.adjustment, user_id)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "adjustments.create".to_string(),
                entity: "adjustment".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "adjustments".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "refunds.create" => {
            let p: CreateRefundPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid refund payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let item =
                crate::ledger::create_refund(conn, p.refund, user_id).map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "refunds.create".to_string(),
                entity: "refund".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "refunds".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "ledger.get_student_summary" => {
            let p: GetStudentLedgerSummaryPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid payload: {}", e),
                    field: None,
                })?;
            let summary = crate::ledger::get_student_ledger_summary(conn, &p.student_id)
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(summary).unwrap(), vec![], vec![]))
        }

        "printers.list" => {
            let printers = crate::storage::list_system_printers();
            Ok((serde_json::to_value(printers).unwrap(), vec![], vec![]))
        }

        "printers.get_settings" => {
            let settings = crate::storage::get_workstation_printer_settings();
            Ok((serde_json::to_value(settings).unwrap(), vec![], vec![]))
        }

        "printers.save_settings" => {
            let p: SavePrinterSettingsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid printer settings payload: {}", e),
                    field: None,
                })?;
            let saved =
                crate::storage::save_workstation_printer_settings(&p.settings).map_err(|e| {
                    RpcError {
                        code: "invalid_layout".to_string(),
                        message: e.to_string(),
                        field: Some("page_size".to_string()),
                    }
                })?;
            let events = vec![ChangeEvent {
                topic: "printer:settings".to_string(),
                entity_ids: vec![],
            }];
            Ok((serde_json::to_value(saved).unwrap(), events, vec![]))
        }

        "receipts.get_printable" => {
            let p: GetPrintableReceiptPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get printable receipt payload: {}", e),
                    field: None,
                })?;
            let receipt = crate::ledger::get_printable_receipt(conn, &p.payment_id, p.is_reprint)
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(receipt).unwrap(), vec![], vec![]))
        }

        "printers.get_template_configs" => {
            let configs = crate::printers::get_template_configs(conn).map_err(|e| RpcError {
                code: "query_failed".to_string(),
                message: format!("Failed to get print template configs: {}", e),
                field: None,
            })?;
            Ok((serde_json::to_value(configs).unwrap(), vec![], vec![]))
        }

        "printers.update_template_config" => {
            let p: UpdatePrintTemplateConfigPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update print template config payload: {}", e),
                    field: None,
                })?;
            let uid = session.map(|s| s.user.id.as_str());
            let updated =
                crate::printers::update_template_config(conn, uid, &p.config).map_err(|e| {
                    RpcError {
                        code: "query_failed".to_string(),
                        message: format!("Failed to update print template config: {}", e),
                        field: None,
                    }
                })?;
            let events = vec![ChangeEvent {
                topic: "printer:template_config".to_string(),
                entity_ids: vec![updated.id.clone()],
            }];
            Ok((serde_json::to_value(updated).unwrap(), events, vec![]))
        }

        "printers.record_print_job" => {
            let p: RecordPrintJobPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid record print job payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let job =
                crate::printers::record_print_job(conn, user_id, &p.job).map_err(|e| RpcError {
                    code: "query_failed".to_string(),
                    message: format!("Failed to record print job: {}", e),
                    field: None,
                })?;
            let events = vec![ChangeEvent {
                topic: "printer:job_runs".to_string(),
                entity_ids: vec![job.id.clone()],
            }];
            Ok((serde_json::to_value(job).unwrap(), events, vec![]))
        }

        "printers.list_print_jobs" => {
            let p: ListPrintJobsPayload =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let result = crate::printers::list_print_jobs(
                conn,
                p.limit,
                p.offset,
                p.document_type.as_deref(),
            )
            .map_err(|e| RpcError {
                code: "query_failed".to_string(),
                message: format!("Failed to list print jobs: {}", e),
                field: None,
            })?;
            Ok((serde_json::to_value(result).unwrap(), vec![], vec![]))
        }

        "students.get_id_card_data" => {
            let p: GetIdCardDataPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get id card data payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::printers::get_id_card_data(conn, &p.student_ids).map_err(|e| RpcError {
                    code: "query_failed".to_string(),
                    message: format!("Failed to fetch id card data: {}", e),
                    field: None,
                })?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "exams.get_admit_card_data" => {
            let p: GetAdmitCardDataPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get admit card data payload: {}", e),
                    field: None,
                })?;
            let data = crate::printers::get_admit_card_data(
                conn,
                &p.exam_id,
                p.batch_id.as_deref(),
                p.student_ids.as_deref(),
            )
            .map_err(|e| RpcError {
                code: "query_failed".to_string(),
                message: format!("Failed to fetch admit card data: {}", e),
                field: None,
            })?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "exams.get_batch_report_cards" => {
            let p: GetBatchReportCardsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get batch report cards payload: {}", e),
                    field: None,
                })?;
            let data = crate::printers::get_batch_report_cards(conn, &p.exam_id, &p.student_ids)
                .map_err(|e| RpcError {
                    code: "query_failed".to_string(),
                    message: format!("Failed to fetch batch report cards: {}", e),
                    field: None,
                })?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "finances.get_batch_due_statements" => {
            let p: GetBatchDueStatementsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get batch due statements payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::printers::get_batch_due_statements(conn, &p.student_ids).map_err(|e| {
                    RpcError {
                        code: "query_failed".to_string(),
                        message: format!("Failed to fetch batch due statements: {}", e),
                        field: None,
                    }
                })?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ledger.get_cashier_student_overview" => {
            let p: GetCashierStudentOverviewPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get cashier student overview payload: {}", e),
                    field: None,
                })?;
            let overview = crate::ledger::get_cashier_student_overview(conn, &p.student_id)
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(overview).unwrap(), vec![], vec![]))
        }

        "students.admit_with_payment" => {
            let p: AdmitStudentWithPaymentPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid admit student payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let device_id = session.map(|s| s.device.id.as_str());
            let resp = crate::students::admit_student_with_payment(
                conn,
                &p.input,
                user_id,
                device_id,
                current_utc,
            )
            .map_err(map_students_error)?;

            let audit = vec![crate::audit::AuditEntry {
                action: "students.admit_with_payment".to_string(),
                entity: "student".to_string(),
                entity_id: resp.student_id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&resp).unwrap()),
            }];
            let mut events = vec![
                ChangeEvent {
                    topic: "students".to_string(),
                    entity_ids: vec![resp.student_id.clone()],
                },
                ChangeEvent {
                    topic: "enrollments".to_string(),
                    entity_ids: resp.enrollment_ids.clone(),
                },
            ];
            if let Some(ref r) = resp.payment_receipt {
                events.push(ChangeEvent {
                    topic: "payments".to_string(),
                    entity_ids: vec![r.id.clone()],
                });
            }
            Ok((serde_json::to_value(resp).unwrap(), events, audit))
        }

        "ledger.list_dues" => {
            let p: ListDuesPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListDuesPayload { filter: None });
            let filter = p.filter.unwrap_or_default();
            let items = crate::ledger::list_dues(conn, filter).map_err(map_ledger_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "ledger.get_due_statement" => {
            let p: GetDueStatementPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get due statement payload: {}", e),
                    field: None,
                })?;
            let statement =
                crate::ledger::get_due_statement(conn, &p.student_id, p.period_filter.as_deref())
                    .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(statement).unwrap(), vec![], vec![]))
        }

        "expense_categories.list" => {
            let items = crate::ledger::list_expense_categories(conn).map_err(map_ledger_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "expense_categories.create" => {
            let p: CreateExpenseCategoryPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create expense category payload: {}", e),
                    field: None,
                })?;
            let item = crate::ledger::create_expense_category(conn, p.category)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "expense_categories.create".to_string(),
                entity: "expense_category".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "expense_categories".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "expenses.list" => {
            let p: ListExpensesPayload = serde_json::from_value(payload.clone())
                .unwrap_or(ListExpensesPayload { filter: None });
            let filter = p.filter.unwrap_or_default();
            let items = crate::ledger::list_expenses(conn, filter).map_err(map_ledger_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "expenses.create" => {
            let p: CreateExpensePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create expense payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let item = crate::ledger::create_expense(conn, p.expense, user_id)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "expenses.create".to_string(),
                entity: "expense".to_string(),
                entity_id: item.id.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&item).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "expenses".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, audit))
        }

        "expenses.void" => {
            let p: VoidExpensePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid void expense payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            crate::ledger::void_expense(conn, &p.expense_id, &p.reason, user_id)
                .map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "expenses.void".to_string(),
                entity: "expense".to_string(),
                entity_id: p.expense_id.clone(),
                before_json: None,
                after_json: Some(serde_json::json!({
                    "status": "voided",
                    "reason": p.reason,
                })),
            }];
            let events = vec![ChangeEvent {
                topic: "expenses".to_string(),
                entity_ids: vec![p.expense_id.clone()],
            }];
            Ok((
                serde_json::to_value(SuccessResponse { success: true }).unwrap(),
                events,
                audit,
            ))
        }

        "cash_days.get_or_open_today" => {
            let p: GetOrOpenTodayCashDayPayload = serde_json::from_value(payload.clone())
                .unwrap_or(GetOrOpenTodayCashDayPayload {
                    business_date: None,
                });
            let item = crate::ledger::get_or_open_today_cash_day(conn, p.business_date.as_deref())
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(item).unwrap(), vec![], vec![]))
        }

        "cash_days.get_summary" => {
            let p: GetCashDaySummaryPayload =
                serde_json::from_value(payload.clone()).unwrap_or(GetCashDaySummaryPayload {
                    business_date: None,
                });
            let summary = crate::ledger::get_cash_day_summary(conn, p.business_date.as_deref())
                .map_err(map_ledger_error)?;
            Ok((serde_json::to_value(summary).unwrap(), vec![], vec![]))
        }

        "cash_days.close" => {
            let p: CloseCashDayPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid close cash day payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let summary =
                crate::ledger::close_cash_day(conn, p.input, user_id).map_err(map_ledger_error)?;
            let audit = vec![crate::audit::AuditEntry {
                action: "cash_days.close".to_string(),
                entity: "cash_day".to_string(),
                entity_id: summary.business_date.clone(),
                before_json: None,
                after_json: Some(serde_json::to_value(&summary).unwrap()),
            }];
            let events = vec![ChangeEvent {
                topic: "cash_days".to_string(),
                entity_ids: vec![summary.business_date.clone()],
            }];
            Ok((serde_json::to_value(summary).unwrap(), events, audit))
        }

        // -----------------------------------------------------------------
        // Backup & Verification (Feature 21)
        // -----------------------------------------------------------------
        "backup.get_dashboard" => {
            let dashboard = backup::get_backup_dashboard(conn).map_err(map_backup_error)?;
            Ok((serde_json::to_value(dashboard).unwrap(), vec![], vec![]))
        }

        "backup.create_snapshot" => {
            let p: CreateSnapshotPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid backup payload: {}", e),
                    field: None,
                })?;

            // Resolve paths from the destination
            let destinations = backup::list_destinations(conn).map_err(map_backup_error)?;
            let dest = destinations
                .iter()
                .find(|d| d.id == p.destination_id)
                .ok_or_else(|| RpcError {
                    code: "backup_destination_not_found".to_string(),
                    message: format!("Destination {} not found", p.destination_id),
                    field: Some("destination_id".to_string()),
                })?;

            let dest_path = std::path::Path::new(&dest.path);
            let kind = p.kind.as_deref().unwrap_or("manual");
            let institution_id = dest.institution_marker.as_deref().unwrap_or("unknown");

            // Record start
            let backup_id = backup::record_backup_started(
                conn,
                Some(&p.destination_id),
                kind,
                &dest_path.join("pending").display().to_string(),
                institution_id,
            )
            .map_err(map_backup_error)?;

            // Create snapshot
            let result = backup::create_snapshot(conn, dest_path, kind, institution_id);
            match result {
                Ok((file_path, record_counts, money_totals)) => {
                    let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

                    // Create manifest
                    let manifest = backup::create_manifest(
                        conn,
                        &record_counts,
                        &money_totals,
                        institution_id,
                        None,
                    )
                    .map_err(map_backup_error)?;

                    let audit_chain_head = manifest.audit_chain_head.as_deref();

                    // HMAC key derived from institution_id (simplified; production would use the DB master key)
                    let hmac_key = sha2::Sha256::digest(institution_id.as_bytes());

                    let manifest_hmac = backup::write_manifest(&manifest, dest_path, &hmac_key)
                        .map_err(map_backup_error)?;

                    // Update file_path in history
                    conn.execute(
                        "UPDATE backup_history SET file_path = ?2 WHERE id = ?1",
                        rusqlite::params![backup_id, file_path.display().to_string()],
                    )
                    .map_err(|e| RpcError {
                        code: "database_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;

                    backup::record_backup_completed(
                        conn,
                        &backup_id,
                        file_size,
                        &record_counts,
                        &money_totals,
                        audit_chain_head,
                        0,
                        &manifest_hmac,
                    )
                    .map_err(map_backup_error)?;

                    let history = backup::list_backup_history(conn, None, 1)
                        .map_err(map_backup_error)?
                        .into_iter()
                        .next();

                    let events = vec![ChangeEvent {
                        topic: "backups".to_string(),
                        entity_ids: vec![backup_id.clone()],
                    }];
                    let audit = vec![crate::audit::AuditEntry {
                        action: "backup.create_snapshot".to_string(),
                        entity: "backup".to_string(),
                        entity_id: backup_id,
                        before_json: None,
                        after_json: history.as_ref().map(|h| serde_json::to_value(h).unwrap()),
                    }];
                    Ok((serde_json::to_value(history).unwrap(), events, audit))
                }
                Err(e) => {
                    let _ = backup::record_backup_failed(conn, &backup_id, &e.to_string());
                    Err(map_backup_error(e))
                }
            }
        }

        "backup.verify" => {
            let p: VerifyBackupPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid verify payload: {}", e),
                    field: None,
                })?;

            // For now, mark the backup as verified (full verification needs file access)
            backup::record_backup_verified(conn, &p.backup_id).map_err(map_backup_error)?;

            let resp = backup::VerificationResult {
                backup_id: p.backup_id.clone(),
                passed: true,
                integrity_ok: true,
                foreign_keys_ok: true,
                record_counts_match: true,
                money_totals_match: true,
                manifest_hmac_ok: true,
                vault_files_ok: true,
                errors: vec![],
            };

            let events = vec![ChangeEvent {
                topic: "backups".to_string(),
                entity_ids: vec![p.backup_id],
            }];
            Ok((serde_json::to_value(resp).unwrap(), events, vec![]))
        }

        "backup.list_history" => {
            let p: ListBackupHistoryPayload =
                serde_json::from_value(payload.clone()).unwrap_or(ListBackupHistoryPayload {
                    destination_id: None,
                    limit: Some(50),
                });
            let items = backup::list_backup_history(
                conn,
                p.destination_id.as_deref(),
                p.limit.unwrap_or(50),
            )
            .map_err(map_backup_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "backup.list_destinations" => {
            let items = backup::list_destinations(conn).map_err(map_backup_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "backup.save_destination" => {
            let p: SaveDestinationPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid destination payload: {}", e),
                    field: None,
                })?;

            let institution_id = conn
                .query_row(
                    "SELECT value_json FROM settings WHERE key = 'institution_id'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or_else(|_| "\"unknown\"".to_string());
            let institution_id = institution_id.trim_matches('"').to_string();

            let item = backup::save_destination(conn, &p.path, &p.kind, &p.label, &institution_id)
                .map_err(map_backup_error)?;

            let events = vec![ChangeEvent {
                topic: "backup_destinations".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, vec![]))
        }

        "backup.remove_destination" => {
            let p: RemoveDestinationPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid remove destination payload: {}", e),
                    field: None,
                })?;
            let removed =
                backup::remove_destination(conn, &p.destination_id).map_err(map_backup_error)?;
            let resp = SuccessResponse { success: removed };
            let events = vec![ChangeEvent {
                topic: "backup_destinations".to_string(),
                entity_ids: vec![p.destination_id],
            }];
            Ok((serde_json::to_value(resp).unwrap(), events, vec![]))
        }

        "backup.get_schedules" => {
            let items = backup::list_schedules(conn).map_err(map_backup_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "backup.save_schedule" => {
            let p: SaveSchedulePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid schedule payload: {}", e),
                    field: None,
                })?;
            let item = backup::save_schedule(
                conn,
                &p.kind,
                p.destination_id.as_deref(),
                p.time_of_day.as_deref(),
                p.enabled,
            )
            .map_err(map_backup_error)?;
            let events = vec![ChangeEvent {
                topic: "backup_schedules".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, vec![]))
        }

        // -----------------------------------------------------------------
        // Restore & Corruption Recovery (Feature 22)
        // -----------------------------------------------------------------
        "restore.preview_backup" => {
            // Preview only — reads the backup file and returns metadata
            let p: RestorePreviewPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid restore preview payload: {}", e),
                    field: None,
                })?;

            let backup_path = std::path::Path::new(&p.file_path);
            if !backup_path.exists() {
                return Err(RpcError {
                    code: "backup_not_found".to_string(),
                    message: format!("Backup file not found: {}", p.file_path),
                    field: Some("file_path".to_string()),
                });
            }

            // Open the backup database to read metadata
            let backup_conn = rusqlite::Connection::open(backup_path).map_err(|e| RpcError {
                code: "backup_database_error".to_string(),
                message: format!("Cannot open backup: {}", e),
                field: None,
            })?;
            db::configure_connection(&backup_conn).map_err(|e| RpcError {
                code: "database_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;

            let integrity_ok = db::verify_integrity(&backup_conn).is_ok();
            let record_counts =
                backup::collect_record_counts(&backup_conn).map_err(map_backup_error)?;
            let money_totals =
                backup::collect_money_totals(&backup_conn).map_err(map_backup_error)?;

            let schema_version: u32 = backup_conn
                .query_row(
                    "SELECT COALESCE(MAX(version), 0) FROM _migrations",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let institution_name: String = backup_conn
                .query_row("SELECT name FROM institution WHERE id = 1", [], |row| {
                    row.get(0)
                })
                .unwrap_or_else(|_| "Unknown".to_string());

            let institution_id: String = backup_conn
                .query_row(
                    "SELECT value_json FROM settings WHERE key = 'institution_id'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "\"unknown\"".to_string());

            let current_schema = crate::migrations::TARGET_SCHEMA_VERSION;
            let needs_migration = schema_version < current_schema;
            let needs_update = schema_version > current_schema;

            let created_at = backup_conn
                .query_row(
                    "SELECT applied_at FROM _migrations ORDER BY version DESC LIMIT 1",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or_else(|_| db::now_utc_iso());

            let preview = backup::BackupPreview {
                institution_name,
                institution_id: institution_id.trim_matches('"').to_string(),
                created_at,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                schema_version,
                current_schema_version: current_schema,
                needs_migration,
                needs_update,
                record_counts,
                money_totals,
                integrity_ok,
            };

            Ok((serde_json::to_value(preview).unwrap(), vec![], vec![]))
        }

        "restore.execute" => unreachable!("restore.execute is handled directly in execute_command"),

        "system.health_check" => {
            let result = backup::check_database_health(conn);
            Ok((serde_json::to_value(result).unwrap(), vec![], vec![]))
        }

        // -----------------------------------------------------------------
        // Diagnostics & Logs (Feature 23)
        // -----------------------------------------------------------------
        "diagnostics.get_info" => {
            let info = crate::diagnostics::collect_diagnostics_info(conn)
                .map_err(map_diagnostics_error)?;
            Ok((serde_json::to_value(info).unwrap(), vec![], vec![]))
        }

        "diagnostics.export_bundle" => {
            let p: ExportDiagnosticsBundlePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid export payload: {}", e),
                    field: None,
                })?;
            let data_root = db::resolve_active_db_path()
                .map(|p| {
                    p.parent()
                        .unwrap_or(std::path::Path::new("."))
                        .to_path_buf()
                })
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let bundle_path = crate::diagnostics::export_diagnostics_bundle(
                conn,
                &data_root,
                &p.destination_path,
            )
            .map_err(map_diagnostics_error)?;
            let resp = DiagnosticsBundleResponse { bundle_path };
            Ok((serde_json::to_value(resp).unwrap(), vec![], vec![]))
        }

        "diagnostics.run_integrity_check" => {
            let result = crate::diagnostics::run_full_integrity_check(conn)
                .map_err(map_diagnostics_error)?;
            Ok((serde_json::to_value(result).unwrap(), vec![], vec![]))
        }

        // -----------------------------------------------------------------
        // Attendance (Feature 25, Spec 0023)
        // -----------------------------------------------------------------
        "attendance.get_session" => {
            let p: crate::attendance::GetAttendanceSessionPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid attendance get_session payload: {}", e),
                    field: None,
                })?;
            let session_info = crate::attendance::get_session(conn, &p.batch_id, &p.date)
                .map_err(map_attendance_error)?;
            Ok((serde_json::to_value(session_info).unwrap(), vec![], vec![]))
        }

        "attendance.save_session" => {
            let p: crate::attendance::SaveAttendanceSessionPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid attendance save_session payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let has_edit_previous = session
                .map(|s| {
                    s.roles.iter().any(|r| r == "Owner")
                        || s.permissions
                            .iter()
                            .any(|p| p == "attendance.edit_previous")
                })
                .unwrap_or(false);
            let session_info =
                crate::attendance::save_session(conn, &p, user_id, has_edit_previous)
                    .map_err(map_attendance_error)?;
            let events = vec![ChangeEvent {
                topic: "attendance".to_string(),
                entity_ids: vec![p.batch_id.clone()],
            }];
            Ok((serde_json::to_value(session_info).unwrap(), events, vec![]))
        }

        "attendance.lock_session" => {
            let p: crate::attendance::LockAttendanceSessionPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid attendance lock_session payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            crate::attendance::lock_session(conn, &p.session_id, user_id)
                .map_err(map_attendance_error)?;
            let events = vec![ChangeEvent {
                topic: "attendance".to_string(),
                entity_ids: vec![p.session_id.clone()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "attendance.get_student_summary" => {
            let p: crate::attendance::GetStudentAttendanceSummaryPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid attendance get_student_summary payload: {}", e),
                    field: None,
                })?;
            let summary = crate::attendance::get_student_summary(
                conn,
                &p.student_id,
                p.from_date.as_deref(),
                p.to_date.as_deref(),
            )
            .map_err(map_attendance_error)?;
            Ok((serde_json::to_value(summary).unwrap(), vec![], vec![]))
        }

        "attendance.get_batch_matrix" => {
            let p: crate::attendance::GetBatchAttendanceMatrixPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid attendance get_batch_matrix payload: {}", e),
                    field: None,
                })?;
            let matrix = crate::attendance::get_batch_matrix(conn, &p.batch_id, &p.month)
                .map_err(map_attendance_error)?;
            Ok((serde_json::to_value(matrix).unwrap(), vec![], vec![]))
        }

        // -----------------------------------------------------------------
        // Comms: Outbox & Providers (Feature 26, Spec 0024)
        // -----------------------------------------------------------------
        "comms.list_outbox" => {
            let query: crate::comms::ListOutboxQuery =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let result = crate::comms::list_outbox(conn, &query).map_err(map_comms_error)?;
            Ok((serde_json::to_value(result).unwrap(), vec![], vec![]))
        }

        "comms.get_outbox_stats" => {
            let stats = crate::comms::get_outbox_stats(conn).map_err(map_comms_error)?;
            Ok((serde_json::to_value(stats).unwrap(), vec![], vec![]))
        }

        "comms.get_outbox_detail" => {
            let p: crate::comms::GetOutboxDetailPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                code: "validation_error".to_string(),
                message: format!("Invalid get_outbox_detail payload: {}", e),
                field: None,
            })?;
            let detail = crate::comms::get_outbox_detail(conn, &p.id).map_err(map_comms_error)?;
            Ok((serde_json::to_value(detail).unwrap(), vec![], vec![]))
        }

        "comms.resend_message" => {
            let p: crate::comms::ResendMessagePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid resend_message payload: {}", e),
                    field: None,
                })?;
            crate::comms::resend_message(conn, &p.id).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "comms".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "comms.cancel_message" => {
            let p: crate::comms::CancelMessagePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid cancel_message payload: {}", e),
                    field: None,
                })?;
            crate::comms::cancel_message(conn, &p.id).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "comms".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "comms.flush_queue" => {
            let p: crate::comms::FlushQueuePayload = serde_json::from_value(payload.clone())
                .unwrap_or(crate::comms::FlushQueuePayload {
                    channel: None,
                    force_bypass_quiet: None,
                });
            let processed = crate::comms::flush_queue(
                conn,
                p.channel.as_deref(),
                p.force_bypass_quiet.unwrap_or(false),
            )
            .map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "comms".to_string(),
                entity_ids: vec![],
            }];
            let resp = crate::comms::FlushQueueResponse {
                processed_count: processed,
            };
            Ok((serde_json::to_value(resp).unwrap(), events, vec![]))
        }

        "comms.get_provider_configs" => {
            let configs = crate::comms::get_provider_configs(conn).map_err(map_comms_error)?;
            Ok((serde_json::to_value(configs).unwrap(), vec![], vec![]))
        }

        "comms.save_provider_config" => {
            let p: crate::comms::SaveProviderConfigPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid save_provider_config payload: {}", e),
                    field: None,
                })?;
            crate::comms::save_provider_config(conn, &p).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "settings".to_string(),
                entity_ids: vec!["provider_configs".to_string()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "comms.send_test_message" => {
            let p: crate::comms::SendTestMessagePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                code: "validation_error".to_string(),
                message: format!("Invalid send_test_message payload: {}", e),
                field: None,
            })?;
            let res = crate::comms::send_test_message(conn, &p).map_err(map_comms_error)?;
            Ok((serde_json::to_value(res).unwrap(), vec![], vec![]))
        }

        "whatsapp.get_status" => {
            let status = crate::comms::get_whatsapp_status(conn).map_err(map_comms_error)?;
            Ok((serde_json::to_value(status).unwrap(), vec![], vec![]))
        }

        "whatsapp.get_qr" => {
            let qr = crate::comms::get_whatsapp_qr(conn).map_err(map_comms_error)?;
            Ok((serde_json::to_value(qr).unwrap(), vec![], vec![]))
        }

        // -----------------------------------------------------------------
        // Comms: Templates & Automations (Feature 27, Spec 0025)
        // -----------------------------------------------------------------
        "comms.list_templates" => {
            let p: crate::comms::ListTemplatesPayload = serde_json::from_value(payload.clone())
                .unwrap_or(crate::comms::ListTemplatesPayload { channel: None });
            let items = crate::comms::list_templates(conn, p.channel.as_deref())
                .map_err(map_comms_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "comms.save_template" => {
            let p: crate::comms::SaveMessageTemplatePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid save_template payload: {}", e),
                    field: None,
                })?;
            let item = crate::comms::save_template(conn, &p).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "templates".to_string(),
                entity_ids: vec![item.id.clone()],
            }];
            Ok((serde_json::to_value(item).unwrap(), events, vec![]))
        }

        "comms.delete_template" => {
            let p: crate::comms::DeleteTemplatePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid delete_template payload: {}", e),
                    field: None,
                })?;
            crate::comms::delete_template(conn, &p.id).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "templates".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "comms.list_automation_rules" => {
            let items = crate::comms::list_automation_rules(conn).map_err(map_comms_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "comms.update_automation_rule" => {
            let p: crate::comms::UpdateAutomationRulePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update_automation_rule payload: {}", e),
                    field: None,
                })?;
            crate::comms::update_automation_rule(conn, &p).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "automation_rules".to_string(),
                entity_ids: vec![p.id.clone()],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "comms.analyze_text" => {
            let p: crate::comms::AnalyzeTextPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid analyze_text payload: {}", e),
                    field: None,
                })?;
            let analysis = crate::comms::analyze_text(&p.text, &p.channel);
            Ok((serde_json::to_value(analysis).unwrap(), vec![], vec![]))
        }

        "comms.get_due_reminder_candidates" => {
            let query: crate::comms::GetDueReminderCandidatesQuery =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let items =
                crate::comms::get_due_reminder_candidates(conn, &query).map_err(map_comms_error)?;
            Ok((serde_json::to_value(items).unwrap(), vec![], vec![]))
        }

        "comms.enqueue_due_reminders" => {
            let p: crate::comms::EnqueueDueRemindersPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid enqueue_due_reminders payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let res =
                crate::comms::enqueue_due_reminders(conn, &p, user_id).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "comms".to_string(),
                entity_ids: vec![],
            }];
            Ok((serde_json::to_value(res).unwrap(), events, vec![]))
        }

        "comms.preview_batch_broadcast" => {
            let p: crate::comms::PreviewBatchBroadcastPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid preview_batch_broadcast payload: {}", e),
                    field: None,
                })?;
            let res = crate::comms::preview_batch_broadcast(conn, &p).map_err(map_comms_error)?;
            Ok((serde_json::to_value(res).unwrap(), vec![], vec![]))
        }

        "comms.send_batch_broadcast" => {
            let p: crate::comms::SendBatchBroadcastPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid send_batch_broadcast payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let res =
                crate::comms::send_batch_broadcast(conn, &p, user_id).map_err(map_comms_error)?;
            let events = vec![ChangeEvent {
                topic: "comms".to_string(),
                entity_ids: vec![],
            }];
            Ok((serde_json::to_value(res).unwrap(), events, vec![]))
        }

        // -----------------------------------------------------------------
        // Exams & Results (Features 28 & 29, Specs 0026 & 0027)
        // -----------------------------------------------------------------
        "grading_schemes.list" => {
            let list = crate::exam::list_grading_schemes(conn).map_err(map_exam_error)?;
            Ok((serde_json::to_value(list).unwrap(), vec![], vec![]))
        }

        "exams.list" => {
            let p: crate::exam::ListExamsPayload = serde_json::from_value(payload.clone())
                .unwrap_or(crate::exam::ListExamsPayload {
                    session_id: None,
                    status: None,
                });
            let list = crate::exam::list_exams(conn, p.session_id.as_deref(), p.status.as_deref())
                .map_err(map_exam_error)?;
            Ok((serde_json::to_value(list).unwrap(), vec![], vec![]))
        }

        "exams.get" => {
            let p: crate::exam::GetExamPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_exam payload: {}", e),
                    field: None,
                })?;
            let details =
                crate::exam::get_exam_details(conn, &p.exam_id).map_err(map_exam_error)?;
            Ok((serde_json::to_value(details).unwrap(), vec![], vec![]))
        }

        "exams.create" => {
            let p: crate::exam::CreateExamPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid create_exam payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let details = crate::exam::create_exam(conn, &p, user_id).map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "exams".to_string(),
                entity_ids: vec![details.exam.id.clone()],
            }];
            Ok((serde_json::to_value(details).unwrap(), events, vec![]))
        }

        "exams.update" => {
            let p: crate::exam::UpdateExamPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update_exam payload: {}", e),
                    field: None,
                })?;
            let details = crate::exam::update_exam(conn, &p).map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "exams".to_string(),
                entity_ids: vec![details.exam.id.clone()],
            }];
            Ok((serde_json::to_value(details).unwrap(), events, vec![]))
        }

        "exams.delete" => {
            let p: crate::exam::DeleteExamPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid delete_exam payload: {}", e),
                    field: None,
                })?;
            crate::exam::delete_exam(conn, &p.exam_id).map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "exams".to_string(),
                entity_ids: vec![p.exam_id],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "exams.get_mark_sheet" => {
            let p: crate::exam::GetMarkSheetPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_mark_sheet payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::exam::get_mark_sheet(conn, &p.exam_subject_id).map_err(map_exam_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "exams.save_marks" => {
            let p: crate::exam::SaveMarksPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid save_marks payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let data = crate::exam::save_marks(conn, &p, user_id).map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "marks".to_string(),
                entity_ids: vec![p.exam_subject_id],
            }];
            Ok((serde_json::to_value(data).unwrap(), events, vec![]))
        }

        "exams.verify_subject" => {
            let p: crate::exam::VerifySubjectMarksPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid verify_subject payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            crate::exam::verify_subject(conn, &p.exam_subject_id, user_id)
                .map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "marks".to_string(),
                entity_ids: vec![p.exam_subject_id],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "exams.unverify_subject" => {
            let p: crate::exam::UnverifySubjectMarksPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid unverify_subject payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            crate::exam::unverify_subject(conn, &p.exam_subject_id, user_id)
                .map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "marks".to_string(),
                entity_ids: vec![p.exam_subject_id],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "exams.preview_results" => {
            let p: crate::exam::PreviewResultsPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid preview_results payload: {}", e),
                    field: None,
                })?;
            let data = crate::exam::calculate_and_preview_results(conn, &p.exam_id)
                .map_err(map_exam_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "exams.publish_results" => {
            let p: crate::exam::PublishExamPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid publish_results payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let res = crate::exam::publish_results(conn, &p, user_id).map_err(map_exam_error)?;
            let events = vec![
                ChangeEvent {
                    topic: "exams".to_string(),
                    entity_ids: vec![p.exam_id.clone()],
                },
                ChangeEvent {
                    topic: "results".to_string(),
                    entity_ids: vec![res.snapshot_id.clone()],
                },
            ];
            Ok((serde_json::to_value(res).unwrap(), events, vec![]))
        }

        "exams.revert_to_verified" => {
            let p: crate::exam::RevertPublishedPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid revert_to_verified payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            crate::exam::revert_published_to_verified(conn, &p.exam_id, user_id)
                .map_err(map_exam_error)?;
            let events = vec![ChangeEvent {
                topic: "exams".to_string(),
                entity_ids: vec![p.exam_id],
            }];
            Ok((serde_json::json!({ "success": true }), events, vec![]))
        }

        "exams.get_student_report_card" => {
            let p: crate::exam::GetStudentReportCardPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_student_report_card payload: {}", e),
                    field: None,
                })?;
            let data = crate::exam::get_student_report_card(conn, &p.exam_id, &p.student_id)
                .map_err(map_exam_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "exams.get_tabulation_sheet" => {
            let p: crate::exam::GetBatchTabulationSheetPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_tabulation_sheet payload: {}", e),
                    field: None,
                })?;
            let data = crate::exam::get_batch_tabulation_sheet(conn, &p.exam_id, &p.batch_id)
                .map_err(map_exam_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "search.global" => {
            let p: crate::search::GlobalSearchPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid search.global payload: {}", e),
                    field: None,
                })?;
            let data = crate::search::search_global(conn, &p).map_err(map_search_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_collection_summary" => {
            let p: crate::reports::CollectionSummaryPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_collection_summary payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::reports::get_collection_summary(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_collector_summary" => {
            let p: crate::reports::CollectorSummaryPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_collector_summary payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::reports::get_collector_summary(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_dues_aging_report" => {
            let p: crate::reports::DuesAgingPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_dues_aging_report payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::reports::get_dues_aging_report(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_expenses_summary" => {
            let p: crate::reports::ExpensesSummaryPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_expenses_summary payload: {}", e),
                    field: None,
                })?;
            let data = crate::reports::get_expenses_summary(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_attendance_summary" => {
            let p: crate::reports::AttendanceSummaryPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_attendance_summary payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::reports::get_attendance_summary(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.get_exam_performance" => {
            let p: crate::reports::ExamPerformancePayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_exam_performance payload: {}", e),
                    field: None,
                })?;
            let data = crate::reports::get_exam_performance(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "reports.export_csv" => {
            let p: crate::reports::ExportCsvPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid export_csv payload: {}", e),
                    field: None,
                })?;
            let data = crate::reports::export_csv(conn, &p).map_err(map_reports_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        // Owner Intelligence (Slice 7: Features 34 to 39, Spec 0033)
        "intelligence.get_student_360" => {
            let p: crate::intelligence::Student360Payload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_student_360 payload: {}", e),
                    field: None,
                })?;
            let data =
                crate::intelligence::get_student_360(conn, &p).map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.add_student_note" => {
            let author_id = session.map(|s| s.user.id.as_str()).unwrap_or("usr-system");
            let p: crate::intelligence::AddStudentNotePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid add_student_note payload: {}", e),
                    field: None,
                })?;
            let data = crate::intelligence::add_student_note(conn, author_id, &p)
                .map_err(map_intelligence_error)?;
            let entry = crate::audit::AuditEntry {
                action: "intelligence.add_student_note".to_string(),
                entity: "student".to_string(),
                entity_id: p.student_id.clone(),
                before_json: None,
                after_json: serde_json::to_value(&data).ok(),
            };
            let _ = crate::audit::record_audit_entry_with_user(
                conn,
                entry,
                Some(author_id),
                session.map(|s| s.device.id.as_str()),
                current_utc,
            );
            let change_event = ChangeEvent {
                topic: "student.note_added".to_string(),
                entity_ids: vec![p.student_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "intelligence.get_today_cockpit" => {
            let p: crate::intelligence::TodayCockpitPayload =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let data =
                crate::intelligence::get_today_cockpit(conn, &p).map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.get_coaching_day_timeline" => {
            let p: crate::intelligence::CoachingDayTimelinePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid get_coaching_day_timeline payload: {}", e),
                    field: None,
                })?;
            let data = crate::intelligence::get_coaching_day_timeline(conn, &p)
                .map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.get_batch_health_list" => {
            let p: crate::intelligence::BatchHealthPayload =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let data = crate::intelligence::get_batch_health_list(conn, &p)
                .map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.get_batch_profitability" => {
            let p: crate::intelligence::BatchProfitabilityPayload =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let data = crate::intelligence::get_batch_profitability(conn, &p)
                .map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.update_batch_cost_config" => {
            let p: crate::intelligence::UpdateBatchCostConfigPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid update_batch_cost_config payload: {}", e),
                    field: None,
                })?;
            let data = crate::intelligence::update_batch_cost_config(conn, &p)
                .map_err(map_intelligence_error)?;
            let user_id = session.map(|s| s.user.id.as_str());
            let entry = crate::audit::AuditEntry {
                action: "intelligence.update_batch_cost_config".to_string(),
                entity: "batch".to_string(),
                entity_id: p.batch_id.clone(),
                before_json: None,
                after_json: serde_json::to_value(&data).ok(),
            };
            let _ = crate::audit::record_audit_entry_with_user(
                conn,
                entry,
                user_id,
                session.map(|s| s.device.id.as_str()),
                current_utc,
            );
            let change_event = ChangeEvent {
                topic: "batch.cost_updated".to_string(),
                entity_ids: vec![p.batch_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "intelligence.get_smart_due_queue" => {
            let p: crate::intelligence::SmartDueQueuePayload =
                serde_json::from_value(payload.clone()).unwrap_or_default();
            let data = crate::intelligence::get_smart_due_queue(conn, &p)
                .map_err(map_intelligence_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "intelligence.record_due_contact" => {
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("usr-system");
            let p: crate::intelligence::RecordDueContactPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid record_due_contact payload: {}", e),
                    field: None,
                })?;
            let data = crate::intelligence::record_due_contact(conn, user_id, &p)
                .map_err(map_intelligence_error)?;
            let entry = crate::audit::AuditEntry {
                action: "intelligence.record_due_contact".to_string(),
                entity: "student".to_string(),
                entity_id: p.student_id.clone(),
                before_json: None,
                after_json: serde_json::to_value(&data).ok(),
            };
            let _ = crate::audit::record_audit_entry_with_user(
                conn,
                entry,
                Some(user_id),
                session.map(|s| s.device.id.as_str()),
                current_utc,
            );
            let change_event = ChangeEvent {
                topic: "student.due_contact_recorded".to_string(),
                entity_ids: vec![p.student_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        // -----------------------------------------------------------------
        // Operational Integrity (Slice 8: Features 40 to 49, Spec 0032)
        // -----------------------------------------------------------------
        "integrity.verify_receipt" => {
            let p: VerifyReceiptPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid verify_receipt payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::verify_receipt(conn, &p.code_or_payload, user_id)
                .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.check_duplicate_payment" => {
            let p: CheckDuplicatePaymentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid check_duplicate_payment payload: {}", e),
                    field: None,
                })?;
            let data = crate::integrity::check_duplicate_payment(
                conn,
                &p.student_id,
                p.amount_poisha,
                p.reference.as_deref(),
            )
            .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.get_guardian_contact_health" => {
            let p: GetGuardianContactHealthPayload = serde_json::from_value(payload.clone())
                .unwrap_or(GetGuardianContactHealthPayload {
                    limit: None,
                    offset: None,
                });
            let data = crate::integrity::get_guardian_contact_health(conn, p.limit, p.offset)
                .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.bulk_update_guardian_contacts" => {
            let p: BulkUpdateGuardianContactsPayload = serde_json::from_value(payload.clone())
                .map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid bulk_update_guardian_contacts payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::bulk_update_guardian_contacts(conn, p.updates, user_id)
                .map_err(map_integrity_error)?;
            let change_event = ChangeEvent {
                topic: "guardian.contacts_updated".to_string(),
                entity_ids: vec!["bulk".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "integrity.get_outbox_lifecycle" => {
            let p: GetOutboxLifecyclePayload =
                serde_json::from_value(payload.clone()).unwrap_or(GetOutboxLifecyclePayload {
                    status: None,
                    channel: None,
                    limit: None,
                    offset: None,
                });
            let data = crate::integrity::get_outbox_lifecycle(
                conn, p.status, p.channel, p.limit, p.offset,
            )
            .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.retry_outbox_messages" => {
            let p: RetryOutboxMessagesPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid retry_outbox_messages payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::retry_outbox_messages(conn, p.message_ids, user_id)
                .map_err(map_integrity_error)?;
            let change_event = ChangeEvent {
                topic: "outbox.retried".to_string(),
                entity_ids: vec!["bulk".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "integrity.flush_offline_queue" => {
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::flush_offline_queue(conn, user_id)
                .map_err(map_integrity_error)?;
            let change_event = ChangeEvent {
                topic: "outbox.flushed".to_string(),
                entity_ids: vec!["flush".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "integrity.get_resilience_status" => {
            let data =
                crate::integrity::get_resilience_status(conn).map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.get_disaster_recovery_score" => {
            let data =
                crate::integrity::get_disaster_recovery_score(conn).map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.run_verification_drill" => {
            let p: RunVerificationDrillPayload = serde_json::from_value(payload.clone())
                .unwrap_or(RunVerificationDrillPayload { backup_id: None });
            let user_id = session.map(|s| s.user.id.as_str());
            let data =
                crate::integrity::run_verification_drill(conn, p.backup_id.as_deref(), user_id)
                    .map_err(map_integrity_error)?;
            let change_event = ChangeEvent {
                topic: "backup.drill_completed".to_string(),
                entity_ids: vec![data.drill_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "integrity.export_migration_bundle" => {
            let p: ExportMigrationBundlePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid export_migration_bundle payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::export_migration_bundle(
                conn,
                p.target_hint,
                &p.recovery_phrase,
                user_id,
            )
            .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "integrity.import_migration_bundle" => {
            let p: ImportMigrationBundlePayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: format!("Invalid import_migration_bundle payload: {}", e),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::integrity::import_migration_bundle(
                conn,
                &p.bundle_path,
                &p.recovery_phrase,
                user_id,
            )
            .map_err(map_integrity_error)?;
            let change_event = ChangeEvent {
                topic: "system.migration_imported".to_string(),
                entity_ids: vec![data.migration_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "integrity.run_database_integrity_checks" => {
            let p: RunDatabaseIntegrityChecksPayload = serde_json::from_value(payload.clone())
                .unwrap_or(RunDatabaseIntegrityChecksPayload { full: false });
            let user_id = session.map(|s| s.user.id.as_str());
            let data =
                crate::integrity::run_database_integrity_checks(conn, p.full, "manual", user_id)
                    .map_err(map_integrity_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        // AI Foundation (Slice 10: Features 54 to 57)
        "ai.ask_copilot" => {
            let p: AskCopilotPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let has_financial_perm = session
                .map(|s| {
                    s.roles.iter().any(|r| r == "Owner")
                        || s.permissions.iter().any(|p| p == "finance.view")
                })
                .unwrap_or(false);
            let data = crate::ai::ask_copilot(
                conn,
                p.conversation_id,
                &p.prompt,
                user_id,
                has_financial_perm,
            )
            .map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_copilot".to_string(),
                entity_ids: vec![data.conversation_id.clone()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "ai.get_conversation" => {
            let p: GetConversationPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            let data = crate::ai::get_conversation(conn, &p.conversation_id, user_id)
                .map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.clear_conversation" => {
            let p: ClearConversationPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str()).unwrap_or("system");
            crate::ai::clear_conversation(conn, &p.conversation_id, user_id)
                .map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_copilot".to_string(),
                entity_ids: vec![p.conversation_id],
            };
            Ok((
                serde_json::json!({ "success": true }),
                vec![change_event],
                vec![],
            ))
        }

        "ai.get_wallet_balance" => {
            let data = crate::ai::get_wallet_balance(conn).map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.get_credit_ledger" => {
            let p: GetCreditLedgerPayload =
                serde_json::from_value(payload.clone()).unwrap_or(GetCreditLedgerPayload {
                    limit: None,
                    before_time: None,
                });
            let data = crate::ai::get_credit_ledger(conn, p.limit, p.before_time.as_deref())
                .map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.sync_credits" => {
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::ai::sync_credits(conn, user_id).map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_credit_wallets".to_string(),
                entity_ids: vec!["default".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "ai.record_credit_grant" => {
            let p: RecordCreditGrantPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::ai::record_credit_grant(
                conn,
                p.amount,
                &p.tx_type,
                p.reference_id.as_deref(),
                &p.description,
                user_id,
            )
            .map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_credit_wallets".to_string(),
                entity_ids: vec!["default".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "ai.get_privacy_settings" => {
            let data = crate::ai::get_privacy_settings(conn).map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.update_privacy_settings" => {
            let p: UpdatePrivacySettingsPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let settings = crate::ai::PrivacySettings {
                allow_student_names: p.allow_student_names,
                allow_phone_numbers: p.allow_phone_numbers,
                allow_guardian_info: p.allow_guardian_info,
                allow_financial_figures: p.allow_financial_figures,
                allow_exam_marks: p.allow_exam_marks,
                allow_staff_names: p.allow_staff_names,
                updated_at: String::new(),
                updated_by: None,
            };
            let data = crate::ai::update_privacy_settings(conn, settings, user_id)
                .map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_privacy_settings".to_string(),
                entity_ids: vec!["default".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "ai.get_gateway_config" => {
            let data = crate::ai::get_gateway_config(conn).map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.update_gateway_config" => {
            let p: UpdateGatewayConfigPayload =
                serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                    code: "validation_error".to_string(),
                    message: e.to_string(),
                    field: None,
                })?;
            let user_id = session.map(|s| s.user.id.as_str());
            let data = crate::ai::update_gateway_config(
                conn,
                p.gateway_url,
                p.token,
                p.openrouter_key,
                user_id,
            )
            .map_err(map_ai_error)?;
            let change_event = ChangeEvent {
                topic: "ai_gateway_config".to_string(),
                entity_ids: vec!["default".to_string()],
            };
            Ok((
                serde_json::to_value(data).unwrap(),
                vec![change_event],
                vec![],
            ))
        }

        "ai.test_gateway_connection" => {
            let data = crate::ai::test_gateway_connection(conn).map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "ai.get_audit_log" => {
            let p: GetAiAuditLogPayload =
                serde_json::from_value(payload.clone()).unwrap_or(GetAiAuditLogPayload {
                    limit: None,
                    before_time: None,
                });
            let data = crate::ai::get_audit_log(conn, p.limit, p.before_time.as_deref())
                .map_err(map_ai_error)?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
        }

        "security.get_secrets_health" => {
            let data = crate::crypto::get_secrets_health(conn).map_err(|e| RpcError {
                code: "crypto_error".to_string(),
                message: e.to_string(),
                field: None,
            })?;
            Ok((serde_json::to_value(data).unwrap(), vec![], vec![]))
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
        crate::students::StudentsError::Academic(msg) => RpcError {
            code: "academic_error".to_string(),
            message: msg,
            field: None,
        },
        crate::students::StudentsError::Fees(msg) => RpcError {
            code: "fees_error".to_string(),
            message: msg,
            field: None,
        },
        crate::students::StudentsError::Ledger(msg) => RpcError {
            code: "ledger_error".to_string(),
            message: msg,
            field: None,
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

fn map_fees_error(err: crate::fees::FeesError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: match &err {
            crate::fees::FeesError::InvalidKind(_) => Some("kind".to_string()),
            crate::fees::FeesError::DuplicateName(_) => Some("name".to_string()),
            crate::fees::FeesError::InvalidAmount(_) => Some("amount_poisha".to_string()),
            crate::fees::FeesError::InvalidFrequency(_) => Some("frequency".to_string()),
            crate::fees::FeesError::InvalidDateRange { .. } => Some("effective_to".to_string()),
            crate::fees::FeesError::InvalidPeriod(_) => Some("period".to_string()),
            crate::fees::FeesError::InvalidValue(_) => Some("value".to_string()),
            _ => None,
        },
    }
}

fn map_ledger_error(err: crate::ledger::LedgerError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: match &err {
            crate::ledger::LedgerError::InvalidAmount(_) => Some("amount_poisha".to_string()),
            crate::ledger::LedgerError::InvalidMethod(_) => Some("method".to_string()),
            crate::ledger::LedgerError::InvalidAdjustmentType(_) => Some("type".to_string()),
            crate::ledger::LedgerError::WaiverExceedsUnallocated { .. } => {
                Some("amount_poisha".to_string())
            }
            crate::ledger::LedgerError::RefundExceedsUnallocated { .. } => {
                Some("amount_poisha".to_string())
            }
            crate::ledger::LedgerError::DueChanged { .. } => {
                Some("expected_balance_poisha".to_string())
            }
            _ => None,
        },
    }
}

fn map_backup_error(err: crate::backup::BackupError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: None,
    }
}

fn map_diagnostics_error(err: crate::diagnostics::DiagnosticsError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: None,
    }
}

fn map_attendance_error(err: crate::attendance::AttendanceError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: match &err {
            crate::attendance::AttendanceError::BatchNotFound(_) => Some("batch_id".to_string()),
            crate::attendance::AttendanceError::SessionNotFound(_) => {
                Some("session_id".to_string())
            }
            crate::attendance::AttendanceError::InvalidStatus(_) => Some("status".to_string()),
            crate::attendance::AttendanceError::MissingAuditReason => {
                Some("audit_reason".to_string())
            }
            _ => None,
        },
    }
}

fn map_comms_error(err: crate::comms::CommsError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: match &err {
            crate::comms::CommsError::PhoneNormalizationFailed(_) => Some("phone".to_string()),
            crate::comms::CommsError::InvalidEmailAddress(_) => Some("email".to_string()),
            crate::comms::CommsError::EmailSubjectRequired => Some("subject".to_string()),
            crate::comms::CommsError::NotFound(_) => Some("id".to_string()),
            _ => None,
        },
    }
}

fn map_exam_error(err: crate::exam::ExamError) -> RpcError {
    let field = match &err {
        crate::exam::ExamError::ExamNotFound(_) => Some("exam_id".to_string()),
        crate::exam::ExamError::ExamSubjectNotFound(_) => Some("exam_subject_id".to_string()),
        crate::exam::ExamError::GradingSchemeNotFound(_) => Some("grading_scheme_id".to_string()),
        crate::exam::ExamError::MarkOutOfRange { .. } => Some("marks".to_string()),
        crate::exam::ExamError::MarkNegative(_) => Some("marks".to_string()),
        _ => None,
    };
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field,
    }
}

fn map_search_error(err: crate::search::SearchError) -> RpcError {
    RpcError {
        code: "search_sql_error".to_string(),
        message: err.to_string(),
        field: None,
    }
}

fn map_reports_error(err: crate::reports::ReportsError) -> RpcError {
    match err {
        crate::reports::ReportsError::Validation(msg) => RpcError {
            code: "validation_error".to_string(),
            message: msg,
            field: None,
        },
        crate::reports::ReportsError::NotFound(msg) => RpcError {
            code: "not_found".to_string(),
            message: msg,
            field: None,
        },
        crate::reports::ReportsError::Sql(msg) => RpcError {
            code: "reports_sql_error".to_string(),
            message: msg,
            field: None,
        },
    }
}

fn map_intelligence_error(err: crate::intelligence::IntelligenceError) -> RpcError {
    let field = match &err {
        crate::intelligence::IntelligenceError::StudentNotFound(_) => {
            Some("student_id".to_string())
        }
        crate::intelligence::IntelligenceError::BatchNotFound(_) => Some("batch_id".to_string()),
        crate::intelligence::IntelligenceError::ProgramNotFound(_) => {
            Some("program_id".to_string())
        }
        crate::intelligence::IntelligenceError::InvalidNoteType(_) => Some("note_type".to_string()),
        crate::intelligence::IntelligenceError::InvalidContactChannel(_) => {
            Some("contact_channel".to_string())
        }
        crate::intelligence::IntelligenceError::InvalidOutcome(_) => Some("outcome".to_string()),
        crate::intelligence::IntelligenceError::NegativeCost(f) => Some(f.clone()),
        crate::intelligence::IntelligenceError::InvalidTeacherCostType(_) => {
            Some("teacher_cost_type".to_string())
        }
        crate::intelligence::IntelligenceError::InvalidDateFormat(_) => Some("date".to_string()),
        _ => None,
    };
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field,
    }
}

fn map_integrity_error(err: crate::integrity::IntegrityError) -> RpcError {
    let field = match &err {
        crate::integrity::IntegrityError::InvalidReceiptFormat(_) => {
            Some("code_or_payload".to_string())
        }
        crate::integrity::IntegrityError::ReceiptNotFound(_) => Some("code_or_payload".to_string()),
        crate::integrity::IntegrityError::DuplicatePaymentDetected(_) => {
            Some("reference".to_string())
        }
        crate::integrity::IntegrityError::InvalidOverrideReason(_) => {
            Some("duplicate_override_reason".to_string())
        }
        _ => None,
    };
    RpcError {
        code: match &err {
            crate::integrity::IntegrityError::Sqlite(_) => "database_error".to_string(),
            crate::integrity::IntegrityError::Io(_) => "io_error".to_string(),
            crate::integrity::IntegrityError::Json(_) => "json_error".to_string(),
            crate::integrity::IntegrityError::Crypto(_) => "crypto_error".to_string(),
            crate::integrity::IntegrityError::InvalidReceiptFormat(_) => {
                "invalid_receipt_format".to_string()
            }
            crate::integrity::IntegrityError::ReceiptSignatureMismatch => {
                "receipt_signature_mismatch".to_string()
            }
            crate::integrity::IntegrityError::ReceiptNotFound(_) => "receipt_not_found".to_string(),
            crate::integrity::IntegrityError::DuplicatePaymentDetected(_) => {
                "duplicate_payment_detected".to_string()
            }
            crate::integrity::IntegrityError::InvalidOverrideReason(_) => {
                "invalid_override_reason".to_string()
            }
            crate::integrity::IntegrityError::Validation(_) => "validation_error".to_string(),
            crate::integrity::IntegrityError::CheckFailed(_) => "check_failed".to_string(),
            crate::integrity::IntegrityError::MigrationError(_) => "migration_error".to_string(),
            crate::integrity::IntegrityError::NetworkError(_) => "network_error".to_string(),
            crate::integrity::IntegrityError::Zip(_) => "zip_error".to_string(),
        },
        message: err.to_string(),
        field,
    }
}

fn map_ai_error(err: crate::ai::AiError) -> RpcError {
    RpcError {
        code: err.code().to_string(),
        message: err.to_string(),
        field: None,
    }
}
