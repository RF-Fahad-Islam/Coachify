# Verify: Students and guardians directory (spec 0008, updated 2026-09-21)

Steps derived from spec 0008 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] Log in as Receptionist or Cashier, navigate to the Students module, and verify that the Student Directory table renders with column headers, status filters, and search bar (verifies AC-10)
- [ ] Click "Add Student", fill in required student fields, leave student code blank, fill in a new primary guardian with a valid Bangladeshi phone number (such as `01712345678`), submit, and verify that a unique student code (such as `S2026-0001`) is generated automatically (verifies AC-1, AC-2, AC-4)
- [ ] In the Add Student dialog, upload a student profile photo (e.g. 1920x1080 JPEG), save, and confirm that the image is saved in the local vault, resized to a maximum of 800 by 800 pixels with file extension, and displays properly as an avatar (verifies AC-5, AC-10)
- [ ] Attempt to upload an image exceeding memory limits (e.g. 30MB file or 8000x8000px) and verify that the system rejects the file with a clear client error without causing high memory usage (verifies AC-5)
- [ ] Click "Add Student" again to register a sibling: in the guardian section, type the first student guardian phone number in the auto complete search, select the existing guardian from the dropdown, verify that guardian fields auto populate, select `is_primary = 1`, and submit (verifies AC-3, AC-4)
- [ ] Open the newly created student profile and click the "Guardians and Siblings" tab: confirm that the shared guardian is displayed along with a list of linked siblings including the first student (verifies AC-3, AC-10)
- [ ] Edit the shared guardian from the student profile: verify that a modal warns that 2 students are linked to this guardian, with choices to "Update for all siblings" or "Unlink and create separate guardian" (verifies AC-3, AC-10)
- [ ] Register a student with a phone number matching another active record: verify that the application returns a warning modal indicating the duplicate conflict, and saves only upon confirming the bypass (verifies AC-4)
- [ ] In the top search bar or Ctrl+K command palette, type a 3 character substring of the student name, phone number, or student code, and confirm that search results update in under 50 milliseconds with high precision (verifies AC-6, AC-7)
- [ ] Attempt to hard delete a student who has an active enrollment or financial ledger history and confirm that the action is blocked with a clear message indicating the student must be archived instead (verifies AC-9)
- [ ] Change the student status to `archived` with a reason note, confirm that the record updates successfully, and verify that archived students are filtered out of the default active student directory (verifies AC-9, AC-10)

## Commands

- [ ] `cargo test -p coachify_core --test students_and_guardians` (runs comprehensive core integration tests verifying atomic student code sequence generation, primary guardian enforcement, sibling guardian linking, Bangladeshi phone normalization and duplicate conflict bypass, local vault photo resizing, memory guards, and content addressed SHA256 storage, FTS5 trigram rowid query execution and ranking under 50ms, RBAC permission gating, and delete rejection triggers, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `students.*`, `guardians.*`, and `vault.*`, verifies AC-8)
- [ ] `cargo run -p xtask check-migrations` (verifies that released migrations in `released.json` remain immutable and migration 0010 adheres to schema standards, verifies AC-6, AC-8, AC-9)
- [ ] `pnpm --dir ui test` (runs UI unit and component tests for the Student Directory view, instant debounced search input, Add Student dialog with guardian auto complete, duplicate phone warning modal, shared guardian edit warning, and tabbed student profile view, verifies AC-3, AC-4, AC-7, AC-10)

## Acceptance criteria coverage

- AC-1 (Student Code Generation): verified by core unit tests asserting sequence allocation with year prefix (`S2026-0001`) and unique validation on manual code entry.
- AC-2 (Mandatory Primary Guardian and SMS Flag): verified by core repository tests rejecting student creation without a guardian or without exactly one primary guardian.
- AC-3 (Sibling Guardian Reuse and Mutation Safety): verified by integration tests asserting that two students can link to the same guardian ID, retrieve each other as siblings, and prompt on shared edits.
- AC-4 (Phone Normalization and Duplicate Conflict Guard): verified by unit tests converting mobile formats into standard 13 digit `8801XXXXXXXXX`, preserving landlines/international formats, and managing duplicate conflict warning bypasses.
- AC-5 (Local Content Addressed Vault with Safe Image Decoding): verified by vault tests confirming file downscaling to max 800x800 pixels, image decoding limits, SHA256 hashing, and disk persistence under `<institution_id>/vault/`.
- AC-6 (SQLite FTS5 Trigram Virtual Table with Rowid Triggers): verified by database migration tests validating automatic search index updates on student and guardian mutations with constant time rowid lookups.
- AC-7 (Instant Substring Search Performance): verified by benchmark tests asserting query execution times under 50 milliseconds across 10,000 indexed student records and 3 character client query threshold.
- AC-8 (Typed RPC Commands and Granular RBAC Permissions): verified by dispatcher tests asserting permission checks for `students.view`, `students.manage`, and `students.archive`.
- AC-9 (Lifecycle Status Transitions and Delete Protection Triggers): verified by trigger tests aborting hard DELETE statements when financial ledger charges or enrollments exist.
- AC-10 (Student Directory, Profile View, and Shared Guardian UX): verified by UI component tests asserting responsive rendering, tab navigation, search filtering, duplicate warnings, and form submission.
