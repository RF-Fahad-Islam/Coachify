# 0041. Vendor Admin, Isolated License Signer, and Activation Refresh Service

**Date**: 2026-09-25
**Status**: Implemented

## Summary

Coachify adds the commercial licensing backend (`crates/cloud_api`), an isolated Ed25519 cryptographic signer (`crates/license_signer`), a six tab Vendor Admin console built with the visual system in `landing pages/`, and the desktop online activation and daily refresh client in `crates/core` and `ui`. A non developer staff member can record a bKash, Nagad, bank, or cash payment and immediately issue both an online refreshable license and a downloadable `.coachifylicense` file for offline centers, while the web application never holds the private signing key. Connected desktop installations automatically activate or refresh once every 24 hours (or on demand via Check Now), while a week long or month long cloud outage changes nothing on any customer PC.

## Structure

This umbrella specification coordinates two focused child specifications covering Slice 12 (Features 63 and 64):

1. [0041-vendor-admin-and-license-issuing.md](0041-vendor-admin-and-license-issuing.md): Feature 63 Vendor Admin console (`landing pages/` UI system), TOTP two factor login, customer and subscription billing ledger, manual bKash/Nagad/bank/cash payment recording, offline request code (`REQ-XXXX-XXXX-XXXX-XXXX` and `COACHIFY-REQ|v1|...`) license studio, revocations, release catalog, tamper evident `admin_audit_log`, and the isolated `crates/license_signer` service holding the Ed25519 private key.
2. [0041-activation-and-refresh-service.md](0041-activation-and-refresh-service.md): Feature 64 versioned desktop endpoints (`POST /v1/activate`, `POST /v1/refresh`, `GET /v1/updates/{channel}`), self service PC transfer flow (capped at 3 transfers per 12 months), 6 digit purchase pairing code activation, and the synchronous `ureq` desktop refresh and online activation client in `crates/core/src/licensing.rs` and `ui/src/components/settings/license-settings-card.tsx`.

**Cross child contract**:
* **Canonical Ed25519 envelope**: Both `crates/license_signer` (Feature 63) and `crates/core/src/licensing.rs` (Feature 64 and Spec 0037) share the identical `LicenseEnvelope` JSON wire format (`payload` canonical UTF8 JSON string + hex `signature`) and public keys (`k2026a`, `k2026b`).
* **Signer isolation boundary**: `crates/cloud_api` never accesses `SEED_K2026A` or `SEED_K2026B` directly in production; all license issues (manual payment extensions, offline request code bindings, online activations, and PC transfers) and all signed revocations must call `SignerService` over an authenticated `HMAC-SHA256` service boundary, which records every signature in `signer_audit_log`.
* **Minimal desktop telemetry guarantee**: Desktop activation and refresh calls send only `installation_id`, `license_id`, `current_serial`, `app_version`, and `active_students` (integer count). No student names, guardian phone numbers, or financial ledger rows ever leave the customer PC.

## Requirements

**User stories**:
* As a non technical vendor administrator, I want to log in with two factor authentication, record a customer's bKash or bank payment with reference and proof, and have the system automatically extend `paid_through`, increment the license serial, and produce a signed `.coachifylicense` for both online refresh and one click WhatsApp download.
* As a vendor support specialist assisting an offline coaching center, I want to paste their 20 character request code (`REQ-XXXX-XXXX-XXXX-XXXX`) or QR string (`COACHIFY-REQ|v1|<installation_id>|<checksum>`), verify its checksum, and download a bound `.coachifylicense` file to send back on WhatsApp.
* As a security conscious founder, I want the Ed25519 private signing key to live exclusively inside an isolated signer service (`crates/license_signer`) with its own append only audit log, so that a compromise or bug in the web application can never leak the private key.
* As an institution owner with internet access, I want my Coachify desktop app to activate via my license key or 6 digit purchase pairing code and silently pull subscription renewals once a day (or when I click Check Now), without ever slowing down or blocking my front desk during internet outages.
* As an institution owner replacing a broken computer, I want to activate my existing license on the new PC and confirm a hardware transfer (up to 3 self service transfers per year) so that the new PC binds cleanly and the old PC transitions safely.

