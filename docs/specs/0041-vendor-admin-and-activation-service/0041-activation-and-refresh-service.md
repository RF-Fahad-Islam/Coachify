# 0041b. Activation and Refresh Service and Desktop Online Client (Feature 64)

**Date**: 2026-09-25

## Summary

Feature 64 delivers the versioned desktop facing cloud endpoints (`POST /v1/activate`, `POST /v1/refresh`, `GET /v1/updates/{channel}`) in `crates/cloud_api` and the desktop online activation and daily background refresh client in `crates/core/src/licensing.rs` and `ui/src/components/settings/license-settings-card.tsx`. Connected Coachify desktop installations can activate online using a license key or 6 digit purchase pairing code, automatically pick up subscription renewals recorded in Vendor Admin within 24 hours (or immediately via Check Now), and transfer a license to a replacement PC (up to 3 self service transfers per year). All network calls are strictly non blocking with a 5 second timeout so a week long outage changes nothing on any customer PC.

## Requirements

Covers umbrella acceptance criteria **AC-6**, **AC-7**, **AC-8**, **AC-9** (desktop revocation verification side), **AC-10**, **AC-11** (endpoint side), and **AC-12** (Desktop Settings UI):

* **AC-6**: Online activation via `POST /v1/activate` (and desktop RPC `license.activate_online`) using either `license_id` + `activation_key` or a short lived 6 digit `pairing_code`, automatically importing and activating the returned `LicenseEnvelope`.
* **AC-7**: Self service PC replacement via `confirm_transfer = true` (and desktop RPC `license.confirm_transfer`), rebinding the license to the new `installation_id`, incrementing `current_serial`, marking the old installation as `moved`, and enforcing a cap of 3 self service transfers per 12 months.
* **AC-8**: Daily background refresh (`POST /v1/refresh` and desktop RPC `license.refresh_now`) sending only `installation_id`, `license_id`, `current_serial`, `app_version`, and `active_students`, and applying `updated`, `no_change`, `moved`, or `revoked` states.
* **AC-9**: Cryptographic verification of `SignedRevocationEnvelope` on the desktop, transitioning the dispatcher to `suspended_read_only` while preserving read, report, export, and backup access.
* **AC-10**: Week long outage immunity: network errors or 5xx responses record `last_refresh_result = 'offline_deferred'` in `license_refresh_state` (Migration 0022) and never alter local license validity or interrupt counter work.
* **AC-11**: `GET /v1/updates/{channel}` returns live release metadata and `blocked_versions` so the desktop updater skips held or blocked releases.

## Rationale

Desktop coaching centers frequently operate on unstable broadband or mobile hotspots. By using a synchronous transport trait (`RefreshTransport`) with a default 5 second `ureq` implementation and storing refresh metadata in `license_refresh_state` (Migration 0022), `crates/core` keeps its zero async footprint while guaranteeing that network failures are completely invisible to receptionists and cashiers.

## Feature design

**Desktop to Cloud Wire Payloads (`crates/cloud_api` & `crates/core/src/licensing.rs`)**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivateEndpointRequest {
    pub license_id_or_pairing_code: String,
    pub activation_key: Option<String>,
    pub installation_id: String,
    pub app_version: String,
    pub active_students: u32,
    pub confirm_transfer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivateEndpointResponse {
    pub outcome: String, // "activated" | "already_bound_transfer_available"
    pub license_id: String,
    pub signed_license_blob: Option<String>,
    pub previous_installation_id: Option<String>,
    pub previous_last_seen_at: Option<String>,
    pub transfers_remaining_12m: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefreshEndpointRequest {
    pub license_id: String,
    pub installation_id: String,
    pub current_serial: i64,
    pub app_version: String,
    pub active_students: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RefreshEndpointResponse {
    pub outcome: String, // "updated" | "no_change" | "moved" | "revoked"
    pub latest_serial: i64,
    pub signed_license_blob: Option<String>,
    pub signed_revocation_blob: Option<String>,
    pub server_time_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateChannelResponse {
    pub channel: String,
    pub latest_live_version: Option<String>,
    pub artifact_url: Option<String>,
    pub sha256: Option<String>,
    pub updater_signature: Option<String>,
    pub min_upgrade_from: Option<String>,
    pub notes: Option<String>,
    pub blocked_versions: Vec<String>,
}
```

**Desktop License Settings UI Enhancements (`ui/src/components/settings/license-settings-card.tsx`)**:
* Styled using the `landing pages/` tokens (`Sora` headings, `Inter` body, `.tnum` counters, warm stone cards, `#f97316` ember primary `CtaButton`, and `Chip` status pills).
* **Online Activation Bar**: Allows the owner to enter either a 6 digit purchase pairing code (e.g., `849 201`) or a `LIC-XXXX-XXXX-XXXX` key + activation secret and click **Activate Online**.
* **Device Transfer Confirmation Modal**: When `ActivateEndpointResponse.outcome == "already_bound_transfer_available"`, presents a clear confirmation dialog showing the previous installation ID, last seen date, and remaining annual self service transfers (`transfers_remaining_12m`), allowing one click **Transfer License to This PC**.
* **Daily Refresh Status & Check Now Button**: Displays `last_refresh_at`, `last_refresh_result` badge (`Up to date`, `Updated`, or `Offline · Local license active`), and a **Check Now** button that calls `license.refresh_now`.
