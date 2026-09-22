# Spec 0020 — Restore & Corruption Recovery

> Feature 22 · Slice 3 (Safety & Delivery)

## Goal

Provide a deterministic, atomic, and safe restoration workflow for Coachify databases on the same or new workstations, with mandatory pre-restore safety backups, schema compatibility migration, and persistent read-only mode upon database corruption.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| In-place restore | SQLite Online Backup API (`Backup::new(&source, conn)`) | Atomic, hot swap into existing connection without file system handle races |
| Safety pre-backup | Mandatory snapshot before restoration | Guarantees existing live state is never destroyed if a restore fails |
| Schema advancement | Forward migrations applied to staging copy | Backups from older versions can be safely restored onto newer app builds |
| New machine recovery | Recovery key or physical Recovery Sheet | Decrypts and restores databases across different Windows machines |
| Corruption handling | Read-only mode flag | Prevents corrupted databases from executing writes, avoiding irreversible damage |

## Acceptance Criteria

### AC-1: Candidate Inspection & Preview
- RPC `restore.preview_backup` validates the candidate SQLite file before restore.
- Performs `PRAGMA integrity_check` and `PRAGMA foreign_key_check`.
- Returns record counts (students, payments, charges, expenses, enrollments) and financial ledger totals.
- Checks candidate schema version:
  - If older than current: flags `needs_migration = true`.
  - If newer than current: flags `needs_update = true` (blocks restore).

### AC-2: Safety Pre-Backup Guarantee
- Before overwriting the live database, `restore.execute` automatically triggers a hot snapshot of the current active database tagged as `pre_restore`.
- If the safety backup fails, the restore process immediately aborts.

### AC-3: Staging Migration & Atomic Swap
- If candidate schema is older than target schema version, migrations are run on an isolated temporary staging file.
- Restoration applies the staged database to the live connection using the SQLite backup API.

### AC-4: Emergency Recovery Key Support
- If restoring on a different machine or new Windows installation, the user can input the 24-word/hex recovery key.

### AC-5: Read-Only Mode & Corruption Banner
- `system.health_check` verifies integrity and foreign keys.
- If corruption is detected, the database is flagged read-only for the process lifetime.
- UI renders `CorruptionBanner` alerting the user and offering restoration options.

## RPC Endpoints

| Command | Auth | Gate | Kind |
|---|---|---|---|
| `restore.preview_backup` | `backup.restore` | None | Read |
| `restore.execute` | `backup.restore` | None | Mutation |
| `system.health_check` | Authenticated | None | Read |
