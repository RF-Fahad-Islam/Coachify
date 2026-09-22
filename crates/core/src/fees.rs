use crate::db::now_utc_iso;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FeesError {
    #[error("Fee head with ID '{0}' not found")]
    FeeHeadNotFound(String),
    #[error("Fee head name '{0}' already exists")]
    DuplicateName(String),
    #[error("Cannot delete fee head '{0}' because it is referenced by existing plans or charges")]
    FeeHeadReferenced(String),
    #[error("Invalid fee head kind '{0}'. Must be one of: tuition, admission, exam, registration, material, fine, other")]
    InvalidKind(String),
    #[error("Fee plan with ID '{0}' not found")]
    FeePlanNotFound(String),
    #[error("Invalid scope: fee plan must be scoped to either a program or a batch, not both or neither")]
    InvalidScope(String),
    #[error("Invalid amount {0}: amount in poisha must be non-negative")]
    InvalidAmount(i64),
    #[error("Invalid frequency '{0}'. Must be 'once' or 'monthly'")]
    InvalidFrequency(String),
    #[error(
        "Invalid date range: effective_to ({to}) cannot be earlier than effective_from ({from})"
    )]
    InvalidDateRange { from: String, to: String },
    #[error("Fee plan overlaps with an existing plan for the same fee head and program/batch")]
    OverlappingPlan(String),
    #[error("Program with ID '{0}' not found")]
    ProgramNotFound(String),
    #[error("Batch with ID '{0}' not found")]
    BatchNotFound(String),
    #[error("Student with ID '{0}' not found")]
    StudentNotFound(String),
    #[error("Enrollment with ID '{0}' not found")]
    EnrollmentNotFound(String),
    #[error("Invalid concession type '{0}'. Must be 'percent' or 'fixed'")]
    InvalidConcessionType(String),
    #[error("Invalid concession value {0}: percentage basis points must be 1-10000, fixed poisha must be positive")]
    InvalidValue(i64),
    #[error("Active concession already exists for this student, fee head, and batch")]
    ConcessionConflict(String),
    #[error("Concession with ID '{0}' not found")]
    ConcessionNotFound(String),
    #[error("Invalid billing period '{0}'. Expected format YYYY-MM (e.g. '2026-09')")]
    InvalidPeriod(String),
    #[error("Database error: {0}")]
    Sql(String),
}

impl FeesError {
    pub fn code(&self) -> &'static str {
        match self {
            FeesError::FeeHeadNotFound(_) => "fee_head_not_found",
            FeesError::DuplicateName(_) => "duplicate_name",
            FeesError::FeeHeadReferenced(_) => "fee_head_referenced",
            FeesError::InvalidKind(_) => "invalid_kind",
            FeesError::FeePlanNotFound(_) => "fee_plan_not_found",
            FeesError::InvalidScope(_) => "invalid_scope",
            FeesError::InvalidAmount(_) => "invalid_amount",
            FeesError::InvalidFrequency(_) => "invalid_frequency",
            FeesError::InvalidDateRange { .. } => "invalid_date_range",
            FeesError::OverlappingPlan(_) => "overlapping_plan",
            FeesError::ProgramNotFound(_) => "program_not_found",
            FeesError::BatchNotFound(_) => "batch_not_found",
            FeesError::StudentNotFound(_) => "student_not_found",
            FeesError::EnrollmentNotFound(_) => "enrollment_not_found",
            FeesError::InvalidConcessionType(_) => "invalid_concession_type",
            FeesError::InvalidValue(_) => "invalid_value",
            FeesError::ConcessionConflict(_) => "concession_conflict",
            FeesError::ConcessionNotFound(_) => "concession_not_found",
            FeesError::InvalidPeriod(_) => "invalid_period",
            FeesError::Sql(_) => "database_error",
        }
    }
}

