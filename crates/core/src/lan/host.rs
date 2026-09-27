use super::crypto::{
    compute_sha256_fingerprint, ensure_crypto_provider_installed, generate_self_signed_cert,
};
use super::mdns::MdnsBroadcaster;
use super::power::set_sleep_inhibition;
use super::protocol::*;
use crate::accounts;
use crate::db;
use crate::dispatcher::{ChangeEvent, RpcError};
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
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

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

#[derive(Clone)]
pub struct HostServerState {
    pub host_id: String,
    pub institution_name: String,
    pub app_version: String,
    pub fingerprint: String,
    pub db_path: Option<PathBuf>,
    pub db_conn: Option<Arc<Mutex<Connection>>>,
    pub started_at: Instant,
    pub port: u16,
    pub rate_limiter: RateLimiter,
    pub control_sender: broadcast::Sender<DeviceControlEvent>,
}

impl HostServerState {
    pub fn with_db<T, F: FnOnce(&mut Connection) -> Result<T, LanError>>(
        &self,
        f: F,
    ) -> Result<T, LanError> {
        if let Some(ref test_conn) = self.db_conn {
            let mut conn = test_conn.lock().unwrap();
            return f(&mut conn);
        }
        if let Some(ref path) = self.db_path {
            let mut conn = Connection::open(path).map_err(|e| LanError::Database(e.to_string()))?;
            return f(&mut conn);
        }
        if let Ok(path) = db::resolve_active_db_path() {
            if path.exists() {
                let mut conn =
                    Connection::open(path).map_err(|e| LanError::Database(e.to_string()))?;
                return f(&mut conn);
            }
        }
        Err(LanError::Database(
            "No database connection available for LAN server".to_string(),
        ))
    }
}

pub struct ActiveLanServer {
    pub port: u16,
    pub bind_address: String,
    pub handle: axum_server::Handle<SocketAddr>,
    pub started_at: Instant,
    pub mdns_broadcaster: Option<MdnsBroadcaster>,
}

pub struct HostController {
    active_server: Mutex<Option<ActiveLanServer>>,
    test_db_conn: Mutex<Option<Arc<Mutex<Connection>>>>,
    control_sender: broadcast::Sender<DeviceControlEvent>,
    tokio_runtime: tokio::runtime::Runtime,
    firewall_cache: RwLock<Option<(Instant, FirewallGuidance)>>,
}

impl Default for HostController {
    fn default() -> Self {
        Self::new()
    }
}

