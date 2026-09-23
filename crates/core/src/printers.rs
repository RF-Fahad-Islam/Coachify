use crate::db::now_utc_iso;
use crate::exam::StudentReportCardData;
use crate::ledger::DueStatementData;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PrintersError {
    #[error("Print template configuration not found for document type: {0}")]
    TemplateNotFound(String),
    #[error("Invalid paper size: {0}")]
    InvalidPaperSize(String),
    #[error("Invalid orientation: {0}")]
    InvalidOrientation(String),
    #[error("Invalid document type: {0}")]
    InvalidDocumentType(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

impl PrintersError {
    pub fn code(&self) -> &'static str {
        match self {
            PrintersError::TemplateNotFound(_) => "print_template_not_found",
            PrintersError::InvalidPaperSize(_) => "invalid_paper_size",
            PrintersError::InvalidOrientation(_) => "invalid_orientation",
            PrintersError::InvalidDocumentType(_) => "invalid_document_type",
            PrintersError::Sql(_) => "printer_sql_error",
            PrintersError::NotFound(_) => "printer_not_found",
        }
    }
}

impl From<rusqlite::Error> for PrintersError {
    fn from(e: rusqlite::Error) -> Self {
        PrintersError::Sql(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// DTOs for Template Configurations and Job History
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PrintTemplateConfig {
    pub id: String,
    pub document_type: String,
    pub paper_size: String,
    pub orientation: String,
    pub show_header: bool,
    pub show_logo: bool,
    pub show_watermark: bool,
    pub custom_header_text: Option<String>,
    pub custom_footer_text: Option<String>,
    pub signature_title: Option<String>,
    pub options_json: String,
    pub updated_at: String,
    pub updated_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PrintTemplateConfigInput {
    pub document_type: String,
    pub paper_size: String,
    pub orientation: String,
    pub show_header: bool,
    pub show_logo: bool,
    pub show_watermark: bool,
    pub custom_header_text: Option<String>,
    pub custom_footer_text: Option<String>,
    pub signature_title: Option<String>,
    pub options_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PrintJobRun {
    pub id: String,
    pub document_type: String,
    pub batch_mode: bool,
    pub item_count: u32,
    pub printer_name: Option<String>,
    pub paper_size: String,
    pub created_at: String,
    pub user_id: String,
    pub status: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct RecordPrintJobInput {
    pub document_type: String,
    pub batch_mode: bool,
    pub item_count: u32,
    pub printer_name: Option<String>,
    pub paper_size: String,
    pub status: String,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct PrintJobListResult {
    pub items: Vec<PrintJobRun>,
    pub total_count: u64,
}

// ---------------------------------------------------------------------------
// DTOs for Student ID Cards and Admit Cards
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct StudentIdCardData {
    pub student_id: String,
    pub student_code: String,
    pub full_name_en: String,
    pub full_name_bn: Option<String>,
    pub gender: String,
    pub blood_group: Option<String>,
    pub emergency_phone: Option<String>,
    pub primary_batch_name: Option<String>,
    pub academic_session: Option<String>,
    pub barcode_payload: String,
    pub photo_data_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct ExamSubjectScheduleItem {
    pub subject_id: String,
    pub subject_name: String,
    pub exam_date: String,
    pub start_time: String,
    pub room_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdmitCardStudentItem {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub batch_name: String,
    pub roll_number: Option<String>,
    pub photo_data_uri: Option<String>,
    pub qr_verification_payload: String,
    pub subjects: Vec<ExamSubjectScheduleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct AdmitCardBatchData {
    pub exam_id: String,
    pub exam_title: String,
    pub academic_session: Option<String>,
    pub center_instructions: Vec<String>,
    pub students: Vec<AdmitCardStudentItem>,
}

// ---------------------------------------------------------------------------
// Template Configuration & Job Audit Methods
// ---------------------------------------------------------------------------

pub fn get_template_configs(
    conn: &Connection,
) -> Result<Vec<PrintTemplateConfig>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, document_type, paper_size, orientation, show_header, show_logo,
                show_watermark, custom_header_text, custom_footer_text, signature_title,
                options_json, updated_at, updated_by_user_id
         FROM print_template_configs
         ORDER BY document_type ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(PrintTemplateConfig {
            id: row.get(0)?,
            document_type: row.get(1)?,
            paper_size: row.get(2)?,
            orientation: row.get(3)?,
            show_header: row.get::<_, i64>(4)? == 1,
            show_logo: row.get::<_, i64>(5)? == 1,
            show_watermark: row.get::<_, i64>(6)? == 1,
            custom_header_text: row.get(7)?,
            custom_footer_text: row.get(8)?,
            signature_title: row.get(9)?,
            options_json: row.get(10)?,
            updated_at: row.get(11)?,
            updated_by_user_id: row.get(12)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn update_template_config(
    conn: &Connection,
    user_id: Option<&str>,
    input: &PrintTemplateConfigInput,
) -> Result<PrintTemplateConfig, rusqlite::Error> {
    let now = now_utc_iso();

    conn.execute(
        "INSERT INTO print_template_configs (
            id, document_type, paper_size, orientation, show_header, show_logo,
            show_watermark, custom_header_text, custom_footer_text, signature_title,
            options_json, updated_at, updated_by_user_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
        )
        ON CONFLICT(document_type) DO UPDATE SET
            paper_size = excluded.paper_size,
            orientation = excluded.orientation,
            show_header = excluded.show_header,
            show_logo = excluded.show_logo,
            show_watermark = excluded.show_watermark,
            custom_header_text = excluded.custom_header_text,
            custom_footer_text = excluded.custom_footer_text,
            signature_title = excluded.signature_title,
            options_json = excluded.options_json,
            updated_at = excluded.updated_at,
            updated_by_user_id = excluded.updated_by_user_id",
        params![
            format!("ptc-{}", input.document_type),
            input.document_type,
            input.paper_size,
            input.orientation,
            if input.show_header { 1 } else { 0 },
            if input.show_logo { 1 } else { 0 },
            if input.show_watermark { 1 } else { 0 },
            input.custom_header_text,
            input.custom_footer_text,
            input.signature_title,
            input.options_json,
            now,
            user_id,
        ],
    )?;

    conn.query_row(
        "SELECT id, document_type, paper_size, orientation, show_header, show_logo,
                show_watermark, custom_header_text, custom_footer_text, signature_title,
                options_json, updated_at, updated_by_user_id
         FROM print_template_configs
         WHERE document_type = ?1",
        params![input.document_type],
        |row| {
            Ok(PrintTemplateConfig {
                id: row.get(0)?,
                document_type: row.get(1)?,
                paper_size: row.get(2)?,
                orientation: row.get(3)?,
                show_header: row.get::<_, i64>(4)? == 1,
                show_logo: row.get::<_, i64>(5)? == 1,
                show_watermark: row.get::<_, i64>(6)? == 1,
                custom_header_text: row.get(7)?,
                custom_footer_text: row.get(8)?,
                signature_title: row.get(9)?,
                options_json: row.get(10)?,
                updated_at: row.get(11)?,
                updated_by_user_id: row.get(12)?,
            })
        },
    )
}

pub fn record_print_job(
    conn: &Connection,
    user_id: &str,
    input: &RecordPrintJobInput,
) -> Result<PrintJobRun, rusqlite::Error> {
    let id = format!("pjr-{}", Uuid::now_v7());
    let now = now_utc_iso();
    let meta = input
        .metadata_json
        .clone()
        .unwrap_or_else(|| "{}".to_string());

    conn.execute(
        "INSERT INTO print_job_runs (
            id, document_type, batch_mode, item_count, printer_name, paper_size,
            created_at, user_id, status, metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.document_type,
            if input.batch_mode { 1 } else { 0 },
            input.item_count as i64,
            input.printer_name,
            input.paper_size,
            now,
            user_id,
            input.status,
            meta,
        ],
    )?;

    Ok(PrintJobRun {
        id,
        document_type: input.document_type.clone(),
        batch_mode: input.batch_mode,
        item_count: input.item_count,
        printer_name: input.printer_name.clone(),
        paper_size: input.paper_size.clone(),
        created_at: now,
        user_id: user_id.to_string(),
        status: input.status.clone(),
        metadata_json: meta,
    })
}

pub fn list_print_jobs(
    conn: &Connection,
    limit: Option<u32>,
    offset: Option<u32>,
    document_type: Option<&str>,
) -> Result<PrintJobListResult, rusqlite::Error> {
    let lim = limit.unwrap_or(50) as i64;
    let off = offset.unwrap_or(0) as i64;

    let (items, total_count) = if let Some(dt) = document_type {
        let total: i64 = conn.query_row(
            "SELECT count(*) FROM print_job_runs WHERE document_type = ?1",
            params![dt],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, document_type, batch_mode, item_count, printer_name, paper_size,
                    created_at, user_id, status, metadata_json
             FROM print_job_runs
             WHERE document_type = ?1
             ORDER BY created_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt.query_map(params![dt, lim, off], |row| {
            Ok(PrintJobRun {
                id: row.get(0)?,
                document_type: row.get(1)?,
                batch_mode: row.get::<_, i64>(2)? == 1,
                item_count: row.get::<_, i64>(3)? as u32,
                printer_name: row.get(4)?,
                paper_size: row.get(5)?,
                created_at: row.get(6)?,
                user_id: row.get(7)?,
                status: row.get(8)?,
                metadata_json: row.get(9)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        (list, total as u64)
    } else {
        let total: i64 =
            conn.query_row("SELECT count(*) FROM print_job_runs", [], |row| row.get(0))?;

        let mut stmt = conn.prepare(
            "SELECT id, document_type, batch_mode, item_count, printer_name, paper_size,
                    created_at, user_id, status, metadata_json
             FROM print_job_runs
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![lim, off], |row| {
            Ok(PrintJobRun {
                id: row.get(0)?,
                document_type: row.get(1)?,
                batch_mode: row.get::<_, i64>(2)? == 1,
                item_count: row.get::<_, i64>(3)? as u32,
                printer_name: row.get(4)?,
                paper_size: row.get(5)?,
                created_at: row.get(6)?,
                user_id: row.get(7)?,
                status: row.get(8)?,
                metadata_json: row.get(9)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        (list, total as u64)
    };

    Ok(PrintJobListResult { items, total_count })
}

// ---------------------------------------------------------------------------
// Bulk Printable Data Queries (AC-1, AC-2, AC-3)
// ---------------------------------------------------------------------------

pub fn get_id_card_data(
    conn: &Connection,
    student_ids: &[String],
) -> Result<Vec<StudentIdCardData>, rusqlite::Error> {
    if student_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut cards = Vec::new();

    for id in student_ids {
        let student_row = conn
            .query_row(
                "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.blood_group,
                    s.emergency_phone_norm, s.photo_vault_id
             FROM students s
             WHERE s.id = ?1",
                params![id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()?;

        let Some((sid, code, name, name_bn, gender, blood_group, emerg_phone, _photo_vault_id)) =
            student_row
        else {
            continue;
        };

        // Fallback emergency contact from guardian if student emergency phone is empty
        let emergency_phone = if emerg_phone.is_some() {
            emerg_phone
        } else {
            conn.query_row(
                "SELECT g.phone_norm
                 FROM student_guardians sg
                 JOIN guardians g ON g.id = sg.guardian_id
                 WHERE sg.student_id = ?1 AND sg.is_primary = 1",
                params![id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten()
        };

        // Primary batch and academic session
        let batch_info = conn
            .query_row(
                "SELECT b.name, s.name
             FROM enrollments e
             JOIN batches b ON b.id = e.batch_id
             JOIN programs p ON p.id = b.program_id
             LEFT JOIN academic_sessions s ON s.id = p.session_id
             WHERE e.student_id = ?1 AND e.status = 'active'
             ORDER BY e.start_date DESC LIMIT 1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()?;

        let (primary_batch_name, academic_session) = match batch_info {
            Some((b_name, sess)) => (Some(b_name), sess),
            None => (None, None),
        };

        cards.push(StudentIdCardData {
            student_id: sid,
            student_code: code.clone(),
            full_name_en: name,
            full_name_bn: name_bn,
            gender,
            blood_group,
            emergency_phone,
            primary_batch_name,
            academic_session,
            barcode_payload: code,
            photo_data_uri: None, // Photo vault rendering handles client-side or fallback SVG
        });
    }

    Ok(cards)
}

pub fn get_admit_card_data(
    conn: &Connection,
    exam_id: &str,
    batch_id: Option<&str>,
    student_ids: Option<&[String]>,
) -> Result<AdmitCardBatchData, rusqlite::Error> {
    let (exam_title, session_name): (String, Option<String>) = conn.query_row(
        "SELECT e.name, s.name
         FROM exams e
         LEFT JOIN academic_sessions s ON s.id = e.session_id
         WHERE e.id = ?1",
        params![exam_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // Fetch subjects with schedule
    let mut subj_stmt = conn.prepare(
        "SELECT es.id, s.name, es.exam_date, es.start_time, es.room_number
         FROM exam_subjects es
         JOIN subjects s ON s.id = es.subject_id
         WHERE es.exam_id = ?1
         ORDER BY es.exam_date ASC, es.start_time ASC",
    )?;

    let subjects = subj_stmt
        .query_map(params![exam_id], |row| {
            Ok(ExamSubjectScheduleItem {
                subject_id: row.get(0)?,
                subject_name: row.get(1)?,
                exam_date: row.get(2)?,
                start_time: row.get(3)?,
                room_number: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Determine target students
    let mut resolved_students = Vec::new();

    if let Some(sids) = student_ids {
        for sid in sids {
            let s_row = conn
                .query_row(
                    "SELECT s.id, s.student_code, s.name, b.name, s.current_school_roll
                 FROM students s
                 LEFT JOIN enrollments e ON e.student_id = s.id AND e.status = 'active'
                 LEFT JOIN batches b ON b.id = e.batch_id
                 WHERE s.id = ?1
                 LIMIT 1",
                    params![sid],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, Option<String>>(3)?
                                .unwrap_or_else(|| "General".to_string()),
                            row.get::<_, Option<i64>>(4)?.map(|r| r.to_string()),
                        ))
                    },
                )
                .optional()?;

            if let Some((id, code, name, batch, roll)) = s_row {
                let qr = format!("ADM|{}|{}|VERIFIED", exam_id, code);
                resolved_students.push(AdmitCardStudentItem {
                    student_id: id,
                    student_code: code,
                    student_name: name,
                    batch_name: batch,
                    roll_number: roll,
                    photo_data_uri: None,
                    qr_verification_payload: qr,
                    subjects: subjects.clone(),
                });
            }
        }
    } else if let Some(bid) = batch_id {
        let mut st_stmt = conn.prepare(
            "SELECT s.id, s.student_code, s.name, b.name, s.current_school_roll
             FROM enrollments e
             JOIN students s ON s.id = e.student_id
             JOIN batches b ON b.id = e.batch_id
             WHERE e.batch_id = ?1 AND e.status = 'active'
             ORDER BY s.student_code ASC",
        )?;

        let s_rows = st_stmt.query_map(params![bid], |row| {
            let id: String = row.get(0)?;
            let code: String = row.get(1)?;
            let name: String = row.get(2)?;
            let batch: String = row.get(3)?;
            let roll: Option<i64> = row.get(4)?;
            let qr = format!("ADM|{}|{}|VERIFIED", exam_id, code);
            Ok(AdmitCardStudentItem {
                student_id: id,
                student_code: code,
                student_name: name,
                batch_name: batch,
                roll_number: roll.map(|r| r.to_string()),
                photo_data_uri: None,
                qr_verification_payload: qr,
                subjects: subjects.clone(),
            })
        })?;

        for sr in s_rows {
            resolved_students.push(sr?);
        }
    }

    let center_instructions = vec![
        "Candidates must arrive at the examination room 15 minutes before start time.".to_string(),
        "Digital watches, programmable calculators, and smartphones are strictly prohibited."
            .to_string(),
        "Admit card must be kept on the desk throughout the duration of each exam.".to_string(),
        "পরীক্ষার হলে কোনো প্রকার অসদুপায় অবলম্বন করলে পরীক্ষা বাতিল বলে গণ্য হবে।".to_string(),
    ];

    Ok(AdmitCardBatchData {
        exam_id: exam_id.to_string(),
        exam_title,
        academic_session: session_name,
        center_instructions,
        students: resolved_students,
    })
}

pub fn get_batch_report_cards(
    conn: &Connection,
    exam_id: &str,
    student_ids: &[String],
) -> Result<Vec<StudentReportCardData>, rusqlite::Error> {
    let mut cards = Vec::new();
    for sid in student_ids {
        if let Ok(rc) = crate::exam::get_student_report_card(conn, exam_id, sid) {
            cards.push(rc);
        }
    }
    Ok(cards)
}

pub fn get_batch_due_statements(
    conn: &Connection,
    student_ids: &[String],
) -> Result<Vec<DueStatementData>, rusqlite::Error> {
    let mut statements = Vec::new();
    for sid in student_ids {
        if let Ok(stmt) = crate::ledger::get_due_statement(conn, sid, None) {
            statements.push(stmt);
        }
    }
    Ok(statements)
}
