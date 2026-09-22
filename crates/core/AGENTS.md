# Coachify Core

## Overview

Pure Rust domain library holding all business logic, local database operations, and data models. Runs headless with zero Tauri or UI dependencies.

## Key files

| File | Owns |
|---|---|
| crates/core/src/lib.rs | Single RPC command handler and Specta type definitions |
| crates/core/src/dispatcher.rs | Central command dispatcher pipeline (session resolution, permission gating, license checks, idempotency tracking, transaction isolation) |
| crates/core/src/accounts.rs | User directory, role definitions, granular permissions, session management, and owner invariant protection |
| crates/core/src/auth.rs | Argon2id password hashing and cryptographically secure session token generation |
| crates/core/src/audit.rs | Cryptographic SHA-256 audit log hash chain and tamper diagnostics |
| crates/core/src/db.rs | SQLite connections, PRAGMA safety settings, integrity and clock guards, transactions |
| crates/core/src/ledger.rs | Financial ledger repository, payment collections, voids, adjustments, refunds, and cashier overviews |
| crates/core/src/fees.rs | Fee heads, fee plans, concessions, and chunked monthly charge generator |
| crates/core/src/attendance.rs | Attendance sessions, keyboard first marking, lock rules, and monthly summaries |
| crates/core/src/comms.rs | Omnichannel outbox for SMS, WhatsApp, and Email, provider adapters, templates, and automations |
| crates/core/src/exam.rs | Examination management, mark entry grid, NCTB GPA 5 calculation, fourth subject bonus, report cards, and tabulation sheets |
| crates/core/src/backup.rs | Encrypted AES-256-GCM backup archives, verification drills, and retention tiers |
| crates/core/src/diagnostics.rs | Structured diagnostics logs, privacy redaction, and offline support bundles |
| crates/core/src/migrations.rs | Released migration hashes and the guard that keeps them unmodified |
| migrations/*.sql | Purpose based schema files, applied in numeric order |

## Commands

```bash
# Run unit tests
cargo test -p coachify_core

# Export TypeScript contract types
cargo run -p xtask export-types
```

## Conventions

- Domain logic stays in core, never in the UI or Tauri shell.
- Return errors as `thiserror` enums mapped to code, params, and optional field.
- Repositories use plain SQL mapped to Rust structs, no heavy ORM.
- Database access uses write ahead logging mode with immediate transactions.
- Money is stored as integer poisha, and rows record a UTC instant next to an Asia/Dhaka business date.
- Immutability of audit rows and posted financial rows is enforced by SQLite triggers, not by Rust code.
- The integrity quick check runs once per process through `run_startup_checks`, never on a per command path.

## Gotchas

- Release builds fail to start without real encryption key management.
- The core has no async runtime, all operations run synchronously.
- A forward clock jump beyond the allowed window stays rejected until `acknowledge_clock_jump` records an operator acknowledgment, and that acknowledgment is consumed by the first jump it lets through.

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
