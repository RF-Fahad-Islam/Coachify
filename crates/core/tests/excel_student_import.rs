use coachify_core::db;
use coachify_core::dispatcher::{self, ChangeEvent};
use coachify_core::import::{
    create_pre_import_backup, excel_serial_to_date, execute_import, inspect_file,
    normalize_bengali_digits, parse_flexible_date, parse_gender, preview_validation,
    suggest_column_mappings, ColumnMappingInput, ExecuteImportInput, ImportError,
    PreviewValidationPayload,
};
use rusqlite::Connection;
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn setup_test_db() -> (tempfile::TempDir, Connection) {
    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test_import.db");
    let conn = db::open_connection_at(&db_path).expect("open test db");
    db::apply_all_migrations(&conn).expect("apply all migrations");
    (dir, conn)
}

fn dispatch_cmd(
    conn: &mut Connection,
    command: &str,
    payload: serde_json::Value,
    token: Option<&str>,
    idem_key: Option<&str>,
) -> (serde_json::Value, Vec<ChangeEvent>) {
    let payload_str = serde_json::to_string(&payload).unwrap();
    let (res_str, events) = dispatcher::dispatch(conn, command, &payload_str, token, idem_key);
    let val: serde_json::Value = serde_json::from_str(&res_str).unwrap();
    (val, events)
}

