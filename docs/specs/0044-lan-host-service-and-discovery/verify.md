# 0044. LAN Host Service and Discovery: Verification Plan

> Verification suite and testing matrix for Spec 0044 (LAN Host Service and Discovery, Feature 69)

## Automated Test Suites

Run the dedicated LAN host integration test suite:

```bash
cargo test -p coachify_core --test lan_host_integration
```

Verify type contracts and run UI unit tests:

```bash
cargo run -p xtask check-types
cargo build --workspace
pnpm --dir ui test
```

## Test Scenarios & Criteria Matrix

| Test Scenario | Acceptance Criteria | Verification Method |
|---|---|---|
| Self-signed X.509 certificate generation with SANs | AC-1 | Initializes LAN host manager on fresh DB; asserts certificate contains `localhost`, `127.0.0.1`, `::1`, hostname, and detected local IPs in SANs; asserts validity is 10 years (3650 days). |
| Certificate persistence in SQLite across restarts | AC-1 | Stops and restarts host; confirms loaded certificate and private key match original PEMs exactly without generating new credentials. |
| SHA-256 fingerprint derivation | AC-1 | Computes DER SHA-256 fingerprint; asserts formatted string matches expected `SHA256:XX:XX:...` standard. |
| License gating on single-PC tier | AC-2 | Attempts to start host on `Coachify Local` license tier (without `lan_multi_device`); asserts dispatcher rejects command with error `LICENSE_FEATURE_NOT_ENTITLED`. |
| License gating on Local Pro / Trial tier | AC-2 | Upgrades license to `Coachify Local Pro` or active trial; asserts `lan.start_host` succeeds and listener starts. |
| TLS 1.3 listener port binding & plain HTTP rejection | AC-3 | Starts host on default port 50443; connects with plain unencrypted HTTP client; asserts transport rejects connection. Connects with TLS 1.3 client; asserts successful handshake. |
| Asynchronous runtime decoupling | AC-3 | Generates sustained synthetic load against host listener; confirms Tauri UI main thread and local database commands experience zero latency spikes or stutter. |
| Fast start/stop lifecycle | AC-3 | Measures elapsed time for `lan.start_host` and `lan.stop_host`; asserts lifecycle transitions complete in under 500ms. |
| Unauthenticated discovery ping (`GET /hello`) | AC-4 | Calls `GET /hello` over TLS; validates response JSON contains `product == "coachify"`, matching `host_id`, `protocol_version == 1`, and `tls_fingerprint_sha256`. |
| Zero data leakage on public ping | AC-4 | Inspects raw response from `/hello`; asserts zero student names, phone numbers, ledger amounts, or credentials appear anywhere in output. |
| Rate-limiting on public ping endpoint | AC-4 | Issues 35 consecutive requests within 10 seconds to `GET /hello`; asserts requests 1–30 return HTTP 200 and requests 31+ receive HTTP 429 Too Many Requests. |
| Encrypted host health check (`GET /health`) | AC-5 | Calls `GET /health` over TLS; validates response contains `status == "healthy"`, `database_status == "ok"`, server UTC timestamp, and accurate `uptime_seconds`. |
| mDNS service broadcast & TXT attributes | AC-6 | Starts host; verifies mDNS responder announces `_coachify._tcp.local` on port 50443 with all required TXT keys (`host_id`, `proto`, `fingerprint`, `inst`, `ver`). |
| mDNS goodbye packet on shutdown | AC-6 | Stops host; verifies RFC 6762 goodbye packet (TTL = 0) is immediately broadcast over multicast group `224.0.0.251:5353`. |
| Workstation discovery scanner | AC-7 | Runs `lan.discover_hosts`; confirms discovery scanner discovers local host via mDNS and returns formatted metadata. |
| Resilient manual IP fallback | AC-7 | Disables mDNS scanner to simulate router multicast drop; executes `lan.ping_host` with manual IP and port; verifies successful connection and fingerprint match. |
| Windows network profile detection | AC-8 | Calls `lan.get_firewall_guidance`; confirms accurate detection of `Private` vs `Public` profile and verifies output includes the valid `netsh` and PowerShell firewall commands. |
| Win32 sleep inhibition | AC-9 | Starts host on Windows; verifies `SetThreadExecutionState` is invoked with `ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED`. Stops host; verifies execution state is restored. |
| System tray minimization on window close | AC-9 | Triggers window close while host is active; confirms window is hidden to tray and host listener remains actively serving requests. |

## Network Edge Case & Failure Drills

1. **Port Conflict Recovery Drill**:
   - Bind another process to TCP port 50443 before starting Coachify LAN host.
   - Call `lan.start_host`.
   - Verify host manager returns `LAN_PORT_IN_USE` with clear diagnostics, suggesting an alternative port (e.g. 50444) without crashing the desktop app.
2. **Dynamic DHCP IP Shift Drill**:
   - Start host on an initial IP (e.g. `192.168.1.50`).
   - Simulate a network interface change or DHCP lease renewal shifting IP to `192.168.1.150`.
   - Verify mDNS responder detects network interface change, re-announces the service under the new IP with identical `host_id` and certificate fingerprint, and continues serving requests seamlessly.
3. **WiFi Client Isolation Simulation Drill**:
   - Block multicast UDP packets on port 5353 between client and host.
   - Verify discovery scanner flags that mDNS discovery timed out and prompts the user to enter the host IP manually.
   - Input the manual host IP; confirm the client connects to `GET /hello`, verifies the certificate fingerprint, and establishes communication.
4. **License Expiry While Hosting Drill**:
   - While host service is actively running, simulate license expiration past grace period.
   - Verify the host logs a warning and transitions incoming RPC commands to read-only or gracefully shuts down the listener with clear notification to the operator.

## Manual Verification Checklist

1. Launch Coachify desktop in development mode (`pnpm dev`).
2. Navigate to **Settings** &rarr; **LAN Host**.
3. Confirm that current Host Status is shown as "Stopped" and your PC hostname is displayed.
4. If running on a `Coachify Local` (single PC) plan, verify that clicking "Enable LAN Host" displays an upgrade message explaining that LAN multi-device mode requires `Coachify Local Pro`.
5. Switch to a `Coachify Local Pro` license or evaluation trial.
6. Click **Enable LAN Host**.
7. Confirm that the status indicator changes to "Listening & Broadcasting", showing the bound port (`50443`), host IP addresses, and SHA-256 certificate fingerprint.
8. Check the **Windows Firewall & Network Profile** panel:
   - Verify that your network profile is detected (e.g. `Private`).
   - If marked `Public`, verify that the warning alert and copyable PowerShell remediation command are displayed.
   - Verify the one-click copy button for the Windows Firewall rule.
9. Open a terminal or browser and query `https://localhost:50443/hello`:
   - Verify that the browser reports a self-signed certificate.
   - Verify that the certificate fingerprint matches the one displayed in Coachify Settings.
   - Verify that the returned JSON contains `product: "coachify"` and zero private student records.
10. Query `https://localhost:50443/health` and verify `status: "healthy"` and `database_status: "ok"`.
11. With LAN Host enabled, click the window close ("X") button:
    - Confirm the main window hides and an icon appears in the Windows system tray with a notification indicating that Coachify is hosting on LAN in the background.
    - Double-click the tray icon to restore the window.
12. Attempt to exit Coachify via the tray menu:
    - Confirm a confirmation prompt appears warning that closing Coachify will disconnect workstation clients.
13. Click **Disable LAN Host** in Settings:
    - Confirm the listener stops, the port is released, and mDNS sends an immediate goodbye packet.
