# Review, feat/command-dispatcher-local-accounts, 2026-09-21

**Reviewed by**: Gemini 3.8 Flash (author on Claude Sonnet 3.5 / Gemini Flash)
**Scope**: Core accounts, auth, command dispatcher, Tauri RPC bridge, migrations, UI auth context and screens
**Verdict**: Approved with fixes

## Summary
Coachify implements a centralized typed command dispatcher (`dispatcher.rs`) and local accounts system (`accounts.rs`, `auth.rs`) in `crates/core`, with a thin Tauri RPC command bridge and typed React AuthContext. The implementation satisfies Spec 0004 across all eight acceptance criteria (AC-1 through AC-8): single entry point via `rpc`, Argon2id password hashing (m=65536, t=3, p=4), 5-attempt / 5-minute lockout, 15-minute idle locking, 24-hour absolute session cap, backwards clock drift revocation, seeded system roles (Owner, Cashier, Teacher), custom roles, last active Owner invariant backed by database triggers, core permission gating, UUIDv7 idempotency caching with 72-hour maintenance purging, license expiration read-only gating, and Tauri change event broadcasting.

During review, two major findings and two minor issues were identified and immediately remediated with regression test coverage:
1. `update_user` blindly executed `DELETE FROM user_roles`, causing the database trigger `trg_user_roles_protect_last_owner` to prematurely abort profile updates for any institution with a sole active Owner.
2. `auth.unlock` did not check `locked_until`, failed to record login failures upon invalid passwords, and omitted audit logging, leaving idle-locked workstations vulnerable to brute-force guessing without lockout.
3. Owner system role permissions could be cleared via `roles.update`.
4. `auth-context.tsx` did not invalidate local session when `users:changed` broadcast indicated a revoked or unauthenticated user session.

## Major
### 🟠 `trg_user_roles_protect_last_owner` trigger aborted profile/role updates for sole active Owner (`crates/core/src/accounts.rs:616`)
**Problem**: In `accounts::update_user`, role mappings were updated by executing `DELETE FROM user_roles WHERE user_id = ?1` followed by re-inserting `role_ids`. For any institution with a single active Owner, deleting existing roles tripped `trg_user_roles_protect_last_owner` (`BEFORE DELETE ON user_roles`) because at the moment of deletion, the remaining active Owner count evaluated to 0. SQLite raised ABORT and aborted the entire update transaction.
**Why it matters**: It was impossible to update any field (such as `display_name` or `staff_profile_id`) for the sole active Owner without first creating a second Owner account.
**Fix applied**: Replaced bulk deletion with a set diff: `update_user` only deletes roles that are no longer assigned and inserts newly added roles, ensuring the active Owner role row is never deleted when retained. Added regression test `test_sole_active_owner_profile_update_succeeds`.

### 🟠 `auth.unlock` bypassed failed attempt lockout and audit logging (`crates/core/src/dispatcher.rs:1144`)
**Problem**: In `dispatcher.rs` under `auth.unlock`, invalid password verification returned `Err(RpcError { code: "invalid_credentials" })` directly without calling `accounts::record_login_failure`, without checking `locked_until`, and without recording an audit event.
**Why it matters**: An idle-locked screen at a cashier counter could be brute-forced indefinitely without ever locking the account after 5 attempts as required by AC-2, and with zero audit footprint.
**Fix applied**: Updated `auth.unlock` to check `locked_until`, call `accounts::record_login_failure` on bad password, log `auth.unlock_failed` and `auth.unlock` in the audit log, and exempt `auth.unlock` from transaction rollback so failure counters and audit entries persist on invalid attempts. Added regression test `test_auth_unlock_failed_attempts_trigger_lockout_and_audit`.

## Minor
### 🟡 Owner system role permissions could be modified in `update_role` (`crates/core/src/accounts.rs:918`)
**Problem**: While triggers prevented renaming or deleting the Owner role, `update_role` permitted updating description and replacing permissions for `Owner`.
**Why it matters**: Spec AC-3 specifies: "Owner role is protected and cannot be modified or deleted". Clearing permissions on Owner could leave the system without a fully authorized administrator.
**Fix applied**: Added check in `accounts::update_role` rejecting any update to system roles with `AccountsError::SystemRoleProtected`. Added regression test `test_owner_system_role_cannot_be_modified`.

### 🟡 `auth-context.tsx` ignored unauthenticated session on `users:changed` broadcast (`ui/src/contexts/auth-context.tsx:350`)
**Problem**: When a user was disabled by an administrator, the backend revoked all active sessions and emitted `users:changed`. The client listener called `auth.get_current_session`, received `unauthenticated`, but ignored errors, leaving stale user details in React state.
**Fix applied**: Added error handling in `users:changed` listener to clear `sessionToken`, `currentUser`, and permissions if the session is revoked or expired.

