# LAN Host Update Mirror Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Maintain strict type safety, zero warnings (`-D warnings`), and test-driven verification.

**Goal:** Implement Feature 72 (Spec 0048) enabling the primary Host PC to mirror verified Coachify Desktop application updates over the local network (LAN) so secondary counter workstations can discover, download, and install updates without internet access.

**Architecture:** Extend the Host Controller's Axum router with authenticated TLS 1.3 streaming endpoints (`/updates/manifest`, `/updates/download/:version`), backing storage in `%LOCALAPPDATA%\Coachify\update_mirror\`, SQLite tracking table `lan_staged_updates`, real-time SSE change event push (`system:update_available`), and client-side cryptographic Ed25519 signature & SHA-256 verification.

**Tech Stack:** Rust 1.85+, SQLite (rusqlite), Axum, Tokio, rustls, ed25519-dalek, sha2, TypeScript, React 19, Tailwind CSS v4, TanStack Query, Vitest.

---

### Task 1: Database Migration 0027 & Protocol DTOs

**Files:**
- Create: `migrations/0027_lan_update_mirror.sql`
- Modify: `crates/core/src/lan/protocol.rs`
- Modify: `crates/core/src/lan/mod.rs`

**Steps:**
1. Create `migrations/0027_lan_update_mirror.sql` with `lan_update_mirror_config` and `lan_staged_updates` tables with audit trigger.
2. In `crates/core/src/lan/protocol.rs`, add types:
   - `LanUpdateMirrorConfig`
   - `LanUpdateMirrorStatus`
   - `HostUpdateManifest`
   - `StageMirrorUpdateInput`
   - `WorkstationUpdateCheckResult`
   - `WorkstationUpdateDownloadResult`
3. Export new types in `crates/core/src/lan/mod.rs` and verify compilation.

---

### Task 2: Host Update Mirror Engine (`crates/core/src/lan/mirror.rs`)

**Files:**
- Create: `crates/core/src/lan/mirror.rs`
- Modify: `crates/core/src/lan/mod.rs`
- Modify: `crates/core/src/lib.rs`

**Steps:**
1. Implement mirror storage operations:
   - `get_or_create_mirror_config(conn: &Connection) -> Result<LanUpdateMirrorConfig, LanError>`
   - `get_mirror_status(conn: &Connection, base_dir: &Path) -> Result<LanUpdateMirrorStatus, LanError>`
   - `configure_mirror(conn: &Connection, enabled: bool) -> Result<LanUpdateMirrorStatus, LanError>`
   - `stage_update(conn: &Connection, base_dir: &Path, input: &StageMirrorUpdateInput, payload_bytes: &[u8]) -> Result<HostUpdateManifest, LanError>`
   - `clear_staged_update(conn: &Connection, base_dir: &Path, version: Option<&str>) -> Result<(), LanError>`
   - `get_active_manifest(conn: &Connection) -> Result<Option<HostUpdateManifest>, LanError>`
   - `get_staged_file_path(conn: &Connection, base_dir: &Path, version: &str) -> Result<PathBuf, LanError>`
   - `record_download_completed(conn: &Connection, version: &str) -> Result<u32, LanError>`
2. Cryptographic signature and SHA-256 verification on staged payload:
   - Compute SHA-256 hash and compare.
   - Verify Ed25519 signature against embedded release public key.
3. Add unit tests for staging, manifest generation, signature verification, and clearing.

---

### Task 3: Axum Streaming Endpoints on HostController (`crates/core/src/lan/host.rs`)

**Files:**
- Modify: `crates/core/src/lan/host.rs`

**Steps:**
1. Add routes to `HostController::router`:
   - `.route("/updates/manifest", get(update_manifest_handler))`
   - `.route("/updates/download/:version", get(update_download_handler))`
2. In `update_manifest_handler`:
   - Validate `X-Coachify-Device-Token` header.
   - Query active manifest from mirror engine. If none active, return 404 Not Found.
   - Return 200 OK with `HostUpdateManifest` JSON.
3. In `update_download_handler`:
   - Validate `X-Coachify-Device-Token` header.
   - Verify device is active (not blocked or revoked).
   - Locate staged binary file on disk.
   - Stream file bytes with `Content-Type: application/vnd.microsoft.portable-executable`, `Content-Length`, and `Content-Disposition`.
   - Increment download count in DB and emit audit log entry.
