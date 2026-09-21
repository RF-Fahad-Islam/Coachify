# Verify: Demo and load data generator (spec 0011, updated 2026-09-22)

Steps derived from spec 0011 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] Log in as an Administrator or Owner, open Settings, and verify that the "Developer Tools & Demo Generator" section is accessible under debug builds or for the Owner role (verifies AC-13)
- [ ] In the Developer Tools section, select the "Sales Demo (500 Students)" profile and click "Generate Demo Database": verify that an active progress dialog displays chunk progress percentages and elapsed time (verifies AC-5, AC-13)
- [ ] Confirm that generation of the 500 student demo profile completes in under 5 seconds, displaying a generation summary with total students, batches, and charges created (verifies AC-5, AC-13)
- [ ] In the completion modal, click "Switch to Demo Database": confirm that the desktop application dynamically reloads without restarting, clears the TanStack Query cache, and displays a high visibility "Demo Mode (Synthetic Data)" banner across the application header (verifies AC-11, AC-13)
- [ ] Navigate to the Students directory while in Demo Mode: verify that student names appear in authentic Bengali and English, that phone numbers reflect valid Bangladeshi prefixes (`017`, `018`, `019`), and that substring search returns results in under 50 milliseconds (verifies AC-2, AC-6)
- [ ] Click on several student profiles: verify that approximately 20 percent of students share a guardian profile with matching family surnames, confirming sibling grouping across separate class batches (verifies AC-2)
- [ ] Navigate to the Academic Batches tab: confirm that batches contain realistic class rosters of 40 to 60 students with sequential roll numbers (verifies AC-3)
- [ ] Open the Cashier module: verify that historical tuition fee charges post on the 1st and payment receipts cluster between the 5th and 10th of each calendar month with integer poisha balance calculations (verifies AC-4)
- [ ] Return to Settings, click "Return to Production Database", and confirm that the application switches cleanly back to the live database, clears query cache, and removes the Demo Mode header banner (verifies AC-11, AC-13)
- [ ] Close the application completely and relaunch it: verify that the application launches into the live production database by default, rather than staying trapped in demo mode (verifies AC-11)
- [ ] Check the Audit Log viewer and verify that the database generation and switching operations are recorded as immutable audit log entries (verifies AC-14)

## Commands

- [ ] `cargo test -p coachify_core --test generator` (runs comprehensive core integration tests verifying deterministic seeding repeatability, authentic demographic generation, sibling surname grouping, payment clustering, multi batch enrollments, financial ledger simulation, bulk transaction performance, disk space pre checks, atomic temporary file cleanup, path sandboxing against directory traversal, production database protection guards, and dynamic database switching, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, AC-10, AC-11, AC-14)
- [ ] `cargo run -p xtask generate-dataset --profile demo --seed 42 --out target/test_demo.db` (verifies headless CLI generation of the 500 student sales demo profile with exit code 0, verifies AC-1, AC-5, AC-12)
- [ ] `cargo run -p xtask generate-dataset --profile load --seed 42 --out target/test_load.db` (verifies headless CLI generation of 10,000 active students and 3 years of records in under 60 seconds, verifying that audit log entries use SYSTEM_CLI, verifies AC-1, AC-5, AC-6, AC-7, AC-12)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript contract bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `generator.*`, verifies AC-13)
- [ ] `cargo clippy -p coachify_core --all-targets -- -D warnings` (verifies clean compilation with zero lint warnings across the generator module)

## Acceptance criteria coverage

- AC-1 (Deterministic Pseudo Random Seeding & Repeatability): verified by integration tests generating two distinct databases with seed `42` and asserting identical table checksums and student codes, while UI invocations default to system entropy.
- AC-2 (Authentic Bangladeshi Demographics & Sibling Links): verified by unit tests asserting authentic name formats, phone normalization prefixes, gender ratios, shared family surnames, and shared guardian links for sibling pairs in distinct batches.
- AC-3 (Academic Structure & Multi Batch Enrollments): verified by tests validating session dates, program structures, batch size bounds (40 to 60 students), and sequential roll number assignments.
- AC-4 (Complete Financial Ledger Simulation & Temporal Distribution): verified by tests verifying integer poisha monetary amounts, monthly tuition fee schedules posted on the 1st, payment clustering between the 5th and 10th, and 85/10/5 payment status distributions.
- AC-5 (Two Distinct Generation Profiles): verified by timing assertions ensuring the demo profile finishes in under 5 seconds and the load profile finishes in under 60 seconds.
- AC-6 (FTS5 Trigram Search Index Integrity & Performance): verified by benchmark tests asserting query execution times under 50 milliseconds across the 10,000 student dataset.
- AC-7 (Bulk Insertion Speed & Transaction Chunking): verified by tests ensuring chunked 1,000 record transactions execute with temporary bulk PRAGMAs without lock timeouts or journal bloating.
- AC-8 (Pre Flight Disk Space Check & Atomic File Cleanup): verified by tests asserting that simulated low disk space errors halt generation and delete temporary `.tmp` files.
- AC-9 (Path Sandboxing & Production Database Protection Guard): verified by tests asserting that targeting the active production database path or providing path traversal input (`../../system.db`) immediately returns a fatal configuration error.
- AC-10 (Synthetic Metadata Identification Table): verified by tests confirming the presence and contents of the `demo_metadata` table in generated databases.
- AC-11 (Dynamic Connection Pool Switching & State Invalidation): verified by tests reinitializing the database pool, executing `PRAGMA quick_check`, asserting that TanStack Query cache is cleared, and verifying default fallback to production on restart.
- AC-12 (Headless CLI Workspace Integration & Headless Audit Identity): verified by testing `cargo run -p xtask generate-dataset` execution with various flags and asserting that audit records use the `SYSTEM_CLI` operator identifier.
- AC-13 (Developer UI Dialog & Demo Mode Status Banner): verified by UI tests testing the generator modal, progress bar components, and the Demo Mode warning banner.
- AC-14 (Cryptographic Audit Logging): verified by asserting that audit log rows are appended with the operator user ID upon dataset generation and database switching.
