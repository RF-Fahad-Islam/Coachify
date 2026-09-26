# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Keyboard first batch attendance sheet with single key entry (P, A, L, E), sub 300 ms save benchmark for 80 students, midnight session locking, and monthly attendance matrix (spec 0023).
- Offline first durable omnichannel outbox written in the same database transaction as domain actions, supporting cellular SMS, WhatsApp pairing via OpenWA, and SMTP email with quiet hours and timeout protection (spec 0024).
- Bilingual English and Bengali message templates with live GSM 7 and UCS 2 Unicode segment math, automatic payment receipt triggers, delayed absence alerts with cancellation buffer, and period deduplicated fee reminders (spec 0025).
- Examination management hub with spreadsheet style mark entry grid, NCTB five point GPA calculation, optional fourth subject bonus math, and fail rules for mandatory subjects (spec 0026).
- Results calculation and immutable snapshot publishing with five tier deterministic tie breaking, bilingual A4 student progress report cards, and fixed width landscape batch tabulation sheets (spec 0027).
- AI Foundation and Copilot Shell (Features 54–57, spec 0036) with zero-SQL access, 6 controlled read-only tools clamped to 50 rows, bounded multi-hop execution, client-side PII and 11-digit Bangladeshi mobile scrubbing, ephemeral in-core re-hydration, institutional credit ledger wallet with 1-turn/1-credit invariant, OpenRouter LLM gateway, and persistent audit logging.
- AI Copilot slide-over drawer accessible globally via Ctrl+J or TopBar icon with live credit pill, Shield Active protection badge, suggested questions, and tool execution chips.
- Institute AI & Privacy Settings card with 6 granular privacy toggles, credit wallet ledger table, recharge grant modal, BYOK gateway configuration, and tool execution audit trail.
- Workstation pairing and device management studio with a six digit pairing code, 10 minute expiration, 5 attempt brute force lockout, seat quota enforcement, and device lifecycle controls to rename, block, or permanently revoke client workstations (see spec 0045).
- Mobile app companion with optical QR pairing, multi interface candidate racing, out of band TLS 1.3 certificate pinning, 10 minute pairing window, and 15 minute brute force cooldown (see spec 0046).
- Mobile attendance camera scanner with live barcode detection, audio and haptic feedback, sub 150 ms scan latency, and a 5 second duplicate scan debounce (see spec 0046).
- Mobile cashier fee collection with student search, payment method selection, remote thermal receipt print dispatch to the host counter, and WhatsApp share links (see spec 0046).
- Quick staff PIN authentication with Argon2id password hashing and sliding session timeout for mobile companion security (see spec 0046).
- Real time Server Sent Events control channel pushing disconnect signals to severed devices within 500 ms of revocation or blocking (see specs 0045 and 0046).
- LAN workstation client mode and transparent RPC transport (Feature 71, spec 0047) enabling secondary desktop PCs (cashier desks, front reception) to operate seamlessly without a local database by forwarding domain commands over pinned TLS 1.3 to the primary host PC.
- Hardware-backed local thermal receipt and A4 report printing strictly isolated to the workstation's local physical printers and JSON configuration (`%LocalAppData%\Coachify\workstation_printer_settings.json`), keeping cashier counter hardware decoupled from the host.
- Multi-counter real-time cache synchronization through persistent SSE event streaming (`GET /events`), invalidating TanStack Query caches across all workstations within 500 ms of any host state change.
- Client-side UUIDv7 mutation idempotency headers (`X-Idempotency-Key`) preventing double fee billing or duplicate student insertions during transient LAN drops and retries.
- Autonomous mDNS host re-discovery (`_coachify._tcp.local.`) recovering workstation connections dynamically when the host PC's LAN IP changes via DHCP.
- Responsive workstation UI status pill in the top navigation bar with live RTT latency, host identity, diagnostic ping, and reconnect popover, accompanied by a sticky amber reconnection banner and full-screen lockout overlay upon remote revocation.
- Centralized Entitlement Seam consolidating operational license gate checks, read-only grace periods, device deactivation, clock tampering watermarks, and workstation seat quotas behind a cohesive policy engine.

### Fixed
- Recorded the acting user ID in the SHA 256 audit log whenever an administrator performs a retroactive edit on a locked attendance session.
- Allowed clearing saved gateway credentials by submitting an empty password, setting secret reference to null in the database.

### Security
- Gated attendance session saving and locking strictly behind the `attendance.mark` permission, preventing users with read only `attendance.view` access from altering or locking student attendance.
- Added certificate fingerprint pinning for mobile pairing sessions to prevent untrusted machine in the middle TLS proxy attacks on local networks (see spec 0046).
- Enforced hard commercial license seat quotas upon workstation and mobile pairing, rejecting unapproved devices with seat limit errors (see specs 0045 and 0046).
- Pinned SHA-256 host TLS certificate fingerprints on workstation clients, rejecting untrusted certificates and MITM proxies on the local network (see spec 0047).
- Layered device token bearer authentication (`Authorization: Bearer devtok_...`) with user session cookies, validating workstation hardware identity and active seat status in memory on every forwarded RPC call (see spec 0047).

