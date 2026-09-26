# 0042. Website, Customer Portal, Code Signing, and Release Channels

**Date**: 2026-09-25
**Status**: In Progress

## Summary

Coachify completes its public commercial web presence and release engineering pipeline by adding a customer portal (`/portal`), a telemetry transparency page (`/privacy`), a live channel download center (`/downloads`) in `Landing pages/`, and an automated Windows code signing, Tauri updater signing, and zero rebuild channel promotion pipeline in `crates/cloud_api`, `crates/license_signer`, and `.github/workflows/release.yml`. Institution owners log in with their phone or email, password, and a 6 digit verification code (OTP, a single use login code) to download their signed `.coachifylicense` file, verified NSIS installer, and paid `INV-YYYY-NNNNN` invoices, while tagged builds sign once in continuous integration (CI), publish to `internal` after automated checks and human approval pass, and promote to `beta` or `stable` by copying signed metadata without recompiling.

## Structure

This umbrella specification coordinates two child specifications covering Slice 12 (Features 65 and 66):

1. [0042-website-downloads-and-customer-portal.md](0042-website-downloads-and-customer-portal.md): Feature 65 story driven marketing landing page (`Landing pages/`), live download center (`/downloads`) with channel selector and SHA256 checksum verification, searchable help center (`/help`), exact 5 field desktop telemetry disclosure (`/privacy`), and the authenticated customer portal (`/portal`) for `.coachifylicense` downloads, installer downloads, paid `INV-YYYY-NNNNN` invoices, and linked PC installations.
2. [0042-code-signing-and-release-channels.md](0042-code-signing-and-release-channels.md): Feature 66 Windows Authenticode signing from CI via cloud key management (Azure Trusted Signing or KMS), Tauri updater Ed25519 key custody inside `crates/license_signer`, `internal` / `beta` / `stable` channel head pointers with zero rebuild promotion, blocked versions enforcement, and executable dual key recovery drills recorded in an append only table (`key_recovery_drills`).

