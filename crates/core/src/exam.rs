use crate::db::now_utc_iso;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ExamError {
    #[error("Exam with ID '{0}' not found")]
    ExamNotFound(String),
    #[error("Academic session with ID '{0}' not found")]
    SessionNotFound(String),
    #[error("Grading scheme with ID '{0}' not found")]
    GradingSchemeNotFound(String),
    #[error("Exam subject with ID '{0}' not found")]
    ExamSubjectNotFound(String),
    #[error("Subject with ID '{0}' not found")]
    SubjectNotFound(String),
    #[error("Student with ID '{0}' not found")]
    StudentNotFound(String),
    #[error("Batch with ID '{0}' not found")]
    BatchNotFound(String),
    #[error("Mark {marks} exceeds full marks {full_marks}")]
    MarkOutOfRange { marks: i64, full_marks: i64 },
    #[error("Mark cannot be negative: {0}")]
    MarkNegative(i64),
    #[error(
        "Cannot verify subject: {unrecorded_count} student(s) lack recorded marks or absent status"
    )]
    MarksIncomplete { unrecorded_count: usize },
    #[error(
        "Exam '{0}' is already published. Modifications require un-publishing to verified state."
    )]
    ExamAlreadyPublished(String),
    #[error("Exam '{0}' is not verified yet. All subjects must be verified before publishing.")]
    ExamNotVerified(String),
    #[error("Editing verified marks requires 'exam.edit_verified' permission")]
    UnauthorizedVerifiedEdit,
    #[error("Result snapshot with ID '{0}' not found")]
    SnapshotNotFound(String),
    #[error("Published results are immutable")]
    ResultsImmutable,
    #[error("Subject '{0}' is already assigned to this exam")]
    DuplicateExamSubject(String),
    #[error("Database error: {0}")]
    Sql(String),
}

impl ExamError {
    pub fn code(&self) -> &'static str {
        match self {
            ExamError::ExamNotFound(_) => "exam_not_found",
            ExamError::SessionNotFound(_) => "session_not_found",
            ExamError::GradingSchemeNotFound(_) => "grading_scheme_not_found",
            ExamError::ExamSubjectNotFound(_) => "exam_subject_not_found",
            ExamError::SubjectNotFound(_) => "subject_not_found",
            ExamError::StudentNotFound(_) => "student_not_found",
            ExamError::BatchNotFound(_) => "batch_not_found",
            ExamError::MarkOutOfRange { .. } => "mark_out_of_range",
            ExamError::MarkNegative(_) => "mark_negative",
            ExamError::MarksIncomplete { .. } => "marks_incomplete",
            ExamError::ExamAlreadyPublished(_) => "exam_already_published",
            ExamError::ExamNotVerified(_) => "exam_not_verified",
            ExamError::UnauthorizedVerifiedEdit => "unauthorized_verified_edit",
            ExamError::SnapshotNotFound(_) => "snapshot_not_found",
            ExamError::ResultsImmutable => "results_immutable",
            ExamError::DuplicateExamSubject(_) => "duplicate_exam_subject",
            ExamError::Sql(_) => "exam_sql_error",
        }
    }
}

