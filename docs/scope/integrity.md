# Operational integrity (Slice 8)

Back to [index](index.md). Finance integrity, communication health, offline resilience, and data safety features that protect the institution's operations. Most of these are tagged `· GA` for full rigor.

## Finance integrity

### 40. Receipt verification · GA
QR or short code on printed receipts that verifies the receipt ID, amount, date, student, and current status (valid, voided, refunded). Guardians or the center can scan to confirm authenticity.
**Done when:** every receipt prints a verification code; scanning or entering the code on the app confirms receipt details; a voided receipt shows its voided status; verification works offline.
spec [0032](../specs/0032-operational-integrity/0032-receipt-verification.md)
- [x] Design it (spec): `/architect receipt verification`
- [ ] Build it: `/develop receipt verification`
  - [ ] Migration 0017 `verification_code` column on payments and `receipt_verifications` table, covers AC-1
  - [ ] HMAC SHA256 offline signature generator and `integrity.verify_receipt` RPC, covers AC-1, AC-2
  - [ ] Voided and refunded payment detection with cashier audit details, covers AC-3
  - [ ] Thermal and A4 receipt QR code rendering and verification dialog UI, covers AC-1, AC-2, AC-3
- [ ] Verify it: `/check verify receipt verification`
- [ ] Test it: `/test receipt verification`
- [ ] Review it (fresh model): `/check review receipt verification`
- [ ] Document it: `/document receipt verification`

### 41. Duplicate payment protection · GA
Detect repeated payment references, suspicious duplicate amounts for the same student on the same day, and double submissions. Warn before recording, not after.
**Done when:** two payments with the same reference within 24 hours trigger a warning; same student, same amount, same day shows a confirmation prompt; the cashier can override with a reason; overrides are audited.
spec [0032](../specs/0032-operational-integrity/0032-duplicate-payment-protection.md)
- [x] Design it (spec): `/architect duplicate payment protection`
- [ ] Build it: `/develop duplicate payment protection`
  - [ ] Migration 0017 `duplicate_override_reason` column and reference composite index, covers AC-4
  - [ ] Core `integrity.check_duplicate_payment` pre submission collision analyzer, covers AC-1, AC-2
  - [ ] Cashier override validation requiring non trivial justification with audit logging, covers AC-3, AC-4
  - [ ] Cashier desk duplicate warning modal with reason capture input, covers AC-1, AC-2, AC-3
- [ ] Verify it: `/check verify duplicate payment protection`
- [ ] Test it: `/test duplicate payment protection`
- [ ] Review it (fresh model): `/check review duplicate payment protection`
- [ ] Document it: `/document duplicate payment protection`

## Communication health

### 42. Guardian contact health
Surface missing or invalid phone numbers, duplicate contacts, bounced emails, and sibling linked contacts. Helps the center maintain clean communication data.
**Done when:** a dashboard shows guardians with no phone, invalid format phones, or duplicate numbers; sibling linked guardians show which students share contacts; the center can bulk update from the health view.
spec [0032](../specs/0032-operational-integrity/0032-guardian-contact-health.md)
- [x] Design it (spec): `/architect guardian contact health`
- [ ] Build it: `/develop guardian contact health`
  - [ ] SQL contact health scanner categorizing invalid 11 digit numbers and duplicates, covers AC-1, AC-2
  - [ ] Sibling contact grouping linking multiple students to shared guardian phone, covers AC-3
  - [ ] Atomic `integrity.bulk_update_guardian_contacts` transaction handler, covers AC-4
  - [ ] Guardian Contact Health management dashboard with inline phone editor, covers AC-1, AC-3, AC-4
- [ ] Verify it: `/check verify guardian contact health`
- [ ] Test it: `/test guardian contact health`

