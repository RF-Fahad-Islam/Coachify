use crate::db::now_utc_iso;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AttendanceError {
    #[error("Batch with ID '{0}' not found")]
    BatchNotFound(String),
    #[error("Attendance session with ID '{0}' not found")]
    SessionNotFound(String),
    #[error("Attendance session is locked. Retroactive edits require 'attendance.edit_previous' permission")]
    Locked,
    #[error("Student with ID '{0}' is not actively enrolled in batch on date '{1}'")]
    StudentNotEnrolled(String, String),
    #[error("Invalid attendance status '{0}'. Must be 'present', 'absent', 'late', or 'excused'")]
    InvalidStatus(String),
    #[error("Retroactive edit of locked attendance session requires a non-empty audit reason")]
    MissingAuditReason,
    #[error("Database error: {0}")]
    Sql(String),
}

impl AttendanceError {
    pub fn code(&self) -> &'static str {
        match self {
            AttendanceError::BatchNotFound(_) => "batch_not_found",
            AttendanceError::SessionNotFound(_) => "session_not_found",
            AttendanceError::Locked => "attendance_locked",
            AttendanceError::StudentNotEnrolled(_, _) => "student_not_enrolled",
            AttendanceError::InvalidStatus(_) => "invalid_attendance_status",
            AttendanceError::MissingAuditReason => "missing_audit_reason",
            AttendanceError::Sql(_) => "attendance_sql_error",
        }
    }
}