**Acceptance criteria**:
* **AC-1**: Admin authentication and two factor security: Vendor admin login requires an Argon2id password plus an RFC 6238 6 digit TOTP code (with QR enrollment and 8 single use recovery codes), and requires step up TOTP verification for high risk actions (revoking a license or blocking a release).
* **AC-2**: Isolated Ed25519 signer boundary: The Ed25519 private signing key (`k2026a` / `k2026b`) resides exclusively inside `crates/license_signer`; `crates/cloud_api` holds zero private key material, requests signatures via `SignLicenseRequest` and `SignRevocationRequest` authenticated with `SIGNER_SHARED_SECRET`, and `crates/license_signer` appends every signature operation to `signer_audit_log`.
* **AC-3**: End to end manual payment and automatic license generation: Recording a confirmed payment (`bkash`, `nagad`, `bank`, `cash`, or `gateway`) in integer poisha extends `paid_through` (anchored to the existing `paid_through` date if renewed before grace expires, or to the payment date if already expired), generates a sequential `INV-YYYY-NNNNN` vendor invoice, increments `current_serial`, invokes the isolated signer, stores the immutable record in `license_issues`, and makes the `.coachifylicense` immediately available for both `POST /v1/refresh` and file download.
* **AC-4**: Offline request code and QR license issuing: The Vendor Admin License Issuing Studio accepts both the 20 character Base32 request code (`REQ-XXXX-XXXX-XXXX-XXXX`) and the QR string (`COACHIFY-REQ|v1|<installation_id>|<checksum>`), validates the HMAC SHA256 checksum, binds the target `installation_id`, calls the isolated signer to produce a valid `.coachifylicense`, and records the event in `activation_requests`.
* **AC-5**: Tamper evident admin audit log: Every mutation in `crates/cloud_api` (customer edits, plan/price updates, payment recording, license issuing, device transfers, revocations, release status changes) appends a row to `admin_audit_log` chained with `prev_hash` and `hash` (SHA256) and protected by database triggers that forbid `UPDATE` and `DELETE`.
* **AC-6**: Online desktop activation and 6 digit pairing: `POST /v1/activate` accepts either a `license_id` + `activation_key` or a short lived 6 digit `pairing_code` along with `installation_id`, `app_version`, and `active_students`; when the license is unbound or already bound to the same `installation_id`, it binds the installation, calls the isolated signer if needed, and returns the signed `LicenseEnvelope` which `license.activate_online` imports and activates in SQLite.
* **AC-7**: Self service hardware transfer with annual cap: When `POST /v1/activate` is called for a license already bound to a different `installation_id` without `confirm_transfer = true`, it returns status `already_bound_transfer_available` with the previous installation metadata; when called with `confirm_transfer = true` (via `license.confirm_transfer` on the desktop), it checks `transfer_count_12m < 3`, marks the old installation as `moved`, increments `transfer_count_12m` and `current_serial`, signs a new license bound to the new `installation_id`, and returns the active envelope (or returns `transfer_limit_exceeded` requiring admin reset if `>= 3`).
* **AC-8**: Daily background refresh and Check Now: `POST /v1/refresh` accepts `license_id`, `installation_id`, `current_serial`, `app_version`, and `active_students`, updates `installations.last_seen_at` and `active_students`, and returns a deterministic status (`updated` with newer signed `LicenseEnvelope` when `server_serial > current_serial`, `no_change` when current, `moved` when transferred to another PC, or `revoked` with a signed revocation blob).
* **AC-9**: Signed revocation enforcement: When a `super_admin` revokes a license in Vendor Admin, `crates/license_signer` produces a `SignedRevocationPayload` stored in `revocations`; when an online desktop receives `revoked` from `POST /v1/refresh`, `crates/core` verifies the Ed25519 signature on the revocation blob, persists `revoked_at` in `license_refresh_state`, and transitions the dispatcher into `suspended_read_only` mode while keeping all read, report, export, and backup commands open.
* **AC-10**: Week long outage resilience: The desktop refresh client in `crates/core/src/licensing.rs` uses synchronous HTTP (`ureq` with a 5 second timeout and injectable transport trait for tests) once every 24 hours on startup or on demand via `license.refresh_now`; any DNS failure, connection refusal, timeout, or HTTP 5xx error records `last_refresh_result = 'offline_deferred'` in `license_refresh_state` without altering local license status, showing modal errors, or blocking any desk command even across a multi week outage.
* **AC-11**: Versioned update metadata and blocked versions: `GET /v1/updates/{channel}` returns signed release metadata for `internal`, `beta`, and `stable` channels along with the list of `blocked` versions from the `releases` table so the desktop updater never upgrades into a held or blocked release.
* **AC-12**: UI design and visual alignment with `landing pages/`: The Vendor Admin console (6 tabs: Customers & Subscriptions, License Issuing Studio, Installations & Transfers, Revocations, Releases & Channels, Admin Audit Log) and the Desktop License Settings online activation controls use the `landing pages/` design language (`Sora` display typography, `Inter` body with `.tnum` tabular figures, `#faf7f2` warm cream canvas, `#16110d` ink headers, `#f97316` ember orange primary actions, `WindowFrame`, `CtaButton`, `SectionTag`, and `Chip` primitives).

