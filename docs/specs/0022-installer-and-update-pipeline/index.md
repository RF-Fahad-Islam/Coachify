# Spec 0022 — Installer & Update Pipeline

> Feature 24 · Slice 3 (Safety & Delivery)

## Goal

Package Coachify Desktop as a robust per-machine Windows installer (NSIS) and continuous update pipeline that guarantees zero data loss across installs, updates, and uninstalls.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Installer type | NSIS (per-machine mode) | Installs to `Program Files`, single install serves all local workstation users |
| Elevation | Administrator (UAC required) | Standard enterprise deployment in schools and coaching centers |
| Uninstall retention | Keep `%LOCALAPPDATA%\com.coachify.desktop` | Prevents accidental deletion of institution records when uninstalling |
| Safe updates | Mandatory pre-update snapshot via `updater.rs` | Prevents bricked databases if an update encounters power loss or crash |
| Standalone runtime | Bundled offline fonts and SQLite engine | 100% offline-first operation with zero internet dependencies |

## Acceptance Criteria

### AC-1: NSIS Per-Machine Packaging
- `tauri.conf.json` configures `installMode: "perMachine"`.
- Creates Desktop shortcut and Start Menu item.
- Installs to `C:\Program Files\Coachify\`.

### AC-2: Data Preservation on Uninstall
- Uninstalling through Windows Settings removes binaries and shortcuts but strictly preserves `%LOCALAPPDATA%\com.coachify.desktop`.

### AC-3: Safe Update Pre-Backup
- `updater::prepare_for_update` captures and verifies a hot backup before update staging.
- `updater::verify_installer_payload` prevents execution of truncated or corrupted downloads.

### AC-4: CI Release Pipeline
- `.github/workflows/release.yml` automatically runs full quality gates, builds UI, and generates the NSIS installer on tag pushes.
