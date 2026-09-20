# 0002. Data model and database conventions

**Date**: 2026-09-20
**Status**: In Progress

## Summary

Coachify stores all institution records in an encrypted SQLite database using strict table constraints, time ordered identifiers, and power safe settings. Financial amounts are kept as integer poisha (one hundred poisha per taka) to prevent rounding errors, while timestamps record UTC time alongside a local business calendar date. Database triggers reject any update or deletion on posted audit and financial rows so historical records cannot be altered. Migration 0002 establishes the comprehensive schema across all core domains, including institution configuration, people directories, academic structures, and the complete financial ledger with payments, charges, adjustments, and cash days.

## Requirements

**User stories**:
- As an institute owner, I want my student records and operational financial data preserved safely during power cuts so that sudden load shedding never corrupts the database.
- As an administrator, I want immutable audit logs and posted financial records so that curious staff or database tools cannot tamper with cash collection history.
- As a cashier, I want exact financial tracking with integer poisha, clear dues, and tamper proof receipt numbers so that accounting stays balanced to the last paisa.
- As a software engineer, I want clear database conventions for identifiers, currency, and timestamps so that queries and domain logic remain consistent across every feature.

**Acceptance criteria**:
- **AC-1**: Database connections configure WAL mode (write ahead logging), `synchronous = FULL`, `foreign_keys = ON`, and a busy timeout of 5000 milliseconds to guarantee crash resilience.
- **AC-2**: The application executes an integrity quick check (`PRAGMA quick_check`) on startup and safely stops if any page corruption is detected.
- **AC-3**: Migration 0002 establishes the schema for foundational tables (institution, academic sessions, students, guardians, student guardians, staff, programs, subjects, batches, batch subjects, enrollments, documents, sequences, settings, clock guard) and the complete financial ledger (fee heads, fee plans, concessions, charges, payments, payment allocations, adjustments, refunds, cash days, expense categories, expenses, student balances).
- **AC-4**: SQLite triggers raise abort errors on any attempt to update or delete rows in the audit log, tamper with the monotonic clock guard, delete posted payments or charges, modify payment allocations, or alter closed cash days.
- **AC-5**: All monetary columns represent currency as integer poisha (`i64` in Rust), percentages as integer basis points, and identifiers as UUIDv7 text strings.
- **AC-6**: Daily operational records (payments, charges, adjustments, refunds, expenses, cash days, and audit logs) record both a UTC ISO 8601 instant and an Asia/Dhaka civil date (`business_date`).

## Decision

**Chosen option**: Option 1: Comprehensive core and financial schema in migration 0002 with defense in depth SQLite triggers and synchronous = FULL.

You build the complete foundational and financial database layer in migration 0002 with strict constraints, power cut safety PRAGMAs, defense in depth immutability triggers, and plain SQL mapping in Rust.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

#### 1. System and Configuration Tables

```sql
-- Single row institution profile
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
    phones TEXT NOT NULL, -- comma separated or JSON array of contact numbers
    website TEXT,
    currency TEXT NOT NULL DEFAULT 'BDT',
    currency_symbol TEXT NOT NULL DEFAULT '৳',
    timezone TEXT NOT NULL DEFAULT 'Asia/Dhaka',
    fiscal_year_start INTEGER NOT NULL DEFAULT 1 CHECK (fiscal_year_start BETWEEN 1 AND 12),
    logo_vault_id TEXT, -- SHA256 hex address in local document vault
    receipt_header_bn TEXT,
    receipt_footer_bn TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Academic years or sessions
CREATE TABLE IF NOT EXISTS academic_sessions (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL UNIQUE, -- e.g. '2026'
    start_date TEXT NOT NULL, -- YYYY-MM-DD
    end_date TEXT NOT NULL, -- YYYY-MM-DD
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Monotonic clock guard with protection against backwards shifts and CMOS battery future jumps
CREATE TABLE IF NOT EXISTS clock_guard (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    high_water_at TEXT NOT NULL, -- ISO 8601 UTC timestamp
    last_ack_by TEXT,
    last_ack_at TEXT
);

-- Atomic sequence counters for human readable identifiers
CREATE TABLE IF NOT EXISTS sequences (
    name TEXT PRIMARY KEY, -- e.g. 'receipt:2026' (calendar year), 'student:2026' (session name)
    prefix TEXT NOT NULL DEFAULT '',
    next_value INTEGER NOT NULL DEFAULT 1
);

-- Application settings key value store
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_by TEXT,
    updated_at TEXT NOT NULL
);
```