**Cross child contract**:
* **Single release source of truth**: Every published installer in `releases` and every active channel pointer in `release_channel_heads` (`internal`, `beta`, `stable`) inside `crates/cloud_api` powers both the desktop auto updater (`GET /v1/updates/{channel}`) from Feature 66 and the public Download Center (`/downloads`) plus Customer Portal installer card (`/portal`) from Feature 65.
* **Isolated Ed25519 key custody**: `crates/license_signer` holds both the commercial license Ed25519 signing seeds (`k2026a`, `k2026b`) and the Tauri updater Ed25519 release signing seeds (`u2026a`, `u2026b`); `crates/cloud_api` requests `.coachifylicense` downloads (Feature 65) and updater artifact signatures or key recovery verifications (Feature 66) exclusively over the `HMAC-SHA256` service boundary.
* **Exact privacy and telemetry contract**: The `/privacy` page in Feature 65 documents the exact 5 telemetry fields (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`) sent to `POST /v1/activate` and `POST /v1/refresh` and the 2 query fields (`channel`, `current_version`) sent to `GET /v1/updates/{channel}` in Feature 66, guaranteeing that student names, guardian phone numbers, and fee records never leave the customer PC.

## Requirements

**User stories**:
* As a coaching center owner visiting the Coachify website, I want to read the product story in English or Bangla, inspect the exact 5 fields the desktop app sends over the internet on `/privacy`, and download a verified Windows demo installer with its SHA256 checksum on `/downloads` so that I can evaluate Coachify with full confidence in my data safety.
* As a paying institution owner, I want to log in to `/portal` using my registered phone number or email, password, and a 6 digit OTP code so that I can view my subscription status, download my latest signed `.coachifylicense` file for offline import, download the verified Windows installer, download my paid `INV-YYYY-NNNNN` vendor invoices, and inspect my bound PC installations.
* As a release engineer, I want a tagged git release (`v*`) to build and sign the Windows NSIS installer with Authenticode and the Tauri updater Ed25519 key only after all Rust and UI verification suites pass and a human reviewer approves the release environment, publishing first to the `internal` channel.
* As an operations administrator (`super_admin` or `ops_admin`), I want to promote a verified build from `internal` to `beta` or `stable` in `crates/cloud_api` by copying its signed artifact reference and Ed25519 updater signature without rebuilding the binary, and block a broken release with step up TOTP so no desktop upgrades into it.
* As a founder preparing for general availability (`1.0`), I want to execute and record a deterministic recovery drill of both the Windows code signing certificate and the Ed25519 updater and license signing keys from encrypted offline backup into an append only audit table (`key_recovery_drills`) so that key loss can never brick existing installations.

**Acceptance criteria**:
* **AC-1**: Customer portal authentication and OTP session security: `POST /v1/portal/login` and `POST /v1/portal/verify_otp` authenticate a customer by matching `phone` or `email` in `customers`, verifying the Argon2id `password_hash` in `customer_portal_credentials`, verifying the 6 digit OTP within 10 minutes (locking for 15 minutes after 5 failed attempts), and issuing a SHA256 hashed bearer token in `customer_portal_sessions`.
* **AC-2**: Customer portal license, installer, invoice, and installation access: Authenticated calls to `GET /v1/portal/overview`, `GET /v1/portal/license_file/{license_id}`, and `GET /v1/portal/invoice/{invoice_id}` return only records owned by the session's `customer_id`, allowing the customer to download their signed `.coachifylicense` file, download the current `stable` or `beta` Windows installer with SHA256 checksum, inspect paid `INV-YYYY-NNNNN` invoices in BDT, and view their bound PC installations (`label`, `installation_id`, `app_version`, `last_seen_at`, `status`).
* **AC-3**: Exact privacy and telemetry transparency page (`/privacy`): The `/privacy` route in `Landing pages/` displays the exact JSON payload schema and field by field explanation of the 5 desktop licensing fields (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`) and update check fields, explicitly certifying that student names, guardian phone numbers, attendance logs, and financial ledger entries remain 100% local in SQLite.
* **AC-4**: Live download center (`/downloads`), help center (`/help`), and story driven conversion flow: The `/downloads` page displays live channel release metadata (`stable` and `beta` version, publication date, file size, Authenticode signer thumbprint, SHA256 checksum with copy button, and release notes) alongside the 30 day free demo installer, while `/help` provides searchable setup, daily operations, LAN, and backup guides in English and Bangla.
* **AC-5**: CI gated Windows Authenticode and Tauri Ed25519 signing to `internal`: `.github/workflows/release.yml` runs `cargo fmt`, `cargo clippy`, `cargo test --workspace`, `cargo run -p xtask check-types`, `cargo run -p xtask check-migrations`, `pnpm lint`, `pnpm format:check`, and `pnpm test`, waits for a required GitHub Environment human approval gate (`release-internal`), signs the NSIS binary with Windows Authenticode plus the isolated `crates/license_signer` updater Ed25519 key (`u2026a`), and registers the build in `releases` and `release_channel_heads` for the `internal` channel only.
* **AC-6**: Zero rebuild channel promotion (`internal` to `beta` to `stable`): `POST /v1/releases/promote` (restricted to `super_admin` and `ops_admin` with step up TOTP when promoting to `stable`) updates `release_channel_heads` for the target channel (`beta` or `stable`) to reference the existing immutable `release_id`, `sha256`, `artifact_url`, `authenticode_thumbprint`, and `updater_signature` from the source channel without recompiling or resigning the binary, and appends a hash chained record to `admin_audit_log`.
* **AC-7**: Held and blocked version enforcement across promotion and updates: Attempting to promote a release whose `status` is `held` or `blocked` fails with structured error `RELEASE_NOT_PROMOTABLE`; marking a version `blocked` (with step up TOTP) immediately removes it from active channel candidates and includes its version string in `blocked_versions` on `GET /v1/updates/{channel}` so desktop clients refuse to install it.
* **AC-8**: Dual key custody and recorded recovery drill (`key_recovery_drills`): `POST /v1/releases/recovery_drill` (invoking `crates/license_signer::verify_key_recovery`) decrypts the offline backup material for both `windows_authenticode` and `tauri_updater_ed25519` (plus `license_ed25519`), signs a canonical test artifact, verifies the resulting signature against the embedded public keys, and writes an immutable row to `key_recovery_drills` protected by `BEFORE UPDATE` and `BEFORE DELETE` abort triggers.
* **AC-9**: Unified `design.md` and `Landing pages/` UI execution: All new routes in `Landing pages/` (`/portal`, `/privacy`, `/downloads` channel bar) and the Release Promotion and Key Recovery Drill studio in `ui/src/components/vendor-admin/vendor-admin-view.tsx` use the `design.md` and `Landing pages/` visual system (`Sora` display headings, `Inter` body with `.tnum` tabular numbers, `Noto Sans Bengali` for Bangla, warm cream `#faf7f2`, ink `#16110d`, ember `#f97316`, `WindowFrame`, `CtaButton`, `SectionTag`, and `Chip` primitives).

