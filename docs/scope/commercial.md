# Commercial (Slice 12)

Back to [index](index.md). Can run in parallel once Slice 3 is done. Company paperwork, merchant accounts, and the code signing certificate start after pilots (decided 2026-09-20), so pilots run free and unsigned, and feature 66 lands just before 1.0. Features 63 to 65 and 67 to 68 belong to the separate cloud repository once it exists; move these sections there when it is created.

### 62. Desktop licensing & read only mode · GA
Signed license verification against embedded public keys, installation binding, serial check, trial, grace, read only mode enforced in the dispatcher, the moved license window, and the clock guard. Plans: Local, and Local Pro, which adds only LAN. Includes offline license file (download/import signed license when a machine cannot stay online).
**Done when:** a tampered, foreign, or older license is refused with a reason; expired installs can still view, export, back up, and restore; a PC clock set back is caught with a friendly message; nothing ever waits on the license server; an offline license file activates a machine that cannot reach the internet.
spec [0037](../specs/0037-desktop-licensing-and-read-only-mode/index.md) · code in `crates/core/src/licensing.rs`, `ui/src/components/settings/license-settings-card.tsx`
- [x] Design it (spec): `/architect desktop licensing & read only mode`
- [x] Build it: `/develop desktop licensing & read only mode`
  - [x] Migration 0021 establishing licenses table, active license view, and append only triggers, covers AC-11
  - [x] Ed25519 signature verification engine with embedded public keys, installation binding, and serial validation, covers AC-1, AC-2, AC-3, AC-8
  - [x] Setup bootstrap 14 day trial generation and core RPC commands (get_status, get_request_code, import_file), covers AC-4, AC-8, AC-9
  - [x] Dispatcher license check pipeline for expiration, 14 day grace, moved machine transitional window, and clock guard, covers AC-5, AC-6, AC-7, AC-10
  - [x] React settings view, top navigation read only banner, and moved license modal with QR code preview, covers AC-5, AC-6, AC-7, AC-9
- [x] Verify it: `/check verify desktop licensing & read only mode`
- [x] Test it: `/test desktop licensing & read only mode`
- [x] Review it (fresh model): `/check review desktop licensing & read only mode`
- [x] Document it: `/document desktop licensing & read only mode`

### 63. Vendor admin & license issuing · in-progress · GA
Admin panel with two factor login: customers, plans, prices, subscriptions, manual payment recording, license issuing (online and from an offline request code), installations, revocations, releases, and an admin audit log. The signing key lives only in an isolated signer.
**Done when:** a non developer records a bKash payment and issues both an online and an offline license end to end; every admin action is audited; the web app never holds the private key.
spec [0041](../specs/0041-vendor-admin-and-activation-service/index.md) ([0041a](../specs/0041-vendor-admin-and-activation-service/0041-vendor-admin-and-license-issuing.md)) · code in `crates/license_signer/src/lib.rs`, `crates/cloud_api/src/lib.rs`, `ui/src/components/vendor-admin/vendor-admin-view.tsx`
- [x] Design it (spec): `/architect vendor admin & license issuing`
- [x] Build it: `/develop vendor admin & license issuing`
  - [x] Isolated `crates/license_signer` service with Ed25519 key custody, HMAC service auth, and append only `signer_audit_log`, covers AC-2, AC-9
  - [x] Commercial schema in `crates/cloud_api` with Argon2id + RFC 6238 TOTP login, step up TOTP checks, and SHA256 hash chained `admin_audit_log`, covers AC-1, AC-5
  - [x] Manual payment recording (`bkash`, `nagad`, `bank`, `cash`), `INV-YYYY-NNNNN` invoicing, grace aware `paid_through` extension, offline `REQ` and QR studio, revocations, and release channels, covers AC-3, AC-4, AC-9, AC-11
  - [x] 6 tab Vendor Admin UI built with `landing pages/` tokens and primitives (`WindowFrame`, `CtaButton`, `SectionTag`, `Chip`, `Stat`) and end to end test suite, covers AC-1, AC-3, AC-4, AC-12
- [x] Verify it: `/check verify vendor admin & license issuing`
- [x] Test it: `/test vendor admin & license issuing`
- [ ] Review it (fresh model): `/check review vendor admin & license issuing`
- [x] Document it: `/document vendor admin & license issuing`

