# 0012. Financial ledger model and fee plans

**Date**: 2026-09-22
**Status**: Accepted

## Summary

Coachify manages tuition fees and institute accounting through an offline first, append only financial ledger. The system replaces mutable payment totals and complex invoice headers with immutable charges, payments, allocations, adjustments, and refunds. Every monetary value is stored as an integer number of poisha, preventing floating point rounding drift.

Tuition rates are configured through fee plans bound to programs or batches, with individual student concessions for discounts and waivers. A monthly charge generation engine creates fee demands idempotently in safe chunks of one hundred students, guaranteeing that duplicate runs never create double charges and intentionally voided charges are not recreated. Any advance credit on a student account is applied automatically to newly generated charges in oldest first order with a clear fee head priority, keeping student balances accurate without manual cashier calculations. Posted records are protected from alteration or deletion by database triggers, and a property test suite verifies that one hundred thousand random transactions produce zero invariant violations.

## Requirements

**User stories**:
- As an institute director, I want to define standard fee plans for programs and batches, so that tuition rates are consistent and transparent.
- As an administrator, I want to grant percentage or fixed concessions to specific students with optional batch scoping, so that scholarships and sibling discounts apply automatically to future fees.
- As an accountant, I want an idempotent monthly fee generator, so that running billing for a month creates exact charges without accidental duplicates even if interrupted by a power failure.
- As a cashier, I want advance payments held as credit and applied automatically to new charges, so that parents who pay ahead do not show false dues.
- As an institute owner, I want all financial records to be immutable and guarded by database triggers, so that no staff member or software bug can rewrite financial history.
- As a software engineer, I want the ledger verified by property testing across one hundred thousand randomized operations, so that balance calculations remain mathematically proven.

**Acceptance criteria**:
- **AC-1**: Fee heads catalog defines charge categories (`tuition`, `admission`, `exam`, `registration`, `material`, `fine`, `other`) with bilingual English and Bengali names, preventing deletion of fee heads referenced by existing plans or charges.
- **AC-2**: Fee plans configure standard amounts in integer poisha for a program or batch with frequency (`once` or `monthly`) and effective date validity periods (`effective_from` to `effective_to`).
- **AC-3**: Concessions support percentage (in integer basis points) or fixed amount discounts on specific fee heads for a student, with optional batch restriction and optional validity period (`from_period` to `to_period`). Percentage calculations round to the nearest integer poisha rounding half up. Concessions are clamped so net charges cannot fall below zero. A batch specific concession overrides any student wide concession on the same fee head for that batch. Multiple active concessions on the exact same fee head and batch are disallowed.
- **AC-4**: Monthly charge generator runs idempotently for a target billing period (for example `2026-09`) in chunked transactions of one hundred students. Running generation repeatedly creates zero duplicate charges, and voided charges are never recreated, enforced by the unique index on `(enrollment_id, fee_head_id, period) WHERE source = 'generated' AND period IS NOT NULL`.
- **AC-5**: Mid month enrollments generate a full month tuition charge for their initial billing period, matching local coaching center business rules and keeping accounting straightforward.
- **AC-6**: Advance credit held in unallocated payments is applied automatically to newly created charges in the generation transaction, prioritizing oldest open charges first by due date, breaking ties by full fee head hierarchy: `tuition`, then `admission`, then `exam`, then `registration`, then `material`, then `other`, and finally `fine`.
- **AC-7**: Posted payments, charges, allocations, adjustments, and refunds are immutable. In place updates to amounts, dates, or student links and hard deletes are rejected by SQLite database triggers with `RAISE(ABORT)`. Voided payment allocations are neutralized because all allocation and balance queries join on `payments.status = 'posted'`. A charge specific waiver cannot exceed remaining unallocated dues (`net_amount_poisha - allocated_amount_poisha`).
- **AC-8**: Atomic monotonic receipt numbers are issued from an institution sequence (for example `RCP-2026-0001`), ensuring no skipped sequence numbers without a posted or voided payment record.
- **AC-9**: Student balance cache in `student_balances` is updated synchronously in the same transaction as every financial mutation, maintaining the invariant that cached balance equals open charges plus fines minus waivers minus posted payments plus refunds.
- **AC-10**: Core test suite includes a property test using the `proptest` crate that executes 100,000 random state machine operations (charges, payments, allocations, adjustments, refunds, voids) with zero invariant violations and zero balance drift.

## Decision

**Chosen option**: Option 1: Append Only Relational Ledger with Synchronous Balance Cache, Robust Partial Index Idempotency, and Proptest State Machine Verification.