fn bootstrap_and_login(conn: &mut Connection) -> String {
    let (res, _) = dispatch_cmd(
        conn,
        "system.bootstrap_owner",
        json!({
            "username": "admin",
            "display_name": "Admin Owner",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "bootstrap failed: {:?}", res);

    let (res, _) = dispatch_cmd(
        conn,
        "auth.login",
        json!({
            "username": "admin",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(res["ok"], true, "login failed: {:?}", res);
    res["data"]["session_token"].as_str().unwrap().to_string()
}

/// Helper that crafts a minimal valid .xlsx file in pure Rust using `zip`.
fn create_sample_xlsx(path: &Path, sheet_name: &str, rows: &[Vec<&str>]) {
    let file = File::create(path).expect("create xlsx file");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#).unwrap();

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#).unwrap();

    // 3. xl/workbook.xml
    zip.start_file("xl/workbook.xml", options).unwrap();
    let wb_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="{}" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
        sheet_name
    );
    zip.write_all(wb_xml.as_bytes()).unwrap();

    // 4. xl/_rels/workbook.xml.rels
    zip.start_file("xl/_rels/workbook.xml.rels", options)
        .unwrap();
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#).unwrap();

    // 5. xl/worksheets/sheet1.xml
    zip.start_file("xl/worksheets/sheet1.xml", options).unwrap();
    let mut sheet_xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
"#,
    );
    for (r_idx, row) in rows.iter().enumerate() {
        let r_num = r_idx + 1;
        sheet_xml.push_str(&format!(r#"    <row r="{}">"#, r_num));
        for (c_idx, val) in row.iter().enumerate() {
            let col_letter = (b'A' + c_idx as u8) as char;
            sheet_xml.push_str(&format!(
                r#"<c r="{}{}" t="inlineStr"><is><t>{}</t></is></c>"#,
                col_letter, r_num, val
            ));
        }
        sheet_xml.push_str("</row>\n");
    }
    sheet_xml.push_str("  </sheetData>\n</worksheet>");
    zip.write_all(sheet_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
}

// -----------------------------------------------------------------------------
// Test 1: Bengali Numeral Transliteration & Gender Parsing (AC-3)
// -----------------------------------------------------------------------------
#[test]
fn test_bengali_numeral_and_gender_normalization() {
    assert_eq!(normalize_bengali_digits("০১৭১২৩৪৫৬৭৮"), "01712345678");
    assert_eq!(normalize_bengali_digits("১২/০৫/২০০৮"), "12/05/2008");
    assert_eq!(normalize_bengali_digits("Roll: ১২"), "Roll: 12");

    assert_eq!(parse_gender("male"), Some("male".to_string()));
    assert_eq!(parse_gender("M"), Some("male".to_string()));
    assert_eq!(parse_gender("ছেলে"), Some("male".to_string()));
    assert_eq!(parse_gender("পুরুষ"), Some("male".to_string()));

    assert_eq!(parse_gender("female"), Some("female".to_string()));
    assert_eq!(parse_gender("F"), Some("female".to_string()));
    assert_eq!(parse_gender("মেয়ে"), Some("female".to_string()));
    assert_eq!(parse_gender("মহিলা"), Some("female".to_string()));

    assert_eq!(parse_gender("other"), Some("other".to_string()));
    assert_eq!(parse_gender("তৃতীয় লিঙ্গ"), Some("other".to_string()));
    assert_eq!(parse_gender("unknown_value"), None);
}

// -----------------------------------------------------------------------------
// Test 2: Flexible Date Parsing & Excel Serial Conversion (AC-3)
// -----------------------------------------------------------------------------
#[test]
fn test_flexible_date_parsing() {
    assert_eq!(
        parse_flexible_date("2008-05-12"),
        Some("2008-05-12".to_string())
    );
    assert_eq!(
        parse_flexible_date("12/05/2008"),
        Some("2008-05-12".to_string())
    );
    assert_eq!(
        parse_flexible_date("12-05-2008"),
        Some("2008-05-12".to_string())
    );
    assert_eq!(
        parse_flexible_date("১২/০৫/২০০৮"),
        Some("2008-05-12".to_string())
    );

    // Excel numeric serial 39580 = 2008-05-12
    assert_eq!(
        excel_serial_to_date(39580.0),
        Some("2008-05-12".to_string())
    );
    assert_eq!(parse_flexible_date("39580"), Some("2008-05-12".to_string()));

    // Invalid dates
    assert_eq!(parse_flexible_date(""), None);
    assert_eq!(parse_flexible_date("not-a-date"), None);
    assert_eq!(parse_flexible_date("2026/99/99"), None);
}

// -----------------------------------------------------------------------------
// Test 3: Bilingual Column Auto-Mapping Heuristics (AC-2)
// -----------------------------------------------------------------------------
#[test]
fn test_bilingual_column_auto_mapping() {
    let bengali_headers = vec![
        "নাম".to_string(),
        "লিঙ্গ".to_string(),
        "জন্ম তারিখ".to_string(),
        "মোবাইল".to_string(),
        "পিতার নাম".to_string(),
        "বাবার মোবাইল".to_string(),
        "বর্তমান ঠিকানা".to_string(),
    ];
    let sample_rows = vec![vec![
        "সাকিব আল হাসান".to_string(),
        "পুরুষ".to_string(),
        "2008-05-12".to_string(),
        "01712345678".to_string(),
        "মাশরুর রেজা".to_string(),
        "01812345678".to_string(),
        "মাগুরা".to_string(),
    ]];

    let suggestions = suggest_column_mappings(&bengali_headers, &sample_rows);
    assert_eq!(suggestions.len(), 7);

    assert_eq!(
        suggestions[0].target_field,
        Some("student_name".to_string())
    );
    assert!(suggestions[0].confidence >= 0.8);
    assert_eq!(suggestions[1].target_field, Some("gender".to_string()));
    assert_eq!(suggestions[2].target_field, Some("dob".to_string()));
    assert_eq!(
        suggestions[3].target_field,
        Some("student_phone".to_string())
    );
    assert_eq!(
        suggestions[4].target_field,
        Some("guardian_name".to_string())
    );
    assert_eq!(
        suggestions[5].target_field,
        Some("guardian_phone".to_string())
    );
    assert_eq!(
        suggestions[6].target_field,
        Some("present_address".to_string())
    );
}

// -----------------------------------------------------------------------------
// Test 4: File Inspection for CSV & XLSX (AC-1)
// -----------------------------------------------------------------------------
#[test]
fn test_file_inspection_csv_and_xlsx() {
    let dir = tempdir().unwrap();

    // 1. Test CSV
    let csv_path = dir.path().join("students.csv");
    let csv_content = "Student Name,Gender,DOB,Phone,Father Name,Father Phone,Address\n\
                       Rahim,Male,2008-05-12,01711111111,Karim,01811111111,Dhaka\n\
                       Fatima,Female,2009-02-18,01722222222,Ali,01822222222,Chittagong\n";
    fs::write(&csv_path, csv_content).unwrap();

    let inspection_csv = inspect_file(&csv_path).expect("inspect csv");
    assert_eq!(inspection_csv.format, "csv");
    assert_eq!(inspection_csv.sheets.len(), 1);
    assert_eq!(inspection_csv.sheets[0].row_count, 2);
    assert_eq!(inspection_csv.sheets[0].sample_headers.len(), 7);

    // 2. Test XLSX
    let xlsx_path = dir.path().join("students.xlsx");
    let xlsx_rows = vec![
        vec![
            "Student Name",
            "Gender",
            "DOB",
            "Student Phone",
            "Guardian Name",
            "Guardian Phone",
            "Address",
        ],
        vec![
            "Rahim",
            "male",
            "2008-05-12",
            "01711111111",
            "Karim",
            "01811111111",
            "Dhaka",
        ],
    ];
    create_sample_xlsx(&xlsx_path, "ClassRoster", &xlsx_rows);

    let inspection_xlsx = inspect_file(&xlsx_path).expect("inspect xlsx");
    assert_eq!(inspection_xlsx.format, "xlsx");
    assert_eq!(inspection_xlsx.sheets.len(), 1);
    assert_eq!(inspection_xlsx.sheets[0].name, "ClassRoster");
    assert_eq!(inspection_xlsx.sheets[0].row_count, 1);
    assert_eq!(inspection_xlsx.sheets[0].sample_headers.len(), 7);
}

// -----------------------------------------------------------------------------
// Test 5: In-Memory Validation Matrix & Issue Diagnostics (AC-5)
// -----------------------------------------------------------------------------
#[test]
fn test_in_memory_validation_matrix() {
    let (_dir, conn) = setup_test_db();
    let temp = tempdir().unwrap();

    let csv_path = temp.path().join("validation_test.csv");
    // Row 1: Valid
    // Row 2: Warning (Sibling with row 1 - same guardian phone)
    // Row 3: Error (Invalid phone and missing student name)
    let csv_content = "Name,Gender,DOB,Phone,Guardian Name,Guardian Phone,Address\n\
                       Hasan,male,2008-05-12,01711111111,Babul,01811111111,Dhaka\n\
                       Hussein,male,2009-08-15,01722222222,Babul,01811111111,Dhaka\n\
                       ,female,2010-01-01,99999,Mother,01833333333,Khulna\n";
    fs::write(&csv_path, csv_content).unwrap();

    let mappings = vec![
        ColumnMappingInput {
            source_column: "Name".to_string(),
            target_field: "student_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Gender".to_string(),
            target_field: "gender".to_string(),
        },
        ColumnMappingInput {
            source_column: "DOB".to_string(),
            target_field: "dob".to_string(),
        },
        ColumnMappingInput {
            source_column: "Phone".to_string(),
            target_field: "student_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Name".to_string(),
            target_field: "guardian_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Phone".to_string(),
            target_field: "guardian_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Address".to_string(),
            target_field: "present_address".to_string(),
        },
    ];

    let payload = PreviewValidationPayload {
        file_path: csv_path.display().to_string(),
        sheet_name: None,
        mappings,
        phone_conflict_mode: Some("link_guardian".to_string()),
        default_present_address: None,
    };

    let result = preview_validation(&conn, &payload).expect("preview validation");
    assert_eq!(result.total_rows, 3);
    assert_eq!(result.valid_count, 2);
    assert_eq!(result.warning_count, 2); // rows 1 and 2 both flagged as siblings
    assert_eq!(result.error_count, 1); // row 3 has errors

    // Verify zero records were written to SQLite during validation
    let count: i64 = conn
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        count, 0,
        "No students must be persisted during validation dry run"
    );
}

// -----------------------------------------------------------------------------
// Test 6: Pre-Import SQLite Online Backup Snapshot (AC-6)
// -----------------------------------------------------------------------------
#[test]
fn test_pre_import_backup_snapshot() {
    let (_dir, conn) = setup_test_db();
    let temp = tempdir().unwrap();
    let backup_dir = temp.path().join("backups");

    let backup_file = create_pre_import_backup(&conn, Some(&backup_dir)).expect("create backup");
    assert!(backup_file.exists(), "Backup snapshot must exist on disk");

    let meta = fs::metadata(&backup_file).unwrap();
    assert!(meta.len() > 0, "Backup file size must be > 0 bytes");

    // Verify the backup database is valid and readable SQLite
    let backup_conn = Connection::open(&backup_file).expect("open backup db");
    let integrity: String = backup_conn
        .query_row("PRAGMA quick_check;", [], |r| r.get(0))
        .expect("check backup integrity");
    assert_eq!(
        integrity, "ok",
        "Backup snapshot must pass PRAGMA quick_check"
    );
}

// -----------------------------------------------------------------------------
// Test 7: Atomic Import with Sibling Deduplication (AC-4, AC-7)
// -----------------------------------------------------------------------------
#[test]
fn test_atomic_bulk_import_with_sibling_deduplication() {
    let (_dir, conn) = setup_test_db();
    let temp = tempdir().unwrap();

    let csv_path = temp.path().join("siblings.csv");
    let csv_content = "Name,Gender,DOB,Phone,Guardian Name,Guardian Phone,Address\n\
                       Tariq,male,2008-01-10,01711111111,Alamgir,01899999999,Mirpur Dhaka\n\
                       Tahmina,female,2010-04-12,01722222222,Alamgir,01899999999,Mirpur Dhaka\n";
    fs::write(&csv_path, csv_content).unwrap();

    let mappings = vec![
        ColumnMappingInput {
            source_column: "Name".to_string(),
            target_field: "student_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Gender".to_string(),
            target_field: "gender".to_string(),
        },
        ColumnMappingInput {
            source_column: "DOB".to_string(),
            target_field: "dob".to_string(),
        },
        ColumnMappingInput {
            source_column: "Phone".to_string(),
            target_field: "student_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Name".to_string(),
            target_field: "guardian_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Phone".to_string(),
            target_field: "guardian_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Address".to_string(),
            target_field: "present_address".to_string(),
        },
    ];

    let input = ExecuteImportInput {
        file_path: csv_path.display().to_string(),
        sheet_name: None,
        mappings,
        skip_error_rows: false,
        phone_conflict_mode: "link_guardian".to_string(),
        target_batch_id: None,
        default_present_address: None,
    };

    let summary = execute_import(&conn, &input, Some("admin_user"), "2026-09-22T02:00:00Z")
        .expect("execute import");

    assert_eq!(summary.total_rows, 2);
    assert_eq!(summary.imported_students_count, 2);
    assert_eq!(
        summary.imported_guardians_count, 1,
        "Only 1 shared guardian entity must be created for siblings"
    );
    assert_eq!(summary.skipped_errors_count, 0);

    // Verify in database: 2 students, 1 guardian, 2 student_guardians links pointing to same guardian
    let student_count: i64 = conn
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    assert_eq!(student_count, 2);

    let guardian_count: i64 = conn
        .query_row("SELECT count(*) FROM guardians", [], |r| r.get(0))
        .unwrap();
    assert_eq!(guardian_count, 1);

    let link_count: i64 = conn
        .query_row("SELECT count(*) FROM student_guardians", [], |r| r.get(0))
        .unwrap();
    assert_eq!(link_count, 2);

    let distinct_guardian_links: i64 = conn
        .query_row(
            "SELECT count(distinct guardian_id) FROM student_guardians",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        distinct_guardian_links, 1,
        "Both students must be linked to the same guardian ID"
    );
}

// -----------------------------------------------------------------------------
// Test 8: Selective Import & Rejected Rows CSV Export (AC-8)
// -----------------------------------------------------------------------------
#[test]
fn test_selective_import_and_rejected_rows_csv() {
    let (_dir, conn) = setup_test_db();
    let temp = tempdir().unwrap();

    let csv_path = temp.path().join("selective_test.csv");
    let csv_content = "Name,Gender,DOB,Phone,Guardian Name,Guardian Phone,Address\n\
                       Valid One,male,2008-05-12,01711111111,Father One,01811111111,Dhaka\n\
                       ,female,2009-01-01,01722222222,Father Two,01822222222,Dhaka\n\
                       Valid Two,female,2007-09-20,01733333333,Father Three,01833333333,Dhaka\n";
    fs::write(&csv_path, csv_content).unwrap();

    let mappings = vec![
        ColumnMappingInput {
            source_column: "Name".to_string(),
            target_field: "student_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Gender".to_string(),
            target_field: "gender".to_string(),
        },
        ColumnMappingInput {
            source_column: "DOB".to_string(),
            target_field: "dob".to_string(),
        },
        ColumnMappingInput {
            source_column: "Phone".to_string(),
            target_field: "student_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Name".to_string(),
            target_field: "guardian_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Phone".to_string(),
            target_field: "guardian_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Address".to_string(),
            target_field: "present_address".to_string(),
        },
    ];

    let input = ExecuteImportInput {
        file_path: csv_path.display().to_string(),
        sheet_name: None,
        mappings,
        skip_error_rows: true, // Allow partial import!
        phone_conflict_mode: "allow_duplicate".to_string(),
        target_batch_id: None,
        default_present_address: None,
    };

    let summary = execute_import(&conn, &input, Some("admin"), "2026-09-22T02:00:00Z")
        .expect("execute selective import");

    assert_eq!(summary.imported_students_count, 2);
    assert_eq!(summary.skipped_errors_count, 1);
    assert!(
        summary.rejected_rows_csv_path.is_some(),
        "Rejected CSV path must be returned"
    );

    let rejected_path = summary.rejected_rows_csv_path.unwrap();
    let content = fs::read_to_string(&rejected_path).unwrap();
    assert!(
        content.contains("_import_error"),
        "Rejected CSV must contain _import_error header"
    );
    assert!(
        content.contains("Missing student name"),
        "Rejected CSV must explain reason for rejection"
    );
}

// -----------------------------------------------------------------------------
// Test 9: Strict Rollback Guarantee on Constraint Failure (AC-7)
// -----------------------------------------------------------------------------
#[test]
fn test_rollback_on_strict_mode() {
    let (_dir, conn) = setup_test_db();
    let temp = tempdir().unwrap();

    let csv_path = temp.path().join("strict_test.csv");
    let csv_content = "Name,Gender,DOB,Phone,Guardian Name,Guardian Phone,Address\n\
                       Valid One,male,2008-05-12,01711111111,Father One,01811111111,Dhaka\n\
                       ,female,2009-01-01,invalid_phone,Father Two,01822222222,Dhaka\n";
    fs::write(&csv_path, csv_content).unwrap();

    let mappings = vec![
        ColumnMappingInput {
            source_column: "Name".to_string(),
            target_field: "student_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Gender".to_string(),
            target_field: "gender".to_string(),
        },
        ColumnMappingInput {
            source_column: "DOB".to_string(),
            target_field: "dob".to_string(),
        },
        ColumnMappingInput {
            source_column: "Phone".to_string(),
            target_field: "student_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Name".to_string(),
            target_field: "guardian_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Phone".to_string(),
            target_field: "guardian_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Address".to_string(),
            target_field: "present_address".to_string(),
        },
    ];

    let input = ExecuteImportInput {
        file_path: csv_path.display().to_string(),
        sheet_name: None,
        mappings,
        skip_error_rows: false, // Strict mode: must fail and rollback!
        phone_conflict_mode: "allow_duplicate".to_string(),
        target_batch_id: None,
        default_present_address: None,
    };

    let result = execute_import(&conn, &input, Some("admin"), "2026-09-22T02:00:00Z");
    assert!(result.is_err(), "Strict import must fail when errors exist");

    // Verify 0 rows persisted in students or guardians
    let s_count: i64 = conn
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    let g_count: i64 = conn
        .query_row("SELECT count(*) FROM guardians", [], |r| r.get(0))
        .unwrap();
    assert_eq!(s_count, 0, "Zero students must remain after rollback");
    assert_eq!(g_count, 0, "Zero guardians must remain after rollback");
}

// -----------------------------------------------------------------------------
// Test 10: Academic Batch Enrollment on Import (AC-9)
// -----------------------------------------------------------------------------
#[test]
fn test_academic_batch_enrollment_on_import() {
    let (_dir, mut conn) = setup_test_db();
    let token = bootstrap_and_login(&mut conn);
    let temp = tempdir().unwrap();

    // 1. Setup session, program, and batch via RPC
    let (prog_res, _) = dispatch_cmd(
        &mut conn,
        "programs.create",
        json!({
            "program": {
                "name": "HSC Science 2026",
                "code": "HSC26",
                "curriculum": "hsc",
                "class_level": "11"
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(prog_res["ok"], true);
    let program_id = prog_res["data"]["id"].as_str().unwrap().to_string();

    let (batch_res, _) = dispatch_cmd(
        &mut conn,
        "batches.create",
        json!({
            "batch": {
                "program_id": program_id,
                "name": "Batch Alpha",
                "code": "ALPHA",
                "capacity": 50,
                "monthly_tuition_poisha": 250000
            }
        }),
        Some(&token),
        None,
    );
    assert_eq!(batch_res["ok"], true);
    let batch_id = batch_res["data"]["id"].as_str().unwrap().to_string();

    // 2. Import students targeting this batch
    let csv_path = temp.path().join("batch_import.csv");
    let csv_content = "Name,Gender,DOB,Phone,Guardian Name,Guardian Phone,Address\n\
                       Student 1,male,2008-05-12,01711111111,Father 1,01811111111,Dhaka\n\
                       Student 2,female,2008-06-15,01722222222,Father 2,01822222222,Dhaka\n";
    fs::write(&csv_path, csv_content).unwrap();

    let mappings = vec![
        ColumnMappingInput {
            source_column: "Name".to_string(),
            target_field: "student_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Gender".to_string(),
            target_field: "gender".to_string(),
        },
        ColumnMappingInput {
            source_column: "DOB".to_string(),
            target_field: "dob".to_string(),
        },
        ColumnMappingInput {
            source_column: "Phone".to_string(),
            target_field: "student_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Name".to_string(),
            target_field: "guardian_name".to_string(),
        },
        ColumnMappingInput {
            source_column: "Guardian Phone".to_string(),
            target_field: "guardian_phone".to_string(),
        },
        ColumnMappingInput {
            source_column: "Address".to_string(),
            target_field: "present_address".to_string(),
        },
    ];

    let input = ExecuteImportInput {
        file_path: csv_path.display().to_string(),
        sheet_name: None,
        mappings,
        skip_error_rows: false,
        phone_conflict_mode: "allow_duplicate".to_string(),
        target_batch_id: Some(batch_id.clone()),
        default_present_address: None,
    };

    let summary = execute_import(&conn, &input, Some("admin"), "2026-09-22T02:00:00Z")
        .expect("execute batch import");
    assert_eq!(summary.imported_students_count, 2);

    // 3. Verify enrollment records in DB
    let enroll_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM enrollments WHERE batch_id = ?1 AND status = 'active'",
            rusqlite::params![batch_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        enroll_count, 2,
        "Both students must be enrolled in the batch"
    );

    // Verify sequential rolls: roll 1 and roll 2
    let mut stmt = conn
        .prepare("SELECT roll FROM enrollments WHERE batch_id = ?1 ORDER BY roll ASC")
        .unwrap();
    let rolls: Vec<i64> = stmt
        .query_map(rusqlite::params![batch_id], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(
        rolls,
        vec![1, 2],
        "Roll numbers must be allocated sequentially"
    );
}

// -----------------------------------------------------------------------------
// Test 11: RBAC Permission Gating & Dispatcher Integration (AC-10)
// -----------------------------------------------------------------------------
#[test]
fn test_rbac_permission_gating_via_dispatcher() {
    let (_dir, mut conn) = setup_test_db();
    let admin_token = bootstrap_and_login(&mut conn);
    let temp = tempdir().unwrap();

    // Create a staff user with read-only role lacking students.manage
    let (role_res, _) = dispatch_cmd(
        &mut conn,
        "roles.create",
        json!({
            "name": "ReadOnlyViewer",
            "description": "Viewer role",
            "permission_codes": ["students.view"]
        }),
        Some(&admin_token),
        None,
    );
    assert_eq!(role_res["ok"], true);
    let role_id = role_res["data"]["id"].as_str().unwrap().to_string();

    let (user_res, _) = dispatch_cmd(
        &mut conn,
        "users.create",
        json!({
            "username": "cashier1",
            "display_name": "Cashier Staff",
            "password": "Password123!",
            "role_ids": [role_id]
        }),
        Some(&admin_token),
        None,
    );
    assert_eq!(user_res["ok"], true);

    let (cashier_login, _) = dispatch_cmd(
        &mut conn,
        "auth.login",
        json!({
            "username": "cashier1",
            "password": "Password123!"
        }),
        None,
        None,
    );
    assert_eq!(cashier_login["ok"], true);
    let cashier_token = cashier_login["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();

    let csv_path = temp.path().join("rbac_test.csv");
    fs::write(&csv_path, "Name,Gender\nTest,Male\n").unwrap();

    // 1. Attempt import.inspect_file as cashier -> Must fail with permission_denied
    let (res, _) = dispatch_cmd(
        &mut conn,
        "import.inspect_file",
        json!({ "file_path": csv_path.display().to_string() }),
        Some(&cashier_token),
        None,
    );
    assert_eq!(res["ok"], false);
    assert_eq!(res["error"]["code"], "forbidden");

    // 2. Attempt as admin -> Must succeed
    let (admin_res, _) = dispatch_cmd(
        &mut conn,
        "import.inspect_file",
        json!({ "file_path": csv_path.display().to_string() }),
        Some(&admin_token),
        None,
    );
    assert_eq!(admin_res["ok"], true);
    assert_eq!(admin_res["data"]["format"], "csv");
}

// -----------------------------------------------------------------------------
// Test 12: File Inspection Edge Cases & Corrupted/Unsupported Formats
// -----------------------------------------------------------------------------
#[test]
fn test_edge_cases_corrupt_and_unsupported_files() {
    let temp = tempdir().unwrap();

    // 1. Non-existent file
    let non_existent = temp.path().join("does_not_exist.xlsx");
    let res = inspect_file(&non_existent);
    assert!(matches!(res, Err(ImportError::FileNotFound(_))));

    // 2. Unsupported format (.txt)
    let txt_file = temp.path().join("data.txt");
    fs::write(&txt_file, "col1,col2\nval1,val2").unwrap();
    let res = inspect_file(&txt_file);
    assert!(matches!(res, Err(ImportError::UnsupportedFormat(_))));

    // 3. CSV with UTF-8 BOM and leading/trailing whitespace in headers
    let bom_csv = temp.path().join("bom_test.csv");
    let content = "\u{feff}  Name  ,  Phone  ,  Gender  \nJohn,01711112233,Male\n";
    fs::write(&bom_csv, content).unwrap();
    let inspection = inspect_file(&bom_csv).expect("inspect CSV with BOM");
    assert_eq!(inspection.sheets.len(), 1);
    assert_eq!(
        inspection.sheets[0].sample_headers,
        vec!["Name", "Phone", "Gender"]
    );
    assert_eq!(inspection.sheets[0].row_count, 1);
}

// -----------------------------------------------------------------------------
// Test 13: Extended Date, Bengali Numeral & Excel Serial Edge Cases
// -----------------------------------------------------------------------------
#[test]
fn test_date_and_bengali_numeral_extended_cases() {
    // 1. Bengali numerals in dates
    assert_eq!(
        parse_flexible_date("২০২৪-০৯-২২"),
        Some("2024-09-22".to_string())
    );
    assert_eq!(
        parse_flexible_date("২২/০৯/২০২৪"),
        Some("2024-09-22".to_string())
    );
    assert_eq!(
        parse_flexible_date("২২-০৯-২০২৪"),
        Some("2024-09-22".to_string())
    );

    // 2. Invalid date strings
    assert_eq!(parse_flexible_date("2024-02-31"), None);
    assert_eq!(parse_flexible_date("32/01/2024"), None);
    assert_eq!(parse_flexible_date("invalid-date"), None);
    assert_eq!(parse_flexible_date(""), None);

    // 3. Boundary Excel date serials
    assert_eq!(
        excel_serial_to_date(25569.0),
        Some("1970-01-01".to_string())
    );
    assert_eq!(
        excel_serial_to_date(45352.0),
        Some("2024-03-01".to_string())
    );
    assert_eq!(excel_serial_to_date(-5.0), None);
    assert_eq!(excel_serial_to_date(5000000.0), None);
}

// -----------------------------------------------------------------------------
// Test 14: Phone Conflict Modes (link_guardian vs skip_row)
// -----------------------------------------------------------------------------
#[test]
fn test_phone_conflict_modes_link_guardian_and_skip_row() {
    let (_dir, mut conn) = setup_test_db();
    let admin_token = bootstrap_and_login(&mut conn);
    let temp = tempdir().unwrap();

    // Seed existing student & guardian via dispatcher
    let seed_student = json!({
        "name": "Existing Student",
        "gender": "male",
        "dob": "2008-01-01",
        "phone": "01711111111",
        "present_address": "Dhaka",
        "guardians": [{
            "is_primary": true,
            "receives_sms": true,
            "new_guardian": {
                "name": "Existing Parent",
                "relationship": "father",
                "phone": "01722222222"
            }
        }]
    });
    let (res, _) = dispatch_cmd(
        &mut conn,
        "students.create",
        json!({ "student": seed_student }),
        Some(&admin_token),
        None,
    );
    assert_eq!(res["ok"], true, "create existing student failed: {:?}", res);

    // Verify exactly 1 guardian in database
    let initial_guardians: i64 = conn
        .query_row("SELECT count(*) FROM guardians", [], |r| r.get(0))
        .unwrap();
    assert_eq!(initial_guardians, 1);

    // Part A: Import with mode "link_guardian" sharing the existing parent's phone (01722222222)
    let csv_link = temp.path().join("link_guardian_test.csv");
    let content = "Student Name,Gender,DOB,Student Phone,Guardian Name,Guardian Phone,Address\n\
                   New Sibling,female,2009-02-02,01733333333,Existing Parent,01722222222,Dhaka\n";
    fs::write(&csv_link, content).unwrap();

    let input = ExecuteImportInput {
        file_path: csv_link.display().to_string(),
        sheet_name: None,
        mappings: vec![
            ColumnMappingInput {
                source_column: "Student Name".to_string(),
                target_field: "student_name".to_string(),
            },
            ColumnMappingInput {
                source_column: "Gender".to_string(),
                target_field: "gender".to_string(),
            },
            ColumnMappingInput {
                source_column: "DOB".to_string(),
                target_field: "dob".to_string(),
            },
            ColumnMappingInput {
                source_column: "Student Phone".to_string(),
                target_field: "student_phone".to_string(),
            },
            ColumnMappingInput {
                source_column: "Guardian Name".to_string(),
                target_field: "guardian_name".to_string(),
            },
            ColumnMappingInput {
                source_column: "Guardian Phone".to_string(),
                target_field: "guardian_phone".to_string(),
            },
            ColumnMappingInput {
                source_column: "Address".to_string(),
                target_field: "present_address".to_string(),
            },
        ],
        skip_error_rows: true,
        phone_conflict_mode: "link_guardian".to_string(),
        target_batch_id: None,
        default_present_address: None,
    };

    let summary = execute_import(&conn, &input, Some("admin"), "2026-09-22T00:00:00Z").unwrap();
    assert_eq!(summary.imported_students_count, 1);
    assert_eq!(summary.imported_guardians_count, 0); // Reused existing guardian!

    // Still exactly 1 guardian in database, but 2 students
    let total_guardians: i64 = conn
        .query_row("SELECT count(*) FROM guardians", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total_guardians, 1);
    let total_students: i64 = conn
        .query_row("SELECT count(*) FROM students", [], |r| r.get(0))
        .unwrap();
    assert_eq!(total_students, 2);

    // Part B: Import with mode "skip_row" where student phone conflicts with an existing student (01711111111)
    let csv_skip = temp.path().join("skip_conflict_test.csv");
    let content_skip = "Student Name,Gender,DOB,Student Phone,Guardian Name,Guardian Phone,Address\n\
                        Conflict Student,male,2008-05-05,01711111111,Some Guardian,01755555555,Dhaka\n\
                        Valid Student,female,2008-06-06,01766666666,Another Guardian,01777777777,Dhaka\n";
    fs::write(&csv_skip, content_skip).unwrap();

    let input_skip = ExecuteImportInput {
        file_path: csv_skip.display().to_string(),
        sheet_name: None,
        mappings: input.mappings.clone(),
        skip_error_rows: true,
        phone_conflict_mode: "skip_row".to_string(),
        target_batch_id: None,
        default_present_address: None,
    };

    let summary_skip =
        execute_import(&conn, &input_skip, Some("admin"), "2026-09-22T00:00:00Z").unwrap();
    assert_eq!(summary_skip.imported_students_count, 1);
    assert_eq!(summary_skip.skipped_errors_count, 1);
    assert!(summary_skip.rejected_rows_csv_path.is_some());

    let rejected_csv_path = summary_skip.rejected_rows_csv_path.unwrap();
    let rejected_content = fs::read_to_string(&rejected_csv_path).unwrap();
    assert!(rejected_content.contains("Conflict Student"));
    assert!(rejected_content.contains("Skipped due to existing student phone conflict"));
}