## Decision

**Chosen option**: Option 1: Workspace Integrated Rust Commercial API (`crates/cloud_api`), Isolated Ed25519 Signer Service (`crates/license_signer`), `landing pages/` Styled Vendor Admin Console, and Synchronous `ureq` Desktop Refresh Client

We implement Features 63 and 64 as two modular Rust crates (`crates/license_signer` and `crates/cloud_api`) plus a React Vendor Admin interface styled directly with the components and tokens from `landing pages/` (`Sora`, `Inter`, warm cream `#faf7f2`, ink `#16110d`, ember `#f97316`, `WindowFrame`, `CtaButton`, `SectionTag`, and `Chip`), paired with Migration 0022 (`license_refresh_state`) and synchronous online activation/refresh RPC commands in `crates/core` and `ui`.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:

1. **Commercial Cloud Schema (`crates/cloud_api` SQL schema)**:
   * `admin_users`: `id TEXT PRIMARY KEY`, `email TEXT UNIQUE NOT NULL`, `name TEXT NOT NULL`, `role TEXT NOT NULL CHECK (role IN ('super_admin', 'ops_admin', 'support_admin'))`, `password_hash TEXT NOT NULL`, `totp_secret_enc TEXT NOT NULL`, `totp_enabled INTEGER NOT NULL DEFAULT 1`, `recovery_codes_json TEXT NOT NULL DEFAULT '[]'`, `status TEXT NOT NULL DEFAULT 'active'`, `last_login_at TEXT`, `created_at TEXT NOT NULL`.
   * `admin_audit_log`: `seq INTEGER PRIMARY KEY AUTOINCREMENT`, `at TEXT NOT NULL`, `admin_user_id TEXT NOT NULL REFERENCES admin_users(id)`, `action TEXT NOT NULL`, `entity_type TEXT NOT NULL`, `entity_id TEXT NOT NULL`, `ip_address TEXT`, `before_json TEXT`, `after_json TEXT`, `prev_hash TEXT NOT NULL`, `hash TEXT NOT NULL` (protected by `BEFORE UPDATE` and `BEFORE DELETE` abort triggers).
   * `customers`: `id TEXT PRIMARY KEY`, `org_name TEXT NOT NULL`, `owner_name TEXT NOT NULL`, `phone TEXT NOT NULL`, `email TEXT`, `district TEXT`, `notes TEXT`, `status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'archived'))`, `created_at TEXT NOT NULL`, `updated_at TEXT NOT NULL`.
   * `plans`: `id TEXT PRIMARY KEY`, `code TEXT UNIQUE NOT NULL CHECK (code IN ('trial', 'local', 'local_pro'))`, `name TEXT NOT NULL`, `student_limit INTEGER`, `workstation_limit INTEGER`, `capabilities_json TEXT NOT NULL`, `is_active INTEGER NOT NULL DEFAULT 1`.
   * `prices`: `id TEXT PRIMARY KEY`, `plan_id TEXT NOT NULL REFERENCES plans(id)`, `period_months INTEGER NOT NULL CHECK (period_months IN (1, 6, 12))`, `currency TEXT NOT NULL DEFAULT 'BDT'`, `amount_poisha INTEGER NOT NULL`, `active_from TEXT NOT NULL`, `active_to TEXT`.
   * `subscriptions`: `id TEXT PRIMARY KEY`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `plan_id TEXT NOT NULL REFERENCES plans(id)`, `status TEXT NOT NULL CHECK (status IN ('trial', 'active', 'grace', 'expired', 'suspended'))`, `paid_through TEXT NOT NULL`, `grace_days INTEGER NOT NULL DEFAULT 14`, `started_at TEXT NOT NULL`, `cancelled_at TEXT`.
   * `payments`: `id TEXT PRIMARY KEY`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `subscription_id TEXT NOT NULL REFERENCES subscriptions(id)`, `amount_poisha INTEGER NOT NULL`, `method TEXT NOT NULL CHECK (method IN ('bkash', 'nagad', 'bank', 'cash', 'gateway'))`, `reference TEXT NOT NULL`, `proof_url TEXT`, `period_months INTEGER NOT NULL`, `recorded_by TEXT NOT NULL REFERENCES admin_users(id)`, `status TEXT NOT NULL DEFAULT 'confirmed' CHECK (status IN ('confirmed', 'refunded'))`, `recorded_at TEXT NOT NULL`.
   * `vendor_invoices`: `id TEXT PRIMARY KEY`, `payment_id TEXT UNIQUE NOT NULL REFERENCES payments(id)`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `invoice_number TEXT UNIQUE NOT NULL`, `subtotal_poisha INTEGER NOT NULL`, `vat_poisha INTEGER NOT NULL DEFAULT 0`, `total_poisha INTEGER NOT NULL`, `issued_at TEXT NOT NULL`, `pdf_url TEXT`.
   * `licenses`: `id TEXT PRIMARY KEY`, `license_id TEXT UNIQUE NOT NULL`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `subscription_id TEXT NOT NULL REFERENCES subscriptions(id)`, `activation_key_hash TEXT NOT NULL`, `pairing_code TEXT`, `pairing_expires_at TEXT`, `current_serial INTEGER NOT NULL DEFAULT 1`, `bound_installation_id TEXT`, `key_id TEXT NOT NULL DEFAULT 'k2026a'`, `last_issued_blob TEXT`, `status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'revoked'))`, `transfer_count_12m INTEGER NOT NULL DEFAULT 0`, `updated_at TEXT NOT NULL`.
   * `license_issues`: `id TEXT PRIMARY KEY`, `license_id TEXT NOT NULL REFERENCES licenses(license_id)`, `serial INTEGER NOT NULL`, `installation_id TEXT NOT NULL`, `kind TEXT NOT NULL CHECK (kind IN ('paid', 'extension', 'trial_conversion', 'transfer'))`, `payload_json TEXT NOT NULL`, `signature TEXT NOT NULL`, `signed_blob TEXT NOT NULL`, `issued_by TEXT REFERENCES admin_users(id)`, `reason TEXT NOT NULL`, `issued_at TEXT NOT NULL`, `UNIQUE(license_id, serial)` (append only via DB triggers).
   * `installations`: `id TEXT PRIMARY KEY`, `installation_id TEXT NOT NULL`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `license_id TEXT NOT NULL REFERENCES licenses(license_id)`, `label TEXT NOT NULL`, `role TEXT NOT NULL DEFAULT 'host'`, `app_version TEXT NOT NULL`, `active_students INTEGER NOT NULL DEFAULT 0`, `first_seen_at TEXT NOT NULL`, `last_seen_at TEXT NOT NULL`, `status TEXT NOT NULL CHECK (status IN ('active', 'moved', 'revoked'))`, `UNIQUE(license_id, installation_id)`.
   * `activation_requests`: `id TEXT PRIMARY KEY`, `customer_id TEXT NOT NULL REFERENCES customers(id)`, `license_id TEXT NOT NULL REFERENCES licenses(license_id)`, `request_code TEXT NOT NULL`, `qr_payload TEXT`, `installation_id TEXT NOT NULL`, `checksum_verified INTEGER NOT NULL DEFAULT 1`, `issued_serial_id TEXT NOT NULL REFERENCES license_issues(id)`, `processed_by TEXT NOT NULL REFERENCES admin_users(id)`, `created_at TEXT NOT NULL`.
   * `revocations`: `id TEXT PRIMARY KEY`, `license_id TEXT NOT NULL REFERENCES licenses(license_id)`, `reason TEXT NOT NULL`, `revoked_by TEXT NOT NULL REFERENCES admin_users(id)`, `revoked_at TEXT NOT NULL`, `key_id TEXT NOT NULL`, `signed_revocation_blob TEXT NOT NULL` (append only via DB triggers).
   * `releases`: `id TEXT PRIMARY KEY`, `version TEXT UNIQUE NOT NULL`, `channel TEXT NOT NULL CHECK (channel IN ('internal', 'beta', 'stable'))`, `notes TEXT NOT NULL`, `artifact_url TEXT NOT NULL`, `sha256 TEXT NOT NULL`, `updater_signature TEXT NOT NULL`, `min_upgrade_from TEXT`, `status TEXT NOT NULL CHECK (status IN ('live', 'held', 'blocked'))`, `published_by TEXT NOT NULL REFERENCES admin_users(id)`, `published_at TEXT NOT NULL`.
   * `signer_audit_log` (inside `crates/license_signer` isolated store): `seq INTEGER PRIMARY KEY AUTOINCREMENT`, `at TEXT NOT NULL`, `caller_service_id TEXT NOT NULL`, `operation TEXT NOT NULL CHECK (operation IN ('sign_license', 'sign_revocation'))`, `license_id TEXT NOT NULL`, `installation_id TEXT NOT NULL`, `serial INTEGER NOT NULL`, `key_id TEXT NOT NULL`, `payload_sha256 TEXT NOT NULL`, `signature_prefix TEXT NOT NULL` (append only via DB triggers).

