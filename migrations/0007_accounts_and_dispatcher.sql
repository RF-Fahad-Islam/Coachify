-- Migration 0007: Command dispatcher and local accounts
-- Tables: users, roles, permissions, role_permissions, user_roles, devices, sessions, command_idempotency
-- Triggers: system role protection, active owner defense-in-depth protection

-- Local users directory
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY, -- UUIDv7
    username TEXT NOT NULL COLLATE NOCASE UNIQUE,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL, -- Argon2id PHC string
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    must_change_password INTEGER NOT NULL DEFAULT 0 CHECK (must_change_password IN (0, 1)),
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_attempt_at TEXT, -- ISO 8601 UTC timestamp
    locked_until TEXT, -- ISO 8601 UTC timestamp
    staff_profile_id TEXT REFERENCES staff(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_staff ON users(staff_profile_id);

-- Role definitions
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Granular permissions catalog
CREATE TABLE IF NOT EXISTS permissions (
    code TEXT PRIMARY KEY, -- e.g. 'payments.collect'
    module TEXT NOT NULL, -- e.g. 'money', 'students', 'academics', 'system'
    description TEXT NOT NULL
);

-- Mapping roles to permissions
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_code)
);

-- Mapping users to roles
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles(role_id);

-- Known devices (host PC today, paired LAN workstations in version 1.1)
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    windows_name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'blocked', 'revoked')),
    is_host INTEGER NOT NULL DEFAULT 0 CHECK (is_host IN (0, 1)),
    paired_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    app_version TEXT NOT NULL
);

-- Ephemeral session records
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY, -- UUIDv7
    token_hash TEXT NOT NULL UNIQUE, -- SHA256 of session bearer token
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    absolute_expires_at TEXT NOT NULL, -- created_at + 24 hours
    revoked_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Idempotency tracking table
CREATE TABLE IF NOT EXISTS command_idempotency (
    idempotency_key TEXT PRIMARY KEY, -- UUIDv7 string
    command_name TEXT NOT NULL,
    payload_hash TEXT NOT NULL, -- SHA256 of canonical payload JSON
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    response_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_command_idempotency_created ON command_idempotency(created_at);

-- Trigger: prevent deleting system roles
CREATE TRIGGER IF NOT EXISTS trg_roles_protect_system
BEFORE DELETE ON roles
WHEN OLD.is_system = 1
BEGIN
    SELECT RAISE(ABORT, 'System roles cannot be deleted');
END;

-- Trigger: prevent renaming the Owner system role
CREATE TRIGGER IF NOT EXISTS trg_roles_protect_owner_name
BEFORE UPDATE ON roles
WHEN OLD.name = 'Owner' AND NEW.name != 'Owner'
BEGIN
    SELECT RAISE(ABORT, 'The Owner system role cannot be renamed');
END;

-- Trigger: prevent disabling the last active Owner
CREATE TRIGGER IF NOT EXISTS trg_users_protect_last_owner_update
BEFORE UPDATE OF status ON users
WHEN OLD.status = 'active' AND NEW.status != 'active'
BEGIN
    SELECT RAISE(ABORT, 'At least one active Owner must exist')
    WHERE (
        SELECT COUNT(*)
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE r.name = 'Owner' AND u.status = 'active' AND u.id != OLD.id
    ) = 0
    AND EXISTS (
        SELECT 1
        FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = OLD.id AND r.name = 'Owner'
    );
END;

-- Trigger: prevent deleting the last active Owner
CREATE TRIGGER IF NOT EXISTS trg_users_protect_last_owner_delete
BEFORE DELETE ON users
WHEN OLD.status = 'active'
BEGIN
    SELECT RAISE(ABORT, 'At least one active Owner must exist')
    WHERE (
        SELECT COUNT(*)
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE r.name = 'Owner' AND u.status = 'active' AND u.id != OLD.id
    ) = 0
    AND EXISTS (
        SELECT 1
        FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = OLD.id AND r.name = 'Owner'
    );
END;

-- Trigger: prevent removing the Owner role from the last active Owner
CREATE TRIGGER IF NOT EXISTS trg_user_roles_protect_last_owner
BEFORE DELETE ON user_roles
BEGIN
    SELECT RAISE(ABORT, 'At least one active Owner must exist')
    WHERE (
        SELECT r.name FROM roles r WHERE r.id = OLD.role_id
    ) = 'Owner'
    AND (
        SELECT u.status FROM users u WHERE u.id = OLD.user_id
    ) = 'active'
    AND (
        SELECT COUNT(*)
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE r.name = 'Owner' AND u.status = 'active' AND u.id != OLD.user_id
    ) = 0;
END;

-- Seed permissions catalog
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('users.manage', 'system', 'Manage user accounts, roles, and passwords'),
    ('roles.manage', 'system', 'Manage system and custom roles and permission mappings'),
    ('system.settings', 'system', 'Configure institution profile, preferences, and system settings'),
    ('students.view', 'students', 'View student directory, enrollment records, and profiles'),
    ('students.manage', 'students', 'Admit students, edit profiles, and update guardian records'),
    ('academics.view', 'academics', 'View programs, courses, subjects, batches, and schedules'),
    ('academics.manage', 'academics', 'Manage programs, courses, subjects, batches, and routines'),
    ('attendance.mark', 'academics', 'Record and update batch and student attendance'),
    ('exams.manage', 'academics', 'Create exams, manage schedules, and input student marks'),
    ('payments.view', 'money', 'View fee heads, fee plans, charges, and payment receipts'),
    ('payments.collect', 'money', 'Collect fee payments and issue receipts'),
    ('payments.refund', 'money', 'Process refunds, void payments, and post fee concessions'),
    ('reports.view', 'reports', 'View analytical summaries and dashboard metrics'),
    ('reports.export', 'reports', 'Export student, financial, and academic reports');

-- Seed built-in system roles
INSERT OR IGNORE INTO roles (id, name, description, is_system, created_at, updated_at) VALUES
    ('role-owner-0000000000000000000000', 'Owner', 'Full administrative access to all institute features and settings', 1, '2026-09-20T00:00:00Z', '2026-09-20T00:00:00Z'),
    ('role-cashier-00000000000000000000', 'Cashier', 'Fee collection, receipt issuance, and student financial viewing', 1, '2026-09-20T00:00:00Z', '2026-09-20T00:00:00Z'),
    ('role-teacher-00000000000000000000', 'Teacher', 'Classroom management, student attendance, and exam marks entry', 1, '2026-09-20T00:00:00Z', '2026-09-20T00:00:00Z');

-- Map permissions for Owner role (all permissions)
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT 'role-owner-0000000000000000000000', code FROM permissions;

-- Map permissions for Cashier role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code) VALUES
    ('role-cashier-00000000000000000000', 'students.view'),
    ('role-cashier-00000000000000000000', 'payments.view'),
    ('role-cashier-00000000000000000000', 'payments.collect'),
    ('role-cashier-00000000000000000000', 'reports.view');

-- Map permissions for Teacher role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code) VALUES
    ('role-teacher-00000000000000000000', 'students.view'),
    ('role-teacher-00000000000000000000', 'academics.view'),
    ('role-teacher-00000000000000000000', 'attendance.mark'),
    ('role-teacher-00000000000000000000', 'exams.manage');

-- Record schema version 7
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (7, datetime('now'));
