# 0036. Crash and Failure Safety Suite (Feature 58)

**Date**: 2026-09-24

## Summary

This specification establishes an automated crash safety test suite and environmental failure simulation framework for Coachify. It verifies that unexpected computer power cuts and peripheral interruptions never corrupt data, generate duplicate transactions, or leave incomplete records. A dedicated cargo xtask test runner launches worker processes and issues immediate process terminations on Windows during sensitive transactions. In addition, controlled in process hooks simulate missing USB drives, low disk storage, and backward clock changes to prove reliable desktop behavior.

## Requirements

**User stories**:
* As a coaching center owner subject to frequent electrical outages, I want payments and student records to survive sudden computer shutdowns, so that our accounts remain completely accurate without missing money or duplicate charges.
* As a counter cashier, I want to record tuition payments reliably even if the receipt printer is unplugged, so that parents receive confirmation and desks do not stall.
* As a receptionist taking admissions, I want the system to warn me if computer disk space is running out before it stops saving files, so that we can clean up files safely.
* As an administrator, I want Coachify to stop financial entries if someone rolls back the computer clock, so that tamper attempts and dead CMOS battery errors are caught immediately.

**Acceptance criteria**:
* **AC-1**: The cargo xtask crash runner (`cargo run -p xtask crash-suite --scenario payments --count 20`) executes 20 forced process terminations during payment transactions, verifying on restart that each payment either committed in full (ledger entry, allocations, receipt number, and audit log) or left zero partial rows, with no database corruption and no duplicate payment on replay with the same idempotency key.
* **AC-2**: The crash runner (`cargo run -p xtask crash-suite --scenario backups --count 10`) executes 10 forced process terminations during encrypted backup creation, verifying on restart that the previous backup remains valid, partial temporary files (`.tmp-backup-*.db`) are removed by startup sweeps, and subsequent backups succeed.
* **AC-3**: The crash runner (`cargo run -p xtask crash-suite --scenario migrations --count 10`) executes 10 forced process terminations during migration execution, verifying on restart that the original database is intact, the temporary migration sandbox is discarded, and the migration re runs cleanly to completion.
* **AC-4**: In process fault injection hooks behind compile time test feature `fault-injection` simulate peripheral and environmental failures including USB drive removal mid backup, low disk space warnings below 2 GB, clock shifts backwards beyond 60 seconds, and SMS delivery timeouts.
* **AC-5**: In a simulated printer offline failure, payments commit successfully to the financial ledger, and the cashier interface displays an on demand reprint banner without rolling back the transaction, verified by UI component tests.
* **AC-6**: When available disk space on either the database or backup volume drops below 2 GB, the application issues a visible non blocking operator warning; when free space drops below 500 MB, backups and software updates refuse with a clear explanation while everyday read and write operations continue safely.
* **AC-7**: Backward clock changes beyond 60 seconds relative to `SELECT MAX(at) FROM audit_log` prevent financial postings with a clear operator message to fix the system clock, while allowing student searches and attendance viewing to proceed.

## Decision

**Chosen option**: Dedicated xtask Process Runner with Compile Time Fault Injection Hooks

We implement a dedicated test orchestrator in `xtask/src/crash_suite.rs` executed via `cargo run -p xtask crash-suite`, with child worker binary target in `xtask/src/bin/crash_worker.rs`. The runner manages worker child processes using standard library process APIs and issues Windows `TerminateProcess` calls at specific transaction checkpoints. Within `crates/core`, we introduce an internal fault module guarded by `#[cfg(feature = "fault-injection")]` that exposes deterministic checkpoints for payments, backups, and migrations, alongside mock switches for missing USB drives, low disk space, and backwards clock jumps.

## Rationale

Real electrical power cuts stop computer execution instantaneously. In memory SQLite transactions or test thread panics run drop handlers and cleanup code that never execute during genuine power loss. Only true operating system process termination validates that write ahead logging and immediate transactions protect disk integrity. Gating all fault injection hooks behind a dedicated Cargo feature ensures that production distribution builds contain zero testing code or security backdoors.

## Feature design

**Architecture & process flow**:

```
+-------------------------------------------------------------+
|              cargo xtask crash-suite (Parent)               |
+-------------------------------------------------------------+
     |                                             ^
     | 1. Spawns child with target scenario        | 3. Child signals
     |    and fault injection flags                |    "CHECKPOINT_REACHED"
     v                                             |    and blocks on stdin
+--------------------------------------------------+----------+
|            crash_worker (Child Process)                     |
|  - Runs real crates/core logic with SQLite database         |
|  - Reaches fault checkpoint inside BEGIN IMMEDIATE block     |
|  - Writes "CHECKPOINT_REACHED\n" to stdout                  |
|  - Blocks indefinitely on stdin.read_exact()                 |
+-------------------------------------------------------------+
     |
     | 4. Parent issues Windows TerminateProcess(child, 1)
     v
+-------------------------------------------------------------+
|              Verification Phase (Parent)                    |
|  - Spawns child in recovery check mode                      |
|  - Runs PRAGMA integrity_check                              |
|  - Asserts ledger sum and audit hash chain integrity        |
|  - Replays original command to verify idempotency           |
+-------------------------------------------------------------+
```