### 64. Activation & refresh service · in-progress · GA
Small versioned endpoints for activation, daily refresh (renewals, moves, revocations), and update metadata, sending only installation ID, license ID, app version, and active student count. Includes automatic desktop activation (linked Coachify desktop refreshes and activates automatically after purchase).
**Done when:** a renewal recorded in admin reaches an online install within a day or on "Check now"; moving a license to a new PC works; a week long outage changes nothing on any customer PC.
spec [0041](../specs/0041-vendor-admin-and-activation-service/index.md) ([0041b](../specs/0041-vendor-admin-and-activation-service/0041-activation-and-refresh-service.md)) · code in `crates/cloud_api/src/lib.rs`, `crates/core/src/licensing.rs`, `ui/src/components/settings/license-settings-card.tsx`
- [x] Design it (spec): `/architect activation & refresh service`
- [x] Build it: `/develop activation & refresh service`
  - [x] Versioned `/v1/activate`, `/v1/refresh`, and `/v1/updates/{channel}` endpoints in `crates/cloud_api` with 6 digit pairing and 3 per year self service PC transfer cap, covers AC-6, AC-7, AC-8, AC-11
  - [x] Desktop Migration 0022 (`license_refresh_state`) and synchronous `ureq` / injectable `RefreshTransport` client (`license.activate_online`, `license.confirm_transfer`, `license.refresh_now`, 24h auto check) in `crates/core/src/licensing.rs`, covers AC-6, AC-7, AC-8, AC-9, AC-10
  - [x] Desktop License Settings online activation bar, pairing input, Check Now button, and transfer confirmation modal styled with `landing pages/` primitives plus outage resilience tests, covers AC-6, AC-7, AC-8, AC-10, AC-12
- [x] Verify it: `/check verify activation & refresh service`
- [x] Test it: `/test activation & refresh service`
- [ ] Review it (fresh model): `/check review activation & refresh service`
- [x] Document it: `/document activation & refresh service`

### 65. Website, downloads & customer portal · in-progress
Marketing landing page (story driven, focused on demo download and sales conversion), download center (installer, release/version information, demo download), support/documentation (help articles, installation guides, support access), and a minimal portal: license status and download, installers, vendor invoices, installations list.
**Done when:** a customer logs in, downloads the installer and their license file, and sees their invoices; the privacy page lists exactly what the app sends; the marketing page tells the Coachify story and converts visitors to demo downloads.
spec [0042](../specs/0042-website-portal-and-release-signing/index.md) ([0042a](../specs/0042-website-portal-and-release-signing/0042-website-downloads-and-customer-portal.md))
- [x] Design it (spec): `/architect website, downloads & customer portal`
- [x] Build it: `/develop website, downloads & customer portal`
  - [x] Customer portal credentials, OTP session authentication, and tenant scoped `/v1/portal/*` endpoints (`login`, `verify_otp`, `overview`, `license_file`, `invoice`) in `crates/cloud_api`, covers AC-1, AC-2
  - [x] Customer Portal (`/portal`) 4 section UI (License Status & `.coachifylicense` Download, Verified Installer Download, Paid Vendor Invoices `INV-YYYY-NNNNN`, Linked PC Installations) in `Landing pages/`, covers AC-1, AC-2, AC-9
  - [x] Exact 5 field desktop telemetry transparency page (`/privacy`) and live channel release metadata on `/downloads` and `/help` in `Landing pages/`, covers AC-3, AC-4, AC-9
- [x] Verify it: `/check verify website, downloads & customer portal`
- [x] Test it: `/test website, downloads & customer portal`
- [ ] Review it (fresh model): `/check review website, downloads & customer portal`
- [x] Document it: `/document website, downloads & customer portal`

