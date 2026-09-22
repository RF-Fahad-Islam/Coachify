-- Migration 0014: Attendance and omnichannel messaging infrastructure
-- Tables for batch attendance, multi-channel outbox (SMS, WhatsApp, Email),
-- provider configs (OpenWA, SMTP, Greenweb, BulkSMS BD), delivery attempts,
-- message templates, and automation rules.

-- Attendance sessions header table
CREATE TABLE attendance_sessions (
    id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL REFERENCES batches(id) ON DELETE RESTRICT,
    date TEXT NOT NULL,
    taken_by TEXT NOT NULL REFERENCES users(id),
    taken_at TEXT NOT NULL,
    locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (batch_id, date)
);

CREATE INDEX idx_attendance_sessions_batch_date ON attendance_sessions(batch_id, date);

-- Attendance individual student marks table
CREATE TABLE attendance_marks (
    session_id TEXT NOT NULL REFERENCES attendance_sessions(id) ON DELETE CASCADE,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE RESTRICT,
    status TEXT NOT NULL CHECK (status IN ('present', 'absent', 'late', 'excused')),
    note TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (session_id, student_id)
);

CREATE INDEX idx_attendance_marks_student ON attendance_marks(student_id);

-- Provider configurations for SMS, WhatsApp (OpenWA), and Email (SMTP)
CREATE TABLE provider_configs (
    id TEXT PRIMARY KEY,
    channel TEXT NOT NULL CHECK (channel IN ('sms', 'whatsapp', 'email')),
    adapter TEXT NOT NULL CHECK (adapter IN ('generic_http', 'greenweb', 'bulksmsbd', 'openwa', 'smtp', 'mock')),
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    config_json TEXT NOT NULL DEFAULT '{}',
    secret_ref TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (channel, adapter)
);

-- Durable multi-channel outbox (SMS, WhatsApp, Email)
CREATE TABLE outbox (
    id TEXT PRIMARY KEY,
    channel TEXT NOT NULL CHECK (channel IN ('sms', 'whatsapp', 'email')),
    to_address TEXT NOT NULL,
    subject TEXT,
    body TEXT NOT NULL,
    segments INTEGER NOT NULL DEFAULT 1,
    encoding TEXT NOT NULL DEFAULT 'gsm' CHECK (encoding IN ('gsm', 'unicode', 'utf8')),
    template_key TEXT,
    related_type TEXT,
    related_id TEXT,
    dedupe_key TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN (
        'queued', 'sending', 'sent', 'failed', 'uncertain', 'cancelled'
    )),
    scheduled_at TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    provider_message_id TEXT,
    created_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL,
    sent_at TEXT
);

CREATE INDEX idx_outbox_status_scheduled ON outbox(status, scheduled_at);
CREATE INDEX idx_outbox_channel_status ON outbox(channel, status);
CREATE INDEX idx_outbox_related ON outbox(related_type, related_id);

-- Delivery attempts log
CREATE TABLE delivery_attempts (
    id TEXT PRIMARY KEY,
    outbox_id TEXT NOT NULL REFERENCES outbox(id) ON DELETE CASCADE,
    attempted_at TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('accepted', 'rejected', 'network_error', 'timeout')),
    http_status INTEGER,
    response_excerpt TEXT,
    duration_ms INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_delivery_attempts_outbox ON delivery_attempts(outbox_id);

-- Message templates catalog (supports SMS, WhatsApp, and Email)
CREATE TABLE message_templates (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('sms', 'whatsapp', 'email')),
    language TEXT NOT NULL DEFAULT 'en' CHECK (language IN ('en', 'bn')),
    title TEXT NOT NULL,
    subject TEXT,
    body TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (key, channel, language)
);

-- Automation trigger rules per channel
CREATE TABLE automation_rules (
    id TEXT PRIMARY KEY,
    trigger TEXT NOT NULL CHECK (trigger IN ('payment_received', 'student_absent', 'due_reminder')),
    channel TEXT NOT NULL CHECK (channel IN ('sms', 'whatsapp', 'email')),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    template_id TEXT NOT NULL REFERENCES message_templates(id),
    options_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL,
    UNIQUE (trigger, channel)
);

-- Triggers for orphaned outbox cancellation
CREATE TRIGGER trg_payments_cancel_outbox
AFTER UPDATE OF status ON payments
WHEN NEW.status = 'voided'
BEGIN
    UPDATE outbox
    SET status = 'cancelled'
    WHERE (dedupe_key = 'payment:' || NEW.id || ':receipt_sms'
       OR  dedupe_key = 'payment:' || NEW.id || ':receipt_whatsapp'
       OR  dedupe_key = 'payment:' || NEW.id || ':receipt_email')
      AND status = 'queued';
END;

CREATE TRIGGER trg_attendance_sessions_cancel_outbox
AFTER DELETE ON attendance_sessions
BEGIN
    UPDATE outbox
    SET status = 'cancelled'
    WHERE dedupe_key LIKE 'attendance:' || OLD.id || ':%'
      AND status = 'queued';
END;

