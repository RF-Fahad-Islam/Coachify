# 0048. LAN Host Update Mirror: Verification Plan

> Verification matrix, automated integration test suites, network failure drills, and manual QA procedures for Spec 0048 (LAN Host Update Mirror, Feature 72)

## Verification Methodology

Verification of Feature 72 is structured in three concentric rings:

1. **Automated Unit & Integration Test Suites**: Executed in Rust (`crates/core/tests/lan_update_mirror_integration.rs`) using in-process SQLite, Tokio TLS listeners, and `InMemoryClientTransport` / `TlsClientTransport`. Tests mirror staging, manifest generation, streaming chunk download, cryptographic signature verification, device token authorization, and real-time SSE push.
2. **Security & Cryptographic Tamper Drills**: Deliberately executes malicious and degraded scenarios: corrupted payload bits, mismatched SHA-256 hashes, invalid Ed25519 signatures, truncated file downloads, unauthenticated requests, and blocked device download attempts.
3. **Manual Hardware Verification**: Validates the end-to-end user experience on two Windows 11 PCs on a local Wi-Fi router, staging an update from a USB drive on the Host and downloading it on a workstation.

## Acceptance Criteria Coverage Matrix

| Acceptance Criteria | Description | Verification Method | Automated Test Name |
|---|---|---|---|
| **AC-1** | Local Mirror Cache Storage & Management | Automated Integration Test | `test_mirror_cache_staging_and_db_tracking` |
| **AC-2** | Pinned TLS Host Update Endpoints (`/updates/*`) | Automated Integration Test | `test_host_update_endpoints_and_auth` |
| **AC-3** | Real-Time Broadcast & SSE Notification | Automated Integration Test | `test_mirror_staged_emits_sse_change_event` |
| **AC-4** | Workstation Discovery & Local LAN Download | Automated Integration Test | `test_workstation_update_check_and_download` |
| **AC-5** | Cryptographic Verification & Pre-Update Safety | Automated Security Test | `test_download_cryptographic_verification_and_tamper_rejection` |
| **AC-6** | Air-Gapped Manual Import (USB / File) | Automated Integration Test | `test_air_gapped_manual_file_staging` |
| **AC-7** | Host Mirror Controls & Configuration UI | Automated Integration Test | `test_configure_mirror_and_status_reporting` |
| **AC-8** | Workstation Update Settings UI | UI Unit Test (Vitest) | `ui/src/components/settings/update-settings.test.tsx` |
| **AC-9** | Concurrency & Async Streaming Resilience | Automated Load Test | `test_concurrent_workstation_downloads` |
| **AC-10** | Audit Log & Observability | Automated Integration Test | `test_update_mirror_audit_trail_logging` |

## Automated Integration Test Suites (`crates/core`)

The test suite in `crates/core/tests/lan_update_mirror_integration.rs` validates the complete update mirror pipeline:

### 1. Test: `test_mirror_cache_staging_and_db_tracking`
- **Scenario**: Verify that staging a valid update package stores the file in `update_mirror/` and records it in `lan_staged_updates`.
- **Assertions**:
  - File exists at `%LOCALAPPDATA%\Coachify\update_mirror\1.1.0\Coachify-Setup-1.1.0.exe`.
  - Database row in `lan_staged_updates` has `version = '1.1.0'`, `file_size_bytes = len`, `sha256 = hash`, and `is_active = 1`.
  - Staging a second release deactivates or replaces the prior active record.

### 2. Test: `test_host_update_endpoints_and_auth`
- **Scenario**: Verify `/updates/manifest` and `/updates/download/:version` access control.
- **Assertions**:
  - Request with missing token receives HTTP 401 Unauthorized.
  - Request with token of a `blocked` or `revoked` device receives HTTP 403 Forbidden.
  - Request with valid active device token receives HTTP 200 with manifest JSON.
  - Calling `/updates/download/1.1.0` streams the exact binary bytes matching the staged file.

### 3. Test: `test_download_cryptographic_verification_and_tamper_rejection`
- **Scenario**: Validate that corrupted or tampered payloads are immediately detected and deleted.
- **Assertions**:
  - Injected 1-bit payload flip causes SHA-256 check to fail; download function returns `ChecksumMismatch` and deletes the temporary file.
  - Invalid Ed25519 signature causes verification to fail with `InvalidSignature`; file is rejected.
  - Valid payload and signature pass verification, leaving the verified installer ready for launch.

### 4. Test: `test_concurrent_workstation_downloads`
- **Scenario**: Simulate 3 workstations concurrently downloading a 50 MB simulated payload from the host.
- **Assertions**:
  - All 3 downloads complete successfully with exact SHA-256 match.
  - Host domain RPC operations (`students.list`, `cashier.get_student_overview`) execute concurrently during the file transfer without deadlock or timeout.
  - `downloads_count` on `lan_staged_updates` increments accurately to 3.

### 5. Test: `test_update_mirror_audit_trail_logging`
- **Scenario**: Verify that mirror staging, clearing, and downloads write to the append-only SHA-256 audit log.
- **Assertions**:
  - `audit_log` contains records for `lan_update_mirror.staged` and `lan_update_mirror.download_completed`.
  - Downloading device ID and IP address are captured in the audit metadata.
