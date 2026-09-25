# 0041a. Vendor Admin Panel and Isolated License Issuing (Feature 63)

**Date**: 2026-09-25

## Summary

Feature 63 delivers the Vendor Admin panel (`ui/src/components/vendor-admin/` styled with the `landing pages/` design language), the commercial backend (`crates/cloud_api`), and the isolated cryptographic signer (`crates/license_signer`). Vendor staff authenticate with Argon2id passwords and RFC 6238 TOTP two factor codes to manage customers, plans, prices, subscriptions, manual bKash/Nagad/bank/cash payments, online and offline license issuing, installations, revocations, software releases, and a SHA256 hash chained `admin_audit_log`. The Ed25519 private key lives exclusively inside `crates/license_signer`, so the web app never holds the private key.

## Requirements

Covers umbrella acceptance criteria **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, **AC-9** (admin revocation side), **AC-11** (admin release management), and **AC-12** (Vendor Admin UI):

* **AC-1**: Argon2id + RFC 6238 6 digit TOTP login for `super_admin`, `ops_admin`, and `support_admin`, plus step up TOTP re verification for revocations and release blocks.
* **AC-2**: Isolated `crates/license_signer` holding the Ed25519 private key seed (`k2026a`, `k2026b`), accepting authenticated `SignLicenseRequest` and `SignRevocationRequest` calls and logging every signature to `signer_audit_log`.
* **AC-3**: Recording a confirmed manual payment (`bkash`, `nagad`, `bank`, `cash`) automatically extends `paid_through` (from existing `paid_through` if before grace expiry, or from payment date if expired), generates invoice `INV-YYYY-NNNNN`, increments `current_serial`, signs the new `LicenseEnvelope`, and stores it for both online refresh and `.coachifylicense` download.
* **AC-4**: Offline License Issuing Studio validates 20 character `REQ-XXXX-XXXX-XXXX-XXXX` codes and `COACHIFY-REQ|v1|<installation_id>|<checksum>` QR strings, binds the `installation_id`, calls `crates/license_signer`, and records `activation_requests`.
* **AC-5**: Every admin action writes an immutable hash chained row (`prev_hash`, `hash`) to `admin_audit_log`.
* **AC-12**: The 6 tab Vendor Admin console uses the `landing pages/` design tokens (`Sora`, `Inter`, `.tnum`, `#faf7f2` cream, `#16110d` ink, `#f97316` ember orange) and UI primitives (`WindowFrame`, `CtaButton`, `SectionTag`, `Chip`, `Stat`).

## Rationale

Putting the private Ed25519 signing seed inside an isolated `crates/license_signer` service with a narrow `SignLicenseRequest` / `SignRevocationRequest` interface prevents any web vulnerability from leaking the master signing key. Coupling manual payment confirmation directly to automatic `paid_through` extension and signer invocation lets a non developer support agent handle a bKash renewal in under 15 seconds.

## Feature design

**Isolated Signer Contract (`crates/license_signer`)**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignLicenseRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_id: String,
    pub payload: LicensePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignLicenseResponse {
    pub envelope: LicenseEnvelope,
    pub signed_blob: String,
    pub payload_sha256: String,
    pub signer_audit_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignRevocationRequest {
    pub caller_service_id: String,
    pub timestamp_utc: String,
    pub hmac_signature: String,
    pub key_id: String,
    pub license_id: String,
    pub reason: String,
    pub revoked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignedRevocationPayload {
    pub license_id: String,
    pub reason: String,
    pub revoked_at: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SignedRevocationEnvelope {
    pub payload: String, // Canonical JSON of SignedRevocationPayload
    pub signature: String, // Ed25519 hex signature
}
```

**Vendor Admin UI Composition (built from `landing pages/` primitives)**:
1. **Top Header & 2FA Gate**:
   * Renders with `landing pages/` warm cream background (`#faf7f2`), `.bg-grid-warm`, ink header (`#16110d`), and `SectionTag` (`VENDOR CLOUD · ISOLATED SIGNER ONLINE`).
   * TOTP login modal verifies email, password, and 6 digit authenticator code (or one of 8 single use recovery codes).
2. **Tab 1: Customers & Subscriptions (with 1 Click bKash Payment Drawer)**:
   * Displays `Stat` cards (`Active Subscriptions`, `Monthly Revenue BDT`, `In 14 Day Grace`, `Pending Offline Requests`) using `.tnum` tabular figures.
   * Searchable table of coaching centers (`org_name`, `owner_name`, `phone`, `district`, plan `Chip`, `paid_through`, current `serial`).
   * **Record Payment & Issue License Drawer**: Select method (`bKash`, `Nagad`, `Bank Transfer`, `Cash`), period (`1`, `6`, or `12` months), amount in BDT (stored as poisha), transaction reference (e.g., `BKASH TrxID`), and optional proof attachment. Submitting records the payment, generates `INV-YYYY-NNNNN`, extends `paid_through`, calls `crates/license_signer`, and shows a success panel with **Download `.coachifylicense` for WhatsApp** and **Copy 6 Digit Pairing Code**.
3. **Tab 2: License Issuing Studio (Online & Offline Request Codes)**:
   * Input field accepting either `REQ-XXXX-XXXX-XXXX-XXXX` or `COACHIFY-REQ|v1|<installation_id>|<checksum>`.
   * Live checksum badge (`Chip tone="green"` when HMAC SHA256 matches, `Chip tone="red"` on typo).
   * One click **Sign & Download Offline License** button (`CtaButton variant="primary"`).
4. **Tab 3: Installations & Transfers**:
   * Shows every bound host PC (`installation_id`, `app_version`, `active_students`, `first_seen_at`, `last_seen_at`, `transfer_count_12m`, status `Chip`), with an **Approve / Reset Transfer Limit** action for `ops_admin` and `super_admin`.
5. **Tab 4: Revocations**:
   * Lists revoked licenses with reason, timestamp, and signed revocation blob; includes **Revoke License** modal requiring `super_admin` role and step up 6 digit TOTP code.
6. **Tab 5: Releases & Channels**:
   * Manages `internal`, `beta`, and `stable` release entries (`version`, `sha256`, `updater_signature`, `min_upgrade_from`, and status `live` / `held` / `blocked`), requiring step up TOTP to mark a release `blocked`.
7. **Tab 6: Tamper Evident Admin Audit Log**:
   * Displays the SHA256 hash chain (`seq`, `at`, `admin_user`, `action`, `entity_id`, `prev_hash`, `hash`) and a **Verify Audit Chain** integrity badge alongside the isolated `signer_audit_log` counter.
