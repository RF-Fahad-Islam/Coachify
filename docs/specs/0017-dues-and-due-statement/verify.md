# Verify: Dues and due statement (spec 0017, updated 2026-09-22)

Steps derived from spec 0017 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Navigate to Payments / Dues Center: verify that the Dues dashboard loads with summary cards showing Total Institute Dues, Due Students count, and Overdue (>30 Days) amount (verifies AC-6)
- [x] Filter by Program and Batch (e.g. HSC Physics Batch A): verify that only students enrolled in that batch appear in the dues table with accurate individual dues (verifies AC-1)
- [x] Check aging badges: verify that students with charges older than 30 days display the orange `[Overdue]` badge, and charges older than 60 days display the red `[Critical Overdue]` badge (verifies AC-2)
- [x] Click the `[Collect Payment]` button on a student row: verify that the Fast Cashier Desk opens immediately with that student's file pre-loaded and dues ready for payment (verifies AC-6)
- [x] In Cashier Desk, record a partial payment of ৳1,000 against that student's dues, then return to the Dues Center: verify that the student's total due and the institute aggregate total decrease by exactly ৳1,000 (verifies AC-3)
- [x] Click `[Print Due Statement]` on a student row: verify that the print preview modal displays the bilingual Student Account & Due Statement with institute crest, itemized charges, payments received, net due, and amount in words in both Bengali and English (verifies AC-4, AC-5)
- [x] Verify that the Due Statement includes valid payment instructions (e.g. institute bKash merchant number, bank details) and signature lines for Cashier and Director (verifies AC-5)
- [x] Click `[Export CSV]`: verify that the downloaded CSV file matches the currently filtered student list and sums to the displayed total dues (verifies AC-7)
- [x] Run the mathematical reconciliation assertion across all active students: verify that `SUM(student_balances.balance_poisha)` equals the dues query sum with zero difference (verifies AC-3)

## Commands

- [x] `cargo test -p coachify_core --test dues_and_statements` (verifies dues aggregation filtering, aging classification buckets, strict `student_balances` cache reconciliation, and due statement document assembly, satisfies AC-1, AC-2, AC-3, AC-4)
- [x] `cargo run -p xtask check-types` (verifies generated TypeScript bindings for `ListDuesFilter`, `StudentDueSummaryItem`, and `DueStatementData`, satisfies AC-6)
- [x] `pnpm --dir ui test` (verifies DuesCenterView data table, summary cards, filter controls, and DueStatementTemplate print layout, satisfies AC-5, AC-6, AC-7)

## Acceptance criteria coverage

- AC-1 (High-Performance Dues Aggregation): verified by query benchmark tests asserting sub-100ms response times with multiple filters.
- AC-2 (Overdue Aging Classification): verified by core unit tests checking aging bucket assignments based on charge due dates.
- AC-3 (Strict Financial Invariant Reconciliation): verified by automated consistency test matching dues total against `student_balances` cache.
- AC-4 (Due Statement Document Model): verified by statement query tests asserting complete charge/payment history and words-in-BDT.
- AC-5 (Bilingual Printable Template): verified by component rendering tests checking Bangla font embedding, crest, and layout fidelity.
- AC-6 (Interactive Dues Center UI): verified by frontend interaction tests for filtering, deep-linking, and payment transition.
- AC-7 (Batch Print & CSV Export): verified by export file generation tests and print engine batch dispatch tests.
