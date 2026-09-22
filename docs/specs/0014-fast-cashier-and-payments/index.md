# 0014. Fast cashier and payments

**Date**: 2026-09-22
**Status**: Accepted

## Summary

In coaching centers and small educational institutes in Bangladesh, fee collection at the front desk is the single most frequent and critical operational loop. Receptionists and counter cashiers handle hundreds of students and parents within tight morning and evening class windows. Any slowness, awkward multi-step navigation, or calculation confusion at the counter leads to long queues, cashier stress, and dissatisfied parents.

Feature 16 delivers the **Fast Cashier Desk**: a keyboard-driven, high-density interface optimized for speed and clarity. From a single screen, a cashier finds a student within 150 ms via an instant FTS5 search (by Student Code, Name, Phone, Guardian Phone, or Roll), immediately reviews all itemized dues grouped by month and fee head alongside any advance credit and recent payment history, enters the payment amount and method with single-keystroke shortcuts, and completes the payment with automatic silent receipt printing in under one second on a reference low-end PC.

Payment recording is atomic, immutable, and protected against concurrency issues: a client-side UUIDv7 idempotency key prevents accidental double collections from rapid clicking, an optimistic balance check (`expected_balance_poisha`) rejects stale due submissions if another counter or automated billing run updated the student balance in the interim, and all mutations execute inside an immediate SQLite transaction updating the synchronous balance cache and cryptographic audit log.

## Requirements

**User stories**:
- As a counter cashier, I want to find a student instantly by typing their ID, roll, name, or phone number, so that I can pull up their file in seconds while they stand at the counter.
- As a cashier, I want to see a clear monthly breakdown of open charges, overdue balances, and advance credit, so that I can explain the exact fee components to parents.
- As a cashier, I want one-click and keyboard shortcuts to fill full dues or current month fees and select payment methods, so that everyday transactions take only a few keystrokes.
- As a cashier, I want the system to automatically allocate payment funds to the oldest open charges first while allowing manual charge selection when needed, so that dues are settled accurately.
- As an institute owner, I want double-clicks on the payment button to produce only one transaction, so that counter haste never causes duplicate charges.
- As an accountant in a multi-counter center, I want transactions to be rejected with fresh numbers if another cashier collected a payment for the same student while my screen was open, so that overcollection and race conditions are eliminated.
- As a cashier, I want immediate silent printing of the receipt upon clicking Receive, so that the student receives their receipt in under one second.

**Acceptance criteria**:
- **AC-1**: Instant student search in cashier desk powered by SQLite FTS5 trigram index across Student Code (`S2026-0001`), Student Name, Bengali Name, Phone, Guardian Phone, and School Roll number, returning matches in under 150 ms with live due badges. Keyboard navigation (`Up`/`Down`/`Enter`) allows selection without touching the mouse.
- **AC-2**: Cashier student overview query (`ledger.get_cashier_student_overview`) retrieves the student's profile, active program/batch enrollments, itemized open charges with overdue indicators, unallocated advance credit balance, and the 5 most recent payments in under 50 ms.
- **AC-3**: Support payment collection with payment methods (`cash`, `bkash`, `nagad`, `bank`, `card`, `other`) and optional transaction reference (e.g. bKash TrxID or cheque number). Provide quick-fill buttons and keyboard shortcuts for `[Pay Full Due]` (`Alt+F`), `[Pay Current Month]` (`Alt+M`), and numeric method keys (`1` for Cash, `2` for bKash, `3` for Nagad, `4` for Bank).
- **AC-4**: Automatic allocation applies funds to open charges oldest-first by due date, breaking ties by fee head priority (`tuition` → `admission` → `exam` → `registration` → `material` → `other` → `fine`). Cashier can toggle manual allocation mode to selectively allocate payments to specific charges.
- **AC-5**: Payment collection executes inside an immediate SQLite transaction: verifies cash day is open, validates positive amount, mints the next strictly monotonic annual receipt sequence (`REC-YYYY-####`), inserts the immutable payment row (`status = 'posted'`), inserts payment allocations, synchronously updates the `student_balances` cache, writes an audit log row, and enqueues payment confirmation SMS in outbox if automation rule is active.
- **AC-6**: Sub-second end-to-end execution: total latency from clicking "Receive Payment" to receipt printing started is under 800 ms on the reference low-end PC (4th gen Core i3, 4 GB RAM, spinning HDD, Windows 10).
- **AC-7**: Concurrency protection via `expected_balance_poisha`: client submits the student's known balance at time of review; if current database balance differs, transaction is rejected with error code `due_changed` returning fresh balance and charges.
- **AC-8**: Idempotency and double-click protection: client generates a UUIDv7 `idempotency_key` upon student selection; UI disables submit button immediately on first click with a loading spinner; server checks `command_idempotency` and returns the existing receipt if re-submitted without creating duplicate records.
- **AC-9**: Post-payment state displays success banner with receipt number, payment summary, silent print status indicator, instant `[Reprint]` and `[View PDF]` buttons, and a single-keystroke shortcut (`Esc` or `Enter`) to clear the form ready for the next student.

## Decision

**Chosen option**: Option 1: Dedicated High-Density Fast Cashier Desk with Optimistic Concurrency Balance Checking, Atomic Single-Writer Financial Transaction, and Instant Keyboard Navigation.

