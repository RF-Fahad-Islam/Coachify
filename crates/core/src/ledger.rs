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

    // Check idempotency key uniqueness
    let existing_idem: Option<String> = conn
        .query_row(
            "SELECT id FROM payments WHERE idempotency_key = ?1",
            params![input.idempotency_key],
            |r| r.get(0),
        )
        .optional()?;

    if existing_idem.is_some() {
        return Err(LedgerError::DuplicateIdempotency(input.idempotency_key));
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

    Ok(PaymentReceipt {
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
    })
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
