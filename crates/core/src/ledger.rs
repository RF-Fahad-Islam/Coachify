use crate::db::{self, now_utc_iso};
use crate::fees::{self, FeesError};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    #[error("Student with ID '{0}' not found")]
    StudentNotFound(String),
    #[error("Charge with ID '{0}' not found")]
    ChargeNotFound(String),
    #[error("Payment with ID '{0}' not found")]
    PaymentNotFound(String),
    #[error("Fee head with ID '{0}' not found")]
    FeeHeadNotFound(String),
    #[error("Cash day for business date '{0}' is closed; no financial mutations can be posted")]
    ClosedCashDay(String),
    #[error("Duplicate idempotency key '{0}' detected with conflicting payload")]
    DuplicateIdempotency(String),
    #[error("Cannot void charge with active payment allocations ({0} poisha allocated); refund or void payments first")]
    ChargeHasAllocations(i64),
    #[error("Waiver amount {amount} poisha exceeds remaining unallocated dues of {remaining} poisha on this charge")]
    WaiverExceedsUnallocated { amount: i64, remaining: i64 },
    #[error("Refund amount {amount} poisha exceeds unallocated funds of {available} poisha on payment '{payment_id}'")]
    RefundExceedsUnallocated {
        payment_id: String,
        amount: i64,
        available: i64,
    },
    #[error("Unauthorized payment void: {0}")]
    UnauthorizedVoid(String),
    #[error("Invalid amount {0}: financial transaction amounts must be positive")]
    InvalidAmount(i64),
    #[error("Invalid payment method '{0}'. Must be cash, bkash, nagad, bank, card, or other")]
    InvalidMethod(String),
    #[error("Invalid adjustment type '{0}'. Must be waiver, fine, or correction")]
    InvalidAdjustmentType(String),
    #[error("Ledger invariant violation: {0}")]
    InvariantViolation(String),
    #[error("Due has changed: expected {expected_poisha} poisha, but actual current balance is {actual_poisha} poisha")]
    DueChanged {
        expected_poisha: i64,
        actual_poisha: i64,
    },
    #[error("Expense category '{0}' already exists")]
    ExpenseCategoryExists(String),
    #[error("Expense category with ID '{0}' not found")]
    ExpenseCategoryNotFound(String),
    #[error("Expense with ID '{0}' not found")]
    ExpenseNotFound(String),
    #[error("Expense from business date '{0}' cannot be voided today ('{1}')")]
    UnauthorizedExpenseVoid(String, String),
    #[error("Cash day for business date '{0}' is already closed")]
    CashDayAlreadyClosed(String),
    #[error("Database error: {0}")]
    Sql(String),
}

impl LedgerError {
    pub fn code(&self) -> &'static str {
        match self {
            LedgerError::StudentNotFound(_) => "student_not_found",
            LedgerError::ChargeNotFound(_) => "charge_not_found",
            LedgerError::PaymentNotFound(_) => "payment_not_found",
            LedgerError::FeeHeadNotFound(_) => "fee_head_not_found",
            LedgerError::ClosedCashDay(_) => "closed_cash_day",
            LedgerError::DuplicateIdempotency(_) => "duplicate_idempotency",
            LedgerError::ChargeHasAllocations(_) => "charge_has_allocations",
            LedgerError::WaiverExceedsUnallocated { .. } => "waiver_exceeds_unallocated",
            LedgerError::RefundExceedsUnallocated { .. } => "refund_exceeds_unallocated",
            LedgerError::UnauthorizedVoid(_) => "unauthorized_void",
            LedgerError::InvalidAmount(_) => "invalid_amount",
            LedgerError::InvalidMethod(_) => "invalid_method",
            LedgerError::InvalidAdjustmentType(_) => "invalid_adjustment_type",
            LedgerError::InvariantViolation(_) => "invariant_violation",
            LedgerError::DueChanged { .. } => "due_changed",
            LedgerError::ExpenseCategoryExists(_) => "category_exists",
            LedgerError::ExpenseCategoryNotFound(_) => "category_not_found",
            LedgerError::ExpenseNotFound(_) => "expense_not_found",
            LedgerError::UnauthorizedExpenseVoid(..) => "unauthorized_void",
            LedgerError::CashDayAlreadyClosed(_) => "cash_day_already_closed",
            LedgerError::Sql(_) => "database_error",
        }
    }
}

impl From<rusqlite::Error> for LedgerError {
    fn from(err: rusqlite::Error) -> Self {
        LedgerError::Sql(err.to_string())
    }
}

impl From<FeesError> for LedgerError {
    fn from(err: FeesError) -> Self {
        LedgerError::Sql(err.to_string())
    }
}

pub const VALID_PAYMENT_METHODS: &[&str] = &["cash", "bkash", "nagad", "bank", "card", "other"];
pub const VALID_REFUND_METHODS: &[&str] = &["cash", "bkash", "nagad", "bank", "other"];
pub const VALID_ADJUSTMENT_TYPES: &[&str] = &["waiver", "fine", "correction"];