You build the Coachify financial ledger and fee engine across `crates/core/src/ledger.rs` (ledger repository, balance calculations, and invariants), `crates/core/src/fees.rs` (fee heads, fee plans, concessions, and chunked monthly generation), migration `0012_financial_ledger_refinements.sql` (concession batch scoping, unconditional generated charge uniqueness, and trigger guards), and `tests/ledger_proptest.rs` (100,000 transaction property verification).

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

The foundational financial schema exists in migration `0005_financial_ledger.sql`, with trigger enforcement in `0006_audit_and_triggers.sql`. Migration `0012_financial_ledger_refinements.sql` refines the idempotency index and concession scoping.

```sql
-- Core schema in migrations/0012_financial_ledger_refinements.sql

-- 1. Concessions table refined with optional batch restriction
CREATE TABLE IF NOT EXISTS concessions (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    batch_id TEXT REFERENCES batches(id) ON DELETE CASCADE,
    fee_head_id TEXT NOT NULL REFERENCES fee_heads(id) ON DELETE RESTRICT,
    type TEXT NOT NULL CHECK (type IN ('percent', 'fixed')),
    value INTEGER NOT NULL CHECK (value > 0),
    from_period TEXT,
    to_period TEXT,
    reason TEXT NOT NULL,
    approved_by TEXT REFERENCES users(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Unique constraint preventing ambiguous duplicate concessions
CREATE UNIQUE INDEX IF NOT EXISTS idx_concessions_unique_active
ON concessions(student_id, fee_head_id, COALESCE(batch_id, 'ALL'));

-- 2. Charges table unique index refined
-- Notice: omits status = 'open' so voided generated charges are never recreated
DROP INDEX IF EXISTS idx_charges_unique_enrollment_period;
CREATE UNIQUE INDEX IF NOT EXISTS idx_charges_unique_enrollment_period
ON charges(enrollment_id, fee_head_id, period)
WHERE source = 'generated' AND period IS NOT NULL;
```

### State transitions

Financial records follow strict unidirectional state machines:

1. **Charge lifecycle**:
   - `open`: charge is created by the generator or manually, awaiting payment or allocation.
   - `voided`: unallocated charge cancelled by administrative action. Transition is only allowed when sum of allocations for this charge is zero. A charge with allocations cannot be voided.
   - Terminal state: once voided, a charge cannot be reopened. The unique index prevents regeneration for that period.

2. **Payment lifecycle**:
   - `posted`: payment recorded, receipt number minted, cash received, funds allocated or credited.
   - `voided`: same day cancellation before cash day close. Payment status changes to `voided`.
   - Allocation treatment: all balance and ledger queries filter on `payments.status = 'posted'`. Voiding a payment immediately decouples its allocations from dues and balances without mutating the immutable `payment_allocations` rows.

### API surface

The core exposes typed RPC commands through the dispatcher:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `fee_heads.list` | RPC | none | `items: FeeHeadItem[]` | `finance.plans_manage` | `unauthorized` |
| `fee_heads.create` | RPC | `name: string`, `name_bn?: string`, `kind: string` | `head: FeeHeadItem` | `finance.plans_manage` | `invalid_kind`, `duplicate_name` |
| `fee_plans.list` | RPC | `program_id?: string`, `batch_id?: string` | `items: FeePlanItem[]` | `finance.plans_manage` | `invalid_scope` |
| `fee_plans.create` | RPC | `fee_head_id: string`, `amount_poisha: number`, `frequency: string`, `effective_from: string` | `plan: FeePlanItem` | `finance.plans_manage` | `overlapping_plan`, `invalid_amount` |
| `concessions.list` | RPC | `student_id?: string`, `batch_id?: string` | `items: ConcessionItem[]` | `finance.concessions_manage` | `student_not_found` |
| `concessions.create` | RPC | `student_id: string`, `batch_id?: string`, `fee_head_id: string`, `type: string`, `value: number`, `reason: string` | `concession: ConcessionItem` | `finance.concessions_manage` | `concession_conflict`, `invalid_value` |
| `charges.generate_monthly` | RPC | `period: string`, `batch_id?: string` | `generated_count: number`, `allocated_credit_poisha: number` | `charges.generate` | `invalid_period`, `batch_not_found` |
| `charges.create_manual` | RPC | `student_id: string`, `fee_head_id: string`, `amount_poisha: number`, `due_date: string` | `charge: ChargeItem` | `charges.manage` | `student_not_found`, `invalid_amount` |
| `charges.void` | RPC | `charge_id: string`, `reason: string` | `success: boolean` | `charges.manage` | `charge_has_allocations`, `charge_not_found` |
| `ledger.get_student_summary` | RPC | `student_id: string` | `summary: StudentLedgerSummary` | `ledger.view` | `student_not_found` |
| `payments.create` | RPC | `student_id: string`, `amount_poisha: number`, `method: string`, `idempotency_key: string` | `payment: PaymentReceipt` | `payments.collect` | `duplicate_idempotency`, `closed_cash_day` |
| `payments.void` | RPC | `payment_id: string`, `reason: string` | `success: boolean` | `payments.void` | `closed_cash_day`, `unauthorized_void` |
| `adjustments.create` | RPC | `student_id: string`, `charge_id?: string`, `type: string`, `amount_poisha: number`, `reason: string` | `adjustment: AdjustmentItem` | `finance.adjust` | `waiver_exceeds_unallocated`, `charge_not_found` |
| `refunds.create` | RPC | `payment_id: string`, `amount_poisha: number`, `reason: string` | `refund: RefundItem` | `finance.adjust` | `refund_exceeds_unallocated`, `payment_not_found` |

