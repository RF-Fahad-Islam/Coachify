# 0004. Command dispatcher and local accounts

**Date**: 2026-09-20
**Status**: Accepted

## Summary

Coachify routes all user interface requests and future local network calls through a single typed command dispatcher in the core library. The dispatcher resolves the calling session, verifies required permissions and license gates, executes the domain transaction, records idempotency keys, and broadcasts change events to the user interface. Local accounts store credentials protected by Argon2id password hashing, five failed attempt lockouts, and fifteen minute idle locking. System roles (Owner, Cashier, Teacher) seed by default with granular permissions, custom roles are supported, and a database constraint ensures at least one active Owner always exists.

## Requirements

**User stories**:
- As an institute owner, I want every administrative action protected by permissions and password authentication so that cashiers and teachers can only access their assigned duties.
- As a cashier, I want rapid unlock when stepping away from the desk so that an idle computer cannot be abused by unauthorized people while preserving my open workflow.
- As an administrator, I want to create custom staff roles with specific permissions so that our coaching center can delegate duties according to our actual organizational structure.
- As a software engineer, I want a single typed command dispatcher that handles authentication, authorization, idempotency, transactions, and event emission so that business logic stays pure and the architecture is ready for local network multi PC mode.

**Acceptance criteria**:
- **AC-1**: All user interface requests reach core domain logic exclusively through one typed `rpc` command dispatcher passing command name, JSON payload, optional session token, and optional idempotency key.
- **AC-2**: User credentials store passwords hashed with Argon2id using sixty four megabytes memory, three iterations, four parallel lanes, and sixteen byte random salts; five consecutive failed login attempts within twenty four hours lock the account for five minutes (300 seconds); active sessions lock after fifteen minutes of user inactivity, requiring password unlock to resume, with an absolute maximum lifespan of twenty four hours.
- **AC-3**: Built in system roles (Owner, Cashier, Teacher) are seeded with granular dot namespaced permissions; administrators can create custom roles composed from permission codes; Owner role is protected and cannot be modified or deleted.
- **AC-4**: A database constraint and transaction trigger enforce that at least one active user holding the Owner role must always exist; any attempt to disable, delete, or remove the Owner role from the last active Owner is rejected with an error.
- **AC-5**: Permission verification executes inside the core dispatcher; any command invoked without required permissions is rejected with a structured `forbidden` error containing code, message, and required permission.
- **AC-6**: When a mutation command provides an idempotency key (formatted as a UUIDv7 string), the core records the key and serialized response in the database within the same transaction; repeating the request with the identical key and payload returns the original cached response without re executing the command; a payload mismatch returns an `idempotency_key_mismatch` error; records older than seventy two hours are purged during startup maintenance.
- **AC-7**: When the institution license is expired past the fourteen day grace period, the dispatcher rejects all write, sms, and lan commands with a structured `license_expired_read_only` error while permitting read and query commands.
- **AC-8**: State modifying commands broadcast typed change events (topic and affected entity IDs) over the Tauri event channel so open UI views refresh automatically.

## Decision

**Chosen option**: Option 1: Centralized typed command dispatcher with database backed session tokens, Argon2id passwords, and granular role permissions.

You build the Coachify command dispatch layer and local accounts model in `crates/core`, handling session resolution, permission validation, license gating, idempotency tracking, and transaction wrapping in one place, while exposing clean named TypeScript contracts to the user interface.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `react-patterns` (`.claude/skills/react-patterns/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

Migration 0007 implements the complete local accounts and command tracking schema.

```sql
-- Local users directory
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY, -- UUIDv7
    username TEXT NOT NULL COLLATE NOCASE UNIQUE,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL, -- Argon2id PHC string
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    must_change_password INTEGER NOT NULL DEFAULT 0 CHECK (must_change_password IN (0, 1)),
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_attempt_at TEXT, -- ISO 8601 UTC timestamp
    locked_until TEXT, -- ISO 8601 UTC timestamp
    staff_profile_id TEXT REFERENCES staff(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_users_staff ON users(staff_profile_id);

-- Role definitions
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Granular permissions catalog
CREATE TABLE IF NOT EXISTS permissions (
    code TEXT PRIMARY KEY, -- e.g. 'payments.collect'
    module TEXT NOT NULL, -- e.g. 'money', 'students', 'academics', 'system'
    description TEXT NOT NULL
);

-- Mapping roles to permissions
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_code)
);

