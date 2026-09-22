# 0011. Demo and load data generator

**Date**: 2026-09-22
**Status**: Accepted

## Summary

Coachify provides a headless dataset generator that builds realistic demo institutions for sales presentations and large datasets for hardware load testing. Operating on pure Rust inside the core library, the generator uses a deterministic random engine and embedded bilingual dictionaries to generate authentic Bangladeshi names, phones, family structures, academic enrollments, and complete financial ledger histories. The generator writes into isolated database files without risking live production records, provides a headless command line runner for performance benchmarks, and allows the desktop shell to switch cleanly into demo mode.

## Requirements

**User stories**:
- As a software sales representative, I want to open Coachify with a preloaded demo institution containing 500 students and a full year of realistic academic and payment history, so that I can demonstrate every workflow to prospective coaching center owners without typing mock data manually.
- As a performance engineer, I want to generate a reproducible load dataset containing 10,000 active students and 3 years of records on low specification hardware, so that I can verify search latencies and memory consumption against institutional release criteria.
- As an institute owner, I want absolute assurance that running a demo or test data generator will never overwrite, corrupt, or modify my live institution database.
- As an intake coordinator or cashier exploring the software during staff training, I want an obvious visual banner whenever the application runs in demo mode, so that I never confuse training exercises with live operational records.
- As an automated test runner or developer, I want a command line interface in the workspace xtask runner to generate test databases headlessly with specific random seeds.

**Acceptance criteria**:

- **AC-1 (Deterministic Pseudo Random Seeding & Repeatability)**:
  - The generator utilizes `ChaCha8Rng` seeded with a 64 bit integer.
  - Headless command line benchmark invocations (`--profile load`) default to constant seed `42` to produce bit for bit identical database records and checksums across all hardware.
  - In app sales demo generations default to a fresh random seed from system entropy so that each generation creates a distinct student cohort, while allowing an optional manual seed input.

- **AC-2 (Authentic Bangladeshi Demographics & Sibling Links)**:
  - Student and guardian records use embedded bilingual dictionaries of authentic Bangladeshi first names, family surnames, and common neighborhoods (for example Farmgate, Mirpur, Dhanmondi, Uttara).
  - Phone numbers are generated using authentic Bangladeshi mobile prefixes (`017`, `018`, `019`, `013`, `016`, `015`) and normalized to 13 digits (`8801...`).
  - Gender distribution models realistic coaching demographics (approximately 52 percent male and 48 percent female).
  - Approximately 20 percent of students are linked as siblings to shared guardian records: siblings share their family surname, have a 1 to 4 year age difference, and are assigned to separate academic batches, satisfying the shared family invariant established in spec 0008.

- **AC-3 (Academic Structure & Multi Batch Enrollments)**:
  - Creates realistic academic sessions (for example '2025' completed session and '2026' active session).
  - Generates standard coaching programs (such as HSC Science, HSC Commerce, SSC General, Class 8 Junior) with realistic monthly tuition fee heads.
  - Divides students into batches of 40 to 60 students with sequential roll numbers, with each student enrolled in 1 or 2 batches.

- **AC-4 (Complete Financial Ledger Simulation & Temporal Distribution)**:
  - Generates monthly tuition charges and historical payments across the configured timeframe.
  - Monetary values are stored as integer poisha with matching Asia/Dhaka business dates.
  - Tuition fee charges post on the 1st day of each calendar month.
  - Payments follow a realistic temporal distribution: payments cluster between the 5th and 10th of the month (the primary fee collection window in Bangladesh coaching centers).
  - Simulates realistic payment collection statuses: approximately 85 percent of charges fully paid, 10 percent partially paid or overdue, and 5 percent granted fee concessions.

- **AC-5 (Two Distinct Generation Profiles)**:
  - `demo`: Generates 500 active students, 4 programs, 12 batches, and 1 full year of charges and payments; completes in under 5 seconds.
  - `load`: Generates 10,000 active students, 15,000 archived students, and 3 years of financial records; generates repeatably in under 60 seconds on spinning hard disks.

