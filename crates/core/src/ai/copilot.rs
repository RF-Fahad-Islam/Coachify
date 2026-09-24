use crate::ai::gateway::{send_chat_completion, GatewayMessage, GatewayRequest};
use crate::ai::privacy::{get_privacy_settings, PrivacyRedactionEngine, ProtectionMetadata};
use crate::ai::tools::{execute_tool, get_available_tool_schemas};
use crate::ai::wallet::{deduct_credits, pre_flight_credit_check};
use crate::ai::AiError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CopilotResponse {
    pub conversation_id: String,
    pub message_id: String,
    pub content: String,
    pub tools_used: Vec<String>,
    pub credits_deducted: i64,
    pub balance_remaining: i64,
    pub protection_metadata: ProtectionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CopilotMessageItem {
    pub id: String,
    pub role: String,
    pub content: String,
    pub tool_calls_json: Option<String>,
    pub tool_result_json: Option<String>,
    pub credits_deducted: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CopilotConversationDetails {
    pub id: String,
    pub title: String,
    pub messages: Vec<CopilotMessageItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiAuditLogItem {
    pub id: String,
    pub request_id: String,
    pub user_id: String,
    pub user_name: Option<String>,
    pub action_type: String,
    pub prompt_summary: String,
    pub tools_invoked: Vec<String>,
    pub categories_allowed: Vec<String>,
    pub categories_redacted: Vec<String>,
    pub redaction_count: usize,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub credits_used: i64,
    pub latency_ms: u64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

const SYSTEM_PROMPT: &str = r#"You are Coachify AI Copilot, an intelligent institutional assistant for coaching center administrators and educators.
You answer questions accurately about students, batches, attendance, fee collections, dues, and exams using the local center data.

Rules:
1. Always base answers strictly on data returned by the provided tools. Never hallucinate or assume unverified statistics.
2. Mirror the user's language: If the user asks in Bengali (Bangla), respond in natural, professional Bangla. If in English, respond in English.
3. Use Bangladeshi numerical and currency conventions: 'Taka' (Tk), 'poisha', and format numbers clearly.
4. You operate ONLY through the declared tools. You CANNOT execute raw SQL queries, mutate records, or access unauthorized tables.
5. If data is not found or a query returns empty results, explain clearly to the user without making up numbers.
"#;

pub fn ask_copilot(
    conn: &Connection,
    conversation_id_opt: Option<String>,
    prompt: &str,
    user_id: &str,
    has_financial_perm: bool,
) -> Result<CopilotResponse, AiError> {
    let start_time = Instant::now();
    let request_id = Uuid::now_v7().to_string();

    // 1. Pre-flight credit check
    pre_flight_credit_check(conn)?;

    let trimmed_prompt = prompt.trim();
    if trimmed_prompt.is_empty() {
        return Err(AiError::Validation("Prompt cannot be empty.".to_string()));
    }

    // 2. Resolve or create active conversation
    let conv_id = match conversation_id_opt {
        Some(cid) => {
            // Verify conversation belongs to active user
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM ai_copilot_conversations WHERE id = ?1 AND user_id = ?2",
                    params![cid, user_id],
                    |_| Ok(true),
                )
                .optional()?
                .unwrap_or(false);

            if !exists {
                create_new_conversation(conn, trimmed_prompt, user_id)?
            } else {
                cid
            }
        }
        None => create_new_conversation(conn, trimmed_prompt, user_id)?,
    };

    // 3. Load last 10 messages for conversation context
    let mut history_messages = load_recent_messages(conn, &conv_id, 10)?;

    // 4. Privacy Redaction Engine setup
    let settings = get_privacy_settings(conn)?;
    let mut redaction_engine = PrivacyRedactionEngine::new(settings.clone());
    let scrubbed_prompt = redaction_engine.scrub_prompt(trimmed_prompt);

    // Build gateway messages list
    let mut gateway_messages = vec![GatewayMessage {
        role: "system".to_string(),
        content: Some(SYSTEM_PROMPT.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];

    for hist in history_messages.drain(..) {
        gateway_messages.push(hist);
    }

    gateway_messages.push(GatewayMessage {
        role: "user".to_string(),
        content: Some(scrubbed_prompt.clone()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    // 5. Multi-Hop Tool Loop (Max 3 rounds)
    let tool_schemas = get_available_tool_schemas(has_financial_perm);
    let mut tools_used = Vec::new();
    let mut total_prompt_tokens = 0u32;
    let mut total_completion_tokens = 0u32;
    let mut final_content = String::new();

    let mut round = 0;
    while round < 3 {
        round += 1;

        let req = GatewayRequest {
            model: "anthropic/claude-3.5-sonnet".to_string(),
            messages: gateway_messages.clone(),
            tools: tool_schemas.clone(),
            tool_choice: Some("auto".to_string()),
            temperature: Some(0.2),
        };

        let response = match send_chat_completion(conn, &req, &request_id) {
            Ok(res) => res,
            Err(e) => {
                // Log failed audit entry
                let _ = record_audit_log(
                    conn,
                    &request_id,
                    user_id,
                    "prompt",
                    trimmed_prompt,
                    &tools_used,
                    &redaction_engine,
                    total_prompt_tokens,
                    total_completion_tokens,
                    0,
                    start_time.elapsed().as_millis() as u64,
                    match &e {
                        AiError::CreditsExhausted => "credits_exhausted",
                        AiError::RateLimited => "rate_limited",
                        AiError::GatewayUnreachable(_) => "gateway_offline",
                        _ => "error",
                    },
                    Some(&e.to_string()),
                );
                return Err(e);
            }
        };

        total_prompt_tokens += response.usage.prompt_tokens;
        total_completion_tokens += response.usage.completion_tokens;

        let choice = response.choices.first().ok_or_else(|| {
            AiError::GatewayUnreachable("Gateway returned no choices".to_string())
        })?;

        // Check if tool invocation was requested
        if let Some(ref tool_calls) = choice.message.tool_calls {
            if !tool_calls.is_empty() {
                // Append assistant message requesting tool calls
                gateway_messages.push(GatewayMessage {
                    role: "assistant".to_string(),
                    content: choice.message.content.clone(),
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                    name: None,
                });

                for call in tool_calls {
                    let tool_name = &call.function.name;
                    if !tools_used.contains(tool_name) {
                        tools_used.push(tool_name.clone());
                    }

                    // Execute local tool securely
                    let tool_result_val = execute_tool(
                        conn,
                        tool_name,
                        &call.function.arguments,
                        has_financial_perm,
                    );

                    let (mut val, is_err) = match tool_result_val {
                        Ok(v) => (v, false),
                        Err(err) => (serde_json::json!({ "error": err.to_string() }), true),
                    };

                    if !is_err {
                        redaction_engine.redact_tool_output(&mut val);
                    }

                    let val_str = serde_json::to_string(&val).unwrap();

                    gateway_messages.push(GatewayMessage {
                        role: "tool".to_string(),
                        content: Some(val_str),
                        tool_calls: None,
                        tool_call_id: Some(call.id.clone()),
                        name: Some(tool_name.clone()),
                    });
                }
                continue;
            }
        }

        // Final text response received
        if let Some(ref text) = choice.message.content {
            final_content = text.clone();
            break;
        }

        break;
    }

    if final_content.is_empty() {
        final_content = "I could not formulate a response. Please refine your inquiry.".to_string();
    }

    // 6. In-Core De-Anonymization Re-hydration
    let (rehydrated_content, _) = redaction_engine.rehydrate_content(&final_content);
    let protection = redaction_engine.protection_metadata();

    // 7. Calculate credits to deduct (success only)
    let total_tokens = total_prompt_tokens + total_completion_tokens;
    let credits_to_deduct = (total_tokens as i64 / 100).max(1);

    let remaining_balance = deduct_credits(
        conn,
        credits_to_deduct,
        &request_id,
        &format!("Copilot Q: {}", truncate_str(trimmed_prompt, 40)),
        Some(user_id),
    )?;

    // 8. Persist turn in conversation history
    let now = crate::db::now_utc_iso();
    let user_msg_id = Uuid::now_v7().to_string();
    let asst_msg_id = Uuid::now_v7().to_string();

    conn.execute(
        "INSERT INTO ai_copilot_messages (id, conversation_id, role, content, credits_deducted, created_at)
         VALUES (?1, ?2, 'user', ?3, 0, ?4)",
        params![user_msg_id, conv_id, trimmed_prompt, now],
    )?;

    conn.execute(
        "INSERT INTO ai_copilot_messages (id, conversation_id, role, content, tool_calls_json, credits_deducted, created_at)
         VALUES (?1, ?2, 'assistant', ?3, ?4, ?5, ?6)",
        params![
            asst_msg_id,
            conv_id,
            rehydrated_content,
            serde_json::to_string(&tools_used).unwrap(),
            credits_to_deduct,
            now,
        ],
    )?;

    conn.execute(
        "UPDATE ai_copilot_conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, conv_id],
    )?;

    // 9. Permanent AI Audit Log Recording
    let latency = start_time.elapsed().as_millis() as u64;
    record_audit_log(
        conn,
        &request_id,
        user_id,
        "prompt",
        trimmed_prompt,
        &tools_used,
        &redaction_engine,
        total_prompt_tokens,
        total_completion_tokens,
        credits_to_deduct,
        latency,
        "success",
        None,
    )?;

    Ok(CopilotResponse {
        conversation_id: conv_id,
        message_id: asst_msg_id,
        content: rehydrated_content,
        tools_used,
        credits_deducted: credits_to_deduct,
        balance_remaining: remaining_balance,
        protection_metadata: protection,
    })
}

fn create_new_conversation(
    conn: &Connection,
    prompt: &str,
    user_id: &str,
) -> Result<String, AiError> {
    let id = Uuid::now_v7().to_string();
    let title = truncate_str(prompt, 50);
    let now = crate::db::now_utc_iso();

    conn.execute(
        "INSERT INTO ai_copilot_conversations (id, title, user_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, title, user_id, now, now],
    )?;

    Ok(id)
}

fn load_recent_messages(
    conn: &Connection,
    conv_id: &str,
    limit: usize,
) -> Result<Vec<GatewayMessage>, AiError> {
    let sql = r#"
        SELECT role, content
        FROM ai_copilot_messages
        WHERE conversation_id = ?1 AND role IN ('user', 'assistant')
        ORDER BY created_at DESC
        LIMIT ?2
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![conv_id, limit], |r| {
        let role: String = r.get(0)?;
        let content: String = r.get(1)?;
        Ok(GatewayMessage {
            role,
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }
    // Reverse to chronological order
    messages.reverse();
    Ok(messages)
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", chars[..max_chars].iter().collect::<String>())
    }
}

#[allow(clippy::too_many_arguments)]
fn record_audit_log(
    conn: &Connection,
    request_id: &str,
    user_id: &str,
    action_type: &str,
    prompt: &str,
    tools_used: &[String],
    redaction: &PrivacyRedactionEngine,
    p_tokens: u32,
    c_tokens: u32,
    credits: i64,
    latency: u64,
    status: &str,
    error_msg: Option<&str>,
) -> Result<(), AiError> {
    let id = Uuid::now_v7().to_string();
    let now = crate::db::now_utc_iso();
    let prompt_summary = truncate_str(prompt, 200);

    let mut allowed = Vec::new();
    if redaction.settings.allow_student_names {
        allowed.push("student_names");
    }
    if redaction.settings.allow_phone_numbers {
        allowed.push("phone_numbers");
    }
    if redaction.settings.allow_guardian_info {
        allowed.push("guardian_info");
    }
    if redaction.settings.allow_financial_figures {
        allowed.push("financial_figures");
    }
    if redaction.settings.allow_exam_marks {
        allowed.push("exam_marks");
    }
    if redaction.settings.allow_staff_names {
        allowed.push("staff_names");
    }

    conn.execute(
        "INSERT INTO ai_audit_logs (
            id, request_id, user_id, action_type, prompt_summary, tools_invoked,
            categories_allowed, categories_redacted, redaction_count,
            prompt_tokens, completion_tokens, credits_used, latency_ms,
            status, error_message, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            id,
            request_id,
            user_id,
            action_type,
            prompt_summary,
            serde_json::to_string(tools_used).unwrap(),
            serde_json::to_string(&allowed).unwrap(),
            serde_json::to_string(&redaction.redacted_categories).unwrap(),
            redaction.total_redactions as i64,
            p_tokens,
            c_tokens,
            credits,
            latency as i64,
            status,
            error_msg,
            now
        ],
    )?;

    Ok(())
}

// -----------------------------------------------------------------------------
// History Management & Queries
// -----------------------------------------------------------------------------

pub fn get_conversation(
    conn: &Connection,
    conv_id: &str,
    user_id: &str,
) -> Result<CopilotConversationDetails, AiError> {
    let title: String = conn
        .query_row(
            "SELECT title FROM ai_copilot_conversations WHERE id = ?1 AND user_id = ?2",
            params![conv_id, user_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| {
            AiError::ConversationNotFound(format!("Conversation '{}' not found", conv_id))
        })?;

    let sql = r#"
        SELECT id, role, content, tool_calls_json, tool_result_json, credits_deducted, created_at
        FROM ai_copilot_messages
        WHERE conversation_id = ?1
        ORDER BY created_at ASC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![conv_id], |r| {
        Ok(CopilotMessageItem {
            id: r.get(0)?,
            role: r.get(1)?,
            content: r.get(2)?,
            tool_calls_json: r.get(3)?,
            tool_result_json: r.get(4)?,
            credits_deducted: r.get(5)?,
            created_at: r.get(6)?,
        })
    })?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row?);
    }

    Ok(CopilotConversationDetails {
        id: conv_id.to_string(),
        title,
        messages,
    })
}

pub fn clear_conversation(conn: &Connection, conv_id: &str, user_id: &str) -> Result<(), AiError> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM ai_copilot_conversations WHERE id = ?1 AND user_id = ?2",
            params![conv_id, user_id],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);

    if !exists {
        return Err(AiError::ConversationNotFound(format!(
            "Conversation '{}' not found",
            conv_id
        )));
    }

    // Delete messages; keep conversation shell or delete both
    conn.execute(
        "DELETE FROM ai_copilot_messages WHERE conversation_id = ?1",
        params![conv_id],
    )?;
    conn.execute(
        "DELETE FROM ai_copilot_conversations WHERE id = ?1",
        params![conv_id],
    )?;

    Ok(())
}

pub fn get_audit_log(
    conn: &Connection,
    limit: Option<u32>,
    before_time: Option<&str>,
) -> Result<Vec<AiAuditLogItem>, AiError> {
    let lim = limit.unwrap_or(50).clamp(1, 100);

    let sql = r#"
        SELECT
            l.id,
            l.request_id,
            l.user_id,
            u.display_name AS user_name,
            l.action_type,
            l.prompt_summary,
            l.tools_invoked,
            l.categories_allowed,
            l.categories_redacted,
            l.redaction_count,
            l.prompt_tokens,
            l.completion_tokens,
            l.credits_used,
            l.latency_ms,
            l.status,
            l.error_message,
            l.created_at
        FROM ai_audit_logs l
        LEFT JOIN users u ON u.id = l.user_id
        WHERE (?1 IS NULL OR l.created_at < ?1)
        ORDER BY l.created_at DESC
        LIMIT ?2
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![before_time, lim], |r| {
        let tools_raw: String = r.get(6)?;
        let allowed_raw: String = r.get(7)?;
        let redacted_raw: String = r.get(8)?;

        Ok(AiAuditLogItem {
            id: r.get(0)?,
            request_id: r.get(1)?,
            user_id: r.get(2)?,
            user_name: r.get(3)?,
            action_type: r.get(4)?,
            prompt_summary: r.get(5)?,
            tools_invoked: serde_json::from_str(&tools_raw).unwrap_or_default(),
            categories_allowed: serde_json::from_str(&allowed_raw).unwrap_or_default(),
            categories_redacted: serde_json::from_str(&redacted_raw).unwrap_or_default(),
            redaction_count: r.get::<_, i64>(9)? as usize,
            prompt_tokens: r.get::<_, i64>(10)? as u32,
            completion_tokens: r.get::<_, i64>(11)? as u32,
            credits_used: r.get(12)?,
            latency_ms: r.get::<_, i64>(13)? as u64,
            status: r.get(14)?,
            error_message: r.get(15)?,
            created_at: r.get(16)?,
        })
    })?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}