2. **Desktop SQLite Migration 0022 (`migrations/0022_license_refresh_state.sql`)**:
   ```sql
   CREATE TABLE IF NOT EXISTS license_refresh_state (
       id INTEGER PRIMARY KEY CHECK (id = 1),
       activation_key TEXT,
       service_base_url TEXT NOT NULL DEFAULT 'https://api.coachify.app',
       last_refresh_at TEXT,
       last_refresh_result TEXT NOT NULL DEFAULT 'never' CHECK (
           last_refresh_result IN ('never', 'ok', 'updated', 'no_change', 'moved', 'revoked', 'offline_deferred')
       ),
       last_error TEXT,
       revoked_at TEXT,
       revocation_reason TEXT,
       revocation_blob TEXT,
       updated_at TEXT NOT NULL
   );
   ```

**State transitions**:
* **Subscription & License State Machine**:
  * `trial` -> (manual payment recorded or online purchase) -> `active` (serial = 1, `paid_through` set to `today + period_months`)
  * `active` -> (within 14 days after `paid_through`) -> `grace`
  * `active` or `grace` -> (payment recorded before grace ends) -> `active` (`new_paid_through = old_paid_through + period_months`, `serial = current_serial + 1`)
  * `expired` -> (payment recorded after grace ends) -> `active` (`new_paid_through = payment_date + period_months`, `serial = current_serial + 1`)
  * `active` / `grace` / `expired` -> (`super_admin` revokes with step up TOTP) -> `revoked` (signed revocation blob created; online desktop enters `suspended_read_only` on next refresh)
