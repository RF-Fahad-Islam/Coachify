use crate::phone::{self, PhoneError};
use crate::schedule::{self, ScheduleConflictItem, ScheduleError, ScheduleSlot};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AcademicError {
    #[error("Program with ID '{0}' not found")]
    ProgramNotFound(String),
    #[error("Program code '{0}' already exists in this session")]
    ProgramCodeExists(String),
    #[error("Academic session with ID '{0}' not found")]
    SessionNotFound(String),
    #[error("Batch with ID '{0}' not found")]
    BatchNotFound(String),
    #[error("Batch code '{0}' already exists in this program")]
    BatchCodeExists(String),
    #[error("Subject with ID '{0}' not found")]
    SubjectNotFound(String),
    #[error("Subject code '{0}' already exists")]
    SubjectCodeExists(String),
    #[error("Staff member with ID '{0}' not found")]
    StaffNotFound(String),
    #[error("Student with ID '{0}' not found")]
    StudentNotFound(String),
    #[error("Student '{0}' is not active")]
    StudentNotActive(String),
    #[error("Student is already actively enrolled in this batch")]
    AlreadyEnrolled,
    #[error("Student is not actively enrolled in source batch '{0}'")]
    NotEnrolledInBatch(String),
    #[error("Roll number {roll} is already assigned to an active student in this batch")]
    RollTaken { roll: i64 },
    #[error("Batch has reached its maximum capacity of {capacity} students ({enrolled} enrolled)")]
    BatchCapacityReached { capacity: i64, enrolled: i64 },
    #[error("Schedule conflict detected: {0:?}")]
    ScheduleConflict(Vec<ScheduleConflictItem>),
    #[error("Invalid schedule configuration: {0}")]
    InvalidSchedule(String),
    #[error("Invalid phone number: {0}")]
    InvalidPhone(String),
    #[error("Cannot delete record: {0}")]
    CannotDelete(String),
    #[error("Invalid status transition from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("Database error: {0}")]
    Sql(String),
}

impl From<rusqlite::Error> for AcademicError {
    fn from(err: rusqlite::Error) -> Self {
        AcademicError::Sql(err.to_string())
    }
}

impl From<PhoneError> for AcademicError {
    fn from(err: PhoneError) -> Self {
        AcademicError::InvalidPhone(err.to_string())
    }
}

