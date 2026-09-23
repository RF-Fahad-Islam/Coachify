//! Owner Intelligence domain engine (Features 34 to 39, Spec 0033).
//!
//! Provides the operational brain for coaching center owners and administrators:
//! - Student 360 timeline (Feature 34)
//! - Today operations cockpit (Feature 35)
//! - Coaching day timeline (Feature 36)
//! - Batch health view (Feature 37)
//! - Batch profitability (Feature 38)
//! - Smart due collection queue (Feature 39)

use crate::schedule::ScheduleSlot;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum IntelligenceError {
    #[error("Student with ID '{0}' was not found")]
    StudentNotFound(String),
    #[error("Batch with ID '{0}' was not found")]
    BatchNotFound(String),
    #[error("Program with ID '{0}' was not found")]
    ProgramNotFound(String),
    #[error("Invalid note type '{0}'. Allowed: academic, behavior, financial, general")]
    InvalidNoteType(String),
    #[error("Invalid contact channel '{0}'. Allowed: phone_call, sms, in_person, whatsapp")]
    InvalidContactChannel(String),
    #[error("Invalid outcome '{0}'. Allowed: promised_payment, dispute, unreachable, wrong_number, follow_up_needed, resolved")]
    InvalidOutcome(String),
    #[error("Cost cannot be negative: {0}")]
    NegativeCost(String),
    #[error(
        "Invalid teacher cost type '{0}'. Allowed: monthly_fixed, per_class, percentage_share"
    )]
    InvalidTeacherCostType(String),
    #[error("Invalid date format '{0}'. Expected YYYY-MM-DD")]
    InvalidDateFormat(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntelligenceError {
    pub fn code(&self) -> &'static str {
        match self {
            IntelligenceError::StudentNotFound(_) => "student_not_found",
            IntelligenceError::BatchNotFound(_) => "batch_not_found",
            IntelligenceError::ProgramNotFound(_) => "program_not_found",
            IntelligenceError::InvalidNoteType(_) => "invalid_note_type",
            IntelligenceError::InvalidContactChannel(_) => "invalid_contact_channel",
            IntelligenceError::InvalidOutcome(_) => "invalid_outcome",
            IntelligenceError::NegativeCost(_) => "negative_cost",
            IntelligenceError::InvalidTeacherCostType(_) => "invalid_teacher_cost_type",
            IntelligenceError::InvalidDateFormat(_) => "invalid_date_format",
            IntelligenceError::Sql(_) => "intelligence_sql_error",
            IntelligenceError::Unauthorized(_) => "unauthorized",
        }
    }
}

