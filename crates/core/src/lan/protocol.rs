use crate::dispatcher::{ChangeEvent, RpcError};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::net::IpAddr;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LanError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Port {0} is already in use by another service")]
    PortInUse(u16),
    #[error("Failed to generate TLS certificate: {0}")]
    CertGenerationFailed(String),
    #[error("LAN host listener is not currently running")]
    HostNotRunning,
    #[error("LAN host listener is already running on port {0}")]
    HostAlreadyRunning(u16),
    #[error("Port {0} is invalid (must be between 1024 and 65535)")]
    InvalidPortRange(u16),
    #[error("Failed to bind listener to {0}: {1}")]
    BindFailed(String, String),
    #[error("Discovery scan timed out")]
    DiscoveryTimeout,
    #[error("Host at {0}:{1} is unreachable: {2}")]
    HostUnreachable(String, u16, String),
    #[error("TLS certificate fingerprint mismatch: expected {expected}, received {received}")]
    FingerprintMismatch { expected: String, received: String },
    #[error("License is not entitled for LAN host feature")]
    LicenseNotEntitled,
    #[error("Active workstation count has reached your license seat limit")]
    WorkstationLimitReached,
    #[error("A pairing window is already currently open on the host")]
    PairingAlreadyOpen,
    #[error("No active pairing window is currently open")]
    PairingNotOpen,
    #[error("Pairing window has been closed or already used")]
    PairingClosed,
    #[error("Pairing code has expired")]
    PairingExpired,
    #[error("Pairing window locked due to multiple failed attempts (15-minute cooldown)")]
    PairingLocked,
    #[error("Pairing window cooldown is active after too many failed attempts")]
    PairingCooldownActive,
    #[error("Invalid 6-digit pairing code")]
    PairingCodeInvalid,
    #[error("Workstation device not found")]
    DeviceNotFound,
    #[error("Workstation device name must be between 1 and 60 characters")]
    InvalidDeviceName,
    #[error("Primary host computer cannot be blocked")]
    CannotBlockHost,
    #[error("Primary host computer cannot be revoked")]
    CannotRevokeHost,
    #[error("Primary host computer cannot be renamed via workstation management")]
    CannotRenameHost,
    #[error("Device is already blocked")]
    DeviceAlreadyBlocked,
    #[error("Device is not currently blocked")]
    DeviceNotBlocked,
    #[error("Device has already been permanently revoked")]
    DeviceAlreadyRevoked,
    #[error("Workstation device has been revoked by host administrator")]
    DeviceRevoked,
    #[error("Workstation device has been suspended by host administrator")]
    DeviceBlocked,
    #[error("Windows DPAPI error: {0}")]
    Dpapi(String),
    #[error("Invalid QR pairing secret")]
    PairingSecretInvalid,
    #[error("Pairing session has already been used")]
    PairingSessionAlreadyUsed,
    #[error("Invalid 4 to 6 digit staff PIN")]
    InvalidStaffPin,
    #[error("Staff member not found or inactive")]
    StaffNotFound,
    #[error("Mobile session is locked; staff PIN entry required")]
    SessionLockedPinRequired,
    #[error("Invalid QR pairing URI: {0}")]
    InvalidQrUri(String),
    #[error("Host printer unavailable: {0}")]
    PrinterUnavailable(String),
    #[error("Workstation is not paired with a host")]
    NotPaired,
    #[error("Workstation device has been permanently revoked by host administrator")]
    WorkstationRevoked,
    #[error("Workstation device has been suspended by host administrator")]
    WorkstationBlocked,
    #[error("LAN update mirror is currently disabled on the host")]
    UpdateMirrorDisabled,
    #[error("No active update is currently mirrored on the host")]
    NoUpdateMirrored,
    #[error("Staged update package not found: {0}")]
    UpdateFileNotFound(String),
    #[error("Update installer checksum mismatch: expected {expected}, computed {computed}")]
    UpdateChecksumMismatch { expected: String, computed: String },
    #[error("Update cryptographic signature is invalid: {0}")]
    UpdateSignatureInvalid(String),
    #[error("Update mirror operation failed: {0}")]
    UpdateMirrorError(String),
}

