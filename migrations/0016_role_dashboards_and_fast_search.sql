-- Migration 0016: Role dashboards, zero training mode RBAC, and fast search indexes
-- Seeds Receptionist and Admin system roles, granular permissions, and search composite indexes.

-- 1. Search Composite Index for Fast Receipt Lookup
CREATE INDEX IF NOT EXISTS idx_payments_search_composite ON payments(receipt_no, student_id, received_at);

-- 2. Seed Granular Permissions for Slice 6
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('search.global', 'system', 'Global search across students, guardians, batches, staff, and payment receipts'),
    ('reports.financial', 'reports', 'View financial collections, cash close, dues aging, and cashier performance reports'),
    ('reports.academic', 'reports', 'View attendance summaries, chronic absenteeism, and exam merit reports'),
    ('ledger.view', 'money', 'View student and center financial ledger accounts');

-- 3. Seed Receptionist and Admin System Roles
INSERT OR IGNORE INTO roles (id, name, description, is_system, created_at, updated_at) VALUES
    ('role-receptionist-0000000000000000', 'Receptionist', 'Front desk admissions, student inquiries, and attendance tracking', 1, '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z'),
    ('role-admin-0000000000000000000000', 'Admin', 'Academic operations, student management, batch rosters, and routine reporting', 1, '2026-09-23T00:00:00Z', '2026-09-23T00:00:00Z');

-- 4. Map Permissions for Receptionist Role
-- Receptionist: students.view, students.manage, academics.view, attendance.mark, search.global
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Receptionist'
  AND p.code IN (
      'students.view',
      'students.manage',
      'academics.view',
      'attendance.mark',
      'search.global'
  );

-- 5. Map Permissions for Cashier Role
-- Cashier: students.view, payments.view, payments.collect, ledger.view, reports.financial, search.global
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Cashier'
  AND p.code IN (
      'students.view',
      'payments.view',
      'payments.collect',
      'ledger.view',
      'reports.view',
      'reports.financial',
      'search.global'
  );

-- 6. Map Permissions for Teacher Role
-- Teacher: students.view, academics.view, attendance.mark, exam.view, exam.marks_entry, exam.verify, search.global
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Teacher'
  AND p.code IN (
      'students.view',
      'academics.view',
      'attendance.mark',
      'exam.view',
      'exam.marks_entry',
      'exam.verify',
      'search.global'
  );

-- 7. Map Permissions for Admin Role
-- Admin: Operational permissions (excluding backup.manage, system.settings, roles.manage, audit.view)
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Admin'
  AND p.code IN (
      'students.view',
      'students.manage',
      'students.archive',
      'academics.view',
      'academics.manage',
      'attendance.mark',
      'exam.view',
      'exam.manage',
      'exam.marks_entry',
      'exam.verify',
      'exam.publish',
      'exam.edit_verified',
      'payments.view',
      'payments.collect',
      'payments.refund',
      'ledger.view',
      'reports.view',
      'reports.financial',
      'reports.academic',
      'reports.export',
      'search.global',
      'comms.view',
      'comms.send',
      'comms.templates',
      'users.manage'
  );

-- 8. Map All Permissions to Owner Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'Owner';
