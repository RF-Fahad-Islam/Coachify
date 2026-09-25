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
- [ ] Verify it: `/check verify vendor admin & license issuing`
- [ ] Test it: `/test vendor admin & license issuing`
- [ ] Review it (fresh model): `/check review vendor admin & license issuing`
- [ ] Document it: `/document vendor admin & license issuing`

### 64. Activation & refresh service · in-progress · GA
Small versioned endpoints for activation, daily refresh (renewals, moves, revocations), and update metadata, sending only installation ID, license ID, app version, and active student count. Includes automatic desktop activation (linked Coachify desktop refreshes and activates automatically after purchase).
**Done when:** a renewal recorded in admin reaches an online install within a day or on "Check now"; moving a license to a new PC works; a week long outage changes nothing on any customer PC.
spec [0041](../specs/0041-vendor-admin-and-activation-service/index.md) ([0041b](../specs/0041-vendor-admin-and-activation-service/0041-activation-and-refresh-service.md)) · code in `crates/cloud_api/src/lib.rs`, `crates/core/src/licensing.rs`, `ui/src/components/settings/license-settings-card.tsx`
- [x] Design it (spec): `/architect activation & refresh service`
- [x] Build it: `/develop activation & refresh service`
  - [x] Versioned `/v1/activate`, `/v1/refresh`, and `/v1/updates/{channel}` endpoints in `crates/cloud_api` with 6 digit pairing and 3 per year self service PC transfer cap, covers AC-6, AC-7, AC-8, AC-11
  - [x] Desktop Migration 0022 (`license_refresh_state`) and synchronous `ureq` / injectable `RefreshTransport` client (`license.activate_online`, `license.confirm_transfer`, `license.refresh_now`, 24h auto check) in `crates/core/src/licensing.rs`, covers AC-6, AC-7, AC-8, AC-9, AC-10
  - [x] Desktop License Settings online activation bar, pairing input, Check Now button, and transfer confirmation modal styled with `landing pages/` primitives plus outage resilience tests, covers AC-6, AC-7, AC-8, AC-10, AC-12
- [ ] Verify it: `/check verify activation & refresh service`
- [ ] Test it: `/test activation & refresh service`
- [ ] Review it (fresh model): `/check review activation & refresh service`
- [ ] Document it: `/document activation & refresh service`

### 65. Website, downloads & customer portal · needs a decision
Marketing landing page (story driven, focused on demo download and sales conversion), download center (installer, release/version information, demo download), support/documentation (help articles, installation guides, support access), and a minimal portal: license status and download, installers, vendor invoices, installations list.
**Done when:** a customer logs in, downloads the installer and their license file, and sees their invoices; the privacy page lists exactly what the app sends; the marketing page tells the Coachify story and converts visitors to demo downloads.
- [ ] Design it (spec): `/architect website, downloads & customer portal`

### 66. Code signing & release channels · needs a decision · GA
Windows code signing from CI through a hardware or cloud backed certificate, updater key custody and backups, internal, beta, and stable channels, and a blocked versions list.
**Done when:** a tagged release is signed and published to internal only after all required suites pass and a person approves; promotion to stable copies metadata without rebuilding; a recovery test of both keys is recorded.
- [ ] Design it (spec): `/architect code signing & release channels`

### 67. Pricing & checkout · needs a decision · GA
Purchase or renew Coachify directly online. Payment gateway integration for the Coachify website. Automatic license generation after verified payment.
**Done when:** a customer can purchase a plan, pay online, and receive a license automatically; renewal works the same way; the checkout handles Bangladesh payment methods (bKash, Nagad, bank transfer).
- [ ] Design it (spec): `/architect pricing & checkout`

### 68. Device management · needs a decision · GA
View, deactivate, and replace licensed PCs through the web portal. The owner can see which devices are active, deactivate a lost or replaced PC, and transfer the license slot to a new machine.
**Done when:** the portal shows all linked devices with last seen time; deactivation triggers the desktop to enter read only mode on next refresh; replacing a device releases the old slot and activates the new one.
- [ ] Design it (spec): `/architect device management`
