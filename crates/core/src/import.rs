use crate::phone;
use calamine::{open_workbook_auto, Data, Reader};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

// -----------------------------------------------------------------------------
// DTOs & Domain Models
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct InspectFilePayload {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SheetInspection {
    pub name: String,
    pub row_count: usize,
    pub sample_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ColumnMappingSuggestion {
    pub source_column: String,
    pub target_field: Option<String>,
    pub confidence: f32,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct FileInspectionResult {
    pub file_name: String,
    pub file_size_bytes: u64,
    pub format: String, // "xlsx", "xls", "csv"
    pub sheets: Vec<SheetInspection>,
    pub suggested_mappings: Vec<ColumnMappingSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ColumnMappingInput {
    pub source_column: String,
    pub target_field: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RowIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct RowIssue {
    pub column: Option<String>,
    pub field: Option<String>,
    pub message: String,
    pub severity: RowIssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ImportPreviewRow {
    pub row_index: usize,
    pub raw_data: HashMap<String, String>,
    pub parsed_name: Option<String>,
    pub parsed_phone: Option<String>,
    pub parsed_guardian_name: Option<String>,
    pub parsed_guardian_phone: Option<String>,
    pub issues: Vec<RowIssue>,
    pub is_valid: bool,
    pub has_warnings: bool,
    pub has_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ValidationPreviewResult {
    pub total_rows: usize,
    pub valid_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub rows_sample: Vec<ImportPreviewRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct PreviewValidationPayload {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub mappings: Vec<ColumnMappingInput>,
    pub phone_conflict_mode: Option<String>, // "link_guardian", "allow_duplicate", "skip_row"
    pub default_present_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ExecuteImportInput {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub mappings: Vec<ColumnMappingInput>,
    pub skip_error_rows: bool,
    pub phone_conflict_mode: String, // "link_guardian", "allow_duplicate", "skip_row"
    pub target_batch_id: Option<String>,
    pub default_present_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ImportSummaryResult {
    pub total_rows: usize,
    pub imported_students_count: usize,
    pub imported_guardians_count: usize,
    pub skipped_errors_count: usize,
    pub backup_path: String,
    pub rejected_rows_csv_path: Option<String>,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("Corrupt or unreadable spreadsheet: {0}")]
    CorruptFile(String),
    #[error("Invalid column mappings: {0}")]
    InvalidMappings(String),
    #[error("Parsing error: {0}")]
    ParsingFailed(String),
    #[error("Pre-import database safety backup failed: {0}")]
    BackupFailed(String),
    #[error("Import transaction aborted: {0}")]
    TransactionAborted(String),
    #[error("All rows are invalid or rejected: {0}")]
    AllRowsInvalid(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl From<rusqlite::Error> for ImportError {
    fn from(err: rusqlite::Error) -> Self {
        ImportError::Sql(err.to_string())
    }
}

// -----------------------------------------------------------------------------
// Normalization & Parsing Helpers
// -----------------------------------------------------------------------------

/// Replaces Bengali numerals (`০-৯`) with ASCII numerals (`0-9`).
pub fn normalize_bengali_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '০' => '0',
            '১' => '1',
            '২' => '2',
            '৩' => '3',
            '৪' => '4',
            '৫' => '5',
            '৬' => '6',
            '৭' => '7',
            '৮' => '8',
            '৯' => '9',
            other => other,
        })
        .collect()
}

/// Converts Excel numeric serial (days since 1899-12-30) into ISO `YYYY-MM-DD`.
pub fn excel_serial_to_date(serial: f64) -> Option<String> {
    if serial < 1.0 || serial > 150_000.0 {
        return None;
    }
    // Unix epoch is 1970-01-01, exactly 25,569 days after 1899-12-30.
    let unix_days = serial.floor() as i64 - 25569;
    let unix_secs = unix_days.checked_mul(86400)?;
    let ts = jiff::Timestamp::from_second(unix_secs).ok()?;
    let date = ts.to_zoned(jiff::tz::TimeZone::UTC).date();
    Some(format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month(),
        date.day()
    ))
}

/// Parses flexible date formats (`YYYY-MM-DD`, `DD/MM/YYYY`, `DD-MM-YYYY`, `MM/DD/YYYY`, and Excel serials) into ISO `YYYY-MM-DD`.
pub fn parse_flexible_date(raw: &str) -> Option<String> {
    let clean = normalize_bengali_digits(raw).trim().to_string();
    if clean.is_empty() {
        return None;
    }

    // 1. Try numeric Excel serial
    if let Ok(num) = clean.parse::<f64>() {
        if let Some(d) = excel_serial_to_date(num) {
            return Some(d);
        }
    }

    // 2. Try string date patterns separated by `-`, `/`, or `.`
    let delimiters = ['-', '/', '.'];
    let parts: Vec<&str> = clean
        .split(|c| delimiters.contains(&c))
        .map(str::trim)
        .collect();

    if parts.len() != 3 {
        return None;
    }

    let p0: u32 = parts[0].parse().ok()?;
    let p1: u32 = parts[1].parse().ok()?;
    let p2: u32 = parts[2].parse().ok()?;

    let is_valid_ymd = |year: u32, month: u32, day: u32| -> bool {
        if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) || day < 1 {
            return false;
        }
        let max_days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let is_leap = (year.is_multiple_of(4) && !year.is_multiple_of(100))
                    || year.is_multiple_of(400);
                if is_leap {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };
        day <= max_days
    };

    if parts[0].len() == 4 {
        // YYYY-MM-DD
        let year = p0;
        let month = p1;
        let day = p2;
        if is_valid_ymd(year, month, day) {
            return Some(format!("{:04}-{:02}-{:02}", year, month, day));
        }
    } else if parts[2].len() == 4 {
        // Could be DD/MM/YYYY or MM/DD/YYYY
        let year = p2;
        let (day, month) = if p0 > 12 && p1 <= 12 {
            // Unambiguous DD/MM/YYYY
            (p0, p1)
        } else if p1 > 12 && p0 <= 12 {
            // Unambiguous MM/DD/YYYY
            (p1, p0)
        } else {
            // Default to Bangladeshi standard DD/MM/YYYY
            (p0, p1)
        };

        if is_valid_ymd(year, month, day) {
            return Some(format!("{:04}-{:02}-{:02}", year, month, day));
        }
    }

    None
}

/// Normalizes gender variations into `"male"`, `"female"`, or `"other"`.
pub fn parse_gender(raw: &str) -> Option<String> {
    let clean = raw.trim().to_lowercase();
    match clean.as_str() {
        "male" | "m" | "boy" | "ছেলে" | "পুরুষ" => Some("male".to_string()),
        "female" | "f" | "girl" | "মেয়ে" | "মহিলা" => {
            Some("female".to_string())
        }
        "other" | "o" | "অন্যান্য" | "তৃতীয় লিঙ্গ" => {
            Some("other".to_string())
        }
        _ => None,
    }
}

/// Normalizes relationship variations into one of the allowed SQLite CHECK constraint values:
/// `'father'`, `'mother'`, `'uncle'`, `'brother'`, `'sister'`, `'legal_guardian'`, or `'other'`.
pub fn normalize_relationship(raw: &str) -> &'static str {
    let clean = raw.trim().to_lowercase();
    match clean.as_str() {
        "father" | "dad" | "baba" | "pita" | "বাবা" | "পিতা" => "father",
        "mother" | "mom" | "ma" | "mata" | "মা" | "মাতা" => "mother",
        "uncle" | "chacha" | "mama" | "কাকা" | "চাচা" | "মামা" => "uncle",
        "brother" | "bhai" | "ভাই" => "brother",
        "sister" | "bon" | "বোন" => "sister",
        "legal_guardian" | "guardian" | "অভিভাবক" => "legal_guardian",
        "other" | "অন্যান্য" => "other",
        _ => "legal_guardian",
    }
}

// -----------------------------------------------------------------------------
// Column Auto-Mapping Heuristics
// -----------------------------------------------------------------------------

struct FieldAlias {
    target_field: &'static str,
    aliases: &'static [&'static str],
}

const FIELD_ALIASES: &[FieldAlias] = &[
    FieldAlias {
        target_field: "student_name",
        aliases: &[
            "name",
            "student_name",
            "student name",
            "student",
            "full_name",
            "full name",
            "applicant_name",
            "নাম",
            "ছাত্র/ছাত্রীর নাম",
            "ছাত্রের নাম",
            "ছাত্রীর নাম",
            "শিক্ষার্থীর নাম",
        ],
    },
    FieldAlias {
        target_field: "student_name_bn",
        aliases: &[
            "name_bn",
            "student_name_bn",
            "bangla_name",
            "bengali_name",
            "নাম (বাংলা)",
            "বাংলা নাম",
        ],
    },
    FieldAlias {
        target_field: "gender",
        aliases: &["gender", "sex", "লিঙ্গ", "ছেলে/মেয়ে"],
    },
    FieldAlias {
        target_field: "dob",
        aliases: &[
            "dob",
            "birth_date",
            "birthdate",
            "date_of_birth",
            "date of birth",
            "জন্ম তারিখ",
            "জন্মতারিখ",
            "জন্মদিন",
        ],
    },
    FieldAlias {
        target_field: "student_phone",
        aliases: &[
            "phone",
            "student_phone",
            "mobile",
            "student_mobile",
            "contact",
            "student_contact",
            "মোবাইল",
            "ফোন",
            "শিক্ষার্থীর মোবাইল",
            "ছাত্রের মোবাইল",
            "ছাত্রীর মোবাইল",
        ],
    },
    FieldAlias {
        target_field: "emergency_phone",
        aliases: &[
            "emergency_phone",
            "emergency_mobile",
            "emergency_contact",
            "জরুরী মোবাইল",
            "জরুরি যোগাযোগ",
            "জরুরী ফোন",
        ],
    },
    FieldAlias {
        target_field: "present_address",
        aliases: &[
            "address",
            "present_address",
            "current_address",
            "ঠিকানা",
            "বর্তমান ঠিকানা",
            "বাসার ঠিকানা",
        ],
    },
    FieldAlias {
        target_field: "permanent_address",
        aliases: &["permanent_address", "স্থায়ী ঠিকানা", "গ্রামের ঠিকানা"],
    },
    FieldAlias {
        target_field: "guardian_name",
        aliases: &[
            "guardian_name",
            "guardian",
            "father_name",
            "father",
            "mother_name",
            "mother",
            "parent_name",
            "parent",
            "অভিভাবক",
            "অভিভাবকের নাম",
            "পিতা",
            "পিতার নাম",
            "বাবার নাম",
            "মাতা",
            "মাতার নাম",
            "মা",
        ],
    },
    FieldAlias {
        target_field: "guardian_phone",
        aliases: &[
            "guardian_phone",
            "guardian_mobile",
            "parent_phone",
            "parent_mobile",
            "father_phone",
            "father_mobile",
            "mother_phone",
            "mother_mobile",
            "অভিভাবকের মোবাইল",
            "বাবার মোবাইল",
            "পিতার মোবাইল",
            "মায়ের মোবাইল",
            "অভিভাবকের ফোন",
        ],
    },
    FieldAlias {
        target_field: "guardian_relationship",
        aliases: &[
            "relationship",
            "guardian_relation",
            "relation",
            "সম্পর্ক",
            "অভিভাবকের সম্পর্ক",
        ],
    },
    FieldAlias {
        target_field: "student_code",
        aliases: &[
            "student_code",
            "student_id",
            "student code",
            "student id",
            "id",
            "code",
            "স্টুডেন্ট কোড",
            "আইডি",
        ],
    },
    FieldAlias {
        target_field: "class_level",
        aliases: &[
            "class",
            "class_level",
            "grade",
            "standard",
            "class/group",
            "শ্রেণী",
            "ক্লাস",
        ],
    },
    FieldAlias {
        target_field: "current_school_roll",
        aliases: &[
            "school_roll",
            "current_school_roll",
            "roll",
            "roll_no",
            "roll number",
            "রোল",
            "রোল নং",
        ],
    },
    FieldAlias {
        target_field: "blood_group",
        aliases: &["blood_group", "blood", "blood group", "রক্তের গ্রুপ"],
    },
    FieldAlias {
        target_field: "religion",
        aliases: &["religion", "ধর্ম"],
    },
    FieldAlias {
        target_field: "email",
        aliases: &["email", "e-mail", "student_email", "ইমেইল"],
    },
    FieldAlias {
        target_field: "previous_school",
        aliases: &[
            "previous_school",
            "school",
            "institution",
            "college",
            "পূর্ববর্তী স্কুল",
            "বিদ্যালয়",
        ],
    },
];

fn clean_token(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '/')
        .collect()
}

pub fn calculate_header_match(header: &str) -> (Option<&'static str>, f32) {
    let clean_hdr = clean_token(header);
    if clean_hdr.is_empty() {
        return (None, 0.0);
    }

    let mut best_field: Option<&'static str> = None;
    let mut best_score: f32 = 0.0;

    for entry in FIELD_ALIASES {
        for &alias in entry.aliases {
            let clean_alias = clean_token(alias);
            if clean_hdr == clean_alias {
                return (Some(entry.target_field), 1.0);
            }
            if clean_hdr.contains(&clean_alias) || clean_alias.contains(&clean_hdr) {
                let ratio =
                    (clean_alias.len() as f32) / (clean_hdr.len().max(clean_alias.len()) as f32);
                let score = 0.80 + 0.15 * ratio;
                if score > best_score {
                    best_score = score;
                    best_field = Some(entry.target_field);
                }
            }
        }
    }

    if best_score >= 0.65 {
        (best_field, best_score)
    } else {
        (None, 0.0)
    }
}

pub fn suggest_column_mappings(
    headers: &[String],
    sample_rows: &[Vec<String>],
) -> Vec<ColumnMappingSuggestion> {
    let mut assigned_targets: HashSet<&'static str> = HashSet::new();
    let mut suggestions = Vec::new();

    // First pass: Calculate candidate matches
    let mut scored: Vec<(usize, Option<&'static str>, f32)> = headers
        .iter()
        .enumerate()
        .map(|(idx, h)| {
            let (target, score) = calculate_header_match(h);
            (idx, target, score)
        })
        .collect();

    // Sort descending by score to give best matches first priority
    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut mapping_by_idx: HashMap<usize, (Option<String>, f32)> = HashMap::new();
    for (idx, target_opt, score) in scored {
        if let Some(target) = target_opt {
            if !assigned_targets.contains(target) && score >= 0.65 {
                assigned_targets.insert(target);
                mapping_by_idx.insert(idx, (Some(target.to_string()), score));
                continue;
            }
        }
        mapping_by_idx.insert(idx, (None, 0.0));
    }

    for (idx, header) in headers.iter().enumerate() {
        let (target, confidence) = mapping_by_idx.remove(&idx).unwrap_or((None, 0.0));
        let sample_values: Vec<String> = sample_rows
            .iter()
            .filter_map(|row| row.get(idx))
            .filter(|v| !v.trim().is_empty())
            .take(3)
            .cloned()
            .collect();

        suggestions.push(ColumnMappingSuggestion {
            source_column: header.clone(),
            target_field: target,
            confidence,
            sample_values,
        });
    }

    suggestions
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(f) => {
            if let Some(d) = excel_serial_to_date(f.as_f64()) {
                d
            } else {
                f.to_string()
            }
        }
        Data::DateTimeIso(s) => s.trim().to_string(),
        Data::DurationIso(s) => s.trim().to_string(),
        Data::Error(e) => format!("{:?}", e),
    }
}

// -----------------------------------------------------------------------------
// Spreadsheet & CSV File Inspection (AC-1, AC-2)
// -----------------------------------------------------------------------------

pub fn inspect_file(file_path: &Path) -> Result<FileInspectionResult, ImportError> {
    if !file_path.exists() {
        return Err(ImportError::FileNotFound(file_path.display().to_string()));
    }

    let metadata = fs::metadata(file_path).map_err(|e| ImportError::CorruptFile(e.to_string()))?;
    let file_size_bytes = metadata.len();
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "csv" => {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_path(file_path)
                .map_err(|e| ImportError::CorruptFile(e.to_string()))?;

            let mut all_rows: Vec<Vec<String>> = Vec::new();
            for result in reader.records() {
                let record = result.map_err(|e| ImportError::CorruptFile(e.to_string()))?;
                let row: Vec<String> = record.iter().map(|s| s.trim().to_string()).collect();
                if row.iter().any(|c| !c.is_empty()) {
                    all_rows.push(row);
                }
            }

            if all_rows.is_empty() {
                return Err(ImportError::CorruptFile(
                    "CSV file is completely empty".to_string(),
                ));
            }

            let headers = all_rows[0].clone();
            let sample_rows: Vec<Vec<String>> = all_rows.iter().skip(1).take(5).cloned().collect();
            let row_count = all_rows.len().saturating_sub(1);

            let sheet = SheetInspection {
                name: "Sheet1".to_string(),
                row_count,
                sample_headers: headers.clone(),
            };

            let suggested_mappings = suggest_column_mappings(&headers, &sample_rows);

            Ok(FileInspectionResult {
                file_name,
                file_size_bytes,
                format: "csv".to_string(),
                sheets: vec![sheet],
                suggested_mappings,
            })
        }
        "xlsx" | "xls" | "ods" | "xlsb" => {
            let mut workbook = open_workbook_auto(file_path)
                .map_err(|e| ImportError::CorruptFile(e.to_string()))?;

            let sheet_names = workbook.sheet_names().to_vec();
            if sheet_names.is_empty() {
                return Err(ImportError::CorruptFile(
                    "Workbook contains no sheets".to_string(),
                ));
            }

            let mut sheets = Vec::new();
            let mut first_sheet_headers = Vec::new();
            let mut first_sheet_samples = Vec::new();

            for (idx, name) in sheet_names.iter().enumerate() {
                if let Ok(range) = workbook.worksheet_range(name) {
                    let mut rows_iter = range.rows();
                    // Find header row (first non-empty row)
                    let mut headers: Vec<String> = Vec::new();
                    for row in rows_iter.by_ref() {
                        let non_empty = row.iter().any(|c| !matches!(c, Data::Empty));
                        if non_empty {
                            headers = row.iter().map(cell_to_string).collect();
                            break;
                        }
                    }

                    let mut sample_rows = Vec::new();
                    let mut data_row_count = 0;
                    for row in rows_iter {
                        if row.iter().any(|c| !matches!(c, Data::Empty)) {
                            data_row_count += 1;
                            if sample_rows.len() < 5 {
                                sample_rows.push(row.iter().map(cell_to_string).collect());
                            }
                        }
                    }

                    if idx == 0 {
                        first_sheet_headers = headers.clone();
                        first_sheet_samples = sample_rows.clone();
                    }

                    sheets.push(SheetInspection {
                        name: name.clone(),
                        row_count: data_row_count,
                        sample_headers: headers,
                    });
                }
            }

            let suggested_mappings =
                suggest_column_mappings(&first_sheet_headers, &first_sheet_samples);

            Ok(FileInspectionResult {
                file_name,
                file_size_bytes,
                format: ext,
                sheets,
                suggested_mappings,
            })
        }
        other => Err(ImportError::UnsupportedFormat(other.to_string())),
    }
}

// -----------------------------------------------------------------------------
// Spreadsheet Data Extraction
// -----------------------------------------------------------------------------

fn read_sheet_data(
    file_path: &Path,
    sheet_name: Option<&str>,
) -> Result<(Vec<String>, Vec<Vec<String>>), ImportError> {
    if !file_path.exists() {
        return Err(ImportError::FileNotFound(file_path.display().to_string()));
    }

    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "csv" => {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_path(file_path)
                .map_err(|e| ImportError::CorruptFile(e.to_string()))?;

            let mut all_rows: Vec<Vec<String>> = Vec::new();
            for result in reader.records() {
                let record = result.map_err(|e| ImportError::CorruptFile(e.to_string()))?;
                let row: Vec<String> = record.iter().map(|s| s.trim().to_string()).collect();
                if row.iter().any(|c| !c.is_empty()) {
                    all_rows.push(row);
                }
            }

            if all_rows.is_empty() {
                return Err(ImportError::CorruptFile("CSV is empty".to_string()));
            }

            let headers = all_rows[0].clone();
            let data_rows = all_rows.into_iter().skip(1).collect();
            Ok((headers, data_rows))
        }
        "xlsx" | "xls" | "ods" | "xlsb" => {
            let mut workbook = open_workbook_auto(file_path)
                .map_err(|e| ImportError::CorruptFile(e.to_string()))?;

            let target_sheet =
                if let Some(name) = sheet_name {
                    name.to_string()
                } else {
                    workbook.sheet_names().first().cloned().ok_or_else(|| {
                        ImportError::CorruptFile("No sheets in workbook".to_string())
                    })?
                };

            let range = workbook.worksheet_range(&target_sheet).map_err(|e| {
                ImportError::CorruptFile(format!("Sheet '{}' error: {}", target_sheet, e))
            })?;

            let mut rows_iter = range.rows();
            let mut headers: Vec<String> = Vec::new();

            for row in rows_iter.by_ref() {
                if row.iter().any(|c| !matches!(c, Data::Empty)) {
                    headers = row.iter().map(cell_to_string).collect();
                    break;
                }
            }

            let mut data_rows: Vec<Vec<String>> = Vec::new();
            for row in rows_iter {
                if row.iter().any(|c| !matches!(c, Data::Empty)) {
                    data_rows.push(row.iter().map(cell_to_string).collect());
                }
            }

            Ok((headers, data_rows))
        }
        other => Err(ImportError::UnsupportedFormat(other.to_string())),
    }
}

// -----------------------------------------------------------------------------
// In-Memory Validation Matrix (AC-3, AC-4, AC-5)
// -----------------------------------------------------------------------------

pub fn preview_validation(
    conn: &Connection,
    payload: &PreviewValidationPayload,
) -> Result<ValidationPreviewResult, ImportError> {
    let (headers, data_rows) =
        read_sheet_data(Path::new(&payload.file_path), payload.sheet_name.as_deref())?;

    // Create column-index lookup
    let header_to_idx: HashMap<&str, usize> = headers
        .iter()
        .enumerate()
        .map(|(idx, h)| (h.as_str(), idx))
        .collect();

    // Map target field -> column index
    let mut field_to_col_idx: HashMap<String, usize> = HashMap::new();
    for mapping in &payload.mappings {
        if let Some(&idx) = header_to_idx.get(mapping.source_column.as_str()) {
            field_to_col_idx.insert(mapping.target_field.clone(), idx);
        }
    }

    let conflict_mode = payload
        .phone_conflict_mode
        .as_deref()
        .unwrap_or("link_guardian");

    // Pre-pass: Count normalized guardian phones for sibling grouping (AC-4)
    let mut guardian_phone_counts: HashMap<String, usize> = HashMap::new();
    for row in &data_rows {
        if let Some(&col_idx) = field_to_col_idx.get("guardian_phone") {
            if let Some(val) = row.get(col_idx) {
                let digits = normalize_bengali_digits(val);
                if let Ok(norm) = phone::normalize_phone(&digits) {
                    *guardian_phone_counts.entry(norm).or_insert(0) += 1;
                }
            }
        }
    }

    let mut preview_rows: Vec<ImportPreviewRow> = Vec::new();
    let mut valid_count = 0;
    let mut warning_count = 0;
    let mut error_count = 0;

    for (row_idx, row) in data_rows.iter().enumerate() {
        let mut raw_data = HashMap::new();
        for (col_idx, val) in row.iter().enumerate() {
            if let Some(col_name) = headers.get(col_idx) {
                raw_data.insert(col_name.clone(), val.clone());
            }
        }

        let mut issues: Vec<RowIssue> = Vec::new();

        let get_val = |target: &str| -> Option<String> {
            field_to_col_idx
                .get(target)
                .and_then(|&idx| row.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        // 1. Student Name (Required)
        let parsed_name = get_val("student_name");
        if parsed_name.is_none() {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("student_name")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("student_name".to_string()),
                message: "Student name is required".to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 2. Gender (Required)
        let gender_raw = get_val("gender");
        let parsed_gender = gender_raw.as_deref().and_then(parse_gender);
        if parsed_gender.is_none() {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("gender")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("gender".to_string()),
                message: "Valid gender (male, female, other) is required".to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 3. Date of Birth (Required)
        let dob_raw = get_val("dob");
        let parsed_dob = dob_raw.as_deref().and_then(parse_flexible_date);
        if parsed_dob.is_none() {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("dob")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("dob".to_string()),
                message: "Valid date of birth (e.g. YYYY-MM-DD or DD/MM/YYYY) is required"
                    .to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 4. Student Phone (Required)
        let student_phone_raw = get_val("student_phone");
        let mut parsed_phone: Option<String> = None;
        if let Some(ref raw_p) = student_phone_raw {
            let digits = normalize_bengali_digits(raw_p);
            match phone::normalize_phone(&digits) {
                Ok(norm) => {
                    parsed_phone = Some(norm.clone());
                    // Conflict check in DB
                    if let Ok(Some(conflict)) = phone::check_phone_conflict(conn, &norm, None, None)
                    {
                        issues.push(RowIssue {
                            column: field_to_col_idx
                                .get("student_phone")
                                .and_then(|&i| headers.get(i))
                                .cloned(),
                            field: Some("student_phone".to_string()),
                            message: format!(
                                "Student phone conflict: already registered to {} '{}'",
                                conflict.existing_owner_type, conflict.existing_owner_name
                            ),
                            severity: RowIssueSeverity::Warning,
                        });
                    }
                }
                Err(err) => {
                    issues.push(RowIssue {
                        column: field_to_col_idx
                            .get("student_phone")
                            .and_then(|&i| headers.get(i))
                            .cloned(),
                        field: Some("student_phone".to_string()),
                        message: format!("Invalid student phone: {}", err),
                        severity: RowIssueSeverity::Error,
                    });
                }
            }
        } else {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("student_phone")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("student_phone".to_string()),
                message: "Student phone is required".to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 5. Guardian Name (Required)
        let parsed_guardian_name = get_val("guardian_name");
        if parsed_guardian_name.is_none() {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("guardian_name")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("guardian_name".to_string()),
                message: "Guardian name is required".to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 6. Guardian Phone (Required)
        let guardian_phone_raw = get_val("guardian_phone");
        let mut parsed_guardian_phone: Option<String> = None;
        if let Some(ref raw_p) = guardian_phone_raw {
            let digits = normalize_bengali_digits(raw_p);
            match phone::normalize_phone(&digits) {
                Ok(norm) => {
                    parsed_guardian_phone = Some(norm.clone());

                    // Intra-file sibling grouping flag
                    if let Some(&cnt) = guardian_phone_counts.get(&norm) {
                        if cnt > 1 {
                            issues.push(RowIssue {
                                column: field_to_col_idx.get("guardian_phone").and_then(|&i| headers.get(i)).cloned(),
                                field: Some("guardian_phone".to_string()),
                                message: "Sibling student detected: shares guardian phone with other row(s)".to_string(),
                                severity: RowIssueSeverity::Warning,
                            });
                        }
                    }

                    // Conflict check in DB
                    if let Ok(Some(conflict)) = phone::check_phone_conflict(conn, &norm, None, None)
                    {
                        let msg = if conflict_mode == "link_guardian" {
                            format!(
                                "Guardian phone matches existing {}: '{}'; will link to existing family",
                                conflict.existing_owner_type, conflict.existing_owner_name
                            )
                        } else {
                            format!(
                                "Guardian phone already registered to {} '{}'",
                                conflict.existing_owner_type, conflict.existing_owner_name
                            )
                        };
                        issues.push(RowIssue {
                            column: field_to_col_idx
                                .get("guardian_phone")
                                .and_then(|&i| headers.get(i))
                                .cloned(),
                            field: Some("guardian_phone".to_string()),
                            message: msg,
                            severity: RowIssueSeverity::Warning,
                        });
                    }
                }
                Err(err) => {
                    issues.push(RowIssue {
                        column: field_to_col_idx
                            .get("guardian_phone")
                            .and_then(|&i| headers.get(i))
                            .cloned(),
                        field: Some("guardian_phone".to_string()),
                        message: format!("Invalid guardian phone: {}", err),
                        severity: RowIssueSeverity::Error,
                    });
                }
            }
        } else {
            issues.push(RowIssue {
                column: field_to_col_idx
                    .get("guardian_phone")
                    .and_then(|&i| headers.get(i))
                    .cloned(),
                field: Some("guardian_phone".to_string()),
                message: "Guardian phone is required".to_string(),
                severity: RowIssueSeverity::Error,
            });
        }

        // 7. Present Address (Required, or fallback to default)
        let addr = get_val("present_address");
        if addr.is_none() {
            if payload.default_present_address.is_some() {
                issues.push(RowIssue {
                    column: field_to_col_idx
                        .get("present_address")
                        .and_then(|&i| headers.get(i))
                        .cloned(),
                    field: Some("present_address".to_string()),
                    message: "Address missing; using default center address".to_string(),
                    severity: RowIssueSeverity::Warning,
                });
            } else {
                issues.push(RowIssue {
                    column: field_to_col_idx
                        .get("present_address")
                        .and_then(|&i| headers.get(i))
                        .cloned(),
                    field: Some("present_address".to_string()),
                    message: "Present address is required (or set default present address)"
                        .to_string(),
                    severity: RowIssueSeverity::Error,
                });
            }
        }

        let has_errors = issues.iter().any(|i| i.severity == RowIssueSeverity::Error);
        let has_warnings = issues
            .iter()
            .any(|i| i.severity == RowIssueSeverity::Warning);
        let is_valid = !has_errors;

        if has_errors {
            error_count += 1;
        } else if has_warnings {
            warning_count += 1;
            valid_count += 1;
        } else {
            valid_count += 1;
        }

        preview_rows.push(ImportPreviewRow {
            row_index: row_idx + 1,
            raw_data,
            parsed_name,
            parsed_phone,
            parsed_guardian_name,
            parsed_guardian_phone,
            issues,
            is_valid,
            has_warnings,
            has_errors,
        });
    }

    Ok(ValidationPreviewResult {
        total_rows: data_rows.len(),
        valid_count,
        warning_count,
        error_count,
        rows_sample: preview_rows.into_iter().take(100).collect(),
    })
}

// -----------------------------------------------------------------------------
// Pre-Import Online Backup Snapshot (AC-6)
// -----------------------------------------------------------------------------

pub fn create_pre_import_backup(
    conn: &Connection,
    backup_dir: Option<&Path>,
) -> Result<PathBuf, ImportError> {
    let dir = if let Some(d) = backup_dir {
        d.to_path_buf()
    } else if let Ok(Some(cfg)) = crate::storage::load_global_config() {
        PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("backups")
    } else {
        let dev_db = crate::db::resolve_dev_db_path()
            .map_err(|e| ImportError::BackupFailed(e.to_string()))?;
        dev_db.parent().unwrap_or(Path::new(".")).join("backups")
    };

    fs::create_dir_all(&dir).map_err(|e| {
        ImportError::BackupFailed(format!("Failed to create backup directory: {}", e))
    })?;

    let timestamp = crate::db::now_utc_iso().replace(':', "-");
    let backup_file = dir.join(format!("pre-import-{}.db", timestamp));

    let mut dest_conn = Connection::open(&backup_file).map_err(|e| {
        ImportError::BackupFailed(format!("Failed to open destination backup file: {}", e))
    })?;

    let backup = rusqlite::backup::Backup::new(conn, &mut dest_conn).map_err(|e| {
        ImportError::BackupFailed(format!("Failed to initialize online backup: {}", e))
    })?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(1), None)
        .map_err(|e| ImportError::BackupFailed(format!("Online backup step failed: {}", e)))?;

    drop(backup);
    drop(dest_conn);

    if !backup_file.exists() || fs::metadata(&backup_file).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(ImportError::BackupFailed(
            "Backup file was not created or is empty".to_string(),
        ));
    }

    Ok(backup_file)
}

// -----------------------------------------------------------------------------
// Rejected Rows CSV Generator (AC-8)
// -----------------------------------------------------------------------------

pub fn generate_rejected_rows_csv(
    headers: &[String],
    rows: &[Vec<String>],
    reasons: &[String],
    export_dir: Option<&Path>,
) -> Result<PathBuf, ImportError> {
    let dir = if let Some(d) = export_dir {
        d.to_path_buf()
    } else if let Ok(Some(cfg)) = crate::storage::load_global_config() {
        PathBuf::from(&cfg.data_dir)
            .join(&cfg.institution_id)
            .join("exports")
    } else {
        let dev_db = crate::db::resolve_dev_db_path()
            .map_err(|e| ImportError::ParsingFailed(e.to_string()))?;
        dev_db.parent().unwrap_or(Path::new(".")).join("exports")
    };

    fs::create_dir_all(&dir)
        .map_err(|e| ImportError::ParsingFailed(format!("Failed to create exports dir: {}", e)))?;

    let timestamp = crate::db::now_utc_iso().replace(':', "-");
    let file_path = dir.join(format!("rejected-import-{}.csv", timestamp));

    let file = File::create(&file_path)
        .map_err(|e| ImportError::ParsingFailed(format!("Failed to create rejected CSV: {}", e)))?;
    let mut writer = csv::WriterBuilder::new().from_writer(file);

    // Write header row with appended _import_error column
    let mut out_headers = headers.to_vec();
    out_headers.push("_import_error".to_string());
    writer
        .write_record(&out_headers)
        .map_err(|e| ImportError::ParsingFailed(e.to_string()))?;

    for (row, reason) in rows.iter().zip(reasons.iter()) {
        let mut out_row = row.to_vec();
        out_row.push(reason.clone());
        writer
            .write_record(&out_row)
            .map_err(|e| ImportError::ParsingFailed(e.to_string()))?;
    }

    writer
        .flush()
        .map_err(|e| ImportError::ParsingFailed(e.to_string()))?;

    Ok(file_path)
}

// -----------------------------------------------------------------------------
// Atomic Transaction Execution (AC-6, AC-7, AC-8, AC-9)
// -----------------------------------------------------------------------------

pub fn execute_import(
    conn: &Connection,
    input: &ExecuteImportInput,
    user_id: Option<&str>,
    current_utc: &str,
) -> Result<ImportSummaryResult, ImportError> {
    // 1. Read sheet data
    let (headers, data_rows) =
        read_sheet_data(Path::new(&input.file_path), input.sheet_name.as_deref())?;

    let header_to_idx: HashMap<&str, usize> = headers
        .iter()
        .enumerate()
        .map(|(idx, h)| (h.as_str(), idx))
        .collect();

    let mut field_to_col_idx: HashMap<String, usize> = HashMap::new();
    for mapping in &input.mappings {
        if let Some(&idx) = header_to_idx.get(mapping.source_column.as_str()) {
            field_to_col_idx.insert(mapping.target_field.clone(), idx);
        }
    }

    // 2. Validate all rows in memory
    let preview_input = PreviewValidationPayload {
        file_path: input.file_path.clone(),
        sheet_name: input.sheet_name.clone(),
        mappings: input.mappings.clone(),
        phone_conflict_mode: Some(input.phone_conflict_mode.clone()),
        default_present_address: input.default_present_address.clone(),
    };
    let validation_result = preview_validation(conn, &preview_input)?;

    if !input.skip_error_rows && validation_result.error_count > 0 {
        return Err(ImportError::AllRowsInvalid(format!(
            "Spreadsheet has {} error row(s) and strict rollback mode was selected",
            validation_result.error_count
        )));
    }

    if validation_result.valid_count == 0 {
        return Err(ImportError::AllRowsInvalid(
            "No valid rows found to import in spreadsheet".to_string(),
        ));
    }

    // 3. Pre-import database safety backup snapshot (AC-6)
    let backup_path = create_pre_import_backup(conn, None)?;

    // 4. Begin savepoint for atomic transaction guarantees (AC-7)
    conn.execute("SAVEPOINT import_sp;", [])?;

    let mut imported_students = 0;
    let mut imported_guardians = 0;
    let mut rejected_data_rows = Vec::new();
    let mut rejected_reasons = Vec::new();
    let mut imported_student_ids = Vec::new();

    // Cache to deduplicate guardians by normalized phone (AC-4)
    let mut guardian_phone_to_id: HashMap<String, String> = HashMap::new();

    for (row_idx, row) in data_rows.iter().enumerate() {
        let get_val = |target: &str| -> Option<String> {
            field_to_col_idx
                .get(target)
                .and_then(|&idx| row.get(idx))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        let student_name = get_val("student_name");
        let gender = get_val("gender").as_deref().and_then(parse_gender);
        let dob = get_val("dob").as_deref().and_then(parse_flexible_date);
        let phone_raw = get_val("student_phone");
        let guardian_name = get_val("guardian_name");
        let guardian_phone_raw = get_val("guardian_phone");
        let addr = get_val("present_address").or_else(|| input.default_present_address.clone());

        // Validate essentials
        let mut row_errors = Vec::new();
        if student_name.is_none() {
            row_errors.push("Missing student name".to_string());
        }
        if gender.is_none() {
            row_errors.push("Invalid gender".to_string());
        }
        if dob.is_none() {
            row_errors.push("Invalid date of birth".to_string());
        }
        if addr.is_none() {
            row_errors.push("Missing present address".to_string());
        }

        let student_phone_norm = match phone_raw.as_deref() {
            Some(p) => {
                let clean = normalize_bengali_digits(p);
                phone::normalize_phone(&clean).ok()
            }
            None => None,
        };
        if student_phone_norm.is_none() {
            row_errors.push("Invalid student phone".to_string());
        }

        let guardian_phone_norm = match guardian_phone_raw.as_deref() {
            Some(p) => {
                let clean = normalize_bengali_digits(p);
                phone::normalize_phone(&clean).ok()
            }
            None => None,
        };
        if guardian_name.is_none() {
            row_errors.push("Missing guardian name".to_string());
        }
        if guardian_phone_norm.is_none() {
            row_errors.push("Invalid guardian phone".to_string());
        }

        // Check if phone conflict mode says skip_row
        if input.phone_conflict_mode == "skip_row" {
            if let Some(ref sp) = student_phone_norm {
                if let Ok(Some(_)) = phone::check_phone_conflict(conn, sp, None, None) {
                    row_errors.push("Skipped due to existing student phone conflict".to_string());
                }
            }
            if let Some(ref gp) = guardian_phone_norm {
                if let Ok(Some(_)) = phone::check_phone_conflict(conn, gp, None, None) {
                    row_errors.push("Skipped due to existing guardian phone conflict".to_string());
                }
            }
        }

        if !row_errors.is_empty() {
            if input.skip_error_rows {
                rejected_data_rows.push(row.clone());
                rejected_reasons.push(row_errors.join("; "));
                continue;
            } else {
                let _ = conn.execute("ROLLBACK TO import_sp;", []);
                let _ = conn.execute("RELEASE import_sp;", []);
                return Err(ImportError::TransactionAborted(format!(
                    "Row {} has errors: {}",
                    row_idx + 1,
                    row_errors.join(", ")
                )));
            }
        }

        let s_name = student_name.unwrap();
        let s_gender = gender.unwrap();
        let s_dob = dob.unwrap();
        let s_phone = student_phone_norm.unwrap();
        let g_name = guardian_name.unwrap();
        let g_phone = guardian_phone_norm.unwrap();
        let s_address = addr.unwrap();

        // 5. Guardian Resolution & Sibling Deduplication (AC-4)
        let guardian_id = if let Some(existing_gid) = guardian_phone_to_id.get(&g_phone) {
            // Sibling row in same spreadsheet!
            existing_gid.clone()
        } else if input.phone_conflict_mode == "link_guardian" {
            // Check if existing guardian in DB has this phone
            let existing_db_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM guardians WHERE phone_norm = ?1 LIMIT 1",
                    rusqlite::params![g_phone],
                    |r| r.get(0),
                )
                .ok();

            if let Some(gid) = existing_db_id {
                guardian_phone_to_id.insert(g_phone.clone(), gid.clone());
                gid
            } else {
                // Create new guardian
                let new_gid = Uuid::now_v7().to_string();
                let g_rel = get_val("guardian_relationship")
                    .map(|s| normalize_relationship(&s))
                    .unwrap_or("legal_guardian");
                conn.execute(
                    "INSERT INTO guardians (
                        id, name, name_bn, relationship, occupation, monthly_income_poisha,
                        nid, phone_norm, phone2_norm, whatsapp_number, email, present_address,
                        permanent_address, notes, created_at, updated_at
                    ) VALUES (?1, ?2, NULL, ?3, NULL, NULL, NULL, ?4, NULL, NULL, NULL, ?5, NULL, NULL, ?6, ?6)",
                    rusqlite::params![new_gid, g_name, g_rel, g_phone, s_address, current_utc],
                )?;
                guardian_phone_to_id.insert(g_phone.clone(), new_gid.clone());
                imported_guardians += 1;
                new_gid
            }
        } else {
            // Create new guardian
            let new_gid = Uuid::now_v7().to_string();
            let g_rel = get_val("guardian_relationship")
                .map(|s| normalize_relationship(&s))
                .unwrap_or("legal_guardian");
            conn.execute(
                "INSERT INTO guardians (
                    id, name, name_bn, relationship, occupation, monthly_income_poisha,
                    nid, phone_norm, phone2_norm, whatsapp_number, email, present_address,
                    permanent_address, notes, created_at, updated_at
                ) VALUES (?1, ?2, NULL, ?3, NULL, NULL, NULL, ?4, NULL, NULL, NULL, ?5, NULL, NULL, ?6, ?6)",
                rusqlite::params![new_gid, g_name, g_rel, g_phone, s_address, current_utc],
            )?;
            guardian_phone_to_id.insert(g_phone.clone(), new_gid.clone());
            imported_guardians += 1;
            new_gid
        };

        // 6. Student Resolution & Code Generation (AC-7)
        let student_code = if let Some(custom_code) = get_val("student_code") {
            let clean = custom_code.trim();
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM students WHERE student_code = ?1",
                rusqlite::params![clean],
                |r| r.get(0),
            )?;
            if count > 0 {
                crate::students::generate_next_student_code(conn, None).map_err(|e| {
                    let _ = conn.execute("ROLLBACK TO import_sp;", []);
                    let _ = conn.execute("RELEASE import_sp;", []);
                    ImportError::TransactionAborted(e.to_string())
                })?
            } else {
                clean.to_string()
            }
        } else {
            crate::students::generate_next_student_code(conn, None).map_err(|e| {
                let _ = conn.execute("ROLLBACK TO import_sp;", []);
                let _ = conn.execute("RELEASE import_sp;", []);
                ImportError::TransactionAborted(e.to_string())
            })?
        };

        let student_id = Uuid::now_v7().to_string();
        let s_name_bn = get_val("student_name_bn");
        let s_blood = get_val("blood_group");
        let s_religion = get_val("religion");
        let s_email = get_val("email");
        let s_class = get_val("class_level");
        let s_school_roll = get_val("current_school_roll").map(|r| normalize_bengali_digits(&r));
        let s_perm_addr = get_val("permanent_address");
        let s_prev_school = get_val("previous_school");
        let s_emerg_phone = get_val("emergency_phone").and_then(|p| {
            let digits = normalize_bengali_digits(&p);
            phone::normalize_phone(&digits).ok()
        });

        conn.execute(
            "INSERT INTO students (
                id, student_code, name, name_bn, gender, dob, blood_group, religion,
                nid_or_birth_cert, phone_norm, emergency_phone_norm, email, present_address,
                permanent_address, district, thana, previous_school, current_school_roll,
                class_level, group_major, photo_vault_id, status, admission_date, notes,
                row_version, created_at, created_by, updated_at, updated_by
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
                NULL, ?9, ?10, ?11, ?12,
                ?13, NULL, NULL, ?14, ?15,
                ?16, NULL, NULL, 'active', ?17, NULL,
                1, ?18, ?19, ?18, ?19
            )",
            rusqlite::params![
                student_id,
                student_code,
                s_name,
                s_name_bn,
                s_gender,
                s_dob,
                s_blood,
                s_religion,
                s_phone,
                s_emerg_phone,
                s_email,
                s_address,
                s_perm_addr,
                s_prev_school,
                s_school_roll,
                s_class,
                &current_utc[0..10],
                current_utc,
                user_id,
            ],
        )?;

        // Link student to guardian
        conn.execute(
            "INSERT INTO student_guardians (
                student_id, guardian_id, is_primary, receives_sms
            ) VALUES (?1, ?2, 1, 1)",
            rusqlite::params![student_id, guardian_id],
        )?;

        imported_students += 1;
        imported_student_ids.push(student_id);
    }

    // 7. Optional Batch Enrollment (AC-9)
    if let Some(ref batch_id) = input.target_batch_id {
        let batch_exists: bool = conn
            .query_row(
                "SELECT 1 FROM batches WHERE id = ?1",
                rusqlite::params![batch_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if batch_exists {
            for sid in &imported_student_ids {
                let next_roll: i64 = conn.query_row(
                    "SELECT coalesce(max(roll), 0) + 1 FROM enrollments WHERE batch_id = ?1 AND status = 'active'",
                    rusqlite::params![batch_id],
                    |r| r.get(0),
                )?;

                let enrollment_id = Uuid::now_v7().to_string();
                conn.execute(
                    "INSERT INTO enrollments (
                        id, student_id, batch_id, roll, start_date, end_date,
                        status, end_reason, notes, created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, 'active', NULL, NULL, ?6, ?6)",
                    rusqlite::params![
                        enrollment_id,
                        sid,
                        batch_id,
                        next_roll,
                        &current_utc[0..10],
                        current_utc,
                    ],
                )?;
            }
        }
    }

    // 8. Rejected Rows CSV Export (AC-8)
    let rejected_rows_csv_path = if !rejected_data_rows.is_empty() {
        let csv_path =
            generate_rejected_rows_csv(&headers, &rejected_data_rows, &rejected_reasons, None)?;
        Some(csv_path.display().to_string())
    } else {
        None
    };

    // 9. Release savepoint
    conn.execute("RELEASE import_sp;", [])?;

    Ok(ImportSummaryResult {
        total_rows: data_rows.len(),
        imported_students_count: imported_students,
        imported_guardians_count: imported_guardians,
        skipped_errors_count: rejected_data_rows.len(),
        backup_path: backup_path.display().to_string(),
        rejected_rows_csv_path,
    })
}
