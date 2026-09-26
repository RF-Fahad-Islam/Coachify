# 0032. Operational Integrity (Slice 8 Umbrella)

**Date**: 2026-09-23
**Status**: Accepted

## Summary

This umbrella specification establishes the operational integrity foundation for Coachify, guaranteeing financial safety, communication durability, offline resilience, and data protection across coaching center PCs. The architecture enforces receipt authenticity through offline cryptographic signatures, stops duplicate payments before recording, ensures zero message loss during internet blackouts, provides a continuous status cockpit, monitors disaster readiness through a 100 point recovery score, and enables seamless one click computer migrations with Windows DPAPI key rewrapping. Every mechanism runs completely offline with immediate SQLite transactions and tamper evident audit trails.

## Structure

This umbrella decision coordinates eight focused child specifications for Slice 8:

1. [0032-receipt-verification.md](0032-receipt-verification.md): Offline receipt authentication via compact HMAC signatures and verification scanning, supporting Feature 40.
2. [0032-duplicate-payment-protection.md](0032-duplicate-payment-protection.md): Real time duplicate reference and daily fee collision prevention with cashier reason audit, supporting Feature 41.
3. [0032-guardian-contact-health.md](0032-guardian-contact-health.md): Contact hygiene analyzer detecting missing, malformed, or shared phone numbers with counter bulk editing, supporting Feature 42.
4. [0032-durable-outbox-and-offline-queue.md](0032-durable-outbox-and-offline-queue.md): Resilient local message queuing, restart survival, and automatic background reconnect flushing, supporting Features 43 and 44.
5. [0032-offline-resilience-center.md](0032-offline-resilience-center.md): Persistent operational health widget monitoring internet, outbox depth, backup recency, and storage within 5 seconds, supporting Feature 45.
6. [0032-disaster-recovery-and-drills.md](0032-disaster-recovery-and-drills.md): 100 point recovery health scoring and weekly headless restore drill validation, supporting Features 46 and 47.
7. [0032-one-click-pc-migration.md](0032-one-click-pc-migration.md): Portable encrypted installation bundle export and DPAPI secret rewrapping for machine transfers, supporting Feature 48.
8. [0032-database-integrity-checks.md](0032-database-integrity-checks.md): Sub 3 second startup and on demand SQLite PRAGMA, foreign key, financial balance, and audit hash chain checks, supporting Feature 49.

## Requirements

**User stories**:
* As an institution owner, I want every printed receipt to carry a tamper evident verification code, so that parents and receptionists can independently verify genuine payments even without an internet connection.
* As a counter cashier, I want the system to warn me if an identical payment reference or student fee was entered today, so that double entries and accidental double billing are prevented before posting.
* As a communications manager, I want to identify invalid or duplicate guardian phone numbers, so that fee reminders and exam result broadcasts reach legitimate parents.
* As a receptionist in an area with unstable electricity and internet, I want all queued messages to wait safely in the database and automatically send once internet returns, so that no critical notices are lost.
* As an administrator, I want a single visible health bar showing backup age, network state, and outbox status, so that I can spot hardware or connection troubles before they disrupt operations.
* As an owner, I want weekly automatic background restore drills and a clear recovery score, so that I know our backups actually work when a hard drive fails.
* As an owner purchasing a new office computer, I want to migrate the complete Coachify environment with one click, so that accounting history, documents, and encryption keys transfer smoothly without manual reconfiguration.

