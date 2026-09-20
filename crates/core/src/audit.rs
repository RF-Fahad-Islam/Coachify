use crate::accounts::ResolvedSession;
use crate::db;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Audit log corruption detected at sequence {seq}: {reason}")]
    Corruption { seq: i64, reason: String },
}

/// 64-character hexadecimal zero string used as the previous hash for the genesis record (seq = 1).
pub const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Blacklisted field keys that must be scrubbed from audit before/after payloads.
pub const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "password_hash",
    "new_password",
    "current_password",
    "session_token",
    "token_hash",
    "token",
    "secret_ref",
    "secret",
    "private_key",
    "encryption_key",
];

/// Domain intent to record an audit log row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AuditLogItem {
    pub seq: i64,
    pub at: String,
    pub business_date: String,
    pub user_id: Option<String>,
    pub user_display_name: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub prev_hash: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListAuditLogsPayload {
    pub entity: Option<String>,
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub user_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListAuditLogsResponse {
    pub items: Vec<AuditLogItem>,
    pub total_count: i64,
    pub head_seq: Option<i64>,
    pub head_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VerifyAuditChainResponse {
    pub is_valid: bool,
    pub total_records: i64,
    pub head_seq: Option<i64>,
    pub head_hash: Option<String>,
    pub broken_seq: Option<i64>,
    pub error_message: Option<String>,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAuditHeadResponse {
    pub head_seq: Option<i64>,
    pub head_hash: Option<String>,
    pub total_records: i64,
}

#[derive(Serialize)]
struct CanonicalAuditPayload<'a> {
    action: &'a str,
    after_json: Option<&'a str>,
    at: &'a str,
    before_json: Option<&'a str>,
    business_date: &'a str,
    device_id: Option<&'a str>,
    entity: &'a str,
    entity_id: &'a str,
    prev_hash: &'a str,
    user_id: Option<&'a str>,
}

/// Recursively inspects JSON objects and arrays, replacing any sensitive key values with "[REDACTED]".
pub fn sanitize_json_value(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let k_lower = k.to_lowercase();
                if SENSITIVE_KEYS.iter().any(|&s| k_lower.contains(s)) {
                    *v = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    sanitize_json_value(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_json_value(item);
            }
        }
        _ => {}
    }
}

/// Computes the deterministic SHA-256 hash of an audit record over canonical JSON representation.
#[allow(clippy::too_many_arguments)]
pub fn compute_audit_hash(
    prev_hash: &str,
    at: &str,
    business_date: &str,
    user_id: Option<&str>,
    device_id: Option<&str>,
    action: &str,
    entity: &str,
    entity_id: &str,
    before_json: Option<&str>,
    after_json: Option<&str>,
) -> String {
    let canonical = CanonicalAuditPayload {
        action,
        after_json,
        at,
        before_json,
        business_date,
        device_id,
        entity,
        entity_id,
        prev_hash,
        user_id,
    };
    let canonical_json =
        serde_json::to_string(&canonical).expect("canonical audit payload serialization");
    let mut hasher = Sha256::new();
    hasher.update(canonical_json.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Records a single audit entry into `audit_log` within the caller's immediate transaction.
pub fn record_audit_entry(
    conn: &Connection,
    mut entry: AuditEntry,
    session: Option<&ResolvedSession>,
    current_utc: &str,
) -> Result<(), AuditError> {
    let business_date = db::current_business_date()
        .unwrap_or_else(|_| current_utc[..10.min(current_utc.len())].to_string());
    let user_id = session.map(|s| s.user.id.as_str());
    let device_id = session.map(|s| s.device.id.as_str());

    // Sanitize before and after JSON
    if let Some(ref mut b) = entry.before_json {
        sanitize_json_value(b);
    }
    if let Some(ref mut a) = entry.after_json {
        sanitize_json_value(a);
    }

    let before_str = entry.before_json.as_ref().map(|v| v.to_string());
    let after_str = entry.after_json.as_ref().map(|v| v.to_string());

    // Query latest audit_log row to get prev_hash
    let latest_hash: Option<String> = conn
        .query_row(
            "SELECT hash FROM audit_log ORDER BY seq DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let prev_hash = latest_hash.as_deref().unwrap_or(GENESIS_PREV_HASH);

    let hash = compute_audit_hash(
        prev_hash,
        current_utc,
        &business_date,
        user_id,
        device_id,
        &entry.action,
        &entry.entity,
        &entry.entity_id,
        before_str.as_deref(),
        after_str.as_deref(),
    );

    conn.execute(
        "INSERT INTO audit_log (
            at, business_date, user_id, device_id, action, entity, entity_id,
            before_json, after_json, prev_hash, hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            current_utc,
            business_date,
            user_id,
            device_id,
            entry.action,
            entry.entity,
            entry.entity_id,
            before_str,
            after_str,
            prev_hash,
            hash,
        ],
    )?;

    Ok(())
}

/// Verifies the cryptographic hash chain from genesis to head.
pub fn verify_audit_log_chain(conn: &Connection) -> Result<VerifyAuditChainResponse, AuditError> {
    let verified_at = db::now_utc_iso();
    let mut stmt = conn.prepare(
        "SELECT seq, at, business_date, user_id, device_id, action, entity, entity_id,
                before_json, after_json, prev_hash, hash
         FROM audit_log ORDER BY seq ASC",
    )?;

    let mut rows = stmt.query([])?;
    let mut total_records: i64 = 0;
    let mut prev_expected_hash = GENESIS_PREV_HASH.to_string();
    let mut head_seq: Option<i64> = None;
    let mut head_hash: Option<String> = None;
    let mut expected_seq: i64 = 1;

    while let Some(row) = rows.next()? {
        let seq: i64 = row.get(0)?;
        let at: String = row.get(1)?;
        let business_date: String = row.get(2)?;
        let user_id: Option<String> = row.get(3)?;
        let device_id: Option<String> = row.get(4)?;
        let action: String = row.get(5)?;
        let entity: String = row.get(6)?;
        let entity_id: String = row.get(7)?;
        let before_json: Option<String> = row.get(8)?;
        let after_json: Option<String> = row.get(9)?;
        let prev_hash: String = row.get(10)?;
        let stored_hash: String = row.get(11)?;

        // 1. Check sequence monotonicity
        if seq != expected_seq {
            return Ok(VerifyAuditChainResponse {
                is_valid: false,
                total_records,
                head_seq,
                head_hash,
                broken_seq: Some(seq),
                error_message: Some(format!(
                    "Sequence gap detected: expected seq {}, found {}",
                    expected_seq, seq
                )),
                verified_at,
            });
        }

        // 2. Check prev_hash linkage
        if prev_hash != prev_expected_hash {
            return Ok(VerifyAuditChainResponse {
                is_valid: false,
                total_records,
                head_seq,
                head_hash,
                broken_seq: Some(seq),
                error_message: Some(format!(
                    "Cryptographic hash chain broken at seq {}: prev_hash does not match preceding hash",
                    seq
                )),
                verified_at,
            });
        }

        // 3. Recompute hash and assert match
        let recomputed = compute_audit_hash(
            &prev_hash,
            &at,
            &business_date,
            user_id.as_deref(),
            device_id.as_deref(),
            &action,
            &entity,
            &entity_id,
            before_json.as_deref(),
            after_json.as_deref(),
        );

        if recomputed != stored_hash {
            return Ok(VerifyAuditChainResponse {
                is_valid: false,
                total_records,
                head_seq,
                head_hash,
                broken_seq: Some(seq),
                error_message: Some(format!(
                    "Audit record payload tampered at seq {}: computed hash {} does not match stored hash {}",
                    seq, recomputed, stored_hash
                )),
                verified_at,
            });
        }

        prev_expected_hash = stored_hash.clone();
        head_seq = Some(seq);
        head_hash = Some(stored_hash);
        total_records += 1;
        expected_seq += 1;
    }

    Ok(VerifyAuditChainResponse {
        is_valid: true,
        total_records,
        head_seq,
        head_hash,
        broken_seq: None,
        error_message: None,
        verified_at,
    })
}

/// Retrieves the current head sequence and head hash of the audit log.
pub fn get_audit_head(conn: &Connection) -> Result<GetAuditHeadResponse, AuditError> {
    let total_records: i64 =
        conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;

    let head: Option<(i64, String)> = conn
        .query_row(
            "SELECT seq, hash FROM audit_log ORDER BY seq DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    Ok(GetAuditHeadResponse {
        head_seq: head.as_ref().map(|h| h.0),
        head_hash: head.map(|h| h.1),
        total_records,
    })
}

/// Queries audit log records with filtering and pagination.
pub fn list_audit_logs(
    conn: &Connection,
    payload: &ListAuditLogsPayload,
) -> Result<ListAuditLogsResponse, AuditError> {
    let limit = payload.limit.unwrap_or(50).clamp(1, 200);
    let offset = payload.offset.unwrap_or(0);

    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref entity) = payload.entity {
        if !entity.trim().is_empty() {
            where_clauses.push("a.entity = ?");
            params.push(Box::new(entity.trim().to_string()));
        }
    }

    if let Some(ref entity_id) = payload.entity_id {
        if !entity_id.trim().is_empty() {
            where_clauses.push("a.entity_id = ?");
            params.push(Box::new(entity_id.trim().to_string()));
        }
    }

    if let Some(ref action) = payload.action {
        if !action.trim().is_empty() {
            where_clauses.push("a.action LIKE ?");
            params.push(Box::new(format!("%{}%", action.trim())));
        }
    }

    if let Some(ref user_id) = payload.user_id {
        if !user_id.trim().is_empty() {
            where_clauses.push("a.user_id = ?");
            params.push(Box::new(user_id.trim().to_string()));
        }
    }

    if let Some(ref date_from) = payload.date_from {
        if !date_from.trim().is_empty() {
            where_clauses.push("a.at >= ?");
            params.push(Box::new(date_from.trim().to_string()));
        }
    }

    if let Some(ref date_to) = payload.date_to {
        if !date_to.trim().is_empty() {
            where_clauses.push("a.at <= ?");
            params.push(Box::new(date_to.trim().to_string()));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Total count query
    let count_query = format!("SELECT COUNT(*) FROM audit_log a {}", where_sql);
    let mut count_stmt = conn.prepare(&count_query)?;
    let count_params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| &**p).collect();
    let total_count: i64 = count_stmt.query_row(&count_params_refs[..], |row| row.get(0))?;

    // Items query with joins to users and devices
    let select_query = format!(
        "SELECT a.seq, a.at, a.business_date, a.user_id, u.display_name,
                a.device_id, d.name, a.action, a.entity, a.entity_id,
                a.before_json, a.after_json, a.prev_hash, a.hash
         FROM audit_log a
         LEFT JOIN users u ON a.user_id = u.id
         LEFT JOIN devices d ON a.device_id = d.id
         {}
         ORDER BY a.seq DESC
         LIMIT ? OFFSET ?",
        where_sql
    );

    let mut select_stmt = conn.prepare(&select_query)?;
    let mut query_params = params;
    query_params.push(Box::new(limit as i64));
    query_params.push(Box::new(offset as i64));
    let query_params_refs: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|p| &**p).collect();

    let mut rows = select_stmt.query(&query_params_refs[..])?;
    let mut items = Vec::new();

    while let Some(row) = rows.next()? {
        items.push(AuditLogItem {
            seq: row.get(0)?,
            at: row.get(1)?,
            business_date: row.get(2)?,
            user_id: row.get(3)?,
            user_display_name: row.get(4)?,
            device_id: row.get(5)?,
            device_name: row.get(6)?,
            action: row.get(7)?,
            entity: row.get(8)?,
            entity_id: row.get(9)?,
            before_json: row.get(10)?,
            after_json: row.get(11)?,
            prev_hash: row.get(12)?,
            hash: row.get(13)?,
        });
    }

    let head_info = get_audit_head(conn)?;

    Ok(ListAuditLogsResponse {
        items,
        total_count,
        head_seq: head_info.head_seq,
        head_hash: head_info.head_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_json_value_redacts_sensitive_keys() {
        let mut val = serde_json::json!({
            "username": "fahad",
            "password": "PlainTextPassword123!",
            "password_hash": "$argon2id$v=19$...",
            "session_token": "secret_session_token_xyz",
            "profile": {
                "display_name": "Fahad",
                "current_password": "OldPassword123!",
                "nested": {
                    "token_hash": "abc123hash",
                    "safe_field": "keep_this"
                }
            }
        });

        sanitize_json_value(&mut val);

        assert_eq!(val["username"], "fahad");
        assert_eq!(val["password"], "[REDACTED]");
        assert_eq!(val["password_hash"], "[REDACTED]");
        assert_eq!(val["session_token"], "[REDACTED]");
        assert_eq!(val["profile"]["display_name"], "Fahad");
        assert_eq!(val["profile"]["current_password"], "[REDACTED]");
        assert_eq!(val["profile"]["nested"]["token_hash"], "[REDACTED]");
        assert_eq!(val["profile"]["nested"]["safe_field"], "keep_this");
    }

    #[test]
    fn test_compute_audit_hash_deterministic() {
        let hash1 = compute_audit_hash(
            GENESIS_PREV_HASH,
            "2026-09-20T12:00:00Z",
            "2026-09-20",
            Some("user-1"),
            Some("device-1"),
            "users.create",
            "user",
            "user-1",
            None,
            Some("{\"display_name\":\"Fahad\"}"),
        );

        let hash2 = compute_audit_hash(
            GENESIS_PREV_HASH,
            "2026-09-20T12:00:00Z",
            "2026-09-20",
            Some("user-1"),
            Some("device-1"),
            "users.create",
            "user",
            "user-1",
            None,
            Some("{\"display_name\":\"Fahad\"}"),
        );

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }
}