-- Seed system templates for SMS
INSERT OR IGNORE INTO message_templates (id, key, channel, language, title, subject, body, is_system, created_at, updated_at) VALUES
    ('tmpl-payment-sms-en', 'payment_received', 'sms', 'en', 'Payment Received', NULL, 'Received Tk {amount} for {student_name} (Roll: {roll}). Receipt: {receipt_no}. Balance: Tk {balance}. {institution_name}', 1, datetime('now'), datetime('now')),
    ('tmpl-payment-sms-bn', 'payment_received', 'sms', 'bn', 'পেমেন্ট প্রাপ্তি', NULL, '{student_name} (রোল: {roll})-এর জন্য ৳{amount} জমা হয়েছে। রসিদ: {receipt_no}। বকেয়া: ৳{balance}। {institution_name}', 1, datetime('now'), datetime('now')),
    ('tmpl-absent-sms-en', 'student_absent', 'sms', 'en', 'Student Absent', NULL, 'Dear Guardian, {student_name} was absent today ({date}) in {batch_name}. Please contact {institution_phone}. {institution_name}', 1, datetime('now'), datetime('now')),
    ('tmpl-absent-sms-bn', 'student_absent', 'sms', 'bn', 'শিক্ষার্থী অনুপস্থিত', NULL, 'সম্মানিত অভিভাবক, {student_name} আজ ({date}) {batch_name} ক্লাসে অনুপস্থিত ছিল। প্রয়োজনে যোগাযোগ করুন: {institution_phone}। {institution_name}', 1, datetime('now'), datetime('now')),
    ('tmpl-due-sms-en', 'due_reminder', 'sms', 'en', 'Fee Due Reminder', NULL, 'Dear Guardian, fee of Tk {due_amount} for {period} is due for {student_name}. Please pay by {due_date}. {institution_name}', 1, datetime('now'), datetime('now')),
    ('tmpl-due-sms-bn', 'due_reminder', 'sms', 'bn', 'বকেয়া ফি স্মরণিকা', NULL, 'সম্মানিত অভিভাবক, {student_name}-এর {period} মাসের ৳{due_amount} ফি বকেয়া রয়েছে। অনুগ্রহ করে পরিশোধ করুন। {institution_name}', 1, datetime('now'), datetime('now'));

-- Seed system templates for WhatsApp (OpenWA rich text)
INSERT OR IGNORE INTO message_templates (id, key, channel, language, title, subject, body, is_system, created_at, updated_at) VALUES
    ('tmpl-payment-wa-en', 'payment_received', 'whatsapp', 'en', 'Payment Received (WhatsApp)', NULL, '🎓 *{institution_name}*\n\nDear Guardian,\nPayment received for *{student_name}* (Roll: {roll}).\n• *Amount Paid*: ৳{amount}\n• *Receipt No*: #{receipt_no}\n• *Current Balance*: ৳{balance}\n\nThank you!', 1, datetime('now'), datetime('now')),
    ('tmpl-payment-wa-bn', 'payment_received', 'whatsapp', 'bn', 'পেমেন্ট প্রাপ্তি (হোয়াটসঅ্যাপ)', NULL, '🎓 *{institution_name}*\n\nসম্মানিত অভিভাবক,\n*{student_name}* (রোল: {roll})-এর পেমেন্ট সফলভাবে জমা হয়েছে।\n• *জমা*: ৳{amount}\n• *রসিদ নম্বর*: #{receipt_no}\n• *বর্তমান বকেয়া*: ৳{balance}\n\nধন্যবাদ!', 1, datetime('now'), datetime('now')),
    ('tmpl-absent-wa-en', 'student_absent', 'whatsapp', 'en', 'Student Absent (WhatsApp)', NULL, '⚠️ *{institution_name} - Attendance Alert*\n\nDear Guardian,\nYour child *{student_name}* was marked *ABSENT* today ({date}) in *{batch_name}*.\n\nIf this was unexpected, please contact us at {institution_phone}.', 1, datetime('now'), datetime('now')),
    ('tmpl-absent-wa-bn', 'student_absent', 'whatsapp', 'bn', 'শিক্ষার্থী অনুপস্থিত (হোয়াটসঅ্যাপ)', NULL, '⚠️ *{institution_name} - উপস্থিতি সতর্কতা*\n\nসম্মানিত অভিভাবক,\nআপনার সন্তান *{student_name}* আজ ({date}) *{batch_name}* ক্লাসে অনুপস্থিত ছিল।\n\nজরুরি প্রয়োজনে যোগাযোগ করুন: {institution_phone}।', 1, datetime('now'), datetime('now')),
    ('tmpl-due-wa-en', 'due_reminder', 'whatsapp', 'en', 'Fee Due Reminder (WhatsApp)', NULL, '📋 *{institution_name} - Fee Reminder*\n\nDear Guardian,\nOutstanding dues of *৳{due_amount}* for *{period}* are pending for *{student_name}*.\n\nPlease clear the balance by {due_date}.\nHelpline: {institution_phone}', 1, datetime('now'), datetime('now')),
    ('tmpl-due-wa-bn', 'due_reminder', 'whatsapp', 'bn', 'বকেয়া ফি স্মরণিকা (হোয়াটসঅ্যাপ)', NULL, '📋 *{institution_name} - ফি স্মরণিকা*\n\nসম্মানিত অভিভাবক,\n*{student_name}*-এর *{period}* মাসের বকেয়া *৳{due_amount}* পরিশোধের জন্য বিনীত অনুরোধ করা হচ্ছে।\n\nপরিশোধের শেষ তারিখ: {due_date}।\nযোগাযোগ: {institution_phone}', 1, datetime('now'), datetime('now'));