impl From<rusqlite::Error> for FeesError {
    fn from(err: rusqlite::Error) -> Self {
        FeesError::Sql(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// DTOs & Domain Models
// -----------------------------------------------------------------------------

pub const VALID_FEE_HEAD_KINDS: &[&str] = &[
    "tuition",
    "admission",
    "exam",
    "registration",
    "material",
    "fine",
    "other",
];

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FeeHeadItem {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub kind: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFeeHeadInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateFeeHeadInput {
    pub name: Option<String>,
    pub name_bn: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FeePlanItem {
    pub id: String,
    pub program_id: Option<String>,
    pub program_name: Option<String>,
    pub batch_id: Option<String>,
    pub batch_name: Option<String>,
    pub fee_head_id: String,
    pub fee_head_name: String,
    pub fee_head_kind: String,
    pub amount_poisha: i64,
    pub frequency: String,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateFeePlanInput {
    pub program_id: Option<String>,
    pub batch_id: Option<String>,
    pub fee_head_id: String,
    pub amount_poisha: i64,
    pub frequency: String,
    pub effective_from: String,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateFeePlanInput {
    pub amount_poisha: Option<i64>,
    pub frequency: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ConcessionItem {
    pub id: String,
    pub student_id: String,
    pub student_name: String,
    pub student_code: String,
    pub batch_id: Option<String>,
    pub batch_name: Option<String>,
    pub fee_head_id: String,
    pub fee_head_name: String,
    pub r#type: String, // "percent" | "fixed"
    pub value: i64,
    pub from_period: Option<String>,
    pub to_period: Option<String>,
    pub reason: String,
    pub approved_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateConcessionInput {
    pub student_id: String,
    pub batch_id: Option<String>,
    pub fee_head_id: String,
    pub r#type: String, // "percent" | "fixed"
    pub value: i64,
    pub from_period: Option<String>,
    pub to_period: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GenerationResult {
    pub period: String,
    pub total_students_processed: usize,
    pub generated_count: usize,
    pub skipped_count: usize,
    pub total_gross_poisha: i64,
    pub total_concession_poisha: i64,
    pub total_net_poisha: i64,
    pub allocated_credit_poisha: i64,
}

// -----------------------------------------------------------------------------
// Math & Priority Helpers
// -----------------------------------------------------------------------------

/// Computes concession discount in poisha with half-up rounding for percentage basis points.
/// Returns discount clamped to gross_amount_poisha.
pub fn calculate_concession(gross_poisha: i64, concession_type: &str, value: i64) -> i64 {
    if gross_poisha <= 0 || value <= 0 {
        return 0;
    }

    let discount = match concession_type {
        "percent" => {
            // value is in basis points: 10000 bps = 100%, 1500 bps = 15%, 3333 bps = 33.33%
            // Half-up rounding: (gross * value + 5000) / 10000
            let num = (gross_poisha as i128) * (value as i128) + 5000;
            (num / 10000) as i64
        }
        "fixed" => value,
        _ => 0,
    };

    discount.min(gross_poisha)
}

/// Fee head hierarchy priority index for automatic advance credit allocation (AC-6):
/// 1: tuition, 2: admission, 3: exam, 4: registration, 5: material, 6: other, 7: fine
pub fn fee_head_priority(kind: &str) -> u8 {
    match kind {
        "tuition" => 1,
        "admission" => 2,
        "exam" => 3,
        "registration" => 4,
        "material" => 5,
        "other" => 6,
        "fine" => 7,
        _ => 8,
    }
}

pub fn validate_period(period: &str) -> Result<(), FeesError> {
    let parts: Vec<&str> = period.split('-').collect();
    if parts.len() != 2 || parts[0].len() != 4 || parts[1].len() != 2 {
        return Err(FeesError::InvalidPeriod(period.to_string()));
    }
    let year: u32 = parts[0]
        .parse()
        .map_err(|_| FeesError::InvalidPeriod(period.to_string()))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| FeesError::InvalidPeriod(period.to_string()))?;
    if !(1900..=2200).contains(&year) || !(1..=12).contains(&month) {
        return Err(FeesError::InvalidPeriod(period.to_string()));
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Fee Heads Repository
// -----------------------------------------------------------------------------

pub fn list_fee_heads(conn: &Connection) -> Result<Vec<FeeHeadItem>, FeesError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, name_bn, kind, created_at, updated_at
         FROM fee_heads
         ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(FeeHeadItem {
            id: row.get(0)?,
            name: row.get(1)?,
            name_bn: row.get(2)?,
            kind: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn get_fee_head(conn: &Connection, id: &str) -> Result<FeeHeadItem, FeesError> {
    conn.query_row(
        "SELECT id, name, name_bn, kind, created_at, updated_at
         FROM fee_heads
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(FeeHeadItem {
                id: row.get(0)?,
                name: row.get(1)?,
                name_bn: row.get(2)?,
                kind: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    )
    .optional()?
    .ok_to_result(FeesError::FeeHeadNotFound(id.to_string()))
}

pub fn create_fee_head(
    conn: &Connection,
    input: CreateFeeHeadInput,
) -> Result<FeeHeadItem, FeesError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(FeesError::DuplicateName("Name cannot be empty".to_string()));
    }

    let kind = input.kind.to_lowercase();
    if !VALID_FEE_HEAD_KINDS.contains(&kind.as_str()) {
        return Err(FeesError::InvalidKind(input.kind));
    }

    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM fee_heads WHERE lower(name) = lower(?1)",
            params![name],
            |row| row.get(0),
        )
        .optional()?;

    if existing.is_some() {
        return Err(FeesError::DuplicateName(name.to_string()));
    }

    let id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO fee_heads (id, name, name_bn, kind, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, input.name_bn, kind, now, now],
    )?;

    get_fee_head(conn, &id)
}

pub fn update_fee_head(
    conn: &Connection,
    id: &str,
    input: UpdateFeeHeadInput,
) -> Result<FeeHeadItem, FeesError> {
    let current = get_fee_head(conn, id)?;

    let name = input
        .name
        .map(|n| n.trim().to_string())
        .unwrap_or(current.name);
    if name.is_empty() {
        return Err(FeesError::DuplicateName("Name cannot be empty".to_string()));
    }

    let kind = input.kind.map(|k| k.to_lowercase()).unwrap_or(current.kind);
    if !VALID_FEE_HEAD_KINDS.contains(&kind.as_str()) {
        return Err(FeesError::InvalidKind(kind));
    }

    // Name uniqueness check if changed
    let conflict: Option<String> = conn
        .query_row(
            "SELECT id FROM fee_heads WHERE lower(name) = lower(?1) AND id != ?2",
            params![name, id],
            |row| row.get(0),
        )
        .optional()?;

    if conflict.is_some() {
        return Err(FeesError::DuplicateName(name));
    }

    let name_bn = input.name_bn.or(current.name_bn);
    let now = now_utc_iso();

    conn.execute(
        "UPDATE fee_heads SET name = ?1, name_bn = ?2, kind = ?3, updated_at = ?4
         WHERE id = ?5",
        params![name, name_bn, kind, now, id],
    )?;

    get_fee_head(conn, id)
}

pub fn delete_fee_head(conn: &Connection, id: &str) -> Result<(), FeesError> {
    let _ = get_fee_head(conn, id)?;

    // Check references in fee_plans
    let plan_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM fee_plans WHERE fee_head_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if plan_count > 0 {
        return Err(FeesError::FeeHeadReferenced(id.to_string()));
    }

    // Check references in charges
    let charge_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM charges WHERE fee_head_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if charge_count > 0 {
        return Err(FeesError::FeeHeadReferenced(id.to_string()));
    }

    // Check references in concessions
    let concession_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM concessions WHERE fee_head_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    if concession_count > 0 {
        return Err(FeesError::FeeHeadReferenced(id.to_string()));
    }

    conn.execute("DELETE FROM fee_heads WHERE id = ?1", params![id])?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Fee Plans Repository
// -----------------------------------------------------------------------------

pub fn list_fee_plans(
    conn: &Connection,
    program_id: Option<&str>,
    batch_id: Option<&str>,
) -> Result<Vec<FeePlanItem>, FeesError> {
    if program_id.is_some() && batch_id.is_some() {
        return Err(FeesError::InvalidScope(
            "Filter by either program_id or batch_id, not both".to_string(),
        ));
    }

    let mut query = String::from(
        "SELECT fp.id, fp.program_id, p.name, fp.batch_id, b.name,
                fp.fee_head_id, fh.name, fh.kind, fp.amount_poisha,
                fp.frequency, fp.effective_from, fp.effective_to,
                fp.created_at, fp.updated_at
         FROM fee_plans fp
         LEFT JOIN programs p ON fp.program_id = p.id
         LEFT JOIN batches b ON fp.batch_id = b.id
         JOIN fee_heads fh ON fp.fee_head_id = fh.id
         WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(pid) = program_id {
        query.push_str("AND fp.program_id = ? ");
        params_vec.push(Box::new(pid.to_string()));
    } else if let Some(bid) = batch_id {
        query.push_str("AND fp.batch_id = ? ");
        params_vec.push(Box::new(bid.to_string()));
    }

    query.push_str("ORDER BY fp.effective_from DESC, fh.name ASC");

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(FeePlanItem {
            id: row.get(0)?,
            program_id: row.get(1)?,
            program_name: row.get(2)?,
            batch_id: row.get(3)?,
            batch_name: row.get(4)?,
            fee_head_id: row.get(5)?,
            fee_head_name: row.get(6)?,
            fee_head_kind: row.get(7)?,
            amount_poisha: row.get(8)?,
            frequency: row.get(9)?,
            effective_from: row.get(10)?,
            effective_to: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn get_fee_plan(conn: &Connection, id: &str) -> Result<FeePlanItem, FeesError> {
    conn.query_row(
        "SELECT fp.id, fp.program_id, p.name, fp.batch_id, b.name,
                fp.fee_head_id, fh.name, fh.kind, fp.amount_poisha,
                fp.frequency, fp.effective_from, fp.effective_to,
                fp.created_at, fp.updated_at
         FROM fee_plans fp
         LEFT JOIN programs p ON fp.program_id = p.id
         LEFT JOIN batches b ON fp.batch_id = b.id
         JOIN fee_heads fh ON fp.fee_head_id = fh.id
         WHERE fp.id = ?1",
        params![id],
        |row| {
            Ok(FeePlanItem {
                id: row.get(0)?,
                program_id: row.get(1)?,
                program_name: row.get(2)?,
                batch_id: row.get(3)?,
                batch_name: row.get(4)?,
                fee_head_id: row.get(5)?,
                fee_head_name: row.get(6)?,
                fee_head_kind: row.get(7)?,
                amount_poisha: row.get(8)?,
                frequency: row.get(9)?,
                effective_from: row.get(10)?,
                effective_to: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        },
    )
    .optional()?
    .ok_to_result(FeesError::FeePlanNotFound(id.to_string()))
}

fn check_plan_overlap(
    conn: &Connection,
    plan_id: Option<&str>,
    program_id: Option<&str>,
    batch_id: Option<&str>,
    fee_head_id: &str,
    from: &str,
    to: Option<&str>,
) -> Result<(), FeesError> {
    let to_val = to.unwrap_or("9999-12-31");

    let query = if program_id.is_some() {
        "SELECT id, effective_from, effective_to FROM fee_plans
         WHERE program_id = ?1 AND fee_head_id = ?2 AND (?3 IS NULL OR id != ?3)"
    } else {
        "SELECT id, effective_from, effective_to FROM fee_plans
         WHERE batch_id = ?1 AND fee_head_id = ?2 AND (?3 IS NULL OR id != ?3)"
    };

    let scope_id = program_id.or(batch_id).unwrap();
    let mut stmt = conn.prepare(query)?;
    let rows = stmt.query_map(params![scope_id, fee_head_id, plan_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })?;

    for r in rows {
        let (existing_id, ex_from, ex_to) = r?;
        let ex_to_val = ex_to.as_deref().unwrap_or("9999-12-31");
        // Overlap: from <= ex_to_val && to_val >= ex_from
        if from <= ex_to_val && to_val >= ex_from.as_str() {
            return Err(FeesError::OverlappingPlan(format!(
                "Overlaps with existing plan '{}' (from {} to {})",
                existing_id, ex_from, ex_to_val
            )));
        }
    }
    Ok(())
}

pub fn create_fee_plan(
    conn: &Connection,
    input: CreateFeePlanInput,
) -> Result<FeePlanItem, FeesError> {
    match (&input.program_id, &input.batch_id) {
        (Some(_), None) => {}
        (None, Some(_)) => {}
        _ => {
            return Err(FeesError::InvalidScope(
                "Fee plan must specify either program_id or batch_id, but not both or neither"
                    .to_string(),
            ))
        }
    }

    if input.amount_poisha < 0 {
        return Err(FeesError::InvalidAmount(input.amount_poisha));
    }

    let freq = input.frequency.to_lowercase();
    if freq != "once" && freq != "monthly" {
        return Err(FeesError::InvalidFrequency(input.frequency));
    }

    if let Some(ref to) = input.effective_to {
        if to.as_str() < input.effective_from.as_str() {
            return Err(FeesError::InvalidDateRange {
                from: input.effective_from,
                to: to.clone(),
            });
        }
    }

    // Verify fee head exists
    let _ = get_fee_head(conn, &input.fee_head_id)?;

    // Verify program or batch exists
    if let Some(ref pid) = input.program_id {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM programs WHERE id = ?1",
            params![pid],
            |r| r.get(0),
        )?;
        if count == 0 {
            return Err(FeesError::ProgramNotFound(pid.clone()));
        }
    } else if let Some(ref bid) = input.batch_id {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM batches WHERE id = ?1",
            params![bid],
            |r| r.get(0),
        )?;
        if count == 0 {
            return Err(FeesError::BatchNotFound(bid.clone()));
        }
    }

    // Check date overlap
    check_plan_overlap(
        conn,
        None,
        input.program_id.as_deref(),
        input.batch_id.as_deref(),
        &input.fee_head_id,
        &input.effective_from,
        input.effective_to.as_deref(),
    )?;

    let id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO fee_plans (id, program_id, batch_id, fee_head_id, amount_poisha, frequency, effective_from, effective_to, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.program_id,
            input.batch_id,
            input.fee_head_id,
            input.amount_poisha,
            freq,
            input.effective_from,
            input.effective_to,
            now,
            now
        ],
    )?;

    get_fee_plan(conn, &id)
}

pub fn update_fee_plan(
    conn: &Connection,
    id: &str,
    input: UpdateFeePlanInput,
) -> Result<FeePlanItem, FeesError> {
    let current = get_fee_plan(conn, id)?;

    let amount = input.amount_poisha.unwrap_or(current.amount_poisha);
    if amount < 0 {
        return Err(FeesError::InvalidAmount(amount));
    }

    let freq = input
        .frequency
        .map(|f| f.to_lowercase())
        .unwrap_or(current.frequency);
    if freq != "once" && freq != "monthly" {
        return Err(FeesError::InvalidFrequency(freq));
    }

    let effective_from = input.effective_from.unwrap_or(current.effective_from);
    let effective_to = input.effective_to.or(current.effective_to);

    if let Some(ref to) = effective_to {
        if to.as_str() < effective_from.as_str() {
            return Err(FeesError::InvalidDateRange {
                from: effective_from,
                to: to.clone(),
            });
        }
    }

    // Check overlap with other plans
    check_plan_overlap(
        conn,
        Some(id),
        current.program_id.as_deref(),
        current.batch_id.as_deref(),
        &current.fee_head_id,
        &effective_from,
        effective_to.as_deref(),
    )?;

    let now = now_utc_iso();
    conn.execute(
        "UPDATE fee_plans SET amount_poisha = ?1, frequency = ?2, effective_from = ?3, effective_to = ?4, updated_at = ?5
         WHERE id = ?6",
        params![amount, freq, effective_from, effective_to, now, id],
    )?;

    get_fee_plan(conn, id)
}

pub fn delete_fee_plan(conn: &Connection, id: &str) -> Result<(), FeesError> {
    let _ = get_fee_plan(conn, id)?;
    conn.execute("DELETE FROM fee_plans WHERE id = ?1", params![id])?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Concessions Repository
// -----------------------------------------------------------------------------

pub fn list_concessions(
    conn: &Connection,
    student_id: Option<&str>,
    batch_id: Option<&str>,
) -> Result<Vec<ConcessionItem>, FeesError> {
    let mut query = String::from(
        "SELECT c.id, c.student_id, s.name, s.student_code,
                c.batch_id, b.name, c.fee_head_id, fh.name,
                c.type, c.value, c.from_period, c.to_period,
                c.reason, c.approved_by, c.created_at, c.updated_at
         FROM concessions c
         JOIN students s ON c.student_id = s.id
         LEFT JOIN batches b ON c.batch_id = b.id
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(sid) = student_id {
        query.push_str("AND c.student_id = ? ");
        params_vec.push(Box::new(sid.to_string()));
    }
    if let Some(bid) = batch_id {
        query.push_str("AND (c.batch_id = ? OR c.batch_id IS NULL) ");
        params_vec.push(Box::new(bid.to_string()));
    }

    query.push_str("ORDER BY c.created_at DESC");

    let mut stmt = conn.prepare(&query)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok(ConcessionItem {
            id: row.get(0)?,
            student_id: row.get(1)?,
            student_name: row.get(2)?,
            student_code: row.get(3)?,
            batch_id: row.get(4)?,
            batch_name: row.get(5)?,
            fee_head_id: row.get(6)?,
            fee_head_name: row.get(7)?,
            r#type: row.get(8)?,
            value: row.get(9)?,
            from_period: row.get(10)?,
            to_period: row.get(11)?,
            reason: row.get(12)?,
            approved_by: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn get_concession(conn: &Connection, id: &str) -> Result<ConcessionItem, FeesError> {
    conn.query_row(
        "SELECT c.id, c.student_id, s.name, s.student_code,
                c.batch_id, b.name, c.fee_head_id, fh.name,
                c.type, c.value, c.from_period, c.to_period,
                c.reason, c.approved_by, c.created_at, c.updated_at
         FROM concessions c
         JOIN students s ON c.student_id = s.id
         LEFT JOIN batches b ON c.batch_id = b.id
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE c.id = ?1",
        params![id],
        |row| {
            Ok(ConcessionItem {
                id: row.get(0)?,
                student_id: row.get(1)?,
                student_name: row.get(2)?,
                student_code: row.get(3)?,
                batch_id: row.get(4)?,
                batch_name: row.get(5)?,
                fee_head_id: row.get(6)?,
                fee_head_name: row.get(7)?,
                r#type: row.get(8)?,
                value: row.get(9)?,
                from_period: row.get(10)?,
                to_period: row.get(11)?,
                reason: row.get(12)?,
                approved_by: row.get(13)?,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        },
    )
    .optional()?
    .ok_to_result(FeesError::ConcessionNotFound(id.to_string()))
}

pub fn create_concession(
    conn: &Connection,
    input: CreateConcessionInput,
    approved_by: Option<&str>,
) -> Result<ConcessionItem, FeesError> {
    // Validate student exists
    let student_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM students WHERE id = ?1",
        params![input.student_id],
        |r| r.get(0),
    )?;
    if student_exists == 0 {
        return Err(FeesError::StudentNotFound(input.student_id));
    }

    // Validate fee head exists
    let _ = get_fee_head(conn, &input.fee_head_id)?;

    // Validate batch if provided
    if let Some(ref bid) = input.batch_id {
        let batch_exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM batches WHERE id = ?1",
            params![bid],
            |r| r.get(0),
        )?;
        if batch_exists == 0 {
            return Err(FeesError::BatchNotFound(bid.clone()));
        }
    }

    let c_type = input.r#type.to_lowercase();
    if c_type != "percent" && c_type != "fixed" {
        return Err(FeesError::InvalidConcessionType(input.r#type));
    }

    if c_type == "percent" && (input.value <= 0 || input.value > 10000) {
        return Err(FeesError::InvalidValue(input.value));
    }
    if c_type == "fixed" && input.value <= 0 {
        return Err(FeesError::InvalidValue(input.value));
    }

    if let (Some(ref from), Some(ref to)) = (&input.from_period, &input.to_period) {
        validate_period(from)?;
        validate_period(to)?;
        if to < from {
            return Err(FeesError::InvalidDateRange {
                from: from.clone(),
                to: to.clone(),
            });
        }
    } else {
        if let Some(ref from) = input.from_period {
            validate_period(from)?;
        }
        if let Some(ref to) = input.to_period {
            validate_period(to)?;
        }
    }

    if input.reason.trim().is_empty() {
        return Err(FeesError::InvalidValue(0));
    }

    // Unique check on (student_id, fee_head_id, COALESCE(batch_id, 'ALL'))
    let existing_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM concessions
         WHERE student_id = ?1 AND fee_head_id = ?2 AND COALESCE(batch_id, 'ALL') = COALESCE(?3, 'ALL')",
        params![input.student_id, input.fee_head_id, input.batch_id],
        |r| r.get(0),
    )?;

    if existing_count > 0 {
        return Err(FeesError::ConcessionConflict(
            "An active concession already exists for this student, fee head, and batch".to_string(),
        ));
    }

    let id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO concessions (id, student_id, batch_id, fee_head_id, type, value, from_period, to_period, reason, approved_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            id,
            input.student_id,
            input.batch_id,
            input.fee_head_id,
            c_type,
            input.value,
            input.from_period,
            input.to_period,
            input.reason.trim(),
            approved_by,
            now,
            now
        ],
    )?;

    get_concession(conn, &id)
}

pub fn delete_concession(conn: &Connection, id: &str) -> Result<(), FeesError> {
    let _ = get_concession(conn, id)?;
    conn.execute("DELETE FROM concessions WHERE id = ?1", params![id])?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Concession Evaluation Hierarchy (AC-3)
// -----------------------------------------------------------------------------

/// Resolves the effective concession for a student, fee head, and batch in a given period.
/// Hierarchy: Batch-specific concession overrides student-wide concession on the same fee head.
pub fn resolve_concession(
    conn: &Connection,
    student_id: &str,
    batch_id: &str,
    fee_head_id: &str,
    period: &str,
) -> Result<Option<(String, i64)>, FeesError> {
    // 1. Try batch-specific concession
    let batch_concession: Option<(String, i64)> = conn
        .query_row(
            "SELECT type, value FROM concessions
             WHERE student_id = ?1 AND batch_id = ?2 AND fee_head_id = ?3
               AND (from_period IS NULL OR from_period <= ?4)
               AND (to_period IS NULL OR to_period >= ?4)
             LIMIT 1",
            params![student_id, batch_id, fee_head_id, period],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if batch_concession.is_some() {
        return Ok(batch_concession);
    }

    // 2. Try student-wide concession
    let student_concession: Option<(String, i64)> = conn
        .query_row(
            "SELECT type, value FROM concessions
             WHERE student_id = ?1 AND batch_id IS NULL AND fee_head_id = ?2
               AND (from_period IS NULL OR from_period <= ?3)
               AND (to_period IS NULL OR to_period >= ?3)
             LIMIT 1",
            params![student_id, fee_head_id, period],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    Ok(student_concession)
}

// -----------------------------------------------------------------------------
// Chunked Monthly Charge Generator (AC-4, AC-5, AC-6)
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct EnrollmentForBilling {
    enrollment_id: String,
    student_id: String,
    batch_id: String,
    program_id: String,
}

pub fn generate_monthly_charges(
    conn: &Connection,
    period: &str,
    batch_id: Option<&str>,
) -> Result<GenerationResult, FeesError> {
    validate_period(period)?;

    let period_start = format!("{}-01", period);
    // Period end approximate last day
    let period_end = format!("{}-31", period);
    let due_date = format!("{}-10", period);

    // If batch_id provided, verify batch exists
    if let Some(bid) = batch_id {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM batches WHERE id = ?1",
            params![bid],
            |r| r.get(0),
        )?;
        if count == 0 {
            return Err(FeesError::BatchNotFound(bid.to_string()));
        }
    }

    // 1. Fetch eligible enrollments
    // AC-5: Mid month enrollments generate a full month tuition charge for their initial billing period.
    // If enrollment start_date <= period_end AND (end_date IS NULL OR end_date >= period_start)
    let mut enrollments_query = String::from(
        "SELECT e.id, e.student_id, e.batch_id, b.program_id, e.start_date, e.end_date
         FROM enrollments e
         JOIN batches b ON e.batch_id = b.id
         JOIN students s ON e.student_id = s.id
         WHERE e.status = 'active'
           AND s.status = 'active'
           AND e.start_date <= ?1
           AND (e.end_date IS NULL OR e.end_date >= ?2) ",
    );

    let mut q_params: Vec<Box<dyn rusqlite::ToSql>> =
        vec![Box::new(period_end.clone()), Box::new(period_start.clone())];

    if let Some(bid) = batch_id {
        enrollments_query.push_str("AND e.batch_id = ?3 ");
        q_params.push(Box::new(bid.to_string()));
    }

    enrollments_query.push_str("ORDER BY e.student_id ASC");

    let mut stmt = conn.prepare(&enrollments_query)?;
    let p_refs: Vec<&dyn rusqlite::ToSql> = q_params.iter().map(|b| b.as_ref()).collect();

    let rows = stmt.query_map(&p_refs[..], |row| {
        Ok(EnrollmentForBilling {
            enrollment_id: row.get(0)?,
            student_id: row.get(1)?,
            batch_id: row.get(2)?,
            program_id: row.get(3)?,
        })
    })?;

    let mut eligible = Vec::new();
    for r in rows {
        eligible.push(r?);
    }
    drop(stmt);

    let mut total_generated = 0;
    let mut total_skipped = 0;
    let mut total_gross = 0;
    let mut total_concession = 0;
    let mut total_net = 0;
    let mut total_allocated_credit = 0;
    let total_students = eligible.len();

    // 2. Process in chunks of 100 students (AC-4)
    let chunk_size = 100;
    for chunk in eligible.chunks(chunk_size) {
        conn.execute("SAVEPOINT chunk_gen", [])?;

        let mut affected_students = Vec::new();

        let chunk_res: Result<(), FeesError> = (|| {
            for enr in chunk {
                // Find active monthly fee plans for this batch or program
                // Check batch plan first
                let mut plans: Vec<(String, i64)> = {
                    let mut p_stmt = conn.prepare(
                        "SELECT fee_head_id, amount_poisha FROM fee_plans
                         WHERE batch_id = ?1 AND frequency = 'monthly'
                           AND effective_from <= ?2
                           AND (effective_to IS NULL OR effective_to >= ?3)",
                    )?;
                    let rows = p_stmt
                        .query_map(params![enr.batch_id, period_end, period_start], |r| {
                            Ok((r.get(0)?, r.get(1)?))
                        })?;
                    let mut v = Vec::new();
                    for r in rows {
                        v.push(r?);
                    }
                    v
                };

                // If no batch plan, fall back to program plan
                if plans.is_empty() {
                    let mut p_stmt = conn.prepare(
                        "SELECT fee_head_id, amount_poisha FROM fee_plans
                         WHERE program_id = ?1 AND frequency = 'monthly'
                           AND effective_from <= ?2
                           AND (effective_to IS NULL OR effective_to >= ?3)",
                    )?;
                    let rows = p_stmt
                        .query_map(params![enr.program_id, period_end, period_start], |r| {
                            Ok((r.get(0)?, r.get(1)?))
                        })?;
                    for r in rows {
                        plans.push(r?);
                    }
                }

                if plans.is_empty() {
                    continue;
                }

                for (fee_head_id, gross_amount) in plans {
                    // Check if generated charge already exists (AC-4 idempotency & void safety)
                    let exists: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM charges
                         WHERE enrollment_id = ?1 AND fee_head_id = ?2 AND period = ?3 AND source = 'generated'",
                        params![enr.enrollment_id, fee_head_id, period],
                        |r| r.get(0),
                    )?;

                    if exists > 0 {
                        total_skipped += 1;
                        continue;
                    }

                    // Evaluate concession
                    let concession_opt = resolve_concession(
                        conn,
                        &enr.student_id,
                        &enr.batch_id,
                        &fee_head_id,
                        period,
                    )?;

                    let concession_amount = match concession_opt {
                        Some((c_type, val)) => calculate_concession(gross_amount, &c_type, val),
                        None => 0,
                    };

                    let net_amount = (gross_amount - concession_amount).max(0);
                    let charge_id = Uuid::now_v7().to_string();
                    let now = now_utc_iso();

                    conn.execute(
                        "INSERT INTO charges (id, student_id, enrollment_id, fee_head_id, period, gross_amount_poisha, concession_amount_poisha, net_amount_poisha, due_date, source, status, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'generated', 'open', ?10, ?10)",
                        params![
                            charge_id,
                            enr.student_id,
                            enr.enrollment_id,
                            fee_head_id,
                            period,
                            gross_amount,
                            concession_amount,
                            net_amount,
                            due_date,
                            now
                        ],
                    )?;

                    total_generated += 1;
                    total_gross += gross_amount;
                    total_concession += concession_amount;
                    total_net += net_amount;

                    if !affected_students.contains(&enr.student_id) {
                        affected_students.push(enr.student_id.clone());
                    }
                }
            }

            // AC-6: Automatic Advance Credit Allocation
            for student_id in &affected_students {
                let allocated = auto_allocate_advance_credit(conn, student_id)?;
                total_allocated_credit += allocated;

                // AC-9: Update synchronous balance cache
                recalculate_student_balance(conn, student_id)?;
            }

            Ok(())
        })();

        match chunk_res {
            Ok(()) => {
                conn.execute("RELEASE SAVEPOINT chunk_gen", [])?;
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK TO SAVEPOINT chunk_gen", []);
                return Err(e);
            }
        }
    }

    Ok(GenerationResult {
        period: period.to_string(),
        total_students_processed: total_students,
        generated_count: total_generated,
        skipped_count: total_skipped,
        total_gross_poisha: total_gross,
        total_concession_poisha: total_concession,
        total_net_poisha: total_net,
        allocated_credit_poisha: total_allocated_credit,
    })
}

// -----------------------------------------------------------------------------
// Auto-Credit Allocation Engine (AC-6)
// -----------------------------------------------------------------------------

#[derive(Debug)]
struct UnallocatedPayment {
    payment_id: String,
    available_poisha: i64,
}

#[derive(Debug)]
struct OpenCharge {
    charge_id: String,
    fee_head_kind: String,
    due_date: String,
    unallocated_poisha: i64,
}

pub fn auto_allocate_advance_credit(conn: &Connection, student_id: &str) -> Result<i64, FeesError> {
    // 1. Find unallocated posted payments for student, ordered oldest first
    let mut payments_stmt = conn.prepare(
        "SELECT p.id,
                p.amount_poisha
                - COALESCE((SELECT SUM(pa.amount_poisha) FROM payment_allocations pa WHERE pa.payment_id = p.id), 0)
                - COALESCE((SELECT SUM(r.amount_poisha) FROM refunds r WHERE r.payment_id = p.id), 0) as available
         FROM payments p
         WHERE p.student_id = ?1 AND p.status = 'posted'
         ORDER BY p.business_date ASC, p.created_at ASC",
    )?;

    let p_rows = payments_stmt.query_map(params![student_id], |r| {
        Ok(UnallocatedPayment {
            payment_id: r.get(0)?,
            available_poisha: r.get(1)?,
        })
    })?;

    let mut payments = Vec::new();
    for r in p_rows {
        let p = r?;
        if p.available_poisha > 0 {
            payments.push(p);
        }
    }
    drop(payments_stmt);

    if payments.is_empty() {
        return Ok(0);
    }

    // 2. Find open charges with unpaid balance
    // AC-6: Prioritize oldest open charges first by due date, breaking ties by fee head hierarchy
    let mut charges_stmt = conn.prepare(
        "SELECT c.id, fh.kind, c.due_date,
                c.net_amount_poisha
                + COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'fine'), 0)
                - COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'waiver'), 0)
                - COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = c.id AND p.status = 'posted'
                ), 0) as remaining_due
         FROM charges c
         JOIN fee_heads fh ON c.fee_head_id = fh.id
         WHERE c.student_id = ?1 AND c.status = 'open'
         ORDER BY c.due_date ASC",
    )?;

    let c_rows = charges_stmt.query_map(params![student_id], |r| {
        Ok(OpenCharge {
            charge_id: r.get(0)?,
            fee_head_kind: r.get(1)?,
            due_date: r.get(2)?,
            unallocated_poisha: r.get(3)?,
        })
    })?;

    let mut charges = Vec::new();
    for r in c_rows {
        let c = r?;
        if c.unallocated_poisha > 0 {
            charges.push(c);
        }
    }
    drop(charges_stmt);

    if charges.is_empty() {
        return Ok(0);
    }

    // Stable sort by (due_date ASC, fee_head_priority ASC)
    charges.sort_by(|a, b| {
        a.due_date.cmp(&b.due_date).then_with(|| {
            fee_head_priority(&a.fee_head_kind).cmp(&fee_head_priority(&b.fee_head_kind))
        })
    });

    let mut total_allocated = 0;
    let now = now_utc_iso();

    let mut p_idx = 0;
    for charge in &mut charges {
        while charge.unallocated_poisha > 0 && p_idx < payments.len() {
            let p = &mut payments[p_idx];
            if p.available_poisha <= 0 {
                p_idx += 1;
                continue;
            }

            let alloc_amount = charge.unallocated_poisha.min(p.available_poisha);
            let alloc_id = Uuid::now_v7().to_string();

            conn.execute(
                "INSERT INTO payment_allocations (id, payment_id, charge_id, amount_poisha, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![alloc_id, p.payment_id, charge.charge_id, alloc_amount, now],
            )?;

            charge.unallocated_poisha -= alloc_amount;
            p.available_poisha -= alloc_amount;
            total_allocated += alloc_amount;

            if p.available_poisha == 0 {
                p_idx += 1;
            }
        }

        if p_idx >= payments.len() {
            break;
        }
    }

    Ok(total_allocated)
}

// -----------------------------------------------------------------------------
// Synchronous Balance Cache Updater (AC-9)
// -----------------------------------------------------------------------------

pub fn recalculate_student_balance(conn: &Connection, student_id: &str) -> Result<(), FeesError> {
    // 1. Total net charged (from all charges with status != 'voided')
    let total_charged: i64 = conn.query_row(
        "SELECT COALESCE(SUM(net_amount_poisha), 0) FROM charges WHERE student_id = ?1 AND status != 'voided'",
        params![student_id],
        |r| r.get(0),
    )?;

    // 2. Total posted payments
    let total_paid: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM payments WHERE student_id = ?1 AND status = 'posted'",
        params![student_id],
        |r| r.get(0),
    )?;

    // 3. Total adjustments: fines increase balance (+), waivers decrease balance (-)
    let total_fines: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM adjustments WHERE student_id = ?1 AND type = 'fine'",
        params![student_id],
        |r| r.get(0),
    )?;

    let total_waivers: i64 = conn.query_row(
        "SELECT COALESCE(SUM(amount_poisha), 0) FROM adjustments WHERE student_id = ?1 AND type = 'waiver'",
        params![student_id],
        |r| r.get(0),
    )?;

    // 4. Total refunds
    let total_refunded: i64 = conn.query_row(
        "SELECT COALESCE(SUM(r.amount_poisha), 0)
         FROM refunds r
         JOIN payments p ON r.payment_id = p.id
         WHERE p.student_id = ?1 AND p.status = 'posted'",
        params![student_id],
        |r| r.get(0),
    )?;

    // Invariant 4: Balance = Total open charges + Fines - Waivers - Posted payments + Refunds
    let total_adjusted = total_fines - total_waivers;
    let balance = total_charged + total_fines - total_waivers - total_paid + total_refunded;
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO student_balances (student_id, total_charged_poisha, total_paid_poisha, total_adjusted_poisha, total_refunded_poisha, balance_poisha, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(student_id) DO UPDATE SET
            total_charged_poisha = excluded.total_charged_poisha,
            total_paid_poisha = excluded.total_paid_poisha,
            total_adjusted_poisha = excluded.total_adjusted_poisha,
            total_refunded_poisha = excluded.total_refunded_poisha,
            balance_poisha = excluded.balance_poisha,
            updated_at = excluded.updated_at",
        params![student_id, total_charged, total_paid, total_adjusted, total_refunded, balance, now],
    )?;

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
