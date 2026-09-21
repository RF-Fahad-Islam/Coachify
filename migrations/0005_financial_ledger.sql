-- Migration 0005: Financial ledger (Fee heads, Fee plans, Concessions, Charges, Payments, Allocations, Adjustments, Refunds, Cash days, Expenses, Balances)

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

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (5, datetime('now'));
