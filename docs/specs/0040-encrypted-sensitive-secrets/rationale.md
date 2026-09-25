# 0040. Encrypted Sensitive Secrets: Rationale

## Key Decisions

### 1. Machine-Scope DPAPI (`CRYPTPROTECT_LOCAL_MACHINE`) for Provider Credentials
Provider credentials (SMS gateway API keys, SMTP passwords, AI tokens) belong to the coaching center workstation rather than a specific Windows OS login profile, allowing different Windows user accounts on the same coaching center PC (if used) to run the Coachify service while rendering the SQLite `.db` file unreadable if exfiltrated or copied to an external machine.

### 2. Automatic `config_json` Sanitization on Write
Even if a client payload inadvertently embeds `"api_key"` or `"password"` inside the freeform `config_json` object, `comms::save_provider_config` intercepts sensitive property names, migrates their values into DPAPI-protected `secret_ref`, and replaces the JSON property with `"[REDACTED_TO_DPAPI]"`. Defense in depth guarantees no plaintext secret ever touches disk.
