-- Migration 0006: Audit log and defense in depth triggers

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

-- Triggers for audit_log
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

-- Trigger for clock_guard
CREATE TRIGGER IF NOT EXISTS trg_clock_guard_monotonic
BEFORE UPDATE ON clock_guard
WHEN NEW.high_water_at < OLD.high_water_at
BEGIN
    SELECT RAISE(ABORT, 'Clock guard violation: system time cannot move backwards');
END;

-- Trigger for students row_version
CREATE TRIGGER IF NOT EXISTS trg_students_row_version
AFTER UPDATE ON students
FOR EACH ROW
WHEN NEW.row_version = OLD.row_version
BEGIN
    UPDATE students SET row_version = OLD.row_version + 1 WHERE id = NEW.id;
END;

-- Triggers for payments
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

-- Triggers for charges
CREATE TRIGGER IF NOT EXISTS trg_charges_no_delete
BEFORE DELETE ON charges
BEGIN
    SELECT RAISE(ABORT, 'Charges are immutable; void the charge if unallocated');
END;

CREATE TRIGGER IF NOT EXISTS trg_charges_no_update
BEFORE UPDATE ON charges
WHEN OLD.gross_amount_poisha != NEW.gross_amount_poisha OR
     OLD.concession_amount_poisha != NEW.concession_amount_poisha OR
     OLD.net_amount_poisha != NEW.net_amount_poisha OR
     OLD.student_id != NEW.student_id OR
     OLD.fee_head_id != NEW.fee_head_id OR
     OLD.period IS NOT NEW.period
BEGIN
    SELECT RAISE(ABORT, 'Charge amounts and key attributes cannot be altered');
END;

-- Triggers for payment allocations
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

-- Triggers for adjustments
CREATE TRIGGER IF NOT EXISTS trg_adjustments_no_update
BEFORE UPDATE ON adjustments
BEGIN
    SELECT RAISE(ABORT, 'Adjustments are immutable and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS trg_adjustments_no_delete
BEFORE DELETE ON adjustments
BEGIN
    SELECT RAISE(ABORT, 'Adjustments are immutable and cannot be deleted');
END;

-- Triggers for refunds
CREATE TRIGGER IF NOT EXISTS trg_refunds_no_update
BEFORE UPDATE ON refunds
BEGIN
    SELECT RAISE(ABORT, 'Refunds are immutable and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS trg_refunds_no_delete
BEFORE DELETE ON refunds
BEGIN
    SELECT RAISE(ABORT, 'Refunds are immutable and cannot be deleted');
END;

-- Trigger for closed cash days
CREATE TRIGGER IF NOT EXISTS trg_cash_days_closed_immutable
BEFORE UPDATE ON cash_days
WHEN OLD.closed_at IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'Closed cash days are locked and cannot be modified');
END;

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (6, datetime('now'));
