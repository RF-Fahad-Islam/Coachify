# 0040. Encrypted Sensitive Secrets

> Status: Ratified
> Feature: 53 (Slice 9 — Printing & Localization · GA)
> Date: 2026-09-25

## Overview

Coaching centers store sensitive third-party credentials in Coachify: SMS gateway API keys, SMTP email passwords, WhatsApp companion tokens, and Cloud AI / OpenRouter API keys. Feature 53 ensures that **all sensitive credentials are encrypted at rest through the native Windows Data Protection API (`DPAPI` — `CryptProtectData` / `CryptUnprotectData` with `CRYPTPROTECT_LOCAL_MACHINE`)**, cannot be read if the SQLite database file is copied to another machine, are automatically stripped from `config_json` if accidentally passed there, and are scrubbed from all logs, error messages, AI prompts, and diagnostics export bundles.

## Security Architecture

1. **DPAPI Encryption at Rest (`crates/core/src/crypto.rs`)**:
   - `protect_secret_dpapi(secret: &str) -> Result<String, CryptoError>` wraps raw credential bytes with `CryptProtectData` (`CRYPTPROTECT_LOCAL_MACHINE`) and encodes as Base64 prefixed with `DPAPI_V1:`.
   - `unprotect_secret_dpapi(cipher_b64: &str) -> Result<String, CryptoError>` decodes and unwraps via `CryptUnprotectData`, failing with `CryptoError::DpapiUnprotectFailed` if moved to a foreign machine.
2. **Automatic Config JSON Sanitization (`crates/core/src/comms.rs`)**:
   - When saving provider configurations (`comms.save_provider`), `sanitize_provider_config_json` inspects `config_json` for any secret-bearing keys (`api_key`, `secret`, `password`, `token`, `auth_token`, `smtp_password`, `private_key`). If found and no explicit `secret` parameter was provided, it extracts and encrypts the secret into `secret_ref` via DPAPI and replaces the JSON value with `"[REDACTED_TO_DPAPI]"`, ensuring plaintext secrets never land in `config_json`.
   - Runtime provider dispatch (`decrypt_provider_secret`) decrypts `secret_ref` exclusively in memory at dispatch time and fails safely with `CommsError::Crypto` (`provider_secret_foreign_machine`) if the database was copied to another PC without re-entering credentials.
3. **Secrets Health Audit Endpoint (`security.get_secrets_health`)**:
   - Inspects all stored credentials (`provider_configs.secret_ref`, `ai_credit_wallets.token_dpapi_blob`, `ai_credit_wallets.openrouter_key_dpapi_blob`) and verifies whether each is `healthy` (decryptable on current machine), `foreign_machine` (cannot be decrypted on this machine), or `unset`, plus confirms `zero_plaintext_leaks: true` across `settings` and `provider_configs.config_json`.
4. **Universal Secret & PII Redaction (`crates/core/src/diagnostics.rs` & `crates/core/src/ai/privacy.rs`)**:
   - `diagnostics::redact_pii` (and `redact_secrets`) scrubs:
     - Bangladeshi phone numbers (`01XXXXXXXXX`, `8801XXXXXXXXX`)
     - API keys (`sk-...`, `pk-...`, `rk-...`)
     - Bearer tokens (`Bearer ...`)
     - Owner recovery keys (`CKREC-...`)
     - Key-value credential patterns (`api_key=...`, `password=...`, `secret=...`, `token=...`, `"secret_ref": "..."`)
     - DPAPI envelopes (`DPAPI_V1:...`, `MOCK_DPAPI_WRAP:...`)
   - `diagnostics::export_diagnostics_bundle` includes a redacted `provider_configs` summary where `secret_ref` is always `"[REDACTED]"` and `config_json` is recursively scrubbed.

## Acceptance Criteria

- **AC-1 (DPAPI Encryption at Rest)**: All SMS, email, WhatsApp, and AI gateway secrets are stored encrypted using Windows DPAPI (`CryptProtectData`); zero plaintext credentials exist in `provider_configs`, `ai_credit_wallets`, or `settings`.
- **AC-2 (Foreign Machine Protection)**: A database copy opened on another machine (simulated via `set_mock_dpapi_failure(true)`) cannot decrypt `provider_configs.secret_ref` or `ai_credit_wallets` blobs, returning a structured error (`DpapiUnprotectFailed`) and reporting `foreign_machine` in `security.get_secrets_health`.
- **AC-3 (Diagnostics Bundle Scrubbing)**: Exporting a diagnostics bundle (`diagnostics.export_bundle`) produces zero occurrences of any configured API key, password, recovery key, or token across all files in the ZIP archive (`settings.json`, `recent_logs.txt`, `info.json`).
- **AC-4 (Log & AI Prompt Redaction)**: `redact_pii` and `PrivacyRedactionEngine::scrub_prompt` redact `sk-*` keys, `Bearer` tokens, `CKREC-*` recovery keys, and `password=/api_key=/token=` patterns in logs and outgoing AI prompts.

## Build Plan

1. Add `protect_secret_dpapi`, `unprotect_secret_dpapi`, and `SecretsHealthReport` (`check_secrets_health`) in `crates/core/src/crypto.rs`.
2. Harden `crates/core/src/comms.rs` with `sanitize_provider_config_json` and `decrypt_provider_secret` using DPAPI.
3. Upgrade `crates/core/src/diagnostics.rs` and `crates/core/src/ai/privacy.rs` to scrub all secret patterns (`sk-*`, `Bearer`, `CKREC-*`, `api_key/password/secret/token` key-values, DPAPI blobs) and recursively redact JSON settings and provider configs in diagnostics bundles.
4. Register `security.get_secrets_health` in `crates/core/src/dispatcher.rs` and export TypeScript bindings (`cargo run -p xtask export-types`).
5. Add comprehensive integration test suite `crates/core/tests/encrypted_secrets_integration.rs` verifying AC-1 through AC-4.
