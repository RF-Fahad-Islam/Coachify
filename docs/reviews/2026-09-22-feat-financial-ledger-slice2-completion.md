# Review: Financial Ledger Slice 2 Completion (Features 17–20)

**Date**: 2026-09-22
**Reviewed by**: Claude Opus (Senior Software & Systems Architect)
**Scope**: Implementation plan, specifications (0015, 0016, 0017, 0018), and scope tracking for Features 17–20
**Verdict**: Approved with follow-up items

---

## Summary

The architectural design and specification suite for the completion of Slice 2 (Money)—spanning **Feature 17** (Voids, Refunds & Adjustments), **Feature 18** (Admission with Initial Payment), **Feature 19** (Dues & Due Statement), and **Feature 20** (Expenses & Daily Cash Close)—is exceptionally well-engineered, robust, and tailored specifically to the operational constraints of Bangladeshi coaching centers.

Key architectural highlights include:
- A strict temporal split restricting payment voids to same-day open sessions while handling post-dated money returns as formal refunds.
- Unallocated ceiling triggers (`trg_refunds_ceiling`) and charge waiver ceiling triggers (`trg_adjustments_waiver_ceiling`) preventing balance drift.
- A single atomic composite transaction (`BEGIN IMMEDIATE`) for student admission, fee calculation, and receipt minting.
- Separation of physical cash drawer reconciliation from digital collection channels (bKash, Nagad, Bank).
- A mathematically enforced invariant reconciling live dues projections against the synchronous `student_balances` cache.

To achieve defense-in-depth before `/develop` execution, four follow-up architectural safeguards must be incorporated into the upcoming migration (`0013_financial_ledger_cash_close_and_mutations.sql`).

---

## Resolved & Validated Strengths

1. **Interlocking Ceiling Triggers**: The relationship between `trg_charges_prevent_void_with_allocations`, `trg_adjustments_waiver_ceiling`, and `trg_refunds_ceiling` guarantees that voided charges do not corrupt refund calculations, and refunds cannot exceed unallocated funds.
2. **Atomic Admission Pipeline**: Modeling student creation, guardian linkage/deduplication, batch enrollments, automatic fee charge derivation, concession adjustments, and initial receipt collection inside a single `BEGIN IMMEDIATE` transaction guarantees zero orphaned records under power loss or client disconnection.
3. **Cash vs. Digital Separation**: Segregating physical drawer cash from digital wallets (bKash/Nagad/Bank) in the daily cash close state machine prevents counter cashiers from confusing digital receivables with drawer notes and coins.
4. **Offline-First Bilingual Presentation**: Using the WebView2 print engine with embedded SolaimanLipi/Kalpurush fonts and `words.rs` ensures that receipts, due statements, and cash close sheets render flawlessly offline without ESC/POS driver corruption.

---

## Major (Required for Migration 0013 & /develop Phase)

### 🟠 1. Idempotency Keys on Non-Payment Financial Mutations
- **Problem**: While `payments` implements `idempotency_key TEXT NOT NULL UNIQUE`, the `expenses`, `refunds`, and `adjustments` tables in `0005_financial_ledger.sql` currently lack explicit unique idempotency columns.
- **Why it matters**: In busy front-desk environments with slow mechanical hard drives or laggy touchpads, staff double-clicking "Submit Refund" or "Save Expense" could create duplicate transactions, leading to phantom cash drain or duplicate student waivers.
- **Suggested fix**: In migration `0013`, add an `idempotency_key TEXT` column to `expenses`, `refunds`, and `adjustments` with unique indexes, and mandate UUIDv7 idempotency keys in their respective RPC payloads (`CreateExpenseInput`, `CreateRefundInput`, `CreateAdjustmentInput`).

