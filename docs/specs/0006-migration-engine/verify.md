# Verify: Migration engine with staging copy, atomic swap, and invariant validation (spec 0006, updated 2026-09-20)

Steps derived from spec 0006 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [ ] Trigger an intentional migration failure (for example by providing an invalid migration script) and confirm that the Tauri application halts boot and displays the Migration Recovery Screen with error diagnostics and failed version details (verifies AC-4, AC-8)
- [ ] On the Migration Recovery Screen, click "Export Diagnostics" and verify that a sanitized diagnostic JSON bundle is saved to disk with technical error details and without sensitive credentials or student records (verifies AC-8)
- [ ] On the Migration Recovery Screen, click "Open Database Folder" and verify that Windows Explorer opens to the local application database directory showing the intact original database file (verifies AC-8)
- [ ] Log in as Owner or Admin, open system diagnostics, and verify that migration status displays current schema version, target version, and zero pending migrations (verifies AC-9)
- [ ] Attempt to call `system.migration_status` or `system.migration_history` as an unprivileged role (such as Teacher) and verify that the request is rejected with a structured forbidden error (verifies AC-9)

## Commands
- [ ] `cargo test -p coachify_core --test migration_engine` (runs integration tests verifying forward migration execution, WAL checkpoint, isolated staging copy, atomic file swap, 30 day archive retention, integrity and foreign key validation gates, financial poisha ledger invariant audits, downgrade prevention, process lock exclusion, and transaction rollback on failure, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-10)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings match the Rust core contract for `system.migration_status` and `system.migration_history`, verifies AC-9)
- [ ] `cargo run -p xtask check-migrations` (verifies that released migrations in `released.json` match file hashes and that migration 0009 adds the `_migrations` tracking table with immutability triggers, verifies AC-10)
- [ ] `pnpm test` (runs UI unit tests for the Migration Recovery Screen, diagnostics export button, and explorer trigger, verifies AC-8)

## Acceptance criteria coverage
- AC-1 (startup schema version inspection and downgrade rejection): verified by integration tests asserting clean startup when versions match and explicit failure when database version exceeds binary version.
- AC-2 (pre flight disk storage verification): verified by tests asserting that low available disk space aborts migration before creating staging files.
- AC-3 (exclusive process locking and staging copy creation): verified by tests asserting single writer lock acquisition and byte exact staging file duplication.
- AC-4 (isolated transactional execution on staging with rollback): verified by tests demonstrating that failing migrations leave the production database completely unmodified.
- AC-5 (post migration validation gates): verified by tests asserting that integrity failures, foreign key violations, or altered poisha ledger balances reject file swap and purge staging.
- AC-6 (atomic file swap and 30 day pre migration archive retention): verified by filesystem tests confirming active database renaming to archive and staging promotion to active.
- AC-7 (crash recovery for interrupted file rename sequence): verified by crash injection tests recovering orphaned staging files or restoring archives.
- AC-8 (Tauri native recovery screen with sanitized diagnostics): verified by UI component tests and manual inspection of exported error logs.
- AC-9 (typed RPC query commands gated by `settings.view`): verified by dispatcher permission tests and TypeScript contract checks.
- AC-10 (`_migrations` tracking table with immutability triggers): verified by database schema tests asserting that UPDATE and DELETE operations on `_migrations` are aborted.
