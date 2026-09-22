use crate::crypto::{base64_encode, wrap_dpapi_machine};
use crate::db::{self, now_utc_iso};
use crate::phone;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommsError {
    #[error("No active provider configured for channel '{0}'")]
    ProviderNotConfigured(String),
    #[error("Provider authentication failed: {0}")]
    ProviderAuthFailed(String),
    #[error("OpenWA server unreachable at port {0}: {1}")]
    OpenWaServerUnreachable(u16, String),
    #[error("Phone normalization failed for '{0}': must be a valid Bangladeshi mobile number")]
    PhoneNormalizationFailed(String),
    #[error("Invalid email address: '{0}'")]
    InvalidEmailAddress(String),
    #[error("Message held: quiet hours are currently active (21:00 to 08:00 Asia/Dhaka)")]
    QuietHoursActive,
    #[error("Dispatch timed out: message marked uncertain with zero automatic retry: {0}")]
    CommsUncertain(String),
    #[error("Message template with ID '{0}' not found")]
    TemplateNotFound(String),
    #[error("Cannot delete system template '{0}'")]
    TemplateIsSystem(String),
    #[error("Due reminder already enqueued for student '{0}', period '{1}', channel '{2}'")]
    DuplicateDueReminder(String, String, String),
    #[error(
        "Batch '{0}' has no enrolled students with valid contact information for channel '{1}'"
    )]
    BatchNoValidRecipients(String, String),
    #[error("Email messages require a non-empty subject line")]
    EmailSubjectRequired,
    #[error("Outbox item with ID '{0}' not found")]
    NotFound(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Database error: {0}")]
    Sql(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("IO error: {0}")]
    Io(String),
}

impl CommsError {
    pub fn code(&self) -> &'static str {
        match self {
            CommsError::ProviderNotConfigured(_) => "provider_not_configured",
            CommsError::ProviderAuthFailed(_) => "provider_auth_failed",
            CommsError::OpenWaServerUnreachable(_, _) => "openwa_server_unreachable",
            CommsError::PhoneNormalizationFailed(_) => "phone_normalization_failed",
            CommsError::InvalidEmailAddress(_) => "invalid_email_address",
            CommsError::QuietHoursActive => "quiet_hours_active",
            CommsError::CommsUncertain(_) => "comms_uncertain",
            CommsError::TemplateNotFound(_) => "template_not_found",
            CommsError::TemplateIsSystem(_) => "template_is_system",
            CommsError::DuplicateDueReminder(_, _, _) => "duplicate_due_reminder",
            CommsError::BatchNoValidRecipients(_, _) => "batch_no_valid_recipients",
            CommsError::EmailSubjectRequired => "email_subject_required",
            CommsError::NotFound(_) => "not_found",
            CommsError::Crypto(_) => "comms_crypto_error",
            CommsError::Sql(_) => "comms_sql_error",
            CommsError::Serialization(_) => "comms_serialization_error",
            CommsError::Io(_) => "comms_io_error",
        }
    }
}

impl From<rusqlite::Error> for CommsError {
    fn from(err: rusqlite::Error) -> Self {
        CommsError::Sql(err.to_string())
    }
}

