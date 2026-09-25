use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ReportsError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Sql(String),
}

impl From<rusqlite::Error> for ReportsError {
    fn from(err: rusqlite::Error) -> Self {
        ReportsError::Sql(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// 1. Collection Summary DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectionSummaryPayload {
    pub start_date: String,
    pub end_date: String,
    pub collector_id: Option<String>,
    pub payment_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MethodDistributionItem {
    pub method: String,
    pub amount_poisha: i64,
    pub amount_formatted: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DailyCollectionPoint {
    pub date: String,
    pub amount_poisha: i64,
    pub amount_formatted: String,
    pub payment_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectionSummaryResponse {
    pub total_collected_poisha: i64,
    pub total_collected_formatted: String,
    pub net_collected_poisha: i64,
    pub net_collected_formatted: String,
    pub payment_count: u32,
    pub void_count: u32,
    pub method_distribution: Vec<MethodDistributionItem>,
    pub daily_breakdown: Vec<DailyCollectionPoint>,
}

// -----------------------------------------------------------------------------
// 2. Collector Performance DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectorSummaryPayload {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectorPerformanceItem {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub total_payments: u32,
    pub total_collected_poisha: i64,
    pub total_collected_formatted: String,
    pub cash_collected_poisha: i64,
    pub cash_collected_formatted: String,
    pub digital_collected_poisha: i64,
    pub digital_collected_formatted: String,
    pub void_count: u32,
    pub void_amount_poisha: i64,
    pub void_amount_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CollectorSummaryResponse {
    pub collectors: Vec<CollectorPerformanceItem>,
}

// -----------------------------------------------------------------------------
// 3. Dues Aging DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DuesAgingPayload {
    pub batch_id: Option<String>,
    pub as_of_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DuesAgingItem {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub batch_name: String,
    pub primary_guardian_name: Option<String>,
    pub primary_guardian_phone: Option<String>,
    pub current_bucket_poisha: i64,
    pub current_bucket_formatted: String,
    pub days_31_60_poisha: i64,
    pub days_31_60_formatted: String,
    pub days_61_90_poisha: i64,
    pub days_61_90_formatted: String,
    pub days_90_plus_poisha: i64,
    pub days_90_plus_formatted: String,
    pub total_due_poisha: i64,
    pub total_due_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DuesAgingBuckets {
    pub total_current_poisha: i64,
    pub total_current_formatted: String,
    pub total_days_31_60_poisha: i64,
    pub total_days_31_60_formatted: String,
    pub total_days_61_90_poisha: i64,
    pub total_days_61_90_formatted: String,
    pub total_days_90_plus_poisha: i64,
    pub total_days_90_plus_formatted: String,
    pub grand_total_due_poisha: i64,
    pub grand_total_due_formatted: String,
    pub total_students_with_dues: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DuesAgingResponse {
    pub items: Vec<DuesAgingItem>,
    pub summary: DuesAgingBuckets,
}

// -----------------------------------------------------------------------------
// 4. Expenses & Cash Drawer DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExpensesSummaryPayload {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExpenseCategoryBreakdown {
    pub category_name: String,
    pub amount_poisha: i64,
    pub amount_formatted: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashDrawerReconciliation {
    pub starting_cash_poisha: i64,
    pub starting_cash_formatted: String,
    pub cash_collections_poisha: i64,
    pub cash_collections_formatted: String,
    pub cash_expenses_poisha: i64,
    pub cash_expenses_formatted: String,
    pub expected_closing_poisha: i64,
    pub expected_closing_formatted: String,
    pub counted_closing_poisha: i64,
    pub counted_closing_formatted: String,
    pub discrepancy_poisha: i64,
    pub discrepancy_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExpensesSummaryResponse {
    pub total_expense_poisha: i64,
    pub total_expense_formatted: String,
    pub expense_count: u32,
    pub categories: Vec<ExpenseCategoryBreakdown>,
    pub drawer_reconciliation: CashDrawerReconciliation,
}

// -----------------------------------------------------------------------------
// 5. Attendance Summary DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AttendanceSummaryPayload {
    pub batch_id: Option<String>,
    pub month: String, // "YYYY-MM"
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentAttendanceSummaryItem {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub batch_name: String,
    pub total_sessions: u32,
    pub present_count: u32,
    pub absent_count: u32,
    pub late_count: u32,
    pub excused_count: u32,
    pub attendance_percentage: f64,
    pub is_chronic_absentee: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AttendanceSummaryResponse {
    pub items: Vec<StudentAttendanceSummaryItem>,
    pub batch_average_percentage: f64,
    pub total_students: u32,
    pub chronic_absentee_count: u32,
}

// -----------------------------------------------------------------------------
// 6. Exam Performance DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamPerformancePayload {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GradeDistributionItem {
    pub grade: String,
    pub count: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamRankerItem {
    pub rank: u32,
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub gpa: f64,
    pub grade: String,
    pub total_marks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamPerformanceResponse {
    pub exam_id: String,
    pub exam_name: String,
    pub total_candidates: u32,
    pub passed_count: u32,
    pub failed_count: u32,
    pub pass_percentage: f64,
    pub average_gpa: f64,
    pub grade_distribution: Vec<GradeDistributionItem>,
    pub top_rankers: Vec<ExamRankerItem>,
}

// -----------------------------------------------------------------------------
// 7. CSV Export DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportCsvPayload {
    pub report_type: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExportCsvResponse {
    pub filename: String,
    pub csv_content: String,
}

// -----------------------------------------------------------------------------
// Formatting Helpers
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct StudentsExportFilterPayload {
    pub status: Option<String>,
    pub batch_id: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct DuesExportFilterPayload {
    pub program_id: Option<String>,
    pub batch_id: Option<String>,
    pub period: Option<String>,
    pub min_due_poisha: Option<i64>,
    pub overdue_only: Option<bool>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamResultsExportFilterPayload {
    pub exam_id: String,
    pub batch_id: Option<String>,
}

pub fn format_taka(poisha: i64) -> String {
    format!("{:.2}", (poisha as f64) / 100.0)
}

fn is_potential_formula_injection(val: &str) -> bool {
    let trimmed = val.trim_start();
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    match first {
        '=' | '@' | '\t' => true,
        '+' | '-' => {
            // Allow plain signed numbers like "-12.50", but prefix '+880...' phone numbers or '+cmd' formulas
            let rest = &trimmed[first.len_utf8()..];
            if first == '+' {
                true
            } else {
                rest.parse::<f64>().is_err()
            }
        }
        _ => false,
    }
}

pub fn escape_csv_cell(val: &str) -> String {
    let safe_val = if is_potential_formula_injection(val) {
        format!("'{}", val)
    } else {
        val.to_string()
    };

    if safe_val.contains(',')
        || safe_val.contains('"')
        || safe_val.contains('\n')
        || safe_val.contains('\r')
        || safe_val.starts_with('\'')
    {
        format!("\"{}\"", safe_val.replace('"', "\"\""))
    } else {
        safe_val
    }
}

// -----------------------------------------------------------------------------
// Implementation Functions
// -----------------------------------------------------------------------------

/// AC-1: Collection summary with payment method distribution and daily timeline.
pub fn get_collection_summary(
    conn: &Connection,
    payload: &CollectionSummaryPayload,
) -> Result<CollectionSummaryResponse, ReportsError> {
    if payload.start_date.trim().is_empty() || payload.end_date.trim().is_empty() {
        return Err(ReportsError::Validation(
            "start_date and end_date cannot be empty".to_string(),
        ));
    }

    // 1. Valid / posted collections query
    let mut where_clauses = vec![
        "p.status = 'posted'".to_string(),
        "substr(p.received_at, 1, 10) >= ?1".to_string(),
        "substr(p.received_at, 1, 10) <= ?2".to_string(),
    ];
    let mut sql_params: Vec<rusqlite::types::Value> = vec![
        rusqlite::types::Value::Text(payload.start_date.clone()),
        rusqlite::types::Value::Text(payload.end_date.clone()),
    ];

    if let Some(ref coll) = payload.collector_id {
        if !coll.trim().is_empty() {
            where_clauses.push(format!("p.collector_user_id = ?{}", sql_params.len() + 1));
            sql_params.push(rusqlite::types::Value::Text(coll.clone()));
        }
    }

    if let Some(ref method) = payload.payment_method {
        if !method.trim().is_empty() {
            where_clauses.push(format!("p.method = ?{}", sql_params.len() + 1));
            sql_params.push(rusqlite::types::Value::Text(method.clone()));
        }
    }

    let where_str = where_clauses.join(" AND ");

    // Query aggregates
    let query_summary = format!(
        "SELECT COUNT(*), COALESCE(SUM(p.amount_poisha), 0)
         FROM payments p
         WHERE {}",
        where_str
    );

    let (payment_count, total_collected_poisha): (u32, i64) = conn.query_row(
        &query_summary,
        rusqlite::params_from_iter(sql_params.iter()),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // Void and refund count
    let (void_count, void_amount_poisha): (u32, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(amount_poisha), 0)
         FROM payments
         WHERE status IN ('voided', 'refunded')
           AND substr(received_at, 1, 10) >= ?1
           AND substr(received_at, 1, 10) <= ?2",
        params![payload.start_date, payload.end_date],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let net_collected_poisha = total_collected_poisha.saturating_sub(void_amount_poisha);

    // Method distribution
    let query_methods = format!(
        "SELECT p.method, COUNT(*), COALESCE(SUM(p.amount_poisha), 0)
         FROM payments p
         WHERE {}
         GROUP BY p.method
         ORDER BY SUM(p.amount_poisha) DESC",
        where_str
    );

    let mut stmt_methods = conn.prepare(&query_methods)?;
    let method_rows =
        stmt_methods.query_map(rusqlite::params_from_iter(sql_params.iter()), |r| {
            let method: String = r.get(0)?;
            let count: u32 = r.get(1)?;
            let amt: i64 = r.get(2)?;
            Ok(MethodDistributionItem {
                method,
                amount_poisha: amt,
                amount_formatted: format_taka(amt),
                count,
            })
        })?;

    let mut method_distribution = Vec::new();
    for m in method_rows {
        method_distribution.push(m?);
    }

    // Daily breakdown
    let query_daily = format!(
        "SELECT substr(p.received_at, 1, 10) as dt, COUNT(*), COALESCE(SUM(p.amount_poisha), 0)
         FROM payments p
         WHERE {}
         GROUP BY dt
         ORDER BY dt ASC",
        where_str
    );

    let mut stmt_daily = conn.prepare(&query_daily)?;
    let daily_rows = stmt_daily.query_map(rusqlite::params_from_iter(sql_params.iter()), |r| {
        let date: String = r.get(0)?;
        let p_count: u32 = r.get(1)?;
        let amt: i64 = r.get(2)?;
        Ok(DailyCollectionPoint {
            date,
            amount_poisha: amt,
            amount_formatted: format_taka(amt),
            payment_count: p_count,
        })
    })?;

    let mut daily_breakdown = Vec::new();
    for d in daily_rows {
        daily_breakdown.push(d?);
    }

    Ok(CollectionSummaryResponse {
        total_collected_poisha,
        total_collected_formatted: format_taka(total_collected_poisha),
        net_collected_poisha,
        net_collected_formatted: format_taka(net_collected_poisha),
        payment_count,
        void_count,
        method_distribution,
        daily_breakdown,
    })
}

/// AC-2: Collector performance summary grouping by cashier/user.
pub fn get_collector_summary(
    conn: &Connection,
    payload: &CollectorSummaryPayload,
) -> Result<CollectorSummaryResponse, ReportsError> {
    let start_date = payload.start_date.as_deref().unwrap_or("2000-01-01");
    let end_date = payload.end_date.as_deref().unwrap_or("2099-12-31");

    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.display_name,
                COUNT(CASE WHEN p.status = 'posted' THEN 1 END) as total_payments,
                COALESCE(SUM(CASE WHEN p.status = 'posted' THEN p.amount_poisha ELSE 0 END), 0) as total_collected,
                COALESCE(SUM(CASE WHEN p.status = 'posted' AND p.method = 'cash' THEN p.amount_poisha ELSE 0 END), 0) as cash_collected,
                COALESCE(SUM(CASE WHEN p.status = 'posted' AND p.method != 'cash' THEN p.amount_poisha ELSE 0 END), 0) as digital_collected,
                COUNT(CASE WHEN p.status IN ('voided', 'refunded') THEN 1 END) as void_count,
                COALESCE(SUM(CASE WHEN p.status IN ('voided', 'refunded') THEN p.amount_poisha ELSE 0 END), 0) as void_amount
         FROM users u
         LEFT JOIN payments p ON p.collector_user_id = u.id
             AND substr(p.received_at, 1, 10) >= ?1
             AND substr(p.received_at, 1, 10) <= ?2
         GROUP BY u.id
         HAVING total_payments > 0 OR void_count > 0
         ORDER BY total_collected DESC, u.username ASC"
    )?;

    let rows = stmt.query_map(params![start_date, end_date], |r| {
        let user_id: String = r.get(0)?;
        let username: String = r.get(1)?;
        let display_name: String = r.get(2)?;
        let total_payments: u32 = r.get(3)?;
        let total_collected: i64 = r.get(4)?;
        let cash_collected: i64 = r.get(5)?;
        let digital_collected: i64 = r.get(6)?;
        let void_count: u32 = r.get(7)?;
        let void_amount: i64 = r.get(8)?;

        Ok(CollectorPerformanceItem {
            user_id,
            username,
            display_name,
            total_payments,
            total_collected_poisha: total_collected,
            total_collected_formatted: format_taka(total_collected),
            cash_collected_poisha: cash_collected,
            cash_collected_formatted: format_taka(cash_collected),
            digital_collected_poisha: digital_collected,
            digital_collected_formatted: format_taka(digital_collected),
            void_count,
            void_amount_poisha: void_amount,
            void_amount_formatted: format_taka(void_amount),
        })
    })?;

    let mut collectors = Vec::new();
    for c in rows {
        collectors.push(c?);
    }

    Ok(CollectorSummaryResponse { collectors })
}

/// AC-3: Dues aging report segmented into 0–30, 31–60, 61–90, 90+ days overdue buckets.
pub fn get_dues_aging_report(
    conn: &Connection,
    payload: &DuesAgingPayload,
) -> Result<DuesAgingResponse, ReportsError> {
    let as_of = payload
        .as_of_date
        .clone()
        .unwrap_or_else(|| crate::db::now_utc_iso()[..10].to_string());

    // Fetch student active charges with remaining balance
    let mut sql = String::from(
        "SELECT s.id, s.student_code, s.name, s.name_bn,
                COALESCE((SELECT b.name FROM enrollments e JOIN batches b ON b.id = e.batch_id WHERE e.student_id = s.id AND e.status = 'active' LIMIT 1), 'No Active Batch') as batch_name,
                (SELECT g.name FROM student_guardians sg JOIN guardians g ON g.id = sg.guardian_id WHERE sg.student_id = s.id AND sg.is_primary = 1 LIMIT 1) as guardian_name,
                (SELECT g.phone_norm FROM student_guardians sg JOIN guardians g ON g.id = sg.guardian_id WHERE sg.student_id = s.id AND sg.is_primary = 1 LIMIT 1) as guardian_phone,
                c.due_date,
                (
                    c.net_amount_poisha
                    + COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'fine'), 0)
                    - COALESCE((SELECT SUM(adj.amount_poisha) FROM adjustments adj WHERE adj.charge_id = c.id AND adj.type = 'waiver'), 0)
                    - COALESCE((
                        SELECT SUM(pa.amount_poisha)
                        FROM payment_allocations pa
                        JOIN payments p ON pa.payment_id = p.id
                        WHERE pa.charge_id = c.id AND p.status IN ('posted', 'completed')
                    ), 0)
                ) as remaining_due
         FROM charges c
         JOIN students s ON s.id = c.student_id
         WHERE c.status = 'open'
           AND s.status = 'active'"
    );

    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(ref b_id) = payload.batch_id {
        if !b_id.trim().is_empty() {
            sql.push_str(" AND EXISTS (SELECT 1 FROM enrollments en WHERE en.student_id = s.id AND en.batch_id = ?1 AND en.status = 'active')");
            params_vec.push(rusqlite::types::Value::Text(b_id.clone()));
        }
    }

    sql.push_str(" ORDER BY s.name ASC, c.due_date ASC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,         // s.id
            r.get::<_, String>(1)?,         // student_code
            r.get::<_, String>(2)?,         // name
            r.get::<_, Option<String>>(3)?, // name_bn
            r.get::<_, String>(4)?,         // batch_name
            r.get::<_, Option<String>>(5)?, // guardian_name
            r.get::<_, Option<String>>(6)?, // guardian_phone
            r.get::<_, String>(7)?,         // due_date
            r.get::<_, i64>(8)?,            // remaining_due
        ))
    })?;

    // Group charges by student and bucket them by age
    let mut student_map: std::collections::HashMap<String, DuesAgingItem> =
        std::collections::HashMap::new();

    // Parse as_of date as days since unix epoch (approximate via days)
    for row in rows {
        let (s_id, s_code, s_name, s_name_bn, b_name, g_name, g_phone, due_date, remaining_due) =
            row?;

        if remaining_due <= 0 {
            continue;
        }

        // Calculate age in days: as_of - due_date
        let days_overdue = calculate_days_difference(&due_date, &as_of);

        let entry = student_map
            .entry(s_id.clone())
            .or_insert_with(|| DuesAgingItem {
                student_id: s_id,
                student_code: s_code,
                name: s_name,
                name_bn: s_name_bn,
                batch_name: b_name,
                primary_guardian_name: g_name,
                primary_guardian_phone: g_phone,
                current_bucket_poisha: 0,
                current_bucket_formatted: String::new(),
                days_31_60_poisha: 0,
                days_31_60_formatted: String::new(),
                days_61_90_poisha: 0,
                days_61_90_formatted: String::new(),
                days_90_plus_poisha: 0,
                days_90_plus_formatted: String::new(),
                total_due_poisha: 0,
                total_due_formatted: String::new(),
            });

        entry.total_due_poisha += remaining_due;

        if days_overdue <= 30 {
            entry.current_bucket_poisha += remaining_due;
        } else if days_overdue <= 60 {
            entry.days_31_60_poisha += remaining_due;
        } else if days_overdue <= 90 {
            entry.days_61_90_poisha += remaining_due;
        } else {
            entry.days_90_plus_poisha += remaining_due;
        }
    }

    let mut items: Vec<DuesAgingItem> = student_map.into_values().collect();
    items.sort_by_key(|a| std::cmp::Reverse(a.total_due_poisha));

    let mut total_curr = 0i64;
    let mut total_31_60 = 0i64;
    let mut total_61_90 = 0i64;
    let mut total_90_plus = 0i64;
    let mut grand_total = 0i64;

    for it in &mut items {
        it.current_bucket_formatted = format_taka(it.current_bucket_poisha);
        it.days_31_60_formatted = format_taka(it.days_31_60_poisha);
        it.days_61_90_formatted = format_taka(it.days_61_90_poisha);
        it.days_90_plus_formatted = format_taka(it.days_90_plus_poisha);
        it.total_due_formatted = format_taka(it.total_due_poisha);

        total_curr += it.current_bucket_poisha;
        total_31_60 += it.days_31_60_poisha;
        total_61_90 += it.days_61_90_poisha;
        total_90_plus += it.days_90_plus_poisha;
        grand_total += it.total_due_poisha;
    }

    let summary = DuesAgingBuckets {
        total_current_poisha: total_curr,
        total_current_formatted: format_taka(total_curr),
        total_days_31_60_poisha: total_31_60,
        total_days_31_60_formatted: format_taka(total_31_60),
        total_days_61_90_poisha: total_61_90,
        total_days_61_90_formatted: format_taka(total_61_90),
        total_days_90_plus_poisha: total_90_plus,
        total_days_90_plus_formatted: format_taka(total_90_plus),
        grand_total_due_poisha: grand_total,
        grand_total_due_formatted: format_taka(grand_total),
        total_students_with_dues: items.len() as u32,
    };

    Ok(DuesAgingResponse { items, summary })
}

/// Helper calculating days between YYYY-MM-DD strings.
fn calculate_days_difference(due_date: &str, as_of: &str) -> i64 {
    // Parse "YYYY-MM-DD"
    let d1 = parse_date_to_days(due_date);
    let d2 = parse_date_to_days(as_of);
    match (d1, d2) {
        (Some(due_days), Some(as_of_days)) if as_of_days >= due_days => as_of_days - due_days,
        _ => 0,
    }
}

fn parse_date_to_days(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
    Some(y * 365 + m * 30 + d)
}

/// AC-4: Expenses breakdown and daily cash drawer reconciliation.
pub fn get_expenses_summary(
    conn: &Connection,
    payload: &ExpensesSummaryPayload,
) -> Result<ExpensesSummaryResponse, ReportsError> {
    // 1. Center expenses by category
    let mut stmt = conn.prepare(
        "SELECT ec.name, COALESCE(SUM(e.amount_poisha), 0), COUNT(e.id)
         FROM expense_categories ec
         LEFT JOIN expenses e ON e.category_id = ec.id
             AND e.status = 'posted'
             AND e.business_date >= ?1
             AND e.business_date <= ?2
         GROUP BY ec.id, ec.name
         HAVING COUNT(e.id) > 0
         ORDER BY SUM(e.amount_poisha) DESC",
    )?;

    let rows = stmt.query_map(params![payload.start_date, payload.end_date], |r| {
        let cat: String = r.get(0)?;
        let amt: i64 = r.get(1)?;
        let cnt: u32 = r.get(2)?;
        Ok(ExpenseCategoryBreakdown {
            category_name: cat,
            amount_poisha: amt,
            amount_formatted: format_taka(amt),
            count: cnt,
        })
    })?;

    let mut categories = Vec::new();
    let mut total_expense_poisha = 0i64;
    let mut expense_count = 0u32;

    for r in rows {
        let item = r?;
        total_expense_poisha += item.amount_poisha;
        expense_count += item.count;
        categories.push(item);
    }

    // 2. Cash drawer aggregation across cash_days
    let drawer_res: Result<(i64, i64, i64, i64, i64, i64), rusqlite::Error> = conn.query_row(
        "SELECT
            COALESCE(SUM(opening_cash_poisha), 0),
            COALESCE(SUM(cash_in_poisha), 0),
            COALESCE(SUM(cash_out_poisha), 0),
            COALESCE(SUM(expected_closing_poisha), 0),
            COALESCE(SUM(COALESCE(counted_closing_poisha, 0)), 0),
            COALESCE(SUM(COALESCE(difference_poisha, 0)), 0)
         FROM cash_days
         WHERE business_date >= ?1 AND business_date <= ?2",
        params![payload.start_date, payload.end_date],
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
    );

    let (starting_cash, cash_in, cash_out, expected_close, counted_close, discrepancy) =
        drawer_res.unwrap_or((0, 0, 0, 0, 0, 0));

    let drawer_reconciliation = CashDrawerReconciliation {
        starting_cash_poisha: starting_cash,
        starting_cash_formatted: format_taka(starting_cash),
        cash_collections_poisha: cash_in,
        cash_collections_formatted: format_taka(cash_in),
        cash_expenses_poisha: cash_out,
        cash_expenses_formatted: format_taka(cash_out),
        expected_closing_poisha: expected_close,
        expected_closing_formatted: format_taka(expected_close),
        counted_closing_poisha: counted_close,
        counted_closing_formatted: format_taka(counted_close),
        discrepancy_poisha: discrepancy,
        discrepancy_formatted: format_taka(discrepancy),
    };

    Ok(ExpensesSummaryResponse {
        total_expense_poisha,
        total_expense_formatted: format_taka(total_expense_poisha),
        expense_count,
        categories,
        drawer_reconciliation,
    })
}

/// AC-5: Batch attendance summary with chronic absenteeism detection (< 75%).
pub fn get_attendance_summary(
    conn: &Connection,
    payload: &AttendanceSummaryPayload,
) -> Result<AttendanceSummaryResponse, ReportsError> {
    let month_prefix = format!("{}%", payload.month);

    let mut sql = String::from(
        "SELECT s.id, s.student_code, s.name, s.name_bn,
                b.name as batch_name,
                COUNT(*) as total_sessions,
                COUNT(CASE WHEN ar.status = 'present' THEN 1 END) as present_count,
                COUNT(CASE WHEN ar.status = 'absent' THEN 1 END) as absent_count,
                COUNT(CASE WHEN ar.status = 'late' THEN 1 END) as late_count,
                COUNT(CASE WHEN ar.status = 'excused' THEN 1 END) as excused_count
         FROM attendance_marks ar
         JOIN attendance_sessions ses ON ses.id = ar.session_id
         JOIN batches b ON b.id = ses.batch_id
         JOIN students s ON s.id = ar.student_id
         WHERE ses.date LIKE ?1",
    );

    let mut params_vec: Vec<rusqlite::types::Value> =
        vec![rusqlite::types::Value::Text(month_prefix)];

    if let Some(ref b_id) = payload.batch_id {
        if !b_id.trim().is_empty() {
            sql.push_str(" AND ses.batch_id = ?2");
            params_vec.push(rusqlite::types::Value::Text(b_id.clone()));
        }
    }

    sql.push_str(" GROUP BY s.id, s.student_code, s.name, s.name_bn, b.name ORDER BY s.name ASC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
        let total: u32 = r.get(5)?;
        let present: u32 = r.get(6)?;
        let absent: u32 = r.get(7)?;
        let late: u32 = r.get(8)?;
        let excused: u32 = r.get(9)?;

        // Attendance rate: (present + late) / total
        let effective_attended = present + late;
        let percentage = if total > 0 {
            ((effective_attended as f64) / (total as f64)) * 100.0
        } else {
            0.0
        };

        // AC-5: chronic absenteeism threshold is < 75.0%
        let is_chronic_absentee = total > 0 && percentage < 75.0;

        Ok(StudentAttendanceSummaryItem {
            student_id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            batch_name: r.get(4)?,
            total_sessions: total,
            present_count: present,
            absent_count: absent,
            late_count: late,
            excused_count: excused,
            attendance_percentage: (percentage * 100.0).round() / 100.0,
            is_chronic_absentee,
        })
    })?;

    let mut items = Vec::new();
    let mut total_pct_sum = 0.0;
    let mut chronic_count = 0u32;

    for r in rows {
        let it = r?;
        total_pct_sum += it.attendance_percentage;
        if it.is_chronic_absentee {
            chronic_count += 1;
        }
        items.push(it);
    }

    let batch_avg = if !items.is_empty() {
        ((total_pct_sum / (items.len() as f64)) * 100.0).round() / 100.0
    } else {
        0.0
    };

    Ok(AttendanceSummaryResponse {
        total_students: items.len() as u32,
        batch_average_percentage: batch_avg,
        chronic_absentee_count: chronic_count,
        items,
    })
}

/// AC-6: Exam merit performance, GPA distribution, and top 5 rankers.
pub fn get_exam_performance(
    conn: &Connection,
    payload: &ExamPerformancePayload,
) -> Result<ExamPerformanceResponse, ReportsError> {
    let exam_name: String = conn
        .query_row(
            "SELECT name FROM exams WHERE id = ?1",
            params![payload.exam_id],
            |r| r.get(0),
        )
        .map_err(|_| ReportsError::NotFound(format!("Exam '{}' not found", payload.exam_id)))?;

    // Candidates count, pass/fail counts, and GPA sum
    let summary: Result<(u32, u32, u32, i64), rusqlite::Error> = conn.query_row(
        "SELECT
            COUNT(*),
            COUNT(CASE WHEN passed = 1 THEN 1 END),
            COUNT(CASE WHEN passed = 0 THEN 1 END),
            COALESCE(SUM(gpa), 0)
         FROM results
         WHERE exam_id = ?1",
        params![payload.exam_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    );

    let (total_candidates, passed_count, failed_count, gpa_sum_hundredths) =
        summary.unwrap_or((0, 0, 0, 0));

    let pass_percentage = if total_candidates > 0 {
        ((passed_count as f64 / total_candidates as f64) * 10000.0).round() / 100.0
    } else {
        0.0
    };

    let average_gpa = if total_candidates > 0 {
        ((gpa_sum_hundredths as f64 / total_candidates as f64) / 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // Grade distribution
    let mut stmt_grades = conn.prepare(
        "SELECT grade, COUNT(*)
         FROM results
         WHERE exam_id = ?1
         GROUP BY grade
         ORDER BY grade ASC",
    )?;

    let grade_rows = stmt_grades.query_map(params![payload.exam_id], |r| {
        let grade: String = r.get(0)?;
        let count: u32 = r.get(1)?;
        let pct = if total_candidates > 0 {
            ((count as f64 / total_candidates as f64) * 10000.0).round() / 100.0
        } else {
            0.0
        };
        Ok(GradeDistributionItem {
            grade,
            count,
            percentage: pct,
        })
    })?;

    let mut grade_distribution = Vec::new();
    for g in grade_rows {
        grade_distribution.push(g?);
    }

    // Top 5 rankers
    let mut stmt_rankers = conn.prepare(
        "SELECT r.rank, r.student_id, s.student_code, s.name, r.gpa, r.grade, r.total_marks
         FROM results r
         JOIN students s ON s.id = r.student_id
         WHERE r.exam_id = ?1
         ORDER BY r.rank ASC
         LIMIT 5",
    )?;

    let ranker_rows = stmt_rankers.query_map(params![payload.exam_id], |r| {
        let rank: u32 = r.get(0)?;
        let s_id: String = r.get(1)?;
        let s_code: String = r.get(2)?;
        let s_name: String = r.get(3)?;
        let gpa_hundredths: i64 = r.get(4)?;
        let grade: String = r.get(5)?;
        let total_marks: i64 = r.get(6)?;

        Ok(ExamRankerItem {
            rank,
            student_id: s_id,
            student_code: s_code,
            name: s_name,
            gpa: (gpa_hundredths as f64) / 100.0,
            grade,
            total_marks,
        })
    })?;

    let mut top_rankers = Vec::new();
    for ranker in ranker_rows {
        top_rankers.push(ranker?);
    }

    Ok(ExamPerformanceResponse {
        exam_id: payload.exam_id.clone(),
        exam_name,
        total_candidates,
        passed_count,
        failed_count,
        pass_percentage,
        average_gpa,
        grade_distribution,
        top_rankers,
    })
}

/// AC-7: UTF-8 BOM RFC 4180 CSV export serializer.
pub fn export_csv(
    conn: &Connection,
    payload: &ExportCsvPayload,
) -> Result<ExportCsvResponse, ReportsError> {
    // Standard UTF-8 Byte Order Mark \u{FEFF}
    let mut csv = String::from("\u{FEFF}");
    let filename: String;

    match payload.report_type.as_str() {
        "collection_summary" => {
            let p: CollectionSummaryPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_collection_summary(conn, &p)?;
            filename = format!("collections_{}_{}.csv", p.start_date, p.end_date);

            csv.push_str("Metric,Value\r\n");
            csv.push_str(&format!(
                "Total Collected (৳),{}\r\n",
                report.total_collected_formatted
            ));
            csv.push_str(&format!(
                "Net Collected (৳),{}\r\n",
                report.net_collected_formatted
            ));
            csv.push_str(&format!("Total Payments,{}\r\n", report.payment_count));
            csv.push_str(&format!("Total Voids,{}\r\n\r\n", report.void_count));

            csv.push_str("Payment Method,Count,Total Amount (৳)\r\n");
            for m in report.method_distribution {
                csv.push_str(&format!(
                    "{},{},{}\r\n",
                    escape_csv_cell(&m.method),
                    m.count,
                    m.amount_formatted
                ));
            }

            csv.push_str("\r\nDate,Payment Count,Total (৳)\r\n");
            for d in report.daily_breakdown {
                csv.push_str(&format!(
                    "{},{},{}\r\n",
                    d.date, d.payment_count, d.amount_formatted
                ));
            }
        }
        "collector_summary" => {
            let p: CollectorSummaryPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_collector_summary(conn, &p)?;
            filename = "collector_performance.csv".to_string();

            csv.push_str("Username,Staff Name,Receipts Taken,Cash (৳),Digital (৳),Total Collected (৳),Voids,Void Amount (৳)\r\n");
            for c in report.collectors {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\r\n",
                    escape_csv_cell(&c.username),
                    escape_csv_cell(&c.display_name),
                    c.total_payments,
                    c.cash_collected_formatted,
                    c.digital_collected_formatted,
                    c.total_collected_formatted,
                    c.void_count,
                    c.void_amount_formatted
                ));
            }
        }
        "dues_aging" => {
            let p: DuesAgingPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_dues_aging_report(conn, &p)?;
            filename = "dues_aging_report.csv".to_string();

            csv.push_str("Student ID,Student Name,Name (Bengali),Batch,Guardian Name,Guardian Phone,Current (0-30d) ৳,31-60d ৳,61-90d ৳,90+d ৳,Total Due ৳\r\n");
            for s in report.items {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{}\r\n",
                    escape_csv_cell(&s.student_code),
                    escape_csv_cell(&s.name),
                    escape_csv_cell(s.name_bn.as_deref().unwrap_or("")),
                    escape_csv_cell(&s.batch_name),
                    escape_csv_cell(s.primary_guardian_name.as_deref().unwrap_or("")),
                    escape_csv_cell(s.primary_guardian_phone.as_deref().unwrap_or("")),
                    s.current_bucket_formatted,
                    s.days_31_60_formatted,
                    s.days_61_90_formatted,
                    s.days_90_plus_formatted,
                    s.total_due_formatted
                ));
            }
            csv.push_str(&format!(
                "SUMMARY,,,,,Grand Total,{},{},{},{},{}\r\n",
                report.summary.total_current_formatted,
                report.summary.total_days_31_60_formatted,
                report.summary.total_days_61_90_formatted,
                report.summary.total_days_90_plus_formatted,
                report.summary.grand_total_due_formatted
            ));
        }
        "expenses" => {
            let p: ExpensesSummaryPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_expenses_summary(conn, &p)?;
            filename = format!("expenses_{}_{}.csv", p.start_date, p.end_date);

            csv.push_str("Category,Count,Total Amount (৳)\r\n");
            for c in report.categories {
                csv.push_str(&format!(
                    "{},{},{}\r\n",
                    escape_csv_cell(&c.category_name),
                    c.count,
                    c.amount_formatted
                ));
            }
            csv.push_str(&format!(
                "TOTAL,{},{}\r\n\r\n",
                report.expense_count, report.total_expense_formatted
            ));

            csv.push_str("Cash Drawer Metric,Amount (৳)\r\n");
            csv.push_str(&format!(
                "Starting Cash,{}\r\n",
                report.drawer_reconciliation.starting_cash_formatted
            ));
            csv.push_str(&format!(
                "Cash In (Collections),{}\r\n",
                report.drawer_reconciliation.cash_collections_formatted
            ));
            csv.push_str(&format!(
                "Cash Out (Expenses),{}\r\n",
                report.drawer_reconciliation.cash_expenses_formatted
            ));
            csv.push_str(&format!(
                "Expected Balance,{}\r\n",
                report.drawer_reconciliation.expected_closing_formatted
            ));
            csv.push_str(&format!(
                "Actual Counted,{}\r\n",
                report.drawer_reconciliation.counted_closing_formatted
            ));
            csv.push_str(&format!(
                "Over/Short Discrepancy,{}\r\n",
                report.drawer_reconciliation.discrepancy_formatted
            ));
        }
        "attendance" => {
            let p: AttendanceSummaryPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_attendance_summary(conn, &p)?;
            filename = format!("attendance_{}.csv", p.month);

            csv.push_str("Student ID,Student Name,Name (Bengali),Batch,Sessions,Present,Late,Absent,Excused,Attendance %,Chronic Absentee\r\n");
            for s in report.items {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{:.1}%,{}\r\n",
                    escape_csv_cell(&s.student_code),
                    escape_csv_cell(&s.name),
                    escape_csv_cell(s.name_bn.as_deref().unwrap_or("")),
                    escape_csv_cell(&s.batch_name),
                    s.total_sessions,
                    s.present_count,
                    s.late_count,
                    s.absent_count,
                    s.excused_count,
                    s.attendance_percentage,
                    if s.is_chronic_absentee { "YES" } else { "NO" }
                ));
            }
        }
        "exam_performance" => {
            let p: ExamPerformancePayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let report = get_exam_performance(conn, &p)?;
            filename = format!(
                "exam_performance_{}.csv",
                report.exam_name.replace(' ', "_")
            );

            csv.push_str(&format!(
                "Exam Name,{}\r\n",
                escape_csv_cell(&report.exam_name)
            ));
            csv.push_str(&format!("Total Candidates,{}\r\n", report.total_candidates));
            csv.push_str(&format!("Passed Candidates,{}\r\n", report.passed_count));
            csv.push_str(&format!("Failed Candidates,{}\r\n", report.failed_count));
            csv.push_str(&format!("Pass Rate,{:.1}%\r\n", report.pass_percentage));
            csv.push_str(&format!("Average GPA,{:.2}\r\n\r\n", report.average_gpa));

            csv.push_str("Grade,Count,Percentage\r\n");
            for g in report.grade_distribution {
                csv.push_str(&format!("{},{},{:.1}%\r\n", g.grade, g.count, g.percentage));
            }

            csv.push_str("\r\nTop Rank,Student ID,Student Name,GPA,Grade,Total Marks\r\n");
            for r in report.top_rankers {
                csv.push_str(&format!(
                    "{},{},{},{:.2},{},{}\r\n",
                    r.rank,
                    escape_csv_cell(&r.student_code),
                    escape_csv_cell(&r.name),
                    r.gpa,
                    r.grade,
                    r.total_marks
                ));
            }
        }
        "students_list" => {
            let p: StudentsExportFilterPayload =
                serde_json::from_value(payload.params.clone()).unwrap_or_default();
            filename = "students_directory.csv".to_string();

            let mut sql = String::from(
                "SELECT s.student_code, s.name, s.name_bn, s.gender, s.phone_norm,
                        s.class_level, s.status, s.admission_date,
                        GROUP_CONCAT(DISTINCT g.name) AS guardian_names,
                        GROUP_CONCAT(DISTINCT g.phone_norm) AS guardian_phones,
                        GROUP_CONCAT(DISTINCT b.name) AS batch_names,
                        COALESCE(sb.balance_poisha, 0) AS due_poisha
                 FROM students s
                 LEFT JOIN student_balances sb ON sb.student_id = s.id
                 LEFT JOIN student_guardians sg ON sg.student_id = s.id
                 LEFT JOIN guardians g ON g.id = sg.guardian_id
                 LEFT JOIN enrollments e ON e.student_id = s.id AND e.status = 'active'
                 LEFT JOIN batches b ON b.id = e.batch_id
                 WHERE 1=1",
            );
            let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(ref status) = p.status {
                if !status.trim().is_empty() && status != "all" {
                    sql.push_str(" AND s.status = ?");
                    params_vec.push(Box::new(status.clone()));
                }
            }
            if let Some(ref batch_id) = p.batch_id {
                if !batch_id.trim().is_empty() && batch_id != "all" {
                    sql.push_str(" AND EXISTS (SELECT 1 FROM enrollments e2 WHERE e2.student_id = s.id AND e2.batch_id = ? AND e2.status = 'active')");
                    params_vec.push(Box::new(batch_id.clone()));
                }
            }
            if let Some(ref q) = p.search_query {
                let trimmed = q.trim();
                if !trimmed.is_empty() {
                    let like = format!("%{}%", trimmed);
                    sql.push_str(" AND (s.name LIKE ? OR s.name_bn LIKE ? OR s.student_code LIKE ? OR s.phone_norm LIKE ? OR g.phone_norm LIKE ?)");
                    for _ in 0..5 {
                        params_vec.push(Box::new(like.clone()));
                    }
                }
            }

            sql.push_str(" GROUP BY s.id ORDER BY s.student_code ASC LIMIT 50000");

            csv.push_str("Student ID,Full Name,Name (Bangla),Gender,Phone,Class,Batches,Guardian Name,Guardian Phone,Status,Admission Date,Net Due (৳)\r\n");

            let mut stmt = conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
            let mut rows = stmt.query(rusqlite::params_from_iter(param_refs))?;
            while let Some(r) = rows.next()? {
                let code: String = r.get(0)?;
                let name: String = r.get(1)?;
                let name_bn: Option<String> = r.get(2)?;
                let gender: String = r.get(3)?;
                let phone: String = r.get(4)?;
                let class_lvl: Option<String> = r.get(5)?;
                let status: String = r.get(6)?;
                let adm_date: String = r.get(7)?;
                let g_names: Option<String> = r.get(8)?;
                let g_phones: Option<String> = r.get(9)?;
                let batches: Option<String> = r.get(10)?;
                let due_poisha: i64 = r.get(11)?;

                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}\r\n",
                    escape_csv_cell(&code),
                    escape_csv_cell(&name),
                    escape_csv_cell(name_bn.as_deref().unwrap_or("")),
                    escape_csv_cell(&gender),
                    escape_csv_cell(&phone),
                    escape_csv_cell(class_lvl.as_deref().unwrap_or("")),
                    escape_csv_cell(batches.as_deref().unwrap_or("")),
                    escape_csv_cell(g_names.as_deref().unwrap_or("")),
                    escape_csv_cell(g_phones.as_deref().unwrap_or("")),
                    escape_csv_cell(&status),
                    escape_csv_cell(&adm_date),
                    format_taka(due_poisha)
                ));
            }
        }
        "dues_list" => {
            let p: DuesExportFilterPayload =
                serde_json::from_value(payload.params.clone()).unwrap_or_default();
            let filter = crate::ledger::ListDuesFilter {
                program_id: p.program_id,
                batch_id: p.batch_id,
                period: p.period,
                min_due_poisha: p.min_due_poisha,
                overdue_only: p.overdue_only,
                search: p.search,
                limit: Some(50000),
                offset: Some(0),
            };
            let dues_items = crate::ledger::list_dues(conn, filter)
                .map_err(|e| ReportsError::Sql(e.to_string()))?;
            filename = "dues_list.csv".to_string();

            csv.push_str("Student ID,Student Name,Name (Bangla),Batch,Roll,Student Phone,Guardian Name,Guardian Phone,Current Month Due (৳),Overdue (৳),Total Due (৳),Oldest Due Date\r\n");
            for item in dues_items {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}\r\n",
                    escape_csv_cell(&item.student_code),
                    escape_csv_cell(&item.name),
                    escape_csv_cell(item.name_bn.as_deref().unwrap_or("")),
                    escape_csv_cell(item.batch_name.as_deref().unwrap_or("")),
                    escape_csv_cell(item.roll_no.as_deref().unwrap_or("")),
                    escape_csv_cell(&item.phone),
                    escape_csv_cell(item.guardian_name.as_deref().unwrap_or("")),
                    escape_csv_cell(item.guardian_phone.as_deref().unwrap_or("")),
                    format_taka(item.current_month_due_poisha),
                    format_taka(item.overdue_poisha),
                    format_taka(item.total_due_poisha),
                    escape_csv_cell(item.oldest_unpaid_due_date.as_deref().unwrap_or(""))
                ));
            }
        }
        "exam_results_list" => {
            let p: ExamResultsExportFilterPayload = serde_json::from_value(payload.params.clone())
                .map_err(|e| ReportsError::Validation(e.to_string()))?;
            let exam_name: String = conn
                .query_row(
                    "SELECT name FROM exams WHERE id = ?1",
                    params![p.exam_id],
                    |r| r.get(0),
                )
                .map_err(|_| ReportsError::NotFound(format!("Exam '{}' not found", p.exam_id)))?;
            filename = format!("exam_results_{}.csv", exam_name.replace(' ', "_"));

            csv.push_str("Merit Rank,Student ID,Student Name,Name (Bangla),Total Marks,Percentage,GPA,Grade,Status\r\n");
            let mut stmt = conn.prepare(
                "SELECT r.rank, s.student_code, s.name, s.name_bn, r.total_marks, r.percentage, r.gpa, r.grade, r.passed
                 FROM results r
                 JOIN students s ON s.id = r.student_id
                 WHERE r.exam_id = ?1
                 ORDER BY r.rank ASC",
            )?;
            let mut rows = stmt.query(params![p.exam_id])?;
            while let Some(r) = rows.next()? {
                let rank: i64 = r.get(0)?;
                let code: String = r.get(1)?;
                let name: String = r.get(2)?;
                let name_bn: Option<String> = r.get(3)?;
                let total_marks: i64 = r.get(4)?;
                let pct_hundredths: i64 = r.get(5)?;
                let gpa_hundredths: i64 = r.get(6)?;
                let grade: String = r.get(7)?;
                let passed: i64 = r.get(8)?;

                csv.push_str(&format!(
                    "{},{},{},{},{},{:.2}%,{:.2},{},{}\r\n",
                    rank,
                    escape_csv_cell(&code),
                    escape_csv_cell(&name),
                    escape_csv_cell(name_bn.as_deref().unwrap_or("")),
                    total_marks,
                    (pct_hundredths as f64) / 100.0,
                    (gpa_hundredths as f64) / 100.0,
                    escape_csv_cell(&grade),
                    if passed != 0 { "PASSED" } else { "FAILED" }
                ));
            }
        }
        other => {
            return Err(ReportsError::Validation(format!(
                "Unknown report type: {}",
                other
            )));
        }
    }

    Ok(ExportCsvResponse {
        filename,
        csv_content: csv,
    })
}
