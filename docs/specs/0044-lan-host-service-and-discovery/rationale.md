# 0044. LAN Host Service and Discovery: Rationale

> Decision record and architectural rationale for Spec 0044 (LAN Host Service and Discovery, Feature 69)

## Context

In Bangladesh, coaching centers often have multiple administrative desks during peak admission, exam enrollment, or monthly tuition collection periods: a main reception desk, one or two cashier counters, and an academic coordinator's office. (basis: project architecture review, Section 5 and Section 9). While smaller institutions operate happily with a single standalone PC, growing institutions require multiple computers working against the same live database simultaneously.

However, coaching centers face severe environmental and infrastructure constraints:
1. **Intermittent Internet**: Broadbands frequently cut out, and mobile hotspots are unreliable. Counter computers must function at full speed without an active internet connection.
2. **Shared, Untrusted Wi-Fi Networks**: Coaching centers often provide Wi-Fi to teachers, students, and visitors on the same router or subnet. Plaintext network traffic would expose student phone numbers, guardian identities, payment records, and staff credentials to anyone running a basic packet capture tool.
3. **Dynamic IP Assignments (DHCP)**: Typical consumer and small-business routers reassign IP addresses periodically or on router reboot. Requiring cashiers to determine the host's IP address and manually reconfigure it causes constant desk stoppages.
4. **Desktop Power and Sleep Defaults**: Windows laptops and desktops are configured by default to enter sleep after 15–30 minutes of inactivity. If the primary host computer goes to sleep while cashiers are taking payments, client workstations freeze.
5. **Accidental Window Closures**: Non-technical staff frequently click the "X" button to close windows, inadvertently terminating background services if hosting is tied solely to visible window lifecycles.

To solve these real-world challenges, Coachify's LAN architecture must provide a rock-solid, secure, zero-configuration local networking service that is easy to manage and resilient against common office IT mishaps.

## Options Considered

### 1. Transport Protocol: HTTPS (TLS 1.3) vs Plain HTTP vs gRPC

- **Option 1A: HTTPS (TLS 1.3) with Pinned Self-Signed Certificate (Selected)**
  The host generates a self-signed X.509 certificate with Subject Alternative Names for all local interfaces. Workstations pin the certificate's SHA-256 fingerprint upon initial handshake.
  - *Pros*: Complete end-to-end cryptographic privacy against eavesdropping on shared Wi-Fi; leverages standard, hardened HTTP/1.1 and HTTP/2 semantics; easy to test and debug using standard curl or browser tools; zero reliance on external public certificate authorities.
  - *Cons*: Requires custom certificate generation logic and client-side fingerprint pinning since self-signed certificates are not signed by a public root CA.
- **Option 1B: Plain HTTP over LAN**
  Host listens on an unencrypted HTTP port.
  - *Pros*: Simplest implementation with no certificate generation or TLS handshake overhead.
  - *Cons*: Disastrous security failure on shared coaching center Wi-Fi. Passwords, session tokens, student phone numbers, and financial payments would travel in plaintext across the local airwaves.
- **Option 1C: gRPC over HTTP/2**
  Use Google Protocol Buffers and gRPC streaming.
  - *Pros*: Strongly typed contracts with binary efficiency.
  - *Cons*: Introduces complex protobuf toolchains; unneeded streaming overhead for simple request/response RPC; significantly harder to troubleshoot on customer PCs compared to JSON-over-HTTPS.

### 2. Discovery Mechanism: mDNS (DNS-SD) with Manual IP Fallback vs UDP Broadcast vs Cloud Rendezvous

- **Option 2A: Multicast DNS (mDNS / DNS-SD `_coachify._tcp.local`) with Manual Fallback (Selected)**
  Host announces its service using standard mDNS / DNS-SD. Workstations browse for services on the local link. If a router suppresses multicast, manual IP/hostname entry is supported.
  - *Pros*: Zero-configuration discovery on standard home and office routers; workstations discover the host by name even after DHCP reassigns IP addresses; manual IP fallback ensures the system still works if router multicast is blocked.
  - *Cons*: Multicast traffic can be filtered on certain enterprise-grade routers with strict IGMP snooping or client isolation enabled (mitigated by manual IP fallback and UI diagnostics).
- **Option 2B: Custom UDP Broadcast Pings**
  Host and client send custom UDP broadcast datagrams on a fixed port.
  - *Pros*: Simple custom protocol.
  - *Cons*: Custom protocol lacks DNS-SD standard tooling; often blocked by Windows Firewall default rules; does not support standard structured TXT records as cleanly as mDNS.
- **Option 2C: Cloud Rendezvous / Relay Discovery**
  Host registers its local IP with a cloud server, and workstations query the cloud server to find the host.
  - *Pros*: Traverses complex local subnets.
  - *Cons*: Completely breaks during internet outages, violating Coachify's foundational offline-first guarantee.

### 3. Service Lifecycle: In-Process Desktop Host with Tray & Sleep Inhibition vs Windows Background Service