impl LanError {
    pub fn code(&self) -> &'static str {
        match self {
            LanError::Database(_) => "LAN_DATABASE_ERROR",
            LanError::PortInUse(_) => "LAN_PORT_IN_USE",
            LanError::CertGenerationFailed(_) => "LAN_CERT_GENERATION_FAILED",
            LanError::HostNotRunning => "LAN_HOST_NOT_RUNNING",
            LanError::HostAlreadyRunning(_) => "LAN_HOST_ALREADY_RUNNING",
            LanError::InvalidPortRange(_) => "INVALID_PORT_RANGE",
            LanError::BindFailed(_, _) => "LAN_BIND_FAILED",
            LanError::DiscoveryTimeout => "LAN_DISCOVERY_TIMEOUT",
            LanError::HostUnreachable(_, _, _) => "HOST_UNREACHABLE",
            LanError::FingerprintMismatch { .. } => "FINGERPRINT_MISMATCH",
            LanError::LicenseNotEntitled => "LICENSE_FEATURE_NOT_ENTITLED",
            LanError::WorkstationLimitReached => "LAN_WORKSTATION_LIMIT_REACHED",
            LanError::PairingAlreadyOpen => "LAN_PAIRING_ALREADY_OPEN",
            LanError::PairingNotOpen => "LAN_PAIRING_NOT_OPEN",
            LanError::PairingClosed => "LAN_PAIRING_CLOSED",
            LanError::PairingExpired => "LAN_PAIRING_EXPIRED",
            LanError::PairingLocked => "LAN_PAIRING_LOCKED",
            LanError::PairingCooldownActive => "LAN_PAIRING_COOLDOWN_ACTIVE",
            LanError::PairingCodeInvalid => "LAN_PAIRING_CODE_INVALID",
            LanError::DeviceNotFound => "DEVICE_NOT_FOUND",
            LanError::InvalidDeviceName => "INVALID_DEVICE_NAME",
            LanError::CannotBlockHost => "CANNOT_BLOCK_HOST",
            LanError::CannotRevokeHost => "CANNOT_REVOKE_HOST",
            LanError::CannotRenameHost => "CANNOT_RENAME_HOST",
            LanError::DeviceAlreadyBlocked => "DEVICE_ALREADY_BLOCKED",
            LanError::DeviceNotBlocked => "DEVICE_NOT_BLOCKED",
            LanError::DeviceAlreadyRevoked => "DEVICE_ALREADY_REVOKED",
            LanError::DeviceRevoked => "DEVICE_REVOKED",
            LanError::DeviceBlocked => "DEVICE_BLOCKED",
            LanError::Dpapi(_) => "DPAPI_ERROR",
            LanError::PairingSecretInvalid => "PAIRING_SECRET_INVALID",
            LanError::PairingSessionAlreadyUsed => "PAIRING_SESSION_ALREADY_USED",
            LanError::InvalidStaffPin => "INVALID_STAFF_PIN",
            LanError::StaffNotFound => "STAFF_NOT_FOUND",
            LanError::SessionLockedPinRequired => "SESSION_LOCKED_PIN_REQUIRED",
            LanError::InvalidQrUri(_) => "INVALID_QR_URI",
            LanError::PrinterUnavailable(_) => "PRINTER_UNAVAILABLE",
            LanError::NotPaired => "NOT_PAIRED",
            LanError::WorkstationRevoked => "DEVICE_REVOKED",
            LanError::WorkstationBlocked => "DEVICE_BLOCKED",
            LanError::UpdateMirrorDisabled => "LAN_UPDATE_MIRROR_DISABLED",
            LanError::NoUpdateMirrored => "LAN_NO_UPDATE_MIRRORED",
            LanError::UpdateFileNotFound(_) => "LAN_UPDATE_FILE_NOT_FOUND",
            LanError::UpdateChecksumMismatch { .. } => "LAN_UPDATE_CHECKSUM_MISMATCH",
            LanError::UpdateSignatureInvalid(_) => "LAN_UPDATE_SIGNATURE_INVALID",
            LanError::UpdateMirrorError(_) => "LAN_UPDATE_MIRROR_ERROR",
        }
    }
}

