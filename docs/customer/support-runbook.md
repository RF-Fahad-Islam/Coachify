# Coachify Internal Technical Support Runbook (Tier 1 & Tier 2)

**Audience**: Coachify Field Technicians & Customer Support Specialists  
**Security Classification**: Internal Operational Procedure (Contains ZERO master keys or hardcoded credentials)

---

## Mandatory Identity Verification Gate (পরিচয় যাচাইকরণ নীতি)

> **CRITICAL SECURITY RULE**: Before assisting any caller with password resets, clock jump overrides, license machine transfers, or backup recovery drills, the support specialist **MUST verify the caller's identity**:
> 1. Confirm the caller is calling from the **Registered Owner Mobile Number** on file in the Coachify Commercial Registry.
> 2. Verify the **Institution Name**, **Device Fingerprint (`Machine ID`)**, and the **last 4 characters of the latest Receipt Number or Backup Manifest Hash**.
> 3. Never ask the customer to read out their full 24-word Encryption Recovery Key over an unrecorded channel, and never store customer recovery keys in support ticket notes.

---

## Procedure 1: Lost Encryption Recovery Sheet (হারানো রিকভারি শিট পুনরুদ্ধার)

### Scenario A: The Institution PC is Still Working & Owner Can Log In
If the owner lost their printed Recovery Sheet (`Recovery Sheet Printable`), **the live database is still intact**:
1. Guide the Owner to log in with their **Owner** account.
2. Navigate to **Settings → Data Safety & Backups → Rotate / Reprint Recovery Key**.
3. Have the Owner authenticate with their current Owner password.
4. Generate a fresh **Encryption Recovery Sheet**, immediately print two physical copies on A4 paper, and immediately run **Create Verified Backup Now** so the newest `.cbackup` archive on their USB drive matches the new Recovery Sheet.
5. Delete any older `.cbackup` files encrypted under the lost key once the new backup passes a **Backup Verification Drill**.

### Scenario B: Both the PC Hard Drive AND the Recovery Sheet Are Lost
- Explain clearly that `.cbackup` files use zero-knowledge `ChaCha20-Poly1305` authenticated encryption derived from the institution's Recovery Key. Without the Recovery Sheet (or the live Windows machine's local database file `coachify.db`), `.cbackup` files cannot be decrypted by vendor personnel.
- Check if the original SSD/HDD from the damaged PC can be connected via a **SATA-to-USB enclosure** (`৳400` adapter) to extract the raw `%LOCALAPPDATA%\Coachify\data\coachify.db` and `coachify.db-wal` files directly, which do not require the `.cbackup` archive wrapper key.

---

## Procedure 2: Database Corruption & WAL Recovery (`coachify.db-wal`)

If a machine suffers physical disk sector damage or forced power removal with a faulty third-party SSD controller and triggers the **Corruption Recovery Banner** (`CorruptionBanner`):

### Tier 1 Steps (In-App Automated Recovery):
1. Instruct the Owner to click **Launch Restore & Recovery Wizard** directly from the red `CorruptionBanner`.
2. Select the most recent verified `.cbackup` file from `D:\CoachifyBackups` or the external USB drive.
3. Run the **Pre-Restore Sandbox Verification** (`PRAGMA integrity_check` + `audit_chain_head` verification).
4. Click **Confirm & Restore Database**. Coachify automatically quarantines the damaged file into `backups/quarantine-<timestamp>/` before swapping in the healthy database.

