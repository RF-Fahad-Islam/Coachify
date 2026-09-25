-- Migration 0022: Online Activation, Daily License Refresh State, and Vendor Admin Sandbox
-- Compliant with Spec 0041 (Features 63 and 64)

CREATE TABLE IF NOT EXISTS license_refresh_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    activation_key TEXT,
    service_base_url TEXT NOT NULL DEFAULT 'https://api.coachify.app',
    last_refresh_at TEXT,
    last_refresh_result TEXT NOT NULL DEFAULT 'never' CHECK (
        last_refresh_result IN ('never', 'ok', 'updated', 'no_change', 'moved', 'revoked', 'offline_deferred')
    ),
    last_error TEXT,
    last_refresh_error TEXT,
    latest_known_serial INTEGER NOT NULL DEFAULT 1,
    transfer_count_12m INTEGER NOT NULL DEFAULT 0,
    revoked_at TEXT,
    revocation_reason TEXT,
    revocation_blob TEXT,
    signed_revocation_blob TEXT,
    updated_at TEXT NOT NULL DEFAULT '2026-09-25T00:00:00Z'
);

INSERT OR IGNORE INTO license_refresh_state (
    id, activation_key, service_base_url, last_refresh_at, last_refresh_result, last_error, last_refresh_error, latest_known_serial, transfer_count_12m, revoked_at, revocation_reason, revocation_blob, signed_revocation_blob, updated_at
) VALUES (
    1, NULL, 'https://api.coachify.app', NULL, 'never', NULL, NULL, 1, 0, NULL, NULL, NULL, NULL, '2026-09-25T00:00:00Z'
);
