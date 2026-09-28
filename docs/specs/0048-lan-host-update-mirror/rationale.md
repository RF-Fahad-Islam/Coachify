# 0048. LAN Host Update Mirror: Rationale

> Decision record and architectural rationale for Spec 0048 (LAN Host Update Mirror, Feature 72)

## Context

In coaching centers across Bangladesh, educational institutions run in environments characterized by:
1. **Patchy and Metered Connectivity**: While the main office computer may have a broadband connection or a 4G SIM router, secondary workstations on cashier counters or reception desks often have no direct internet access. Downloading a 100 MB installer separately on 3 to 5 workstations wastes expensive cellular data and can take hours on congested uplinks.
2. **Offline-First Survivability**: In rural or district areas (e.g. Bogura, Cumilla, Barishal), coaching centers can be completely offline for weeks. When an administrator receives an update file on a USB thumb drive from an IT consultant or vendor visit, they must be able to distribute it across all desks without internet.
3. **Version Skew Risks**: If the primary Host updates its database schema or RPC models to version 1.1.0 while counter workstations stay on version 1.0.0, transactions could fail due to protocol or model mismatches. Maintaining version parity across the counter fleet is an operational safety imperative.

## Options Considered

### 1. Update Distribution Protocol: Axum TLS 1.3 Streaming vs Windows SMB Shared Folder vs P2P Gossip

- **Option 1A: Integrated Axum TLS 1.3 Streaming (`/updates/*`) (Selected)**
  The Host PC's existing HTTPS listener on port 50443 exposes authenticated `/updates/manifest` and `/updates/download/:version` endpoints. Workstations reuse their established TLS 1.3 connection and bearer device token (`X-Coachify-Device-Token`) to download the binary.
  - *Pros*:
    - **Zero Additional Network Ports or Firewall Setup**: Operates on the exact same port (50443) and firewall exception already authorized for LAN Mode.
    - **Hardware-Bound Authentication**: Only paired and active workstations with valid device tokens can download update payloads.
    - **Universal Windows Compatibility**: No reliance on Windows Network Sharing, SMBv2/v3, NetBIOS, or guest access policies (which Windows 11 disables by default for security).
    - **Encrypted in Transit**: Pinned TLS 1.3 prevents tampering by other devices or rogue machines on the local Wi-Fi.
  - *Cons*: Requires minor streaming logic in the Host Axum router.
- **Option 1B: Windows SMB / Network File Sharing (`\\HOST\CoachifyUpdates`)**
  The Host creates a Windows shared directory on the local disk.
  - *Pros*: Offloads file transfer to Windows OS.
  - *Cons*:
    - **Severe Configuration Friction**: Windows 11 blocks insecure guest SMB access by default; requires complex Windows user account setup or registry hacks.
    - **Firewall Hurdles**: Requires opening multiple SMB ports (445, 139) that corporate or school firewalls routinely block.
    - **No Fine-Grained Authorization**: Cannot easily bind downloads to Coachify device tokens or revoke severed workstations.
- **Option 1C: Peer-to-Peer (P2P) Gossip / BitTorrent on LAN**
  Workstations share installer chunks among themselves.
  - *Pros*: Scalable for hundreds of nodes.
  - *Cons*: Massive over-engineering for a coaching center with 2 to 6 workstations. Introduces complex peer discovery, chunk hashing, and UDP socket overhead.

### 2. Update Notification Mechanism: Persistent SSE Push vs Scheduled Polling

- **Option 2A: SSE ChangeEvent Push (`system:update_available`) + On-Demand Check (Selected)**
  When the Host stages an update, it immediately emits a `ChangeEvent` across the active `/events` channel. Workstations display an update indicator within 500 ms. In addition, workstations poll `/updates/manifest` on connection startup or manual button click.
  - *Pros*: Instant real-time awareness without background CPU or network polling overhead. Zero lag between host staging and workstation visibility.
  - *Cons*: Requires workstations to handle the event in their existing SSE event listener (already implemented in Spec 0047).
- **Option 2B: Pure Polling (e.g. Every 15 Minutes)**
  Workstations periodically send HTTP requests to check for new versions.
  - *Pros*: Simple to reason about.
  - *Cons*: Workstations experience up to 15 minutes of delay before noticing a new update, or generate unnecessary network traffic and CPU wakeups.

### 3. Verification Strategy: Multi-Tier Cryptographic Integrity vs Simple Size Check

- **Option 3A: End-to-End Vendor Ed25519 Digital Signature & SHA-256 Check (Selected)**
  The installer package contains an Ed25519 signature generated during release signing (Spec 0042) using key `u2026a`. Both the Host (upon staging) and the Workstation (upon download) verify:
  1. File size equals expected manifest length.
  2. SHA-256 hash matches the signed manifest hash.
  3. Ed25519 signature verifies against the hardcoded vendor public key embedded in `crates/core`.
  - *Pros*: Completely eliminates the possibility of malware injection, rogue host spoofing, or corrupted transfers. Even if an attacker compromised the local router and redirected traffic, an altered binary will be rejected.
  - *Cons*: Requires cryptographic dependencies in `crates/core` (already present: `ed25519-dalek`, `sha2`).
- **Option 3B: Simple File Size & Checksum Only**
  Verify only that file size > 50 MB and SHA-256 matches what the Host reported.
  - *Pros*: Fewer lines of code.
  - *Cons*: If a rogue PC paired to the host managed to upload a bogus file, or if the host machine itself was compromised, untrusted code could execute on counter workstations.

## Decision Summary

We implement **Option 1A (Axum TLS 1.3 Streaming)** combined with **Option 2A (SSE Push + Query)** and **Option 3A (Full Ed25519 + SHA-256 Cryptographic Verification)**. This provides maximum security, zero network configuration headache, instant multi-desk synchronization, and robust support for both connected and air-gapped coaching centers.
