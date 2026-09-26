# 0047. LAN Workstation Client Mode and RPC Transport: Verification Plan

> Verification matrix, automated integration test suites, network failure drills, and manual QA procedures for Spec 0047 (LAN Workstation Client Mode and RPC Transport, Feature 71)

## Verification Methodology

Verification of Feature 71 is structured in three concentric rings:

1. **Automated Unit & Integration Test Suites**: Executed in Rust (`crates/core/tests/workstation_client_mode_integration.rs`) against an in-process SQLite instance and real Tokio HTTPS/rustls listener on loopback. Tests command routing taxonomy (local vs remote), pinned TLS 1.3 forwarding, device token validation, client-side idempotency across network drops, and real-time SSE event fan-out.
2. **Security & Network Failure Drills**: Deliberately executes malicious and degraded scenarios: certificate spoofing / MITM interception, network socket dropouts mid-transaction, dynamic DHCP IP address shifts, in-flight mutation retries, and high-frequency concurrent multi-counter payments.
3. **Manual Two-PC Hardware Lab Matrix**: Conducted in the physical Coachify QA lab using two Windows 11 machines connected over an unmanaged Wi-Fi router with physical USB thermal receipt printers (POS-80) attached to the workstation PC.

## Acceptance Criteria Coverage Matrix

| Acceptance Criteria | Description | Verification Method | Automated Test Name |
|---|---|---|---|
| **AC-1** | Transparent Dual-Mode RPC Dispatcher | Automated Integration Test | `test_transparent_dual_mode_rpc_dispatch` |
| **AC-2** | Pinned TLS 1.3 Forwarding Transport (`POST /rpc`) | Automated Integration Test | `test_pinned_tls_transport_and_mitm_rejection` |
| **AC-3** | Hardware-Bound Bearer Device Token Authentication | Automated Integration Test | `test_device_token_authentication_and_session_layering` |
| **AC-4** | Complete Domain Module Feature Parity | Automated Integration Test | `test_domain_module_parity_across_workstation` |
| **AC-5** | Strictly Local Document & Receipt Printing | Automated + Platform Test | `test_strictly_local_printing_isolation` |
| **AC-6** | Real-Time Multi-Counter SSE Event Synchronization | Automated Integration Test | `test_realtime_multi_counter_sse_cache_sync` |
| **AC-7** | Resilient Disconnect Handling & Dynamic Host Re-Discovery | Automated Network Drill | `test_dynamic_mdns_host_ip_shift_recovery` |
| **AC-8** | In-Flight Mutation Protection via Client Idempotency | Automated Integration Test | `test_in_flight_mutation_idempotency_recovery` |
| **AC-9** | Sub-500ms Instant Remote Severance Enforcement | Automated Latency Benchmark | `test_instant_remote_revocation_lockout_latency` |
| **AC-10** | Workstation Health Monitoring & Latency Diagnostics | Automated Integration Test | `test_workstation_health_and_latency_monitoring` |

## Automated Integration Test Suites (`crates/core`)

The test suite in `crates/core/tests/workstation_client_mode_integration.rs` validates the complete workstation runtime:

### 1. Test: `test_transparent_dual_mode_rpc_dispatch`
- **Scenario**: Verify that the client RPC bridge properly classifies commands into local execution vs remote TLS forwarding.
- **Assertions**:
  - Calling `printers.list` and `printers.get_settings` executes locally against workstation machine storage without generating network packets.
  - Calling `lan.get_client_status` returns local connection metrics (`state: Connected`, RTT latency, host identity).
  - Calling `students.list` or `cashier.get_student_overview` dispatches an HTTPS request to `POST /rpc` on the host PC and returns data from the host's SQLite database.
  - Both local and remote commands return identical `RpcEnvelope<T>` shapes.

### 2. Test: `test_pinned_tls_transport_and_mitm_rejection`
- **Scenario**: Validate transport layer encryption, pinned X.509 SHA-256 fingerprint verification, and rogue host protection.
- **Assertions**:
  - Client connects to `POST /rpc` with valid pinned fingerprint; assert 200 OK.
  - Attempting to connect with an altered or rogue TLS certificate fails immediately during the TLS handshake with `FingerprintMismatch`.
  - Plain unencrypted HTTP request to `http://127.0.0.1:50443/rpc` is rejected at the transport layer before request parsing.