- **AC-6 (FTS5 Trigram Search Index Integrity & Performance)**:
  - All generated students are indexed in SQLite FTS5 with trigram tokenization via standard database triggers.
  - Substring search queries across the 10,000 student dataset execute in under 50 milliseconds on the reference test PC.

- **AC-7 (Bulk Insertion Speed & Transaction Chunking)**:
  - Data insertion runs in chunked transactions of 1,000 records using prepared statements.
  - During generation of the temporary file, the engine sets `PRAGMA synchronous = OFF` and `PRAGMA journal_mode = OFF` to maximize write throughput on spinning hard disks.
  - Prior to renaming, the engine executes `PRAGMA integrity_check`, sets `PRAGMA journal_mode = WAL`, and checkpoints all pages.

- **AC-8 (Pre Flight Disk Space Check & Atomic File Cleanup)**:
  - The generator inspects target drive free space prior to execution, requiring at least 100 MB for the demo profile and 1 GB for the load profile.
  - Data is written to a temporary file (`<target>.tmp`) in `<data_root>/demo/`, which is atomically renamed to the destination file upon successful completion.
  - In the event of an IO error, user cancellation, or panic, any partial `.tmp` file is immediately deleted.

- **AC-9 (Path Sandboxing & Production Database Protection Guard)**:
  - Target output paths are strictly constrained to the application demo directory (`<data_root>/demo/`), rejecting any path containing directory traversal elements (`..`).
  - Target paths and the active production database path are resolved to canonical paths via `std::fs::canonicalize`.
  - Any attempt to target the active production database path is rejected with a fatal configuration error (`DbError::Config`), preventing accidental overwrite.

- **AC-10 (Synthetic Metadata Identification Table)**:
  - Creates a dedicated `demo_metadata` table in the generated database recording profile name, seed, total student count, generated timestamp (UTC ISO 8601), and generator version.

- **AC-11 (Dynamic Connection Pool Switching & State Invalidation)**:
  - The core connection manager provides atomic operations to checkpoint and close active database handles and switch to a target demo database, or return to the production database via `generator.switch_to_production_database`.
  - `run_startup_checks` runs against the newly opened database to verify table constraints and page integrity before processing application commands.
  - Upon switching databases, the desktop user interface clears all cached data in TanStack Query and resets client session state, preventing memory cache leaks between databases.
  - Upon fresh application launch or restart, Coachify always loads the real production database by default.

- **AC-12 (Headless CLI Workspace Integration & Headless Audit Identity)**:
  - A command line tool in `xtask` (`cargo run -p xtask generate-dataset`) exposes flags `--profile <demo|load>`, `--seed <u64>`, and `--out <path>`, printing live console progress.
  - Headless CLI invocations log audit records using the reserved system operator identifier `00000000-0000-0000-0000-000000000000` (`SYSTEM_CLI`).

- **AC-13 (Developer UI Dialog & Demo Mode Status Banner)**:
  - The desktop user interface provides a developer dialog accessible in Settings (under debug builds or for users with the Owner role) to trigger demo generation with an active progress bar.
  - When the application is connected to a database containing a `demo_metadata` table, a persistent warning badge is displayed in the application header indicating "Demo Mode (Synthetic Data)".

- **AC-14 (Cryptographic Audit Logging)**:
  - Initiating demo dataset generation or switching the active database connection writes an immutable record to the audit log with the operator user ID and target profile.

## Decision

**Chosen option**: Option 1: Pure Rust headless generator in `crates/core` with deterministic `ChaCha8Rng`, embedded bilingual dictionaries, chunked bulk transactions, path sandboxing, CLI xtask runner, and dynamic desktop database switcher with TanStack Query cache clearing.

You build the demo and load data generator as a native module in `crates/core/src/generator.rs` with zero external network dependencies, isolated SQLite database file targets, and a typed RPC bridge for Tauri.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:

```sql
-- Metadata table written only in generated synthetic databases
CREATE TABLE IF NOT EXISTS demo_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    profile TEXT NOT NULL,           -- 'demo_500' or 'load_10k'
    seed INTEGER NOT NULL,           -- 64 bit integer seed
    student_count INTEGER NOT NULL,  -- total students generated
    generated_at TEXT NOT NULL,      -- UTC ISO 8601 timestamp
    generator_version TEXT NOT NULL  -- application version tag
);
```

