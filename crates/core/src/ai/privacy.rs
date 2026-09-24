use crate::ai::AiError;
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct PrivacySettings {
    pub allow_student_names: bool,
    pub allow_phone_numbers: bool,
    pub allow_guardian_info: bool,
    pub allow_financial_figures: bool,
    pub allow_exam_marks: bool,
    pub allow_staff_names: bool,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProtectionMetadata {
    pub shield_active: bool,
    pub records_protected: usize,
    pub redacted_categories: Vec<String>,
}

pub fn get_privacy_settings(conn: &Connection) -> Result<PrivacySettings, AiError> {
    let row = conn
        .query_row(
            "SELECT allow_student_names, allow_phone_numbers, allow_guardian_info,
                    allow_financial_figures, allow_exam_marks, allow_staff_names,
                    updated_at, updated_by
             FROM ai_privacy_settings
             WHERE id = 'default'
             LIMIT 1",
            [],
            |r| {
                Ok(PrivacySettings {
                    allow_student_names: r.get::<_, i64>(0)? != 0,
                    allow_phone_numbers: r.get::<_, i64>(1)? != 0,
                    allow_guardian_info: r.get::<_, i64>(2)? != 0,
                    allow_financial_figures: r.get::<_, i64>(3)? != 0,
                    allow_exam_marks: r.get::<_, i64>(4)? != 0,
                    allow_staff_names: r.get::<_, i64>(5)? != 0,
                    updated_at: r.get(6)?,
                    updated_by: r.get(7)?,
                })
            },
        )
        .optional()?;

    Ok(row.unwrap_or_default())
}

pub fn update_privacy_settings(
    conn: &Connection,
    settings: PrivacySettings,
    user_id: Option<&str>,
) -> Result<PrivacySettings, AiError> {
    let now = crate::db::now_utc_iso();

    conn.execute(
        "INSERT INTO ai_privacy_settings (
            id, allow_student_names, allow_phone_numbers, allow_guardian_info,
            allow_financial_figures, allow_exam_marks, allow_staff_names,
            updated_at, updated_by
         ) VALUES ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            allow_student_names = excluded.allow_student_names,
            allow_phone_numbers = excluded.allow_phone_numbers,
            allow_guardian_info = excluded.allow_guardian_info,
            allow_financial_figures = excluded.allow_financial_figures,
            allow_exam_marks = excluded.allow_exam_marks,
            allow_staff_names = excluded.allow_staff_names,
            updated_at = excluded.updated_at,
            updated_by = excluded.updated_by",
        params![
            if settings.allow_student_names { 1 } else { 0 },
            if settings.allow_phone_numbers { 1 } else { 0 },
            if settings.allow_guardian_info { 1 } else { 0 },
            if settings.allow_financial_figures {
                1
            } else {
                0
            },
            if settings.allow_exam_marks { 1 } else { 0 },
            if settings.allow_staff_names { 1 } else { 0 },
            now,
            user_id
        ],
    )?;

    get_privacy_settings(conn)
}

// -----------------------------------------------------------------------------
// Client-Side Redaction & De-Anonymization Engine
// -----------------------------------------------------------------------------

fn bd_phone_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?:\+?88)?01[3-9](?:[ -]?\d{4}[ -]?\d{4}|\d{8})").expect("valid regex")
    })
}

#[derive(Debug, Default)]
pub struct PrivacyRedactionEngine {
    pub settings: PrivacySettings,
    /// Ephemeral map: Pseudonym -> Original Text (e.g. "Student #STU-1042" -> "Rahim Uddin")
    pub token_map: HashMap<String, String>,
    pub redacted_categories: Vec<String>,
    pub total_redactions: usize,
}

impl PrivacyRedactionEngine {
    pub fn new(settings: PrivacySettings) -> Self {
        let mut redacted_categories = Vec::new();
        if !settings.allow_student_names {
            redacted_categories.push("student_names".to_string());
        }
        if !settings.allow_phone_numbers {
            redacted_categories.push("phone_numbers".to_string());
        }
        if !settings.allow_guardian_info {
            redacted_categories.push("guardian_info".to_string());
        }
        if !settings.allow_financial_figures {
            redacted_categories.push("financial_figures".to_string());
        }
        if !settings.allow_exam_marks {
            redacted_categories.push("exam_marks".to_string());
        }
        if !settings.allow_staff_names {
            redacted_categories.push("staff_names".to_string());
        }

        Self {
            settings,
            token_map: HashMap::new(),
            redacted_categories,
            total_redactions: 0,
        }
    }

    /// Redacts Bangladeshi mobile phone numbers from user prompt if phone numbers are disabled.
    pub fn scrub_prompt(&mut self, prompt: &str) -> String {
        if self.settings.allow_phone_numbers {
            return prompt.to_string();
        }

        let re = bd_phone_regex();
        let mut count = 0;
        let scrubbed = re.replace_all(prompt, |caps: &regex::Captures| {
            count += 1;
            let original = caps.get(0).unwrap().as_str().to_string();
            let pseudonym = format!("[PHONE-{}]", count);
            self.token_map.insert(pseudonym.clone(), original);
            self.total_redactions += 1;
            pseudonym
        });

        scrubbed.into_owned()
    }