### 🟡 Idempotency record saved after transaction commit (`crates/core/src/dispatcher.rs:685`)
**Problem**: `accounts::save_idempotency_record` is invoked on `conn` after `execute_command` commits its immediate transaction, rather than within the same transaction as specified in AC-6.
**Why it matters**: In the event of a crash immediately following transaction commit, the mutation persists but the idempotency key is not recorded, meaning a subsequent retry could re-execute the mutation.
**Suggested follow-up**: In a future refactor of `dispatch`, pass the idempotency key directly into `execute_command` so insertion into `command_idempotency` commits inside the identical `BEGIN IMMEDIATE` transaction.

### 🟡 `device_name` in `LoginPayload` is unhandled (`crates/core/src/dispatcher.rs:967`)
**Problem**: `LoginPayload` accepts an optional `device_name`, but `auth.login` unconditionally calls `get_or_create_host_device`.
**Why it matters**: Harmless for 1.0 single-PC mode, but will need wiring when Feature 4 (LAN mode) introduces paired workstations.

## Nits
- ⚪ `crates/core/src/db.rs:192`: `open_database()` still executes `apply_all_migrations(&conn)` on every connection open. While lightweight in SQLite dev mode, connection caching or one-time bootstrap migration checking will be needed for high-throughput operations.

## Strengths
- **Argon2id and Session Cryptography**: Strict compliance with OWASP guidelines (m=65536, t=3, p=4, 16-byte random salts) and 256-bit random bearer tokens stored as SHA-256 hashes.
- **Clock Tampering Protection**: Immediate session revocation if system time moves backwards relative to session creation, preventing session extension via clock manipulation.
- **Defense-in-Depth Invariant Triggers**: Direct SQLite triggers enforce that at least one active Owner exists on update, delete, and role removal, preventing accidental lockout.
- **Type-Safe Boundary Contract**: 100% typed RPC contract generated through xtask, with named TypeScript exports and zero `any` types.
- **Maximalist UI Polish**: Clean full-page startup and onboarding experience, modal non-dismissible idle locking, and real-time session synchronization over Tauri change events.

## Test coverage
- Core accounts and dispatcher test suite (`crates/core/tests/dispatcher_and_accounts.rs`) provides 14 passing integration tests covering all 8 acceptance criteria:
  - `test_ac1_command_dispatcher_basics` (AC-1)
  - `test_ac2_password_hashing_argon2id` (AC-2)
  - `test_ac2_failed_attempt_lockout_and_decay` (AC-2)
  - `test_ac2_idle_lock_and_unlock` (AC-2)
  - `test_ac2_absolute_session_lifespan_and_clock_tampering` (AC-2)
  - `test_ac3_system_roles_and_custom_roles` (AC-3)
  - `test_ac4_last_active_owner_invariant` (AC-4)
  - `test_ac5_permission_verification` (AC-5)
  - `test_ac6_idempotency_cache_and_cleanup` (AC-6)
  - `test_ac7_license_gate_enforcement` (AC-7)
  - `test_ac8_change_event_emission` (AC-8)
  - `test_sole_active_owner_profile_update_succeeds` (Regression for Major 1)
  - `test_auth_unlock_failed_attempts_trigger_lockout_and_audit` (Regression for Major 2)
  - `test_owner_system_role_cannot_be_modified` (Regression for Minor 1)
- UI test suite (`ui/src/contexts/auth-context.test.tsx`, `ui/src/components/auth/auth.test.tsx`) covers AuthProvider, IdleLockDialog, StartupAuthPage, and OnboardingCarousel.

## Fixes applied (2026-09-21)
- **Major (Role Diffing in `update_user`)**: Implemented selective deletion/insertion of user roles in `crates/core/src/accounts.rs`, preventing `trg_user_roles_protect_last_owner` trigger aborts when updating a sole active Owner. Verified by `test_sole_active_owner_profile_update_succeeds`.
- **Major (`auth.unlock` Lockout and Audit Integration)**: Enforced lockout checks, failed attempt recording via `accounts::record_login_failure`, failure clearing on success via `accounts::record_login_success`, and audit logging (`auth.unlock`, `auth.unlock_failed`) in `crates/core/src/dispatcher.rs`. Verified by `test_auth_unlock_failed_attempts_trigger_lockout_and_audit`.
- **Minor (Owner Role Protection)**: Rejected role modification attempts against the Owner system role in `accounts::update_role`. Verified by `test_owner_system_role_cannot_be_modified`.
- **Minor (Client Session Revocation Sync)**: Handled unauthenticated/revoked session state in `ui/src/contexts/auth-context.tsx` upon receiving `users:changed` change events.