### 🟠 2. Defense-in-Depth for Payment Over-Allocation
- **Problem**: There is currently no database trigger preventing `payment_allocations` from exceeding a charge's `net_amount_poisha` minus applied `waivers`.
- **Why it matters**: While the Rust core's `BEGIN IMMEDIATE` serializes application logic, any bug in manual allocation selection or a future script could allocate more funds than a charge requires (e.g. allocating ৳2,000 to a ৳1,500 tuition charge), silently violating Ledger Invariant #2.
- **Suggested fix**: Add a trigger `trg_payment_allocations_ceiling` on `BEFORE INSERT ON payment_allocations` asserting that total allocated amount on that charge does not exceed `net_amount_poisha - SUM(waivers)`.

### 🟠 3. State Machine Directional Lock (Prevent Un-Voiding)
- **Problem**: Triggers `trg_payments_no_update` and `trg_charges_no_update` allow transitions from `posted` to `voided`, but do not explicitly disallow updating `status` from `'voided'` back to `'posted'` or `'open'`.
- **Why it matters**: A malicious actor or bug executing a raw update could "un-void" a historical payment or charge, bypassing business logic and desynchronizing the balance cache.
- **Suggested fix**: Add `trg_payments_no_unvoid` and `trg_charges_no_unvoid` triggers:
  ```sql
  CREATE TRIGGER trg_payments_no_unvoid
  BEFORE UPDATE OF status ON payments
  WHEN OLD.status = 'voided' AND NEW.status != 'voided'
  BEGIN
      SELECT RAISE(ABORT, 'Voided payments are permanently terminal and cannot be reopened');
  END;
  ```

### 🟠 4. Database-Enforced Retroactive Mutation Lock
- **Problem**: While `verify_open_cash_day` in Rust checks whether a day is closed, there are no triggers in SQLite enforcing Invariant #5 at the database layer.
- **Why it matters**: Direct SQL queries, batch scripts, or background routines could inadvertently insert or void transactions dated to a past closed day.
- **Suggested fix**: Add triggers on `payments`, `expenses`, `refunds`, and `adjustments` before insert/update:
  ```sql
  CREATE TRIGGER trg_payments_closed_cash_day_lock
  BEFORE INSERT ON payments
  WHEN EXISTS (SELECT 1 FROM cash_days WHERE business_date = NEW.business_date AND closed_at IS NOT NULL)
  BEGIN
      SELECT RAISE(ABORT, 'Cannot insert financial records into a closed cash day');
  END;
  ```

---

## Minor

### 🟡 1. Explicit Policy for Erroneous Refunds
- **Problem**: The specs state refunds are immutable and do not allow voids.
- **Why it matters**: If a cashier issues an accidental refund of ৳500 instead of ৳50, how is it corrected?
- **Suggested fix**: Document explicitly in Spec 0015 that refunds cannot be voided; erroneous refunds must be corrected by assessing a compensatory fine/correction adjustment on the student account and recording a counter cash-in payment.

### 🟡 2. Morning Cash Float Support
- **Problem**: When a cashier starts a morning shift, the institute owner may provide a small physical float (e.g. ৳2,000 in small change notes) in addition to yesterday's closing cash.
- **Why it matters**: Without float tracking, the evening count will show an artificial ৳2,000 cash surplus.
- **Suggested fix**: In `cash_days.get_or_open_today`, support an optional `float_adjustment_poisha` or record it as a dedicated "Cash Drawer Float In" transaction.

---

## Nits
- ⚪ Ensure WebView2 thermal print CSS handles automated paper cutting commands (`cut` or standard margin page breaks) for 80 mm Cash Close sheets.
- ⚪ In `docs/specs/0015-voids-refunds-and-adjustments/index.md`, confirm that the Rust query explicitly enforces `business_date == current_business_date()`.

---

## Test Coverage Assessment
The test plan across `tests/voids_refunds_adjustments.rs`, `tests/admission_with_payment.rs`, `tests/dues_and_statements.rs`, and `tests/expenses_and_cash_close.rs` covers:
- Invariant reconciliation between the dues list query and the `student_balances` cache.
- Multi-step rollback tests asserting zero database residue upon admission failure.
- Sequential multi-day cash close simulations verifying balance carry-forward and variance logging.
- Concurrency collisions and double-click idempotency.

The implementation plan is **Approved with follow-up items** and fully ready for `/develop`.
