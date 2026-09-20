# Verify: Audit log and cryptographic hash chain (spec 0005, updated 2026-09-20)

Steps derived from spec 0005 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [ ] Log in as Owner, perform a sensitive action (such as creating a new user or updating a role), and verify that a corresponding audit row appears in the Settings > Audit Log view with timestamp, actor, action badge, and target entity (verifies AC-1, AC-6, AC-7)
- [ ] Click on an audit row in the UI and inspect the Before/After diff modal, verifying that values are formatted clearly and that sensitive keys (passwords, tokens) are marked `[REDACTED]` (verifies AC-3, AC-7)
- [ ] In the Audit Log view, click "Verify Chain Integrity" and confirm that the cryptographic badge reports "Valid chain" with record count and head hash (verifies AC-5, AC-7)
- [ ] Attempt to access the Audit Log view or invoke `audit.list` using a user account lacking the `audit.view` permission (e.g. Teacher), and verify that access is rejected with a structured `forbidden` error (verifies AC-6)
- [ ] Filter audit records by entity type (`user`), action (`users.update`), and date range, verifying that results update dynamically without full page refresh (verifies AC-6, AC-7)

## Commands
- [ ] `cargo test -p coachify_core --test audit_log` (runs integration tests verifying hash chain continuity, genesis prev_hash 64 zeros, recursive field redaction, SQLite trigger aborts on UPDATE/DELETE, tampered payload detection, deleted row detection, transaction rollback consistency, and query pagination, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings and command maps match the Rust core contract for `audit.list`, `audit.verify_chain`, and `audit.get_head`, verifies AC-6)
- [ ] `cargo run -p xtask check-migrations` (verifies released migration immutability and successful execution of migration 0008, verifies AC-4, AC-6)
- [ ] `pnpm test` (runs UI tests for AuditLogView, filter controls, diff modal, and chain verification status badge, verifies AC-7)

## Acceptance criteria coverage
- AC-1 (in-transaction audit logging for mutation commands): verified by `audit_log` integration tests asserting audit rows commit with domain mutations and roll back on error.
- AC-2 (cryptographic SHA-256 hash chain with genesis 64 zeros): verified by hash calculation tests and chain progression assertions.
- AC-3 (recursive redaction of sensitive credentials and tokens): verified by payload sanitization tests asserting sensitive keys are replaced with `[REDACTED]`.
- AC-4 (immutability triggers preventing UPDATE/DELETE): verified by SQLite trigger abort tests on `trg_audit_log_no_update` and `trg_audit_log_no_delete`.
- AC-5 (diagnostics verification detecting tampered, broken, or deleted rows): verified by corruption injection tests validating that broken links and tampered payloads are detected with failing sequence numbers.
- AC-6 (typed RPC commands `audit.list`, `audit.verify_chain`, `audit.get_head` gated by `audit.view`): verified by command dispatcher permission tests and TypeScript contract check.
- AC-7 (React Audit Log UI view with timeline, diff modal, and verification badge): verified by Vitest component tests and manual inspection.
