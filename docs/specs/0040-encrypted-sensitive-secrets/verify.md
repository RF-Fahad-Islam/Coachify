# 0040. Encrypted Sensitive Secrets: Verification Plan

## Automated Verification

```bash
cargo test -p coachify_core --test encrypted_secrets_integration
```

## Verification Matrix

| Criterion | Test | Expected Outcome |
|---|---|---|
| AC-1 | `test_provider_and_ai_secrets_encrypted_at_rest_and_config_json_sanitized` | Raw SQL inspection confirms `secret_ref` and `token_dpapi_blob` never contain plaintext secret strings, and `config_json` strips embedded `api_key`/`password` fields |
| AC-2 | `test_foreign_machine_cannot_decrypt_secrets` | Enabling `set_mock_dpapi_failure(true)` blocks decryption of both comms and AI secrets and reports `foreign_machine` in `security.get_secrets_health` |
| AC-3 | `test_diagnostics_bundle_contains_zero_secrets` | Exporting diagnostics ZIP with active secrets and log leaks produces zero occurrences of any secret string across every file in the ZIP |
| AC-4 | `test_log_and_ai_prompt_secret_redaction` | `redact_pii` and `PrivacyRedactionEngine::scrub_prompt` scrub `sk-*`, `Bearer`, `CKREC-*`, and `api_key=`/`password=` patterns |