impl From<rusqlite::Error> for IntelligenceError {
    fn from(err: rusqlite::Error) -> Self {
        IntelligenceError::Sql(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// 1. Shared Timeline Models (Features 34 & 36)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TimelineCursor {
    pub before_time: String,
    pub before_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TimelineEvent {
    pub id: String,
    pub event_time: String,
    pub category: String, // admission, enrollment, financial, attendance, academic, communication, note, settings
    pub event_type: String,
    pub actor_name: Option<String>,
    pub actor_username: Option<String>,
    pub title: String,
    pub description: String,
    pub metadata_json: Option<String>,
}

// -----------------------------------------------------------------------------
// 2. Feature 34: Student 360 Timeline DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Student360Payload {
    pub student_id: String,
    pub category: Option<String>,
    pub before_time: Option<String>,
    pub before_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Student360Response {
    pub student_id: String,
    pub student_name: String,
    pub student_code: String,
    pub student_roll: Option<i64>,
    pub total_paid_poisha: i64,
    pub total_due_poisha: i64,
    pub gpa_average: Option<f64>,
    pub events: Vec<TimelineEvent>,
    pub next_cursor: Option<TimelineCursor>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AddStudentNotePayload {
    pub student_id: String,
    pub note_type: String, // academic, behavior, financial, general
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentTimelineNote {
    pub id: String,
    pub student_id: String,
    pub author_id: String,
    pub author_name: Option<String>,
    pub note_type: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

// -----------------------------------------------------------------------------
// 3. Feature 35: Today Operations Cockpit DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct TodayCockpitPayload {
    pub target_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TodayScheduledClass {
    pub batch_id: String,
    pub batch_name: String,
    pub program_name: String,
    pub room: Option<String>,
    pub teacher_id: Option<String>,
    pub teacher_name: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub is_completed: bool,
    pub attendance_submitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MissingAttendanceAlert {
    pub batch_id: String,
    pub batch_name: String,
    pub program_name: String,
    pub teacher_id: Option<String>,
    pub teacher_name: Option<String>,
    pub teacher_phone: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub room: Option<String>,
    pub enrolled_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AbsentTeacherAlert {
    pub teacher_id: String,
    pub teacher_name: String,
    pub teacher_phone: Option<String>,
    pub batch_id: String,
    pub batch_name: String,
    pub missed_slot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PaymentMethodBreakdown {
    pub method: String,
    pub total_poisha: i64,
    pub transaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpcomingExamSummary {
    pub exam_id: String,
    pub exam_name: String,
    pub batch_name: String,
    pub exam_date: String,
    pub subject_count: u32,
    pub enrolled_candidates_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BackupHealthStatus {
    pub last_backup_at: Option<String>,
    pub is_healthy: bool,
    pub hours_since_last_backup: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TodayCockpitResponse {
    pub target_date: String,
    pub scheduled_classes: Vec<TodayScheduledClass>,
    pub missing_attendance_alerts: Vec<MissingAttendanceAlert>,
    pub absent_teachers: Vec<AbsentTeacherAlert>,
    pub today_collections_poisha: i64,
    pub collections_by_method: Vec<PaymentMethodBreakdown>,
    pub total_overdue_poisha: i64,
    pub overdue_students_count: u32,
    pub upcoming_exams: Vec<UpcomingExamSummary>,
    pub failed_messages_count: u32,
    pub backup_health: BackupHealthStatus,
}

// -----------------------------------------------------------------------------
// 4. Feature 36: Coaching Day Timeline DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoachingDayTimelinePayload {
    pub date: String,
    pub filter_category: Option<String>,
    pub before_time: Option<String>,
    pub before_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CoachingDayTimelineResponse {
    pub date: String,
    pub events: Vec<TimelineEvent>,
    pub next_cursor: Option<TimelineCursor>,
    pub has_more: bool,
}

// -----------------------------------------------------------------------------
// 5. Feature 37: Batch Health View DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct BatchHealthPayload {
    pub program_id: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchHealthItem {
    pub batch_id: String,
    pub batch_name: String,
    pub program_id: String,
    pub program_name: String,
    pub teacher_name: Option<String>,
    pub capacity: u32,
    pub enrolled_count: u32,
    pub capacity_utilization_pct: f64,
    pub thirty_day_attendance_pct: f64,
    pub total_billed_poisha: i64,
    pub total_due_poisha: i64,
    pub dues_ratio_pct: f64,
    pub inactive_students_count: u32,
    pub average_exam_gpa: Option<f64>,
    pub health_status: String, // "healthy", "warning", "critical"
    pub flags: Vec<String>,
}

// -----------------------------------------------------------------------------
// 6. Feature 38: Batch Profitability DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct BatchProfitabilityPayload {
    pub batch_id: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchProfitabilityItem {
    pub batch_id: String,
    pub batch_name: String,
    pub program_name: String,
    pub enrolled_students: u32,
    pub projected_monthly_revenue_poisha: i64,
    pub net_collected_revenue_poisha: i64,
    pub teacher_monthly_cost_poisha: i64,
    pub teacher_cost_type: String, // monthly_fixed, per_class, percentage_share
    pub room_monthly_cost_poisha: i64,
    pub operational_overhead_cost_poisha: i64,
    pub total_cost_poisha: i64,
    pub net_margin_poisha: i64,
    pub margin_percentage: f64,
    pub is_breakeven: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateBatchCostConfigPayload {
    pub batch_id: String,
    pub teacher_monthly_cost: i64,
    pub teacher_cost_type: String,
    pub room_monthly_cost: i64,
    pub operational_overhead_cost: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchCostConfig {
    pub id: String,
    pub batch_id: String,
    pub teacher_monthly_cost: i64,
    pub teacher_cost_type: String,
    pub room_monthly_cost: i64,
    pub operational_overhead_cost: i64,
    pub created_at: String,
    pub updated_at: String,
}

// -----------------------------------------------------------------------------
// 7. Feature 39: Smart Due Collection Queue DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct SmartDueQueuePayload {
    pub batch_id: Option<String>,
    pub min_overdue_days: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SmartDueQueueItem {
    pub student_id: String,
    pub student_name: String,
    pub roll_no: Option<i64>,
    pub batch_id: String,
    pub batch_name: String,
    pub primary_guardian_name: Option<String>,
    pub primary_guardian_phone: Option<String>,
    pub total_due_poisha: i64,
    pub oldest_unpaid_due_date: String,
    pub overdue_days: u32,
    pub days_since_last_contact: u32,
    pub reminder_count: u32,
    pub priority_score: f64,
    pub last_contact_outcome: Option<String>,
    pub last_contact_date: Option<String>,
    pub promised_payment_date: Option<String>,
    pub last_payment_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SmartDueQueueResponse {
    pub total_students_due: u32,
    pub total_due_poisha: i64,
    pub items: Vec<SmartDueQueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecordDueContactPayload {
    pub student_id: String,
    pub contact_channel: String, // phone_call, sms, in_person, whatsapp
    pub outcome: String, // promised_payment, dispute, unreachable, wrong_number, follow_up_needed, resolved
    pub promised_payment_date: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DueContactLogItem {
    pub id: String,
    pub student_id: String,
    pub contacted_by_user_id: String,
    pub contacted_by_name: Option<String>,
    pub contact_channel: String,
    pub outcome: String,
    pub promised_payment_date: Option<String>,
    pub notes: String,
    pub created_at: String,
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn get_current_date_str() -> String {
    let now = jiff::Timestamp::now();
    // Dhaka local time is UTC+6
    let dhaka = match now.checked_add(jiff::SignedDuration::from_hours(6)) {
        Ok(t) => t,
        Err(_) => now,
    };
    dhaka.to_string()[..10].to_string()
}

fn get_current_time_str() -> String {
    let now = jiff::Timestamp::now();
    let dhaka = match now.checked_add(jiff::SignedDuration::from_hours(6)) {
        Ok(t) => t,
        Err(_) => now,
    };
    let s = dhaka.to_string();
    if s.len() >= 16 {
        s[11..16].to_string()
    } else {
        "00:00".to_string()
    }
}

fn get_day_of_week(date_str: &str) -> String {
    if let Ok(ts) = format!("{}T00:00:00Z", date_str).parse::<jiff::Timestamp>() {
        let zoned = ts.to_zoned(jiff::tz::TimeZone::UTC);
        match zoned.date().weekday() {
            jiff::civil::Weekday::Sunday => "Sunday".to_string(),
            jiff::civil::Weekday::Monday => "Monday".to_string(),
            jiff::civil::Weekday::Tuesday => "Tuesday".to_string(),
            jiff::civil::Weekday::Wednesday => "Wednesday".to_string(),
            jiff::civil::Weekday::Thursday => "Thursday".to_string(),
            jiff::civil::Weekday::Friday => "Friday".to_string(),
            jiff::civil::Weekday::Saturday => "Saturday".to_string(),
        }
    } else {
        "Sunday".to_string()
    }
}

fn days_between(start_date: &str, end_date: &str) -> u32 {
    let d1: Result<jiff::civil::Date, _> = start_date.parse();
    let d2: Result<jiff::civil::Date, _> = end_date.parse();
    if let (Ok(s), Ok(e)) = (d1, d2) {
        if let Ok(diff) = e.since(s) {
            let days = diff.get_days();
            if days > 0 {
                return days as u32;
            }
        }
    }
    0
}

// -----------------------------------------------------------------------------
// Implementations
// -----------------------------------------------------------------------------

/// AC-1, AC-2, AC-3: Retrieve complete Student 360 timeline
pub fn get_student_360(
    conn: &Connection,
    payload: &Student360Payload,
) -> Result<Student360Response, IntelligenceError> {
    // 1. Verify student exists
    let student_row = conn
        .query_row(
            "SELECT id, name, student_code, created_at FROM students WHERE id = ?1",
            params![payload.student_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;

    let (s_id, s_name, s_code, s_created_at) = match student_row {
        Some(row) => row,
        None => {
            return Err(IntelligenceError::StudentNotFound(
                payload.student_id.clone(),
            ))
        }
    };

    // 2. Roll number lookup
    let roll_no: Option<i64> = conn
        .query_row(
            "SELECT roll FROM enrollments WHERE student_id = ?1 AND status = 'active' LIMIT 1",
            params![payload.student_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    // 3. Financial totals
    let total_paid_poisha: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_poisha), 0) FROM payments WHERE student_id = ?1 AND status IN ('posted', 'completed')",
            params![payload.student_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_due_poisha: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(
                c.net_amount_poisha - COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = c.id AND p.status IN ('posted', 'completed')
                ), 0)
            ), 0)
            FROM charges c
            WHERE c.student_id = ?1 AND c.status = 'open'",
            params![payload.student_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 4. GPA Average from published results
    let gpa_average: Option<f64> = conn
        .query_row(
            "SELECT AVG(r.gpa) / 100.0 FROM results r WHERE r.student_id = ?1",
            params![payload.student_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    // 5. Gather raw events across all domain categories
    let mut all_events: Vec<TimelineEvent> = Vec::new();
    let cat_filter = payload.category.as_deref().unwrap_or("all").to_lowercase();

    // Admission Event
    if cat_filter == "all" || cat_filter == "admission" {
        all_events.push(TimelineEvent {
            id: format!("adm_{}", s_id),
            event_time: s_created_at,
            category: "admission".to_string(),
            event_type: "student_admitted".to_string(),
            actor_name: None,
            actor_username: None,
            title: "Student Admitted".to_string(),
            description: format!("Student record initialized with ID: {}", s_code),
            metadata_json: None,
        });
    }

    // Enrollments
    if cat_filter == "all" || cat_filter == "enrollment" {
        let mut stmt = conn.prepare(
            "SELECT en.id, en.created_at, en.status, en.roll, b.name as batch_name, p.name as program_name
             FROM enrollments en
             JOIN batches b ON en.batch_id = b.id
             JOIN programs p ON b.program_id = p.id
             WHERE en.student_id = ?1"
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        for row in rows {
            let (en_id, en_time, status, roll, b_name, p_name) = row?;
            let roll_str = roll.map(|r| format!(" (Roll: {})", r)).unwrap_or_default();
            all_events.push(TimelineEvent {
                id: format!("en_{}", en_id),
                event_time: en_time,
                category: "enrollment".to_string(),
                event_type: format!("enrollment_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("Enrolled in {}", b_name),
                description: format!("Program: {}{}", p_name, roll_str),
                metadata_json: None,
            });
        }
    }

    // Financial: Payments & Charges
    if cat_filter == "all" || cat_filter == "financial" {
        // Payments
        let mut stmt = conn.prepare(
            "SELECT p.id, p.received_at, p.receipt_no, p.amount_poisha, p.method, p.status, u.display_name, u.username
             FROM payments p
             LEFT JOIN users u ON p.collector_user_id = u.id
             WHERE p.student_id = ?1"
        )?;
        let p_rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        })?;
        for row in p_rows {
            let (p_id, p_time, receipt, amount, method, status, actor, username) = row?;
            all_events.push(TimelineEvent {
                id: format!("pay_{}", p_id),
                event_time: p_time,
                category: "financial".to_string(),
                event_type: format!("payment_{}", status),
                actor_name: actor,
                actor_username: username,
                title: format!("Payment ৳{:.2} Received", amount as f64 / 100.0),
                description: format!(
                    "Receipt #{}, Method: {}, Status: {}",
                    receipt, method, status
                ),
                metadata_json: None,
            });
        }

        // Charges
        let mut stmt = conn.prepare(
            "SELECT c.id, c.created_at, fh.name, c.net_amount_poisha, c.due_date, c.status
             FROM charges c
             JOIN fee_heads fh ON c.fee_head_id = fh.id
             WHERE c.student_id = ?1",
        )?;
        let c_rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        for row in c_rows {
            let (c_id, c_time, fh_name, amount, due_date, status) = row?;
            all_events.push(TimelineEvent {
                id: format!("chg_{}", c_id),
                event_time: c_time,
                category: "financial".to_string(),
                event_type: format!("charge_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("Fee Billed: {} (৳{:.2})", fh_name, amount as f64 / 100.0),
                description: format!("Due Date: {}, Status: {}", due_date, status),
                metadata_json: None,
            });
        }
    }

    // Attendance
    if cat_filter == "all" || cat_filter == "attendance" {
        let mut stmt = conn.prepare(
            "SELECT ar.session_id || '_' || ar.student_id, s.date || 'T12:00:00Z', ar.status, b.name, coalesce(s.notes, 'Class Session')
             FROM attendance_marks ar
             JOIN attendance_sessions s ON ar.session_id = s.id
             JOIN batches b ON s.batch_id = b.id
             WHERE ar.student_id = ?1"
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        for row in rows {
            let (ar_id, ar_time, status, b_name, slot) = row?;
            let status_capitalized = match status.as_str() {
                "present" => "Present",
                "absent" => "Absent",
                "late" => "Late",
                "excused" => "Excused",
                other => other,
            };
            all_events.push(TimelineEvent {
                id: format!("att_{}", ar_id),
                event_time: ar_time,
                category: "attendance".to_string(),
                event_type: format!("attendance_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("Attendance: {} ({})", status_capitalized, b_name),
                description: format!("Slot: {}", slot),
                metadata_json: None,
            });
        }
    }

    // Academic: Exams & Marks
    if cat_filter == "all" || cat_filter == "academic" {
        let mut stmt = conn.prepare(
            "SELECT (m.exam_subject_id || '_' || m.student_id), coalesce(m.updated_at, m.created_at), e.name as exam_name, sub.name as subject_name,
                    m.marks, es.full_marks, m.is_absent
             FROM marks m
             JOIN exam_subjects es ON m.exam_subject_id = es.id
             JOIN exams e ON es.exam_id = e.id
             JOIN subjects sub ON es.subject_id = sub.id
             WHERE m.student_id = ?1"
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)? != 0,
            ))
        })?;
        for row in rows {
            let (m_id, m_time, exam_name, sub_name, score, full_marks, is_absent) = row?;
            let score_str = if is_absent {
                "Absent".to_string()
            } else if let Some(s) = score {
                format!("{:.1} / {}", s as f64 / 100.0, full_marks as f64 / 100.0)
            } else {
                "Pending".to_string()
            };
            all_events.push(TimelineEvent {
                id: format!("mark_{}", m_id),
                event_time: m_time,
                category: "academic".to_string(),
                event_type: "exam_mark".to_string(),
                actor_name: None,
                actor_username: None,
                title: format!("{} - {}", exam_name, sub_name),
                description: format!("Score: {}", score_str),
                metadata_json: None,
            });
        }
    }

    // Communication: Outbound SMS to student or guardians
    if cat_filter == "all" || cat_filter == "communication" {
        let mut stmt = conn.prepare(
            "SELECT o.id, o.created_at, o.to_address, o.body, o.status
             FROM outbox o
             WHERE o.to_address IN (
                 SELECT phone_norm FROM guardians g
                 JOIN student_guardians sg ON g.id = sg.guardian_id
                 WHERE sg.student_id = ?1
             )",
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        for row in rows {
            let (o_id, o_time, phone, text, status) = row?;
            all_events.push(TimelineEvent {
                id: format!("sms_{}", o_id),
                event_time: o_time,
                category: "communication".to_string(),
                event_type: format!("sms_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("SMS to {}", phone),
                description: format!("[{}] {}", status, text),
                metadata_json: None,
            });
        }
    }

    // Notes: Staff Timeline Notes & Due Contact Logs
    if cat_filter == "all" || cat_filter == "note" {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.created_at, n.note_type, n.title, n.content, u.display_name, u.username
             FROM student_timeline_notes n
             JOIN users u ON n.author_id = u.id
             WHERE n.student_id = ?1",
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?;
        for row in rows {
            let (n_id, n_time, note_type, title, content, actor, username) = row?;
            all_events.push(TimelineEvent {
                id: format!("note_{}", n_id),
                event_time: n_time,
                category: "note".to_string(),
                event_type: format!("staff_note_{}", note_type),
                actor_name: actor,
                actor_username: username,
                title: format!("[{}] {}", note_type.to_uppercase(), title),
                description: content,
                metadata_json: None,
            });
        }

        // Contact logs
        let mut stmt = conn.prepare(
            "SELECT cl.id, cl.created_at, cl.contact_channel, cl.outcome, cl.promised_payment_date, cl.notes, u.display_name, u.username
             FROM due_collection_contact_logs cl
             JOIN users u ON cl.contacted_by_user_id = u.id
             WHERE cl.student_id = ?1"
        )?;
        let rows = stmt.query_map(params![payload.student_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        })?;
        for row in rows {
            let (cl_id, cl_time, channel, outcome, promise, notes, actor, username) = row?;
            let promise_str = promise
                .map(|p| format!(" (Promised: {})", p))
                .unwrap_or_default();
            all_events.push(TimelineEvent {
                id: format!("contact_{}", cl_id),
                event_time: cl_time,
                category: "note".to_string(),
                event_type: "due_contact_log".to_string(),
                actor_name: actor,
                actor_username: username,
                title: format!("Due Contact: {} via {}{}", outcome, channel, promise_str),
                description: notes,
                metadata_json: None,
            });
        }
    }

    // 6. Sort deterministically: event_time DESC, id DESC
    all_events.sort_by(|a, b| {
        b.event_time
            .cmp(&a.event_time)
            .then_with(|| b.id.cmp(&a.id))
    });

    // 7. Cursor Pagination
    let filtered_events: Vec<TimelineEvent> =
        if let (Some(ref b_time), Some(ref b_id)) = (&payload.before_time, &payload.before_id) {
            all_events
                .into_iter()
                .filter(|e| {
                    if e.event_time == *b_time {
                        e.id < *b_id
                    } else {
                        e.event_time < *b_time
                    }
                })
                .collect()
        } else {
            all_events
        };

    let limit = payload.limit.unwrap_or(50) as usize;
    let has_more = filtered_events.len() > limit;
    let page_events: Vec<TimelineEvent> = filtered_events.into_iter().take(limit).collect();

    let next_cursor = if has_more {
        page_events.last().map(|e| TimelineCursor {
            before_time: e.event_time.clone(),
            before_id: e.id.clone(),
        })
    } else {
        None
    };

    Ok(Student360Response {
        student_id: s_id,
        student_name: s_name,
        student_code: s_code,
        student_roll: roll_no,
        total_paid_poisha,
        total_due_poisha,
        gpa_average,
        events: page_events,
        next_cursor,
        has_more,
    })
}

/// AC-4: Add a staff note to student's timeline
pub fn add_student_note(
    conn: &Connection,
    author_id: &str,
    payload: &AddStudentNotePayload,
) -> Result<StudentTimelineNote, IntelligenceError> {
    // Validate student
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM students WHERE id = ?1",
            params![payload.student_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !exists {
        return Err(IntelligenceError::StudentNotFound(
            payload.student_id.clone(),
        ));
    }

    let note_type = payload.note_type.trim().to_lowercase();
    if !matches!(
        note_type.as_str(),
        "academic" | "behavior" | "financial" | "general"
    ) {
        return Err(IntelligenceError::InvalidNoteType(
            payload.note_type.clone(),
        ));
    }

    let note_id = format!("note-{}", uuid::Uuid::now_v7());
    let now = jiff::Timestamp::now().to_string();

    conn.execute(
        "INSERT INTO student_timeline_notes (id, student_id, author_id, note_type, title, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            note_id,
            payload.student_id,
            author_id,
            note_type,
            payload.title.trim(),
            payload.content.trim(),
            now
        ],
    )?;

    let author_name: Option<String> = conn
        .query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            params![author_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    Ok(StudentTimelineNote {
        id: note_id,
        student_id: payload.student_id.clone(),
        author_id: author_id.to_string(),
        author_name,
        note_type,
        title: payload.title.clone(),
        content: payload.content.clone(),
        created_at: now,
    })
}

// -----------------------------------------------------------------------------
// Feature 35: Today Operations Cockpit
// -----------------------------------------------------------------------------

/// AC-1 through AC-6: Retrieve Today Operations Cockpit
pub fn get_today_cockpit(
    conn: &Connection,
    payload: &TodayCockpitPayload,
) -> Result<TodayCockpitResponse, IntelligenceError> {
    let target_date = payload
        .target_date
        .clone()
        .unwrap_or_else(get_current_date_str);

    let day_of_week = get_day_of_week(&target_date);
    let current_time = get_current_time_str();

    // 1. Scheduled classes today
    let mut stmt = conn.prepare(
        "SELECT b.id, b.name, p.name, b.room_number, b.teacher_staff_id, s.name, b.schedule_json
         FROM batches b
         JOIN programs p ON b.program_id = p.id
         LEFT JOIN staff s ON b.teacher_staff_id = s.id
         WHERE b.status = 'active'",
    )?;

    let batch_rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, String>(6)?,
        ))
    })?;

    let mut scheduled_classes: Vec<TodayScheduledClass> = Vec::new();
    let mut missing_attendance_alerts: Vec<MissingAttendanceAlert> = Vec::new();
    let mut absent_teachers: Vec<AbsentTeacherAlert> = Vec::new();

    for row in batch_rows {
        let (b_id, b_name, p_name, room, teacher_id, teacher_name, schedule_json) = row?;
        let slots: Vec<ScheduleSlot> = serde_json::from_str(&schedule_json).unwrap_or_default();

        for slot in slots {
            if slot.day_of_week.eq_ignore_ascii_case(&day_of_week) {
                // Check if attendance session exists today
                let has_attendance: bool = conn
                    .query_row(
                        "SELECT 1 FROM attendance_sessions WHERE batch_id = ?1 AND date = ?2 LIMIT 1",
                        params![b_id, target_date],
                        |_| Ok(true),
                    )
                    .optional()?
                    .unwrap_or(false);

                // Class end time has passed?
                let is_completed = slot.end_time <= current_time;

                scheduled_classes.push(TodayScheduledClass {
                    batch_id: b_id.clone(),
                    batch_name: b_name.clone(),
                    program_name: p_name.clone(),
                    room: slot.room.clone().or_else(|| room.clone()),
                    teacher_id: teacher_id.clone(),
                    teacher_name: teacher_name.clone(),
                    start_time: slot.start_time.clone(),
                    end_time: slot.end_time.clone(),
                    is_completed,
                    attendance_submitted: has_attendance,
                });

                // Missing attendance alert: class end time passed AND no attendance session
                if is_completed && !has_attendance {
                    let enrolled_count: u32 = conn
                        .query_row(
                            "SELECT COUNT(*) FROM enrollments WHERE batch_id = ?1 AND status = 'active'",
                            params![b_id],
                            |r| r.get(0),
                        )
                        .unwrap_or(0);

                    let teacher_phone: Option<String> = if let Some(ref t_id) = teacher_id {
                        conn.query_row(
                            "SELECT phone_norm FROM staff WHERE id = ?1",
                            params![t_id],
                            |r| r.get(0),
                        )
                        .optional()?
                        .flatten()
                    } else {
                        None
                    };

                    missing_attendance_alerts.push(MissingAttendanceAlert {
                        batch_id: b_id.clone(),
                        batch_name: b_name.clone(),
                        program_name: p_name.clone(),
                        teacher_id: teacher_id.clone(),
                        teacher_name: teacher_name.clone(),
                        teacher_phone: teacher_phone.clone(),
                        start_time: slot.start_time.clone(),
                        end_time: slot.end_time.clone(),
                        room: slot.room.clone().or_else(|| room.clone()),
                        enrolled_count,
                    });

                    // Flag absent teacher
                    if let (Some(t_id), Some(t_name)) = (&teacher_id, &teacher_name) {
                        absent_teachers.push(AbsentTeacherAlert {
                            teacher_id: t_id.clone(),
                            teacher_name: t_name.clone(),
                            teacher_phone,
                            batch_id: b_id.clone(),
                            batch_name: b_name.clone(),
                            missed_slot: format!("{} - {}", slot.start_time, slot.end_time),
                        });
                    }
                }
            }
        }
    }

    // Sort scheduled classes by start_time
    scheduled_classes.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    // 2. Collections today
    let start_of_day = format!("{}T00:00:00", target_date);
    let end_of_day = format!("{}T23:59:59.999Z", target_date);

    let today_collections_poisha: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount_poisha), 0) FROM payments
             WHERE status IN ('posted', 'completed')
               AND received_at >= ?1 AND received_at <= ?2",
            params![start_of_day, end_of_day],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // Breakdown by payment method
    let mut stmt = conn.prepare(
        "SELECT method, COALESCE(SUM(amount_poisha), 0), COUNT(*)
         FROM payments
         WHERE status IN ('posted', 'completed')
           AND received_at >= ?1 AND received_at <= ?2
         GROUP BY method
         ORDER BY SUM(amount_poisha) DESC",
    )?;
    let method_rows = stmt.query_map(params![start_of_day, end_of_day], |r| {
        Ok(PaymentMethodBreakdown {
            method: r.get(0)?,
            total_poisha: r.get(1)?,
            transaction_count: r.get(2)?,
        })
    })?;
    let collections_by_method = method_rows.collect::<Result<Vec<_>, _>>()?;

    // 3. Total overdue dues across center
    let (total_overdue_poisha, overdue_students_count) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(remaining_due), 0),
                COUNT(DISTINCT student_id)
             FROM (
                 SELECT
                     c.student_id,
                     (c.net_amount_poisha - COALESCE((
                         SELECT SUM(pa.amount_poisha)
                         FROM payment_allocations pa
                         JOIN payments p ON pa.payment_id = p.id
                         WHERE pa.charge_id = c.id AND p.status IN ('posted', 'completed')
                     ), 0)) as remaining_due
                 FROM charges c
                 WHERE c.status = 'open' AND c.due_date < ?1
             )
             WHERE remaining_due > 0",
            params![target_date],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, u32>(1)?)),
        )
        .unwrap_or((0, 0));

    // 4. Upcoming exams in the next 7 days
    let mut upcoming_exams: Vec<UpcomingExamSummary> = Vec::new();
    let max_exam_date = if let Ok(d) = target_date.parse::<jiff::civil::Date>() {
        let span = jiff::Span::new().days(7);
        if let Ok(next_d) = d.checked_add(span) {
            next_d.to_string()
        } else {
            target_date.clone()
        }
    } else {
        target_date.clone()
    };

    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, coalesce(b.name, 'All Batches'), e.start_date,
                (SELECT COUNT(*) FROM exam_subjects es WHERE es.exam_id = e.id),
                (SELECT COUNT(*) FROM enrollments en WHERE en.batch_id = eb.batch_id AND en.status = 'active')
         FROM exams e
         LEFT JOIN exam_batches eb ON e.id = eb.exam_id
         LEFT JOIN batches b ON eb.batch_id = b.id
         WHERE e.start_date >= ?1 AND e.start_date <= ?2
         ORDER BY e.start_date ASC"
    )?;
    let exam_rows = stmt.query_map(params![target_date, max_exam_date], |r| {
        Ok(UpcomingExamSummary {
            exam_id: r.get(0)?,
            exam_name: r.get(1)?,
            batch_name: r.get(2)?,
            exam_date: r.get(3)?,
            subject_count: r.get(4)?,
            enrolled_candidates_count: r.get(5)?,
        })
    })?;
    for ex in exam_rows {
        upcoming_exams.push(ex?);
    }

    // 5. Message failures in last 24h
    let one_day_ago =
        if let Ok(ts) = jiff::Timestamp::now().checked_sub(jiff::SignedDuration::from_hours(24)) {
            ts.to_string()
        } else {
            target_date.clone()
        };
    let failed_messages_count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM outbox WHERE status = 'failed' AND created_at >= ?1",
            params![one_day_ago],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 6. Backup health
    let last_backup_at: Option<String> = conn
        .query_row(
            "SELECT created_at FROM backup_history WHERE status = 'completed' ORDER BY created_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let (is_healthy, hours_since_last_backup) = if let Some(ref bk_time) = last_backup_at {
        if let Ok(ts) = bk_time.parse::<jiff::Timestamp>() {
            if let Ok(dur) = jiff::Timestamp::now().since(ts) {
                let hours = dur.get_hours() as u32;
                (hours < 24, Some(hours))
            } else {
                (true, Some(0))
            }
        } else {
            (true, Some(0))
        }
    } else {
        (false, None)
    };

    Ok(TodayCockpitResponse {
        target_date,
        scheduled_classes,
        missing_attendance_alerts,
        absent_teachers,
        today_collections_poisha,
        collections_by_method,
        total_overdue_poisha,
        overdue_students_count,
        upcoming_exams,
        failed_messages_count,
        backup_health: BackupHealthStatus {
            last_backup_at,
            is_healthy,
            hours_since_last_backup,
        },
    })
}

// -----------------------------------------------------------------------------
// Feature 36: Coaching Day Timeline
// -----------------------------------------------------------------------------

/// AC-1 through AC-4: Coaching Day Timeline feed
pub fn get_coaching_day_timeline(
    conn: &Connection,
    payload: &CoachingDayTimelinePayload,
) -> Result<CoachingDayTimelineResponse, IntelligenceError> {
    let date_str = &payload.date;
    let start_of_day = format!("{}T00:00:00", date_str);
    let end_of_day = format!("{}T23:59:59.999Z", date_str);

    let cat_filter = payload
        .filter_category
        .as_deref()
        .unwrap_or("all")
        .to_lowercase();

    let mut all_events: Vec<TimelineEvent> = Vec::new();

    // 1. Finance (Payments)
    if cat_filter == "all" || cat_filter == "finance" {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.received_at, p.receipt_no, p.amount_poisha, p.method, u.display_name, u.username, s.name
             FROM payments p
             JOIN students s ON p.student_id = s.id
             LEFT JOIN users u ON p.collector_user_id = u.id
             WHERE p.received_at >= ?1 AND p.received_at <= ?2"
        )?;
        let rows = stmt.query_map(params![start_of_day, end_of_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
            ))
        })?;
        for row in rows {
            let (p_id, p_time, receipt, amount, method, actor, username, student_name) = row?;
            all_events.push(TimelineEvent {
                id: format!("pay_{}", p_id),
                event_time: p_time,
                category: "finance".to_string(),
                event_type: "counter_payment".to_string(),
                actor_name: actor,
                actor_username: username,
                title: format!("Payment ৳{:.2} Collected", amount as f64 / 100.0),
                description: format!("Receipt #{} from {} via {}", receipt, student_name, method),
                metadata_json: None,
            });
        }
    }

    // 2. Admissions
    if cat_filter == "all" || cat_filter == "admissions" {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.created_at, s.name, s.student_code
             FROM students s
             WHERE s.created_at >= ?1 AND s.created_at <= ?2",
        )?;
        let rows = stmt.query_map(params![start_of_day, end_of_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        for row in rows {
            let (s_id, s_time, s_name, code) = row?;
            all_events.push(TimelineEvent {
                id: format!("adm_{}", s_id),
                event_time: s_time,
                category: "admissions".to_string(),
                event_type: "student_admitted".to_string(),
                actor_name: None,
                actor_username: None,
                title: format!("New Admission: {}", s_name),
                description: format!("Assigned Student ID: {}", code),
                metadata_json: None,
            });
        }
    }

    // 3. Attendance
    if cat_filter == "all" || cat_filter == "attendance" {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.created_at, b.name, coalesce(s.notes, 'Class Session'), u.display_name, u.username,
                    (SELECT COUNT(*) FROM attendance_marks ar WHERE ar.session_id = s.id AND ar.status = 'present') as present_cnt,
                    (SELECT COUNT(*) FROM attendance_marks ar WHERE ar.session_id = s.id) as total_cnt
             FROM attendance_sessions s
             JOIN batches b ON s.batch_id = b.id
             LEFT JOIN users u ON s.taken_by = u.id
             WHERE s.date = ?1"
        )?;
        let rows = stmt.query_map(params![date_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, u32>(6)?,
                r.get::<_, u32>(7)?,
            ))
        })?;
        for row in rows {
            let (s_id, s_time, b_name, slot, actor, username, present_cnt, total_cnt) = row?;
            all_events.push(TimelineEvent {
                id: format!("att_{}", s_id),
                event_time: s_time,
                category: "attendance".to_string(),
                event_type: "attendance_marked".to_string(),
                actor_name: actor,
                actor_username: username,
                title: format!("Attendance Marked: {}", b_name),
                description: format!("Slot: {} (Present: {}/{})", slot, present_cnt, total_cnt),
                metadata_json: None,
            });
        }
    }

    // 4. Exams
    if cat_filter == "all" || cat_filter == "exams" {
        let mut stmt = conn.prepare(
            "SELECT e.id, coalesce(e.updated_at, e.created_at), e.name, coalesce(b.name, 'All Batches'), e.status
             FROM exams e
             LEFT JOIN exam_batches eb ON e.id = eb.exam_id
             LEFT JOIN batches b ON eb.batch_id = b.id
             WHERE e.start_date = ?1 OR (e.updated_at >= ?2 AND e.updated_at <= ?3)"
        )?;
        let rows = stmt.query_map(params![date_str, start_of_day, end_of_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        for row in rows {
            let (e_id, e_time, e_name, b_name, status) = row?;
            all_events.push(TimelineEvent {
                id: format!("exam_{}", e_id),
                event_time: e_time,
                category: "exams".to_string(),
                event_type: format!("exam_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("Exam {}: {}", status.to_uppercase(), e_name),
                description: format!("Batch: {}", b_name),
                metadata_json: None,
            });
        }
    }

    // 5. Messages
    if cat_filter == "all" || cat_filter == "messages" {
        let mut stmt = conn.prepare(
            "SELECT o.id, o.created_at, o.to_address, o.body, o.status
             FROM outbox o
             WHERE o.created_at >= ?1 AND o.created_at <= ?2",
        )?;
        let rows = stmt.query_map(params![start_of_day, end_of_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        for row in rows {
            let (o_id, o_time, phone, text, status) = row?;
            all_events.push(TimelineEvent {
                id: format!("sms_{}", o_id),
                event_time: o_time,
                category: "messages".to_string(),
                event_type: format!("sms_{}", status),
                actor_name: None,
                actor_username: None,
                title: format!("SMS [{}] to {}", status, phone),
                description: text,
                metadata_json: None,
            });
        }
    }

    // 6. Settings / Audit Log
    if cat_filter == "all" || cat_filter == "settings" {
        let mut stmt = conn.prepare(
            "SELECT a.seq, a.at, a.action, a.entity, a.entity_id, a.user_display_name, u.username
             FROM audit_log a
             LEFT JOIN users u ON a.user_id = u.id
             WHERE a.business_date = ?1",
        )?;
        let rows = stmt.query_map(params![date_str], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?;
        for row in rows {
            let (seq, a_time, action, entity, entity_id, actor, username) = row?;
            all_events.push(TimelineEvent {
                id: format!("audit_{}", seq),
                event_time: a_time,
                category: "settings".to_string(),
                event_type: action.clone(),
                actor_name: actor,
                actor_username: username,
                title: format!("Audit: {}", action),
                description: format!("Target entity: {} ({})", entity, entity_id),
                metadata_json: None,
            });
        }
    }

    // Sort deterministically: event_time DESC, id DESC
    all_events.sort_by(|a, b| {
        b.event_time
            .cmp(&a.event_time)
            .then_with(|| b.id.cmp(&a.id))
    });

    // Pagination
    let filtered_events: Vec<TimelineEvent> =
        if let (Some(ref b_time), Some(ref b_id)) = (&payload.before_time, &payload.before_id) {
            all_events
                .into_iter()
                .filter(|e| {
                    if e.event_time == *b_time {
                        e.id < *b_id
                    } else {
                        e.event_time < *b_time
                    }
                })
                .collect()
        } else {
            all_events
        };

    let limit = payload.limit.unwrap_or(50) as usize;
    let has_more = filtered_events.len() > limit;
    let page_events: Vec<TimelineEvent> = filtered_events.into_iter().take(limit).collect();

    let next_cursor = if has_more {
        page_events.last().map(|e| TimelineCursor {
            before_time: e.event_time.clone(),
            before_id: e.id.clone(),
        })
    } else {
        None
    };

    Ok(CoachingDayTimelineResponse {
        date: date_str.clone(),
        events: page_events,
        next_cursor,
        has_more,
    })
}

// -----------------------------------------------------------------------------
// Feature 37: Batch Health View
// -----------------------------------------------------------------------------

/// AC-1 through AC-5: Batch health diagnostics
pub fn get_batch_health_list(
    conn: &Connection,
    payload: &BatchHealthPayload,
) -> Result<Vec<BatchHealthItem>, IntelligenceError> {
    let mut sql = String::from(
        "SELECT b.id, b.name, b.program_id, p.name, s.name, b.capacity
         FROM batches b
         JOIN programs p ON b.program_id = p.id
         LEFT JOIN staff s ON b.teacher_staff_id = s.id
         WHERE b.status = 'active'",
    );

    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(ref p_id) = payload.program_id {
        if !p_id.trim().is_empty() {
            sql.push_str(" AND b.program_id = ?1");
            params_vec.push(rusqlite::types::Value::Text(p_id.clone()));
        }
    }

    sql.push_str(" ORDER BY p.name ASC, b.name ASC");

    let mut stmt = conn.prepare(&sql)?;
    let batch_rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, u32>(5)?,
        ))
    })?;

    let mut items: Vec<BatchHealthItem> = Vec::new();
    let thirty_days_ago = if let Ok(ts) =
        jiff::Timestamp::now().checked_sub(jiff::SignedDuration::from_hours(24 * 30))
    {
        ts.to_string()[..10].to_string()
    } else {
        "2000-01-01".to_string()
    };

    for row in batch_rows {
        let (b_id, b_name, p_id, p_name, teacher_name, capacity) = row?;

        // 1. Enrolled students count
        let enrolled_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM enrollments WHERE batch_id = ?1 AND status = 'active'",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let capacity_utilization_pct = if capacity > 0 {
            (enrolled_count as f64 / capacity as f64) * 100.0
        } else {
            0.0
        };

        // 2. 30-day attendance rate
        let (present_marks, total_opportunities): (u32, u32) = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN ar.status = 'present' THEN 1 ELSE 0 END), 0),
                    COUNT(ar.session_id)
                 FROM attendance_marks ar
                 JOIN attendance_sessions s ON ar.session_id = s.id
                 WHERE s.batch_id = ?1 AND s.date >= ?2",
                params![b_id, thirty_days_ago],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((0, 0));

        let thirty_day_attendance_pct = if total_opportunities > 0 {
            (present_marks as f64 / total_opportunities as f64) * 100.0
        } else {
            100.0 // Default healthy if no sessions held yet
        };

        // 3. Dues & Total Billed
        let total_billed_poisha: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(c.net_amount_poisha), 0)
                 FROM charges c
                 JOIN enrollments en ON (c.enrollment_id = en.id OR (c.enrollment_id IS NULL AND c.student_id = en.student_id))
                 WHERE en.batch_id = ?1 AND en.status = 'active' AND c.status != 'voided'",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_due_poisha: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(
                    c.net_amount_poisha - COALESCE((
                        SELECT SUM(pa.amount_poisha)
                        FROM payment_allocations pa
                        JOIN payments p ON pa.payment_id = p.id
                        WHERE pa.charge_id = c.id AND p.status IN ('posted', 'completed')
                    ), 0)
                 ), 0)
                 FROM charges c
                 JOIN enrollments en ON (c.enrollment_id = en.id OR (c.enrollment_id IS NULL AND c.student_id = en.student_id))
                 WHERE en.batch_id = ?1 AND en.status = 'active' AND c.status = 'open'",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let dues_ratio_pct = if total_billed_poisha > 0 {
            (total_due_poisha as f64 / total_billed_poisha as f64) * 100.0
        } else {
            0.0
        };

        // 4. Inactive students (enrolled inactive status or 0 attendance in last 30 days)
        let inactive_students_count: u32 = conn
            .query_row(
                "SELECT COUNT(DISTINCT en.student_id)
                 FROM enrollments en
                 WHERE en.batch_id = ?1 AND (
                     en.status != 'active' OR
                     (en.status = 'active' AND NOT EXISTS (
                         SELECT 1 FROM attendance_marks ar
                         JOIN attendance_sessions s ON ar.session_id = s.id
                         WHERE ar.student_id = en.student_id AND s.batch_id = ?1 AND s.date >= ?2
                     ) AND EXISTS (
                         SELECT 1 FROM attendance_sessions s WHERE s.batch_id = ?1 AND s.date >= ?2
                     ))
                 )",
                params![b_id, thirty_days_ago],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // 5. Average exam GPA
        let average_exam_gpa: Option<f64> = conn
            .query_row(
                "SELECT AVG(r.gpa) / 100.0
                 FROM results r
                 WHERE r.batch_id = ?1",
                params![b_id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        // 6. Threshold evaluation (AC-2):
        // Thresholds: attendance < 60%, dues > 30%, capacity < 50%
        // Critical: 2+ thresholds breach OR attendance < 50% (when sessions exist)
        // Warning: 1 threshold breaches
        // Healthy: otherwise
        let mut failed_thresholds = 0;
        let mut flags: Vec<String> = Vec::new();

        if total_opportunities > 0 && thirty_day_attendance_pct < 60.0 {
            failed_thresholds += 1;
            flags.push(format!(
                "Low attendance ({:.1}%)",
                thirty_day_attendance_pct
            ));
        }

        if total_billed_poisha > 0 && dues_ratio_pct > 30.0 {
            failed_thresholds += 1;
            flags.push(format!("High dues ratio ({:.1}%)", dues_ratio_pct));
        }

        if capacity > 0 && capacity_utilization_pct < 50.0 {
            failed_thresholds += 1;
            flags.push(format!(
                "Low capacity utilization ({:.1}%)",
                capacity_utilization_pct
            ));
        }

        let health_status = if (total_opportunities > 0 && thirty_day_attendance_pct < 50.0)
            || failed_thresholds >= 2
        {
            "critical".to_string()
        } else if failed_thresholds == 1 {
            "warning".to_string()
        } else {
            "healthy".to_string()
        };

        items.push(BatchHealthItem {
            batch_id: b_id,
            batch_name: b_name,
            program_id: p_id,
            program_name: p_name,
            teacher_name,
            capacity,
            enrolled_count,
            capacity_utilization_pct,
            thirty_day_attendance_pct,
            total_billed_poisha,
            total_due_poisha,
            dues_ratio_pct,
            inactive_students_count,
            average_exam_gpa,
            health_status,
            flags,
        });
    }

    Ok(items)
}

// -----------------------------------------------------------------------------
// Feature 38: Batch Profitability
// -----------------------------------------------------------------------------

/// AC-1, AC-3: Calculate batch profitability
pub fn get_batch_profitability(
    conn: &Connection,
    payload: &BatchProfitabilityPayload,
) -> Result<Vec<BatchProfitabilityItem>, IntelligenceError> {
    let mut sql = String::from(
        "SELECT b.id, b.name, p.name
         FROM batches b
         JOIN programs p ON b.program_id = p.id
         WHERE b.status = 'active'",
    );

    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(ref b_id) = payload.batch_id {
        if !b_id.trim().is_empty() {
            sql.push_str(" AND b.id = ?1");
            params_vec.push(rusqlite::types::Value::Text(b_id.clone()));
        }
    }

    sql.push_str(" ORDER BY p.name ASC, b.name ASC");

    let mut stmt = conn.prepare(&sql)?;
    let batch_rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;

    let mut items: Vec<BatchProfitabilityItem> = Vec::new();
    let thirty_days_ago = if let Ok(ts) =
        jiff::Timestamp::now().checked_sub(jiff::SignedDuration::from_hours(24 * 30))
    {
        ts.to_string()[..10].to_string()
    } else {
        "2000-01-01".to_string()
    };

    for row in batch_rows {
        let (b_id, b_name, p_name) = row?;

        // 1. Enrolled students
        let enrolled_students: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM enrollments WHERE batch_id = ?1 AND status = 'active'",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // 2. Projected monthly revenue: fee plans for batch or program * active students
        let monthly_fee_sum: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(fp.amount_poisha), 0)
                 FROM fee_plans fp
                 JOIN batches b ON b.id = ?1
                 WHERE (fp.batch_id = b.id OR (fp.program_id = b.program_id AND fp.batch_id IS NULL))
                   AND fp.frequency = 'monthly'",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let projected_monthly_revenue_poisha = monthly_fee_sum * (enrolled_students as i64);

        // 3. Net collected revenue for this batch
        let net_collected_revenue_poisha: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(pa.amount_poisha), 0)
                 FROM payment_allocations pa
                 JOIN charges c ON pa.charge_id = c.id
                 JOIN enrollments en ON (c.enrollment_id = en.id OR (c.enrollment_id IS NULL AND c.student_id = en.student_id))
                 JOIN payments p ON pa.payment_id = p.id
                 WHERE en.batch_id = ?1 AND p.status IN ('posted', 'completed')",
                params![b_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // 4. Batch cost configuration
        let cost_config = conn
            .query_row(
                "SELECT teacher_monthly_cost, teacher_cost_type, room_monthly_cost, operational_overhead_cost
                 FROM batch_cost_configs WHERE batch_id = ?1",
                params![b_id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()?;

        let (t_cost_input, t_type, room_cost, overhead_cost) = match cost_config {
            Some(cfg) => cfg,
            None => (0, "monthly_fixed".to_string(), 0, 0),
        };

        // 5. Calculate teacher compensation based on cost type
        let teacher_cost_poisha = match t_type.as_str() {
            "per_class" => {
                let classes_held: i64 = conn
                    .query_row(
                        "SELECT COUNT(DISTINCT date) FROM attendance_sessions WHERE batch_id = ?1 AND date >= ?2",
                        params![b_id, thirty_days_ago],
                        |r| r.get(0),
                    )
                    .unwrap_or(1);
                t_cost_input * classes_held.max(1)
            }
            "percentage_share" => {
                // If <= 100, treated as percentage (e.g. 40 = 40%)
                if t_cost_input <= 100 {
                    (net_collected_revenue_poisha * t_cost_input) / 100
                } else {
                    (net_collected_revenue_poisha * t_cost_input) / 10000
                }
            }
            _ => t_cost_input, // monthly_fixed
        };

        let total_cost_poisha = teacher_cost_poisha + room_cost + overhead_cost;
        let net_margin_poisha = net_collected_revenue_poisha - total_cost_poisha;

        let margin_percentage = if net_collected_revenue_poisha > 0 {
            (net_margin_poisha as f64 / net_collected_revenue_poisha as f64) * 100.0
        } else {
            0.0
        };

        let is_breakeven = net_margin_poisha >= 0;

        items.push(BatchProfitabilityItem {
            batch_id: b_id,
            batch_name: b_name,
            program_name: p_name,
            enrolled_students,
            projected_monthly_revenue_poisha,
            net_collected_revenue_poisha,
            teacher_monthly_cost_poisha: teacher_cost_poisha,
            teacher_cost_type: t_type,
            room_monthly_cost_poisha: room_cost,
            operational_overhead_cost_poisha: overhead_cost,
            total_cost_poisha,
            net_margin_poisha,
            margin_percentage,
            is_breakeven,
        });
    }

    Ok(items)
}

/// AC-2: Update batch cost config
pub fn update_batch_cost_config(
    conn: &Connection,
    payload: &UpdateBatchCostConfigPayload,
) -> Result<BatchCostConfig, IntelligenceError> {
    // Validate batch
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM batches WHERE id = ?1",
            params![payload.batch_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !exists {
        return Err(IntelligenceError::BatchNotFound(payload.batch_id.clone()));
    }

    // Validate non-negative costs
    if payload.teacher_monthly_cost < 0 {
        return Err(IntelligenceError::NegativeCost(
            "teacher_monthly_cost".to_string(),
        ));
    }
    if payload.room_monthly_cost < 0 {
        return Err(IntelligenceError::NegativeCost(
            "room_monthly_cost".to_string(),
        ));
    }
    if payload.operational_overhead_cost < 0 {
        return Err(IntelligenceError::NegativeCost(
            "operational_overhead_cost".to_string(),
        ));
    }

    let t_type = payload.teacher_cost_type.trim().to_lowercase();
    if !matches!(
        t_type.as_str(),
        "monthly_fixed" | "per_class" | "percentage_share"
    ) {
        return Err(IntelligenceError::InvalidTeacherCostType(
            payload.teacher_cost_type.clone(),
        ));
    }

    let now = jiff::Timestamp::now().to_string();

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM batch_cost_configs WHERE batch_id = ?1",
            params![payload.batch_id],
            |r| r.get(0),
        )
        .optional()?;

    let config_id = if let Some(id) = existing_id {
        conn.execute(
            "UPDATE batch_cost_configs
             SET teacher_monthly_cost = ?1,
                 teacher_cost_type = ?2,
                 room_monthly_cost = ?3,
                 operational_overhead_cost = ?4,
                 updated_at = ?5
             WHERE id = ?6",
            params![
                payload.teacher_monthly_cost,
                t_type,
                payload.room_monthly_cost,
                payload.operational_overhead_cost,
                now,
                id
            ],
        )?;
        id
    } else {
        let new_id = format!("bcc-{}", uuid::Uuid::now_v7());
        conn.execute(
            "INSERT INTO batch_cost_configs (id, batch_id, teacher_monthly_cost, teacher_cost_type, room_monthly_cost, operational_overhead_cost, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                new_id,
                payload.batch_id,
                payload.teacher_monthly_cost,
                t_type,
                payload.room_monthly_cost,
                payload.operational_overhead_cost,
                now,
                now
            ],
        )?;
        new_id
    };

    Ok(BatchCostConfig {
        id: config_id,
        batch_id: payload.batch_id.clone(),
        teacher_monthly_cost: payload.teacher_monthly_cost,
        teacher_cost_type: t_type,
        room_monthly_cost: payload.room_monthly_cost,
        operational_overhead_cost: payload.operational_overhead_cost,
        created_at: now.clone(),
        updated_at: now,
    })
}

// -----------------------------------------------------------------------------
// Feature 39: Smart Due Collection Queue
// -----------------------------------------------------------------------------

/// AC-1 through AC-3, AC-7: Get Smart Due Collection Queue
pub fn get_smart_due_queue(
    conn: &Connection,
    payload: &SmartDueQueuePayload,
) -> Result<SmartDueQueueResponse, IntelligenceError> {
    let today = get_current_date_str();

    // 1. Gather all students with remaining overdue charges
    let mut sql = String::from(
        "SELECT
            s.id,
            s.name,
            en.roll,
            b.id as batch_id,
            b.name as batch_name,
            c.due_date,
            (c.net_amount_poisha - COALESCE((
                SELECT SUM(pa.amount_poisha)
                FROM payment_allocations pa
                JOIN payments p ON pa.payment_id = p.id
                WHERE pa.charge_id = c.id AND p.status IN ('posted', 'completed')
            ), 0)) as remaining_due
         FROM charges c
         JOIN students s ON c.student_id = s.id
         LEFT JOIN enrollments en ON (c.enrollment_id = en.id OR (c.enrollment_id IS NULL AND c.student_id = en.student_id)) AND en.status = 'active'
         LEFT JOIN batches b ON en.batch_id = b.id
         WHERE c.status = 'open' AND c.due_date < ?1 AND s.status = 'active'"
    );

    let mut params_vec: Vec<rusqlite::types::Value> =
        vec![rusqlite::types::Value::Text(today.clone())];

    if let Some(ref b_id) = payload.batch_id {
        if !b_id.trim().is_empty() {
            sql.push_str(" AND b.id = ?2");
            params_vec.push(rusqlite::types::Value::Text(b_id.clone()));
        }
    }

    sql.push_str(" ORDER BY s.id ASC, c.due_date ASC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |r| {
        Ok((
            r.get::<_, String>(0)?,         // s.id
            r.get::<_, String>(1)?,         // s.name
            r.get::<_, Option<i64>>(2)?,    // en.roll
            r.get::<_, Option<String>>(3)?, // b.id
            r.get::<_, Option<String>>(4)?, // b.name
            r.get::<_, String>(5)?,         // c.due_date
            r.get::<_, i64>(6)?,            // remaining_due
        ))
    })?;

    // Group overdue charges by student
    struct StudentDueAccumulator {
        student_id: String,
        student_name: String,
        roll: Option<i64>,
        batch_id: String,
        batch_name: String,
        oldest_due_date: String,
        total_due_poisha: i64,
    }

    let mut student_map: std::collections::HashMap<String, StudentDueAccumulator> =
        std::collections::HashMap::new();

    for row in rows {
        let (s_id, s_name, roll, b_id_opt, b_name_opt, due_date, remaining_due) = row?;
        if remaining_due <= 0 {
            continue;
        }

        let b_id = b_id_opt.unwrap_or_default();
        let b_name = b_name_opt.unwrap_or_else(|| "Unassigned".to_string());

        let entry = student_map
            .entry(s_id.clone())
            .or_insert_with(|| StudentDueAccumulator {
                student_id: s_id,
                student_name: s_name,
                roll,
                batch_id: b_id,
                batch_name: b_name,
                oldest_due_date: due_date.clone(),
                total_due_poisha: 0,
            });

        entry.total_due_poisha += remaining_due;
        if due_date < entry.oldest_due_date {
            entry.oldest_due_date = due_date;
        }
    }

    let min_days = payload.min_overdue_days.unwrap_or(0);
    let mut queue_items: Vec<SmartDueQueueItem> = Vec::new();
    let mut total_due_center: i64 = 0;

    for (_, acc) in student_map {
        let overdue_days = days_between(&acc.oldest_due_date, &today);
        if overdue_days < min_days {
            continue;
        }

        total_due_center += acc.total_due_poisha;

        // Primary Guardian info
        let guardian_info: Option<(String, String)> = conn
            .query_row(
                "SELECT g.name, g.phone_norm
                 FROM student_guardians sg
                 JOIN guardians g ON sg.guardian_id = g.id
                 WHERE sg.student_id = ?1 AND sg.is_primary = 1
                 LIMIT 1",
                params![acc.student_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;

        let (g_name, g_phone) = match guardian_info {
            Some((n, p)) => (Some(n), Some(p)),
            None => (None, None),
        };

        // Last contact details & days since last contact
        let last_contact = conn
            .query_row(
                "SELECT outcome, promised_payment_date, created_at
                 FROM due_collection_contact_logs
                 WHERE student_id = ?1
                 ORDER BY created_at DESC LIMIT 1",
                params![acc.student_id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;

        let (last_outcome, promise_date, contact_time) = match last_contact {
            Some((o, p, t)) => (Some(o), p, Some(t)),
            None => (None, None, None),
        };

        let days_since_last_contact = if let Some(ref t) = contact_time {
            let contact_date = &t[..10.min(t.len())];
            days_between(contact_date, &today)
        } else {
            overdue_days.max(30)
        };

        // Total reminders count: sms + contact logs
        let contact_log_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM due_collection_contact_logs WHERE student_id = ?1",
                params![acc.student_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let sms_count: u32 = if let Some(ref ph) = g_phone {
            conn.query_row(
                "SELECT COUNT(*) FROM outbox WHERE to_address = ?1",
                params![ph],
                |r| r.get(0),
            )
            .unwrap_or(0)
        } else {
            0
        };

        let reminder_count = contact_log_count + sms_count;

        // AC-2 & AC-7 Formula:
        // priority_score = (overdue_days * 1.5) + (total_due_poisha / 100000.0) - (days_since_last_contact * 2.0) + (reminder_count * 5.0)
        let priority_score = (overdue_days as f64 * 1.5)
            + (acc.total_due_poisha as f64 / 100_000.0)
            - (days_since_last_contact as f64 * 2.0)
            + (reminder_count as f64 * 5.0);

        // Last payment date
        let last_payment_date: Option<String> = conn
            .query_row(
                "SELECT received_at FROM payments WHERE student_id = ?1 AND status IN ('posted', 'completed') ORDER BY received_at DESC LIMIT 1",
                params![acc.student_id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        queue_items.push(SmartDueQueueItem {
            student_id: acc.student_id,
            student_name: acc.student_name,
            roll_no: acc.roll,
            batch_id: acc.batch_id,
            batch_name: acc.batch_name,
            primary_guardian_name: g_name,
            primary_guardian_phone: g_phone,
            total_due_poisha: acc.total_due_poisha,
            oldest_unpaid_due_date: acc.oldest_due_date,
            overdue_days,
            days_since_last_contact,
            reminder_count,
            priority_score,
            last_contact_outcome: last_outcome,
            last_contact_date: contact_time,
            promised_payment_date: promise_date,
            last_payment_date,
        });
    }

    // Sort descending by priority_score
    queue_items.sort_by(|a, b| {
        b.priority_score
            .partial_cmp(&a.priority_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_students_due = queue_items.len() as u32;

    // Apply offset and limit
    let offset = payload.offset.unwrap_or(0) as usize;
    let limit = payload.limit.unwrap_or(50) as usize;

    let paged_items = queue_items.into_iter().skip(offset).take(limit).collect();

    Ok(SmartDueQueueResponse {
        total_students_due,
        total_due_poisha: total_due_center,
        items: paged_items,
    })
}

/// AC-4, AC-8: Record due contact log
pub fn record_due_contact(
    conn: &Connection,
    contacted_by_user_id: &str,
    payload: &RecordDueContactPayload,
) -> Result<DueContactLogItem, IntelligenceError> {
    // Validate student exists
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM students WHERE id = ?1",
            params![payload.student_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !exists {
        return Err(IntelligenceError::StudentNotFound(
            payload.student_id.clone(),
        ));
    }

    let channel = payload.contact_channel.trim().to_lowercase();
    if !matches!(
        channel.as_str(),
        "phone_call" | "sms" | "in_person" | "whatsapp"
    ) {
        return Err(IntelligenceError::InvalidContactChannel(
            payload.contact_channel.clone(),
        ));
    }

    let outcome = payload.outcome.trim().to_lowercase();
    if !matches!(
        outcome.as_str(),
        "promised_payment"
            | "dispute"
            | "unreachable"
            | "wrong_number"
            | "follow_up_needed"
            | "resolved"
    ) {
        return Err(IntelligenceError::InvalidOutcome(payload.outcome.clone()));
    }

    let log_id = format!("dcl-{}", uuid::Uuid::now_v7());
    let now = jiff::Timestamp::now().to_string();

    conn.execute(
        "INSERT INTO due_collection_contact_logs (id, student_id, contacted_by_user_id, contact_channel, outcome, promised_payment_date, notes, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            log_id,
            payload.student_id,
            contacted_by_user_id,
            channel,
            outcome,
            payload.promised_payment_date.as_deref(),
            payload.notes.trim(),
            now
        ],
    )?;

    let user_name: Option<String> = conn
        .query_row(
            "SELECT display_name FROM users WHERE id = ?1",
            params![contacted_by_user_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    Ok(DueContactLogItem {
        id: log_id,
        student_id: payload.student_id.clone(),
        contacted_by_user_id: contacted_by_user_id.to_string(),
        contacted_by_name: user_name,
        contact_channel: channel,
        outcome,
        promised_payment_date: payload.promised_payment_date.clone(),
        notes: payload.notes.clone(),
        created_at: now,
    })
}