impl From<rusqlite::Error> for AttendanceError {
    fn from(err: rusqlite::Error) -> Self {
        AttendanceError::Sql(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// DTOs and Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAttendanceSessionPayload {
    pub batch_id: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStudentAttendanceSummaryPayload {
    pub student_id: String,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchAttendanceMatrixPayload {
    pub batch_id: String,
    pub month: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AttendanceMarkItem {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub roll: Option<i64>,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AttendanceSessionInfo {
    pub id: Option<String>,
    pub batch_id: String,
    pub batch_name: String,
    pub date: String,
    pub session_exists: bool,
    pub locked: bool,
    pub taken_by: Option<String>,
    pub taken_by_name: Option<String>,
    pub taken_at: Option<String>,
    pub notes: Option<String>,
    pub marks: Vec<AttendanceMarkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveAttendanceMarkInput {
    pub student_id: String,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveAttendanceSessionPayload {
    pub batch_id: String,
    pub date: String,
    pub notes: Option<String>,
    pub audit_reason: Option<String>,
    pub marks: Vec<SaveAttendanceMarkInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LockAttendanceSessionPayload {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentAttendanceSummary {
    pub student_id: String,
    pub total_sessions: i64,
    pub present_count: i64,
    pub absent_count: i64,
    pub late_count: i64,
    pub excused_count: i64,
    pub attendance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchMatrixStudentRow {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub roll: Option<i64>,
    pub marks: HashMap<String, String>,
    pub attendance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchAttendanceMatrixResponse {
    pub batch_id: String,
    pub batch_name: String,
    pub month: String,
    pub dates: Vec<String>,
    pub students: Vec<BatchMatrixStudentRow>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn is_valid_status(status: &str) -> bool {
    matches!(status, "present" | "absent" | "late" | "excused")
}

/// Evaluates whether an attendance session is locked based on explicit flag and lock window rules.
/// Prevents the "midnight trap" by requiring both past date and at least 12 elapsed hours, or 24 hours total.
pub fn evaluate_is_locked(
    conn: &Connection,
    locked_flag: i64,
    session_date: &str,
    taken_at_iso: &str,
) -> bool {
    if locked_flag == 1 {
        return true;
    }

    // Check if 24 hours have elapsed since taken_at
    let is_over_24h: bool = conn
        .query_row(
            "SELECT datetime('now') > datetime(?1, '+24 hours')",
            params![taken_at_iso],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if is_over_24h {
        return true;
    }

    // Check midnight trap guard: both date < current_date AND now > taken_at + 12 hours
    let is_over_12h_and_past_date: bool = conn
        .query_row(
            "SELECT date('now') > date(?1) AND datetime('now') > datetime(?2, '+12 hours')",
            params![session_date, taken_at_iso],
            |row| row.get(0),
        )
        .unwrap_or(false);

    is_over_12h_and_past_date
}

// ---------------------------------------------------------------------------
// Core Operations
// ---------------------------------------------------------------------------

/// Retrieves the attendance session and student mark roster for a specific batch and date.
pub fn get_session(
    conn: &Connection,
    batch_id: &str,
    date: &str,
) -> Result<AttendanceSessionInfo, AttendanceError> {
    // 1. Verify batch exists
    let batch_name: String = conn
        .query_row(
            "SELECT name FROM batches WHERE id = ?1",
            params![batch_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AttendanceError::BatchNotFound(batch_id.to_string()))?;

    // 2. Fetch session header if exists
    let session_header = conn
        .query_row(
            "SELECT s.id, s.taken_by, u.display_name, s.taken_at, s.locked, s.notes
             FROM attendance_sessions s
             LEFT JOIN users u ON s.taken_by = u.id
             WHERE s.batch_id = ?1 AND s.date = ?2",
            params![batch_id, date],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .optional()?;

    // 3. Fetch all active enrollments for this batch on this date
    let mut stmt = conn.prepare(
        "SELECT e.student_id, st.student_code, st.name, e.roll
         FROM enrollments e
         JOIN students st ON e.student_id = st.id
         WHERE e.batch_id = ?1
           AND e.status = 'active'
           AND date(e.start_date) <= date(?2)
           AND (e.end_date IS NULL OR date(e.end_date) >= date(?2))
         ORDER BY e.roll ASC, st.name ASC",
    )?;

    let enrolled_students = stmt.query_map(params![batch_id, date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;

    let mut enrolled_list = Vec::new();
    for item in enrolled_students {
        enrolled_list.push(item?);
    }

    if let Some((session_id, taken_by, taken_by_name, taken_at, locked_val, notes)) = session_header
    {
        let locked = evaluate_is_locked(conn, locked_val, date, &taken_at);

        // Fetch existing marks map
        let mut mark_stmt = conn.prepare(
            "SELECT student_id, status, note FROM attendance_marks WHERE session_id = ?1",
        )?;
        let marks_rows = mark_stmt.query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut mark_map = HashMap::new();
        for m in marks_rows {
            let (student_id, status, note) = m?;
            mark_map.insert(student_id, (status, note));
        }

        let mut marks = Vec::with_capacity(enrolled_list.len());
        for (student_id, student_code, student_name, roll) in enrolled_list {
            let (status, note) = if let Some((st, nt)) = mark_map.remove(&student_id) {
                (st, nt)
            } else {
                ("present".to_string(), None)
            };

            marks.push(AttendanceMarkItem {
                student_id,
                student_code,
                student_name,
                roll,
                status,
                note,
            });
        }

        Ok(AttendanceSessionInfo {
            id: Some(session_id),
            batch_id: batch_id.to_string(),
            batch_name,
            date: date.to_string(),
            session_exists: true,
            locked,
            taken_by: Some(taken_by),
            taken_by_name,
            taken_at: Some(taken_at),
            notes,
            marks,
        })
    } else {
        // Session does not exist yet (not taken)
        let marks = enrolled_list
            .into_iter()
            .map(
                |(student_id, student_code, student_name, roll)| AttendanceMarkItem {
                    student_id,
                    student_code,
                    student_name,
                    roll,
                    status: "present".to_string(), // UI default
                    note: None,
                },
            )
            .collect();

        Ok(AttendanceSessionInfo {
            id: None,
            batch_id: batch_id.to_string(),
            batch_name,
            date: date.to_string(),
            session_exists: false,
            locked: false,
            taken_by: None,
            taken_by_name: None,
            taken_at: None,
            notes: None,
            marks,
        })
    }
}

/// Saves an attendance session and its student marks atomically.
/// Enforces lock window permissions and absence SMS triggers/cancellations.
pub fn save_session(
    conn: &Connection,
    payload: &SaveAttendanceSessionPayload,
    user_id: &str,
    has_edit_previous: bool,
) -> Result<AttendanceSessionInfo, AttendanceError> {
    // 1. Verify batch exists
    let batch_name: String = conn
        .query_row(
            "SELECT name FROM batches WHERE id = ?1",
            params![payload.batch_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AttendanceError::BatchNotFound(payload.batch_id.clone()))?;

    // 2. Validate all mark statuses
    for mark in &payload.marks {
        if !is_valid_status(&mark.status) {
            return Err(AttendanceError::InvalidStatus(mark.status.clone()));
        }
    }

    // 3. Check for existing session
    let existing_session = conn
        .query_row(
            "SELECT id, taken_at, locked FROM attendance_sessions WHERE batch_id = ?1 AND date = ?2",
            params![payload.batch_id, payload.date],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?;

    let mut old_marks_map: HashMap<String, String> = HashMap::new();

    let session_id = if let Some((id, taken_at, locked_val)) = existing_session {
        // Retrieve existing marks map to detect status transitions
        let mut old_stmt =
            conn.prepare("SELECT student_id, status FROM attendance_marks WHERE session_id = ?1")?;
        let rows = old_stmt.query_map(params![id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for r in rows {
            let (sid, st) = r?;
            old_marks_map.insert(sid, st);
        }

        let is_locked = evaluate_is_locked(conn, locked_val, &payload.date, &taken_at);
        if is_locked {
            if !has_edit_previous {
                return Err(AttendanceError::Locked);
            }
            let reason = payload.audit_reason.as_deref().unwrap_or("").trim();
            if reason.is_empty() {
                return Err(AttendanceError::MissingAuditReason);
            }

            // Audit log recording for retroactive edits (AC-4)
            let before_val = serde_json::to_value(&old_marks_map).unwrap_or_default();
            let after_val = serde_json::to_value(&payload.marks).unwrap_or_default();
            let entry = crate::audit::AuditEntry {
                action: "attendance.edit_locked".to_string(),
                entity: "attendance_sessions".to_string(),
                entity_id: id.clone(),
                before_json: Some(serde_json::json!({ "marks": before_val })),
                after_json: Some(serde_json::json!({ "marks": after_val, "reason": reason })),
            };
            let _ = crate::audit::record_audit_entry(conn, entry, None, &now_utc_iso());
        }

        // Update existing session notes and updated_at
        conn.execute(
            "UPDATE attendance_sessions
             SET notes = ?1, updated_at = ?2
             WHERE id = ?3",
            params![payload.notes, now_utc_iso(), id],
        )?;

        id
    } else {
        // Create new session header
        let id = format!("att-{}", Uuid::now_v7());
        let now = now_utc_iso();
        conn.execute(
            "INSERT INTO attendance_sessions (
                id, batch_id, date, taken_by, taken_at, locked, notes, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8)",
            params![
                id,
                payload.batch_id,
                payload.date,
                user_id,
                now,
                payload.notes,
                now,
                now
            ],
        )?;
        id
    };

    // 4. Insert/Update student marks and handle absence cancellations across SMS & WhatsApp
    for mark in &payload.marks {
        let old_status = old_marks_map.get(&mark.student_id);

        // Cancellation Hook (AC-6):
        // If student was previously 'absent' and is now marked present/late/excused,
        // cancel pending queued absence alerts across both SMS and WhatsApp.
        if let Some(prev) = old_status {
            if prev == "absent" && mark.status != "absent" {
                let sms_key = format!("attendance:{}:{}:absence_sms", session_id, mark.student_id);
                let wa_key = format!(
                    "attendance:{}:{}:absence_whatsapp",
                    session_id, mark.student_id
                );
                conn.execute(
                    "UPDATE outbox
                     SET status = 'cancelled'
                     WHERE (dedupe_key = ?1 OR dedupe_key = ?2) AND status = 'queued'",
                    params![sms_key, wa_key],
                )?;
            }
        }

        // Upsert mark
        let now = now_utc_iso();
        conn.execute(
            "INSERT INTO attendance_marks (
                session_id, student_id, status, note, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(session_id, student_id) DO UPDATE SET
                status = excluded.status,
                note = excluded.note,
                updated_at = excluded.updated_at",
            params![
                session_id,
                mark.student_id,
                mark.status,
                mark.note,
                now,
                now
            ],
        )?;

        // Omnichannel Absence Automation Hook (AC-5):
        // If student is newly marked absent, queue delayed absence alerts across enabled channels.
        if mark.status == "absent" && old_status.map(|s| s.as_str()) != Some("absent") {
            enqueue_absence_alerts_if_enabled(
                conn,
                &session_id,
                &mark.student_id,
                &payload.batch_id,
                &batch_name,
                &payload.date,
                user_id,
            )?;
        }
    }

    // Fetch and return the updated session info
    get_session(conn, &payload.batch_id, &payload.date)
}

/// Enqueues delayed absence alerts across SMS and WhatsApp if rules are enabled.
fn enqueue_absence_alerts_if_enabled(
    conn: &Connection,
    session_id: &str,
    student_id: &str,
    _batch_id: &str,
    batch_name: &str,
    session_date: &str,
    user_id: &str,
) -> Result<(), AttendanceError> {
    // 1. Fetch enabled student_absent rules across SMS and WhatsApp
    let mut stmt = conn.prepare(
        "SELECT r.channel, r.options_json, t.body
         FROM automation_rules r
         JOIN message_templates t ON r.template_id = t.id
         WHERE r.trigger = 'student_absent' AND r.enabled = 1",
    )?;

    let rules = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut rules_vec = Vec::new();
    for r in rules {
        rules_vec.push(r?);
    }

    if rules_vec.is_empty() {
        return Ok(());
    }

    // 2. Find primary guardian phone or fallback to student phone
    let phone: Option<String> = conn
        .query_row(
            "SELECT g.phone_norm
             FROM student_guardians sg
             JOIN guardians g ON sg.guardian_id = g.id
             WHERE sg.student_id = ?1 AND sg.is_primary = 1 AND sg.receives_sms = 1",
            params![student_id],
            |row| row.get(0),
        )
        .optional()?
        .or_else(|| {
            conn.query_row(
                "SELECT phone_norm FROM students WHERE id = ?1",
                params![student_id],
                |row| row.get(0),
            )
            .optional()
            .unwrap_or(None)
        });

    let to_address = match phone {
        Some(p) if !p.trim().is_empty() => p,
        _ => return Ok(()), // No recipient phone available
    };

    // 3. Fetch student details for template interpolation
    let (student_name, student_code): (String, String) = conn.query_row(
        "SELECT name, student_code FROM students WHERE id = ?1",
        params![student_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // Fetch institute info
    let inst_phone: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.phone'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "".to_string());

    let inst_name: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.name'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "Coachify".to_string());

    let now = now_utc_iso();

    for (channel, options_json, template_body) in rules_vec {
        let delay_minutes = serde_json::from_str::<serde_json::Value>(&options_json)
            .ok()
            .and_then(|v| v.get("delay_minutes").and_then(|d| d.as_i64()))
            .unwrap_or(30);

        let message_body = template_body
            .replace("{student_name}", &student_name)
            .replace("{student_code}", &student_code)
            .replace("{batch_name}", batch_name)
            .replace("{date}", session_date)
            .replace("{institution_phone}", &inst_phone)
            .replace("{institution_name}", &inst_name);

        let analysis = crate::comms::analyze_text(&message_body, &channel);
        let dedupe_key = format!(
            "attendance:{}:{}:absence_{}",
            session_id, student_id, channel
        );
        let outbox_id = format!("out-{}", Uuid::now_v7());

        conn.execute(
            "INSERT OR IGNORE INTO outbox (
                id, channel, to_address, body, segments, encoding,
                template_key, related_type, related_id, dedupe_key,
                status, scheduled_at, attempts, max_attempts,
                created_by, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                'student_absent', 'attendance', ?7, ?8,
                'queued', datetime('now', '+' || ?9 || ' minutes'), 0, 3,
                ?10, ?11
            )",
            params![
                outbox_id,
                channel,
                to_address,
                message_body,
                analysis.segment_count as i64,
                analysis.encoding,
                session_id,
                dedupe_key,
                delay_minutes,
                user_id,
                now
            ],
        )?;
    }

    Ok(())
}

/// Explicitly locks an attendance session.
pub fn lock_session(
    conn: &Connection,
    session_id: &str,
    _user_id: &str,
) -> Result<(), AttendanceError> {
    let rows_affected = conn.execute(
        "UPDATE attendance_sessions SET locked = 1, updated_at = ?1 WHERE id = ?2",
        params![now_utc_iso(), session_id],
    )?;

    if rows_affected == 0 {
        return Err(AttendanceError::SessionNotFound(session_id.to_string()));
    }

    Ok(())
}

/// Aggregates a student's attendance history and calculates percentage.
pub fn get_student_summary(
    conn: &Connection,
    student_id: &str,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Result<StudentAttendanceSummary, AttendanceError> {
    let sql = format!(
        "SELECT
            COUNT(m.session_id) as total_sessions,
            SUM(CASE WHEN m.status = 'present' THEN 1 ELSE 0 END) as present_count,
            SUM(CASE WHEN m.status = 'absent' THEN 1 ELSE 0 END) as absent_count,
            SUM(CASE WHEN m.status = 'late' THEN 1 ELSE 0 END) as late_count,
            SUM(CASE WHEN m.status = 'excused' THEN 1 ELSE 0 END) as excused_count
         FROM attendance_marks m
         JOIN attendance_sessions s ON m.session_id = s.id
         WHERE m.student_id = ?1
           {}
           {}",
        if from_date.is_some() {
            "AND date(s.date) >= date(?2)"
        } else {
            ""
        },
        if to_date.is_some() {
            if from_date.is_some() {
                "AND date(s.date) <= date(?3)"
            } else {
                "AND date(s.date) <= date(?2)"
            }
        } else {
            ""
        }
    );

    let summary = {
        let mapper = |row: &rusqlite::Row| {
            let total: i64 = row.get(0)?;
            let present: i64 = row.get::<_, Option<i64>>(1)?.unwrap_or(0);
            let absent: i64 = row.get::<_, Option<i64>>(2)?.unwrap_or(0);
            let late: i64 = row.get::<_, Option<i64>>(3)?.unwrap_or(0);
            let excused: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);

            let percentage = if total > 0 {
                ((present + late) as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            Ok(StudentAttendanceSummary {
                student_id: student_id.to_string(),
                total_sessions: total,
                present_count: present,
                absent_count: absent,
                late_count: late,
                excused_count: excused,
                attendance_percentage: (percentage * 10.0).round() / 10.0,
            })
        };

        if let (Some(from), Some(to)) = (from_date, to_date) {
            conn.query_row(&sql, params![student_id, from, to], mapper)?
        } else if let Some(from) = from_date {
            conn.query_row(&sql, params![student_id, from], mapper)?
        } else if let Some(to) = to_date {
            conn.query_row(&sql, params![student_id, to], mapper)?
        } else {
            conn.query_row(&sql, params![student_id], mapper)?
        }
    };

    Ok(summary)
}

/// Generates a monthly attendance register matrix for an entire batch.
pub fn get_batch_matrix(
    conn: &Connection,
    batch_id: &str,
    month: &str, // Format: "YYYY-MM"
) -> Result<BatchAttendanceMatrixResponse, AttendanceError> {
    let batch_name: String = conn
        .query_row(
            "SELECT name FROM batches WHERE id = ?1",
            params![batch_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AttendanceError::BatchNotFound(batch_id.to_string()))?;

    // 1. Fetch all session dates in this month
    let mut date_stmt = conn.prepare(
        "SELECT DISTINCT date FROM attendance_sessions
         WHERE batch_id = ?1 AND strftime('%Y-%m', date) = ?2
         ORDER BY date ASC",
    )?;
    let date_rows = date_stmt.query_map(params![batch_id, month], |row| row.get::<_, String>(0))?;
    let mut dates = Vec::new();
    for d in date_rows {
        dates.push(d?);
    }

    // 2. Fetch active students in batch
    let mut student_stmt = conn.prepare(
        "SELECT e.student_id, st.student_code, st.name, e.roll
         FROM enrollments e
         JOIN students st ON e.student_id = st.id
         WHERE e.batch_id = ?1 AND e.status = 'active'
         ORDER BY e.roll ASC, st.name ASC",
    )?;
    let student_rows = student_stmt.query_map(params![batch_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;

    let mut students = Vec::new();
    for s in student_rows {
        let (student_id, student_code, student_name, roll) = s?;

        // Fetch marks for this student across the month
        let mut mark_stmt = conn.prepare(
            "SELECT s.date, m.status
             FROM attendance_marks m
             JOIN attendance_sessions s ON m.session_id = s.id
             WHERE s.batch_id = ?1 AND m.student_id = ?2 AND strftime('%Y-%m', s.date) = ?3",
        )?;
        let marks_rows = mark_stmt.query_map(params![batch_id, student_id, month], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut marks = HashMap::new();
        let mut present_late_count = 0;
        let mut student_sessions = 0;
        for m in marks_rows {
            let (dt, status) = m?;
            if status == "present" || status == "late" {
                present_late_count += 1;
            }
            student_sessions += 1;
            marks.insert(dt, status);
        }

        let attendance_percentage = if student_sessions > 0 {
            ((present_late_count as f64 / student_sessions as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };

        students.push(BatchMatrixStudentRow {
            student_id,
            student_code,
            student_name,
            roll,
            marks,
            attendance_percentage,
        });
    }

    Ok(BatchAttendanceMatrixResponse {
        batch_id: batch_id.to_string(),
        batch_name,
        month: month.to_string(),
        dates,
        students,
    })
}