impl HostController {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("coachify-host-worker")
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime for HostController");
        let (tx, _) = broadcast::channel(256);
        Self {
            active_server: Mutex::new(None),
            test_db_conn: Mutex::new(None),
            control_sender: tx,
            tokio_runtime: rt,
            firewall_cache: RwLock::new(None),
        }
    }

    pub fn set_test_db_conn(&self, conn: Option<Arc<Mutex<Connection>>>) {
        let mut lock = self.test_db_conn.lock().unwrap();
        *lock = conn;
    }

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

    pub fn control_sender(&self) -> broadcast::Sender<DeviceControlEvent> {
        self.control_sender.clone()
    }

    pub fn broadcast_change_events(&self, events: Vec<ChangeEvent>) {
        if !events.is_empty() {
            let _ = self
                .control_sender
                .send(DeviceControlEvent::Change { events });
        }
    }

    pub fn broadcast_control_event(&self, event: DeviceControlEvent) {
        let _ = self.control_sender.send(event);
    }

    pub fn build_router(&self, state: HostServerState) -> Router {
        Router::new()
            .route("/hello", get(hello_handler))
            .route("/health", get(health_handler))
            .route("/pair", post(pair_handler))
            .route("/pair/qr", post(pair_qr_handler))
            .route("/auth/staff-login", post(staff_login_handler))
            .route("/rpc", post(rpc_handler))
            .route("/print/host-receipt", post(host_receipt_print_handler))
            .route("/events", get(events_handler))
            .with_state(state)
    }

    pub fn get_or_create_host_config(
        &self,
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

    pub fn start(
        &self,
        conn: &Connection,
        port_override: Option<u16>,
        bind_override: Option<String>,
        current_utc: &str,
    ) -> Result<LanHostStatus, LanError> {
        ensure_crypto_provider_installed();

        if let Some(p) = self.running_port() {
            return Err(LanError::HostAlreadyRunning(p));
        }

        let mut config = self.get_or_create_host_config(conn, current_utc)?;

        let target_port = port_override.unwrap_or(config.port);
        if !(1024..=65535).contains(&target_port) {
            return Err(LanError::InvalidPortRange(target_port));
        }

        let target_bind = bind_override.unwrap_or_else(|| config.bind_address.clone());

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
        let db_conn = self.test_db_conn.lock().unwrap().clone();

        let cert_pem = config.tls_cert_pem.clone();
        let key_pem = config.tls_key_pem.clone();
        let fingerprint = config.tls_fingerprint_sha256.clone();

        let tls_config = self.tokio_runtime.block_on(async {
            RustlsConfig::from_pem(cert_pem.as_bytes().to_vec(), key_pem.as_bytes().to_vec())
                .await
                .map_err(|e| LanError::CertGenerationFailed(e.to_string()))
        })?;

        let rate_limiter = RateLimiter::new();
        let server_state = HostServerState {
            host_id: host_dev.id.clone(),
            institution_name: institution_name.clone(),
            app_version: "0.1.0".to_string(),
            fingerprint: fingerprint.clone(),
            db_path,
            db_conn,
            started_at: Instant::now(),
            port: target_port,
            rate_limiter,
            control_sender: self.control_sender.clone(),
        };

        let app = self.build_router(server_state);

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();

        self.tokio_runtime.spawn(async move {
            let server = axum_server::bind_rustls(test_addr, tls_config).handle(handle_clone);
            if let Err(err) = server
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await
            {
                eprintln!("LAN host axum server terminated: {err}");
            }
        });

        let listening_addr = self.tokio_runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(500), handle.listening()).await
        });

        let bound_port = match listening_addr {
            Ok(Some(addr)) => addr.port(),
            _ => target_port,
        };

        let mut mdns_broadcaster = None;
        if config.mdns_enabled {
            if let Ok(b) = MdnsBroadcaster::new(
                &institution_name,
                &host_dev.windows_name,
                &host_dev.id,
                &fingerprint,
                bound_port,
                false,
            ) {
                mdns_broadcaster = Some(b);
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

        let mut lock = self.active_server.lock().unwrap();
        *lock = Some(ActiveLanServer {
            port: bound_port,
            bind_address: target_bind,
            handle,
            started_at: Instant::now(),
            mdns_broadcaster,
        });
        drop(lock);

        self.get_host_status(conn, current_utc)
    }

    pub fn stop(&self, conn: &Connection, current_utc: &str) -> Result<LanHostStatus, LanError> {
        let mut lock = self.active_server.lock().unwrap();
        let active = match lock.take() {
            Some(s) => s,
            None => return Err(LanError::HostNotRunning),
        };

        if let Some(mdns) = active.mdns_broadcaster {
            mdns.shutdown();
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
        self.get_host_status(conn, current_utc)
    }

    pub fn get_host_status(
        &self,
        conn: &Connection,
        current_utc: &str,
    ) -> Result<LanHostStatus, LanError> {
        let config = self.get_or_create_host_config(conn, current_utc)?;
        let host_dev = accounts::get_or_create_host_device(conn, current_utc)
            .map_err(|e| LanError::Database(e.to_string()))?;

        let active_count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM devices WHERE is_host = 0 AND status = 'active'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let lock = self.active_server.lock().unwrap();
        let is_running = lock.is_some();
        let uptime = lock
            .as_ref()
            .map(|s| s.started_at.elapsed().as_secs())
            .unwrap_or(0);
        let port = lock.as_ref().map(|s| s.port).unwrap_or(config.port);
        let bind_address = lock
            .as_ref()
            .map(|s| s.bind_address.clone())
            .unwrap_or_else(|| config.bind_address.clone());
        drop(lock);

        let bound_addresses = if bind_address == "0.0.0.0" {
            get_local_ip_addresses()
        } else {
            vec![bind_address.clone()]
        };

        let license = crate::licensing::get_license_status(conn, current_utc).ok();
        let license_entitled = license
            .as_ref()
            .map(|l| {
                l.tier == "trial"
                    || l.tier == "local_pro"
                    || l.capabilities
                        .iter()
                        .any(|c| c == "lan_multi_device" || c == "lan" || c == "lan_host")
            })
            .unwrap_or(true);

        let guidance = self.get_firewall_guidance();

        Ok(LanHostStatus {
            is_running,
            port,
            bind_address,
            bound_addresses,
            fingerprint: config.tls_fingerprint_sha256,
            active_workstations: active_count,
            network_profile: guidance.profile,
            firewall_status: if guidance.rule_exists {
                "allowed".to_string()
            } else {
                "not_configured".to_string()
            },
            mdns_broadcasting: config.mdns_enabled && is_running,
            license_entitled,
            prevent_sleep: config.prevent_sleep,
            auto_start_on_boot: config.auto_start_on_boot,
            uptime_seconds: uptime,
            host_device_id: host_dev.id,
            hostname: host_dev.windows_name,
        })
    }

    pub fn update_host_config(
        &self,
        conn: &Connection,
        payload: UpdateHostConfigPayload,
        current_utc: &str,
    ) -> Result<LanHostStatus, LanError> {
        let current_config = self.get_or_create_host_config(conn, current_utc)?;

        if let Some(port) = payload.port {
            if !(1024..=65535).contains(&port) {
                return Err(LanError::InvalidPortRange(port));
            }
            if self.is_running() && port != current_config.port {
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

        if self.is_running() {
            set_sleep_inhibition(prevent_sleep);
        }

        self.get_host_status(conn, current_utc)
    }

    pub fn get_firewall_guidance(&self) -> FirewallGuidance {
        let powershell_command = "Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } | Set-NetConnectionProfile -NetworkCategory Private".to_string();
        let netsh_command = "netsh advfirewall firewall add rule name=\"Coachify LAN Host\" dir=in action=allow protocol=TCP localport=50443 profile=private".to_string();

        if let Ok(guard) = self.firewall_cache.read() {
            if let Some((instant, ref guidance)) = *guard {
                if instant.elapsed() < Duration::from_secs(30) {
                    return guidance.clone();
                }
            }
        }

        #[cfg(windows)]
        let (profile, is_private, rule_exists) = {
            let mut p = "Private".to_string();
            let mut priv_flag = true;
            if let Ok(output) = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "(Get-NetConnectionProfile).NetworkCategory",
                ])
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if text.contains("Public") {
                    p = "Public".to_string();
                    priv_flag = false;
                }
            }

            let mut re = false;
            if let Ok(output) = std::process::Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "show",
                    "rule",
                    "name=Coachify LAN Host",
                ])
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                if text.contains("Rule Name:") || text.contains("Enabled: Yes") {
                    re = true;
                }
            }
            (p, priv_flag, re)
        };

        #[cfg(not(windows))]
        let (profile, is_private, rule_exists) = ("Private".to_string(), true, true);

        let guidance = FirewallGuidance {
            profile,
            is_private,
            rule_exists,
            powershell_command,
            netsh_command,
        };

        if let Ok(mut guard) = self.firewall_cache.write() {
            *guard = Some((Instant::now(), guidance.clone()));
        }

        guidance
    }

    pub fn open_pairing_window(
        &self,
        conn: &Connection,
        duration_seconds: Option<u64>,
        actor_user_id: &str,
        current_utc: &str,
    ) -> Result<PairingWindowInfo, LanError> {
        if !self.is_running() {
            return Err(LanError::HostNotRunning);
        }

        let locked_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
            params![current_utc],
            |r| r.get(0),
        )?;
        if locked_count > 0 {
            return Err(LanError::PairingCooldownActive);
        }

        let pending_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'pending' AND expires_at > ?1",
            params![current_utc],
            |r| r.get(0),
        )?;
        if pending_count > 0 {
            return Err(LanError::PairingAlreadyOpen);
        }

        crate::entitlement::guard_workstation_seat(conn, current_utc)?;

        conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'expired', updated_at = ?1 WHERE status = 'pending' AND expires_at <= ?1",
            params![current_utc],
        )?;

        let dur = duration_seconds.unwrap_or(600).clamp(60, 3600);
        let (code, salt, code_hash) = generate_pairing_code();
        let session_id = uuid::Uuid::now_v7().to_string();

        let now_ts =
            jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());
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

        let config = self.get_or_create_host_config(conn, current_utc)?;
        self.update_mdns_pairing_status(conn, current_utc, true);

        Ok(PairingWindowInfo {
            session_id,
            code,
            expires_at: expires_ts,
            expires_in_seconds: dur,
            host_fingerprint: config.tls_fingerprint_sha256,
        })
    }

    pub fn close_pairing_window(
        &self,
        conn: &Connection,
        current_utc: &str,
    ) -> Result<(), LanError> {
        let rows_affected = conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'cancelled', updated_at = ?1 WHERE status = 'pending'",
            params![current_utc],
        )?;

        if rows_affected == 0 {
            return Err(LanError::PairingNotOpen);
        }

        self.update_mdns_pairing_status(conn, current_utc, false);
        Ok(())
    }

    pub fn open_mobile_pairing_window(
        &self,
        conn: &Connection,
        duration_seconds: Option<u64>,
        actor_user_id: &str,
        current_utc: &str,
    ) -> Result<MobilePairingWindowInfo, LanError> {
        if !self.is_running() {
            return Err(LanError::HostNotRunning);
        }

        let locked_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
            params![current_utc],
            |r| r.get(0),
        )?;
        if locked_count > 0 {
            return Err(LanError::PairingCooldownActive);
        }

        conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'cancelled', updated_at = ?1 WHERE status = 'pending'",
            params![current_utc],
        )?;

        let dur = duration_seconds.unwrap_or(600).clamp(60, 3600);
        let (code, salt, code_hash) = generate_pairing_code();
        let (qr_secret, qr_secret_hash) = generate_qr_secret();
        let session_id = uuid::Uuid::now_v7().to_string();

        let now_ts =
            jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());
        let expires_ts = now_ts
            .checked_add(jiff::Span::new().seconds(dur as i64))
            .map(|t| t.to_string())
            .unwrap_or_else(|_| current_utc.to_string());

        let expires_epoch = now_ts
            .checked_add(jiff::Span::new().seconds(dur as i64))
            .map(|t| t.as_second())
            .unwrap_or_else(|_| now_ts.as_second() + dur as i64);

        conn.execute(
            "INSERT INTO lan_pairing_sessions (
                id, code_hash, salt, expires_at, failed_attempts, status,
                created_by_user_id, paired_device_id, created_at, updated_at,
                qr_secret_hash
            ) VALUES (
                ?1, ?2, ?3, ?4, 0, 'pending',
                ?5, NULL, ?6, ?6, ?7
            )",
            params![
                session_id,
                code_hash,
                salt,
                expires_ts,
                actor_user_id,
                current_utc,
                qr_secret_hash,
            ],
        )?;

        let config = self.get_or_create_host_config(conn, current_utc)?;
        self.update_mdns_pairing_status(conn, current_utc, true);

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
            .query_row(
                "SELECT value FROM system_config WHERE key = 'institution_name'",
                [],
                |r| r.get(0),
            )
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

    pub fn update_mdns_pairing_status(
        &self,
        conn: &Connection,
        current_utc: &str,
        pairing_open: bool,
    ) {
        let mut lock = self.active_server.lock().unwrap();
        if let Some(ref mut server) = *lock {
            if let Some(ref mut broadcaster) = server.mdns_broadcaster {
                let host_dev = accounts::get_or_create_host_device(conn, current_utc).ok();
                let inst: String = conn
                    .query_row(
                        "SELECT value FROM system_config WHERE key = 'institution_name'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| "Coachify Coaching Center".to_string());
                if let Some(dev) = host_dev {
                    if let Ok(config) = self.get_or_create_host_config(conn, current_utc) {
                        broadcaster.update_pairing_status(
                            &inst,
                            &dev.windows_name,
                            &dev.id,
                            &config.tls_fingerprint_sha256,
                            server.port,
                            pairing_open,
                        );
                    }
                }
            }
        }
    }

    pub fn list_workstations(
        &self,
        conn: &Connection,
        current_utc: &str,
    ) -> Result<WorkstationListResponse, LanError> {
        let mut stmt = conn.prepare(
            "SELECT id, name, windows_name, ip_address, os_version, app_version,
                    status, is_host, paired_at, last_seen_at, blocked_at, revoked_at,
                    revocation_reason, paired_via, platform, device_type
             FROM devices
             ORDER BY is_host DESC, paired_at ASC",
        )?;

        let now_ts =
            jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());

        let devices = stmt
            .query_map([], |row| {
                let is_host: bool = row.get::<_, i32>(7)? != 0;
                let last_seen_str: String = row.get(9)?;
                let status_str: String = row.get(6)?;

                let is_online = if status_str == "active" {
                    if is_host {
                        true
                    } else if let Ok(seen_ts) = jiff::Timestamp::from_str(&last_seen_str) {
                        now_ts.duration_since(seen_ts).as_secs() <= 30
                    } else {
                        false
                    }
                } else {
                    false
                };

                let dev_id: String = row.get(0)?;
                let active_sessions: u32 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sessions WHERE device_id = ?1 AND revoked_at IS NULL AND expires_at > ?2",
                        params![dev_id, current_utc],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);

                Ok(WorkstationSummary {
                    id: dev_id,
                    name: row.get(1)?,
                    windows_name: row.get(2)?,
                    ip_address: row.get(3)?,
                    os_version: row.get(4)?,
                    app_version: row.get(5)?,
                    status: status_str,
                    is_host,
                    paired_at: row.get(8)?,
                    last_seen_at: last_seen_str,
                    blocked_at: row.get(10)?,
                    revoked_at: row.get(11)?,
                    revocation_reason: row.get(12)?,
                    is_online,
                    active_sessions_count: active_sessions,
                    paired_via: row.get::<_, Option<String>>(13)?.unwrap_or_else(|| "code".to_string()),
                    platform: row.get(14)?,
                    device_type: row.get(15)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let quota = crate::entitlement::get_workstation_quota(conn, current_utc)?;

        let pairing_open = conn
            .query_row(
                "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'pending' AND expires_at > ?1",
                params![current_utc],
                |r| r.get::<_, i64>(0),
            )
            .map(|cnt| cnt > 0)
            .unwrap_or(false);

        Ok(WorkstationListResponse {
            devices,
            quota,
            pairing_open,
        })
    }

    pub fn get_workstation(
        &self,
        conn: &Connection,
        device_id: &str,
        current_utc: &str,
    ) -> Result<WorkstationDetail, LanError> {
        let dev = conn
            .query_row(
                "SELECT id, name, windows_name, ip_address, os_version, app_version,
                        status, is_host, paired_at, last_seen_at, blocked_at, revoked_at,
                        revocation_reason, paired_via, platform, device_type
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
                        row.get::<_, Option<String>>(13)?
                            .unwrap_or_else(|| "code".to_string()),
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
        ) = match dev {
            Some(d) => d,
            None => return Err(LanError::DeviceNotFound),
        };

        let now_ts =
            jiff::Timestamp::from_str(current_utc).unwrap_or_else(|_| jiff::Timestamp::now());
        let is_online = if status == "active" {
            if is_host {
                true
            } else if let Ok(seen_ts) = jiff::Timestamp::from_str(&last_seen_at) {
                now_ts.duration_since(seen_ts).as_secs() <= 30
            } else {
                false
            }
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

    pub fn rename_workstation(
        &self,
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

        let detail = self.get_workstation(conn, device_id, current_utc)?;
        Ok(detail.device)
    }

    pub fn block_workstation(
        &self,
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

        self.broadcast_control_event(DeviceControlEvent::Blocked {
            device_id: device_id.to_string(),
            reason: reason.unwrap_or("Administrative suspension").to_string(),
        });

        let detail = self.get_workstation(conn, device_id, current_utc)?;
        Ok(detail.device)
    }

    pub fn unblock_workstation(
        &self,
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

        crate::entitlement::guard_workstation_seat(conn, current_utc)?;

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

        let detail = self.get_workstation(conn, device_id, current_utc)?;
        Ok(detail.device)
    }

    pub fn revoke_workstation(
        &self,
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

        self.broadcast_control_event(DeviceControlEvent::Revoked {
            device_id: device_id.to_string(),
            reason: reason.unwrap_or("Permanently decommissioned").to_string(),
        });

        let detail = self.get_workstation(conn, device_id, current_utc)?;
        Ok(detail.device)
    }

    pub fn list_device_events(
        &self,
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

    pub fn update_host_receipt_printer(
        &self,
        conn: &Connection,
        printer_name: Option<String>,
        current_utc: &str,
    ) -> Result<(), LanError> {
        let config = self.get_or_create_host_config(conn, current_utc)?;
        conn.execute(
            "UPDATE lan_host_config SET receipt_printer_name = ?1, updated_at = ?2 WHERE id = ?3",
            params![printer_name, current_utc, config.id],
        )?;
        Ok(())
    }

    pub fn set_user_pin(
        &self,
        conn: &Connection,
        user_id: &str,
        pin: &str,
    ) -> Result<(), LanError> {
        set_user_pin(conn, user_id, pin)
    }
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
    super::lan_manager().update_host_receipt_printer(conn, printer_name, current_utc)
}

// -----------------------------------------------------------------------------
// Axum HTTP Handlers
// -----------------------------------------------------------------------------

async fn hello_handler(
    State(ctx): State<HostServerState>,
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
    let pairing_open = ctx
        .with_db(|conn| {
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

async fn health_handler(State(ctx): State<HostServerState>) -> Response {
    let mut db_status = "ok".to_string();
    let mut active_count = 0u32;

    let res = ctx.with_db(|conn| {
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
    State(ctx): State<HostServerState>,
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

    let result = ctx.with_db(|conn| {
        let locked_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM lan_pairing_sessions WHERE status = 'locked' AND expires_at > ?1",
            params![now_utc],
            |r| r.get(0),
        )?;
        if locked_count > 0 {
            return Err(LanError::PairingLocked);
        }

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
                let _ = conn.execute(
                    "UPDATE lan_pairing_sessions SET status = 'expired', updated_at = ?1 WHERE status = 'pending' AND expires_at <= ?1",
                    params![now_utc],
                );
                return Err(LanError::PairingClosed);
            }
        };

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

                crate::lan::lan_manager().update_mdns_pairing_status(conn, &now_utc, false);

                let audit_entry = crate::audit::AuditEntry {
                    action: "security.pairing_brute_force_locked".to_string(),
                    entity: "lan_pairing_sessions".to_string(),
                    entity_id: session_id.clone(),
                    before_json: None,
                    after_json: Some(serde_json::json!({
                        "failed_attempts": new_failed,
                        "source": "code",
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
                return Err(LanError::PairingCodeInvalid);
            }
        }

        crate::entitlement::guard_workstation_seat(conn, &now_utc)?;

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

        conn.execute(
            "UPDATE lan_pairing_sessions SET status = 'used', paired_device_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![device_id, now_utc, session_id],
        )?;

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
    State(ctx): State<HostServerState>,
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

    let result = ctx.with_db(|conn| {
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

        let used_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM lan_pairing_sessions WHERE qr_secret_hash = ?1 AND status = 'used'",
                params![input_secret_hash],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if used_count > 0 {
            return Err(LanError::PairingSessionAlreadyUsed);
        }

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
                return Err(LanError::PairingClosed);
            }
        };

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

                crate::lan::lan_manager().update_mdns_pairing_status(conn, &now_utc, false);

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

        crate::entitlement::guard_workstation_seat(conn, &now_utc)?;

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
    State(ctx): State<HostServerState>,
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

    let result = ctx.with_db(|conn| {
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

        let (user_id, username, display_name, staff_profile_id) = match user_match {
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

        let staff_id = staff_profile_id.clone().unwrap_or_else(|| user_id.clone());
        let primary_role = roles
            .first()
            .cloned()
            .unwrap_or_else(|| "staff".to_string());

        Ok(StaffLoginResponse {
            session_token,
            user_id: Some(user_id),
            username: Some(username),
            display_name: Some(display_name.clone()),
            staff_profile_id,
            staff_id: Some(staff_id),
            role: Some(primary_role),
            full_name: Some(display_name),
            permissions: Some(permissions),
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
    State(ctx): State<HostServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let token = headers
        .get("X-Coachify-Device-Token")
        .and_then(|h| h.to_str().ok())
        .or_else(|| payload.get("device_token").and_then(|v| v.as_str()));

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
        })
        .or_else(|| payload.get("session_token").and_then(|v| v.as_str()));

    let idempotency_header = headers
        .get("X-Coachify-Idempotency-Key")
        .and_then(|h| h.to_str().ok())
        .or_else(|| payload.get("idempotency_key").and_then(|v| v.as_str()));

    let command_str = payload
        .get("command")
        .or_else(|| payload.get("method"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let params_val = payload
        .get("payload")
        .or_else(|| payload.get("params"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let (status_code, body) = ctx.with_db(|conn| {
        let dev = conn.query_row(
            "SELECT id, status, COALESCE(device_type, 'workstation') FROM devices WHERE token_hash = ?1",
            params![token_hash],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
        ).optional()?;

        let (device_id, dev_status, dev_type) = match dev {
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
                if dev_type.starts_with("mobile") || dev_type == "tablet" {
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
                }

                if let Ok(exp_ts) = jiff::Timestamp::from_str(&expires_at) {
                    if now_ts > exp_ts {
                        return Ok((
                            StatusCode::UNAUTHORIZED,
                            serde_json::json!({
                                "ok": false,
                                "error": {
                                    "code": "SESSION_EXPIRED",
                                    "message": "Session has expired"
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

        let (resp_str, events) = crate::dispatcher::dispatch_at(
            conn,
            command_str,
            &params_val.to_string(),
            session_header,
            idempotency_header,
            &now_utc,
        );

        if !events.is_empty() {
            let _ = ctx.control_sender.send(DeviceControlEvent::Change { events: events.clone() });
        }

        let parsed: serde_json::Value = serde_json::from_str(&resp_str).unwrap_or_else(|_| {
            serde_json::json!({
                "ok": false,
                "error": { "code": "INTERNAL_ERROR", "message": "Failed to parse dispatcher response" }
            })
        });

        let is_ok = parsed.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let data = parsed.get("data").cloned();
        let error = parsed.get("error").cloned().and_then(|v| serde_json::from_value::<RpcError>(v).ok());

        let forwarded_resp = ForwardedRpcResponse {
            ok: is_ok,
            data,
            error,
            events,
            server_time: now_utc.clone(),
        };

        let json_body = serde_json::to_value(forwarded_resp).unwrap_or(parsed);

        Ok((StatusCode::OK, json_body))
    }).unwrap_or_else(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "ok": false,
                "error": { "code": err.code(), "message": err.to_string() }
            })
        )
    });

    let mut response = (status_code, Json(body)).into_response();
    if let Ok(val) = axum::http::HeaderValue::from_str(&now_utc) {
        response.headers_mut().insert("X-Coachify-Server-Time", val);
    }
    response
}

async fn host_receipt_print_handler(
    State(ctx): State<HostServerState>,
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

    let result = ctx.with_db(|conn| {
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
    State(ctx): State<HostServerState>,
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

    let check = ctx.with_db(|conn| {
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

    let rx = ctx.control_sender.subscribe();
    let target_id = dev_id.clone();

    let stream = BroadcastStream::new(rx).filter_map(move |item| match item {
        Ok(DeviceControlEvent::Change { events }) => Some(Ok::<_, std::convert::Infallible>(
            Event::default()
                .event("coachify_change")
                .data(serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())),
        )),
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

pub fn ping_host(payload: PingHostPayload) -> Result<HostPingResult, LanError> {
    ensure_crypto_provider_installed();
    let target_ip = payload.ip;
    let target_port = payload.port;
    let expected_fp = payload.expected_fingerprint;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| LanError::Database(e.to_string()))?;

    rt.block_on(async move {
        let start_time = Instant::now();
        let captured_der = Arc::new(Mutex::new(None));
        let verifier = Arc::new(super::crypto::PinnedCertVerifier {
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

        let server_name = match rustls::pki_types::ServerName::try_from("localhost".to_string()) {
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

        if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut tls_stream, http_request.as_bytes()).await {
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
            let read_step = tokio::time::timeout(Duration::from_millis(500), tokio::io::AsyncReadExt::read(&mut tls_stream, &mut buf)).await;
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