-- Mapping users to roles
CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_role ON user_roles(role_id);

-- Known devices (host PC today, paired LAN workstations in version 1.1)
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL,
    windows_name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'blocked', 'revoked')),
    is_host INTEGER NOT NULL DEFAULT 0 CHECK (is_host IN (0, 1)),
    paired_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    app_version TEXT NOT NULL
);

-- Ephemeral session records
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY, -- UUIDv7
    token_hash TEXT NOT NULL UNIQUE, -- SHA256 of session bearer token
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    absolute_expires_at TEXT NOT NULL, -- created_at + 24 hours
    revoked_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

-- Idempotency tracking table
CREATE TABLE IF NOT EXISTS command_idempotency (
    idempotency_key TEXT PRIMARY KEY, -- UUIDv7 string
    command_name TEXT NOT NULL,
    payload_hash TEXT NOT NULL, -- SHA256 of canonical payload JSON
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    response_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_command_idempotency_created ON command_idempotency(created_at);

-- Trigger: prevent deleting or modifying system roles
CREATE TRIGGER IF NOT EXISTS trg_roles_protect_system
BEFORE DELETE ON roles
WHEN OLD.is_system = 1
BEGIN
    SELECT RAISE(ABORT, 'System roles cannot be deleted');
END;

CREATE TRIGGER IF NOT EXISTS trg_roles_protect_owner_name
BEFORE UPDATE ON roles
WHEN OLD.name = 'Owner' AND NEW.name != 'Owner'
BEGIN
    SELECT RAISE(ABORT, 'The Owner system role cannot be renamed');
END;
```

### State transitions

#### User Account Lifecycle
- `active` -> `locked`: Triggered after 5 consecutive failed login attempts within 24 hours; `locked_until` set to `now + 300 seconds`.
- `locked` -> `active`: Clears automatically after 5 minutes on the next valid password attempt, or when reset by an Owner.
- `failed_attempt_decay`: If `last_failed_attempt_at` is older than 24 hours, `failed_login_attempts` resets to zero on the next login evaluation.
- `active` -> `disabled`: Manual administrative action; revokes all active sessions for this user. Blocked inside the write transaction if the user is the sole remaining active Owner.

#### Session Lifecycle
- `created`: Issued upon successful `auth.login` or `system.bootstrap_owner`. `expires_at` set to `now + 15 minutes`; `absolute_expires_at` set to `now + 24 hours`.
- `active` -> `idle_locked`: Occurs when `now > last_seen_at + 15 minutes` while `now <= absolute_expires_at`. The session remains in the database, but write commands reject with `session_idle_locked` until `auth.unlock` succeeds with the user's password.
- `active` | `idle_locked` -> `expired`: Occurs when `now > absolute_expires_at`. Session is terminated and requires fresh `auth.login` with full credentials.
- `clock_tampering_revocation`: If `now < created_at` or clock guard detects backwards time movement, session is immediately revoked.
- `active` | `idle_locked` -> `revoked`: Occurs on `auth.logout`, password change, user disable, or when a fresh login occurs for that user on the same device.

#### Command Execution Lifecycle
1. Request received: `{ command, payload, session_token, idempotency_key }`.
2. Idempotency check: If `idempotency_key` exists in `command_idempotency`:
   - If `payload_hash` matches: return stored `response_json` immediately.
   - If `payload_hash` differs: abort with `idempotency_key_mismatch`.
3. Bootstrap check: If `command` is `system.bootstrap_owner`, wrap in `BEGIN IMMEDIATE` transaction, verify no active Owner exists, then create initial Owner and seed system roles.
4. Session resolution: Lookup `session_token` in `sessions` table. Validate `expires_at`, `absolute_expires_at`, and `revoked_at`. If backwards clock drift is detected, revoke session. If idle window exceeded, allow only `auth.unlock` and `auth.logout`.
5. Permission check: Load effective user permissions from `user_roles` and `role_permissions`. Verify command required permission (or public command).
6. License check: Verify institution license state. If past grace period, reject `Write`, `Sms`, and `Lan` commands with `license_expired_read_only`.
7. Execute domain logic:
   - Read commands: execute using shared read connection with deferred transaction.
   - Mutation commands: execute on writer connection wrapped in `BEGIN IMMEDIATE` transaction.
   - For `users.update` or role changes, count remaining active Owners before commit; rollback if count is zero.
   - Write audit log entries if sensitive.
8. Store idempotency: If `idempotency_key` is provided on a mutation command, insert record into `command_idempotency` within the same transaction.
9. Commit transaction.
10. Broadcast change events: Emit Tauri event `{ topic, entity_ids }` so front end caches refresh.

### API surface

The core exposes typed commands mapped through `handle_rpc` and mirrored in TypeScript:

| Command | License Gate | Required Permission | Key Inputs | Key Outputs | Key Errors |
|---|---|---|---|---|---|
| `system.ping` | None | Public | `message?: string` | `status, database_status, roundtrip_timestamp, echo` | `database_error` |
| `system.get_status` | None | Public | none | `needs_setup: boolean, institution_name?: string, app_version: string` | none |
| `system.bootstrap_owner` | None | Public (only when 0 users exist) | `username, display_name, password` | `session_token, user, expires_at` | `setup_already_completed, validation_error` |
| `auth.login` | None | Public | `username, password, device_name?: string` | `session_token, user, roles, permissions, expires_at` | `invalid_credentials, account_locked, account_disabled` |
| `auth.logout` | None | Authenticated | none | `success: boolean` | `unauthenticated` |
| `auth.unlock` | None | Authenticated (idle ok) | `password` | `success: boolean, extended_expires_at: string` | `invalid_credentials, session_expired` |
| `auth.change_password` | None | Authenticated | `current_password, new_password` | `success: boolean` | `invalid_credentials, weak_password` |
| `auth.get_current_session` | None | Authenticated | none | `user, roles, permissions, expires_at, absolute_expires_at, device` | `unauthenticated, session_expired, session_idle_locked` |
| `auth.heartbeat` | None | Authenticated | none | `extended_expires_at: string` | `unauthenticated, session_expired, session_idle_locked` |
| `users.list` | None | `users.manage` | `status_filter?: string, limit?: u32, offset?: u32` | `users: UserSummary[], total_count: u32` | `forbidden, unauthenticated` |
| `users.create` | Write | `users.manage` | `username, display_name, password, role_ids: string[], staff_profile_id?: string` | `user: UserSummary` | `username_taken, role_not_found, forbidden` |
| `users.update` | Write | `users.manage` | `id, display_name, status, role_ids: string[], staff_profile_id?: string` | `user: UserSummary` | `last_owner_required, user_not_found, forbidden` |
| `roles.list` | None | `roles.manage` or `users.manage` | none | `roles: RoleWithPermissions[]` | `forbidden, unauthenticated` |
| `roles.create` | Write | `roles.manage` | `name, description?: string, permission_codes: string[]` | `role: RoleWithPermissions` | `role_name_taken, invalid_permission, forbidden` |
| `roles.update` | Write | `roles.manage` | `id, name, description?: string, permission_codes: string[]` | `role: RoleWithPermissions` | `system_role_protected, role_not_found, forbidden` |
| `permissions.list` | None | Authenticated | none | `permissions: PermissionItem[]` | `unauthenticated` |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| Initialize database | Seed permissions | Hardcoded static permission catalog in migration 0007 |
| Initialize database | Seed system roles | Seeded in migration 0007 ('Owner', 'Cashier', 'Teacher') |
| Bootstrap first user | Owner role assignment | Assigned ID of system role 'Owner' from `roles` table |
| Hash password | Password salt and hash | 16 byte random salt via `rand::rngs::OsRng`; Argon2id (m=65536, t=3, p=4) |
| Login user | `password_hash` verification | Verified against stored Argon2id string in `users` table |
| Login user | `session_token` | Secure random 256 bit string generated in Rust core |
| Login user | `token_hash` | SHA256 hash of raw `session_token` stored in `sessions` table |
| Login user | `device_id` | Resolved from `devices` table or registered as local host |
| Dispatch command | Session user context | Loaded from `sessions` joined with `users`, `user_roles`, and `role_permissions` |
| Dispatch command | `business_date` | Computed from Asia/Dhaka civil calendar date via `jiff` |
| Dispatch command | Idempotency result | Cached `response_json` from `command_idempotency` table |
| Inactivity refresh | Session extension | Updated via UI heartbeat every 3 minutes, capped at `absolute_expires_at` |
| Maintenance job | Cleaned idempotency rows | Rows where `created_at < now - 72 hours` purged on startup |
| Broadcast event | Event payload | Emitted to Tauri window with channel `coachify://change` |

### Key invariants

1. **Single entry point**: All UI and future LAN commands pass through `handle_rpc` and the core command dispatcher. The UI never runs raw SQL or bypasses permission checks.
2. **Core permission check**: Permissions are verified in Rust core before any domain handler runs. A client hiding a button does not constitute authorization.
3. **Last active Owner invariant**: At all times, at least one active user with the Owner role must exist in the database. Inside the write transaction, modifying or disabling an Owner checks that remaining active Owners count is at least one; otherwise rolls back with `last_owner_required`.
4. **Idempotency guarantee**: Mutation commands with a UUIDv7 `idempotency_key` store output atomically. Retrying with identical parameters returns cached results; altered payloads return `idempotency_key_mismatch`. Old records are purged after 72 hours.
5. **License gating in core**: Expired licenses past the fourteen day grace period reject Write, Sms, and Lan operations in the dispatcher before handler execution.
6. **Idle lock and absolute cap**: Sessions inactive past fifteen minutes reject write commands with `session_idle_locked` until unlocked via `auth.unlock`. Sessions reach hard expiry after twenty four hours from creation regardless of unlock attempts.
7. **Lockout and attempt decay**: Five consecutive failed logins within 24 hours lock an account for 300 seconds. If the last failed attempt occurred over 24 hours ago, the counter resets to zero.
8. **Transaction isolation**: Read queries use shared read connections with deferred transactions; mutations use the single writer connection with `BEGIN IMMEDIATE`.

### Security model

- Passwords are never stored in plain text. Hashing uses Argon2id with m=65536, t=3, p=4, and 16 byte cryptographically secure random salts.
- Session bearer tokens are 256 bit random values. The database stores only the SHA256 `token_hash`, preventing session hijacking if a raw database backup is read.
- The UI layer receives session tokens in memory / sessionStorage only and passes them in the `rpc` request. Local storage does not store plain user passwords. Sessions do not persist across application restarts.
- Audit log rows are written inside the write transaction for all user creation, role updates, and password changes.

### Configuration required

No new external environment variables or third party API keys are required. Password hashing parameters (m=65536, t=3, p=4), lockout limits, and session timeouts are configured in core constants with idle lock timeout customizable in the `settings` table.

### Critical test scenarios

- Happy path: User logs in with valid credentials, receives session token, and executes an authorized command, verifies **AC-1**, **AC-2**, **AC-5**.
- Password verification and lockout: Five failed logins with invalid passwords lock the account for five minutes, rejecting subsequent login attempts, verifies **AC-2**.
- Failed attempt decay: Failed login attempt counter resets to zero if the previous failure occurred more than 24 hours ago, verifies **AC-2**.
- Idle lock and unlock: A session inactive past fifteen minutes rejects mutation commands with `session_idle_locked`, and resumes normal access following `auth.unlock`, verifies **AC-2**.
- Absolute session expiry: A session older than 24 hours rejects heartbeats and unlock attempts, requiring full `auth.login`, verifies **AC-2**.
- Clock tampering rejection: If system time is rolled back earlier than session `created_at`, the session is revoked immediately, verifies **AC-2**.
- Permission rejection: A user with only the 'Teacher' role attempting to call `users.create` or `payments.collect` is rejected with `forbidden`, verifies **AC-5**.
- Idempotency replay: Repeating a command with identical UUIDv7 idempotency key returns the original response without executing the handler twice, verifies **AC-6**.
- Idempotency mismatch: Sending an identical idempotency key with different payload parameters returns `idempotency_key_mismatch`, verifies **AC-6**.
- Idempotency cleanup: Maintenance routine purges idempotency records older than 72 hours, verifies **AC-6**.
- Last active Owner protection: Concurrent or single attempts to disable the only active Owner or remove the Owner role are rejected inside the transaction with `last_owner_required`, verifies **AC-4**.
- License expiration enforcement: When the institution license is marked expired past grace, write commands return `license_expired_read_only`, verifies **AC-7**.
- System role protection: Attempting to delete the Owner system role is aborted by SQLite triggers, verifies **AC-3**.
- Change event emission: Executing `users.create` emits an event with topic `users:changed` and the new user ID, verifies **AC-8**.

## Build plan

1. Add `argon2` and `rand` dependencies to `crates/core/Cargo.toml`, satisfies **AC-2**.
2. Create migration 0007 in `migrations/0007_accounts_and_dispatcher.sql` creating `users`, `roles`, `permissions`, `role_permissions`, `user_roles`, `devices`, `sessions`, and `command_idempotency` tables along with system role triggers, satisfies **AC-1**, **AC-3**, **AC-7**.
3. Implement password hashing with Argon2id, verification, and secure token generation in `crates/core/src/auth.rs`, satisfies **AC-2**.
4. Implement repository queries for users, roles, permissions, sessions, and idempotency in `crates/core/src/accounts.rs`, satisfies **AC-1**, **AC-3**, **AC-4**, **AC-7**.
5. Build the core command dispatcher pipeline with session resolution, permission validation, license gate check, transaction separation (deferred read versus immediate write), and idempotency cache in `crates/core/src/dispatcher.rs`, satisfies **AC-1**, **AC-4**, **AC-5**, **AC-6**.
6. Expose auth, user, and role command handlers and update `handle_rpc` in `crates/core/src/lib.rs`, satisfies **AC-1**, **AC-4**, **AC-5**.
7. Update TypeScript RPC client contracts and type generation in `crates/core/src/lib.rs` and `xtask`, satisfies **AC-1**.
8. Implement React Auth Context, session management, activity heartbeat, and idle lock screen in `ui/src/contexts/AuthContext.tsx` and `ui/src/components/auth/`, satisfies **AC-2**, **AC-8**.
9. Write unit and integration tests verifying dispatcher pipeline, idempotency cache, permission rejections, failed attempt decay, and Owner protection triggers in `crates/core/tests/dispatcher_and_accounts.rs`, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, **AC-6**, **AC-7**.

## Consequences

**Positive**:
- A single typed command dispatcher guarantees that every mutation is checked for permissions, license state, and idempotency before reaching domain code.
- Read commands execute concurrently without being serialized by immediate write locks.
- Defense in depth database triggers and transaction checks prevent an institution from ever being left without an active Owner.
- Argon2id hashing and hashed session tokens protect administrative and financial security even if PC physical backups are copied.
- Hard 24 hour session limits and backwards clock drift protection safeguard against session hijacking and time tampering.
- The single dispatcher architecture forms the exact seam needed for local network multi PC mode in version 1.1 without refactoring domain logic.

**Negative / tradeoffs**:
- Every new domain feature must declare its required permissions and license gate in the dispatcher command table.
- Argon2id password verification requires dedicated CPU cycles during login and unlock operations on older computers.
- Long running operations require client side heartbeats every 3 minutes to keep active sessions alive.

**Neutral**:
- The user interface must maintain an active session token in memory and handle `session_idle_locked` responses gracefully with an unlock modal.
- Seeded permissions are immutable in code constants and cataloged in migration 0007.

## Follow-up

- [ ] Connect the desktop setup wizard in Feature 8 to the `system.bootstrap_owner` command for first run center initialization.
- [ ] Connect the audit logging hash chain in Feature 6 to the dispatcher post execution hook.
- [ ] Wire the offline license verification engine in Feature 32 into the dispatcher license gate check.
