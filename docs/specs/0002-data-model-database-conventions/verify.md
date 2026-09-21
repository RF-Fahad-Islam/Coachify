# Verify: Data model and database conventions (spec 0002, updated 2026-09-20)

Steps derived from spec 0002 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [x] Inspect database schema using SQLite CLI or testing harness to confirm all foundational, academic, and financial ledger tables exist (verifies AC-3)
- [x] Attempt manual update or deletion on an audit log row or posted payment using raw SQL and verify that SQLite aborts the operation with an error (verifies AC-4)

## Commands
- [x] `cargo test -p coachify_core --test database_conventions` -> Tests PRAGMA settings, table schemas, unique constraints, foreign keys, and trigger abort behavior across people and financial tables (verifies AC-1, AC-3, AC-4, AC-5, AC-6)
- [x] `cargo test -p coachify_core --test crash_recovery` -> Tests integrity check on clean and corrupted headers and verifies transaction rollback safety (verifies AC-1, AC-2)
- [x] `cargo run -p xtask check-migrations` -> Verifies released migrations manifest remains unmodified and migration 0001 is untouched (verifies AC-3)

## Acceptance criteria coverage
- AC-1 (WAL mode, synchronous = FULL, foreign keys ON, busy timeout): verified by database connection test and crash recovery tests.
- AC-2 (integrity quick check on startup): verified by `verify_integrity()` unit tests against valid and simulated corrupt databases.
- AC-3 (purpose based migrations 0002 to 0006 comprehensive schema): verified by migration execution tests and table structure assertions across core and financial tables.
- AC-4 (immutability triggers on audit log, clock guard, posted payments, charges, payment allocations, adjustments, refunds, and closed cash days): verified by integration tests attempting update and delete operations.
- AC-5 (integer poisha, basis points, and UUIDv7 text keys): verified by model serialization and database insert assertions.
- AC-6 (separation of UTC timestamp and local Asia/Dhaka business date): verified by timestamp query and filter tests.