## Decision

**Chosen option**: Option 1: Integrated `crates/cloud_api` Portal and Channel Promotion Engine, Isolated `crates/license_signer` Updater and License Key Custody, `Landing pages/` Customer Portal and Privacy Routes (`/portal`, `/privacy`, `/downloads`), and Gated GitHub Actions Authenticode + Ed25519 Pipeline

We extend `crates/cloud_api` with four tables (`customer_portal_credentials`, `customer_portal_sessions`, `release_channel_heads`, and `key_recovery_drills`) and versioned `/v1/portal/*` and `/v1/releases/*` endpoints, extend `crates/license_signer` with Tauri updater release signing (`u2026a` / `u2026b`) and automated backup recovery verification, add the `/portal` and `/privacy` routes plus live channel metadata on `/downloads` inside `Landing pages/`, and wire `.github/workflows/release.yml` with full verification gates, human environment approval, Authenticode signing, and zero rebuild channel promotion.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:

1. **Customer Portal Credentials (`customer_portal_credentials` in `crates/cloud_api`)**:
   * `customer_id TEXT PRIMARY KEY REFERENCES customers(id)`
   * `password_hash TEXT NOT NULL` (Argon2id hash)
   * `otp_code_hash TEXT` (SHA256 hash of 6 digit OTP code)
   * `otp_expires_at TEXT` (ISO 8601 UTC timestamp, 10 minute window)
   * `failed_attempts INTEGER NOT NULL DEFAULT 0`
   * `locked_until TEXT` (ISO 8601 UTC timestamp, set to `now + 15 minutes` when `failed_attempts >= 5`)
   * `updated_at TEXT NOT NULL`