-- Seed system templates for Email (HTML & plain-text subjects)
INSERT OR IGNORE INTO message_templates (id, key, channel, language, title, subject, body, is_system, created_at, updated_at) VALUES
    ('tmpl-payment-email-en', 'payment_received', 'email', 'en', 'Payment Receipt (Email)', 'Fee Payment Receipt - #{receipt_no} - {institution_name}', 'Dear Guardian,\n\nWe have received your tuition fee payment of BDT {amount} for {student_name} (Student ID: {student_code}, Roll: {roll}).\n\nReceipt Number: {receipt_no}\nPayment Date: {date}\nCurrent Outstanding Balance: BDT {balance}\n\nThank you for choosing {institution_name}.\n\nWarm regards,\n{institution_name} Accounts Office', 1, datetime('now'), datetime('now')),
    ('tmpl-absent-email-en', 'student_absent', 'email', 'en', 'Absence Notice (Email)', 'Daily Attendance Notice: {student_name} Absent on {date}', 'Dear Guardian,\n\nThis is an automated notification from {institution_name} to inform you that {student_name} was marked absent for class in {batch_name} on {date}.\n\nIf you have questions or this absence was excused, please reach out to us at {institution_phone}.\n\nSincerely,\n{institution_name} Academic Team', 1, datetime('now'), datetime('now')),
    ('tmpl-due-email-en', 'due_reminder', 'email', 'en', 'Fee Due Statement (Email)', 'Tuition Fee Due Notice for {period} - {student_name}', 'Dear Guardian,\n\nThis is a friendly reminder regarding the outstanding tuition fee of BDT {due_amount} for the period of {period} for {student_name}.\n\nPlease arrange for payment by {due_date} at our cashier counter.\n\nThank you for your cooperation.\n\n{institution_name}', 1, datetime('now'), datetime('now'));

-- Seed default automation rules across channels
INSERT OR IGNORE INTO automation_rules (id, trigger, channel, enabled, template_id, options_json, updated_at) VALUES
    ('rule-payment-sms', 'payment_received', 'sms', 1, 'tmpl-payment-sms-en', '{"quiet_hours_enabled":true}', datetime('now')),
    ('rule-payment-wa', 'payment_received', 'whatsapp', 1, 'tmpl-payment-wa-en', '{"quiet_hours_enabled":true}', datetime('now')),
    ('rule-payment-email', 'payment_received', 'email', 0, 'tmpl-payment-email-en', '{}', datetime('now')),
    ('rule-absent-sms', 'student_absent', 'sms', 1, 'tmpl-absent-sms-en', '{"delay_minutes":30,"quiet_hours_enabled":true}', datetime('now')),
    ('rule-absent-wa', 'student_absent', 'whatsapp', 1, 'tmpl-absent-wa-en', '{"delay_minutes":30,"quiet_hours_enabled":true}', datetime('now')),
    ('rule-due-sms', 'due_reminder', 'sms', 0, 'tmpl-due-sms-en', '{"min_due_poisha":10000,"quiet_hours_enabled":true}', datetime('now')),
    ('rule-due-wa', 'due_reminder', 'whatsapp', 0, 'tmpl-due-wa-en', '{"min_due_poisha":10000,"quiet_hours_enabled":true}', datetime('now')),
    ('rule-due-email', 'due_reminder', 'email', 0, 'tmpl-due-email-en', '{"min_due_poisha":10000}', datetime('now'));

-- Seed permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('attendance.mark', 'academics', 'Record and update batch attendance sessions'),
    ('attendance.edit_previous', 'academics', 'Edit locked or past-dated attendance records'),
    ('attendance.view', 'academics', 'View attendance registers and student summaries'),
    ('comms.view', 'communications', 'View communications outbox history and delivery logs'),
    ('comms.send', 'communications', 'Queue and trigger multi-channel messages'),
    ('comms.manage', 'communications', 'Configure SMS, WhatsApp, and Email gateways and retry uncertain messages');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN (
    SELECT 'attendance.mark' AS code
    UNION ALL SELECT 'attendance.edit_previous'
    UNION ALL SELECT 'attendance.view'
    UNION ALL SELECT 'comms.view'
    UNION ALL SELECT 'comms.send'
    UNION ALL SELECT 'comms.manage'
) p
WHERE r.name IN ('Owner', 'Admin');

-- Grant Teacher role permissions
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT r.id, p.code
FROM roles r
CROSS JOIN (
    SELECT 'attendance.mark' AS code
    UNION ALL SELECT 'attendance.view'
) p
WHERE r.name = 'Teacher';

-- Record schema version
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (14, datetime('now'));
