-- Migration 0002: System and Configuration tables

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

-- Academic years or sessions
CREATE TABLE IF NOT EXISTS academic_sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Monotonic clock guard with protection against backwards shifts
CREATE TABLE IF NOT EXISTS clock_guard (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    high_water_at TEXT NOT NULL,
    last_ack_by TEXT,
    last_ack_at TEXT
);

-- Atomic sequence counters for human readable identifiers
CREATE TABLE IF NOT EXISTS sequences (
    name TEXT PRIMARY KEY,
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

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (2, datetime('now'));
