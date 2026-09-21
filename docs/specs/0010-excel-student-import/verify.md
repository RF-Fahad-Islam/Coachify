# Verify: Excel and CSV student import engine (spec 0010, updated 2026-09-22)

Steps derived from spec 0010 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Log in as Administrator or Intake Coordinator, navigate to the Students module, and confirm that the "Import from Excel" action button is visible in the directory toolbar (verifies AC-10)
- [x] Click "Import from Excel" to open the import wizard: drop a messy `.xlsx` workbook containing multiple sheets and verify that the sheet selector displays sheet names and estimated row counts (verifies AC-1, AC-10)
- [x] Select the target sheet: verify that the column mapping step accurately auto maps English and Bengali headers (for example matching "Student Name", "নাম", "Mobile", "মোবাইল", "DOB", "Father Name") with high confidence badges (verifies AC-2)
- [x] Intentionally modify a column mapping using the dropdown picker and confirm that sample data previews from the first rows update interactively (verifies AC-2, AC-10)
- [x] Proceed to the Validation and Preview step: confirm that the summary cards display correct counts for Total Rows, Ready to Import (Valid), Warnings, and Errors (verifies AC-5)
- [x] Verify that rows containing Bengali numerals (`০১৭...`) and varied date formats (`12/05/2008`, `2008-05-12`) are correctly normalized and marked valid (verifies AC-3)
- [x] Filter the preview table by "Errors": click on an invalid cell to verify that an informative tooltip explains the exact validation failure (for example "Invalid Bangladeshi mobile number" or "Missing student name") (verifies AC-5)
- [x] Filter the preview table by "Warnings": confirm that rows sharing the same guardian phone are flagged with a sibling icon indicating they will be linked to a single shared guardian (verifies AC-4)
- [x] Select an optional academic batch from the batch assignment dropdown, select "Import Valid Rows Only", and click "Start Import" (verifies AC-8, AC-9)
- [x] During execution, confirm that a pre import safety backup notification is displayed and verify on disk that a snapshot file exists in `<data_root>/<institution_id>/backups/` (verifies AC-6)
- [x] Once import completes, verify that the success screen shows the imported count, navigate to the Student Directory, and confirm that imported students appear with sequential student codes and can be searched instantly via FTS5 (verifies AC-7, AC-10)
- [x] Return to the import completion screen, click "Download Failed Rows (CSV)", open the downloaded file in Microsoft Excel or text editor, and confirm that it contains only the rejected rows with the added `_import_error` column explaining why each row failed (verifies AC-8)
- [x] Attempt to run an import with an account lacking `students.manage` permission (such as a read only cashier) and confirm that the import action is hidden or blocked with permission denied (verifies AC-10)

## Commands

- [x] `cargo test -p coachify_core --test excel_student_import` (runs comprehensive core integration tests verifying file inspection across `.xlsx`, `.xls`, and `.csv`, Bengali numeral normalization, flexible date parsing, phone normalization and sibling grouping, in memory validation matrix, SQLite online backup snapshot generation, atomic single transaction commits, rollback on simulated constraint failure, rejected rows CSV file generation, and RBAC permission gating, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, AC-10)
- [x] `cargo run -p xtask check-types` (verifies that TypeScript contract bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `import.*`, verifies AC-10)
- [x] `pnpm --dir ui test` (runs UI component tests for `import-students-dialog.tsx`, testing file upload dropzone, sheet picker, column mapper heuristics, validation status tabs, and error CSV download triggers, verifies AC-2, AC-5, AC-10)
- [x] `cargo clippy -p coachify_core --all-targets -- -D warnings` (verifies clean compilation with zero lint warnings across the import engine)

## Acceptance criteria coverage

- AC-1 (File Format Inspection & Sheet Selection): verified by core unit tests parsing sample `.xlsx`, `.xls`, and `.csv` files with row count and sheet name assertions.
- AC-2 (Bilingual Column Auto Mapping): verified by heuristic tests matching English and Bengali aliases (`name`, `নাম`, `phone`, `মোবাইল`, `dob`, `জন্ম তারিখ`) with confidence thresholds.
- AC-3 (Bilingual Value Parsing & Normalization): verified by unit tests parsing Bengali digits into ASCII numbers, converting diverse date strings and Excel serials, and validating required fields.
- AC-4 (Phone Normalization, Conflict Detection & Sibling Grouping): verified by tests ensuring phone numbers normalize to 13 digits, existing database conflicts trigger warnings, and multiple rows sharing a guardian phone resolve to one guardian ID.
- AC-5 (In Memory Validation Matrix): verified by tests calculating total, valid, warning, and error row counts with cell level issue diagnostics without touching SQLite.
- AC-6 (Pre Import Database Safety Snapshot): verified by tests asserting that `rusqlite::backup::Backup` creates a verifiable point in time snapshot file prior to write transaction execution.
- AC-7 (Atomic Database Transaction & Rollback Guarantee): verified by tests asserting that a simulated failure during bulk insert rolls back all rows, leaving zero orphan records.
- AC-8 (Selective Import Mode & Rejected Rows CSV Export): verified by tests importing valid rows while writing rejected rows to a CSV file with an accurate `_import_error` column.
- AC-9 (Optional Academic Batch Enrollment): verified by tests enrolling valid imported students into a batch with auto sequential roll numbers.
- AC-10 (Typed RPC API & Desktop Wizard UX): verified by Specta contract checks, permission gating tests for `students.manage`, and UI component tests for the step by step wizard dialog.
