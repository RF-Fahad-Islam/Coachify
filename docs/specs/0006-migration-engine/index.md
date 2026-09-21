# 0006. Migration engine with staging copy, atomic swap, and invariant validation

**Date**: 2026-09-20
**Status**: Accepted

## Summary

Coachify protects local institution databases during schema upgrades by executing numbered forward migrations on an isolated staging copy before touching production data. The migration engine checks available storage space, checkpoints the write ahead log, duplicates the active database into a temporary staging file, and runs pending migrations within individual transactions. Before any file replacement occurs, the migrated staging copy must pass complete database integrity checks, foreign key checks, and financial ledger invariant audits. On successful validation, Coachify atomically swaps the staging file into place and archives the original database for thirty days. If any migration or validation fails, the active database remains completely untouched, and Coachify presents a clear recovery screen with sanitized diagnostic logs.

## Requirements

**User stories**:
- As an institute owner, I want software updates to never corrupt or destroy my existing student and financial records, so that my center operations never halt due to a failed database upgrade.
- As an institute owner, I want a historical archive of my database preserved before every migration, so that support technicians can inspect the pre migration state if unexpected issues emerge.
- As a software engineer, I want migrations to execute inside transactions on an isolated copy of the database, so that partial execution or power cuts leave the active database completely intact.
- As a software engineer, I want automated validation of SQLite foreign keys and financial ledger poisha balances before the migrated copy becomes active, so that silent data divergence is caught immediately.
- As a system administrator, I want a dedicated recovery screen with diagnostic logs if a migration fails, so that I can report the technical details to support without exposing confidential institute data.

**Acceptance criteria**:
- **AC-1**: On startup before application connections open, Coachify inspects the database schema version against the target version embedded in the compiled binary. If schema versions match, startup proceeds immediately. If the database schema version exceeds the compiled binary version, startup aborts with an explicit error explaining that the database was created by a newer version of Coachify.
- **AC-2**: Prior to creating staging files, Coachify checks available disk volume space using Windows `GetDiskFreeSpaceExW`. If free space is less than 2.5 times the database file size, the migration process halts before touching any files and returns a clear insufficient storage error.
- **AC-3**: When a forward migration is required, Coachify acquires an exclusive operating system level file lock on a lock file `migration.active.lock` using Windows `LockFileEx` (which automatically releases upon process termination or crash), checkpoints the active database WAL log with `PRAGMA wal_checkpoint(TRUNCATE)`, and creates an exact staging copy `staging.db`.
- **AC-4**: All pending migrations are applied in strict numeric order against `staging.db`. Each migration executes inside its own dedicated SQLite transaction. If any migration step fails, the transaction rolls back, `staging.db` is purged, and the live database file remains completely unmodified.
- **AC-5**: After applying all pending migrations, the staging copy must pass three validation gates: `PRAGMA integrity_check` returning ok, `PRAGMA foreign_key_check` returning zero rows, and financial invariant audits asserting that total poisha sums and row counts across all ledger tables (`payments`, `charges`, `allocations`, `adjustments`, `refunds`, and `cash_days`) match the baseline captured from `staging.db` before migration execution. If any validation fails, `staging.db` is abandoned and the live database remains untouched.
- **AC-6**: Upon successful validation, Coachify flushes staging with `PRAGMA wal_checkpoint(TRUNCATE)`, closes staging connections, removes any stale `-wal` and `-shm` sidecar files, and executes an atomic file swap sequence wrapped in a retry loop with exponential backoff (resilient to Windows Defender file scanning): the active database is renamed to `pre-migrate-v{version}-{timestamp}.db`, and `staging.db` is renamed to the active database file path. Pre migration archives older than 30 days are automatically pruned from the directory on startup.
- **AC-7**: If a power failure or crash occurs during the file rename sequence where active database is missing but `staging.db` and an archive file exist, startup detects the interrupted swap: if `staging.db` passes validation it renames it to active, otherwise it identifies the newest archive file matching `pre-migrate-v{version}-{timestamp}.db` and restores it to active.
- **AC-8**: If a migration fails at any stage or validation rejects the staging file, the application prevents normal workspace initialization and presents a dedicated Migration Recovery Screen displaying technical diagnostics, a button to export sanitized diagnostic logs, and a button to open the database directory in Windows Explorer.
- **AC-9**: The core exposes typed query commands `system.migration_status` and `system.migration_history` gated by `settings.view` permission, reporting current and target schema versions, pending migration lists, execution durations, and SHA-256 checksums.
- **AC-10**: The database schema includes a `_migrations` tracking table recording version, name, sha256, applied_at, and execution_time_ms, with immutability triggers `trg_migrations_no_update` and `trg_migrations_no_delete` that reject updates and deletions, while maintaining synchronization with legacy `_schema_version`.

