use crate::accounts;
use crate::db;
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
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
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
}

// -----------------------------------------------------------------------------
// Axum HTTP Handlers (/hello and /health)
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

    let body = HelloResponse {
        product: "coachify".to_string(),
        protocol_version: 1,
        app_version: ctx.app_version,
        host_id: ctx.host_id,
        institution_name: ctx.institution_name,
        pairing_open: false,
        tls_fingerprint_sha256: ctx.fingerprint,
    };

    Json(body).into_response()
}

async fn health_handler(State(ctx): State<LanServerContext>) -> Response {
    let mut db_status = "ok".to_string();
    if let Some(ref path) = ctx.db_path {
        if let Ok(conn) = rusqlite::Connection::open(path) {
            let res: Result<i32, _> = conn.query_row("SELECT 1", [], |r| r.get(0));
            if res.is_err() {
                db_status = "degraded".to_string();
            }
        } else {
            db_status = "degraded".to_string();
        }
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
        active_workstations_count: 0,
        bound_port: ctx.port,
        server_time_utc: now_utc,
    };

    Json(body).into_response()
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

    Ok(LanHostStatus {
        is_running,
        port: config.port,
        bind_address: config.bind_address,
        bound_addresses,
        fingerprint: config.tls_fingerprint_sha256,
        active_workstations: 0,
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