### Value sourcing

Every required value traces to a concrete source:

| Action | Value produced or displayed | Source |
|---|---|---|
| Monthly fee generation | `gross_amount_poisha` | Matching active `fee_plans.amount_poisha` for batch or program |
| Monthly fee generation | `concession_amount_poisha` | Evaluated from active `concessions`, rounded to nearest poisha, clamped to gross |
| Monthly fee generation | `net_amount_poisha` | Computed as `max(0, gross_amount_poisha - concession_amount_poisha)` |
| Monthly fee generation | `due_date` | First business day of the target month from institution calendar rules |
| Advance credit application | Allocated payment funds | Oldest unallocated payments joining on `status = 'posted'` and sorted by date |
| Payment collection | `receipt_no` | Atomic monotonic allocation from `sequences` table for current year |
| Payment collection | `business_date` | Institution active business date resolved from session context |
| Student balance lookup | `balance_poisha` | Derived from `student_balances` cache, checked against live ledger sum |

### Key invariants

1. **Payment allocation bounded**: For every payment record, the sum of active allocations (joining on `payments.status = 'posted'`) plus `refunds.amount_poisha` cannot exceed `payments.amount_poisha`.
2. **Charge allocation bounded**: For every charge record, the sum of active allocations cannot exceed `charges.net_amount_poisha` plus fine adjustments minus waiver adjustments on that charge.
3. **Waiver ceiling**: A waiver adjustment attached to a charge cannot exceed `net_amount_poisha - allocated_amount_poisha`, preventing negative net dues on partially paid charges.
4. **Balance consistency**: For every student, `student_balances.balance_poisha` equals total open net charges plus fine adjustments minus waiver adjustments minus posted payments plus refunds.
5. **Receipt sequence density**: Receipt numbers are strictly monotonic and unique per year. Every generated sequence value corresponds to a posted or voided payment.
6. **No mutation of posted history**: Triggers `trg_payments_no_update`, `trg_payments_no_delete`, `trg_charges_no_update`, and `trg_charges_no_delete` reject any direct modification of posted rows.
7. **Closed day seal**: No payment, adjustment, refund, or void can carry a business date belonging to a closed cash day.
8. **Idempotent generation**: Re-executing charge generation for the same enrollment, fee head, and period produces exactly zero new charge rows, even if prior charges were voided.
9. **Chunked generation concurrency**: Generation processes in chunks of 100 students per transaction, releasing database locks between chunks.

### Security model

Access is governed by the centralized command dispatcher:
- `finance.plans_manage`: granted to Owners and Institute Administrators. Allows creating and editing fee heads and plans.
- `finance.concessions_manage`: granted to Owners and Headmasters. Allows granting student discounts.
- `charges.generate`: granted to Owners and Accountants. Allows running monthly billing generation.
- `charges.manage`: granted to Owners and Accountants. Allows creating manual one off charges or voiding unallocated charges.
- `payments.collect`: granted to Cashiers and Receptionists. Allows taking student fees and printing receipts.
- `payments.void`: restricted to Owners and Center Managers. Allows voiding a payment on the same open cash day.
- `ledger.view`: granted to all authenticated staff members. Allows inspecting student dues and ledgers.
- `finance.adjust`: restricted to Owners and Center Managers. Allows waivers, fines, and refunds.

### Critical test scenarios

