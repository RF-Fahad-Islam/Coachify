use crate::ai::AiError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;

// -----------------------------------------------------------------------------
// Tool Input Arguments & DTOs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct QueryStudentsWithDuesArgs {
    pub batch_id: Option<String>,
    pub min_due_poisha: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentDueItem {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub batch_name: Option<String>,
    pub total_due_poisha: i64,
    pub unpaid_charges_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetBatchSummaryArgs {
    pub batch_name_or_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchSummaryItem {
    pub batch_id: String,
    pub batch_name: String,
    pub batch_code: String,
    pub program_name: String,
    pub teacher_name: Option<String>,
    pub active_students_count: i64,
    pub capacity: i64,
    pub room_number: Option<String>,
    pub shift: Option<String>,
    pub schedule_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAttendanceStatsArgs {
    pub batch_id: Option<String>,
    pub days: Option<u32>,
    pub min_attendance_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentAttendanceSummary {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub present_count: i64,
    pub absent_count: i64,
    pub attendance_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AttendanceStatsResult {
    pub batch_id: Option<String>,
    pub batch_name: Option<String>,
    pub days_inspected: u32,
    pub total_sessions: i64,
    pub average_attendance_pct: f64,
    pub low_attendance_students: Vec<StudentAttendanceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetTodayOperationalSummaryArgs {
    pub target_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TodayOperationalSummary {
    pub target_date: String,
    pub classes_scheduled: i64,
    pub attendance_marked_count: i64,
    pub payments_count: i64,
    pub total_collection_poisha: i64,
    pub admissions_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchStudentsArgs {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentSearchSummary {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub phone: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub active_batches: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetExamResultsSummaryArgs {
    pub exam_id_or_title: String,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamResultsSummary {
    pub exam_id: String,
    pub exam_name: String,
    pub batch_name: Option<String>,
    pub total_students_graded: i64,
    pub average_mark: f64,
    pub highest_mark: f64,
    pub lowest_mark: f64,
    pub pass_rate_pct: f64,
    pub gpa5_count: i64,
}

// -----------------------------------------------------------------------------
// OpenAI Compatible Tool Declarations for LLM
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDeclaration {
    pub r#type: String,
    pub function: ToolFunction,
}

pub fn get_available_tool_schemas(has_financial_perm: bool) -> Vec<ToolDeclaration> {
    let mut tools = Vec::new();

    if has_financial_perm {
        tools.push(ToolDeclaration {
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "query_students_with_dues".to_string(),
                description: "Query students who have outstanding fee dues or balances above a minimum threshold".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "batch_id": { "type": "string", "description": "Optional specific batch ID to filter dues" },
                        "min_due_poisha": { "type": "integer", "description": "Minimum unpaid amount in poisha (e.g. 500000 for 5000 Taka)" },
                        "limit": { "type": "integer", "description": "Maximum number of rows to return (max 50)" }
                    }
                }),
            },
        });
    }

    tools.push(ToolDeclaration {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "get_batch_summary".to_string(),
            description: "Get details for an academic batch including enrolled student count, schedule, capacity, and teacher".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "batch_name_or_id": { "type": "string", "description": "Name, code, or UUID of the batch" }
                },
                "required": ["batch_name_or_id"]
            }),
        },
    });

    tools.push(ToolDeclaration {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "get_attendance_stats".to_string(),
            description: "Get attendance statistics and identify students with low attendance over a recent period".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "batch_id": { "type": "string", "description": "Optional batch ID filter" },
                    "days": { "type": "integer", "description": "Lookback window in days (default: 30)" },
                    "min_attendance_pct": { "type": "number", "description": "Threshold percentage to filter low-attendance students (e.g. 75.0)" }
                }
            }),
        },
    });

    tools.push(ToolDeclaration {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "get_today_operational_summary".to_string(),
            description: "Summarize today's operations: classes scheduled, attendance marked, collections, and new admissions".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "target_date": { "type": "string", "description": "Target date in YYYY-MM-DD format (defaults to today)" }
                }
            }),
        },
    });

    tools.push(ToolDeclaration {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "search_students".to_string(),
            description: "Search students by name, student code, or phone number to find enrollment and contact details".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Student name, code, or phone substring" },
                    "limit": { "type": "integer", "description": "Maximum results to return (max 50)" }
                },
                "required": ["query"]
            }),
        },
    });

    tools.push(ToolDeclaration {
        r#type: "function".to_string(),
        function: ToolFunction {
            name: "get_exam_results_summary".to_string(),
            description:
                "Calculate GPA averages, pass rates, and mark distributions for an exam or batch"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "exam_id_or_title": { "type": "string", "description": "Exam name or ID" },
                    "batch_id": { "type": "string", "description": "Optional batch ID filter" }
                },
                "required": ["exam_id_or_title"]
            }),
        },
    });

    tools
}