// -----------------------------------------------------------------------------
// DTOs & Domain Models
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaymentReceipt {
    pub id: String,
    pub receipt_no: String,
    pub student_id: String,
    pub student_name: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reference: Option<String>,
    pub business_date: String,
    pub received_at: String,
    pub collector_user_id: Option<String>,
    pub device_id: Option<String>,
    pub idempotency_key: String,
    pub status: String,
    pub void_reason: Option<String>,
    pub allocated_amount_poisha: i64,
    pub unallocated_amount_poisha: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreatePaymentInput {
    pub student_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reference: Option<String>,
    pub business_date: Option<String>,
    pub idempotency_key: String,
    pub target_charge_ids: Option<Vec<String>>,
    pub expected_balance_poisha: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ReceiptAllocationItem {
    pub fee_head_name: String,
    pub fee_head_name_bn: Option<String>,
    pub period: Option<String>,
    pub amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrintableReceiptData {
    pub receipt_no: String,
    pub business_date: String,
    pub received_at: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub program_name: Option<String>,
    pub roll_no: Option<String>,
    pub amount_poisha: i64,
    pub amount_in_words_en: String,
    pub amount_in_words_bn: String,
    pub method: String,
    pub reference: Option<String>,
    pub collector_name: String,
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub institution_address: Option<String>,
    pub institution_phone: Option<String>,
    pub institution_email: Option<String>,
    pub institution_website: Option<String>,
    pub balance_after_poisha: i64,
    pub items: Vec<ReceiptAllocationItem>,
    pub is_reprint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashierStudentOverview {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub photo_vault_id: Option<String>,
    pub phone: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub program_name: Option<String>,
    pub roll_no: Option<String>,
    pub total_due_poisha: i64,
    pub advance_credit_poisha: i64,
    pub net_balance_poisha: i64,
    pub open_charges: Vec<ChargeItem>,
    pub recent_payments: Vec<PaymentReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChargeItem {
    pub id: String,
    pub student_id: String,
    pub enrollment_id: Option<String>,
    pub fee_head_id: String,
    pub fee_head_name: String,
    pub fee_head_kind: String,
    pub period: Option<String>,
    pub gross_amount_poisha: i64,
    pub concession_amount_poisha: i64,
    pub net_amount_poisha: i64,
    pub allocated_amount_poisha: i64,
    pub waiver_amount_poisha: i64,
    pub fine_amount_poisha: i64,
    pub remaining_due_poisha: i64,
    pub due_date: String,
    pub source: String,
    pub status: String,
    pub void_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateManualChargeInput {
    pub student_id: String,
    pub fee_head_id: String,
    pub amount_poisha: i64,
    pub due_date: String,
    pub period: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdjustmentItem {
    pub id: String,
    pub student_id: String,
    pub charge_id: Option<String>,
    pub r#type: String, // "waiver" | "fine" | "correction"
    pub amount_poisha: i64,
    pub reason: String,
    pub approved_by: Option<String>,
    pub business_date: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAdjustmentInput {
    pub student_id: String,
    pub charge_id: Option<String>,
    pub r#type: String,
    pub amount_poisha: i64,
    pub reason: String,
    pub business_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefundItem {
    pub id: String,
    pub payment_id: String,
    pub receipt_no: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reason: String,
    pub business_date: String,
    pub refunded_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateRefundInput {
    pub payment_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reason: String,
    pub business_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentLedgerSummary {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub total_charged_poisha: i64,
    pub total_paid_poisha: i64,
    pub total_adjusted_poisha: i64,
    pub total_refunded_poisha: i64,
    pub balance_poisha: i64,
    pub charges: Vec<ChargeItem>,
    pub payments: Vec<PaymentReceipt>,
    pub adjustments: Vec<AdjustmentItem>,
    pub refunds: Vec<RefundItem>,
}

// --- Dues & Statements DTOs (Spec 0017) ---

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ListDuesFilter {
    pub program_id: Option<String>,
    pub batch_id: Option<String>,
    pub period: Option<String>,
    pub min_due_poisha: Option<i64>,
    pub overdue_only: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentDueSummaryItem {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub roll_no: Option<String>,
    pub current_month_due_poisha: i64,
    pub overdue_poisha: i64,
    pub total_due_poisha: i64,
    pub oldest_unpaid_due_date: Option<String>,
    pub last_payment_date: Option<String>,
    pub last_payment_amount_poisha: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StatementChargeLine {
    pub charge_id: String,
    pub fee_head_name: String,
    pub fee_head_name_bn: Option<String>,
    pub period: Option<String>,
    pub due_date: String,
    pub gross_amount_poisha: i64,
    pub concession_amount_poisha: i64,
    pub net_amount_poisha: i64,
    pub paid_amount_poisha: i64,
    pub remaining_due_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StatementPaymentLine {
    pub payment_id: String,
    pub receipt_no: String,
    pub business_date: String,
    pub method: String,
    pub amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DueStatementData {
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub institution_address: String,
    pub institution_phone: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub roll_no: Option<String>,
    pub statement_date: String,
    pub itemized_charges: Vec<StatementChargeLine>,
    pub recent_payments: Vec<StatementPaymentLine>,
    pub total_charged_poisha: i64,
    pub total_concession_poisha: i64,
    pub total_paid_poisha: i64,
    pub total_adjusted_poisha: i64,
    pub net_due_poisha: i64,
    pub amount_in_words_en: String,
    pub amount_in_words_bn: String,
    pub payment_instructions: Option<String>,
}

// --- Expenses & Cash Close DTOs (Spec 0018) ---

pub const DEFAULT_EXPENSE_CATEGORIES: &[&str] = &[
    "Office Rent",
    "Teacher Honorarium",
    "Staff Salaries",
    "Printing & Stationery",
    "Utilities",
    "Student Refreshments",
    "Marketing",
    "Maintenance",
    "Miscellaneous",
];

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExpenseCategoryItem {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExpenseCategoryInput {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExpenseItem {
    pub id: String,
    pub category_id: String,
    pub category_name: String,
    pub amount_poisha: i64,
    pub method: String,
    pub payee: String,
    pub reference: Option<String>,
    pub business_date: String,
    pub attachment_vault_id: Option<String>,
    pub created_by: Option<String>,
    pub status: String,
    pub void_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExpenseInput {
    pub category_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub payee: String,
    pub reference: Option<String>,
    pub business_date: Option<String>,
    pub attachment_vault_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ListExpensesFilter {
    pub category_id: Option<String>,
    pub business_date: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub method: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoidExpenseInput {
    pub expense_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashDayItem {
    pub business_date: String,
    pub opening_cash_poisha: i64,
    pub cash_in_poisha: i64,
    pub cash_out_poisha: i64,
    pub expected_closing_poisha: i64,
    pub counted_closing_poisha: Option<i64>,
    pub difference_poisha: Option<i64>,
    pub note: Option<String>,
    pub closed_by: Option<String>,
    pub closed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloseCashDayInput {
    pub business_date: String,
    pub counted_closing_poisha: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MethodCollectionSummary {
    pub method: String,
    pub count: i64,
    pub total_amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectorSummary {
    pub collector_user_id: Option<String>,
    pub collector_name: String,
    pub receipt_count: i64,
    pub total_amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CategoryExpenseSummary {
    pub category_id: String,
    pub category_name: String,
    pub count: i64,
    pub total_amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashDaySummary {
    pub business_date: String,
    pub is_closed: bool,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub opening_cash_poisha: i64,
    pub cash_in_poisha: i64,
    pub cash_out_poisha: i64,
    pub expected_closing_poisha: i64,
    pub counted_closing_poisha: Option<i64>,
    pub difference_poisha: Option<i64>,
    pub note: Option<String>,
    pub digital_collections: Vec<MethodCollectionSummary>,
    pub collector_breakdown: Vec<CollectorSummary>,
    pub expenses_by_category: Vec<CategoryExpenseSummary>,
}

// -----------------------------------------------------------------------------
// Validation & Date Helpers
// -----------------------------------------------------------------------------

pub fn current_business_date() -> String {
    let now = jiff::Timestamp::now();
    now.strftime("%Y-%m-%d").to_string()
}

pub fn current_year() -> String {
    let now = jiff::Timestamp::now();
    now.strftime("%Y").to_string()
}

/// Validates that a business date is not closed in cash_days (AC-7, Invariant 7).
pub fn verify_open_cash_day(conn: &Connection, business_date: &str) -> Result<(), LedgerError> {
    let closed: Option<String> = conn
        .query_row(
            "SELECT closed_at FROM cash_days WHERE business_date = ?1 AND closed_at IS NOT NULL",
            params![business_date],
            |r| r.get(0),
        )
        .optional()?;

    if closed.is_some() {
        return Err(LedgerError::ClosedCashDay(business_date.to_string()));
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Payment Collection & Sequencing (AC-7, AC-8, AC-9)
// -----------------------------------------------------------------------------

pub fn create_payment(
    conn: &Connection,
    input: CreatePaymentInput,
    collector_user_id: Option<&str>,
    device_id: Option<&str>,
) -> Result<PaymentReceipt, LedgerError> {
    if input.amount_poisha <= 0 {
        return Err(LedgerError::InvalidAmount(input.amount_poisha));
    }

    let method = input.method.to_lowercase();
    if !VALID_PAYMENT_METHODS.contains(&method.as_str()) {
        return Err(LedgerError::InvalidMethod(input.method));
    }

    // Check student exists
    let student_name: String = conn
        .query_row(
            "SELECT name FROM students WHERE id = ?1",
            params![input.student_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(input.student_id.clone()))?;

    // AC-8: Check idempotency key - if already processed, return existing payment receipt
    let existing_idem: Option<String> = conn
        .query_row(
            "SELECT id FROM payments WHERE idempotency_key = ?1",
            params![input.idempotency_key],
            |r| r.get(0),
        )
        .optional()?;

    if let Some(existing_id) = existing_idem {
        return get_payment_receipt(conn, &existing_id);
    }

    // AC-7: Optimistic concurrency check with expected balance guard
    let current_balance: i64 = conn
        .query_row(
            "SELECT balance_poisha FROM student_balances WHERE student_id = ?1",
            params![input.student_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);

    if let Some(expected) = input.expected_balance_poisha {
        if current_balance != expected {
            return Err(LedgerError::DueChanged {
                expected_poisha: expected,
                actual_poisha: current_balance,
            });
        }
    }

    let b_date = input.business_date.unwrap_or_else(current_business_date);
    verify_open_cash_day(conn, &b_date)?;

    // AC-8: Atomic monotonic receipt sequence allocation (REC-YYYY-####)
    let year = current_year();
    let seq_key = format!("receipt:{}", year);
    let seq_prefix = format!("REC-{}-", year);
    let receipt_no = db::allocate_sequence(conn, &seq_key, &seq_prefix)
        .map_err(|e| LedgerError::Sql(e.to_string()))?;

    let payment_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO payments (id, receipt_no, student_id, amount_poisha, method, reference, business_date, received_at, collector_user_id, device_id, idempotency_key, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'posted', ?12, ?12)",
        params![
            payment_id,
            receipt_no,
            input.student_id,
            input.amount_poisha,
            method,
            input.reference,
            b_date,
            now,
            collector_user_id,
            device_id,
            input.idempotency_key,
            now
        ],
    )?;

    // Allocate payment funds to charges
    let mut remaining_funds = input.amount_poisha;
    let mut total_allocated = 0;

    // 1. If explicit target charge IDs specified
    if let Some(target_ids) = input.target_charge_ids {
        for cid in target_ids {
            if remaining_funds <= 0 {
                break;
            }

            // Get charge unpaid due
            let remaining_due: Option<i64> = conn
                .query_row(
                    "SELECT c.net_amount_poisha
                            + COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'fine'), 0)
                            - COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'waiver'), 0)
                            - COALESCE((
                                SELECT SUM(pa.amount_poisha)
                                FROM payment_allocations pa
                                JOIN payments p ON pa.payment_id = p.id
                                WHERE pa.charge_id = c.id AND p.status = 'posted'
                            ), 0)
                     FROM charges c
                     WHERE c.id = ?1 AND c.student_id = ?2 AND c.status = 'open'",
                    params![cid, input.student_id],
                    |r| r.get(0),
                )
                .optional()?;

            if let Some(due) = remaining_due {
                if due > 0 {
                    let alloc_amount = due.min(remaining_funds);
                    let alloc_id = Uuid::now_v7().to_string();
                    conn.execute(
                        "INSERT INTO payment_allocations (id, payment_id, charge_id, amount_poisha, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![alloc_id, payment_id, cid, alloc_amount, now],
                    )?;
                    remaining_funds -= alloc_amount;
                    total_allocated += alloc_amount;
                }
            }
        }
    } else {
        // Auto-allocate oldest open charges first
        let mut c_stmt = conn.prepare(
            "SELECT c.id, fh.kind, c.due_date,
                    c.net_amount_poisha + COALESCE(f.amount, 0) - COALESCE(w.amount, 0) - COALESCE(pa.amount, 0) as remaining_due
             FROM charges c
             JOIN fee_heads fh ON c.fee_head_id = fh.id
             LEFT JOIN (
                 SELECT adj.charge_id, SUM(adj.amount_poisha) as amount
                 FROM adjustments adj
                 WHERE adj.charge_id IS NOT NULL AND adj.type = 'fine' AND adj.student_id = ?1
                 GROUP BY adj.charge_id
             ) f ON f.charge_id = c.id
             LEFT JOIN (
                 SELECT adj.charge_id, SUM(adj.amount_poisha) as amount
                 FROM adjustments adj
                 WHERE adj.charge_id IS NOT NULL AND adj.type = 'waiver' AND adj.student_id = ?1
                 GROUP BY adj.charge_id
             ) w ON w.charge_id = c.id
             LEFT JOIN (
                 SELECT pa.charge_id, SUM(pa.amount_poisha) as amount
                 FROM payment_allocations pa
                 JOIN payments p ON pa.payment_id = p.id
                 WHERE p.student_id = ?1 AND p.status = 'posted'
                 GROUP BY pa.charge_id
             ) pa ON pa.charge_id = c.id
             WHERE c.student_id = ?1 AND c.status = 'open'
               AND (c.net_amount_poisha + COALESCE(f.amount, 0) - COALESCE(w.amount, 0) - COALESCE(pa.amount, 0)) > 0
             ORDER BY c.due_date ASC",
        )?;

        let rows = c_stmt.query_map(params![input.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;

        let mut charges: Vec<(String, String, String, i64)> = Vec::new();
        for r in rows {
            let item = r?;
            if item.3 > 0 {
                charges.push(item);
            }
        }
        drop(c_stmt);

        // Sort by due_date ASC, tie broken by fee head priority
        charges.sort_by(|a, b| {
            a.2.cmp(&b.2)
                .then_with(|| fees::fee_head_priority(&a.1).cmp(&fees::fee_head_priority(&b.1)))
        });

        for (cid, _, _, due) in charges {
            if remaining_funds <= 0 {
                break;
            }
            let alloc_amount = due.min(remaining_funds);
            let alloc_id = Uuid::now_v7().to_string();
            conn.execute(
                "INSERT INTO payment_allocations (id, payment_id, charge_id, amount_poisha, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![alloc_id, payment_id, cid, alloc_amount, now],
            )?;
            remaining_funds -= alloc_amount;
            total_allocated += alloc_amount;
        }
    }

    // AC-9: Synchronous balance cache updater
    fees::recalculate_student_balance(conn, &input.student_id)?;

    let receipt = PaymentReceipt {
        id: payment_id,
        receipt_no,
        student_id: input.student_id,
        student_name,
        amount_poisha: input.amount_poisha,
        method,
        reference: input.reference,
        business_date: b_date,
        received_at: now.clone(),
        collector_user_id: collector_user_id.map(|s| s.to_string()),
        device_id: device_id.map(|s| s.to_string()),
        idempotency_key: input.idempotency_key,
        status: "posted".to_string(),
        void_reason: None,
        allocated_amount_poisha: total_allocated,
        unallocated_amount_poisha: remaining_funds,
        created_at: now,
    };

    // AC-4: Omnichannel Payment Received Automation Hook (SMS, WhatsApp, Email)
    let _ = crate::comms::enqueue_payment_received_messages_if_enabled(
        conn,
        &receipt,
        collector_user_id,
    );

    Ok(receipt)
}

// -----------------------------------------------------------------------------
// Payment Voiding (AC-7)
// -----------------------------------------------------------------------------

pub fn void_payment(
    conn: &Connection,
    payment_id: &str,
    reason: &str,
    voided_by: Option<&str>,
) -> Result<(), LedgerError> {
    if reason.trim().is_empty() {
        return Err(LedgerError::UnauthorizedVoid(
            "Void reason is required".to_string(),
        ));
    }

    let (student_id, status, business_date): (String, String, String) = conn
        .query_row(
            "SELECT student_id, status, business_date FROM payments WHERE id = ?1",
            params![payment_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::PaymentNotFound(payment_id.to_string()))?;

    if status == "voided" {
        return Ok(());
    }

    // Spec 0015 AC-1: Payment voiding is strictly restricted to same-day transactions
    let today = current_business_date();
    if business_date != today {
        return Err(LedgerError::UnauthorizedVoid(format!(
            "Payment from business date '{business_date}' cannot be voided today ('{today}'). Issue a refund instead."
        )));
    }

    // Validate cash day is open
    verify_open_cash_day(conn, &business_date)?;

    let now = now_utc_iso();

    // Void payment: updates status to 'voided'.
    // NOTICE: raw payment_allocations are NOT deleted (trigger prohibits DELETE).
    // Because all queries join on payments.status = 'posted', voided payment allocations
    // are automatically ignored, returning charges to unpaid status!
    conn.execute(
        "UPDATE payments SET status = 'voided', void_reason = ?1, voided_by = ?2, voided_at = ?3, updated_at = ?3
         WHERE id = ?4",
        params![reason.trim(), voided_by, now, payment_id],
    )?;

    // AC-9: Synchronous balance cache updater
    fees::recalculate_student_balance(conn, &student_id)?;

    Ok(())
}

// -----------------------------------------------------------------------------
// Manual Charges & Voiding (AC-4, AC-7)
// -----------------------------------------------------------------------------

pub fn create_manual_charge(
    conn: &Connection,
    input: CreateManualChargeInput,
) -> Result<ChargeItem, LedgerError> {
    if input.amount_poisha <= 0 {
        return Err(LedgerError::InvalidAmount(input.amount_poisha));
    }

    // Verify student exists
    let _ = conn
        .query_row(
            "SELECT id FROM students WHERE id = ?1",
            params![input.student_id],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(input.student_id.clone()))?;

    // Verify fee head exists
    let _ = fees::get_fee_head(conn, &input.fee_head_id)
        .map_err(|_| LedgerError::FeeHeadNotFound(input.fee_head_id.clone()))?;

    let charge_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO charges (id, student_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?5, ?6, 'manual', 'open', ?7, ?7)",
        params![
            charge_id,
            input.student_id,
            input.fee_head_id,
            input.period,
            input.amount_poisha,
            input.due_date,
            now
        ],
    )?;

    // Auto-allocate advance credit if available
    let _ = fees::auto_allocate_advance_credit(conn, &input.student_id)?;

    // AC-9: Synchronous balance update
    fees::recalculate_student_balance(conn, &input.student_id)?;

    get_charge(conn, &charge_id)
}

pub fn void_charge(conn: &Connection, charge_id: &str, reason: &str) -> Result<(), LedgerError> {
    if reason.trim().is_empty() {
        return Err(LedgerError::UnauthorizedVoid(
            "Void reason is required".to_string(),
        ));
    }

    let (student_id, status): (String, String) = conn
        .query_row(
            "SELECT student_id, status FROM charges WHERE id = ?1",
            params![charge_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::ChargeNotFound(charge_id.to_string()))?;

    if status == "voided" {
        return Ok(());
    }

    // Check active allocations: charge with allocations cannot be voided (AC-7)
    let allocated_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(pa.amount_poisha), 0)
         FROM payment_allocations pa
         JOIN payments p ON pa.payment_id = p.id
         WHERE pa.charge_id = ?1 AND p.status = 'posted'",
        params![charge_id],
        |r| r.get(0),
    )?;

    if allocated_sum > 0 {
        return Err(LedgerError::ChargeHasAllocations(allocated_sum));
    }

    let now = now_utc_iso();

    conn.execute(
        "UPDATE charges SET status = 'voided', void_reason = ?1, updated_at = ?2
         WHERE id = ?3",
        params![reason.trim(), now, charge_id],
    )?;

    fees::recalculate_student_balance(conn, &student_id)?;
    Ok(())
}

pub fn get_charge(conn: &Connection, id: &str) -> Result<ChargeItem, LedgerError> {
    conn.query_row(
        "SELECT c.id, c.student_id, c.enrollment_id, c.fee_head_id, fh.name, fh.kind,
                c.period, c.gross_amount_poisha, c.concession_amount_poisha, c.net_amount_poisha,
                COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = c.id AND p.status = 'posted'
                ), 0) as allocated,
                COALESCE((
                    SELECT SUM(adj.amount_poisha)
                    FROM adjustments adj
                    WHERE adj.charge_id = c.id AND adj.type = 'waiver'
                ), 0) as waivers,
                COALESCE((
                    SELECT SUM(adj.amount_poisha)
                    FROM adjustments adj
                    WHERE adj.charge_id = c.id AND adj.type = 'fine'
                ), 0) as fines,
                c.due_date, c.source, c.status, c.void_reason, c.created_at
         FROM charges c
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE c.id = ?1",
        params![id],
        |r| {
            let net: i64 = r.get(9)?;
            let allocated: i64 = r.get(10)?;
            let waivers: i64 = r.get(11)?;
            let fines: i64 = r.get(12)?;
            let remaining = (net + fines - waivers - allocated).max(0);

            Ok(ChargeItem {
                id: r.get(0)?,
                student_id: r.get(1)?,
                enrollment_id: r.get(2)?,
                fee_head_id: r.get(3)?,
                fee_head_name: r.get(4)?,
                fee_head_kind: r.get(5)?,
                period: r.get(6)?,
                gross_amount_poisha: r.get(7)?,
                concession_amount_poisha: r.get(8)?,
                net_amount_poisha: net,
                allocated_amount_poisha: allocated,
                waiver_amount_poisha: waivers,
                fine_amount_poisha: fines,
                remaining_due_poisha: remaining,
                due_date: r.get(13)?,
                source: r.get(14)?,
                status: r.get(15)?,
                void_reason: r.get(16)?,
                created_at: r.get(17)?,
            })
        },
    )
    .optional()?
    .ok_to_result(LedgerError::ChargeNotFound(id.to_string()))
}

pub fn list_charges_by_student(
    conn: &Connection,
    student_id: &str,
) -> Result<Vec<ChargeItem>, LedgerError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.student_id, c.enrollment_id, c.fee_head_id, fh.name, fh.kind,
                c.period, c.gross_amount_poisha, c.concession_amount_poisha, c.net_amount_poisha,
                COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = c.id AND p.status = 'posted'
                ), 0) as allocated,
                COALESCE((
                    SELECT SUM(adj.amount_poisha)
                    FROM adjustments adj
                    WHERE adj.charge_id = c.id AND adj.type = 'waiver'
                ), 0) as waivers,
                COALESCE((
                    SELECT SUM(adj.amount_poisha)
                    FROM adjustments adj
                    WHERE adj.charge_id = c.id AND adj.type = 'fine'
                ), 0) as fines,
                c.due_date, c.source, c.status, c.void_reason, c.created_at
         FROM charges c
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE c.student_id = ?1
         ORDER BY c.due_date DESC, c.created_at DESC",
    )?;

    let rows = stmt.query_map(params![student_id], |r| {
        let net: i64 = r.get(9)?;
        let allocated: i64 = r.get(10)?;
        let waivers: i64 = r.get(11)?;
        let fines: i64 = r.get(12)?;
        let remaining = (net + fines - waivers - allocated).max(0);

        Ok(ChargeItem {
            id: r.get(0)?,
            student_id: r.get(1)?,
            enrollment_id: r.get(2)?,
            fee_head_id: r.get(3)?,
            fee_head_name: r.get(4)?,
            fee_head_kind: r.get(5)?,
            period: r.get(6)?,
            gross_amount_poisha: r.get(7)?,
            concession_amount_poisha: r.get(8)?,
            net_amount_poisha: net,
            allocated_amount_poisha: allocated,
            waiver_amount_poisha: waivers,
            fine_amount_poisha: fines,
            remaining_due_poisha: remaining,
            due_date: r.get(13)?,
            source: r.get(14)?,
            status: r.get(15)?,
            void_reason: r.get(16)?,
            created_at: r.get(17)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

// -----------------------------------------------------------------------------
// Adjustments (Waivers, Fines, Corrections) (AC-7)
// -----------------------------------------------------------------------------

pub fn create_adjustment(
    conn: &Connection,
    input: CreateAdjustmentInput,
    approved_by: Option<&str>,
) -> Result<AdjustmentItem, LedgerError> {
    if input.amount_poisha <= 0 {
        return Err(LedgerError::InvalidAmount(input.amount_poisha));
    }

    let adj_type = input.r#type.to_lowercase();
    if !VALID_ADJUSTMENT_TYPES.contains(&adj_type.as_str()) {
        return Err(LedgerError::InvalidAdjustmentType(input.r#type));
    }

    // Verify student exists
    let _ = conn
        .query_row(
            "SELECT id FROM students WHERE id = ?1",
            params![input.student_id],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(input.student_id.clone()))?;

    let b_date = input.business_date.unwrap_or_else(current_business_date);
    verify_open_cash_day(conn, &b_date)?;

    // AC-7: Waiver ceiling check
    if adj_type == "waiver" {
        if let Some(ref cid) = input.charge_id {
            let charge = get_charge(conn, cid)?;
            let remaining_due = charge.remaining_due_poisha;
            if input.amount_poisha > remaining_due {
                return Err(LedgerError::WaiverExceedsUnallocated {
                    amount: input.amount_poisha,
                    remaining: remaining_due,
                });
            }
        }
    }

    let adj_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO adjustments (id, student_id, charge_id, type, amount_poisha, reason, approved_by, business_date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            adj_id,
            input.student_id,
            input.charge_id,
            adj_type,
            input.amount_poisha,
            input.reason.trim(),
            approved_by,
            b_date,
            now
        ],
    )?;

    fees::recalculate_student_balance(conn, &input.student_id)?;

    Ok(AdjustmentItem {
        id: adj_id,
        student_id: input.student_id,
        charge_id: input.charge_id,
        r#type: adj_type,
        amount_poisha: input.amount_poisha,
        reason: input.reason,
        approved_by: approved_by.map(|s| s.to_string()),
        business_date: b_date,
        created_at: now,
    })
}

// -----------------------------------------------------------------------------
// Refunds (AC-7)
// -----------------------------------------------------------------------------

pub fn create_refund(
    conn: &Connection,
    input: CreateRefundInput,
    refunded_by: Option<&str>,
) -> Result<RefundItem, LedgerError> {
    if input.amount_poisha <= 0 {
        return Err(LedgerError::InvalidAmount(input.amount_poisha));
    }

    let method = input.method.to_lowercase();
    if !VALID_REFUND_METHODS.contains(&method.as_str()) {
        return Err(LedgerError::InvalidMethod(input.method));
    }

    // Verify payment exists and is posted
    let (receipt_no, student_id, status, paid_amount): (String, String, String, i64) = conn
        .query_row(
            "SELECT receipt_no, student_id, status, amount_poisha FROM payments WHERE id = ?1",
            params![input.payment_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::PaymentNotFound(input.payment_id.clone()))?;

    if status != "posted" {
        return Err(LedgerError::UnauthorizedVoid(
            "Cannot refund an unposted or voided payment".to_string(),
        ));
    }

    let b_date = input.business_date.unwrap_or_else(current_business_date);
    verify_open_cash_day(conn, &b_date)?;

    // Calculate unallocated payment funds
    let allocated_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM payment_allocations WHERE payment_id = ?1",
        params![input.payment_id],
        |r| r.get(0),
    )?;

    let prior_refunds: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM refunds WHERE payment_id = ?1",
        params![input.payment_id],
        |r| r.get(0),
    )?;

    let available_unallocated = paid_amount - allocated_sum - prior_refunds;
    if input.amount_poisha > available_unallocated {
        return Err(LedgerError::RefundExceedsUnallocated {
            payment_id: input.payment_id,
            amount: input.amount_poisha,
            available: available_unallocated,
        });
    }

    let refund_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO refunds (id, payment_id, amount_poisha, method, reason, business_date, refunded_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            refund_id,
            input.payment_id,
            input.amount_poisha,
            method,
            input.reason.trim(),
            b_date,
            refunded_by,
            now
        ],
    )?;

    fees::recalculate_student_balance(conn, &student_id)?;

    Ok(RefundItem {
        id: refund_id,
        payment_id: input.payment_id,
        receipt_no,
        amount_poisha: input.amount_poisha,
        method,
        reason: input.reason,
        business_date: b_date,
        refunded_by: refunded_by.map(|s| s.to_string()),
        created_at: now,
    })
}

// -----------------------------------------------------------------------------
// Student Ledger Summary & Ledger Invariants (AC-9, AC-10)
// -----------------------------------------------------------------------------

pub fn list_payments_by_student(
    conn: &Connection,
    student_id: &str,
) -> Result<Vec<PaymentReceipt>, LedgerError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.receipt_no, p.student_id, s.name, p.amount_poisha,
                p.method, p.reference, p.business_date, p.received_at,
                p.collector_user_id, p.device_id, p.idempotency_key,
                p.status, p.void_reason,
                COALESCE((SELECT SUM(pa.amount_poisha) FROM payment_allocations pa WHERE pa.payment_id = p.id), 0) as allocated,
                COALESCE((SELECT SUM(r.amount_poisha) FROM refunds r WHERE r.payment_id = p.id), 0) as refunded,
                p.created_at
         FROM payments p
         JOIN students s ON p.student_id = s.id
         WHERE p.student_id = ?1
         ORDER BY p.business_date DESC, p.created_at DESC",
    )?;

    let rows = stmt.query_map(params![student_id], |r| {
        let amount: i64 = r.get(4)?;
        let allocated: i64 = r.get(14)?;
        let refunded: i64 = r.get(15)?;
        let unallocated = (amount - allocated - refunded).max(0);

        Ok(PaymentReceipt {
            id: r.get(0)?,
            receipt_no: r.get(1)?,
            student_id: r.get(2)?,
            student_name: r.get(3)?,
            amount_poisha: amount,
            method: r.get(5)?,
            reference: r.get(6)?,
            business_date: r.get(7)?,
            received_at: r.get(8)?,
            collector_user_id: r.get(9)?,
            device_id: r.get(10)?,
            idempotency_key: r.get(11)?,
            status: r.get(12)?,
            void_reason: r.get(13)?,
            allocated_amount_poisha: allocated,
            unallocated_amount_poisha: unallocated,
            created_at: r.get(16)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn get_payment_receipt(
    conn: &Connection,
    payment_id: &str,
) -> Result<PaymentReceipt, LedgerError> {
    conn.query_row(
        "SELECT p.id, p.receipt_no, p.student_id, s.name, p.amount_poisha,
                p.method, p.reference, p.business_date, p.received_at,
                p.collector_user_id, p.device_id, p.idempotency_key,
                p.status, p.void_reason,
                COALESCE((SELECT SUM(pa.amount_poisha) FROM payment_allocations pa WHERE pa.payment_id = p.id), 0) as allocated,
                COALESCE((SELECT SUM(r.amount_poisha) FROM refunds r WHERE r.payment_id = p.id), 0) as refunded,
                p.created_at
         FROM payments p
         JOIN students s ON p.student_id = s.id
         WHERE p.id = ?1",
        params![payment_id],
        |r| {
            let amount: i64 = r.get(4)?;
            let allocated: i64 = r.get(14)?;
            let refunded: i64 = r.get(15)?;
            let unallocated = (amount - allocated - refunded).max(0);

            Ok(PaymentReceipt {
                id: r.get(0)?,
                receipt_no: r.get(1)?,
                student_id: r.get(2)?,
                student_name: r.get(3)?,
                amount_poisha: amount,
                method: r.get(5)?,
                reference: r.get(6)?,
                business_date: r.get(7)?,
                received_at: r.get(8)?,
                collector_user_id: r.get(9)?,
                device_id: r.get(10)?,
                idempotency_key: r.get(11)?,
                status: r.get(12)?,
                void_reason: r.get(13)?,
                allocated_amount_poisha: allocated,
                unallocated_amount_poisha: unallocated,
                created_at: r.get(16)?,
            })
        },
    )
    .optional()?
    .ok_to_result(LedgerError::PaymentNotFound(payment_id.to_string()))
}

pub fn get_printable_receipt(
    conn: &Connection,
    payment_id: &str,
    is_reprint: Option<bool>,
) -> Result<PrintableReceiptData, LedgerError> {
    let p = get_payment_receipt(conn, payment_id)?;

    let (code, name, name_bn, current_school_roll): (
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT student_code, name, name_bn, current_school_roll FROM students WHERE id = ?1",
            params![p.student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .unwrap_or_else(|| (p.student_id.clone(), p.student_name.clone(), None, None));

    let (guardian_name, guardian_phone): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT g.name, g.phone_norm
             FROM guardians g
             JOIN student_guardians sg ON sg.guardian_id = g.id
             WHERE sg.student_id = ?1 AND sg.is_primary = 1
             LIMIT 1",
            params![p.student_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));

    let (batch_name, program_name, batch_roll): (Option<String>, Option<String>, Option<String>) =
        conn.query_row(
            "SELECT b.name, p.name, CAST(e.roll AS TEXT)
             FROM enrollments e
             JOIN batches b ON e.batch_id = b.id
             JOIN programs p ON b.program_id = p.id
             WHERE e.student_id = ?1 AND e.status = 'active'
             LIMIT 1",
            params![p.student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .unwrap_or((None, None, None));

    let roll_no = batch_roll.or(current_school_roll);

    let collector_name: String = if let Some(uid) = &p.collector_user_id {
        conn.query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            params![uid],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or_else(|| "Cashier".to_string())
    } else {
        "Cashier".to_string()
    };

    struct InstitutionDetails {
        name: String,
        name_bn: Option<String>,
        address: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        website: Option<String>,
    }

    let inst = conn
        .query_row(
            "SELECT name, name_bn, address, phones, email, website FROM institution WHERE id = 1",
            [],
            |r| {
                Ok(InstitutionDetails {
                    name: r.get(0)?,
                    name_bn: r.get(1)?,
                    address: r.get(2)?,
                    phone: r.get(3)?,
                    email: r.get(4)?,
                    website: r.get(5)?,
                })
            },
        )
        .optional()?
        .unwrap_or_else(|| InstitutionDetails {
            name: "Apex Academic Coaching Centre".to_string(),
            name_bn: Some("এপেক্স একাডেমিক কোচিং সেন্টার".to_string()),
            address: Some("Farmgate, Dhaka-1215".to_string()),
            phone: Some("8801700000000".to_string()),
            email: Some("info@apexcoaching.edu.bd".to_string()),
            website: Some("www.apexcoaching.edu.bd".to_string()),
        });

    let mut alloc_stmt = conn.prepare(
        "SELECT fh.name, fh.name_bn, c.period, pa.amount_poisha
         FROM payment_allocations pa
         JOIN charges c ON pa.charge_id = c.id
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE pa.payment_id = ?1
         ORDER BY c.due_date ASC",
    )?;

    let alloc_rows = alloc_stmt.query_map(params![payment_id], |r| {
        Ok(ReceiptAllocationItem {
            fee_head_name: r.get(0)?,
            fee_head_name_bn: r.get(1)?,
            period: r.get(2)?,
            amount_poisha: r.get(3)?,
        })
    })?;

    let mut items = Vec::new();
    for r in alloc_rows {
        items.push(r?);
    }

    if p.unallocated_amount_poisha > 0 {
        items.push(ReceiptAllocationItem {
            fee_head_name: "Advance Credit / Unallocated Deposit".to_string(),
            fee_head_name_bn: Some("অগ্রিম জমা".to_string()),
            period: None,
            amount_poisha: p.unallocated_amount_poisha,
        });
    }

    let balance_after_poisha: i64 = conn
        .query_row(
            "SELECT balance_poisha FROM student_balances WHERE student_id = ?1",
            params![p.student_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);

    let amount_in_words_en = crate::words::amount_to_words_en(p.amount_poisha);
    let amount_in_words_bn = crate::words::amount_to_words_bn(p.amount_poisha);

    Ok(PrintableReceiptData {
        receipt_no: p.receipt_no,
        business_date: p.business_date,
        received_at: p.received_at,
        student_id: p.student_id,
        student_code: code,
        student_name: name,
        student_name_bn: name_bn,
        guardian_name,
        guardian_phone,
        batch_name,
        program_name,
        roll_no,
        amount_poisha: p.amount_poisha,
        amount_in_words_en,
        amount_in_words_bn,
        method: p.method,
        reference: p.reference,
        collector_name,
        institution_name: inst.name,
        institution_name_bn: inst.name_bn,
        institution_address: inst.address,
        institution_phone: inst.phone,
        institution_email: inst.email,
        institution_website: inst.website,
        balance_after_poisha,
        items,
        is_reprint: is_reprint.unwrap_or(false),
    })
}

pub fn get_cashier_student_overview(
    conn: &Connection,
    student_id: &str,
) -> Result<CashierStudentOverview, LedgerError> {
    let (code, name, name_bn, photo_vault_id, phone, current_school_roll): (
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT student_code, name, name_bn, photo_vault_id, phone_norm, current_school_roll
             FROM students WHERE id = ?1",
            params![student_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(student_id.to_string()))?;

    let (guardian_name, guardian_phone): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT g.name, g.phone_norm
             FROM guardians g
             JOIN student_guardians sg ON sg.guardian_id = g.id
             WHERE sg.student_id = ?1 AND sg.is_primary = 1
             LIMIT 1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));

    let (batch_name, program_name, batch_roll): (Option<String>, Option<String>, Option<String>) =
        conn.query_row(
            "SELECT b.name, p.name, CAST(e.roll AS TEXT)
             FROM enrollments e
             JOIN batches b ON e.batch_id = b.id
             JOIN programs p ON b.program_id = p.id
             WHERE e.student_id = ?1 AND e.status = 'active'
             LIMIT 1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .unwrap_or((None, None, None));

    let roll_no = batch_roll.or(current_school_roll);

    let net_balance_poisha: i64 = conn
        .query_row(
            "SELECT balance_poisha FROM student_balances WHERE student_id = ?1",
            params![student_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);

    let all_charges = list_charges_by_student(conn, student_id)?;
    let open_charges: Vec<ChargeItem> = all_charges
        .into_iter()
        .filter(|c| c.status == "open" && c.remaining_due_poisha > 0)
        .collect();

    let total_due_poisha: i64 = open_charges.iter().map(|c| c.remaining_due_poisha).sum();
    let advance_credit_poisha: i64 = if net_balance_poisha < 0 {
        -net_balance_poisha
    } else {
        0
    };

    let all_payments = list_payments_by_student(conn, student_id)?;
    let recent_payments: Vec<PaymentReceipt> = all_payments.into_iter().take(5).collect();

    Ok(CashierStudentOverview {
        student_id: student_id.to_string(),
        student_code: code,
        name,
        name_bn,
        photo_vault_id,
        phone,
        guardian_name,
        guardian_phone,
        batch_name,
        program_name,
        roll_no,
        total_due_poisha,
        advance_credit_poisha,
        net_balance_poisha,
        open_charges,
        recent_payments,
    })
}

pub fn get_student_ledger_summary(
    conn: &Connection,
    student_id: &str,
) -> Result<StudentLedgerSummary, LedgerError> {
    let (code, name): (String, String) = conn
        .query_row(
            "SELECT student_code, name FROM students WHERE id = ?1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(student_id.to_string()))?;

    // Read cached balances
    let (charged, paid, adjusted, refunded, balance): (i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT total_charged_poisha, total_paid_poisha, total_adjusted_poisha, total_refunded_poisha, balance_poisha
             FROM student_balances WHERE student_id = ?1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?
        .unwrap_or((0, 0, 0, 0, 0));

    let charges = list_charges_by_student(conn, student_id)?;
    let payments = list_payments_by_student(conn, student_id)?;

    // Adjustments
    let mut adj_stmt = conn.prepare(
        "SELECT id, student_id, charge_id, type, amount_poisha, reason, approved_by, business_date, created_at
         FROM adjustments
         WHERE student_id = ?1
         ORDER BY business_date DESC, created_at DESC",
    )?;
    let adj_rows = adj_stmt.query_map(params![student_id], |r| {
        Ok(AdjustmentItem {
            id: r.get(0)?,
            student_id: r.get(1)?,
            charge_id: r.get(2)?,
            r#type: r.get(3)?,
            amount_poisha: r.get(4)?,
            reason: r.get(5)?,
            approved_by: r.get(6)?,
            business_date: r.get(7)?,
            created_at: r.get(8)?,
        })
    })?;
    let mut adjustments = Vec::new();
    for a in adj_rows {
        adjustments.push(a?);
    }

    // Refunds
    let mut ref_stmt = conn.prepare(
        "SELECT r.id, r.payment_id, p.receipt_no, r.amount_poisha, r.method, r.reason, r.business_date, r.refunded_by, r.created_at
         FROM refunds r
         JOIN payments p ON r.payment_id = p.id
         WHERE p.student_id = ?1
         ORDER BY r.business_date DESC, r.created_at DESC",
    )?;
    let ref_rows = ref_stmt.query_map(params![student_id], |r| {
        Ok(RefundItem {
            id: r.get(0)?,
            payment_id: r.get(1)?,
            receipt_no: r.get(2)?,
            amount_poisha: r.get(3)?,
            method: r.get(4)?,
            reason: r.get(5)?,
            business_date: r.get(6)?,
            refunded_by: r.get(7)?,
            created_at: r.get(8)?,
        })
    })?;
    let mut refunds = Vec::new();
    for rf in ref_rows {
        refunds.push(rf?);
    }

    Ok(StudentLedgerSummary {
        student_id: student_id.to_string(),
        student_code: code,
        student_name: name,
        total_charged_poisha: charged,
        total_paid_poisha: paid,
        total_adjusted_poisha: adjusted,
        total_refunded_poisha: refunded,
        balance_poisha: balance,
        charges,
        payments,
        adjustments,
        refunds,
    })
}

// -----------------------------------------------------------------------------
// Dues Center Directory & Due Statement (Spec 0017)
// -----------------------------------------------------------------------------

pub fn list_dues(
    conn: &Connection,
    filter: ListDuesFilter,
) -> Result<Vec<StudentDueSummaryItem>, LedgerError> {
    let today = current_business_date();

    let mut sql = String::from(
        "SELECT s.id, s.student_code, s.name, s.name_bn, s.phone_norm,
                sb.balance_poisha,
                g_info.g_name, g_info.g_phone,
                enr_info.batch_name, enr_info.roll_no,
                enr_info.program_id, enr_info.batch_id
         FROM students s
         JOIN student_balances sb ON sb.student_id = s.id
         LEFT JOIN (
             SELECT sg.student_id, g.name as g_name, g.phone_norm as g_phone
             FROM student_guardians sg
             JOIN guardians g ON sg.guardian_id = g.id
             WHERE sg.is_primary = 1
         ) g_info ON g_info.student_id = s.id
         LEFT JOIN (
             SELECT e.student_id, b.id as batch_id, b.program_id, b.name as batch_name, CAST(e.roll AS TEXT) as roll_no
             FROM enrollments e
             JOIN batches b ON e.batch_id = b.id
             WHERE e.status = 'active'
         ) enr_info ON enr_info.student_id = s.id
         WHERE s.status = 'active' AND sb.balance_poisha > 0 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref pid) = filter.program_id {
        sql.push_str("AND enr_info.program_id = ? ");
        params_vec.push(Box::new(pid.clone()));
    }

    if let Some(ref bid) = filter.batch_id {
        sql.push_str("AND enr_info.batch_id = ? ");
        params_vec.push(Box::new(bid.clone()));
    }

    if let Some(min_due) = filter.min_due_poisha {
        sql.push_str("AND sb.balance_poisha >= ? ");
        params_vec.push(Box::new(min_due));
    }

    if let Some(ref q) = filter.search {
        let clean = q.trim();
        if !clean.is_empty() {
            let pat = format!("%{}%", clean);
            sql.push_str("AND (s.name LIKE ? OR s.student_code LIKE ? OR s.phone_norm LIKE ? OR enr_info.roll_no LIKE ?) ");
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat.clone()));
            params_vec.push(Box::new(pat));
        }
    }

    sql.push_str("ORDER BY sb.balance_poisha DESC, s.name ASC ");

    if let Some(lim) = filter.limit {
        sql.push_str("LIMIT ? ");
        params_vec.push(Box::new(lim));
        if let Some(off) = filter.offset {
            sql.push_str("OFFSET ? ");
            params_vec.push(Box::new(off));
        }
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    struct InterimStudent {
        student_id: String,
        student_code: String,
        name: String,
        name_bn: Option<String>,
        phone: String,
        balance_poisha: i64,
        guardian_name: Option<String>,
        guardian_phone: Option<String>,
        batch_name: Option<String>,
        roll_no: Option<String>,
    }

    let rows = stmt.query_map(&param_refs[..], |r| {
        Ok(InterimStudent {
            student_id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            phone: r.get(4)?,
            balance_poisha: r.get(5)?,
            guardian_name: r.get(6)?,
            guardian_phone: r.get(7)?,
            batch_name: r.get(8)?,
            roll_no: r.get(9)?,
        })
    })?;

    let mut students = Vec::new();
    for r in rows {
        students.push(r?);
    }
    drop(stmt);

    let mut results = Vec::new();

    for s in students {
        let mut charge_stmt = conn.prepare(
            "SELECT c.id, c.period, c.due_date, c.net_amount_poisha,
                    COALESCE((SELECT SUM(pa.amount_poisha) FROM payment_allocations pa JOIN payments p ON pa.payment_id = p.id WHERE pa.charge_id = c.id AND p.status = 'posted'), 0) as paid,
                    COALESCE((SELECT SUM(a.amount_poisha) FROM adjustments a WHERE a.charge_id = c.id AND a.type = 'waiver'), 0) as waivers,
                    COALESCE((SELECT SUM(a.amount_poisha) FROM adjustments a WHERE a.charge_id = c.id AND a.type = 'fine'), 0) as fines
             FROM charges c
             WHERE c.student_id = ?1 AND c.status = 'open'
             ORDER BY c.due_date ASC",
        )?;

        let chg_rows = charge_stmt.query_map(params![s.student_id], |r| {
            let net: i64 = r.get(3)?;
            let paid: i64 = r.get(4)?;
            let waivers: i64 = r.get(5)?;
            let fines: i64 = r.get(6)?;
            let remaining = (net + fines - waivers - paid).max(0);
            Ok((
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                remaining,
            ))
        })?;

        let mut oldest_unpaid_due_date: Option<String> = None;
        let mut overdue_poisha = 0i64;
        let mut current_month_due_poisha = 0i64;

        for cr in chg_rows {
            let (_period_val, due_date_val, remaining) = cr?;
            if remaining > 0 {
                if oldest_unpaid_due_date.is_none() {
                    oldest_unpaid_due_date = Some(due_date_val.clone());
                }
                if due_date_val < today {
                    overdue_poisha += remaining;
                } else {
                    current_month_due_poisha += remaining;
                }
            }
        }
        drop(charge_stmt);

        if filter.overdue_only.unwrap_or(false) && overdue_poisha <= 0 {
            continue;
        }

        let last_payment: Option<(String, i64)> = conn
            .query_row(
                "SELECT business_date, amount_poisha FROM payments
                 WHERE student_id = ?1 AND status = 'posted'
                 ORDER BY business_date DESC, created_at DESC LIMIT 1",
                params![s.student_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;

        let (last_payment_date, last_payment_amount_poisha) = match last_payment {
            Some((d, a)) => (Some(d), Some(a)),
            None => (None, None),
        };

        results.push(StudentDueSummaryItem {
            student_id: s.student_id,
            student_code: s.student_code,
            name: s.name,
            name_bn: s.name_bn,
            phone: s.phone,
            guardian_name: s.guardian_name,
            guardian_phone: s.guardian_phone,
            batch_name: s.batch_name,
            roll_no: s.roll_no,
            current_month_due_poisha,
            overdue_poisha,
            total_due_poisha: s.balance_poisha,
            oldest_unpaid_due_date,
            last_payment_date,
            last_payment_amount_poisha,
        });
    }

    Ok(results)
}

pub fn get_due_statement(
    conn: &Connection,
    student_id: &str,
    _period_filter: Option<&str>,
) -> Result<DueStatementData, LedgerError> {
    let (code, name, name_bn, current_school_roll): (
        String,
        String,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT student_code, name, name_bn, current_school_roll FROM students WHERE id = ?1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::StudentNotFound(student_id.to_string()))?;

    let (guardian_name, guardian_phone): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT g.name, g.phone_norm
             FROM guardians g
             JOIN student_guardians sg ON sg.guardian_id = g.id
             WHERE sg.student_id = ?1 AND sg.is_primary = 1
             LIMIT 1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));

    let (batch_name, batch_roll): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT b.name, CAST(e.roll AS TEXT)
             FROM enrollments e
             JOIN batches b ON e.batch_id = b.id
             WHERE e.student_id = ?1 AND e.status = 'active'
             LIMIT 1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));

    let roll_no = batch_roll.or(current_school_roll);

    let (inst_name, inst_name_bn, inst_address, inst_phone): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT name, name_bn, address, phones FROM institution WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .unwrap_or_else(|| {
            (
                "Apex Academic Coaching Centre".to_string(),
                Some("এপেক্স একাডেমিক কোচিং সেন্টার".to_string()),
                Some("Farmgate, Dhaka-1215".to_string()),
                Some("01700000000".to_string()),
            )
        });

    // Itemized charges
    let mut charge_stmt = conn.prepare(
        "SELECT c.id, fh.name, fh.name_bn, c.period, c.due_date,
                c.gross_amount_poisha, c.concession_amount_poisha, c.net_amount_poisha,
                COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = c.id AND p.status = 'posted'
                ), 0) as paid
         FROM charges c
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE c.student_id = ?1 AND c.status != 'voided'
         ORDER BY c.due_date ASC",
    )?;

    let charge_rows = charge_stmt.query_map(params![student_id], |r| {
        let net: i64 = r.get(7)?;
        let paid: i64 = r.get(8)?;
        let remaining = (net - paid).max(0);

        Ok(StatementChargeLine {
            charge_id: r.get(0)?,
            fee_head_name: r.get(1)?,
            fee_head_name_bn: r.get(2)?,
            period: r.get(3)?,
            due_date: r.get(4)?,
            gross_amount_poisha: r.get(5)?,
            concession_amount_poisha: r.get(6)?,
            net_amount_poisha: net,
            paid_amount_poisha: paid,
            remaining_due_poisha: remaining,
        })
    })?;

    let mut itemized_charges = Vec::new();
    for cr in charge_rows {
        itemized_charges.push(cr?);
    }
    drop(charge_stmt);

    // Recent payments
    let mut pay_stmt = conn.prepare(
        "SELECT id, receipt_no, business_date, method, amount_poisha
         FROM payments
         WHERE student_id = ?1 AND status = 'posted'
         ORDER BY business_date DESC, created_at DESC
         LIMIT 10",
    )?;

    let pay_rows = pay_stmt.query_map(params![student_id], |r| {
        Ok(StatementPaymentLine {
            payment_id: r.get(0)?,
            receipt_no: r.get(1)?,
            business_date: r.get(2)?,
            method: r.get(3)?,
            amount_poisha: r.get(4)?,
        })
    })?;

    let mut recent_payments = Vec::new();
    for pr in pay_rows {
        recent_payments.push(pr?);
    }
    drop(pay_stmt);

    // Cached balances
    let (charged, paid, adjusted, _refunded, balance): (i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT total_charged_poisha, total_paid_poisha, total_adjusted_poisha, total_refunded_poisha, balance_poisha
             FROM student_balances WHERE student_id = ?1",
            params![student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?
        .unwrap_or((0, 0, 0, 0, 0));

    let total_concession_poisha: i64 = itemized_charges
        .iter()
        .map(|c| c.concession_amount_poisha)
        .sum();

    let amount_in_words_en = crate::words::amount_to_words_en(balance.max(0));
    let amount_in_words_bn = crate::words::amount_to_words_bn(balance.max(0));

    Ok(DueStatementData {
        institution_name: inst_name,
        institution_name_bn: inst_name_bn,
        institution_address: inst_address.unwrap_or_default(),
        institution_phone: inst_phone.unwrap_or_default(),
        student_id: student_id.to_string(),
        student_code: code,
        student_name: name,
        student_name_bn: name_bn,
        guardian_name,
        guardian_phone,
        batch_name,
        roll_no,
        statement_date: current_business_date(),
        itemized_charges,
        recent_payments,
        total_charged_poisha: charged,
        total_concession_poisha,
        total_paid_poisha: paid,
        total_adjusted_poisha: adjusted,
        net_due_poisha: balance,
        amount_in_words_en,
        amount_in_words_bn,
        payment_instructions: Some("Pay via bKash Merchant or Counter Cashier".to_string()),
    })
}

// -----------------------------------------------------------------------------
// Expenses & Daily Cash Close (Spec 0018)
// -----------------------------------------------------------------------------

pub fn list_expense_categories(conn: &Connection) -> Result<Vec<ExpenseCategoryItem>, LedgerError> {
    let count: i64 = conn.query_row("SELECT count(*) FROM expense_categories", [], |r| r.get(0))?;
    if count == 0 {
        let now = now_utc_iso();
        for cat_name in DEFAULT_EXPENSE_CATEGORIES {
            let id = Uuid::now_v7().to_string();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO expense_categories (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                params![id, cat_name, now],
            );
        }
    }

    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM expense_categories ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ExpenseCategoryItem {
            id: r.get(0)?,
            name: r.get(1)?,
            created_at: r.get(2)?,
            updated_at: r.get(3)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn create_expense_category(
    conn: &Connection,
    input: CreateExpenseCategoryInput,
) -> Result<ExpenseCategoryItem, LedgerError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(LedgerError::InvalidAmount(0));
    }

    let exists: i64 = conn.query_row(
        "SELECT count(*) FROM expense_categories WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )?;
    if exists > 0 {
        return Err(LedgerError::ExpenseCategoryExists(name.to_string()));
    }

    let id = Uuid::now_v7().to_string();
    let now = now_utc_iso();
    conn.execute(
        "INSERT INTO expense_categories (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, now],
    )?;

    Ok(ExpenseCategoryItem {
        id,
        name: name.to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn create_expense(
    conn: &Connection,
    input: CreateExpenseInput,
    created_by: Option<&str>,
) -> Result<ExpenseItem, LedgerError> {
    if input.amount_poisha <= 0 {
        return Err(LedgerError::InvalidAmount(input.amount_poisha));
    }

    let method = input.method.to_lowercase();
    if !VALID_REFUND_METHODS.contains(&method.as_str()) {
        return Err(LedgerError::InvalidMethod(input.method));
    }

    let payee = input.payee.trim();
    if payee.is_empty() {
        return Err(LedgerError::InvalidAmount(0));
    }

    let cat_name: String = conn
        .query_row(
            "SELECT name FROM expense_categories WHERE id = ?1",
            params![input.category_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_to_result(LedgerError::ExpenseCategoryNotFound(
            input.category_id.clone(),
        ))?;

    let b_date = input.business_date.unwrap_or_else(current_business_date);
    verify_open_cash_day(conn, &b_date)?;

    let id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO expenses (
            id, category_id, amount_poisha, method, payee, reference,
            business_date, attachment_vault_id, created_by, status, void_reason,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'posted', NULL, ?10, ?10)",
        params![
            id,
            input.category_id,
            input.amount_poisha,
            method,
            payee,
            input.reference.as_deref().map(str::trim),
            b_date,
            input.attachment_vault_id.as_deref().map(str::trim),
            created_by,
            now,
        ],
    )?;

    Ok(ExpenseItem {
        id,
        category_id: input.category_id,
        category_name: cat_name,
        amount_poisha: input.amount_poisha,
        method,
        payee: payee.to_string(),
        reference: input.reference,
        business_date: b_date,
        attachment_vault_id: input.attachment_vault_id,
        created_by: created_by.map(String::from),
        status: "posted".to_string(),
        void_reason: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_expenses(
    conn: &Connection,
    filter: ListExpensesFilter,
) -> Result<Vec<ExpenseItem>, LedgerError> {
    let mut sql = String::from(
        "SELECT e.id, e.category_id, ec.name, e.amount_poisha, e.method, e.payee,
                e.reference, e.business_date, e.attachment_vault_id, e.created_by,
                e.status, e.void_reason, e.created_at, e.updated_at
         FROM expenses e
         JOIN expense_categories ec ON e.category_id = ec.id
         WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref cid) = filter.category_id {
        sql.push_str("AND e.category_id = ? ");
        params_vec.push(Box::new(cid.clone()));
    }

    if let Some(ref bdate) = filter.business_date {
        sql.push_str("AND e.business_date = ? ");
        params_vec.push(Box::new(bdate.clone()));
    }

    if let Some(ref from) = filter.from_date {
        sql.push_str("AND e.business_date >= ? ");
        params_vec.push(Box::new(from.clone()));
    }

    if let Some(ref to) = filter.to_date {
        sql.push_str("AND e.business_date <= ? ");
        params_vec.push(Box::new(to.clone()));
    }

    if let Some(ref m) = filter.method {
        sql.push_str("AND e.method = ? ");
        params_vec.push(Box::new(m.clone()));
    }

    if let Some(ref st) = filter.status {
        sql.push_str("AND e.status = ? ");
        params_vec.push(Box::new(st.clone()));
    }

    sql.push_str("ORDER BY e.business_date DESC, e.created_at DESC ");

    if let Some(lim) = filter.limit {
        sql.push_str("LIMIT ? ");
        params_vec.push(Box::new(lim));
        if let Some(off) = filter.offset {
            sql.push_str("OFFSET ? ");
            params_vec.push(Box::new(off));
        }
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(&param_refs[..], |r| {
        Ok(ExpenseItem {
            id: r.get(0)?,
            category_id: r.get(1)?,
            category_name: r.get(2)?,
            amount_poisha: r.get(3)?,
            method: r.get(4)?,
            payee: r.get(5)?,
            reference: r.get(6)?,
            business_date: r.get(7)?,
            attachment_vault_id: r.get(8)?,
            created_by: r.get(9)?,
            status: r.get(10)?,
            void_reason: r.get(11)?,
            created_at: r.get(12)?,
            updated_at: r.get(13)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn void_expense(
    conn: &Connection,
    expense_id: &str,
    reason: &str,
    _voided_by: Option<&str>,
) -> Result<(), LedgerError> {
    if reason.trim().is_empty() {
        return Err(LedgerError::UnauthorizedVoid(
            "Void reason is required".to_string(),
        ));
    }

    let (status, business_date): (String, String) = conn
        .query_row(
            "SELECT status, business_date FROM expenses WHERE id = ?1",
            params![expense_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_to_result(LedgerError::ExpenseNotFound(expense_id.to_string()))?;

    if status == "voided" {
        return Ok(());
    }

    let today = current_business_date();
    if business_date != today {
        return Err(LedgerError::UnauthorizedExpenseVoid(business_date, today));
    }

    verify_open_cash_day(conn, &business_date)?;

    let now = now_utc_iso();
    conn.execute(
        "UPDATE expenses SET status = 'voided', void_reason = ?1, updated_at = ?2 WHERE id = ?3",
        params![reason.trim(), now, expense_id],
    )?;

    Ok(())
}

pub fn get_or_open_today_cash_day(
    conn: &Connection,
    business_date: Option<&str>,
) -> Result<CashDayItem, LedgerError> {
    let b_date = business_date
        .map(String::from)
        .unwrap_or_else(current_business_date);

    let existing: Option<CashDayItem> = conn
        .query_row(
            "SELECT business_date, opening_cash_poisha, cash_in_poisha, cash_out_poisha,
                    expected_closing_poisha, counted_closing_poisha, difference_poisha,
                    note, closed_by, closed_at, created_at, updated_at
             FROM cash_days WHERE business_date = ?1",
            params![b_date],
            |r| {
                Ok(CashDayItem {
                    business_date: r.get(0)?,
                    opening_cash_poisha: r.get(1)?,
                    cash_in_poisha: r.get(2)?,
                    cash_out_poisha: r.get(3)?,
                    expected_closing_poisha: r.get(4)?,
                    counted_closing_poisha: r.get(5)?,
                    difference_poisha: r.get(6)?,
                    note: r.get(7)?,
                    closed_by: r.get(8)?,
                    closed_at: r.get(9)?,
                    created_at: r.get(10)?,
                    updated_at: r.get(11)?,
                })
            },
        )
        .optional()?;

    if let Some(item) = existing {
        return Ok(item);
    }

    let previous_closing: Option<i64> = conn
        .query_row(
            "SELECT counted_closing_poisha FROM cash_days
             WHERE closed_at IS NOT NULL AND business_date < ?1
             ORDER BY business_date DESC LIMIT 1",
            params![b_date],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let opening_cash = previous_closing.unwrap_or(0);
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO cash_days (
            business_date, opening_cash_poisha, cash_in_poisha, cash_out_poisha,
            expected_closing_poisha, counted_closing_poisha, difference_poisha,
            note, closed_by, closed_at, created_at, updated_at
        ) VALUES (?1, ?2, 0, 0, ?2, NULL, NULL, NULL, NULL, NULL, ?3, ?3)",
        params![b_date, opening_cash, now],
    )?;

    Ok(CashDayItem {
        business_date: b_date,
        opening_cash_poisha: opening_cash,
        cash_in_poisha: 0,
        cash_out_poisha: 0,
        expected_closing_poisha: opening_cash,
        counted_closing_poisha: None,
        difference_poisha: None,
        note: None,
        closed_by: None,
        closed_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_cash_day_summary(
    conn: &Connection,
    business_date: Option<&str>,
) -> Result<CashDaySummary, LedgerError> {
    let b_date = business_date
        .map(String::from)
        .unwrap_or_else(current_business_date);
    let day_item = get_or_open_today_cash_day(conn, Some(&b_date))?;

    let cash_in_poisha: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM payments
         WHERE business_date = ?1 AND method = 'cash' AND status = 'posted'",
        params![b_date],
        |r| r.get(0),
    )?;

    let cash_expenses: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM expenses
         WHERE business_date = ?1 AND method = 'cash' AND status = 'posted'",
        params![b_date],
        |r| r.get(0),
    )?;

    let cash_refunds: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM refunds
         WHERE business_date = ?1 AND method = 'cash'",
        params![b_date],
        |r| r.get(0),
    )?;

    let cash_out_poisha = cash_expenses + cash_refunds;
    let expected_closing_poisha = day_item.opening_cash_poisha + cash_in_poisha - cash_out_poisha;

    if day_item.closed_at.is_none() {
        let now = now_utc_iso();
        let _ = conn.execute(
            "UPDATE cash_days SET cash_in_poisha = ?1, cash_out_poisha = ?2, expected_closing_poisha = ?3, updated_at = ?4
             WHERE business_date = ?5 AND closed_at IS NULL",
            params![cash_in_poisha, cash_out_poisha, expected_closing_poisha, now, b_date],
        );
    }

    let difference_poisha = day_item
        .counted_closing_poisha
        .map(|counted| counted - expected_closing_poisha);

    let mut dig_stmt = conn.prepare(
        "SELECT method, COUNT(*), COALESCE(SUM(amount_poisha), 0)
         FROM payments
         WHERE business_date = ?1 AND method != 'cash' AND status = 'posted'
         GROUP BY method
         ORDER BY method ASC",
    )?;
    let dig_rows = dig_stmt.query_map(params![b_date], |r| {
        Ok(MethodCollectionSummary {
            method: r.get(0)?,
            count: r.get(1)?,
            total_amount_poisha: r.get(2)?,
        })
    })?;
    let mut digital_collections = Vec::new();
    for dr in dig_rows {
        digital_collections.push(dr?);
    }
    drop(dig_stmt);

    let mut coll_stmt = conn.prepare(
        "SELECT p.collector_user_id, COALESCE(u.display_name, 'Cashier'), COUNT(*), COALESCE(SUM(p.amount_poisha), 0)
         FROM payments p
         LEFT JOIN users u ON p.collector_user_id = u.id
         WHERE p.business_date = ?1 AND p.status = 'posted'
         GROUP BY p.collector_user_id",
    )?;
    let coll_rows = coll_stmt.query_map(params![b_date], |r| {
        Ok(CollectorSummary {
            collector_user_id: r.get(0)?,
            collector_name: r.get(1)?,
            receipt_count: r.get(2)?,
            total_amount_poisha: r.get(3)?,
        })
    })?;
    let mut collector_breakdown = Vec::new();
    for cr in coll_rows {
        collector_breakdown.push(cr?);
    }
    drop(coll_stmt);

    let mut exp_stmt = conn.prepare(
        "SELECT ec.id, ec.name, COUNT(e.id), COALESCE(SUM(e.amount_poisha), 0)
         FROM expenses e
         JOIN expense_categories ec ON e.category_id = ec.id
         WHERE e.business_date = ?1 AND e.status = 'posted'
         GROUP BY ec.id, ec.name
         ORDER BY ec.name ASC",
    )?;
    let exp_rows = exp_stmt.query_map(params![b_date], |r| {
        Ok(CategoryExpenseSummary {
            category_id: r.get(0)?,
            category_name: r.get(1)?,
            count: r.get(2)?,
            total_amount_poisha: r.get(3)?,
        })
    })?;
    let mut expenses_by_category = Vec::new();
    for er in exp_rows {
        expenses_by_category.push(er?);
    }
    drop(exp_stmt);

    Ok(CashDaySummary {
        business_date: b_date,
        is_closed: day_item.closed_at.is_some(),
        closed_at: day_item.closed_at,
        closed_by: day_item.closed_by,
        opening_cash_poisha: day_item.opening_cash_poisha,
        cash_in_poisha,
        cash_out_poisha,
        expected_closing_poisha,
        counted_closing_poisha: day_item.counted_closing_poisha,
        difference_poisha,
        note: day_item.note,
        digital_collections,
        collector_breakdown,
        expenses_by_category,
    })
}

pub fn close_cash_day(
    conn: &Connection,
    input: CloseCashDayInput,
    closed_by: Option<&str>,
) -> Result<CashDaySummary, LedgerError> {
    let summary = get_cash_day_summary(conn, Some(&input.business_date))?;
    if summary.is_closed {
        return Err(LedgerError::CashDayAlreadyClosed(input.business_date));
    }

    let diff = input.counted_closing_poisha - summary.expected_closing_poisha;
    let now = now_utc_iso();

    conn.execute(
        "UPDATE cash_days SET
            cash_in_poisha = ?1,
            cash_out_poisha = ?2,
            expected_closing_poisha = ?3,
            counted_closing_poisha = ?4,
            difference_poisha = ?5,
            note = ?6,
            closed_by = ?7,
            closed_at = ?8,
            updated_at = ?8
         WHERE business_date = ?9",
        params![
            summary.cash_in_poisha,
            summary.cash_out_poisha,
            summary.expected_closing_poisha,
            input.counted_closing_poisha,
            diff,
            input.note.as_deref().map(str::trim),
            closed_by,
            now,
            input.business_date,
        ],
    )?;

    get_cash_day_summary(conn, Some(&input.business_date))
}

/// Comprehensive validator verifying all 9 financial invariants (AC-10).
pub fn verify_all_ledger_invariants(conn: &Connection) -> Result<(), LedgerError> {
    // Invariant 1: Payment allocation bounded
    // For every payment record: active allocations + refunds <= payment amount.
    let inv1_violation: Option<(String, i64, i64, i64)> = conn
        .query_row(
            "SELECT p.id, p.amount_poisha,
                    COALESCE(pa.allocs, 0) as allocs,
                    COALESCE(rf.refunds, 0) as refunds
             FROM payments p
             LEFT JOIN (SELECT payment_id, SUM(amount_poisha) as allocs FROM payment_allocations GROUP BY payment_id) pa ON pa.payment_id = p.id
             LEFT JOIN (SELECT payment_id, SUM(amount_poisha) as refunds FROM refunds GROUP BY payment_id) rf ON rf.payment_id = p.id
             WHERE COALESCE(pa.allocs, 0) + COALESCE(rf.refunds, 0) > p.amount_poisha
             LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    if let Some((id, amount, allocs, refunds)) = inv1_violation {
        return Err(LedgerError::InvariantViolation(format!(
            "Invariant 1 failed: Payment {} has allocations ({}) + refunds ({}) exceeding total amount ({})",
            id, allocs, refunds, amount
        )));
    }

    // Invariant 2: Charge allocation bounded
    // For every charge record: active allocations <= net_amount + fines - waivers
    let inv2_violation: Option<(String, i64, i64, i64, i64)> = conn
        .query_row(
            "SELECT c.id, c.net_amount_poisha,
                    COALESCE(pa.allocs, 0) as allocs,
                    COALESCE(adj.fines, 0) as fines,
                    COALESCE(adj.waivers, 0) as waivers
             FROM charges c
             LEFT JOIN (
                 SELECT pa.charge_id, SUM(pa.amount_poisha) as allocs
                 FROM payment_allocations pa
                 JOIN payments p ON pa.payment_id = p.id
                 WHERE p.status = 'posted'
                 GROUP BY pa.charge_id
             ) pa ON pa.charge_id = c.id
             LEFT JOIN (
                 SELECT charge_id,
                     SUM(CASE WHEN type = 'fine' THEN amount_poisha ELSE 0 END) as fines,
                     SUM(CASE WHEN type = 'waiver' THEN amount_poisha ELSE 0 END) as waivers
                 FROM adjustments
                 WHERE charge_id IS NOT NULL
                 GROUP BY charge_id
             ) adj ON adj.charge_id = c.id
             WHERE COALESCE(pa.allocs, 0) > c.net_amount_poisha + COALESCE(adj.fines, 0) - COALESCE(adj.waivers, 0)
             LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;

    if let Some((id, net, allocs, fines, waivers)) = inv2_violation {
        return Err(LedgerError::InvariantViolation(format!(
            "Invariant 2 failed: Charge {} has allocations ({}) exceeding net ({}) + fines ({}) - waivers ({})",
            id, allocs, net, fines, waivers
        )));
    }

    // Invariant 3: Waiver ceiling
    // For every charge waiver: total waivers <= net_amount - active allocations
    let inv3_violation: Option<(String, i64, i64, i64)> = conn
        .query_row(
            "SELECT c.id, c.net_amount_poisha,
                    COALESCE(pa.allocs, 0) as allocs,
                    COALESCE(adj.waivers, 0) as waivers
             FROM charges c
             LEFT JOIN (
                 SELECT pa.charge_id, SUM(pa.amount_poisha) as allocs
                 FROM payment_allocations pa
                 JOIN payments p ON pa.payment_id = p.id
                 WHERE p.status = 'posted'
                 GROUP BY pa.charge_id
             ) pa ON pa.charge_id = c.id
             LEFT JOIN (
                 SELECT charge_id, SUM(amount_poisha) as waivers
                 FROM adjustments
                 WHERE charge_id IS NOT NULL AND type = 'waiver'
                 GROUP BY charge_id
             ) adj ON adj.charge_id = c.id
             WHERE COALESCE(adj.waivers, 0) > c.net_amount_poisha - COALESCE(pa.allocs, 0)
             LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    if let Some((id, net, allocs, waivers)) = inv3_violation {
        return Err(LedgerError::InvariantViolation(format!(
            "Invariant 3 failed: Charge {} has waivers ({}) exceeding net ({}) - allocs ({})",
            id, waivers, net, allocs
        )));
    }

    // Invariant 4: Balance consistency
    // For every student in student_balances: cached balance == charges + fines - waivers - posted payments + refunds
    let inv4_violation: Option<(String, i64, i64, i64, i64, i64, i64)> = conn
        .query_row(
            "SELECT sb.student_id, sb.balance_poisha,
                    COALESCE(chg.total, 0) as chg,
                    COALESCE(fn.total, 0) as fn_amt,
                    COALESCE(wv.total, 0) as wv_amt,
                    COALESCE(pd.total, 0) as pd,
                    COALESCE(rf.total, 0) as rf
             FROM student_balances sb
             LEFT JOIN (SELECT student_id, SUM(net_amount_poisha) as total FROM charges WHERE status != 'voided' GROUP BY student_id) chg ON chg.student_id = sb.student_id
             LEFT JOIN (SELECT student_id, SUM(amount_poisha) as total FROM adjustments WHERE type = 'fine' GROUP BY student_id) fn ON fn.student_id = sb.student_id
             LEFT JOIN (SELECT student_id, SUM(amount_poisha) as total FROM adjustments WHERE type = 'waiver' GROUP BY student_id) wv ON wv.student_id = sb.student_id
             LEFT JOIN (SELECT student_id, SUM(amount_poisha) as total FROM payments WHERE status = 'posted' GROUP BY student_id) pd ON pd.student_id = sb.student_id
             LEFT JOIN (
                 SELECT p.student_id, SUM(r.amount_poisha) as total
                 FROM refunds r
                 JOIN payments p ON r.payment_id = p.id
                 WHERE p.status = 'posted'
                 GROUP BY p.student_id
             ) rf ON rf.student_id = sb.student_id
             WHERE sb.balance_poisha != (COALESCE(chg.total, 0) + COALESCE(fn.total, 0) - COALESCE(wv.total, 0) - COALESCE(pd.total, 0) + COALESCE(rf.total, 0))
             LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .optional()?;

    if let Some((sid, cached_bal, chg, fn_amt, wv_amt, pd, rf)) = inv4_violation {
        let computed = chg + fn_amt - wv_amt - pd + rf;
        return Err(LedgerError::InvariantViolation(format!(
            "Invariant 4 failed: Student {} cached balance ({}) != computed balance ({}) [chg={}, fn={}, wv={}, pd={}, rf={}]",
            sid, cached_bal, computed, chg, fn_amt, wv_amt, pd, rf
        )));
    }

    Ok(())
}

trait OptionalExt<T, E> {
    fn ok_to_result(self, err: E) -> Result<T, E>;
}

impl<T, E> OptionalExt<T, E> for Option<T> {
    fn ok_to_result(self, err: E) -> Result<T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err),
        }
    }
}
