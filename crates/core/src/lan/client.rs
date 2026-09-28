use super::crypto::{
    compute_sha256_fingerprint, ensure_crypto_provider_installed, FingerprintPinnedCertVerifier,
    PinnedCertVerifier,
};
use super::protocol::*;
use crate::dispatcher::{ChangeEvent, RpcEnvelope};
use axum::body::Body;
use axum::http;
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tower::util::ServiceExt;

// -----------------------------------------------------------------------------
// Workstation Windows DPAPI Credential Custody
// -----------------------------------------------------------------------------

pub fn resolve_workstation_credentials_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("COACHIFY_TEST_WORKSTATION_CREDS_PATH") {
        if !override_path.trim().is_empty() {
            let path = PathBuf::from(override_path);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            return path;
        }
    }
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
// TransportClient Seam Trait
// -----------------------------------------------------------------------------

pub trait TransportClient: Send + Sync {
    fn is_client_mode(&self) -> bool {
        self.get_credentials().is_some()
    }
    fn get_credentials(&self) -> Option<WorkstationCredentials>;
    fn reload_credentials(&self) -> Option<WorkstationCredentials>;
    fn get_status(&self) -> WorkstationClientStatus;
    fn disconnect(&self) -> Result<(), LanError>;
    fn pair(&self, payload: ClientPairPayload) -> Result<PairSuccessResponse, LanError>;
    fn forward_rpc_sync(
        &self,
        command: &str,
        payload_json: &str,
        session_token: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<(String, Vec<ChangeEvent>), LanError>;
    fn test_connection_sync(&self) -> Result<TestConnectionResponse, LanError>;
    fn reconnect_sync(&self) -> Result<ReconnectWorkstationResponse, LanError>;
    fn start_sse_stream_sync(
        &self,
        on_change: Box<dyn Fn(Vec<ChangeEvent>) + Send + Sync + 'static>,
    ) -> Result<(), LanError>;
    fn check_host_update_sync(&self) -> Result<Option<HostUpdateManifest>, LanError>;
    fn download_host_update_sync(
        &self,
        version: Option<&str>,
        target_path: &Path,
    ) -> Result<PathBuf, LanError>;
}

// -----------------------------------------------------------------------------
// Production Adapter: TlsClientTransport
// -----------------------------------------------------------------------------

pub struct TlsClientTransport {
    creds: RwLock<Option<WorkstationCredentials>>,
    state: RwLock<WorkstationConnectionState>,
    latency_ms: RwLock<Option<u64>>,
    last_synced_at: RwLock<Option<String>>,
    reconnect_attempts: AtomicU32,
    active_sse_cancel: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
    tokio_runtime: tokio::runtime::Runtime,
}

impl Default for TlsClientTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsClientTransport {
    pub fn new() -> Self {
        let creds = load_workstation_credentials().ok().flatten();
        let initial_state = if creds.is_some() {
            WorkstationConnectionState::Connected
        } else {
            WorkstationConnectionState::Offline
        };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("coachify-client-worker")
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime for TlsClientTransport");
        Self {
            creds: RwLock::new(creds),
            state: RwLock::new(initial_state),
            latency_ms: RwLock::new(None),
            last_synced_at: RwLock::new(None),
            reconnect_attempts: AtomicU32::new(0),
            active_sse_cancel: Mutex::new(None),
            tokio_runtime: rt,
        }
    }

    pub async fn forward_rpc(
        &self,
        command: &str,
        payload: &serde_json::Value,
        session_token: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<(String, Vec<ChangeEvent>), LanError> {
        ensure_crypto_provider_installed();

        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let generated_key;
        let actual_idempotency_key = match idempotency_key {
            Some(k) if !k.trim().is_empty() => Some(k),
            _ => {
                if is_mutating_command(command) {
                    generated_key = uuid::Uuid::now_v7().to_string();
                    Some(generated_key.as_str())
                } else {
                    None
                }
            }
        };

        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&creds.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let socket_addr: SocketAddr = match format!("{}:{}", creds.host_ip, creds.host_port).parse()
        {
            Ok(a) => a,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    format!("{}", e),
                ));
            }
        };

