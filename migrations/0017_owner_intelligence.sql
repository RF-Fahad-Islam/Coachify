-- Migration 0017: Owner intelligence (Features 34 to 39, Spec 0033)
-- Batch operational and cost configurations, student timeline notes, due collection contact logs, and intelligence permissions.

-- 1. Batch Operational & Cost Configurations for Profitability
CREATE TABLE IF NOT EXISTS batch_cost_configs (
    id TEXT PRIMARY KEY NOT NULL,
    batch_id TEXT NOT NULL UNIQUE REFERENCES batches(id) ON DELETE CASCADE,
    teacher_monthly_cost INTEGER NOT NULL DEFAULT 0,
    teacher_cost_type TEXT NOT NULL CHECK (teacher_cost_type IN ('monthly_fixed', 'per_class', 'percentage_share')),
    room_monthly_cost INTEGER NOT NULL DEFAULT 0,
    operational_overhead_cost INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 2. Staff Notes and Counseling Observations on Students
CREATE TABLE IF NOT EXISTS student_timeline_notes (
    id TEXT PRIMARY KEY NOT NULL,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    author_id TEXT NOT NULL REFERENCES users(id),
    note_type TEXT NOT NULL CHECK (note_type IN ('academic', 'behavior', 'financial', 'general')),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_student_notes_student ON student_timeline_notes(student_id, created_at DESC);

-- 3. Follow-up Logs and Promise Tracking for Overdue Due Collection
CREATE TABLE IF NOT EXISTS due_collection_contact_logs (
    id TEXT PRIMARY KEY NOT NULL,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    contacted_by_user_id TEXT NOT NULL REFERENCES users(id),
    contact_channel TEXT NOT NULL CHECK (contact_channel IN ('phone_call', 'sms', 'in_person', 'whatsapp')),
    outcome TEXT NOT NULL CHECK (outcome IN ('promised_payment', 'dispute', 'unreachable', 'wrong_number', 'follow_up_needed', 'resolved')),
    promised_payment_date TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_contact_logs_student ON due_collection_contact_logs(student_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contact_logs_created ON due_collection_contact_logs(created_at DESC);

-- 4. Seed Granular Permissions for Slice 7
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('intelligence.view', 'intelligence', 'View operational cockpit, coaching day feed, and batch health diagnostics'),
    ('intelligence.profitability', 'intelligence', 'View and adjust batch profitability, margins, and cost models'),
    ('intelligence.due_queue', 'intelligence', 'View smart due collection queue and record contact logs'),
    ('intelligence.notes', 'students', 'Record academic, behavioral, and general student timeline notes');

-- 5. Map Permissions for Admin Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Admin'
  AND p.code IN (
      'intelligence.view',
      'intelligence.profitability',
      'intelligence.due_queue',
      'intelligence.notes'
  );

-- 6. Map Permissions for Cashier Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Cashier'
  AND p.code IN (
      'intelligence.due_queue'
  );

-- 7. Map Permissions for Receptionist Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Receptionist'
  AND p.code IN (
      'intelligence.due_queue',
      'intelligence.notes'
  );

-- 8. Map Permissions for Teacher Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r, permissions p
WHERE r.name = 'Teacher'
  AND p.code IN (
      'intelligence.view',
      'intelligence.notes'
  );

-- 9. Map All Permissions to Owner Role
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'Owner';
