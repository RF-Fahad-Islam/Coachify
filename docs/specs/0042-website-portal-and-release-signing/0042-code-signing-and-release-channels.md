# 0042b. Code Signing, Release Channels, and Key Recovery Custody (Feature 66)

**Date**: 2026-09-25

## Summary

Feature 66 establishes Coachify's production Windows code signing, Tauri updater Ed25519 signing, zero rebuild release channel promotion (`internal`, `beta`, `stable`), blocked versions list, and dual key recovery verification across `crates/license_signer`, `crates/cloud_api`, `ui/src/components/vendor-admin/vendor-admin-view.tsx`, and `.github/workflows/release.yml`. Tagged releases build and sign once in CI after all Rust and UI verification suites and a human approval gate pass, publish to `internal`, promote to `beta` or `stable` by copying signed metadata without recompiling, and verify key recovery for both Windows Authenticode and Ed25519 updater keys into an append only audit table (`key_recovery_drills`).

## Requirements

**User stories**:
* As a release engineer, I want tagged builds (`v*`) to sign the Windows NSIS installer with Authenticode and the Tauri updater Ed25519 key (`u2026a`) and publish to `internal` only after all verification suites and a human reviewer approval gate pass.
* As an operations admin (`super_admin` or `ops_admin`), I want to promote a release from `internal` to `beta` or `stable` without rebuilding the binary, and block defective versions with step up TOTP so no desktop upgrades into them.
* As a founder preparing for `1.0`, I want to execute and record a recovery test of both the Windows code signing certificate and the Ed25519 updater and license keys in an append only table (`key_recovery_drills`).

**Acceptance criteria**:
* **AC-5**: CI gated Windows Authenticode and Tauri Ed25519 signing to `internal` (`.github/workflows/release.yml` runs full Rust and UI test suites, pauses at `release-internal` human approval gate, signs with Authenticode and `crates/license_signer` updater key `u2026a`, and publishes to `internal`).
* **AC-6**: Zero rebuild channel promotion (`POST /v1/releases/promote` copies the immutable `release_id`, `sha256`, `artifact_url`, `authenticode_thumbprint`, and `updater_signature` from `internal` to `beta` or `stable` in `release_channel_heads` without rebuilding, requiring step up TOTP for `stable`).
* **AC-7**: Held and blocked version enforcement (refuses promoting `held` or `blocked` builds with `RELEASE_NOT_PROMOTABLE`, omits them as `latest` on `GET /v1/updates/{channel}`, and lists `blocked` versions in `blocked_versions`).
* **AC-8**: Dual key custody and recorded recovery drill (`POST /v1/releases/recovery_drill` and `crates/license_signer::verify_key_recovery` verify backup decryption and test signature generation for both `windows_authenticode` and `tauri_updater_ed25519` plus `license_ed25519`, recording an immutable row in `key_recovery_drills`).
* **AC-9**: Vendor Admin Release Channels and Key Recovery Drill UI controls in `ui/src/components/vendor-admin/vendor-admin-view.tsx` styled with `design.md` and `Landing pages/` primitives.

## Decision