**Fault injection checkpoints**:
* `payment_before_commit`: Pauses child immediately after inserting fee allocations, right before `tx.commit()`.
* `backup_mid_stream`: Pauses child while streaming snapshot pages into the temporary `.tmp-backup-{kind}-{safe_ts}.db` file.
* `migration_mid_script`: Pauses child while executing SQL statements in a temporary database clone before swap.

**Clock guard rules**:
* Watermark definition: `SELECT MAX(at) FROM audit_log`. If database has zero audit rows, current time is accepted.
* Drift tolerance: 60 seconds backward drift allowed to accommodate minor NTP adjustments.
* Blocked commands: `payments.create`, `payments.void`, `students.admit_with_payment`, `adjustments.create`, `expenses.create`.
* Emergency unblock: Owner can invoke existing `acknowledge_clock_jump` RPC command if computer clock was permanently reset.

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| Trigger crash test | Test run result JSON | xtask execution monitor |
| Verify ledger recovery | Row count and balance sum | Database query on `financial_ledger` |
| Verify audit log | Hash chain consistency | `coachify_core::audit::verify_audit_log_chain` |
| Verify idempotency | Existing receipt number | Lookup in `payments.idempotency_key` |
| Check disk space | Available free bytes | Windows API call `GetDiskFreeSpaceExW` on DB and backup paths |
| Check clock validity | Clock difference error | Comparison between current UTC and latest audit row timestamp |

**Key invariants**:
* The `fault-injection` feature must be disabled by default in `crates/core/Cargo.toml`.
* Child process termination during a transaction must never leave orphaned allocation rows without a parent payment record.
* Retrying a crashed payment with the same idempotency key must return the original receipt number if committed, or create a single new record if uncommitted.
* Printing errors in the UI must never cause core financial transactions to roll back.

**Security model**:
* The test worker binary is built only during automated test execution in the `target/` directory.
* Fault injection APIs cannot be called via RPC in production binaries because the feature is omitted from release builds.

**Critical test scenarios**:
* Scenario 1: 20 forced terminations during payments; on reboot, all 20 databases report 0 errors on `PRAGMA integrity_check`, verifying **AC-1**.
* Scenario 2: 10 forced terminations during backups; on reboot, temporary files (`.tmp-backup-*.db`) in the backup directory are purged and prior backups remain readable, verifying **AC-2**.
* Scenario 3: 10 forced terminations during migrations; on reboot, the target database remains at the previous valid migration version and reapplies cleanly, verifying **AC-3**.
* Scenario 4: Simulated printer offline status during fee collection; cashier desk receives success response and displays reprint banner, verifying **AC-5**.
* Scenario 5: Simulated 1 GB free disk space triggers low space alert; simulated 300 MB space blocks backup creation with explicit storage error, verifying **AC-6**.
* Scenario 6: Simulated backwards clock shift of 3 hours halts cashier payments with friendly error while attendance queries succeed, verifying **AC-7**.

## Build plan

1. Define compile time `fault-injection` feature in `crates/core/Cargo.toml` with thread local checkpoint controls, satisfies **AC-4**
2. Add checkpoint hooks to `crates/core/src/ledger.rs` (payments), `crates/core/src/backup.rs` (backups), and `crates/core/src/migrations.rs` (migrations), satisfies **AC-1**, **AC-2**, **AC-3**
3. Implement `xtask/src/bin/crash_worker.rs` and `xtask/src/crash_suite.rs` with child process spawning, pipe listening, and Windows `TerminateProcess` calls, satisfies **AC-1**
4. Implement payment crash test scenario running 20 iterations with restart integrity check and idempotency replay verification, satisfies **AC-1**
5. Implement backup crash test scenario running 10 iterations with temporary file sweep verification in `crates/core/src/db.rs`, satisfies **AC-2**
6. Implement migration crash test scenario running 10 iterations with clean reapplication verification, satisfies **AC-3**
7. Implement peripheral failure simulators in `crates/core` for USB disconnects, low disk storage, and clock jumps, satisfies **AC-4**, **AC-6**, **AC-7**
8. Expose `cargo xtask crash-suite` command and verify output report generation in `target/crash-reports/`, satisfies **AC-1**, **AC-2**, **AC-3**

## Consequences

**Positive**:
* Guarantees that coaching center data is safe against abrupt power loss without requiring physical equipment.
* Cashiers and receptionists can operate without fear of printer jams or hardware disconnects corrupting ledger totals.

**Negative / tradeoffs**:
* Spawning and terminating 40 separate operating system processes takes approximately 2 to 4 minutes to complete.

**Neutral**:
* The test harness requires Windows APIs for process termination, running natively on Windows developer workstations and CI runners.
