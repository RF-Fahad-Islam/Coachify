# Verify: Admission with initial payment (spec 0016, updated 2026-09-22)

Steps derived from spec 0016 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Click `[+ Admit Student]` in the Students tab: verify that the Direct Admission Wizard opens with steps for Demographics, Guardian, Batches, Fees & Concessions, and Initial Payment (verifies AC-6)
- [x] Enter student name, phone, school, and upload a student photo: verify that photo previews correctly via vault hash (verifies AC-6)
- [x] Enter an existing guardian's phone number: verify that the system detects the existing guardian record and links the sibling relationship (verifies AC-4)
- [x] Select two academic batches (e.g. HSC Physics and HSC Chemistry): verify that active fee plans for both batches automatically populate the fee overview with standard admission fee and first month's tuition (verifies AC-2)
- [x] Add a 20% concession on tuition fees: verify that the net payable amount dynamically recalculates and displays the exact discount (verifies AC-2)
- [x] Enable the "Collect Initial Payment" toggle and enter the net amount via Cash: click `[Complete Admission & Print]`: verify that the button disables, a loading spinner displays, and the dialog closes upon success (verifies AC-1, AC-3)
- [x] Inspect the student directory: verify that the new student appears with status `active`, auto-generated student code `STU-YYYY-####`, and enrolled in the selected batches (verifies AC-1, AC-5)
- [x] Open the newly created student's ledger: verify that charges are posted for admission and tuition, a posted payment with receipt number exists, and the current balance is ৳0 (verifies AC-1, AC-3)
- [x] Verify that the print preview or silent print trigger immediately offers both the Student Admission Form / Roster Card and the Payment Receipt voucher (verifies AC-7)
- [x] Intentional failure test: set up a scenario where payment submission fails (e.g. invalid payment method or closed cash day) and attempt admission: verify that the entire transaction rolls back and no student or enrollment records are left in the database (verifies AC-1)

## Commands

- [x] `cargo test -p coachify_core --test admission_with_payment` (verifies atomic transaction execution, fee plan charge derivation, concession calculations, payment allocation, and zero-residue rollback on error, satisfies AC-1, AC-2, AC-3, AC-4, AC-5)
- [x] `cargo run -p xtask check-types` (verifies generated TypeScript bindings for `AdmitStudentInput` and `AdmitStudentResponse`, satisfies AC-6)
- [x] `pnpm --dir ui test` (verifies Direct Admission Wizard multi-step form, batch fee calculators, and print triggers, satisfies AC-6, AC-7)

## Acceptance criteria coverage

- AC-1 (Atomic Multi-Entity Transaction): verified by integration tests asserting all-or-nothing rollback on mid-flight failure.
- AC-2 (Automatic Fee Plan Derivation): verified by fee plan query tests asserting correct charge creation for selected batches.
- AC-3 (Upfront Payment & Sequence Minting): verified by payment persistence tests asserting monotonic receipt numbering and allocation.
- AC-4 (Sibling Guardian Deduplication): verified by phone normalization search tests linking existing guardian IDs.
- AC-5 (Monotonic Student Code): verified by student code generator tests ensuring sequence continuity.
- AC-6 (High-Density Admission Wizard): verified by UI component testing across wizard steps and validation states.
- AC-7 (Dual Print Trigger): verified by print engine dispatch tests triggering both admission form and receipt voucher.
