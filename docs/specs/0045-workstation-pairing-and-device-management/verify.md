# 0045. Workstation Pairing and Device Management: Verification Plan

> Verification matrix, automated integration test suites, security failure drills, and manual QA procedures for Spec 0045 (Workstation Pairing and Device Management, Feature 70)

## Verification Methodology

Verification of Feature 70 is structured in three concentric rings:
1. **Automated Unit & Integration Test Suites**: Executed in Rust (`crates/core/tests/workstation_pairing_integration.rs`) against an in-process SQLite instance and real Tokio HTTPS/rustls listener on loopback. Tests handshake cryptography, token generation, anti-brute-force state machines, license seat limits, and revocation cascading.
2. **Security & Boundary Failure Drills**: Deliberately executes malicious and degraded scenarios: rapid code brute-forcing, replay attacks, concurrent pairing race conditions, clock manipulation, and network dropouts during pairing.
3. **Manual Two-PC Hardware Lab Matrix**: Tests physical pairing, Windows DPAPI persistence across OS reboots, mDNS discovery over Wi-Fi, and visual UI responsiveness on two physical Windows 11 machines.

## Acceptance Criteria Coverage Matrix

| Acceptance Criteria | Description | Verification Method | Automated Test Name |
|---|---|---|---|
| **AC-1** | Ephemeral 6-Digit Pairing Code & Anti-Brute-Force Window | Automated Integration Test | `test_pairing_window_lifecycle_and_brute_force_lockout` |
| **AC-2** | Mutual Cryptographic Handshake over Pinned TLS 1.3 | Automated Integration Test | `test_mutual_tls_and_single_use_handshake` |
| **AC-3** | High-Entropy Persistent Device Tokens & Windows DPAPI Storage | Automated + Platform Test | `test_device_token_generation_and_dpapi_custody` |
| **AC-4** | Commercial License Workstation Limit Enforcement (`workstation_limit`) | Automated Integration Test | `test_workstation_seat_quota_enforcement` |
| **AC-5** | Authoritative Host Workstation Registry (`devices` Table via Migration 0025) | Automated Integration Test | `test_devices_registry_crud_and_status` |
| **AC-6** | Workstation Label Management & Friendly Renaming | Automated Integration Test | `test_workstation_rename_and_audit` |
| **AC-7** | Workstation Blocking & Temporary Suspension | Automated Integration Test | `test_workstation_blocking_and_session_purge` |
| **AC-8** | Permanent Workstation Revocation & Immediate Disconnection | Automated Integration Test | `test_workstation_permanent_revocation_irreversible` |
| **AC-9** | Real-Time SSE Control Stream & Instant Push Severance (<500ms) | Automated Performance & Integration Test | `test_realtime_sse_revocation_push_latency` |
| **AC-10** | Security Audit Trail & Event Immutability | Automated Database Test | `test_device_events_audit_trail_immutability` |

## Automated Integration Test Suites (`crates/core`)

The test suite in `crates/core/tests/workstation_pairing_integration.rs` validates the complete pairing engine:

### 1. Test: `test_pairing_window_lifecycle_and_brute_force_lockout`
- **Scenario**: Validate code generation, 10-minute timeout, and 5-attempt brute-force protection.
- **Assertions**:
  - `lan::open_pairing_window` generates a valid 6-digit numeric string matching `^\d{6}$`.
  - Calling `lan.open_pairing_window` when already open returns `LAN_PAIRING_ALREADY_OPEN`.
  - Submitting 4 consecutive incorrect codes returns `LAN_PAIRING_CODE_INVALID` with decreasing `attempts_remaining` (4, 3, 2, 1).
  - Submitting a 5th incorrect code transitions session status to `'locked'`, updates mDNS to `pairing=0`, and returns HTTP 423 Locked (`LAN_PAIRING_LOCKED`).
  - Attempting to submit the correct code after lockout fails with `LAN_PAIRING_LOCKED`.
  - Opening a new pairing window during the 15-minute lockout cooldown is rejected with `LAN_PAIRING_COOLDOWN_ACTIVE`.

