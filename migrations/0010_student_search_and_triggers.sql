-- Migration 0010: Student search index, rowid triggers, and permissions

-- FTS5 trigram virtual table for lightning fast substring matching
-- Note: Uses the native SQLite integer rowid to link with students(rowid)
CREATE VIRTUAL TABLE IF NOT EXISTS student_search_index USING fts5(
    student_code,
    name,
    name_bn,
    phone_norm,
    emergency_phone_norm,
    current_school_roll,
    guardian_names,
    guardian_phones,
    tokenize = 'trigram'
);

-- Triggers to maintain student_search_index on student insert
CREATE TRIGGER IF NOT EXISTS trg_students_fts_insert AFTER INSERT ON students
BEGIN
    INSERT INTO student_search_index (
        rowid, student_code, name, name_bn, phone_norm, emergency_phone_norm,
        current_school_roll, guardian_names, guardian_phones
    )
    VALUES (
        NEW.rowid, NEW.student_code, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm,
        coalesce(NEW.emergency_phone_norm, ''), coalesce(NEW.current_school_roll, ''),
        (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id),
        (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id)
    );
END;

-- Triggers to update search index on student update using direct rowid index lookup
CREATE TRIGGER IF NOT EXISTS trg_students_fts_update AFTER UPDATE ON students
BEGIN
    DELETE FROM student_search_index WHERE rowid = OLD.rowid;
    INSERT INTO student_search_index (
        rowid, student_code, name, name_bn, phone_norm, emergency_phone_norm,
        current_school_roll, guardian_names, guardian_phones
    )
    VALUES (
        NEW.rowid, NEW.student_code, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm,
        coalesce(NEW.emergency_phone_norm, ''), coalesce(NEW.current_school_roll, ''),
        (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id),
        (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id)
    );
END;

-- Triggers to remove from search index on student delete using direct rowid lookup
CREATE TRIGGER IF NOT EXISTS trg_students_fts_delete AFTER DELETE ON students
BEGIN
    DELETE FROM student_search_index WHERE rowid = OLD.rowid;
END;

-- Triggers to refresh search index when student guardians change
CREATE TRIGGER IF NOT EXISTS trg_student_guardians_fts_insert AFTER INSERT ON student_guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.student_id),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.student_id)
    WHERE rowid = (SELECT rowid FROM students WHERE id = NEW.student_id);
END;

CREATE TRIGGER IF NOT EXISTS trg_student_guardians_fts_delete AFTER DELETE ON student_guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = OLD.student_id),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = OLD.student_id)
    WHERE rowid = (SELECT rowid FROM students WHERE id = OLD.student_id);
END;

-- Triggers to refresh search index when guardian contact info changes
CREATE TRIGGER IF NOT EXISTS trg_guardians_fts_update AFTER UPDATE ON guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = (SELECT id FROM students WHERE rowid = student_search_index.rowid)),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = (SELECT id FROM students WHERE rowid = student_search_index.rowid))
    WHERE rowid IN (
        SELECT s.rowid FROM students s
        JOIN student_guardians sg ON sg.student_id = s.id
        WHERE sg.guardian_id = NEW.id
    );
END;

-- Prevent hard delete of students with financial ledger records or enrollments
CREATE TRIGGER IF NOT EXISTS trg_students_prevent_delete_with_history
BEFORE DELETE ON students
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM charges WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with existing financial charges; archive the student instead')
        WHEN EXISTS (SELECT 1 FROM payments WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with recorded payments; archive the student instead')
        WHEN EXISTS (SELECT 1 FROM enrollments WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with enrollment history; archive the student instead')
    END;
END;

-- Backfill search index for existing students
INSERT OR IGNORE INTO student_search_index (
    rowid, student_code, name, name_bn, phone_norm, emergency_phone_norm,
    current_school_roll, guardian_names, guardian_phones
)
SELECT s.rowid, s.student_code, s.name, coalesce(s.name_bn, ''), s.phone_norm,
       coalesce(s.emergency_phone_norm, ''), coalesce(s.current_school_roll, ''),
       (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id),
       (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = s.id)
FROM students s;

-- Seed student permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('students.view', 'students', 'View student directory, profiles, documents, and guardian records'),
    ('students.manage', 'students', 'Create and modify students, guardians, and profile documents'),
    ('students.archive', 'students', 'Change student operational status and archive student profiles');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.manage' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.archive' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (10, datetime('now'));
