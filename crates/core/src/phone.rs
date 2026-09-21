use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PhoneError {
    #[error("Phone number cannot be empty")]
    Empty,
    #[error("Invalid characters in phone number: {0}")]
    InvalidCharacters(String),
    #[error(
        "Invalid Bangladeshi mobile number: must be 11 digits starting with 01 (e.g. 01712345678)"
    )]
    InvalidBdMobile(String),
    #[error("Invalid phone length: must be between 6 and 20 digits")]
    InvalidLength(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct PhoneConflict {
    pub phone_norm: String,
    pub existing_owner_type: String, // "student" or "guardian"
    pub existing_owner_id: String,
    pub existing_owner_name: String,
    pub existing_code_or_relation: Option<String>,
}

impl std::fmt::Display for PhoneConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Phone {} is already registered to {} '{}'",
            self.phone_norm, self.existing_owner_type, self.existing_owner_name
        )
    }
}

/// Normalizes a phone number string into a standard format.
///
/// For Bangladeshi mobile numbers:
/// - Accepts `01XXXXXXXXX`, `+8801XXXXXXXXX`, `8801XXXXXXXXX`, with spaces/dashes/brackets stripped.
/// - Valid operator prefixes: 013, 014, 015, 016, 017, 018, 019.
/// - Produces the 13-digit standard `8801XXXXXXXXX`.
///
/// For other numbers (landlines, international):
/// - Strips spaces, dashes, brackets, and `+`.
/// - Preserves clean digit sequence (length 6 to 20 digits).
pub fn normalize_phone(raw: &str) -> Result<String, PhoneError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PhoneError::Empty);
    }

    // Check for invalid characters (letters or unusual symbols)
    for c in trimmed.chars() {
        if !c.is_ascii_digit()
            && c != '+'
            && c != '-'
            && c != ' '
            && c != '('
            && c != ')'
            && c != '.'
        {
            return Err(PhoneError::InvalidCharacters(format!(
                "Invalid character '{}' in phone number",
                c
            )));
        }
    }

    // Strip non-digit characters
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return Err(PhoneError::Empty);
    }

    // Check if it's a Bangladeshi mobile number
    // BD mobile has 11 digits starting with 01: 01[3-9]XXXXXXXX
    // Or 13 digits starting with 8801: 8801[3-9]XXXXXXXX
    if digits.starts_with("01") && digits.len() == 11 {
        let op = digits.as_bytes()[2];
        if (b'3'..=b'9').contains(&op) {
            return Ok(format!("88{}", digits));
        } else {
            return Err(PhoneError::InvalidBdMobile(trimmed.to_string()));
        }
    }

    if digits.starts_with("8801") && digits.len() == 13 {
        let op = digits.as_bytes()[4];
        if (b'3'..=b'9').contains(&op) {
            return Ok(digits);
        } else {
            return Err(PhoneError::InvalidBdMobile(trimmed.to_string()));
        }
    }

    // If starts with 01 but not 11 digits, likely a malformed BD mobile number
    if digits.starts_with("01") && digits.len() < 13 {
        return Err(PhoneError::InvalidBdMobile(trimmed.to_string()));
    }

    // For other numbers (landline, international), enforce 6 to 20 digits
    if digits.len() < 6 || digits.len() > 20 {
        return Err(PhoneError::InvalidLength(digits.len()));
    }

    Ok(digits)
}

/// Checks if an active student or guardian already uses the normalized phone number.
///
/// Can exclude a specific student ID or guardian ID when updating existing records.
pub fn check_phone_conflict(
    conn: &Connection,
    phone_norm: &str,
    exclude_student_id: Option<&str>,
    exclude_guardian_id: Option<&str>,
) -> Result<Option<PhoneConflict>, rusqlite::Error> {
    // Check students first (only active or non-archived)
    let mut student_stmt = conn.prepare(
        "SELECT id, name, student_code FROM students
         WHERE phone_norm = ?1 AND status != 'archived' AND (?2 IS NULL OR id != ?2)
         LIMIT 1",
    )?;
    let mut student_rows = student_stmt.query(rusqlite::params![phone_norm, exclude_student_id])?;
    if let Some(row) = student_rows.next()? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let student_code: String = row.get(2)?;
        return Ok(Some(PhoneConflict {
            phone_norm: phone_norm.to_string(),
            existing_owner_type: "student".to_string(),
            existing_owner_id: id,
            existing_owner_name: name,
            existing_code_or_relation: Some(student_code),
        }));
    }

    // Check guardians
    let mut guardian_stmt = conn.prepare(
        "SELECT id, name, relationship FROM guardians
         WHERE phone_norm = ?1 AND (?2 IS NULL OR id != ?2)
         LIMIT 1",
    )?;
    let mut guardian_rows =
        guardian_stmt.query(rusqlite::params![phone_norm, exclude_guardian_id])?;
    if let Some(row) = guardian_rows.next()? {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let relation: String = row.get(2)?;
        return Ok(Some(PhoneConflict {
            phone_norm: phone_norm.to_string(),
            existing_owner_type: "guardian".to_string(),
            existing_owner_id: id,
            existing_owner_name: name,
            existing_code_or_relation: Some(relation),
        }));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bd_mobile_normalization() {
        assert_eq!(normalize_phone("01712345678").unwrap(), "8801712345678");
        assert_eq!(normalize_phone("+8801712345678").unwrap(), "8801712345678");
        assert_eq!(normalize_phone("8801712345678").unwrap(), "8801712345678");
        assert_eq!(normalize_phone("017-12345678").unwrap(), "8801712345678");
        assert_eq!(
            normalize_phone("+880 181 999 8888").unwrap(),
            "8801819998888"
        );
        assert_eq!(normalize_phone("(019) 11223344").unwrap(), "8801911223344");
    }

    #[test]
    fn test_international_and_landline() {
        assert_eq!(normalize_phone("+1 555 123 4567").unwrap(), "15551234567");
        assert_eq!(normalize_phone("029876543").unwrap(), "029876543");
    }

    #[test]
    fn test_invalid_phones() {
        assert!(normalize_phone("").is_err());
        assert!(normalize_phone("   ").is_err());
        assert!(normalize_phone("abc01712345678").is_err());
        assert!(normalize_phone("017123").is_err()); // short BD mobile
        assert!(normalize_phone("123").is_err()); // too short
    }
}
