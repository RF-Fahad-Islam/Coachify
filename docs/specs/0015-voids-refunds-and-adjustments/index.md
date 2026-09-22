# 0015. Voids, refunds and adjustments

**Date**: 2026-09-22
**Status**: Accepted

## Summary

In educational institutes and coaching centers, counter mistakes and enrollment changes are unavoidable: a cashier may type the wrong amount or student ID, a guardian may demand money back after a student discontinues, or an administrator may grant a special hardship waiver or assess a disciplinary fine. In traditional desktop software, staff often tamper with raw database records or silently overwrite historical totals, erasing auditability and enabling counter fraud.

Feature 17 establishes an append-only, strictly audited corrections engine covering three distinct financial operations:
1. **Same-Day Payment Voids**: Cancels an errant payment recorded earlier on the same open business date, requiring mandatory reason, `payments.void` permission, and open cash day status. Voiding immediately decouples the payment's allocations, restoring open charges without deleting raw allocation records.
2. **Post-Dated Refunds**: Disburses money back to guardians from unallocated payment funds (advance credit) on the current date, guarded by SQLite trigger `trg_refunds_ceiling` and method-specific tracking (`cash`, `bkash`, `nagad`, `bank`, `other`).
3. **Financial Adjustments**: Records explicit waivers, fines, and accounting corrections. Charge-linked waivers are capped by remaining unallocated dues via trigger `trg_adjustments_waiver_ceiling`, and all adjustments synchronously update the student balance cache.

Every correction generates an entry in the SHA-256 cryptographic audit hash chain, guaranteeing complete accounting integrity for institute owners.

## Requirements

**User stories**:
- As a counter cashier, I want to void a mistaken payment I recorded ten minutes ago, so that the student's dues are restored without having to call a developer or delete database files.
- As an institute owner, I want voids to be prohibited once a business day is closed or on past dates, so that staff cannot quietly cancel old payments and pocket cash.
- As an administrator, I want to issue a refund to a student who has excess advance credit, so that the money returned is recorded in today's cash out and tracked by refund method.
- As a branch manager, I want to grant a special fee waiver on an outstanding charge, so that the discount reflects immediately in the student's ledger without modifying the original fee plan.
- As an accountant, I want every void, refund, and adjustment to show prominently in the student's account statement and the audit log, so that all financial history is 100% transparent.

**Acceptance criteria**:
- **AC-1**: Payment voiding is strictly restricted to same-day transactions (`payments.business_date == current_business_date()`) while the business date's cash day is open (`cash_days.closed_at IS NULL`). Any attempt to void a past-dated payment or a payment on a closed cash day is rejected with `unauthorized_void` or `closed_cash_day`.
- **AC-2**: Payment voiding requires the `payments.void` permission and a non-empty `void_reason`. The payment status transitions from `posted` to `voided`, recording `voided_by` and `voided_at`. All downstream balance and ledger queries filter on `status = 'posted'`, cleanly restoring charges to unpaid status without violating SQLite allocation immutability triggers (`trg_payment_allocations_no_delete`).
- **AC-3**: Refunds can only be created against posted payments (`status = 'posted'`) on an open cash day. The refund amount cannot exceed available unallocated funds (`amount_poisha - allocated - prior_refunds`), enforced by trigger `trg_refunds_ceiling`.
- **AC-4**: Adjustments support `waiver`, `fine`, and `correction` types with positive integer poisha amounts, audited reasons, and optional approval tracking (`approved_by`). Waivers linked to a specific charge cannot exceed remaining unallocated dues (`net_amount_poisha - allocated_amount_poisha - prior_waivers`), enforced by trigger `trg_adjustments_waiver_ceiling`.
- **AC-5**: All voids, refunds, and adjustments synchronously recalculate and update the `student_balances` cache (`total_charged_poisha`, `total_paid_poisha`, `total_adjusted_poisha`, `total_refunded_poisha`, `balance_poisha`) within the same SQLite transaction, satisfying the ledger invariant:
  $$\text{balance} = \text{open charges net} + \text{fines} - \text{waivers} - \text{posted payments} + \text{refunds}$$
- **AC-6**: User interface in the student profile ledger tab and cashier desk provides intuitive, guarded dialogs for voiding same-day payments, issuing refunds on eligible payments, and creating waivers/fines, with live preview of the resulting student balance.

## Decision

