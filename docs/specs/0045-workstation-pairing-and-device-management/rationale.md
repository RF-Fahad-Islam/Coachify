# 0045. Workstation Pairing and Device Management: Rationale

> Decision record and architectural rationale for Spec 0045 (Workstation Pairing and Device Management, Feature 70)

## Context

In growing coaching centers across Bangladesh, operations during morning rush hours, admission cycles, and monthly fee collection windows cannot be sustained on a single computer. A typical center requires two to five workstations: a reception desk registering new student admissions, one or two cashier counters taking cash payments and issuing printed receipts, and a coordinator's desk managing batch schedules and exams.

However, coaching center staff and owners are educators and administrators, not network systems engineers. They operate in challenging environments:
1. **Zero IT Staff**: There is no sysadmin to configure Active Directory, VPNs, LDAP, static IP routes, or complex certificate authorities. Setup must be accomplished by non-technical operators in seconds.
2. **Physical Desk Proximity**: Workstations and the host PC are physically located in the same building or floor, connected to the same office Wi-Fi router or switch. Operators can easily look across the counter or speak to each other.
3. **High Cash Flow & Counter Security**: Cashiers handle substantial cash collections. If an unauthorized laptop on the office Wi-Fi could connect to the database, an attacker could tamper with financial records, falsify receipts, or steal student contact lists.
4. **Frequent Staff Turnover & Laptop Hardware Swaps**: Temporary clerks are hired during admission season, or personal laptops are brought in. When a staff member departs, the owner must be able to instantly revoke that machine's access with absolute certainty.
5. **Completely Offline Guarantees**: Any pairing or device verification mechanism that depends on an external cloud server, SMS verification, or internet access will fail during routine broadband outages, violating Coachify's core offline promise.

To address these conditions, Coachify's pairing and device management architecture must balance effortless, non-technical pairing with bank-grade local cryptographic security and instant administrative control.

## Options Considered

### 1. Pairing Handshake Mechanism: 6-Digit Code (OTP) vs Long Pre-Shared Key (PSK) vs QR Code Only vs Cloud Account Relay

- **Option 1A: 6-Digit Numeric Code with Ephemeral Window & Lockout (Selected)**
  The host generates a 6-digit CSPRNG numeric code valid for 10 minutes when the owner clicks "Pair New Workstation". The workstation enters the code over pinned TLS 1.3. After 5 incorrect attempts, the window locks for 15 minutes.
  - *Pros*: Extreme ease of use (takes 5 seconds to type); easily read aloud across the room or transcribed; no camera hardware required on desktop PCs; highly secure when constrained to a 10-minute window and backed by a 5-attempt brute-force lockout.
  - *Cons*: Lower theoretical entropy than a 128-bit key if the pairing window were permanently open (mitigated by the 10-minute expiry and 5-attempt rate-limiting).
- **Option 1B: Long Cryptographic Pre-Shared Key (e.g. 32-character Base64 string)**
  Host displays a long random key (e.g. `9f8b2a-c71d-4e92-ba03-68d904b7...`) which the cashier must type on the secondary PC.
  - *Pros*: Maximum cryptographic entropy.
  - *Cons*: Dreadful user experience on computers without shared clipboards. Typing 32 random characters across two offline PCs causes frequent typos, user frustration, and support tickets.
- **Option 1C: Optical QR Code Scan Only**
  Host displays a QR code on screen; secondary computer scans it using a webcam.
  - *Pros*: Instant zero-typing pairing.
  - *Cons*: The vast majority of Bangladeshi coaching center desktop computers are budget tower PCs without webcams or cameras. Requiring an optical camera would break compatibility for over 80% of our target hardware fleet. (QR code display is preserved as an optional visual aid for camera-equipped laptops/tablets, but not mandatory).
- **Option 1D: Cloud Account Magic Link / Cloud Relay Pairing**
  Both computers log into a Coachify Cloud Account and pair through a centralized cloud server.
  - *Pros*: Traverses complex subnets automatically.
  - *Cons*: Completely unviable: breaks Coachify's offline-first architecture whenever broadband cuts out.

