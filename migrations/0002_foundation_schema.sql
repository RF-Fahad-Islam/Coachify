-- Migration 0002: Foundation schema, people directories, academic structures, financial ledger, and immutability triggers

-- 1. System and Configuration Tables
CREATE TABLE IF NOT EXISTS institution (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL,
    name_bn TEXT,
    eiin_number TEXT,
    registration_number TEXT,
    established_year INTEGER,
    address TEXT NOT NULL,
    address_bn TEXT,
    email TEXT,
    phones TEXT NOT NULL,
    website TEXT,
    currency TEXT NOT NULL DEFAULT 'BDT',
    currency_symbol TEXT NOT NULL DEFAULT '৳',
    timezone TEXT NOT NULL DEFAULT 'Asia/Dhaka',
    fiscal_year_start INTEGER NOT NULL DEFAULT 1 CHECK (fiscal_year_start BETWEEN 1 AND 12),
    logo_vault_id TEXT,
    receipt_header_bn TEXT,
    receipt_footer_bn TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS academic_sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS clock_guard (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    high_water_at TEXT NOT NULL,
    last_ack_by TEXT,
    last_ack_at TEXT
);

CREATE TABLE IF NOT EXISTS sequences (
    name TEXT PRIMARY KEY,
    prefix TEXT NOT NULL DEFAULT '',
    next_value INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_by TEXT,
    updated_at TEXT NOT NULL
);

-- 2. People Tables (Students, Guardians, Staff)
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

-- 3. Academic Structure Tables
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

-- 4. Financial Ledger Tables
CREATE TABLE IF NOT EXISTS fee_heads (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_bn TEXT,
    kind TEXT NOT NULL CHECK (kind IN ('tuition', 'admission', 'exam', 'registration', 'material', 'fine', 'other')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS fee_plans (
    id TEXT PRIMARY KEY,
    program_id TEXT REFERENCES programs(id) ON DELETE CASCADE,
    batch_id TEXT REFERENCES batches(id) ON DELETE CASCADE,
    fee_head_id TEXT NOT NULL REFERENCES fee_heads(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha >= 0),
    frequency TEXT NOT NULL CHECK (frequency IN ('once', 'monthly')),
    effective_from TEXT NOT NULL,
    effective_to TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK ((program_id IS NOT NULL AND batch_id IS NULL) OR (program_id IS NULL AND batch_id IS NOT NULL))
);

CREATE TABLE IF NOT EXISTS concessions (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    fee_head_id TEXT REFERENCES fee_heads(id) ON DELETE RESTRICT,
    type TEXT NOT NULL CHECK (type IN ('percent', 'fixed')),
    value INTEGER NOT NULL CHECK (value > 0),
    from_period TEXT,
    to_period TEXT,
    reason TEXT NOT NULL,
    approved_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS charges (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    enrollment_id TEXT REFERENCES enrollments(id) ON DELETE RESTRICT,
    fee_head_id TEXT NOT NULL REFERENCES fee_heads(id) ON DELETE RESTRICT,
    period TEXT,
    gross_amount_poisha INTEGER NOT NULL CHECK (gross_amount_poisha >= 0),
    concession_amount_poisha INTEGER NOT NULL DEFAULT 0 CHECK (concession_amount_poisha >= 0),
    net_amount_poisha INTEGER NOT NULL CHECK (net_amount_poisha = gross_amount_poisha - concession_amount_poisha),
    due_date TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('generated', 'manual')),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'voided')),
    void_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_charges_unique_enrollment_period
ON charges(enrollment_id, fee_head_id, period)
WHERE source = 'generated' AND status = 'open' AND period IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_charges_student_status ON charges(student_id, status);

CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    receipt_no TEXT NOT NULL UNIQUE,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'card', 'other')),
    reference TEXT,
    business_date TEXT NOT NULL,
    received_at TEXT NOT NULL,
    collector_user_id TEXT,
    device_id TEXT,
    idempotency_key TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'voided')),
    void_reason TEXT,
    voided_by TEXT,
    voided_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_payments_student ON payments(student_id);
CREATE INDEX IF NOT EXISTS idx_payments_business_date ON payments(business_date);

