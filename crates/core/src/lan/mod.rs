pub mod client;
pub mod crypto;
pub mod host;
pub mod mdns;
pub mod power;
pub mod protocol;

pub use client::{
    clear_workstation_credentials, load_workstation_credentials, race_connect_candidate_ips,
    resolve_workstation_credentials_path, save_workstation_credentials, InMemoryClientTransport,
    TlsClientTransport, TransportClient,
};
pub use crypto::{
    compute_sha256_fingerprint, ensure_crypto_provider_installed, generate_self_signed_cert,
    FingerprintPinnedCertVerifier,
};
pub use host::{ping_host, HostController, RateLimiter};
pub use mdns::discover_hosts;
pub use power::set_sleep_inhibition;
pub use protocol::*;

use crate::dispatcher::{ChangeEvent, RpcError};
use rusqlite::Connection;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub struct LanRuntime {
    host: HostController,
    client: RwLock<Arc<dyn TransportClient>>,
}

static RUNTIME: OnceLock<LanRuntime> = OnceLock::new();

pub fn lan_runtime() -> &'static LanRuntime {
    RUNTIME.get_or_init(LanRuntime::new)
}

impl Default for LanRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl LanRuntime {
    pub fn new() -> Self {
        Self {
            host: HostController::new(),
            client: RwLock::new(Arc::new(TlsClientTransport::new())),
        }
    }

    pub fn host(&self) -> &HostController {
        &self.host
    }

    pub fn client(&self) -> Arc<dyn TransportClient> {
        self.client.read().unwrap().clone()
    }

    pub fn set_client(&self, client: Arc<dyn TransportClient>) {
        *self.client.write().unwrap() = client;
    }

