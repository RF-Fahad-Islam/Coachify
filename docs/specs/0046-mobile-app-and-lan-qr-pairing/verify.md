# Verification: Mobile App and LAN QR Pairing

## Traceability matrix

This matrix maps every acceptance criterion from [index.md](index.md) to its automated integration tests and manual verification procedures:

| Criterion | Description | Test Type | Verification Method | Target SLA |
|---|---|---|---|---|
| **AC-1** | Ephemeral QR code generation and URI formatting | Automated | `test_open_mobile_pairing_window_success` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Window opens in under 100ms with valid QR URI |
| **AC-2** | Anti brute force lockout and cooldown | Automated | `test_pairing_brute_force_lockout` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Locks after exactly 5 failures for 15 minutes |
| **AC-3** | Optical TLS 1.3 certificate pinning | Automated | `test_tls_fingerprint_pinning_rejection` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Rejects mismatched fingerprint with 0 bytes leaked |
| **AC-4** | Instant QR handshake and keystore custody | Automated | `test_qr_pairing_handshake_flow` in `crates/core/tests/lan_mobile_pairing_integration.rs` | End to end pairing completes in under 1 second |
| **AC-5** | License workstation seat limit enforcement | Automated | `test_mobile_pairing_respects_seat_limit` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Returns HTTP 403 when quota reached |
| **AC-6** | High speed camera barcode attendance scanning | Manual / Hardware | Classroom scanning test on Android and iOS hardware | Continuous scanning under 150ms per student badge |
| **AC-7** | Host print routing and WhatsApp sharing | Automated & Manual | `test_host_receipt_print_dispatch` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Host printer triggers within 500ms of mobile payment |
| **AC-8** | Strict live RPC and Wi-Fi auto reconnect | Automated & Manual | Network drop simulation drill with simulated router disconnect | Seamless query cache recovery upon Wi-Fi reconnect |
| **AC-9** | Real time SSE control and instant revocation | Automated | `test_mobile_instant_revocation_sla` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Complete client lockout in under 500ms |
| **AC-10** | Immutable audit logging of mobile events | Automated | `test_mobile_audit_log_records` in `crates/core/tests/lan_mobile_pairing_integration.rs` | Records device ID, IP, and staff user ID |

## Automated test suite

### Rust Core Integration Tests (`crates/core/tests/lan_mobile_pairing_integration.rs`)

1. `test_open_mobile_pairing_window_success`:
   - Initialize in memory SQLite database and LAN host config.
   - Call `lan.open_mobile_pairing_window`.
   - Assert `lan_pairing_sessions` contains a record with `status = 'active'`, valid expiration 10 minutes in the future, and non null `qr_secret_hash`.
   - Assert returned QR URI contains valid scheme `coachify://pair` and matches host IPv4 addresses and certificate fingerprint.

2. `test_pairing_brute_force_lockout`:
   - Open mobile pairing window.
   - Submit 5 consecutive invalid `qr_secret` values to `POST /pair/qr`.
   - Verify first 4 attempts return HTTP 403 with `PAIRING_SECRET_INVALID`.
   - Verify 5th attempt triggers lockout and returns HTTP 403 with `PAIRING_LOCKED`.
   - Verify 6th attempt within cooldown window is immediately rejected with `PAIRING_COOLDOWN_ACTIVE`.
   - Verify security audit event `security.pairing_brute_force_locked` is recorded in `audit_log`.

3. `test_qr_pairing_handshake_flow`:
   - Open mobile pairing window, capturing `qr_secret`.
   - Spawn mock mobile client with custom rustls verifier pinned to the host certificate fingerprint.
   - Client executes `POST /pair/qr` over TLS 1.3 with client nonce, device name "Pixel 8", and platform "android".
   - Verify HTTP 200 response contains valid bearer token prefixed with `dt_`.
   - Verify `devices` table records device with `platform = 'android'`, `device_type = 'mobile_phone'`, `paired_via = 'qr'`, and `status = 'active'`.
   - Verify session in `lan_pairing_sessions` transitions to `status = 'used'`.
   - Verify re submitting the same `qr_secret` fails with `PAIRING_SESSION_ALREADY_USED`.

4. `test_tls_fingerprint_pinning_rejection`:
   - Host runs on port 50443 with valid self signed certificate.
   - Mock client attempts connection using an altered SHA-256 fingerprint (one byte corrupted).
   - Verify TLS handshake fails at the transport layer before any HTTP payload or secret is transmitted.

5. `test_mobile_pairing_respects_seat_limit`:
   - Initialize license with `workstation_limit = 1`.
   - Register one active desktop workstation.
   - Attempt `POST /pair/qr` from a mobile phone.
   - Verify request is rejected with HTTP 403 and error code `LAN_WORKSTATION_LIMIT_REACHED`.
   - Revoke the desktop workstation.
   - Re attempt mobile pairing.
   - Verify pairing now succeeds and assigns the freed license seat.

6. `test_mobile_instant_revocation_sla`:
   - Pair mobile device and establish authenticated SSE stream on `GET /events`.
   - Record baseline timestamp.
   - Host administrator executes `lan.revoke_workstation(device_id)`.
   - Verify mobile SSE receiver receives `device.revoked` event within 500 milliseconds.
   - Verify subsequent RPC calls with the revoked device token immediately return HTTP 403 (`DEVICE_REVOKED`).

7. `test_ip_candidate_racing_fast_connect`:
   - Host presents multiple IP candidates in QR payload, including one valid local IP and one unroutable dummy IP (simulating a dead virtual network bridge).
   - Mobile client dispatches concurrent connection racing with a 2.5 second timeout.
   - Verify connection completes successfully in under 1 second using the valid IP, cleanly cancelling the stalled candidate without delay.

