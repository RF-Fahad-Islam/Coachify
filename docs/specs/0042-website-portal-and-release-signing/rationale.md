# 0042. Decision Record and Rationale: Website, Customer Portal, Code Signing, and Release Channels

**Date**: 2026-09-25

## Context

Coachify (NF Management Local) serves coaching centers, academies, and schools across Bangladesh that run their daily student admissions, fee collections, attendance, and exam grading on local Windows PCs. Specs `0037` and `0041` established offline Ed25519 license verification (`crates/core/src/licensing.rs`), the isolated license signer (`crates/license_signer`), the commercial database and activation service (`crates/cloud_api`), and the bilingual story driven marketing web app (`Landing pages/`). However, two load bearing capabilities remain before general availability (`1.0`).

First, paying coaching center owners currently rely on manual WhatsApp support whenever they need to redownload their `.coachifylicense` file after reinstalling Windows, grab a verified copy of the installer, print their paid `INV-YYYY-NNNNN` vendor invoices for institutional accounting, or check which PCs are bound to their license. Furthermore, prospective institutions evaluating a desktop program want verifiable proof on a public `/privacy` page of the exact fields the desktop application sends during online activation, daily refresh, and update checks, confirming that student records and financial ledgers never leave their office PC.

Second, shipping `1.0` to hundreds of coaching centers requires a trustworthy Windows code signing and release channel pipeline. Unsigned or inconsistently rebuilt installers trigger Windows SmartScreen blocks, risk binary drift between internal testing and stable production, and expose the fleet to supply chain or broken update incidents if a bad version cannot be held or blocked immediately. Equally critical, if either the Windows Authenticode certificate or the Tauri Ed25519 updater and license private keys were lost without a proven recovery path, existing customer installations could never receive signed updates or license renewals again.

## Options considered

### Option 1: Integrated `crates/cloud_api` Portal and Channel Promotion Engine, Isolated `crates/license_signer` Updater and License Key Custody, `Landing pages/` Customer Portal and Privacy Routes, and Gated GitHub Actions Authenticode + Ed25519 Pipeline

We extend `crates/cloud_api` with `customer_portal_credentials`, `customer_portal_sessions`, `release_channel_heads`, and append only `key_recovery_drills`, add `/v1/portal/*` and `/v1/releases/*` endpoints, house Tauri updater Ed25519 private keys (`u2026a`, `u2026b`) inside `crates/license_signer` alongside the license keys, build `/portal` and `/privacy` plus live channel metadata on `/downloads` inside `Landing pages/`, and configure `.github/workflows/release.yml` to sign once into `internal` after all tests and human approval pass and promote to `beta` or `stable` by copying metadata without rebuilding.

**Pros**:
* Shares the exact commercial tables (`customers`, `subscriptions`, `licenses`, `vendor_invoices`, `installations`, `releases`) already built in `crates/cloud_api` under Spec `0041`, preventing duplicate data synchronization or drift.
* Guarantees binary identity across `internal`, `beta`, and `stable` channels because promotion updates `release_channel_heads` to point to the immutable `releases` record (`sha256`, `authenticode_thumbprint`, `updater_signature`) without recompiling.
* Keeps all Ed25519 private keys (both `.coachifylicense` signing and Tauri updater signing) inside `crates/license_signer` with `signer_audit_log` and executable `key_recovery_drills` verification.

**Cons**:
* Requires a two step portal login (password plus 6 digit OTP) and step up TOTP for `stable` promotions, adding intentional confirmation friction for owners and release admins.

### Option 2: Separate Third Party Customer Billing Portal and Per Channel CI Rebuilds

We integrate an external SaaS customer portal for invoice and license file hosting while configuring GitHub Actions to run separate compile and sign jobs for `internal`, `beta`, and `stable` git branches.

**Pros**:
* Offloads session management for the customer portal to an external web service.

**Cons**:
* Recompiling the Rust and React codebase when promoting from `internal` to `beta` or `stable` produces a different binary hash (`sha256`) and new signature, meaning the binary shipped to `stable` is not the exact binary verified in `internal`.
* Splits commercial source of truth between `crates/cloud_api` (where manual bKash and Nagad payments and offline `REQ` codes live) and an external SaaS database.

### Option 3: Static Website Only with Manual WhatsApp License Delivery and Local USB Token Builds

We keep `Landing pages/` purely static, send `.coachifylicense` files and PDF invoices exclusively through manual WhatsApp messages from support staff, and build and sign releases manually on a founder's local Windows workstation using a physical USB token.

**Pros**:
* Zero new portal authentication or CI signing configuration required.

**Cons**:
* Creates a permanent support bottleneck every time a coaching center formats a PC or requests a duplicate `INV-YYYY-NNNNN` invoice.
* Local workstation builds lack reproducible CI quality gates (`cargo test`, `clippy`, `xtask`, `vitest`) and create a single point of failure if the local build PC fails.

## Rationale

Option 1 is the right architecture because it directly resolves both operational and cryptographic forces in Context using the infrastructure Coachify already runs. Because `crates/cloud_api` already stores `customers`, `subscriptions`, `vendor_invoices`, `licenses`, `installations`, and `releases` from Spec `0041`, adding `customer_portal_credentials` and `customer_portal_sessions` directly inside `crates/cloud_api` allows `/portal` in `Landing pages/` to serve live `.coachifylicense` blobs, `INV-YYYY-NNNNN` invoices, and linked PC installations with strict `customer_id` isolation and zero data duplication.

For Feature 66, building and signing the NSIS installer once on tag push in `.github/workflows/release.yml` after every Rust and UI check and the `release-internal` human approval gate pass, and then promoting `internal` to `beta` and `stable` via `release_channel_heads` in `crates/cloud_api`, guarantees that the exact `sha256` binary and `updater_signature` tested internally is what reaches customer PCs. Housing the Tauri updater Ed25519 keys (`u2026a`, `u2026b`) inside `crates/license_signer` and enforcing an append only `key_recovery_drills` verification record ensures that both Windows Authenticode and updater key backups are cryptographically tested before `1.0`.