    pub fn dispatch(
        &self,
        command: &str,
        payload: &serde_json::Value,
        conn: Option<&Connection>,
        current_utc: &str,
        actor_id: Option<&str>,
    ) -> Result<serde_json::Value, RpcError> {
        let map_err = |e: LanError| RpcError {
            code: e.code().to_string(),
            message: e.to_string(),
            field: None,
        };

        let require_conn = || -> Result<&Connection, RpcError> {
            conn.ok_or_else(|| RpcError {
                code: "DB_REQUIRED".to_string(),
                message: "Database connection required for this host command".to_string(),
                field: None,
            })
        };

        match command {
            "lan.get_host_status" => {
                let conn = require_conn()?;
                let status = self
                    .host
                    .get_host_status(conn, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(status).unwrap())
            }
            "lan.start_host" => {
                let conn = require_conn()?;
                let p: StartHostPayload =
                    serde_json::from_value(payload.clone()).unwrap_or(StartHostPayload {
                        port: None,
                        bind_address: None,
                    });
                let status = self
                    .host
                    .start(conn, p.port, p.bind_address, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(status).unwrap())
            }
            "lan.stop_host" => {
                let conn = require_conn()?;
                let status = self.host.stop(conn, current_utc).map_err(map_err)?;
                Ok(serde_json::to_value(status).unwrap())
            }
            "lan.update_host_config" => {
                let conn = require_conn()?;
                let p: UpdateHostConfigPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let status = self
                    .host
                    .update_host_config(conn, p, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(status).unwrap())
            }
            "lan.get_firewall_guidance" => {
                let guidance = self.host.get_firewall_guidance();
                Ok(serde_json::to_value(guidance).unwrap())
            }
            "lan.discover_hosts" => {
                let p: DiscoverHostsPayload = serde_json::from_value(payload.clone())
                    .unwrap_or(DiscoverHostsPayload { timeout_ms: None });
                let hosts = discover_hosts(p.timeout_ms).map_err(map_err)?;
                Ok(serde_json::to_value(hosts).unwrap())
            }
            "lan.ping_host" => {
                let p: PingHostPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let res = ping_host(p).map_err(map_err)?;
                Ok(serde_json::to_value(res).unwrap())
            }
            "lan.open_pairing_window" => {
                let conn = require_conn()?;
                let p: OpenPairingWindowPayload = serde_json::from_value(payload.clone())
                    .unwrap_or(OpenPairingWindowPayload {
                        duration_seconds: None,
                    });
                let info = self
                    .host
                    .open_pairing_window(
                        conn,
                        p.duration_seconds,
                        actor_id.unwrap_or("system"),
                        current_utc,
                    )
                    .map_err(map_err)?;
                Ok(serde_json::to_value(info).unwrap())
            }
            "lan.close_pairing_window" => {
                let conn = require_conn()?;
                self.host
                    .close_pairing_window(conn, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::json!({ "success": true }))
            }
            "lan.list_workstations" => {
                let conn = require_conn()?;
                let list = self
                    .host
                    .list_workstations(conn, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(list).unwrap())
            }
            "lan.get_workstation" => {
                let conn = require_conn()?;
                let p: GetWorkstationPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let detail = self
                    .host
                    .get_workstation(conn, &p.device_id, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(detail).unwrap())
            }
            "lan.rename_workstation" => {
                let conn = require_conn()?;
                let p: RenameWorkstationPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let updated = self
                    .host
                    .rename_workstation(conn, &p.device_id, &p.new_name, actor_id, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(updated).unwrap())
            }
            "lan.block_workstation" => {
                let conn = require_conn()?;
                let p: BlockWorkstationPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let updated = self
                    .host
                    .block_workstation(
                        conn,
                        &p.device_id,
                        p.reason.as_deref(),
                        actor_id,
                        current_utc,
                    )
                    .map_err(map_err)?;
                Ok(serde_json::to_value(updated).unwrap())
            }
            "lan.unblock_workstation" => {
                let conn = require_conn()?;
                let p: UnblockWorkstationPayload = serde_json::from_value(payload.clone())
                    .map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let updated = self
                    .host
                    .unblock_workstation(conn, &p.device_id, actor_id, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(updated).unwrap())
            }
            "lan.revoke_workstation" => {
                let conn = require_conn()?;
                let p: RevokeWorkstationPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let updated = self
                    .host
                    .revoke_workstation(
                        conn,
                        &p.device_id,
                        p.reason.as_deref(),
                        actor_id,
                        current_utc,
                    )
                    .map_err(map_err)?;
                Ok(serde_json::to_value(updated).unwrap())
            }
            "lan.list_device_events" => {
                let conn = require_conn()?;
                let p: ListDeviceEventsPayload =
                    serde_json::from_value(payload.clone()).unwrap_or(ListDeviceEventsPayload {
                        device_id: None,
                        limit: None,
                    });
                let events = self
                    .host
                    .list_device_events(conn, p.device_id.as_deref(), p.limit)
                    .map_err(map_err)?;
                Ok(serde_json::to_value(events).unwrap())
            }
            "lan.open_mobile_pairing_window" => {
                let conn = require_conn()?;
                let p: OpenPairingWindowPayload = serde_json::from_value(payload.clone())
                    .unwrap_or(OpenPairingWindowPayload {
                        duration_seconds: None,
                    });
                let info = self
                    .host
                    .open_mobile_pairing_window(
                        conn,
                        p.duration_seconds,
                        actor_id.unwrap_or("system"),
                        current_utc,
                    )
                    .map_err(map_err)?;
                Ok(serde_json::to_value(info).unwrap())
            }
            "lan.update_host_receipt_printer" => {
                let conn = require_conn()?;
                let p: UpdateHostReceiptPrinterPayload = serde_json::from_value(payload.clone())
                    .map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                self.host
                    .update_host_receipt_printer(conn, p.printer_name, current_utc)
                    .map_err(map_err)?;
                Ok(serde_json::json!({ "success": true }))
            }
            "lan.set_staff_pin" => {
                let conn = require_conn()?;
                let p: SetStaffPinPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                self.host
                    .set_user_pin(conn, &p.user_id, &p.staff_pin)
                    .map_err(map_err)?;
                Ok(serde_json::json!({ "success": true }))
            }
            "lan.pair_with_host" => {
                let p: ClientPairPayload =
                    serde_json::from_value(payload.clone()).map_err(|e| RpcError {
                        code: "validation_error".to_string(),
                        message: e.to_string(),
                        field: None,
                    })?;
                let res = self.client().pair(p).map_err(map_err)?;
                Ok(serde_json::to_value(res).unwrap())
            }
            "lan.get_workstation_credentials" => {
                let creds = self.client().get_credentials();
                Ok(serde_json::to_value(creds).unwrap())
            }
            "lan.clear_workstation_credentials" | "lan.disconnect_workstation" => {
                self.client().disconnect().map_err(map_err)?;
                Ok(serde_json::json!({ "success": true }))
            }
            "lan.get_client_status" => {
                let status = self.client().get_status();
                Ok(serde_json::to_value(status).unwrap())
            }
            "lan.reconnect" => {
                let res = self.client().reconnect_sync().map_err(map_err)?;
                Ok(serde_json::to_value(res).unwrap())
            }
            "lan.test_connection" => {
                let res = self.client().test_connection_sync().map_err(map_err)?;
                Ok(serde_json::to_value(res).unwrap())
            }
            _ => Err(RpcError {
                code: "UNKNOWN_COMMAND".to_string(),
                message: format!("Unknown LAN command: {}", command),
                field: None,
            }),
        }
    }
}

