# Safety & delivery (Slice 3, pilot gate)

Back to [index](index.md). When this slice is done, a friendly pilot center can start using fee collection.

### 21. Backup & verification · needs a decision · GA
Daily snapshots plus an incremental document vault mirror at the destination, hourly local snapshots during business hours, verification after each backup, a weekly automatic restore drill, and tiered retention.
**Done when:** a backup taken during active writes restores with identical counts and totals; a flipped byte fails verification; a removed USB drive leaves the previous backup intact; the dashboard shows created and verified separately.
- [ ] Design it (spec): `/architect backup & verification`

### 22. Restore & corruption recovery · needs a decision · GA
Restore on the same PC or a new one (with the Recovery Sheet), safety backup first, migrate if older, validate, swap; and a read only recovery path when the live database fails its check.
**Done when:** the wipe, reinstall, restore test passes; a wrong recovery key is refused clearly; a corrupted database is never written to again.
- [ ] Design it (spec): `/architect restore & corruption recovery`

### 23. Diagnostics & logs
Structured logs with redaction and rotation, a diagnostics screen, and an exportable bundle that contains no names, phones, or secrets.
**Done when:** a support bundle exports offline and a scan of it finds no secrets or personal data.
- [ ] Build it: `/develop diagnostics & logs`

### 24. Installer & update pipeline · needs a decision · GA
Per machine installer (standard and offline variants), data kept on uninstall, firewall rule placeholder for 1.1, updater with pre update backup, cached previous installer, and CI that builds and publishes to the internal channel.
**Done when:** clean install, upgrade by installer, upgrade by updater, and uninstall keeping data all pass on Windows 10 and 11; an interrupted download never installs.
- [ ] Design it (spec): `/architect installer & update pipeline`