* **Installation Binding & Hardware Transfer State Machine**:
  * `unbound` -> (`POST /v1/activate` or offline request code issue) -> `bound (active)` on `installation_id = A`
  * `bound (active on A)` -> (`POST /v1/activate` from `installation_id = B` with `confirm_transfer = false`) -> returns `already_bound_transfer_available`
  * `bound (active on A)` -> (`POST /v1/activate` from `installation_id = B` with `confirm_transfer = true` and `transfer_count_12m < 3`) -> installation `A` marked `moved`, installation `B` marked `active`, `transfer_count_12m` incremented, `serial` incremented, signed license returned for `B`; when `A` next calls `POST /v1/refresh`, `A` receives `moved` and enters its 14 day `moved_transitional` window.

**API surface**:

| Endpoint / Command | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `signer.sign_license` | Internal RPC / HTTP `POST /internal/v1/sign-license` | `caller_id`, `hmac_token`, `payload: LicensePayload` | `LicenseEnvelope`, `audit_seq` | `HMAC-SHA256` (`SIGNER_SHARED_SECRET`) | `signer_unauthorized`, `signer_key_unavailable`, `signer_payload_invalid` |
| `signer.sign_revocation` | Internal RPC / HTTP `POST /internal/v1/sign-revocation` | `caller_id`, `hmac_token`, `license_id`, `reason`, `revoked_at` | `SignedRevocationEnvelope`, `audit_seq` | `HMAC-SHA256` (`SIGNER_SHARED_SECRET`) | `signer_unauthorized`, `signer_key_unavailable` |
| `POST /v1/activate` | Public Versioned HTTP | `license_id_or_pairing_code`, `activation_key` (opt), `installation_id`, `app_version`, `active_students`, `confirm_transfer: bool` | `ActivateResponse` (`activated` + `envelope`, or `already_bound_transfer_available`) | Activation key or 6 digit pairing code (rate limited) | `license_not_found`, `activation_credentials_invalid`, `license_revoked`, `transfer_limit_exceeded` |
| `POST /v1/refresh` | Public Versioned HTTP | `license_id`, `installation_id`, `current_serial`, `app_version`, `active_students` | `RefreshResponse` (`updated` + `envelope`, `no_change`, `moved`, or `revoked` + `revocation_envelope`) | Bound `installation_id` + `license_id` (rate limited) | `license_not_found`, `refresh_payload_invalid` |
| `GET /v1/updates/{channel}` | Public Versioned HTTP | `channel` (`internal`, `beta`, `stable`), `current_version` | `UpdateMetadataResponse` (`latest_release`, `blocked_versions`, `is_upgrade_allowed`) | Public (rate limited) | `invalid_channel` |
| `admin.auth.login_totp` | Admin Command / API | `email`, `password`, `totp_code` | `AdminSessionToken`, `AdminUserView` | Public login endpoint | `admin_invalid_credentials`, `admin_totp_invalid`, `admin_account_suspended` |
| `admin.payments.record` | Admin Command / API | `customer_id`, `subscription_id`, `amount_poisha`, `method`, `reference`, `proof_url`, `period_months`, `installation_id` (opt) | `RecordedPaymentResult` (`payment`, `invoice`, `license_issue`, `signed_blob`) | `ops_admin` or `super_admin` | `subscription_not_found`, `invalid_payment_amount`, `signer_unavailable` |
| `admin.licenses.issue_offline` | Admin Command / API | `license_id`, `request_code_or_qr`, `reason` | `OfflineIssueResult` (`installation_id`, `serial`, `signed_blob`, `download_filename`) | `ops_admin` or `super_admin` | `request_code_checksum_invalid`, `request_code_unresolvable`, `license_revoked` |
| `admin.licenses.revoke` | Admin Command / API | `license_id`, `reason`, `step_up_totp_code` | `RevocationRecord` (`signed_revocation_blob`) | `super_admin` + step up TOTP | `admin_totp_invalid`, `admin_permission_denied`, `license_already_revoked` |
| `license.activate_online` | Desktop RPC (`crates/core`) | `code_or_license_id: String`, `activation_key: Option<String>`, `service_url: Option<String>` | `OnlineActivationResult` (`status`, `license_status`, `transfer_prompt`) | Desktop `settings.manage` | `activation_credentials_invalid`, `transfer_limit_exceeded`, `network_unreachable` |
| `license.confirm_transfer` | Desktop RPC (`crates/core`) | `license_id: String`, `activation_key: String`, `service_url: Option<String>` | `LicenseStatusResponse` | Desktop `settings.manage` | `transfer_limit_exceeded`, `license_revoked`, `network_unreachable` |
| `license.refresh_now` | Desktop RPC (`crates/core`) | `force: bool` | `RefreshNowResult` (`outcome`, `license_status`, `last_refresh_at`) | Desktop Authenticated | Never errors on network outage (`offline_deferred` returned cleanly) |

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| `admin.payments.record` | Extended `paid_through` date | Computed in `crates/cloud_api`: if `today <= subscription.paid_through + 14 days`, `subscription.paid_through + period_months`; else `today + period_months` |
| `admin.payments.record` | Sequential invoice number (`INV-YYYY-NNNNN`) | Derived from current year (`Asia/Dhaka`) and `COUNT(*) + 1` in `vendor_invoices` |
| `admin.payments.record` | Next license `serial` | `licenses.current_serial + 1` in `licenses` table |
| `admin.licenses.issue_offline` | Target `installation_id` and checksum verification | Parsed from `COACHIFY-REQ\|v1\|<installation_id>\|<checksum>` or looked up from `installations` / resolved installation UUID and verified via `HMAC-SHA256(REQUEST_CODE_HMAC_KEY, installation_id)` |
| `POST /v1/activate` | Self service transfer eligibility (`transfer_count_12m < 3`) | `licenses.transfer_count_12m` column in cloud DB |
| `POST /v1/refresh` | Desktop telemetry fields (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`) | Read from local `%ProgramData%\Coachify\installation_id`, `active_license` view, `env!("CARGO_PKG_VERSION")`, and `SELECT COUNT(*) FROM enrollments WHERE status = 'active'` |
| `POST /v1/refresh` | Revocation enforcement (`suspended_read_only`) | Ed25519 verification of `SignedRevocationEnvelope` against embedded `SEED_K2026A` / `SEED_K2026B` public keys in `crates/core/src/licensing.rs`, stored in `license_refresh_state.revoked_at` |
| `GET /v1/updates/{channel}` | Blocked versions list | `SELECT version FROM releases WHERE status = 'blocked'` in `crates/cloud_api` |

**Key invariants**:
* The web app (`crates/cloud_api` and React UI) never holds or reads the Ed25519 private key seed; only `crates/license_signer` loads the private seed and signs canonical payloads.
* Every admin mutation and every signature operation writes an append only audit row (`admin_audit_log` with SHA256 hash chaining, and `signer_audit_log`); database triggers block `UPDATE` and `DELETE` on `admin_audit_log`, `signer_audit_log`, `license_issues`, and `revocations`.
* Renewing before the 14 day grace period expires extends from the old `paid_through` date; renewing after expiry extends from the payment date.
* Desktop background refresh (`POST /v1/refresh`) never sends personal student records, financial entries, or staff data; it transmits strictly the 5 scalar telemetry fields (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`).
* A network failure, DNS outage, or 5xx error during daily desktop refresh never downgrades or invalidates the local license; even a month long cloud outage leaves all customer PCs running normally.
* Self service PC transfers via `POST /v1/activate` with `confirm_transfer = true` are capped at 3 transfers per 12 months per license unless reset by an admin in Vendor Admin.

