# 0041. Decision Record and Rationale: Vendor Admin and Activation Service

## Context

Coachify's desktop application (Spec 0037, Feature 62) enforces Ed25519 signed licenses completely offline, with a 14 day grace window after `paid_through` and a 14 day transitional window when a backup is restored onto a replacement computer. However, without a vendor administration system and an online activation and refresh service (Features 63 and 64), every trial conversion, monthly or annual bKash renewal, offline request code binding, and PC hardware replacement would require a software engineer to manually construct JSON payloads and run command line signing scripts.

In Bangladesh's coaching center market, most customers pay manually via bKash, Nagad, bank transfer, or cash and expect immediate confirmation on WhatsApp or automatic updates inside their app. Support and operations staff (non developers) must be able to record these payments, verify payment references, and issue licenses safely. At the same time, the security stakes are high: if the Ed25519 private signing key resides inside the general web application process, any SQL injection, server side request forgery, or dependency vulnerability in the web app could expose the master key and allow an attacker to mint unlimited lifetime licenses that offline desktops would trust forever.

Finally, the activation and refresh service (`POST /v1/activate`, `POST /v1/refresh`, `GET /v1/updates/{channel}`) must bridge online convenience with Coachify's strict offline first guarantee. Online installations should receive renewals automatically within 24 hours (or immediately when clicking Check Now) and support self service hardware transfers when a coaching center replaces a broken reception PC. Yet a week long internet outage or cloud server failure must never lock out a customer PC or block front desk operations.

## Options considered

### Option 1: Workspace Integrated Rust Commercial API (`crates/cloud_api`), Isolated Ed25519 Signer (`crates/license_signer`), `landing pages/` Styled Vendor Admin UI, and Synchronous `ureq` Desktop Client (Chosen)

Build two focused Rust crates inside the workspace (`crates/license_signer` holding only the Ed25519 private key and `signer_audit_log`, and `crates/cloud_api` holding the commercial ledger, TOTP admin authentication, and `/v1/*` endpoints), render the 6 tab Vendor Admin console and Desktop online activation controls using the `landing pages/` design system (`Sora`, `Inter`, `#faf7f2` cream, `#16110d` ink, `#f97316` ember), and connect `crates/core` via synchronous HTTP (`ureq` with a 5 second timeout and injectable transport trait) backed by Migration 0022 (`license_refresh_state`).

**Pros**:
* Shares the exact Rust `LicensePayload`, `LicenseEnvelope`, Ed25519 verification, and Base32 request code checksum logic with `crates/core/src/licensing.rs`, eliminating serialization drift between the signer and desktop verifier.
* Isolates the Ed25519 private key inside `crates/license_signer` behind a narrow `HMAC-SHA256` authenticated boundary (`sign_license` and `sign_revocation` only) with its own immutable `signer_audit_log`.
* Keeps `coachify_core` 100 percent synchronous (zero `tokio` runtime bloat) while allowing full end to end integration tests (`bKash payment -> isolated signer -> /v1/refresh -> desktop activation`) inside `cargo test --workspace`.
* Adopts the `landing pages/` components (`WindowFrame`, `CtaButton`, `SectionTag`, `Chip`) for immediate visual polish and consistency across commercial touchpoints.

**Cons**:
* Requires maintaining the `HMAC-SHA256` service boundary between `crates/cloud_api` and `crates/license_signer`.
* When the team later deploys the cloud stack from a separate repository, `crates/cloud_api` and `crates/license_signer` will be extracted from this monorepo.

### Option 2: Next.js TypeScript Monolith with In Process WebCrypto Ed25519 Signing

Build the Vendor Admin and `/v1/*` API routes inside a standalone Next.js TypeScript application that loads the Ed25519 private key from an environment variable in the Next.js server process and signs license blobs directly in API route handlers.

**Pros**:
* Single web process to deploy with no internal RPC call between the web backend and a separate signer service.

**Cons**:
* Violates the core requirement that the web app never holds the private signing key; any vulnerability in the web app or its npm dependency tree compromises the master Ed25519 private key.
* Reimplements canonical JSON serialization, `REQ-XXXX-...` HMAC Base32 checksum verification, and Ed25519 signing in TypeScript, risking subtle byte level mismatches with `crates/core/src/licensing.rs`.

### Option 3: Manual CLI Signer with Static File Uploads Only

Keep license signing strictly on a developer workstation via an offline CLI tool (`cargo run -p xtask sign-license`) and use the web admin only as a read/write payment ledger where admins queue signing requests for a developer to run in batches.

**Pros**:
* The private key never exists on any network connected server.

**Cons**:
* Fails the core operational requirement that a non developer can record a bKash payment and issue both an online and an offline license end to end without waiting for an engineer.
* Prevents instant online activation (`POST /v1/activate`) and self service PC hardware transfers (`confirm_transfer = true`).

## Rationale

Option 1 is the clear winner because it satisfies both operational speed for non technical staff and strict cryptographic key isolation without splitting the runtime stack prematurely. In `AGENTS.md` and `crates/core/AGENTS.md`, Coachify's core library is pure synchronous Rust with strict type safety and zero heavy ORMs. By implementing `crates/license_signer` and `crates/cloud_api` in Rust alongside the React UI styled with `landing pages/`, every cryptographic primitive (`SEED_K2026A`, `LicensePayload`, `LicenseEnvelope`, `COACHIFY-REQ|v1|...`) is guaranteed to be bit for bit compatible between the cloud issuer and the desktop verifier.

Separating `crates/license_signer` from `crates/cloud_api` enforces a hard security boundary: the web backend can only request signatures for structured claims using a time bounded `HMAC-SHA256` service credential, and `crates/license_signer` logs every signature in `signer_audit_log` before returning the signed bytes. Even if an attacker gained read access to `crates/cloud_api`'s database, they could not extract the Ed25519 private seed.

On the desktop side, using a synchronous HTTP client (`ureq` with a strict 5 second timeout behind a mockable `RefreshTransport` trait) preserves `crates/core`'s synchronous architecture and guarantees that network failures degrade silently to `offline_deferred`. A coaching center can lose internet connectivity for weeks or months and experience zero slowdowns or false lockouts at the cashier counter.