// -----------------------------------------------------------------------------
// Compatibility Facade Functions
// -----------------------------------------------------------------------------

pub fn lan_manager() -> &'static HostController {
    lan_runtime().host()
}

pub fn set_test_db_conn(conn: Option<Arc<Mutex<Connection>>>) {
    lan_runtime().host().set_test_db_conn(conn);
}

pub fn workstation_client_manager() -> Arc<dyn TransportClient> {
    lan_runtime().client()
}

pub fn is_workstation_client_mode() -> bool {
    lan_runtime().client().is_client_mode()
}

pub fn reload_workstation_credentials() -> Option<WorkstationCredentials> {
    lan_runtime().client().reload_credentials()
}

pub fn get_workstation_client_status() -> WorkstationClientStatus {
    lan_runtime().client().get_status()
}

pub fn reconnect_workstation_client() -> Result<ReconnectWorkstationResponse, LanError> {
    lan_runtime().client().reconnect_sync()
}

pub fn disconnect_workstation_client() -> Result<(), LanError> {
    lan_runtime().client().disconnect()
}

pub fn test_workstation_connection() -> Result<TestConnectionResponse, LanError> {
    lan_runtime().client().test_connection_sync()
}

pub fn pair_with_host(payload: ClientPairPayload) -> Result<PairSuccessResponse, LanError> {
    lan_runtime().client().pair(payload)
}

pub fn forward_workstation_rpc(
    command: &str,
    payload_json: &str,
    session_token: Option<&str>,
    idempotency_key: Option<&str>,
) -> Result<(String, Vec<ChangeEvent>), LanError> {
    lan_runtime()
        .client()
        .forward_rpc_sync(command, payload_json, session_token, idempotency_key)
}

pub fn start_workstation_sse_background<F>(on_change: F)
where
    F: Fn(Vec<ChangeEvent>) + Send + Sync + 'static,
{
    let _ = lan_runtime()
        .client()
        .start_sse_stream_sync(Box::new(on_change));
}

pub fn start_host(
    conn: &Connection,
    port_override: Option<u16>,
    bind_override: Option<String>,
    current_utc: &str,
) -> Result<LanHostStatus, LanError> {
    lan_runtime()
        .host()
        .start(conn, port_override, bind_override, current_utc)
}

pub fn stop_host(conn: &Connection, current_utc: &str) -> Result<LanHostStatus, LanError> {
    lan_runtime().host().stop(conn, current_utc)
}

pub fn get_host_status(conn: &Connection, current_utc: &str) -> Result<LanHostStatus, LanError> {
    lan_runtime().host().get_host_status(conn, current_utc)
}

pub fn open_pairing_window(
    conn: &Connection,
    duration_seconds: Option<u64>,
    actor_user_id: &str,
    current_utc: &str,
) -> Result<PairingWindowInfo, LanError> {
    lan_runtime()
        .host()
        .open_pairing_window(conn, duration_seconds, actor_user_id, current_utc)
}

