# Spec 0019 — Backup & Verification

> Feature 21 · Slice 3 (Safety)

## Goal

Protect institution data against device loss, theft, corruption, and accidental
deletion by providing automated, verifiable backups with a clear restore path.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Backup method | SQLite online backup API (`sqlite3_backup_*`) | Consistent snapshot without stopping the app |
| Manifest integrity | HMAC-SHA256 over manifest JSON | Detects silent corruption and tampering |
| Vault mirroring | Incremental file copy (content-addressed) | Only copies new documents |
| Retention | 7 daily + tagged for 30 days | Balances storage with safety |
| Background jobs | Lease-based job queue in `jobs` table | Simple, no external runtime |

## Acceptance Criteria

### AC-1: Backup Destination Management
- Owner can add, list, and remove backup destinations.
- Each destination writes a `.coachify-backup-target` marker file.
- Destinations are validated (writable, sufficient space).

### AC-2: Manual Backup
- Owner can trigger a manual backup via RPC `backup.create_snapshot`.
- Backup uses SQLite online backup API (hot, non-blocking).
- A `manifest.json` is written alongside the database file.
- Manifest includes: record counts, money totals, audit chain head, vault entries.
- HMAC-SHA256 is computed over the manifest and stored in `manifest.hmac`.

### AC-3: Backup Verification
- RPC `backup.verify` marks a backup as verified after integrity checks.
- Full verification runs: `PRAGMA integrity_check`, `PRAGMA foreign_key_check`,
  record count comparison, money total comparison, HMAC verification.

### AC-4: Backup History
- RPC `backup.list_history` returns paginated backup records.
- Each record includes: kind, status, file size, record counts, money totals.
- Statuses: `in_progress`, `completed`, `verified`, `failed`, `corrupted`.

### AC-5: Backup Schedules
- Owner can configure daily and hourly snapshot schedules.
- Schedules are persisted in `backup_schedules` table.
- Background job runner picks up scheduled jobs.

### AC-6: Backup Dashboard
- `backup.get_dashboard` returns: last backup, last verified, last drill,
  destinations, schedules, warnings, database size, backup count.
- Warnings generated for: no backups in 3 days, no verified backup,
  inaccessible destinations, no destinations configured.

### AC-7: Retention & Pruning
- `prune_backups` keeps the 7 most recent daily backups.
- Tagged backups (pre_update, pre_restore) kept for 30 days.
- The last verified backup is never pruned.

### AC-8: Health Check
- `system.health_check` runs integrity + foreign key check.
- If either fails, database is flagged as corrupted (read-only mode).
- Read-only flag persists for the process lifetime.

## Database Schema

### Migration 0013

Tables: `backup_destinations`, `backup_history`, `backup_schedules`,
`restore_drills`, `jobs`.

Permissions seeded: `backup.manage`, `backup.restore`, `diagnostics.view`,
`diagnostics.export`.

## RPC Endpoints

| Command | Auth | Gate | Kind |
|---|---|---|---|
| `backup.get_dashboard` | Authenticated | None | Read |
| `backup.create_snapshot` | `backup.manage` | None | Mutation |
| `backup.verify` | `backup.manage` | None | Mutation |
| `backup.list_history` | `backup.manage` | None | Read |
| `backup.list_destinations` | `backup.manage` | None | Read |
| `backup.save_destination` | `backup.manage` | None | Mutation |
| `backup.remove_destination` | `backup.manage` | None | Mutation |
| `backup.get_schedules` | `backup.manage` | None | Read |
| `backup.save_schedule` | `backup.manage` | None | Mutation |

> **Note**: All backup operations use `LicenseGate::None` — backups always
> work, even when the institution license has expired past the grace period.

## File Inventory

| File | Role |
|---|---|
| `migrations/0013_backup_infrastructure.sql` | Schema for backup tables |
| `crates/core/src/backup.rs` | Snapshot, manifest, verify, prune, health |
| `crates/core/src/jobs.rs` | Background job queue with leases |
| `crates/core/src/diagnostics.rs` | System info, bundle export, PII redaction |
| `crates/core/src/dispatcher.rs` | RPC routing for backup.*, system.health_check |