**Security model**:
* **Vendor Admin roles**:
  * `super_admin`: Full access including admin user creation, plan and price edits, manual payments, license issuing, transfer count resets, license revocations (requires step up TOTP), and release blocking (requires step up TOTP).
  * `ops_admin`: Customer management, subscription management, manual payment recording (`bkash`, `nagad`, `bank`, `cash`), online/offline license issuing, and device transfer approval. Cannot revoke licenses or block releases.
  * `support_admin`: Read only access to customers, subscriptions, installations, and audit logs, plus ability to reissue/download existing `.coachifylicense` files and generateoffline request bindings.
* **Two factor authentication**: All vendor admin roles must authenticate with Argon2id password + RFC 6238 6 digit TOTP code.
* **Isolated signer authentication**: `crates/license_signer` verifies an `HMAC-SHA256` request signature computed over `(timestamp || caller_service_id || canonical_payload_bytes)` using `SIGNER_SHARED_SECRET` and rejects requests older than 60 seconds.
* **Desktop permissions**: `license.activate_online` and `license.confirm_transfer` require `settings.manage` (Owner/Admin), while `license.refresh_now` is permitted for any authenticated staff session and runs silently on startup when the last check is older than 24 hours.

**Configuration required**:
* `COACHIFY_SIGNER_PRIVATE_SEED_HEX`: 64 character hex Ed25519 seed loaded exclusively by `crates/license_signer` (falls back to deterministic dev seed `SEED_K2026A` only in debug/test builds).
* `COACHIFY_SIGNER_KEY_ID`: Active key identifier (`k2026a` default, `k2026b` for rotation).
* `COACHIFY_SIGNER_SHARED_SECRET`: Shared HMAC secret authenticating calls from `crates/cloud_api` to `crates/license_signer`.
* `COACHIFY_CLOUD_DB_PATH`: SQLite/Postgres connection string for `crates/cloud_api`.
* `COACHIFY_LICENSE_API_URL`: Base URL for desktop activation and daily refresh (defaults to `https://api.coachify.app`, overridable in tests or via env).