impl From<rusqlite::Error> for ExamError {
    fn from(err: rusqlite::Error) -> Self {
        ExamError::Sql(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// DTOs and Specta Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GradeBandItem {
    pub id: String,
    pub grade: String,
    pub grade_point: i64,    // hundredths: 500 = 5.00
    pub min_percentage: i64, // hundredths: 8000 = 80.00%
    pub max_percentage: i64, // hundredths: 10000 = 100.00%
    pub is_fail: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GradingSchemeSummary {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub scale_type: String, // "gpa_5", "gpa_4", "letter_only", "percentage"
    pub is_system: bool,
    pub description: Option<String>,
    pub bands: Vec<GradeBandItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamSummary {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub session_id: String,
    pub session_name: String,
    pub exam_type: String, // "term", "class_test", "model_test", "final", "quiz"
    pub start_date: String,
    pub end_date: Option<String>,
    pub grading_scheme_id: String,
    pub grading_scheme_name: String,
    pub status: String, // "draft", "entry", "verified", "published"
    pub batch_count: i64,
    pub subject_count: i64,
    pub total_students: i64,
    pub verified_subjects_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamSubjectConfigInput {
    pub subject_id: String,
    pub full_marks: i64, // hundredths: 10000 = 100.00
    pub pass_marks: i64, // hundredths: 3300 = 33.00
    pub exam_date: Option<String>,
    pub start_time: Option<String>,
    pub room_number: Option<String>,
    pub is_optional_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExamPayload {
    pub name: String,
    pub name_bn: Option<String>,
    pub session_id: String,
    pub exam_type: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub grading_scheme_id: String,
    pub batch_ids: Vec<String>,
    pub subjects: Vec<ExamSubjectConfigInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateExamPayload {
    pub exam_id: String,
    pub name: Option<String>,
    pub name_bn: Option<String>,
    pub exam_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub grading_scheme_id: Option<String>,
    pub batch_ids: Option<Vec<String>>,
    pub subjects: Option<Vec<ExamSubjectConfigInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetExamPayload {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListExamsPayload {
    pub session_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteExamPayload {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetMarkSheetPayload {
    pub exam_subject_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerifySubjectMarksPayload {
    pub exam_subject_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UnverifySubjectMarksPayload {
    pub exam_subject_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreviewResultsPayload {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RevertPublishedPayload {
    pub exam_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStudentReportCardPayload {
    pub exam_id: String,
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchTabulationSheetPayload {
    pub exam_id: String,
    pub batch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamBatchItem {
    pub batch_id: String,
    pub batch_name: String,
    pub batch_code: String,
    pub student_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamSubjectItem {
    pub id: String,
    pub exam_id: String,
    pub subject_id: String,
    pub subject_name: String,
    pub subject_name_bn: Option<String>,
    pub subject_code: String,
    pub full_marks: i64,
    pub pass_marks: i64,
    pub exam_date: Option<String>,
    pub start_time: Option<String>,
    pub room_number: Option<String>,
    pub is_optional_allowed: bool,
    pub is_verified: bool,
    pub entered_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamDetails {
    pub exam: ExamSummary,
    pub batches: Vec<ExamBatchItem>,
    pub subjects: Vec<ExamSubjectItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MarkSheetStudentRow {
    pub student_id: String,
    pub roll: Option<i64>,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub marks: Option<i64>, // hundredths, None if absent or unentered
    pub is_absent: bool,
    pub is_optional: bool,
    pub status: String, // "draft", "verified"
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MarkSheetData {
    pub exam_id: String,
    pub exam_name: String,
    pub exam_subject_id: String,
    pub subject_id: String,
    pub subject_name: String,
    pub subject_code: String,
    pub full_marks: i64,
    pub pass_marks: i64,
    pub is_verified: bool,
    pub rows: Vec<MarkSheetStudentRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveMarksRowInput {
    pub student_id: String,
    pub marks: Option<i64>, // hundredths
    pub is_absent: bool,
    pub is_optional: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveMarksPayload {
    pub exam_subject_id: String,
    pub marks: Vec<SaveMarksRowInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResultSubjectDetailItem {
    pub exam_subject_id: String,
    pub subject_name: String,
    pub subject_code: String,
    pub marks: Option<i64>, // hundredths
    pub full_marks: i64,
    pub pass_marks: i64,
    pub is_absent: bool,
    pub is_optional: bool,
    pub grade: String,
    pub grade_point: i64, // hundredths: 500 = 5.00
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResultPreviewStudentRow {
    pub student_id: String,
    pub roll: Option<i64>,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub batch_id: String,
    pub batch_name: String,
    pub total_marks: i64,
    pub total_max_marks: i64,
    pub percentage: i64, // basis points
    pub gpa: i64,        // hundredths
    pub grade: String,
    pub passed: bool,
    pub rank: i64,
    pub subjects: Vec<ResultSubjectDetailItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamResultPreviewData {
    pub exam_id: String,
    pub exam_name: String,
    pub grading_scheme_name: String,
    pub total_students: usize,
    pub passed_students: usize,
    pub failed_students: usize,
    pub average_gpa: i64,
    pub unverified_subjects: Vec<String>,
    pub students: Vec<ResultPreviewStudentRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PublishExamPayload {
    pub exam_id: String,
    #[serde(default)]
    pub send_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PublishExamResponse {
    pub snapshot_id: String,
    pub version: i64,
    pub published_at: String,
    pub student_count: usize,
    pub notifications_enqueued: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentReportCardData {
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub institution_address: String,
    pub institution_phones: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub roll: Option<i64>,
    pub batch_name: String,
    pub program_name: String,
    pub exam_name: String,
    pub session_name: String,
    pub total_marks: i64,
    pub total_max_marks: i64,
    pub percentage: i64,
    pub gpa: i64,
    pub grade: String,
    pub passed: bool,
    pub rank: i64,
    pub total_students: i64,
    pub subjects: Vec<ResultSubjectDetailItem>,
    pub attendance_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TabulationSubjectHeader {
    pub exam_subject_id: String,
    pub subject_name: String,
    pub subject_code: String,
    pub full_marks: i64,
    pub pass_marks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TabulationMarkCell {
    pub marks: Option<i64>,
    pub grade: String,
    pub grade_point: i64,
    pub is_absent: bool,
    pub is_optional: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TabulationStudentRow {
    pub student_id: String,
    pub roll: Option<i64>,
    pub student_code: String,
    pub student_name: String,
    pub marks_by_subject: HashMap<String, TabulationMarkCell>,
    pub total_marks: i64,
    pub gpa: i64,
    pub grade: String,
    pub passed: bool,
    pub rank: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchTabulationSheetData {
    pub exam_id: String,
    pub exam_name: String,
    pub batch_id: String,
    pub batch_name: String,
    pub session_name: String,
    pub subjects: Vec<TabulationSubjectHeader>,
    pub rows: Vec<TabulationStudentRow>,
}

// ---------------------------------------------------------------------------
// Domain Logic Implementation
// ---------------------------------------------------------------------------

pub fn list_grading_schemes(conn: &Connection) -> Result<Vec<GradingSchemeSummary>, ExamError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, name_bn, scale_type, is_system, description
         FROM grading_schemes
         ORDER BY is_system DESC, name ASC",
    )?;

    let schemes_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)? == 1,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    let mut result = Vec::new();
    for row in schemes_rows {
        let (id, name, name_bn, scale_type, is_system, description) = row?;

        let mut band_stmt = conn.prepare(
            "SELECT id, grade, grade_point, min_percentage, max_percentage, is_fail
             FROM grade_bands
             WHERE scheme_id = ?1
             ORDER BY min_percentage DESC",
        )?;

        let bands = band_stmt
            .query_map(params![id], |b_row| {
                Ok(GradeBandItem {
                    id: b_row.get(0)?,
                    grade: b_row.get(1)?,
                    grade_point: b_row.get(2)?,
                    min_percentage: b_row.get(3)?,
                    max_percentage: b_row.get(4)?,
                    is_fail: b_row.get::<_, i64>(5)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        result.push(GradingSchemeSummary {
            id,
            name,
            name_bn,
            scale_type,
            is_system,
            description,
            bands,
        });
    }

    Ok(result)
}

pub fn get_exam_details(conn: &Connection, exam_id: &str) -> Result<ExamDetails, ExamError> {
    let exam_row = conn
        .query_row(
            "SELECT e.id, e.name, e.name_bn, e.session_id, s.name as session_name,
                    e.exam_type, e.start_date, e.end_date, e.grading_scheme_id,
                    gs.name as scheme_name, e.status
             FROM exams e
             JOIN academic_sessions s ON s.id = e.session_id
             JOIN grading_schemes gs ON gs.id = e.grading_scheme_id
             WHERE e.id = ?1",
            params![exam_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(exam_id.to_string()))?;

    // Batches
    let mut batch_stmt = conn.prepare(
        "SELECT b.id, b.name, b.code,
                (SELECT count(distinct e.student_id) FROM enrollments e WHERE e.batch_id = b.id AND e.status = 'active') as student_count
         FROM exam_batches eb
         JOIN batches b ON b.id = eb.batch_id
         WHERE eb.exam_id = ?1
         ORDER BY b.name ASC",
    )?;

    let batches = batch_stmt
        .query_map(params![exam_id], |row| {
            Ok(ExamBatchItem {
                batch_id: row.get(0)?,
                batch_name: row.get(1)?,
                batch_code: row.get(2)?,
                student_count: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total_students: i64 = batches.iter().map(|b| b.student_count).sum();

    // Subjects
    let mut sub_stmt = conn.prepare(
        "SELECT es.id, es.exam_id, es.subject_id, s.name, s.name_bn, s.code,
                es.full_marks, es.pass_marks, es.exam_date, es.start_time, es.room_number,
                es.is_optional_allowed,
                (SELECT count(*) FROM marks m WHERE m.exam_subject_id = es.id AND m.status = 'verified') as verified_marks_count,
                (SELECT count(*) FROM marks m WHERE m.exam_subject_id = es.id) as entered_count
         FROM exam_subjects es
         JOIN subjects s ON s.id = es.subject_id
         WHERE es.exam_id = ?1
         ORDER BY es.exam_date ASC, s.name ASC",
    )?;

    let subjects = sub_stmt
        .query_map(params![exam_id], |row| {
            let verified_marks_count: i64 = row.get(12)?;
            let total_count = total_students;
            let is_verified = total_count > 0 && verified_marks_count >= total_count;

            Ok(ExamSubjectItem {
                id: row.get(0)?,
                exam_id: row.get(1)?,
                subject_id: row.get(2)?,
                subject_name: row.get(3)?,
                subject_name_bn: row.get(4)?,
                subject_code: row.get(5)?,
                full_marks: row.get(6)?,
                pass_marks: row.get(7)?,
                exam_date: row.get(8)?,
                start_time: row.get(9)?,
                room_number: row.get(10)?,
                is_optional_allowed: row.get::<_, i64>(11)? == 1,
                is_verified,
                entered_count: row.get(13)?,
                total_count,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let verified_subjects_count = subjects.iter().filter(|s| s.is_verified).count() as i64;
    let subject_count = subjects.len() as i64;
    let batch_count = batches.len() as i64;

    let exam_summary = ExamSummary {
        id: exam_row.0,
        name: exam_row.1,
        name_bn: exam_row.2,
        session_id: exam_row.3,
        session_name: exam_row.4,
        exam_type: exam_row.5,
        start_date: exam_row.6,
        end_date: exam_row.7,
        grading_scheme_id: exam_row.8,
        grading_scheme_name: exam_row.9,
        status: exam_row.10,
        batch_count,
        subject_count,
        total_students,
        verified_subjects_count,
    };

    Ok(ExamDetails {
        exam: exam_summary,
        batches,
        subjects,
    })
}

pub fn list_exams(
    conn: &Connection,
    session_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<ExamSummary>, ExamError> {
    let mut query = String::from(
        "SELECT e.id, e.name, e.name_bn, e.session_id, s.name as session_name,
                e.exam_type, e.start_date, e.end_date, e.grading_scheme_id,
                gs.name as scheme_name, e.status,
                (SELECT count(*) FROM exam_batches eb WHERE eb.exam_id = e.id) as batch_count,
                (SELECT count(*) FROM exam_subjects es WHERE es.exam_id = e.id) as subject_count,
                (SELECT count(distinct en.student_id)
                 FROM exam_batches eb
                 JOIN enrollments en ON en.batch_id = eb.batch_id AND en.status = 'active'
                 WHERE eb.exam_id = e.id) as total_students,
                (SELECT count(*)
                 FROM exam_subjects es
                 WHERE es.exam_id = e.id
                   AND (SELECT count(*) FROM marks m WHERE m.exam_subject_id = es.id AND m.status = 'verified') > 0
                   AND (SELECT count(*) FROM marks m WHERE m.exam_subject_id = es.id AND m.status = 'verified') >=
                       (SELECT count(distinct en2.student_id)
                        FROM exam_batches eb2
                        JOIN enrollments en2 ON en2.batch_id = eb2.batch_id AND en2.status = 'active'
                        WHERE eb2.exam_id = e.id)) as verified_subjects_count
         FROM exams e
         JOIN academic_sessions s ON s.id = e.session_id
         JOIN grading_schemes gs ON gs.id = e.grading_scheme_id
         WHERE 1=1 ",
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(sid) = session_id {
        query.push_str("AND e.session_id = ? ");
        params_vec.push(Box::new(sid.to_string()));
    }
    if let Some(st) = status {
        query.push_str("AND e.status = ? ");
        params_vec.push(Box::new(st.to_string()));
    }
    query.push_str("ORDER BY e.start_date DESC, e.created_at DESC");

    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(&params_slice[..], |row| {
        Ok(ExamSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            name_bn: row.get(2)?,
            session_id: row.get(3)?,
            session_name: row.get(4)?,
            exam_type: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            grading_scheme_id: row.get(8)?,
            grading_scheme_name: row.get(9)?,
            status: row.get(10)?,
            batch_count: row.get(11)?,
            subject_count: row.get(12)?,
            total_students: row.get(13)?,
            verified_subjects_count: row.get(14)?,
        })
    })?;

    let mut exams = Vec::new();
    for r in rows {
        exams.push(r?);
    }
    Ok(exams)
}

pub fn create_exam(
    conn: &Connection,
    payload: &CreateExamPayload,
    user_id: &str,
) -> Result<ExamDetails, ExamError> {
    let exam_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO exams (id, name, name_bn, session_id, exam_type, start_date, end_date,
                            grading_scheme_id, status, rules_config_json, created_by, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'draft', '{}', ?9, ?10, ?10)",
        params![
            exam_id,
            payload.name,
            payload.name_bn,
            payload.session_id,
            payload.exam_type,
            payload.start_date,
            payload.end_date,
            payload.grading_scheme_id,
            user_id,
            now
        ],
    )?;

    for batch_id in &payload.batch_ids {
        conn.execute(
            "INSERT INTO exam_batches (exam_id, batch_id) VALUES (?1, ?2)",
            params![exam_id, batch_id],
        )?;
    }

    for sub in &payload.subjects {
        let sub_id = Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO exam_subjects (id, exam_id, subject_id, full_marks, pass_marks,
                                        exam_date, start_time, room_number, is_optional_allowed, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![
                sub_id,
                exam_id,
                sub.subject_id,
                sub.full_marks,
                sub.pass_marks,
                sub.exam_date,
                sub.start_time,
                sub.room_number,
                if sub.is_optional_allowed { 1 } else { 0 },
                now
            ],
        )?;
    }

    get_exam_details(conn, &exam_id)
}

pub fn update_exam(
    conn: &Connection,
    payload: &UpdateExamPayload,
) -> Result<ExamDetails, ExamError> {
    let current_status: String = conn
        .query_row(
            "SELECT status FROM exams WHERE id = ?1",
            params![payload.exam_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(payload.exam_id.clone()))?;

    if current_status == "published" {
        return Err(ExamError::ExamAlreadyPublished(payload.exam_id.clone()));
    }

    let now = now_utc_iso();

    if let Some(ref name) = payload.name {
        conn.execute(
            "UPDATE exams SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now, payload.exam_id],
        )?;
    }
    if let Some(ref name_bn) = payload.name_bn {
        conn.execute(
            "UPDATE exams SET name_bn = ?1, updated_at = ?2 WHERE id = ?3",
            params![name_bn, now, payload.exam_id],
        )?;
    }
    if let Some(ref exam_type) = payload.exam_type {
        conn.execute(
            "UPDATE exams SET exam_type = ?1, updated_at = ?2 WHERE id = ?3",
            params![exam_type, now, payload.exam_id],
        )?;
    }
    if let Some(ref start_date) = payload.start_date {
        conn.execute(
            "UPDATE exams SET start_date = ?1, updated_at = ?2 WHERE id = ?3",
            params![start_date, now, payload.exam_id],
        )?;
    }
    if let Some(ref end_date) = payload.end_date {
        conn.execute(
            "UPDATE exams SET end_date = ?1, updated_at = ?2 WHERE id = ?3",
            params![end_date, now, payload.exam_id],
        )?;
    }
    if let Some(ref scheme_id) = payload.grading_scheme_id {
        conn.execute(
            "UPDATE exams SET grading_scheme_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![scheme_id, now, payload.exam_id],
        )?;
    }

    if let Some(ref batch_ids) = payload.batch_ids {
        conn.execute(
            "DELETE FROM exam_batches WHERE exam_id = ?1",
            params![payload.exam_id],
        )?;
        for bid in batch_ids {
            conn.execute(
                "INSERT INTO exam_batches (exam_id, batch_id) VALUES (?1, ?2)",
                params![payload.exam_id, bid],
            )?;
        }
    }

    get_exam_details(conn, &payload.exam_id)
}

pub fn delete_exam(conn: &Connection, exam_id: &str) -> Result<(), ExamError> {
    let status: String = conn
        .query_row(
            "SELECT status FROM exams WHERE id = ?1",
            params![exam_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(exam_id.to_string()))?;

    if status == "published" {
        return Err(ExamError::ExamAlreadyPublished(exam_id.to_string()));
    }

    conn.execute("DELETE FROM exams WHERE id = ?1", params![exam_id])?;
    Ok(())
}

pub fn get_mark_sheet(
    conn: &Connection,
    exam_subject_id: &str,
) -> Result<MarkSheetData, ExamError> {
    let (exam_id, exam_name, subject_id, subject_name, subject_code, full_marks, pass_marks) = conn
        .query_row(
            "SELECT e.id, e.name, s.id, s.name, s.code, es.full_marks, es.pass_marks
             FROM exam_subjects es
             JOIN exams e ON e.id = es.exam_id
             JOIN subjects s ON s.id = es.subject_id
             WHERE es.id = ?1",
            params![exam_subject_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamSubjectNotFound(exam_subject_id.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT st.id, en.roll, st.student_code, st.name, st.name_bn,
                m.marks, coalesce(m.is_absent, 0) as is_absent,
                coalesce(m.is_optional, 0) as is_optional,
                coalesce(m.status, 'draft') as status, m.note
         FROM exam_batches eb
         JOIN enrollments en ON en.batch_id = eb.batch_id AND en.status = 'active'
         JOIN students st ON st.id = en.student_id
         LEFT JOIN marks m ON m.exam_subject_id = ?1 AND m.student_id = st.id
         WHERE eb.exam_id = ?2
         ORDER BY coalesce(en.roll, 999999) ASC, st.student_code ASC",
    )?;

    let rows = stmt
        .query_map(params![exam_subject_id, exam_id], |row| {
            Ok(MarkSheetStudentRow {
                student_id: row.get(0)?,
                roll: row.get(1)?,
                student_code: row.get(2)?,
                name: row.get(3)?,
                name_bn: row.get(4)?,
                marks: row.get(5)?,
                is_absent: row.get::<_, i64>(6)? == 1,
                is_optional: row.get::<_, i64>(7)? == 1,
                status: row.get(8)?,
                note: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total = rows.len();
    let verified_count = rows.iter().filter(|r| r.status == "verified").count();
    let is_verified = total > 0 && verified_count >= total;

    Ok(MarkSheetData {
        exam_id,
        exam_name,
        exam_subject_id: exam_subject_id.to_string(),
        subject_id,
        subject_name,
        subject_code,
        full_marks,
        pass_marks,
        is_verified,
        rows,
    })
}

pub fn save_marks(
    conn: &Connection,
    payload: &SaveMarksPayload,
    user_id: &str,
) -> Result<MarkSheetData, ExamError> {
    let (exam_id, exam_status, full_marks) = conn
        .query_row(
            "SELECT e.id, e.status, es.full_marks
             FROM exam_subjects es
             JOIN exams e ON e.id = es.exam_id
             WHERE es.id = ?1",
            params![payload.exam_subject_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamSubjectNotFound(payload.exam_subject_id.clone()))?;

    if exam_status == "published" {
        return Err(ExamError::ExamAlreadyPublished(exam_id));
    }

    // Check if the subject marks are already verified
    let is_verified: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM marks WHERE exam_subject_id = ?1 AND status = 'verified')",
            params![payload.exam_subject_id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if is_verified {
        return Err(ExamError::UnauthorizedVerifiedEdit);
    }

    // Pre-validate range
    for row in &payload.marks {
        if !row.is_absent {
            if let Some(m) = row.marks {
                if m < 0 {
                    return Err(ExamError::MarkNegative(m));
                }
                if m > full_marks {
                    return Err(ExamError::MarkOutOfRange {
                        marks: m,
                        full_marks,
                    });
                }
            }
        }
    }

    let now = now_utc_iso();

    for row in &payload.marks {
        let (marks_val, is_absent_val) = if row.is_absent {
            (None, 1)
        } else {
            (row.marks, 0)
        };

        // We only insert or update if row is not verified, or update will trigger trigger check
        conn.execute(
            "INSERT INTO marks (exam_subject_id, student_id, marks, is_absent, is_optional, status, entered_by, entered_at, note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'draft', ?6, ?7, ?8, ?7, ?7)
             ON CONFLICT(exam_subject_id, student_id) DO UPDATE SET
                marks = excluded.marks,
                is_absent = excluded.is_absent,
                is_optional = excluded.is_optional,
                note = excluded.note,
                entered_by = excluded.entered_by,
                entered_at = excluded.entered_at,
                updated_at = excluded.updated_at",
            params![
                payload.exam_subject_id,
                row.student_id,
                marks_val,
                is_absent_val,
                if row.is_optional { 1 } else { 0 },
                user_id,
                now,
                row.note
            ],
        )?;
    }

    // Update parent exam status to 'entry' if still 'draft'
    conn.execute(
        "UPDATE exams SET status = 'entry', updated_at = ?1 WHERE id = ?2 AND status = 'draft'",
        params![now, exam_id],
    )?;

    get_mark_sheet(conn, &payload.exam_subject_id)
}

pub fn verify_subject(
    conn: &Connection,
    exam_subject_id: &str,
    user_id: &str,
) -> Result<(), ExamError> {
    let (exam_id, exam_status) = conn
        .query_row(
            "SELECT e.id, e.status
             FROM exam_subjects es
             JOIN exams e ON e.id = es.exam_id
             WHERE es.id = ?1",
            params![exam_subject_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamSubjectNotFound(exam_subject_id.to_string()))?;

    if exam_status == "published" {
        return Err(ExamError::ExamAlreadyPublished(exam_id));
    }

    // Check complete roster
    let (total_enrolled, recorded_count): (i64, i64) = conn.query_row(
        "SELECT
            (SELECT count(distinct en.student_id)
             FROM exam_batches eb
             JOIN enrollments en ON en.batch_id = eb.batch_id AND en.status = 'active'
             WHERE eb.exam_id = ?1),
            (SELECT count(*)
             FROM marks m
             WHERE m.exam_subject_id = ?2
               AND (m.is_absent = 1 OR m.marks IS NOT NULL))",
        params![exam_id, exam_subject_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if recorded_count < total_enrolled {
        return Err(ExamError::MarksIncomplete {
            unrecorded_count: (total_enrolled - recorded_count) as usize,
        });
    }

    let now = now_utc_iso();

    conn.execute(
        "UPDATE marks
         SET status = 'verified', verified_by = ?1, verified_at = ?2, updated_at = ?2
         WHERE exam_subject_id = ?3",
        params![user_id, now, exam_subject_id],
    )?;

    // Check if ALL subjects of this exam are now verified
    let (all_subjects_count, verified_subjects_count): (i64, i64) = conn
        .query_row(
            "SELECT
            (SELECT count(*) FROM exam_subjects WHERE exam_id = ?1),
            (SELECT count(*) FROM (
                SELECT es.id
                FROM exam_subjects es
                JOIN marks m ON m.exam_subject_id = es.id AND m.status = 'verified'
                WHERE es.exam_id = ?1
                GROUP BY es.id
                HAVING count(*) >= ?2
            ))",
            params![exam_id, total_enrolled],
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0))),
        )
        .unwrap_or((1, 0));

    if all_subjects_count > 0 && verified_subjects_count >= all_subjects_count {
        conn.execute(
            "UPDATE exams SET status = 'verified', updated_at = ?1 WHERE id = ?2 AND status != 'published'",
            params![now, exam_id],
        )?;
    }

    Ok(())
}

pub fn unverify_subject(
    conn: &Connection,
    exam_subject_id: &str,
    user_id: &str,
) -> Result<(), ExamError> {
    let (exam_id, exam_status) = conn
        .query_row(
            "SELECT e.id, e.status
             FROM exam_subjects es
             JOIN exams e ON e.id = es.exam_id
             WHERE es.id = ?1",
            params![exam_subject_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamSubjectNotFound(exam_subject_id.to_string()))?;

    if exam_status == "published" {
        return Err(ExamError::ExamAlreadyPublished(exam_id));
    }

    let now = now_utc_iso();

    conn.execute(
        "UPDATE marks
         SET status = 'draft', verified_by = NULL, verified_at = NULL, updated_at = ?1
         WHERE exam_subject_id = ?2",
        params![now, exam_subject_id],
    )?;

    conn.execute(
        "UPDATE exams SET status = 'entry', updated_at = ?1 WHERE id = ?2 AND status = 'verified'",
        params![now, exam_id],
    )?;

    let entry = crate::audit::AuditEntry {
        action: "exam.unverify_subject".to_string(),
        entity: "exam_subject".to_string(),
        entity_id: exam_subject_id.to_string(),
        before_json: Some(serde_json::json!({ "exam_id": exam_id, "status": "verified" })),
        after_json: Some(serde_json::json!({ "exam_id": exam_id, "status": "draft" })),
    };
    let _ = crate::audit::record_audit_entry_with_user(conn, entry, Some(user_id), None, &now);

    Ok(())
}

// ---------------------------------------------------------------------------
// Result Calculation & NCTB GPA-5 Math
// ---------------------------------------------------------------------------

struct StudentCandidateMarks {
    student_id: String,
    roll: Option<i64>,
    student_code: String,
    name: String,
    name_bn: Option<String>,
    batch_id: String,
    batch_name: String,
    subject_marks: Vec<ResultSubjectDetailItem>,
}

pub fn calculate_and_preview_results(
    conn: &Connection,
    exam_id: &str,
) -> Result<ExamResultPreviewData, ExamError> {
    let (exam_name, scheme_id, scheme_name, scale_type): (String, String, String, String) = conn
        .query_row(
            "SELECT e.name, gs.id, gs.name, gs.scale_type
             FROM exams e
             JOIN grading_schemes gs ON gs.id = e.grading_scheme_id
             WHERE e.id = ?1",
            params![exam_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(exam_id.to_string()))?;

    // Fetch grading bands
    let mut band_stmt = conn.prepare(
        "SELECT id, grade, grade_point, min_percentage, max_percentage, is_fail
         FROM grade_bands
         WHERE scheme_id = ?1
         ORDER BY min_percentage DESC",
    )?;

    let bands = band_stmt
        .query_map(params![scheme_id], |row| {
            Ok(GradeBandItem {
                id: row.get(0)?,
                grade: row.get(1)?,
                grade_point: row.get(2)?,
                min_percentage: row.get(3)?,
                max_percentage: row.get(4)?,
                is_fail: row.get::<_, i64>(5)? == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Check for unverified subjects
    let mut unverified_stmt = conn.prepare(
        "SELECT s.name
         FROM exam_subjects es
         JOIN subjects s ON s.id = es.subject_id
         WHERE es.exam_id = ?1
           AND (SELECT count(*) FROM marks m WHERE m.exam_subject_id = es.id AND m.status = 'verified') = 0",
    )?;

    let unverified_subjects = unverified_stmt
        .query_map(params![exam_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    // Retrieve all students and their subject marks
    let mut stmt = conn.prepare(
        "SELECT st.id, en.roll, st.student_code, st.name, st.name_bn,
                b.id, b.name,
                es.id, s.name, s.code,
                m.marks, coalesce(m.is_absent, 0), coalesce(m.is_optional, 0),
                es.full_marks, es.pass_marks
         FROM exam_batches eb
         JOIN batches b ON b.id = eb.batch_id
         JOIN enrollments en ON en.batch_id = b.id AND en.status = 'active'
         JOIN students st ON st.id = en.student_id
         JOIN exam_subjects es ON es.exam_id = eb.exam_id
         JOIN subjects s ON s.id = es.subject_id
         LEFT JOIN marks m ON m.exam_subject_id = es.id AND m.student_id = st.id
         WHERE eb.exam_id = ?1
         ORDER BY coalesce(en.roll, 999999) ASC, st.student_code ASC, s.name ASC",
    )?;

    let mut students_map: HashMap<String, StudentCandidateMarks> = HashMap::new();
    let mut rows_iter = stmt.query(params![exam_id])?;

    while let Some(row) = rows_iter.next()? {
        let student_id: String = row.get(0)?;
        let roll: Option<i64> = row.get(1)?;
        let student_code: String = row.get(2)?;
        let name: String = row.get(3)?;
        let name_bn: Option<String> = row.get(4)?;
        let batch_id: String = row.get(5)?;
        let batch_name: String = row.get(6)?;

        let exam_sub_id: String = row.get(7)?;
        let sub_name: String = row.get(8)?;
        let sub_code: String = row.get(9)?;
        let marks: Option<i64> = row.get(10)?;
        let is_absent = row.get::<_, i64>(11)? == 1;
        let is_optional = row.get::<_, i64>(12)? == 1;
        let full_marks: i64 = row.get(13)?;
        let pass_marks: i64 = row.get(14)?;

        // Calculate subject grade and points
        let (grade, grade_point, passed) = if is_absent {
            ("F".to_string(), 0, false)
        } else if let Some(m) = marks {
            let pct = (m * 10000) / full_marks;
            let matching_band = bands
                .iter()
                .find(|b| pct >= b.min_percentage && pct <= b.max_percentage)
                .or_else(|| bands.last());

            let band = matching_band.cloned().unwrap_or(GradeBandItem {
                id: "".into(),
                grade: "F".into(),
                grade_point: 0,
                min_percentage: 0,
                max_percentage: 0,
                is_fail: true,
            });

            let passed = m >= pass_marks && !band.is_fail;
            (band.grade, band.grade_point, passed)
        } else {
            ("F".to_string(), 0, false)
        };

        let sub_detail = ResultSubjectDetailItem {
            exam_subject_id: exam_sub_id,
            subject_name: sub_name,
            subject_code: sub_code,
            marks,
            full_marks,
            pass_marks,
            is_absent,
            is_optional,
            grade,
            grade_point,
            passed,
        };

        students_map
            .entry(student_id.clone())
            .or_insert_with(|| StudentCandidateMarks {
                student_id,
                roll,
                student_code,
                name,
                name_bn,
                batch_id,
                batch_name,
                subject_marks: Vec::new(),
            })
            .subject_marks
            .push(sub_detail);
    }

    // Now evaluate NCTB GPA-5 per student
    struct EvaluatedStudent {
        student_id: String,
        roll: Option<i64>,
        student_code: String,
        name: String,
        name_bn: Option<String>,
        batch_id: String,
        batch_name: String,
        total_marks: i64,
        total_max_marks: i64,
        mandatory_total_marks: i64,
        percentage: i64,
        gpa: i64,
        grade: String,
        passed: bool,
        subjects: Vec<ResultSubjectDetailItem>,
    }

    let mut evaluated: Vec<EvaluatedStudent> = Vec::new();

    for (_s_id, candidate) in students_map {
        let mut total_marks = 0;
        let mut total_max_marks = 0;
        let mut mandatory_total_marks = 0;
        let mut mandatory_gp_sum = 0;
        let mut mandatory_count = 0;
        let mut fourth_subject_bonus = 0;
        let mut any_mandatory_failed = false;

        for sub in &candidate.subject_marks {
            let m = sub.marks.unwrap_or(0);
            total_marks += m;
            total_max_marks += sub.full_marks;

            if sub.is_optional {
                // 4th subject bonus: max(0, gp - 2.00)
                let bonus = std::cmp::max(0, sub.grade_point - 200);
                fourth_subject_bonus += bonus;
                // Note: failing 4th subject does NOT fail the student overall
            } else {
                mandatory_count += 1;
                mandatory_total_marks += m;
                mandatory_gp_sum += sub.grade_point;

                if !sub.passed {
                    any_mandatory_failed = true;
                }
            }
        }

        let overall_passed = !any_mandatory_failed && mandatory_count > 0;
        let percentage = if total_max_marks > 0 {
            (total_marks * 10000) / total_max_marks
        } else {
            0
        };

        let (gpa, grade) = if !overall_passed {
            let fail_grade = bands
                .iter()
                .find(|b| b.is_fail)
                .map(|b| b.grade.clone())
                .unwrap_or_else(|| "F".to_string());
            (0, fail_grade)
        } else {
            let divisor = if mandatory_count > 0 {
                mandatory_count
            } else {
                1
            };
            let calculated_gpa = (mandatory_gp_sum + fourth_subject_bonus) / divisor;
            let max_gp = bands
                .iter()
                .filter(|b| !b.is_fail)
                .map(|b| b.grade_point)
                .max()
                .unwrap_or(500);
            let capped_gpa = std::cmp::min(max_gp, calculated_gpa);

            // Find matching grade for overall performance
            let gpa_grade = if scale_type == "gpa_4" {
                // In British GPA 4.0, A* (>=90%) and A (>=80%) both share GP 4.00; percentage breaks the top grade
                bands
                    .iter()
                    .filter(|b| !b.is_fail)
                    .find(|b| capped_gpa >= b.grade_point && percentage >= b.min_percentage)
                    .or_else(|| {
                        bands
                            .iter()
                            .filter(|b| !b.is_fail)
                            .find(|b| capped_gpa >= b.grade_point)
                    })
                    .map(|b| b.grade.clone())
                    .unwrap_or_else(|| "A".to_string())
            } else if scale_type == "percentage" {
                bands
                    .iter()
                    .filter(|b| !b.is_fail)
                    .find(|b| percentage >= b.min_percentage && percentage <= b.max_percentage)
                    .map(|b| b.grade.clone())
                    .unwrap_or_else(|| "Pass".to_string())
            } else {
                // Standard GPA-5 or custom grade point scales: match highest band where GPA meets threshold
                bands
                    .iter()
                    .filter(|b| !b.is_fail)
                    .find(|b| capped_gpa >= b.grade_point)
                    .map(|b| b.grade.clone())
                    .unwrap_or_else(|| {
                        bands
                            .iter()
                            .rfind(|b| !b.is_fail)
                            .map(|b| b.grade.clone())
                            .unwrap_or_else(|| "D".to_string())
                    })
            };

            (capped_gpa, gpa_grade)
        };

        evaluated.push(EvaluatedStudent {
            student_id: candidate.student_id,
            roll: candidate.roll,
            student_code: candidate.student_code,
            name: candidate.name,
            name_bn: candidate.name_bn,
            batch_id: candidate.batch_id,
            batch_name: candidate.batch_name,
            total_marks,
            total_max_marks,
            mandatory_total_marks,
            percentage,
            gpa,
            grade,
            passed: overall_passed,
            subjects: candidate.subject_marks,
        });
    }

    // 5-Tier Deterministic Sorting
    evaluated.sort_by(|a, b| {
        b.gpa
            .cmp(&a.gpa)
            .then_with(|| b.total_marks.cmp(&a.total_marks))
            .then_with(|| b.mandatory_total_marks.cmp(&a.mandatory_total_marks))
            .then_with(|| a.roll.unwrap_or(i64::MAX).cmp(&b.roll.unwrap_or(i64::MAX)))
            .then_with(|| a.student_id.cmp(&b.student_id))
    });

    let total_students = evaluated.len();
    let passed_students = evaluated.iter().filter(|e| e.passed).count();
    let failed_students = total_students - passed_students;
    let average_gpa = if total_students > 0 {
        evaluated.iter().map(|e| e.gpa).sum::<i64>() / total_students as i64
    } else {
        0
    };

    let student_rows: Vec<ResultPreviewStudentRow> = evaluated
        .into_iter()
        .enumerate()
        .map(|(idx, e)| ResultPreviewStudentRow {
            student_id: e.student_id,
            roll: e.roll,
            student_code: e.student_code,
            name: e.name,
            name_bn: e.name_bn,
            batch_id: e.batch_id,
            batch_name: e.batch_name,
            total_marks: e.total_marks,
            total_max_marks: e.total_max_marks,
            percentage: e.percentage,
            gpa: e.gpa,
            grade: e.grade,
            passed: e.passed,
            rank: (idx + 1) as i64,
            subjects: e.subjects,
        })
        .collect();

    Ok(ExamResultPreviewData {
        exam_id: exam_id.to_string(),
        exam_name,
        grading_scheme_name: scheme_name,
        total_students,
        passed_students,
        failed_students,
        average_gpa,
        unverified_subjects,
        students: student_rows,
    })
}

pub fn publish_results(
    conn: &Connection,
    payload: &PublishExamPayload,
    user_id: &str,
) -> Result<PublishExamResponse, ExamError> {
    let preview = calculate_and_preview_results(conn, &payload.exam_id)?;

    let (current_status, scheme_id, scale_type): (String, String, String) = conn
        .query_row(
            "SELECT e.status, e.grading_scheme_id, gs.scale_type
             FROM exams e
             JOIN grading_schemes gs ON gs.id = e.grading_scheme_id
             WHERE e.id = ?1",
            params![payload.exam_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(payload.exam_id.clone()))?;

    if current_status == "published" {
        return Err(ExamError::ExamAlreadyPublished(payload.exam_id.clone()));
    }

    if !preview.unverified_subjects.is_empty() || current_status != "verified" {
        return Err(ExamError::ExamNotVerified(payload.exam_id.clone()));
    }

    // Fetch frozen grading scheme JSON
    let scheme_summary = list_grading_schemes(conn)?
        .into_iter()
        .find(|s| s.id == scheme_id)
        .ok_or_else(|| ExamError::GradingSchemeNotFound(scheme_id.clone()))?;
    let grading_scheme_json =
        serde_json::to_string(&scheme_summary).unwrap_or_else(|_| "{}".to_string());

    let calculation_rules_json = serde_json::json!({
        "scale": scale_type,
        "fourth_subject_bonus": if scale_type == "gpa_5" { "max(0, gp - 200)" } else { "none" },
        "mandatory_fail_caps_gpa": true,
        "tie_breaking": ["gpa_desc", "total_marks_desc", "mandatory_marks_desc", "roll_asc", "student_id_asc"]
    }).to_string();

    let next_version: i64 = conn
        .query_row(
            "SELECT coalesce(max(version), 0) + 1 FROM result_snapshots WHERE exam_id = ?1",
            params![payload.exam_id],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let snapshot_id = Uuid::now_v7().to_string();
    let now = now_utc_iso();

    // Mark previous snapshots as is_current = 0
    conn.execute(
        "UPDATE result_snapshots SET is_current = 0 WHERE exam_id = ?1",
        params![payload.exam_id],
    )?;

    // Insert snapshot
    conn.execute(
        "INSERT INTO result_snapshots (id, exam_id, version, published_at, published_by, grading_scheme_json, calculation_rules_json, is_current, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?4)",
        params![
            snapshot_id,
            payload.exam_id,
            next_version,
            now,
            user_id,
            grading_scheme_json,
            calculation_rules_json
        ],
    )?;

    // Insert results and subject details
    let mut notifications_count = 0;

    for s in &preview.students {
        let result_id = Uuid::now_v7().to_string();
        let tie_breaker_json = serde_json::json!({
            "total_marks": s.total_marks,
            "gpa": s.gpa,
            "rank": s.rank
        })
        .to_string();

        conn.execute(
            "INSERT INTO results (id, snapshot_id, student_id, batch_id, total_marks, total_max_marks, percentage, gpa, grade, passed, rank, tie_breaker_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                result_id,
                snapshot_id,
                s.student_id,
                s.batch_id,
                s.total_marks,
                s.total_max_marks,
                s.percentage,
                s.gpa,
                s.grade,
                if s.passed { 1 } else { 0 },
                s.rank,
                tie_breaker_json,
                now
            ],
        )?;

        for sub in &s.subjects {
            let detail_id = Uuid::now_v7().to_string();
            conn.execute(
                "INSERT INTO result_subject_details (id, result_id, exam_subject_id, marks, full_marks, pass_marks, is_absent, is_optional, grade, grade_point, passed, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    detail_id,
                    result_id,
                    sub.exam_subject_id,
                    sub.marks,
                    sub.full_marks,
                    sub.pass_marks,
                    if sub.is_absent { 1 } else { 0 },
                    if sub.is_optional { 1 } else { 0 },
                    sub.grade,
                    sub.grade_point,
                    if sub.passed { 1 } else { 0 },
                    now
                ],
            )?;
        }

        // Omnichannel notification hook
        if payload.send_notifications {
            let guardian_phone: Option<String> = conn
                .query_row(
                    "SELECT g.phone_norm
                     FROM student_guardians sg
                     JOIN guardians g ON g.id = sg.guardian_id
                     WHERE sg.student_id = ?1 AND sg.receives_sms = 1
                     LIMIT 1",
                    params![s.student_id],
                    |row| row.get(0),
                )
                .optional()
                .unwrap_or(None);

            if let Some(phone) = guardian_phone {
                let body = format!(
                    "Exam Result: {}, Student: {}, GPA: {:.2}, Grade: {}, Merit: #{}.",
                    preview.exam_name,
                    s.name,
                    s.gpa as f64 / 100.0,
                    s.grade,
                    s.rank
                );
                let dedupe_key = format!("result:{}:{}:sms", snapshot_id, s.student_id);

                let _ = conn.execute(
                    "INSERT OR IGNORE INTO outbox (id, channel, to_address, body, segments, encoding, status, scheduled_at, attempts, dedupe_key, related_type, related_id, created_by, created_at)
                     VALUES (?1, 'sms', ?2, ?3, 1, 'gsm', 'queued', ?4, 0, ?5, 'exam_result', ?6, ?7, ?4)",
                    params![
                        Uuid::now_v7().to_string(),
                        phone,
                        body,
                        now,
                        dedupe_key,
                        result_id,
                        user_id
                    ],
                );
                notifications_count += 1;
            }
        }
    }

    // Set exam status to published
    conn.execute(
        "UPDATE exams SET status = 'published', updated_at = ?1 WHERE id = ?2",
        params![now, payload.exam_id],
    )?;

    Ok(PublishExamResponse {
        snapshot_id,
        version: next_version,
        published_at: now,
        student_count: preview.students.len(),
        notifications_enqueued: notifications_count,
    })
}

pub fn revert_published_to_verified(
    conn: &Connection,
    exam_id: &str,
    user_id: &str,
) -> Result<(), ExamError> {
    let current_status: String = conn
        .query_row(
            "SELECT status FROM exams WHERE id = ?1",
            params![exam_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ExamError::ExamNotFound(exam_id.to_string()))?;

    if current_status != "published" {
        return Ok(());
    }

    let now = now_utc_iso();
    conn.execute(
        "UPDATE exams SET status = 'verified', updated_at = ?1 WHERE id = ?2",
        params![now, exam_id],
    )?;

    let entry = crate::audit::AuditEntry {
        action: "exam.revert_published".to_string(),
        entity: "exam".to_string(),
        entity_id: exam_id.to_string(),
        before_json: Some(serde_json::json!({ "status": "published" })),
        after_json: Some(serde_json::json!({ "status": "verified" })),
    };
    let _ = crate::audit::record_audit_entry_with_user(conn, entry, Some(user_id), None, &now);

    Ok(())
}

// ---------------------------------------------------------------------------
// Print & Report Delivery Helpers
// ---------------------------------------------------------------------------

pub fn get_student_report_card(
    conn: &Connection,
    exam_id: &str,
    student_id: &str,
) -> Result<StudentReportCardData, ExamError> {
    let (inst_name, inst_name_bn, inst_address, inst_phones): (
        String,
        Option<String>,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT name, name_bn, address, phones FROM institution WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap_or_else(|_| {
            (
                "Coachify Academy".to_string(),
                None,
                "Dhaka, Bangladesh".to_string(),
                "01700000000".to_string(),
            )
        });

    let (exam_name, session_name): (String, String) = conn.query_row(
        "SELECT e.name, s.name FROM exams e JOIN academic_sessions s ON s.id = e.session_id WHERE e.id = ?1",
        params![exam_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let snapshot_id: String = conn
        .query_row(
            "SELECT id FROM result_snapshots WHERE exam_id = ?1 AND is_current = 1",
            params![exam_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ExamError::SnapshotNotFound(exam_id.to_string()))?;

    let total_students: i64 = conn.query_row(
        "SELECT count(*) FROM results WHERE snapshot_id = ?1",
        params![snapshot_id],
        |row| row.get(0),
    )?;

    let (result_id, total_marks, total_max_marks, percentage, gpa, grade, passed, rank, batch_id): (
        String,
        i64,
        i64,
        i64,
        i64,
        String,
        bool,
        i64,
        String,
    ) = conn.query_row(
        "SELECT id, total_marks, total_max_marks, percentage, gpa, grade, (passed = 1), rank, batch_id
         FROM results
         WHERE snapshot_id = ?1 AND student_id = ?2",
        params![snapshot_id, student_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
            ))
        },
    )?;

    let (st_code, st_name, st_name_bn, roll, batch_name, prog_name): (
        String,
        String,
        Option<String>,
        Option<i64>,
        String,
        String,
    ) = conn.query_row(
        "SELECT st.student_code, st.name, st.name_bn, en.roll, b.name, p.name
         FROM students st
         JOIN batches b ON b.id = ?1
         JOIN programs p ON p.id = b.program_id
         LEFT JOIN enrollments en ON en.student_id = st.id AND en.batch_id = b.id AND en.status = 'active'
         WHERE st.id = ?2",
        params![batch_id, student_id],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT rsd.exam_subject_id, s.name, s.code, rsd.marks, rsd.full_marks, rsd.pass_marks,
                (rsd.is_absent = 1), (rsd.is_optional = 1), rsd.grade, rsd.grade_point, (rsd.passed = 1)
         FROM result_subject_details rsd
         JOIN exam_subjects es ON es.id = rsd.exam_subject_id
         JOIN subjects s ON s.id = es.subject_id
         WHERE rsd.result_id = ?1
         ORDER BY s.name ASC",
    )?;

    let subjects = stmt
        .query_map(params![result_id], |row| {
            Ok(ResultSubjectDetailItem {
                exam_subject_id: row.get(0)?,
                subject_name: row.get(1)?,
                subject_code: row.get(2)?,
                marks: row.get(3)?,
                full_marks: row.get(4)?,
                pass_marks: row.get(5)?,
                is_absent: row.get(6)?,
                is_optional: row.get(7)?,
                grade: row.get(8)?,
                grade_point: row.get(9)?,
                passed: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Optional attendance percentage
    let attendance_pct: Option<f64> = conn
        .query_row(
            "SELECT
                cast(count(case when am.status IN ('present', 'late') then 1 end) as real) /
                nullif(count(*), 0) * 100.0
             FROM attendance_marks am
             WHERE am.student_id = ?1",
            params![student_id],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    Ok(StudentReportCardData {
        institution_name: inst_name,
        institution_name_bn: inst_name_bn,
        institution_address: inst_address,
        institution_phones: inst_phones,
        student_id: student_id.to_string(),
        student_code: st_code,
        student_name: st_name,
        student_name_bn: st_name_bn,
        roll,
        batch_name,
        program_name: prog_name,
        exam_name,
        session_name,
        total_marks,
        total_max_marks,
        percentage,
        gpa,
        grade,
        passed,
        rank,
        total_students,
        subjects,
        attendance_percentage: attendance_pct,
    })
}

pub fn get_batch_tabulation_sheet(
    conn: &Connection,
    exam_id: &str,
    batch_id: &str,
) -> Result<BatchTabulationSheetData, ExamError> {
    let (exam_name, session_name, batch_name): (String, String, String) = conn.query_row(
        "SELECT e.name, s.name, b.name
         FROM exams e
         JOIN academic_sessions s ON s.id = e.session_id
         JOIN batches b ON b.id = ?2
         WHERE e.id = ?1",
        params![exam_id, batch_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    // Subject Headers
    let mut sub_stmt = conn.prepare(
        "SELECT es.id, s.name, s.code, es.full_marks, es.pass_marks
         FROM exam_subjects es
         JOIN subjects s ON s.id = es.subject_id
         WHERE es.exam_id = ?1
         ORDER BY s.name ASC",
    )?;

    let subjects = sub_stmt
        .query_map(params![exam_id], |row| {
            Ok(TabulationSubjectHeader {
                exam_subject_id: row.get(0)?,
                subject_name: row.get(1)?,
                subject_code: row.get(2)?,
                full_marks: row.get(3)?,
                pass_marks: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let snapshot_id: Option<String> = conn
        .query_row(
            "SELECT id FROM result_snapshots WHERE exam_id = ?1 AND is_current = 1",
            params![exam_id],
            |row| row.get(0),
        )
        .optional()?;

    let rows = if let Some(snap_id) = snapshot_id {
        let mut row_stmt = conn.prepare(
            "SELECT r.id, st.id, en.roll, st.student_code, st.name,
                    r.total_marks, r.gpa, r.grade, (r.passed = 1), r.rank
             FROM results r
             JOIN students st ON st.id = r.student_id
             JOIN batches b ON b.id = r.batch_id
             LEFT JOIN enrollments en ON en.student_id = st.id AND en.batch_id = b.id AND en.status = 'active'
             WHERE r.snapshot_id = ?1 AND r.batch_id = ?2
             ORDER BY r.rank ASC",
        )?;

        let student_results = row_stmt
            .query_map(params![snap_id, batch_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, bool>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut tabulation_rows = Vec::new();

        for (res_id, s_id, roll, s_code, s_name, tot_m, gpa, grade, passed, rank) in student_results
        {
            let mut cell_stmt = conn.prepare(
                "SELECT rsd.exam_subject_id, rsd.marks, rsd.grade, rsd.grade_point,
                        (rsd.is_absent = 1), (rsd.is_optional = 1), (rsd.passed = 1)
                 FROM result_subject_details rsd
                 WHERE rsd.result_id = ?1",
            )?;

            let cells = cell_stmt
                .query_map(params![res_id], |c_row| {
                    Ok((
                        c_row.get::<_, String>(0)?,
                        TabulationMarkCell {
                            marks: c_row.get(1)?,
                            grade: c_row.get(2)?,
                            grade_point: c_row.get(3)?,
                            is_absent: c_row.get(4)?,
                            is_optional: c_row.get(5)?,
                            passed: c_row.get(6)?,
                        },
                    ))
                })?
                .collect::<Result<HashMap<_, _>, _>>()?;

            tabulation_rows.push(TabulationStudentRow {
                student_id: s_id,
                roll,
                student_code: s_code,
                student_name: s_name,
                marks_by_subject: cells,
                total_marks: tot_m,
                gpa,
                grade,
                passed,
                rank,
            });
        }

        tabulation_rows
    } else {
        // Exam not published yet: build from draft/preview calculation
        let preview = calculate_and_preview_results(conn, exam_id)?;
        let mut tabulation_rows = Vec::new();

        for s in preview
            .students
            .into_iter()
            .filter(|st| st.batch_id == batch_id)
        {
            let mut marks_map = HashMap::new();
            for sub in s.subjects {
                marks_map.insert(
                    sub.exam_subject_id,
                    TabulationMarkCell {
                        marks: sub.marks,
                        grade: sub.grade,
                        grade_point: sub.grade_point,
                        is_absent: sub.is_absent,
                        is_optional: sub.is_optional,
                        passed: sub.passed,
                    },
                );
            }

            tabulation_rows.push(TabulationStudentRow {
                student_id: s.student_id,
                roll: s.roll,
                student_code: s.student_code,
                student_name: s.name,
                marks_by_subject: marks_map,
                total_marks: s.total_marks,
                gpa: s.gpa,
                grade: s.grade,
                passed: s.passed,
                rank: s.rank,
            });
        }

        tabulation_rows
    };

    Ok(BatchTabulationSheetData {
        exam_id: exam_id.to_string(),
        exam_name,
        batch_id: batch_id.to_string(),
        batch_name,
        session_name,
        subjects,
        rows,
    })
}