4. When `stage_update` or `clear_staged_update` is called, broadcast a `ChangeEvent` (`topic: "system"`, `entity: "update"`, `action: "mirrored"`) down the SSE stream.

---

### Task 4: Workstation Client Transport Extension (`crates/core/src/lan/client.rs`)

**Files:**
- Modify: `crates/core/src/lan/client.rs`

**Steps:**
1. Extend `TransportClient` trait with:
   - `check_host_update_sync(&self) -> Result<Option<HostUpdateManifest>, LanError>`
   - `download_host_update_sync(&self, version: &str, target_path: &Path) -> Result<PathBuf, LanError>`
2. In `TlsClientTransport`:
   - `check_host_update_sync`: Sends GET to `https://{host}:{port}/updates/manifest` with pinned certificate and device token.
   - `download_host_update_sync`: Streams response from `https://{host}:{port}/updates/download/{version}` to temporary file, verifies SHA-256 checksum and Ed25519 signature against vendor public key, then renames to target path.
3. In `InMemoryClientTransport`:
   - Implement mock routes for in-memory testing.

---

### Task 5: Command Dispatcher & Seam Registration

**Files:**
- Modify: `crates/core/src/dispatcher.rs`
- Modify: `crates/core/src/seam.rs`
- Modify: `crates/core/src/lan/mod.rs`

**Steps:**
1. Add commands to `dispatcher.rs`:
   - `lan.get_update_mirror_status` (Locality: `HostOnly`)
   - `lan.configure_update_mirror` (Locality: `HostOnly`)
   - `lan.stage_mirror_update` (Locality: `HostOnly`)
   - `lan.clear_mirror_update` (Locality: `HostOnly`)
   - `lan.check_workstation_update` (Locality: `LocalOnly`)
   - `lan.download_workstation_update` (Locality: `LocalOnly`)
2. Connect `LanRuntime::dispatch` to handle the new `lan.*` commands.
3. Register types in `crates/core/src/seam.rs`.
4. Run `cargo run -p xtask export-types` to update `ui/src/gen/rpc.ts`.

---

### Task 6: Automated Integration Test Suite (`crates/core/tests/lan_update_mirror_integration.rs`)

**Files:**
- Create: `crates/core/tests/lan_update_mirror_integration.rs`

**Steps:**
1. Implement integration tests:
   - `test_mirror_cache_staging_and_db_tracking`
   - `test_host_update_endpoints_and_auth`
   - `test_mirror_staged_emits_sse_change_event`
   - `test_workstation_update_check_and_download`
   - `test_download_cryptographic_verification_and_tamper_rejection`
   - `test_air_gapped_manual_file_staging`
   - `test_configure_mirror_and_status_reporting`
   - `test_update_mirror_audit_trail_logging`
2. Run `cargo test --test lan_update_mirror_integration`.

---

### Task 7: UI Implementation (Host Mirror Card & Workstation Update Card)

**Files:**
- Modify: `ui/src/components/settings/lan-host-settings-card.tsx`
- Modify: `ui/src/components/settings/update-settings.tsx`
- Modify: `ui/src/components/settings/workstation-settings-card.tsx`
- Create / Update tests:
  - `ui/src/components/settings/lan-host-settings-card.test.tsx`
  - `ui/src/components/settings/update-settings.test.tsx`

**Steps:**
1. In `LanHostSettingsCard`:
   - Add "LAN Update Mirror" section showing toggle switch, staged version, size, download count, and "Stage from File" modal.
2. In `UpdateSettingsCard`:
   - In Workstation Client Mode: detect paired host, show "Update Source: Host PC ({host_name}) via LAN", "Check Host for Updates" button, progress bar, and "Download from Host" button.
3. Update Vitest unit tests and ensure 100% pass rate.

---

### Task 8: Verification & Quality Gates

**Steps:**
1. Run `cargo test --workspace`.
2. Run `cargo run -p xtask check-types`.
3. Run `cargo run -p xtask check-migrations`.
4. Run `pnpm lint`, `pnpm format:check`, and `pnpm test`.
5. Update `docs/scope/lan.md` and `docs/scope/index.md` marking Feature 72 complete.
6. Record commit with message: `feat(lan): implement LAN host update mirror and air-gapped workstation updates (Feature 72, Spec 0048)`.
