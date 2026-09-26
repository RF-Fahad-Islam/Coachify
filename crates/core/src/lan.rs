use crate::accounts;
use crate::db;
use argon2::PasswordVerifier;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use rand::{Rng, RngCore};
use rcgen::{CertificateParams, KeyPair};
use rusqlite::{params, Connection, OptionalExtension};
use rustls::client::danger::ServerCertVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::DigitallySignedStruct;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use specta::Type;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

static INIT_CRYPTO: Once = Once::new();

pub fn ensure_crypto_provider_installed() {
    INIT_CRYPTO.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

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
            LanError::FingerprintMismatch { .. } => "TLS_FINGERPRINT_MISMATCH",
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
        }
    }
}

impl From<rusqlite::Error> for LanError {
    fn from(err: rusqlite::Error) -> Self {
        LanError::Database(err.to_string())
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
    pub staff_id: String,
    pub role: String,
    pub full_name: String,
    pub permissions: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationQuota {
    pub active_workstations: u32,
    pub workstation_limit: Option<u32>,
    pub seats_remaining: Option<u32>,
    pub license_tier: String,
}

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

// -----------------------------------------------------------------------------
// Rate Limiter for /hello Endpoint (Sliding window: max 30 req/min per IP)
// -----------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct RateLimiter {
    history: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let mut map = self.history.lock().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(60);

        let timestamps = map.entry(ip).or_default();
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= 30 {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

// -----------------------------------------------------------------------------
// Win32 Sleep Inhibition
// -----------------------------------------------------------------------------

pub fn set_sleep_inhibition(inhibit: bool) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Power::{
            SetThreadExecutionState, ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_SYSTEM_REQUIRED,
        };
        unsafe {
            if inhibit {
                SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED);
            } else {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = inhibit;
    }
}

// -----------------------------------------------------------------------------
// Dynamic SAN & Host IP Resolution
// -----------------------------------------------------------------------------

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

pub fn get_system_hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

// -----------------------------------------------------------------------------
// Certificate & Key Generation (AC-1)
// -----------------------------------------------------------------------------

pub fn compute_sha256_fingerprint(der_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(der_bytes);
    let hash = hasher.finalize();
    let hex_pairs: Vec<String> = hash.iter().map(|b| format!("{:02X}", b)).collect();
    format!("SHA256:{}", hex_pairs.join(":"))
}

pub fn generate_self_signed_cert() -> Result<(String, String, String), LanError> {
    ensure_crypto_provider_installed();

    let mut sans = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let hostname = get_system_hostname();
    if !sans.contains(&hostname) {
        sans.push(hostname.clone());
    }

    for ip in get_local_ip_addresses() {
        if !sans.contains(&ip) {
            sans.push(ip);
        }
    }

    let mut params =
        CertificateParams::new(sans).map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        format!("Coachify - {}", hostname),
    );

    let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
        .map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| LanError::CertGenerationFailed(e.to_string()))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let fingerprint = compute_sha256_fingerprint(cert.der());

    Ok((cert_pem, key_pem, fingerprint))
}

pub fn get_or_create_host_config(
    conn: &Connection,
    current_utc: &str,
) -> Result<LanHostConfig, LanError> {
    let mut stmt = conn.prepare(
        "SELECT id, is_enabled, port, bind_address, tls_cert_pem, tls_key_pem,
                tls_fingerprint_sha256, mdns_enabled, auto_start_on_boot, prevent_sleep,
                created_at, updated_at
         FROM lan_host_config WHERE id = 'default'",
    )?;

    let maybe_config = stmt
        .query_row([], |row| {
            Ok(LanHostConfig {
                id: row.get(0)?,
                is_enabled: row.get::<_, i32>(1)? != 0,
                port: row.get::<_, u16>(2)?,
                bind_address: row.get(3)?,
                tls_cert_pem: row.get(4)?,
                tls_key_pem: row.get(5)?,
                tls_fingerprint_sha256: row.get(6)?,
                mdns_enabled: row.get::<_, i32>(7)? != 0,
                auto_start_on_boot: row.get::<_, i32>(8)? != 0,
                prevent_sleep: row.get::<_, i32>(9)? != 0,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .optional()?;

    if let Some(config) = maybe_config {
        return Ok(config);
    }

    let (cert_pem, key_pem, fingerprint) = generate_self_signed_cert()?;

    conn.execute(
        "INSERT INTO lan_host_config (
            id, is_enabled, port, bind_address, tls_cert_pem, tls_key_pem,
            tls_fingerprint_sha256, mdns_enabled, auto_start_on_boot, prevent_sleep,
            created_at, updated_at
        ) VALUES (
            'default', 0, 50443, '0.0.0.0', ?1, ?2, ?3, 1, 0, 1, ?4, ?4
        )",
        params![cert_pem, key_pem, fingerprint, current_utc],
    )?;

    Ok(LanHostConfig {
        id: "default".to_string(),
        is_enabled: false,
        port: 50443,
        bind_address: "0.0.0.0".to_string(),
        tls_cert_pem: cert_pem,
        tls_key_pem: key_pem,
        tls_fingerprint_sha256: fingerprint,
        mdns_enabled: true,
        auto_start_on_boot: false,
        prevent_sleep: true,
        created_at: current_utc.to_string(),
        updated_at: current_utc.to_string(),
    })
}

// -----------------------------------------------------------------------------
// Real-Time SSE Device Control & Event Broadcast (AC-8, AC-9)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceControlEvent {
    Revoked { device_id: String, reason: String },
    Blocked { device_id: String, reason: String },
}

static CONTROL_BROADCAST: OnceLock<broadcast::Sender<DeviceControlEvent>> = OnceLock::new();

pub fn control_sender() -> &'static broadcast::Sender<DeviceControlEvent> {
    CONTROL_BROADCAST.get_or_init(|| {
        let (tx, _rx) = broadcast::channel(128);
        tx
    })
}

pub fn broadcast_control_event(event: DeviceControlEvent) {
    let _ = control_sender().send(event);
}

// -----------------------------------------------------------------------------
// Database Accessor for LAN Server Context & Testing
// -----------------------------------------------------------------------------

static TEST_DB_CONN: Mutex<Option<Arc<Mutex<Connection>>>> = Mutex::new(None);

pub fn set_test_db_conn(conn: Option<Arc<Mutex<Connection>>>) {
    let mut lock = TEST_DB_CONN.lock().unwrap();
    *lock = conn;
}

pub fn with_db<T, F: FnOnce(&mut Connection) -> Result<T, LanError>>(
    ctx: &LanServerContext,
    f: F,
) -> Result<T, LanError> {
    if let Some(ref test_conn) = *TEST_DB_CONN.lock().unwrap() {
        let mut conn = test_conn.lock().unwrap();
        return f(&mut conn);
    }
    if let Some(ref path) = ctx.db_path {
        let mut conn = Connection::open(path).map_err(|e| LanError::Database(e.to_string()))?;
        return f(&mut conn);
    }
    if let Ok(path) = db::resolve_active_db_path() {
        if path.exists() {
            let mut conn = Connection::open(path).map_err(|e| LanError::Database(e.to_string()))?;
            return f(&mut conn);
        }
    }
    Err(LanError::Database(
        "No database connection available for LAN server".to_string(),
    ))
}

// -----------------------------------------------------------------------------
// Cryptographic Pairing & Token Generation (AC-1, AC-2, AC-3)
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

// -----------------------------------------------------------------------------
// Global Server State & Manager
// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct LanServerContext {
    pub host_id: String,
    pub institution_name: String,
    pub app_version: String,
    pub fingerprint: String,
    pub db_path: Option<PathBuf>,
    pub started_at: Instant,
    pub port: u16,
    pub rate_limiter: RateLimiter,
}

pub struct ActiveLanServer {
    pub port: u16,
    pub bind_address: String,
    pub handle: axum_server::Handle<SocketAddr>,
    pub started_at: Instant,
    pub mdns_daemon: Option<ServiceDaemon>,
    pub mdns_fullname: Option<String>,
}

impl ActiveLanServer {
    pub fn update_mdns_pairing(
        &mut self,
        institution_name: &str,
        host_windows_name: &str,
        host_id: &str,
        fingerprint: &str,
        pairing: bool,
    ) {
        if let Some(ref mdns) = self.mdns_daemon {
            let service_type = "_coachify._tcp.local.";
            let instance_name = format!("Coachify - {} ({})", institution_name, host_windows_name);
            let host_name = format!("{}.local.", host_windows_name.replace(' ', "-"));

            let mut properties = HashMap::new();
            properties.insert("host_id".to_string(), host_id.to_string());
            properties.insert("proto".to_string(), "1".to_string());
            properties.insert("fingerprint".to_string(), fingerprint.to_string());
            properties.insert("inst".to_string(), institution_name.to_string());
            properties.insert("ver".to_string(), "0.1.0".to_string());
            properties.insert(
                "pairing".to_string(),
                if pairing {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            );

            let primary_ip = get_local_ip_addresses()
                .first()
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());

            if let Ok(service_info) = ServiceInfo::new(
                service_type,
                &instance_name,
                &host_name,
                primary_ip.as_str(),
                self.port,
                properties,
            ) {
                let _ = mdns.register(service_info);
            }
        }
    }
}

pub struct LanHostManager {
    active_server: Mutex<Option<ActiveLanServer>>,
    tokio_runtime: tokio::runtime::Runtime,
}

static LAN_MANAGER: OnceLock<LanHostManager> = OnceLock::new();

pub fn lan_manager() -> &'static LanHostManager {
    LAN_MANAGER.get_or_init(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("coachify-lan-worker")
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime for LAN host");
        LanHostManager {
            active_server: Mutex::new(None),
            tokio_runtime: rt,
        }
    })
}

impl LanHostManager {
    pub fn is_running(&self) -> bool {
        self.active_server.lock().unwrap().is_some()
    }

