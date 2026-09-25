-- Migration 0021: Desktop Licensing and Entitlements
-- Compliant with Spec 0037

CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    license_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    installation_id TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('trial', 'local', 'local_pro')),
    kind TEXT NOT NULL CHECK (kind IN ('trial', 'paid', 'extension', 'trial_conversion')),
    serial INTEGER NOT NULL DEFAULT 0,
    paid_through TEXT NOT NULL,
    grace_days INTEGER NOT NULL DEFAULT 14,
    valid_until TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    limits_json TEXT NOT NULL DEFAULT '{}',
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    key_id TEXT NOT NULL,
    raw_payload TEXT NOT NULL,
    signature TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    moved_first_detected_at TEXT,
    created_at TEXT NOT NULL,
    imported_by_user_id TEXT REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_licenses_active ON licenses(is_active) WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_licenses_serial ON licenses(license_id, serial DESC);

-- Active license view for rapid single row retrieval
CREATE VIEW IF NOT EXISTS active_license AS
SELECT * FROM licenses WHERE is_active = 1 LIMIT 1;

-- Immutability trigger: licenses table is append only, records cannot be deleted
CREATE TRIGGER IF NOT EXISTS prevent_delete_licenses
BEFORE DELETE ON licenses
BEGIN
    SELECT RAISE(ABORT, 'Historical license records cannot be deleted');
END;
