-- Migration 0009: Migration engine tracking ledger with immutability triggers and settings.view permission

CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    execution_time_ms INTEGER NOT NULL
);

CREATE TRIGGER IF NOT EXISTS trg_migrations_no_update
BEFORE UPDATE ON _migrations
BEGIN
    SELECT RAISE(ABORT, 'Historical migration records are immutable and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS trg_migrations_no_delete
BEFORE DELETE ON _migrations
BEGIN
    SELECT RAISE(ABORT, 'Historical migration records are immutable and cannot be deleted');
END;

-- Seed settings.view permission
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('settings.view', 'settings', 'View system settings, diagnostics, and database migration status');

-- Map settings.view permission to Owner and Admin system roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'settings.view' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (9, datetime('now'));
