use crate::db;
use crate::phone::{self, PhoneConflict, PhoneError};
use crate::vault::DocumentSummary;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StudentsError {
    #[error("Student code '{0}' already exists")]
    StudentCodeExists(String),
    #[error("Invalid phone number: {0}")]
    InvalidPhone(String),
    #[error("Phone conflict warning for {0}")]
    PhoneConflictWarning(PhoneConflict),
    #[error("A student record must have exactly one primary guardian")]
    PrimaryGuardianMissing,
    #[error("A student record cannot have multiple primary guardians")]
    MultiplePrimaryGuardians,
    #[error("At least one guardian must be designated to receive SMS")]
    SmsGuardianMissing,
    #[error("Student with ID '{0}' not found")]
    StudentNotFound(String),
    #[error("Guardian with ID '{0}' not found")]
    GuardianNotFound(String),
    #[error("Concurrency conflict: record was modified by another user (expected version {expected}, found {actual})")]
    ConcurrencyConflict { expected: u32, actual: u32 },
    #[error("Invalid status transition from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("Database error: {0}")]
    Sql(String),
}

impl From<rusqlite::Error> for StudentsError {
    fn from(err: rusqlite::Error) -> Self {
        StudentsError::Sql(err.to_string())
    }
}

impl From<PhoneError> for StudentsError {
    fn from(err: PhoneError) -> Self {
        StudentsError::InvalidPhone(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// DTOs & Domain Models
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateGuardianInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub relationship: String,
    pub occupation: Option<String>,
    pub monthly_income_poisha: Option<i64>,
    pub nid: Option<String>,
    pub phone: String,
    pub phone2: Option<String>,
    pub whatsapp_number: Option<String>,
    pub email: Option<String>,
    pub present_address: Option<String>,
    pub permanent_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateGuardianInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub relationship: String,
    pub occupation: Option<String>,
    pub monthly_income_poisha: Option<i64>,
    pub nid: Option<String>,
    pub phone: String,
    pub phone2: Option<String>,
    pub whatsapp_number: Option<String>,
    pub email: Option<String>,
    pub present_address: Option<String>,
    pub permanent_address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GuardianLinkInput {
    pub guardian_id: Option<String>,
    pub is_primary: bool,
    pub receives_sms: bool,
    pub relationship: Option<String>,
    pub new_guardian: Option<CreateGuardianInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateStudentInput {
    pub student_code: Option<String>,
    pub name: String,
    pub name_bn: Option<String>,
    pub gender: String, // 'male', 'female', 'other'
    pub dob: String,
    pub blood_group: Option<String>,
    pub religion: Option<String>,
    pub nid_or_birth_cert: Option<String>,
    pub phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub present_address: String,
    pub permanent_address: Option<String>,
    pub district: Option<String>,
    pub thana: Option<String>,
    pub previous_school: Option<String>,
    pub current_school_roll: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub photo_vault_id: Option<String>,
    pub admission_date: Option<String>,
    pub notes: Option<String>,
    pub guardians: Vec<GuardianLinkInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateStudentInput {
    pub name: String,
    pub name_bn: Option<String>,
    pub gender: String,
    pub dob: String,
    pub blood_group: Option<String>,
    pub religion: Option<String>,
    pub nid_or_birth_cert: Option<String>,
    pub phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub present_address: String,
    pub permanent_address: Option<String>,
    pub district: Option<String>,
    pub thana: Option<String>,
    pub previous_school: Option<String>,
    pub current_school_roll: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub photo_vault_id: Option<String>,
    pub admission_date: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub guardians: Option<Vec<GuardianLinkInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct GuardianDetail {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub relationship: String,
    pub occupation: Option<String>,
    pub monthly_income_poisha: Option<i64>,
    pub nid: Option<String>,
    pub phone_norm: String,
    pub phone2_norm: Option<String>,
    pub whatsapp_number: Option<String>,
    pub email: Option<String>,
    pub present_address: Option<String>,
    pub permanent_address: Option<String>,
    pub notes: Option<String>,
    pub is_primary: bool,
    pub receives_sms: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct StudentDetail {
    pub id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub gender: String,
    pub dob: String,
    pub blood_group: Option<String>,
    pub religion: Option<String>,
    pub nid_or_birth_cert: Option<String>,
    pub phone_norm: String,
    pub emergency_phone_norm: Option<String>,
    pub email: Option<String>,
    pub present_address: String,
    pub permanent_address: Option<String>,
    pub district: Option<String>,
    pub thana: Option<String>,
    pub previous_school: Option<String>,
    pub current_school_roll: Option<String>,
    pub class_level: Option<String>,
    pub group_major: Option<String>,
    pub photo_vault_id: Option<String>,
    pub status: String,
    pub admission_date: String,
    pub notes: Option<String>,
    pub row_version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub guardians: Vec<GuardianDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct StudentSummary {
    pub id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub gender: String,
    pub class_level: Option<String>,
    pub current_school_roll: Option<String>,
    pub phone_norm: String,
    pub emergency_phone_norm: Option<String>,
    pub status: String,
    pub photo_vault_id: Option<String>,
    pub admission_date: String,
    pub row_version: u32,
    pub guardian_names: Option<String>,
    pub guardian_phones: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct StudentSearchHit {
    pub id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub gender: String,
    pub class_level: Option<String>,
    pub current_school_roll: Option<String>,
    pub phone_norm: String,
    pub status: String,
    pub photo_vault_id: Option<String>,
    pub guardian_names: Option<String>,
    pub guardian_phones: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct GuardianSearchHit {
    pub id: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub relationship: String,
    pub phone_norm: String,
    pub email: Option<String>,
    pub present_address: Option<String>,
    pub student_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStudentsPayload {
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListStudentsResponse {
    pub items: Vec<StudentSummary>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchStudentsResponse {
    pub matches: Vec<StudentSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchGuardiansResponse {
    pub matches: Vec<GuardianSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetStudentResponse {
    pub student: StudentDetail,
    pub siblings: Vec<StudentSummary>,
    pub documents: Vec<DocumentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetGuardianResponse {
    pub guardian: GuardianDetail,
    pub siblings: Vec<StudentSummary>,
}

// -----------------------------------------------------------------------------
// Sequence & Student Code Generation (AC-1)
// -----------------------------------------------------------------------------

pub fn generate_next_student_code(
    conn: &Connection,
    academic_year: Option<&str>,
) -> Result<String, StudentsError> {
    let year = if let Some(y) = academic_year {
        y.to_string()
    } else {
        // Resolve current session or year
        let session_opt: Option<String> = conn
            .query_row(
                "SELECT name FROM academic_sessions WHERE is_current = 1 LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        session_opt.unwrap_or_else(|| {
            let now = db::now_utc_iso();
            now[0..4].to_string()
        })
    };

    let seq_name = format!("student:{}", year);
    let pfx = format!("S{}-", year);

    // Run within savepoint/immediate
    conn.execute(
        "INSERT INTO sequences (name, prefix, next_value)
         VALUES (?1, ?2, 1)
         ON CONFLICT(name) DO NOTHING",
        rusqlite::params![seq_name, pfx],
    )?;

    let (db_pfx, val): (String, i64) = conn.query_row(
        "SELECT prefix, next_value FROM sequences WHERE name = ?1",
        rusqlite::params![seq_name],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    conn.execute(
        "UPDATE sequences SET next_value = next_value + 1 WHERE name = ?1",
        rusqlite::params![seq_name],
    )?;

    let effective_prefix = if db_pfx.is_empty() { pfx } else { db_pfx };
    Ok(format!("{}{:04}", effective_prefix, val))
}

// -----------------------------------------------------------------------------
// Guardian Repository & Operations (AC-2, AC-3)
// -----------------------------------------------------------------------------

pub fn create_guardian(
    conn: &Connection,
    input: &CreateGuardianInput,
    now: &str,
) -> Result<String, StudentsError> {
    let phone_norm = phone::normalize_phone(&input.phone)?;
    let phone2_norm = if let Some(p2) = &input.phone2 {
        if !p2.trim().is_empty() {
            Some(phone::normalize_phone(p2)?)
        } else {
            None
        }
    } else {
        None
    };

    let id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO guardians (
            id, name, name_bn, relationship, occupation, monthly_income_poisha,
            nid, phone_norm, phone2_norm, whatsapp_number, email, present_address,
            permanent_address, notes, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
        rusqlite::params![
            id,
            input.name.trim(),
            input.name_bn.as_deref().map(str::trim),
            input.relationship.trim().to_lowercase(),
            input.occupation.as_deref().map(str::trim),
            input.monthly_income_poisha,
            input.nid.as_deref().map(str::trim),
            phone_norm,
            phone2_norm,
            input.whatsapp_number.as_deref().map(str::trim),
            input.email.as_deref().map(str::trim),
            input.present_address.as_deref().map(str::trim),
            input.permanent_address.as_deref().map(str::trim),
            input.notes.as_deref().map(str::trim),
            now,
        ],
    )?;

    Ok(id)
}

pub fn search_guardians(
    conn: &Connection,
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<GuardianSearchHit>, StudentsError> {
    let clean_query = query.trim();
    if clean_query.is_empty() {
        return Ok(Vec::new());
    }

    let lim = limit.unwrap_or(20).min(50);
    let pattern = format!("%{}%", clean_query);

    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.name_bn, g.relationship, g.phone_norm, g.email, g.present_address,
                (SELECT count(*) FROM student_guardians sg WHERE sg.guardian_id = g.id) as student_count
         FROM guardians g
         WHERE g.name LIKE ?1 OR g.name_bn LIKE ?1 OR g.phone_norm LIKE ?1 OR g.nid LIKE ?1
         ORDER BY student_count DESC, g.name ASC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![pattern, lim], |row| {
        Ok(GuardianSearchHit {
            id: row.get(0)?,
            name: row.get(1)?,
            name_bn: row.get(2)?,
            relationship: row.get(3)?,
            phone_norm: row.get(4)?,
            email: row.get(5)?,
            present_address: row.get(6)?,
            student_count: row.get(7)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

pub fn get_guardian(
    conn: &Connection,
    guardian_id: &str,
) -> Result<GetGuardianResponse, StudentsError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, name_bn, relationship, occupation, monthly_income_poisha,
                nid, phone_norm, phone2_norm, whatsapp_number, email, present_address,
                permanent_address, notes, created_at, updated_at
         FROM guardians WHERE id = ?1 LIMIT 1",
    )?;

    let mut rows = stmt.query(rusqlite::params![guardian_id])?;
    let guardian = if let Some(r) = rows.next()? {
        GuardianDetail {
            id: r.get(0)?,
            name: r.get(1)?,
            name_bn: r.get(2)?,
            relationship: r.get(3)?,
            occupation: r.get(4)?,
            monthly_income_poisha: r.get(5)?,
            nid: r.get(6)?,
            phone_norm: r.get(7)?,
            phone2_norm: r.get(8)?,
            whatsapp_number: r.get(9)?,
            email: r.get(10)?,
            present_address: r.get(11)?,
            permanent_address: r.get(12)?,
            notes: r.get(13)?,
            is_primary: false,
            receives_sms: false,
            created_at: r.get(14)?,
            updated_at: r.get(15)?,
        }
    } else {
        return Err(StudentsError::GuardianNotFound(guardian_id.to_string()));
    };

    let siblings = get_guardian_linked_students(conn, guardian_id, None)?;

    Ok(GetGuardianResponse { guardian, siblings })
}

pub fn get_guardian_linked_students(
    conn: &Connection,
    guardian_id: &str,
    exclude_student_id: Option<&str>,
) -> Result<Vec<StudentSummary>, StudentsError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                s.current_school_roll, s.phone_norm, s.emergency_phone_norm, s.status,
                s.photo_vault_id, s.admission_date, s.row_version
         FROM students s
         JOIN student_guardians sg ON sg.student_id = s.id
         WHERE sg.guardian_id = ?1 AND (?2 IS NULL OR s.id != ?2)
         ORDER BY s.student_code ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![guardian_id, exclude_student_id], |r| {
        Ok(StudentSummary {
            id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            gender: r.get(4)?,
            class_level: r.get(5)?,
            current_school_roll: r.get(6)?,
            phone_norm: r.get(7)?,
            emergency_phone_norm: r.get(8)?,
            status: r.get(9)?,
            photo_vault_id: r.get(10)?,
            admission_date: r.get(11)?,
            row_version: r.get(12)?,
            guardian_names: None,
            guardian_phones: None,
        })
    })?;

    let mut students = Vec::new();
    for s in rows {
        students.push(s?);
    }
    Ok(students)
}

pub fn update_guardian(
    conn: &Connection,
    guardian_id: &str,
    input: &UpdateGuardianInput,
    split_from_siblings: bool,
    student_id: Option<&str>,
    now: &str,
) -> Result<GuardianDetail, StudentsError> {
    let phone_norm = phone::normalize_phone(&input.phone)?;
    let phone2_norm = if let Some(p2) = &input.phone2 {
        if !p2.trim().is_empty() {
            Some(phone::normalize_phone(p2)?)
        } else {
            None
        }
    } else {
        None
    };

    if let (true, Some(sid)) = (split_from_siblings, student_id) {
        // Create a separate new guardian record
        let new_id = Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO guardians (
                id, name, name_bn, relationship, occupation, monthly_income_poisha,
                nid, phone_norm, phone2_norm, whatsapp_number, email, present_address,
                permanent_address, notes, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
            rusqlite::params![
                new_id,
                input.name.trim(),
                input.name_bn.as_deref().map(str::trim),
                input.relationship.trim(),
                input.occupation.as_deref().map(str::trim),
                input.monthly_income_poisha,
                input.nid.as_deref().map(str::trim),
                phone_norm,
                phone2_norm,
                input.whatsapp_number.as_deref().map(str::trim),
                input.email.as_deref().map(str::trim),
                input.present_address.as_deref().map(str::trim),
                input.permanent_address.as_deref().map(str::trim),
                input.notes.as_deref().map(str::trim),
                now,
            ],
        )?;

        // Re-point this student's guardian junction to new_id
        conn.execute(
            "UPDATE student_guardians SET guardian_id = ?1 WHERE student_id = ?2 AND guardian_id = ?3",
            rusqlite::params![new_id, sid, guardian_id],
        )?;

        return Ok(GuardianDetail {
            id: new_id,
            name: input.name.trim().to_string(),
            name_bn: input.name_bn.as_deref().map(str::trim).map(String::from),
            relationship: input.relationship.trim().to_lowercase(),
            occupation: input.occupation.as_deref().map(str::trim).map(String::from),
            monthly_income_poisha: input.monthly_income_poisha,
            nid: input.nid.as_deref().map(str::trim).map(String::from),
            phone_norm,
            phone2_norm,
            whatsapp_number: input
                .whatsapp_number
                .as_deref()
                .map(str::trim)
                .map(String::from),
            email: input.email.as_deref().map(str::trim).map(String::from),
            present_address: input
                .present_address
                .as_deref()
                .map(str::trim)
                .map(String::from),
            permanent_address: input
                .permanent_address
                .as_deref()
                .map(str::trim)
                .map(String::from),
            notes: input.notes.as_deref().map(str::trim).map(String::from),
            is_primary: false,
            receives_sms: false,
            created_at: now.to_string(),
            updated_at: now.to_string(),
        });
    }

    // Direct in-place update (updates for all siblings)
    let updated = conn.execute(
        "UPDATE guardians SET
            name = ?1, name_bn = ?2, relationship = ?3, occupation = ?4,
            monthly_income_poisha = ?5, nid = ?6, phone_norm = ?7, phone2_norm = ?8,
            whatsapp_number = ?9, email = ?10, present_address = ?11,
            permanent_address = ?12, notes = ?13, updated_at = ?14
         WHERE id = ?15",
        rusqlite::params![
            input.name.trim(),
            input.name_bn.as_deref().map(str::trim),
            input.relationship.trim().to_lowercase(),
            input.occupation.as_deref().map(str::trim),
            input.monthly_income_poisha,
            input.nid.as_deref().map(str::trim),
            phone_norm,
            phone2_norm,
            input.whatsapp_number.as_deref().map(str::trim),
            input.email.as_deref().map(str::trim),
            input.present_address.as_deref().map(str::trim),
            input.permanent_address.as_deref().map(str::trim),
            input.notes.as_deref().map(str::trim),
            now,
            guardian_id,
        ],
    )?;

    if updated == 0 {
        return Err(StudentsError::GuardianNotFound(guardian_id.to_string()));
    }

    let mut g = get_guardian(conn, guardian_id)?.guardian;
    g.updated_at = now.to_string();
    Ok(g)
}

// -----------------------------------------------------------------------------
// Student Repository & Operations (AC-1 through AC-10)
// -----------------------------------------------------------------------------

pub fn create_student(
    conn: &Connection,
    input: &CreateStudentInput,
    allow_duplicate_phone: bool,
    user_id: Option<&str>,
    now: &str,
) -> Result<StudentDetail, StudentsError> {
    // 1. Validate guardians
    if input.guardians.is_empty() {
        return Err(StudentsError::PrimaryGuardianMissing);
    }

    let primary_count = input.guardians.iter().filter(|g| g.is_primary).count();
    if primary_count == 0 {
        return Err(StudentsError::PrimaryGuardianMissing);
    }
    if primary_count > 1 {
        return Err(StudentsError::MultiplePrimaryGuardians);
    }

    let sms_count = input.guardians.iter().filter(|g| g.receives_sms).count();
    if sms_count == 0 {
        return Err(StudentsError::SmsGuardianMissing);
    }

    // 2. Validate and normalize student phones
    let student_phone_norm = phone::normalize_phone(&input.phone)?;
    let emergency_phone_norm = if let Some(e) = &input.emergency_phone {
        if !e.trim().is_empty() {
            Some(phone::normalize_phone(e)?)
        } else {
            None
        }
    } else {
        None
    };

    // 3. Conflict checks on phone numbers
    if !allow_duplicate_phone {
        if let Some(conflict) = phone::check_phone_conflict(conn, &student_phone_norm, None, None)?
        {
            return Err(StudentsError::PhoneConflictWarning(conflict));
        }
    }

    // 4. Resolve or generate student code
    let student_code = if let Some(manual_code) = &input.student_code {
        let clean = manual_code.trim();
        if clean.is_empty() {
            generate_next_student_code(conn, None)?
        } else {
            // Check uniqueness
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM students WHERE student_code = ?1",
                rusqlite::params![clean],
                |r| r.get(0),
            )?;
            if count > 0 {
                return Err(StudentsError::StudentCodeExists(clean.to_string()));
            }
            clean.to_string()
        }
    } else {
        generate_next_student_code(conn, None)?
    };

    // 5. Insert student
    let student_id = Uuid::now_v7().to_string();
    let admission_date = input
        .admission_date
        .as_deref()
        .map(str::trim)
        .unwrap_or(&now[0..10]);

    conn.execute(
        "INSERT INTO students (
            id, student_code, name, name_bn, gender, dob, blood_group, religion,
            nid_or_birth_cert, phone_norm, emergency_phone_norm, email, present_address,
            permanent_address, district, thana, previous_school, current_school_roll,
            class_level, group_major, photo_vault_id, status, admission_date, notes,
            row_version, created_at, created_by, updated_at, updated_by
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            ?9, ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17, ?18,
            ?19, ?20, ?21, 'active', ?22, ?23,
            1, ?24, ?25, ?24, ?25
        )",
        rusqlite::params![
            student_id,
            student_code,
            input.name.trim(),
            input.name_bn.as_deref().map(str::trim),
            input.gender.trim(),
            input.dob.trim(),
            input.blood_group.as_deref().map(str::trim),
            input.religion.as_deref().map(str::trim),
            input.nid_or_birth_cert.as_deref().map(str::trim),
            student_phone_norm,
            emergency_phone_norm,
            input.email.as_deref().map(str::trim),
            input.present_address.trim(),
            input.permanent_address.as_deref().map(str::trim),
            input.district.as_deref().map(str::trim),
            input.thana.as_deref().map(str::trim),
            input.previous_school.as_deref().map(str::trim),
            input.current_school_roll.as_deref().map(str::trim),
            input.class_level.as_deref().map(str::trim),
            input.group_major.as_deref().map(str::trim),
            input.photo_vault_id.as_deref().map(str::trim),
            admission_date,
            input.notes.as_deref().map(str::trim),
            now,
            user_id,
        ],
    )?;

    // 6. Process guardians & links
    for g_link in &input.guardians {
        let guardian_id = if let Some(existing_id) = &g_link.guardian_id {
            existing_id.clone()
        } else if let Some(new_g) = &g_link.new_guardian {
            if !allow_duplicate_phone {
                let norm = phone::normalize_phone(&new_g.phone)?;
                if let Some(conflict) =
                    phone::check_phone_conflict(conn, &norm, Some(&student_id), None)?
                {
                    return Err(StudentsError::PhoneConflictWarning(conflict));
                }
            }
            create_guardian(conn, new_g, now)?
        } else {
            return Err(StudentsError::PrimaryGuardianMissing);
        };

        conn.execute(
            "INSERT INTO student_guardians (student_id, guardian_id, is_primary, receives_sms)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(student_id, guardian_id) DO UPDATE SET
                is_primary = ?3, receives_sms = ?4",
            rusqlite::params![
                student_id,
                guardian_id,
                if g_link.is_primary { 1 } else { 0 },
                if g_link.receives_sms { 1 } else { 0 },
            ],
        )?;
    }

    get_student_detail(conn, &student_id)
}

pub fn update_student(
    conn: &Connection,
    id: &str,
    expected_version: u32,
    input: &UpdateStudentInput,
    allow_duplicate_phone: bool,
    user_id: Option<&str>,
    now: &str,
) -> Result<StudentDetail, StudentsError> {
    // 1. Check existing record & optimistic concurrency
    let current_version: u32 = match conn.query_row(
        "SELECT row_version FROM students WHERE id = ?1",
        rusqlite::params![id],
        |r| r.get(0),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(StudentsError::StudentNotFound(id.to_string()))
        }
        Err(e) => return Err(StudentsError::Sql(e.to_string())),
    };

    if current_version != expected_version {
        return Err(StudentsError::ConcurrencyConflict {
            expected: expected_version,
            actual: current_version,
        });
    }

    // 2. Normalize and check phone numbers
    let student_phone_norm = phone::normalize_phone(&input.phone)?;
    let emergency_phone_norm = if let Some(e) = &input.emergency_phone {
        if !e.trim().is_empty() {
            Some(phone::normalize_phone(e)?)
        } else {
            None
        }
    } else {
        None
    };

    if !allow_duplicate_phone {
        if let Some(conflict) =
            phone::check_phone_conflict(conn, &student_phone_norm, Some(id), None)?
        {
            return Err(StudentsError::PhoneConflictWarning(conflict));
        }
    }

    let status_val = input.status.as_deref().unwrap_or("active");

    // 3. Update student table
    let new_version = current_version + 1;
    conn.execute(
        "UPDATE students SET
            name = ?1, name_bn = ?2, gender = ?3, dob = ?4, blood_group = ?5,
            religion = ?6, nid_or_birth_cert = ?7, phone_norm = ?8, emergency_phone_norm = ?9,
            email = ?10, present_address = ?11, permanent_address = ?12, district = ?13,
            thana = ?14, previous_school = ?15, current_school_roll = ?16, class_level = ?17,
            group_major = ?18, photo_vault_id = ?19, status = ?20, notes = ?21,
            row_version = ?22, updated_at = ?23, updated_by = ?24
         WHERE id = ?25",
        rusqlite::params![
            input.name.trim(),
            input.name_bn.as_deref().map(str::trim),
            input.gender.trim(),
            input.dob.trim(),
            input.blood_group.as_deref().map(str::trim),
            input.religion.as_deref().map(str::trim),
            input.nid_or_birth_cert.as_deref().map(str::trim),
            student_phone_norm,
            emergency_phone_norm,
            input.email.as_deref().map(str::trim),
            input.present_address.trim(),
            input.permanent_address.as_deref().map(str::trim),
            input.district.as_deref().map(str::trim),
            input.thana.as_deref().map(str::trim),
            input.previous_school.as_deref().map(str::trim),
            input.current_school_roll.as_deref().map(str::trim),
            input.class_level.as_deref().map(str::trim),
            input.group_major.as_deref().map(str::trim),
            input.photo_vault_id.as_deref().map(str::trim),
            status_val,
            input.notes.as_deref().map(str::trim),
            new_version,
            now,
            user_id,
            id,
        ],
    )?;

    // 4. Update guardians if provided
    if let Some(guardians) = &input.guardians {
        if !guardians.is_empty() {
            let primary_count = guardians.iter().filter(|g| g.is_primary).count();
            if primary_count == 0 {
                return Err(StudentsError::PrimaryGuardianMissing);
            }
            if primary_count > 1 {
                return Err(StudentsError::MultiplePrimaryGuardians);
            }

            let sms_count = guardians.iter().filter(|g| g.receives_sms).count();
            if sms_count == 0 {
                return Err(StudentsError::SmsGuardianMissing);
            }

            conn.execute(
                "DELETE FROM student_guardians WHERE student_id = ?1",
                rusqlite::params![id],
            )?;

            for g_link in guardians {
                let guardian_id = if let Some(gid) = &g_link.guardian_id {
                    gid.clone()
                } else if let Some(new_g) = &g_link.new_guardian {
                    create_guardian(conn, new_g, now)?
                } else {
                    return Err(StudentsError::PrimaryGuardianMissing);
                };

                conn.execute(
                    "INSERT INTO student_guardians (student_id, guardian_id, is_primary, receives_sms)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        id,
                        guardian_id,
                        if g_link.is_primary { 1 } else { 0 },
                        if g_link.receives_sms { 1 } else { 0 },
                    ],
                )?;
            }
        }
    }

    get_student_detail(conn, id)
}

pub fn archive_student(
    conn: &Connection,
    id: &str,
    reason: &str,
    expected_version: u32,
    user_id: Option<&str>,
    now: &str,
) -> Result<StudentDetail, StudentsError> {
    let current_version: u32 = match conn.query_row(
        "SELECT row_version FROM students WHERE id = ?1",
        rusqlite::params![id],
        |r| r.get(0),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(StudentsError::StudentNotFound(id.to_string()))
        }
        Err(e) => return Err(StudentsError::Sql(e.to_string())),
    };

    if current_version != expected_version {
        return Err(StudentsError::ConcurrencyConflict {
            expected: expected_version,
            actual: current_version,
        });
    }

    let note_append = format!("[Archived {}: {}]", &now[0..10], reason.trim());
    conn.execute(
        "UPDATE students SET
            status = 'archived',
            notes = CASE WHEN notes IS NULL OR notes = '' THEN ?1 ELSE notes || char(10) || ?1 END,
            row_version = row_version + 1,
            updated_at = ?2,
            updated_by = ?3
         WHERE id = ?4",
        rusqlite::params![note_append, now, user_id, id],
    )?;

    get_student_detail(conn, id)
}

pub fn get_student_detail(conn: &Connection, id: &str) -> Result<StudentDetail, StudentsError> {
    let mut stmt = conn.prepare(
        "SELECT id, student_code, name, name_bn, gender, dob, blood_group, religion,
                nid_or_birth_cert, phone_norm, emergency_phone_norm, email, present_address,
                permanent_address, district, thana, previous_school, current_school_roll,
                class_level, group_major, photo_vault_id, status, admission_date, notes,
                row_version, created_at, updated_at
         FROM students WHERE id = ?1 LIMIT 1",
    )?;

    let mut rows = stmt.query(rusqlite::params![id])?;
    let mut detail = if let Some(r) = rows.next()? {
        StudentDetail {
            id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            gender: r.get(4)?,
            dob: r.get(5)?,
            blood_group: r.get(6)?,
            religion: r.get(7)?,
            nid_or_birth_cert: r.get(8)?,
            phone_norm: r.get(9)?,
            emergency_phone_norm: r.get(10)?,
            email: r.get(11)?,
            present_address: r.get(12)?,
            permanent_address: r.get(13)?,
            district: r.get(14)?,
            thana: r.get(15)?,
            previous_school: r.get(16)?,
            current_school_roll: r.get(17)?,
            class_level: r.get(18)?,
            group_major: r.get(19)?,
            photo_vault_id: r.get(20)?,
            status: r.get(21)?,
            admission_date: r.get(22)?,
            notes: r.get(23)?,
            row_version: r.get(24)?,
            created_at: r.get(25)?,
            updated_at: r.get(26)?,
            guardians: Vec::new(),
        }
    } else {
        return Err(StudentsError::StudentNotFound(id.to_string()));
    };

    // Load guardians
    let mut g_stmt = conn.prepare(
        "SELECT g.id, g.name, g.name_bn, g.relationship, g.occupation, g.monthly_income_poisha,
                g.nid, g.phone_norm, g.phone2_norm, g.whatsapp_number, g.email, g.present_address,
                g.permanent_address, g.notes, sg.is_primary, sg.receives_sms, g.created_at, g.updated_at
         FROM guardians g
         JOIN student_guardians sg ON sg.guardian_id = g.id
         WHERE sg.student_id = ?1
         ORDER BY sg.is_primary DESC, g.name ASC",
    )?;

    let g_rows = g_stmt.query_map(rusqlite::params![id], |r| {
        Ok(GuardianDetail {
            id: r.get(0)?,
            name: r.get(1)?,
            name_bn: r.get(2)?,
            relationship: r.get(3)?,
            occupation: r.get(4)?,
            monthly_income_poisha: r.get(5)?,
            nid: r.get(6)?,
            phone_norm: r.get(7)?,
            phone2_norm: r.get(8)?,
            whatsapp_number: r.get(9)?,
            email: r.get(10)?,
            present_address: r.get(11)?,
            permanent_address: r.get(12)?,
            notes: r.get(13)?,
            is_primary: r.get::<_, i32>(14)? == 1,
            receives_sms: r.get::<_, i32>(15)? == 1,
            created_at: r.get(16)?,
            updated_at: r.get(17)?,
        })
    })?;

    for g in g_rows {
        detail.guardians.push(g?);
    }

    Ok(detail)
}

pub fn get_student_view(conn: &Connection, id: &str) -> Result<GetStudentResponse, StudentsError> {
    let student = get_student_detail(conn, id)?;

    // Get siblings linked via any shared guardian
    let mut sibling_ids_stmt = conn.prepare(
        "SELECT DISTINCT sg.student_id
         FROM student_guardians sg
         WHERE sg.guardian_id IN (SELECT guardian_id FROM student_guardians WHERE student_id = ?1)
           AND sg.student_id != ?1",
    )?;

    let s_rows = sibling_ids_stmt.query_map(rusqlite::params![id], |r| r.get::<_, String>(0))?;
    let mut siblings = Vec::new();
    for s_id in s_rows {
        let sid = s_id?;
        if let Ok(s) = get_student_summary(conn, &sid) {
            siblings.push(s);
        }
    }

    // Get documents
    let mut d_stmt = conn.prepare(
        "SELECT id, owner_type, owner_id, kind, original_name, mime, size, sha256, created_at
         FROM documents WHERE owner_type = 'student' AND owner_id = ?1
         ORDER BY created_at DESC",
    )?;

    let d_rows = d_stmt.query_map(rusqlite::params![id], |r| {
        Ok(DocumentSummary {
            id: r.get(0)?,
            owner_type: r.get(1)?,
            owner_id: r.get(2)?,
            kind: r.get(3)?,
            original_name: r.get(4)?,
            mime: r.get(5)?,
            size: r.get(6)?,
            sha256: r.get(7)?,
            created_at: r.get(8)?,
        })
    })?;

    let mut documents = Vec::new();
    for d in d_rows {
        documents.push(d?);
    }

    Ok(GetStudentResponse {
        student,
        siblings,
        documents,
    })
}

pub fn get_student_summary(conn: &Connection, id: &str) -> Result<StudentSummary, StudentsError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                s.current_school_roll, s.phone_norm, s.emergency_phone_norm, s.status,
                s.photo_vault_id, s.admission_date, s.row_version,
                (SELECT group_concat(g.name, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id),
                (SELECT group_concat(g.phone_norm, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id)
         FROM students s WHERE s.id = ?1 LIMIT 1",
    )?;

    stmt.query_row(rusqlite::params![id], |r| {
        Ok(StudentSummary {
            id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            gender: r.get(4)?,
            class_level: r.get(5)?,
            current_school_roll: r.get(6)?,
            phone_norm: r.get(7)?,
            emergency_phone_norm: r.get(8)?,
            status: r.get(9)?,
            photo_vault_id: r.get(10)?,
            admission_date: r.get(11)?,
            row_version: r.get(12)?,
            guardian_names: r.get(13)?,
            guardian_phones: r.get(14)?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => StudentsError::StudentNotFound(id.to_string()),
        other => StudentsError::Sql(other.to_string()),
    })
}

pub fn list_students(
    conn: &Connection,
    payload: &ListStudentsPayload,
) -> Result<ListStudentsResponse, StudentsError> {
    let limit = payload.limit.unwrap_or(50).min(100);
    let offset = payload.offset.unwrap_or(0);

    let status_filter = payload.status.as_deref();

    let total: u32 = match status_filter {
        Some(s) if !s.is_empty() && s != "all" => conn.query_row(
            "SELECT count(*) FROM students WHERE status = ?1",
            rusqlite::params![s],
            |r| r.get(0),
        )?,
        _ => conn.query_row("SELECT count(*) FROM students", [], |r| r.get(0))?,
    };

    let sql = match status_filter {
        Some(s) if !s.is_empty() && s != "all" => {
            "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                    s.current_school_roll, s.phone_norm, s.emergency_phone_norm, s.status,
                    s.photo_vault_id, s.admission_date, s.row_version,
                    (SELECT group_concat(g.name, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id),
                    (SELECT group_concat(g.phone_norm, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id)
             FROM students s
             WHERE s.status = ?1
             ORDER BY s.student_code DESC
             LIMIT ?2 OFFSET ?3"
        }
        _ => {
            "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                    s.current_school_roll, s.phone_norm, s.emergency_phone_norm, s.status,
                    s.photo_vault_id, s.admission_date, s.row_version,
                    (SELECT group_concat(g.name, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id),
                    (SELECT group_concat(g.phone_norm, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id)
             FROM students s
             ORDER BY s.student_code DESC
             LIMIT ?1 OFFSET ?2"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<StudentSummary> = match status_filter {
        Some(s) if !s.is_empty() && s != "all" => {
            let mapped = stmt.query_map(rusqlite::params![s, limit, offset], |r| {
                Ok(StudentSummary {
                    id: r.get(0)?,
                    student_code: r.get(1)?,
                    name: r.get(2)?,
                    name_bn: r.get(3)?,
                    gender: r.get(4)?,
                    class_level: r.get(5)?,
                    current_school_roll: r.get(6)?,
                    phone_norm: r.get(7)?,
                    emergency_phone_norm: r.get(8)?,
                    status: r.get(9)?,
                    photo_vault_id: r.get(10)?,
                    admission_date: r.get(11)?,
                    row_version: r.get(12)?,
                    guardian_names: r.get(13)?,
                    guardian_phones: r.get(14)?,
                })
            })?;
            mapped.filter_map(Result::ok).collect()
        }
        _ => {
            let mapped = stmt.query_map(rusqlite::params![limit, offset], |r| {
                Ok(StudentSummary {
                    id: r.get(0)?,
                    student_code: r.get(1)?,
                    name: r.get(2)?,
                    name_bn: r.get(3)?,
                    gender: r.get(4)?,
                    class_level: r.get(5)?,
                    current_school_roll: r.get(6)?,
                    phone_norm: r.get(7)?,
                    emergency_phone_norm: r.get(8)?,
                    status: r.get(9)?,
                    photo_vault_id: r.get(10)?,
                    admission_date: r.get(11)?,
                    row_version: r.get(12)?,
                    guardian_names: r.get(13)?,
                    guardian_phones: r.get(14)?,
                })
            })?;
            mapped.filter_map(Result::ok).collect()
        }
    };

    Ok(ListStudentsResponse { items: rows, total })
}

// -----------------------------------------------------------------------------
// FTS5 Trigram Substring Search (AC-6, AC-7)
// -----------------------------------------------------------------------------

pub fn search_students(
    conn: &Connection,
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<StudentSearchHit>, StudentsError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let lim = limit.unwrap_or(20).min(50);

    // AC-7: Client and server enforce 3-character threshold for trigram lookups,
    // falling back to exact / prefix code matching for short inputs.
    if trimmed.len() < 3 {
        let pattern = format!("{}%", trimmed);
        let mut stmt = conn.prepare(
            "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                    s.current_school_roll, s.phone_norm, s.status, s.photo_vault_id,
                    (SELECT group_concat(g.name, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id),
                    (SELECT group_concat(g.phone_norm, ', ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id)
             FROM students s
             WHERE s.student_code LIKE ?1 OR s.current_school_roll = ?2
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(rusqlite::params![pattern, trimmed, lim], |r| {
            Ok(StudentSearchHit {
                id: r.get(0)?,
                student_code: r.get(1)?,
                name: r.get(2)?,
                name_bn: r.get(3)?,
                gender: r.get(4)?,
                class_level: r.get(5)?,
                current_school_roll: r.get(6)?,
                phone_norm: r.get(7)?,
                status: r.get(8)?,
                photo_vault_id: r.get(9)?,
                guardian_names: r.get(10)?,
                guardian_phones: r.get(11)?,
            })
        })?;

        let mut hits = Vec::new();
        for r in rows {
            hits.push(r?);
        }
        return Ok(hits);
    }

    // FTS5 trigram query: escape double quotes in query and enclose in quotes
    let escaped_query = trimmed.replace('"', "\"\"");
    let match_query = format!("\"{}\"", escaped_query);

    let mut stmt = conn.prepare(
        "SELECT s.id, s.student_code, s.name, s.name_bn, s.gender, s.class_level,
                s.current_school_roll, s.phone_norm, s.status, s.photo_vault_id,
                fts.guardian_names, fts.guardian_phones
         FROM student_search_index fts
         JOIN students s ON s.rowid = fts.rowid
         WHERE student_search_index MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![match_query, lim], |r| {
        Ok(StudentSearchHit {
            id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            name_bn: r.get(3)?,
            gender: r.get(4)?,
            class_level: r.get(5)?,
            current_school_roll: r.get(6)?,
            phone_norm: r.get(7)?,
            status: r.get(8)?,
            photo_vault_id: r.get(9)?,
            guardian_names: r.get(10)?,
            guardian_phones: r.get(11)?,
        })
    })?;

    let mut hits = Vec::new();
    for r in rows {
        hits.push(r?);
    }

    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::tempdir;

    fn setup_test_db() -> (tempfile::TempDir, rusqlite::Connection) {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let conn = db::open_connection_at(&db_path).expect("open test db");
        db::apply_all_migrations(&conn).expect("apply migrations");
        (dir, conn)
    }

    #[test]
    fn test_create_student_happy_path() {
        let (_dir, conn) = setup_test_db();
        let now = "2026-09-21T12:00:00Z";

        let input = CreateStudentInput {
            student_code: None,
            name: "Rahim Uddin".to_string(),
            name_bn: Some("রহিম উদ্দিন".to_string()),
            gender: "male".to_string(),
            dob: "2010-05-15".to_string(),
            blood_group: Some("B+".to_string()),
            religion: Some("Islam".to_string()),
            nid_or_birth_cert: None,
            phone: "01711223344".to_string(),
            emergency_phone: Some("01811223344".to_string()),
            email: None,
            present_address: "Dhanmondi, Dhaka".to_string(),
            permanent_address: None,
            district: Some("Dhaka".to_string()),
            thana: Some("Dhanmondi".to_string()),
            previous_school: None,
            current_school_roll: Some("12".to_string()),
            class_level: Some("Class 10".to_string()),
            group_major: Some("Science".to_string()),
            photo_vault_id: None,
            admission_date: Some("2026-01-10".to_string()),
            notes: None,
            guardians: vec![GuardianLinkInput {
                guardian_id: None,
                is_primary: true,
                receives_sms: true,
                relationship: Some("father".to_string()),
                new_guardian: Some(CreateGuardianInput {
                    name: "Karim Uddin".to_string(),
                    name_bn: None,
                    relationship: "father".to_string(),
                    occupation: Some("Engineer".to_string()),
                    monthly_income_poisha: Some(8000000),
                    nid: None,
                    phone: "01911223344".to_string(),
                    phone2: None,
                    whatsapp_number: None,
                    email: None,
                    present_address: Some("Dhanmondi, Dhaka".to_string()),
                    permanent_address: None,
                    notes: None,
                }),
            }],
        };

        let student = create_student(&conn, &input, false, Some("usr_owner"), now).unwrap();
        assert!(student.student_code.starts_with("S2026-"));
        assert_eq!(student.name, "Rahim Uddin");
        assert_eq!(student.phone_norm, "8801711223344");
        assert_eq!(student.guardians.len(), 1);
        assert!(student.guardians[0].is_primary);
        assert!(student.guardians[0].receives_sms);

        // FTS5 Substring Search
        let hits = search_students(&conn, "Rahim", None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, student.id);

        let phone_hits = search_students(&conn, "0191", None).unwrap();
        assert_eq!(phone_hits.len(), 1);
        assert_eq!(phone_hits[0].id, student.id);
    }
}