**State transitions**:

```
[Idle] ---> (Validate Target, Sandbox & Disk Space) ---> [Preparing]
                                                               |
                                                               v
[Aborted/Cleaned] <--- (Error or Cancel) <--- [Generating Chunks (sync=OFF)]
                                                               |
                                                               v
[Complete] <--- (Atomic Rename, Checkpoint, WAL Mode) <--------+
```

**API surface**:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `generator.generate_dataset` | RPC command | `profile: DemoProfile`, `seed: Option<u64>`, `out_filename: Option<String>` | `summary: GenerationSummary` | Owner or Debug | `400 invalid profile`, `403 forbidden`, `409 target is active db`, `507 low disk space` |
| `generator.get_status` | RPC query | None | `status: GeneratorStatus`, `percent: u8`, `stage: String`, `is_demo_active: bool` | Authenticated | `500 internal error` |
| `generator.switch_database` | RPC command | `target_filename: String` | `success: bool`, `active_path: String` | Owner or Debug | `403 forbidden`, `404 file missing`, `422 integrity check failed` |
| `generator.switch_to_production_database` | RPC command | None | `success: bool`, `production_path: String` | Owner or Debug | `403 forbidden`, `500 internal error` |

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| `generator.generate_dataset` | Profile configuration | Input parameter `profile` |
| `generator.generate_dataset` | Random seed | Input parameter `seed`, falling back to constant `42` for CLI and random entropy for UI |
| `generator.generate_dataset` | Target file path | Derived from `<data_root>/demo/<profile>.db` or sandboxed input `out_filename` |
| `generator.generate_dataset` | Institution profile | Embedded demographic dictionary selected by seed |
| `generator.generate_dataset` | Student names & phones | Embedded Bengali and English name tables combined via `ChaCha8Rng` |
| `generator.generate_dataset` | Sibling guardian links | Sibling grouping algorithm sharing surnames and guardians with 1 to 4 year age gaps |
| `generator.generate_dataset` | Fee charges & payments | Monthly schedule posted on the 1st with payments clustered between the 5th and 10th |
| `generator.switch_database` | Target file path | Input parameter `target_filename` inside `<data_root>/demo/` |
| `generator.switch_database` | Operator user ID | Session token from command dispatcher context, or `SYSTEM_CLI` for headless runs |
| `generator.switch_database` | Audit log entry | Cryptographic hash chain from `crate::audit` |
| `generator.switch_to_production_database` | Production path | Cached production path loaded from `crate::storage::load_global_config()` |

**Key invariants**:
- The generator strictly refuses to execute if the canonicalized target database path matches the canonicalized active production database path.
- All target output files must reside inside `<data_root>/demo/`, with directory traversal strictly prohibited.
- The generator always writes to a temporary file (`.tmp`) and renames only after a successful `PRAGMA integrity_check`.
- When invoked with the same profile and seed, output records are bit for bit identical.
- Financial records are stored exclusively as integer poisha (`i64`), with UTC timestamps accompanied by Asia/Dhaka business dates.
- Database switching flushes and invalidates all client side TanStack Query caches.
- Fresh application launches always default to opening the real production database.

**Security model**:
- In desktop production builds, triggering demo data generation and switching databases is restricted exclusively to users holding the `Owner` role.
- In debug builds, generator commands are accessible to facilitate automated integration tests and developer workflows.
- Switching into a demo database leaves the production database file closed, unlocked, and completely unaltered.

**Configuration required**:
- None. All data dictionaries and generation logic are embedded in the core binary with zero external API keys or network dependencies.