### 2. Test: `test_mutual_tls_and_single_use_handshake`
- **Scenario**: Verify end-to-end handshake over TLS 1.3 listener with pinned certificate and single-use code retirement.
- **Assertions**:
  - Client connects to `POST https://127.0.0.1:50443/pair` with verified SHA-256 host certificate fingerprint.
  - Plain HTTP connection to `http://127.0.0.1:50443/pair` is terminated at the TCP layer.
  - Submitting valid code returns 200 OK with `device_id` and `device_token` (prefixed `dt_`).
  - Submitting the same code immediately a second time returns `LAN_PAIRING_CODE_INVALID` or `LAN_PAIRING_CLOSED` (single-use invariant).
  - Host `lan_pairing_sessions.status` is updated to `'used'`.
  - Host mDNS TXT record resets to `pairing=0`.

### 3. Test: `test_device_token_generation_and_dpapi_custody`
- **Scenario**: Ensure issued token is 256-bit high entropy, stored as SHA-256 hash in SQLite, and verifiable via DPAPI on Windows.
- **Assertions**:
  - Issued token is at least 32 bytes of entropy (`dt_` prefix + Base64URL string).
  - SQLite query `SELECT token_hash FROM devices WHERE id = ?` matches `sha256(device_token)`.
  - Plaintext token is never present in any database column.
  - DPAPI encryption roundtrip (`CryptProtectData` -> `CryptUnprotectData`) restores original token bytes.

### 4. Test: `test_workstation_seat_quota_enforcement`
- **Scenario**: Validate license seat capping across tiers.
- **Assertions**:
  - License with `tier: "trial", workstation_limit: Some(1)`:
    - 1st workstation pairs successfully.
    - Attempting to pair a 2nd workstation fails with `LAN_WORKSTATION_LIMIT_REACHED`.
  - License with `tier: "local_pro", workstation_limit: Some(2)`:
    - Workstation 1 and 2 pair successfully.
    - Attempting to pair Workstation 3 fails with `LAN_WORKSTATION_LIMIT_REACHED`.
    - Revoking Workstation 1 decrements active count from 2 to 1.
    - Workstation 3 now pairs successfully.

### 5. Test: `test_workstation_blocking_and_session_purge`
- **Scenario**: Validate temporary blocking and automatic session severance.
- **Assertions**:
  - Workstation pairs and logs in a cashier (`accounts.login` creates a session in `sessions`).
  - Host operator calls `lan.block_workstation(device_id, "Maintenance")`.
  - `devices.status` becomes `'blocked'`.
  - Associated session in `sessions` has `revoked_at` populated immediately.
  - Subsequent RPC call carrying the blocked device token receives HTTP 403 Forbidden with `DEVICE_BLOCKED`.
  - Host operator calls `lan.unblock_workstation(device_id)`.
  - `devices.status` reverts to `'active'`.
  - Workstation can log in again using existing device token without re-pairing.

### 6. Test: `test_workstation_permanent_revocation_irreversible`
- **Scenario**: Validate permanent revocation and token nullification.
- **Assertions**:
  - Workstation pairs successfully.
  - Host calls `lan.revoke_workstation(device_id, "Laptop decommissioned")`.
  - `devices.status` becomes `'revoked'`.
  - `devices.token_hash` is overwritten with random tombstones or cleared.
  - Associated sessions revoked immediately.
  - Attempting to unblock or authenticate with old token fails permanently with `DEVICE_REVOKED`.

### 7. Test: `test_realtime_sse_revocation_push_latency`
- **Scenario**: Measure latency between host revocation and client SSE disconnect event receipt.
- **Assertions**:
  - Simulated client establishes TLS 1.3 SSE stream on `GET /events` with device token.
  - Host triggers `lan.revoke_workstation`.
  - Client receives `event: device_revoked` payload.
  - Time elapsed from dispatcher invocation to client event handler execution is **< 100 milliseconds** on loopback, safely within the **< 500 millisecond** SLA.

