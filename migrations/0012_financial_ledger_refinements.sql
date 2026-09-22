-- Migration 0012: Financial ledger refinements, concession batch scoping, idempotency unique index, and safety triggers

-- 1. Add batch_id to concessions for batch-specific concession scoping (AC-3)
ALTER TABLE concessions ADD COLUMN batch_id TEXT REFERENCES batches(id) ON DELETE CASCADE;

-- Unique index preventing duplicate active concessions on the same fee head and batch (AC-3)
CREATE UNIQUE INDEX IF NOT EXISTS idx_concessions_unique_active
ON concessions(student_id, fee_head_id, COALESCE(batch_id, 'ALL'));

-- 2. Refine charges unique index for generated charges (AC-4)
-- Omits status = 'open' so voided generated charges are never recreated
DROP INDEX IF EXISTS idx_charges_unique_enrollment_period;
CREATE UNIQUE INDEX IF NOT EXISTS idx_charges_unique_enrollment_period
ON charges(enrollment_id, fee_head_id, period)
WHERE source = 'generated' AND period IS NOT NULL;

-- 3. Trigger: prevent adjustment with non-positive amount
CREATE TRIGGER IF NOT EXISTS trg_adjustments_positive_amount
BEFORE INSERT ON adjustments
WHEN NEW.amount_poisha <= 0
BEGIN
    SELECT RAISE(ABORT, 'Adjustment amount must be positive');
END;

-- 4. Trigger: enforce waiver ceiling on charge-linked waivers (AC-7)
CREATE TRIGGER IF NOT EXISTS trg_adjustments_waiver_ceiling
BEFORE INSERT ON adjustments
WHEN NEW.type = 'waiver' AND NEW.charge_id IS NOT NULL
BEGIN
    SELECT CASE
        WHEN NEW.amount_poisha > (
            SELECT (
                c.net_amount_poisha
                - COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    JOIN payments p ON pa.payment_id = p.id
                    WHERE pa.charge_id = NEW.charge_id AND p.status = 'posted'
                ), 0)
                - COALESCE((
                    SELECT SUM(a.amount_poisha)
                    FROM adjustments a
                    WHERE a.charge_id = NEW.charge_id AND a.type = 'waiver'
                ), 0)
            )
            FROM charges c
            WHERE c.id = NEW.charge_id
        )
        THEN RAISE(ABORT, 'Waiver amount exceeds remaining unallocated dues on this charge')
    END;
END;

-- 5. Trigger: prevent voiding charges that have active payment allocations (AC-7)
CREATE TRIGGER IF NOT EXISTS trg_charges_prevent_void_with_allocations
BEFORE UPDATE ON charges
WHEN OLD.status = 'open' AND NEW.status = 'voided'
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1
            FROM payment_allocations pa
            JOIN payments p ON pa.payment_id = p.id
            WHERE pa.charge_id = OLD.id AND p.status = 'posted'
        )
        THEN RAISE(ABORT, 'Cannot void charge with active payment allocations')
    END;
END;

-- 6. Trigger: enforce refund ceiling on payments (AC-7)
CREATE TRIGGER IF NOT EXISTS trg_refunds_ceiling
BEFORE INSERT ON refunds
BEGIN
    SELECT CASE
        WHEN (SELECT status FROM payments WHERE id = NEW.payment_id) != 'posted'
        THEN RAISE(ABORT, 'Cannot refund an unposted or voided payment')
    END;

    SELECT CASE
        WHEN NEW.amount_poisha > (
            SELECT (
                p.amount_poisha
                - COALESCE((
                    SELECT SUM(pa.amount_poisha)
                    FROM payment_allocations pa
                    WHERE pa.payment_id = NEW.payment_id
                ), 0)
                - COALESCE((
                    SELECT SUM(r.amount_poisha)
                    FROM refunds r
                    WHERE r.payment_id = NEW.payment_id
                ), 0)
            )
            FROM payments p
            WHERE p.id = NEW.payment_id
        )
        THEN RAISE(ABORT, 'Refund amount exceeds unallocated payment funds')
    END;
END;

-- 7. Seed Money & Finance RBAC permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('finance.plans_manage', 'money', 'Create and modify fee heads and fee plans'),
    ('finance.concessions_manage', 'money', 'Grant and manage student fee concessions'),
    ('charges.generate', 'money', 'Generate monthly fee charges for enrollments'),
    ('charges.manage', 'money', 'Create manual charges and void unallocated charges'),
    ('payments.void', 'money', 'Void posted payments on same open cash day'),
    ('ledger.view', 'money', 'View student financial ledgers, balances, and statements'),
    ('finance.adjust', 'money', 'Create adjustments (waivers, fines) and process refunds');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'finance.plans_manage' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'finance.concessions_manage' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'charges.generate' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'charges.manage' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'payments.void' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'ledger.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'finance.adjust' FROM roles WHERE name IN ('Owner', 'Admin');

-- 8. Performance indexes for financial ledger queries and invariant validation
CREATE INDEX IF NOT EXISTS idx_charges_student_status ON charges(student_id, status);
CREATE INDEX IF NOT EXISTS idx_payments_student_status ON payments(student_id, status);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_payment_id ON payment_allocations(payment_id);
CREATE INDEX IF NOT EXISTS idx_payment_allocations_charge_id ON payment_allocations(charge_id);
CREATE INDEX IF NOT EXISTS idx_adjustments_student_type ON adjustments(student_id, type);
CREATE INDEX IF NOT EXISTS idx_adjustments_charge_type ON adjustments(charge_id, type);
CREATE INDEX IF NOT EXISTS idx_refunds_payment_id ON refunds(payment_id);

-- 9. Record schema version 12
INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (12, datetime('now'));
