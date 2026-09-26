# LAN Workstation Client Mode and RPC Transport Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Workstation Client Mode runtime and RPC Transport (Spec 0047, Feature 71), enabling secondary Windows workstations to operate without a local database by forwarding domain operations over pinned TLS 1.3 to the primary host, syncing state via real-time SSE event fan-out, and strictly isolating physical printing to local counter hardware.

**Architecture:** A dual-mode RPC bridge routes local peripheral and diagnostics commands to local workstation handlers while forwarding domain queries/mutations to the host via pinned TLS 1.3 (`POST /rpc`) with device bearer tokens, staff session tokens, and UUIDv7 idempotency keys. Real-time SSE (`GET /events`) synchronizes TanStack Query caches across counters (<500ms) and enforces instant remote severance.

**Tech Stack:** Rust 1.85+, Tauri 2, Axum 0.8, Tokio, tokio-rustls, rustls, mDNS-SD, React 19, TypeScript 5.7+, Tailwind CSS v4.

---

### Task 1: Core Data Models, Types, and DPAPI Credential Handling

**Files:**
- Modify: `crates/core/src/lan.rs`
- Modify: `crates/core/src/lib.rs`
- Test: `crates/core/tests/workstation_client_mode_integration.rs`

**Step 1: Write integration tests for data structures and local credential resolution**
Define test cases asserting:
- `WorkstationCredentials` stores and serializes `host_name` and `workstation_name`.
- `WorkstationClientStatus` serialization and state machine states.
- DPAPI save/load/clear with `COACHIFY_TEST_WORKSTATION_CREDS_PATH` isolation.

**Step 2: Implement types and credentials helpers in `crates/core/src/lan.rs`**
Add:
- `WorkstationConnectionState`
- `WorkstationClientStatus`
- `ReconnectWorkstationResponse`
- `TestConnectionResponse`
- `ForwardedRpcRequest` and `ForwardedRpcResponse`
- Update `DeviceControlEvent::Change { events: Vec<ChangeEvent> }`
- Update `WorkstationCredentials` with `host_name` and `workstation_name`
- Support `COACHIFY_TEST_WORKSTATION_CREDS_PATH` in `resolve_workstation_credentials_path`

---

### Task 2: Host RPC Handler & Device Token Gate (`POST /rpc` & `GET /events`)

**Files:**
- Modify: `crates/core/src/lan.rs`
- Test: `crates/core/tests/workstation_client_mode_integration.rs`

**Step 1: Enhance `rpc_handler` in `lan.rs`**
- Validate `X-Coachify-Device-Token` header.
- Reject revoked tokens with HTTP 403 `DEVICE_REVOKED`.
- Reject blocked tokens with HTTP 403 `DEVICE_BLOCKED`.
- Update `devices.last_seen_at` and `devices.ip_address`.
- Extract `idempotency_key` (header or payload) and pass to `dispatch_at`.
- Extract `session_token` (header or payload) and pass to `dispatch_at`.
- On mutation producing `ChangeEvent`s, broadcast to `control_sender()`.
- Return `ForwardedRpcResponse` with `events` and monotonic server time.

**Step 2: Enhance `events_handler` in `lan.rs`**
- Stream `coachify_change` SSE events when `DeviceControlEvent::Change` is broadcast.
- Stream `device.revoked` and `device.blocked` events.

---

### Task 3: Workstation Client Transport & State Manager

**Files:**
- Modify: `crates/core/src/lan.rs`
- Test: `crates/core/tests/workstation_client_mode_integration.rs`

**Step 1: Implement `WorkstationClientManager` in `lan.rs`**
- State management: `WorkstationConnectionState`, `latency_ms`, `last_synced_at`, `reconnect_attempts`.
- `forward_rpc`:
  - Connects to host via pinned TLS 1.3 with `FingerprintPinnedCertVerifier`.
  - Injects `X-Coachify-Device-Token`, `Authorization: Bearer <session>`, `X-Coachify-Idempotency-Key`.
  - Parses `ForwardedRpcResponse`.
  - Handles auto-wipe on `DEVICE_REVOKED`.