**Chosen option**: Option 1: Immutable State Machine Transitions with Same-Day Void Guards, Ceiling-Enforced Refunds, and Synchronous Balance Reconciliation.

Implementation lives in:
- Backend: `crates/core/src/ledger.rs` (enhanced same-day void validation, refund processor, adjustment engine), `crates/core/src/dispatcher.rs` (RPC endpoints `payments.void`, `refunds.create`, `adjustments.create`, `charges.void`).
- Database: Triggers in `migrations/0006_audit_and_triggers.sql` and `migrations/0012_financial_ledger_refinements.sql`.
- Frontend: `ui/src/components/cashier/` and `ui/src/components/students/` (corrections dialog, void payment confirmation modal, refund issuance sheet, adjustment drawer).

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Corrections workflow & permissions

```
   ┌──────────────────────────────────────────────────────────┐
   │                  FINANCIAL CORRECTION                    │
   └──────────────────────────────────────────────────────────┘
             │                                │
    [ Payment Mistake ]              [ Policy / Billing Change ]
             │                                │
      Same Day & Open?              ┌───────────────────┬──────────────────┐
      ┌──────┴──────┐               │                   │                  │
    [Yes]          [No]         [Waiver]             [Fine]          [Refund]
      │              │              │                   │                  │
  payments.void   Formal       finance.adjust     finance.adjust    finance.adjust
  Status='voided' Refund       Capped by due       Increases due    Capped by credit
```

### Data model & RPC inputs

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoidPaymentInput {
    pub payment_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateRefundInput {
    pub payment_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub reason: String,
    pub business_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAdjustmentInput {
    pub student_id: String,
    pub charge_id: Option<String>,
    pub r#type: String, // "waiver" | "fine" | "correction"
    pub amount_poisha: i64,
    pub reason: String,
    pub business_date: Option<String>,
}
```

### Invariant & state transition rules

1. **Payment Voiding**:
   - Condition: `payment.business_date == current_business_date() && cash_day.closed_at IS NULL`.
   - Action: `UPDATE payments SET status = 'voided', void_reason = ?1, voided_by = ?2, voided_at = ?3 WHERE id = ?4`.
   - Effect: In `student_balances`, `total_paid_poisha` decreases by `amount_poisha`, increasing `balance_poisha`. Open charges previously satisfied by this payment automatically revert to unpaid.
2. **Refund Issuance**:
   - Condition: `payment.status == 'posted' && refund.amount_poisha <= (payment.amount_poisha - allocated - prior_refunds)`.
   - Action: `INSERT INTO refunds (...) VALUES (...)`.
   - Effect: In `student_balances`, `total_refunded_poisha` increases, increasing `balance_poisha`. Today's cash day `cash_out_poisha` increases if method is cash.
3. **Adjustment Application**:
   - Waiver: Decreases student balance. If linked to charge, ceiling check ensures `amount <= charge.remaining_due`.
   - Fine: Increases student balance.
   - Correction: Adjusts balance with mandatory explanation.

## Build plan

1. Refine `void_payment` in `crates/core/src/ledger.rs` to strictly enforce same-day business date matching and open cash day validation, satisfies **AC-1**, **AC-2**.
2. Enhance `create_refund` in `crates/core/src/ledger.rs` to validate unallocated payment ceiling, payment posted state, and dispatch audit event, satisfies **AC-3**, **AC-5**.
3. Implement `create_adjustment` in `crates/core/src/ledger.rs` supporting waivers with charge ceiling validation, fines, and corrections, satisfies **AC-4**, **AC-5**.
4. Verify synchronous balance cache recalculation across all correction operations, satisfies **AC-5**.
5. Build React UI components (`VoidPaymentDialog`, `RefundDialog`, `AdjustmentDialog`) integrated into Student Profile and Cashier Desk, satisfies **AC-6**.
6. Add comprehensive unit and integration tests verifying same-day restrictions, ceiling aborts, and balance invariants, satisfies **AC-1**, **AC-3**, **AC-4**, **AC-5**.

## Consequences

**Positive**:
- Eliminates counter fraud: cashiers cannot void historical payments on past or closed days.
- Perfect financial immutability: zero rows are deleted; all changes are traceable.
- Automatic dues restoration: voiding an errant receipt instantly resets charges to due.
- Safe refunds: parents can be refunded advance credit without exceeding payments.

**Negative / Tradeoffs**:
- Legitimate multi-day errors cannot be voided; they must be resolved via formal refund and charge adjustment.
