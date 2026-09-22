# Coachify Desktop Installation & Deployment Guide

> Reference Guide for Institute Administrators & Pilot Deployments  
> Feature 24 · Slice 3 (Safety & Delivery)

---

## 1. System Requirements

Coachify is engineered to run reliably on modern and reference-tier Windows desktop workstations:

| Component | Minimum Specification | Recommended Specification |
|---|---|---|
| **Operating System** | Windows 10 (64-bit, Build 19041+) | Windows 11 (64-bit, Build 22000+) |
| **Processor** | Intel Core i3 / AMD Ryzen 3 | Intel Core i5 / AMD Ryzen 5 or higher |
| **System Memory (RAM)**| 4 GB RAM | 8 GB RAM or higher |
| **Disk Storage** | 500 MB free space for application; 5 GB+ recommended for database snapshots | Solid State Drive (SSD) |
| **Architecture** | x86_64 (64-bit) | x86_64 (64-bit) |
| **Display Resolution** | 1280 x 800 | 1920 x 1080 (Full HD) |
| **Internet Access** | None required (100% offline-first operation) | Optional for external updates |

---

## 2. Installation Architecture

Coachify distributes as an official **Nullsoft Scriptable Install System (NSIS)** Windows installer package:

### 2.1 Per-Machine Installation Mode
- Installs into `C:\Program Files\Coachify\`.
- Requires administrator elevation (UAC prompt) during initial setup.
- Enables any local Windows user account on the workstation (e.g. `Cashier1`, `Admin`, `Reception`) to launch and run Coachify without individual per-user app reinstallation.
- Automatically creates:
  - Desktop shortcut: `Coachify.lnk`
  - Start Menu entry: `Coachify` under `Programs`

### 2.2 Offline Runtimes
The standalone installer bundles all assets needed for execution:
- Embedded SQLite engine with WAL journaling and online hot backup support
- Bundled Unicode fonts for Bangla script rendering (Hind Siliguri) and Latin typography (Noto Sans, Inter)
- Windows DPAPI envelope encryption integration

---

## 3. First-Time Setup & Deployment

1. **Download & Launch**:
   Run `Coachify-Setup-x64.exe` as Administrator.
2. **Windows SmartScreen Prompt** (Pilot Phase):
   Because pilot releases use internal test certificates before EV certificate reputation maturity:
   - Click **More info**.
   - Click **Run anyway**.
3. **Target Selection**:
   The installer defaults to `C:\Program Files\Coachify`. Click **Install**.
4. **Initial Bootstrap**:
   Launch Coachify from the Desktop icon. The **Setup Wizard** will appear:
   - Configure institution name, code, and address.
   - Bootstrap the Owner account credentials.
   - Print or save the **Physical Owner Recovery Sheet**.
   - Select primary and backup storage drives.

---

## 4. Upgrading & Continuous Delivery

### 4.1 Safe Update Pipeline
Coachify guarantees zero-risk application upgrades via the pre-update safety pipeline:

1. **Pre-Update Safety Snapshot**:
   Before new binaries or migrations are executed, Coachify triggers `updater::prepare_for_update`, capturing a full hot database backup into the backups directory.
2. **Transactional Migration Staging**:
   When new versions contain database migrations:
   - The migration engine runs on an isolated staging database copy.
   - If migration succeeds, the production database is advanced.
   - If migration fails, the live database is untouched and Coachify enters the **Migration Recovery Screen** with full rollback safety.
3. **Payload Verification**:
   Interrupted or corrupted installer downloads are rejected before execution via `updater::verify_installer_payload`.

---

## 5. Data Preservation & Uninstallation

### 5.1 Uninstall Policy
When uninstalling Coachify via **Windows Settings > Apps > Installed apps**:
- Application binaries in `C:\Program Files\Coachify` and shortcuts are cleanly removed.
- **Institutional databases, audit trail logs, and encryption envelopes are strictly preserved** in:
  ```text
  %LOCALAPPDATA%\com.coachify.desktop\
  ```
- Reinstalling Coachify or upgrading to a newer release immediately reconnects to existing institution data without data loss.

### 5.2 Complete Purge (Manual Only)
If a test workstation must be completely wiped for factory clean testing:
1. Uninstall Coachify via Add or Remove Programs.
2. Manually delete the directory `%LOCALAPPDATA%\com.coachify.desktop`.
3. Manually delete any external backup directory configured on secondary drives (e.g. `D:\CoachifyBackups`).

---

## 6. Security, Antivirus, and Firewall

- **Firewall**: Coachify operates strictly local-first. In version 1.0, no inbound network listening ports are required. (LAN multi-counter sync in v1.1 will request a single inbound port rule).
- **Antivirus Exclusions**: If high-performance scanning slows SQLite WAL write operations on older spinning disks, add an exclusion for `%LOCALAPPDATA%\com.coachify.desktop\*.db*`.