    pub fn running_port(&self) -> Option<u16> {
        self.active_server.lock().unwrap().as_ref().map(|s| s.port)
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.active_server
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.started_at.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn update_mdns_pairing_status(&self, conn: &Connection, current_utc: &str, pairing: bool) {
        let mut lock = self.active_server.lock().unwrap();
        if let Some(ref mut server) = *lock {
            let host_dev = accounts::get_or_create_host_device(conn, current_utc).ok();
            let inst: String = conn
                .query_row(
                    "SELECT value FROM system_config WHERE key = 'institution_name'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "Coachify Coaching Center".to_string());
            if let Some(dev) = host_dev {
                if let Ok(config) = get_or_create_host_config(conn, current_utc) {
                    server.update_mdns_pairing(
                        &inst,
                        &dev.windows_name,
                        &dev.id,
                        &config.tls_fingerprint_sha256,
                        pairing,
                    );
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Pairing Window Management (AC-1)
// -----------------------------------------------------------------------------

pub fn open_pairing_window(
    conn: &Connection,
    duration_seconds: Option<u64>,
    actor_user_id: &str,
    current_utc: &str,
) -> Result<PairingWindowInfo, LanError> {
    let mgr = lan_manager();
    if !mgr.is_running() {
        return Err(LanError::HostNotRunning);
    }

    // Check if lockout cooldown is active
    let locked_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
        params![current_utc],
        |r| r.get(0),
    )?;
    if locked_count > 0 {
        return Err(LanError::PairingCooldownActive);
    }

    // Check if already open
    let pending_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'pending' AND expires_at > ?1",
        params![current_utc],
        |r| r.get(0),
    )?;
    if pending_count > 0 {
        return Err(LanError::PairingAlreadyOpen);
    }

    // Check workstation license limit
    let active_workstations: u32 = conn.query_row(
        "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
        [],
        |r| r.get(0),
    )?;
    let license = crate::licensing::get_license_status(conn, current_utc).ok();
    let limit = license
        .as_ref()
        .and_then(|l| l.workstation_limit)
        .unwrap_or(1);
    if active_workstations >= limit {
        return Err(LanError::WorkstationLimitReached);
    }

    // Mark any stale pending sessions as expired
    conn.execute(
        "UPDATE lan_pairing_sessions SET status = 'expired', updated_at = ?1 WHERE status = 'pending' AND expires_at <= ?1",
        params![current_utc],
    )?;

    let dur = duration_seconds.unwrap_or(600).clamp(60, 3600);
    let (code, salt, code_hash) = generate_pairing_code();
    let session_id = uuid::Uuid::now_v7().to_string();

    let now_ts = jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());
    let expires_ts = now_ts
        .checked_add(jiff::Span::new().seconds(dur as i64))
        .map(|t| t.to_string())
        .unwrap_or_else(|_| current_utc.to_string());

    conn.execute(
        "INSERT INTO lan_pairing_sessions (
            id, code_hash, salt, expires_at, failed_attempts, status,
            created_by_user_id, paired_device_id, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, 0, 'pending',
            ?5, NULL, ?6, ?6
        )",
        params![
            session_id,
            code_hash,
            salt,
            expires_ts,
            actor_user_id,
            current_utc
        ],
    )?;

    let config = get_or_create_host_config(conn, current_utc)?;
    mgr.update_mdns_pairing_status(conn, current_utc, true);

    Ok(PairingWindowInfo {
        session_id,
        code,
        expires_at: expires_ts,
        expires_in_seconds: dur,
        host_fingerprint: config.tls_fingerprint_sha256,
    })
}

pub fn close_pairing_window(conn: &Connection, current_utc: &str) -> Result<(), LanError> {
    let rows_affected = conn.execute(
        "UPDATE lan_pairing_sessions SET status = 'cancelled', updated_at = ?1 WHERE status = 'pending'",
        params![current_utc],
    )?;

    if rows_affected == 0 {
        return Err(LanError::PairingNotOpen);
    }

    let mgr = lan_manager();
    mgr.update_mdns_pairing_status(conn, current_utc, false);

    Ok(())
}

// -----------------------------------------------------------------------------
// Mobile App & QR Pairing Functions (Spec 0046, Feature 70b)
// -----------------------------------------------------------------------------

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

pub fn open_mobile_pairing_window(
    conn: &Connection,
    duration_seconds: Option<u64>,
    actor_user_id: &str,
    current_utc: &str,
) -> Result<MobilePairingWindowInfo, LanError> {
    let mgr = lan_manager();
    if !mgr.is_running() {
        return Err(LanError::HostNotRunning);
    }

    // Check if lockout cooldown is active
    let locked_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
        params![current_utc],
        |r| r.get(0),
    )?;
    if locked_count > 0 {
        return Err(LanError::PairingCooldownActive);
    }

    // Mark any stale pending sessions as expired and cancel any active pending sessions so a fresh pairing window opens cleanly
    conn.execute(
        "UPDATE lan_pairing_sessions SET status = 'cancelled', updated_at = ?1 WHERE status = 'pending'",
        params![current_utc],
    )?;

    let dur = duration_seconds.unwrap_or(600).clamp(60, 3600);
    let (code, salt, code_hash) = generate_pairing_code();
    let (qr_secret, qr_secret_hash) = generate_qr_secret();
    let session_id = uuid::Uuid::now_v7().to_string();

    let now_ts = jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());
    let expires_ts = now_ts
        .checked_add(jiff::Span::new().seconds(dur as i64))
        .map(|t| t.to_string())
        .unwrap_or_else(|_| current_utc.to_string());

    let expires_epoch = now_ts
        .checked_add(jiff::Span::new().seconds(dur as i64))
        .map(|t| t.as_second())
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO lan_pairing_sessions (
            id, code_hash, salt, qr_secret_hash, expires_at, failed_attempts, status,
            created_by_user_id, paired_device_id, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, 0, 'pending',
            ?6, NULL, ?7, ?7
        )",
        params![
            session_id,
            code_hash,
            salt,
            qr_secret_hash,
            expires_ts,
            actor_user_id,
            current_utc
        ],
    )?;

    let config = get_or_create_host_config(conn, current_utc)?;
    mgr.update_mdns_pairing_status(conn, current_utc, true);

    let host_ips = if config.bind_address == "127.0.0.1" || config.bind_address == "localhost" {
        vec!["127.0.0.1".to_string()]
    } else {
        let mut ips = get_active_ipv4_addresses();
        if !ips.contains(&config.bind_address) && config.bind_address != "0.0.0.0" {
            ips.insert(0, config.bind_address.clone());
        }
        ips
    };
    let institution_name: String = conn
        .query_row("SELECT name FROM institution WHERE id = 1", [], |r| {
            r.get(0)
        })
        .or_else(|_| {
            conn.query_row(
                "SELECT value FROM system_config WHERE key = 'institution_name'",
                [],
                |r| r.get(0),
            )
        })
        .unwrap_or_else(|_| "Coachify Coaching Center".to_string());

    let qr_uri = format_qr_pairing_uri(
        &host_ips,
        config.port,
        &config.tls_fingerprint_sha256,
        &qr_secret,
        &institution_name,
        expires_epoch,
    );

    Ok(MobilePairingWindowInfo {
        session_id,
        qr_secret,
        qr_uri,
        code,
        expires_at: expires_ts,
        expires_in_seconds: dur,
        host_fingerprint: config.tls_fingerprint_sha256,
        host_ips,
        port: config.port,
        institution_name,
    })
}

pub fn set_user_pin(conn: &Connection, user_id: &str, pin: &str) -> Result<(), LanError> {
    use argon2::password_hash::PasswordHasher;
    let pin_trimmed = pin.trim();
    if pin_trimmed.len() < 4
        || pin_trimmed.len() > 6
        || !pin_trimmed.chars().all(|c| c.is_ascii_digit())
    {
        return Err(LanError::InvalidStaffPin);
    }
    let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
    let argon2 = argon2::Argon2::default();
    let pin_hash = argon2
        .hash_password(pin_trimmed.as_bytes(), &salt)
        .map_err(|e| LanError::Database(e.to_string()))?
        .to_string();

    conn.execute(
        "UPDATE users SET pin_hash = ?1, updated_at = ?2 WHERE id = ?3",
        params![pin_hash, jiff::Timestamp::now().to_string(), user_id],
    )?;
    Ok(())
}

pub fn update_host_receipt_printer(
    conn: &Connection,
    printer_name: Option<String>,
    current_utc: &str,
) -> Result<(), LanError> {
    let config = get_or_create_host_config(conn, current_utc)?;
    conn.execute(
        "UPDATE lan_host_config SET receipt_printer_name = ?1, updated_at = ?2 WHERE id = ?3",
        params![printer_name, current_utc, config.id],
    )?;
    Ok(())
}

// -----------------------------------------------------------------------------
// Axum HTTP Handlers (/hello, /health, /pair, /events)
// -----------------------------------------------------------------------------

async fn hello_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    if !ctx.rate_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded (max 30 requests/minute)",
        )
            .into_response();
    }

    let now_utc = jiff::Timestamp::now().to_string();
    let pairing_open = with_db(&ctx, |conn| {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'pending' AND expires_at > ?1",
                params![now_utc],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(count > 0)
    })
    .unwrap_or(false);

    let body = HelloResponse {
        product: "coachify".to_string(),
        protocol_version: 1,
        app_version: ctx.app_version,
        host_id: ctx.host_id,
        institution_name: ctx.institution_name,
        pairing_open,
        tls_fingerprint_sha256: ctx.fingerprint,
    };

    Json(body).into_response()
}

async fn health_handler(State(ctx): State<LanServerContext>) -> Response {
    let mut db_status = "ok".to_string();
    let mut active_count = 0u32;

    let res = with_db(&ctx, |conn| {
        let _: i32 = conn.query_row("SELECT 1", [], |r| r.get(0))?;
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(count)
    });

    match res {
        Ok(count) => active_count = count,
        Err(_) => db_status = "degraded".to_string(),
    }

    let status = if db_status == "ok" {
        "healthy"
    } else {
        "degraded"
    };

    let now_utc = jiff::Timestamp::now().to_string();

    let body = HealthResponse {
        status: status.to_string(),
        host_id: ctx.host_id,
        protocol_version: 1,
        database_status: db_status,
        uptime_seconds: ctx.started_at.elapsed().as_secs(),
        active_workstations_count: active_count,
        bound_port: ctx.port,
        server_time_utc: now_utc,
    };

    Json(body).into_response()
}

async fn pair_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<PairRequest>,
) -> Response {
    if !ctx.rate_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "code": "RATE_LIMIT_EXCEEDED",
                "message": "Too many requests. Please wait."
            })),
        )
            .into_response();
    }

    let now_utc = jiff::Timestamp::now().to_string();
    let client_ip = addr.ip().to_string();

    let result = with_db(&ctx, |conn| {
        // 1. Check if locked cooldown is active
        let locked_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
            params![now_utc],
            |r| r.get(0),
        )?;
        if locked_count > 0 {
            return Err(LanError::PairingLocked);
        }

        // 2. Fetch current active pending session
        let maybe_session = conn
            .query_row(
                "SELECT id, code_hash, salt, expires_at, failed_attempts
                 FROM lan_pairing_sessions
                 WHERE status = 'pending' AND expires_at > ?1
                 ORDER BY created_at DESC LIMIT 1",
                params![now_utc],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;

        let (session_id, code_hash, salt, _expires_at, failed_attempts) = match maybe_session {
            Some(s) => s,
            None => {
                // Mark any expired sessions
                let _ = conn.execute(
                    "UPDATE lan_pairing_sessions SET status = 'expired', updated_at = ?1 WHERE status = 'pending' AND expires_at <= ?1",
                    params![now_utc],
                );
                return Err(LanError::PairingClosed);
            }
        };

        // 3. Verify code using constant-time comparison (AC-2)
        if !verify_pairing_code(&payload.pairing_code, &salt, &code_hash) {
            let new_failed = failed_attempts + 1;
            if new_failed >= 5 {
                let lockout_expires_ts = jiff::Timestamp::now()
                    .checked_add(jiff::Span::new().minutes(15))
                    .map(|t| t.to_string())
                    .unwrap_or_else(|_| now_utc.clone());

                conn.execute(
                    "UPDATE lan_pairing_sessions
                     SET status = 'locked', expires_at = ?1, failed_attempts = ?2, updated_at = ?3
                     WHERE id = ?4",
                    params![lockout_expires_ts, new_failed, now_utc, session_id],
                )?;

                lan_manager().update_mdns_pairing_status(conn, &now_utc, false);
                return Err(LanError::PairingLocked);
            } else {
                conn.execute(
                    "UPDATE lan_pairing_sessions SET failed_attempts = ?1, updated_at = ?2 WHERE id = ?3",
                    params![new_failed, now_utc, session_id],
                )?;
                return Err(LanError::PairingCodeInvalid);
            }
        }

        // 4. Check license workstation limit (AC-4)
        let active_workstations: u32 = conn.query_row(
            "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
            [],
            |r| r.get(0),
        )?;
        let license = crate::licensing::get_license_status(conn, &now_utc).ok();
        let limit = license
            .as_ref()
            .and_then(|l| l.workstation_limit)
            .unwrap_or(1);
        if active_workstations >= limit {
            return Err(LanError::WorkstationLimitReached);
        }

        // 5. Generate 256-bit token & register device (AC-3)
        let (device_token, token_hash) = generate_device_token();
        let device_id = uuid::Uuid::now_v7().to_string();
        let os_ver = payload.os_version.unwrap_or_else(|| "Windows".to_string());
        let app_ver = payload.app_version.unwrap_or_else(|| "0.1.0".to_string());

        conn.execute(
            "INSERT INTO devices (
                id, name, windows_name, token_hash, status, is_host,
                paired_at, last_seen_at, app_version, ip_address, os_version
            ) VALUES (
                ?1, ?2, ?3, ?4, 'active', 0,
                ?5, ?5, ?6, ?7, ?8
            )",
            params![
                device_id,
                payload.workstation_name,
                payload.windows_name,
                token_hash,
                now_utc,
                app_ver,
                client_ip,
                os_ver
            ],
        )?;

        let details = serde_json::json!({
            "workstation_name": payload.workstation_name,
            "windows_name": payload.windows_name,
            "nonce": payload.client_nonce,
        })
        .to_string();

        let event_id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
             VALUES (?1, ?2, 'paired', NULL, ?3, ?4, ?5)",
            params![event_id, device_id, client_ip, details, now_utc],
        )?;

        // Close pairing session (single-use invariant AC-2)
        conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'used', paired_device_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![device_id, now_utc, session_id],
        )?;

        lan_manager().update_mdns_pairing_status(conn, &now_utc, false);

        Ok(PairSuccessResponse {
            device_id,
            device_token,
            institution_name: ctx.institution_name.clone(),
            host_device_id: ctx.host_id.clone(),
            host_fingerprint: ctx.fingerprint.clone(),
        })
    });

    match result {
        Ok(resp) => Json(resp).into_response(),
        Err(LanError::PairingLocked) => (
            StatusCode::LOCKED,
            Json(serde_json::json!({
                "code": "LAN_PAIRING_LOCKED",
                "message": "Pairing window locked due to multiple failed attempts (15-minute cooldown)"
            })),
        )
            .into_response(),
        Err(LanError::PairingClosed) | Err(LanError::PairingExpired) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": "LAN_PAIRING_CLOSED",
                "message": "No active pairing window is currently open on the host"
            })),
        )
            .into_response(),
        Err(LanError::PairingCodeInvalid) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": "LAN_PAIRING_CODE_INVALID",
                "message": "Invalid 6-digit pairing code"
            })),
        )
            .into_response(),
        Err(LanError::WorkstationLimitReached) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "LAN_WORKSTATION_LIMIT_REACHED",
                "message": "License workstation quota has been reached on the host"
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": err.code(),
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