2. **Customer Portal Sessions (`customer_portal_sessions` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `customer_id TEXT NOT NULL REFERENCES customers(id)`
   * `session_token_hash TEXT UNIQUE NOT NULL` (SHA256 hash of bearer token)
   * `ip_address TEXT`
   * `user_agent TEXT`
   * `expires_at TEXT NOT NULL` (24 hour session expiry)
   * `revoked_at TEXT`
   * `created_at TEXT NOT NULL`

3. **Immutable Release Artifacts (`releases` extended in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `version TEXT UNIQUE NOT NULL` (semantic version, e.g. `1.0.0`)
   * `channel TEXT NOT NULL CHECK (channel IN ('internal', 'beta', 'stable'))` (initial publish channel, always `internal` from CI)
   * `notes TEXT NOT NULL`
   * `artifact_url TEXT NOT NULL`
   * `sha256 TEXT NOT NULL` (64 character lowercase hex SHA256 of the signed NSIS `.exe`)
   * `size_bytes INTEGER NOT NULL DEFAULT 0`
   * `authenticode_thumbprint TEXT NOT NULL DEFAULT ''` (Windows Authenticode certificate SHA1/SHA256 thumbprint)
   * `updater_signature TEXT NOT NULL` (hex Ed25519 signature produced by `crates/license_signer`)
   * `updater_key_id TEXT NOT NULL DEFAULT 'u2026a'`
   * `min_upgrade_from TEXT`
   * `status TEXT NOT NULL CHECK (status IN ('live', 'held', 'blocked'))`
   * `published_by TEXT NOT NULL REFERENCES admin_users(id)`
   * `published_at TEXT NOT NULL`

4. **Zero Rebuild Channel Pointers (`release_channel_heads` in `crates/cloud_api`)**:
   * `channel TEXT PRIMARY KEY CHECK (channel IN ('internal', 'beta', 'stable'))`
   * `release_id TEXT NOT NULL REFERENCES releases(id)`
   * `version TEXT NOT NULL`
   * `promoted_from_channel TEXT CHECK (promoted_from_channel IN ('internal', 'beta') OR promoted_from_channel IS NULL)`
   * `promoted_by TEXT NOT NULL REFERENCES admin_users(id)`
   * `promoted_at TEXT NOT NULL`

5. **Append Only Key Recovery Drills (`key_recovery_drills` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `key_type TEXT NOT NULL CHECK (key_type IN ('windows_authenticode', 'tauri_updater_ed25519', 'license_ed25519'))`
   * `key_id TEXT NOT NULL`
   * `backup_source TEXT NOT NULL`
   * `backup_sha256 TEXT NOT NULL`
   * `test_artifact_sha256 TEXT NOT NULL`
   * `signature_verified INTEGER NOT NULL CHECK (signature_verified = 1)`
   * `notes TEXT NOT NULL`
   * `performed_by TEXT NOT NULL REFERENCES admin_users(id)`
   * `performed_at TEXT NOT NULL`
   * Protected by `BEFORE UPDATE` and `BEFORE DELETE` abort triggers (`prevent_update_key_recovery_drills`, `prevent_delete_key_recovery_drills`).

**State transitions**:
* **Customer Portal Session Lifecycle**:
  * `unauthenticated` -> (`POST /v1/portal/login` with valid `phone` or `email` + `password`) -> `otp_pending` (6 digit OTP generated, `otp_expires_at = now + 10m`)
  * `otp_pending` -> (`POST /v1/portal/verify_otp` with valid 6 digit OTP before `otp_expires_at`) -> `authenticated` (`session_token` issued, `failed_attempts = 0`)
  * `otp_pending` -> (5 consecutive wrong OTP attempts) -> `locked` (`locked_until = now + 15m`)
  * `authenticated` -> (`POST /v1/portal/logout` or `now > expires_at`) -> `revoked`
* **Release Artifact & Channel Promotion Lifecycle**:
  * `ci_tagged` -> (all test suites pass + human approval gate + Authenticode & Ed25519 signing) -> `published_internal` (`releases.status = 'live'`, `release_channel_heads['internal'] = release_id`)
  * `published_internal` -> (`POST /v1/releases/promote` to `beta` by `ops_admin` or `super_admin`) -> `promoted_beta` (`release_channel_heads['beta'] = release_id`, zero rebuild)
  * `promoted_beta` or `published_internal` -> (`POST /v1/releases/promote` to `stable` with step up TOTP) -> `promoted_stable` (`release_channel_heads['stable'] = release_id`, zero rebuild)
  * `live` -> (`set_release_status` to `held`) -> `held` (cannot be promoted; omitted from `GET /v1/updates/{channel}`)
  * `live` or `held` -> (`set_release_status` to `blocked` with step up TOTP) -> `blocked` (never promoted; listed in `blocked_versions` on `GET /v1/updates/{channel}`)

**API surface**:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `/v1/portal/login` | `POST` | `identifier: String` (phone or email), `password: String` | `customer_id`, `otp_required: bool`, `masked_destination`, `otp_expires_at` | Public (rate limited) | `INVALID_CREDENTIALS`, `PORTAL_ACCOUNT_LOCKED` |
| `/v1/portal/verify_otp` | `POST` | `customer_id: String`, `otp_code: String` (6 digits) | `session_token`, `expires_at`, `customer: CustomerSummary` | Public (rate limited) | `INVALID_OTP`, `OTP_EXPIRED`, `PORTAL_ACCOUNT_LOCKED` |
| `/v1/portal/overview` | `GET` | `session_token: String` (Bearer) | `customer`, `subscription`, `license`, `invoices: Vec<PortalInvoiceItem>`, `installations: Vec<PortalInstallationItem>`, `latest_release: ReleaseManifest` | Customer Bearer token | `UNAUTHORIZED_PORTAL_SESSION`, `CUSTOMER_SUSPENDED` |
| `/v1/portal/license_file/{license_id}` | `GET` | `session_token: String`, `license_id: String` | `filename: String` (`<license_id>.coachifylicense`), `signed_blob: String`, `serial: i64` | Customer Bearer token (owner only) | `UNAUTHORIZED_PORTAL_SESSION`, `FORBIDDEN_LICENSE_OWNER`, `LICENSE_NOT_FOUND` |
| `/v1/portal/invoice/{invoice_id}` | `GET` | `session_token: String`, `invoice_id: String` | `invoice_number`, `issued_at`, `org_name`, `plan_name`, `period_months`, `method`, `reference`, `subtotal_poisha`, `vat_poisha`, `total_poisha` | Customer Bearer token (owner only) | `UNAUTHORIZED_PORTAL_SESSION`, `FORBIDDEN_INVOICE_OWNER` |
| `/v1/releases/publish` | `POST` | `admin_user_id`, `version`, `notes`, `artifact_url`, `sha256`, `size_bytes`, `authenticode_thumbprint`, `min_upgrade_from` | `release: ReleaseRecord`, `channel_head: ReleaseChannelHead` (`internal`) | Admin + CI HMAC / `ops_admin` | `DUPLICATE_VERSION`, `INVALID_AUTHENTICODE_THUMBPRINT`, `SIGNER_FAILURE` |
| `/v1/releases/promote` | `POST` | `admin_user_id`, `version`, `from_channel`, `to_channel`, `step_up_totp: Option<String>` | `channel_head: ReleaseChannelHead`, `release: ReleaseRecord` | `ops_admin` / `super_admin` (+ TOTP for `stable`) | `RELEASE_NOT_PROMOTABLE`, `CHANNEL_HEAD_MISMATCH`, `STEP_UP_TOTP_REQUIRED` |
| `/v1/releases/recovery_drill` | `POST` | `admin_user_id`, `key_type`, `key_id`, `backup_source`, `encrypted_backup_blob`, `passphrase`, `notes`, `step_up_totp` | `drill: KeyRecoveryDrillRecord` | `super_admin` + step up TOTP | `RECOVERY_SIGNATURE_MISMATCH`, `STEP_UP_TOTP_REQUIRED` |
| `/v1/updates/{channel}` | `GET` | `channel: String` (`internal`, `beta`, `stable`) | `channel`, `latest: Option<ReleaseManifest>`, `blocked_versions: Vec<String>` | Public | `INVALID_CHANNEL` |

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| `POST /v1/portal/login` | `customer_id` and `masked_destination` | Resolved from `customers.phone` or `customers.email` matched against `customer_portal_credentials.password_hash` |
| `POST /v1/portal/verify_otp` | `session_token`, `expires_at`, and `customer` summary | Verified against `customer_portal_credentials.otp_code_hash` (`now <= otp_expires_at`), persisted as SHA256 `session_token_hash` in `customer_portal_sessions` |
| `GET /v1/portal/overview` | `license.status`, `paid_through`, `current_serial`, and `last_issued_blob` | Joined from `subscriptions` and `licenses` where `customers.id = customer_portal_sessions.customer_id` |
| `GET /v1/portal/overview` | `invoices` (`invoice_number`, `total_poisha`, `method`, `issued_at`) | Queried from `vendor_invoices` joined with `payments` where `vendor_invoices.customer_id = customer_portal_sessions.customer_id` ordered by `issued_at DESC` |
| `GET /v1/portal/overview` | `installations` (`label`, `installation_id`, `app_version`, `active_students`, `last_seen_at`, `status`) | Queried from `installations` where `installations.customer_id = customer_portal_sessions.customer_id` ordered by `last_seen_at DESC` |
| `GET /v1/portal/license_file/{license_id}` | Downloadable `.coachifylicense` content (`signed_blob`) | Loaded from `licenses.last_issued_blob` (or latest `license_issues.signed_blob`) after verifying `licenses.customer_id == session.customer_id` |
| `/privacy` Telemetry Inspector | Exact 5 activation/refresh telemetry fields and 2 update check fields | Defined by `DesktopRefreshRequest` / `DesktopActivateRequest` in `crates/cloud_api/src/lib.rs` (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`) |
| `POST /v1/releases/publish` | `updater_signature` and `updater_key_id` | Returned by `crates/license_signer::SignerService::sign_updater_release` over the canonical release payload (`version`, `sha256`, `artifact_url`, `authenticode_thumbprint`) |
| `POST /v1/releases/promote` | Target channel (`beta` or `stable`) `release_id`, `sha256`, `updater_signature`, and `authenticode_thumbprint` | Copied directly from the immutable `releases` row pointed to by `release_channel_heads[from_channel]` without rebuilding |
| `GET /v1/updates/{channel}` and `/downloads` | `latest` release manifest and `blocked_versions` | `latest` resolved via `release_channel_heads[channel]` joined with `releases` where `status = 'live'`; `blocked_versions` queried from `SELECT version FROM releases WHERE status = 'blocked'` |
| `POST /v1/releases/recovery_drill` | `backup_sha256`, `test_artifact_sha256`, `signature_verified` | Computed by `crates/license_signer` during live backup decryption, test payload signing, and Ed25519 / Authenticode public key verification |

**Key invariants**:
* **Strict tenant scoping in Customer Portal**: Every `/v1/portal/*` query filters by the `customer_id` bound to the verified `session_token_hash` in `customer_portal_sessions`. A valid session for Customer A can never download Customer B's `.coachifylicense` or `INV-YYYY-NNNNN` invoice (`FORBIDDEN_LICENSE_OWNER` / `FORBIDDEN_INVOICE_OWNER`).
* **Build once, promote metadata only**: A version in `releases` is built and signed exactly once when published to `internal`. Promoting to `beta` or `stable` updates `release_channel_heads` to point to that `release_id`; `sha256`, `authenticode_thumbprint`, and `updater_signature` remain byte for byte identical across all channels.
* **Non promotable held/blocked versions**: Any release with `status IN ('held', 'blocked')` cannot be set as the head of `internal`, `beta`, or `stable`, and if an already promoted release is changed to `held` or `blocked`, `GET /v1/updates/{channel}` immediately withholds it and includes `blocked` versions in `blocked_versions`.
* **Dual key recovery proof before GA**: `key_recovery_drills` is append only via SQLite triggers, and the release readiness check requires at least one verified recovery drill (`signature_verified = 1`) for both `windows_authenticode` and `tauri_updater_ed25519` (plus `license_ed25519`).

**Security model**:
* **Customer Portal (`customer_portal_sessions`)**: Requires Argon2id password verification plus a 6 digit OTP valid for 10 minutes. Rate limited to 5 OTP attempts before a 15 minute lockout (`locked_until`). Bearer tokens are stored as SHA256 hashes (`session_token_hash`) and expire after 24 hours.
* **Vendor Admin & CI Release Controls**: Publishing to `internal` requires an authenticated admin or CI service HMAC plus non empty `authenticode_thumbprint` and verified `updater_signature`. Promoting to `stable`, blocking a version, or executing a key recovery drill requires `super_admin` or `ops_admin` role plus a live RFC 6238 step up TOTP code.
* **Key Custody (`crates/license_signer`)**: Private Ed25519 keys for both licensing (`k2026a`, `k2026b`) and Tauri updater releases (`u2026a`, `u2026b`) live only inside `crates/license_signer` and encrypted offline backup vaults, never inside `crates/cloud_api` or frontend bundles.

**Configuration required**:
* `SIGNER_SHARED_SECRET`: HMAC SHA256 shared secret authenticating calls from `crates/cloud_api` to `crates/license_signer`.
* `TAURI_UPDATER_PRIVATE_SEED_U2026A`: Isolated Ed25519 seed inside `crates/license_signer` used to sign release update manifests.
* `AZURE_TRUSTED_SIGNING_ENDPOINT` / `AZURE_TRUSTED_SIGNING_ACCOUNT` / `AZURE_TRUSTED_SIGNING_PROFILE`: Cloud backed Windows Authenticode certificate credentials in `.github/workflows/release.yml`.
* `VITE_CLOUD_API_URL`: Optional base URL in `Landing pages/` for connecting `/portal` and `/downloads` to a live commercial API server (with deterministic embedded client runtime fallback for local dev and tests).

**Critical test scenarios**:
* **Happy path (Customer Portal login, license download, and invoice view)**: Customer logs in with phone + password + 6 digit OTP, loads `/portal`, downloads `<license_id>.coachifylicense` matching `last_issued_blob`, views `INV-2026-00001`, and sees active PC installations, verifies **AC-1**, **AC-2**, **AC-9**.
* **Happy path (Zero rebuild promotion `internal` -> `beta` -> `stable`)**: CI publishes `v1.1.0` with Authenticode thumbprint and `crates/license_signer` Ed25519 signature to `internal`; admin promotes `v1.1.0` to `beta` and then with step up TOTP to `stable`; `GET /v1/updates/stable` returns the exact same `sha256` and `updater_signature`, verifies **AC-5**, **AC-6**.
* **Failure case (Promoting or serving a held or blocked release)**: Admin marks `v1.1.0` as `blocked` with step up TOTP; attempting to promote `v1.1.0` fails with `RELEASE_NOT_PROMOTABLE`, and `GET /v1/updates/stable` omits `v1.1.0` as `latest` while returning `["1.1.0"]` in `blocked_versions`, verifies **AC-7**.
* **Failure case (Key recovery drill tamper check and append only log)**: Executing `POST /v1/releases/recovery_drill` with valid encrypted backup material records `signature_verified = 1` in `key_recovery_drills` for both `windows_authenticode` and `tauri_updater_ed25519`, and attempting `UPDATE` or `DELETE` on `key_recovery_drills` aborts via SQLite trigger, verifies **AC-8**.
* **Auth/permission (Cross customer portal isolation and OTP lockout)**: Customer A attempts to fetch Customer B's `license_id` or `invoice_id` and receives `FORBIDDEN_LICENSE_OWNER` / `FORBIDDEN_INVOICE_OWNER`; submitting 5 wrong OTP codes locks the account with `PORTAL_ACCOUNT_LOCKED`, verifies **AC-1**, **AC-2**.

## Build plan

Following Coachify's **Tracer Bullet** build approach, we first establish the cryptographic updater signing and cloud database schema thread through `crates/license_signer` and `crates/cloud_api`, then thicken the portal and release promotion APIs, and finish with the customer facing `Landing pages/` views (`/portal`, `/privacy`, `/downloads`), Vendor Admin channel promotion studio, and `.github/workflows/release.yml` signing pipeline:

1. Extend `crates/license_signer` with Tauri updater Ed25519 key custody (`u2026a`, `u2026b`), `SignUpdaterReleaseRequest`, `VerifyKeyRecoveryRequest`, and `signer_audit_log` operations (`sign_updater_release`, `verify_key_recovery`), satisfies **AC-5**, **AC-8**
2. Extend `crates/cloud_api` schema (`customer_portal_credentials`, `customer_portal_sessions`, `releases` Authenticode and updater columns, `release_channel_heads`, and append only `key_recovery_drills`) and implement `/v1/releases/publish`, `/v1/releases/promote` (zero rebuild metadata copy), `/v1/releases/recovery_drill`, and channel aware `GET /v1/updates/{channel}`, satisfies **AC-5**, **AC-6**, **AC-7**, **AC-8**
3. Implement Customer Portal authentication and tenant scoped endpoints in `crates/cloud_api` (`POST /v1/portal/login`, `POST /v1/portal/verify_otp`, `GET /v1/portal/overview`, `GET /v1/portal/license_file/{license_id}`, `GET /v1/portal/invoice/{invoice_id}`) with Argon2id, 6 digit OTP rate limiting, and customer ownership enforcement, satisfies **AC-1**, **AC-2**
4. Build the `/portal` Customer Portal view (4 sections: License Status & `.coachifylicense` Download, Verified Installer Download, Paid Vendor Invoices `INV-YYYY-NNNNN`, and Linked PC Installations), `/privacy` exact 5 field Telemetry Transparency page, and live channel selector on `/downloads` in `Landing pages/`, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-9**
5. Update `ui/src/components/vendor-admin/vendor-admin-view.tsx` (Releases & Channels tab) with zero rebuild `internal` -> `beta` -> `stable` promotion controls and the Key Recovery Drill studio, and upgrade `.github/workflows/release.yml` with the `release-internal` approval gate, Windows Authenticode signing, and Tauri updater Ed25519 signing, satisfies **AC-5**, **AC-6**, **AC-7**, **AC-8**, **AC-9**

## Consequences

**Positive**:
* Institution owners can self serve their `.coachifylicense` file, verified Windows installer, `INV-YYYY-NNNNN` invoices, and linked PC installations 24/7 on `/portal` without waiting on support staff.
* Privacy conscious schools and coaching centers can verify on `/privacy` the exact 5 fields sent during activation and daily refresh, reinforcing trust in Coachify's local SQLite architecture.
* Every release binary is compiled and signed once in CI after passing all quality gates and human approval, eliminating build drift between `internal`, `beta`, and `stable` channels.
* Recording executable key recovery drills for both the Windows Authenticode certificate and the Ed25519 updater/license keys in an append only table (`key_recovery_drills`) proves before `1.0` that a lost secret can be recovered without stranding customer PCs.

**Negative / tradeoffs**:
* Two step portal login (password plus 6 digit OTP) adds one extra screen for coaching owners signing in to `/portal`, which we mitigate with a 24 hour session token and clear Bangla and English prompts.
* Requiring a human approval gate (`release-internal`) and step up TOTP for `stable` promotion means hotfixes take an explicit admin confirmation step rather than deploying unattended.

**Neutral**:
* Features 65 and 66 run inside the monorepo (`Landing pages/`, `crates/cloud_api`, `crates/license_signer`, and `ui/`) today and can move cleanly to the dedicated cloud repository later without changing any wire contract.

## Follow-up

* [ ] Connect online payment gateway webhooks (bKash, Nagad, SSLCommerz) to automatically provision customer portal credentials and issue initial licenses upon checkout in Feature 67 (`/architect pricing & checkout`).
* [ ] Add self service device deactivation and slot replacement buttons to the Linked PC Installations section of `/portal` in Feature 68 (`/architect device management`).