- Happy path: define monthly tuition fee plan for batch, create enrollment, run monthly charge generator, record cash payment allocating full net charge, verify student balance becomes zero, verifies **AC-1**, **AC-2**, **AC-4**, **AC-9**.
- Concession deduction and rounding: configure 15 percent concession on 2,500 BDT (250,000 poisha) tuition fee, verify discount is exactly 37,500 poisha with net charge 212,500 poisha. Test odd amount 33.33 percent on 1,000 poisha and verify half up rounding to 333 poisha, verifies **AC-3**.
- Concession override: grant student wide 10 percent concession on tuition, grant batch specific 20 percent concession for Batch A, confirm that Batch A generates with 20 percent discount while Batch B generates with 10 percent discount, verifies **AC-3**.
- Advance credit priority: student pays 5,000 poisha in advance with no open dues, charge generator creates 3,000 poisha tuition charge, 1,000 poisha exam charge, and 500 poisha registration charge, credit automatically allocates 3,000 to tuition, 1,000 to exam, and 500 to registration, leaving 500 remaining credit, verifies **AC-6**.
- Mid month join: student admitted on the twentieth of September, fee generator creates full month charge for September, verifies **AC-5**.
- Generation idempotency and void safety: monthly generator runs creating charges, one charge is voided with reason, generator is run again for identical period, confirms that zero new charges are created and the voided charge is not recreated, verifies **AC-4**.
- Voided payment allocation decoupling: payment is collected and allocated to a charge, payment is voided, confirm that charge returns to fully unpaid status and allocations are ignored in balance queries without deleting allocation rows, verifies **AC-7**, **AC-9**.
- Waiver ceiling enforcement: charge has net amount 200,000 poisha, 150,000 poisha is paid and allocated, attempt to post waiver for 60,000 poisha, verify system rejects waiver because it exceeds unallocated 50,000 poisha, verifies **AC-7**.
- Receipt sequence continuity: three concurrent payments processed, each receives distinct strictly incremented receipt number without gaps, verifies **AC-8**.
- Property test verification: proptest executes one hundred thousand random state transitions across charges, payments, allocations, waivers, and refunds, asserting all nine invariants after every step, verifies **AC-10**.

## Build plan

Work follows the project Tracer Bullet approach, standing up a thin vertical slice through the ledger first, then expanding to fee plans and generation:

1. Create migration `0012_financial_ledger_refinements.sql` with batch scoped concessions, unconditional generated charge uniqueness index, and sequence triggers, satisfies **AC-1**, **AC-3**, **AC-4**, **AC-7**.
2. Implement core integer poisha math utilities, half up rounding, fee head repository, and fee plan models in `crates/core/src/fees.rs`, satisfies **AC-1**, **AC-2**, **AC-3**.
3. Implement student concession evaluation engine with batch override hierarchy and mid month join rules in `crates/core/src/fees.rs`, satisfies **AC-3**, **AC-5**.
4. Implement chunked idempotent monthly charge generation engine processing one hundred students per transaction, satisfies **AC-4**.
5. Implement advance credit allocation engine applying unallocated funds oldest first by due date and full fee head priority hierarchy, satisfies **AC-6**.
6. Implement atomic payment recording, annual receipt sequence allocation, and synchronous balance cache updater in `crates/core/src/ledger.rs`, satisfies **AC-7**, **AC-8**, **AC-9**.
7. Implement adjustments (waivers with ceiling check, fines), voids, and refund handlers with immutability trigger validation, satisfies **AC-7**, **AC-9**.
8. Build comprehensive property based test suite in `tests/ledger_proptest.rs` running 100,000 randomized operations verifying ledger invariants, satisfies **AC-10**.
9. Expose RPC commands across `crates/core/src/dispatcher.rs` with RBAC permissions and export TypeScript contract via xtask, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-8**.

## Consequences

**Positive**:
- Pure integer poisha storage eliminates decimal rounding bugs completely.
- Append only architecture ensures that financial records cannot be surreptitiously altered, establishing high trust with coaching center owners.
- Idempotent charge generation prevents accidental double billing during power disruptions or network hiccups, and respects prior voids.
- Advance credit automation eliminates manual credit matching by cashiers, speeding up daily front desk queues.
- Property based testing proves mathematical ledger consistency across one hundred thousand state combinations.

**Negative / tradeoffs**:
- Corrections cannot be made by simple record editing; staff must post explicit waiver or adjustment entries with reasons, adding slight administrative overhead.
- Voiding payments requires same day open cash day status; historic corrections must use adjustments or refunds rather than deletions.
- Memory and processing time for monthly generation across thousands of students requires batch transaction chunking.

**Neutral**:
- Front desk staff must be trained on how advance credit applies automatically on the first day of each month.
- Receipt sequence is partitioned per calendar year, resetting to 1 on January first.

## Follow-up

- [ ] Verify that migration 0012 passes SQLite immutability checks on existing development databases.
- [ ] Connect fee plans and charge generation to the Slice 2 cashier UI in Feature 16.