async fn pair_qr_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<PairQrRequest>,
) -> Response {
    if !ctx.rate_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "code": "RATE_LIMIT_EXCEEDED",
                "message": "Too many requests. Please wait."
            })),
        )
            .into_response();
    }

    let now_utc = jiff::Timestamp::now().to_string();
    let client_ip = addr.ip().to_string();

    let result = with_db(&ctx, |conn| {
        // 1. Check if lockout cooldown is active
        let locked_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
            params![now_utc],
            |r| r.get(0),
        )?;
        if locked_count > 0 {
            return Err(LanError::PairingCooldownActive);
        }

        let input_secret_hash = {
            let mut hasher = Sha256::new();
            hasher.update(payload.qr_secret.trim().as_bytes());
            hex::encode(hasher.finalize())
        };

        // Check if this secret was already used
        let used_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE qr_secret_hash = ?1 AND status = 'used'",
            params![input_secret_hash],
            |r| r.get(0),
        ).unwrap_or(0);
        if used_count > 0 {
            return Err(LanError::PairingSessionAlreadyUsed);
        }

        // 2. Fetch current active pending session
        let maybe_session = conn
            .query_row(
                "SELECT id, qr_secret_hash, expires_at, failed_attempts
                 FROM lan_pairing_sessions
                 WHERE status = 'pending' AND expires_at > ?1 AND qr_secret_hash IS NOT NULL
                 ORDER BY created_at DESC LIMIT 1",
                params![now_utc],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()?;

        let (session_id, expected_secret_hash, _expires_at, failed_attempts) = match maybe_session {
            Some(s) => s,
            None => {
                let expired_with_secret: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM lan_pairing_sessions WHERE qr_secret_hash = ?1 AND expires_at <= ?2",
                    params![input_secret_hash, now_utc],
                    |r| r.get(0),
                ).unwrap_or(0);
                if expired_with_secret > 0 {
                    return Err(LanError::PairingExpired);
                }

                let _ = conn.execute(
                    "UPDATE lan_pairing_sessions SET status = 'expired', updated_at = ?1 WHERE status = 'pending' AND expires_at <= ?1",
                    params![now_utc],
                );
                return Err(LanError::PairingClosed);
            }
        };

        // 3. Verify secret using constant-time comparison
        let a = input_secret_hash.as_bytes();
        let b = expected_secret_hash.as_bytes();
        let mut diff = 0u8;
        if a.len() == b.len() {
            for (x, y) in a.iter().zip(b.iter()) {
                diff |= x ^ y;
            }
        } else {
            diff = 1;
        }

        if diff != 0 {
            let new_failed = failed_attempts + 1;
            if new_failed >= 5 {
                let lockout_expires_ts = jiff::Timestamp::now()
                    .checked_add(jiff::Span::new().minutes(15))
                    .map(|t| t.to_string())
                    .unwrap_or_else(|_| now_utc.clone());

                conn.execute(
                    "UPDATE lan_pairing_sessions
                     SET status = 'locked', expires_at = ?1, failed_attempts = ?2, updated_at = ?3
                     WHERE id = ?4",
                    params![lockout_expires_ts, new_failed, now_utc, session_id],
                )?;

                lan_manager().update_mdns_pairing_status(conn, &now_utc, false);

                let audit_entry = crate::audit::AuditEntry {
                    action: "security.pairing_brute_force_locked".to_string(),
                    entity: "lan_pairing_sessions".to_string(),
                    entity_id: session_id.clone(),
                    before_json: None,
                    after_json: Some(serde_json::json!({
                        "failed_attempts": new_failed,
                        "source": "qr_secret",
                        "remote_ip": client_ip,
                    })),
                };
                let _ = crate::audit::record_audit_entry_with_user(
                    conn,
                    audit_entry,
                    None,
                    None,
                    &now_utc,
                );

                return Err(LanError::PairingLocked);
            } else {
                conn.execute(
                    "UPDATE lan_pairing_sessions SET failed_attempts = ?1, updated_at = ?2 WHERE id = ?3",
                    params![new_failed, now_utc, session_id],
                )?;
                return Err(LanError::PairingSecretInvalid);
            }
        }

        // 4. Check license workstation limit
        let active_workstations: u32 = conn.query_row(
            "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
            [],
            |r| r.get(0),
        )?;
        let license = crate::licensing::get_license_status(conn, &now_utc).ok();
        let limit = license
            .as_ref()
            .and_then(|l| l.workstation_limit)
            .unwrap_or(1);
        if active_workstations >= limit {
            return Err(LanError::WorkstationLimitReached);
        }

        // 5. Generate 256-bit token & register device
        let (device_token, token_hash) = generate_device_token();
        let device_id = uuid::Uuid::now_v7().to_string();
        let platform = payload.platform.to_lowercase();
        let os_ver = payload.os_version.unwrap_or_else(|| platform.clone());
        let app_ver = payload.app_version.unwrap_or_else(|| "0.1.0".to_string());
        let device_type = payload
            .device_type
            .unwrap_or_else(|| "mobile_phone".to_string());

        conn.execute(
            "INSERT INTO devices (
                id, name, windows_name, token_hash, status, is_host,
                paired_at, last_seen_at, app_version, ip_address, os_version,
                paired_via, platform, device_type
            ) VALUES (
                ?1, ?2, ?2, ?3, 'active', 0,
                ?4, ?4, ?5, ?6, ?7,
                'qr', ?8, ?9
            )",
            params![
                device_id,
                payload.device_name,
                token_hash,
                now_utc,
                app_ver,
                client_ip,
                os_ver,
                platform,
                device_type,
            ],
        )?;

        let details = serde_json::json!({
            "device_name": payload.device_name,
            "platform": platform,
            "device_type": device_type,
            "nonce": payload.client_nonce,
            "paired_via": "qr",
        })
        .to_string();

        let event_id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
             VALUES (?1, ?2, 'paired', NULL, ?3, ?4, ?5)",
            params![event_id, device_id, client_ip, details, now_utc],
        )?;

        conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'used', paired_device_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![device_id, now_utc, session_id],
        )?;

        lan_manager().update_mdns_pairing_status(conn, &now_utc, false);

        let active_features = vec![
            "attendance".to_string(),
            "cashier".to_string(),
            "admissions".to_string(),
            "exams".to_string(),
            "cockpit".to_string(),
        ];

        Ok(PairQrSuccessResponse {
            device_id,
            device_token,
            institution_name: ctx.institution_name.clone(),
            host_id: ctx.host_id.clone(),
            host_name: "Coachify Host".to_string(),
            active_features,
        })
    });

    match result {
        Ok(resp) => Json(resp).into_response(),
        Err(LanError::PairingCooldownActive) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "PAIRING_COOLDOWN_ACTIVE",
                "message": "Pairing window cooldown is active after too many failed attempts"
            })),
        )
            .into_response(),
        Err(LanError::PairingLocked) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "PAIRING_LOCKED",
                "message": "Pairing window locked due to multiple failed attempts (15-minute cooldown)"
            })),
        )
            .into_response(),
        Err(LanError::PairingSessionAlreadyUsed) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "code": "PAIRING_SESSION_ALREADY_USED",
                "message": "Pairing session has already been used"
            })),
        )
            .into_response(),
        Err(LanError::PairingExpired) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "PAIRING_EXPIRED",
                "message": "Pairing secret has expired"
            })),
        )
            .into_response(),
        Err(LanError::PairingClosed) | Err(LanError::PairingNotOpen) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "PAIRING_CLOSED",
                "message": "No active pairing window is currently open on the host"
            })),
        )
            .into_response(),
        Err(LanError::PairingSecretInvalid) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "PAIRING_SECRET_INVALID",
                "message": "Invalid QR pairing secret"
            })),
        )
            .into_response(),
        Err(LanError::WorkstationLimitReached) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "LAN_WORKSTATION_LIMIT_REACHED",
                "message": "License workstation quota has been reached on the host"
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": err.code(),
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

