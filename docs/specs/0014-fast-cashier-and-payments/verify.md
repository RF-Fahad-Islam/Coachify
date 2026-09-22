# Verify: Fast cashier and payments (spec 0014, updated 2026-09-22)

Steps derived from spec 0014 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] Navigate to the Payments tab (`/payments`): verify that the Fast Cashier Desk layout displays with the search bar automatically focused (verifies AC-1)
- [ ] Type a student's partial name, ID, phone, or roll number: verify that matching results appear in under 150 ms with student photo, batch, and current outstanding due badge (verifies AC-1)
- [ ] Use `Down Arrow` and press `Enter` to select a student: verify that the left pane immediately loads the student profile, itemized open charges grouped by month/fee head, overdue flags, advance balance, and last 5 payments in under 50 ms (verifies AC-1, AC-2)
- [ ] Press `Alt+F` (or click `[Pay Full Due]`): verify that the payment amount input is populated with the total due amount (verifies AC-3)
- [ ] Press `Alt+M` (or click `[Current Month]`): verify that the payment amount is populated with only the current billing month's charges (verifies AC-3)
- [ ] Press numeric keys `1` through `6`: verify that the payment method switches between Cash, bKash, Nagad, Bank, Card, and Other (verifies AC-3)
- [ ] Type a partial payment amount (e.g. ৳1,000 against a ৳2,500 due): verify that the allocation preview dynamically highlights which charges will be paid oldest-first (verifies AC-4)
- [ ] Toggle manual allocation mode and check specific charges: verify that the payment specifically targets the selected charge IDs (verifies AC-4)
- [ ] Click `[Receive Payment]` (or press `Ctrl+Enter`): verify that the submit button immediately disables with a loading spinner, payment records in the database, and silent receipt printing is initiated in under 800 ms (verifies AC-5, AC-6, AC-8)
- [ ] Rapidly double-click the `[Receive Payment]` button: verify that only a single payment row and a single receipt number are created in the database (verifies AC-8)
- [ ] Concurrency test: open the same student in two browser windows / workstations; record a payment in Window 1; attempt to record a payment in Window 2: verify that Window 2 rejects the transaction with a `due_changed` error and refreshes the student overview with fresh numbers (verifies AC-7)
- [ ] After payment completion, verify that the post-payment view displays the receipt number, payment summary, print status indicator, and instant `[Reprint]` / `[View PDF]` buttons (verifies AC-9)
- [ ] Press `Esc` or `Enter` on the post-payment banner: verify that the cashier desk clears and returns keyboard focus to the student search bar, ready for the next student (verifies AC-9)

## Commands

- [ ] `cargo test -p coachify_core --test cashier_and_payments` (verifies cashier student overview query, atomic payment collection, stale due concurrency rejection, double-click idempotency caching, oldest-first allocation hierarchy, and synchronous balance cache consistency, satisfies AC-2, AC-4, AC-5, AC-7, AC-8)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `ledger.get_cashier_student_overview` and updated `CreatePaymentInput`, satisfies AC-2, AC-3, AC-7)
- [ ] `cargo run -p xtask check-migrations` (verifies schema standards and released migration immutability)
- [ ] `pnpm --dir ui test` (verifies CashierDeskView, search bar keyboard navigation, quick-fill amount logic, allocation preview calculation, and post-payment receipt dialog, satisfies AC-1, AC-3, AC-4, AC-8, AC-9)

## Acceptance criteria coverage

- AC-1 (Instant Search with Due Badges): verified by search component unit tests and manual keyboard navigation tests.
- AC-2 (Cashier Student Overview): verified by core aggregated query tests asserting profile, charges, advance credit, and payment history.
- AC-3 (Payment Methods & Quick-Fill Shortcuts): verified by UI input tests asserting shortcut keybindings and amount population.
- AC-4 (Oldest-First & Manual Allocation): verified by core allocation unit tests and UI allocation preview tests.
- AC-5 (Atomic Immediate Transaction): verified by integration tests asserting monotonic receipt numbering, posted status, and balance cache sync.
- AC-6 (Sub-Second Latency): verified by benchmark test asserting end-to-end processing under 800 ms on reference hardware.
- AC-7 (Concurrency & Stale Due Guard): verified by concurrency collision tests asserting rejection with `due_changed` code.
- AC-8 (Idempotency & Double-Click Protection): verified by duplicate submission tests asserting single payment persistence and button locking.
- AC-9 (Post-Payment Receipt State): verified by UI workflow tests checking receipt banner, reprint action, and keyboard reset.