### 43. Durable inbox/outbox · GA
Full lifecycle visibility for every message: queued, waiting, sending, sent, delivered, failed, and retry states. The outbox built in feature 26 gets a proper monitoring and management UI.
**Done when:** every message shows its current lifecycle state; failed messages can be retried individually or in bulk; delivery timestamps are recorded when providers report them; the view supports filtering by status, channel, and date.
spec [0032](../specs/0032-operational-integrity/0032-durable-outbox-and-offline-queue.md)
- [x] Design it (spec): `/architect durable inbox/outbox`
- [ ] Build it: `/develop durable inbox/outbox`
  - [ ] Outbox lifecycle aggregator RPC `integrity.get_outbox_lifecycle`, covers AC-4
  - [ ] Delivery attempt telemetry and individual or bulk retry handler, covers AC-4, AC-5
  - [ ] Outbox queue monitor UI with status badges and channel filters, covers AC-4, AC-5
- [ ] Verify it: `/check verify durable inbox/outbox`
- [ ] Test it: `/test durable inbox/outbox`
- [ ] Review it (fresh model): `/check review durable inbox/outbox`
- [ ] Document it: `/document durable inbox/outbox`

### 44. Offline message queue · GA
Messages wait safely in the local database when internet is unavailable and resume sending automatically when connectivity returns. No messages are lost during offline periods.
**Done when:** messages queued while offline survive a restart and send on reconnect; the UI clearly shows queued count and connectivity status; manual "send now" works when connectivity returns; no duplicate sends after reconnection.
spec [0032](../specs/0032-operational-integrity/0032-durable-outbox-and-offline-queue.md)
- [x] Design it (spec): `/architect offline message queue`
- [ ] Build it: `/develop offline message queue`
  - [ ] Startup queue recovery resetting interrupted sending rows, covers AC-1
  - [ ] Background network monitor with exponential backoff and auto flush on reconnect, covers AC-2
  - [ ] Deduplication key validator preventing duplicate transmissions, covers AC-3
  - [ ] Manual queue flush RPC `integrity.flush_offline_queue` and counter trigger, covers AC-6
- [ ] Verify it: `/check verify offline message queue`
- [ ] Test it: `/test offline message queue`
- [ ] Review it (fresh model): `/check review offline message queue`
- [ ] Document it: `/document offline message queue`

## Offline & resilience

### 45. Offline resilience center
A visible status panel showing internet connectivity, messaging pipeline health, LAN status (1.1), backup recency, database health, and system resource usage. Always accessible from the app shell.
**Done when:** connectivity changes are reflected within 5 seconds; the panel shows last successful backup time; messaging queue depth is visible; a simple green/yellow/red indicator summarizes overall health; clicking any indicator explains the status.
spec [0032](../specs/0032-operational-integrity/0032-offline-resilience-center.md)
- [x] Design it (spec): `/architect offline resilience center`
- [ ] Build it: `/develop offline resilience center`
  - [ ] Core `integrity.get_resilience_status` aggregator for network, outbox, backup, and disk, covers AC-2
  - [ ] Sub 5 second reactive polling hook with window idle detection, covers AC-1
  - [ ] App shell header status indicator with green, amber, red color coding, covers AC-1, AC-3
  - [ ] Diagnostic slide over drawer with queue breakdown and one click flush, covers AC-4
- [ ] Verify it: `/check verify offline resilience center`
- [ ] Test it: `/test offline resilience center`

## Data safety

### 46. Disaster recovery score · GA
A composite score summarizing backup age, whether an external copy exists, last verification result, disk health indicators, and overall recoverability status. Warns the owner when recovery readiness degrades.
**Done when:** the score updates after every backup; a backup older than 24 hours lowers the score; missing external copy lowers it further; the score is visible on the cockpit; the owner gets an alert when the score drops below a threshold.
spec [0032](../specs/0032-operational-integrity/0032-disaster-recovery-and-drills.md)
- [x] Design it (spec): `/architect disaster recovery score`
- [ ] Build it: `/develop disaster recovery score`
  - [ ] Migration 0017 `recovery_score_snapshots` table, covers AC-2
  - [ ] 100 point composite formula evaluating backup age, USB presence, drill result, and DB integrity, covers AC-1
  - [ ] Owner dashboard recovery score card with warning alerts below 70, covers AC-2
