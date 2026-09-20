-- Migration 0004: Academic structure (Programs, Subjects, Batches, Enrollments, Documents)

CREATE TABLE IF NOT EXISTS programs (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES academic_sessions(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    name_bn TEXT,
    code TEXT,
    class_level TEXT,
    group_major TEXT CHECK (group_major IN ('Science', 'Commerce', 'Humanities', 'General')),
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS subjects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    code TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS batches (
    id TEXT PRIMARY KEY,
    program_id TEXT NOT NULL REFERENCES programs(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    teacher_staff_id TEXT REFERENCES staff(id) ON DELETE SET NULL,
    room_number TEXT,
    shift TEXT DEFAULT 'morning' CHECK (shift IN ('morning', 'day', 'evening')),
    schedule_json TEXT NOT NULL DEFAULT '[]',
    capacity INTEGER NOT NULL DEFAULT 40 CHECK (capacity > 0),
    start_date TEXT NOT NULL,
    end_date TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'cancelled')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (program_id, code)
);

CREATE TABLE IF NOT EXISTS batch_subjects (
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    subject_id TEXT NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    teacher_staff_id TEXT REFERENCES staff(id) ON DELETE SET NULL,
    PRIMARY KEY (batch_id, subject_id)
);

CREATE TABLE IF NOT EXISTS enrollments (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
    roll INTEGER,
    start_date TEXT NOT NULL,
    end_date TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'ended')),
    end_reason TEXT CHECK (end_reason IN ('completed', 'transferred', 'dropped', 'left')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_enrollment_unique_active
ON enrollments(student_id, batch_id)
WHERE status = 'active';

CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY,
    owner_type TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime TEXT NOT NULL,
    size INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    uploaded_by TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_owner ON documents(owner_type, owner_id);

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (4, datetime('now'));