### 2. Transport & Cryptographic Security: Pinned TLS 1.3 + CSPRNG Token vs Raw PAKE vs Plain LAN HTTP

- **Option 2A: Pinned TLS 1.3 Transport with CSPRNG Token Exchange (Selected)**
  The workstation first verifies the host's pinned TLS 1.3 certificate fingerprint (from mDNS or visual check). Over this encrypted tunnel, the 6-digit code is submitted, and the host issues a 256-bit CSPRNG bearer token stored as a SHA-256 hash in SQLite.
  - *Pros*: Reuses the battle-tested, high-performance TLS 1.3 infrastructure from Spec 0044; protects against all packet sniffing on office Wi-Fi; simple and reliable implementation using standard HTTP/JSON contracts; constant-time comparison prevents timing attacks.
  - *Cons*: Relies on TLS certificate fingerprint verification as the first line of trust (already solved in Spec 0044).
- **Option 2B: Pure PAKE (Password-Authenticated Key Exchange, e.g. SPAKE2+ or SRP)**
  Implement an in-band SPAKE2+ protocol over raw TCP sockets to derive a session key directly from the 6-digit code without trusting any initial certificate.
  - *Pros*: Cryptographically zero-trust even over an unencrypted channel.
  - *Cons*: Substantial cryptographic complexity; requires multi-round-trip stateful handshakes; difficult to debug; redundant given that TLS 1.3 with certificate fingerprint pinning is already fully active.
- **Option 2C: Unencrypted Plain HTTP Pairing**
  Transmit the 6-digit code and device tokens in plaintext HTTP over the local network.
  - *Pros*: Simplest conceivable code.
  - *Cons*: Catastrophic security failure. Anyone with Wireshark on the office Wi-Fi could intercept the pairing code or the bearer token and gain permanent unauthorized access to the database.

### 3. Workstation Credential Custody: Windows DPAPI vs Plaintext Configuration File vs Encrypted SQLite Vault

- **Option 3A: Windows Data Protection API (DPAPI via `CryptProtectData`) (Selected)**
  The workstation encrypts the device token using Windows DPAPI, storing the encrypted blob in `%LocalAppData%\Coachify\workstation_credentials.dat`.
  - *Pros*: Native Windows platform security; cryptographic keys are managed securely by the Windows OS and tied to the user profile and machine TPM/LSA secrets; zero passwords or master keys required from the cashier; protected against cold-disk extraction or casual file copying.
  - *Cons*: Windows-specific API (Coachify desktop targets Windows 10/11 64-bit per `AGENTS.md`).
- **Option 3B: Plaintext JSON Config File (`settings.json`)**
  Store the device token in plaintext in the application data folder.
  - *Pros*: Trivially simple to read and write.
  - *Cons*: Any low-privilege script, malware, or user copying the folder to a USB stick can steal the workstation identity and clone the device.
- **Option 3C: Passphrase-Encrypted SQLite Database**
  Require the cashier to enter a master password each time Coachify starts to unlock an encrypted SQLite database containing the token.
  - *Pros*: Portable across operating systems.
  - *Cons*: Horrible cashier ergonomics: defeats the purpose of pairing if staff must enter a complex database password every morning before logging into their local account.

### 4. Workstation Revocation & Immediate Disconnection: Push SSE + Transport Gate vs Periodic Polling vs WebSockets

- **Option 4A: Dual-Barrier: Server-Sent Events (SSE) Push + Transport Layer Gate (Selected)**
  Each workstation maintains an open, lightweight TLS 1.3 SSE stream (`GET /events`). Revocation pushes a targeted `device.revoked` event down the pipe, immediately triggering client UI termination (< 500ms). Concurrently, any in-flight or subsequent HTTP RPC carrying that device token is rejected with HTTP 403 `DEVICE_REVOKED`.
  - *Pros*: Guaranteed sub-500ms disconnection; extremely low memory footprint compared to full duplex WebSockets; standard HTTP/1.1 and HTTP/2 semantics; dual-barrier guarantees that even if the SSE stream dropped momentarily, the workstation is blocked instantly at the transport gate.
  - *Cons*: Host must maintain one open persistent connection per paired workstation (trivially handled by Axum/Tokio: 5 connections use < 100 KB of RAM).