pub fn close_pairing_window(conn: &Connection, current_utc: &str) -> Result<(), LanError> {
    lan_runtime().host().close_pairing_window(conn, current_utc)
}

pub fn open_mobile_pairing_window(
    conn: &Connection,
    duration_seconds: Option<u64>,
    actor_user_id: &str,
    current_utc: &str,
) -> Result<MobilePairingWindowInfo, LanError> {
    lan_runtime().host().open_mobile_pairing_window(
        conn,
        duration_seconds,
        actor_user_id,
        current_utc,
    )
}

pub fn list_workstations(
    conn: &Connection,
    current_utc: &str,
) -> Result<WorkstationListResponse, LanError> {
    lan_runtime().host().list_workstations(conn, current_utc)
}

pub fn get_workstation(
    conn: &Connection,
    device_id: &str,
    current_utc: &str,
) -> Result<WorkstationDetail, LanError> {
    lan_runtime()
        .host()
        .get_workstation(conn, device_id, current_utc)
}

pub fn rename_workstation(
    conn: &Connection,
    device_id: &str,
    new_name: &str,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    lan_runtime()
        .host()
        .rename_workstation(conn, device_id, new_name, actor_user_id, current_utc)
}

pub fn block_workstation(
    conn: &Connection,
    device_id: &str,
    reason: Option<&str>,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    lan_runtime()
        .host()
        .block_workstation(conn, device_id, reason, actor_user_id, current_utc)
}

pub fn unblock_workstation(
    conn: &Connection,
    device_id: &str,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    lan_runtime()
        .host()
        .unblock_workstation(conn, device_id, actor_user_id, current_utc)
}

pub fn revoke_workstation(
    conn: &Connection,
    device_id: &str,
    reason: Option<&str>,
    actor_user_id: Option<&str>,
    current_utc: &str,
) -> Result<WorkstationSummary, LanError> {
    lan_runtime()
        .host()
        .revoke_workstation(conn, device_id, reason, actor_user_id, current_utc)
}

pub fn list_device_events(
    conn: &Connection,
    device_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<DeviceEventItem>, LanError> {
    lan_runtime()
        .host()
        .list_device_events(conn, device_id, limit)
}

pub fn update_host_receipt_printer(
    conn: &Connection,
    printer_name: Option<String>,
    current_utc: &str,
) -> Result<(), LanError> {
    lan_runtime()
        .host()
        .update_host_receipt_printer(conn, printer_name, current_utc)
}

pub fn set_user_pin(conn: &Connection, user_id: &str, pin: &str) -> Result<(), LanError> {
    lan_runtime().host().set_user_pin(conn, user_id, pin)
}

pub fn update_host_config(
    conn: &Connection,
    payload: UpdateHostConfigPayload,
    current_utc: &str,
) -> Result<LanHostStatus, LanError> {
    lan_runtime()
        .host()
        .update_host_config(conn, payload, current_utc)
}

pub fn get_or_create_host_config(
    conn: &Connection,
    current_utc: &str,
) -> Result<LanHostConfig, LanError> {
    lan_runtime()
        .host()
        .get_or_create_host_config(conn, current_utc)
}

pub fn get_firewall_guidance() -> FirewallGuidance {
    lan_runtime().host().get_firewall_guidance()
}

pub fn broadcast_change_events(events: Vec<ChangeEvent>) {
    lan_runtime().host().broadcast_change_events(events);
}

pub fn control_sender() -> tokio::sync::broadcast::Sender<DeviceControlEvent> {
    lan_runtime().host().control_sender()
}

pub fn broadcast_control_event(event: DeviceControlEvent) {
    lan_runtime().host().broadcast_control_event(event);
}

pub fn get_client_info() -> ClientInfoResponse {
    ClientInfoResponse {
        hostname: get_system_hostname(),
        os: std::env::consts::OS.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        mode: if is_workstation_client_mode() {
            "workstation_client".to_string()
        } else {
            "standalone".to_string()
        },
    }
}
