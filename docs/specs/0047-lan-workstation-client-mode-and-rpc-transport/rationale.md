# 0047. LAN Workstation Client Mode and RPC Transport: Rationale

> Decision record and architectural rationale for Spec 0047 (LAN Workstation Client Mode and RPC Transport, Feature 71)

## Context

In growing coaching centers across Bangladesh, customer-facing counter operations during morning admission rushes and monthly fee collection cycles cannot be handled on a single computer. A typical coaching center running `Coachify Local Pro` operates two to five workstations concurrently:
- **Counter 1 (Front Desk / Reception)**: Student counseling, walk-in inquiries, batch enrollments, and student card printing.
- **Counter 2 & 3 (Cashier Desks)**: High-speed fee collection, cash counting, invoice generation, and thermal receipt printing.
- **Counter 4 (Floor Coordinator)**: Class attendance marking, exam marks entry, and batch schedule adjustments.
- **Primary Host PC (Owner / Administrator Office)**: Central database custody, automated SMS/WhatsApp broadcasts, financial audits, daily cash close reconciliation, and system backups.

Coaching centers operate in constrained local infrastructure environments:
1. **Zero Internet Dependency**: The entire multi-counter coaching center must function flawlessly during prolonged broadband and optical fiber outages. No external cloud relays or centralized SaaS servers can be required for counter operations.
2. **Strict Financial ACID Guarantees**: Ledger transactions, sequential receipt numbering (`REC-YYYY-XXXX`), and student fee balances cannot tolerate eventual consistency or split-brain divergences. A payment taken at Counter 2 must instantly update Counter 1's dues balance to prevent double payments or disputes.
3. **Local Peripheral Proximity**: Cashiers and receptionists sit directly beside their own physical desktop hardware—specifically USB thermal receipt printers (POS-80), laser A4 printers, barcode scanners, and cash drawers.
4. **Unmanaged Consumer Networking**: The local network is typically powered by budget consumer Wi-Fi routers (e.g. TP-Link, D-Link, Tenda) with dynamic DHCP leases, concrete wall interference, and occasional packet drops. There is no resident network engineer to assign static IP reservations or configure corporate subnets.

To meet these demanding operational realities, Feature 71 establishes an architecture that provides absolute database integrity, seamless multi-counter concurrency, local peripheral isolation, and resilient network recovery.

## Options Considered

### 1. Data Architecture: Thin Client over TLS RPC vs Thick Client with Multi-Master SQLite Replication

- **Option 1A: Thin Client over Forwarding TLS RPC (Selected)**
  The primary host PC is the sole authoritative writer and database holder. Secondary workstations run a thin client runtime where all domain queries and mutations are forwarded over pinned TLS 1.3 to the host's central dispatcher (`POST /rpc`). Workstations do not maintain a local business database.
  - *Pros*:
    - **Zero Data Divergence**: Single SQLite database guarantees strict ACID transactions, immediate serializability, and monotonic receipt number increments without merge conflicts.
    - **Zero Synchronization Lag**: Changes committed on the host reflect instantaneously across all workstations via SSE push notifications.
    - **Minimal Hardware Overhead**: Secondary workstations can be lightweight budget PCs or laptops with minimal RAM and disk requirements.
    - **Single Backup & Restore Authority**: The owner backs up only the primary host PC; there are no stray database files scattered across employee laptops.
  - *Cons*: Workstations require active local network connectivity to the host PC to perform business mutations (mitigated by transparent automatic reconnection and resilient offline state indicators).
- **Option 1B: Thick Client with Multi-Master SQLite Replication (e.g. CRDTs / symmetric sync)**
  Each workstation maintains an independent local SQLite database. Transactions are written locally and synchronized asynchronously between peers using conflict-free replicated data types (CRDTs) or log-based peer-to-peer sync.
  - *Pros*: Workstations could theoretically record transactions while completely disconnected from the host.
  - *Cons*:
    - **Catastrophic Financial Conflicts**: Financial ledgers and sequential receipt numbers cannot be resolved with CRDTs. If Counter 1 collects 2,000 BDT and Counter 2 collects 2,000 BDT for the same student while disconnected, the student's concession balance, due statement, and sequential receipt numbers collide irreconcilably.
    - **Severe Architectural Bloat**: Requires hundreds of lines of complex distributed consensus, vector clocks, conflict resolution dialogs, and schema synchronization.
    - **Audit Trail Tampering**: Disconnected laptops could tamper with local SQLite timestamps or audit log hash chains before synchronizing.

