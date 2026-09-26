# Rationale: Mobile App and LAN QR Pairing

## Context

Coaching centers in Bangladesh operate in physically active, high velocity environments. While the primary coaching database and cashier desk reside on the owner's desktop computer, teachers move between classrooms holding physical class sheets, counter clerks handle queues of parents paying fees during morning peaks, and owners walk the coaching floor supervising operations. Expecting every staff member to sit at a full desktop PC is financially prohibitive and operationally rigid.

Equipping coaching staff with smartphones and tablets provides immense operational leverage, but introduces formidable networking and security challenges:

1. **Intermittent and Local Only Connectivity**: Most coaching centers lack static IP addresses and run on consumer Wi-Fi routers where broadband frequently drops. The mobile solution must work completely offline from the global internet, communicating strictly across the local area network.
2. **The Mobile OS Trust Barrier**: Modern mobile operating systems (Android 10+ and iOS 14+) enforce strict transport security policies (Android Network Security Configuration and Apple App Transport Security). They categorically reject self signed certificates from local IP addresses and refuse plain HTTP connections. Expecting a non technical coaching center owner to install custom root Certificate Authority profiles on employee personal phones is completely unacceptable.
3. **Typing Friction and DHCP Churn**: Mobile users cannot reasonably be asked to type complex IPv4 addresses like `192.168.1.145:50443` or 64 character cryptographic hashes. Routers frequently reassign host IP addresses via DHCP, which would break manually entered configurations.
4. **Data Integrity and Financial Safety**: Student tuition fees, admission payments, and attendance records must remain strictly synchronized. A naive mobile implementation with disconnected local databases risks severe data divergence, accidental double billing, or loss of offline transactions if a phone is damaged or stolen.

## Options considered

### Option 1: Tauri 2 Mobile Companion with Optical QR TLS Pinning (Chosen)

Build the mobile companion using Tauri 2 Mobile (React 19, TypeScript, Rust), sharing the existing frontend codebase and typed RPC schema. The desktop host displays a dynamic QR code containing its IP addresses, port, and X.509 SHA-256 certificate fingerprint alongside an ephemeral pairing secret. Scanning the QR code with the mobile camera configures custom TLS certificate pinning directly in the Rust network transport, establishes a mutual TLS 1.3 handshake, and provisions a persistent hardware keystore device token.

**Pros**:
- Near complete code reuse: shares desktop React 19 views, Tailwind CSS v4 styling, and Specta typed RPC contracts without duplicate development.
- Zero mobile certificate warnings: optical transfer of the certificate fingerprint solves local TLS trust cleanly without installing system profiles.
- Native camera speed: native barcode scanning plugin leverages Google ML Kit and Apple Vision for sub 150ms attendance scanning.
- Hardware level credential security: tokens persist safely in Android Keystore and iOS Keychain.

**Cons**:
- Requires mobile build toolchains (Android SDK and Xcode) to produce native app binaries.
- Tauri 2 mobile ecosystem has fewer third party community plugins compared to React Native, though core plugins cover all Coachify requirements.

### Option 2: Standalone React Native with Expo Companion

Build a separate React Native application using Expo, creating custom mobile views and connecting to the desktop host HTTPS listener.

**Pros**:
- Mature mobile ecosystem with extensive native libraries and camera integrations.
- Excellent developer tooling with Expo Go for quick client iteration.

**Cons**:
- Complete duplication of UI code: requires rebuilding all Coachify attendance, cashier, and dashboard views in React Native primitives.
- Contract drift: TypeScript RPC contracts and validation rules must be manually copied or synchronized between the desktop repository and the mobile repository.
- Increased maintenance burden: two distinct engineering stacks to test, update, and maintain.

### Option 3: Flutter Cross Platform Mobile Application

Build the mobile application from scratch in Dart using the Flutter framework, communicating with the Axum host via JSON REST calls.

**Pros**:
- High performance custom rendering engine with consistent UI across Android and iOS.
- Rich ecosystem of mobile hardware plugins.

**Cons**:
- Complete language rewrite: introduces Dart into a team and codebase built entirely on Rust and TypeScript.
- Zero component sharing: none of Coachify's existing React components or Tailwind tokens can be reused.
- Heavy binary sizes and separate state management architecture.

### Option 4: Responsive Progressive Web App (PWA) over LAN

Do not build a native mobile app; instead instruct mobile staff to open their mobile web browser (Chrome or Safari) and browse to the host desktop local IP address.

**Pros**:
- Zero app store distribution or mobile build pipelines required.
- Accessible from any smartphone browser immediately.

