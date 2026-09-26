# LAN Multi PC (Slice 13)

Back to [index](index.md). Enables Coachify Local Pro where a coaching center runs multiple PCs across a local network without an active internet connection.

### 69. LAN host service & discovery · done · GA
Host mode listener on the primary PC with local discovery, port binding, and pinned TLS certificate creation so workstation PCs connect safely.
**Done when:** the host starts its local listener, announces itself on the local network, guides Windows firewall setup if needed, and answers an encrypted health check.
spec [0044](../specs/0044-lan-host-service-and-discovery/index.md)
- [x] Design it (spec): `/architect lan host service & discovery`
- [x] Build it: `/develop lan host service & discovery`
  - [x] SQLite Migration 0024 (`lan_host_config`), host key & X.509 certificate generation with SANs, SHA-256 fingerprint derivation, and database persistence, covers AC-1, AC-2
  - [x] Asynchronous TLS 1.3 HTTPS listener on port 50443, request router, rate-limited `GET /hello` discovery endpoint, and `GET /health` encrypted health check, covers AC-3, AC-4, AC-5
  - [x] Multicast DNS (mDNS / DNS-SD `_coachify._tcp.local`) responder with TXT metadata and client discovery scanner with manual IP fallback, covers AC-6, AC-7
  - [x] Windows network profile (`Private` vs `Public`) detection, firewall rule guidance, Win32 `SetThreadExecutionState` sleep inhibitor, and system tray minimize integration, covers AC-8, AC-9
- [x] Verify it: `/check verify lan host service & discovery`
- [x] Test it: `/test lan host service & discovery`
- [x] Review it (fresh model): `/check review lan host service & discovery`
- [x] Document it: `/document lan host service & discovery`

### 70. Workstation pairing & device management · done · GA
Pair workstation PCs using a six digit handshake, grant persistent device tokens, and let the host owner inspect, rename, or revoke connected PCs.
**Done when:** a workstation pairs with a six digit code, gets an authorized device token, appears in the host device registry, and disconnects immediately if revoked.
spec [0045](../specs/0045-workstation-pairing-and-device-management/index.md)
- [x] Design it (spec): `/architect workstation pairing & device management`
- [x] Build it: `/develop workstation pairing & device management`
  - [x] SQLite Migration 0025 (`devices` extension, `lan_pairing_sessions`, `device_events` audit trail with immutable triggers), covers AC-5, AC-10
  - [x] Ephemeral CSPRNG 6-digit numeric pairing code with 10-minute expiry, constant-time verification, and 5-attempt brute-force lockout with 15-minute cooldown, covers AC-1
  - [x] Mutual cryptographic TLS 1.3 handshake on `/pair`, single-use session consumption, and 256-bit persistent bearer device tokens (`dt_...`) encrypted via Windows DPAPI machine-scope, covers AC-2, AC-3
  - [x] Hard commercial license quota enforcement (`workstation_limit`) preventing pairing and unblocking beyond capacity, covers AC-4
  - [x] Complete workstation lifecycle management: rename, block (with immediate user session purge), unblock, and permanent revoke (`revoked:...` tombstoning), covers AC-6, AC-7, AC-8
  - [x] Real-time Server-Sent Events (SSE `GET /events`) control channel pushing sub-500ms disconnect signals for revoked and blocked workstations, covers AC-9
  - [x] Comprehensive Settings -> Workstations management UI with seat quota progress bar, pairing modal with countdown and copy, device inventory table, action dialogs, and activity history drawer, covers AC-1 through AC-10
- [x] Verify it: `/check verify workstation pairing & device management`
- [x] Test it: `/test workstation pairing & device management`
- [x] Review it (fresh model): `/check review workstation pairing & device management`
- [x] Document it: `/document workstation pairing & device management`

### 70b. Mobile app companion & optical QR pairing · done · GA
Mobile companion app for iOS and Android that pairs in seconds through optical QR scanning, with hardware certificate pinning, camera attendance scanning, cashier payments, and host print routing.
**Done when:** a mobile phone scans the host screen QR code, completes a pinned TLS 1.3 handshake, stores credentials in hardware keystore, scans student cards with camera audio feedback, and disconnects within 500ms if revoked.
spec [0046](../specs/0046-mobile-app-and-lan-qr-pairing/index.md)
- [x] Design it (spec): `/architect mobile app and pairing in LAN through QR`
- [x] Build it: `/develop mobile app companion & optical QR pairing`
  - [x] Ephemeral QR generation, URI formatting, anti brute force lockout, and host pairing endpoint, covers AC-1, AC-2, AC-4, AC-5
  - [x] Tauri 2 Mobile client scaffolding with custom TLS certificate pinning and hardware keystore credential custody, covers AC-3, AC-4
  - [x] High speed camera barcode and QR attendance scanner with audio feedback and duplicate debounce, covers AC-6
  - [x] Mobile cashier payment view, admissions, and host thermal printer dispatch with WhatsApp PDF sharing, covers AC-7, AC-8
  - [x] Real time SSE control channel, sub 500ms instant remote severance, and audit logging, covers AC-8, AC-9, AC-10
- [x] Verify it: `/check verify mobile app companion & optical QR pairing`
- [x] Test it: `/test mobile app companion & optical QR pairing`
- [x] Review it (fresh model): `/check review mobile app companion & optical QR pairing`
- [x] Document it: `/document mobile app companion & optical QR pairing`

### 71. LAN workstation client mode & RPC transport · done · GA
Workstation app mode that forwards UI RPC commands over pinned TLS to the host database while keeping document printing local to the workstation.
**Done when:** a workstation connects to the host, navigates all app screens against the host database, recovers smoothly from brief network disconnects, and prints receipts on its local printer.
spec [0047](../specs/0047-lan-workstation-client-mode-and-rpc-transport/index.md) · code in `crates/core/src/lan.rs`, `crates/core/src/lib.rs`, `ui/src/components/layout/workstation-pill.tsx`, `ui/src/components/settings/workstation-settings-card.tsx`
- [x] Design it (spec): `/architect lan workstation client mode & rpc transport`
- [x] Build it: `/develop lan workstation client mode & rpc transport`
  - [x] Dual-mode RPC bridge routing local commands locally and domain commands remotely over pinned TLS 1.3, covers AC-1, AC-2, AC-3
  - [x] Strictly local document & thermal receipt printing preserving local Windows print spooler and hardware settings, covers AC-5
  - [x] Persistent SSE event fan-out pipeline broadcasting ChangeEvents to all connected counters with sub-500ms TanStack cache sync, covers AC-6
  - [x] Resilient network watchdog with exponential backoff and dynamic mDNS host re-discovery for DHCP IP shifts, covers AC-7
  - [x] Client-side UUIDv7 idempotency key injection ensuring zero duplicate payments or admissions on Wi-Fi drops, covers AC-8
  - [x] Workstation status header pill, connection popover, unpair dialog, and bilingual localization, covers AC-10
- [x] Verify it: `/check verify lan workstation client mode & rpc transport`
- [x] Test it: `/test lan workstation client mode & rpc transport`
- [x] Review it (fresh model): `/check review lan workstation client mode & rpc transport`
- [x] Document it: `/document lan workstation client mode & rpc transport`

### 72. LAN host update mirror · needs a decision · GA
Mirror verified software updates on the host PC so client workstations download and install app updates directly across the local network without internet.
**Done when:** a workstation notices a newer app version on the host, downloads the signed update directly from the host PC, verifies the signature, and updates without internet access.
- [ ] Design it (spec): `/architect lan host update mirror`
