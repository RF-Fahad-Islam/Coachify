# 0032-disaster-recovery-and-drills. Disaster Recovery Score & Verification Drills

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Features 46 and 47

**Date**: 2026-09-23

## Summary

This specification implements the Disaster Recovery Score and automatic background restore verification drills. The recovery score is a 100 point composite evaluating backup recency (40 points), presence of an external copy (25 points), success of the most recent restore drill (20 points), and core database integrity (15 points). An automatic weekly verification drill proves the integrity of the latest backup snapshot by restoring it into an isolated in memory SQLite database and running comprehensive schema, foreign key, and financial ledger balance checks without interrupting ongoing counter operations.

## Requirements

**User stories**:
* As a coaching center owner, I want a single clear disaster recovery score from 0 to 100 on my dashboard, so that I immediately know whether our business records are safe from hardware failure.
* As an owner, I want automatic weekly drills that actually restore our backups in the background, so that we discover corrupted files before a real disaster strikes.
* As an administrator, I want an alert if our recovery score drops below 70, so that I can plug in a backup USB drive or run a manual backup immediately.

**Acceptance criteria**:
* **AC-1**: RPC `integrity.get_disaster_recovery_score` computes a 100 point score:
  * Backup recency: Up to 40 points (40 for under 24 hours, 20 for 24 to 48 hours, 0 for older than 72 hours).
  * External copy: Up to 25 points (25 if a verified backup exists on a configured USB drive or secondary path).
  * Restore drill success: Up to 20 points (20 for drill passed within 7 days, 10 within 14 days, 0 otherwise).
  * Database integrity: Up to 15 points (15 for passed integrity check, 0 for warnings or errors).
* **AC-2**: The disaster recovery score persists a historical record in `recovery_score_snapshots` and triggers a high priority notification banner if the total score drops below 70.
* **AC-3**: RPC `integrity.run_verification_drill` executes a restore drill by copying the latest backup archive into an isolated temporary in memory SQLite database.
* **AC-4**: The restore drill executes:
  * SQLite `PRAGMA integrity_check`
  * SQLite `PRAGMA foreign_key_check`
  * Total student count and ledger balance reconciliation
  * HMAC manifest signature verification
* **AC-5**: Results are logged in the `restore_drills` table with full diagnostic details and execution duration.

## Decision

**Chosen option**: Headless Memory Sandbox Verification with Weighted Recovery Score Formula

The drill unpacks and mounts the backup database strictly in memory (`sqlite3_open(":memory:")` or isolated temporary folder), completely detached from the live production database. The recovery score formula applies clear deterministic weights, eliminating subjective guesswork.

## Rationale

Most small organizations believe they have backups until a hard disk crashes and they discover their snapshot files are unreadable, truncated, or missing decryption keys. Running automated weekly restore drills in an isolated sandbox guarantees that backups are genuinely restorable. The 100 point score translates complex storage and cryptographic health into an intuitive metric that non technical owners can understand and act upon.

## Feature design

**Data model sketch**:
* `recovery_score_snapshots`: Table created in Migration 0017 with columns `id TEXT PRIMARY KEY`, `calculated_at TEXT`, `total_score INTEGER`, `backup_recency_score INTEGER`, `external_copy_score INTEGER`, `drill_success_score INTEGER`, `db_integrity_score INTEGER`, `warnings_json TEXT`.
* Reuses `restore_drills` table from Migration 0013.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.get_disaster_recovery_score` | Read | none | RecoveryScoreResult | Authenticated | 500 calculation error |
| `integrity.run_verification_drill` | Mutation | backup_id: Option<String> | DrillResult | Authenticated | 404 backup not found, 500 drill error |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Score calculation | total_score and component breakdown | Mathematical evaluation over backup_history, backup_destinations, and restore_drills |
| Restore drill | drill outcome (pass, fail) | Execution of PRAGMAs and balance query in temporary memory database |

**Key invariants**:
* The restore drill must never write to, lock, or modify the active production SQLite database.
* A backup that fails manifest verification or PRAGMA check is marked with status corrupted in `backup_history`.

**Security model**:
* Cashier, Admin, and Owner roles can view the recovery score and trigger verification drills.
* Score alerts below 70 are displayed to Owner and Admin roles on the dashboard.

**Critical test scenarios**:
* Perfect score calculation: Evaluates fresh backup with external destination and passed drill, asserts score is 100, verifies AC-1.
* Degraded score warning: Simulates backup older than 48 hours and missing USB drive, asserts score drops below 70 and warning alert generated, verifies AC-1 and AC-2.
* Memory restore drill: Runs drill on valid backup archive, asserts in memory PRAGMA checks pass and record counts match live snapshot, verifies AC-3, AC-4, AC-5.

## Build plan

1. Create `recovery_score_snapshots` table in Migration 0017, satisfies **AC-2**.
2. Implement recovery score calculation algorithm in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Implement sandbox restore verification drill runner in `crates/core/src/integrity.rs`, satisfies **AC-3**, **AC-4**, **AC-5**.
4. Wire weekly background drill schedule into core background job runner, satisfies **AC-3**.
5. Build Disaster Recovery card and Drill History modal in `ui/src/components/backup/disaster-recovery-view.tsx`, satisfies **AC-1**, **AC-5**.

## Consequences

**Positive**:
* Proactively discovers unreadable or corrupted backups before emergencies happen.
* Gives owners a transparent, actionable metric for data safety readiness.

**Negative / tradeoffs**:
* Unpacking and verifying large databases consumes temporary memory and disk I/O for 15 to 45 seconds.
