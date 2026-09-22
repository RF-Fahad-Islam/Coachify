-- Migration 0015: Exams, mark entry, grading schemes, and immutable results
-- Tables for grading schemes, grade bands, exams, exam subjects, marks,
-- frozen result snapshots, student results, and result subject details.

-- 1. Grading Schemes and Grade Bands
CREATE TABLE grading_schemes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    scale_type TEXT NOT NULL DEFAULT 'gpa_5' CHECK (scale_type IN ('gpa_5', 'gpa_4', 'letter_only', 'percentage')),
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE grade_bands (
    id TEXT PRIMARY KEY,
    scheme_id TEXT NOT NULL REFERENCES grading_schemes(id) ON DELETE CASCADE,
    grade TEXT NOT NULL,
    grade_point INTEGER NOT NULL DEFAULT 0, -- hundredths: 5.00 -> 500
    min_percentage INTEGER NOT NULL,        -- hundredths: 80.00% -> 8000
    max_percentage INTEGER NOT NULL,        -- hundredths: 100.00% -> 10000
    is_fail INTEGER NOT NULL DEFAULT 0 CHECK (is_fail IN (0, 1)),
    created_at TEXT NOT NULL,
    UNIQUE(scheme_id, grade),
    CHECK (min_percentage <= max_percentage)
);

CREATE INDEX idx_grade_bands_scheme ON grade_bands(scheme_id);

-- 2. Exams and Participating Batches
CREATE TABLE exams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    session_id TEXT NOT NULL REFERENCES academic_sessions(id) ON DELETE RESTRICT,
    exam_type TEXT NOT NULL DEFAULT 'term' CHECK (exam_type IN ('term', 'class_test', 'model_test', 'final', 'quiz')),
    start_date TEXT NOT NULL,
    end_date TEXT,
    grading_scheme_id TEXT NOT NULL REFERENCES grading_schemes(id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'entry', 'verified', 'published')),
    rules_config_json TEXT NOT NULL DEFAULT '{}',
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_exams_session_status ON exams(session_id, status);

CREATE TABLE exam_batches (
    exam_id TEXT NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
    PRIMARY KEY (exam_id, batch_id)
);

CREATE INDEX idx_exam_batches_batch ON exam_batches(batch_id);

-- 3. Exam Subjects
CREATE TABLE exam_subjects (
    id TEXT PRIMARY KEY,
    exam_id TEXT NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    subject_id TEXT NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    full_marks INTEGER NOT NULL CHECK (full_marks > 0), -- hundredths: 100.00 -> 10000
    pass_marks INTEGER NOT NULL CHECK (pass_marks >= 0 AND pass_marks <= full_marks),
    exam_date TEXT,
    start_time TEXT,
    room_number TEXT,
    is_optional_allowed INTEGER NOT NULL DEFAULT 1 CHECK (is_optional_allowed IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (exam_id, subject_id)
);

CREATE INDEX idx_exam_subjects_exam ON exam_subjects(exam_id);

-- 4. Student Marks
CREATE TABLE marks (
    exam_subject_id TEXT NOT NULL REFERENCES exam_subjects(id) ON DELETE CASCADE,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    marks INTEGER, -- hundredths, NULL if absent
    is_absent INTEGER NOT NULL DEFAULT 0 CHECK (is_absent IN (0, 1)),
    is_optional INTEGER NOT NULL DEFAULT 0 CHECK (is_optional IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'verified')),
    entered_by TEXT REFERENCES users(id),
    entered_at TEXT,
    verified_by TEXT REFERENCES users(id),
    verified_at TEXT,
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (exam_subject_id, student_id),
    CHECK ((is_absent = 1 AND marks IS NULL) OR (is_absent = 0 AND marks IS NOT NULL AND marks >= 0))
);

CREATE INDEX idx_marks_student ON marks(student_id);

-- 5. Mark Validation & Immutability Triggers
CREATE TRIGGER trg_marks_validate_range_insert
BEFORE INSERT ON marks
BEGIN
    SELECT CASE
        WHEN NEW.is_absent = 0 AND NEW.marks > (SELECT full_marks FROM exam_subjects WHERE id = NEW.exam_subject_id)
            THEN RAISE(ABORT, 'Mark cannot exceed exam subject full marks')
    END;
END;

CREATE TRIGGER trg_marks_validate_range_update
BEFORE UPDATE ON marks
BEGIN
    SELECT CASE
        WHEN NEW.is_absent = 0 AND NEW.marks > (SELECT full_marks FROM exam_subjects WHERE id = NEW.exam_subject_id)
            THEN RAISE(ABORT, 'Mark cannot exceed exam subject full marks')
    END;
