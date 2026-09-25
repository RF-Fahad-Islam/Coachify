# 0036. Release Hardening (Slice 11 Umbrella)

**Date**: 2026-09-24
**Status**: Proposed

## Summary

This umbrella specification establishes the release hardening foundation for Coachify version 1.0. It defines an automated crash safety test suite, full customer documentation with an internal support runbook, a lightweight in app help viewer, and a structured three institution pilot readiness protocol. The architecture guarantees that unexpected power cuts, printer disconnects, low storage, and clock changes never corrupt data or create duplicate transactions. Everything operates completely offline, protecting coaching center records and keeping daily desk operations reliable.

## Structure

This umbrella decision coordinates three focused child specifications for Slice 11:

1. [0036-crash-and-failure-safety-suite.md](0036-crash-and-failure-safety-suite.md): Automated process termination harness and environmental fault injection covering Feature 58.
2. [0036-customer-docs-and-support-runbook.md](0036-customer-docs-and-support-runbook.md): Complete customer documentation, internal support procedures, and in app help dialog covering Feature 60.
3. [0036-pilot-readiness-and-release-criteria.md](0036-pilot-readiness-and-release-criteria.md): Four week three center pilot protocol and release sign off matrix covering Feature 61.

Feature 59 (Performance on the reference PC) is excluded from 1.0 scope per user decision, and deferred to the 1.1 cycle.

## Requirements

**User stories**:
* As an institution owner facing frequent power cuts (load shedding), I want the application to survive sudden computer shutdowns without corrupting financial ledgers or creating duplicate payments, so that my coaching center accounts remain completely accurate.
* As a counter cashier operating during a busy admission morning, I want payments to post safely even if the receipt printer runs out of paper or disconnects, so that student queues move smoothly without losing transaction records.
* As a non technical receptionist setting up Coachify for the first time, I want clear illustrated guides and an in app help window, so that I can import students, configure printers, and perform backups without calling support.
* As a technical support specialist answering emergency calls from coaching owners, I want a concrete troubleshooting runbook for antivirus alerts, printer jams, and disaster recovery drills, so that I can resolve field issues quickly.
* As the product release manager, I want an evidence based readiness report verifying at least three pilot institutions over four weeks of live operations, so that we sell version 1.0 with proven confidence.

**Acceptance criteria**:
* **AC-1**: The cargo xtask crash runner (`cargo run -p xtask crash-suite --scenario payments --count 20`) executes 20 automated forced process terminations during payment transactions, verifying on restart that each payment either fully posted with receipt and audit log or leaves zero partial rows, with no database corruption and no duplicate charge on replay with the same idempotency key.
* **AC-2**: The crash runner executes 10 automated forced process terminations during encrypted backup creation, verifying on restart that the previous verified backup remains intact, partial temporary archive files (`.tmp-backup-*`) are cleaned up by startup sweeps, and subsequent backups succeed.
* **AC-3**: The crash runner executes 10 automated forced process terminations during database migration execution, verifying on restart that the original database is intact, the temporary migration sandbox is discarded, and the migration re runs cleanly to completion.
* **AC-4**: In process fault injection hooks behind compile time test feature `fault-injection` simulate environmental failures including USB drive removal mid backup, low disk space warnings below 2 GB, clock shifts backwards beyond 60 seconds, and SMS delivery timeouts.
* **AC-5**: In a printer offline failure, payments commit successfully to the financial ledger, and the cashier interface displays an on demand reprint banner without rolling back the transaction, verified by UI component tests.
* **AC-6**: When available disk space on either the database or backup volume drops below 2 GB, the application issues a visible non blocking operator warning; when free space drops below 500 MB, backups and software updates refuse with a clear explanation while everyday read and write operations continue safely.
* **AC-7**: Backward clock changes beyond 60 seconds relative to the latest audit watermark (`MAX(at) FROM audit_log`) prevent financial postings with a clear operator message to fix the system clock, while allowing student searches and attendance viewing to proceed.
* **AC-8**: The desktop shell provides a lightweight in app Help dialog accessible from the top navigation bar, rendering bundled markdown documentation loaded directly at build time by Vite with zero internet dependency.
* **AC-9**: Comprehensive customer documentation is authored in `docs/customer/` with printable PDF versions covering Quick Start, Backup and Restore, Thermal and Laser Printer Setup, SMS Gateway Configuration, Excel Student Import, Troubleshooting (including Windows SmartScreen and antivirus exceptions), Privacy and Data Ownership, and the Owner License Guide.
* **AC-10**: An internal support runbook in `docs/customer/support-runbook.md` provides Tier 1 and Tier 2 resolution procedures for lost encryption recovery sheets, database corruption recovery, emergency clock jump recovery, damaged thermal printers, and forgotten staff credentials.
* **AC-11**: A structured release qualification document in `docs/release/1.0-readiness-report.md` tracks and validates the release acceptance criteria defined in Section 17 of the architecture review, excluding reference hardware performance benchmarks which are deferred to 1.1.
* **AC-12**: The pilot verification program tracks at least 3 live pilot institutions operating for 4 or more consecutive weeks each, verifying at least one monthly collection cycle, one examination cycle, a successful restore drill at each location, and zero recorded data loss incidents.

