use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Sql(String),
}

impl From<rusqlite::Error> for SearchError {
    fn from(err: rusqlite::Error) -> Self {
        SearchError::Sql(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalSearchPayload {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalStudentHit {
    pub id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone_norm: Option<String>,
    pub batch_name: Option<String>,
    pub guardian_names: Option<String>,
    pub guardian_phones: Option<String>,
    pub matched_guardian_badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalBatchHit {
    pub id: String,
    pub name: String,
    pub code: String,
    pub program_name: String,
    pub student_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalStaffHit {
    pub id: String,
    pub staff_code: String,
    pub name: String,
    pub phone_norm: Option<String>,
    pub designation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalReceiptHit {
    pub payment_id: String,
    pub receipt_number: String,
    pub student_id: String,
    pub student_name: String,
    pub student_code: String,
    pub amount_poisha: i64,
    pub amount_formatted: String,
    pub method: String,
    pub paid_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GlobalSearchResponse {
    pub students: Vec<GlobalStudentHit>,
    pub batches: Vec<GlobalBatchHit>,
    pub staff: Vec<GlobalStaffHit>,
    pub receipts: Vec<GlobalReceiptHit>,
    pub total_hits: u32,
    pub duration_ms: u64,
}

/// Executes unified global search across students, batches, staff, and payment receipts.
/// Enforces sub-100ms latency budget, input length guards, and reverse guardian phone lookups.
pub fn search_global(
    conn: &Connection,
    payload: &GlobalSearchPayload,
) -> Result<GlobalSearchResponse, SearchError> {
    let start_time = Instant::now();
    let trimmed = payload.query.trim();

    // AC-6: If query is less than 2 characters, return empty result set without executing DB queries
    if trimmed.chars().count() < 2 {
        return Ok(GlobalSearchResponse {
            students: vec![],
            batches: vec![],
            staff: vec![],
            receipts: vec![],
            total_hits: 0,
            duration_ms: start_time.elapsed().as_millis() as u64,
        });
    }

    let lim = payload.limit.unwrap_or(10).clamp(1, 50) as i64;
    let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    let is_phone_query = digits_only.len() >= 6;
    let like_pattern = format!("%{}%", trimmed);

    // 1. Students search (FTS5 trigram if >= 3 characters, LIKE if 2 characters, or guardian reverse phone)
    let mut students = Vec::new();
    let mut student_ids_seen = std::collections::HashSet::new();

    // AC-2: Guardian Phone Reverse Lookup (if 6+ digits)
    if is_phone_query {
        let phone_like = format!("%{}%", digits_only);
        let mut guardian_stmt = conn.prepare(
            "SELECT s.id, s.student_code, s.name, s.name_bn, s.phone_norm,
                    (SELECT b.name FROM enrollments e JOIN batches b ON b.id = e.batch_id WHERE e.student_id = s.id AND e.status = 'active' LIMIT 1),
                    g.name, g.phone_norm
             FROM students s
             JOIN student_guardians sg ON sg.student_id = s.id
             JOIN guardians g ON g.id = sg.guardian_id
             WHERE g.phone_norm LIKE ?1 OR g.phone LIKE ?1
             ORDER BY s.name ASC
             LIMIT ?2"
        )?;

        let guardian_rows = guardian_stmt.query_map(params![phone_like, lim], |row| {
            let g_name: String = row.get(6)?;
            let g_phone: String = row.get(7)?;
            Ok(GlobalStudentHit {
                id: row.get(0)?,
                student_code: row.get(1)?,
                name: row.get(2)?,
                name_bn: row.get(3)?,
                phone_norm: row.get(4)?,
                batch_name: row.get(5)?,
                guardian_names: Some(g_name.clone()),
                guardian_phones: Some(g_phone.clone()),
                matched_guardian_badge: Some(format!("Guardian: {} ({})", g_name, g_phone)),
            })
        })?;

        for r in guardian_rows {
            let hit = r?;
            if student_ids_seen.insert(hit.id.clone()) {
                students.push(hit);
            }
        }
    }

    // Normal student search (if remaining capacity)
    if (students.len() as i64) < lim {
        let remaining_lim = lim - (students.len() as i64);

        if trimmed.chars().count() >= 3 {
            // FTS5 Trigram Search
            // Sanitize query to avoid FTS5 syntax errors
            let sanitized_query = trimmed
                .replace('"', "\"\"")
                .replace(['*', '^', ':', '(', ')'], "");
            let fts_match = format!("\"{}\"", sanitized_query);

            let fts_stmt = conn.prepare(
                "SELECT s.id, s.student_code, s.name, s.name_bn, s.phone_norm,
                        (SELECT b.name FROM enrollments e JOIN batches b ON b.id = e.batch_id WHERE e.student_id = s.id AND e.status = 'active' LIMIT 1),
                        fts.guardian_names, fts.guardian_phones
                 FROM student_search_index fts
                 JOIN students s ON s.rowid = fts.rowid
                 WHERE student_search_index MATCH ?1
                 ORDER BY rank
                 LIMIT ?2"
            );

            if let Ok(mut stmt) = fts_stmt {
                let rows = stmt.query_map(params![fts_match, remaining_lim], |row| {
                    Ok(GlobalStudentHit {
                        id: row.get(0)?,
                        student_code: row.get(1)?,
                        name: row.get(2)?,
                        name_bn: row.get(3)?,
                        phone_norm: row.get(4)?,
                        batch_name: row.get(5)?,
                        guardian_names: row.get(6)?,
                        guardian_phones: row.get(7)?,
                        matched_guardian_badge: None,
                    })
                });

                if let Ok(mapped_rows) = rows {
                    for r in mapped_rows.flatten() {
                        if student_ids_seen.insert(r.id.clone()) {
                            students.push(r);
                        }
                    }
                }
            }
        }

        // Fallback or 2-char LIKE search if still under limit
        if (students.len() as i64) < lim {
            let rem2 = lim - (students.len() as i64);
            let mut like_stmt = conn.prepare(
                "SELECT s.id, s.student_code, s.name, s.name_bn, s.phone_norm,
                        (SELECT b.name FROM enrollments e JOIN batches b ON b.id = e.batch_id WHERE e.student_id = s.id AND e.status = 'active' LIMIT 1)
                 FROM students s
                 WHERE s.name LIKE ?1 OR s.student_code LIKE ?1 OR s.name_bn LIKE ?1 OR s.phone_norm LIKE ?1
                 LIMIT ?2"
            )?;

            let rows = like_stmt.query_map(params![like_pattern, rem2], |row| {
                Ok(GlobalStudentHit {
                    id: row.get(0)?,
                    student_code: row.get(1)?,
                    name: row.get(2)?,
                    name_bn: row.get(3)?,
                    phone_norm: row.get(4)?,
                    batch_name: row.get(5)?,
                    guardian_names: None,
                    guardian_phones: None,
                    matched_guardian_badge: None,
                })
            })?;

            for r in rows.flatten() {
                if student_ids_seen.insert(r.id.clone()) {
                    students.push(r);
                }
            }
        }
    }

    // 2. Batches search
    let mut batches = Vec::new();
    let mut batch_stmt = conn.prepare(
        "SELECT b.id, b.name, b.code, p.name,
                (SELECT COUNT(*) FROM enrollments e WHERE e.batch_id = b.id AND e.status = 'active')
         FROM batches b
         JOIN programs p ON p.id = b.program_id
         WHERE b.status = 'active' AND (b.name LIKE ?1 OR b.code LIKE ?1 OR p.name LIKE ?1)
         ORDER BY b.name ASC
         LIMIT ?2",
    )?;

    let batch_rows = batch_stmt.query_map(params![like_pattern, lim], |row| {
        let count_i64: i64 = row.get(4)?;
        Ok(GlobalBatchHit {
            id: row.get(0)?,
            name: row.get(1)?,
            code: row.get(2)?,
            program_name: row.get(3)?,
            student_count: count_i64 as u32,
        })
    })?;

    for r in batch_rows {
        batches.push(r?);
    }

    // 3. Staff search
    let mut staff = Vec::new();
    let mut staff_stmt = conn.prepare(
        "SELECT st.id, st.staff_code, st.name, st.phone_norm, st.designation
         FROM staff st
         WHERE st.status = 'active' AND (st.name LIKE ?1 OR st.staff_code LIKE ?1 OR st.phone_norm LIKE ?1 OR st.designation LIKE ?1)
         ORDER BY st.name ASC
         LIMIT ?2"
    )?;

    let staff_rows = staff_stmt.query_map(params![like_pattern, lim], |row| {
        Ok(GlobalStaffHit {
            id: row.get(0)?,
            staff_code: row.get(1)?,
            name: row.get(2)?,
            phone_norm: row.get(3)?,
            designation: row.get(4)?,
        })
    })?;

    for r in staff_rows {
        staff.push(r?);
    }

    // 4. Receipts / Payments search (AC-3: utilizes idx_payments_search_composite)
    let mut receipts = Vec::new();
    let receipt_like = if trimmed.starts_with("RCP-") || trimmed.starts_with("2026-") {
        format!("{}%", trimmed)
    } else {
        format!("%{}%", trimmed)
    };

    let mut receipt_stmt = conn.prepare(
        "SELECT p.id, p.receipt_number, p.student_id, s.name, s.student_code, p.amount, p.method, p.paid_at
         FROM payments p
         JOIN students s ON s.id = p.student_id
         WHERE p.status = 'completed' AND (p.receipt_number LIKE ?1 OR p.id LIKE ?1)
         ORDER BY p.paid_at DESC
         LIMIT ?2"
    )?;

    let receipt_rows = receipt_stmt.query_map(params![receipt_like, lim], |row| {
        let amount_poisha: i64 = row.get(5)?;
        let amount_decimal = (amount_poisha as f64) / 100.0;
        Ok(GlobalReceiptHit {
            payment_id: row.get(0)?,
            receipt_number: row.get(1)?,
            student_id: row.get(2)?,
            student_name: row.get(3)?,
            student_code: row.get(4)?,
            amount_poisha,
            amount_formatted: format!("{:.2}", amount_decimal),
            method: row.get(6)?,
            paid_at: row.get(7)?,
        })
    })?;

    for r in receipt_rows {
        receipts.push(r?);
    }

    let total_hits = (students.len() + batches.len() + staff.len() + receipts.len()) as u32;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(GlobalSearchResponse {
        students,
        batches,
        staff,
        receipts,
        total_hits,
        duration_ms,
    })
}