#### 2. People Tables (Students, Guardians, Staff)

```sql
-- Students directory with robust fields for Bangladesh institutions
CREATE TABLE IF NOT EXISTS students (
    id TEXT PRIMARY KEY, -- UUIDv7
    student_code TEXT NOT NULL UNIQUE, -- human facing admission or roll code
    name TEXT NOT NULL,
    name_bn TEXT,
    gender TEXT NOT NULL CHECK (gender IN ('male', 'female', 'other')),
    dob TEXT NOT NULL, -- YYYY-MM-DD
    blood_group TEXT CHECK (blood_group IN ('A+', 'A-', 'B+', 'B-', 'O+', 'O-', 'AB+', 'AB-')),
    religion TEXT CHECK (religion IN ('Islam', 'Hinduism', 'Buddhism', 'Christianity', 'Other')),
    nid_or_birth_cert TEXT, -- National ID or Birth Registration Number
    phone_norm TEXT NOT NULL, -- normalized 8801XXXXXXXXX
    emergency_phone_norm TEXT,
    email TEXT,
    present_address TEXT NOT NULL,
    permanent_address TEXT,
    district TEXT,
    thana TEXT,
    previous_school TEXT,
    current_school_roll TEXT,
    class_level TEXT, -- e.g. 'Class 9', 'Class 10', 'HSC 1st Year'
    group_major TEXT CHECK (group_major IN ('Science', 'Commerce', 'Humanities', 'General')),
    photo_vault_id TEXT, -- SHA256 hex address in local document vault
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'completed', 'transferred', 'archived')),
    admission_date TEXT NOT NULL, -- YYYY-MM-DD
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

-- Guardians directory
CREATE TABLE IF NOT EXISTS guardians (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    name_bn TEXT,
    relationship TEXT NOT NULL CHECK (relationship IN ('father', 'mother', 'uncle', 'brother', 'sister', 'legal_guardian', 'other')),
    occupation TEXT,
    monthly_income_poisha INTEGER, -- integer poisha (optional)
    nid TEXT,
    phone_norm TEXT NOT NULL, -- normalized 8801XXXXXXXXX
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

-- Student Guardian relationship map
CREATE TABLE IF NOT EXISTS student_guardians (
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    guardian_id TEXT NOT NULL REFERENCES guardians(id) ON DELETE RESTRICT,
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
    receives_sms INTEGER NOT NULL DEFAULT 0 CHECK (receives_sms IN (0, 1)),
    PRIMARY KEY (student_id, guardian_id)
);

-- Staff and teachers directory
CREATE TABLE IF NOT EXISTS staff (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    name_bn TEXT,
    phone_norm TEXT NOT NULL, -- normalized 8801XXXXXXXXX
    email TEXT,
    designation TEXT NOT NULL, -- e.g. 'Senior Teacher', 'Branch Manager', 'Cashier'
    department TEXT,
    educational_qualifications TEXT,
    nid TEXT,
    blood_group TEXT CHECK (blood_group IN ('A+', 'A-', 'B+', 'B-', 'O+', 'O-', 'AB+', 'AB-')),
    gender TEXT CHECK (gender IN ('male', 'female', 'other')),
    dob TEXT,
    present_address TEXT,
    joining_date TEXT NOT NULL, -- YYYY-MM-DD
    resignation_date TEXT,
    salary_type TEXT DEFAULT 'monthly' CHECK (salary_type IN ('monthly', 'per_class')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'on_leave', 'resigned', 'terminated')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

#### 3. Academic Structure Tables

```sql
-- Programs or courses offered by the coaching center
CREATE TABLE IF NOT EXISTS programs (
    id TEXT PRIMARY KEY, -- UUIDv7
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

-- Academic subjects
CREATE TABLE IF NOT EXISTS subjects (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    name_bn TEXT,
    code TEXT NOT NULL UNIQUE, -- e.g. 'MATH-101'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Batches or class sections
CREATE TABLE IF NOT EXISTS batches (
    id TEXT PRIMARY KEY, -- UUIDv7
    program_id TEXT NOT NULL REFERENCES programs(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    code TEXT NOT NULL, -- unique within session
    teacher_staff_id TEXT REFERENCES staff(id) ON DELETE SET NULL, -- class teacher or head
    room_number TEXT,
    shift TEXT DEFAULT 'morning' CHECK (shift IN ('morning', 'day', 'evening')),
    schedule_json TEXT NOT NULL DEFAULT '[]', -- array of day, start_time, end_time objects
    capacity INTEGER NOT NULL DEFAULT 40 CHECK (capacity > 0),
    start_date TEXT NOT NULL, -- YYYY-MM-DD
    end_date TEXT, -- YYYY-MM-DD
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'cancelled')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (program_id, code)
);

-- Curriculum mapping between batch and subjects
CREATE TABLE IF NOT EXISTS batch_subjects (
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    subject_id TEXT NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    teacher_staff_id TEXT REFERENCES staff(id) ON DELETE SET NULL,
    PRIMARY KEY (batch_id, subject_id)
);

-- Student enrollment in a batch
CREATE TABLE IF NOT EXISTS enrollments (
    id TEXT PRIMARY KEY, -- UUIDv7
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
    roll INTEGER, -- batch specific roll number
    start_date TEXT NOT NULL, -- YYYY-MM-DD
    end_date TEXT, -- YYYY-MM-DD
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'ended')),
    end_reason TEXT CHECK (end_reason IN ('completed', 'transferred', 'dropped', 'left')),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Ensure a student has only one active enrollment per batch
CREATE UNIQUE INDEX IF NOT EXISTS idx_enrollment_unique_active
ON enrollments(student_id, batch_id)
WHERE status = 'active';

-- Content addressed document vault registry
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY, -- UUIDv7
    owner_type TEXT NOT NULL, -- e.g. 'student', 'staff', 'institution'
    owner_id TEXT NOT NULL,
    kind TEXT NOT NULL, -- 'photo', 'nid_copy', 'birth_certificate', 'marksheet'
    original_name TEXT NOT NULL,
    mime TEXT NOT NULL,
    size INTEGER NOT NULL,
    sha256 TEXT NOT NULL, -- references physical file vault/ab/abcdef...
    uploaded_by TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_owner ON documents(owner_type, owner_id);
```

#### 4. Financial Ledger Tables

```sql
-- Fee category definitions
CREATE TABLE IF NOT EXISTS fee_heads (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    name_bn TEXT,
    kind TEXT NOT NULL CHECK (kind IN ('tuition', 'admission', 'exam', 'registration', 'material', 'fine', 'other')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Fee schedules for programs or batches
CREATE TABLE IF NOT EXISTS fee_plans (
    id TEXT PRIMARY KEY, -- UUIDv7
    program_id TEXT REFERENCES programs(id) ON DELETE CASCADE,
    batch_id TEXT REFERENCES batches(id) ON DELETE CASCADE,
    fee_head_id TEXT NOT NULL REFERENCES fee_heads(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha >= 0),
    frequency TEXT NOT NULL CHECK (frequency IN ('once', 'monthly')),
    effective_from TEXT NOT NULL, -- YYYY-MM-DD
    effective_to TEXT, -- YYYY-MM-DD
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK ((program_id IS NOT NULL AND batch_id IS NULL) OR (program_id IS NULL AND batch_id IS NOT NULL))
);

-- Student specific concessions and scholarships
CREATE TABLE IF NOT EXISTS concessions (
    id TEXT PRIMARY KEY, -- UUIDv7
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    fee_head_id TEXT REFERENCES fee_heads(id) ON DELETE RESTRICT, -- NULL means all fee heads
    type TEXT NOT NULL CHECK (type IN ('percent', 'fixed')),
    value INTEGER NOT NULL CHECK (value > 0), -- basis points if percent, poisha if fixed
    from_period TEXT, -- YYYY-MM
    to_period TEXT, -- YYYY-MM
    reason TEXT NOT NULL,
    approved_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Ledger charges issued to students
CREATE TABLE IF NOT EXISTS charges (
    id TEXT PRIMARY KEY, -- UUIDv7
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    enrollment_id TEXT REFERENCES enrollments(id) ON DELETE RESTRICT,
    fee_head_id TEXT NOT NULL REFERENCES fee_heads(id) ON DELETE RESTRICT,
    period TEXT, -- YYYY-MM, NULL for one time fees
    gross_amount_poisha INTEGER NOT NULL CHECK (gross_amount_poisha >= 0),
    concession_amount_poisha INTEGER NOT NULL DEFAULT 0 CHECK (concession_amount_poisha >= 0),
    net_amount_poisha INTEGER NOT NULL CHECK (net_amount_poisha = gross_amount_poisha - concession_amount_poisha),
    due_date TEXT NOT NULL, -- YYYY-MM-DD
    source TEXT NOT NULL CHECK (source IN ('generated', 'manual')),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'voided')),
    void_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Prevent double charge generation for the same enrollment and period
CREATE UNIQUE INDEX IF NOT EXISTS idx_charges_unique_enrollment_period
ON charges(enrollment_id, fee_head_id, period)
WHERE source = 'generated' AND status = 'open' AND period IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_charges_student_status ON charges(student_id, status);

-- Payments received from students
CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY, -- UUIDv7
    receipt_no TEXT NOT NULL UNIQUE, -- allocated atomically from sequences 'receipt:<year>'
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'card', 'other')),
    reference TEXT,
    business_date TEXT NOT NULL, -- YYYY-MM-DD Asia/Dhaka civil date
    received_at TEXT NOT NULL, -- UTC ISO 8601 timestamp
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

-- Allocation of payment amounts to specific charges
CREATE TABLE IF NOT EXISTS payment_allocations (
    id TEXT PRIMARY KEY, -- UUIDv7
    payment_id TEXT NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    charge_id TEXT NOT NULL REFERENCES charges(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_payment_allocations_charge ON payment_allocations(charge_id);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_payment ON payment_allocations(payment_id);

-- Balance adjustments (waivers, manual fines, error corrections)
CREATE TABLE IF NOT EXISTS adjustments (
    id TEXT PRIMARY KEY, -- UUIDv7
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    charge_id TEXT REFERENCES charges(id) ON DELETE RESTRICT,
    type TEXT NOT NULL CHECK (type IN ('waiver', 'fine', 'correction')),
    amount_poisha INTEGER NOT NULL, -- signed poisha (negative for waiver, positive for fine)
    reason TEXT NOT NULL,
    approved_by TEXT,
    business_date TEXT NOT NULL, -- YYYY-MM-DD Asia/Dhaka civil date
    created_at TEXT NOT NULL
);

-- Refunds issued back to students
CREATE TABLE IF NOT EXISTS refunds (
    id TEXT PRIMARY KEY, -- UUIDv7
    payment_id TEXT NOT NULL REFERENCES payments(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'other')),
    reason TEXT NOT NULL,
    business_date TEXT NOT NULL, -- YYYY-MM-DD Asia/Dhaka civil date
    refunded_by TEXT,
    created_at TEXT NOT NULL
);

-- Daily cash drawer sessions
CREATE TABLE IF NOT EXISTS cash_days (
    business_date TEXT PRIMARY KEY, -- YYYY-MM-DD
    opening_cash_poisha INTEGER NOT NULL DEFAULT 0,
    cash_in_poisha INTEGER NOT NULL DEFAULT 0,
    cash_out_poisha INTEGER NOT NULL DEFAULT 0,
    expected_closing_poisha INTEGER NOT NULL DEFAULT 0,
    counted_closing_poisha INTEGER,
    difference_poisha INTEGER,
    note TEXT,
    closed_by TEXT,
    closed_at TEXT, -- UTC ISO 8601 timestamp
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Institutional expense categories
CREATE TABLE IF NOT EXISTS expense_categories (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Institutional expenses
CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY, -- UUIDv7
    category_id TEXT NOT NULL REFERENCES expense_categories(id) ON DELETE RESTRICT,
    amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0),
    method TEXT NOT NULL CHECK (method IN ('cash', 'bkash', 'nagad', 'bank', 'other')),
    payee TEXT NOT NULL,
    reference TEXT,
    business_date TEXT NOT NULL, -- YYYY-MM-DD Asia/Dhaka civil date
    attachment_vault_id TEXT,
    created_by TEXT,
    status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'voided')),
    void_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Fast read student balance cache (verified against ledger transactions)
CREATE TABLE IF NOT EXISTS student_balances (
    student_id TEXT PRIMARY KEY REFERENCES students(id) ON DELETE CASCADE,
    total_charged_poisha INTEGER NOT NULL DEFAULT 0,
    total_paid_poisha INTEGER NOT NULL DEFAULT 0,
    total_adjusted_poisha INTEGER NOT NULL DEFAULT 0,
    total_refunded_poisha INTEGER NOT NULL DEFAULT 0,
    balance_poisha INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
```

#### 5. Audit Log, Triggers, and Safety Rules

```sql
-- Append only audit trail with cryptographic hash chain
CREATE TABLE IF NOT EXISTS audit_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at TEXT NOT NULL, -- ISO 8601 UTC timestamp
    business_date TEXT NOT NULL, -- YYYY-MM-DD Asia/Dhaka civil date
    user_id TEXT,
    device_id TEXT,
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    before_json TEXT,
    after_json TEXT,
    prev_hash TEXT NOT NULL, -- 64 zeros for seq = 1, otherwise previous row hash
    hash TEXT NOT NULL -- SHA256(prev_hash || at || user_id || action || entity || entity_id || after_json)
);

-- Trigger: prevent updates to audit rows
CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_update
BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be updated');
END;

-- Trigger: prevent deletes on audit rows
CREATE TRIGGER IF NOT EXISTS trg_audit_log_no_delete
BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'Audit log records are immutable and cannot be deleted');
END;