CREATE TABLE IF NOT EXISTS payment_allocations (
    id TEXT PRIMARY KEY,
    payment_id TEXT NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    charge_id TEXT NOT NULL REFERENCES charges(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_payment_allocations_charge ON payment_allocations(charge_id);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_payment ON payment_allocations(payment_id);

CREATE TABLE IF NOT EXISTS adjustments (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    charge_id TEXT REFERENCES charges(id) ON DELETE RESTRICT,
    type TEXT NOT NULL CHECK (type IN ('waiver', 'fine', 'correction')),
    amount_poisha INTEGER NOT NULL,
    reason TEXT NOT NULL,
    approved_by TEXT,
    business_date TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS refunds (
    id TEXT PRIMARY KEY,
    payment_id TEXT NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'other')),
    reason TEXT NOT NULL,
    business_date TEXT NOT NULL,
    refunded_by TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cash_days (
    business_date TEXT PRIMARY KEY,
    opening_cash_poisha INTEGER NOT NULL DEFAULT 0,
    cash_in_poisha INTEGER NOT NULL DEFAULT 0,
    cash_out_poisha INTEGER NOT NULL DEFAULT 0,
    expected_closing_poisha INTEGER NOT NULL DEFAULT 0,
    counted_closing_poisha INTEGER,
    difference_poisha INTEGER,
    note TEXT,
    closed_by TEXT,
    closed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS expense_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES expense_categories(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'other')),
    payee TEXT NOT NULL,
    reference TEXT,
    business_date TEXT NOT NULL,
    attachment_vault_id TEXT,
    created_by TEXT,
    status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'voided')),
    void_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS student_balances (
    student_id TEXT PRIMARY KEY REFERENCES students(id) ON DELETE CASCADE,
    total_charged_poisha INTEGER NOT NULL DEFAULT 0,
    total_paid_poisha INTEGER NOT NULL DEFAULT 0,
    total_adjusted_poisha INTEGER NOT NULL DEFAULT 0,
    total_refunded_poisha INTEGER NOT NULL DEFAULT 0,
    balance_poisha INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

-- 5. Audit Log Table
CREATE TABLE IF NOT EXISTS audit_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL,
    business_date TEXT NOT NULL,
    user_id TEXT,
    device_id TEXT,
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    before_json TEXT,
    after_json TEXT,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL
);

-- 6. Defense in Depth SQLite Triggers
CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_update
BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_delete
BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be deleted');
END;

CREATE TRIGGER IF NOT EXISTS trg_clock_guard_monotonic
BEFORE UPDATE ON clock_guard
WHEN NEW.high_water_at < OLD.high_water_at
BEGIN
    SELECT RAISE(ABORT, 'Clock guard violation: system time cannot move backwards');
END;

CREATE TRIGGER IF NOT EXISTS trg_students_row_version
AFTER UPDATE ON students
FOR EACH ROW
WHEN NEW.row_version = OLD.row_version
BEGIN
    UPDATE students SET row_version = OLD.row_version + 1 WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_payments_no_delete
BEFORE DELETE ON payments
BEGIN
    SELECT RAISE(ABORT, 'Posted payments cannot be deleted; void the payment instead');
END;

CREATE TRIGGER IF NOT EXISTS trg_payments_no_update
BEFORE UPDATE ON payments
WHEN OLD.status = 'posted' AND NEW.status = 'posted' AND (
    OLD.amount_poisha != NEW.amount_poisha OR
    OLD.receipt_no != NEW.receipt_no OR
    OLD.student_id != NEW.student_id OR
    OLD.business_date != NEW.business_date
)
BEGIN
    SELECT RAISE(ABORT, 'Posted payment details cannot be altered');
END;

CREATE TRIGGER IF NOT EXISTS trg_charges_no_delete
BEFORE DELETE ON charges
BEGIN
    SELECT RAISE(ABORT, 'Charges are immutable; void the charge if unallocated');
END;

CREATE TRIGGER IF NOT EXISTS trg_payment_allocations_no_update
BEFORE UPDATE ON payment_allocations
BEGIN
    SELECT RAISE(ABORT, 'Payment allocations are immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_payment_allocations_no_delete
BEFORE DELETE ON payment_allocations
BEGIN
    SELECT RAISE(ABORT, 'Payment allocations cannot be deleted');
END;

CREATE TRIGGER IF NOT EXISTS trg_cash_days_closed_immutable
BEFORE UPDATE ON cash_days
WHEN OLD.closed_at IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'Closed cash days are locked and cannot be modified');
END;

-- 7. Schema Version Record
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (2, datetime('now'));
