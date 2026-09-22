# Verify: Voids, refunds and adjustments (spec 0015, updated 2026-09-22)

Steps derived from spec 0015 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Navigate to Student Profile -> Ledger Tab: verify that each posted payment displays a `[Void]` button if recorded today, and `[Refund]` button if unallocated credit remains (verifies AC-6)
- [x] Click `[Void]` on a payment recorded today: enter a reason (e.g. "Typo in payment method") and confirm: verify that payment status changes to Voided, allocations are removed from charge calculations, and the student's due balance increases by the payment amount (verifies AC-2, AC-5)
- [x] Attempt to void a payment from yesterday or a past date: verify that the system rejects the operation with error message "Only same-day payments on an open cash day can be voided" (verifies AC-1)
- [x] Close today's cash day and attempt to void a same-day payment: verify that the operation is rejected with `closed_cash_day` (verifies AC-1)
- [x] Open an eligible payment with ৳1,000 unallocated advance credit and click `[Refund]`: enter ৳500 refund via Cash with reason "Course withdrawal partial return": verify that refund is saved, today's cash drawer records ৳500 cash-out, and the student's credit decreases by ৳500 (verifies AC-3, AC-5)
- [x] Attempt to enter a refund of ৳1,200 against ৳1,000 available credit: verify that submission fails with ceiling error from trigger `trg_refunds_ceiling` (verifies AC-3)
- [x] On an open charge of ৳1,500 with ৳1,000 already paid, open the Adjustments dialog and select `Waiver`: enter ৳500 waiver with reason "Academic excellence discount": verify that charge net due drops to ৳0 and student balance is updated (verifies AC-4, AC-5)
- [x] Attempt to enter a waiver of ৳600 on that same charge: verify that transaction aborts with `trg_adjustments_waiver_ceiling` error (verifies AC-4)
- [x] Add a fine adjustment of ৳200 with reason "Late submission penalty": verify that the student's total balance due increases by ৳200 (verifies AC-4, AC-5)
- [x] Review the Audit Log in Settings: verify that every void, refund, waiver, and fine has an audited record with actor ID, timestamp, and explanation (verifies AC-2, AC-3, AC-4)

## Commands

- [x] `cargo test -p coachify_core --test voids_refunds_adjustments` (verifies same-day void restrictions, unallocated refund ceiling enforcement, waiver ceiling trigger aborts, fine applications, and balance cache recalculation invariants, satisfies AC-1, AC-2, AC-3, AC-4, AC-5)
- [x] `cargo run -p xtask check-types` (verifies generated TypeScript bindings for `VoidPaymentInput`, `CreateRefundInput`, and `CreateAdjustmentInput`, satisfies AC-6)
- [x] `pnpm --dir ui test` (verifies UI dialogs, confirmation prompts, error handling, and ledger balance previews, satisfies AC-6)

## Acceptance criteria coverage

- AC-1 (Same-Day & Open Cash Day Void): verified by integration tests asserting rejection of past-dated or closed cash day voids.
- AC-2 (Permission & Allocations Neutralization): verified by RBAC check and query assertions showing unpaid charge restoration.
- AC-3 (Refunds with Unallocated Ceiling): verified by trigger tests rejecting excess refund amounts and verifying cash-out accounting.
- AC-4 (Adjustments & Waiver Ceiling): verified by waiver ceiling trigger tests and fine application assertions.
- AC-5 (Synchronous Balance Cache Recalculation): verified by invariant checks asserting balance formula matches `student_balances` cache.
- AC-6 (Guarded UI Workflow): verified by frontend component tests for void, refund, and adjustment modal interactions.
