# Verify: Financial ledger model and fee plans (spec 0012, updated 2026-09-22)

Steps derived from spec 0012 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] Log in as Administrator, navigate to the Fees module, create a fee head named "Monthly Tuition" under kind `tuition`, and verify that the new fee head appears in the active fee catalog (verifies AC-1)
- [ ] Create a fee plan for a program or batch with a monthly amount of 250,000 poisha (2,500 BDT) and valid effective dates, verifying that the plan saves and displays in the batch fee overview (verifies AC-2)
- [ ] Open a student profile, navigate to the Concessions tab, grant a twenty percent concession on the "Monthly Tuition" head with a reason note, and confirm that only one active concession is permitted for that head and batch (verifies AC-3)
- [ ] Trigger monthly charge generation for the current period (for example `2026-09`): verify that an open charge is created with gross amount 250,000 poisha, concession 50,000 poisha, and net amount 200,000 poisha (verifies AC-3, AC-4)
- [ ] Trigger the monthly charge generator a second time for the exact same billing period and confirm that zero new charges are created, demonstrating idempotency (verifies AC-4)
- [ ] Void a generated charge with reason, trigger monthly generation again for that period, and verify that the voided charge is not recreated (verifies AC-4)
- [ ] Enroll a student on the twentieth of September and generate September charges, verifying that a full month tuition charge is created in accordance with mid month enrollment rules (verifies AC-5)
- [ ] Record a 500,000 poisha advance payment for a student with no current dues, generate charges for the following month totaling 200,000 poisha, and verify that 200,000 poisha of advance credit is automatically allocated oldest first, leaving 300,000 poisha unallocated credit (verifies AC-6)
- [ ] Attempt to post a charge waiver exceeding the unpaid amount on a partially paid charge, confirming that the system rejects the waiver (verifies AC-7)
- [ ] Void a payment that has active allocations: confirm that the parent charge returns to fully unpaid status while raw allocation rows remain immutable (verifies AC-7, AC-9)
- [ ] Attempt a direct SQL UPDATE or DELETE on a posted payment or charge row and confirm that the SQLite database triggers abort the operation with an error (verifies AC-7)
- [ ] Record multiple payments and confirm that each receives a unique, monotonically increasing receipt number without gaps (verifies AC-8)
- [ ] Inspect the `student_balances` table for a student with charges, payments, and waivers, verifying that the balance column matches the sum of open charges minus payments plus adjustments (verifies AC-9)

## Commands

- [x] `cargo test -p coachify_core --test ledger_proptest` (runs proptest state machine verification executing 100,000 randomized financial operations, asserting that all nine ledger invariants hold after every step with zero balance drift, verifies AC-10)
- [x] `cargo test -p coachify_core --test fees_and_ledger` (runs core integration tests verifying fee head catalogs, batch fee plans, student concessions with batch override and half up rounding, chunked idempotent charge generation, mid month join rules, advance credit priority, waiver ceilings, payment void allocation decoupling, and receipt sequencing, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9)
- [x] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `fee_heads.*`, `fee_plans.*`, `concessions.*`, `charges.*`, `ledger.*`, and `payments.*`, verifies AC-1, AC-2, AC-3, AC-4, AC-8)
- [x] `cargo run -p xtask check-migrations` (verifies that released migrations in `released.json` remain immutable and migration 0012 adheres to schema standards, verifies AC-4, AC-7)

## Acceptance criteria coverage

- AC-1 (Fee Heads Catalog): verified by core unit tests ensuring valid fee head kinds and deletion prevention for referenced records.
- AC-2 (Fee Plans Configuration): verified by repository tests asserting integer poisha amounts, frequency constraints, and date validity periods.
- AC-3 (Student Concessions): verified by tests calculating percentage and fixed discounts, half up rounding, batch overrides, and single active concession uniqueness.
- AC-4 (Idempotent Charge Generation): verified by generation tests asserting that repeated runs for the same period create zero duplicate rows and preserve voided records.
- AC-5 (Mid Month Enrollments): verified by tests ensuring that initial enrollment in the middle of a month generates a full calendar month fee.
- AC-6 (Advance Credit Allocation): verified by allocation tests asserting oldest first settlement by due date and complete fee head priority hierarchy.
- AC-7 (Immutability Triggers and Waiver Ceilings): verified by database trigger tests aborting raw UPDATE and DELETE queries, and logic tests enforcing waiver limits and payment void decoupling.
- AC-8 (Atomic Receipt Numbers): verified by sequence tests asserting monotonic incrementing without skipped values.
- AC-9 (Synchronous Balance Cache): verified by transaction tests asserting that `student_balances` matches calculated ledger history after every mutation.
- AC-10 (Property Test Verification): verified by proptest executing 100,000 random operations without invariant violations.
