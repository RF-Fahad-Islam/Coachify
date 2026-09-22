# Spec 0021 — Diagnostics & Logs

> Feature 23 · Slice 3 (Safety & Delivery)

## Goal

Provide institutional administrators and remote support teams with comprehensive system telemetry, database health metrics, and an offline diagnostic bundle that strictly protects institutional privacy by redacting all Personally Identifiable Information (PII) and secrets.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| PII Redaction | Automated phone & name masking | Prevents student/guardian privacy violations under GDPR and Bangladesh data regulations |
| Diagnostic Bundle | ZIP archive containing structured JSON & logs | Easily exportable offline without requiring internet or telemetry daemons |
| Low-level metrics | SQLite PRAGMAs (page_count, freelist_count, integrity_check) | Real-time visibility into database file health and WAL growth |

## Acceptance Criteria

### AC-1: Diagnostics Information Collection
- RPC `diagnostics.get_info` returns:
  - Application version, schema version, OS version
  - Database file path, database size in bytes, WAL size in bytes, freelist page count
  - Free space on active data drive and backup storage drive
  - Row counts for all domain database tables

### AC-2: Strict PII Redaction
- Bangladeshi mobile phone numbers (`01XXXXXXXXX`, `8801XXXXXXXXX`) are masked as `01X***XXXX` / `8801X***XXXX`.
- Passwords, recovery keys, and API tokens are omitted or replaced with `[REDACTED]`.
- Student names, guardian identities, and financial balance details are never included in exported diagnostic archives.

### AC-3: Sanitized Bundle Export
- RPC `diagnostics.export_bundle` packages:
  - `info.json`: system and build metadata
  - `database_stats.json`: table row counts and file sizes
  - `backup_history.json`: recent backup history without student records
  - `settings.json`: application settings with secrets redacted
  - `integrity_check.txt`: PRAGMA integrity_check output
  - `recent_logs.txt`: sanitized log file output

## RPC Endpoints

| Command | Auth | Gate | Kind |
|---|---|---|---|
| `diagnostics.get_info` | `diagnostics.view` | None | Read |
| `diagnostics.export_bundle` | `diagnostics.export` | None | Mutation |
| `diagnostics.run_integrity_check` | `diagnostics.export` | None | Read |