8. `test_staff_pin_authentication_and_session_timeout`:
   - Submit paired device token and 4 digit staff PIN to `POST /auth/staff-login`.
   - Verify response returns valid `staff_session_token`, staff ID, and assigned permissions.
   - Execute test `/rpc` call carrying session token; verify successful execution.
   - Simulate 15 minute background eviction; verify subsequent `/rpc` calls are rejected with `SESSION_LOCKED_PIN_REQUIRED`.

9. `test_designated_receipt_printer_dispatch`:
   - Configure `receipt_printer_name = 'POS-80C'` in `lan_host_config` table.
   - Set Windows default OS printer to an A4 document printer.
   - Submit `POST /print/host-receipt` from mobile client.
   - Verify host print engine routes thermal receipt job specifically to `'POS-80C'` rather than the Windows default printer.

10. `test_quick_scan_ip_update_without_repairing`:
   - Simulate DHCP renewal by rebinding host listener to an alternate local IP address while keeping the same certificate and host UUID.
   - Mobile client fails mDNS lookup and enters host disconnected state.
   - Mobile client scans updated QR code in recovery mode.
   - Verify mobile transport updates target address and successfully resumes `/rpc` operations using its existing device token without creating a duplicate device record.

## Security & Penetration Drills

### Drill 1: Wi-Fi Man in the Middle (MITM) Interception
- **Threat**: An attacker on the coaching center Wi-Fi uses ARP spoofing to position a rogue proxy between the mobile phone and host desktop, presenting a fraudulent TLS certificate.
- **Verification**: Mobile client attempts connection. Because the mobile client validates the server certificate against the SHA-256 hash scanned from the host screen, the client rejects the attacker certificate instantly. The connection terminates cleanly with zero plaintext leakage.

### Drill 2: Visual Shoulder Surfing & Expired Secret Replay
- **Threat**: A visitor takes a smartphone photograph of the pairing QR code displayed on the front desk monitor and attempts to pair from the parking lot later that evening.
- **Verification**: Attacker submits the captured `qr_secret` after 10 minutes. The host verifies `expires_at < current_time`, rejects the request with `PAIRING_EXPIRED`, and logs the failed attempt.

### Drill 3: Subverted Mobile Device Credential Theft
- **Threat**: An employee loses their smartphone or it is stolen by an unauthorized third party who extracts app sandboxed files.
- **Verification**: Device token is protected by Android Keystore using AES-256-GCM hardware encryption (and iOS Keychain Services). The token cannot be extracted without device biometric or hardware unlock. Simultaneously, the institution owner revokes the device from the host desktop, permanently invalidating the token hash in SQLite.

## Mobile OS Compatibility Matrix

| Operating System | Minimum Version | Tested Reference Devices | Validation Status |
|---|---|---|---|
| Android | Android 10 (API Level 29) | Samsung Galaxy A14, Xiaomi Redmi Note 12, Google Pixel 7 | Passing |
| iOS / iPadOS | iOS 16.0 / iPadOS 16.0 | iPhone 11, iPhone 14 Pro, iPad 9th Gen | Passing |

## Step by step verification script

1. **Host Setup**:
   - Launch Coachify Desktop on the primary Windows PC.
   - Navigate to **Settings → Devices**.
   - Ensure LAN Host service is active on port `50443`.
   - Click **Pair Mobile Device**.
   - Verify modal opens with high contrast QR code, live countdown timer (starting at 10:00), and fallback 6 digit code.

2. **Mobile Pairing**:
   - Open Coachify Mobile app on an Android or iOS device connected to the coaching center Wi-Fi.
   - Tap **Scan QR Code**.
   - Point the camera at the desktop monitor.
   - Verify the camera viewfinder recognizes the QR code immediately (under 200ms).
   - Verify the mobile screen confirms institution name (e.g. "Connected to Daffodil Academy").
   - Verify the desktop host modal updates automatically to "Device Paired Successfully!" and closes.
   - Verify the mobile device appears in the host **Devices** list with friendly name and status `active`.

3. **Attendance Scanner Rapid Drill**:
   - On the mobile phone, navigate to **Attendance**.
   - Select an active batch (for example "Physics 1st Year (Morning Batch)").
   - Tap **Camera Attendance Scanner**.
   - Point the camera at five student ID card barcodes in sequence.
   - Verify each scan produces an instant audio beep and green confirmation banner in under 150 milliseconds.
   - Re scan one card immediately; verify yellow "Already Scanned" badge appears and no duplicate entry is sent.
   - Check the host desktop Attendance screen; verify all 5 students reflect as `Present` in real time.

4. **Cashier Payment & Host Printing**:
   - On the mobile phone, navigate to **Cashier**.
   - Search for a student by name or phone number.
   - Enter fee collection of 2,500 Taka and click **Collect & Print**.
   - Verify the desktop host USB receipt printer immediately prints the thermal slip.
   - Verify mobile app displays "Payment Recorded" with a prominent green "Share via WhatsApp" button.

5. **Instant Revocation Test**:
   - On the host desktop, navigate to **Settings → Devices**.
   - Locate the paired mobile device and click **Revoke**.
   - Enter reason "Employee Resigned" and confirm.
   - Observe the mobile phone screen.
   - Verify that in under 500 milliseconds, the mobile app closes the active view, clears stored credentials, and displays the "Device Access Revoked" screen.
   - Try tapping any button on the phone; verify no data is displayed and all actions are blocked.
