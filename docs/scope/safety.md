# Safety & delivery (Slice 3, pilot gate)

Back to [index](index.md). When this slice is done, a friendly pilot center can start using fee collection.

### 21. Backup & verification · done · GA
Daily snapshots plus an incremental document vault mirror at the destination, hourly local snapshots during business hours, verification after each backup, a weekly automatic restore drill, and tiered retention.
**Done when:** a backup taken during active writes restores with identical counts and totals; a flipped byte fails verification; a removed USB drive leaves the previous backup intact; the dashboard shows created and verified separately.
spec [0019](../specs/0019-backup-and-verification/index.md) · code in `crates/core/src/backup/`
- [x] Design it (spec): `/architect backup & verification`
- [x] Build it: `/develop backup & verification`
- [x] Verify it: `/check verify backup & verification`
- [x] Test it: `/test backup & verification`
- [x] Review it (fresh model): `/check review backup & verification`
- [x] Document it: `/document backup & verification`

### 22. Restore & corruption recovery · done · GA
Restore on the same PC or a new one (with the Recovery Sheet), safety backup first, migrate if older, validate, swap; and a read only recovery path when the live database fails its check.
**Done when:** the wipe, reinstall, restore test passes; a wrong recovery key is refused clearly; a corrupted database is never written to again.
spec [0020](../specs/0020-restore-and-corruption-recovery/index.md) · code in `crates/core/src/restore/`
- [x] Design it (spec): `/architect restore & corruption recovery`
- [x] Build it: `/develop restore & corruption recovery`
- [x] Verify it: `/check verify restore & corruption recovery`
- [x] Test it: `/test restore & corruption recovery`
- [x] Review it (fresh model): `/check review restore & corruption recovery`
- [x] Document it: `/document restore & corruption recovery`

### 23. Diagnostics & logs · done
Structured logs with redaction and rotation, a diagnostics screen, and an exportable bundle that contains no names, phones, or secrets.
**Done when:** a support bundle exports offline and a scan of it finds no secrets or personal data.
spec [0021](../specs/0021-diagnostics-and-logs/index.md) · code in `crates/core/src/diagnostics/`
- [x] Design it (spec): `/architect diagnostics & logs`
- [x] Build it: `/develop diagnostics & logs`
- [x] Verify it: `/check verify diagnostics & logs`
- [x] Test it: `/test diagnostics & logs`

### 24. Installer & update pipeline · done · GA
Per machine installer (standard and offline variants), data kept on uninstall, firewall rule placeholder for 1.1, updater with pre update backup, cached previous installer, and CI that builds and publishes to the internal channel.
**Done when:** clean install, upgrade by installer, upgrade by updater, and uninstall keeping data all pass on Windows 10 and 11; an interrupted download never installs.
spec [0022](../specs/0022-installer-and-update-pipeline/index.md) · code in `src-tauri/` and `.github/workflows/release.yml`
- [x] Design it (spec): `/architect installer & update pipeline`
- [x] Build it: `/develop installer & update pipeline`
- [x] Verify it: `/check verify installer & update pipeline`
- [x] Test it: `/test installer & update pipeline`
- [x] Review it (fresh model): `/check review installer & update pipeline`
- [x] Document it: `/document installer & update pipeline`