- `test_connection`:
  - Pings `/health` and measures RTT latency in milliseconds.
- `reconnect`:
  - Probes cached IP:port, falls back to dynamic mDNS query (`_coachify._tcp.local`) for matching `host_id` and fingerprint.
- `disconnect_workstation`:
  - Wipes credentials and resets state.
- `start_sse_listener`:
  - Background SSE loop dispatching events to registered handler.

---

### Task 4: Dual-Mode Dispatcher & Core Bridge Routing

**Files:**
- Modify: `crates/core/src/lib.rs`
- Modify: `crates/core/src/dispatcher.rs`
- Test: `crates/core/tests/workstation_client_mode_integration.rs`

**Step 1: Wire `handle_rpc_with_events` for dual-mode execution**
- If `is_workstation_client_mode()`:
  - Local commands (`printers.*`, `lan.get_client_status`, `lan.reconnect`, `lan.disconnect_workstation`, `lan.test_connection`, `setup.get_status`, `system.ping`, `system.get_client_info`) execute locally without database.
  - Remote commands forward over TLS 1.3 to Host.
- Add dispatcher handlers for `lan.get_client_status`, `lan.reconnect`, `lan.disconnect_workstation`, `lan.test_connection`.

---

### Task 5: Strictly Local Document & Receipt Printing Verification

**Files:**
- Modify/Verify: `crates/core/src/dispatcher.rs`
- Modify/Verify: `ui/src/lib/printer.ts`
- Test: `crates/core/tests/workstation_client_mode_integration.rs`

**Step 1: Ensure receipt print flow isolates local printers**
- `receipts.get_printable` is remote (fetches itemized receipt from host SQLite).
- `printers.get_settings` and `printers.list` are local.
- Thermal and A4 printing render and print locally on workstation machine spooler.

---

### Task 6: Comprehensive Automated Integration Test Suite

**Files:**
- Create: `crates/core/tests/workstation_client_mode_integration.rs`

Implement all 10 verification scenarios specified in `docs/specs/0047-lan-workstation-client-mode-and-rpc-transport/verify.md`:
1. `test_transparent_dual_mode_rpc_dispatch` (AC-1)
2. `test_pinned_tls_transport_and_mitm_rejection` (AC-2, AC-3)
3. `test_device_token_authentication_and_session_layering` (AC-3, AC-4)
4. `test_strictly_local_printing_isolation` (AC-5)
5. `test_realtime_multi_counter_sse_cache_sync` (AC-6)
6. `test_in_flight_mutation_idempotency_recovery` (AC-8)
7. `test_dynamic_mdns_host_ip_shift_recovery` (AC-7)
8. `test_instant_remote_revocation_lockout_latency` (AC-9)
9. `test_multi_counter_concurrency_stress` (AC-4, AC-6)
10. `test_workstation_health_and_latency_monitoring` (AC-10)

---

### Task 7: UI Workstation Header Pill, Reconnection Banner, Settings & Dialogs

**Files:**
- Create: `ui/src/components/layout/workstation-pill.tsx`
- Create: `ui/src/components/layout/reconnection-banner.tsx`
- Create: `ui/src/components/auth/workstation-lockout-overlay.tsx`
- Create: `ui/src/components/settings/workstation-settings-card.tsx`
- Create: `ui/src/components/setup/workstation-pair-dialog.tsx`
- Modify: `ui/src/components/layout/top-bar.tsx`
- Modify: `ui/src/components/auth/startup-auth-page.tsx`
- Modify: `ui/src/App.tsx`
- Modify: `ui/src/lib/i18n.ts`
- Sync: `cargo run -p xtask export-types`
- Test: `ui/src/components/layout/workstation-pill.test.tsx`
