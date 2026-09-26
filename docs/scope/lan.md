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

### 70. Workstation pairing & device management · needs a decision · GA
Pair workstation PCs using a six digit handshake, grant persistent device tokens, and let the host owner inspect, rename, or revoke connected PCs.
**Done when:** a workstation pairs with a six digit code, gets an authorized device token, appears in the host device registry, and disconnects immediately if revoked.
- [ ] Design it (spec): `/architect workstation pairing & device management`

### 71. LAN workstation client mode & RPC transport · needs a decision · GA
Workstation app mode that forwards UI RPC commands over pinned TLS to the host database while keeping document printing local to the workstation.
**Done when:** a workstation connects to the host, navigates all app screens against the host database, recovers smoothly from brief network disconnects, and prints receipts on its local printer.
- [ ] Design it (spec): `/architect lan workstation client mode & rpc transport`

### 72. LAN host update mirror · needs a decision · GA
Mirror verified software updates on the host PC so client workstations download and install app updates directly across the local network without internet.
**Done when:** a workstation notices a newer app version on the host, downloads the signed update directly from the host PC, verifies the signature, and updates without internet access.
- [ ] Design it (spec): `/architect lan host update mirror`
