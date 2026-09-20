# Verify: Command dispatcher and local accounts (spec 0004, updated 2026-09-20)

Steps derived from spec 0004 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [x] Launch application and verify that when zero users exist, the system signals setup required and allows bootstrapping the first Owner account (verifies AC-1, AC-4)
- [x] Log in as a user, leave the application idle for fifteen minutes (or simulate idle timeout), and verify that the lock screen appears and requires password entry to resume (verifies AC-2)
- [x] Confirm active session stays alive during active data entry via periodic three minute heartbeat (verifies AC-2)
- [x] Verify absolute session expiration requires full credential login after twenty four hours (verifies AC-2)
- [x] Attempt to perform an unauthorized action (such as creating a user with a Teacher account) and verify that the UI shows a clear permission denied message (verifies AC-5)
- [x] Verify that navigating across screens after creating a user triggers instant list updates via Tauri change events without manual page reload (verifies AC-8)

## Commands
- [x] `cargo test -p coachify_core --test dispatcher_and_accounts` (runs unit and integration tests verifying session resolution, permission validation, idempotency cache replay, account lockout, failed attempt decay, absolute session cap, clock tampering rejection, and last Owner invariant, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7)
- [x] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings match the Rust core command contract, verifies AC-1)
- [x] `cargo run -p xtask check-migrations` (verifies released migration immutability and successful execution of migration 0007, verifies AC-3)
- [x] `pnpm test` (runs UI tests for AuthContext, heartbeat, and idle locking state transitions, verifies AC-2, AC-8)

## Acceptance criteria coverage
- AC-1 (single typed command dispatcher entry point): verified by `handle_rpc` integration tests and TypeScript contract check.
- AC-2 (Argon2id password hashing, five attempt lockout, failed attempt decay, fifteen minute idle locking, and twenty four hour absolute cap): verified by authentication test suite and UI idle lock tests.
- AC-3 (system roles Owner, Cashier, Teacher, custom roles, and protected Owner role): verified by role repository tests and SQLite trigger abort assertions.
- AC-4 (at least one active Owner invariant): verified by transaction abort tests when attempting to disable or demote the last remaining Owner.
- AC-5 (core side permission verification rejecting unauthorized requests): verified by permission check tests across multiple user roles.
- AC-6 (idempotency key tracking, deduplication, and 72 hour maintenance cleanup): verified by duplicate request tests confirming identical cached responses, payload mismatch errors, and cleanup pruning.
- AC-7 (license gate rejecting write commands when expired): verified by dispatcher license state checks with expired simulated licenses.
- AC-8 (change event emission over Tauri channel): verified by event broadcast assertions upon successful mutation commands.