        let tcp_stream = match tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect(socket_addr),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => {
                self.reconnect_attempts.fetch_add(1, Ordering::Relaxed);
                *self.state.write().unwrap() = WorkstationConnectionState::Reconnecting;
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    "Connection timed out or host unreachable".to_string(),
                ));
            }
        };

        let server_name = match ServerName::try_from("coachify.lan") {
            Ok(sn) => sn,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    format!("{}", e),
                ));
            }
        };

        let mut tls_stream = match connector.connect(server_name, tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                return Err(LanError::FingerprintMismatch {
                    expected: creds.host_fingerprint.clone(),
                    received: format!("TLS error: {}", e),
                });
            }
        };

        let req_body = serde_json::json!({
            "command": command,
            "payload": payload,
            "session_token": session_token,
            "idempotency_key": actual_idempotency_key,
        })
        .to_string();

        let mut headers_str = format!(
            "POST /rpc HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Coachify-Device-Token: {}\r\nConnection: close\r\n",
            creds.host_ip, creds.host_port, req_body.len(), creds.device_token
        );
        if let Some(st) = session_token {
            headers_str.push_str(&format!("Authorization: Bearer {}\r\n", st));
            headers_str.push_str(&format!("X-Coachify-Session-Token: {}\r\n", st));
        }
        if let Some(k) = actual_idempotency_key {
            headers_str.push_str(&format!("X-Coachify-Idempotency-Key: {}\r\n", k));
        }
        headers_str.push_str("\r\n");

        tls_stream
            .write_all(headers_str.as_bytes())
            .await
            .map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;
        tls_stream
            .write_all(req_body.as_bytes())
            .await
            .map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

        let mut resp_bytes = Vec::new();
        tls_stream.read_to_end(&mut resp_bytes).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let resp_str = String::from_utf8_lossy(&resp_bytes);
        let (header_part, body_part) = match resp_str.find("\r\n\r\n") {
            Some(pos) => (&resp_str[..pos], &resp_str[pos + 4..]),
            None => return Err(LanError::Database("Malformed HTTP response".to_string())),
        };

        let first_line = header_part.lines().next().unwrap_or("");
        if first_line.contains("403") {
            if body_part.contains("DEVICE_REVOKED") {
                let _ = clear_workstation_credentials();
                *self.state.write().unwrap() = WorkstationConnectionState::Revoked;
                *self.creds.write().unwrap() = None;
                return Err(LanError::WorkstationRevoked);
            }
            if body_part.contains("DEVICE_BLOCKED") {
                *self.state.write().unwrap() = WorkstationConnectionState::Blocked;
                return Err(LanError::WorkstationBlocked);
            }
        }

        let parsed_resp: ForwardedRpcResponse = match serde_json::from_str(body_part) {
            Ok(r) => r,
            Err(_) => {
                if let Ok(_env) = serde_json::from_str::<RpcEnvelope<serde_json::Value>>(body_part)
                {
                    *self.state.write().unwrap() = WorkstationConnectionState::Connected;
                    self.reconnect_attempts.store(0, Ordering::Relaxed);
                    *self.last_synced_at.write().unwrap() =
                        Some(jiff::Timestamp::now().to_string());
                    return Ok((body_part.to_string(), vec![]));
                }
                return Err(LanError::Database(
                    "Failed to parse host RPC response".to_string(),
                ));
            }
        };

        *self.state.write().unwrap() = WorkstationConnectionState::Connected;
        self.reconnect_attempts.store(0, Ordering::Relaxed);
        *self.last_synced_at.write().unwrap() = Some(jiff::Timestamp::now().to_string());

        let envelope = RpcEnvelope {
            ok: parsed_resp.ok,
            data: parsed_resp.data,
            error: parsed_resp.error,
        };
        let envelope_json =
            serde_json::to_string(&envelope).unwrap_or_else(|_| body_part.to_string());

        Ok((envelope_json, parsed_resp.events))
    }

    pub async fn test_connection(&self) -> Result<TestConnectionResponse, LanError> {
        ensure_crypto_provider_installed();
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let start = Instant::now();
        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&creds.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let socket_addr: SocketAddr = match format!("{}:{}", creds.host_ip, creds.host_port).parse()
        {
            Ok(a) => a,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    e.to_string(),
                ));
            }
        };

        let tcp_stream = match tokio::time::timeout(
            Duration::from_secs(3),
            tokio::net::TcpStream::connect(socket_addr),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => {
                *self.latency_ms.write().unwrap() = None;
                *self.state.write().unwrap() = WorkstationConnectionState::Offline;
                return Ok(TestConnectionResponse {
                    success: false,
                    latency_ms: None,
                    host_id: Some(creds.host_id.clone()),
                    message: "Host unreachable or connection timed out".to_string(),
                });
            }
        };

        let server_name = match ServerName::try_from("coachify.lan") {
            Ok(sn) => sn,
            Err(e) => {
                *self.latency_ms.write().unwrap() = None;
                *self.state.write().unwrap() = WorkstationConnectionState::Offline;
                return Ok(TestConnectionResponse {
                    success: false,
                    latency_ms: None,
                    host_id: Some(creds.host_id.clone()),
                    message: format!("{}", e),
                });
            }
        };

        let mut tls_stream = match connector.connect(server_name, tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                *self.latency_ms.write().unwrap() = None;
                *self.state.write().unwrap() = WorkstationConnectionState::Offline;
                return Ok(TestConnectionResponse {
                    success: false,
                    latency_ms: None,
                    host_id: Some(creds.host_id.clone()),
                    message: format!("TLS handshake failed: {}", e),
                });
            }
        };

        let req = format!(
            "GET /health HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
            creds.host_ip, creds.host_port
        );
        tls_stream.write_all(req.as_bytes()).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let mut resp_bytes = Vec::new();
        tls_stream.read_to_end(&mut resp_bytes).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let rtt = start.elapsed().as_millis() as u64;
        *self.latency_ms.write().unwrap() = Some(rtt);
        *self.state.write().unwrap() = WorkstationConnectionState::Connected;
        *self.last_synced_at.write().unwrap() = Some(jiff::Timestamp::now().to_string());

        let resp_str = String::from_utf8_lossy(&resp_bytes);
        let host_id = if let Some(pos) = resp_str.find("\r\n\r\n") {
            let body = &resp_str[pos + 4..];
            serde_json::from_str::<HealthResponse>(body)
                .ok()
                .map(|h| h.host_id)
        } else {
            None
        };

        Ok(TestConnectionResponse {
            success: true,
            latency_ms: Some(rtt),
            host_id: host_id.or(Some(creds.host_id.clone())),
            message: "Connected to host successfully".to_string(),
        })
    }

    pub async fn reconnect(&self) -> Result<ReconnectWorkstationResponse, LanError> {
        *self.state.write().unwrap() = WorkstationConnectionState::Connecting;
        let mut creds = match self.get_credentials() {
            Some(c) => c,
            None => {
                *self.state.write().unwrap() = WorkstationConnectionState::Offline;
                return Err(LanError::NotPaired);
            }
        };

        if let Ok(test_res) = self.test_connection().await {
            *self.state.write().unwrap() = WorkstationConnectionState::Connected;
            self.reconnect_attempts.store(0, Ordering::Relaxed);
            return Ok(ReconnectWorkstationResponse {
                success: true,
                connection_state: WorkstationConnectionState::Connected,
                latency_ms: test_res.latency_ms,
                message: "Reconnected to host successfully".to_string(),
            });
        }

        *self.state.write().unwrap() = WorkstationConnectionState::Reconnecting;
        self.reconnect_attempts.fetch_add(1, Ordering::Relaxed);

        if let Ok(discovered) = crate::lan::discover_hosts(Some(3000)) {
            for host in discovered.hosts {
                let norm_creds = creds
                    .host_fingerprint
                    .replace("SHA256:", "")
                    .replace(':', "")
                    .to_lowercase();
                let norm_found = host
                    .fingerprint
                    .replace("SHA256:", "")
                    .replace(':', "")
                    .to_lowercase();

                if (host.host_id == creds.host_id || norm_found == norm_creds)
                    && !host.ip.is_empty()
                {
                    creds.host_ip = host.ip;
                    creds.host_port = host.port;
                    let _ = save_workstation_credentials(&creds);
                    *self.creds.write().unwrap() = Some(creds.clone());

                    if let Ok(test_res) = self.test_connection().await {
                        *self.state.write().unwrap() = WorkstationConnectionState::Connected;
                        self.reconnect_attempts.store(0, Ordering::Relaxed);
                        return Ok(ReconnectWorkstationResponse {
                            success: true,
                            connection_state: WorkstationConnectionState::Connected,
                            latency_ms: test_res.latency_ms,
                            message: "Host re-discovered and reconnected successfully".to_string(),
                        });
                    }
                }
            }
        }

        *self.state.write().unwrap() = WorkstationConnectionState::Offline;
        Ok(ReconnectWorkstationResponse {
            success: false,
            connection_state: WorkstationConnectionState::Offline,
            latency_ms: None,
            message: "Unable to reach host. Please check network connection.".to_string(),
        })
    }

    pub async fn start_sse_stream<F>(&self, on_change: F) -> Result<(), LanError>
    where
        F: Fn(Vec<ChangeEvent>) + Send + Sync + 'static,
    {
        ensure_crypto_provider_installed();
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&creds.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let socket_addr: SocketAddr = match format!("{}:{}", creds.host_ip, creds.host_port).parse()
        {
            Ok(a) => a,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    e.to_string(),
                ));
            }
        };

        let tcp_stream = tokio::net::TcpStream::connect(socket_addr)
            .await
            .map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

        let server_name = match ServerName::try_from("coachify.lan") {
            Ok(sn) => sn,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    e.to_string(),
                ));
            }
        };

        let tls_stream = connector
            .connect(server_name, tcp_stream)
            .await
            .map_err(|e| LanError::FingerprintMismatch {
                expected: creds.host_fingerprint.clone(),
                received: format!("TLS error: {}", e),
            })?;

        let (reader, mut writer) = tokio::io::split(tls_stream);

        let req = format!(
            "GET /events HTTP/1.1\r\nHost: {}:{}\r\nX-Coachify-Device-Token: {}\r\nAccept: text/event-stream\r\nConnection: keep-alive\r\n\r\n",
            creds.host_ip, creds.host_port, creds.device_token
        );

        writer.write_all(req.as_bytes()).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let mut lines = tokio::io::BufReader::new(reader).lines();
        let mut current_event_type = String::new();

        while let Ok(Some(line)) = lines.next_line().await {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                current_event_type.clear();
                continue;
            }

            if let Some(evt) = line_trimmed.strip_prefix("event:") {
                current_event_type = evt.trim().to_string();
                continue;
            }

            if let Some(data) = line_trimmed.strip_prefix("data:") {
                let data_str = data.trim();
                match current_event_type.as_str() {
                    "coachify_change" => {
                        if let Ok(events) = serde_json::from_str::<Vec<ChangeEvent>>(data_str) {
                            on_change(events);
                        }
                    }
                    "device.revoked" | "device_revoked" => {
                        let _ = clear_workstation_credentials();
                        *self.state.write().unwrap() = WorkstationConnectionState::Revoked;
                        *self.creds.write().unwrap() = None;
                        break;
                    }
                    "device.blocked" | "device_blocked" => {
                        *self.state.write().unwrap() = WorkstationConnectionState::Blocked;
                    }
                    "heartbeat" => {
                        *self.last_synced_at.write().unwrap() =
                            Some(jiff::Timestamp::now().to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub async fn check_host_update(&self) -> Result<Option<HostUpdateManifest>, LanError> {
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        ensure_crypto_provider_installed();
        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&creds.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
        let socket_addr: SocketAddr = format!("{}:{}", creds.host_ip, creds.host_port)
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

        let tcp_stream = match tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect(socket_addr),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    "Connection timed out or host unreachable".to_string(),
                ));
            }
        };

        let server_name = match ServerName::try_from("coachify.lan") {
            Ok(sn) => sn,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    format!("{}", e),
                ));
            }
        };

        let mut tls_stream = match connector.connect(server_name, tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                return Err(LanError::FingerprintMismatch {
                    expected: creds.host_fingerprint.clone(),
                    received: format!("TLS error: {}", e),
                });
            }
        };

        let req = format!(
            "GET /updates/manifest HTTP/1.1\r\nHost: {}:{}\r\nX-Coachify-Device-Token: {}\r\nConnection: close\r\n\r\n",
            creds.host_ip, creds.host_port, creds.device_token
        );
        tls_stream.write_all(req.as_bytes()).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let mut resp_bytes = Vec::new();
        tls_stream.read_to_end(&mut resp_bytes).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let resp_str = String::from_utf8_lossy(&resp_bytes);
        let header_end = resp_str.find("\r\n\r\n");
        let (headers, body) = match header_end {
            Some(pos) => (&resp_str[..pos], &resp_str[pos + 4..]),
            None => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    "Invalid HTTP response from host".to_string(),
                ));
            }
        };

        let status_line = headers.lines().next().unwrap_or("");
        if status_line.contains("404") {
            return Ok(None);
        }
        if status_line.contains("503") {
            return Err(LanError::UpdateMirrorDisabled);
        }
        if status_line.contains("401") {
            return Err(LanError::PairingCodeInvalid);
        }
        if status_line.contains("403") {
            if body.contains("DEVICE_REVOKED") {
                *self.state.write().unwrap() = WorkstationConnectionState::Revoked;
                return Err(LanError::WorkstationRevoked);
            }
            if body.contains("DEVICE_BLOCKED") {
                *self.state.write().unwrap() = WorkstationConnectionState::Blocked;
                return Err(LanError::WorkstationBlocked);
            }
            return Err(LanError::WorkstationRevoked);
        }
        if !status_line.contains("200") {
            return Err(LanError::UpdateMirrorError(format!(
                "Host returned: {status_line}"
            )));
        }

        let manifest: HostUpdateManifest = serde_json::from_str(body)
            .map_err(|e| LanError::UpdateMirrorError(format!("Invalid manifest JSON: {e}")))?;

        Ok(Some(manifest))
    }

    pub async fn download_host_update(
        &self,
        version: Option<&str>,
        target_path: &Path,
    ) -> Result<PathBuf, LanError> {
        let manifest = self
            .check_host_update()
            .await?
            .ok_or(LanError::NoUpdateMirrored)?;

        if let Some(v) = version {
            if !v.trim().is_empty() && v != manifest.version {
                return Err(LanError::UpdateMirrorError(format!(
                    "Requested version '{v}' does not match active mirrored version '{}'",
                    manifest.version
                )));
            }
        }

        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        ensure_crypto_provider_installed();
        let verifier = Arc::new(FingerprintPinnedCertVerifier::new(&creds.host_fingerprint));
        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();

        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
        let socket_addr: SocketAddr = format!("{}:{}", creds.host_ip, creds.host_port)
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

        let tcp_stream = match tokio::time::timeout(
            Duration::from_secs(10),
            tokio::net::TcpStream::connect(socket_addr),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    "Connection timed out or host unreachable".to_string(),
                ));
            }
        };

        let server_name = match ServerName::try_from("coachify.lan") {
            Ok(sn) => sn,
            Err(e) => {
                return Err(LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    format!("{}", e),
                ));
            }
        };

        let mut tls_stream = match connector.connect(server_name, tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                return Err(LanError::FingerprintMismatch {
                    expected: creds.host_fingerprint.clone(),
                    received: format!("TLS error: {}", e),
                });
            }
        };

        let req_path = format!("/updates/download/{}", manifest.version);
        let req = format!(
            "GET {} HTTP/1.1\r\nHost: {}:{}\r\nX-Coachify-Device-Token: {}\r\nConnection: close\r\n\r\n",
            req_path, creds.host_ip, creds.host_port, creds.device_token
        );
        tls_stream.write_all(req.as_bytes()).await.map_err(|e| {
            LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
        })?;

        let mut response_bytes = Vec::new();
        let mut buf = [0u8; 16384];
        let read_deadline = Instant::now() + Duration::from_secs(30);

        while Instant::now() < read_deadline {
            let read_step =
                tokio::time::timeout(Duration::from_secs(5), tls_stream.read(&mut buf)).await;
            match read_step {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    response_bytes.extend_from_slice(&buf[..n]);
                    if let Some(pos) = response_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let header_part = String::from_utf8_lossy(&response_bytes[..pos]);
                        if let Some(len_line) = header_part
                            .lines()
                            .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                        {
                            if let Ok(expected_len) = len_line[15..].trim().parse::<usize>() {
                                let body_len = response_bytes.len() - (pos + 4);
                                if body_len >= expected_len {
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    return Err(LanError::HostUnreachable(
                        creds.host_ip.clone(),
                        creds.host_port,
                        format!("Read error: {e}"),
                    ));
                }
                Err(_) => {
                    return Err(LanError::HostUnreachable(
                        creds.host_ip.clone(),
                        creds.host_port,
                        "Download read timed out".to_string(),
                    ));
                }
            }
        }

        let header_pos = response_bytes
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or_else(|| {
                LanError::HostUnreachable(
                    creds.host_ip.clone(),
                    creds.host_port,
                    "Invalid HTTP response from host".to_string(),
                )
            })?;

        let header_bytes = &response_bytes[..header_pos];
        let body_bytes = &response_bytes[header_pos + 4..];

        let header_str = String::from_utf8_lossy(header_bytes);
        let status_line = header_str.lines().next().unwrap_or("");
        if status_line.contains("404") {
            return Err(LanError::NoUpdateMirrored);
        }
        if status_line.contains("503") {
            return Err(LanError::UpdateMirrorDisabled);
        }
        if status_line.contains("403") {
            return Err(LanError::WorkstationRevoked);
        }
        if !status_line.contains("200") {
            return Err(LanError::UpdateMirrorError(format!(
                "Host returned status: {status_line}"
            )));
        }

        // 1. File size check
        if manifest.file_size_bytes > 0 && body_bytes.len() as i64 != manifest.file_size_bytes {
            return Err(LanError::UpdateMirrorError(format!(
                "Downloaded file size ({}) does not match manifest ({})",
                body_bytes.len(),
                manifest.file_size_bytes
            )));
        }

        // 2. SHA-256 verification
        let computed_sha256 = super::mirror::compute_sha256(body_bytes);
        if !computed_sha256.eq_ignore_ascii_case(&manifest.sha256) {
            if target_path.exists() {
                let _ = std::fs::remove_file(target_path);
            }
            return Err(LanError::UpdateChecksumMismatch {
                expected: manifest.sha256.clone(),
                computed: computed_sha256,
            });
        }

        // 3. Ed25519 signature verification
        if let Err(e) = super::mirror::verify_staged_update_signature(
            body_bytes,
            &computed_sha256,
            &manifest.updater_signature,
            &manifest.updater_key_id,
        ) {
            if target_path.exists() {
                let _ = std::fs::remove_file(target_path);
            }
            return Err(e);
        }

        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(target_path, body_bytes).map_err(|e| {
            LanError::UpdateMirrorError(format!("Failed to write verified update file: {e}"))
        })?;

        Ok(target_path.to_path_buf())
    }
}