async fn staff_login_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<StaffLoginRequest>,
) -> Response {
    let header_token = headers
        .get("X-Coachify-Device-Token")
        .and_then(|h| h.to_str().ok());

    let raw_token = match header_token {
        Some(t) => t.to_string(),
        None => match payload.device_token {
            Some(ref dt) if !dt.trim().is_empty() => dt.trim().to_string(),
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "code": "DEVICE_TOKEN_REQUIRED",
                        "message": "Header X-Coachify-Device-Token or device_token in payload is required"
                    })),
                )
                    .into_response();
            }
        },
    };

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let now_utc = jiff::Timestamp::now().to_string();

    let result = with_db(&ctx, |conn| {
        let dev = conn
            .query_row(
                "SELECT id, status FROM devices WHERE token_hash = ?1",
                params![token_hash],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;

        let (device_id, status) = match dev {
            Some(d) => d,
            None => return Err(LanError::DeviceNotFound),
        };

        if status == "revoked" {
            return Err(LanError::DeviceRevoked);
        }
        if status == "blocked" {
            return Err(LanError::DeviceBlocked);
        }

        let pin = payload.staff_pin.trim();
        if pin.len() < 4 || pin.len() > 6 || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err(LanError::InvalidStaffPin);
        }

        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, password_hash, pin_hash, staff_profile_id, status
             FROM users WHERE status = 'active'",
        )?;

        let mut user_match = None;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let u_id: String = row.get(0)?;
            let u_name: String = row.get(1)?;
            let d_name: String = row.get(2)?;
            let pw_hash: String = row.get(3)?;
            let pin_hash_opt: Option<String> = row.get(4)?;
            let staff_profile_id: Option<String> = row.get(5)?;

            if let Some(ref target_u) = payload.username {
                if !u_name.eq_ignore_ascii_case(target_u) {
                    continue;
                }
            }

            let matched = if let Some(ref ph) = pin_hash_opt {
                if let Ok(parsed_hash) = argon2::PasswordHash::new(ph) {
                    argon2::Argon2::default()
                        .verify_password(pin.as_bytes(), &parsed_hash)
                        .is_ok()
                } else {
                    false
                }
            } else if let Ok(parsed_hash) = argon2::PasswordHash::new(&pw_hash) {
                argon2::Argon2::default()
                    .verify_password(pin.as_bytes(), &parsed_hash)
                    .is_ok()
            } else {
                false
            };

            if matched {
                user_match = Some((u_id, u_name, d_name, staff_profile_id));
                break;
            }
        }

        let (user_id, _username, display_name, staff_profile_id) = match user_match {
            Some(u) => u,
            None => return Err(LanError::InvalidStaffPin),
        };

        let roles: Vec<String> = {
            let mut r_stmt = conn.prepare(
                "SELECT r.name FROM roles r JOIN user_roles ur ON ur.role_id = r.id WHERE ur.user_id = ?1"
            )?;
            let r_rows = r_stmt.query_map(params![user_id], |r| r.get(0))?;
            r_rows.filter_map(|r| r.ok()).collect()
        };

        let permissions: Vec<String> = {
            let mut p_stmt = conn.prepare(
                "SELECT DISTINCT rp.permission_code FROM role_permissions rp
                 JOIN user_roles ur ON ur.role_id = rp.role_id
                 WHERE ur.user_id = ?1",
            )?;
            let p_rows = p_stmt.query_map(params![user_id], |r| r.get(0))?;
            p_rows.filter_map(|r| r.ok()).collect()
        };

        let session_token = format!("st_{}", uuid::Uuid::now_v7().simple());
        let session_token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(session_token.as_bytes());
            hex::encode(hasher.finalize())
        };

        let now_ts = jiff::Timestamp::from_str(&now_utc).unwrap_or_else(|_| jiff::Timestamp::now());
        let expires_at_iso = now_ts
            .checked_add(jiff::Span::new().hours(4))
            .map(|t| t.to_string())
            .unwrap_or_else(|_| now_utc.clone());
        let abs_expires_at_iso = now_ts
            .checked_add(jiff::Span::new().hours(8))
            .map(|t| t.to_string())
            .unwrap_or_else(|_| now_utc.clone());

        conn.execute(
            "INSERT INTO sessions (
                id, user_id, device_id, token_hash,
                created_at, last_seen_at, expires_at, absolute_expires_at, revoked_at
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?5, ?6, ?7, NULL
            )",
            params![
                session_token,
                user_id,
                device_id,
                session_token_hash,
                now_utc,
                expires_at_iso,
                abs_expires_at_iso,
            ],
        )?;

        let staff_id = staff_profile_id.unwrap_or_else(|| user_id.clone());
        let primary_role = roles
            .first()
            .cloned()
            .unwrap_or_else(|| "staff".to_string());

        Ok(StaffLoginResponse {
            session_token,
            staff_id,
            role: primary_role,
            full_name: display_name,
            permissions,
            expires_at: expires_at_iso,
        })
    });

    match result {
        Ok(resp) => Json(resp).into_response(),
        Err(LanError::DeviceRevoked) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_REVOKED",
                "message": "Workstation device has been revoked by host administrator"
            })),
        )
            .into_response(),
        Err(LanError::DeviceBlocked) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_BLOCKED",
                "message": "Workstation device is currently blocked by host"
            })),
        )
            .into_response(),
        Err(LanError::InvalidStaffPin) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": "INVALID_STAFF_PIN",
                "message": "Invalid 4 to 6 digit staff PIN"
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": err.code(),
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

async fn rpc_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let token = headers
        .get("X-Coachify-Device-Token")
        .and_then(|h| h.to_str().ok());

    let raw_token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "ok": false,
                    "error": {
                        "code": "DEVICE_TOKEN_REQUIRED",
                        "message": "Header X-Coachify-Device-Token is required"
                    }
                })),
            )
                .into_response();
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let now_utc = jiff::Timestamp::now().to_string();
    let client_ip = addr.ip().to_string();

    let session_header = headers
        .get("X-Coachify-Session-Token")
        .and_then(|h| h.to_str().ok())
        .or_else(|| {
            headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
        });

    let command_str = payload
        .get("method")
        .or_else(|| payload.get("command"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let params_val = payload
        .get("params")
        .or_else(|| payload.get("payload"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let (status_code, body) = with_db(&ctx, |conn| {
        let dev = conn.query_row(
            "SELECT id, status FROM devices WHERE token_hash = ?1",
            params![token_hash],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        ).optional()?;

        let (device_id, dev_status) = match dev {
            Some(d) => d,
            None => {
                return Ok((
                    StatusCode::FORBIDDEN,
                    serde_json::json!({
                        "ok": false,
                        "error": {
                            "code": "DEVICE_REVOKED",
                            "message": "Device token not found or revoked"
                        }
                    })
                ));
            }
        };

        if dev_status == "revoked" {
            return Ok((
                StatusCode::FORBIDDEN,
                serde_json::json!({
                    "ok": false,
                    "error": {
                        "code": "DEVICE_REVOKED",
                        "message": "Workstation device has been revoked by host administrator"
                    }
                })
            ));
        }

        if dev_status == "blocked" {
            return Ok((
                StatusCode::FORBIDDEN,
                serde_json::json!({
                    "ok": false,
                    "error": {
                        "code": "DEVICE_BLOCKED",
                        "message": "Workstation device is currently blocked by host"
                    }
                })
            ));
        }

        let _ = conn.execute(
            "UPDATE devices SET last_seen_at = ?1, ip_address = ?2 WHERE id = ?3",
            params![now_utc, client_ip, device_id],
        );

        if let Some(sess_tok) = session_header {
            let sess = conn.query_row(
                "SELECT id, expires_at, last_seen_at, revoked_at FROM sessions WHERE id = ?1 OR token_hash = ?2",
                params![sess_tok, {
                    let mut h = Sha256::new();
                    h.update(sess_tok.as_bytes());
                    hex::encode(h.finalize())
                }],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?)),
            ).optional()?;

            if let Some((sess_id, expires_at, last_seen_at, revoked_at)) = sess {
                if revoked_at.is_some() {
                    return Ok((
                        StatusCode::UNAUTHORIZED,
                        serde_json::json!({
                            "ok": false,
                            "error": {
                                "code": "SESSION_REVOKED",
                                "message": "Session has been revoked"
                            }
                        })
                    ));
                }

                let now_ts = jiff::Timestamp::from_str(&now_utc).unwrap_or_else(|_| jiff::Timestamp::now());
                if let Ok(last_seen_ts) = jiff::Timestamp::from_str(&last_seen_at) {
                    if now_ts.duration_since(last_seen_ts).as_secs() > 900 {
                        return Ok((
                            StatusCode::UNAUTHORIZED,
                            serde_json::json!({
                                "ok": false,
                                "error": {
                                    "code": "SESSION_LOCKED_PIN_REQUIRED",
                                    "message": "Mobile session is locked; staff PIN entry required"
                                }
                            })
                        ));
                    }
                }

                if let Ok(exp_ts) = jiff::Timestamp::from_str(&expires_at) {
                    if now_ts > exp_ts {
                        return Ok((
                            StatusCode::UNAUTHORIZED,
                            serde_json::json!({
                                "ok": false,
                                "error": {
                                    "code": "SESSION_LOCKED_PIN_REQUIRED",
                                    "message": "Mobile session has expired; staff PIN entry required"
                                }
                            })
                        ));
                    }
                }

                let _ = conn.execute(
                    "UPDATE sessions SET last_seen_at = ?1 WHERE id = ?2",
                    params![now_utc, sess_id],
                );
            }
        }

        let (resp_str, _) = crate::dispatcher::dispatch_at(
            conn,
            command_str,
            &params_val.to_string(),
            session_header,
            None,
            &now_utc,
        );

        let parsed: serde_json::Value = serde_json::from_str(&resp_str).unwrap_or_else(|_| {
            serde_json::json!({
                "ok": false,
                "error": { "code": "INTERNAL_ERROR", "message": "Failed to parse dispatcher response" }
            })
        });

        Ok((StatusCode::OK, parsed))
    }).unwrap_or_else(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "ok": false,
                "error": { "code": err.code(), "message": err.to_string() }
            })
        )
    });

    (status_code, Json(body)).into_response()
}

async fn host_receipt_print_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<HostReceiptPrintRequest>,
) -> Response {
    let header_token = headers
        .get("X-Coachify-Device-Token")
        .and_then(|h| h.to_str().ok());

    let raw_token = match header_token {
        Some(t) => t.to_string(),
        None => match payload.device_token {
            Some(ref dt) if !dt.trim().is_empty() => dt.trim().to_string(),
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "code": "DEVICE_TOKEN_REQUIRED",
                        "message": "Header X-Coachify-Device-Token or device_token in payload is required"
                    })),
                )
                    .into_response();
            }
        },
    };

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let now_utc = jiff::Timestamp::now().to_string();

    let result = with_db(&ctx, |conn| {
        let (device_id, dev_status): (String, String) = conn.query_row(
            "SELECT id, status FROM devices WHERE token_hash = ?1",
            params![token_hash],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        if dev_status == "revoked" {
            return Err(LanError::DeviceRevoked);
        }
        if dev_status == "blocked" {
            return Err(LanError::DeviceBlocked);
        }

        let designated_printer: Option<String> = conn
            .query_row(
                "SELECT receipt_printer_name FROM lan_host_config WHERE id = 'default'",
                [],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        let printer_name =
            designated_printer.unwrap_or_else(|| "Default OS Thermal Printer".to_string());
        let print_job_id = format!("pj_{}", uuid::Uuid::now_v7().simple());

        let print_audit_entry = crate::audit::AuditEntry {
            action: "print.host_receipt".to_string(),
            entity: "payment".to_string(),
            entity_id: payload.payment_id.clone(),
            before_json: None,
            after_json: Some(serde_json::json!({
                "print_job_id": print_job_id,
                "printer_name": printer_name,
                "copies": payload.copies.unwrap_or(1),
                "template": payload.receipt_template,
                "remote_ip": addr.ip().to_string(),
            })),
        };
        let _ = crate::audit::record_audit_entry_with_user(
            conn,
            print_audit_entry,
            None,
            Some(&device_id),
            &now_utc,
        );

        Ok(HostReceiptPrintResponse {
            print_job_id,
            status: "dispatched".to_string(),
            printer_name,
        })
    });

    match result {
        Ok(resp) => Json(resp).into_response(),
        Err(LanError::DeviceRevoked) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_REVOKED",
                "message": "Workstation device has been revoked by host administrator"
            })),
        )
            .into_response(),
        Err(LanError::DeviceBlocked) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_BLOCKED",
                "message": "Workstation device is currently blocked by host"
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": err.code(),
                "message": err.to_string()
            })),
        )
            .into_response(),
    }
}

