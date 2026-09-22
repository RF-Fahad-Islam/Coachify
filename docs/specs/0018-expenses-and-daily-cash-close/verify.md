# Verify: Expenses and daily cash close (spec 0018, updated 2026-09-22)

Steps derived from spec 0018 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Navigate to the Expenses / Cash Close desk (`/expenses`): verify that the screen displays the active business date, open status, and summary cards (Opening Cash, Cash In, Cash Out, Expected Drawer Balance) (verifies AC-4, AC-9)
- [x] Click `[+ Record Expense]`: select category "Printing & Stationery", enter amount ৳450, method Cash, payee "Ideal Paper House", and upload a photo of the cash memo voucher: verify that the expense saves and Cash Out increases by ৳450 (verifies AC-2, AC-4)
- [x] Record another expense via bKash for ৳1,500 (e.g. Internet Bill): verify that the bKash expense appears in the expense list but does NOT deduct from the physical Cash Out drawer total (verifies AC-2, AC-4)
- [x] In Cashier Desk, collect a cash fee payment of ৳2,000 and a bKash payment of ৳1,000: return to Cash Close Desk: verify that Cash In increases by ৳2,000, and the digital channel breakdown displays ৳1,000 under bKash (verifies AC-4)
- [x] Click `[Close Day]`: verify that the dialog displays Expected Closing Cash (Opening + Cash In - Cash Out): enter counted physical cash with a ৳50 shortage: verify that the difference is highlighted as `-৳50 (Shortage)`: enter note "৳50 shortage due to coin change" and confirm close (verifies AC-5)
- [x] Verify that the day status transitions to `Closed` with timestamp and user name: verify that all action buttons for adding payments, expenses, refunds, or voids on this closed date are disabled or rejected (verifies AC-5, AC-6)
- [x] Attempt to insert or void a payment dated into the closed day via direct RPC or test: verify that the database trigger or backend rejects the operation with error code `closed_cash_day` (verifies AC-6)
- [x] Advance the system date or open the next business day: verify that the new cash day initializes with `opening_cash_poisha` exactly matching yesterday's counted closing cash (verifies AC-7)
- [x] Click `[Print Cash Close Sheet]`: verify that the print preview renders the official Daily Cash Close Summary sheet with opening cash, receipts by channel, categorized expenses, counted cash, shortage note, and signature lines (verifies AC-8)

## Commands

- [x] `cargo test -p coachify_core --test expenses_and_cash_close` (verifies expense entry validation, same-day voiding, cash day opening/closing transitions, balance carry-forward, variance calculations, and retroactive mutation rejection on closed days, satisfies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7)
- [x] `cargo run -p xtask check-types` (verifies generated TypeScript bindings for `CreateExpenseInput`, `CloseCashDayInput`, and `CashDaySummary`, satisfies AC-9)
- [x] `pnpm --dir ui test` (verifies CashCloseView, ExpenseEntryDialog, variance calculation logic, and CashCloseSheetTemplate layout, satisfies AC-8, AC-9)

## Acceptance criteria coverage

- AC-1 (Expense Categories Catalog): verified by category repository unit tests ensuring foreign key restriction on deletion.
- AC-2 (Expense Recording with Vault Attachment): verified by expense insertion tests with receipt file hash linkage.
- AC-3 (Same-Day Expense Voiding): verified by voiding tests asserting open-day same-date restrictions.
- AC-4 (Real-Time Daily Cash Summary): verified by aggregation tests ensuring correct segregation of cash vs digital channels.
- AC-5 (Cash Day Close & Variance Calculation): verified by closing transaction tests asserting difference calculation and audit stamping.
- AC-6 (Retroactive Mutation Lock): verified by trigger tests asserting rejection of any mutation on closed business dates.
- AC-7 (Next-Day Opening Balance Carry-Forward): verified by sequential multi-day simulation tests ensuring continuous cash chaining.
- AC-8 (Printable Cash Close Summary Sheet): verified by component print rendering tests for A4 and 80 mm thermal layouts.
- AC-9 (Cash Close & Expense Desk UI): verified by frontend component interaction and form validation tests.
