-- Migration 0013: Backup infrastructure
-- Adds tables for backup destinations, backup history, scheduled jobs, and restore drills.

-- Backup destinations (USB, second drive, network path)
CREATE TABLE backup_destinations (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('local', 'usb', 'network')),
    label TEXT NOT NULL DEFAULT '',
    institution_marker TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,
    last_ok_at TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Backup history log
CREATE TABLE backup_history (
    id TEXT PRIMARY KEY,
    destination_id TEXT REFERENCES backup_destinations(id),
    kind TEXT NOT NULL CHECK (kind IN (
        'daily', 'hourly_snapshot', 'full_export',
        'pre_update', 'pre_restore', 'pre_import', 'manual'
    )),
    file_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    app_version TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    institution_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    record_counts_json TEXT NOT NULL DEFAULT '{}',
    money_totals_json TEXT NOT NULL DEFAULT '{}',
    audit_chain_head TEXT,
    vault_file_count INTEGER NOT NULL DEFAULT 0,
    manifest_hmac TEXT,
    verified_at TEXT,
    status TEXT NOT NULL CHECK (status IN (
        'in_progress', 'completed', 'verified', 'failed', 'corrupted'
    )),
    error_message TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_backup_history_destination ON backup_history(destination_id);
CREATE INDEX idx_backup_history_status ON backup_history(status);
CREATE INDEX idx_backup_history_kind_created ON backup_history(kind, created_at);

-- Background job queue (idempotent, lease-based)
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN (
        'pending', 'running', 'completed', 'failed', 'cancelled'
    )),
    payload_json TEXT NOT NULL DEFAULT '{}',
    run_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    lease_until TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    result_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_jobs_status_run_at ON jobs(status, run_at);
CREATE INDEX idx_jobs_kind ON jobs(kind);

-- Restore drill history
CREATE TABLE restore_drills (
    id TEXT PRIMARY KEY,
    backup_id TEXT NOT NULL REFERENCES backup_history(id),
    started_at TEXT NOT NULL,
    completed_at TEXT,
    result TEXT NOT NULL CHECK (result IN ('pass', 'fail', 'error')),
    integrity_ok INTEGER,
    foreign_keys_ok INTEGER,
    record_counts_match INTEGER,
    money_totals_match INTEGER,
    details_json TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_restore_drills_backup ON restore_drills(backup_id);

-- Backup schedule configuration
CREATE TABLE backup_schedules (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('daily', 'hourly_snapshot')),
    destination_id TEXT REFERENCES backup_destinations(id),
    time_of_day TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Seed default permissions for backup and diagnostics
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('backup.manage', 'backup', 'Create, verify, and manage backups'),
    ('backup.restore', 'backup', 'Restore institution data from a backup'),
    ('diagnostics.view', 'system', 'View diagnostics and system information'),
    ('diagnostics.export', 'system', 'Export diagnostic bundles');

-- Grant backup and diagnostics permissions to Owner and Admin default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN (
    SELECT 'backup.manage' AS code
    UNION ALL SELECT 'backup.restore'
    UNION ALL SELECT 'diagnostics.view'
    UNION ALL SELECT 'diagnostics.export'
) p
WHERE r.name IN ('Owner', 'Admin');