async fn events_handler(
    State(ctx): State<LanServerContext>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    uri: axum::http::Uri,
) -> Response {
    let query_token = uri.query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next()? == "token" {
                parts.next()
            } else {
                None
            }
        })
    });
    let token = headers
        .get("X-Coachify-Device-Token")
        .and_then(|h| h.to_str().ok())
        .or(query_token);

    let raw_token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": "DEVICE_TOKEN_REQUIRED",
                    "message": "Header X-Coachify-Device-Token is required"
                })),
            )
                .into_response();
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let now_utc = jiff::Timestamp::now().to_string();
    let client_ip = addr.ip().to_string();

    let check = with_db(&ctx, |conn| {
        let dev = conn
            .query_row(
                "SELECT id, status FROM devices WHERE token_hash = ?1",
                params![token_hash],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;

        if let Some((ref dev_id, ref status)) = dev {
            if status == "active" {
                let _ = conn.execute(
                    "UPDATE devices SET last_seen_at = ?1, ip_address = ?2 WHERE id = ?3",
                    params![now_utc, client_ip, dev_id],
                );
            }
        }
        Ok(dev)
    });

    let (dev_id, status) = match check {
        Ok(Some((id, st))) => (id, st),
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "code": "DEVICE_REVOKED",
                    "message": "Device not found or token has been revoked"
                })),
            )
                .into_response();
        }
    };

    if status == "revoked" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_REVOKED",
                "message": "Workstation device has been permanently revoked by host"
            })),
        )
            .into_response();
    }

    if status == "blocked" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "code": "DEVICE_BLOCKED",
                "message": "Workstation device is currently blocked by host"
            })),
        )
            .into_response();
    }

    let rx = control_sender().subscribe();
    let target_id = dev_id.clone();

    let stream = BroadcastStream::new(rx).filter_map(move |item| match item {
        Ok(DeviceControlEvent::Revoked { device_id, reason }) if device_id == target_id => {
            Some(Ok::<_, std::convert::Infallible>(
                Event::default().event("device.revoked").data(
                    serde_json::json!({
                        "event": "device.revoked",
                        "device_id": device_id,
                        "reason": reason
                    })
                    .to_string(),
                ),
            ))
        }
        Ok(DeviceControlEvent::Blocked { device_id, reason }) if device_id == target_id => {
            Some(Ok::<_, std::convert::Infallible>(
                Event::default().event("device.blocked").data(
                    serde_json::json!({
                        "event": "device.blocked",
                        "device_id": device_id,
                        "reason": reason
                    })
                    .to_string(),
                ),
            ))
        }
        _ => None,
    });

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response()
}

// -----------------------------------------------------------------------------
// Lifecycle: Start Host
// -----------------------------------------------------------------------------

pub fn start_host(
    conn: &Connection,
    port_override: Option<u16>,
    bind_override: Option<String>,
    current_utc: &str,
) -> Result<LanHostStatus, LanError> {
    ensure_crypto_provider_installed();
    let mgr = lan_manager();

    if let Some(p) = mgr.running_port() {
        return Err(LanError::HostAlreadyRunning(p));
    }

    let mut config = get_or_create_host_config(conn, current_utc)?;

    let target_port = port_override.unwrap_or(config.port);
    if !(1024..=65535).contains(&target_port) {
        return Err(LanError::InvalidPortRange(target_port));
    }

    let target_bind = bind_override.unwrap_or_else(|| config.bind_address.clone());

    // Pre-flight check port binding
    let test_addr: SocketAddr = format!("{}:{}", target_bind, target_port)
        .parse()
        .map_err(|e| LanError::BindFailed(target_bind.clone(), format!("{}", e)))?;

    if let Err(e) = std::net::TcpListener::bind(test_addr) {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            return Err(LanError::PortInUse(target_port));
        }
    }

    let host_dev = accounts::get_or_create_host_device(conn, current_utc)
        .map_err(|e| LanError::Database(e.to_string()))?;

    let institution_name: String = conn
        .query_row(
            "SELECT value FROM system_config WHERE key = 'institution_name'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "Coachify Coaching Center".to_string());

    let db_path = db::resolve_dev_db_path().ok();

    let cert_pem = config.tls_cert_pem.clone();
    let key_pem = config.tls_key_pem.clone();
    let fingerprint = config.tls_fingerprint_sha256.clone();

    let tls_config = mgr.tokio_runtime.block_on(async {
        RustlsConfig::from_pem(cert_pem.as_bytes().to_vec(), key_pem.as_bytes().to_vec())
            .await
            .map_err(|e| LanError::CertGenerationFailed(e.to_string()))
    })?;

    let rate_limiter = RateLimiter::new();
    let server_ctx = LanServerContext {
        host_id: host_dev.id.clone(),
        institution_name: institution_name.clone(),
        app_version: "0.1.0".to_string(),
        fingerprint: fingerprint.clone(),
        db_path,
        started_at: Instant::now(),
        port: target_port,
        rate_limiter,
    };

    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/health", get(health_handler))
        .route("/pair", post(pair_handler))
        .route("/pair/qr", post(pair_qr_handler))
        .route("/auth/staff-login", post(staff_login_handler))
        .route("/rpc", post(rpc_handler))
        .route("/print/host-receipt", post(host_receipt_print_handler))
        .route("/events", get(events_handler))
        .with_state(server_ctx);

    let handle = axum_server::Handle::new();
    let handle_clone = handle.clone();

    mgr.tokio_runtime.spawn(async move {
        let server = axum_server::bind_rustls(test_addr, tls_config).handle(handle_clone);
        if let Err(err) = server
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
        {
            eprintln!("LAN host axum server terminated: {err}");
        }
    });

    // Wait up to 500ms for listener to bind
    let listening_addr = mgr.tokio_runtime.block_on(async {
        tokio::time::timeout(Duration::from_millis(500), handle.listening()).await
    });

    let bound_port = match listening_addr {
        Ok(Some(addr)) => addr.port(),
        _ => target_port,
    };

    // mDNS announcement
    let mut mdns_daemon = None;
    let mut mdns_fullname = None;

    if config.mdns_enabled {
        if let Ok(mdns) = ServiceDaemon::new() {
            let service_type = "_coachify._tcp.local.";
            let instance_name = format!(
                "Coachify - {} ({})",
                institution_name, host_dev.windows_name
            );
            let host_name = format!("{}.local.", host_dev.windows_name.replace(' ', "-"));

            let mut properties = HashMap::new();
            properties.insert("host_id".to_string(), host_dev.id.clone());
            properties.insert("proto".to_string(), "1".to_string());
            properties.insert("fingerprint".to_string(), fingerprint.clone());
            properties.insert("inst".to_string(), institution_name.clone());
            properties.insert("ver".to_string(), "0.1.0".to_string());
            properties.insert("pairing".to_string(), "0".to_string());

            let primary_ip = get_local_ip_addresses()
                .first()
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string());

            if let Ok(service_info) = ServiceInfo::new(
                service_type,
                &instance_name,
                &host_name,
                primary_ip.as_str(),
                bound_port,
                properties,
            ) {
                let fname = service_info.get_fullname().to_string();
                if mdns.register(service_info).is_ok() {
                    mdns_fullname = Some(fname);
                    mdns_daemon = Some(mdns);
                }
            }
        }
    }

    if config.prevent_sleep {
        set_sleep_inhibition(true);
    }

    conn.execute(
        "UPDATE lan_host_config SET is_enabled = 1, port = ?1, bind_address = ?2, updated_at = ?3 WHERE id = 'default'",
        params![bound_port, target_bind, current_utc],
    )?;

    config.is_enabled = true;
    config.port = bound_port;
    config.bind_address = target_bind.clone();

    let mut lock = mgr.active_server.lock().unwrap();
    *lock = Some(ActiveLanServer {
        port: bound_port,
        bind_address: target_bind,
        handle,
        started_at: Instant::now(),
        mdns_daemon,
        mdns_fullname,
    });
    drop(lock);

    get_host_status(conn, current_utc)
}

// -----------------------------------------------------------------------------
// Lifecycle: Stop Host
// -----------------------------------------------------------------------------

pub fn stop_host(conn: &Connection, current_utc: &str) -> Result<LanHostStatus, LanError> {
    let mgr = lan_manager();
    let mut lock = mgr.active_server.lock().unwrap();

    let active = match lock.take() {
        Some(s) => s,
        None => return Err(LanError::HostNotRunning),
    };

    if let (Some(mdns), Some(fullname)) = (active.mdns_daemon, active.mdns_fullname) {
        let _ = mdns.unregister(&fullname);
        let _ = mdns.shutdown();
    }

    active
        .handle
        .graceful_shutdown(Some(Duration::from_millis(200)));
    set_sleep_inhibition(false);

    conn.execute(
        "UPDATE lan_host_config SET is_enabled = 0, updated_at = ?1 WHERE id = 'default'",
        params![current_utc],
    )?;

    drop(lock);
    get_host_status(conn, current_utc)
}

// -----------------------------------------------------------------------------
// Host Configuration & Status
// -----------------------------------------------------------------------------

pub fn update_host_config(
    conn: &Connection,
    payload: UpdateHostConfigPayload,
    current_utc: &str,
) -> Result<LanHostStatus, LanError> {
    let mgr = lan_manager();
    let current_config = get_or_create_host_config(conn, current_utc)?;

    if let Some(port) = payload.port {
        if !(1024..=65535).contains(&port) {
            return Err(LanError::InvalidPortRange(port));
        }
        if mgr.is_running() && port != current_config.port {
            return Err(LanError::PortInUse(port));
        }
    }

    let port = payload.port.unwrap_or(current_config.port);
    let bind_address = payload.bind_address.unwrap_or(current_config.bind_address);
    let auto_start_on_boot = payload
        .auto_start_on_boot
        .unwrap_or(current_config.auto_start_on_boot);
    let prevent_sleep = payload
        .prevent_sleep
        .unwrap_or(current_config.prevent_sleep);
    let mdns_enabled = payload.mdns_enabled.unwrap_or(current_config.mdns_enabled);

    if mgr.is_running() {
        set_sleep_inhibition(prevent_sleep);
    }

    conn.execute(
        "UPDATE lan_host_config
         SET port = ?1, bind_address = ?2, auto_start_on_boot = ?3,
             prevent_sleep = ?4, mdns_enabled = ?5, updated_at = ?6
         WHERE id = 'default'",
        params![
            port,
            bind_address,
            if auto_start_on_boot { 1 } else { 0 },
            if prevent_sleep { 1 } else { 0 },
            if mdns_enabled { 1 } else { 0 },
            current_utc
        ],
    )?;

    get_host_status(conn, current_utc)
}

pub fn get_host_status(conn: &Connection, current_utc: &str) -> Result<LanHostStatus, LanError> {
    let config = get_or_create_host_config(conn, current_utc)?;
    let mgr = lan_manager();
    let is_running = mgr.is_running();
    let uptime_seconds = mgr.uptime_seconds();

    let host_dev = accounts::get_or_create_host_device(conn, current_utc)
        .map_err(|e| LanError::Database(e.to_string()))?;

    let bound_addresses = get_local_ip_addresses();
    let guidance = get_firewall_guidance();

    let license_entitled = crate::licensing::get_license_status(conn, current_utc)
        .ok()
        .is_some_and(|s| {
            s.tier == "trial"
                || s.tier == "local_pro"
                || s.capabilities
                    .iter()
                    .any(|c| c == "lan_multi_device" || c == "lan")
        });

    let active_workstations: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(LanHostStatus {
        is_running,
        port: config.port,
        bind_address: config.bind_address,
        bound_addresses,
        fingerprint: config.tls_fingerprint_sha256,
        active_workstations,
        network_profile: guidance.profile,
        firewall_status: if guidance.rule_exists {
            "configured".to_string()
        } else {
            "rule_missing".to_string()
        },
        mdns_broadcasting: is_running && config.mdns_enabled,
        license_entitled,
        prevent_sleep: config.prevent_sleep,
        auto_start_on_boot: config.auto_start_on_boot,
        uptime_seconds,
        host_device_id: host_dev.id,
        hostname: host_dev.windows_name,
    })
}

// -----------------------------------------------------------------------------
// Windows Platform Inspection: Network Profile & Firewall (AC-8)
// -----------------------------------------------------------------------------

static FIREWALL_CACHE: OnceLock<std::sync::RwLock<(Instant, FirewallGuidance)>> = OnceLock::new();

