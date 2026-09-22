# Release hardening (Slice 11)

Back to [index](index.md). Release criteria live in Section 17 of [architecture-review.md](../architecture-review.md).

### 58. Crash & failure safety suite · needs a decision · GA
Automated forced kill and failure tests on Windows: power loss during payment, backup, and migration; printer offline; USB removed; disk low; SMS timeouts; clock changes; interrupted updates.
**Done when:** 200 forced kills during payments, 50 during backups, and 50 during migrations produce zero corrupted databases, zero duplicate payments, and zero partial records; every failure in the review's table shows its expected behavior.
- [ ] Design it (spec): `/architect crash & failure safety suite`

### 59. Performance on the reference PC
Measure the targets on a low end PC (4 GB RAM, spinning disk) with the large generated dataset and fix what misses.
**Done when:** search, cashier, payment, attendance, report, backup, startup, and memory targets are all met and recorded.
- [ ] Build it: `/develop performance on the reference PC`

### 60. Customer docs & support runbook
Quick start, backup and restore, printer setup, SMS setup, Excel import, troubleshooting (including SmartScreen and antivirus), privacy and data ownership, license guide, and an internal support runbook.
**Done when:** a person who has never seen the app installs it, imports students, prints a receipt, and restores a backup using only the docs.
- [ ] Build it: `/develop customer docs & support runbook`

### 61. 1.0 readiness & pilots · GA
Three pilot institutions for at least four weeks each, the print and install matrix, and the readiness report stating exactly what was tested.
**Done when:** every item in the 1.0 release criteria is met and the readiness report is written.
- [ ] Build it: `/develop 1.0 readiness & pilots`