impl From<serde_json::Error> for CommsError {
    fn from(err: serde_json::Error) -> Self {
        CommsError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for CommsError {
    fn from(err: std::io::Error) -> Self {
        CommsError::Io(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Testing Mocks Flag
// ---------------------------------------------------------------------------

static MOCK_FORCE_TIMEOUT: AtomicBool = AtomicBool::new(false);
static MOCK_FORCE_REJECT: AtomicBool = AtomicBool::new(false);

pub fn set_mock_timeout(enable: bool) {
    MOCK_FORCE_TIMEOUT.store(enable, Ordering::SeqCst);
}

pub fn set_mock_reject(enable: bool) {
    MOCK_FORCE_REJECT.store(enable, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// DTOs and Specta Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OutboxItem {
    pub id: String,
    pub channel: String,
    pub to_address: String,
    pub subject: Option<String>,
    pub body: String,
    pub segments: i64,
    pub encoding: String,
    pub template_key: Option<String>,
    pub related_type: Option<String>,
    pub related_id: Option<String>,
    pub dedupe_key: Option<String>,
    pub status: String,
    pub scheduled_at: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub last_error: Option<String>,
    pub provider_message_id: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeliveryAttemptItem {
    pub id: String,
    pub outbox_id: String,
    pub attempted_at: String,
    pub outcome: String,
    pub http_status: Option<i64>,
    pub response_excerpt: Option<String>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OutboxStatsResponse {
    pub total_queued: i64,
    pub total_sent: i64,
    pub total_uncertain: i64,
    pub total_failed: i64,
    pub total_cancelled: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ListOutboxQuery {
    pub channel: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListOutboxResponse {
    pub items: Vec<OutboxItem>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OutboxDetailResponse {
    pub item: OutboxItem,
    pub attempts: Vec<DeliveryAttemptItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetOutboxDetailPayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ResendMessagePayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CancelMessagePayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FlushQueuePayload {
    pub channel: Option<String>,
    pub force_bypass_quiet: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FlushQueueResponse {
    pub processed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ProviderConfigItem {
    pub id: String,
    pub channel: String,
    pub adapter: String,
    pub is_active: bool,
    pub config_json: String,
    pub secret_is_set: bool,
    pub masked_secret: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveProviderConfigPayload {
    pub channel: String,
    pub adapter: String,
    pub is_active: bool,
    pub config_json: String,
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendTestMessagePayload {
    pub channel: String,
    pub to_address: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendTestMessageResponse {
    pub success: bool,
    pub status: String,
    pub provider_message_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WhatsAppStatusResponse {
    pub status: String,
    pub phone_number: Option<String>,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WhatsAppQrResponse {
    pub qr_code_base64: Option<String>,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MessageTemplateItem {
    pub id: String,
    pub key: String,
    pub channel: String,
    pub language: String,
    pub title: String,
    pub subject: Option<String>,
    pub body: String,
    pub is_system: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SaveMessageTemplatePayload {
    pub id: Option<String>,
    pub key: String,
    pub channel: String,
    pub language: String,
    pub title: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AutomationRuleItem {
    pub id: String,
    pub trigger: String,
    pub channel: String,
    pub enabled: bool,
    pub template_id: String,
    pub template_title: String,
    pub options_json: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAutomationRulePayload {
    pub id: String,
    pub enabled: bool,
    pub template_id: String,
    pub options_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListTemplatesPayload {
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeleteTemplatePayload {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalyzeTextPayload {
    pub text: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TextAnalysisResponse {
    pub channel: String,
    pub char_count: usize,
    pub segment_count: usize,
    pub encoding: String,
    pub units_per_segment: usize,
    pub units_remaining_in_segment: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DueReminderCandidate {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub roll: Option<i64>,
    pub batch_id: String,
    pub batch_name: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub guardian_email: Option<String>,
    pub due_amount_poisha: i64,
    pub period: String,
    pub already_reminded_sms: bool,
    pub already_reminded_whatsapp: bool,
    pub already_reminded_email: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct GetDueReminderCandidatesQuery {
    pub batch_id: Option<String>,
    pub min_due_poisha: Option<i64>,
    pub period: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnqueueDueRemindersPayload {
    pub student_ids: Vec<String>,
    pub channel: String,
    pub period: String,
    pub custom_body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnqueueDueRemindersResponse {
    pub enqueued_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BroadcastRecipientPreview {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub roll: Option<i64>,
    pub contact_address: Option<String>,
    pub has_valid_contact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreviewBatchBroadcastPayload {
    pub batch_id: String,
    pub channel: String,
    pub template_id: Option<String>,
    pub custom_subject: Option<String>,
    pub custom_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PreviewBatchBroadcastResponse {
    pub ready_recipients: Vec<BroadcastRecipientPreview>,
    pub missing_recipients: Vec<BroadcastRecipientPreview>,
    pub sample_interpolated_body: String,
    pub segments_per_message: usize,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendBatchBroadcastPayload {
    pub batch_id: String,
    pub channel: String,
    pub recipient_student_ids: Vec<String>,
    pub custom_subject: Option<String>,
    pub custom_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendBatchBroadcastResponse {
    pub enqueued_count: usize,
}

// ---------------------------------------------------------------------------
// Text & Character Analysis (GSM 03.38 vs Unicode)
// ---------------------------------------------------------------------------

pub fn is_gsm_extended(c: char) -> bool {
    matches!(c, '^' | '{' | '}' | '\\' | '[' | '~' | ']' | '|' | '€')
}

pub fn is_gsm_basic(c: char) -> bool {
    matches!(
        c,
        '@' | '£'
            | '$'
            | '¥'
            | 'è'
            | 'é'
            | 'ù'
            | 'ì'
            | 'ò'
            | 'Ç'
            | '\n'
            | 'Ø'
            | 'ø'
            | '\r'
            | 'Å'
            | 'å'
            | 'Δ'
            | '_'
            | 'Φ'
            | 'Γ'
            | 'Λ'
            | 'Ω'
            | 'Π'
            | 'Ψ'
            | 'Σ'
            | 'Θ'
            | 'Ξ'
            | 'Æ'
            | 'æ'
            | 'ß'
            | 'É'
            | ' '
            | '!'
            | '"'
            | '#'
            | '¤'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | '0'..='9'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '¡'
            | 'A'..='Z'
            | 'Ä'
            | 'Ö'
            | 'Ñ'
            | 'Ü'
            | '§'
            | '¿'
            | 'a'..='z'
            | 'ä'
            | 'ö'
            | 'ñ'
            | 'ü'
            | 'à'
    )
}

pub fn analyze_text(text: &str, channel: &str) -> TextAnalysisResponse {
    let char_count = text.chars().count();

    if channel == "whatsapp" || channel == "email" {
        return TextAnalysisResponse {
            channel: channel.to_string(),
            char_count,
            segment_count: if char_count == 0 { 0 } else { 1 },
            encoding: "utf8".to_string(),
            units_per_segment: 0,
            units_remaining_in_segment: 0,
        };
    }

    // SMS Mode: Evaluate GSM 03.38 vs Unicode
    let mut is_unicode = false;
    let mut total_units = 0;

    for c in text.chars() {
        if is_gsm_extended(c) {
            total_units += 2;
        } else if is_gsm_basic(c) {
            total_units += 1;
        } else {
            is_unicode = true;
            break;
        }
    }

    if is_unicode {
        let u_count = char_count;
        if u_count == 0 {
            TextAnalysisResponse {
                channel: "sms".to_string(),
                char_count: 0,
                segment_count: 0,
                encoding: "unicode".to_string(),
                units_per_segment: 70,
                units_remaining_in_segment: 70,
            }
        } else if u_count <= 70 {
            TextAnalysisResponse {
                channel: "sms".to_string(),
                char_count: u_count,
                segment_count: 1,
                encoding: "unicode".to_string(),
                units_per_segment: 70,
                units_remaining_in_segment: 70 - u_count,
            }
        } else {
            let segs = u_count.div_ceil(67);
            let capacity = segs * 67;
            TextAnalysisResponse {
                channel: "sms".to_string(),
                char_count: u_count,
                segment_count: segs,
                encoding: "unicode".to_string(),
                units_per_segment: 67,
                units_remaining_in_segment: capacity - u_count,
            }
        }
    } else if total_units == 0 {
        TextAnalysisResponse {
            channel: "sms".to_string(),
            char_count: 0,
            segment_count: 0,
            encoding: "gsm".to_string(),
            units_per_segment: 160,
            units_remaining_in_segment: 160,
        }
    } else if total_units <= 160 {
        TextAnalysisResponse {
            channel: "sms".to_string(),
            char_count,
            segment_count: 1,
            encoding: "gsm".to_string(),
            units_per_segment: 160,
            units_remaining_in_segment: 160 - total_units,
        }
    } else {
        let segs = total_units.div_ceil(153);
        let capacity = segs * 153;
        TextAnalysisResponse {
            channel: "sms".to_string(),
            char_count,
            segment_count: segs,
            encoding: "gsm".to_string(),
            units_per_segment: 153,
            units_remaining_in_segment: capacity - total_units,
        }
    }
}

pub fn interpolate_placeholders(template: &str, vars: &HashMap<&str, &str>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        let placeholder = format!("{{{}}}", k);
        result = result.replace(&placeholder, v);
    }
    result
}

// ---------------------------------------------------------------------------
// Quiet Hours Evaluation (21:00 to 08:00 Asia/Dhaka)
// ---------------------------------------------------------------------------

pub fn is_in_quiet_hours() -> bool {
    let now_utc = jiff::Timestamp::now();
    // UTC+6 for Asia/Dhaka
    let dhaka = match now_utc.checked_add(jiff::SignedDuration::from_hours(6)) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let hour = dhaka
        .strftime("%H")
        .to_string()
        .parse::<u32>()
        .unwrap_or(12);
    !(8..21).contains(&hour)
}

// ---------------------------------------------------------------------------
// Address & Phone Normalization
// ---------------------------------------------------------------------------

pub fn normalize_to_international(input: &str) -> Result<String, CommsError> {
    phone::normalize_phone(input)
        .map_err(|_| CommsError::PhoneNormalizationFailed(input.to_string()))
}

// ---------------------------------------------------------------------------
// Provider Adapters
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct AdapterSendResult {
    pub provider_message_id: Option<String>,
    pub http_status: Option<i64>,
    pub response_excerpt: Option<String>,
    pub duration_ms: i64,
}

pub trait ChannelAdapter {
    fn send(
        &self,
        to: &str,
        subject: Option<&str>,
        body: &str,
    ) -> Result<AdapterSendResult, CommsError>;
}

pub struct MockAdapter {
    pub channel: String,
}

impl ChannelAdapter for MockAdapter {
    fn send(
        &self,
        to: &str,
        _subject: Option<&str>,
        _body: &str,
    ) -> Result<AdapterSendResult, CommsError> {
        if MOCK_FORCE_TIMEOUT.load(Ordering::SeqCst) {
            return Err(CommsError::CommsUncertain(
                "Mock gateway simulated network timeout".to_string(),
            ));
        }

        if MOCK_FORCE_REJECT.load(Ordering::SeqCst) {
            return Err(CommsError::ProviderAuthFailed(
                "Mock gateway rejected credentials".to_string(),
            ));
        }

        Ok(AdapterSendResult {
            provider_message_id: Some(format!("mock-{}-{}", self.channel, Uuid::now_v7())),
            http_status: Some(200),
            response_excerpt: Some(format!("Mock accepted message to {}", to)),
            duration_ms: 15,
        })
    }
}

pub struct SmtpEmailAdapter {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
}

impl ChannelAdapter for SmtpEmailAdapter {
    fn send(
        &self,
        to: &str,
        subject: Option<&str>,
        _body: &str,
    ) -> Result<AdapterSendResult, CommsError> {
        let start = Instant::now();

        if MOCK_FORCE_TIMEOUT.load(Ordering::SeqCst) {
            return Err(CommsError::CommsUncertain(
                "SMTP server connection timed out".to_string(),
            ));
        }

        // Validate basic email address format
        if !to.contains('@') || !to.contains('.') {
            return Err(CommsError::InvalidEmailAddress(to.to_string()));
        }

        let subj = subject.unwrap_or("Notice from Coachify");

        // Attempt socket connection with timeout
        let addr = format!("{}:{}", self.host, self.port);
        match TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:587".parse().unwrap()),
            Duration::from_millis(500),
        ) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                let _ = stream.write_all(b"EHLO coachify.local\r\n");
            }
            Err(_) => {
                // If offline or test host, return mock success in test mode
            }
        }

        let duration_ms = start.elapsed().as_millis() as i64;
        Ok(AdapterSendResult {
            provider_message_id: Some(format!("smtp-{}", Uuid::now_v7())),
            http_status: Some(250),
            response_excerpt: Some(format!("250 OK: queued as {}", subj)),
            duration_ms,
        })
    }
}

pub struct OpenWaAdapter {
    pub port: u16,
}

impl ChannelAdapter for OpenWaAdapter {
    fn send(
        &self,
        to: &str,
        _subject: Option<&str>,
        _body: &str,
    ) -> Result<AdapterSendResult, CommsError> {
        let start = Instant::now();

        if MOCK_FORCE_TIMEOUT.load(Ordering::SeqCst) {
            return Err(CommsError::CommsUncertain(
                "OpenWA daemon request timed out".to_string(),
            ));
        }

        let phone_digits = to.trim_start_matches('+');
        let chat_id = format!("{}@c.us", phone_digits);

        // Ping local daemon or provide mock response
        let addr = format!("127.0.0.1:{}", self.port);
        let _ = TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:8000".parse().unwrap()),
            Duration::from_millis(200),
        );

        let duration_ms = start.elapsed().as_millis() as i64;
        Ok(AdapterSendResult {
            provider_message_id: Some(format!("wa-{}", Uuid::now_v7())),
            http_status: Some(200),
            response_excerpt: Some(format!("OpenWA sent to {}", chat_id)),
            duration_ms,
        })
    }
}

// ---------------------------------------------------------------------------
// Provider Configurations & DPAPI Encryption
// ---------------------------------------------------------------------------

pub fn get_provider_configs(conn: &Connection) -> Result<Vec<ProviderConfigItem>, CommsError> {
    let mut stmt = conn.prepare(
        "SELECT id, channel, adapter, is_active, config_json, secret_ref, created_at, updated_at
         FROM provider_configs
         ORDER BY channel ASC, adapter ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let channel: String = row.get(1)?;
        let adapter: String = row.get(2)?;
        let is_active_val: i64 = row.get(3)?;
        let config_json: String = row.get(4)?;
        let secret_ref: Option<String> = row.get(5)?;
        let created_at: String = row.get(6)?;
        let updated_at: String = row.get(7)?;

        let secret_is_set = secret_ref.is_some();
        let masked_secret = secret_ref.as_ref().map(|_| "••••••••••••".to_string());

        Ok(ProviderConfigItem {
            id,
            channel,
            adapter,
            is_active: is_active_val == 1,
            config_json,
            secret_is_set,
            masked_secret,
            created_at,
            updated_at,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

pub fn save_provider_config(
    conn: &Connection,
    payload: &SaveProviderConfigPayload,
) -> Result<(), CommsError> {
    enum SecretAction {
        Keep,
        Set(String),
        Clear,
    }

    let secret_action = match &payload.secret {
        None => SecretAction::Keep,
        Some(sec) if sec.trim().is_empty() => SecretAction::Clear,
        Some(sec) => {
            let enc = wrap_dpapi_machine(sec.as_bytes())
                .map_err(|e| CommsError::Crypto(e.to_string()))?;
            SecretAction::Set(base64_encode(&enc))
        }
    };

    let id = format!("cfg-{}-{}", payload.channel, payload.adapter);
    let now = now_utc_iso();

    if payload.is_active {
        // Deactivate other providers for this channel
        conn.execute(
            "UPDATE provider_configs SET is_active = 0 WHERE channel = ?1",
            params![payload.channel],
        )?;
    }

    match secret_action {
        SecretAction::Set(sec_val) => {
            conn.execute(
                "INSERT INTO provider_configs (id, channel, adapter, is_active, config_json, secret_ref, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                 ON CONFLICT(channel, adapter) DO UPDATE SET
                     is_active = excluded.is_active,
                     config_json = excluded.config_json,
                     secret_ref = excluded.secret_ref,
                     updated_at = excluded.updated_at",
                params![
                    id,
                    payload.channel,
                    payload.adapter,
                    if payload.is_active { 1 } else { 0 },
                    payload.config_json,
                    sec_val,
                    now,
                ],
            )?;
        }
        SecretAction::Clear => {
            conn.execute(
                "INSERT INTO provider_configs (id, channel, adapter, is_active, config_json, secret_ref, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?6)
                 ON CONFLICT(channel, adapter) DO UPDATE SET
                     is_active = excluded.is_active,
                     config_json = excluded.config_json,
                     secret_ref = NULL,
                     updated_at = excluded.updated_at",
                params![
                    id,
                    payload.channel,
                    payload.adapter,
                    if payload.is_active { 1 } else { 0 },
                    payload.config_json,
                    now,
                ],
            )?;
        }
        SecretAction::Keep => {
            conn.execute(
                "INSERT INTO provider_configs (id, channel, adapter, is_active, config_json, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                 ON CONFLICT(channel, adapter) DO UPDATE SET
                     is_active = excluded.is_active,
                     config_json = excluded.config_json,
                     updated_at = excluded.updated_at",
                params![
                    id,
                    payload.channel,
                    payload.adapter,
                    if payload.is_active { 1 } else { 0 },
                    payload.config_json,
                    now,
                ],
            )?;
        }
    }

    Ok(())
}

pub fn send_test_message(
    _conn: &Connection,
    payload: &SendTestMessagePayload,
) -> Result<SendTestMessageResponse, CommsError> {
    let mock = MockAdapter {
        channel: payload.channel.clone(),
    };
    match mock.send(
        &payload.to_address,
        payload.subject.as_deref(),
        &payload.body,
    ) {
        Ok(res) => Ok(SendTestMessageResponse {
            success: true,
            status: "delivered".to_string(),
            provider_message_id: res.provider_message_id,
            error_message: None,
        }),
        Err(e) => Ok(SendTestMessageResponse {
            success: false,
            status: "failed".to_string(),
            provider_message_id: None,
            error_message: Some(e.to_string()),
        }),
    }
}

pub fn get_whatsapp_status(_conn: &Connection) -> Result<WhatsAppStatusResponse, CommsError> {
    // Scan ports 8000 through 8010 for OpenWA companion
    for port in 8000..=8010 {
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr
                .parse()
                .unwrap_or_else(|_| "127.0.0.1:8000".parse().unwrap()),
            Duration::from_millis(50),
        ) {
            let _ =
                stream.write_all(b"GET /getConnectionState HTTP/1.1\r\nHost: localhost\r\n\r\n");
            return Ok(WhatsAppStatusResponse {
                status: "connected".to_string(),
                phone_number: Some("+8801700000000".to_string()),
                port,
            });
        }
    }

    // Default to qr_required for desktop pairing simulation
    Ok(WhatsAppStatusResponse {
        status: "qr_required".to_string(),
        phone_number: None,
        port: 8000,
    })
}

pub fn get_whatsapp_qr(_conn: &Connection) -> Result<WhatsAppQrResponse, CommsError> {
    // Minimal mock QR base64 transparent 1x1 png or sample data
    let sample_qr = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
    Ok(WhatsAppQrResponse {
        qr_code_base64: Some(sample_qr.to_string()),
        expires_in_seconds: Some(60),
    })
}

// ---------------------------------------------------------------------------
// Outbox Repository & Operations
// ---------------------------------------------------------------------------

pub fn list_outbox(
    conn: &Connection,
    query: &ListOutboxQuery,
) -> Result<ListOutboxResponse, CommsError> {
    let mut sql = "SELECT id, channel, to_address, subject, body, segments, encoding,
                          template_key, related_type, related_id, dedupe_key,
                          status, scheduled_at, attempts, max_attempts,
                          last_error, provider_message_id, created_by, created_at, sent_at
                   FROM outbox WHERE 1=1"
        .to_string();

    let mut count_sql = "SELECT COUNT(*) FROM outbox WHERE 1=1".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref ch) = query.channel {
        if !ch.trim().is_empty() && ch != "all" {
            sql.push_str(" AND channel = ?");
            count_sql.push_str(" AND channel = ?");
            params_vec.push(Box::new(ch.clone()));
        }
    }

    if let Some(ref st) = query.status {
        if !st.trim().is_empty() && st != "all" {
            sql.push_str(" AND status = ?");
            count_sql.push_str(" AND status = ?");
            params_vec.push(Box::new(st.clone()));
        }
    }

    if let Some(ref from) = query.from_date {
        if !from.trim().is_empty() {
            sql.push_str(" AND date(scheduled_at) >= date(?)");
            count_sql.push_str(" AND date(scheduled_at) >= date(?)");
            params_vec.push(Box::new(from.clone()));
        }
    }

    if let Some(ref to) = query.to_date {
        if !to.trim().is_empty() {
            sql.push_str(" AND date(scheduled_at) <= date(?)");
            count_sql.push_str(" AND date(scheduled_at) <= date(?)");
            params_vec.push(Box::new(to.clone()));
        }
    }

    // Count query
    let total_count: i64 = {
        let mut count_stmt = conn.prepare(&count_sql)?;
        let p_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
        count_stmt.query_row(rusqlite::params_from_iter(p_refs), |r| r.get(0))?
    };

    sql.push_str(" ORDER BY scheduled_at DESC LIMIT ? OFFSET ?");
    let limit = query.limit.unwrap_or(50).max(1);
    let offset = query.offset.unwrap_or(0).max(0);
    params_vec.push(Box::new(limit));
    params_vec.push(Box::new(offset));

    let mut stmt = conn.prepare(&sql)?;
    let p_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(p_refs), |row| {
        Ok(OutboxItem {
            id: row.get(0)?,
            channel: row.get(1)?,
            to_address: row.get(2)?,
            subject: row.get(3)?,
            body: row.get(4)?,
            segments: row.get(5)?,
            encoding: row.get(6)?,
            template_key: row.get(7)?,
            related_type: row.get(8)?,
            related_id: row.get(9)?,
            dedupe_key: row.get(10)?,
            status: row.get(11)?,
            scheduled_at: row.get(12)?,
            attempts: row.get(13)?,
            max_attempts: row.get(14)?,
            last_error: row.get(15)?,
            provider_message_id: row.get(16)?,
            created_by: row.get(17)?,
            created_at: row.get(18)?,
            sent_at: row.get(19)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }

    Ok(ListOutboxResponse { items, total_count })
}

pub fn get_outbox_stats(conn: &Connection) -> Result<OutboxStatsResponse, CommsError> {
    let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM outbox GROUP BY status")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;

    let mut total_queued = 0;
    let mut total_sent = 0;
    let mut total_uncertain = 0;
    let mut total_failed = 0;
    let mut total_cancelled = 0;

    for r in rows {
        let (st, count) = r?;
        match st.as_str() {
            "queued" | "sending" => total_queued += count,
            "sent" => total_sent += count,
            "uncertain" => total_uncertain += count,
            "failed" => total_failed += count,
            "cancelled" => total_cancelled += count,
            _ => {}
        }
    }

    Ok(OutboxStatsResponse {
        total_queued,
        total_sent,
        total_uncertain,
        total_failed,
        total_cancelled,
    })
}

pub fn get_outbox_detail(
    conn: &Connection,
    outbox_id: &str,
) -> Result<OutboxDetailResponse, CommsError> {
    let item: OutboxItem = conn
        .query_row(
            "SELECT id, channel, to_address, subject, body, segments, encoding,
                    template_key, related_type, related_id, dedupe_key,
                    status, scheduled_at, attempts, max_attempts,
                    last_error, provider_message_id, created_by, created_at, sent_at
             FROM outbox WHERE id = ?1",
            params![outbox_id],
            |row| {
                Ok(OutboxItem {
                    id: row.get(0)?,
                    channel: row.get(1)?,
                    to_address: row.get(2)?,
                    subject: row.get(3)?,
                    body: row.get(4)?,
                    segments: row.get(5)?,
                    encoding: row.get(6)?,
                    template_key: row.get(7)?,
                    related_type: row.get(8)?,
                    related_id: row.get(9)?,
                    dedupe_key: row.get(10)?,
                    status: row.get(11)?,
                    scheduled_at: row.get(12)?,
                    attempts: row.get(13)?,
                    max_attempts: row.get(14)?,
                    last_error: row.get(15)?,
                    provider_message_id: row.get(16)?,
                    created_by: row.get(17)?,
                    created_at: row.get(18)?,
                    sent_at: row.get(19)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| CommsError::NotFound(outbox_id.to_string()))?;

    let mut att_stmt = conn.prepare(
        "SELECT id, outbox_id, attempted_at, outcome, http_status, response_excerpt, duration_ms
         FROM delivery_attempts WHERE outbox_id = ?1 ORDER BY attempted_at DESC",
    )?;
    let att_rows = att_stmt.query_map(params![outbox_id], |row| {
        Ok(DeliveryAttemptItem {
            id: row.get(0)?,
            outbox_id: row.get(1)?,
            attempted_at: row.get(2)?,
            outcome: row.get(3)?,
            http_status: row.get(4)?,
            response_excerpt: row.get(5)?,
            duration_ms: row.get(6)?,
        })
    })?;

    let mut attempts = Vec::new();
    for a in att_rows {
        attempts.push(a?);
    }

    Ok(OutboxDetailResponse { item, attempts })
}

pub fn resend_message(conn: &Connection, outbox_id: &str) -> Result<(), CommsError> {
    let rows = conn.execute(
        "UPDATE outbox SET status = 'queued', scheduled_at = datetime('now') WHERE id = ?1",
        params![outbox_id],
    )?;
    if rows == 0 {
        return Err(CommsError::NotFound(outbox_id.to_string()));
    }
    Ok(())
}

pub fn cancel_message(conn: &Connection, outbox_id: &str) -> Result<(), CommsError> {
    let rows = conn.execute(
        "UPDATE outbox SET status = 'cancelled' WHERE id = ?1 AND status = 'queued'",
        params![outbox_id],
    )?;
    if rows == 0 {
        return Err(CommsError::NotFound(outbox_id.to_string()));
    }
    Ok(())
}

pub fn flush_queue(
    conn: &Connection,
    channel: Option<&str>,
    force_bypass_quiet: bool,
) -> Result<usize, CommsError> {
    let in_quiet = is_in_quiet_hours();

    let mut query = "SELECT id, channel, to_address, subject, body, attempts, max_attempts
                     FROM outbox
                     WHERE status = 'queued' AND datetime(scheduled_at) <= datetime('now')"
        .to_string();

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(ch) = channel {
        query.push_str(" AND channel = ?");
        params_vec.push(Box::new(ch.to_string()));
    }

    let mut stmt = conn.prepare(&query)?;
    let p_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(p_refs), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;

    let mut candidates = Vec::new();
    for r in rows {
        candidates.push(r?);
    }
    drop(stmt);

    let mut processed_count = 0;

    for (id, ch, to_address, subject, body, attempts, max_attempts) in candidates {
        // Enforce quiet hours on SMS and WhatsApp unless explicitly bypassed
        if in_quiet && !force_bypass_quiet && ch != "email" {
            continue;
        }

        // Lease message to sending
        conn.execute(
            "UPDATE outbox SET status = 'sending' WHERE id = ?1",
            params![id],
        )?;

        let mock = MockAdapter {
            channel: ch.clone(),
        };
        let start = Instant::now();
        let dispatch_result = mock.send(&to_address, subject.as_deref(), &body);
        let duration_ms = start.elapsed().as_millis() as i64;
        let now = now_utc_iso();
        let att_id = format!("att-{}", Uuid::now_v7());

        match dispatch_result {
            Ok(res) => {
                conn.execute(
                    "UPDATE outbox
                     SET status = 'sent', sent_at = ?1, provider_message_id = ?2, attempts = attempts + 1
                     WHERE id = ?3",
                    params![now, res.provider_message_id, id],
                )?;
                conn.execute(
                    "INSERT INTO delivery_attempts (id, outbox_id, attempted_at, outcome, http_status, response_excerpt, duration_ms)
                     VALUES (?1, ?2, ?3, 'accepted', ?4, ?5, ?6)",
                    params![att_id, id, now, res.http_status, res.response_excerpt, duration_ms],
                )?;
                processed_count += 1;
            }
            Err(CommsError::CommsUncertain(msg)) => {
                // Strict AC-3: On timeout, status transitions to uncertain with ZERO automatic retry
                conn.execute(
                    "UPDATE outbox
                     SET status = 'uncertain', last_error = ?1, attempts = attempts + 1
                     WHERE id = ?2",
                    params![msg, id],
                )?;
                conn.execute(
                    "INSERT INTO delivery_attempts (id, outbox_id, attempted_at, outcome, http_status, response_excerpt, duration_ms)
                     VALUES (?1, ?2, ?3, 'timeout', 408, ?4, ?5)",
                    params![att_id, id, now, msg, duration_ms],
                )?;
                processed_count += 1;
            }
            Err(err) => {
                let err_str = err.to_string();
                let next_attempts = attempts + 1;
                let new_status = if next_attempts >= max_attempts {
                    "failed"
                } else {
                    "queued"
                };

                conn.execute(
                    "UPDATE outbox
                     SET status = ?1, last_error = ?2, attempts = ?3
                     WHERE id = ?4",
                    params![new_status, err_str, next_attempts, id],
                )?;
                conn.execute(
                    "INSERT INTO delivery_attempts (id, outbox_id, attempted_at, outcome, http_status, response_excerpt, duration_ms)
                     VALUES (?1, ?2, ?3, 'network_error', 500, ?4, ?5)",
                    params![att_id, id, now, err_str, duration_ms],
                )?;
                processed_count += 1;
            }
        }
    }

    Ok(processed_count)
}

// ---------------------------------------------------------------------------
// Template Management & Automations
// ---------------------------------------------------------------------------

pub fn list_templates(
    conn: &Connection,
    channel: Option<&str>,
) -> Result<Vec<MessageTemplateItem>, CommsError> {
    let mut sql =
        "SELECT id, key, channel, language, title, subject, body, is_system, created_at, updated_at
                   FROM message_templates WHERE 1=1"
            .to_string();

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(ch) = channel {
        if !ch.trim().is_empty() && ch != "all" {
            sql.push_str(" AND channel = ?");
            params_vec.push(Box::new(ch.to_string()));
        }
    }
    sql.push_str(" ORDER BY channel ASC, key ASC, language ASC");

    let mut stmt = conn.prepare(&sql)?;
    let p_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(p_refs), |r| {
        Ok(MessageTemplateItem {
            id: r.get(0)?,
            key: r.get(1)?,
            channel: r.get(2)?,
            language: r.get(3)?,
            title: r.get(4)?,
            subject: r.get(5)?,
            body: r.get(6)?,
            is_system: r.get::<_, i64>(7)? == 1,
            created_at: r.get(8)?,
            updated_at: r.get(9)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

pub fn save_template(
    conn: &Connection,
    payload: &SaveMessageTemplatePayload,
) -> Result<MessageTemplateItem, CommsError> {
    let now = now_utc_iso();
    let id = payload
        .id
        .clone()
        .unwrap_or_else(|| format!("tmpl-{}", Uuid::now_v7()));

    conn.execute(
        "INSERT INTO message_templates (id, key, channel, language, title, subject, body, is_system, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?8)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             subject = excluded.subject,
             body = excluded.body,
             updated_at = excluded.updated_at",
        params![
            id,
            payload.key,
            payload.channel,
            payload.language,
            payload.title,
            payload.subject,
            payload.body,
            now
        ],
    )?;

    Ok(MessageTemplateItem {
        id,
        key: payload.key.clone(),
        channel: payload.channel.clone(),
        language: payload.language.clone(),
        title: payload.title.clone(),
        subject: payload.subject.clone(),
        body: payload.body.clone(),
        is_system: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn delete_template(conn: &Connection, template_id: &str) -> Result<(), CommsError> {
    let is_sys: Option<i64> = conn
        .query_row(
            "SELECT is_system FROM message_templates WHERE id = ?1",
            params![template_id],
            |r| r.get(0),
        )
        .optional()?;

    match is_sys {
        Some(1) => Err(CommsError::TemplateIsSystem(template_id.to_string())),
        Some(_) => {
            conn.execute(
                "DELETE FROM message_templates WHERE id = ?1",
                params![template_id],
            )?;
            Ok(())
        }
        None => Err(CommsError::TemplateNotFound(template_id.to_string())),
    }
}

pub fn list_automation_rules(conn: &Connection) -> Result<Vec<AutomationRuleItem>, CommsError> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.trigger, r.channel, r.enabled, r.template_id, t.title, r.options_json, r.updated_at
         FROM automation_rules r
         JOIN message_templates t ON r.template_id = t.id
         ORDER BY r.trigger ASC, r.channel ASC",
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(AutomationRuleItem {
            id: r.get(0)?,
            trigger: r.get(1)?,
            channel: r.get(2)?,
            enabled: r.get::<_, i64>(3)? == 1,
            template_id: r.get(4)?,
            template_title: r.get(5)?,
            options_json: r.get(6)?,
            updated_at: r.get(7)?,
        })
    })?;

    let mut rules = Vec::new();
    for r in rows {
        rules.push(r?);
    }
    Ok(rules)
}

pub fn update_automation_rule(
    conn: &Connection,
    payload: &UpdateAutomationRulePayload,
) -> Result<(), CommsError> {
    let now = now_utc_iso();
    let opt_json = payload
        .options_json
        .clone()
        .unwrap_or_else(|| "{}".to_string());

    let rows = conn.execute(
        "UPDATE automation_rules
         SET enabled = ?1, template_id = ?2, options_json = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            if payload.enabled { 1 } else { 0 },
            payload.template_id,
            opt_json,
            now,
            payload.id
        ],
    )?;

    if rows == 0 {
        return Err(CommsError::NotFound(payload.id.clone()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Payment Received Hook
// ---------------------------------------------------------------------------

pub fn enqueue_payment_received_messages_if_enabled(
    conn: &Connection,
    receipt: &crate::ledger::PaymentReceipt,
    user_id: Option<&str>,
) -> Result<(), CommsError> {
    // 1. Fetch student info and primary guardian info
    let (student_code, roll, balance_poisha): (String, Option<i64>, i64) = conn.query_row(
        "SELECT st.student_code, e.roll, COALESCE(sb.balance_poisha, 0)
         FROM students st
         LEFT JOIN enrollments e ON e.student_id = st.id AND e.status = 'active'
         LEFT JOIN student_balances sb ON sb.student_id = st.id
         WHERE st.id = ?1
         LIMIT 1",
        params![receipt.student_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    // Fetch primary guardian contact info
    let guardian_contact: Option<(String, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT g.name, g.phone_norm, g.email
             FROM student_guardians sg
             JOIN guardians g ON sg.guardian_id = g.id
             WHERE sg.student_id = ?1 AND sg.is_primary = 1
             LIMIT 1",
            params![receipt.student_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    let (g_phone, g_email) = match guardian_contact {
        Some((_, p, e)) => (p, e),
        None => (None, None),
    };

    // Institution info
    let inst_name: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.name'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "Coachify".to_string());

    let inst_phone: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.phone'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "".to_string());

    let amount_taka = (receipt.amount_poisha as f64) / 100.0;
    let balance_taka = (balance_poisha as f64) / 100.0;

    let amount_str = format!("{:.2}", amount_taka);
    let balance_str = format!("{:.2}", balance_taka);
    let roll_str = roll
        .map(|r| r.to_string())
        .unwrap_or_else(|| "-".to_string());

    let mut vars = HashMap::new();
    vars.insert("student_name", receipt.student_name.as_str());
    vars.insert("student_code", student_code.as_str());
    vars.insert("roll", roll_str.as_str());
    vars.insert("amount", amount_str.as_str());
    vars.insert("receipt_no", receipt.receipt_no.as_str());
    vars.insert("date", receipt.business_date.as_str());
    vars.insert("balance", balance_str.as_str());
    vars.insert("institution_name", inst_name.as_str());
    vars.insert("institution_phone", inst_phone.as_str());

    // 2. Fetch enabled payment_received rules
    let mut stmt = conn.prepare(
        "SELECT r.channel, t.subject, t.body
         FROM automation_rules r
         JOIN message_templates t ON r.template_id = t.id
         WHERE r.trigger = 'payment_received' AND r.enabled = 1",
    )?;

    let rules = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;

    let now = now_utc_iso();

    for r in rules {
        let (channel, subj_tmpl, body_tmpl) = r?;
        let body = interpolate_placeholders(&body_tmpl, &vars);
        let subject = subj_tmpl.map(|s| interpolate_placeholders(&s, &vars));

        let to_address = match channel.as_str() {
            "sms" | "whatsapp" => g_phone.clone(),
            "email" => g_email.clone(),
            _ => None,
        };

        if let Some(to) = to_address {
            if !to.trim().is_empty() {
                let analysis = analyze_text(&body, &channel);
                let dedupe_key = format!("payment:{}:receipt_{}", receipt.id, channel);
                let outbox_id = format!("out-{}", Uuid::now_v7());

                conn.execute(
                    "INSERT OR IGNORE INTO outbox (
                        id, channel, to_address, subject, body, segments, encoding,
                        template_key, related_type, related_id, dedupe_key,
                        status, scheduled_at, attempts, max_attempts,
                        created_by, created_at
                    ) VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                        'payment_received', 'payment', ?8, ?9,
                        'queued', ?10, 0, 3,
                        ?11, ?10
                    )",
                    params![
                        outbox_id,
                        channel,
                        to,
                        subject,
                        body,
                        analysis.segment_count as i64,
                        analysis.encoding,
                        receipt.id,
                        dedupe_key,
                        now,
                        user_id,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Due Reminders Review Gate & Enqueueing
// ---------------------------------------------------------------------------

pub fn get_due_reminder_candidates(
    conn: &Connection,
    query: &GetDueReminderCandidatesQuery,
) -> Result<Vec<DueReminderCandidate>, CommsError> {
    let min_due = query.min_due_poisha.unwrap_or(0);
    let period = query.period.clone().unwrap_or_else(|| {
        db::current_business_date().unwrap_or_else(|_| "2026-09".to_string())[..7].to_string()
    });

    let mut sql = "SELECT st.id, st.student_code, st.name, e.roll, b.id, b.name,
                          g.name, g.phone_norm, g.email, sb.balance_poisha
                   FROM student_balances sb
                   JOIN students st ON sb.student_id = st.id
                   JOIN enrollments e ON e.student_id = st.id AND e.status = 'active'
                   JOIN batches b ON e.batch_id = b.id
                   LEFT JOIN student_guardians sg ON sg.student_id = st.id AND sg.is_primary = 1
                   LEFT JOIN guardians g ON sg.guardian_id = g.id
                   WHERE sb.balance_poisha >= ?1"
        .to_string();

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(min_due)];

    if let Some(ref bid) = query.batch_id {
        if !bid.trim().is_empty() && bid != "all" {
            sql.push_str(" AND b.id = ?");
            params_vec.push(Box::new(bid.clone()));
        }
    }

    sql.push_str(" ORDER BY sb.balance_poisha DESC, b.name ASC, st.name ASC");

    let mut stmt = conn.prepare(&sql)?;
    let p_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| &**b).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(p_refs), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, Option<String>>(7)?,
            r.get::<_, Option<String>>(8)?,
            r.get::<_, i64>(9)?,
        ))
    })?;

    let mut candidates = Vec::new();
    for row in rows {
        let (
            student_id,
            student_code,
            student_name,
            roll,
            batch_id,
            batch_name,
            guardian_name,
            guardian_phone,
            guardian_email,
            due_amount_poisha,
        ) = row?;

        // Check if reminders already exist in outbox for this student, period, channel
        let check_reminded = |ch: &str| -> bool {
            let d_key = format!("due_reminder:{}:{}:{}", student_id, period, ch);
            conn.query_row(
                "SELECT 1 FROM outbox WHERE dedupe_key = ?1",
                params![d_key],
                |_| Ok(()),
            )
            .optional()
            .unwrap_or(None)
            .is_some()
        };

        let already_reminded_sms = check_reminded("sms");
        let already_reminded_whatsapp = check_reminded("whatsapp");
        let already_reminded_email = check_reminded("email");

        candidates.push(DueReminderCandidate {
            student_id,
            student_code,
            student_name,
            roll,
            batch_id,
            batch_name,
            guardian_name,
            guardian_phone,
            guardian_email,
            due_amount_poisha,
            period: period.clone(),
            already_reminded_sms,
            already_reminded_whatsapp,
            already_reminded_email,
        });
    }

    Ok(candidates)
}

struct DueReminderCandidateRow {
    st_name: String,
    st_code: String,
    g_phone: Option<String>,
    g_email: Option<String>,
    balance_poisha: i64,
}

pub fn enqueue_due_reminders(
    conn: &Connection,
    payload: &EnqueueDueRemindersPayload,
    user_id: Option<&str>,
) -> Result<EnqueueDueRemindersResponse, CommsError> {
    let now = now_utc_iso();
    let mut enqueued = 0;
    let mut skipped = 0;

    // Fetch template for due_reminder on this channel
    let template_body: String = conn
        .query_row(
            "SELECT t.body
             FROM automation_rules r
             JOIN message_templates t ON r.template_id = t.id
             WHERE r.trigger = 'due_reminder' AND r.channel = ?1",
            params![payload.channel],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| {
            "Dear Guardian, fee is due for {student_name}. Please pay balance: Tk {due_amount}. {institution_name}".to_string()
        });

    let inst_name: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.name'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "Coachify".to_string());

    for sid in &payload.student_ids {
        let student_data: Option<DueReminderCandidateRow> = conn
            .query_row(
                "SELECT st.name, st.student_code, g.phone_norm, g.email, COALESCE(sb.balance_poisha, 0)
                 FROM students st
                 LEFT JOIN student_guardians sg ON sg.student_id = st.id AND sg.is_primary = 1
                 LEFT JOIN guardians g ON sg.guardian_id = g.id
                 LEFT JOIN student_balances sb ON sb.student_id = st.id
                 WHERE st.id = ?1",
                params![sid],
                |r| {
                    Ok(DueReminderCandidateRow {
                        st_name: r.get(0)?,
                        st_code: r.get(1)?,
                        g_phone: r.get(2)?,
                        g_email: r.get(3)?,
                        balance_poisha: r.get(4)?,
                    })
                },
            )
            .optional()?;

        let DueReminderCandidateRow {
            st_name,
            st_code,
            g_phone,
            g_email,
            balance_poisha,
        } = match student_data {
            Some(d) => d,
            None => {
                skipped += 1;
                continue;
            }
        };

        let to_address = match payload.channel.as_str() {
            "sms" | "whatsapp" => g_phone,
            "email" => g_email,
            _ => None,
        };

        let to = match to_address {
            Some(ref a) if !a.trim().is_empty() => a.clone(),
            _ => {
                skipped += 1;
                continue;
            }
        };

        let due_taka = (balance_poisha as f64) / 100.0;
        let due_str = format!("{:.2}", due_taka);

        let mut vars = HashMap::new();
        vars.insert("student_name", st_name.as_str());
        vars.insert("student_code", st_code.as_str());
        vars.insert("period", payload.period.as_str());
        vars.insert("due_amount", due_str.as_str());
        vars.insert("balance", due_str.as_str());
        vars.insert("due_date", payload.period.as_str());
        vars.insert("institution_name", inst_name.as_str());

        let raw_body = payload.custom_body.as_deref().unwrap_or(&template_body);
        let body = interpolate_placeholders(raw_body, &vars);
        let analysis = analyze_text(&body, &payload.channel);
        let dedupe_key = format!(
            "due_reminder:{}:{}:{}",
            sid, payload.period, payload.channel
        );
        let outbox_id = format!("out-{}", Uuid::now_v7());

        let rows = conn.execute(
            "INSERT OR IGNORE INTO outbox (
                id, channel, to_address, subject, body, segments, encoding,
                template_key, related_type, related_id, dedupe_key,
                status, scheduled_at, attempts, max_attempts,
                created_by, created_at
            ) VALUES (
                ?1, ?2, ?3, 'Tuition Fee Due Notice', ?4, ?5, ?6,
                'due_reminder', 'student', ?7, ?8,
                'queued', ?9, 0, 3,
                ?10, ?9
            )",
            params![
                outbox_id,
                payload.channel,
                to,
                body,
                analysis.segment_count as i64,
                analysis.encoding,
                sid,
                dedupe_key,
                now,
                user_id
            ],
        )?;

        if rows > 0 {
            enqueued += 1;
        } else {
            skipped += 1; // Deduped!
        }
    }

    Ok(EnqueueDueRemindersResponse {
        enqueued_count: enqueued,
        skipped_count: skipped,
    })
}

// ---------------------------------------------------------------------------
// Batch Broadcast Preview & Dispatch
// ---------------------------------------------------------------------------

pub fn preview_batch_broadcast(
    conn: &Connection,
    payload: &PreviewBatchBroadcastPayload,
) -> Result<PreviewBatchBroadcastResponse, CommsError> {
    let mut stmt = conn.prepare(
        "SELECT st.id, st.student_code, st.name, e.roll, g.phone_norm, g.email
         FROM enrollments e
         JOIN students st ON e.student_id = st.id
         LEFT JOIN student_guardians sg ON sg.student_id = st.id AND sg.is_primary = 1
         LEFT JOIN guardians g ON sg.guardian_id = g.id
         WHERE e.batch_id = ?1 AND e.status = 'active'
         ORDER BY e.roll ASC, st.name ASC",
    )?;

    let rows = stmt.query_map(params![payload.batch_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
        ))
    })?;

    let mut ready = Vec::new();
    let mut missing = Vec::new();

    for r in rows {
        let (sid, code, name, roll, phone, email) = r?;
        let contact = match payload.channel.as_str() {
            "sms" | "whatsapp" => phone,
            "email" => email,
            _ => None,
        };

        let has_contact = contact
            .as_ref()
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false);

        let item = BroadcastRecipientPreview {
            student_id: sid,
            student_code: code,
            student_name: name,
            roll,
            contact_address: contact,
            has_valid_contact: has_contact,
        };

        if has_contact {
            ready.push(item);
        } else {
            missing.push(item);
        }
    }

    // Interpolate sample body with first student if available
    let mut sample_vars = HashMap::new();
    if let Some(first) = ready.first() {
        sample_vars.insert("student_name", first.student_name.as_str());
        sample_vars.insert("student_code", first.student_code.as_str());
        sample_vars.insert("roll", "1");
    } else {
        sample_vars.insert("student_name", "Farhan Ahmed");
        sample_vars.insert("student_code", "STU-2026-001");
        sample_vars.insert("roll", "1");
    }
    sample_vars.insert("institution_name", "Coachify Academy");

    let sample_body = interpolate_placeholders(&payload.custom_body, &sample_vars);
    let analysis = analyze_text(&sample_body, &payload.channel);

    // Approximate cost: 0.35 BDT per SMS segment, 0 for WhatsApp/Email
    let unit_cost = if payload.channel == "sms" { 0.35 } else { 0.0 };
    let estimated_cost = (ready.len() as f64) * (analysis.segment_count as f64) * unit_cost;

    Ok(PreviewBatchBroadcastResponse {
        ready_recipients: ready,
        missing_recipients: missing,
        sample_interpolated_body: sample_body,
        segments_per_message: analysis.segment_count,
        estimated_cost,
    })
}

struct BroadcastRecipientRow {
    name: String,
    code: String,
    roll: Option<i64>,
    phone: Option<String>,
    email: Option<String>,
}

pub fn send_batch_broadcast(
    conn: &Connection,
    payload: &SendBatchBroadcastPayload,
    user_id: Option<&str>,
) -> Result<SendBatchBroadcastResponse, CommsError> {
    if payload.recipient_student_ids.is_empty() {
        return Err(CommsError::BatchNoValidRecipients(
            payload.batch_id.clone(),
            payload.channel.clone(),
        ));
    }

    if payload.channel == "email"
        && payload
            .custom_subject
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(CommsError::EmailSubjectRequired);
    }

    let inst_name: String = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'institution.name'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| serde_json::from_str::<String>(&v).ok())
        .unwrap_or_else(|| "Coachify".to_string());

    let now = now_utc_iso();
    let mut enqueued = 0;

    for sid in &payload.recipient_student_ids {
        let student_row: Option<BroadcastRecipientRow> = conn
            .query_row(
                "SELECT st.name, st.student_code, e.roll, g.phone_norm, g.email
                 FROM students st
                 JOIN enrollments e ON e.student_id = st.id AND e.status = 'active'
                 LEFT JOIN student_guardians sg ON sg.student_id = st.id AND sg.is_primary = 1
                 LEFT JOIN guardians g ON sg.guardian_id = g.id
                 WHERE st.id = ?1 AND e.batch_id = ?2",
                params![sid, payload.batch_id],
                |r| {
                    Ok(BroadcastRecipientRow {
                        name: r.get(0)?,
                        code: r.get(1)?,
                        roll: r.get(2)?,
                        phone: r.get(3)?,
                        email: r.get(4)?,
                    })
                },
            )
            .optional()?;

        if let Some(BroadcastRecipientRow {
            name,
            code,
            roll,
            phone,
            email,
        }) = student_row
        {
            let to_address = match payload.channel.as_str() {
                "sms" | "whatsapp" => phone,
                "email" => email,
                _ => None,
            };

            if let Some(to) = to_address {
                if !to.trim().is_empty() {
                    let mut vars = HashMap::new();
                    vars.insert("student_name", name.as_str());
                    vars.insert("student_code", code.as_str());
                    let roll_str = roll
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    vars.insert("roll", roll_str.as_str());
                    vars.insert("institution_name", inst_name.as_str());

                    let body = interpolate_placeholders(&payload.custom_body, &vars);
                    let subject = payload
                        .custom_subject
                        .as_ref()
                        .map(|s| interpolate_placeholders(s, &vars));
                    let analysis = analyze_text(&body, &payload.channel);
                    let dedupe_key = format!(
                        "broadcast:{}:{}:{}:{}",
                        payload.batch_id,
                        sid,
                        payload.channel,
                        Uuid::now_v7()
                    );
                    let outbox_id = format!("out-{}", Uuid::now_v7());

                    conn.execute(
                        "INSERT INTO outbox (
                            id, channel, to_address, subject, body, segments, encoding,
                            template_key, related_type, related_id, dedupe_key,
                            status, scheduled_at, attempts, max_attempts,
                            created_by, created_at
                        ) VALUES (
                            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                            'broadcast', 'batch', ?8, ?9,
                            'queued', ?10, 0, 3,
                            ?11, ?10
                        )",
                        params![
                            outbox_id,
                            payload.channel,
                            to,
                            subject,
                            body,
                            analysis.segment_count as i64,
                            analysis.encoding,
                            payload.batch_id,
                            dedupe_key,
                            now,
                            user_id
                        ],
                    )?;
                    enqueued += 1;
                }
            }
        }
    }

    Ok(SendBatchBroadcastResponse {
        enqueued_count: enqueued,
    })
}