## Decision

**Chosen option**: Option 1: Staging Copy with Atomic Swap, Pre Migration Archives, and Post Migration Validation Gates.

You build the Coachify migration engine in `crates/core/src/migrations.rs`, integrating pre flight storage checks, write ahead log checkpoints, isolated staging execution, rigorous validation gates, atomic file swaps on Windows, and a dedicated React recovery screen.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `react-patterns` (`.claude/skills/react-patterns/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

The `_migrations` table records every applied schema file with its cryptographic checksum and execution metadata. The legacy `_schema_version` table is maintained simultaneously for backward compatibility.

```sql
-- Migration tracking ledger with immutability triggers
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    applied_at TEXT NOT NULL, -- ISO 8601 UTC string (YYYY-MM-DDTHH:MM:SSZ)
    execution_time_ms INTEGER NOT NULL
);

-- Immutability triggers to guard historical records against tampering
CREATE TRIGGER IF NOT EXISTS trg_migrations_no_update
BEFORE UPDATE ON _migrations
BEGIN
    SELECT RAISE(ABORT, 'Historical migration records are immutable and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS trg_migrations_no_delete
BEFORE DELETE ON _migrations
BEGIN
    SELECT RAISE(ABORT, 'Historical migration records are immutable and cannot be deleted');
END;

-- Legacy table kept in sync for simple queries
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);
```

### Migration step runner definition

The Rust migration catalog defines migrations using a step enum to permit both declarative SQL batches and programmatic data transformation steps:

```rust
pub enum MigrationStep {
    Sql(&'static str),
    Data(fn(&rusqlite::Transaction) -> Result<(), MigrationError>),
}

pub struct MigrationDefinition {
    pub version: u32,
    pub name: &'static str,
    pub sha256: &'static str,
    pub step: MigrationStep,
}
```

### State transitions

The migration lifecycle flows through explicit sequential phases:

```
[Start Check]
      |
      v
(Check Version) ---> [No Pending] ---> [Boot Normally]
      |
      v (Newer Pending)
(Check Free Space >= 2.5x) ---> [Low Space] ---> [Abort with Insufficient Storage]
      |
      v (Space OK)
(Acquire OS Lock: migration.active.lock)
      |
      v
(Checkpoint WAL TRUNCATE & Copy to staging.db)
      |
      v
(Capture Staging Ledger Baseline Counts & Sums)
      |
      v
(Execute Pending Migrations on Staging) ---> [Error] ---> [Purge Staging & Show Recovery]
      |
      v (Execution Succeeded)
(Run Validation Gates: Integrity, FK, Ledger) ---> [Invalid] ---> [Purge Staging & Show Recovery]
      |
      v (Validation Passed)
(Flush Staging WAL TRUNCATE & Close Connections)
      |
      v
(Rename active to pre-migrate-v{old}-{ts}.db with Retry Backoff)
      |
      v
(Rename staging.db to active with Retry Backoff)
      |
      v
(Release OS Lock & Prune Archives > 30 Days)
      |
      v
[Boot Normally]
```

### API surface

The core library exposes typed query commands through the command dispatcher:

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| system.migration_status | Query | None | current_version, target_version, is_upgrade_needed, pending_migrations | settings.view | 403 Forbidden |
| system.migration_history | Query | None | history: list of version, name, sha256, applied_at, execution_time_ms | settings.view | 403 Forbidden |
| system.recovery_info | Query | None | is_in_recovery, failed_version, error_message, active_db_path, archive_paths | Public (Startup) | None |

### Value sourcing

Every value produced or displayed by migration actions traces directly to a verified source:

| Action | Value produced or displayed | Source |
|---|---|---|
| system.migration_status | current_version | Database query: `SELECT MAX(version) FROM _migrations` (defaults to 0 if empty) |
| system.migration_status | target_version | Rust compile time constant derived from `MIGRATIONS.last().version` |
| system.migration_status | is_upgrade_needed | Evaluated expression: `current_version < target_version` |
| system.migration_status | pending_migrations | Sliced list from compiled catalog where `version > current_version` |
| system.migration_history | history list | Database query: `SELECT version, name, sha256, applied_at, execution_time_ms FROM _migrations ORDER BY version ASC` |
| run_migrations_with_staging | free_space_bytes | Windows API call `GetDiskFreeSpaceExW` on database directory |
| run_migrations_with_staging | database_size_bytes | Filesystem metadata: `std::fs::metadata(active_db_path).len()` |
| run_migrations_with_staging | required_space_bytes | Calculated value: `(database_size_bytes * 25) / 10` |
| run_migrations_with_staging | archive_filename | Generated string: `pre-migrate-v{old_version}-{timestamp}.db` |
| run_migrations_with_staging | execution_time_ms | High resolution timer: `std::time::Instant::now().elapsed().as_millis()` |
| run_migrations_with_staging | applied_at | ISO 8601 UTC string formatted via `chrono::Utc::now().to_rfc3339()` |
| system.recovery_info | is_in_recovery | Application startup state flag set to true when migration engine fails |
| system.recovery_info | active_db_path | Configured path string from `resolve_dev_db_path()` |
| system.recovery_info | archive_paths | Directory scan matching `pre-migrate-v*.db` sorted by timestamp descending |
| system.recovery_info | error_message | Formatted string from captured `MigrationError` enumeration |
| system.recovery_info | failed_version | Version field from the failed `MigrationDefinition` entry |

### Key invariants

- **Zero modification on error**: The active database file is never touched or modified until the staging copy has completely passed all migration steps and validation gates.
- **Single writer lock**: Only one migration engine process may operate at a time, guaranteed by an operating system level file lock on `migration.active.lock`.
- **Sidecar cleanliness**: The active database WAL is truncated prior to copy, and staging WAL is truncated prior to swap, preventing orphaned sidecar files from corrupting new schemas.
- **Windows rename resilience**: File renames during swap use an exponential backoff retry loop to avoid sharing violations from Windows Defender or indexing services.
- **Pre migration copy preserved**: An exact physical copy of the active database before migration is always retained in the local database folder for at least 30 days.
- **Checksum consistency**: The SHA-256 hash of every migration script must match its registered hash in `released.json` and in the compiled binary catalog.
- **Strict monotonic order**: Migrations are applied in ascending numeric order with no skipped versions.
- **Financial conservation**: Migrations must not alter total poisha sums or row counts in ledger tables (`payments`, `charges`, `allocations`, `adjustments`, `refunds`, `cash_days`) unless a data migration explicitly records and verifies an adjustment.

### Security model

- **Startup privilege**: The migration engine runs during application boot under the native operating system process identity with local disk access, before user authentication occurs.
- **Diagnostics access**: Runtime inspection commands `system.migration_status` and `system.migration_history` require the authenticated session to possess the `settings.view` permission (granted to Owner and Admin roles).
- **Diagnostics sanitization**: When the Recovery Screen exports diagnostics or error details, it strictly redacts passwords, token secrets, encryption keys, and personal student identity fields, including only technical SQLite error codes, stack traces, schema versions, and disk metrics.

### Configuration required

No new external environment variables or cloud service credentials are required. The migration engine uses local filesystem paths resolved by `resolve_dev_db_path()` or production database path configuration.

### Critical test scenarios

- Happy path: Application startup detects pending migrations, copies active database to staging, executes migrations in transactions, passes validation gates, swaps files, and boots cleanly, verifying **AC-1**, **AC-3**, **AC-4**, **AC-5**, **AC-6**, **AC-10**.
- Insufficient disk space: Startup detects free disk space below 2.5 times database size, halts before creating any files, and returns an insufficient storage error, verifying **AC-2**.
- Migration SQL failure: A deliberate syntax or constraint error in a pending migration triggers transaction rollback, staging deletion, leaves the original database untouched, and surfaces the Recovery Screen, verifying **AC-4**, **AC-8**.
- Validation gate failure: A migration that creates an orphaned foreign key or modifies financial poisha balances fails validation, aborts file swap, leaves original database untouched, and shows recovery diagnostics, verifying **AC-5**, **AC-8**.
- Interrupted rename recovery: A simulated power failure between active archiving and staging rename is detected on the next startup, successfully completing the rename or restoring the archive, verifying **AC-7**.
- Downgrade prevention: Launching an application binary against a database with a higher schema version immediately aborts with an update required message, verifying **AC-1**.
- Archive cleanup: Startup checks and purges pre migration archive files whose file creation timestamp exceeds 30 days, verifying **AC-6**.
- Concurrent process exclusion: Attempting to launch a second instance while a migration is active fails cleanly due to lock acquisition failure, verifying **AC-3**.

## Build plan

Applying the Tracer Bullet approach, you prove the end to end copy and swap thread through every layer first, then thicken it with validation gates, diagnostics commands, recovery UI, and historical upgrade fixture tests:

1. Create migration definitions, step runner enum, and `_migrations` schema tracking with immutability triggers in `crates/core/src/migrations.rs`, satisfies **AC-4**, **AC-10**.
2. Implement pre flight storage check via Windows `GetDiskFreeSpaceExW` and operating system file locking via `LockFileEx` in `crates/core/src/migrations.rs`, satisfies **AC-2**, **AC-3**.
3. Implement WAL truncate checkpoint, staging duplication, isolated transactional execution, and atomic file swap with backoff retry and 30 day archive retention, satisfies **AC-1**, **AC-3**, **AC-4**, **AC-6**.
4. Implement staging validation gates: `PRAGMA integrity_check`, `PRAGMA foreign_key_check`, and financial poisha ledger invariant audits, satisfies **AC-5**.
5. Implement crash recovery handling for interrupted rename sequences and downgrade prevention checks, satisfies **AC-1**, **AC-7**.
6. Expose typed RPC query commands `system.migration_status` and `system.migration_history` in `crates/core/src/dispatcher.rs` and generate TypeScript contracts, satisfies **AC-9**.
7. Implement the React Migration Recovery Screen in `ui/src/components/system/migration-recovery-screen.tsx` with sanitized diagnostics export, satisfies **AC-8**.
8. Integrate startup migration execution into the Tauri native application lifecycle in `src-tauri/src/lib.rs`, satisfies **AC-1**, **AC-8**.
9. Build automated integration test suite in `crates/core/tests/migration_engine.rs` verifying copy and swap, rollback on failure, power cut recovery, and historical fixture upgrades, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, **AC-6**, **AC-7**.

## Consequences

**Positive**:
- Database upgrades can never leave the active database in a corrupted or half migrated state.
- Institute owners are protected against data loss caused by power cuts or crashes during software upgrades.
- Every migration creates an automatic frozen physical backup that remains available for 30 days.
- Invariant validation prevents silent ledger corruption or broken foreign keys from entering production.
- Administrators receive clear, sanitized diagnostic logs to share with support technicians.

**Negative / tradeoffs**:
- Executing migrations on a staging copy requires at least 2.5 times the database file size in free disk space.
- Duplicating large database files adds a few seconds of initial startup delay during version upgrades.

**Neutral**:
- Developers must write both SQL migration files and optional Rust data migration steps with care, keeping released migrations immutable.
- Startup logic must be tested thoroughly on Windows filesystems to prevent file handle locking conflicts during rename operations.

## Follow-up

- [ ] Add historical database fixtures for version 1.0 to `tests/fixtures/` as release versions are published, ensuring CI tests continuous upgrade paths.
- [ ] Connect the Migration Recovery Screen to the Windows Explorer shell launcher in Tauri for opening the data directory.