### 2. Forwarding RPC Transport: Pinned TLS 1.3 HTTPS (`POST /rpc`) vs Full-Duplex WebSockets vs gRPC

- **Option 2A: Pinned TLS 1.3 HTTPS (`POST /rpc`) with SSE Push Channel (Selected)**
  Commands are dispatched via standard HTTP/1.1 or HTTP/2 `POST /rpc` over pinned TLS 1.3. A separate persistent Server-Sent Events stream (`GET /events`) handles real-time server-to-client notifications.
  - *Pros*:
    - **Parity with Core Architecture**: Directly maps to Coachify's single typed RPC paradigm (`command`, `payload`, `session_token`, `idempotency_key`).
    - **Independent Request Isolation**: Each RPC transaction operates with its own HTTP timeout, connection pooling, and standard HTTP error semantics (401, 403, 422, 503).
    - **Lightweight Asymmetric Design**: SSE (`GET /events`) is unidirectional, extremely lightweight (< 50 KB memory per client), and native to HTTP/1.1, eliminating the framing and ping-pong state machinery of WebSockets.
    - **Seamless Reconnection**: If the SSE stream drops, standard HTTP RPC requests continue executing uninterrupted while the background SSE worker reconnects.
  - *Cons*: Requires two TCP connections per workstation (one for RPC connection pool, one for SSE keep-alive), which is negligible on modern OS kernels.
- **Option 2B: Full-Duplex WebSockets for Both RPC and Events**
  Establish a bidirectional WebSocket connection per workstation and multiplex all RPC requests, responses, and events over a single socket.
  - *Pros*: Single persistent TCP connection per workstation.
  - *Cons*:
    - **Head-of-Line Blocking**: A slow report generation query can block or delay other queued RPC messages across the multiplexed socket.
    - **Complex Framing & State Tracking**: Requires custom request-response matching (`correlation_id`), custom heartbeat timeouts, and fragile socket state machines.
    - **Harder Network Diagnostics**: Harder to inspect, trace, and debug than standard HTTP endpoints with distinct status codes and headers.
- **Option 2C: gRPC / Protobuf over HTTP/2**
  Use Google Protocol Buffers and gRPC for client-host communication.
  - *Pros*: Compact binary serialization and high throughput.
  - *Cons*:
    - Breaks the unified TypeScript contract generated by `specta` / `xtask export-types`.
    - Introduces heavy `protoc` build-time dependencies, conflicting with Coachify's lightweight, pure-Rust core philosophy.

### 3. Peripheral & Printing Custody: Strictly Local Rasterization vs Host Print Routing

- **Option 3A: Strictly Local Print Rasterization (Selected)**
  When a cashier clicks "Print Receipt", the workstation fetches the printable receipt data from the host database (`receipts.get_printable`), but renders the HTML template and submits the print job locally to the workstation's own Windows print spooler and attached printer.
  - *Pros*:
    - **Correct Physical Location**: The thermal receipt prints immediately on the printer sitting on Counter 2's desk, right next to the customer and cashier, rather than in the owner's private office.
    - **Independent Per-Counter Hardware**: Counter 1 can use an A4 laser printer for student admission forms while Counter 2 and 3 use 80mm thermal receipt printers.
    - **Local Cash Drawer Activation**: The workstation can trigger the cash drawer kick pin via the local printer driver without network latency or host intervention.
  - *Cons*: Requires local printer settings storage on each workstation (implemented cleanly via local machine storage).
- **Option 3B: Host-Side Print Dispatch (All Printing via Host PC)**
  The workstation tells the host to print the receipt, and the host sends the job to its own default printer.
  - *Pros*: Centralized printer driver installation.
  - *Cons*: Completely unworkable in practice: receptionists and cashiers would have to walk into the owner's office to pick up every receipt, disrupting operations and creating counter chaos.

### 4. Network Resilience & Re-Discovery: Dynamic mDNS Re-Discovery vs Static IP Requirement

