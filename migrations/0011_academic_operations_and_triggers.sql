-- Migration 0011: Academic operations, roll unique constraint, staff search, and safety triggers

-- 1. Ensure roll numbers are unique among active students in the same batch
CREATE UNIQUE INDEX IF NOT EXISTS idx_batch_active_roll
ON enrollments(batch_id, roll)
WHERE status = 'active' AND roll IS NOT NULL;

-- 2. Prevent hard deletion of programs that have batches
CREATE TRIGGER IF NOT EXISTS trg_programs_prevent_delete_with_batches
BEFORE DELETE ON programs
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batches WHERE program_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete program with existing batches; archive the program instead')
    END;
END;

-- 3. Prevent hard deletion of batches with enrollment history
CREATE TRIGGER IF NOT EXISTS trg_batches_prevent_delete_with_enrollments
BEFORE DELETE ON batches
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM enrollments WHERE batch_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete batch with enrollment history; cancel or complete the batch instead')
    END;
END;

-- 4. Prevent hard deletion of staff assigned to batches or linked to user accounts
CREATE TRIGGER IF NOT EXISTS trg_staff_prevent_delete_with_assignments
BEFORE DELETE ON staff
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batches WHERE teacher_staff_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member assigned as lead teacher of a batch')
        WHEN EXISTS (SELECT 1 FROM batch_subjects WHERE teacher_staff_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member assigned to batch subjects')
        WHEN EXISTS (SELECT 1 FROM users WHERE staff_profile_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member linked to an active software user account')
    END;
END;

-- 5. Prevent hard deletion of subjects linked to batches
CREATE TRIGGER IF NOT EXISTS trg_subjects_prevent_delete_with_batches
BEFORE DELETE ON subjects
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batch_subjects WHERE subject_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete subject assigned to active or past batches')
    END;
END;

-- 6. SQLite FTS5 trigram virtual table for Staff search
CREATE VIRTUAL TABLE IF NOT EXISTS staff_search_index USING fts5(
    name,
    name_bn,
    phone_norm,
    designation,
    department,
    tokenize = 'trigram'
);

-- Triggers to synchronize staff search index
CREATE TRIGGER IF NOT EXISTS trg_staff_fts_insert AFTER INSERT ON staff
BEGIN
    INSERT INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
    VALUES (NEW.rowid, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm, NEW.designation, coalesce(NEW.department, ''));
END;

CREATE TRIGGER IF NOT EXISTS trg_staff_fts_update AFTER UPDATE ON staff
BEGIN
    DELETE FROM staff_search_index WHERE rowid = OLD.rowid;
    INSERT INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
    VALUES (NEW.rowid, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm, NEW.designation, coalesce(NEW.department, ''));
END;

CREATE TRIGGER IF NOT EXISTS trg_staff_fts_delete AFTER DELETE ON staff
BEGIN
    DELETE FROM staff_search_index WHERE rowid = OLD.rowid;
END;

-- Backfill staff search index for existing records
INSERT OR IGNORE INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
SELECT rowid, name, coalesce(name_bn, ''), phone_norm, designation, coalesce(department, '')
FROM staff;

-- 7. Seed Academic & Staff RBAC Permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('academic.view', 'academic', 'View programs, batches, subjects, timetable schedules, and student rosters'),
    ('academic.manage', 'academic', 'Create and modify programs, batches, subjects, curriculum assignments, enrollments, and transfers'),
    ('staff.view', 'staff', 'View staff directory, profiles, and teaching assignments'),
    ('staff.manage', 'staff', 'Create and modify staff records, designations, and salary structures');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'academic.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'academic.manage' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'staff.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'staff.manage' FROM roles WHERE name IN ('Owner', 'Admin');

-- 8. Record schema version
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (11, datetime('now'));