**Critical test scenarios**:
- Happy path: Generate the 500 student demo profile to `demo.db`, verify that generation completes in under 5 seconds, switch the active connection, assert that TanStack Query cache is cleared, and assert that student search returns results in under 50 milliseconds, verifies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, **AC-6**, **AC-11**.
- Repeatability check: Generate two separate databases using seed `12345` and assert that table row counts, student codes, and hash checksums match identically, verifies **AC-1**.
- Sibling validation: Inspect generated students and assert that sibling pairs share identical family surnames and guardians while attending distinct class batches, verifies **AC-2**.
- Payment temporal clustering: Verify that payment receipts cluster between the 5th and 10th of each calendar month, verifies **AC-4**.
- Protection guard: Attempt to run the generator targeting the active production database path and verify that the command fails with a safety error without modifying the file, verifies **AC-9**.
- Path traversal rejection: Attempt to generate a file with path `../../system.db` and assert that the request is rejected with a validation error, verifies **AC-9**.
- Return to production: Call `generator.switch_to_production_database` and assert that Coachify reloads the original production database and removes the Demo Mode header banner, verifies **AC-11**, **AC-13**.
- Low disk space or abort: Simulate an IO error or low disk space condition and assert that no orphaned `.tmp` file remains on disk, verifies **AC-8**.
- Role permission: Attempt to call `generator.generate_dataset` as a cashier user in a release build and assert that permission is denied, verifies **AC-13**, **AC-14**.

## Build plan

1. Create the `generator` core module in `crates/core/src/generator.rs` with embedded bilingual dictionaries (Bengali first names, surnames, Dhaka locations, phone prefixes) and deterministic `ChaCha8Rng` seeding, satisfies **AC-1**, **AC-2**.
2. Implement demographic and academic entity generation logic (institution profile, academic sessions, programs, batches, students, and shared sibling guardians with age gaps and distinct batch assignments), satisfies **AC-2**, **AC-3**.
3. Implement financial ledger simulation generating fee plans, tuition charges posted on the 1st, payments clustered between the 5th and 10th, integer poisha balances, and Asia/Dhaka business dates, satisfies **AC-4**.
4. Build the chunked bulk insertion pipeline with temporary bulk PRAGMAs (`synchronous = OFF`, `journal_mode = OFF`), transaction batching of 1,000 records, integrity checking, checkpointing, and WAL transition, satisfies **AC-5**, **AC-7**.
5. Implement pre flight disk space inspection, path canonicalization, sandbox directory restriction, atomic temporary file creation (`.tmp`), failure cleanup handlers, and the production database safety guard, satisfies **AC-8**, **AC-9**, **AC-10**.
6. Implement dynamic connection pool switching in `crates/core/src/db.rs` (including `switch_to_production_database`), startup integrity checks, and cryptographic audit logging with `SYSTEM_CLI` support, satisfies **AC-11**, **AC-12**, **AC-14**.
7. Implement workspace CLI command in `crates/xtask` (`cargo run -p xtask generate-dataset`) with live progress reporting and default seed `42`, satisfies **AC-1**, **AC-12**.
8. Build the developer settings dialog in the desktop UI (`ui/src/components/settings/demo-generator-dialog.tsx`), TanStack Query cache reset on database switch, and the top bar Demo Mode indicator banner, satisfies **AC-11**, **AC-13**.
9. Write comprehensive integration tests in `crates/core/tests/generator.rs` verifying 500 student demo and 10,000 student load generation repeatability, payment clustering, sibling surname matching, FTS5 trigram performance, and safety guards, satisfies **AC-1**, **AC-2**, **AC-4**, **AC-5**, **AC-6**, **AC-9**.

## Consequences

**Positive**:
- Sales representatives can demonstrate a vibrant coaching institution with complete historical records on offline laptops.
- Performance benchmarks on low end hardware can be reproduced deterministically using fixed random seeds.
- Zero risk of production data corruption due to strict path sandboxing, canonicalization, and atomic file operations.
- Dynamic database switching resets query caches cleanly without requiring full application restarts.

**Negative / tradeoffs**:
- Generating 10,000 active students, 15,000 past students, and 3 years of ledger history produces an SQLite database of roughly 150 MB to 250 MB, requiring temporary disk headroom.
- Embedded name dictionaries add approximately 40 KB of static strings to the core binary.

**Neutral**:
- Requires export of new Specta RPC types for `generator.*` commands into `ui/src/gen/rpc.ts`.

## Follow-up

- [ ] Export updated TypeScript contract types using `cargo run -p xtask export-types` once generator commands are wired.
- [ ] Connect attendance generation to this generator once Feature 25 (Attendance) is implemented in Slice 4.