- **Option 4A: Pinned Identity with Dynamic mDNS Re-Discovery (Selected)**
  The client stores the host's unique `host_id` and SHA-256 certificate fingerprint in DPAPI. If the cached IP address becomes unreachable (e.g. router rebooted and reassigned the host from `192.168.1.50` to `192.168.1.75`), the client automatically triggers an mDNS browse for `_coachify._tcp.local` matching the pinned `host_id`.
  - *Pros*:
    - **Zero Maintenance for Customers**: Survives DHCP lease renewals, router reboots, and IP address shifts without requiring staff to reconfigure IP addresses or re-pair devices.
    - **Robust Security**: The client only connects to the new IP if the server certificate matches the exact pinned SHA-256 fingerprint, completely preventing rogue host interception.
  - *Cons*: Requires background mDNS resolution capabilities on the workstation (already provided by `mdns-sd` in Spec 0044).
- **Option 4B: Mandatory Static IP Configuration**
  Require the coaching center owner to assign a static IP address to the host PC in Windows and router DHCP reservation tables.
  - *Pros*: Simple client networking code.
  - *Cons*: Terrible onboarding experience: the vast majority of coaching center owners lack the technical expertise to log into router admin panels and configure DHCP reservations. Any IP change would completely break workstation connections and flood customer support.

### 5. In-Flight Mutation Protection: Client-Side UUIDv7 Idempotency vs Blind Retry

- **Option 5A: Client-Side UUIDv7 Idempotency Keys (Selected)**
  The client generates a unique UUIDv7 idempotency key for every mutating operation before sending it over the network. If the connection drops while the request was in flight, the client retries with the identical key.
  - *Pros*:
    - **Zero Duplicate Financial Charges**: If the host processed a 1,500 BDT payment but the Wi-Fi dropped before the response reached the client, retrying returns the existing receipt without debiting or recording a second payment.
    - **Deterministic Recovery**: Reconnection is safe and transparent; cashiers never have to wonder "did that payment go through or should I click pay again?".
  - *Cons*: Host must persist idempotency records for 24 hours (already implemented in Spec 0004).
- **Option 5B: Blind Network Retry without Idempotency**
  Simply retry dropped HTTP requests on connection recovery.
  - *Pros*: Trivial implementation.
  - *Cons*: Unacceptable financial risk: creates duplicate student admissions, duplicate payments, and corrupted ledger totals whenever office Wi-Fi blips.

## Rationale Summary

We selected **Option 1A (Thin Client over TLS RPC)**, **Option 2A (Pinned TLS 1.3 HTTPS + SSE)**, **Option 3A (Strictly Local Print Rasterization)**, **Option 4A (Dynamic mDNS Re-Discovery)**, and **Option 5A (Client-Side UUIDv7 Idempotency)** because they fulfill the maximalist product bar of Coachify:

1. **Absolute Ledger Integrity**: Single authoritative SQLite writer on the host PC guarantees zero split-brain ledger conflicts and sequential receipt numbering.
2. **True Counter Independence**: Secondary desks operate with full speed, physical receipt printing at the cashier's desk, and local cash drawer control.
3. **Frictionless Resilience**: Automatic reconnection, mDNS dynamic IP recovery, and in-flight idempotency guarantee that office Wi-Fi fluctuations never corrupt institution data or halt morning admissions.
4. **Architectural Simplicity**: Reuses the single typed RPC bridge and TanStack Query cache invalidation pipeline without duplicate code paths between single-PC and multi-PC modes.

## References

- **Spec 0004**: `docs/specs/0004-command-dispatcher-local-accounts/index.md` (Command dispatcher, idempotency engine, and session management)
- **Spec 0013**: `docs/specs/0013-print-engine-and-receipts/index.md` (Thermal receipt printing and local layout rendering)
- **Spec 0035**: `docs/specs/0035-professional-printing-system/index.md` (Document templates, A4 dual receipt layouts, and print options)
- **Spec 0044**: `docs/specs/0044-lan-host-service-and-discovery/index.md` (Host HTTPS TLS 1.3 listener, self-signed certificates, and mDNS responder)
- **Spec 0045**: `docs/specs/0045-workstation-pairing-and-device-management/index.md` (Workstation pairing handshake, persistent device tokens, and SSE control streams)
- **RFC 8446**: The Transport Layer Security Protocol Version 1.3
- **RFC 6762 / 6763**: Multicast DNS and DNS-Based Service Discovery
- **W3C Server-Sent Events**: Real-time server-to-client unidirectional event transport