    /// Redacts sensitive fields from tool output JSON based on active privacy settings.
    pub fn redact_tool_output(&mut self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.redact_tool_output(item);
                }
            }
            serde_json::Value::Object(map) => {
                // 1. Student Name Redaction
                if !self.settings.allow_student_names {
                    if let Some(name_val) = map.get("name").and_then(|v| v.as_str()) {
                        let student_code = map
                            .get("student_code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("UNKNOWN");
                        let pseudonym = format!("Student #{}", student_code);
                        self.token_map
                            .insert(pseudonym.clone(), name_val.to_string());
                        map.insert("name".to_string(), serde_json::Value::String(pseudonym));
                        self.total_redactions += 1;
                    }
                }

                // 2. Phone Numbers Redaction
                if !self.settings.allow_phone_numbers {
                    if let Some(phone_val) = map.get_mut("phone") {
                        if !phone_val.is_null() {
                            *phone_val = serde_json::Value::String("[PHONE REDACTED]".to_string());
                            self.total_redactions += 1;
                        }
                    }
                    if let Some(phone_norm_val) = map.get_mut("phone_norm") {
                        if !phone_norm_val.is_null() {
                            *phone_norm_val =
                                serde_json::Value::String("[PHONE REDACTED]".to_string());
                            self.total_redactions += 1;
                        }
                    }
                }

                // 3. Guardian Info Redaction
                if !self.settings.allow_guardian_info {
                    if let Some(gname_val) = map.get_mut("guardian_name") {
                        if !gname_val.is_null() {
                            *gname_val =
                                serde_json::Value::String("[GUARDIAN REDACTED]".to_string());
                            self.total_redactions += 1;
                        }
                    }
                    if let Some(gphone_val) = map.get_mut("guardian_phone") {
                        if !gphone_val.is_null() {
                            *gphone_val =
                                serde_json::Value::String("[GUARDIAN PHONE REDACTED]".to_string());
                            self.total_redactions += 1;
                        }
                    }
                }

                // 4. Financial Figures Redaction
                if !self.settings.allow_financial_figures {
                    if let Some(due_val) = map.get_mut("total_due_poisha") {
                        if !due_val.is_null() {
                            *due_val = serde_json::Value::String("*** poisha".to_string());
                            self.total_redactions += 1;
                        }
                    }
                    if let Some(coll_val) = map.get_mut("total_collection_poisha") {
                        if !coll_val.is_null() {
                            *coll_val = serde_json::Value::String("*** poisha".to_string());
                            self.total_redactions += 1;
                        }
                    }
                }

                // 5. Exam Marks Redaction
                if !self.settings.allow_exam_marks {
                    if let Some(avg_val) = map.get_mut("average_mark") {
                        if !avg_val.is_null() {
                            *avg_val = serde_json::Value::String("*** marks".to_string());
                            self.total_redactions += 1;
                        }
                    }
                    if let Some(max_val) = map.get_mut("highest_mark") {
                        if !max_val.is_null() {
                            *max_val = serde_json::Value::String("*** marks".to_string());
                            self.total_redactions += 1;
                        }
                    }
                    if let Some(min_val) = map.get_mut("lowest_mark") {
                        if !min_val.is_null() {
                            *min_val = serde_json::Value::String("*** marks".to_string());
                            self.total_redactions += 1;
                        }
                    }
                }

                // 6. Staff / Teacher Name Redaction
                if !self.settings.allow_staff_names {
                    if let Some(tname_val) = map.get_mut("teacher_name") {
                        if !tname_val.is_null() {
                            *tname_val = serde_json::Value::String("Teacher #STAFF".to_string());
                            self.total_redactions += 1;
                        }
                    }
                }

                // Recursively redact any nested objects or arrays
                for (_, val) in map.iter_mut() {
                    if val.is_object() || val.is_array() {
                        self.redact_tool_output(val);
                    }
                }
            }
            _ => {}
        }
    }

    /// Replaces pseudonyms back with original real values for display in the local UI.
    pub fn rehydrate_content(&self, text: &str) -> (String, usize) {
        let mut rehydrated = text.to_string();
        let mut rehydrated_count = 0;

        for (pseudonym, original) in &self.token_map {
            if rehydrated.contains(pseudonym) {
                rehydrated = rehydrated.replace(pseudonym, original);
                rehydrated_count += 1;
            }
        }

        (rehydrated, rehydrated_count)
    }

    pub fn protection_metadata(&self) -> ProtectionMetadata {
        ProtectionMetadata {
            shield_active: !self.redacted_categories.is_empty(),
            records_protected: self.total_redactions,
            redacted_categories: self.redacted_categories.clone(),
        }
    }
}
