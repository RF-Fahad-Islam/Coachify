-- Migration 0003: People directory (Students, Guardians, Student Guardians, Staff)

CREATE TABLE IF NOT EXISTS students (
    id TEXT PRIMARY KEY,
    student_code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    name_bn TEXT,
    gender TEXT NOT NULL CHECK (gender IN ('male', 'female', 'other')),
    dob TEXT NOT NULL,
    blood_group TEXT CHECK (blood_group IN ('A+', 'A-', 'B+', 'B-', 'O+', 'O-', 'AB+', 'AB-')),
    religion TEXT CHECK (religion IN ('Islam', 'Hinduism', 'Buddhism', 'Christianity', 'Other')),
    nid_or_birth_cert TEXT,
    phone_norm TEXT NOT NULL,
    emergency_phone_norm TEXT,
    email TEXT,
    present_address TEXT NOT NULL,
    permanent_address TEXT,
    district TEXT,
    thana TEXT,
    previous_school TEXT,
    current_school_roll TEXT,
    class_level TEXT,
    group_major TEXT CHECK (group_major IN ('Science', 'Commerce', 'Humanities', 'General')),
    photo_vault_id TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'completed', 'transferred', 'archived')),
    admission_date TEXT NOT NULL,
    custom_json TEXT NOT NULL DEFAULT '{}',
    notes TEXT,
    row_version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_students_status ON students(status);
CREATE INDEX IF NOT EXISTS idx_students_phone ON students(phone_norm);
CREATE INDEX IF NOT EXISTS idx_students_admission_date ON students(admission_date);

CREATE TABLE IF NOT EXISTS guardians (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    relationship TEXT NOT NULL CHECK (relationship IN ('father', 'mother', 'uncle', 'brother', 'sister', 'legal_guardian', 'other')),
    occupation TEXT,
    monthly_income_poisha INTEGER,
    nid TEXT,
    phone_norm TEXT NOT NULL,
    phone2_norm TEXT,
    whatsapp_number TEXT,
    email TEXT,
    present_address TEXT,
    permanent_address TEXT,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_guardians_phone ON guardians(phone_norm);

CREATE TABLE IF NOT EXISTS student_guardians (
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    guardian_id TEXT NOT NULL REFERENCES guardians(id) ON DELETE RESTRICT,
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
    receives_sms INTEGER NOT NULL DEFAULT 0 CHECK (receives_sms IN (0, 1)),
    PRIMARY KEY (student_id, guardian_id)
);

CREATE TABLE IF NOT EXISTS staff (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    phone_norm TEXT NOT NULL,
    email TEXT,
    designation TEXT NOT NULL,
    department TEXT,
    educational_qualifications TEXT,
    nid TEXT,
    blood_group TEXT CHECK (blood_group IN ('A+', 'A-', 'B+', 'B-', 'O+', 'O-', 'AB+', 'AB-')),
    gender TEXT CHECK (gender IN ('male', 'female', 'other')),
    dob TEXT,
    present_address TEXT,
    joining_date TEXT NOT NULL,
    resignation_date TEXT,
    salary_type TEXT DEFAULT 'monthly' CHECK (salary_type IN ('monthly', 'per_class')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'on_leave', 'resigned', 'terminated')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (3, datetime('now'));
