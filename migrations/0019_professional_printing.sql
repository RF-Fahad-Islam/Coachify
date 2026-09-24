-- Migration 0019: Professional Printing System (Spec 0035, Feature 50)
-- Configurable print templates per document type, print job run history, and default template seeds.

-- 1. Print Template Configurations
CREATE TABLE IF NOT EXISTS print_template_configs (
    id TEXT PRIMARY KEY NOT NULL,
    document_type TEXT NOT NULL UNIQUE CHECK (document_type IN (
        'receipt', 'due_statement', 'due_slip_thermal', 'attendance_sheet',
        'id_card', 'admit_card', 'mark_sheet', 'tabulation_sheet',
        'report_card', 'financial_ledger', 'cash_close_thermal'
    )),
    paper_size TEXT NOT NULL DEFAULT 'a4' CHECK (paper_size IN (
        'a4', 'a5', 'thermal_80mm', 'id_card_pvc'
    )),
    orientation TEXT NOT NULL DEFAULT 'portrait' CHECK (orientation IN (
        'portrait', 'landscape'
    )),
    show_header INTEGER NOT NULL DEFAULT 1 CHECK (show_header IN (0, 1)),
    show_logo INTEGER NOT NULL DEFAULT 1 CHECK (show_logo IN (0, 1)),
    show_watermark INTEGER NOT NULL DEFAULT 0 CHECK (show_watermark IN (0, 1)),
    custom_header_text TEXT,
    custom_footer_text TEXT,
    signature_title TEXT DEFAULT 'Authorized Signature',
    options_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL,
    updated_by_user_id TEXT REFERENCES users(id)
);

-- Seed initial print template configs for all 11 document types
INSERT OR IGNORE INTO print_template_configs (
    id, document_type, paper_size, orientation, show_header, show_logo, show_watermark, custom_header_text, custom_footer_text, signature_title, options_json, updated_at
) VALUES
('ptc-receipt', 'receipt', 'thermal_80mm', 'portrait', 1, 1, 0, NULL, 'Thank you for your payment. Please preserve this receipt for future reference.', 'Cashier', '{"thermal_cut_line": true, "show_qr_code": true}', datetime('now')),
('ptc-due-stmt', 'due_statement', 'a4', 'portrait', 1, 1, 0, NULL, 'Please clear all outstanding dues by the specified due date.', 'Accounts Officer', '{"include_recent_payments": true, "show_account_summary": true}', datetime('now')),
('ptc-due-slip', 'due_slip_thermal', 'thermal_80mm', 'portrait', 1, 1, 0, NULL, 'Kindly settle your outstanding balance at the counter.', 'Cashier', '{"compact_mode": true, "show_barcode": true}', datetime('now')),
('ptc-attendance', 'attendance_sheet', 'a4', 'landscape', 1, 1, 0, NULL, 'Official coaching attendance roster. Verified by batch teacher.', 'Teacher / Instructor', '{"show_roll_numbers": true, "include_notes_column": true}', datetime('now')),
('ptc-id-card', 'id_card', 'a4', 'portrait', 1, 1, 0, NULL, 'Student must carry this identity card while on coaching premises.', 'Principal / Director', '{"layout_mode": "a4_sheet", "cards_per_sheet": 8, "show_blood_group": true, "show_barcode": true}', datetime('now')),
('ptc-admit-card', 'admit_card', 'a4', 'portrait', 1, 1, 0, NULL, 'Students must bring this admit card to the examination hall.', 'Controller of Examinations', '{"layout_mode": "a4_dual", "show_exam_routine": true, "show_qr_code": true}', datetime('now')),
('ptc-mark-sheet', 'mark_sheet', 'a4', 'landscape', 1, 1, 0, NULL, 'Official assessment evaluation marksheet.', 'Head of Department', '{"show_grading_scale": true, "highlight_fails": true}', datetime('now')),
('ptc-tabulation', 'tabulation_sheet', 'a4', 'landscape', 1, 1, 0, NULL, 'Consolidated exam tabulation sheet approved by academic board.', 'Controller of Examinations', '{"show_subject_ranks": true, "include_statistics": true}', datetime('now')),
('ptc-report-card', 'report_card', 'a4', 'portrait', 1, 1, 0, NULL, 'Academic progress report for student guardian review.', 'Principal / Director', '{"show_teacher_remarks": true, "show_grade_breakdown": true}', datetime('now')),
('ptc-fin-ledger', 'financial_ledger', 'a4', 'portrait', 1, 1, 0, NULL, 'Official institutional financial audit ledger.', 'Accounts Officer', '{"show_audit_trail": true, "include_closing_balance": true}', datetime('now')),
('ptc-cash-close', 'cash_close_thermal', 'thermal_80mm', 'portrait', 1, 1, 0, NULL, 'Daily counter cash settlement sheet.', 'Cashier', '{"show_payment_methods": true, "show_expense_summary": true}', datetime('now'));

-- 2. Print job run audit history
CREATE TABLE IF NOT EXISTS print_job_runs (
    id TEXT PRIMARY KEY NOT NULL,
    document_type TEXT NOT NULL,
    batch_mode INTEGER NOT NULL DEFAULT 0 CHECK (batch_mode IN (0, 1)),
    item_count INTEGER NOT NULL DEFAULT 1,
    printer_name TEXT,
    paper_size TEXT NOT NULL,
    created_at TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN (
        'completed', 'failed', 'cancelled'
    )),
    metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_print_job_runs_type_date ON print_job_runs(document_type, created_at DESC);

-- 3. Bump schema version to 19
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (19, datetime('now'));
