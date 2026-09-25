# Release hardening (Slice 11)

Back to [index](index.md). Release criteria live in Section 17 of [architecture-review.md](../architecture-review.md).

### 58. Crash & failure safety suite · GA
Automated forced process termination and failure tests on Windows: power loss during payment, backup, and migration; printer offline; USB removed; disk low; SMS timeouts; clock changes; interrupted updates.
**Done when:** 20 forced kills during payments, 10 during backups, and 10 during migrations produce zero corrupted databases, zero duplicate payments, and zero partial records; every failure shows its expected behavior.
spec [0036](../specs/0036-release-hardening/0036-crash-and-failure-safety-suite.md)
- [x] Design it (spec): `/architect crash & failure safety suite`
- [x] Build it: `/develop crash & failure safety suite`
  - [x] Compile time fault injection feature and deterministic checkpoint hooks in core, covers AC-4
  - [x] xtask crash suite runner with Windows TerminateProcess and child worker binary, covers AC-1
  - [x] Automated payment, backup, and migration crash tests (40 total forced kills), covers AC-1, AC-2, AC-3
  - [x] Environmental failure simulation for low disk, USB disconnect, and clock jumps, covers AC-4, AC-6, AC-7
- [x] Verify it: `/check verify crash & failure safety suite`
- [x] Test it: `/test crash & failure safety suite`
- [x] Review it (fresh model): `/check review crash & failure safety suite`
- [x] Document it: `/document crash & failure safety suite`

### 59. Performance on the reference PC · deferred to 1.1
Measure the targets on a low end PC (4 GB RAM, spinning disk) with the large generated dataset and fix what misses. Deferred out of 1.0 scope per user decision.

### 60. Customer docs & support runbook
Quick start, backup and restore, printer setup, SMS setup, Excel import, troubleshooting (including SmartScreen and antivirus), privacy and data ownership, license guide, internal support runbook, and in app help dialog.
**Done when:** a person who has never seen the app installs it, imports students, prints a receipt, and restores a backup using only the docs; in app help modal provides offline guidance.
spec [0036](../specs/0036-release-hardening/0036-customer-docs-and-support-runbook.md)
- [x] Design it (spec): `/architect customer docs & support runbook`
- [ ] Build it: `/develop customer docs & support runbook`
  - [ ] Author eight customer guides in docs/customer/ and support runbook, covers AC-9, AC-10
  - [ ] Configure Vite raw markdown loader and build in app Help modal, covers AC-8
  - [ ] Add Help navigation button to desktop shell and verify printable PDFs, covers AC-8, AC-9
- [ ] Verify it: `/check verify customer docs & support runbook`
- [ ] Test it: `/test customer docs & support runbook`

### 61. 1.0 readiness & pilots · GA
Three pilot institutions for at least four weeks each, the print and install matrix, and the readiness report stating exactly what was tested.
**Done when:** every item in the 1.0 release criteria is met and the readiness report is written.
spec [0036](../specs/0036-release-hardening/0036-pilot-readiness-and-release-criteria.md)
- [x] Design it (spec): `/architect 1.0 readiness & pilots`
- [ ] Build it: `/develop 1.0 readiness & pilots`
  - [ ] Structured 1.0 readiness qualification report template in docs/release/, covers AC-11
  - [ ] Pilot coaching center tracking registry and verification matrix in docs/release/, covers AC-12
  - [ ] Real field trial verification across 3 pilot institutions for 4+ weeks with zero data loss, covers AC-12
  - [ ] Final release criteria audit and sign off, covers AC-11, AC-12
- [ ] Verify it: `/check verify 1.0 readiness & pilots`
- [ ] Test it: `/test 1.0 readiness & pilots`
- [ ] Review it (fresh model): `/check review 1.0 readiness & pilots`
- [ ] Document it: `/document 1.0 readiness & pilots`