You build the Fast Cashier subsystem across `ui/src/components/cashier/` (cashier desk view, search bar, charge breakdown list, quick payment form, post-payment receipt banner), `crates/core/src/ledger.rs` (cashier overview query, stale-due validation, enriched receipt model), and `crates/core/src/dispatcher.rs` (RPC endpoints `ledger.get_cashier_student_overview` and `payments.create`).

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Cashier desk workflow & layout

```
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │ [ F2 / '/' ] Search: Student ID, Roll, Phone, Guardian Phone, Name              │
 └─────────────────────────────────────────────────────────────────────────────────┘
                                         │
 ┌───────────────────────────────────────┬─────────────────────────────────────────┐
 │ STUDENT & LEDGER OVERVIEW             │ QUICK PAYMENT COUNTER                   │
 │                                       │                                         │
 │ [Photo] Tanvir Ahmed (STU-2026-0042)  │ Amount to Pay (BDT):                    │
 │ HSC 2028 • Physics Batch A • Roll 12  │ [ 2,200           ]  [Pay Full (Alt+F)] │
 │ Guardian: Rafiqul Islam (01711234567) │                      [Current Month]    │
 │ ───────────────────────────────────── │ Method:                                 │
 │ Open Charges Due:                     │ (*) (1) Cash     ( ) (2) bKash          │
 │ • Sep 2026 Tuition:   ৳1,500          │ ( ) (3) Nagad    ( ) (4) Bank           │
 │ • Model Test Exam:      ৳300          │ ( ) (5) Card     ( ) (6) Other          │
 │ • Aug 2026 Overdue:     ৳400 [Overdue]│                                         │
 │ ───────────────────────────────────── │ Reference / TrxID:                      │
 │ Total Due:            ৳2,200          │ [ e.g. bKash TrxID (optional)         ] │
 │ Advance Credit:           ৳0          │                                         │
 │                                       │ Allocation Strategy:                    │
 │ Recent Payments (Last 5):             │ [x] Auto-allocate oldest first          │
 │ • 2026-08-05: ৳1,500 (#0038) [Reprint]│                                         │
 │ • 2026-07-04: ৳1,500 (#0021) [Reprint]│ [ Receive Payment (Ctrl+Enter / Alt+S) ]│
 └───────────────────────────────────────┴─────────────────────────────────────────┘
```

### Data model & DTOs

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashierStudentOverview {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub photo_vault_id: Option<String>,
    pub phone: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub program_name: Option<String>,
    pub roll_no: Option<String>,
    pub total_due_poisha: i64,
    pub advance_credit_poisha: i64,
    pub net_balance_poisha: i64,
    pub open_charges: Vec<ChargeItem>,
    pub recent_payments: Vec<PaymentReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreatePaymentInput {
    pub student_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reference: Option<String>,
    pub business_date: Option<String>,
    pub idempotency_key: String,
    pub target_charge_ids: Option<Vec<String>>,
    pub expected_balance_poisha: Option<i64>, // Concurrency guard
}
```

### Concurrency state machine

1. Cashier opens student overview at \(T_0\), observing `net_balance_poisha = 220000` (৳2,200).
2. While cashier enters cash, another process at \(T_1\) records a transaction changing balance to `320000` (৳3,200) or `0`.
3. Cashier submits payment with `expected_balance_poisha = Some(220000)`.
4. Rust core in `create_payment` executes:
   ```rust
   let current_balance = get_student_balance(conn, &input.student_id)?;
   if let Some(expected) = input.expected_balance_poisha {
       if current_balance != expected {
           return Err(LedgerError::DueChanged {
               expected_poisha: expected,
               actual_poisha: current_balance,
           });
       }
   }
   ```
5. Transaction rolls back immediately with zero writes. Client receives `due_changed` error, displays prominent alert, and refreshes the student overview with the fresh figures.

## Build plan

1. Add `expected_balance_poisha` concurrency check and `DueChanged` error to `crates/core/src/ledger.rs`, satisfies **AC-7**.
2. Implement fast aggregated cashier query `ledger.get_cashier_student_overview` in `crates/core/src/ledger.rs` and dispatcher handler, satisfies **AC-2**.
3. Build `ui/src/components/cashier/cashier-desk-view.tsx` with high-density layout, due breakdown, and recent payments with reprint shortcuts, satisfies **AC-2**, **AC-3**.
4. Implement instant FTS5 search combobox with keyboard arrow selection and current due badge, satisfies **AC-1**.
5. Implement quick-fill payment shortcuts (`Alt+F` for Full, `Alt+M` for Current Month) and numeric method shortcuts (`1` - `6`), satisfies **AC-3**.
6. Implement allocation preview engine and manual allocation toggles, satisfies **AC-4**.
7. Connect payment submission to automatic silent print trigger from Spec 0013 with double-click lock and post-payment receipt banner, satisfies **AC-5**, **AC-6**, **AC-8**, **AC-9**.
8. Verify sub-second roundtrip latency on reference low-end PC profile, satisfies **AC-6**.

## Consequences

**Positive**:
- Extremely fast front-desk throughput: students can be checked in and receipted in under 5 seconds total.
- Concurrency guard prevents duplicate payments and race conditions in multi-counter centers.
- Keyboard-first shortcuts reduce repetitive strain for cashiers handling hundreds of payments.
- Automatic receipt sequencing and ledger immutability guarantee complete accounting integrity.

**Negative / tradeoffs**:
- Stale-due rejections require the cashier to re-verify numbers when a concurrent update occurs.
- High-density screen requires responsive layout testing on smaller 1366x768 desktop monitors.
