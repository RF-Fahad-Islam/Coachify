# Commercial (Slice 7)

Back to [index](index.md). Can run in parallel once Slice 3 is done. Company paperwork, merchant accounts, and the code signing certificate start after pilots (decided 2026-09-20), so pilots run free and unsigned, and feature 36 lands just before 1.0. Features 33 to 35 belong to the separate cloud repository once it exists; move these sections there when it is created.

### 32. Desktop licensing & read only mode · needs a decision · GA
Signed license verification against embedded public keys, installation binding, serial check, trial, grace, read only mode enforced in the dispatcher, the moved license window, and the clock guard. Plans: Local, and Local Pro, which adds only LAN.
**Done when:** a tampered, foreign, or older license is refused with a reason; expired installs can still view, export, back up, and restore; a PC clock set back is caught with a friendly message; nothing ever waits on the license server.
- [ ] Design it (spec): `/architect desktop licensing & read only mode`

### 33. Vendor admin & license issuing · needs a decision · GA
Admin panel with two factor login: customers, plans, prices, subscriptions, manual payment recording, license issuing (online and from an offline request code), installations, revocations, releases, and an admin audit log. The signing key lives only in an isolated signer.
**Done when:** a non developer records a bKash payment and issues both an online and an offline license end to end; every admin action is audited; the web app never holds the private key.
- [ ] Design it (spec): `/architect vendor admin & license issuing`

### 34. Activation & refresh service · needs a decision · GA
Small versioned endpoints for activation, daily refresh (renewals, moves, revocations), and update metadata, sending only installation ID, license ID, app version, and active student count.
**Done when:** a renewal recorded in admin reaches an online install within a day or on "Check now"; moving a license to a new PC works; a week long outage changes nothing on any customer PC.
- [ ] Design it (spec): `/architect activation & refresh service`

### 35. Website, downloads & customer portal · needs a decision
Product site (features, pricing, demo, download, docs, privacy), and a minimal portal: license status and download, installers, vendor invoices, installations list.
**Done when:** a customer logs in, downloads the installer and their license file, and sees their invoices; the privacy page lists exactly what the app sends.
- [ ] Design it (spec): `/architect website, downloads & customer portal`

### 36. Code signing & release channels · needs a decision · GA
Windows code signing from CI through a hardware or cloud backed certificate, updater key custody and backups, internal, beta, and stable channels, and a blocked versions list.
**Done when:** a tagged release is signed and published to internal only after all required suites pass and a person approves; promotion to stable copies metadata without rebuilding; a recovery test of both keys is recorded.
- [ ] Design it (spec): `/architect code signing & release channels`
