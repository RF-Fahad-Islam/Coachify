use crate::ai::AiError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiWalletBalance {
    pub balance_credits: i64,
    pub is_active: bool,
    pub last_synced_at: Option<String>,
    pub gateway_url: String,
    pub gateway_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreditTransactionItem {
    pub id: String,
    pub wallet_id: String,
    pub transaction_type: String,
    pub amount: i64,
    pub balance_after: i64,
    pub reference_id: Option<String>,
    pub description: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub created_by_name: Option<String>,
}

pub fn get_wallet_balance(conn: &Connection) -> Result<AiWalletBalance, AiError> {
    let row = conn
        .query_row(
            "SELECT balance_credits, is_active, last_synced_at, gateway_url, token_dpapi_blob
             FROM ai_credit_wallets
             WHERE id = 'default'
             LIMIT 1",
            [],
            |r| {
                let bal: i64 = r.get(0)?;
                let active: i64 = r.get(1)?;
                let synced: Option<String> = r.get(2)?;
                let url: String = r.get(3)?;
                let blob: Option<Vec<u8>> = r.get(4)?;
                let has_token = blob.map(|b| !b.is_empty()).unwrap_or(false);

                Ok(AiWalletBalance {
                    balance_credits: bal,
                    is_active: active != 0,
                    last_synced_at: synced,
                    gateway_url: url,
                    gateway_configured: has_token,
                })
            },
        )
        .optional()?;

    Ok(row.unwrap_or(AiWalletBalance {
        balance_credits: 0,
        is_active: true,
        last_synced_at: None,
        gateway_url: "https://ai.coachify.cloud/v1".to_string(),
        gateway_configured: false,
    }))
}

/// Pre-flight guard: if credit balance is zero or negative, immediately reject.
pub fn pre_flight_credit_check(conn: &Connection) -> Result<(), AiError> {
    let balance: i64 = conn
        .query_row(
            "SELECT balance_credits FROM ai_credit_wallets WHERE id = 'default' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if balance <= 0 {
        return Err(AiError::CreditsExhausted);
    }
    Ok(())
}

/// Deducts credits atomically and records an immutable usage transaction.
pub fn deduct_credits(
    conn: &Connection,
    amount: i64,
    reference_id: &str,
    description: &str,
    user_id: Option<&str>,
) -> Result<i64, AiError> {
    if amount <= 0 {
        return Err(AiError::Validation(
            "Credit deduction amount must be strictly positive.".to_string(),
        ));
    }

    let current_balance: i64 = conn
        .query_row(
            "SELECT balance_credits FROM ai_credit_wallets WHERE id = 'default' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_balance = current_balance - amount;
    let now = crate::db::now_utc_iso();
    let tx_id = Uuid::now_v7().to_string();

    conn.execute(
        "UPDATE ai_credit_wallets SET balance_credits = ?1, updated_at = ?2 WHERE id = 'default'",
        params![new_balance, now],
    )?;

    conn.execute(
        "INSERT INTO ai_credit_transactions (
            id, wallet_id, transaction_type, amount, balance_after, reference_id, description, created_at, created_by
         ) VALUES (?1, 'default', 'usage', ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            tx_id,
            -amount,
            new_balance,
            reference_id,
            description,
            now,
            user_id
        ],
    )?;

    Ok(new_balance)
}

/// Grants or recharges credits and records an immutable transaction.
pub fn record_credit_grant(
    conn: &Connection,
    amount: i64,
    tx_type: &str,
    reference_id: Option<&str>,
    description: &str,
    user_id: Option<&str>,
) -> Result<CreditTransactionItem, AiError> {
    if amount <= 0 {
        return Err(AiError::Validation(
            "Credit grant amount must be positive.".to_string(),
        ));
    }

    let valid_types = ["purchase", "refund", "promotional", "sync_adjustment"];
    if !valid_types.contains(&tx_type) {
        return Err(AiError::Validation(format!(
            "Invalid transaction type: '{}'",
            tx_type
        )));
    }

    let current_balance: i64 = conn
        .query_row(
            "SELECT balance_credits FROM ai_credit_wallets WHERE id = 'default' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_balance = current_balance + amount;
    let now = crate::db::now_utc_iso();
    let tx_id = Uuid::now_v7().to_string();

    conn.execute(
        "UPDATE ai_credit_wallets SET balance_credits = ?1, updated_at = ?2 WHERE id = 'default'",
        params![new_balance, now],
    )?;

    conn.execute(
        "INSERT INTO ai_credit_transactions (
            id, wallet_id, transaction_type, amount, balance_after, reference_id, description, created_at, created_by
         ) VALUES (?1, 'default', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            tx_id,
            tx_type,
            amount,
            new_balance,
            reference_id,
            description,
            now,
            user_id
        ],
    )?;

    Ok(CreditTransactionItem {
        id: tx_id,
        wallet_id: "default".to_string(),
        transaction_type: tx_type.to_string(),
        amount,
        balance_after: new_balance,
        reference_id: reference_id.map(|s| s.to_string()),
        description: description.to_string(),
        created_at: now,
        created_by: user_id.map(|s| s.to_string()),
        created_by_name: None,
    })
}

pub fn get_credit_ledger(
    conn: &Connection,
    limit: Option<u32>,
    before_time: Option<&str>,
) -> Result<Vec<CreditTransactionItem>, AiError> {
    let lim = limit.unwrap_or(50).clamp(1, 100);

    let sql = r#"
        SELECT
            t.id,
            t.wallet_id,
            t.transaction_type,
            t.amount,
            t.balance_after,
            t.reference_id,
            t.description,
            t.created_at,
            t.created_by,
            u.display_name AS created_by_name
        FROM ai_credit_transactions t
        LEFT JOIN users u ON u.id = t.created_by
        WHERE (?1 IS NULL OR t.created_at < ?1)
        ORDER BY t.created_at DESC
        LIMIT ?2
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![before_time, lim], |r| {
        Ok(CreditTransactionItem {
            id: r.get(0)?,
            wallet_id: r.get(1)?,
            transaction_type: r.get(2)?,
            amount: r.get(3)?,
            balance_after: r.get(4)?,
            reference_id: r.get(5)?,
            description: r.get(6)?,
            created_at: r.get(7)?,
            created_by: r.get(8)?,
            created_by_name: r.get(9)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn sync_credits(conn: &Connection, _user_id: Option<&str>) -> Result<AiWalletBalance, AiError> {
    let now = crate::db::now_utc_iso();

    conn.execute(
        "UPDATE ai_credit_wallets SET last_synced_at = ?1, updated_at = ?1 WHERE id = 'default'",
        params![now],
    )?;

    get_wallet_balance(conn)
}