pub fn get_firewall_guidance() -> FirewallGuidance {
    let powershell_command = "Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } | Set-NetConnectionProfile -NetworkCategory Private".to_string();
    let netsh_command = "netsh advfirewall firewall add rule name=\"Coachify LAN Host\" dir=in action=allow protocol=TCP localport=50443 profile=private".to_string();

    let cache = FIREWALL_CACHE.get_or_init(|| {
        std::sync::RwLock::new((
            Instant::now() - Duration::from_secs(100),
            FirewallGuidance {
                profile: "Private".to_string(),
                is_private: true,
                rule_exists: false,
                powershell_command: powershell_command.clone(),
                netsh_command: netsh_command.clone(),
            },
        ))
    });

    if let Ok(guard) = cache.read() {
        if guard.0.elapsed() < Duration::from_secs(30) {
            return guard.1.clone();
        }
    }

    #[cfg(windows)]
    {
        let mut profile = "Private".to_string();
        let mut is_private = true;

        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-NetConnectionProfile | Select-Object -ExpandProperty NetworkCategory",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("Public") {
                profile = "Public".to_string();
                is_private = false;
            } else if text.contains("Private") {
                profile = "Private".to_string();
                is_private = true;
            }
        }

        let rule_exists = std::process::Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "show",
                "rule",
                "name=Coachify LAN Host",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let guidance = FirewallGuidance {
            profile,
            is_private,
            rule_exists,
            powershell_command,
            netsh_command,
        };

        if let Ok(mut guard) = cache.write() {
            *guard = (Instant::now(), guidance.clone());
        }

        guidance
    }
    #[cfg(not(windows))]
    {
        FirewallGuidance {
            profile: "Private".to_string(),
            is_private: true,
            rule_exists: true,
            powershell_command,
            netsh_command,
        }
    }
}

// -----------------------------------------------------------------------------
// Workstation Discovery Scanner & Manual IP Ping (AC-6, AC-7)
// -----------------------------------------------------------------------------

pub fn discover_hosts(payload: DiscoverHostsPayload) -> Result<DiscoveredHostList, LanError> {
    let timeout = Duration::from_millis(payload.timeout_ms.unwrap_or(2000));
    let mdns = ServiceDaemon::new().map_err(|_e| LanError::DiscoveryTimeout)?;
    let receiver = mdns
        .browse("_coachify._tcp.local.")
        .map_err(|_| LanError::DiscoveryTimeout)?;

    let start = Instant::now();
    let mut discovered: HashMap<String, DiscoveredHost> = HashMap::new();

    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        if let Ok(ServiceEvent::ServiceResolved(info)) =
            receiver.recv_timeout(remaining.min(Duration::from_millis(300)))
        {
            let host_id = info
                .get_property_val_str("host_id")
                .unwrap_or("unknown")
                .to_string();
            let fingerprint = info
                .get_property_val_str("fingerprint")
                .unwrap_or("")
                .to_string();
            let app_ver = info
                .get_property_val_str("ver")
                .unwrap_or("0.1.0")
                .to_string();
            let name = info
                .get_property_val_str("inst")
                .unwrap_or(info.get_fullname())
                .to_string();

            let ip = info
                .get_addresses_v4()
                .iter()
                .next()
                .map(|ip| ip.to_string())
                .or_else(|| info.get_addresses().iter().next().map(|ip| ip.to_string()))
                .unwrap_or_else(|| "127.0.0.1".to_string());

            let host = DiscoveredHost {
                host_id: host_id.clone(),
                name,
                ip,
                port: info.get_port(),
                fingerprint,
                app_version: app_ver,
            };
            discovered.insert(host_id, host);
        }
    }

    let _ = mdns.shutdown();
    Ok(DiscoveredHostList {
        hosts: discovered.into_values().collect(),
    })
}

// Custom Pinned TLS Verifier for Workstation Ping
#[derive(Debug)]
struct PinnedCertVerifier {
    captured_der: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let mut lock = self.captured_der.lock().unwrap();
        *lock = Some(end_entity.as_ref().to_vec());
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub fn ping_host(payload: PingHostPayload) -> Result<HostPingResult, LanError> {
    ensure_crypto_provider_installed();
    let mgr = lan_manager();

    let target_ip = payload.ip;
    let target_port = payload.port;
    let expected_fp = payload.expected_fingerprint;

    mgr.tokio_runtime.block_on(async move {
        let start_time = Instant::now();
        let captured_der = Arc::new(Mutex::new(None));
        let verifier = Arc::new(PinnedCertVerifier {
            captured_der: captured_der.clone(),
        });

        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let stream = match tokio::net::TcpStream::connect((target_ip.as_str(), target_port)).await {
            Ok(s) => s,
            Err(e) => {
                return Ok(HostPingResult {
                    reachable: false,
                    product: None,
                    protocol_version: None,
                    host_id: None,
                    institution_name: None,
                    fingerprint_matched: false,
                    rtt_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(format!("TCP connection failed: {}", e)),
                });
            }
        };

        let server_name = match ServerName::try_from("localhost".to_string()) {
            Ok(sn) => sn,
            Err(e) => {
                return Ok(HostPingResult {
                    reachable: false,
                    product: None,
                    protocol_version: None,
                    host_id: None,
                    institution_name: None,
                    fingerprint_matched: false,
                    rtt_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(format!("ServerName error: {}", e)),
                });
            }
        };

        let mut tls_stream = match connector.connect(server_name, stream).await {
            Ok(s) => s,
            Err(e) => {
                return Ok(HostPingResult {
                    reachable: false,
                    product: None,
                    protocol_version: None,
                    host_id: None,
                    institution_name: None,
                    fingerprint_matched: false,
                    rtt_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(format!("TLS handshake failed: {}", e)),
                });
            }
        };

        let der_opt = captured_der.lock().unwrap().clone();
        let actual_fp = der_opt.as_ref().map(|d| compute_sha256_fingerprint(d));

        let fingerprint_matched = match (&expected_fp, &actual_fp) {
            (Some(exp), Some(act)) => exp.eq_ignore_ascii_case(act),
            _ => true,
        };

        let http_request = format!(
            "GET /hello HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\nAccept: application/json\r\n\r\n",
            target_ip, target_port
        );

        if let Err(e) = tls_stream.write_all(http_request.as_bytes()).await {
            return Ok(HostPingResult {
                reachable: false,
                product: None,
                protocol_version: None,
                host_id: None,
                institution_name: None,
                fingerprint_matched,
                rtt_ms: start_time.elapsed().as_millis() as u64,
                error: Some(format!("Write error: {}", e)),
            });
        }

        let mut response_bytes = Vec::new();
        let mut buf = [0u8; 1024];
        let read_deadline = Instant::now() + Duration::from_secs(3);

        while Instant::now() < read_deadline {
            let read_step = tokio::time::timeout(Duration::from_millis(500), tls_stream.read(&mut buf)).await;
            match read_step {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    response_bytes.extend_from_slice(&buf[..n]);
                    if let Some(pos) = response_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let body_slice = &response_bytes[pos + 4..];
                        if serde_json::from_slice::<HelloResponse>(body_slice).is_ok() {
                            break;
                        }
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => break,
            }
        }

        let rtt_ms = start_time.elapsed().as_millis() as u64;
        let response_str = String::from_utf8_lossy(&response_bytes);

        // Find double CRLF separating header and body
        if let Some(pos) = response_str.find("\r\n\r\n") {
            let body = &response_str[pos + 4..];
            if let Ok(hello) = serde_json::from_str::<HelloResponse>(body) {
                return Ok(HostPingResult {
                    reachable: true,
                    product: Some(hello.product),
                    protocol_version: Some(hello.protocol_version),
                    host_id: Some(hello.host_id),
                    institution_name: Some(hello.institution_name),
                    fingerprint_matched,
                    rtt_ms,
                    error: None,
                });
            }
        }

        Ok(HostPingResult {
            reachable: true,
            product: None,
            protocol_version: None,
            host_id: None,
            institution_name: None,
            fingerprint_matched,
            rtt_ms,
            error: Some("Failed to parse /hello JSON payload".to_string()),
        })
    })
}
// Custom Pinned TLS Verifier for SHA-256 Fingerprint Pinning (Spec 0046, AC-3)
#[derive(Debug)]
pub struct FingerprintPinnedCertVerifier {
    pub expected_fingerprint_hex: String,
}

impl FingerprintPinnedCertVerifier {
    pub fn new(expected_fingerprint_hex: &str) -> Self {
        let cleaned = expected_fingerprint_hex
            .trim()
            .trim_start_matches("SHA256:")
            .trim_start_matches("sha256:")
            .trim_start_matches("SHA256")
            .trim_start_matches("sha256")
            .replace(':', "")
            .to_lowercase();
        Self {
            expected_fingerprint_hex: cleaned,
        }
    }
}

impl ServerCertVerifier for FingerprintPinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let mut hasher = Sha256::new();
        hasher.update(end_entity.as_ref());
        let cert_hash = hex::encode(hasher.finalize()).to_lowercase();

        if cert_hash == self.expected_fingerprint_hex {
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub async fn race_connect_candidate_ips(
    candidate_ips: &[String],
    port: u16,
    expected_fingerprint: &str,
    timeout: Duration,
) -> Result<String, LanError> {
    ensure_crypto_provider_installed();
    if candidate_ips.is_empty() {
        return Err(LanError::HostUnreachable(
            "unknown".to_string(),
            port,
            "No candidate IPs provided".to_string(),
        ));
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut tasks = Vec::new();

    for ip in candidate_ips {
        let ip_clone = ip.clone();
        let fp_clone = expected_fingerprint.to_string();
        let tx_clone = tx.clone();

        let task = tokio::spawn(async move {
            let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&fp_clone));
            let client_config = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth();

            let tls_connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

            let connect_fut = async {
                let socket_addr: SocketAddr = format!("{}:{}", ip_clone, port)
                    .parse()
                    .map_err(|e| format!("Invalid address: {}", e))?;

                let tcp_stream = tokio::net::TcpStream::connect(socket_addr)
                    .await
                    .map_err(|e| format!("TCP connect failed: {}", e))?;

                let server_name = ServerName::try_from("coachify.lan")
                    .map_err(|e| format!("Server name invalid: {}", e))?;

                let _tls_stream = tls_connector
                    .connect(server_name, tcp_stream)
                    .await
                    .map_err(|e| format!("TLS handshake failed: {}", e))?;

                Ok::<String, String>(ip_clone)
            };

            if let Ok(Ok(winning_ip)) = tokio::time::timeout(timeout, connect_fut).await {
                let _ = tx_clone.send(winning_ip).await;
            }
        });
        tasks.push(task);
    }
    drop(tx);

    let winner = rx.recv().await;

    for task in tasks {
        task.abort();
    }

    match winner {
        Some(ip) => Ok(ip),
        None => Err(LanError::DiscoveryTimeout),
    }
}

impl MobileClientState {
    pub fn new(
        device_id: String,
        device_token: String,
        host_id: String,
        host_fingerprint: String,
        host_ip: String,
        host_port: u16,
    ) -> Self {
        Self {
            device_id,
            device_token,
            host_id,
            host_fingerprint,
            candidate_ips: vec![host_ip.clone()],
            host_ip,
            host_port,
            session_token: None,
        }
    }

    pub fn update_from_scanned_qr(&mut self, qr_uri: &str) -> Result<(), LanError> {
        let parsed = parse_qr_pairing_uri(qr_uri)?;
        let norm_expected = self
            .host_fingerprint
            .trim()
            .trim_start_matches("SHA256:")
            .trim_start_matches("sha256:")
            .replace(':', "")
            .to_lowercase();
        let norm_received = parsed
            .fingerprint
            .trim()
            .trim_start_matches("SHA256:")
            .trim_start_matches("sha256:")
            .replace(':', "")
            .to_lowercase();
        if norm_received != norm_expected {
            return Err(LanError::FingerprintMismatch {
                expected: self.host_fingerprint.clone(),
                received: parsed.fingerprint,
            });
        }
        self.candidate_ips = parsed.host_ips.clone();
        if let Some(ip) = parsed.host_ips.first() {
            self.host_ip = ip.clone();
        }
        self.host_port = parsed.port;
        Ok(())
    }