## Decision

**Chosen option**: Option 1: Modular Release Hardening with Headless xtask Harness and Bundled Documentation

We establish a dedicated release hardening suite within `crates/core` behind a compile time `fault-injection` test feature, coordinated by `cargo run -p xtask crash-suite` with child worker `xtask/src/bin/crash_worker.rs`. The suite exercises real process terminations on Windows via `TerminateProcess` and verifies SQLite write ahead logging durability. Customer guides and support runbooks live in `docs/customer/`, bundled into the React application at compile time via Vite without adding unnecessary RPC handlers. Release readiness is audited and tracked in `docs/release/1.0-readiness-report.md`.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:
Zero production database migrations are required for Slice 11. All test records, crash drill outputs, and pilot tracking records are persisted as structured JSON files and markdown documents within `docs/release/` and `target/crash-reports/`:

* `docs/release/1.0-readiness-report.md`: Core audit document tracking criteria sign offs, pilot institution telemetry, and restore drill results.
* `docs/release/pilot-matrix.json`: Pilot site registry tracking institution name, active students, weekly operational milestones, and `data_verification_hash` (the `audit_chain_head` string from the latest verified backup).
* `target/crash-reports/crash-summary-<timestamp>.json`: Test run metrics tracking total kills attempted, clean restarts, transaction recovery checks, and execution duration.
* `docs/customer/*.md`: Markdown files imported directly by the UI via Vite for offline in app display.

**State transitions**:

1. Crash Drill Execution State Machine:
   * `Idle` -> `SpawningWorker`: Parent launches child worker binary `xtask/src/bin/crash_worker.rs` with scenario parameters.
   * `SpawningWorker` -> `AwaitingCheckpoint`: Worker reaches checkpoint, writes `"CHECKPOINT_REACHED"` to stdout, and blocks on standard input.
   * `AwaitingCheckpoint` -> `Terminating`: Parent receives notification and calls Windows `TerminateProcess(child_handle, 1)`.
   * `Terminating` -> `VerifyingRecovery`: Parent starts verification pass, running `PRAGMA integrity_check`, verifying ledger sums, and checking idempotency replay.
   * `VerifyingRecovery` -> `Idle`: Results appended to crash summary report.

2. Pilot Site Verification State Machine:
   * `Enrolled` -> `Week1Onboarding`: Initial student import and fee plan setup.
   * `Week1Onboarding` -> `ActiveOperations`: Daily attendance and cashier operations under observation.
   * `ActiveOperations` -> `CycleCompleted`: Monthly billing cycle and exam cycle completed.
   * `CycleCompleted` -> `DrillVerified`: Restore drill successfully conducted from USB backup.
   * `DrillVerified` -> `ReleaseQualified`: Site sign off recorded in readiness report.

**API surface**:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| system.get_system_status | rpc | none | db_disk_free_bytes: u64, backup_disk_free_bytes: u64, low_disk_warning: bool, disk_critical: bool, clock_ok: bool | authenticated | 500 internal |

Note: In app customer documentation requires no new RPC endpoints. Vite bundles `docs/customer/*.md` directly into the client bundle at build time using raw markdown imports.

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| Open help topic | Markdown article text | Static markdown files in `docs/customer/` loaded via Vite |
| Filter help topics | Matching topic titles | Local client search over topic titles and markdown headings |
| Check disk warning | Low disk status booleans | Windows API `GetDiskFreeSpaceExW` on DB and backup drives |
| Check clock status | Clock shift warning banner | Comparison between current UTC and `SELECT MAX(at) FROM audit_log` |
| Verify crash payment | Zero partial payment rows | Core ledger table verification query on startup |
| Verify backup recovery | Backup integrity status | Existing backup verification drill in `crates/core/src/backup.rs` |
| Clean stale backups | Number of files purged | Startup sweep matching `.tmp-backup-*.db` in backup folders |

**Key invariants**:
* The compile time feature `fault-injection` must never be compiled into production distribution binaries.
* Forced process termination at any microsecond must leave the SQLite database consistent under write ahead logging with synchronous FULL.
* A payment command interrupted by power loss must either commit in full (ledger entry, allocations, receipt number, and audit row) or leave zero rows; reissuing the command with the identical idempotency key must never create two charges.
* In app documentation files must be self contained and render cleanly without active internet access or external asset downloads.
* Customer facing documentation must never include secret recovery phrases, private encryption seeds, or internal administrative tokens.