// -----------------------------------------------------------------------------
// Controlled Tool Implementations (Strict Read-Only, 50-row max cap)
// -----------------------------------------------------------------------------

pub fn query_students_with_dues(
    conn: &Connection,
    args: QueryStudentsWithDuesArgs,
    has_financial_perm: bool,
) -> Result<Vec<StudentDueItem>, AiError> {
    if !has_financial_perm {
        return Err(AiError::Unauthorized(
            "Financial dues access is restricted to authorized financial roles.".to_string(),
        ));
    }

    let min_due = args.min_due_poisha.unwrap_or(0);
    let limit = args.limit.unwrap_or(50).clamp(1, 50);

    let sql = r#"
        WITH charge_paid AS (
            SELECT
                c.id AS charge_id,
                c.student_id,
                c.enrollment_id,
                c.net_amount_poisha,
                COALESCE(SUM(pa.amount_poisha), 0) AS paid_poisha
            FROM charges c
            LEFT JOIN payment_allocations pa ON pa.charge_id = c.id
            WHERE c.status = 'open'
            GROUP BY c.id
        ),
        student_due_totals AS (
            SELECT
                cp.student_id,
                cp.enrollment_id,
                SUM(cp.net_amount_poisha - cp.paid_poisha) AS total_due_poisha,
                COUNT(CASE WHEN cp.net_amount_poisha > cp.paid_poisha THEN 1 END) AS unpaid_charges_count
            FROM charge_paid cp
            GROUP BY cp.student_id
            HAVING total_due_poisha >= ?1
        )
        SELECT
            s.id,
            s.student_code,
            s.name,
            b.name AS batch_name,
            sdt.total_due_poisha,
            sdt.unpaid_charges_count
        FROM student_due_totals sdt
        JOIN students s ON s.id = sdt.student_id
        LEFT JOIN enrollments e ON e.id = sdt.enrollment_id
        LEFT JOIN batches b ON b.id = e.batch_id
        WHERE (?2 IS NULL OR e.batch_id = ?2)
        ORDER BY sdt.total_due_poisha DESC
        LIMIT ?3
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![min_due, args.batch_id, limit], |r| {
        Ok(StudentDueItem {
            student_id: r.get(0)?,
            student_code: r.get(1)?,
            name: r.get(2)?,
            batch_name: r.get(3)?,
            total_due_poisha: r.get(4)?,
            unpaid_charges_count: r.get(5)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_batch_summary(
    conn: &Connection,
    args: GetBatchSummaryArgs,
) -> Result<BatchSummaryItem, AiError> {
    let query_param = args.batch_name_or_id.trim();

    let sql = r#"
        SELECT
            b.id,
            b.name,
            b.code,
            p.name AS program_name,
            st.name AS teacher_name,
            b.capacity,
            b.room_number,
            b.shift,
            b.schedule_json,
            (SELECT COUNT(*) FROM enrollments e WHERE e.batch_id = b.id AND e.status = 'active') AS active_count
        FROM batches b
        JOIN programs p ON p.id = b.program_id
        LEFT JOIN staff st ON st.id = b.teacher_staff_id
        WHERE b.id = ?1 OR b.code = ?1 OR b.name LIKE '%' || ?1 || '%'
        LIMIT 1
    "#;

    let row = conn
        .query_row(sql, params![query_param], |r| {
            let sched_raw: String = r.get(8).unwrap_or_else(|_| "[]".to_string());
            Ok(BatchSummaryItem {
                batch_id: r.get(0)?,
                batch_name: r.get(1)?,
                batch_code: r.get(2)?,
                program_name: r.get(3)?,
                teacher_name: r.get(4)?,
                capacity: r.get(5)?,
                room_number: r.get(6)?,
                shift: r.get(7)?,
                schedule_summary: sched_raw,
                active_students_count: r.get(9)?,
            })
        })
        .optional()?;

    row.ok_or_else(|| {
        AiError::ToolExecutionFailed(format!("Batch '{}' not found", args.batch_name_or_id))
    })
}

pub fn get_attendance_stats(
    conn: &Connection,
    args: GetAttendanceStatsArgs,
) -> Result<AttendanceStatsResult, AiError> {
    let days = args.days.unwrap_or(30).clamp(1, 365);
    let min_pct = args.min_attendance_pct.unwrap_or(100.0);

    let (batch_name, total_sessions): (Option<String>, i64) = if let Some(ref bid) = args.batch_id {
        let bname: Option<String> = conn
            .query_row(
                "SELECT name FROM batches WHERE id = ?1",
                params![bid],
                |r| r.get(0),
            )
            .optional()?;
        let sess_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM attendance_sessions WHERE batch_id = ?1 AND date >= date('now', '-' || ?2 || ' days')",
            params![bid, days],
            |r| r.get(0),
        ).unwrap_or(0);
        (bname, sess_count)
    } else {
        let sess_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM attendance_sessions WHERE date >= date('now', '-' || ?1 || ' days')",
            params![days],
            |r| r.get(0),
        ).unwrap_or(0);
        (None, sess_count)
    };

    let sql = r#"
        SELECT
            s.id,
            s.student_code,
            s.name,
            COUNT(CASE WHEN am.status IN ('present', 'late') THEN 1 END) AS present_count,
            COUNT(CASE WHEN am.status = 'absent' THEN 1 END) AS absent_count,
            COUNT(*) AS total_marked
        FROM attendance_marks am
        JOIN attendance_sessions ses ON ses.id = am.session_id
        JOIN students s ON s.id = am.student_id
        WHERE ses.date >= date('now', '-' || ?1 || ' days')
          AND (?2 IS NULL OR ses.batch_id = ?2)
        GROUP BY s.id
        ORDER BY present_count ASC
        LIMIT 50
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![days, args.batch_id], |r| {
        let sid: String = r.get(0)?;
        let scode: String = r.get(1)?;
        let sname: String = r.get(2)?;
        let present: i64 = r.get(3)?;
        let absent: i64 = r.get(4)?;
        let total: i64 = r.get(5)?;
        let pct = if total > 0 {
            (present as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Ok(StudentAttendanceSummary {
            student_id: sid,
            student_code: scode,
            name: sname,
            present_count: present,
            absent_count: absent,
            attendance_pct: (pct * 10.0).round() / 10.0,
        })
    })?;

    let mut low_students = Vec::new();
    let mut sum_pct = 0.0;
    let mut student_count = 0;

    for row in rows {
        let item = row?;
        sum_pct += item.attendance_pct;
        student_count += 1;
        if item.attendance_pct <= min_pct {
            low_students.push(item);
        }
    }

    let avg_pct = if student_count > 0 {
        (sum_pct / student_count as f64 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(AttendanceStatsResult {
        batch_id: args.batch_id,
        batch_name,
        days_inspected: days,
        total_sessions,
        average_attendance_pct: avg_pct,
        low_attendance_students: low_students,
    })
}

pub fn get_today_operational_summary(
    conn: &Connection,
    args: GetTodayOperationalSummaryArgs,
    has_financial_perm: bool,
) -> Result<TodayOperationalSummary, AiError> {
    let target_date = args
        .target_date
        .unwrap_or_else(crate::ledger::current_business_date);

    // 1. Classes scheduled today
    let classes_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT batch_id) FROM attendance_sessions WHERE date = ?1",
            params![target_date],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 2. Attendance marked sessions count
    let attendance_marked: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM attendance_sessions WHERE date = ?1",
            params![target_date],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 3. Payments collected today (gated by financial permission)
    let (pay_count, total_collected): (i64, i64) = if has_financial_perm {
        conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_poisha), 0) FROM payments WHERE business_date = ?1 AND status != 'voided'",
            params![target_date],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    // 4. Admissions today
    let admissions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM students WHERE admission_date = ?1",
            params![target_date],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(TodayOperationalSummary {
        target_date,
        classes_scheduled: classes_count,
        attendance_marked_count: attendance_marked,
        payments_count: pay_count,
        total_collection_poisha: total_collected,
        admissions_count: admissions,
    })
}

pub fn search_students(
    conn: &Connection,
    args: SearchStudentsArgs,
) -> Result<Vec<StudentSearchSummary>, AiError> {
    let limit = args.limit.unwrap_or(50).clamp(1, 50);
    let term = args.query.trim();

    let sql = r#"
        SELECT
            s.id,
            s.student_code,
            s.name,
            s.phone_norm,
            g.name AS guardian_name,
            g.phone_norm AS guardian_phone,
            s.status
        FROM students s
        LEFT JOIN student_guardians sg ON sg.student_id = s.id AND sg.is_primary = 1
        LEFT JOIN guardians g ON g.id = sg.guardian_id
        WHERE s.name LIKE '%' || ?1 || '%'
           OR s.student_code LIKE '%' || ?1 || '%'
           OR s.phone_norm LIKE '%' || ?1 || '%'
        LIMIT ?2
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![term, limit], |r| {
        let sid: String = r.get(0)?;
        let scode: String = r.get(1)?;
        let sname: String = r.get(2)?;
        let phone: String = r.get(3)?;
        let gname: Option<String> = r.get(4)?;
        let gphone: Option<String> = r.get(5)?;
        let status: String = r.get(6)?;

        Ok((sid, scode, sname, phone, gname, gphone, status))
    })?;

    let mut results = Vec::new();
    for row in rows {
        let (sid, scode, sname, phone, gname, gphone, status) = row?;

        // Retrieve active batch names for student
        let mut b_stmt = conn.prepare(
            "SELECT b.name FROM enrollments e JOIN batches b ON b.id = e.batch_id WHERE e.student_id = ?1 AND e.status = 'active'"
        )?;
        let batch_names: Vec<String> = b_stmt
            .query_map(params![sid], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        results.push(StudentSearchSummary {
            student_id: sid,
            student_code: scode,
            name: sname,
            phone,
            guardian_name: gname,
            guardian_phone: gphone,
            active_batches: batch_names,
            status,
        });
    }

    Ok(results)
}

pub fn get_exam_results_summary(
    conn: &Connection,
    args: GetExamResultsSummaryArgs,
) -> Result<ExamResultsSummary, AiError> {
    let term = args.exam_id_or_title.trim();

    let exam_info: (String, String) = conn
        .query_row(
            "SELECT id, name FROM exams WHERE id = ?1 OR name LIKE '%' || ?1 || '%' LIMIT 1",
            params![term],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| {
            AiError::ToolExecutionFailed(format!("Exam '{}' not found", args.exam_id_or_title))
        })?;

    let (exam_id, exam_name) = exam_info;

    let batch_name: Option<String> = if let Some(ref bid) = args.batch_id {
        conn.query_row(
            "SELECT name FROM batches WHERE id = ?1",
            params![bid],
            |r| r.get(0),
        )
        .optional()?
    } else {
        None
    };

    let sql = r#"
        SELECT
            COUNT(m.student_id),
            COALESCE(AVG(m.marks), 0),
            COALESCE(MAX(m.marks), 0),
            COALESCE(MIN(m.marks), 0),
            COUNT(CASE WHEN m.marks >= es.pass_marks THEN 1 END)
        FROM marks m
        JOIN exam_subjects es ON es.id = m.exam_subject_id
        LEFT JOIN enrollments e ON e.student_id = m.student_id
        WHERE es.exam_id = ?1
          AND (?2 IS NULL OR e.batch_id = ?2)
          AND m.is_absent = 0
    "#;

    let (total_graded, avg_raw, max_raw, min_raw, pass_count): (i64, f64, i64, i64, i64) = conn
        .query_row(sql, params![exam_id, args.batch_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap_or((0, 0.0, 0, 0, 0));

    // Marks are stored in hundredths (e.g. 8850 = 88.50)
    let avg_mark = (avg_raw / 100.0 * 10.0).round() / 10.0;
    let max_mark = max_raw as f64 / 100.0;
    let min_mark = min_raw as f64 / 100.0;
    let pass_rate = if total_graded > 0 {
        ((pass_count as f64 / total_graded as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    // Calculate approximate GPA 5.0 count (>= 80% marks)
    let gpa5_count: i64 = conn
        .query_row(
            r#"
        SELECT COUNT(DISTINCT m.student_id)
        FROM marks m
        JOIN exam_subjects es ON es.id = m.exam_subject_id
        LEFT JOIN enrollments e ON e.student_id = m.student_id
        WHERE es.exam_id = ?1
          AND (?2 IS NULL OR e.batch_id = ?2)
          AND m.marks >= (es.full_marks * 80 / 100)
        "#,
            params![exam_id, args.batch_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(ExamResultsSummary {
        exam_id,
        exam_name,
        batch_name,
        total_students_graded: total_graded,
        average_mark: avg_mark,
        highest_mark: max_mark,
        lowest_mark: min_mark,
        pass_rate_pct: pass_rate,
        gpa5_count,
    })
}

// -----------------------------------------------------------------------------
// Unified Tool Dispatcher
// -----------------------------------------------------------------------------

pub fn execute_tool(
    conn: &Connection,
    name: &str,
    args_json: &str,
    has_financial_perm: bool,
) -> Result<serde_json::Value, AiError> {
    match name {
        "query_students_with_dues" => {
            let args: QueryStudentsWithDuesArgs = serde_json::from_str(args_json).map_err(|e| {
                AiError::Validation(format!("Invalid query_students_with_dues args: {}", e))
            })?;
            let res = query_students_with_dues(conn, args, has_financial_perm)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "get_batch_summary" => {
            let args: GetBatchSummaryArgs = serde_json::from_str(args_json).map_err(|e| {
                AiError::Validation(format!("Invalid get_batch_summary args: {}", e))
            })?;
            let res = get_batch_summary(conn, args)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "get_attendance_stats" => {
            let args: GetAttendanceStatsArgs = serde_json::from_str(args_json).map_err(|e| {
                AiError::Validation(format!("Invalid get_attendance_stats args: {}", e))
            })?;
            let res = get_attendance_stats(conn, args)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "get_today_operational_summary" => {
            let args: GetTodayOperationalSummaryArgs = serde_json::from_str(args_json).map_err(
                |e| AiError::Validation(format!("Invalid get_today_operational_summary args: {}", e)),
            )?;
            let res = get_today_operational_summary(conn, args, has_financial_perm)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "search_students" => {
            let args: SearchStudentsArgs = serde_json::from_str(args_json).map_err(|e| {
                AiError::Validation(format!("Invalid search_students args: {}", e))
            })?;
            let res = search_students(conn, args)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        "get_exam_results_summary" => {
            let args: GetExamResultsSummaryArgs = serde_json::from_str(args_json).map_err(|e| {
                AiError::Validation(format!("Invalid get_exam_results_summary args: {}", e))
            })?;
            let res = get_exam_results_summary(conn, args)?;
            Ok(serde_json::to_value(res).unwrap())
        }
        unknown => Err(AiError::Validation(format!(
            "Tool '{}' is not recognized. Direct SQL execution or arbitrary system access is prohibited.",
            unknown
        ))),
    }
}
