# Verify: Setup wizard and key management (spec 0007, updated 2026-09-21)

Steps derived from spec 0007 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] On a fresh installation without existing configuration, launch Coachify and verify that the application presents the Setup Wizard welcome screen rather than the standard login dialog (verifies AC-1, AC-8)
- [ ] In the Storage Location step, observe detected drives and confirm that if a non-system fixed drive (e.g. `D:\`) exists, the wizard highlights it as recommended (`D:\Coachify Data`) and displays the technician reformatting warning (verifies AC-3)
- [ ] Attempt to proceed past the Recovery Sheet step without entering the verification code or with incorrect characters, and verify that the "Complete Setup" action is strictly disabled (verifies AC-7)
- [ ] On the Recovery Sheet step, click "Print Recovery Sheet" / "Save as PDF" and verify that a clean printable document is rendered containing institution name, creation timestamp, QR code, and formatted recovery key chunks (verifies AC-6)
- [ ] Type the exact last 4 characters of the Recovery Key into the confirmation field, submit the wizard, and confirm that the application bootstraps the institution, generates keys, seeds the 30-day offline trial license, and navigates directly to the authenticated Dashboard (verifies AC-1, AC-4, AC-8)
- [ ] Copy the generated database file and `key.envelope` to a separate machine (or run under an isolated Windows user profile) and verify that startup detects the foreign environment, blocks database access, and displays the Emergency Recovery Screen (verifies AC-2, AC-9)
- [ ] On the Emergency Recovery Screen, enter the valid Owner Recovery Key and confirm that the database is successfully unlocked, re-wrapped with the current machine's DPAPI, and boots to the login screen (verifies AC-9)
- [ ] On the Emergency Recovery Screen, enter the valid Owner Recovery Key along with a new Owner password, and verify that the Owner password is reset, allowing immediate login with the new password (verifies AC-9)
- [ ] Log in as Owner, navigate to `Settings → Security & Key Management`, click "Reprint Recovery Sheet", enter the Owner password, and verify that the original Recovery Sheet is accurately displayed with its original key and QR code (verifies AC-10)

## Commands

- [ ] `cargo test -p coachify_core --test setup_and_keys` (runs comprehensive core integration tests verifying 256-bit CSPRNG key generation, DPAPI machine-scope wrapping and unwrapping, Argon2id/AES-256-GCM recovery key envelope serialization, Crockford Base32 formatting and CRC checksum validation, logical drive scanning and non-system drive prioritization, backup target marker file verification, atomic institutional bootstrap, trial license seeding, foreign machine rejection, and emergency recovery password reset, verifies AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, AC-10)
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings match the Rust core contract for `setup.get_status`, `setup.detect_drives`, `setup.validate_path`, `setup.initialize_institution`, `recovery.inspect_database`, `recovery.unlock_with_key`, and `recovery.reprint_sheet`, verifies AC-8, AC-9, AC-10)
- [ ] `cargo run -p xtask check-migrations` (verifies that released migrations in `released.json` remain completely immutable, verifies AC-8)
- [ ] `pnpm test` (runs UI unit tests for the multi-step Setup Wizard, drive recommendation selector, Recovery Sheet printable component, and last-4-character confirmation gate, verifies AC-3, AC-6, AC-7)

## Acceptance criteria coverage

- AC-1 (Two-Tier Key Envelope with DPAPI and Argon2id/AES-GCM): verified by core unit and integration tests asserting both wrapping layers in `key.envelope` and validating successful unwrapping through both paths.
- AC-2 (Database Encryption Enforcement): verified by tests asserting that opening the SQLCipher database file without the master key fails with an explicit encryption error.
- AC-3 (Non-System Drive Scanning & Recommendation): verified by tests asserting that `setup.detect_drives` identifies fixed drives and marks non-system volumes with `is_recommended = true`.
- AC-4 (Data Folder Topology & Isolation): verified by filesystem tests confirming the creation of `db/`, `vault/`, `snapshots/`, `logs/`, and `staging/` directories under `<data_root>/<institution_id>/` and configuration persistence in `%ProgramData%`.
- AC-5 (Backup Destination & Marker Validation): verified by tests asserting the creation and validation of `.coachify-backup-target` on the target volume.
- AC-6 (Printable Recovery Sheet & Monospace Chunks): verified by UI tests asserting proper 4-character chunk formatting, Crockford Base32 alphabet enforcement, and QR code generation.
- AC-7 (Mandatory Confirmation Gate): verified by UI component tests asserting button disabled state until the last 4 characters match, and backend RPC validation rejecting mismatched confirmation values.
- AC-8 (Offline Institutional Bootstrap & Trial Seeding): verified by integration tests asserting single-transaction creation of institution, academic session, Owner account, receipt sequence, 30-day offline trial license, and genesis audit row.
- AC-9 (Disaster Recovery & Password Reset): verified by simulated foreign machine tests asserting database recovery and Owner credential reset using the Recovery Key.
- AC-10 (Owner Recovery Sheet Reprint): verified by permission-checked RPC tests asserting that authenticated Owners can retrieve and reprint the Recovery Sheet using their password.
