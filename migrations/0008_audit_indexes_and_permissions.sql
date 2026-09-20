-- Migration 0008: Audit log lookup indexes and audit.view permission

-- Query optimization indexes for audit_log
CREATE INDEX IF NOT EXISTS idx_audit_log_at ON audit_log(at);
CREATE INDEX IF NOT EXISTS idx_audit_log_business_date ON audit_log(business_date);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);

-- Seed audit.view permission
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('audit.view', 'system', 'View audit trail timeline, before/after change diffs, and verify cryptographic chain integrity');

-- Map audit.view permission to the built-in Owner system role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'audit.view' FROM roles WHERE name = 'Owner';

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (8, datetime('now'));