### Tier 2 Steps (Manual WAL Salvage if USB Backup is Days Behind):
1. Close Coachify completely (`Task Manager` → verify `coachify.exe` is not running).
2. Copy the entire `%LOCALAPPDATA%\Coachify\data\` folder (`coachify.db`, `coachify.db-wal`, `coachify.db-shm`) to a safe working folder `C:\CoachifySalvage\`.
3. Run SQLite recovery inspection on the salvage copy:
   - Execute `sqlite3 coachify.db "PRAGMA wal_checkpoint(TRUNCATE); PRAGMA integrity_check;"`
   - If `integrity_check` reports only non-critical index corruption (`idx_...`), run `sqlite3 coachify.db "REINDEX;"` and re-verify `PRAGMA integrity_check;`.
   - Verify the financial ledger and audit trail hash chain via **Settings → Database Integrity View** (`DatabaseIntegrityView`).

---

## Procedure 3: Emergency Clock Jump Recovery (`acknowledge_clock_jump`)

When a desktop motherboard's CMOS battery (`CR2032`) dies during a power blackout, Windows may boot with a system timestamp in the past (e.g., `2021-01-01`). Coachify's monotonic clock guard blocks financial writes when `current_system_time < max_recorded_timestamp` to prevent out-of-order ledger entries.

### Resolution Steps:
1. **Case 1 — CMOS Battery Reset (Clock jumped backward into the past)**:
   - Instruct the user to fix the Windows system clock (`Settings → Time & language → Date & time → Set time automatically` or manual `UTC+06:00 Dhaka`).
   - Once the Windows clock is restored to the real current date/time (`>= max_recorded_timestamp`), Coachify clears the clock jump lock automatically on the next action.
2. **Case 2 — Accidental Future Date Entry (Someone accidentally set the PC clock to next year, recorded 1 receipt, and then fixed the clock back to today)**:
   - Because `max_recorded_timestamp` is now in the future relative to today's real date, standard writes remain paused.
   - Verify the Owner's identity per the **Mandatory Identity Verification Gate**.
   - Instruct the Owner to open the **Clock Jump Recovery Prompt** in Coachify and execute the **Acknowledge Clock Jump (`acknowledge_clock_jump`)** action using their **Owner Password** and a mandatory audit reason (e.g., `"Corrected accidental Windows date change from 2027 back to 2026-09-25"`).
   - This appends a signed `CLOCK_JUMP_ACKNOWLEDGED` event to the `audit_log` and re-baselines the monotonic watermark to the current verified system time.

---

## Procedure 4: DPAPI Key Rewrapping After Windows User Profile Reset

Coachify encrypts workstation-specific secrets (SMS Gateway API keys, Cloud AI tokens) using Windows DPAPI (`CryptProtectData`) tied to the local Windows user SID.

If a local technician reinstalls Windows or creates a new Windows user profile on the same PC while keeping `coachify.db`:
1. Coachify will start normally and all student, fee, attendance, and exam records will open without error.
2. However, reading the encrypted SMS API key or AI Gateway secret will return a `DPAPI_DECRYPT_FAILED` warning in **Settings → Communication & SMS Setup**.
3. **Resolution**:
   - Have the Owner open **Settings → Communication & SMS Setup**.
   - Re-enter the SMS Gateway API Key / Token and click **Save & Test Connection**.
   - Coachify immediately re-wraps the secret under the new Windows user profile's DPAPI master key (`Encrypted Sensitive Secrets`).

---

## Procedure 5: Thermal Printer Driver Diagnosis & Cash Drawer Kick

### Symptom 1: 80mm Thermal Printer Cuts Receipt Mid-Line or Prints Garbled Characters
1. Open **Windows Settings → Printers & scanners → [Thermal Printer] → Printing preferences**.
2. Verify the driver is using the dedicated manufacturer GDI/POS driver (`XP-80C` / `Rongta RP80`), **not** `Generic / Text Only`.
3. Set **Paper Size** to `80(72.1) x 297 mm` and **Print Density** to `High / Dark` so Bengali conjunct characters (`যুক্তবর্ণ`) print crisply.

### Symptom 2: RJ11 Cash Drawer Does Not Open on Receipt Print
1. Confirm the RJ11 cable from the cash drawer is plugged into the back of the **80mm Thermal Printer (`DK` port)**, not the PC's Ethernet (`LAN`) port.
2. In Windows **Printer Properties → Device Settings**, set **Cash Drawer #1 `#2`** to `Open Before Printing`.
3. In Coachify **Settings → Printer & Receipt Settings**, toggle **Open Cash Drawer on Payment (`open_cash_drawer`)** to **Enabled**.

---

## Procedure 6: Forgotten Staff & Owner Credentials Reset

### Scenario A: Cashier, Accountant, or Teacher Forgot Their Password
1. Any logged-in **Owner** can open **Staff Directory** (`StaffDirectoryView`), select the staff member, and click **Reset Staff Password** without calling support.

### Scenario B: The Owner Forgot the Owner Password
1. Complete the **Mandatory Identity Verification Gate** with the institution owner.
2. Instruct the Owner to click **Forgot Owner Password / Emergency Account Recovery** (`EmergencyRecoveryDialog`) on the Coachify login screen.
3. Have the Owner enter their **24-word Encryption Recovery Key** from their physical **Recovery Sheet** stored in their office safe.
4. Coachify cryptographically verifies the Recovery Key against the local key commitment stored during Setup Wizard initialization, unlocks the Owner account locally, sets a new `Argon2id` password, and logs an `EMERGENCY_OWNER_PASSWORD_RESET` entry in the immutable `audit_log`.

---

## Procedure 7: LAN Host & Multi-PC Workstation Connection Troubleshooting

### Symptom 1: Secondary Workstation Cannot Discover Host PC Automatically
1. Verify both computers are connected to the same Wi-Fi router or office Ethernet switch.
2. In the coaching center's Wi-Fi router management panel, ensure **AP Isolation / Client Isolation / Guest Mode** is **Disabled** (AP isolation blocks devices from talking to one another).
3. On the Host PC, open Coachify **Settings → LAN Host** and note down the local IP address (e.g. `192.168.1.55`).
4. On the Workstation PC, enter the manual IP and port (`50443`) in the Manual IP Connectivity tool and click **Ping**.

### Symptom 2: Workstation Reports "TCP Connection Failed / Timed Out"
1. On the Host PC, verify that the **Windows Network Profile** is set to `Private` (Windows Firewall blocks inbound traffic on `Public` networks by default). Run the remediation command in PowerShell if needed:
   ```powershell
   Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } | Set-NetConnectionProfile -NetworkCategory Private
   ```
2. Confirm the inbound firewall rule exists by running in Command Prompt as Administrator:
   ```cmd
   netsh advfirewall firewall add rule name="Coachify LAN Host" dir=in action=allow protocol=TCP localport=50443 profile=private
   ```
3. Confirm that the Host service badge in Coachify Settings shows **Running**.

### Symptom 3: Workstation Reports "TLS Fingerprint Mismatch"
1. Compare the SHA-256 fingerprint displayed on the Host PC (**Settings → LAN Host**) with the fingerprint on the Workstation.
2. If fingerprints do not match, the workstation may be attempting to connect to a different computer or a stale cache. Re-verify the target IP address.