impl From<rusqlite::Error> for LanError {
    fn from(err: rusqlite::Error) -> Self {
        LanError::Database(err.to_string())
    }
}

impl From<crate::entitlement::EntitlementError> for LanError {
    fn from(err: crate::entitlement::EntitlementError) -> Self {
        match err {
            crate::entitlement::EntitlementError::WorkstationLimitReached => {
                LanError::WorkstationLimitReached
            }
            crate::entitlement::EntitlementError::FeatureNotEntitled { .. } => {
                LanError::LicenseNotEntitled
            }
            crate::entitlement::EntitlementError::Db(e) => LanError::Database(e),
            other => LanError::Database(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LanHostConfig {
    pub id: String,
    pub is_enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub tls_cert_pem: String,
    pub tls_key_pem: String,
    pub tls_fingerprint_sha256: String,
    pub mdns_enabled: bool,
    pub auto_start_on_boot: bool,
    pub prevent_sleep: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LanHostStatus {
    pub is_running: bool,
    pub port: u16,
    pub bind_address: String,
    pub bound_addresses: Vec<String>,
    pub fingerprint: String,
    pub active_workstations: u32,
    pub network_profile: String,
    pub firewall_status: String,
    pub mdns_broadcasting: bool,
    pub license_entitled: bool,
    pub prevent_sleep: bool,
    pub auto_start_on_boot: bool,
    pub uptime_seconds: u64,
    pub host_device_id: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HelloResponse {
    pub product: String,
    pub protocol_version: u32,
    pub app_version: String,
    pub host_id: String,
    pub institution_name: String,
    pub pairing_open: bool,
    pub tls_fingerprint_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PairingWindowInfo {
    pub session_id: String,
    pub code: String,
    pub expires_at: String,
    pub expires_in_seconds: u64,
    pub host_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PairRequest {
    pub pairing_code: String,
    pub workstation_name: String,
    pub windows_name: String,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub client_nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PairSuccessResponse {
    pub device_id: String,
    pub device_token: String,
    pub institution_name: String,
    pub host_device_id: String,
    pub host_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationSummary {
    pub id: String,
    pub name: String,
    pub windows_name: String,
    pub ip_address: Option<String>,
    pub os_version: Option<String>,
    pub app_version: String,
    pub status: String,
    pub is_host: bool,
    pub paired_at: String,
    pub last_seen_at: String,
    pub blocked_at: Option<String>,
    pub revoked_at: Option<String>,
    pub revocation_reason: Option<String>,
    pub is_online: bool,
    pub active_sessions_count: u32,
    pub paired_via: String,
    pub platform: Option<String>,
    pub device_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MobilePairingWindowInfo {
    pub session_id: String,
    pub qr_secret: String,
    pub qr_uri: String,
    pub code: String,
    pub expires_at: String,
    pub expires_in_seconds: u64,
    pub host_fingerprint: String,
    pub host_ips: Vec<String>,
    pub port: u16,
    pub institution_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PairQrRequest {
    pub qr_secret: String,
    pub device_name: String,
    pub platform: String,
    pub device_type: Option<String>,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub client_nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PairQrSuccessResponse {
    pub device_id: String,
    pub device_token: String,
    pub institution_name: String,
    pub host_id: String,
    pub host_name: String,
    pub active_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffLoginRequest {
    pub staff_pin: String,
    pub device_token: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StaffLoginResponse {
    pub session_token: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub staff_profile_id: Option<String>,
    #[serde(default)]
    pub staff_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HostReceiptPrintRequest {
    pub payment_id: String,
    pub device_token: Option<String>,
    pub receipt_template: Option<String>,
    pub copies: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HostReceiptPrintResponse {
    pub print_job_id: String,
    pub status: String,
    pub printer_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ParsedPairingQrUri {
    pub version: u32,
    pub host_ips: Vec<String>,
    pub port: u16,
    pub fingerprint: String,
    pub secret: String,
    pub institution_name: String,
    pub expires_at_timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateHostReceiptPrinterPayload {
    pub printer_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetStaffPinPayload {
    pub user_id: String,
    pub staff_pin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MobileClientState {
    pub device_id: String,
    pub device_token: String,
    pub host_id: String,
    pub host_fingerprint: String,
    pub host_ip: String,
    pub host_port: u16,
    pub session_token: Option<String>,
    #[serde(default)]
    pub candidate_ips: Vec<String>,
}

pub use crate::entitlement::WorkstationQuota;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationListResponse {
    pub devices: Vec<WorkstationSummary>,
    pub quota: WorkstationQuota,
    pub pairing_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DeviceEventItem {
    pub id: String,
    pub device_id: String,
    pub event_type: String,
    pub actor_user_id: Option<String>,
    pub actor_user_name: Option<String>,
    pub remote_ip: Option<String>,
    pub details_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationDetail {
    pub device: WorkstationSummary,
    pub recent_events: Vec<DeviceEventItem>,
    pub active_sessions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OpenPairingWindowPayload {
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RenameWorkstationPayload {
    pub device_id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BlockWorkstationPayload {
    pub device_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UnblockWorkstationPayload {
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RevokeWorkstationPayload {
    pub device_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetWorkstationPayload {
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListDeviceEventsPayload {
    pub device_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClientPairPayload {
    pub host_ip: String,
    pub host_port: u16,
    pub pairing_code: String,
    pub workstation_name: String,
    pub expected_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationCredentials {
    pub device_id: String,
    pub device_token: String,
    pub host_id: String,
    pub host_fingerprint: String,
    pub host_ip: String,
    pub host_port: u16,
    pub paired_at: String,
    #[serde(default)]
    pub host_name: Option<String>,
    #[serde(default)]
    pub workstation_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkstationConnectionState {
    Connected,
    Connecting,
    Reconnecting,
    #[default]
    Offline,
    Blocked,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationClientStatus {
    #[serde(default)]
    pub is_client_mode: bool,
    #[serde(default)]
    pub state: WorkstationConnectionState,
    pub mode: String,
    pub connection_state: WorkstationConnectionState,
    pub host_id: Option<String>,
    pub host_ip: Option<String>,
    pub host_port: Option<u16>,
    pub host_name: Option<String>,
    pub device_id: Option<String>,
    pub workstation_name: Option<String>,
    pub latency_ms: Option<u64>,
    pub rtt_ms: Option<u64>,
    pub last_synced_at: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub paired_at: Option<String>,
    pub is_reconnecting: bool,
    pub reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ReconnectWorkstationResponse {
    pub success: bool,
    pub connection_state: WorkstationConnectionState,
    pub latency_ms: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub host_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClientInfoResponse {
    pub hostname: String,
    pub os: String,
    pub app_version: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedRpcRequest {
    pub command: String,
    pub payload: serde_json::Value,
    pub session_token: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedRpcResponse {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub events: Vec<ChangeEvent>,
    pub server_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HealthResponse {
    pub status: String,
    pub host_id: String,
    pub protocol_version: u32,
    pub database_status: String,
    pub uptime_seconds: u64,
    pub active_workstations_count: u32,
    pub bound_port: u16,
    pub server_time_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FirewallGuidance {
    pub profile: String,
    pub is_private: bool,
    pub rule_exists: bool,
    pub powershell_command: String,
    pub netsh_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiscoveredHost {
    pub host_id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub fingerprint: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiscoveredHostList {
    pub hosts: Vec<DiscoveredHost>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HostPingResult {
    pub reachable: bool,
    pub product: Option<String>,
    pub protocol_version: Option<u32>,
    pub host_id: Option<String>,
    pub institution_name: Option<String>,
    pub fingerprint_matched: bool,
    pub rtt_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StartHostPayload {
    pub port: Option<u16>,
    pub bind_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateHostConfigPayload {
    pub port: Option<u16>,
    pub bind_address: Option<String>,
    pub auto_start_on_boot: Option<bool>,
    pub prevent_sleep: Option<bool>,
    pub mdns_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DiscoverHostsPayload {
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PingHostPayload {
    pub ip: String,
    pub port: u16,
    pub expected_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceControlEvent {
    Revoked { device_id: String, reason: String },
    Blocked { device_id: String, reason: String },
    Change { events: Vec<ChangeEvent> },
}

// -----------------------------------------------------------------------------
// URL and String Helper Functions
// -----------------------------------------------------------------------------

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

pub fn url_decode(input: &str) -> String {
    let mut decoded = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(val) =
                    u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""), 16)
                {
                    decoded.push(val);
                    i += 3;
                } else {
                    decoded.push(b'%');
                    i += 1;
                }
            }
            b => {
                decoded.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).to_string()
}

pub fn get_local_ip_addresses() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in ifaces {
            if !ip.is_loopback() {
                let s = ip.to_string();
                if !ips.contains(&s) {
                    ips.push(s);
                }
            }
        }
    }
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    ips
}

pub fn get_active_ipv4_addresses() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (_name, ip) in interfaces {
            if let IpAddr::V4(ipv4) = ip {
                if !ipv4.is_loopback() && !ipv4.is_link_local() {
                    let s = ipv4.to_string();
                    if !ips.contains(&s) {
                        ips.push(s);
                    }
                }
            }
        }
    }
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    ips
}

pub fn get_system_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

pub fn format_qr_pairing_uri(
    ips: &[String],
    port: u16,
    fingerprint: &str,
    secret: &str,
    institution_name: &str,
    expires_at_epoch: i64,
) -> String {
    let clean_fp = fingerprint.replace(':', "").to_lowercase();
    let ips_str = ips.join(",");
    let encoded_inst = url_encode(institution_name);
    format!(
        "coachify://pair?v=1&host={}&port={}&fp={}&sec={}&inst={}&exp={}",
        ips_str, port, clean_fp, secret, encoded_inst, expires_at_epoch
    )
}

pub fn parse_qr_pairing_uri(uri: &str) -> Result<ParsedPairingQrUri, LanError> {
    let trimmed = uri.trim();
    if !trimmed.starts_with("coachify://pair") {
        return Err(LanError::InvalidQrUri(
            "URI scheme must start with coachify://pair".to_string(),
        ));
    }

    let query_str = match trimmed.find('?') {
        Some(idx) => &trimmed[idx + 1..],
        None => {
            return Err(LanError::InvalidQrUri(
                "Missing query parameters in pairing URI".to_string(),
            ))
        }
    };

    let mut version = 1u32;
    let mut host_ips = Vec::new();
    let mut port = 50443u16;
    let mut fingerprint = String::new();
    let mut secret = String::new();
    let mut institution_name = String::new();
    let mut expires_at_timestamp = None;

    for part in query_str.split('&') {
        let mut key_val = part.splitn(2, '=');
        let key = key_val.next().unwrap_or("").trim();
        let val = key_val.next().unwrap_or("").trim();

        match key {
            "v" => {
                if let Ok(v) = val.parse::<u32>() {
                    version = v;
                }
            }
            "host" => {
                for ip in val.split(',') {
                    let cleaned = ip.trim();
                    if !cleaned.is_empty() {
                        host_ips.push(cleaned.to_string());
                    }
                }
            }
            "port" => {
                if let Ok(p) = val.parse::<u16>() {
                    port = p;
                }
            }
            "fp" | "fingerprint" => {
                fingerprint = val.replace(':', "").to_lowercase();
            }
            "sec" | "secret" => {
                secret = val.to_string();
            }
            "inst" => {
                institution_name = url_decode(val);
            }
            "exp" => {
                if let Ok(exp) = val.parse::<i64>() {
                    expires_at_timestamp = Some(exp);
                }
            }
            _ => {}
        }
    }

    if host_ips.is_empty() {
        return Err(LanError::InvalidQrUri(
            "Missing host IP addresses".to_string(),
        ));
    }
    if fingerprint.is_empty() {
        return Err(LanError::InvalidQrUri(
            "Missing host TLS certificate fingerprint".to_string(),
        ));
    }
    if secret.is_empty() {
        return Err(LanError::InvalidQrUri("Missing pairing secret".to_string()));
    }

    Ok(ParsedPairingQrUri {
        version,
        host_ips,
        port,
        fingerprint,
        secret,
        institution_name,
        expires_at_timestamp,
    })
}

// -----------------------------------------------------------------------------
// Cryptographic Pairing & Token Generation
// -----------------------------------------------------------------------------

pub fn generate_pairing_code() -> (String, String, String) {
    let code: u32 = rand::thread_rng().gen_range(100_000..=999_999);
    let code_str = format!("{:06}", code);
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt_hex = hex::encode(salt_bytes);
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", code_str, salt_hex).as_bytes());
    let code_hash = hex::encode(hasher.finalize());
    (code_str, salt_hex, code_hash)
}

pub fn verify_pairing_code(input_code: &str, salt: &str, expected_hash: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", input_code.trim(), salt).as_bytes());
    let actual_hash = hex::encode(hasher.finalize());
    let a = actual_hash.as_bytes();
    let b = expected_hash.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn generate_device_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = format!("dt_{}", hex::encode(bytes));
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());
    (token, token_hash)
}

pub fn generate_qr_secret() -> (String, String) {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    let secret = format!("qs_{}", hex::encode(bytes));
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let hash = hex::encode(hasher.finalize());
    (secret, hash)
}

pub fn is_mutating_command(cmd: &str) -> bool {
    if let Some(meta) = crate::dispatcher::find_command_meta(cmd) {
        meta.kind == crate::dispatcher::CommandKind::Mutation
    } else {
        cmd.contains("create")
            || cmd.contains("update")
            || cmd.contains("delete")
            || cmd.contains("save")
            || cmd.contains("record")
            || cmd.contains("void")
            || cmd.contains("refund")
            || cmd.contains("mark")
            || cmd.contains("admit")
            || cmd.contains("set_")
            || cmd.contains("close")
            || cmd.contains("post")
    }
}

// -----------------------------------------------------------------------------
// LAN Host Update Mirror DTOs (Feature 72, Spec 0048)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct LanUpdateMirrorConfig {
    pub id: String,
    pub mirror_enabled: bool,
    pub storage_path: Option<String>,
    pub max_cached_releases: i64,
    pub auto_stage_on_host_update: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct LanStagedUpdate {
    pub id: String,
    pub version: String,
    pub channel: String,
    pub artifact_filename: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub updater_signature: String,
    pub updater_key_id: String,
    pub notes: String,
    pub min_upgrade_from: Option<String>,
    pub staged_at: String,
    pub downloads_count: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct LanUpdateMirrorStatus {
    pub mirror_enabled: bool,
    pub storage_dir: String,
    pub active_update: Option<LanStagedUpdate>,
    pub total_cached_releases: i64,
    pub total_downloads_served: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct HostUpdateManifest {
    pub version: String,
    pub channel: String,
    pub artifact_filename: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub updater_signature: String,
    pub updater_key_id: String,
    pub notes: String,
    pub min_upgrade_from: Option<String>,
    pub download_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StageMirrorUpdateInput {
    pub file_path: String,
    pub version: String,
    pub channel: Option<String>,
    pub notes: Option<String>,
    pub expected_sha256: Option<String>,
    pub updater_signature: Option<String>,
    pub updater_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ConfigureMirrorInput {
    pub mirror_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ClearMirrorUpdateInput {
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct WorkstationUpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub host_version: Option<String>,
    pub manifest: Option<HostUpdateManifest>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct WorkstationUpdateDownloadResult {
    pub version: String,
    pub target_path: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct DownloadWorkstationUpdatePayload {
    pub version: Option<String>,
}
