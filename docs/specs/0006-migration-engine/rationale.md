# 0006. Migration engine with staging copy, atomic swap, and invariant validation: Decision Record

## Context

Coachify is designed as a standalone, offline first desktop application running on local Windows PCs in coaching centers and educational institutions across Bangladesh. In these environments, hardware conditions present unique reliability challenges: sudden power outages (load shedding) occur frequently without uninterrupted power supplies, machines often have limited RAM (4 GB) and spinning magnetic hard drives, and local Windows user accounts are frequently shared across non technical office staff.

When an application update introduces changes to the SQLite database schema, executing migrations directly on the active production database presents unacceptable risks. If a power cut occurs while a multi statement migration is altering tables, or if a migration script encounters an unexpected foreign key constraint violation or data typing error halfway through execution, the database can be left in a corrupted or half migrated state. In an offline desktop architecture, there is no remote cloud database administrator to manually recover or restore broken tables.

Furthermore, coaching centers depend entirely on the integrity of their student attendance, exam marks, and financial cash ledgers. A subtle bug in a migration script that accidentally alters poisha balances or severs guardian foreign keys could remain undetected for days, eroding owner trust and producing erroneous financial reports.

Coachify requires an automated migration mechanism that guarantees complete failure isolation. The migration engine must verify available disk space before touching files, execute all pending migrations on an isolated copy of the database, enforce rigorous invariant audits on the migrated copy, atomically swap the validated copy into place, and preserve a safety archive of the pre migration database. If anything goes wrong at any step, the production database must remain completely unmodified, and the user must be presented with clear diagnostic options.

## Options considered

### Option 1: Staging Copy with Atomic Swap, Pre Migration Archives, and Post Migration Validation Gates (Chosen)

Under this option, Coachify never applies schema migrations directly to the live production database. On startup, when pending migrations are detected, the engine verifies that local volume storage has at least 2.5 times the active database size. The write ahead log is checkpointed with `PRAGMA wal_checkpoint(TRUNCATE)`, active connections are closed, and the database file is duplicated into a temporary staging copy (`staging.db`). All pending migrations execute sequentially within individual transactions on the staging file.

Once all migration steps succeed, the staging copy undergoes comprehensive validation gates: SQLite integrity checks, foreign key checks, and financial ledger invariant audits (asserting that all poisha balances and transaction row counts match pre migration expectations). Only after all validation gates pass does Coachify execute an atomic file swap on Windows: the active database is renamed to a timestamped archive (`pre-migrate-v{version}-{timestamp}.db`), and the staging file is renamed to the active database path using a retry loop with exponential backoff against Windows Defender file locks. Safety archives older than thirty days are automatically pruned.

**Pros**:
- The live database file is never exposed to partial execution, syntax failures, or constraint violations.
- A sudden power cut or crash during migration leaves the original database untouched and runnable.
- Invariant validation gates detect silent data corruption or balance divergence before the database becomes active.
- The pre migration archive guarantees that an exact physical backup is retained on disk for disaster recovery.
- If a migration fails, the application cleanly routes the user to a dedicated recovery screen with sanitized diagnostics.

**Cons**:
- Requires at least 2.5 times the database file size in free disk space during the migration operation.
- Duplicating large database files on spinning magnetic hard drives introduces a small delay during version upgrades.

### Option 2: In Place Single Transaction with SQLite Savepoint Rollback

This option executes all pending migrations directly on the live database file within a single `BEGIN IMMEDIATE` transaction or nested savepoints. If any migration step fails, the engine issues a `ROLLBACK` command and aborts startup.

**Pros**:
- Extremely fast execution with minimal disk I/O because no physical database duplication occurs.
- Requires virtually zero additional free disk space, functioning on drives with low remaining capacity.

**Cons**:
- SQLite transactions cannot protect against catastrophic power cuts during complex DDL operations like table rebuilds, index recreation, or vacuum operations.
- Does not preserve an independent physical snapshot of the database prior to migration for support troubleshooting.
- Does not allow running comprehensive integrity checks on the post migration state before committing changes to the active file.

### Option 3: External Compressed Zip Archive with In Place Migration

This option creates a compressed zip or tar archive of the database folder in a backup subfolder before running migrations directly on the live database file.

**Pros**:
- Preserves a compressed historical backup before modifying production data.
- Uses less disk space than an uncompressed physical copy.

**Cons**:
- Compressing large database files on low specification dual core processor PCs creates significant CPU and memory pressure.
- Restoring from a compressed archive following a migration failure requires manual decompression and file reconstruction.
- Migrations still execute on the live database file, risking file corruption if power cuts occur mid write.

## Rationale

Coachify chooses Option 1 because data safety is the non negotiable foundation of the product. In coaching centers in Bangladesh, power cuts without battery backup are a daily reality. A desktop application that modifies production database files during schema updates will inevitably experience power failure during a migration at some pilot site.

Under Option 1, the live database file is never touched during the migration process. All write ahead log entries are cleanly flushed to the main file, connections are closed, and an isolated duplicate is created. If a power outage occurs while migrations are running, or while SQLite is rebuilding indexes, only the temporary staging copy is interrupted. Upon restarting the PC, Coachify detects the orphaned staging copy, discards it, and boots normally using the untouched live database.

The addition of post migration validation gates addresses the second major failure mode: subtle migration bugs that execute without SQL syntax errors but violate application domain invariants. By running foreign key checks and verifying that financial ledger poisha sums remain unchanged across migrations, Coachify guarantees that no corrupted or inconsistent data can become active.

The requirement for 2.5 times free disk space is realistic and manageable for institutional database sizes (typically under 100 MB), and the storage check prevents out of space disk crashes before any file operations begin. The automatic thirty day retention of pre migration archives provides an essential safety net for support personnel without creating unbounded disk growth.
