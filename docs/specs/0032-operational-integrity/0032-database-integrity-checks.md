# 0032-database-integrity-checks. Database Integrity Checks

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 49

**Date**: 2026-09-23

## Summary

This specification establishes comprehensive startup and on demand database integrity verification. Designed to execute in under 3 seconds on reference hardware during application startup, the integrity engine runs SQLite `PRAGMA integrity_check`, `PRAGMA foreign_key_check`, financial balance reconciliation, and cryptographic audit log hash chain validation. If severe corruption or foreign key violations are detected, the system immediately locks write access into safety read only mode and surfaces an urgent recovery prompt directing the owner to restore a verified backup.

## Requirements

**User stories**:
* As an owner, I want Coachify to check database integrity every time the computer boots up, so that sudden power cut corruptions or failing hard drives are identified before bad data multiplies.
* As an accountant, I want continuous validation that posted ledger balances equal the sum of fee payments and adjustments, so that financial discrepancies are surfaced immediately.
* As a compliance officer, I want cryptographic verification of our audit log chain, so that any unauthorized database tampering is detected.

**Acceptance criteria**:
* **AC-1**: Startup database integrity check executes in under 3000 milliseconds on reference PC hardware with 10,000 students and 50,000 ledger records.
* **AC-2**: The check executes four verification layers:
  * Low level SQLite `PRAGMA integrity_check` (or `quick_check` during high frequency startup)
  * Relational integrity via `PRAGMA foreign_key_check`
  * Financial ledger balance reconciliation (sum of debits equals sum of credits across all accounts)
  * Cryptographic SHA256 audit log hash chain validation
* **AC-3**: RPC `integrity.run_database_integrity_checks` allows counter staff or administrators to run a full diagnostic scan on demand.
* **AC-4**: If corruption or foreign key anomalies are detected, the application enters safety read only mode, preventing mutations and displaying an urgent recovery banner with backup restore options.
* **AC-5**: Each check run persists a structured diagnostic row in the `integrity_check_runs` table.

## Decision

**Chosen option**: Four Layer Diagnostic Suite with Protective Read Only Mode

The integrity suite integrates deep SQLite storage checks with application level accounting and cryptographic invariants. Fast quick check runs during startup to preserve sub 3 second boot times, while deep checks run on demand or during scheduled restore drills.

## Rationale

Power cuts are frequent in Bangladesh and budget PCs often lack uninterruptible power supplies. While SQLite WAL mode with synchronous FULL offers strong crash resilience, storage controllers and hard drive bad sectors can still corrupt data pages. Catching corruption at boot time before executing counter writes prevents quiet data rot from spreading into subsequent backups.

## Feature design

**Data model sketch**:
* `integrity_check_runs`: Table created in Migration 0017 with columns `id TEXT PRIMARY KEY`, `check_type TEXT CHECK (check_type IN ('startup', 'scheduled', 'manual'))`, `started_at TEXT`, `completed_at TEXT`, `duration_ms INTEGER`, `pragma_integrity_ok INTEGER`, `foreign_keys_ok INTEGER`, `balance_reconciliation_ok INTEGER`, `audit_chain_ok INTEGER`, `error_details_json TEXT`.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.run_database_integrity_checks` | Mutation | full: bool | DbIntegrityResult | Authenticated | 500 integrity scan failed |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Integrity scan | pragma_ok, foreign_keys_ok | SQLite PRAGMA quick_check and PRAGMA foreign_key_check queries |
| Balance check | balance_reconciliation_ok | SQL query checking sum(amount) from ledger_entries groups |
| Audit chain check | audit_chain_ok | Incremental SHA256 hash recalculation from genesis block |

**Key invariants**:
* Startup integrity check must never exceed a 3000 ms budget on reference hardware.
* Any failure in storage integrity or foreign keys must immediately trigger the process wide read only safety flag.

**Security model**:
* All counter roles (Cashier, Receptionist, Teacher, Admin, Owner) can execute on demand integrity scans.
* When safety read only mode is active, all state modifying RPC commands return a 423 Locked error code.

**Critical test scenarios**:
* Clean startup execution: Runs startup check on reference dataset, verifies all four layers pass in under 3000 ms, verifies AC-1, AC-2, AC-5.
* Balance mismatch detection: Manually corrupts a ledger entry in test database, runs integrity check, asserts balance check fails and errors reported, verifies AC-2 and AC-4.
* Protective read only lock: Simulates PRAGMA integrity failure, verifies subsequent payment collection command is rejected with read only lock error, verifies AC-4.

## Build plan

1. Create `integrity_check_runs` table in Migration 0017, satisfies **AC-5**.
2. Implement four layer integrity check engine in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Integrate startup check into application bootstrap pipeline with sub 3 second benchmark guard, satisfies **AC-1**, **AC-4**.
4. Implement `integrity.run_database_integrity_checks` RPC endpoint, satisfies **AC-3**.
5. Build Database Integrity diagnostics tab and corruption emergency banner in `ui/src/components/restore/database-integrity-view.tsx`, satisfies **AC-4**.

## Consequences

**Positive**:
* Guarantees that corrupted database pages or tampered records are flagged before new transactions are written.
* Reconciles financial ledgers automatically to preserve strict double entry integrity.

**Negative / tradeoffs**:
* Adds between 400 and 1200 milliseconds to the desktop application launch time.