    pub async fn execute_rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LanError> {
        ensure_crypto_provider_installed();
        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&self.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let tls_connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let mut ips_to_try = vec![self.host_ip.clone()];
        for cand in &self.candidate_ips {
            if cand != &self.host_ip {
                ips_to_try.push(cand.clone());
            }
        }

        let mut last_err = None;
        for target_ip in ips_to_try {
            let socket_addr: SocketAddr = match format!("{}:{}", target_ip, self.host_port).parse()
            {
                Ok(a) => a,
                Err(e) => {
                    last_err = Some(LanError::HostUnreachable(
                        target_ip.clone(),
                        self.host_port,
                        format!("{}", e),
                    ));
                    continue;
                }
            };

            let tcp_stream = match tokio::net::TcpStream::connect(socket_addr).await {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(LanError::HostUnreachable(
                        target_ip.clone(),
                        self.host_port,
                        format!("{}", e),
                    ));
                    continue;
                }
            };

            let server_name = match ServerName::try_from("coachify.lan") {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(LanError::HostUnreachable(
                        target_ip.clone(),
                        self.host_port,
                        format!("{}", e),
                    ));
                    continue;
                }
            };

            let mut tls_stream = match tls_connector.connect(server_name, tcp_stream).await {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(LanError::HostUnreachable(
                        target_ip.clone(),
                        self.host_port,
                        format!("TLS error: {}", e),
                    ));
                    continue;
                }
            };

            let req_body = serde_json::json!({
                "method": method,
                "params": params
            })
            .to_string();

            let mut headers_str = format!(
                "POST /rpc HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Coachify-Device-Token: {}\r\nConnection: close\r\n",
                target_ip, self.host_port, req_body.len(), self.device_token
            );
            if let Some(ref st) = self.session_token {
                headers_str.push_str(&format!("X-Coachify-Session-Token: {}\r\n", st));
            }
            headers_str.push_str("\r\n");

            if let Err(e) = tls_stream.write_all(headers_str.as_bytes()).await {
                last_err = Some(LanError::HostUnreachable(
                    target_ip.clone(),
                    self.host_port,
                    e.to_string(),
                ));
                continue;
            }
            if let Err(e) = tls_stream.write_all(req_body.as_bytes()).await {
                last_err = Some(LanError::HostUnreachable(
                    target_ip.clone(),
                    self.host_port,
                    e.to_string(),
                ));
                continue;
            }

            let mut resp_bytes = Vec::new();
            if let Err(e) = tls_stream.read_to_end(&mut resp_bytes).await {
                last_err = Some(LanError::HostUnreachable(
                    target_ip.clone(),
                    self.host_port,
                    e.to_string(),
                ));
                continue;
            }

            let resp_str = String::from_utf8_lossy(&resp_bytes);
            if let Some(body_start) = resp_str.find("\r\n\r\n") {
                let body = &resp_str[body_start + 4..];
                let json_val: serde_json::Value = serde_json::from_str(body)
                    .map_err(|e| LanError::Database(format!("JSON parse error: {}", e)))?;
                return Ok(json_val);
            } else {
                return Err(LanError::Database("Malformed HTTP response".to_string()));
            }
        }

        Err(last_err.unwrap_or_else(|| {
            LanError::HostUnreachable(
                self.host_ip.clone(),
                self.host_port,
                "No reachable host IP".to_string(),
            )
        }))
    }
}

// -----------------------------------------------------------------------------
// Workstation Device Registry Management (AC-5, AC-6, AC-7, AC-8, AC-10)
// -----------------------------------------------------------------------------

pub fn list_workstations(
    conn: &Connection,
    current_utc: &str,
) -> Result<WorkstationListResponse, LanError> {
    let now_ts = jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());

    let mut stmt = conn.prepare(
        "SELECT id, name, windows_name, ip_address, os_version, app_version,
                status, is_host, paired_at, last_seen_at, blocked_at, revoked_at, revocation_reason,
                COALESCE(paired_via, 'code'), platform, device_type
         FROM devices
         ORDER BY is_host DESC, paired_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i32>(7)? != 0,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, Option<String>>(10)?,
            row.get::<_, Option<String>>(11)?,
            row.get::<_, Option<String>>(12)?,
            row.get::<_, String>(13)?,
            row.get::<_, Option<String>>(14)?,
            row.get::<_, Option<String>>(15)?,
        ))
    })?;

    let mut devices = Vec::new();
    let mut active_workstations = 0u32;

    for item in rows {
        let (
            id,
            name,
            windows_name,
            ip_address,
            os_version,
            app_version,
            status,
            is_host,
            paired_at,
            last_seen_at,
            blocked_at,
            revoked_at,
            revocation_reason,
            paired_via,
            platform,
            device_type,
        ) = item?;

        if !is_host && status == "active" {
            active_workstations += 1;
        }

        let is_online = if is_host {
            lan_manager().is_running()
        } else if let Ok(seen_ts) = jiff::Timestamp::from_str(&last_seen_at) {
            let diff = now_ts.duration_since(seen_ts);
            diff.as_secs().abs() <= 30
        } else {
            false
        };

        let active_sessions_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE device_id = ?1 AND revoked_at IS NULL AND expires_at > ?2",
                params![id, current_utc],
                |r| r.get(0),
            )
            .unwrap_or(0);

        devices.push(WorkstationSummary {
            id,
            name,
            windows_name,
            ip_address,
            os_version,
            app_version,
            status,
            is_host,
            paired_at,
            last_seen_at,
            blocked_at,
            revoked_at,
            revocation_reason,
            is_online,
            active_sessions_count,
            paired_via,
            platform,
            device_type,
        });
    }

    let license = crate::licensing::get_license_status(conn, current_utc).ok();
    let limit = license.as_ref().and_then(|l| l.workstation_limit);
    let tier = license
        .as_ref()
        .map(|l| l.tier.clone())
        .unwrap_or_else(|| "trial".to_string());
    let seats_remaining = limit.map(|l| l.saturating_sub(active_workstations));

    let pairing_open: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'pending' AND expires_at > ?1",
            params![current_utc],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    Ok(WorkstationListResponse {
        devices,
        quota: WorkstationQuota {
            active_workstations,
            workstation_limit: limit,
            seats_remaining,
            license_tier: tier,
        },
        pairing_open,
    })
}

