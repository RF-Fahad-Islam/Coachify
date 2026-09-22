# Safety & delivery (Slice 3, pilot gate)

Back to [index](index.md). When this slice is done, a friendly pilot center can start using fee collection.

### 21. Backup & verification · in-progress · GA
Daily snapshots plus an incremental document vault mirror at the destination, hourly local snapshots during business hours, verification after each backup, a weekly automatic restore drill, and tiered retention.
**Done when:** a backup taken during active writes restores with identical counts and totals; a flipped byte fails verification; a removed USB drive leaves the previous backup intact; the dashboard shows created and verified separately.
spec [0019](../specs/0019-backup-and-verification/index.md)
- [x] Design it (spec): `/architect backup & verification`
- [ ] Build it: `/develop backup & verification`
- [ ] Verify it: `/check verify backup & verification`
- [ ] Test it: `/test backup & verification`
- [ ] Review it (fresh model): `/check review backup & verification`
- [ ] Document it: `/document backup & verification`

### 22. Restore & corruption recovery · in-progress · GA
Restore on the same PC or a new one (with the Recovery Sheet), safety backup first, migrate if older, validate, swap; and a read only recovery path when the live database fails its check.
**Done when:** the wipe, reinstall, restore test passes; a wrong recovery key is refused clearly; a corrupted database is never written to again.
spec [0020](../specs/0020-restore-and-corruption-recovery/index.md)
- [x] Design it (spec): `/architect restore & corruption recovery`
- [ ] Build it: `/develop restore & corruption recovery`
- [ ] Verify it: `/check verify restore & corruption recovery`
- [ ] Test it: `/test restore & corruption recovery`
- [ ] Review it (fresh model): `/check review restore & corruption recovery`
- [ ] Document it: `/document restore & corruption recovery`

### 23. Diagnostics & logs · in-progress
Structured logs with redaction and rotation, a diagnostics screen, and an exportable bundle that contains no names, phones, or secrets.
**Done when:** a support bundle exports offline and a scan of it finds no secrets or personal data.
spec [0021](../specs/0021-diagnostics-and-logs/index.md)
- [x] Design it (spec): `/architect diagnostics & logs`
- [ ] Build it: `/develop diagnostics & logs`
- [ ] Verify it: `/check verify diagnostics & logs`
- [ ] Test it: `/test diagnostics & logs`

### 24. Installer & update pipeline · in-progress · GA
Per machine installer (standard and offline variants), data kept on uninstall, firewall rule placeholder for 1.1, updater with pre update backup, cached previous installer, and CI that builds and publishes to the internal channel.
**Done when:** clean install, upgrade by installer, upgrade by updater, and uninstall keeping data all pass on Windows 10 and 11; an interrupted download never installs.
spec [0022](../specs/0022-installer-and-update-pipeline/index.md)
- [x] Design it (spec): `/architect installer & update pipeline`
- [ ] Build it: `/develop installer & update pipeline`
- [ ] Verify it: `/check verify installer & update pipeline`
- [ ] Test it: `/test installer & update pipeline`
- [ ] Review it (fresh model): `/check review installer & update pipeline`
- [ ] Document it: `/document installer & update pipeline`