### 66. Code signing & release channels · in-progress · GA
Windows code signing from CI through a hardware or cloud backed certificate, updater key custody and backups, internal, beta, and stable channels, and a blocked versions list.
**Done when:** a tagged release is signed and published to internal only after all required suites pass and a person approves; promotion to stable copies metadata without rebuilding; a recovery test of both keys is recorded.
spec [0042](../specs/0042-website-portal-and-release-signing/index.md) ([0042b](../specs/0042-website-portal-and-release-signing/0042-code-signing-and-release-channels.md))
- [x] Design it (spec): `/architect code signing & release channels`
- [x] Build it: `/develop code signing & release channels`
  - [x] Tauri updater Ed25519 key custody (`u2026a`, `u2026b`), `sign_updater_release`, and `verify_key_recovery` inside isolated `crates/license_signer`, covers AC-5, AC-8
  - [x] `release_channel_heads` and append only `key_recovery_drills` tables plus zero rebuild promotion (`/v1/releases/promote`), blocked version enforcement, and recovery drill endpoints in `crates/cloud_api`, covers AC-5, AC-6, AC-7, AC-8
  - [x] Vendor Admin Release Promotion and Key Recovery Drill studio in `ui/src/components/vendor-admin/vendor-admin-view.tsx` plus gated `.github/workflows/release.yml` (`release-internal` human approval, Authenticode, and Ed25519 signing), covers AC-5, AC-6, AC-8, AC-9
- [x] Verify it: `/check verify code signing & release channels`
- [x] Test it: `/test code signing & release channels`
- [ ] Review it (fresh model): `/check review code signing & release channels`
- [x] Document it: `/document code signing & release channels`

### 67. Pricing & checkout · in-progress · GA
Purchase or renew Coachify directly online. Payment gateway integration for the Coachify website. Automatic license generation after verified payment.
**Done when:** a customer can purchase a plan, pay online, and receive a license automatically; renewal works the same way; the checkout handles Bangladesh payment methods (bKash, Nagad, bank transfer).
spec [0043](../specs/0043-pricing-checkout-and-device-management/index.md) ([0043a](../specs/0043-pricing-checkout-and-device-management/0043-pricing-and-checkout.md))
- [x] Design it (spec): `/architect pricing & checkout`
- [x] Build it: `/develop pricing & checkout`
  - [x] Seed 4 tier prices catalog (3m at 1,500 BDT/mo, 6m at 1,200 BDT/mo, 12m at 1,000 BDT/mo, lifetime at 50,000 BDT), Vendor Admin Plans & Pricing Studio editor, and GET /v1/pricing/catalog, covers AC-1
  - [x] Checkout sessions, pro rata upgrade calculations, and guided bank transfer submission with admin review queue, covers AC-2, AC-5
  - [x] Trait based PaymentGatewayProvider (sslcommerz, bkash_direct, nagad_direct, sandbox) and HMAC-SHA256 webhook verification with mismatch audit logging, covers AC-3
  - [x] Transactional payment, INV-YYYY-NNNNN invoice, pairing code, and isolated Ed25519 license fulfillment pipeline (including lifetime and late webhooks), covers AC-4
  - [x] Landing pages 4 tier pricing grid and interactive Checkout Modal with live receipt card, covers AC-10
- [x] Verify it: `/check verify pricing & checkout`
- [x] Test it: `/test pricing & checkout`
- [ ] Review it (fresh model): `/check review pricing & checkout`
- [x] Document it: `/document pricing & checkout`

### 68. Device management · in-progress · GA
View, deactivate, and replace licensed PCs through the web portal. The owner can see which devices are active, deactivate a lost or replaced PC, and transfer the license slot to a new machine.
**Done when:** the portal shows all linked devices with last seen time; deactivation triggers the desktop to enter read only mode on next refresh; replacing a device releases the old slot and activates the new one.
spec [0043](../specs/0043-pricing-checkout-and-device-management/index.md) ([0043b](../specs/0043-pricing-checkout-and-device-management/0043-device-management.md))
- [x] Design it (spec): `/architect device management`
- [x] Build it: `/develop device management`
  - [x] Extended installations schema, device_management_events table, and portal device telemetry listing with label renaming, covers AC-6, AC-10
  - [x] Step up OTP verification challenge and portal device deactivation with pairing code issue, covers AC-7
  - [x] Desktop Migration 0023, RefreshStatus::Deactivated handling, and immediate deactivated_read_only dispatcher enforcement, covers AC-8
  - [x] One step online pairing and offline REQ device replacement with 3 transfer per year cap and support reset request, covers AC-9, AC-10
- [x] Verify it: `/check verify device management`
- [x] Test it: `/test device management`
- [ ] Review it (fresh model): `/check review device management`
- [x] Document it: `/document device management`