**Acceptance criteria**:
* **AC-1**: Thermal and A4 printed receipts include a compact 12 character HMAC SHA256 verification code and QR code formatted as `RCP|<receipt_number>|<amount_poisha>|<business_date>|<student_code>|<full_hmac_hex>`.
* **AC-2**: Scanning or typing the verification code in RPC `integrity.verify_receipt` validates the cryptographic signature offline, returning current payment status (valid, voided, or refunded) and student identity.
* **AC-3**: RPC `integrity.check_duplicate_payment` evaluates incoming cashier submissions against posted payments within 24 hours for matching reference strings, or identical student and poisha amount on the same Asia/Dhaka business date.
* **AC-4**: A cashier can override a duplicate payment warning only by supplying a non trivial explanation string (minimum two words, rejecting repeated single characters), which is persisted on the payment record and logged as a security audit event.
* **AC-5**: RPC `integrity.get_guardian_contact_health` inspects the student and guardian directory, categorizing invalid 11 digit Bangladeshi phone patterns, missing phone fields, cross family duplicate numbers, and sibling links, sorted by student code ascending.
* **AC-6**: RPC `integrity.bulk_update_guardian_contacts` applies normalized 11 digit phone corrections to multiple guardian records in a single immediate database transaction with duplicate collision alerts.
* **AC-7**: Messages queued in the `outbox` table during internet outages survive application restarts and background crashes without data loss, retaining their exact payload and destination.
* **AC-8**: Background outbox worker monitors network connectivity with exponential backoff (5s to 300s), automatically flushing pending messages upon verified internet restoration with unique deduplication keys.
* **AC-9**: App shell displays the Offline Resilience Center widget, updating network connectivity, pending outbox depth, database health, disk free space, and backup age within 5 seconds.
* **AC-10**: RPC `integrity.get_disaster_recovery_score` computes a composite 100 point score (40 points backup recency: 40 for under 24h, 20 for 24 to 48h, 10 for 48 to 72h; 25 points external destination presence via marker verification; 20 points drill verification success; 15 points database integrity), raising visual warnings below 70.
* **AC-11**: Headless backup verification drill restores the latest snapshot into an isolated disk sandbox database, executing integrity checks, foreign key checks, and ledger balance reconciliation without locking the production database.
* **AC-12**: RPC `integrity.export_migration_bundle` streams an AES 256 GCM encrypted portable archive (.coachify-bundle) directly to disk using Argon2id key derivation from the owner recovery phrase (m=64MB, t=3, p=4), containing relative path document assets and a companion SHA256 checksum manifest.
* **AC-13**: RPC `integrity.import_migration_bundle` imports a migration bundle on a new machine, verifies SHA256 checksums, restores tables, and rewraps cryptographic credentials under the new computer Windows DPAPI context.
* **AC-14**: Startup database integrity check executes in under 3 seconds on reference hardware, running PRAGMA integrity_check, foreign key verification, ledger sum reconciliation, and cryptographic validation over the latest 100 audit log blocks (with full chain validation running during the weekly drill).

## Decision

**Chosen option**: Option 1: Unified Operational Integrity Engine with Modular Child Specifications

We establish a unified integrity module inside `crates/core/src/integrity.rs` backed by Migration 0017, exposing an `integrity.*` RPC namespace. The engine operates completely locally, enforcing cryptographic receipt codes, proactive cashier guards, resilient background outbox processing, automated restore drills, and portable DPAPI migration packages.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`)

## Rationale

Reasoning and architectural options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:
The integrity engine adds Migration 0017 with dedicated tables and column enhancements:

```sql
-- Receipt verification audit log
CREATE TABLE receipt_verifications (
    id TEXT PRIMARY KEY,
    receipt_number TEXT NOT NULL,
    verification_code TEXT NOT NULL,
    status_at_check TEXT NOT NULL CHECK (status_at_check IN (
        'valid', 'voided', 'refunded', 'not_found', 'signature_mismatch'
    )),
    verified_by_user_id TEXT REFERENCES users(id),
    scanned_payload TEXT NOT NULL,
    checked_at TEXT NOT NULL
);

-- Payments schema enhancements
ALTER TABLE payments ADD COLUMN verification_code TEXT;
ALTER TABLE payments ADD COLUMN duplicate_override_reason TEXT;

-- PC migration package registry
CREATE TABLE pc_migrations (
    id TEXT PRIMARY KEY,
    package_filename TEXT NOT NULL,
    target_machine_hint TEXT,
    exported_by_user_id TEXT NOT NULL REFERENCES users(id),
    exported_at TEXT NOT NULL,
    imported_at TEXT,
    checksum_sha256 TEXT NOT NULL,
    rewrap_salt TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('exported', 'verified', 'imported', 'failed'))
);

-- Database integrity run history
CREATE TABLE integrity_check_runs (
    id TEXT PRIMARY KEY,
    check_type TEXT NOT NULL CHECK (check_type IN ('startup', 'scheduled', 'manual')),
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    pragma_integrity_ok INTEGER NOT NULL CHECK (pragma_integrity_ok IN (0, 1)),
    foreign_keys_ok INTEGER NOT NULL CHECK (foreign_keys_ok IN (0, 1)),
    balance_reconciliation_ok INTEGER NOT NULL CHECK (balance_reconciliation_ok IN (0, 1)),
    audit_chain_ok INTEGER NOT NULL CHECK (audit_chain_ok IN (0, 1)),
    error_details_json TEXT
);