END;

CREATE TRIGGER trg_marks_prevent_edit_when_verified
BEFORE UPDATE ON marks
BEGIN
    SELECT CASE
        WHEN OLD.status = 'verified' AND NEW.status = 'verified' AND (OLD.marks != NEW.marks OR OLD.is_absent != NEW.is_absent)
            THEN RAISE(ABORT, 'Cannot modify verified marks without re-opening verification')
    END;
END;

CREATE TRIGGER trg_marks_prevent_insert_when_published
BEFORE INSERT ON marks
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1 FROM exam_subjects es
            JOIN exams e ON e.id = es.exam_id
            WHERE es.id = NEW.exam_subject_id AND e.status = 'published'
        )
        THEN RAISE(ABORT, 'Cannot insert marks for a published exam')
    END;
END;

CREATE TRIGGER trg_marks_prevent_edit_when_published
BEFORE UPDATE ON marks
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1 FROM exam_subjects es
            JOIN exams e ON e.id = es.exam_id
            WHERE es.id = OLD.exam_subject_id AND e.status = 'published'
        )
        THEN RAISE(ABORT, 'Cannot alter marks for a published exam; republishing creates a new snapshot')
    END;
END;

CREATE TRIGGER trg_marks_prevent_delete_when_published
BEFORE DELETE ON marks
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1 FROM exam_subjects es
            JOIN exams e ON e.id = es.exam_id
            WHERE es.id = OLD.exam_subject_id AND e.status = 'published'
        )
        THEN RAISE(ABORT, 'Cannot delete marks for a published exam')
    END;
END;

-- 6. Result Snapshots and Final Grades
CREATE TABLE result_snapshots (
    id TEXT PRIMARY KEY,
    exam_id TEXT NOT NULL REFERENCES exams(id) ON DELETE RESTRICT,
    version INTEGER NOT NULL DEFAULT 1,
    published_at TEXT NOT NULL,
    published_by TEXT NOT NULL REFERENCES users(id),
    grading_scheme_json TEXT NOT NULL,
    calculation_rules_json TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0, 1)),
    created_at TEXT NOT NULL,
    UNIQUE(exam_id, version)
);

CREATE INDEX idx_result_snapshots_exam ON result_snapshots(exam_id, is_current);

CREATE TABLE results (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL REFERENCES result_snapshots(id) ON DELETE CASCADE,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
    total_marks INTEGER NOT NULL,
    total_max_marks INTEGER NOT NULL,
    percentage INTEGER NOT NULL, -- basis points
    gpa INTEGER NOT NULL,        -- hundredths (e.g. 500 = 5.00, 483 = 4.83)
    grade TEXT NOT NULL,
    passed INTEGER NOT NULL CHECK (passed IN (0, 1)),
    rank INTEGER NOT NULL,
    tie_breaker_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    UNIQUE(snapshot_id, student_id)
);

CREATE INDEX idx_results_snapshot_rank ON results(snapshot_id, rank);
CREATE INDEX idx_results_student ON results(student_id);

CREATE TABLE result_subject_details (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES results(id) ON DELETE CASCADE,
    exam_subject_id TEXT NOT NULL REFERENCES exam_subjects(id) ON DELETE RESTRICT,
    marks INTEGER,
    full_marks INTEGER NOT NULL,
    pass_marks INTEGER NOT NULL,
    is_absent INTEGER NOT NULL DEFAULT 0 CHECK (is_absent IN (0, 1)),
    is_optional INTEGER NOT NULL DEFAULT 0 CHECK (is_optional IN (0, 1)),
    grade TEXT NOT NULL,
    grade_point INTEGER NOT NULL,
    passed INTEGER NOT NULL CHECK (passed IN (0, 1)),
    created_at TEXT NOT NULL,
    UNIQUE(result_id, exam_subject_id)
);

CREATE INDEX idx_result_subject_details_result ON result_subject_details(result_id);

-- Triggers for published results immutability
CREATE TRIGGER trg_results_immutable_update
BEFORE UPDATE ON results
BEGIN
    SELECT RAISE(ABORT, 'Published results are immutable');
END;

CREATE TRIGGER trg_results_immutable_delete
BEFORE DELETE ON results
BEGIN
    SELECT RAISE(ABORT, 'Published results are immutable');
END;

CREATE TRIGGER trg_result_subject_details_immutable_update
BEFORE UPDATE ON result_subject_details
BEGIN
    SELECT RAISE(ABORT, 'Published result details are immutable');