impl TransportClient for TlsClientTransport {
    fn get_credentials(&self) -> Option<WorkstationCredentials> {
        let read_lock = self.creds.read().unwrap();
        if let Some(ref c) = *read_lock {
            return Some(c.clone());
        }
        drop(read_lock);
        if let Ok(Some(loaded)) = load_workstation_credentials() {
            let mut write_lock = self.creds.write().unwrap();
            *write_lock = Some(loaded.clone());
            return Some(loaded);
        }
        None
    }

    fn reload_credentials(&self) -> Option<WorkstationCredentials> {
        let loaded = load_workstation_credentials().ok().flatten();
        let mut write_lock = self.creds.write().unwrap();
        *write_lock = loaded.clone();
        if loaded.is_some() {
            *self.state.write().unwrap() = WorkstationConnectionState::Connected;
        } else {
            *self.state.write().unwrap() = WorkstationConnectionState::Offline;
        }
        loaded
    }

    fn get_status(&self) -> WorkstationClientStatus {
        let creds_opt = self.get_credentials();
        let mode = if creds_opt.is_some() {
            "workstation_client".to_string()
        } else {
            "standalone".to_string()
        };
        let state = *self.state.read().unwrap();
        let latency_ms = *self.latency_ms.read().unwrap();
        let last_synced_at = self.last_synced_at.read().unwrap().clone();
        let reconnect_attempts = self.reconnect_attempts.load(Ordering::Relaxed);

        let is_client_mode = creds_opt.is_some();
        WorkstationClientStatus {
            is_client_mode,
            state,
            mode,
            connection_state: state,
            host_id: creds_opt.as_ref().map(|c| c.host_id.clone()),
            host_ip: creds_opt.as_ref().map(|c| c.host_ip.clone()),
            host_port: creds_opt.as_ref().map(|c| c.host_port),
            host_name: creds_opt.as_ref().and_then(|c| c.host_name.clone()),
            device_id: creds_opt.as_ref().map(|c| c.device_id.clone()),
            workstation_name: creds_opt.as_ref().and_then(|c| c.workstation_name.clone()),
            latency_ms,
            rtt_ms: latency_ms,
            last_synced_at: last_synced_at.clone(),
            last_heartbeat_at: last_synced_at,
            paired_at: creds_opt.as_ref().map(|c| c.paired_at.clone()),
            is_reconnecting: state == WorkstationConnectionState::Reconnecting,
            reconnect_attempts,
        }
    }