**Chosen option**: Isolate Tauri updater Ed25519 signing (`u2026a`, `u2026b`) and backup verification inside `crates/license_signer`, track per channel heads (`release_channel_heads`) and append only `key_recovery_drills` in `crates/cloud_api`, and gate `.github/workflows/release.yml` with full verification checks, human environment approval, and cloud backed Windows Authenticode signing.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`)

## Rationale

Promoting a release by updating `release_channel_heads` to reference the existing `releases` row guarantees that the binary hash (`sha256`), Windows Authenticode signature (`authenticode_thumbprint`), and Tauri updater signature (`updater_signature`) shipped to `stable` are identical to the artifact tested in `internal` and `beta`. Keeping the updater Ed25519 private key inside `crates/license_signer` alongside the license signing key preserves a single hardened cryptographic boundary with an append only `signer_audit_log` and verifiable recovery drills.

## Feature design

**Data model sketch**:
* `releases` (extended columns): `size_bytes INTEGER NOT NULL DEFAULT 0`, `authenticode_thumbprint TEXT NOT NULL DEFAULT ''`, `updater_key_id TEXT NOT NULL DEFAULT 'u2026a'`.
* `release_channel_heads`: `channel TEXT PRIMARY KEY CHECK (channel IN ('internal', 'beta', 'stable'))`, `release_id TEXT NOT NULL REFERENCES releases(id)`, `version TEXT NOT NULL`, `promoted_from_channel TEXT`, `promoted_by TEXT NOT NULL REFERENCES admin_users(id)`, `promoted_at TEXT NOT NULL`.
* `key_recovery_drills`: `id TEXT PRIMARY KEY`, `key_type TEXT NOT NULL CHECK (key_type IN ('windows_authenticode', 'tauri_updater_ed25519', 'license_ed25519'))`, `key_id TEXT NOT NULL`, `backup_source TEXT NOT NULL`, `backup_sha256 TEXT NOT NULL`, `test_artifact_sha256 TEXT NOT NULL`, `signature_verified INTEGER NOT NULL CHECK (signature_verified = 1)`, `notes TEXT NOT NULL`, `performed_by TEXT NOT NULL REFERENCES admin_users(id)`, `performed_at TEXT NOT NULL` (append only via `BEFORE UPDATE` and `BEFORE DELETE` triggers).

**State transitions**:
* `ci_tagged` -> (`release-internal` approval + Authenticode + `u2026a` Ed25519 sign) -> `published_internal` -> (`promote` to `beta`) -> `promoted_beta` -> (`promote` to `stable` with step up TOTP) -> `promoted_stable`.
* `live` -> (`held` or `blocked`) -> excluded from promotion and `GET /v1/updates/{channel}` `latest` (`blocked` versions appended to `blocked_versions`).

**API surface**:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `/v1/releases/publish` | `POST` | `admin_user_id`, `version`, `notes`, `artifact_url`, `sha256`, `size_bytes`, `authenticode_thumbprint`, `min_upgrade_from` | `release`, `channel_head` (`internal`) | `ops_admin` / `super_admin` | `DUPLICATE_VERSION`, `INVALID_AUTHENTICODE_THUMBPRINT` |
| `/v1/releases/promote` | `POST` | `admin_user_id`, `version`, `from_channel`, `to_channel`, `step_up_totp` | `channel_head`, `release` | `ops_admin` / `super_admin` (+ TOTP for `stable`) | `RELEASE_NOT_PROMOTABLE`, `CHANNEL_HEAD_MISMATCH`, `STEP_UP_TOTP_REQUIRED` |
| `/v1/releases/recovery_drill` | `POST` | `admin_user_id`, `key_type`, `key_id`, `backup_source`, `encrypted_backup_blob`, `passphrase`, `notes`, `step_up_totp` | `drill: KeyRecoveryDrillRecord` | `super_admin` + step up TOTP | `RECOVERY_SIGNATURE_MISMATCH`, `STEP_UP_TOTP_REQUIRED` |
| `/v1/updates/{channel}` | `GET` | `channel` (`internal`, `beta`, `stable`) | `channel`, `latest`, `blocked_versions` | Public | `INVALID_CHANNEL` |

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| `POST /v1/releases/publish` | `updater_signature` and `updater_key_id` | `crates/license_signer::SignerService::sign_updater_release` over canonical release JSON |
| `POST /v1/releases/promote` | Promoted channel's `release_id`, `sha256`, `authenticode_thumbprint`, `updater_signature` | Copied from `releases` row pointed to by `release_channel_heads[from_channel]` |
| `GET /v1/updates/{channel}` | `latest` manifest and `blocked_versions` | `release_channel_heads[channel]` joined with `releases` (`status = 'live'`) plus `SELECT version FROM releases WHERE status = 'blocked'` |
| `POST /v1/releases/recovery_drill` | `backup_sha256`, `test_artifact_sha256`, `signature_verified` | Computed and verified by `crates/license_signer::SignerService::verify_key_recovery` |

**Key invariants**:
* Zero rebuild promotion: `sha256`, `authenticode_thumbprint`, and `updater_signature` never change when promoting from `internal` to `beta` or `stable`.
* `held` and `blocked` releases cannot be promoted and never appear as `latest` on `GET /v1/updates/{channel}`.
* `key_recovery_drills` is append only and requires verified recovery tests of both Windows Authenticode and Ed25519 updater keys.

**Security model**:
* `super_admin` or `ops_admin` required for release publishing and promotion; step up RFC 6238 TOTP required for promoting to `stable`, blocking a version, or recording a key recovery drill.

**Configuration required**:
* `TAURI_UPDATER_PRIVATE_SEED_U2026A`: Isolated Ed25519 seed in `crates/license_signer`.
* `AZURE_TRUSTED_SIGNING_ENDPOINT` / `AZURE_TRUSTED_SIGNING_ACCOUNT` / `AZURE_TRUSTED_SIGNING_PROFILE`: Cloud backed Authenticode signing in `.github/workflows/release.yml`.

**Critical test scenarios**:
* Happy path: Publish `v1.1.0` to `internal`, promote to `beta` and `stable` with step up TOTP, and verify `GET /v1/updates/stable` returns identical `sha256` and `updater_signature` without rebuild, verifies **AC-5**, **AC-6**.
* Failure case: Mark `v1.1.0` as `blocked` with step up TOTP; verify promotion fails with `RELEASE_NOT_PROMOTABLE` and `GET /v1/updates/stable` lists `1.1.0` in `blocked_versions`, verifies **AC-7**.
* Key recovery drill: Run `POST /v1/releases/recovery_drill` for `windows_authenticode` and `tauri_updater_ed25519`, verify `signature_verified = 1` in `key_recovery_drills`, and verify `UPDATE`/`DELETE` abort via trigger, verifies **AC-8**.

## Build plan

1. Add `u2026a` / `u2026b` updater Ed25519 signing (`sign_updater_release`) and backup recovery verification (`verify_key_recovery`) to `crates/license_signer`, satisfies **AC-5**, **AC-8**
2. Add `release_channel_heads` and `key_recovery_drills` tables plus `publish_release_to_internal`, `promote_release_channel`, `record_key_recovery_drill`, and channel pointer resolution in `crates/cloud_api`, satisfies **AC-5**, **AC-6**, **AC-7**, **AC-8**
3. Add channel promotion (`internal` -> `beta` -> `stable`) and Key Recovery Drill studio controls to `ui/src/components/vendor-admin/vendor-admin-view.tsx` and update `.github/workflows/release.yml` with the `release-internal` approval gate and signing steps, satisfies **AC-5**, **AC-6**, **AC-8**, **AC-9**

## Consequences

**Positive**:
* Eliminates rebuild drift across `internal`, `beta`, and `stable` channels and proves key recovery readiness before `1.0`.

**Negative / tradeoffs**:
* Requires human approval on `release-internal` and step up TOTP on `stable` promotion.