**Cons**:
- Fatal user experience failure: mobile browsers display terrifying red security warnings ("Your connection is not private") because of the host self signed certificate, requiring users to click hidden bypass links.
- No native background camera scanning: browser camera APIs exhibit severe latency and require repeated permission prompts.
- No hardware keystore integration: sensitive bearer tokens must sit in vulnerable browser localStorage.
- No push notification or background SSE maintenance when the phone screen turns off.

## Deep dive on key tradeoffs

### Zero Configuration TLS Verification: Optical Pinning vs System CA Profiles

Mobile operating systems deliberately prevent unencrypted HTTP communication and reject self signed certificates to protect users against Wi-Fi eavesdropping. Enterprise solutions typically resolve this by installing a custom Certificate Authority root profile onto every device using mobile device management (MDM) software.

For Bangladeshi coaching centers, this enterprise approach is impossible. Staff use personal Android and iPhone devices, and coaching center administrators lack the technical capability to manage root certificates. 

Optical QR TLS pinning elegantly dissolves this dilemma. The desktop host screen is an authentic visual channel. By encoding the SHA-256 fingerprint of the host certificate directly into the QR code, the mobile camera transfers cryptographic trust out of band. When the mobile client initiates its TLS 1.3 handshake, it does not check Android or iOS system certificate stores; instead, it checks whether the certificate presented by the server matches the exact SHA-256 hash scanned from the monitor. This achieves total cryptographic protection against Wi-Fi spoofing and man in the middle attacks with zero operating system configuration.

### Instant Scan Pairing vs Two Way Desktop Confirmation

Spec 0045 defined workstation pairing for secondary desktop PCs using a manual 6 digit numeric code. For mobile devices, requiring the operator to type a 6 digit code or click secondary confirmation dialogs on the desktop adds needless friction.

Because optical QR scanning requires physical line of sight to the host computer monitor, physical presence is already proven. An unauthorized attacker sitting outside the office on Wi-Fi cannot see the host monitor. Therefore, scanning the ephemeral 256 bit QR secret immediately completes mutual authentication and provisions the device token. To safeguard against shoulder surfing or malicious employees taking photos of the screen, pairing windows expire after 10 minutes and lock out automatically after 5 failed attempts.

### Strict Live Mode vs Offline Local Queueing

Coaching centers sometimes experience spotty Wi-Fi in classrooms with thick brick walls. A natural temptation is to implement an offline queue that stores attendance scans and payments locally on the phone, syncing back when Wi-Fi reconnects.

However, financial transactions and student admissions cannot be safely reconciled asynchronously without complex conflict resolution. If two cashiers collect fees offline for the same student, double billing and receipt sequence corruption occur. Furthermore, local database replicas on employee personal phones create severe data leakage risks if a device is lost or stolen.

Coachify adopts strict live RPC mode: all mutations execute directly against the host SQLite database. For brief Wi-Fi drops, the mobile network layer provides automatic background reconnection with exponential backoff and non blocking UI alerts. Classroom attendance scanning leverages fast optimistic visual feedback while maintaining a synchronous network pipe to the host.

### Host Print Routing vs Mobile Native Bluetooth Printers

When staff collect fees at a front desk counter or roaming kiosk using a smartphone, parents expect a printed physical receipt. Equipping every mobile staff member with an expensive battery powered Bluetooth thermal printer adds substantial hardware expense and charging maintenance.

Coachify resolves this by routing print jobs through the desktop host. When a mobile cashier completes a payment, the mobile app sends an RPC print command to the host PC (`POST /print/host-receipt`). The desktop host immediately prints the receipt on its heavy duty USB thermal counter printer. For parents who prefer digital records, the mobile app simultaneously offers an instant WhatsApp share button that generates and shares a digital PDF receipt.

### Native Camera Viewfinder vs Browser WebRTC

Continuous attendance scanning requires scanning dozens of student ID badges consecutively with zero hesitation. In browser environments, camera video processing through canvas draws introduces 400ms to 800ms delays and causes significant battery drain and thermal throttling.

By utilizing `@tauri-apps/plugin-barcode-scanner`, Coachify delegates image processing directly to platform hardware: Google ML Kit on Android and Apple Vision on iOS. This achieves hardware accelerated barcode recognition under 150 milliseconds, allowing teachers to scan an entire classroom of students seamlessly.

### Parallel IP Candidate Racing vs Sequential Connection Fallback

Windows desktop PCs hosting Coachify often possess multiple network interface adapters simultaneously. A typical host machine might have an active Wi-Fi adapter, a physical Ethernet connection, and virtual network bridge adapters created by software such as WSL, Hyper-V, or Docker. 

