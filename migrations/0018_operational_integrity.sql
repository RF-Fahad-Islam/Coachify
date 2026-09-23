-- Migration 0018: Operational integrity (Features 40 to 49, Spec 0032)
-- Receipt verification, duplicate payment detection, PC migration bundles, integrity check runs, recovery score snapshots, and integrity permissions.

-- 1. Receipt Verification Audit Log
CREATE TABLE IF NOT EXISTS receipt_verifications (
    id TEXT PRIMARY KEY NOT NULL,
    receipt_number TEXT NOT NULL,
    verification_code TEXT NOT NULL,
    status_at_check TEXT NOT NULL CHECK (status_at_check IN (
        'valid', 'voided', 'refunded', 'not_found', 'signature_mismatch'
    )),
    verified_by_user_id TEXT REFERENCES users(id),
    scanned_payload TEXT NOT NULL,
    checked_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_receipt_verifications_num ON receipt_verifications(receipt_number, checked_at DESC);

-- 2. Enhance Payments for Receipt Verification and Duplicate Override Reason
ALTER TABLE payments ADD COLUMN verification_code TEXT;
ALTER TABLE payments ADD COLUMN duplicate_override_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_payments_ref_created ON payments(reference, created_at);

-- 3. PC Migration Registry
CREATE TABLE IF NOT EXISTS pc_migrations (
    id TEXT PRIMARY KEY NOT NULL,
    package_filename TEXT NOT NULL,
    target_machine_hint TEXT,
    exported_by_user_id TEXT NOT NULL REFERENCES users(id),
    exported_at TEXT NOT NULL,
    imported_at TEXT,
    checksum_sha256 TEXT NOT NULL,
    rewrap_salt TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('exported', 'verified', 'imported', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_pc_migrations_status ON pc_migrations(status, exported_at DESC);

-- 4. Database Integrity Diagnostic Run History
CREATE TABLE IF NOT EXISTS integrity_check_runs (
    id TEXT PRIMARY KEY NOT NULL,
    check_type TEXT NOT NULL CHECK (check_type IN ('startup', 'scheduled', 'manual')),
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    pragma_integrity_ok INTEGER NOT NULL CHECK (pragma_integrity_ok IN (0, 1)),
    foreign_keys_ok INTEGER NOT NULL CHECK (foreign_keys_ok IN (0, 1)),
    balance_reconciliation_ok INTEGER NOT NULL CHECK (balance_reconciliation_ok IN (0, 1)),
    audit_chain_ok INTEGER NOT NULL CHECK (audit_chain_ok IN (0, 1)),
    error_details_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_integrity_check_runs_created ON integrity_check_runs(started_at DESC);

-- 5. Disaster Recovery Score Snapshots
CREATE TABLE IF NOT EXISTS recovery_score_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    calculated_at TEXT NOT NULL,
    total_score INTEGER NOT NULL,
    backup_recency_score INTEGER NOT NULL,
    external_copy_score INTEGER NOT NULL,
    drill_success_score INTEGER NOT NULL,
    db_integrity_score INTEGER NOT NULL,
    warnings_json TEXT NOT NULL DEFAULT '[]'
);

CREATE INDEX IF NOT EXISTS idx_recovery_score_snapshots_time ON recovery_score_snapshots(calculated_at DESC);

-- 6. Seed Granular Permissions for Slice 8
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('integrity.view', 'integrity', 'View offline resilience center, contact health, outbox monitor, and recovery score'),
    ('integrity.manage', 'integrity', 'Bulk update contact health, retry outbox messages, manual queue flush, and run drills'),
    ('integrity.migrate', 'system', 'Export and import PC migration installation bundles');

-- 7. Map Permissions for Admin Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Admin'
  AND p.code IN (
      'integrity.view',
      'integrity.manage'
  );

-- 8. Map Permissions for Cashier Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Cashier'
  AND p.code IN (
      'integrity.view'
  );

-- 9. Map Permissions for Receptionist Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Receptionist'
  AND p.code IN (
      'integrity.view'
  );

-- 10. Map Permissions for Teacher Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Teacher'
  AND p.code IN (
      'integrity.view'
  );

-- 11. Map All Permissions to Owner Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'Owner';