- **Option 3A: In-Process Desktop Host with Win32 Sleep Inhibition and System Tray (Selected)**
  The host listener runs inside the primary Coachify desktop application process on a background Tokio runtime. Closing the window hides the app to the system tray, and `SetThreadExecutionState` prevents Windows from sleeping.
  - *Pros*: Single unified installer and application binary; zero administrative privilege escalation required for daily operation; shared access to SQLite database without complex cross-process locking or IPC; user always has visible status indicators.
  - *Cons*: The primary computer must remain turned on and logged in with the application running.
- **Option 3B: Headless Windows Service (Daemon)**
  Install a standalone Windows Service running under the `LocalSystem` or `NetworkService` account.
  - *Pros*: Runs even when no user is logged into Windows.
  - *Cons*: Substantially more complex deployment requiring administrator privileges; difficult to update atomically alongside the desktop UI; cross-process SQLite file locks between the Windows service and desktop UI introduce severe corruption and concurrency risks. (Preserved as a potential V2 enterprise enhancement if field telemetry proves necessity).

### 4. Certificate Custody: SQLite-Persisted Certificate vs Ephemeral Certificate

- **Option 4A: SQLite-Persisted Certificate and Key (`lan_host_config`) (Selected)**
  The self-signed certificate and private key are generated once on initial host setup, stored in SQLite (`lan_host_config`), and backed up along with the database.
  - *Pros*: The host certificate fingerprint remains identical across app restarts, IP reassignments, and even full hardware migrations when restoring a backup to a replacement PC. Workstations never suffer broken trust errors or require re-pairing after a reboot or hardware migration.
  - *Cons*: Requires secure database storage and DPAPI protection for key material.
- **Option 4B: Ephemeral In-Memory Certificate Generated on Each Boot**
  Generate a new self-signed certificate in memory every time the host application launches.
  - *Pros*: No disk persistence of private keys.
  - *Cons*: SHA-256 fingerprint changes on every restart. Paired workstations would reject the host certificate after every reboot, requiring tedious re-pairing every morning.

## Rationale

We selected Option 1A, Option 2A, Option 3A, and Option 4A because they directly align with the core philosophy of Coachify: **absolute operational speed, offline reliability, ironclad security, and zero IT friction for coaching center operators**.

1. **Defense in Depth on Local Airwaves**:
   Coaching center Wi-Fi routers are rarely secured with enterprise 802.1X isolation. By mandating TLS 1.3 encryption and certificate fingerprint pinning, Coachify guarantees that sensitive student records, attendance data, and fee payments are impenetrable to packet sniffing, even if unauthorized parties connect to the office router.
2. **Resilience to DHCP IP Churn**:
   Consumer routers in Bangladesh frequently reset or power-cycle due to load-shedding, causing host IP addresses to change from `192.168.1.100` to `192.168.1.105`. Because mDNS advertises the service by unique `host_id` rather than raw IP, workstation clients discover the host seamlessly without requiring an IT technician to assign static IP reservations.
3. **Operational Continuity via Sleep Inhibition & System Tray**:
   Front desk operators cannot be expected to know advanced Windows power-management settings. By programmatically calling `SetThreadExecutionState` while hosting is active, Coachify ensures that Windows does not enter standby or sleep modes during office hours. Intercepting window close events to minimize to the system tray guarantees that an accidental click of the "X" button will never take secondary cashier counters offline.
4. **Dispatcher Level License Enforcement**:
   Following our project-wide gating conventions, `lan.start_host` is guarded by `LicenseGate::Lan` directly within the core command dispatcher. This cleanly enforces our commercial tiering (`Coachify Local Pro` entitlement) while keeping the network runtime strictly decoupled from licensing business logic.

## References

**Project sources**:
- `AGENTS.md`: Stack conventions, core domain separation, and command dispatcher rules
- `docs/architecture-review.md`: Section 5 (LAN Architecture), Section 8 (Commercial Rules), Section 9 (Licensing Model), and Section 20 (Decision 2 and Decision 3)
- `docs/scope/lan.md`: Slice 13 (LAN Multi PC) scope definitions
- Spec 0004: `docs/specs/0004-command-dispatcher-local-accounts/index.md` (Command dispatcher pipeline and permission gating)
- Spec 0037: `docs/specs/0037-desktop-licensing-and-read-only-mode/index.md` (Desktop licensing and entitlement capabilities)

**Standards & APIs**:
- **TLS 1.3 Protocol**: RFC 8446 (The Transport Layer Security Protocol Version 1.3)
- **Multicast DNS (mDNS)**: RFC 6762 (Multicast DNS)
- **DNS-Based Service Discovery (DNS-SD)**: RFC 6763 (DNS-Based Service Discovery)
- **X.509 Public Key Infrastructure**: RFC 5280 (Internet X.509 Public Key Infrastructure Certificate and Certificate Revocation List Profile)
- **Windows Power Management**: Win32 `SetThreadExecutionState` API (Microsoft Learn)
- **Windows Network List Manager**: INetworkListManager COM interface & PowerShell `Get-NetConnectionProfile`