    fn disconnect(&self) -> Result<(), LanError> {
        if let Some(cancel) = self.active_sse_cancel.lock().unwrap().take() {
            let _ = cancel.send(());
        }
        let _ = clear_workstation_credentials();
        *self.creds.write().unwrap() = None;
        *self.state.write().unwrap() = WorkstationConnectionState::Offline;
        *self.latency_ms.write().unwrap() = None;
        self.reconnect_attempts.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn pair(&self, payload: ClientPairPayload) -> Result<PairSuccessResponse, LanError> {
        ensure_crypto_provider_installed();
        let target_ip = payload.host_ip;
        let target_port = payload.host_port;
        let expected_fp = payload.expected_fingerprint;
        let pairing_code = payload.pairing_code;
        let workstation_name = payload.workstation_name;

        self.tokio_runtime.block_on(async move {
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

            let hostname = get_system_hostname();
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
                    let creds = WorkstationCredentials {
                        device_id: pair_res.device_id.clone(),
                        device_token: pair_res.device_token.clone(),
                        host_id: pair_res.host_device_id.clone(),
                        host_fingerprint: pair_res.host_fingerprint.clone(),
                        host_ip: target_ip,
                        host_port: target_port,
                        paired_at: jiff::Timestamp::now().to_string(),
                        host_name: Some(pair_res.institution_name.clone()),
                        workstation_name: Some(workstation_name),
                    };
                    let _ = save_workstation_credentials(&creds);
                    let _ = self.reload_credentials();

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

    fn forward_rpc_sync(
        &self,
        command: &str,
        payload_json: &str,
        session_token: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<(String, Vec<ChangeEvent>), LanError> {
        let payload_val: serde_json::Value =
            serde_json::from_str(payload_json).unwrap_or(serde_json::Value::Null);

        self.tokio_runtime.block_on(async {
            self.forward_rpc(command, &payload_val, session_token, idempotency_key)
                .await
        })
    }

    fn test_connection_sync(&self) -> Result<TestConnectionResponse, LanError> {
        self.tokio_runtime
            .block_on(async { self.test_connection().await })
    }

    fn reconnect_sync(&self) -> Result<ReconnectWorkstationResponse, LanError> {
        self.tokio_runtime
            .block_on(async { self.reconnect().await })
    }

    fn start_sse_stream_sync(
        &self,
        _on_change: Box<dyn Fn(Vec<ChangeEvent>) + Send + Sync + 'static>,
    ) -> Result<(), LanError> {
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let mut lock = self.active_sse_cancel.lock().unwrap();
        *lock = Some(tx);
        drop(lock);

        let this = self.get_credentials();
        if this.is_none() {
            return Err(LanError::NotPaired);
        }

        Ok(())
    }

    fn check_host_update_sync(&self) -> Result<Option<HostUpdateManifest>, LanError> {
        self.tokio_runtime
            .block_on(async { self.check_host_update().await })
    }

    fn download_host_update_sync(
        &self,
        version: Option<&str>,
        target_path: &Path,
    ) -> Result<PathBuf, LanError> {
        self.tokio_runtime
            .block_on(async { self.download_host_update(version, target_path).await })
    }
}

// -----------------------------------------------------------------------------
// Test Adapter: InMemoryClientTransport (Exercises 100% router logic via tower oneshot)
// -----------------------------------------------------------------------------

pub struct InMemoryClientTransport {
    router: axum::Router,
    creds: RwLock<Option<WorkstationCredentials>>,
    state: RwLock<WorkstationConnectionState>,
    latency_ms: RwLock<Option<u64>>,
    last_synced_at: RwLock<Option<String>>,
    reconnect_attempts: AtomicU32,
    tokio_runtime: tokio::runtime::Runtime,
}

impl InMemoryClientTransport {
    pub fn new(router: axum::Router) -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize runtime for InMemoryClientTransport");
        Self {
            router,
            creds: RwLock::new(None),
            state: RwLock::new(WorkstationConnectionState::Offline),
            latency_ms: RwLock::new(None),
            last_synced_at: RwLock::new(None),
            reconnect_attempts: AtomicU32::new(0),
            tokio_runtime: rt,
        }
    }

    pub fn with_credentials(router: axum::Router, creds: WorkstationCredentials) -> Self {
        let client = Self::new(router);
        *client.creds.write().unwrap() = Some(creds);
        *client.state.write().unwrap() = WorkstationConnectionState::Connected;
        client
    }
}

impl TransportClient for InMemoryClientTransport {
    fn get_credentials(&self) -> Option<WorkstationCredentials> {
        self.creds.read().unwrap().clone()
    }

    fn reload_credentials(&self) -> Option<WorkstationCredentials> {
        self.creds.read().unwrap().clone()
    }

    fn get_status(&self) -> WorkstationClientStatus {
        let creds_opt = self.get_credentials();
        let mode = if creds_opt.is_some() {
            "workstation_client".to_string()
        } else {
            "standalone".to_string()
        };
        let state = *self.state.read().unwrap();
        let latency_ms = *self.latency_ms.read().unwrap();
        let last_synced_at = self.last_synced_at.read().unwrap().clone();
        let reconnect_attempts = self.reconnect_attempts.load(Ordering::Relaxed);

        let is_client_mode = creds_opt.is_some();
        WorkstationClientStatus {
            is_client_mode,
            state,
            mode,
            connection_state: state,
            host_id: creds_opt.as_ref().map(|c| c.host_id.clone()),
            host_ip: creds_opt.as_ref().map(|c| c.host_ip.clone()),
            host_port: creds_opt.as_ref().map(|c| c.host_port),
            host_name: creds_opt.as_ref().and_then(|c| c.host_name.clone()),
            device_id: creds_opt.as_ref().map(|c| c.device_id.clone()),
            workstation_name: creds_opt.as_ref().and_then(|c| c.workstation_name.clone()),
            latency_ms,
            rtt_ms: latency_ms,
            last_synced_at: last_synced_at.clone(),
            last_heartbeat_at: last_synced_at,
            paired_at: creds_opt.as_ref().map(|c| c.paired_at.clone()),
            is_reconnecting: state == WorkstationConnectionState::Reconnecting,
            reconnect_attempts,
        }
    }

    fn disconnect(&self) -> Result<(), LanError> {
        *self.creds.write().unwrap() = None;
        *self.state.write().unwrap() = WorkstationConnectionState::Offline;
        *self.latency_ms.write().unwrap() = None;
        self.reconnect_attempts.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn pair(&self, payload: ClientPairPayload) -> Result<PairSuccessResponse, LanError> {
        let req_body = serde_json::to_string(&PairRequest {
            pairing_code: payload.pairing_code,
            workstation_name: payload.workstation_name.clone(),
            windows_name: "test-pc".to_string(),
            os_version: Some("Windows".to_string()),
            app_version: Some("0.1.0".to_string()),
            client_nonce: Some(uuid::Uuid::now_v7().to_string()),
        })
        .unwrap();

        let req = http::Request::builder()
            .method("POST")
            .uri("/pair")
            .header("content-type", "application/json")
            .body(Body::from(req_body))
            .unwrap();

        self.tokio_runtime.block_on(async {
            let res = self.router.clone().oneshot(req).await.map_err(|e| {
                LanError::HostUnreachable(payload.host_ip.clone(), payload.host_port, e.to_string())
            })?;

            let status = res.status();
            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .map_err(|e| LanError::Database(e.to_string()))?;

            if status == http::StatusCode::LOCKED {
                return Err(LanError::PairingLocked);
            }
            if status == http::StatusCode::FORBIDDEN {
                return Err(LanError::WorkstationLimitReached);
            }
            if status == http::StatusCode::UNAUTHORIZED {
                return Err(LanError::PairingCodeInvalid);
            }
            if !status.is_success() {
                return Err(LanError::HostUnreachable(
                    payload.host_ip,
                    payload.host_port,
                    format!("Status {}", status),
                ));
            }

            let pair_res: PairSuccessResponse = serde_json::from_slice(&body_bytes)
                .map_err(|e| LanError::Database(format!("JSON error: {}", e)))?;

            let creds = WorkstationCredentials {
                device_id: pair_res.device_id.clone(),
                device_token: pair_res.device_token.clone(),
                host_id: pair_res.host_device_id.clone(),
                host_fingerprint: pair_res.host_fingerprint.clone(),
                host_ip: payload.host_ip,
                host_port: payload.host_port,
                paired_at: jiff::Timestamp::now().to_string(),
                host_name: Some(pair_res.institution_name.clone()),
                workstation_name: Some(payload.workstation_name),
            };

            *self.creds.write().unwrap() = Some(creds);
            *self.state.write().unwrap() = WorkstationConnectionState::Connected;
            self.reconnect_attempts.store(0, Ordering::Relaxed);

            Ok(pair_res)
        })
    }

    fn forward_rpc_sync(
        &self,
        command: &str,
        payload_json: &str,
        session_token: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<(String, Vec<ChangeEvent>), LanError> {
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let payload_val: serde_json::Value =
            serde_json::from_str(payload_json).unwrap_or(serde_json::Value::Null);

        let req_body = serde_json::json!({
            "command": command,
            "payload": payload_val,
            "session_token": session_token,
            "idempotency_key": idempotency_key,
        })
        .to_string();

        let mut req_builder = http::Request::builder()
            .method("POST")
            .uri("/rpc")
            .header("content-type", "application/json")
            .header("x-coachify-device-token", &creds.device_token);

        if let Some(st) = session_token {
            req_builder = req_builder.header("authorization", format!("Bearer {}", st));
        }
        if let Some(k) = idempotency_key {
            req_builder = req_builder.header("x-coachify-idempotency-key", k);
        }

        let req = req_builder.body(Body::from(req_body)).unwrap();

        self.tokio_runtime.block_on(async {
            let res = self.router.clone().oneshot(req).await.map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

            let status = res.status();
            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .map_err(|e| LanError::Database(e.to_string()))?;

            if status == http::StatusCode::FORBIDDEN {
                let body_str = String::from_utf8_lossy(&body_bytes);
                if body_str.contains("DEVICE_REVOKED") {
                    *self.state.write().unwrap() = WorkstationConnectionState::Revoked;
                    *self.creds.write().unwrap() = None;
                    return Err(LanError::WorkstationRevoked);
                }
                if body_str.contains("DEVICE_BLOCKED") {
                    *self.state.write().unwrap() = WorkstationConnectionState::Blocked;
                    return Err(LanError::WorkstationBlocked);
                }
            }

            let parsed: ForwardedRpcResponse = match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(_) => {
                    return Err(LanError::Database("Invalid response".to_string()));
                }
            };

            *self.state.write().unwrap() = WorkstationConnectionState::Connected;
            *self.last_synced_at.write().unwrap() = Some(jiff::Timestamp::now().to_string());

            let env = RpcEnvelope {
                ok: parsed.ok,
                data: parsed.data,
                error: parsed.error,
            };
            let env_json = serde_json::to_string(&env).unwrap();
            Ok((env_json, parsed.events))
        })
    }

    fn test_connection_sync(&self) -> Result<TestConnectionResponse, LanError> {
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let req = http::Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        self.tokio_runtime.block_on(async {
            let start = Instant::now();
            let res = self.router.clone().oneshot(req).await.map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

            let rtt = start.elapsed().as_millis() as u64;
            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .map_err(|e| LanError::Database(e.to_string()))?;

            let health: Option<HealthResponse> = serde_json::from_slice(&body_bytes).ok();
            *self.latency_ms.write().unwrap() = Some(rtt);
            *self.state.write().unwrap() = WorkstationConnectionState::Connected;

            Ok(TestConnectionResponse {
                success: true,
                latency_ms: Some(rtt),
                host_id: health.map(|h| h.host_id).or(Some(creds.host_id)),
                message: "Connected to host successfully".to_string(),
            })
        })
    }

    fn reconnect_sync(&self) -> Result<ReconnectWorkstationResponse, LanError> {
        let test_res = self.test_connection_sync()?;
        Ok(ReconnectWorkstationResponse {
            success: test_res.success,
            connection_state: WorkstationConnectionState::Connected,
            latency_ms: test_res.latency_ms,
            message: "Reconnected successfully".to_string(),
        })
    }

    fn start_sse_stream_sync(
        &self,
        _on_change: Box<dyn Fn(Vec<ChangeEvent>) + Send + Sync + 'static>,
    ) -> Result<(), LanError> {
        Ok(())
    }

    fn check_host_update_sync(&self) -> Result<Option<HostUpdateManifest>, LanError> {
        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let req = http::Request::builder()
            .method("GET")
            .uri("/updates/manifest")
            .header("x-coachify-device-token", &creds.device_token)
            .body(Body::empty())
            .unwrap();

        self.tokio_runtime.block_on(async {
            let res = self.router.clone().oneshot(req).await.map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

            let status = res.status();
            if status == http::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            if status == http::StatusCode::SERVICE_UNAVAILABLE {
                return Err(LanError::UpdateMirrorDisabled);
            }
            if status == http::StatusCode::FORBIDDEN {
                return Err(LanError::WorkstationRevoked);
            }
            if status == http::StatusCode::UNAUTHORIZED {
                return Err(LanError::PairingCodeInvalid);
            }
            if !status.is_success() {
                return Err(LanError::UpdateMirrorError(format!(
                    "Host returned status {status}"
                )));
            }

            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .map_err(|e| LanError::Database(e.to_string()))?;

            let manifest: HostUpdateManifest = serde_json::from_slice(&body_bytes)
                .map_err(|e| LanError::UpdateMirrorError(format!("Invalid manifest: {e}")))?;

            Ok(Some(manifest))
        })
    }

    fn download_host_update_sync(
        &self,
        version: Option<&str>,
        target_path: &Path,
    ) -> Result<PathBuf, LanError> {
        let manifest = self
            .check_host_update_sync()?
            .ok_or(LanError::NoUpdateMirrored)?;

        if let Some(v) = version {
            if !v.trim().is_empty() && v != manifest.version {
                return Err(LanError::UpdateMirrorError(format!(
                    "Requested version '{v}' does not match active mirrored version '{}'",
                    manifest.version
                )));
            }
        }

        let creds = match self.get_credentials() {
            Some(c) => c,
            None => return Err(LanError::NotPaired),
        };

        let target_ver = version.unwrap_or(&manifest.version);
        let uri = format!("/updates/download/{target_ver}");
        let req = http::Request::builder()
            .method("GET")
            .uri(uri)
            .header("x-coachify-device-token", &creds.device_token)
            .body(Body::empty())
            .unwrap();

        self.tokio_runtime.block_on(async {
            let res = self.router.clone().oneshot(req).await.map_err(|e| {
                LanError::HostUnreachable(creds.host_ip.clone(), creds.host_port, e.to_string())
            })?;

            let status = res.status();
            if status == http::StatusCode::NOT_FOUND {
                return Err(LanError::NoUpdateMirrored);
            }
            if status == http::StatusCode::SERVICE_UNAVAILABLE {
                return Err(LanError::UpdateMirrorDisabled);
            }
            if status == http::StatusCode::FORBIDDEN {
                return Err(LanError::WorkstationRevoked);
            }
            if !status.is_success() {
                return Err(LanError::UpdateMirrorError(format!(
                    "Host returned status {status}"
                )));
            }

            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .map_err(|e| LanError::Database(e.to_string()))?;

            // 1. Check file size
            if manifest.file_size_bytes > 0 && body_bytes.len() as i64 != manifest.file_size_bytes {
                return Err(LanError::UpdateMirrorError(format!(
                    "Downloaded size {} does not match manifest {}",
                    body_bytes.len(),
                    manifest.file_size_bytes
                )));
            }

            // 2. Check SHA-256
            let computed_sha256 = super::mirror::compute_sha256(&body_bytes);
            if !computed_sha256.eq_ignore_ascii_case(&manifest.sha256) {
                if target_path.exists() {
                    let _ = std::fs::remove_file(target_path);
                }
                return Err(LanError::UpdateChecksumMismatch {
                    expected: manifest.sha256.clone(),
                    computed: computed_sha256,
                });
            }

            // 3. Check Ed25519 signature
            if let Err(e) = super::mirror::verify_staged_update_signature(
                &body_bytes,
                &computed_sha256,
                &manifest.updater_signature,
                &manifest.updater_key_id,
            ) {
                if target_path.exists() {
                    let _ = std::fs::remove_file(target_path);
                }
                return Err(e);
            }

            if let Some(parent) = target_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(target_path, &body_bytes).map_err(|e| {
                LanError::UpdateMirrorError(format!("Failed to write update target file: {e}"))
            })?;

            Ok(target_path.to_path_buf())
        })
    }
}

// -----------------------------------------------------------------------------
// Mobile Client Helpers (AC-8, AC-11)
// -----------------------------------------------------------------------------

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
