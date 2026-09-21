# Rationale: Audit log and cryptographic hash chain

## Context

Coachify is an offline-first desktop management application designed for coaching centers and private educational institutions in Bangladesh. In these environments, owners delegate cash collection, student admissions, attendance logging, and exam grading to receptionists, accountants, and teachers. Because multiple employees often share the same office PC or work across local workstations during peak counter hours, owners have severe and well-founded concerns about internal fraud:
- Cashiers collecting tuition fees and then voiding or altering the payment amount later.
- Unauthorized staff granting unrecorded fee discounts or concessions.
- Manipulating student marks or altering exam merit lists.
- Curious or malicious users attempting to clear activity logs to hide mistakes or illicit actions.

Furthermore, Bangladesh suffers frequent electrical load shedding without uninterrupted power supplies (UPS). Any logging architecture that writes outside the database transaction risks divergence: a financial transaction might commit while its corresponding audit entry fails to write, or vice versa.

Finally, physical access to the host computer is typical. A technician or employee with access to external SQLite tools could attempt to alter rows in the database file directly. Coachify requires a robust, insert-only audit trail with cryptographic tamper detection (a hash chain), automatic field sanitization, database trigger defenses, and in-app verification diagnostics.

## Options considered

### Option 1: Cryptographic SHA-256 Hash Chain with In-Transaction Dispatcher Post-Execution Recording, Recursive Field Redaction, and On-Demand Diagnostics Verification

Each state-modifying mutation command in the centralized dispatcher produces one or more domain-level audit records within the same `BEGIN IMMEDIATE` database transaction as the business operation. Each record includes a SHA-256 cryptographic hash calculated over the previous record's hash (`prev_hash`) and canonical payload data (timestamp, business date, actor, device, action, entity, entity ID, and before/after JSON states). The genesis record uses 64 zeros as its previous hash. Sensitive fields (passwords, tokens, keys) are scrubbed before persistence. Existing SQLite triggers abort any `UPDATE` or `DELETE` on the `audit_log` table, and an in-app verification tool checks sequence continuity, hash validity, and link integrity from genesis to head.

**Pros**:
- Guarantees strict transactional atomicity: audit entries commit or roll back with the exact mutation they describe, preventing orphaned or missing audit records.
- Provides mathematical proof of log integrity: any row alteration, deletion, or insertion breaks the SHA-256 link chain and is immediately identified by sequence number.
- High-level domain actions (e.g. `auth.change_password` vs `users.update`) are explicitly captured with contextual actor and device identifiers.
- Recursive sanitization protects sensitive credentials from being stored in change payloads.
- The latest head hash can be included in backup manifests (`.coachifybackup`), allowing external detection of post-backup database modifications.

**Cons**:
- Computes one SHA-256 hash per audit entry (typically < 5 microseconds, negligible impact on transaction latency).
- Audit rows cannot be truncated, requiring permanent retention in the SQLite database.

### Option 2: SQLite Triggers Automatically Writing Audit Rows

Create `AFTER INSERT`, `AFTER UPDATE`, and `AFTER DELETE` triggers on every domain table in SQLite that serialize old and new columns into `audit_log`.

**Pros**:
- Automatically captures low-level SQL changes even if a command handler forgets to declare an audit entry.
- Runs entirely within the SQLite database engine.

**Cons**:
- Triggers lack context regarding the calling user session, device identity, or business command intention (e.g., distinguishing a password reset from a profile name update).
- SQLite lacks built-in cryptographic SHA-256 hashing functions without compiling and maintaining custom C extension modules.
- Dynamic recursive JSON redaction of sensitive fields is difficult and fragile using standard SQLite trigger SQL.
- Increases schema complexity and migration maintenance across dozens of domain tables.

### Option 3: External Log File with Append-Only File System Permissions

Write audit events to a dedicated flat JSON Lines (`.jsonl`) file on the Windows filesystem outside SQLite, using Windows access control lists (ACLs) to enforce append-only permissions.

**Pros**:
- Keeps the SQLite database file smaller.
- File can be read using standard external log ingestion tools.

**Cons**:
- Lacks transactional atomicity with SQLite: a power cut during the millisecond window between SQLite `COMMIT` and filesystem write causes state divergence.
- On Windows PCs where institute staff share an Administrator account, filesystem ACLs can be easily overridden or files deleted entirely.
- In-app querying, filtering, and diff inspection become slower and require custom file indexing.
- Backups and database snapshots become split across multiple files, increasing the likelihood of restore inconsistency.

## Rationale

Option 1 is selected because it delivers unbreakable transactional atomicity, cryptographic tamper detection, and domain-level semantic audit logging.

By executing inside the dispatcher's `BEGIN IMMEDIATE` transaction, Coachify guarantees that an audit row is written if and only if the underlying mutation succeeds. Even during sudden power cuts or system restarts, SQLite's write-ahead log (WAL) and `synchronous = FULL` setting protect both the business record and its audit row together.

The SHA-256 hash chain provides tamper detection that survives beyond database triggers. If an attacker opens the SQLite database file using an external editor that bypasses triggers, modifying or deleting any historical record invalidates all subsequent hashes. The in-app diagnostics engine immediately identifies the broken sequence and alerts the institute owner.

Furthermore, recursive sensitive field scrubbing ensures that password hashes, reset tokens, and session secrets are never persisted in plaintext within audit payloads, satisfying security compliance and preventing privilege escalation.