- [ ] Verify it: `/check verify disaster recovery score`
- [ ] Test it: `/test disaster recovery score`
- [ ] Review it (fresh model): `/check review disaster recovery score`
- [ ] Document it: `/document disaster recovery score`

### 47. Backup verification drill · GA
Automatic weekly restore drill that verifies the latest backup is actually readable, valid, and contains consistent data. Surfaces problems before a real disaster.
**Done when:** the drill runs on schedule without interrupting operations; a verification failure triggers an alert; the drill log shows what was checked and the result; drill history is accessible from the backup settings.
spec [0032](../specs/0032-operational-integrity/0032-disaster-recovery-and-drills.md)
- [x] Design it (spec): `/architect backup verification drill`
- [ ] Build it: `/develop backup verification drill`
  - [ ] Headless sandbox restore drill restoring snapshots to temporary disk instance, covers AC-3
  - [ ] PRAGMA checks, foreign key checks, and ledger balance reconciliation, covers AC-4
  - [ ] Weekly scheduled background drill runner and history log, covers AC-5
  - [ ] Drill history view and manual drill trigger in backup settings, covers AC-5
- [ ] Verify it: `/check verify backup verification drill`
- [ ] Test it: `/test backup verification drill`
- [ ] Review it (fresh model): `/check review backup verification drill`
- [ ] Document it: `/document backup verification drill`

### 48. One click PC migration · GA
Move the complete Coachify installation (database, documents, settings, templates, and user accounts) to another PC. Guides the user through export on the old PC and import on the new one.
**Done when:** migration from PC A to PC B preserves all data and settings; the old PC's license is released; the new PC activates with the same license; the migration handles the recovery key re-wrapping for the new machine's DPAPI context.
spec [0032](../specs/0032-operational-integrity/0032-one-click-pc-migration.md)
- [x] Design it (spec): `/architect one click PC migration`
- [ ] Build it: `/develop one click PC migration`
  - [ ] Migration 0017 `pc_migrations` registry table, covers AC-5
  - [ ] Streaming AES 256 GCM encrypted bundle packager with Argon2id recovery key derivation, covers AC-1, AC-2
  - [ ] Archive unpacker, checksum validator, and Windows DPAPI key rewrapper, covers AC-3, AC-4
  - [ ] Guided PC Migration export and import wizard UI, covers AC-1, AC-3
- [ ] Verify it: `/check verify one click PC migration`
- [ ] Test it: `/test one click PC migration`
- [ ] Review it (fresh model): `/check review one click PC migration`
- [ ] Document it: `/document one click PC migration`

### 49. Database integrity checks · GA
Detect database and storage problems (corruption, orphaned records, broken foreign keys, inconsistent balances, broken audit chain) before they become serious. Runs on startup and on demand.
**Done when:** startup check completes within 3 seconds on the reference PC; a detected problem shows a clear message and recovery option; the check covers integrity_check, foreign key check, balance reconciliation, and audit chain verification; results are logged.
spec [0032](../specs/0032-operational-integrity/0032-database-integrity-checks.md)
- [x] Design it (spec): `/architect database integrity checks`
- [ ] Build it: `/develop database integrity checks`
  - [ ] Migration 0017 `integrity_check_runs` diagnostic history table, covers AC-5
  - [ ] Core 4 layer integrity check engine with sub 3 second startup benchmark, covers AC-1, AC-2
  - [ ] Protective read only lock enforcement in dispatcher upon corruption detection, covers AC-4
  - [ ] On demand check RPC `integrity.run_database_integrity_checks` and emergency recovery banner UI, covers AC-3, AC-4
- [ ] Verify it: `/check verify database integrity checks`
- [ ] Test it: `/test database integrity checks`
- [ ] Review it (fresh model): `/check review database integrity checks`
- [ ] Document it: `/document database integrity checks`