- **Option 4B: Periodic Client Polling (Heartbeat query every 10–30 seconds)**
  Workstations poll the host periodically to check their status.
  - *Pros*: Stateless host architecture.
  - *Cons*: Unacceptable security lag: a revoked cashier could continue performing sensitive financial transactions or exporting records for up to 30 seconds after the owner clicks "Revoke".
- **Option 4C: Full Duplex WebSockets**
  Maintain bidirectional WebSocket connections for all RPC traffic.
  - *Pros*: Instant push capability.
  - *Cons*: Heavy complexity: replaces standard REST/RPC request-response mechanics with stateful framing; harder to inspect and recover across network blips compared to standard HTTPS RPC + lightweight SSE control stream.

### 5. Workstation Quota Enforcement: Hard Limit on Pairing vs Floating Concurrent Leases

- **Option 5A: Hard Limit on Registered Active Workstations (Selected)**
  A license tier specifies a maximum number of active paired workstations (e.g. 2 for Standard, 5 for Plus). The host blocks pairing when `active_workstations >= limit`. Revoking a machine immediately frees up a slot.
  - *Pros*: Crystal clear to the customer; aligns with commercial desktop software conventions; prevents license abuse; zero risk of unexpected "no seats available" lockouts during busy mornings.
  - *Cons*: If an owner buys a new computer, they must explicitly revoke the old one in Settings before pairing the new one (solved by an intuitive one-click "Revoke & Replace" action in UI).
- **Option 5B: Floating Concurrent Leases (Dynamic Login Seats)**
  Any number of computers can be paired, but only N computers can be logged in simultaneously.
  - *Pros*: Flexibility if staff rotate between different PCs.
  - *Cons*: Highly disruptive in practice: if Cashier 1 is taking a payment and Cashier 2 opens Coachify, Cashier 1 gets kicked out mid-receipt; generates confusion and panic during morning admissions.

## Rationale Summary

We selected **Option 1A (6-digit OTP)**, **Option 2A (Pinned TLS 1.3 + CSPRNG Token)**, **Option 3A (Windows DPAPI)**, **Option 4A (Push SSE + Transport Gate)**, and **Option 5A (Hard Seat Limit)** because they deliver an unbeatable combination of:

1. **Frictionless Counter Onboarding**: Staff can pair a new counter in under 30 seconds by simply reading a 6-digit code off the owner's monitor, requiring zero technical literacy.
2. **Defensive Cryptography**: Despite the human-friendly 6-digit code, the system provides bank-grade security: the code is only valid for 10 minutes, protected by a 5-attempt anti-brute-force lockout, transmitted exclusively across an encrypted TLS 1.3 tunnel, and resolved into a 256-bit persistent token stored safely via Windows DPAPI.
3. **Deterministic Administrative Power**: The owner has real-time visibility into every machine connected to the center's database. If a cashier leaves or a laptop is misplaced, clicking "Revoke" severs access within 500 milliseconds, wipes local credentials, and releases the license seat immediately.
4. **Commercial Integrity**: License limits (`Coachify Local Pro` workstation quotas) are enforced strictly at the database boundary, protecting business revenues while giving customers clear, fair controls to reassign seats when hardware is replaced.

## References

- **Spec 0007**: `docs/specs/0007-setup-wizard-key-management/index.md` (Initial `devices` table and local account security)
- **Spec 0037**: `docs/specs/0037-desktop-licensing-and-read-only-mode/index.md` (License capabilities and limits)
- **Spec 0044**: `docs/specs/0044-lan-host-service-and-discovery/index.md` (LAN host runtime, TLS 1.3 listener, and mDNS discovery)
- **RFC 8446**: The Transport Layer Security Protocol Version 1.3 (IETF)
- **RFC 6762 / 6763**: Multicast DNS and DNS-Based Service Discovery
- **Microsoft Learn**: Windows Data Protection API (DPAPI) and `CryptProtectData` Architecture
- **OWASP Application Security Guide**: Cryptographic Storage and Brute Force Authentication Defense
