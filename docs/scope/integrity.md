# Operational integrity (Slice 8)

Back to [index](index.md). Finance integrity, communication health, offline resilience, and data safety features that protect the institution's operations. Most of these are tagged `· GA` for full rigor.

## Finance integrity

### 40. Receipt verification · needs a decision · GA
QR or short code on printed receipts that verifies the receipt ID, amount, date, student, and current status (valid, voided, refunded). Guardians or the center can scan to confirm authenticity.
**Done when:** every receipt prints a verification code; scanning or entering the code on the app confirms receipt details; a voided receipt shows its voided status; verification works offline.
- [ ] Design it (spec): `/architect receipt verification`

### 41. Duplicate payment protection · needs a decision · GA
Detect repeated payment references, suspicious duplicate amounts for the same student on the same day, and double submissions. Warn before recording, not after.
**Done when:** two payments with the same reference within 24 hours trigger a warning; same student, same amount, same day shows a confirmation prompt; the cashier can override with a reason; overrides are audited.
- [ ] Design it (spec): `/architect duplicate payment protection`

## Communication health

### 42. Guardian contact health · needs a decision
Surface missing or invalid phone numbers, duplicate contacts, bounced emails, and sibling linked contacts. Helps the center maintain clean communication data.
**Done when:** a dashboard shows guardians with no phone, invalid format phones, or duplicate numbers; sibling linked guardians show which students share contacts; the center can bulk update from the health view.
- [ ] Design it (spec): `/architect guardian contact health`

### 43. Durable inbox/outbox · needs a decision · GA
Full lifecycle visibility for every message: queued, waiting, sending, sent, delivered, failed, and retry states. The outbox built in feature 26 gets a proper monitoring and management UI.
**Done when:** every message shows its current lifecycle state; failed messages can be retried individually or in bulk; delivery timestamps are recorded when providers report them; the view supports filtering by status, channel, and date.
- [ ] Design it (spec): `/architect durable inbox/outbox`

### 44. Offline message queue · needs a decision · GA
Messages wait safely in the local database when internet is unavailable and resume sending automatically when connectivity returns. No messages are lost during offline periods.
**Done when:** messages queued while offline survive a restart and send on reconnect; the UI clearly shows queued count and connectivity status; manual "send now" works when connectivity returns; no duplicate sends after reconnection.
- [ ] Design it (spec): `/architect offline message queue`

## Offline & resilience

### 45. Offline resilience center · needs a decision
A visible status panel showing internet connectivity, messaging pipeline health, LAN status (1.1), backup recency, database health, and system resource usage. Always accessible from the app shell.
**Done when:** connectivity changes are reflected within 5 seconds; the panel shows last successful backup time; messaging queue depth is visible; a simple green/yellow/red indicator summarizes overall health; clicking any indicator explains the status.
- [ ] Design it (spec): `/architect offline resilience center`

## Data safety

### 46. Disaster recovery score · needs a decision · GA
A composite score summarizing backup age, whether an external copy exists, last verification result, disk health indicators, and overall recoverability status. Warns the owner when recovery readiness degrades.
**Done when:** the score updates after every backup; a backup older than 24 hours lowers the score; missing external copy lowers it further; the score is visible on the cockpit; the owner gets an alert when the score drops below a threshold.
- [ ] Design it (spec): `/architect disaster recovery score`

### 47. Backup verification drill · needs a decision · GA
Automatic weekly restore drill that verifies the latest backup is actually readable, valid, and contains consistent data. Surfaces problems before a real disaster.
**Done when:** the drill runs on schedule without interrupting operations; a verification failure triggers an alert; the drill log shows what was checked and the result; drill history is accessible from the backup settings.
- [ ] Design it (spec): `/architect backup verification drill`

### 48. One click PC migration · needs a decision · GA
Move the complete Coachify installation (database, documents, settings, templates, and user accounts) to another PC. Guides the user through export on the old PC and import on the new one.
**Done when:** migration from PC A to PC B preserves all data and settings; the old PC's license is released; the new PC activates with the same license; the migration handles the recovery key re-wrapping for the new machine's DPAPI context.
- [ ] Design it (spec): `/architect one click PC migration`

### 49. Database integrity checks · needs a decision · GA
Detect database and storage problems (corruption, orphaned records, broken foreign keys, inconsistent balances, broken audit chain) before they become serious. Runs on startup and on demand.
**Done when:** startup check completes within 3 seconds on the reference PC; a detected problem shows a clear message and recovery option; the check covers integrity_check, foreign key check, balance reconciliation, and audit chain verification; results are logged.
- [ ] Design it (spec): `/architect database integrity checks`