If the QR payload encodes multiple IP addresses and the mobile client attempts to connect sequentially, an unreachable virtual adapter listed first will trigger the operating system default TCP socket timeout (typically 20 to 30 seconds). This would cause pairing to freeze and violate the user experience requirement of five second pairing.

Coachify resolves this through a parallel Happy Eyeballs candidate racing algorithm. The mobile client initiates concurrent TLS 1.3 handshakes to all host IP candidates with a strict 2.5 second timeout. The first IP address to complete the TLS handshake and verify the pinned certificate fingerprint becomes the active transport. All competing connection attempts are cancelled immediately, guaranteeing instant pairing even across complex multi homed host networks.

### Staff PIN Session Lifecycle Atop Device Tokens

A persistent hardware device token (`Bearer dt_...`) authenticates the physical smartphone as a trusted workstation. However, a coaching center phone is frequently shared among staff (such as counter receptionists on alternating shifts) or handled by student assistants taking attendance. Allowing anyone who picks up an unlocked phone to execute financial transactions or alter records would severely compromise accountability.

Coachify separates device authentication from human operator authentication. The device token proves the hardware is authorized. To perform actions, staff members must authenticate with their personal numeric Staff PIN via `POST /auth/staff-login`. The host verifies the PIN using Argon2id and returns a scoped session token (`st_...`). To prevent unauthorized access if the phone is set down on a desk, the mobile app automatically locks after 15 minutes in the background or 4 hours of inactivity, requiring PIN re-entry or biometric unlock.

### Dedicated Hardware Receipt Printer vs Windows OS Default

In coaching centers, the host PC is often attached to multiple printers: an A4 laser printer for office reports and exam papers, a thermal receipt printer for counter payments, and virtual software printers like Microsoft Print to PDF.

Relying on the host Windows default printer causes severe operational bugs. If an administrator prints an invoice to the office laser printer, the default printer shifts, causing subsequent mobile cashier receipts to print onto expensive A4 paper or stall waiting for manual paper feeder trays. 

Coachify requires the host configuration to store an explicit `receipt_printer_name` in `lan_host_config`. The mobile print routing endpoint (`POST /print/host-receipt`) sends ESC/POS print jobs directly to this configured hardware printer, completely isolated from whatever document printer the host desktop operator was using.

### Android Multicast Suppression and Quick Scan IP Recovery

Spec 0044 uses multicast DNS (`_coachify._tcp.local`) for automatic host discovery. While mDNS functions reliably on desktop operating systems, Android implementation through `NsdManager` is notoriously brittle. Many Android handset manufacturers aggressively throttle or kill background multicast listening to conserve battery power. Furthermore, budget Wi-Fi routers frequently enable IGMP snooping or client isolation, silently dropping mDNS broadcast packets.

If a coaching center router reboots and assigns a new DHCP IP to the host PC, an Android phone where mDNS is suppressed would be permanently stranded in an endless reconnection loop. To eliminate this point of failure, Coachify provides an optical fallback. If automatic reconnection cannot locate the host after 10 seconds, the mobile app displays an "Update Host IP" button. Staff simply point the camera at the host monitor to scan the QR code. The app extracts the new IP and port and resumes live operation instantly, without invalidating the existing device token or requiring administrative re-pairing.

## Decision & Justification

Tauri 2 Mobile (Option 1) is selected as Coachify's official mobile architecture. It maximizes investment in our existing desktop React 19 and Rust codebase while meeting every operational, security, and performance requirement of Bangladesh coaching centers. Optical QR pairing provides effortless setup without technical friction, TLS fingerprint pinning guarantees airtight data safety across local Wi-Fi, and strict live RPC protects financial ledger integrity.

## References

### Project sources
- `AGENTS.md`: Stack conventions and architecture boundaries.
- Spec 0044 (`docs/specs/0044-lan-host-service-and-discovery/index.md`): LAN host runtime, TLS 1.3 listener, and self signed certificates.
- Spec 0045 (`docs/specs/0045-workstation-pairing-and-device-management/index.md`): Workstation pairing, device tokens, and SSE control streams.
- Spec 0034 (`docs/specs/0034-ui-ux-design-sizing-typography-responsiveness/index.md`): Responsive design system and mobile touch ergonomics.
- Spec 0023 (`docs/specs/0023-attendance/index.md`): Attendance domain models and rapid marking workflows.

### Practices & standards
- **RFC 8446: The Transport Layer Security (TLS) Protocol Version 1.3**: Cryptographic transport security.
- **OWASP Mobile Security Testing Guide**: Mobile certificate pinning and hardware keystore credential storage.
- **RFC 6762 / RFC 6763**: Multicast DNS and DNS Based Service Discovery.