pub fn get_workstation(
    conn: &Connection,
    device_id: &str,
    current_utc: &str,
) -> Result<WorkstationDetail, LanError> {
    let now_ts = jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());

    let maybe_device = conn
        .query_row(
            "SELECT id, name, windows_name, ip_address, os_version, app_version,
                    status, is_host, paired_at, last_seen_at, blocked_at, revoked_at, revocation_reason,
                    COALESCE(paired_via, 'code'), platform, device_type
             FROM devices WHERE id = ?1",
            params![device_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i32>(7)? != 0,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, Option<String>>(15)?,
                ))
            },
        )
        .optional()?;

    let (
        id,
        name,
        windows_name,
        ip_address,
        os_version,
        app_version,
        status,
        is_host,
        paired_at,
        last_seen_at,
        blocked_at,
        revoked_at,
        revocation_reason,
        paired_via,
        platform,
        device_type,
    ) = match maybe_device {
        Some(d) => d,
        None => return Err(LanError::DeviceNotFound),
    };

    let is_online = if is_host {
        lan_manager().is_running()
    } else if let Ok(seen_ts) = jiff::Timestamp::from_str(&last_seen_at) {
        let diff = now_ts.duration_since(seen_ts);
        diff.as_secs().abs() <= 30
    } else {
        false
    };

    let active_sessions: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE device_id = ?1 AND revoked_at IS NULL AND expires_at > ?2",
            params![device_id, current_utc],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut event_stmt = conn.prepare(
        "SELECT de.id, de.device_id, de.event_type, de.actor_user_id, u.display_name,
                de.remote_ip, de.details_json, de.created_at
         FROM device_events de
         LEFT JOIN users u ON de.actor_user_id = u.id
         WHERE de.device_id = ?1
         ORDER BY de.created_at DESC
         LIMIT 50",
    )?;

    let events = event_stmt
        .query_map(params![device_id], |row| {
            Ok(DeviceEventItem {
                id: row.get(0)?,
                device_id: row.get(1)?,
                event_type: row.get(2)?,
                actor_user_id: row.get(3)?,
                actor_user_name: row.get(4)?,
                remote_ip: row.get(5)?,
                details_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WorkstationDetail {
        device: WorkstationSummary {
            id,
            name,
            windows_name,
            ip_address,
            os_version,
            app_version,
            status,
            is_host,
            paired_at,
            last_seen_at,
            blocked_at,
            revoked_at,
            revocation_reason,
            is_online,
            active_sessions_count: active_sessions,
            paired_via,
            platform,
            device_type,
        },
        recent_events: events,
        active_sessions,
    })
}

pub fn list_device_events(
    conn: &Connection,
    device_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<DeviceEventItem>, LanError> {
    let limit_val = limit.unwrap_or(100).min(500);
    if let Some(id) = device_id {
        let mut stmt = conn.prepare(
            "SELECT de.id, de.device_id, de.event_type, de.actor_user_id, u.display_name,
                    de.remote_ip, de.details_json, de.created_at
             FROM device_events de
             LEFT JOIN users u ON de.actor_user_id = u.id
             WHERE de.device_id = ?1
             ORDER BY de.created_at DESC
             LIMIT ?2",
        )?;
        let items = stmt
            .query_map(params![id, limit_val], |row| {
                Ok(DeviceEventItem {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    event_type: row.get(2)?,
                    actor_user_id: row.get(3)?,
                    actor_user_name: row.get(4)?,
                    remote_ip: row.get(5)?,
                    details_json: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    } else {
        let mut stmt = conn.prepare(
            "SELECT de.id, de.device_id, de.event_type, de.actor_user_id, u.display_name,
                    de.remote_ip, de.details_json, de.created_at
             FROM device_events de
             LEFT JOIN users u ON de.actor_user_id = u.id
             ORDER BY de.created_at DESC
             LIMIT ?1",
        )?;
        let items = stmt
            .query_map(params![limit_val], |row| {
                Ok(DeviceEventItem {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    event_type: row.get(2)?,
                    actor_user_id: row.get(3)?,
                    actor_user_name: row.get(4)?,
                    remote_ip: row.get(5)?,
                    details_json: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }
}

pub fn rename_workstation(
    conn: &Connection,
    device_id: &str,
    new_name: &str,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() || trimmed.len() > 60 {
        return Err(LanError::InvalidDeviceName);
    }

    let dev: Option<(String, bool, String)> = conn
        .query_row(
            "SELECT name, is_host, status FROM devices WHERE id = ?1",
            params![device_id],
            |r| Ok((r.get(0)?, r.get::<_, i32>(1)? != 0, r.get(2)?)),
        )
        .optional()?;

    let (old_name, is_host, status) = match dev {
        Some(d) => d,
        None => return Err(LanError::DeviceNotFound),
    };

    if is_host {
        return Err(LanError::CannotRenameHost);
    }
    if status == "revoked" {
        return Err(LanError::DeviceAlreadyRevoked);
    }

    conn.execute(
        "UPDATE devices SET name = ?1 WHERE id = ?2",
        params![trimmed, device_id],
    )?;

    let details = serde_json::json!({
        "old_name": old_name,
        "new_name": trimmed,
    })
    .to_string();

    let event_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
         VALUES (?1, ?2, 'renamed', ?3, NULL, ?4, ?5)",
        params![event_id, device_id, actor_user_id, details, current_utc],
    )?;

    let detail = get_workstation(conn, device_id, current_utc)?;
    Ok(detail.device)
}

pub fn block_workstation(
    conn: &Connection,
    device_id: &str,
    reason: Option<&str>,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    let dev: Option<(bool, String)> = conn
        .query_row(
            "SELECT is_host, status FROM devices WHERE id = ?1",
            params![device_id],
            |r| Ok((r.get::<_, i32>(0)? != 0, r.get(1)?)),
        )
        .optional()?;

    let (is_host, status) = match dev {
        Some(d) => d,
        None => return Err(LanError::DeviceNotFound),
    };

    if is_host {
        return Err(LanError::CannotBlockHost);
    }
    if status == "blocked" {
        return Err(LanError::DeviceAlreadyBlocked);
    }
    if status == "revoked" {
        return Err(LanError::DeviceAlreadyRevoked);
    }

    conn.execute(
        "UPDATE devices SET status = 'blocked', blocked_at = ?1 WHERE id = ?2",
        params![current_utc, device_id],
    )?;

    // Purge active user sessions immediately
    conn.execute(
        "UPDATE sessions SET revoked_at = ?1 WHERE device_id = ?2 AND revoked_at IS NULL",
        params![current_utc, device_id],
    )?;

    let details = serde_json::json!({
        "reason": reason.unwrap_or("Administrative suspension"),
    })
    .to_string();

    let event_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
         VALUES (?1, ?2, 'blocked', ?3, NULL, ?4, ?5)",
        params![event_id, device_id, actor_user_id, details, current_utc],
    )?;

    broadcast_control_event(DeviceControlEvent::Blocked {
        device_id: device_id.to_string(),
        reason: reason.unwrap_or("Administrative suspension").to_string(),
    });

    let detail = get_workstation(conn, device_id, current_utc)?;
    Ok(detail.device)
}

pub fn unblock_workstation(
    conn: &Connection,
    device_id: &str,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    let dev: Option<(bool, String)> = conn
        .query_row(
            "SELECT is_host, status FROM devices WHERE id = ?1",
            params![device_id],
            |r| Ok((r.get::<_, i32>(0)? != 0, r.get(1)?)),
        )
        .optional()?;

    let (is_host, status) = match dev {
        Some(d) => d,
        None => return Err(LanError::DeviceNotFound),
    };

    if is_host {
        return Err(LanError::CannotBlockHost);
    }
    if status != "blocked" {
        return Err(LanError::DeviceNotBlocked);
    }

    // Check workstation license limit
    let active_workstations: u32 = conn.query_row(
        "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
        [],
        |r| r.get(0),
    )?;
    let license = crate::licensing::get_license_status(conn, current_utc).ok();
    let limit = license
        .as_ref()
        .and_then(|l| l.workstation_limit)
        .unwrap_or(1);
    if active_workstations >= limit {
        return Err(LanError::WorkstationLimitReached);
    }

    conn.execute(
        "UPDATE devices SET status = 'active', blocked_at = NULL WHERE id = ?1",
        params![device_id],
    )?;

    let event_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
         VALUES (?1, ?2, 'unblocked', ?3, NULL, '{}', ?4)",
        params![event_id, device_id, actor_user_id, current_utc],
    )?;

    let detail = get_workstation(conn, device_id, current_utc)?;
    Ok(detail.device)
}

pub fn revoke_workstation(
    conn: &Connection,
    device_id: &str,
    reason: Option<&str>,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    let dev: Option<(bool, String)> = conn
        .query_row(
            "SELECT is_host, status FROM devices WHERE id = ?1",
            params![device_id],
            |r| Ok((r.get::<_, i32>(0)? != 0, r.get(1)?)),
        )
        .optional()?;

    let (is_host, status) = match dev {
        Some(d) => d,
        None => return Err(LanError::DeviceNotFound),
    };

    if is_host {
        return Err(LanError::CannotRevokeHost);
    }
    if status == "revoked" {
        return Err(LanError::DeviceAlreadyRevoked);
    }

    let tombstone_hash = format!("revoked:{}", uuid::Uuid::now_v7());

    conn.execute(
        "UPDATE devices
         SET status = 'revoked',
             revoked_at = ?1,
             revocation_reason = ?2,
             token_hash = ?3
         WHERE id = ?4",
        params![
            current_utc,
            reason.unwrap_or("Permanently decommissioned"),
            tombstone_hash,
            device_id
        ],
    )?;

    // Purge active user sessions immediately
    conn.execute(
        "UPDATE sessions SET revoked_at = ?1 WHERE device_id = ?2 AND revoked_at IS NULL",
        params![current_utc, device_id],
    )?;

    let details = serde_json::json!({
        "reason": reason.unwrap_or("Permanently decommissioned"),
    })
    .to_string();

    let event_id = uuid::Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO device_events (id, device_id, event_type, actor_user_id, remote_ip, details_json, created_at)
         VALUES (?1, ?2, 'revoked', ?3, NULL, ?4, ?5)",
        params![event_id, device_id, actor_user_id, details, current_utc],
    )?;

    broadcast_control_event(DeviceControlEvent::Revoked {
        device_id: device_id.to_string(),
        reason: reason.unwrap_or("Permanently decommissioned").to_string(),
    });

    let detail = get_workstation(conn, device_id, current_utc)?;
    Ok(detail.device)
}

// -----------------------------------------------------------------------------
// Workstation Windows DPAPI Credential Custody (AC-3)
// -----------------------------------------------------------------------------

pub fn resolve_workstation_credentials_path() -> PathBuf {
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let dir = PathBuf::from(local_app).join("Coachify");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("workstation_credentials.dat");
    }
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(home).join(".coachify");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("workstation_credentials.dat")
}

pub fn save_workstation_credentials(creds: &WorkstationCredentials) -> Result<(), LanError> {
    let json_bytes = serde_json::to_vec(creds).map_err(|e| LanError::Dpapi(e.to_string()))?;
    let encrypted = crate::crypto::wrap_dpapi_machine(&json_bytes)
        .map_err(|e| LanError::Dpapi(e.to_string()))?;
    let path = resolve_workstation_credentials_path();
    std::fs::write(&path, encrypted).map_err(|e| LanError::Dpapi(e.to_string()))?;
    Ok(())
}

pub fn load_workstation_credentials() -> Result<Option<WorkstationCredentials>, LanError> {
    let path = resolve_workstation_credentials_path();
    if !path.exists() {
        return Ok(None);
    }
    let encrypted = std::fs::read(&path).map_err(|e| LanError::Dpapi(e.to_string()))?;
    let decrypted = crate::crypto::unwrap_dpapi_machine(&encrypted)
        .map_err(|e| LanError::Dpapi(e.to_string()))?;
    let creds: WorkstationCredentials =
        serde_json::from_slice(&decrypted).map_err(|e| LanError::Dpapi(e.to_string()))?;
    Ok(Some(creds))
}

pub fn clear_workstation_credentials() -> Result<(), LanError> {
    let path = resolve_workstation_credentials_path();
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// Workstation Client Pairing Handshake Routine (AC-2, AC-3)
// -----------------------------------------------------------------------------

pub fn pair_with_host(payload: ClientPairPayload) -> Result<PairSuccessResponse, LanError> {
    ensure_crypto_provider_installed();
    let mgr = lan_manager();

    let target_ip = payload.host_ip;
    let target_port = payload.host_port;
    let expected_fp = payload.expected_fingerprint;
    let pairing_code = payload.pairing_code;
    let workstation_name = payload.workstation_name;

    mgr.tokio_runtime.block_on(async move {
        let captured_der = Arc::new(Mutex::new(None));
        let verifier = Arc::new(PinnedCertVerifier {
            captured_der: captured_der.clone(),
        });

        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let stream = match tokio::net::TcpStream::connect((target_ip.as_str(), target_port)).await {
            Ok(s) => s,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    target_ip.clone(),
                    target_port,
                    format!("Connection failed: {}", e),
                ));
            }
        };

        let server_name = match ServerName::try_from("localhost".to_string()) {
            Ok(sn) => sn,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    target_ip.clone(),
                    target_port,
                    format!("ServerName error: {}", e),
                ));
            }
        };

        let mut tls_stream = match connector.connect(server_name, stream).await {
            Ok(s) => s,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    target_ip.clone(),
                    target_port,
                    format!("TLS handshake failed: {}", e),
                ));
            }
        };

        let der_opt = captured_der.lock().unwrap().clone();
        let actual_fp = der_opt.as_ref().map(|d| compute_sha256_fingerprint(d));

        if let (Some(exp), Some(act)) = (&expected_fp, &actual_fp) {
            if !exp.eq_ignore_ascii_case(act) {
                return Err(LanError::FingerprintMismatch {
                    expected: exp.clone(),
                    received: act.clone(),
                });
            }
        }

        let hostname = whoami_hostname();
        let pair_req = PairRequest {
            pairing_code,
            workstation_name: workstation_name.clone(),
            windows_name: hostname,
            os_version: Some("Windows".to_string()),
            app_version: Some("0.1.0".to_string()),
            client_nonce: Some(uuid::Uuid::now_v7().to_string()),
        };

        let req_json = serde_json::to_string(&pair_req)
            .map_err(|e| LanError::Database(e.to_string()))?;

        let http_request = format!(
            "POST /pair HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            target_ip,
            target_port,
            req_json.len(),
            req_json
        );

        tls_stream
            .write_all(http_request.as_bytes())
            .await
            .map_err(|e| {
                LanError::HostUnreachable(
                    target_ip.clone(),
                    target_port,
                    format!("Write error: {}", e),
                )
            })?;

        let mut response_bytes = Vec::new();
        let mut buf = [0u8; 1024];
        let read_deadline = Instant::now() + Duration::from_secs(5);

        while Instant::now() < read_deadline {
            let read_step =
                tokio::time::timeout(Duration::from_millis(500), tls_stream.read(&mut buf)).await;
            match read_step {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    response_bytes.extend_from_slice(&buf[..n]);
                    if let Some(pos) = response_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let header_part = String::from_utf8_lossy(&response_bytes[..pos]);
                        if let Some(len_line) = header_part.lines().find(|l| {
                            l.to_ascii_lowercase().starts_with("content-length:")
                        }) {
                            if let Ok(expected_len) = len_line[15..].trim().parse::<usize>() {
                                if response_bytes.len() >= pos + 4 + expected_len {
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => break,
            }
        }

        let response_str = String::from_utf8_lossy(&response_bytes);

        // Check HTTP status line
        let first_line = response_str.lines().next().unwrap_or("");
        if first_line.contains("423") {
            return Err(LanError::PairingLocked);
        }
        if first_line.contains("403") {
            return Err(LanError::WorkstationLimitReached);
        }
        if first_line.contains("401") {
            return Err(LanError::PairingCodeInvalid);
        }
        if !first_line.contains("200") {
            return Err(LanError::HostUnreachable(
                target_ip,
                target_port,
                format!("Pairing rejected by host: {}", first_line),
            ));
        }

        if let Some(pos) = response_str.find("\r\n\r\n") {
            let body = &response_str[pos + 4..];
            if let Ok(pair_res) = serde_json::from_str::<PairSuccessResponse>(body) {
                // Securely save credentials to DPAPI
                let creds = WorkstationCredentials {
                    device_id: pair_res.device_id.clone(),
                    device_token: pair_res.device_token.clone(),
                    host_id: pair_res.host_device_id.clone(),
                    host_fingerprint: pair_res.host_fingerprint.clone(),
                    host_ip: target_ip,
                    host_port: target_port,
                    paired_at: jiff::Timestamp::now().to_string(),
                };
                let _ = save_workstation_credentials(&creds);

                return Ok(pair_res);
            }
        }

        Err(LanError::HostUnreachable(
            target_ip,
            target_port,
            "Failed to parse /pair response from host".to_string(),
        ))
    })
}

fn whoami_hostname() -> String {
    get_system_hostname()
}
