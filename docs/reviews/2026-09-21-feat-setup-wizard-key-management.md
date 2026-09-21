# Review, feat/setup-wizard-key-management, 2026-09-21

**Reviewed by**: Gemini 3.8 Flash (High) (fresh model review)
**Scope**: Core crypto (`crypto.rs`), storage topology (`storage.rs`), setup & recovery workflows (`setup.rs`), command dispatcher handlers, React Setup Wizard (`ui/src/components/setup/`), and reprint dialog (`ui/src/components/settings/`)
**Verdict**: Approved with fixes

## Summary
Coachify implements an offline-first setup wizard and two-tier cryptographic key envelope architecture designed specifically for the operational conditions of coaching centers in Bangladesh (shared Windows PCs, frequent hardware mortality, local technician `C:\` re-imaging, and zero vendor key escrow).

The implementation satisfies Spec 0007 across all 10 acceptance criteria (AC-1 through AC-10):
- **AC-1**: Cryptographically secure 256-bit CSPRNG master database key wrapped with dual envelopes: Windows DPAPI machine scope (`CRYPTPROTECT_LOCAL_MACHINE`) for daily shared-account use, and Argon2id (m=65536, t=3, p=4) + AES-256-GCM for the Owner Recovery Key.
- **AC-2**: Key envelope file (`key.envelope`) validation with SQLite decryption barrier against unauthorized database file copying.
- **AC-3**: Native Windows drive enumeration (`GetLogicalDriveStringsW`, `GetDriveTypeW`, `GetDiskFreeSpaceExW`), recommending secondary fixed volumes (`D:\Coachify Data`) with explicit warnings regarding `C:\` technician wipe risks.
- **AC-4**: Data folder layout scaffolding under `<data_root>/<institution_id>/` with `db/`, `vault/`, `snapshots/`, `logs/`, and `staging/` and system-wide configuration persistence (`%ProgramData%\Coachify\config.json`).
- **AC-5**: External backup destination write testing and `.coachify-backup-target` marker file binding.
- **AC-6**: Crockford Base32 formatting with CRC-16 polynomial checksum into 4-character chunks (`CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`) and printable bilingual Recovery Sheet with QR code.
- **AC-7**: Mandatory confirmation gate requiring exact last-4-character verification before completing setup.
- **AC-8**: Single-transaction institutional bootstrap seeding `institution`, academic session `2026`, student & receipt sequences, Owner user account with Argon2id hash, 30-day offline trial license, clock guard high-water mark, and genesis audit record (`seq = 1`, `action = 'system.bootstrap'`).
- **AC-9**: Disaster recovery pipeline (`recovery.inspect_database`, `recovery.unlock_with_key`) supporting foreign machine DPAPI re-wrapping and Owner credential recovery.
- **AC-10**: Owner Recovery Sheet reprint dialog with password confirmation and audit verification.

During review, one minor finding and one robustness enhancement were identified and immediately remediated with regression test coverage:
1. `recovery.reprint_sheet` was registered with `kind: CommandKind::Read` and returned an empty audit entry vector, omitting the tamper-evident audit log row promised by the UI and security model.
2. `recovery.reprint_sheet` resolved database and envelope paths exclusively via `db::resolve_dev_db_path()`, which could mispoint when custom non-system data directories are configured.

## Findings

### 🟡 `recovery.reprint_sheet` omitted audit trail despite UI security promise (`crates/core/src/dispatcher.rs:208`)
- **Problem**: `reprint-recovery-dialog.tsx` explicitly warns: *"Only the Institute Owner may reprint the physical recovery sheet. This action is recorded permanently in the tamper-evident audit log."* However, `recovery.reprint_sheet` was classified as `CommandKind::Read` in `COMMANDS` metadata, and its handler returned `vec![]` for audit entries.
- **Why it matters**: In an educational center with multiple supervisors or during administrative disputes, reprinting a physical disaster recovery document is a high-security event that must leave a cryptographic, hash-chained audit record.
- **Fix applied**: Updated `recovery.reprint_sheet` to `CommandKind::Mutation` so it executes inside a transaction, generates a structured `AuditEntry` (`action: "recovery.reprint_sheet"`, entity: `"institution"`, payload containing owner username and key checksum), and records it permanently in `audit_log`. Added regression assertion in `crates/core/tests/setup_and_keys.rs`.

### 🟡 Path resolution in `recovery.reprint_sheet` did not check global config (`crates/core/src/dispatcher.rs:1804`)
- **Problem**: The handler resolved `envelope_path` by calling `db::resolve_dev_db_path()` directly, which defaults to `dev.db` if `main.db` does not exist at the exact dev path, rather than reading `%ProgramData%\Coachify\config.json` via `storage::load_global_config()`.
- **Why it matters**: If Coachify is configured on a secondary drive (e.g. `D:\Coachify Data\inst_01\db\key.envelope`), `reprint_sheet` could fail to locate `key.envelope` and display `"N/A"` for the checksum.
- **Fix applied**: Updated path resolution to inspect `storage::load_global_config()` first (matching `setup::inspect_database` and `setup::unlock_database_with_key`), falling back gracefully to the development path.

## Nits
- ⚪ `crates/core/src/crypto.rs:257`: Non-Windows mock DPAPI wrapper prepends `MOCK_DPAPI_WRAP:`. This cleanly enables cross-platform testing in CI and development while preserving strict DPAPI calls on Windows production builds.
- ⚪ `crates/core/src/setup.rs:387`: Initial trial license seeds valid until the end of the year (`{year}-01-01T00:00:00Z`). In Slice 7 (Commercial), this will be replaced with dynamic duration calculations based on official vendor license tokens.

## Strengths
- **Two-Tier Envelope Architecture**: Perfectly balances shared-account day-to-day usability with machine-binding security. Staff can launch Coachify without knowing the master password, but copying the files to another PC renders them useless without the physical Recovery Sheet.
- **Defensive Storage Engineering**: Proactive detection of non-system drives (`GetLogicalDriveStringsW`) and recommending `D:\Coachify Data` guards institutions against the single most common cause of small-office data loss in Bangladesh (technician OS reinstallation).
- **Offline Autonomy**: Zero reliance on vendor cloud servers, external licensing servers, or internet access during setup.
- **Transcription-Safe Key Format**: Crockford Base32 alphabet avoids ambiguous characters (`0/O`, `1/I/L`), while the CRC-16 polynomial checksum detects transcription typos immediately before running expensive Argon2id KDF operations.
- **Atomic Bootstrap**: Institution metadata, academic sessions, receipt and student sequences, Owner user, offline trial license, and genesis audit record (`seq = 1`) commit together inside a single transaction.

## Test Coverage
- `crates/core/tests/setup_and_keys.rs`:
  - `test_setup_status_and_drive_detection`: Verifies setup status detection, drive enumeration, non-system drive flagging, and directory path validation.
  - `test_recovery_preview_and_confirmation_gate`: Verifies preview key generation, Crockford Base32 formatting, checksum extraction, and rejection of mismatched confirmation codes.
  - `test_full_setup_and_disaster_recovery_workflow`: Verifies end-to-end institutional bootstrap, key envelope persistence, backup marker creation, database inspection on healthy machines, Recovery Sheet reprint with password confirmation, audit entry generation, foreign machine DPAPI failure simulation, recovery key unlocking, Owner password reset, and subsequent login.
- `crates/core/src/crypto.rs` (unit tests):
  - `test_base64_roundtrip`, `test_generate_master_key_randomness`, `test_recovery_key_format_and_verification`, `test_two_tier_envelope_create_and_unwrap_dpapi`, `test_envelope_file_save_and_load`.
- `crates/core/src/storage.rs` (unit tests):
  - `test_detect_logical_drives_returns_entries`, `test_validate_data_folder_happy_path`, `test_scaffold_institution_storage_creates_all_subdirs`, `test_backup_marker_write_and_validation`.
- `ui/src/components/setup/setup.test.tsx` (Vitest UI tests):
  - Renders `RecoverySheetPrintable` with key chunks, checksum, bilingual warnings, and print layout.
  - Renders `SetupWizard` starting at Step 1 (Institution Profile).
  - Renders `EmergencyRecoveryDialog` and `ReprintRecoveryDialog`.

## Fixes Applied (2026-09-21)
- **Minor (Audit Trail on Recovery Sheet Reprint)**: Configured `recovery.reprint_sheet` as `CommandKind::Mutation` in `crates/core/src/dispatcher.rs` and emitted an `AuditEntry` recording the reprint event into `audit_log`. Verified by `test_full_setup_and_disaster_recovery_workflow`.
- **Minor (Global Config Path Resolution)**: Updated `recovery.reprint_sheet` to resolve `key.envelope` using `storage::load_global_config()` when present.