END;

CREATE TRIGGER trg_result_subject_details_immutable_delete
BEFORE DELETE ON result_subject_details
BEGIN
    SELECT RAISE(ABORT, 'Published result details are immutable');
END;

-- 7. Seed System Grading Schemes
-- Scheme 1: NCTB Bangladesh National Curriculum (GPA 5.0)
INSERT OR IGNORE INTO grading_schemes (id, name, name_bn, scale_type, is_system, description, created_at, updated_at)
VALUES (
    'scheme_nctb_gpa5',
    'NCTB National Curriculum (GPA 5.0)',
    'জাতীয় শিক্ষাক্রম ও পাঠ্যপুস্তক বোর্ড (জিপিএ ৫.০)',
    'gpa_5',
    1,
    'Standard grading scale for SSC, HSC, and general schooling in Bangladesh (A+ to F)',
    datetime('now'),
    datetime('now')
);

INSERT OR IGNORE INTO grade_bands (id, scheme_id, grade, grade_point, min_percentage, max_percentage, is_fail, created_at) VALUES
    ('band_nctb_aplus', 'scheme_nctb_gpa5', 'A+', 500, 8000, 10000, 0, datetime('now')),
    ('band_nctb_a',     'scheme_nctb_gpa5', 'A',  400, 7000, 7999,  0, datetime('now')),
    ('band_nctb_aminus','scheme_nctb_gpa5', 'A-', 350, 6000, 6999,  0, datetime('now')),
    ('band_nctb_b',     'scheme_nctb_gpa5', 'B',  300, 5000, 5999,  0, datetime('now')),
    ('band_nctb_c',     'scheme_nctb_gpa5', 'C',  200, 4000, 4999,  0, datetime('now')),
    ('band_nctb_d',     'scheme_nctb_gpa5', 'D',  100, 3300, 3999,  0, datetime('now')),
    ('band_nctb_f',     'scheme_nctb_gpa5', 'F',  0,   0,    3299,  1, datetime('now'));

-- Scheme 2: British Curriculum (Cambridge / Edexcel GPA 4.0)
INSERT OR IGNORE INTO grading_schemes (id, name, name_bn, scale_type, is_system, description, created_at, updated_at)
VALUES (
    'scheme_british_gpa4',
    'British Curriculum (Cambridge / Edexcel)',
    'ব্রিটিশ কারিকুলাম (ক্যামব্রিজ / এডেক্সেল)',
    'gpa_4',
    1,
    'Letter scale with GPA 4.0 for English Medium O/A Level curricula',
    datetime('now'),
    datetime('now')
);

INSERT OR IGNORE INTO grade_bands (id, scheme_id, grade, grade_point, min_percentage, max_percentage, is_fail, created_at) VALUES
    ('band_uk_astar', 'scheme_british_gpa4', 'A*', 400, 9000, 10000, 0, datetime('now')),
    ('band_uk_a',     'scheme_british_gpa4', 'A',  400, 8000, 8999,  0, datetime('now')),
    ('band_uk_b',     'scheme_british_gpa4', 'B',  300, 7000, 7999,  0, datetime('now')),
    ('band_uk_c',     'scheme_british_gpa4', 'C',  200, 6000, 6999,  0, datetime('now')),
    ('band_uk_d',     'scheme_british_gpa4', 'D',  100, 5000, 5999,  0, datetime('now')),
    ('band_uk_e',     'scheme_british_gpa4', 'E',  100, 4000, 4999,  0, datetime('now')),
    ('band_uk_u',     'scheme_british_gpa4', 'U',  0,   0,    3999,  1, datetime('now'));

-- 8. Seed Exam RBAC Permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('exam.view', 'exam', 'View exams, schedules, marks rosters, and published results'),
    ('exam.manage', 'exam', 'Create and modify exams, subjects, full/pass marks, and grading schemes'),
    ('exam.marks_entry', 'exam', 'Enter draft marks and mark absent for batch subjects'),
    ('exam.verify', 'exam', 'Verify and lock completed subject marks sheets'),
    ('exam.publish', 'exam', 'Calculate final GPAs, generate ranks, freeze snapshots, and publish results'),
    ('exam.edit_verified', 'exam', 'Re-open or modify verified marks prior to result publication');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Teacher', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.manage' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.marks_entry' FROM roles WHERE name IN ('Owner', 'Admin', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.verify' FROM roles WHERE name IN ('Owner', 'Admin', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.publish' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'exam.edit_verified' FROM roles WHERE name IN ('Owner', 'Admin');