-- Disaster recovery score history
CREATE TABLE recovery_score_snapshots (
    id TEXT PRIMARY KEY,
    calculated_at TEXT NOT NULL,
    total_score INTEGER NOT NULL,
    backup_recency_score INTEGER NOT NULL,
    external_copy_score INTEGER NOT NULL,
    drill_success_score INTEGER NOT NULL,
    db_integrity_score INTEGER NOT NULL,
    warnings_json TEXT NOT NULL DEFAULT '[]'
);
```

**State transitions**:
Outbox state machine:
`queued` -> `sending` -> `sent`
If offline or network error: `sending` -> `queued` (reconnect backoff: 5s, 10s, 20s, up to 300s max)
If in flight exceeds 120s: `sending` -> `uncertain`
Manual retry: `failed` or `uncertain` -> `queued`

PC migration state machine:
`exported` -> `verified` -> `imported` (or `failed`)

Safety enforcement state machine:
`normal_read_write` -> on critical corruption detected -> `safety_read_only` (blocks all mutating commands with error 423)

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.verify_receipt` | Mutation | code_or_payload: String | ReceiptVerificationResult | Authenticated | 400 invalid format, 404 not found |
| `integrity.check_duplicate_payment` | Read | student_id: String, amount_poisha: i64, reference: Option<String> | DuplicateWarningResult | `payments.collect` | 400 invalid params |
| `integrity.get_guardian_contact_health` | Read | limit: Option<u32>, offset: Option<u32> | ContactHealthReport | `students.view` | 500 query failed |
| `integrity.bulk_update_guardian_contacts` | Mutation | updates: Vec<GuardianContactUpdate> | BulkUpdateResult | `students.manage` | 400 validation error |
| `integrity.get_outbox_lifecycle` | Read | status: Option<String>, channel: Option<String> | OutboxLifecycleSummary | Authenticated | 500 query failed |
| `integrity.retry_outbox_messages` | Mutation | message_ids: Vec<String> | RetryResult | Authenticated | 403 forbidden |
| `integrity.flush_offline_queue` | Mutation | none | FlushResult | Authenticated | 503 network unavailable |
| `integrity.get_resilience_status` | Read | none | ResilienceStatus | Authenticated | 500 internal error |
| `integrity.get_disaster_recovery_score` | Read | none | RecoveryScoreResult | Authenticated | 500 calculation error |
| `integrity.run_verification_drill` | Mutation | backup_id: Option<String> | DrillResult | Authenticated | 500 drill execution error |
| `integrity.export_migration_bundle` | Mutation | target_hint: Option<String> | MigrationBundleResult | `system.manage` or Owner | 500 export error |
| `integrity.import_migration_bundle` | Mutation | bundle_path: String, recovery_phrase: String | MigrationImportResult | Owner | 401 invalid key, 409 checksum error |
| `integrity.run_database_integrity_checks` | Mutation | full: bool | DbIntegrityResult | Authenticated | 500 integrity check failed |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Print receipt | 12 character verification code | HMAC SHA256 of receipt_number, amount, paid_at, student_code keyed by local secret |
| Print receipt | QR code payload | `RCP\|<receipt_number>\|<amount_poisha>\|<business_date>\|<student_code>\|<hmac_hex>` |
| Verify receipt | Verification authenticity and status | Parsed input (detects pipe delimiter vs 12 character code) and status read from payments table |
| Cashier warning | Duplicate payment alert | SQL lookup on payments matching reference within 24 hours, or student and amount on business date |
| Cashier warning | Business date calculation | Asia/Dhaka calendar date associated with active cash day |
| Contact health | Invalid phone count and sibling groups | Regex check on guardian phones and group by phone in student_guardians table, sorted by student code |
| Outbox message | Deduplication key | Formatted string `<channel>:<related_type>:<related_id>:<recipient_phone>` |
| Reconnect flush | Outbox delivery batch | Oldest pending rows in outbox table with status queued |
| Resilience bar | Network connectivity & latency | Socket probe to local DNS/gateway measuring round trip time |
| Resilience bar | Disk free space | Windows GetDiskFreeSpaceExW call on application storage drive |
| Resilience bar | Database indicators | Cached startup check status and latest integrity check run |
| Recovery score | Composite score 0 to 100 | Weighted formula over backup age (up to 40), external marker presence (25), drill result (20), DB integrity (15) |
| Migration export | Encrypted archive payload | Streaming ZIP compression with Argon2id derived key directly to destination file |
| Migration import | Restored database and new DPAPI keys | Decompressed archive decrypted with recovery key and rewrapped via CryptProtectData |

**Key invariants**:
* Printed receipt verification codes are deterministic and can be verified entirely offline with zero external network access.
* Duplicate payment overrides require a non empty reason containing at least two words and produce a cryptographic audit log row.
* Outbox messages never get deleted during offline restarts; only explicitly cancelled or successfully delivered messages leave the active queue.
* The backup restore verification drill extracts snapshots to an isolated temporary sandbox on disk and never touches production tables.
* PC migration archives stream directly to disk chunk by chunk, mapping document vault items to relative paths (`documents/<filename>`).
* Critical database integrity checks run on startup in under 3 seconds; any corruption detection puts the system into protective read only mode enforced by an atomic in memory flag in the core dispatcher.

