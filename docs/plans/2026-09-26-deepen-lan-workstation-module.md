# Deepen LAN Workstation Module Behind Host and Client Seams Implementation Plan

> **For Agent:** Implement this plan task-by-task. Maintain strict type safety, zero warnings (`-D warnings`), and test-driven verification.

**Goal:** Refactor the monolithic `crates/core/src/lan.rs` (4,770 lines) into a deep module (`crates/core/src/lan/`) exposing only two clean seams (`HostController` and `TransportClient`), encapsulating internal networking, cryptography, and Win32 sleep inhibition, backed by production TLS and in-memory test adapters (`tower::ServiceExt::oneshot`).

**Domain Alignment:** Uses `CONTEXT.md` terms: `Host Mode`, `Workstation Client Mode`, `Command Dispatcher`, `Command Locality`, `Host Controller`, `Transport Client`, `Lan Runtime`.

---

### Task 1: Module Decomposition & Shared Protocol / Crypto / Power / mDNS Extraction

**Files:**
- Create: `crates/core/src/lan/protocol.rs` (DTOs, error types, change event schemas, QR formatting)
- Create: `crates/core/src/lan/crypto.rs` (pinned cert verifier, rustls config, self-signed X.509 generation, fingerprint calculation)
- Create: `crates/core/src/lan/power.rs` (Win32 `SetThreadExecutionState` guard)
- Create: `crates/core/src/lan/mdns.rs` (mDNS advertiser and browser)
- Create: `crates/core/src/lan/mod.rs` (re-export seams, types, and DTOs)

**Steps:**
1. Extract shared types and DTOs into `crates/core/src/lan/protocol.rs` (`LanHostStatus`, `WorkstationClientStatus`, `PairSuccessResponse`, `ParsedPairingQrUri`, `ForwardedRpcRequest`, `ForwardedRpcResponse`, etc.).
2. Extract crypto routines into `crates/core/src/lan/crypto.rs` (`FingerprintPinnedCertVerifier`, `generate_self_signed_cert`, `compute_sha256_fingerprint`).
3. Extract power management into `crates/core/src/lan/power.rs` (`set_sleep_inhibition`).
4. Extract mDNS helpers into `crates/core/src/lan/mdns.rs` (`advertise_host`, `discover_hosts_internal`).

---

### Task 2: HostController Deep Interface & Axum Listener

**Files:**
- Create: `crates/core/src/lan/host.rs`
- Modify: `crates/core/src/lan/mod.rs`

**Steps:**
1. Implement `HostController`:
   - Methods: `start`, `stop`, `status`, `open_pairing_window`, `close_pairing_window`, `open_mobile_pairing_window`, `list_workstations`, `get_workstation`, `rename_workstation`, `block_workstation`, `unblock_workstation`, `revoke_workstation`, `list_device_events`, `broadcast_change`.
   - Dedicated connection factory: accept database path or connection provider so background Axum HTTP request worker threads have isolated SQLite access without contending with UI threads.
   - Axum router builder: encapsulate `/hello`, `/health`, `/pair`, `/mobile/pair`, `/rpc`, `/events`, `/receipts/printable`.
   - Expose `router()` method on `HostController` (or internal to `lan`) for in-memory testing.

---

### Task 3: TransportClient Deep Interface & Dual Adapters (TLS & InMemory)

**Files:**
- Create: `crates/core/src/lan/client.rs`
- Modify: `crates/core/src/lan/mod.rs`

**Steps:**
1. Define `TransportClient` trait:
   - `pair(&self, host_address: &str, pin: &str, workstation_name: &str) -> Result<PairSuccessResponse, LanError>`
   - `forward(&self, command: &str, payload: &Value, idempotency_key: Option<&str>, session_token: Option<&str>) -> Result<ForwardedRpcResponse, RpcError>`
   - `status(&self) -> WorkstationClientStatus`
   - `reconnect(&self) -> Result<ReconnectWorkstationResponse, LanError>`
   - `disconnect(&self) -> Result<(), LanError>`
   - `test_connection(&self) -> Result<TestConnectionResponse, LanError>`
   - `subscribe_events(&self, callback: Box<dyn Fn(Vec<ChangeEvent>) + Send + Sync + 'static>) -> Result<EventSubscriptionGuard, LanError>`
2. Implement `TlsClientTransport` (Production adapter):
   - Encapsulates DPAPI credentials storage (`host_name`, `device_token`, `cert_fingerprint`, etc.).
   - Connects to Host over pinned TLS 1.3.
   - Background SSE listener loop with exponential backoff and dynamic mDNS recovery.
3. Implement `InMemoryClientTransport` (Test adapter):
   - Accepts an Axum `Router` and uses `tower::ServiceExt::oneshot` to route requests in-memory.
   - Change events delivered via in-memory broadcast channel.
   - Zero TCP sockets, zero DPAPI locking, enabling fast, isolated parallel tests.

---

### Task 4: LanRuntime & Command Dispatcher Delegation with Command Locality

**Files:**
- Modify: `crates/core/src/lan/mod.rs`
- Modify: `crates/core/src/dispatcher.rs`

**Steps:**
1. Implement `LanRuntime` in `lan/mod.rs`:
   - Holds `Option<HostController>` and `Option<Arc<dyn TransportClient>>`.
   - Exposes `dispatch(command, payload, conn, current_utc, actor_id)` handling all `lan.*` commands across a single seam.
2. In `dispatcher.rs`:
   - Define `CommandLocality` enum: `HostOnly`, `LocalOnly`, `RemoteForwardable`.
   - Map commands to locality.
   - When in Workstation Client Mode:
     - `LocalOnly` -> executes locally on workstation (printers, local settings).
     - `RemoteForwardable` -> automatically forwards through `client.forward(...)`.
     - `HostOnly` -> returns error or blocked.
   - Delegate all `lan.*` match arms to `LanRuntime::dispatch`.

---

### Task 5: Core Re-Exports & Tauri Shell Wiring

**Files:**
- Modify: `crates/core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Update `crates/core/src/lib.rs`: export `lan::{HostController, TransportClient, TlsClientTransport, InMemoryClientTransport, LanRuntime, ...}`.
2. Update `src-tauri/src/lib.rs`: connect Tauri's event listener and shutdown hooks to `LanRuntime`.

---

### Task 6: Test Suite Migration ("Replace, Don't Layer") & Verification

**Files:**
- Delete old monolithic `crates/core/src/lan.rs`.
- Update: `crates/core/tests/lan_host_integration.rs`
- Update: `crates/core/tests/workstation_pairing_integration.rs`
- Update: `crates/core/tests/lan_mobile_pairing_integration.rs`
- Update: `crates/core/tests/workstation_client_mode_integration.rs`

**Steps:**
1. Refactor tests to exercise `HostController` and `TransportClient` interfaces directly.
2. Replace socket-heavy tests with `InMemoryClientTransport` tests where appropriate; keep end-to-end TLS test for `TlsClientTransport`.
3. Eliminate `SERIAL_TEST_MUTEX`.
4. Run `cargo test -p coachify-core --test workstation_client_mode_integration`.
5. Run full workspace tests `cargo test --workspace`.
6. Run `cargo run -p xtask export-types` and `cargo run -p xtask check-types`.