impl From<ScheduleError> for AcademicError {
    fn from(err: ScheduleError) -> Self {
        AcademicError::InvalidSchedule(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// DTOs & Domain Models
// -----------------------------------------------------------------------------

// --- Programs ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateProgramInput {
    pub session_id: Option<String>,
    pub name: String,
    pub name_bn: Option<String>,
    pub code: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>, // "Science", "Commerce", "Humanities", "General"
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateProgramInput {
    pub name: Option<String>,
    pub name_bn: Option<String>,
    pub code: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>, // "active", "inactive", "archived"
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProgramSummary {
    pub id: String,
    pub session_id: String,
    pub session_name: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub code: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub status: String,
    pub batch_count: i64,
    pub active_student_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProgramDetail {
    pub id: String,
    pub session_id: String,
    pub session_name: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub code: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub batch_count: i64,
    pub active_student_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

// --- Subjects ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateSubjectInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SubjectItem {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub code: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchSubjectInput {
    pub subject_id: String,
    pub teacher_staff_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchSubjectItem {
    pub subject_id: String,
    pub subject_name: String,
    pub subject_code: String,
    pub teacher_staff_id: Option<String>,
    pub teacher_name: Option<String>,
}

// --- Batches ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateBatchInput {
    pub program_id: String,
    pub name: String,
    pub code: Option<String>,
    pub teacher_staff_id: Option<String>,
    pub room_number: Option<String>,
    pub shift: Option<String>, // "morning", "day", "evening"
    #[serde(default)]
    pub schedule: Vec<ScheduleSlot>,
    pub capacity: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateBatchInput {
    pub name: Option<String>,
    pub code: Option<String>,
    pub teacher_staff_id: Option<String>,
    pub room_number: Option<String>,
    pub shift: Option<String>,
    pub schedule: Option<Vec<ScheduleSlot>>,
    pub capacity: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>, // "active", "completed", "cancelled"
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchSummary {
    pub id: String,
    pub program_id: String,
    pub program_name: String,
    pub name: String,
    pub code: String,
    pub teacher_staff_id: Option<String>,
    pub teacher_name: Option<String>,
    pub room_number: Option<String>,
    pub shift: String,
    pub schedule: Vec<ScheduleSlot>,
    pub capacity: i64,
    pub enrolled_count: i64,
    pub available_seats: i64,
    pub status: String,
    pub start_date: String,
    pub end_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchDetail {
    pub id: String,
    pub program_id: String,
    pub program_name: String,
    pub name: String,
    pub code: String,
    pub teacher_staff_id: Option<String>,
    pub teacher_name: Option<String>,
    pub room_number: Option<String>,
    pub shift: String,
    pub schedule: Vec<ScheduleSlot>,
    pub capacity: i64,
    pub enrolled_count: i64,
    pub available_seats: i64,
    pub status: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub subjects: Vec<BatchSubjectItem>,
    pub created_at: String,
    pub updated_at: String,
}

// --- Staff ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateStaffInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub email: Option<String>,
    pub designation: Option<String>,
    pub department: Option<String>,
    pub educational_qualifications: Option<String>,
    pub nid: Option<String>,
    pub blood_group: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<String>,
    pub present_address: Option<String>,
    pub joining_date: Option<String>,
    pub resignation_date: Option<String>,
    pub salary_type: Option<String>, // "monthly", "per_class"
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateStaffInput {
    pub name: Option<String>,
    pub name_bn: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub designation: Option<String>,
    pub department: Option<String>,
    pub educational_qualifications: Option<String>,
    pub nid: Option<String>,
    pub blood_group: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<String>,
    pub present_address: Option<String>,
    pub joining_date: Option<String>,
    pub resignation_date: Option<String>,
    pub salary_type: Option<String>,
    pub status: Option<String>, // "active", "on_leave", "resigned", "terminated"
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffSummary {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone_norm: String,
    pub email: Option<String>,
    pub designation: String,
    pub department: Option<String>,
    pub status: String,
    pub salary_type: String,
    pub joining_date: String,
    pub active_batch_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffDetail {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone_norm: String,
    pub email: Option<String>,
    pub designation: String,
    pub department: Option<String>,
    pub educational_qualifications: Option<String>,
    pub nid: Option<String>,
    pub blood_group: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<String>,
    pub present_address: Option<String>,
    pub joining_date: String,
    pub resignation_date: Option<String>,
    pub salary_type: String,
    pub status: String,
    pub notes: Option<String>,
    pub linked_user_id: Option<String>,
    pub assigned_batches: Vec<BatchSummary>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffSearchHit {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone_norm: String,
    pub designation: String,
    pub department: Option<String>,
    pub status: String,
}

// --- Enrollments ---

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateEnrollmentInput {
    pub student_id: String,
    pub batch_id: String,
    pub roll: Option<i64>,
    pub start_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransferEnrollmentInput {
    pub student_id: String,
    pub from_batch_id: String,
    pub to_batch_id: String,
    pub new_roll: Option<i64>,
    pub transfer_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EndEnrollmentInput {
    pub enrollment_id: String,
    pub end_reason: String, // "completed", "transferred", "dropped", "left"
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BulkEnrollInput {
    pub batch_id: String,
    pub student_ids: Vec<String>,
    pub start_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnrollmentDetail {
    pub id: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub batch_id: String,
    pub batch_name: String,
    pub program_name: String,
    pub roll: Option<i64>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchRosterItem {
    pub enrollment_id: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub phone_norm: String,
    pub guardian_phone: Option<String>,
    pub guardian_name: Option<String>,
    pub roll: Option<i64>,
    pub start_date: String,
    pub status: String,
    pub admission_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentEnrollmentHistoryItem {
    pub enrollment_id: String,
    pub batch_id: String,
    pub batch_name: String,
    pub batch_code: String,
    pub program_name: String,
    pub shift: String,
    pub room_number: Option<String>,
    pub roll: Option<i64>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub status: String,
    pub end_reason: Option<String>,
    pub notes: Option<String>,
}

// -----------------------------------------------------------------------------
// Program Operations
// -----------------------------------------------------------------------------

pub fn create_program(
    conn: &Connection,
    input: &CreateProgramInput,
    current_utc: &str,
) -> Result<ProgramDetail, AcademicError> {
    let session_id = match input
        .session_id
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(s) => {
            let session_exists: bool = conn
                .query_row("SELECT 1 FROM academic_sessions WHERE id = ?", [s], |_| {
                    Ok(true)
                })
                .optional()?
                .unwrap_or(false);

            if !session_exists {
                return Err(AcademicError::SessionNotFound(s.to_string()));
            }
            s.to_string()
        }
        None => {
            let current_sess: Option<String> = conn
                .query_row(
                    "SELECT id FROM academic_sessions WHERE is_current = 1 ORDER BY start_date DESC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?;
            match current_sess {
                Some(s) => s,
                None => {
                    let any_sess: Option<String> = conn
                        .query_row(
                            "SELECT id FROM academic_sessions ORDER BY start_date DESC LIMIT 1",
                            [],
                            |r| r.get(0),
                        )
                        .optional()?;
                    match any_sess {
                        Some(s) => s,
                        None => {
                            let sess_id = Uuid::now_v7().to_string();
                            let year = if current_utc.len() >= 4 {
                                &current_utc[0..4]
                            } else {
                                "2026"
                            };
                            let start_date = format!("{}-01-01", year);
                            let end_date = format!("{}-12-31", year);
                            let name = format!("Academic Session {}", year);
                            conn.execute(
                                "INSERT INTO academic_sessions (id, name, start_date, end_date, is_current, created_at, updated_at) VALUES (?, ?, ?, ?, 1, ?, ?)",
                                params![sess_id, name, start_date, end_date, current_utc, current_utc],
                            )?;
                            sess_id
                        }
                    }
                }
            }
        }
    };

    if let Some(ref code) = input.code {
        let clean = code.trim();
        if !clean.is_empty() {
            let code_exists: bool = conn
                .query_row(
                    "SELECT 1 FROM programs WHERE session_id = ? AND code = ?",
                    params![&session_id, clean],
                    |_| Ok(true),
                )
                .optional()?
                .unwrap_or(false);

            if code_exists {
                return Err(AcademicError::ProgramCodeExists(clean.to_string()));
            }
        }
    }

    let program_id = Uuid::now_v7().to_string();
    let status = "active";

    conn.execute(
        r#"
        INSERT INTO programs (
            id, session_id, name, name_bn, code, class_level, group_major,
            description, status, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            program_id,
            session_id,
            input.name.trim(),
            input.name_bn.as_deref().map(|s| s.trim()),
            input.code.as_deref().map(|s| s.trim()),
            input.class_level.as_deref().map(|s| s.trim()),
            input.group_major.as_deref().map(|s| s.trim()),
            input.description.as_deref().map(|s| s.trim()),
            status,
            current_utc,
            current_utc,
        ],
    )?;

    get_program(conn, &program_id)
}

pub fn update_program(
    conn: &Connection,
    id: &str,
    input: &UpdateProgramInput,
    current_utc: &str,
) -> Result<ProgramDetail, AcademicError> {
    let existing = get_program(conn, id)?;

    if let Some(ref code) = input.code {
        let clean = code.trim();
        if !clean.is_empty() {
            let code_exists: bool = conn
                .query_row(
                    "SELECT 1 FROM programs WHERE session_id = ? AND code = ? AND id != ?",
                    params![existing.session_id, clean, id],
                    |_| Ok(true),
                )
                .optional()?
                .unwrap_or(false);

            if code_exists {
                return Err(AcademicError::ProgramCodeExists(clean.to_string()));
            }
        }
    }

    let name = input
        .name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.name);
    let name_bn = input
        .name_bn
        .as_deref()
        .map(|s| s.trim())
        .or(existing.name_bn.as_deref());
    let code = input
        .code
        .as_deref()
        .map(|s| s.trim())
        .or(existing.code.as_deref());
    let class_level = input
        .class_level
        .as_deref()
        .map(|s| s.trim())
        .or(existing.class_level.as_deref());
    let group_major = input
        .group_major
        .as_deref()
        .map(|s| s.trim())
        .or(existing.group_major.as_deref());
    let description = input
        .description
        .as_deref()
        .map(|s| s.trim())
        .or(existing.description.as_deref());
    let status = input
        .status
        .as_deref()
        .map(|s| s.trim())
        .unwrap_or(&existing.status);

    conn.execute(
        r#"
        UPDATE programs
        SET name = ?,
            name_bn = ?,
            code = ?,
            class_level = ?,
            group_major = ?,
            description = ?,
            status = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        params![
            name,
            name_bn,
            code,
            class_level,
            group_major,
            description,
            status,
            current_utc,
            id,
        ],
    )?;

    get_program(conn, id)
}

pub fn get_program(conn: &Connection, id: &str) -> Result<ProgramDetail, AcademicError> {
    let row = conn
        .query_row(
            r#"
            SELECT p.id, p.session_id, s.name as session_name, p.name, p.name_bn,
                   p.code, p.class_level, p.group_major, p.description, p.status,
                   p.created_at, p.updated_at,
                   (SELECT count(*) FROM batches b WHERE b.program_id = p.id) as batch_count,
                   (SELECT count(DISTINCT e.student_id) FROM enrollments e
                    JOIN batches b ON b.id = e.batch_id
                    WHERE b.program_id = p.id AND e.status = 'active') as active_student_count
            FROM programs p
            JOIN academic_sessions s ON s.id = p.session_id
            WHERE p.id = ?
            "#,
            [id],
            |row| {
                Ok(ProgramDetail {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    session_name: row.get(2)?,
                    name: row.get(3)?,
                    name_bn: row.get(4)?,
                    code: row.get(5)?,
                    class_level: row.get(6)?,
                    group_major: row.get(7)?,
                    description: row.get(8)?,
                    status: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    batch_count: row.get(12)?,
                    active_student_count: row.get(13)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AcademicError::ProgramNotFound(id.to_string()))?;

    Ok(row)
}

pub fn list_programs(
    conn: &Connection,
    session_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<ProgramSummary>, AcademicError> {
    let mut sql = String::from(
        r#"
        SELECT p.id, p.session_id, s.name as session_name, p.name, p.name_bn,
               p.code, p.class_level, p.group_major, p.status,
               (SELECT count(*) FROM batches b WHERE b.program_id = p.id) as batch_count,
               (SELECT count(DISTINCT e.student_id) FROM enrollments e
                JOIN batches b ON b.id = e.batch_id
                WHERE b.program_id = p.id AND e.status = 'active') as active_student_count
        FROM programs p
        JOIN academic_sessions s ON s.id = p.session_id
        WHERE 1=1
        "#,
    );

    let mut param_vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(s_id) = session_id.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND p.session_id = ?");
        param_vals.push(Box::new(s_id.to_string()));
    }

    if let Some(st) = status.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND p.status = ?");
        param_vals.push(Box::new(st.to_string()));
    }

    sql.push_str(" ORDER BY p.name ASC");

    let params_slice: Vec<&dyn rusqlite::ToSql> = param_vals.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let programs = stmt
        .query_map(params_slice.as_slice(), |row| {
            Ok(ProgramSummary {
                id: row.get(0)?,
                session_id: row.get(1)?,
                session_name: row.get(2)?,
                name: row.get(3)?,
                name_bn: row.get(4)?,
                code: row.get(5)?,
                class_level: row.get(6)?,
                group_major: row.get(7)?,
                status: row.get(8)?,
                batch_count: row.get(9)?,
                active_student_count: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(programs)
}

// -----------------------------------------------------------------------------
// Subject Operations
// -----------------------------------------------------------------------------

pub fn create_subject(
    conn: &Connection,
    input: &CreateSubjectInput,
    current_utc: &str,
) -> Result<SubjectItem, AcademicError> {
    let clean_code = input.code.trim();
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM subjects WHERE code = ?",
            [clean_code],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if exists {
        return Err(AcademicError::SubjectCodeExists(clean_code.to_string()));
    }

    let subject_id = Uuid::now_v7().to_string();

    conn.execute(
        r#"
        INSERT INTO subjects (id, name, name_bn, code, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        params![
            subject_id,
            input.name.trim(),
            input.name_bn.as_deref().map(|s| s.trim()),
            clean_code,
            current_utc,
            current_utc,
        ],
    )?;

    Ok(SubjectItem {
        id: subject_id,
        name: input.name.trim().to_string(),
        name_bn: input.name_bn.as_deref().map(|s| s.trim().to_string()),
        code: clean_code.to_string(),
        created_at: current_utc.to_string(),
        updated_at: current_utc.to_string(),
    })
}

pub fn list_subjects(conn: &Connection) -> Result<Vec<SubjectItem>, AcademicError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, name_bn, code, created_at, updated_at FROM subjects ORDER BY name ASC",
    )?;

    let items = stmt
        .query_map([], |row| {
            Ok(SubjectItem {
                id: row.get(0)?,
                name: row.get(1)?,
                name_bn: row.get(2)?,
                code: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

// -----------------------------------------------------------------------------
// Batch Operations
// -----------------------------------------------------------------------------

pub fn create_batch(
    conn: &Connection,
    input: &CreateBatchInput,
    allow_conflict: bool,
    current_utc: &str,
) -> Result<BatchDetail, AcademicError> {
    let program_exists: bool = conn
        .query_row(
            "SELECT 1 FROM programs WHERE id = ?",
            [&input.program_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !program_exists {
        return Err(AcademicError::ProgramNotFound(input.program_id.clone()));
    }

    let clean_code = match input
        .code
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(c) => c.to_string(),
        None => {
            let short_id = &Uuid::now_v7().to_string()[0..6].to_uppercase();
            format!("B-{}", short_id)
        }
    };

    let code_exists: bool = conn
        .query_row(
            "SELECT 1 FROM batches WHERE program_id = ? AND code = ?",
            params![input.program_id, &clean_code],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if code_exists {
        return Err(AcademicError::BatchCodeExists(clean_code));
    }

    if let Some(ref t_id) = input.teacher_staff_id {
        let clean_t = t_id.trim();
        if !clean_t.is_empty() {
            let staff_exists: bool = conn
                .query_row("SELECT 1 FROM staff WHERE id = ?", [clean_t], |_| Ok(true))
                .optional()?
                .unwrap_or(false);

            if !staff_exists {
                return Err(AcademicError::StaffNotFound(clean_t.to_string()));
            }
        }
    }

    schedule::validate_schedule(&input.schedule)?;

    // Schedule conflict detection
    let conflicts = schedule::detect_conflicts(
        conn,
        None,
        input.teacher_staff_id.as_deref(),
        input.room_number.as_deref(),
        &input.schedule,
    )?;

    if !conflicts.is_empty() && !allow_conflict {
        return Err(AcademicError::ScheduleConflict(conflicts));
    }

    let batch_id = Uuid::now_v7().to_string();
    let shift = input.shift.as_deref().unwrap_or("morning");
    let capacity = input.capacity.unwrap_or(40).max(1);
    let start_date = input
        .start_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]);
    let schedule_json = serde_json::to_string(&input.schedule).map_err(|e| {
        AcademicError::InvalidSchedule(format!("Failed to serialize schedule: {}", e))
    })?;

    conn.execute(
        r#"
        INSERT INTO batches (
            id, program_id, name, code, teacher_staff_id, room_number,
            shift, schedule_json, capacity, start_date, end_date,
            status, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)
        "#,
        params![
            batch_id,
            input.program_id,
            input.name.trim(),
            clean_code,
            input.teacher_staff_id.as_deref().map(|s| s.trim()),
            input.room_number.as_deref().map(|s| s.trim()),
            shift,
            schedule_json,
            capacity,
            start_date,
            input.end_date.as_deref().map(|s| s.trim()),
            current_utc,
            current_utc,
        ],
    )?;

    get_batch(conn, &batch_id)
}

pub fn update_batch(
    conn: &Connection,
    id: &str,
    input: &UpdateBatchInput,
    allow_conflict: bool,
    current_utc: &str,
) -> Result<BatchDetail, AcademicError> {
    let existing = get_batch(conn, id)?;

    let clean_code = match input
        .code
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(c) => c.to_string(),
        None => existing.code.clone(),
    };

    if clean_code != existing.code {
        let code_exists: bool = conn
            .query_row(
                "SELECT 1 FROM batches WHERE program_id = ? AND code = ? AND id != ?",
                params![existing.program_id, &clean_code, id],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);

        if code_exists {
            return Err(AcademicError::BatchCodeExists(clean_code));
        }
    }

    let teacher_staff_id = input
        .teacher_staff_id
        .as_deref()
        .map(|s| s.trim())
        .or(existing.teacher_staff_id.as_deref());
    if let Some(clean_t) = teacher_staff_id {
        if !clean_t.is_empty() {
            let staff_exists: bool = conn
                .query_row("SELECT 1 FROM staff WHERE id = ?", [clean_t], |_| Ok(true))
                .optional()?
                .unwrap_or(false);

            if !staff_exists {
                return Err(AcademicError::StaffNotFound(clean_t.to_string()));
            }
        }
    }

    let schedule_slots = input.schedule.as_ref().unwrap_or(&existing.schedule);
    schedule::validate_schedule(schedule_slots)?;

    // Schedule conflict detection (excluding this batch)
    let room_number = input
        .room_number
        .as_deref()
        .map(|s| s.trim())
        .or(existing.room_number.as_deref());
    let conflicts = schedule::detect_conflicts(
        conn,
        Some(id),
        teacher_staff_id,
        room_number,
        schedule_slots,
    )?;

    if !conflicts.is_empty() && !allow_conflict {
        return Err(AcademicError::ScheduleConflict(conflicts));
    }

    let schedule_json = serde_json::to_string(schedule_slots).map_err(|e| {
        AcademicError::InvalidSchedule(format!("Failed to serialize schedule: {}", e))
    })?;

    let name = input
        .name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.name);
    let shift = input
        .shift
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.shift);
    let capacity = input.capacity.unwrap_or(existing.capacity);
    let start_date = input
        .start_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.start_date);
    let end_date = input
        .end_date
        .as_deref()
        .map(|s| s.trim())
        .or(existing.end_date.as_deref());
    let status = input
        .status
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.status);

    conn.execute(
        r#"
        UPDATE batches
        SET name = ?,
            code = ?,
            teacher_staff_id = ?,
            room_number = ?,
            shift = ?,
            schedule_json = ?,
            capacity = ?,
            start_date = ?,
            end_date = ?,
            status = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        params![
            name,
            clean_code,
            teacher_staff_id,
            room_number,
            shift,
            schedule_json,
            capacity,
            start_date,
            end_date,
            status,
            current_utc,
            id,
        ],
    )?;

    get_batch(conn, id)
}

pub fn get_batch(conn: &Connection, id: &str) -> Result<BatchDetail, AcademicError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT b.id, b.program_id, p.name as program_name, b.name, b.code,
               b.teacher_staff_id, s.name as teacher_name, b.room_number,
               b.shift, b.schedule_json, b.capacity,
               (SELECT count(*) FROM enrollments e WHERE e.batch_id = b.id AND e.status = 'active') as enrolled_count,
               b.start_date, b.end_date, b.status, b.created_at, b.updated_at
        FROM batches b
        JOIN programs p ON p.id = b.program_id
        LEFT JOIN staff s ON s.id = b.teacher_staff_id
        WHERE b.id = ?
        "#,
    )?;

    let detail = stmt
        .query_row([id], |row| {
            let id: String = row.get(0)?;
            let program_id: String = row.get(1)?;
            let program_name: String = row.get(2)?;
            let name: String = row.get(3)?;
            let code: String = row.get(4)?;
            let teacher_staff_id: Option<String> = row.get(5)?;
            let teacher_name: Option<String> = row.get(6)?;
            let room_number: Option<String> = row.get(7)?;
            let shift: String = row.get(8)?;
            let schedule_json: String = row.get(9)?;
            let capacity: i64 = row.get(10)?;
            let enrolled_count: i64 = row.get(11)?;
            let start_date: String = row.get(12)?;
            let end_date: Option<String> = row.get(13)?;
            let status: String = row.get(14)?;
            let created_at: String = row.get(15)?;
            let updated_at: String = row.get(16)?;

            let schedule: Vec<ScheduleSlot> =
                serde_json::from_str(&schedule_json).unwrap_or_default();
            let available_seats = (capacity - enrolled_count).max(0);

            Ok(BatchDetail {
                id,
                program_id,
                program_name,
                name,
                code,
                teacher_staff_id,
                teacher_name,
                room_number,
                shift,
                schedule,
                capacity,
                enrolled_count,
                available_seats,
                status,
                start_date,
                end_date,
                subjects: vec![],
                created_at,
                updated_at,
            })
        })
        .optional()?
        .ok_or_else(|| AcademicError::BatchNotFound(id.to_string()))?;

    // Load batch subjects
    let mut sub_stmt = conn.prepare(
        r#"
        SELECT bs.subject_id, sub.name as subject_name, sub.code as subject_code,
               bs.teacher_staff_id, st.name as teacher_name
        FROM batch_subjects bs
        JOIN subjects sub ON sub.id = bs.subject_id
        LEFT JOIN staff st ON st.id = bs.teacher_staff_id
        WHERE bs.batch_id = ?
        ORDER BY sub.name ASC
        "#,
    )?;

    let subjects = sub_stmt
        .query_map([id], |row| {
            Ok(BatchSubjectItem {
                subject_id: row.get(0)?,
                subject_name: row.get(1)?,
                subject_code: row.get(2)?,
                teacher_staff_id: row.get(3)?,
                teacher_name: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = detail;
    result.subjects = subjects;
    Ok(result)
}

pub fn list_batches(
    conn: &Connection,
    program_id: Option<&str>,
    teacher_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<BatchSummary>, AcademicError> {
    let mut sql = String::from(
        r#"
        SELECT b.id, b.program_id, p.name as program_name, b.name, b.code,
               b.teacher_staff_id, s.name as teacher_name, b.room_number,
               b.shift, b.schedule_json, b.capacity,
               (SELECT count(*) FROM enrollments e WHERE e.batch_id = b.id AND e.status = 'active') as enrolled_count,
               b.status, b.start_date, b.end_date
        FROM batches b
        JOIN programs p ON p.id = b.program_id
        LEFT JOIN staff s ON s.id = b.teacher_staff_id
        WHERE 1=1
        "#,
    );

    let mut param_vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(p_id) = program_id.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND b.program_id = ?");
        param_vals.push(Box::new(p_id.to_string()));
    }

    if let Some(t_id) = teacher_id.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND b.teacher_staff_id = ?");
        param_vals.push(Box::new(t_id.to_string()));
    }

    if let Some(st) = status.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND b.status = ?");
        param_vals.push(Box::new(st.to_string()));
    }

    sql.push_str(" ORDER BY b.start_date DESC, b.name ASC");

    let params_slice: Vec<&dyn rusqlite::ToSql> = param_vals.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let batches = stmt
        .query_map(params_slice.as_slice(), |row| {
            let id: String = row.get(0)?;
            let program_id: String = row.get(1)?;
            let program_name: String = row.get(2)?;
            let name: String = row.get(3)?;
            let code: String = row.get(4)?;
            let teacher_staff_id: Option<String> = row.get(5)?;
            let teacher_name: Option<String> = row.get(6)?;
            let room_number: Option<String> = row.get(7)?;
            let shift: String = row.get(8)?;
            let schedule_json: String = row.get(9)?;
            let capacity: i64 = row.get(10)?;
            let enrolled_count: i64 = row.get(11)?;
            let status: String = row.get(12)?;
            let start_date: String = row.get(13)?;
            let end_date: Option<String> = row.get(14)?;

            let schedule: Vec<ScheduleSlot> =
                serde_json::from_str(&schedule_json).unwrap_or_default();
            let available_seats = (capacity - enrolled_count).max(0);

            Ok(BatchSummary {
                id,
                program_id,
                program_name,
                name,
                code,
                teacher_staff_id,
                teacher_name,
                room_number,
                shift,
                schedule,
                capacity,
                enrolled_count,
                available_seats,
                status,
                start_date,
                end_date,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(batches)
}

pub fn set_batch_subjects(
    conn: &Connection,
    batch_id: &str,
    subjects: &[BatchSubjectInput],
) -> Result<(), AcademicError> {
    let batch_exists: bool = conn
        .query_row("SELECT 1 FROM batches WHERE id = ?", [batch_id], |_| {
            Ok(true)
        })
        .optional()?
        .unwrap_or(false);

    if !batch_exists {
        return Err(AcademicError::BatchNotFound(batch_id.to_string()));
    }

    conn.execute("DELETE FROM batch_subjects WHERE batch_id = ?", [batch_id])?;

    for item in subjects {
        let sub_exists: bool = conn
            .query_row(
                "SELECT 1 FROM subjects WHERE id = ?",
                [&item.subject_id],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);

        if !sub_exists {
            return Err(AcademicError::SubjectNotFound(item.subject_id.clone()));
        }

        conn.execute(
            "INSERT INTO batch_subjects (batch_id, subject_id, teacher_staff_id) VALUES (?, ?, ?)",
            params![
                batch_id,
                item.subject_id,
                item.teacher_staff_id.as_deref().map(|s| s.trim()),
            ],
        )?;
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Staff Operations
// -----------------------------------------------------------------------------

pub fn create_staff(
    conn: &Connection,
    input: &CreateStaffInput,
    current_utc: &str,
) -> Result<StaffDetail, AcademicError> {
    let phone_norm = phone::normalize_phone(&input.phone)?;

    let staff_id = Uuid::now_v7().to_string();
    let salary_type = input.salary_type.as_deref().unwrap_or("monthly");
    let status = "active";
    let designation = input
        .designation
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Teacher");
    let joining_date = input
        .joining_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]);

    conn.execute(
        r#"
        INSERT INTO staff (
            id, name, name_bn, phone_norm, email, designation, department,
            educational_qualifications, nid, blood_group, gender, dob,
            present_address, joining_date, resignation_date, salary_type,
            status, notes, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        params![
            staff_id,
            input.name.trim(),
            input.name_bn.as_deref().map(|s| s.trim()),
            phone_norm,
            input.email.as_deref().map(|s| s.trim()),
            designation,
            input.department.as_deref().map(|s| s.trim()),
            input
                .educational_qualifications
                .as_deref()
                .map(|s| s.trim()),
            input.nid.as_deref().map(|s| s.trim()),
            input.blood_group.as_deref().map(|s| s.trim()),
            input.gender.as_deref().map(|s| s.trim()),
            input.dob.as_deref().map(|s| s.trim()),
            input.present_address.as_deref().map(|s| s.trim()),
            joining_date,
            input.resignation_date.as_deref().map(|s| s.trim()),
            salary_type,
            status,
            input.notes.as_deref().map(|s| s.trim()),
            current_utc,
            current_utc,
        ],
    )?;

    get_staff(conn, &staff_id)
}

pub fn update_staff(
    conn: &Connection,
    id: &str,
    input: &UpdateStaffInput,
    current_utc: &str,
) -> Result<StaffDetail, AcademicError> {
    let existing = get_staff(conn, id)?;

    let name = input
        .name
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.name);
    let name_bn = input
        .name_bn
        .as_deref()
        .map(|s| s.trim())
        .or(existing.name_bn.as_deref());
    let phone_raw = input
        .phone
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.phone_norm);
    let phone_norm = phone::normalize_phone(phone_raw)?;
    let email = input
        .email
        .as_deref()
        .map(|s| s.trim())
        .or(existing.email.as_deref());
    let designation = input
        .designation
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.designation);
    let department = input
        .department
        .as_deref()
        .map(|s| s.trim())
        .or(existing.department.as_deref());
    let educational_qualifications = input
        .educational_qualifications
        .as_deref()
        .map(|s| s.trim())
        .or(existing.educational_qualifications.as_deref());
    let nid = input
        .nid
        .as_deref()
        .map(|s| s.trim())
        .or(existing.nid.as_deref());
    let blood_group = input
        .blood_group
        .as_deref()
        .map(|s| s.trim())
        .or(existing.blood_group.as_deref());
    let gender = input
        .gender
        .as_deref()
        .map(|s| s.trim())
        .or(existing.gender.as_deref());
    let dob = input
        .dob
        .as_deref()
        .map(|s| s.trim())
        .or(existing.dob.as_deref());
    let present_address = input
        .present_address
        .as_deref()
        .map(|s| s.trim())
        .or(existing.present_address.as_deref());
    let joining_date = input
        .joining_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.joining_date);
    let resignation_date = input
        .resignation_date
        .as_deref()
        .map(|s| s.trim())
        .or(existing.resignation_date.as_deref());
    let salary_type = input
        .salary_type
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.salary_type);
    let status = input
        .status
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&existing.status);
    let notes = input
        .notes
        .as_deref()
        .map(|s| s.trim())
        .or(existing.notes.as_deref());

    conn.execute(
        r#"
        UPDATE staff
        SET name = ?,
            name_bn = ?,
            phone_norm = ?,
            email = ?,
            designation = ?,
            department = ?,
            educational_qualifications = ?,
            nid = ?,
            blood_group = ?,
            gender = ?,
            dob = ?,
            present_address = ?,
            joining_date = ?,
            resignation_date = ?,
            salary_type = ?,
            status = ?,
            notes = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        params![
            name,
            name_bn,
            phone_norm,
            email,
            designation,
            department,
            educational_qualifications,
            nid,
            blood_group,
            gender,
            dob,
            present_address,
            joining_date,
            resignation_date,
            salary_type,
            status,
            notes,
            current_utc,
            id,
        ],
    )?;

    get_staff(conn, id)
}

pub fn get_staff(conn: &Connection, id: &str) -> Result<StaffDetail, AcademicError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT s.id, s.name, s.name_bn, s.phone_norm, s.email, s.designation,
               s.department, s.educational_qualifications, s.nid, s.blood_group,
               s.gender, s.dob, s.present_address, s.joining_date, s.resignation_date,
               s.salary_type, s.status, s.notes, s.created_at, s.updated_at,
               (SELECT u.id FROM users u WHERE u.staff_profile_id = s.id LIMIT 1) as linked_user_id
        FROM staff s
        WHERE s.id = ?
        "#,
    )?;

    let detail = stmt
        .query_row([id], |row| {
            Ok(StaffDetail {
                id: row.get(0)?,
                name: row.get(1)?,
                name_bn: row.get(2)?,
                phone_norm: row.get(3)?,
                email: row.get(4)?,
                designation: row.get(5)?,
                department: row.get(6)?,
                educational_qualifications: row.get(7)?,
                nid: row.get(8)?,
                blood_group: row.get(9)?,
                gender: row.get(10)?,
                dob: row.get(11)?,
                present_address: row.get(12)?,
                joining_date: row.get(13)?,
                resignation_date: row.get(14)?,
                salary_type: row.get(15)?,
                status: row.get(16)?,
                notes: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
                linked_user_id: row.get(20)?,
                assigned_batches: vec![],
            })
        })
        .optional()?
        .ok_or_else(|| AcademicError::StaffNotFound(id.to_string()))?;

    // Load assigned batches
    let assigned_batches = list_batches(conn, None, Some(id), None)?;

    let mut result = detail;
    result.assigned_batches = assigned_batches;
    Ok(result)
}

pub fn list_staff(
    conn: &Connection,
    status: Option<&str>,
    department: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<StaffSummary>, AcademicError> {
    let mut sql = String::from(
        r#"
        SELECT s.id, s.name, s.name_bn, s.phone_norm, s.email, s.designation,
               s.department, s.status, s.salary_type, s.joining_date,
               (SELECT count(*) FROM batches b WHERE b.teacher_staff_id = s.id AND b.status = 'active') as active_batch_count
        FROM staff s
        WHERE 1=1
        "#,
    );

    let mut param_vals: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(st) = status.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND s.status = ?");
        param_vals.push(Box::new(st.to_string()));
    }

    if let Some(dep) = department.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND s.department = ?");
        param_vals.push(Box::new(dep.to_string()));
    }

    sql.push_str(" ORDER BY s.name ASC");

    let lim = limit.unwrap_or(50).max(1);
    let off = offset.unwrap_or(0).max(0);
    sql.push_str(" LIMIT ? OFFSET ?");
    param_vals.push(Box::new(lim));
    param_vals.push(Box::new(off));

    let params_slice: Vec<&dyn rusqlite::ToSql> = param_vals.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let staff_list = stmt
        .query_map(params_slice.as_slice(), |row| {
            Ok(StaffSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                name_bn: row.get(2)?,
                phone_norm: row.get(3)?,
                email: row.get(4)?,
                designation: row.get(5)?,
                department: row.get(6)?,
                status: row.get(7)?,
                salary_type: row.get(8)?,
                joining_date: row.get(9)?,
                active_batch_count: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(staff_list)
}

pub fn search_staff(
    conn: &Connection,
    query: &str,
    limit: Option<i64>,
) -> Result<Vec<StaffSearchHit>, AcademicError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    let lim = limit.unwrap_or(20).max(1);

    // Escape double quotes in query for FTS5
    let escaped = trimmed.replace('"', "\"\"");
    let fts_query = format!("\"{}\"", escaped);

    let mut stmt = conn.prepare(
        r#"
        SELECT s.id, s.name, s.name_bn, s.phone_norm, s.designation, s.department, s.status
        FROM staff_search_index idx
        JOIN staff s ON s.rowid = idx.rowid
        WHERE staff_search_index MATCH ?
        ORDER BY rank
        LIMIT ?
        "#,
    )?;

    let matches = stmt
        .query_map(params![fts_query, lim], |row| {
            Ok(StaffSearchHit {
                id: row.get(0)?,
                name: row.get(1)?,
                name_bn: row.get(2)?,
                phone_norm: row.get(3)?,
                designation: row.get(4)?,
                department: row.get(5)?,
                status: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(matches)
}

// -----------------------------------------------------------------------------
// Enrollment Operations
// -----------------------------------------------------------------------------

pub fn create_enrollment(
    conn: &Connection,
    input: &CreateEnrollmentInput,
    allow_overcapacity: bool,
    current_utc: &str,
) -> Result<EnrollmentDetail, AcademicError> {
    // 1. Verify student exists and is active
    let student_status: Option<String> = conn
        .query_row(
            "SELECT status FROM students WHERE id = ?",
            [&input.student_id],
            |row| row.get(0),
        )
        .optional()?;

    let status =
        student_status.ok_or_else(|| AcademicError::StudentNotFound(input.student_id.clone()))?;
    if status != "active" {
        return Err(AcademicError::StudentNotActive(input.student_id.clone()));
    }

    // 2. Verify batch exists and is active
    let batch_info: Option<(i64, String)> = conn
        .query_row(
            "SELECT capacity, status FROM batches WHERE id = ?",
            [&input.batch_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    let (capacity, b_status) =
        batch_info.ok_or_else(|| AcademicError::BatchNotFound(input.batch_id.clone()))?;
    if b_status != "active" {
        return Err(AcademicError::InvalidStatusTransition {
            from: b_status,
            to: "active".to_string(),
        });
    }

    // 3. Verify not already actively enrolled in this batch
    let already_enrolled: bool = conn
        .query_row(
            "SELECT 1 FROM enrollments WHERE student_id = ? AND batch_id = ? AND status = 'active'",
            params![input.student_id, input.batch_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if already_enrolled {
        return Err(AcademicError::AlreadyEnrolled);
    }

    // 4. Check capacity
    let active_count: i64 = conn.query_row(
        "SELECT count(*) FROM enrollments WHERE batch_id = ? AND status = 'active'",
        [&input.batch_id],
        |row| row.get(0),
    )?;

    if active_count >= capacity && !allow_overcapacity {
        return Err(AcademicError::BatchCapacityReached {
            capacity,
            enrolled: active_count,
        });
    }

    // 5. Determine roll number
    let roll = if let Some(req_roll) = input.roll {
        let roll_taken: bool = conn
            .query_row(
                "SELECT 1 FROM enrollments WHERE batch_id = ? AND roll = ? AND status = 'active'",
                params![input.batch_id, req_roll],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);

        if roll_taken {
            return Err(AcademicError::RollTaken { roll: req_roll });
        }
        Some(req_roll)
    } else {
        // Auto-allocate next roll
        let next_roll: i64 = conn.query_row(
            "SELECT coalesce(max(roll), 0) + 1 FROM enrollments WHERE batch_id = ? AND status = 'active'",
            [&input.batch_id],
            |row| row.get(0),
        )?;
        Some(next_roll)
    };

    let enrollment_id = Uuid::now_v7().to_string();
    let start_date = input
        .start_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]); // YYYY-MM-DD

    conn.execute(
        r#"
        INSERT INTO enrollments (
            id, student_id, batch_id, roll, start_date, end_date,
            status, end_reason, notes, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, NULL, 'active', NULL, ?, ?, ?)
        "#,
        params![
            enrollment_id,
            input.student_id,
            input.batch_id,
            roll,
            start_date,
            input.notes.as_deref().map(|s| s.trim()),
            current_utc,
            current_utc,
        ],
    )?;

    get_enrollment_detail(conn, &enrollment_id)
}

pub fn transfer_enrollment(
    conn: &Connection,
    input: &TransferEnrollmentInput,
    allow_overcapacity: bool,
    current_utc: &str,
) -> Result<EnrollmentDetail, AcademicError> {
    // 1. Check existing active enrollment in source batch
    let existing_enrollment_id: Option<String> = conn
        .query_row(
            "SELECT id FROM enrollments WHERE student_id = ? AND batch_id = ? AND status = 'active'",
            params![input.student_id, input.from_batch_id],
            |row| row.get(0),
        )
        .optional()?;

    let old_id = existing_enrollment_id
        .ok_or_else(|| AcademicError::NotEnrolledInBatch(input.from_batch_id.clone()))?;

    // 2. Check destination batch exists and is active
    let dest_capacity: i64 = conn
        .query_row(
            "SELECT capacity FROM batches WHERE id = ? AND status = 'active'",
            [&input.to_batch_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AcademicError::BatchNotFound(input.to_batch_id.clone()))?;

    // 3. Check not already enrolled in destination batch
    let already_in_dest: bool = conn
        .query_row(
            "SELECT 1 FROM enrollments WHERE student_id = ? AND batch_id = ? AND status = 'active'",
            params![input.student_id, input.to_batch_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if already_in_dest {
        return Err(AcademicError::AlreadyEnrolled);
    }

    // 4. Check destination capacity
    let dest_active_count: i64 = conn.query_row(
        "SELECT count(*) FROM enrollments WHERE batch_id = ? AND status = 'active'",
        [&input.to_batch_id],
        |row| row.get(0),
    )?;

    if dest_active_count >= dest_capacity && !allow_overcapacity {
        return Err(AcademicError::BatchCapacityReached {
            capacity: dest_capacity,
            enrolled: dest_active_count,
        });
    }

    // 5. Determine roll in destination batch
    let new_roll = if let Some(req_roll) = input.new_roll {
        let roll_taken: bool = conn
            .query_row(
                "SELECT 1 FROM enrollments WHERE batch_id = ? AND roll = ? AND status = 'active'",
                params![input.to_batch_id, req_roll],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);

        if roll_taken {
            return Err(AcademicError::RollTaken { roll: req_roll });
        }
        Some(req_roll)
    } else {
        let next_roll: i64 = conn.query_row(
            "SELECT coalesce(max(roll), 0) + 1 FROM enrollments WHERE batch_id = ? AND status = 'active'",
            [&input.to_batch_id],
            |row| row.get(0),
        )?;
        Some(next_roll)
    };

    let transfer_date = input
        .transfer_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]);

    // 6. Close old enrollment
    conn.execute(
        r#"
        UPDATE enrollments
        SET status = 'ended',
            end_reason = 'transferred',
            end_date = ?,
            updated_at = ?
        WHERE id = ?
        "#,
        params![transfer_date, current_utc, old_id],
    )?;

    // 7. Create new enrollment
    let new_enrollment_id = Uuid::now_v7().to_string();

    conn.execute(
        r#"
        INSERT INTO enrollments (
            id, student_id, batch_id, roll, start_date, end_date,
            status, end_reason, notes, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, NULL, 'active', NULL, ?, ?, ?)
        "#,
        params![
            new_enrollment_id,
            input.student_id,
            input.to_batch_id,
            new_roll,
            transfer_date,
            input.notes.as_deref().map(|s| s.trim()),
            current_utc,
            current_utc,
        ],
    )?;

    get_enrollment_detail(conn, &new_enrollment_id)
}

pub fn end_enrollment(
    conn: &Connection,
    input: &EndEnrollmentInput,
    current_utc: &str,
) -> Result<EnrollmentDetail, AcademicError> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM enrollments WHERE id = ?",
            [&input.enrollment_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !exists {
        return Err(AcademicError::StudentNotFound(input.enrollment_id.clone()));
    }

    let end_date = input
        .end_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]);

    conn.execute(
        r#"
        UPDATE enrollments
        SET status = 'ended',
            end_reason = ?,
            end_date = ?,
            notes = coalesce(?, notes),
            updated_at = ?
        WHERE id = ?
        "#,
        params![
            input.end_reason.trim(),
            end_date,
            input.notes.as_deref().map(|s| s.trim()),
            current_utc,
            input.enrollment_id,
        ],
    )?;

    get_enrollment_detail(conn, &input.enrollment_id)
}

pub fn bulk_create_enrollments(
    conn: &Connection,
    input: &BulkEnrollInput,
    allow_overcapacity: bool,
    current_utc: &str,
) -> Result<Vec<EnrollmentDetail>, AcademicError> {
    let batch_capacity: i64 = conn
        .query_row(
            "SELECT capacity FROM batches WHERE id = ? AND status = 'active'",
            [&input.batch_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AcademicError::BatchNotFound(input.batch_id.clone()))?;

    let mut results = Vec::new();
    let start_date = input
        .start_date
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&current_utc[0..10]);

    for student_id in &input.student_ids {
        let active_count: i64 = conn.query_row(
            "SELECT count(*) FROM enrollments WHERE batch_id = ? AND status = 'active'",
            [&input.batch_id],
            |row| row.get(0),
        )?;

        if active_count >= batch_capacity && !allow_overcapacity {
            return Err(AcademicError::BatchCapacityReached {
                capacity: batch_capacity,
                enrolled: active_count,
            });
        }

        // Allocate roll
        let next_roll: i64 = conn.query_row(
            "SELECT coalesce(max(roll), 0) + 1 FROM enrollments WHERE batch_id = ? AND status = 'active'",
            [&input.batch_id],
            |row| row.get(0),
        )?;

        let eid = Uuid::now_v7().to_string();

        conn.execute(
            r#"
            INSERT INTO enrollments (
                id, student_id, batch_id, roll, start_date, end_date,
                status, end_reason, notes, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, NULL, 'active', NULL, ?, ?, ?)
            "#,
            params![
                eid,
                student_id,
                input.batch_id,
                next_roll,
                start_date,
                input.notes.as_deref().map(|s| s.trim()),
                current_utc,
                current_utc,
            ],
        )?;

        results.push(eid);
    }

    let mut details = Vec::new();
    for id in results {
        details.push(get_enrollment_detail(conn, &id)?);
    }

    Ok(details)
}

pub fn get_enrollment_detail(
    conn: &Connection,
    id: &str,
) -> Result<EnrollmentDetail, AcademicError> {
    let row = conn
        .query_row(
            r#"
            SELECT e.id, e.student_id, s.student_code, s.name as student_name,
                   e.batch_id, b.name as batch_name, p.name as program_name,
                   e.roll, e.start_date, e.end_date, e.status, e.end_reason,
                   e.notes, e.created_at, e.updated_at
            FROM enrollments e
            JOIN students s ON s.id = e.student_id
            JOIN batches b ON b.id = e.batch_id
            JOIN programs p ON p.id = b.program_id
            WHERE e.id = ?
            "#,
            [id],
            |row| {
                Ok(EnrollmentDetail {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    student_code: row.get(2)?,
                    student_name: row.get(3)?,
                    batch_id: row.get(4)?,
                    batch_name: row.get(5)?,
                    program_name: row.get(6)?,
                    roll: row.get(7)?,
                    start_date: row.get(8)?,
                    end_date: row.get(9)?,
                    status: row.get(10)?,
                    end_reason: row.get(11)?,
                    notes: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AcademicError::StudentNotFound(id.to_string()))?;

    Ok(row)
}

pub fn list_batch_roster(
    conn: &Connection,
    batch_id: &str,
    status: Option<&str>,
) -> Result<Vec<BatchRosterItem>, AcademicError> {
    let batch_exists: bool = conn
        .query_row("SELECT 1 FROM batches WHERE id = ?", [batch_id], |_| {
            Ok(true)
        })
        .optional()?
        .unwrap_or(false);

    if !batch_exists {
        return Err(AcademicError::BatchNotFound(batch_id.to_string()));
    }

    let mut sql = String::from(
        r#"
        SELECT e.id as enrollment_id, s.id as student_id, s.student_code,
               s.name as student_name, s.name_bn as student_name_bn,
               s.phone_norm,
               (SELECT g.phone_norm FROM guardians g
                JOIN student_guardians sg ON sg.guardian_id = g.id
                WHERE sg.student_id = s.id AND sg.is_primary = 1 LIMIT 1) as guardian_phone,
               (SELECT g.name FROM guardians g
                JOIN student_guardians sg ON sg.guardian_id = g.id
                WHERE sg.student_id = s.id AND sg.is_primary = 1 LIMIT 1) as guardian_name,
               e.roll, e.start_date, e.status, s.admission_date
        FROM enrollments e
        JOIN students s ON s.id = e.student_id
        WHERE e.batch_id = ?
        "#,
    );

    let mut param_vals: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(batch_id.to_string())];

    if let Some(st) = status.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND e.status = ?");
        param_vals.push(Box::new(st.to_string()));
    }

    sql.push_str(" ORDER BY e.roll ASC, s.name ASC");

    let params_slice: Vec<&dyn rusqlite::ToSql> = param_vals.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let roster = stmt
        .query_map(params_slice.as_slice(), |row| {
            Ok(BatchRosterItem {
                enrollment_id: row.get(0)?,
                student_id: row.get(1)?,
                student_code: row.get(2)?,
                student_name: row.get(3)?,
                student_name_bn: row.get(4)?,
                phone_norm: row.get(5)?,
                guardian_phone: row.get(6)?,
                guardian_name: row.get(7)?,
                roll: row.get(8)?,
                start_date: row.get(9)?,
                status: row.get(10)?,
                admission_date: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(roster)
}

pub fn list_student_enrollments(
    conn: &Connection,
    student_id: &str,
) -> Result<Vec<StudentEnrollmentHistoryItem>, AcademicError> {
    let student_exists: bool = conn
        .query_row("SELECT 1 FROM students WHERE id = ?", [student_id], |_| {
            Ok(true)
        })
        .optional()?
        .unwrap_or(false);

    if !student_exists {
        return Err(AcademicError::StudentNotFound(student_id.to_string()));
    }

    let mut stmt = conn.prepare(
        r#"
        SELECT e.id as enrollment_id, b.id as batch_id, b.name as batch_name,
               b.code as batch_code, p.name as program_name, b.shift,
               b.room_number, e.roll, e.start_date, e.end_date, e.status,
               e.end_reason, e.notes
        FROM enrollments e
        JOIN batches b ON b.id = e.batch_id
        JOIN programs p ON p.id = b.program_id
        WHERE e.student_id = ?
        ORDER BY (CASE WHEN e.status = 'active' THEN 0 ELSE 1 END), e.start_date DESC
        "#,
    )?;

    let history = stmt
        .query_map([student_id], |row| {
            Ok(StudentEnrollmentHistoryItem {
                enrollment_id: row.get(0)?,
                batch_id: row.get(1)?,
                batch_name: row.get(2)?,
                batch_code: row.get(3)?,
                program_name: row.get(4)?,
                shift: row.get(5)?,
                room_number: row.get(6)?,
                roll: row.get(7)?,
                start_date: row.get(8)?,
                end_date: row.get(9)?,
                status: row.get(10)?,
                end_reason: row.get(11)?,
                notes: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(history)
}