### 8. Test: `test_device_events_audit_trail_immutability`
- **Scenario**: Verify SQL triggers prevent tampering with device history.
- **Assertions**:
  - Every action (`paired`, `renamed`, `blocked`, `unblocked`, `revoked`) creates a row in `device_events`.
  - Direct SQL `UPDATE device_events SET event_type = 'paired'` fails with trigger abort: `"device_events records are append-only"`.
  - Direct SQL `DELETE FROM device_events` fails with trigger abort: `"device_events records cannot be deleted"`.

## Security & Failure Drills

| Scenario | Execution Method | Expected Result | Pass Criteria |
|---|---|---|---|
| **Brute-Force Attack** | Script generates 100 random 6-digit codes in parallel against `/pair` | Host terminates active session on 5th error and locks for 15 minutes; all subsequent requests return HTTP 423 | Attacker cannot guess code; server CPU stays < 5% |
| **Replay Attack** | Intercept completed `POST /pair` payload and resend 1 second later | Host rejects request with `LAN_PAIRING_CLOSED` | Single-use invariant preserved |
| **Expired Code** | Wait 601 seconds after opening pairing window before submitting valid code | Host rejects request with `LAN_PAIRING_EXPIRED` | Expired codes never accepted |
| **Clock Manipulation** | Client PC advances its clock 24 hours into the future before pairing | Host checks timestamp against its own monotonic clock | Handshake validates against host time only |
| **Host Process Crash Mid-Pairing** | Kill host process during `POST /pair` transaction before response sent | SQLite transaction rolls back atomically | Neither partial device record nor half-active token persists |
| **Wi-Fi Disconnect during Active Session** | Unplug LAN cable on workstation while logged in; wait 10 seconds; plug back in | Workstation client enters reconnecting state; SSE re-establishes; session resumes without data loss | Zero unhandled UI exceptions |

## Manual Two-PC Hardware Lab Matrix

Conducted in the Coachify QA lab using two physical Windows 11 machines connected over a TP-Link Archer C6 Wi-Fi router (isolated office network):

- [ ] **Step 1: Fresh Workstation Onboarding**:
  - Primary PC starts Coachify Local Pro and navigates to **Settings → Devices**.
  - Click **Pair New Workstation**. Observe 6-digit code `[CODE]` and 10:00 countdown timer.
  - Secondary PC launches Coachify in Workstation Setup Mode.
  - Verify primary PC appears in Discovered Hosts list within 3 seconds via mDNS.
  - Verify SHA-256 certificate fingerprint matches on both screens.
  - Type 6-digit code on secondary PC and click **Connect**.
  - Verify secondary PC transitions to Login screen in under 2 seconds.
  - Verify primary PC table updates immediately showing secondary PC name and `Active` badge.
- [ ] **Step 2: Windows DPAPI Persistence across Reboot**:
  - Log into secondary PC with a cashier account.
  - Restart Windows 11 on the secondary PC.
  - Launch Coachify on secondary PC.
  - Verify Coachify opens directly to the Login screen without asking to pair again.
- [ ] **Step 3: Workstation Renaming**:
  - On Host PC, click **Actions → Rename** on the workstation.
  - Change name to `"Counter 1 - Reception Desk"`.
  - Verify name updates in the Devices table and audit drawer.
  - Perform a cashier transaction on secondary PC; verify printed receipt metadata shows `"Counter 1 - Reception Desk"`.
- [ ] **Step 4: Immediate Revocation Disconnect**:
  - While secondary PC is on the Cashier Payment entry screen, click **Actions → Revoke** on Host PC.
  - Type `"REVOKE"` in the confirmation dialog and submit.
  - Observe secondary PC: screen must immediately display the "Workstation Access Revoked" modal and drop the session in under 1 second.
  - Attempt clicking any button on secondary PC; verify UI is locked.
  - Verify local credential file `%LocalAppData%\Coachify\workstation_credentials.dat` is wiped.
  - Verify license seat counter on Host PC decrements immediately, allowing a replacement device to pair.