**Critical test scenarios**:
* End to end bKash payment to desktop refresh: Non developer `ops_admin` logs in with TOTP, records a 6 month bKash payment (`reference = "BKASH998811"`), verifies `paid_through` extension, `INV-YYYY-NNNNN` invoice creation, `signer_audit_log` entry, and `admin_audit_log` hash chain, then desktop client calls `license.refresh_now` and automatically activates the new serial, verifies **AC-1**, **AC-2**, **AC-3**, **AC-5**, **AC-8**.
* End to end offline request code license issuance: Desktop generates QR payload (`COACHIFY-REQ|v1|<installation_id>|<checksum>`) and 20 char `REQ-XXXX-...` code; admin submits it in License Issuing Studio, verifies checksum, downloads `.coachifylicense`, and imports it via `license.import_file` on an offline desktop, verifies **AC-2**, **AC-4**.
* Self service PC move and transfer cap: Desktop B calls `license.activate_online` with a license already bound to Desktop A, receives `already_bound_transfer_available`, calls `license.confirm_transfer`, verifies Desktop B receives active serial `N+1`, Desktop A receives `moved` on next refresh, and a 4th transfer attempt within 12 months returns `transfer_limit_exceeded`, verifies **AC-6**, **AC-7**, **AC-8**.
* Step up TOTP revocation and signed desktop enforcement: `ops_admin` attempt to revoke is denied (`admin_permission_denied`); `super_admin` revokes with step up TOTP, `crates/license_signer` signs the revocation payload, and Desktop B's next `license.refresh_now` verifies the Ed25519 revocation signature and enters `suspended_read_only` mode while still permitting reports and backups, verifies **AC-1**, **AC-2**, **AC-9**.
* Week long cloud outage resilience: Desktop with an active license executes `license.refresh_now` and startup checks against an unreachable/503 endpoint across simulated 10 day clock advance, verifying `last_refresh_result = 'offline_deferred'` and zero disruption to admissions or fee collection, verifies **AC-10**.
* Update channel metadata and blocked version filter: Admin marks release `1.0.2` as `blocked` in Vendor Admin; `GET /v1/updates/stable` lists `1.0.2` in `blocked_versions` and returns `1.0.1` as the safe live release, verifies **AC-11**.

## Build plan

Following the project's **Tracer Bullet** build approach, implementation slices through the isolated signer, cloud API, desktop refresh state, and `landing pages/` UI in 6 end to end increments:

1. [x] Create `crates/license_signer` holding the isolated Ed25519 signing engine (`sign_license`, `sign_revocation`), `HMAC-SHA256` service authentication, and append only `signer_audit_log` table with immutability triggers, satisfies **AC-2**, **AC-9**.
2. [x] Create `crates/cloud_api` with the commercial database schema (`admin_users`, `admin_audit_log`, `customers`, `plans`, `prices`, `subscriptions`, `payments`, `vendor_invoices`, `licenses`, `license_issues`, `installations`, `activation_requests`, `revocations`, `releases`), Argon2id + RFC 6238 TOTP login, step up TOTP checks, and hash chained `admin_audit_log`, satisfies **AC-1**, **AC-5**.
3. [x] Implement the Vendor Admin billing and license issuing workflows in `crates/cloud_api` (`admin.payments.record` with grace aware `paid_through` calculation and `INV-YYYY-NNNNN` invoicing, `admin.licenses.issue_offline` with `REQ-XXXX-...` and `COACHIFY-REQ|v1|...` verification, `admin.licenses.revoke`, and release channel management), satisfies **AC-3**, **AC-4**, **AC-9**, **AC-11**.
4. [x] Implement the versioned desktop endpoints (`POST /v1/activate` with 6 digit pairing and 3/year self service transfer flow, `POST /v1/refresh`, and `GET /v1/updates/{channel}`) in `crates/cloud_api`, plus Migration 0022 (`license_refresh_state`) and synchronous `ureq` / injectable transport handlers (`license.activate_online`, `license.confirm_transfer`, `license.refresh_now`, and 24h startup auto refresh) in `crates/core/src/licensing.rs`, satisfies **AC-6**, **AC-7**, **AC-8**, **AC-9**, **AC-10**, **AC-11**.
5. [x] Build the 6 tab Vendor Admin UI (`Customers & Subscriptions`, `License Issuing Studio`, `Installations & Transfers`, `Revocations`, `Releases & Channels`, `Admin Audit Log`) and update `ui/src/components/settings/license-settings-card.tsx` (Online Activation, 6 digit pairing, Check Now button, and Transfer confirmation modal) using the `landing pages/` visual system (`Sora`, `Inter`, `#faf7f2` cream, `#16110d` ink, `#f97316` ember, `WindowFrame`, `CtaButton`, `SectionTag`, `Chip`), satisfies **AC-1**, **AC-3**, **AC-4**, **AC-6**, **AC-7**, **AC-8**, **AC-12**.
6. [x] Add comprehensive end to end integration tests across `crates/license_signer`, `crates/cloud_api`, and `crates/core` plus Vitest UI component tests covering bKash manual payment to desktop auto activation, offline request code issuance, self service PC transfer limits, signed revocation enforcement, and week long cloud outage resilience, satisfies **AC-1** through **AC-12**.

## Consequences

**Positive**:
* Non technical staff can record a bKash payment and issue both online and offline licenses in one click without ever touching a CLI or private key.
* The Ed25519 private signing key is completely isolated inside `crates/license_signer` with its own append only audit log, satisfying the security boundary that the web app never holds the private key.
* Desktop customers with internet access enjoy zero friction activation, automatic daily renewal updates, and self service PC replacement (up to 3 times a year) while offline customers continue to work seamlessly with `.coachifylicense` files.
* Reusing the `landing pages/` visual components (`WindowFrame`, `CtaButton`, `SectionTag`, `Chip`, warm cream `#faf7f2`, ink `#16110d`, ember `#f97316`) gives the Vendor Admin and Desktop licensing settings a cohesive, high craft commercial aesthetic.

**Negative / tradeoffs**:
* Maintaining an isolated signer crate (`crates/license_signer`) adds an internal service call and shared secret (`SIGNER_SHARED_SECRET`) configuration compared to signing directly inside the web backend.
* Revocations only reach a customer PC when that PC connects to `POST /v1/refresh`; a permanently airgapped PC remains active until its signed `valid_until` date passes.

**Neutral**:
* Migration 0022 adds the single row `license_refresh_state` table to the desktop SQLite schema.
* Placing `crates/cloud_api` and `crates/license_signer` inside the Cargo workspace allows `/develop` and `cargo test --workspace` to verify the entire desktop to cloud to signer handshake in a single repository before splitting out a standalone cloud deployment repository later.

## Follow-up

* [ ] When the standalone cloud repository is provisioned for production hosting, move `crates/cloud_api`, `crates/license_signer`, and the Vendor Admin UI routes there while keeping the shared wire contract tests in `crates/core`.
* [ ] Provision the production `k2026a` and `k2026b` Ed25519 seeds in cloud KMS / secret manager for `crates/license_signer` before 1.0 commercial launch.
