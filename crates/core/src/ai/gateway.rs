use crate::ai::tools::ToolDeclaration;
use crate::ai::AiError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AiGatewayConfig {
    pub gateway_url: String,
    pub is_configured: bool,
    pub is_mock: bool,
    pub upstream_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GatewayTestResult {
    pub ok: bool,
    pub latency_ms: u64,
    pub provider_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GatewayToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayToolCall {
    pub id: String,
    pub r#type: String,
    pub function: GatewayFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub model: String,
    pub messages: Vec<GatewayMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayChoiceMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GatewayToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayChoice {
    pub index: usize,
    pub message: GatewayChoiceMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub credits_deducted: Option<i64>,
    pub balance_remaining: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub id: String,
    pub choices: Vec<GatewayChoice>,
    pub usage: GatewayUsage,
}

// -----------------------------------------------------------------------------
// Mock Gateway Harness for Unit & Integration Testing
// -----------------------------------------------------------------------------

type MockHandler = Box<dyn Fn(&GatewayRequest) -> Result<GatewayResponse, AiError> + Send + Sync>;
static MOCK_GATEWAY: RwLock<Option<MockHandler>> = RwLock::new(None);

pub fn set_mock_gateway_response<F>(handler: F)
where
    F: Fn(&GatewayRequest) -> Result<GatewayResponse, AiError> + Send + Sync + 'static,
{
    let mut guard = MOCK_GATEWAY.write().unwrap();
    *guard = Some(Box::new(handler));
}

pub fn clear_mock_gateway() {
    let mut guard = MOCK_GATEWAY.write().unwrap();
    *guard = None;
}

pub fn is_mock_active() -> bool {
    let guard = MOCK_GATEWAY.read().unwrap();
    guard.is_some()
}

// -----------------------------------------------------------------------------
// Configuration & Credentials Management
// -----------------------------------------------------------------------------

pub fn get_gateway_config(conn: &Connection) -> Result<AiGatewayConfig, AiError> {
    let row = conn
        .query_row(
            "SELECT gateway_url, token_dpapi_blob FROM ai_credit_wallets WHERE id = 'default' LIMIT 1",
            [],
            |r| {
                let url: String = r.get(0)?;
                let blob: Option<Vec<u8>> = r.get(1)?;
                let configured = blob.map(|b| !b.is_empty()).unwrap_or(false);
                Ok((url, configured))
            },
        )
        .optional()?;

    let (url, configured) =
        row.unwrap_or_else(|| ("https://ai.coachify.cloud/v1".to_string(), false));

    let mock_active = is_mock_active();

    let provider = if url.contains("openrouter.ai") {
        "openrouter".to_string()
    } else {
        "hosted_gateway".to_string()
    };

    Ok(AiGatewayConfig {
        gateway_url: url,
        is_configured: configured || mock_active,
        is_mock: mock_active,
        upstream_provider: provider,
    })
}

pub fn update_gateway_config(
    conn: &Connection,
    gateway_url: String,
    token: Option<String>,
    openrouter_key: Option<String>,
    _user_id: Option<&str>,
) -> Result<AiGatewayConfig, AiError> {
    let trimmed_url = gateway_url.trim().to_string();
    if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
        return Err(AiError::Validation(
            "Gateway URL must begin with http:// or https://".to_string(),
        ));
    }

    let token_to_wrap = if let Some(ref or_key) = openrouter_key {
        if !or_key.trim().is_empty() {
            Some(or_key.trim())
        } else {
            token.as_deref().map(|s| s.trim())
        }
    } else {
        token.as_deref().map(|s| s.trim())
    };

    let blob = if let Some(raw_tok) = token_to_wrap {
        if !raw_tok.is_empty() {
            let wrapped = crate::crypto::wrap_dpapi_machine(raw_tok.as_bytes()).map_err(|e| {
                AiError::Validation(format!("Failed to encrypt gateway token: {}", e))
            })?;
            Some(wrapped)
        } else {
            None
        }
    } else {
        None
    };

    let now = crate::db::now_utc_iso();

    conn.execute(
        "UPDATE ai_credit_wallets
         SET gateway_url = ?1,
             token_dpapi_blob = COALESCE(?2, token_dpapi_blob),
             updated_at = ?3
         WHERE id = 'default'",
        params![trimmed_url, blob, now],
    )?;

    get_gateway_config(conn)
}

pub fn get_decrypted_token(conn: &Connection) -> Result<Option<String>, AiError> {
    let blob: Option<Vec<u8>> = conn
        .query_row(
            "SELECT token_dpapi_blob FROM ai_credit_wallets WHERE id = 'default' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    match blob {
        Some(b) if !b.is_empty() => {
            let decrypted = crate::crypto::unwrap_dpapi_machine(&b).map_err(|e| {
                AiError::GatewayAuthFailed(format!("Failed to decrypt token: {}", e))
            })?;
            let token_str = String::from_utf8(decrypted)
                .map_err(|e| AiError::GatewayAuthFailed(format!("Invalid UTF-8 token: {}", e)))?;
            Ok(Some(token_str))
        }
        _ => Ok(None),
    }
}

// -----------------------------------------------------------------------------
// Dispatch Gateway Request (Mock or HTTPS via ureq)
// -----------------------------------------------------------------------------

pub fn send_chat_completion(
    conn: &Connection,
    request: &GatewayRequest,
    request_id: &str,
) -> Result<GatewayResponse, AiError> {
    // 1. Check if mock harness is active
    {
        let guard = MOCK_GATEWAY.read().unwrap();
        if let Some(ref handler) = *guard {
            return handler(request);
        }
    }

    // 2. Resolve gateway URL and credentials
    let (url, _) = {
        let cfg = get_gateway_config(conn)?;
        (cfg.gateway_url, cfg.is_configured)
    };

    let token = get_decrypted_token(conn)?.unwrap_or_else(|| "anonymous_demo_token".to_string());

    let target_endpoint = if url.ends_with("/chat/completions") {
        url.clone()
    } else if url.ends_with("/v1") {
        format!("{}/chat/completions", url)
    } else {
        format!("{}/v1/chat/completions", url.trim_end_matches('/'))
    };

    let auth_header = format!("Bearer {}", token);

    let res = ureq::post(&target_endpoint)
        .set("Authorization", &auth_header)
        .set("Content-Type", "application/json")
        .set("X-Coachify-App-Version", "0.1.0")
        .set("X-Coachify-Platform", "windows-x86_64")
        .set("X-Coachify-Request-Id", request_id)
        .timeout(std::time::Duration::from_secs(30))
        .send_json(serde_json::to_value(request).unwrap());

    match res {
        Ok(response) => {
            let parsed: GatewayResponse = response.into_json().map_err(|e| {
                AiError::GatewayUnreachable(format!("Failed to parse gateway response JSON: {}", e))
            })?;
            Ok(parsed)
        }
        Err(ureq::Error::Status(402, _resp)) => Err(AiError::CreditsExhausted),
        Err(ureq::Error::Status(401, _resp)) | Err(ureq::Error::Status(403, _resp)) => Err(
            AiError::GatewayAuthFailed("Gateway rejected token authentication".to_string()),
        ),
        Err(ureq::Error::Status(429, _resp)) => Err(AiError::RateLimited),
        Err(ureq::Error::Status(status, resp)) => {
            let err_text = resp.into_string().unwrap_or_default();
            Err(AiError::GatewayUnreachable(format!(
                "Gateway HTTP error {}: {}",
                status, err_text
            )))
        }
        Err(ureq::Error::Transport(transport_err)) => {
            Err(AiError::GatewayUnreachable(transport_err.to_string()))
        }
    }
}

pub fn test_gateway_connection(conn: &Connection) -> Result<GatewayTestResult, AiError> {
    let start = Instant::now();

    if is_mock_active() {
        return Ok(GatewayTestResult {
            ok: true,
            latency_ms: 5,
            provider_status: "mock_active".to_string(),
        });
    }

    let req = GatewayRequest {
        model: "openai/gpt-4o-mini".to_string(),
        messages: vec![GatewayMessage {
            role: "user".to_string(),
            content: Some("ping".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }],
        tools: Vec::new(),
        tool_choice: None,
        temperature: Some(0.0),
    };

    let test_id = format!("test-{}", uuid::Uuid::now_v7());
    match send_chat_completion(conn, &req, &test_id) {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(GatewayTestResult {
                ok: true,
                latency_ms: elapsed,
                provider_status: "connected".to_string(),
            })
        }
        Err(AiError::CreditsExhausted) => Ok(GatewayTestResult {
            ok: true,
            latency_ms: start.elapsed().as_millis() as u64,
            provider_status: "connected_credits_exhausted".to_string(),
        }),
        Err(e) => Err(e),
    }
}
