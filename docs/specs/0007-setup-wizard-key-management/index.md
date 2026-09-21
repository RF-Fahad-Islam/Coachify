# 0007. Setup wizard and key management

**Date**: 2026-09-21
**Status**: Accepted

## Summary

Coachify delivers a seamless, offline-first initial setup experience that protects educational records against hardware theft and catastrophic machine failure. On first launch, the Setup Wizard guides the center director through institution profiling, automated non-system drive detection (recommending storage on secondary drives like `D:\` to survive technician `C:\` re-imaging), Owner account creation, backup target initialization, and 30-day offline trial licensing. 

To eliminate data loss when office PCs inevitably fail while guarding student and financial records against unauthorized file duplication, Coachify implements a two-tier key envelope architecture. The SQLite database is encrypted with a cryptographically secure 256-bit master key. For day-to-day operation across shared Windows accounts, this key is wrapped using Windows Data Protection API (DPAPI) in machine scope (`CRYPTPROTECT_LOCAL_MACHINE`). For disaster recovery, hardware replacement, and emergency Owner password resets, the master key is wrapped using an Owner Recovery Key via Argon2id and AES-256-GCM. The Setup Wizard renders a dedicated printable Recovery Sheet with a scannable QR code and enforces a strict confirmation gate requiring the Owner to verify the last four characters of the key before entering the dashboard.

## Requirements

**User stories**:
- As a coaching center director, I want a step-by-step setup wizard on first launch so that I can configure my institute profile, academic year, and administrative credentials without technical assistance.
- As a coaching center director, I want the application to recommend storing my database on a secondary drive (like `D:\`) so that routine Windows reinstallation by local technicians does not erase my records.
- As an institute owner, I want my database protected by strong encryption so that if someone copies the database file to a USB stick, they cannot read student records, fees, or exam marks.
- As an institute owner, I want a printed Recovery Sheet with a clear recovery key and QR code so that if the computer motherboard or operating system crashes, I can restore my data on a replacement PC.
- As an institute owner, I want to reset a forgotten Owner account password using my Recovery Sheet so that I never lose administrative control of my institute.
- As a system administrator, I want backups to target a validated drive marked with an institution identifier so that automated backups are never written to an incorrect USB drive.

**Acceptance criteria**:
- **AC-1**: During initial institution creation, Coachify generates a cryptographically secure 256-bit random master database key. The key is persisted in a structured key envelope file (`key.envelope`) containing two independent wrapping layers:
  1. Wrapped using Windows DPAPI machine scope (`CRYPTPROTECT_LOCAL_MACHINE`) for daily use across shared local Windows accounts.
  2. Wrapped using an Owner Recovery Key via Argon2id key derivation and authenticated AES-256-GCM encryption.
- **AC-2**: The local SQLite database file is encrypted using SQLCipher with the 256-bit master key. If the database file is copied to another machine or accessed without the unwrapped master key, SQLite rejects access with an authentication/encryption failure.
- **AC-3**: The Setup Wizard detects all mounted logical volumes using Windows storage APIs (`GetLogicalDriveStringsW` / `GetDriveTypeW`). If a non-system fixed drive (such as `D:\` or `E:\`) is available, the wizard recommends creating the data root there (e.g. `D:\Coachify Data`) with an explicit notice that technician reformatting of `C:\` will erase records stored on the system drive.
- **AC-4**: The chosen data directory is initialized with an isolated directory layout under `<data_root>/<institution_id>/` containing `db/`, `vault/`, `snapshots/`, `logs/`, and `staging/`. A system-wide configuration file `%ProgramData%\Coachify\config.json` (with a local fallback in `%LOCALAPPDATA%\Coachify\config.json`) records `data_dir`, `institution_id`, and `app_mode` without storing secrets.
- **AC-5**: The wizard verifies write permissions on the selected backup target (external drive, secondary volume, or network share) and writes a marker file `.coachify-backup-target` containing the `institution_id` and initialization timestamp, preventing automated backups from writing to mismatched storage media.
- **AC-6**: The wizard formats the Owner Recovery Key into 4-character monospace alphanumeric chunks with an embedded checksum (e.g. `CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`) and renders a dedicated printable Recovery Sheet containing clear bilingual instructions, emergency guidelines, and a high-density QR code.
- **AC-7**: The wizard enforces a mandatory confirmation gate: the "Complete Setup" action remains disabled until the Owner types the last 4 characters of the Recovery Key into a verification field.
- **AC-8**: Upon confirmation, Coachify executes an atomic bootstrap transaction that saves institution metadata, seeds the initial academic session "2026", creates system roles and permissions, creates the primary `Owner` user account with an Argon2id password hash, sets up the receipt sequence `receipt:<year>`, seeds an offline 30-day trial license in `license_state`, writes a genesis audit log row (`action = 'system.bootstrap'`), and issues an authenticated session token.
- **AC-9**: If a database is moved to a new PC where DPAPI unwrapping fails, Coachify enters Emergency Recovery Mode. Entering the valid Owner Recovery Key unwraps the master key, re-wraps it with the new machine's DPAPI, and provides an authenticated option to reset the Owner password.
- **AC-10**: An authenticated user with the `Owner` role can re-view and re-print the Recovery Sheet from `Settings → Security & Key Management` after verifying their Owner password.

## Decision

**Chosen option**: Option 1: Two-Tier Key Envelope with Machine-Scope DPAPI Wrapping, Argon2id/AES-GCM Recovery Key Wrapping, Drive Recommendation, and Mandatory Printed Recovery Sheet.

You build the Coachify setup wizard and key management subsystem across `crates/core/src/crypto.rs` (key generation, DPAPI wrapping, AES-GCM envelope), `crates/core/src/storage.rs` (drive scanning, data topology, backup marker), `crates/core/src/setup.rs` (atomic bootstrap transaction and recovery workflows), and `ui/src/components/setup/` (multi-step React setup wizard, drive picker, printable Recovery Sheet with QR code, and confirmation gate).

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `react-patterns` (`.claude/skills/react-patterns/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Two-Tier Key Envelope Specification

The 256-bit database master key is never persisted in plaintext. It is encapsulated within `<data_root>/<institution_id>/db/key.envelope`:

```json
{
  "version": 1,
  "institution_id": "018e6e5a-8b14-7d92-936d-556942ad0001",
  "created_at": "2026-09-21T00:00:00Z",
  "dpapi": {
    "scope": "local_machine",
    "wrapped_key": "base64-encoded-dpapi-ciphertext"
  },
  "recovery": {
    "kdf": "argon2id",
    "argon2_params": {
      "m_cost": 65536,
      "t_cost": 3,
      "p_cost": 4
    },
    "salt": "base64-16-bytes-salt",
    "nonce": "base64-12-bytes-nonce",
    "ciphertext": "base64-aes256gcm-ciphertext-and-tag",
    "checksum": "a1b2c3d4"
  }
}
```

### Cryptographic Workflow

```
[Setup Wizard]
      │
      ├─► 1. Generate 256-bit CSPRNG Master Key: K_db
      │
      ├─► 2. Daily Use Path:
      │       K_db ──► CryptProtectData(CRYPTPROTECT_LOCAL_MACHINE) ──► dpapi.wrapped_key
      │
      └─► 3. Disaster Recovery Path:
              Generate Recovery Key: "CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"
              Recovery Key + Salt ──► Argon2id KDF ──► K_rec (256-bit)
              K_db ──► AES-256-GCM Encrypt (K_rec, Nonce) ──► recovery.ciphertext
```

### Recovery Key Representation

The recovery key is formatted for high visual clarity, physical transcription accuracy, and offline resilience:
- Format: `CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX` (24 uppercase alphanumeric characters divided into 6 distinct 4-character blocks).
- Character set: Crockford's Base32 alphabet (`0-9`, `A-Z` excluding `I`, `L`, `O`, `U` to prevent reading ambiguities).
- Checksum: The final block incorporates a CRC-16 / polynomial checksum of the preceding blocks to instantly catch transcription errors before executing expensive KDF iterations.

### Drive Recommendation Logic

```
Logical Drives Enumeration (GetLogicalDriveStringsW)
      │
      ▼
Filter Fixed Drives (DRIVE_FIXED)
      │
      ├─► Has non-system drive (e.g. D:\, E:\)?
      │       ├─► YES: Recommend D:\Coachify Data (is_recommended = true)
      │       │        Prompt: "Store data on D: to survive Windows re-installation."
      │       └─► NO:  Default to %ProgramData%\Coachify\data (is_recommended = false)
      │                Warn: "Only system drive detected. USB backups strongly advised."
      ▼
Verify Write Permissions & Free Space (> 500 MB)
```

### API Surface (RPC Commands)

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `setup.get_status` | Query | None | `needs_setup: bool`, `app_version: str`, `system_drive: str` | Public | None |
| `setup.detect_drives` | Query | None | `drives: list[DriveInfo]` (letter, label, total_bytes, free_bytes, is_recommended, is_system) | Public | `io_error` |
| `setup.validate_path` | Query | `path: str`, `kind: "data" \| "backup"` | `valid: bool`, `free_bytes: u64`, `normalized_path: str` | Public | `path_invalid`, `permission_denied`, `disk_full` |
| `setup.initialize_institution` | Mutation | Full bootstrap payload (profile, session, data_dir, backup_dest, owner, recovery_confirm) | `session_token: str`, `user: UserSummary`, `recovery_key: str`, `institution_id: str` | Public (Setup only) | `already_initialized`, `invalid_recovery_confirm`, `validation_error` |
| `recovery.inspect_database` | Query | `db_path?: str` | `is_encrypted: bool`, `institution_name?: str`, `schema_version?: u32`, `needs_recovery: bool` | Public | `file_not_found`, `corrupted_file` |
| `recovery.unlock_with_key` | Mutation | `recovery_key: str`, `reset_owner_password?: str` | `success: bool`, `message: str`, `institution_name: str` | Public | `invalid_recovery_key`, `checksum_mismatch`, `decryption_failed` |
| `recovery.reprint_sheet` | Query | `owner_password: str` | `institution_name: str`, `owner_username: str`, `recovery_key: str`, `created_at: str` | Owner only (`settings.view`) | `invalid_password`, `forbidden` |

### Value Sourcing

| Action | Value produced or displayed | Source |
|---|---|---|
| `setup.detect_drives` | Drive list and space metrics | Windows `GetLogicalDriveStringsW`, `GetDriveTypeW`, and `GetDiskFreeSpaceExW` |
| `setup.detect_drives` | `is_recommended` | Evaluated logic: drive is `DRIVE_FIXED` and not matching `%SystemDrive%` |
| `setup.initialize_institution` | `institution_id` | Cryptographic UUIDv7 generated via `uuid::Uuid::now_v7()` |
| `setup.initialize_institution` | Master database key | 32 bytes from OS cryptographically secure pseudo-random number generator (`rand::rngs::OsRng`) |
| `setup.initialize_institution` | Recovery key string | 20 random bytes + 2 byte checksum encoded into Crockford Base32 blocks |
| `setup.initialize_institution` | `session_token` | 32-byte secure random token hashed with SHA-256 in session table |
| `setup.initialize_institution` | Initial receipt sequence | Number 1 formatted with configured prefix (e.g. `REC-2026-0001`) |
| `setup.initialize_institution` | Initial trial license | Built-in offline trial: valid from `now` to `now + 30 days` |

### Key Invariants

1. **Never Plaintext**: The master database encryption key is never written to disk or logs unencrypted.
2. **Setup Exclusivity**: `setup.initialize_institution` can succeed only once per installation. Subsequent calls fail immediately with `already_initialized`.
3. **Mandatory Sheet Verification**: Initial setup cannot complete without matching the exact last 4 characters of the generated Recovery Key.
4. **Offline Autonomy**: Initial setup, key wrapping, database encryption, and trial licensing require zero internet connectivity.
5. **Machine Binding**: A copied database and `key.envelope` cannot be decrypted by DPAPI on another computer without entering the Owner Recovery Key.
6. **Last Active Owner Invariant**: Owner account created during setup is seeded with the immutable `Owner` system role.
7. **Single Writer Transaction**: Bootstrap operations occur inside an exclusive transaction; failure purges partially created files cleanly.

### Security Model

- **Threat: Physical Drive Theft or Backup Copying**: An unauthorized party stealing a database backup or copy of the files cannot read SQLCipher records without the Windows DPAPI machine secret or the Owner Recovery Key.
- **Threat: Shared Windows Account Access**: Non-technical staff sharing a single Windows login can run Coachify without knowing the master database password, but cannot access plaintext keys or view the Recovery Sheet without the Owner's password.
- **Threat: Lost Machine / Dead Hardware**: The printed Recovery Sheet allows complete data restoration on a brand-new PC without vendor involvement.
- **Threat: Forgotten Owner Password**: The physical Recovery Sheet allows an authorized director to reset their administrative password locally.

### Critical Test Scenarios

- **Setup Happy Path**: Fresh install launches setup wizard, detects secondary drive `D:\`, creates institute, prints Recovery Sheet, verifies last 4 characters, and enters dashboard with active Owner session (verifies AC-1, AC-3, AC-4, AC-6, AC-7, AC-8).
- **Drive Recommendation**: System with drives `C:\` and `D:\` recommends `D:\Coachify Data`. System with only `C:\` defaults to `%ProgramData%` and displays external backup guidance (verifies AC-3).
- **Encryption Portability Barrier**: A database and envelope initialized on Machine A cannot be opened on Machine B via DPAPI, returning an explicit recovery requirement (verifies AC-1, AC-2).
- **Emergency Disaster Recovery**: Entering the valid Owner Recovery Key on Machine B decrypts the database, re-wraps with Machine B's DPAPI, and successfully unlocks records (verifies AC-1, AC-9).
- **Owner Password Reset**: Unlocking an existing database with the Recovery Key and specifying a new password updates the Owner's Argon2id password hash and permits login (verifies AC-9).
- **Confirmation Gate Rejection**: Supplying invalid characters for the Recovery Key confirmation prevents wizard progression (verifies AC-7).
- **Backup Target Marker**: Specifying a backup destination writes `.coachify-backup-target`; selecting a read-only or invalid path surfaces actionable errors (verifies AC-5).
- **Recovery Sheet Reprint**: Authenticated Owner entering valid password views Recovery Sheet; incorrect password is rejected (verifies AC-10).

## Build plan

Applying the Tracer Bullet methodology, we construct the foundation key envelope and drive scanner first, integrate the core initialization and recovery pipelines, and thicken with the multi-step React Setup Wizard:

1. **Milestone 1**: Key envelope engine, Windows DPAPI machine-scope wrapping, Argon2id/AES-256-GCM recovery key wrapping, and Crockford Base32 chunk formatting with checksum in `crates/core/src/crypto.rs`, satisfies **AC-1**, **AC-2**, **AC-6**.
2. **Milestone 2**: Storage topology manager, drive enumeration (`GetLogicalDriveStringsW`), non-system drive recommendation, directory structure scaffolding, and backup destination marker validation in `crates/core/src/storage.rs`, satisfies **AC-3**, **AC-4**, **AC-5**.
3. **Milestone 3**: Setup dispatcher pipeline (`setup.get_status`, `setup.detect_drives`, `setup.validate_path`, `setup.initialize_institution`), atomic institutional bootstrap, trial license seeding, and database key unlocking in `crates/core/src/setup.rs` and `crates/core/src/dispatcher.rs`, satisfies **AC-4**, **AC-7**, **AC-8**.
4. **Milestone 4**: Disaster recovery and emergency unlock engine (`recovery.inspect_database`, `recovery.unlock_with_key`, `recovery.reprint_sheet`) with Owner password reset capabilities, satisfies **AC-9**, **AC-10**.
5. **Milestone 5**: Full React Setup Wizard UI (`ui/src/components/setup/`) with multi-step wizard, drive selection cards, printable Recovery Sheet with QR code, last-4-character confirmation gate, and settings reprint view, satisfies **AC-3**, **AC-6**, **AC-7**, **AC-10**.

## Consequences

**Positive**:
- Eliminates the primary cause of small-office data loss: lost database files during Windows reformatting by technicians.
- Guarantees complete database confidentiality if computers or backups are stolen.
- Provides bulletproof disaster recovery on new PCs without vendor reliance or cloud escrow risk.
- Protects institute owners from lockout via physical Recovery Sheet credential reset.

**Negative / tradeoffs**:
- If an owner loses both the computer hardware and the printed physical Recovery Sheet, data is permanently unrecoverable. (This is an intentional cryptographic tradeoff).
- Initial setup requires approximately 2 minutes to complete drive selection and physical sheet verification.

**Neutral**:
- Requires Windows DPAPI machine-scope APIs on production builds, with mockable abstraction layers for headless CI/tests.

## Follow-up

- [ ] Connect the WebView2 native silent print pipeline to the Recovery Sheet "Print Now" action.
- [ ] Connect automated backup routines in Slice 3 to verify the `.coachify-backup-target` marker file before executing backups.