-- Trigger: prevent backwards clock adjustments in clock_guard
CREATE TRIGGER IF NOT EXISTS trg_clock_guard_monotonic
BEFORE UPDATE ON clock_guard
WHEN NEW.high_water_at < OLD.high_water_at
BEGIN
    SELECT RAISE(ABORT, 'Clock guard violation: system time cannot move backwards');
END;

-- Trigger: automatic row version increment on students update
CREATE TRIGGER IF NOT EXISTS trg_students_row_version
AFTER UPDATE ON students
FOR EACH ROW
WHEN NEW.row_version = OLD.row_version
BEGIN
    UPDATE students SET row_version = OLD.row_version + 1 WHERE id = NEW.id;
END;

-- Trigger: prevent deletion of posted payments
CREATE TRIGGER IF NOT EXISTS trg_payments_no_delete
BEFORE DELETE ON payments
BEGIN
    SELECT RAISE(ABORT, 'Posted payments cannot be deleted; void the payment instead');
END;

-- Trigger: prevent editing posted payments (except voiding)
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

-- Trigger: prevent deletion of charges with payments
CREATE TRIGGER IF NOT EXISTS trg_charges_no_delete
BEFORE DELETE ON charges
BEGIN
    SELECT RAISE(ABORT, 'Charges are immutable; void the charge if unallocated');
