# Rationale: Command dispatcher and local accounts

## Context

Coachify is a standalone, offline first desktop application for coaching centers and private educational institutes in Bangladesh. Institutes manage confidential student personal data, examination results, staff records, and cash fee collections. In many coaching centers, multiple staff members (owners, accountants, cashiers, teachers) share the same computer at different hours of the day, or step away from their desk during busy counter sessions.

Without a centralized command dispatch and access control architecture, domain authorization logic easily fragments across user interface screens and scattered Tauri commands. If permissions are checked only by hiding user interface buttons, an unauthorized user or modified client script could invoke sensitive operations like waiving fees, editing grades, or deleting records. Furthermore, sudden load shedding power cuts or network retries during cashier operations risk duplicate charge or payment creation unless mutations are strictly idempotent.

Finally, while version 1.0 operates on a single PC, version 1.1 introduces local network multi PC mode where satellite cashier workstations talk to the host machine. If each user interface action talks to a separate ad hoc Tauri command, building local network mode would require a complete rewrite of the communication layer. The application requires a single typed command dispatch seam that enforces session resolution, permission validation, license gating, idempotency, and transactional safety in the Rust core from day one.

## Options considered

### Option 1: Centralized typed command dispatcher with database backed session tokens, Argon2id passwords, and granular role permissions

All user interface and network requests travel through one typed envelope: command name, JSON payload, session token, and idempotency key. The core dispatcher validates the session from the database, checks required permissions and license state, wraps domain execution in a database transaction (immediate for mutations, deferred for reads), stores idempotency output, and broadcasts change events. Passwords use Argon2id with recommended parameters (m=65536, t=3, p=4), and sessions enforce a fifteen minute idle lock, a twenty four hour absolute cap, five attempt lockouts, and backward clock drift protection.

**Pros**:
- Forms the exact architectural seam needed for local network multi PC mode in version 1.1.
- Guarantees that every state mutation is authenticated, authorized, license gated, and deduplicated in the core.
- Protects administrative continuity by ensuring at least one active Owner account always exists.
- Inactive counters automatically lock without losing in progress data, allowing quick password unlock.
- Concurrency and transaction isolation separate read queries from immediate write locks.

**Cons**:
- Requires registering every command in a central dispatch table and managing session tokens in client memory.
- Argon2id password verification requires minor CPU time during login compared to older algorithms.

### Option 2: Direct Tauri IPC command mapping per domain area with in memory session cache

Each feature exposes its own individual Tauri commands (e.g. `create_student`, `collect_payment`), maintaining active user sessions purely in an in memory global map in Rust. Permissions are checked inside individual command handlers.

**Pros**:
- Slightly faster initial scaffolding with direct function calls.
- Avoids writing session records to SQLite on every login.

**Cons**:
- Lacks a unified seam for local network multi PC mode, requiring an overhaul for version 1.1.
- In memory sessions and idempotency caches disappear on application restart or unexpected crash.
- Easy to accidentally omit authorization checks or idempotency deduplication on new commands.

### Option 3: Coarse role permission model with flat bitmask flags and client enforced idle locking

The core supports only hardcoded roles (Owner and Staff) with fixed permissions represented as integer bitmasks. Idle locking is handled solely in the React user interface by displaying a lock screen over components.

**Pros**:
- Simpler data model with fewer database tables.
- No need for a granular permissions table or role mapping joins.

**Cons**:
- Cannot accommodate real world coaching center organizational structures requiring custom roles.
- Client only idle locking is insecure against client tampering or direct command calls.
- Bitmasks limit future extensibility as dozens of academic and financial permissions are introduced.

## Rationale

Option 1 is selected because it directly addresses the security, operational, and architectural requirements of Coachify.

Coaching centers in Bangladesh operate in shared environments where physical access to computers is common. Protecting credentials with Argon2id and enforcing automatic fifteen minute idle locking directly prevents unauthorized staff or curious students from altering grades or cash records. Storing session tokens in SQLite (with SHA256 hashed bearer tokens) ensures session state survives power cuts and provides a clean audit trail of active logins.

A single typed command dispatcher guarantees defense in depth. Authorization is never delegated to the user interface. Whether a request originates from the local React UI or later across a local network socket in version 1.1, the exact same pipeline evaluates session identity, granular permissions, license validity, and idempotency. The strict database and transaction invariant requiring at least one active Owner prevents institutes from accidentally locking themselves out of administrative control.