**Security model**:
* The test worker binary is built only during automated test execution in the `target/` directory.
* Fault injection APIs cannot be called via RPC in production binaries because the feature is omitted from release builds.
* Clock rollback detection prevents backdating receipts and fraud while allowing read queries to function.

**Configuration required**:
* `COACHIFY_CRASH_CHECKPOINT`: Environment variable passed by the parent crash runner to instruct child test workers to halt at specific transaction points.

**Critical test scenarios**:
* Scenario 1 (Payment Power Loss): 20 child processes terminated mid transaction during fee collection; all 20 leave zero partial rows on restart and replay idempotently, verifying **AC-1**.
* Scenario 2 (Backup Power Loss): 10 child processes terminated mid backup; all 10 preserve previous backup integrity and clean up temporary scratch files on restart, verifying **AC-2**.
* Scenario 3 (Migration Interruption): 10 child processes terminated mid migration; all 10 discard scratch databases and rerun the migration successfully from step zero, verifying **AC-3**.
* Scenario 4 (Printer Disconnect): Payment collection attempted while receipt printer is disconnected; payment posts to ledger, receipt reprint option appears, and error is logged without transaction rollback, verifying **AC-5**.
* Scenario 5 (Low Storage Warning): Simulated disk space below 2 GB triggers non blocking yellow warning; space below 500 MB blocks backup creation and software updates with explicit storage error, verifying **AC-6**.
* Scenario 6 (Clock Shift Block): System clock set backwards by more than 60 seconds; financial entry commands are rejected with guidance to fix system time, verifying **AC-7**.
* Scenario 7 (In App Help Viewing): User opens the Help modal from the top navigation bar and navigates through all guides while network interface is disabled, verifying **AC-8**.
* Scenario 8 (Documentation Validation): A first time user follows customer documentation to install, import students, and perform a backup drill, verifying **AC-9**, **AC-10**.
* Scenario 9 (Readiness Sign Off): The 1.0 readiness document records verified sign offs across three pilot institutions for four weeks with zero data loss, verifying **AC-11**, **AC-12**.

## Build plan

Tasks follow the Tracer Bullet delivery approach, proving an end to end vertical thread through the crash test harness and in app help viewer before thickening failure simulations and documentation guides:

1. Create compile time `fault-injection` feature in `crates/core` with deterministic checkpoint primitives, satisfies **AC-4**
2. Build `xtask/src/bin/crash_worker.rs` and `xtask/src/crash_suite.rs` capable of launching child workers, listening for checkpoints, and issuing Windows `TerminateProcess`, satisfies **AC-1**
3. Implement automated payment power cut test runner executing 20 forced kills with SQLite startup consistency validation and idempotency replay verification, satisfies **AC-1**
4. Implement automated backup and migration power cut test runners executing 10 forced kills each with temporary file cleanup validation in `crates/core/src/db.rs`, satisfies **AC-2**, **AC-3**
5. Add core fault simulation hooks for low disk space, USB disconnection, and backwards clock changes, satisfies **AC-4**, **AC-6**, **AC-7**
6. Build in app Help modal in React UI with top navigation entry point and Vite markdown import from `docs/customer/`, satisfies **AC-8**
7. Implement `system.get_system_status` RPC endpoint reporting disk free space across database and backup drives alongside clock status, satisfies **AC-6**, **AC-7**
8. Author comprehensive customer guides in `docs/customer/` covering Quick Start, Printer Setup, SMS Setup, Excel Import, and Troubleshooting, satisfies **AC-9**
9. Author internal support runbook in `docs/customer/support-runbook.md` with Tier 1 and Tier 2 incident procedures including emergency clock jump recovery, satisfies **AC-10**
10. Establish pilot program tracking registry in `docs/release/pilot-matrix.json` and create `docs/release/1.0-readiness-report.md` recording qualification criteria, satisfies **AC-11**, **AC-12**

## Consequences

**Positive**:
* Owners gain absolute confidence that sudden electrical blackouts common in Bangladesh will not damage financial accounts or student records.
* Support teams receive a structured troubleshooting playbook, dramatically reducing resolution time for field emergencies.
* Counter cashiers can continue taking student fees even during printer malfunctions without holding up queues.
* The product ships with verified pilot evidence from real coaching centers rather than assumptions.

**Negative / tradeoffs**:
* Running the full automated crash suite takes several minutes because it spawns and terminates multiple separate operating system processes.
* Packaging bundled customer documentation adds a modest amount (under 2 MB) to the desktop application distribution bundle.

**Neutral**:
* Performance benchmarking on 4 GB RAM spinning disk hardware (Feature 59) is deferred to the 1.1 release milestone, allowing 1.0 release efforts to focus strictly on data safety and operational stability.

## Follow-up

- [ ] Re evaluate low end reference computer performance benchmarking during the 1.1 release cycle.
- [ ] Incorporate live feedback from the three pilot coaching centers into customer documentation before general public distribution.