END;

-- Trigger: prevent modifications to payment allocations
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

-- Trigger: prevent modifications to closed cash days
CREATE TRIGGER IF NOT EXISTS trg_cash_days_closed_immutable
BEFORE UPDATE ON cash_days
WHEN OLD.closed_at IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'Closed cash days are locked and cannot be modified');
END;
```

### State transitions

#### Student Lifecycle
- `active` -> `inactive`: Temporary pause in attendance or fee payments.
- `active` -> `transferred`: Student switched to another batch or branch.
- `active` -> `completed`: Student graduated or completed the course.
- `active` | `inactive` -> `archived`: Long term historical retention after student departs.

#### Payment Lifecycle
- `posted` -> `voided`: Voided on the same business day before cash close with a required explanation. Deletions are blocked by database triggers.

#### Charge Lifecycle
- `open` -> `voided`: Voided only when no payment allocations exist.

#### Cash Day Lifecycle
- Open (business day active, transactions recording) -> Closed (`closed_at` set, closing cash verified, day locked permanently against backdated writes).

### API surface

Database operations are executed through synchronous Rust repository functions in `crates/core/src/db.rs` and domain modules.

| Function | Access Mode | Key Inputs | Key Outputs | Guarantees |
|---|---|---|---|---|
| `open_database()` | Read Write | None | `Result<Connection, DbError>` | Configures WAL, synchronous FULL, foreign keys, busy timeout |
| `verify_integrity(&conn)` | Read Only | `&Connection` | `Result<(), DbError>` | Executes PRAGMA quick_check, errors on corruption |
| `run_in_transaction<F, T>(&mut conn, f)` | Read Write | `&mut Connection`, closure `FnOnce(&Transaction) -> Result<T, E>` | `Result<T, E>` | Executes `BEGIN IMMEDIATE`, commits on success, rolls back on error |
| `allocate_sequence(&tx, name, prefix)` | Read Write | `&Transaction`, `&str`, `&str` | `Result<String, DbError>` | Atomically increments sequence counter and formats sequence ID |
| `verify_clock(&tx, current_utc, max_forward_hours)` | Read Write | `&Transaction`, `DateTime`, `Option<u32>` | `Result<(), DbError>` | Checks and updates clock guard high water mark, rejects forward jumps over 24h without ack |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| Initialize database | PRAGMA configuration values | Explicitly configured in Rust `open_database()` function |
| Startup health check | Integrity status | Result of `PRAGMA quick_check` query |
| Insert student | `student_code` | Derived from `sequences` table via `allocate_sequence('student:<session_name>')` |
| Insert student | `created_at`, `updated_at` | Generated in Rust as UTC ISO 8601 string via `jiff::Timestamp::now()` |
| Insert student | `row_version` | Defaults to integer 1 on insert, automatically incremented by trigger `trg_students_row_version` |
| Receive payment | `receipt_no` | Derived from `sequences` table via `allocate_sequence('receipt:<calendar_year>')` |
| Receive payment | `business_date` | Computed in Rust domain logic from Asia/Dhaka civil date |
| Receive payment | `idempotency_key` | Generated in client when cashier payment form opens |
| Record audit event | `prev_hash` | Hash of immediately preceding row, or 64 zeros for first entry |
| Record audit event | `hash` | Computed in Rust SHA256 over previous row hash plus current entry attributes |

### Key invariants

1. **Integer money**: All financial columns store money as integer poisha (`i64` in Rust). One BDT equals 100 poisha. Floating point representations are prohibited.
2. **Time separation**: Operational timestamps record UTC instant strings (`created_at`, `at`), while operational business days are explicitly recorded in `business_date` columns (`YYYY-MM-DD`) mapped to Asia/Dhaka.
3. **Database immutability**: Audit log entries, posted payments, allocations, adjustments, refunds, and closed cash days are permanently protected by SQLite `BEFORE UPDATE` and `BEFORE DELETE` triggers executing `RAISE(ABORT)`.
4. **Power safety**: All write operations begin with `BEGIN IMMEDIATE` on connections running WAL mode with `synchronous = FULL`.
5. **Foreign key integrity**: `PRAGMA foreign_keys = ON` is enforced on every database connection.
6. **Unique human codes**: Sequence generated human codes (`student_code`, receipt numbers) are allocated atomically within write transactions without gaps.
7. **Monotonic time protection**: `clock_guard` updates prevent backwards clock drift, and reject forward jumps greater than 24 hours unless acknowledged with `last_ack_by`.
8. **Allocation limits**: For each payment, sum of allocations must not exceed amount minus refunds. For each charge, sum of allocations must not exceed net charge amount plus fines minus waivers.

### Security model

- The database file is stored locally on the host computer under the configured data directory (`%LOCALAPPDATA%\Coachify` in development, dedicated directory in production).
- Encryption is enforced via SQLCipher with keys managed in the Rust core.
- The UI layer has no direct database access; it interacts exclusively via the typed `rpc` boundary.
- Database triggers run inside the SQLite engine, preventing modification even if raw queries are executed directly on an unlocked connection.

### Configuration required

No external environment variables or third party credentials are required. Database PRAGMAs and paths are configured directly within `coachify_core`.

### Critical test scenarios

- Happy path: Opening a database connection sets WAL mode, synchronous FULL, and foreign keys ON, verifies **AC-1**.
- Startup integrity check: Valid database passes `verify_integrity()`, while an artificially modified header triggers a failure error, verifies **AC-2**.
- Comprehensive schema migration: Executing migration 0002 successfully creates all system, people, academic, and financial tables, verifies **AC-3**.
- Immutability trigger rejection: Updating or deleting an inserted row in `audit_log`, deleting a posted payment, or modifying payment allocations is rejected with an abort error, verifies **AC-4**.
- Clock guard backward protection: Attempting to update `clock_guard` with an earlier timestamp is rejected by the database trigger, verifies **AC-4**.
- Automatic row version increment: Updating a student row increments `row_version` automatically from 1 to 2, verifies **AC-3**.
- Financial integrity invariants: Money columns store integer poisha, and payments correctly reject negative or zero values, verifies **AC-5**.
- Date invariants: Business date filtering and UTC timestamps record clean civil date separation, verifies **AC-6**.

## Build plan

1. Update database connection configuration in `crates/core/src/db.rs` to enforce `synchronous = FULL`, `busy_timeout = 5000`, `foreign_keys = ON`, and implement `verify_integrity()`, satisfies **AC-1**, **AC-2**.
2. Create migration file `migrations/0002_foundation_schema.sql` implementing the complete schema for system, people, academic, and financial ledger tables, along with row version and immutability triggers, satisfies **AC-3**, **AC-4**.
3. Implement Rust database utilities for sequence allocation, transaction wrapping with `BEGIN IMMEDIATE`, and clock guard validation in `crates/core/src/db.rs`, satisfies **AC-1**, **AC-4**, **AC-5**.
4. Write unit and integration tests in `crates/core/tests/database_conventions.rs` verifying table creation, foreign key enforcement, unique constraints, and trigger abort behavior across people and financial tables, satisfies **AC-3**, **AC-4**, **AC-5**, **AC-6**.
5. Write simulated crash and rollback tests verifying that uncommitted transactions leave zero trace and database integrity remains valid, satisfies **AC-1**, **AC-2**.

## Consequences

**Positive**:
- A single comprehensive foundation and ledger schema prevents disruptive database migrations when building Slice 1 and Slice 2.
- Database records survive abrupt power cuts during load shedding without corruption.
- Defense in depth triggers prevent accidental or malicious updates to financial history and audit logs.
- Real school fields (birth registration numbers, blood group, school background) prevent later migration rework.
- Integer poisha and dual timestamp and business date conventions prevent rounding discrepancies and reporting timezone bugs.

**Negative / tradeoffs**:
- `synchronous = FULL` causes an extra fsync call on write commits, creating a minor performance cost on older spinning hard drives compared to `NORMAL`.
- Immutable triggers require all financial corrections to be modeled as distinct forward adjusting transactions.

**Neutral**:
- Feature 13 (Financial ledger model) and Feature 16 (Fast cashier) will connect business logic to the already prepared financial tables without altering the database schema.
- Migration runner will apply migration 0002 forward while preserving released migration 0001 immutability.

## Follow-up

- [ ] Implement migration runner engine in Feature 7 to automate forward migrations on startup with copy and swap validation.
- [ ] Connect the sequence allocator to the student admission flow in Feature 9 and cashier in Feature 16.
- [ ] Connect student photo storage to the document vault in Feature 9.