**Security model**:
* All counter staff (Receptionist, Cashier, Teacher, Admin, Owner) can view the resilience status widget and verify printed receipts.
* Cashier, Admin, and Owner can execute resilience actions, run verification drills, retry outbox messages, and trigger integrity checks.
* PC migration export and import require Administrator or Owner role with recovery phrase confirmation.
* Verification signatures use the institution local secret stored securely in `system_config` and protected by Windows DPAPI.

**Critical test scenarios**:
* Offline receipt verification: Validates genuine receipt code offline, verifies that modifying the amount string fails signature verification, verifies AC-1 and AC-2.
* Duplicate payment warning and override: Cashier receives warning on identical reference within 24 hours, enters override reason, verifies reason is saved and audited, verifies AC-3 and AC-4.
* Contact health and bulk normalization: Detects malformed numbers, converts 01711 to normalized +8801711 format in bulk, verifies AC-5 and AC-6.
* Offline queue survival: Queue 5 messages while offline, restart app process, restore connectivity, verify all 5 messages deliver without duplicates, verifies AC-7 and AC-8.
* Resilience widget status: Induces simulated network disconnect, verifies widget flags offline within 5 seconds, verifies AC-9.
* Recovery score calculation: Evaluates test database with 2 hour old verified backup on external USB, verifies score reaches 95+, verifies AC-10.
* Restore verification drill: Executes headless drill on test backup, confirms isolated disk sandbox executes PRAGMA checks and reconciles ledger balance, verifies AC-11.
* PC migration round trip: Exports encrypted bundle, imports on simulated target machine, verifies database tables and DPAPI secret rewrap, verifies AC-12 and AC-13.
* Startup integrity check performance: Measures startup integrity check on 10,000 student database, verifies completion in under 3000 ms, verifies AC-14.

## Build plan

Following the Tracer Bullet build approach, Slice 8 progresses in five vertical end to end layers:

1. Create Migration 0017 adding `receipt_verifications`, `pc_migrations`, `integrity_check_runs`, `recovery_score_snapshots` tables, and `verification_code`, `duplicate_override_reason` columns to `payments`, satisfies **AC-1**, **AC-4**, **AC-10**, **AC-12**, **AC-14**.
2. Implement core receipt HMAC signing, receipt QR payload generator, and `integrity.verify_receipt` RPC command in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Implement `integrity.check_duplicate_payment` cashier validation rule, payment override logging, and cashier desk warning modal, satisfies **AC-3**, **AC-4**.
4. Implement `integrity.get_guardian_contact_health` scanner, sibling contact grouping, and `integrity.bulk_update_guardian_contacts` transaction, satisfies **AC-5**, **AC-6**.
5. Implement durable outbox retry pipeline, offline queue restart recovery, and background network watcher with reconnect flushing, satisfies **AC-7**, **AC-8**.
6. Build global Offline Resilience Center header component with 5 second reactive polling for network, outbox, backup, disk space, and database health, satisfies **AC-9**.
7. Implement 100 point disaster recovery score calculation and automated weekly headless restore verification drill, satisfies **AC-10**, **AC-11**.
8. Implement PC migration streaming archive packager, AES 256 GCM encryption, DPAPI credential rewrapping, and import restoration handler, satisfies **AC-12**, **AC-13**.
9. Implement startup and on demand database integrity check suite (PRAGMA, foreign keys, ledger reconciliation, recent 100 audit blocks) with sub 3 second performance and safety read only lock, satisfies **AC-14**.
10. Integrate comprehensive end to end integration test suites in `crates/core/tests/operational_integrity_integration.rs` covering all fourteen acceptance criteria, satisfies **AC-1** through **AC-14**.

## Consequences

**Positive**:
* Receipts can be authenticated anywhere without internet, shutting down voucher forgery.
* Cashiers are protected against accidental double billing and duplicate mobile money reference entries.
* SMS and email notices never get dropped during internet downtime or power interruptions.
* Institute owners gain real visibility into backup health and actual restore readiness.
* Moving to a new office PC becomes a guided 5 minute procedure instead of a risky manual migration.
* Database corruption is detected on startup before corrupt writes propagate.

**Negative / tradeoffs**:
* Startup integrity check adds approximately 500 to 1200 milliseconds to initial application launch time.
* Background restore drills consume CPU and temporary disk storage for 10 to 30 seconds during weekly execution.
* Receipts require extra printed area for the 12 character verification code and verification QR symbol.

**Neutral**:
* Migration 0017 introduces five new tables and updates the payments table.
* Cashiers must type an explanation when knowingly processing identical payment amounts.

## Follow-up

* [ ] Coordinate receipt layout updates in `ui/src/components/print/receipt-printable.tsx` to display verification code and QR code.
* [ ] Verify that WebView2 silent printing on thermal receipt printers renders the verification QR code with sufficient scanning contrast.