### 3. Test: `test_device_token_authentication_and_session_layering`
- **Scenario**: Validate dual-layer authentication: device token (hardware) and session token (staff user).
- **Assertions**:
  - Request with missing `X-Coachify-Device-Token` header receives HTTP 401 Unauthorized (`INVALID_DEVICE_TOKEN`).
  - Request with a token belonging to a `blocked` device receives HTTP 403 Forbidden (`DEVICE_BLOCKED`).
  - Request with a token belonging to a `revoked` device receives HTTP 403 Forbidden (`DEVICE_REVOKED`).
  - Request with a valid active device token updates `devices.last_seen_at` and `devices.ip_address` in the host database.
  - Domain commands require both a valid device token AND a valid staff session token; missing staff session returns `UNAUTHENTICATED`.

### 4. Test: `test_strictly_local_printing_isolation`
- **Scenario**: Verify that payment receipts fetch data remotely from the host but configure and execute printing strictly on the local workstation.
- **Assertions**:
  - Client executes `cashier.create_payment` remotely, producing a payment record in the host SQLite database.
  - Client calls `receipts.get_printable` remotely to fetch payment details, receipt number (`REC-2026-0042`), and amount in English/Bengali words.
  - Workstation verifies its own local storage for printer settings (`workstation_printer_settings.json`).
  - The rendered HTML print job targets the local workstation's configured printer name (`POS-80 Counter 2`).
  - Host PC's print spooler receives zero print jobs.

### 5. Test: `test_realtime_multi_counter_sse_cache_sync`
- **Scenario**: Verify that mutations on one counter broadcast change events over SSE to other connected workstations in under 500ms.
- **Assertions**:
  - Two simulated workstation clients (WS-1 and WS-2) connect to the host's `GET /events` SSE stream.
  - WS-1 records a fee payment via `POST /rpc`.
  - Host commits transaction and broadcasts `ChangeEvent("ledger:payment", [payment_id])`.
  - WS-2 receives the `coachify_change` SSE event in **< 150 milliseconds** on loopback.
  - WS-2's client runtime emits `coachify://change` to the local window to invalidate TanStack Query caches.

### 6. Test: `test_in_flight_mutation_idempotency_recovery`
- **Scenario**: Simulate a network dropout where the host commits a payment, but the network connection breaks before the workstation receives the HTTP response.
- **Assertions**:
  - WS-1 generates an idempotency key (`uuid::Uuid::now_v7()`) and submits `cashier.create_payment`.
  - Host processes the payment, writes to `idempotency_records`, and commits.
  - Network connection is forcefully dropped to simulate client packet loss.
  - WS-1 detects connection failure and retries the exact same command with the identical `idempotency_key`.
  - Host returns the previously committed receipt and payment details without:
    - Creating a second payment record.
    - Incrementing the sequential receipt number twice.
    - Double-charging the student's ledger balance.

### 7. Test: `test_dynamic_mdns_host_ip_shift_recovery`
- **Scenario**: Verify that a workstation recovers automatically when the host PC's local IP address changes via DHCP renewal.
- **Assertions**:
  - Workstation is paired with Host at `192.168.1.50:50443`.
  - Host listener simulates an IP change by re-binding to a new simulated address (`192.168.1.75:50443`) and announcing the new IP via mDNS `_coachify._tcp.local` with matching `host_id` and `fingerprint`.
  - Workstation request to `192.168.1.50` fails with network timeout.
  - Workstation enters `Reconnecting` state and launches background mDNS browse.
  - Workstation resolves `_coachify._tcp.local`, matches `host_id`, updates internal host endpoint to `192.168.1.75:50443`, and verifies the pinned certificate.
  - Workstation transitions back to `Connected` and successfully executes subsequent RPC requests without operator intervention.

### 8. Test: `test_instant_remote_revocation_lockout_latency`
- **Scenario**: Measure end-to-end latency when the host administrator revokes a workstation.
- **Assertions**:
  - Workstation is active on the cashier desk screen.
  - Host administrator triggers `lan.revoke_workstation(device_id)`.
  - Host broadcasts `device.revoked` over the workstation's open SSE connection.
  - Workstation receives event, deletes `%LocalAppData%\Coachify\workstation_credentials.dat`, purges in-memory session tokens, and renders the "Access Revoked" screen.
  - Total elapsed time from host execution to workstation lockout is **< 350 milliseconds**, easily satisfying the **< 500 millisecond** SLA.
  - Any subsequent `POST /rpc` request carrying the old device token returns HTTP 403 `DEVICE_REVOKED`.

### 9. Test: `test_multi_counter_concurrency_stress`
- **Scenario**: Simulate 5 workstations simultaneously recording payments and admissions against the host SQLite database.
- **Assertions**:
  - 5 concurrent worker threads submit 50 payment transactions each (250 total transactions) across 10 seconds.
  - SQLite WAL mode and immediate transactions handle concurrency cleanly without database lock timeouts (`busy_timeout = 5000ms`).
  - All 250 receipt numbers (`REC-YYYY-XXXX`) are strictly sequential, monotonic, and without gaps or duplicates.
  - Student ledger balances match the exact sum of all posted payments.

## Security & Failure Drills

| Scenario | Execution Method | Expected Result | Pass Criteria |
|---|---|---|---|
| **Man-in-the-Middle Attack** | Proxy LAN traffic with a spoofed self-signed certificate | Workstation detects certificate SHA-256 fingerprint mismatch and aborts handshake immediately | Zero plaintext data leaked; connection terminated |
| **Rogue Workstation Infiltration** | Unregistered PC sends `POST /rpc` with a forged or guess token | Host rejects request with HTTP 401 `INVALID_DEVICE_TOKEN` | Attacker cannot access database or read records |
| **Wi-Fi Router Power Cut** | Pull power plug on Wi-Fi router during active cashier session | Workstation transitions to `Reconnecting` with non-blocking banner; form data preserved; retries with backoff | Zero unhandled UI exceptions; clean recovery when router reboots |
| **Host PC Crash / Reboot** | Kill host process during office hours; restart 10 seconds later | Workstations detect offline state; re-establish TLS connection and SSE stream when host restarts; UI resumes cleanly | Zero client crashes; TanStack Query revalidates cache |
| **Clock Skew Attack** | Workstation PC advances its local clock by 3 days | Workstation submits payment; host stamps transaction with host authoritative UTC time | Transaction timestamp matches host clock, not workstation clock |

## Manual Two-PC Hardware Lab Matrix

Conducted in the Coachify QA lab using two physical Windows 11 machines connected over an isolated TP-Link Archer Wi-Fi router:

- [ ] **Step 1: Fresh Workstation Onboarding & Pair**:
  - Machine A (Host) starts Coachify Local Pro and opens **Settings → Devices → Pair New Workstation**.
  - Machine B (Workstation) launches Coachify, selects "Connect as Workstation", discovers Machine A via mDNS, verifies certificate fingerprint, and enters the 6-digit code.
  - Verify Machine B reaches the Login screen and Machine A shows Machine B as `Active`.
- [ ] **Step 2: Dual-Counter Simultaneous Payments**:
  - Staff logs into Machine A as "Receptionist".
  - Staff logs into Machine B as "Cashier".
  - Simultaneously submit fee payments for two different students on both machines.
  - Verify both payments commit cleanly with distinct sequential receipt numbers (e.g. `REC-2026-0001` on Host, `REC-2026-0002` on Workstation).
- [ ] **Step 3: Strictly Local Thermal Receipt Printing**:
  - Attach a POS-80 USB thermal printer to Machine B (Workstation).
  - Configure printer settings on Machine B (**Settings → Workstation → Default Receipt Printer** = "POS-80").
  - On Machine B, record a fee payment of 1,200 BDT with auto-print enabled.
  - Verify receipt prints physically on Machine B's attached thermal printer with correct student details and Bengali words.
  - Verify Machine A's default printer did NOT receive any print jobs.
- [ ] **Step 4: Real-Time SSE Cross-Counter Sync**:
  - On Machine A, open the Student Directory and view "Rahim Ahmed" (Balance: 2,500 BDT).
  - On Machine B, record a 2,500 BDT fee payment for "Rahim Ahmed".
  - Observe Machine A: within 1 second, verify "Rahim Ahmed"'s balance updates to 0 BDT automatically without manual page refresh.
- [ ] **Step 5: Router Wi-Fi Drop Recovery**:
  - On Machine B, disconnect from Wi-Fi.
  - Observe Machine B: header indicator turns amber ("Reconnecting to Host..."), and top banner appears.
  - Type a student's name into the Cashier search bar; verify UI remains responsive.
  - Reconnect to Wi-Fi.
  - Observe Machine B: within 3 seconds, connection recovers, header turns green ("Connected (9ms)"), and toast confirms reconnection.
- [ ] **Step 6: Remote Workstation Revocation SLA**:
  - On Machine A, open **Settings → Devices** and click **Actions → Revoke** on Machine B.
  - Confirm revocation.
  - Observe Machine B: in under 1 second, the active session terminates, the screen displays "Workstation Decommissioned by Administrator", and `%LocalAppData%\Coachify\workstation_credentials.dat` is wiped.
