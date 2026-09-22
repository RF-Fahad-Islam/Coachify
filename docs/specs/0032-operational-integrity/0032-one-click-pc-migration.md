# 0032-one-click-pc-migration. One Click PC Migration

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 48

**Date**: 2026-09-23

## Summary

This specification establishes the One Click PC Migration engine for transferring a complete Coachify installation to a new computer. The engine packages the SQLite database, document vault files, application configuration, and system templates into a single encrypted portable archive (.coachify-bundle) protected by the owner recovery key phrase. On the target computer, importing the bundle unpacks the contents, verifies SHA256 integrity checksums, restores tables, and rewraps cryptographic keys under the new computer Windows Data Protection API (DPAPI) context.

## Requirements

**User stories**:
* As a coaching center owner upgrading to a faster office PC, I want to export our entire Coachify setup to a USB drive with one click, so that I can move to the new machine without hiring an IT specialist.
* As an administrator importing onto a new machine, I want the system to re encrypt our master keys using the new machine Windows security context, so that our credentials remain protected without manual key commands.

**Acceptance criteria**:
* **AC-1**: RPC `integrity.export_migration_bundle` packages the SQLite database, uploaded student documents, report templates, and settings into an AES 256 GCM encrypted archive with a companion SHA256 checksum manifest.
* **AC-2**: The migration bundle encryption key is derived from the owner recovery phrase using Argon2id with a fresh random salt.
* **AC-3**: RPC `integrity.import_migration_bundle` accepts the bundle file path and owner recovery phrase, validates archive checksums, and restores the database tables.
* **AC-4**: During import, all DPAPI encrypted credentials (including database encryption keys and SMS provider secrets) are decrypted using the recovery key and rewrapped under the new computer Windows DPAPI context.
* **AC-5**: The migration event records status, timestamps, and target machine hints in the `pc_migrations` table.

## Decision

**Chosen option**: Recovery Key Derived Portable Archive with Windows DPAPI Rewrapping

Because Windows DPAPI keys are tied to a specific Windows user profile and machine SID, raw database files cannot simply be copied to a new PC without breaking encrypted credential access. The migration export decrypts sensitive secrets into an ephemeral payload encrypted under the owner recovery phrase. The target machine decrypts with the recovery phrase and calls Windows `CryptProtectData` to establish the new machine local DPAPI envelope.

## Rationale

Coaching centers replace hardware every few years or swap computers when machines break down. Without a seamless migration tool, centers risk losing document vaults, historical settings, or getting locked out of encrypted databases. Deriving the bundle encryption from the owner recovery phrase ensures that even if a USB drive is lost in transit, customer financial records and PII remain completely secure.

## Feature design

**Data model sketch**:
* `pc_migrations`: Table created in Migration 0017 with columns `id TEXT PRIMARY KEY`, `package_filename TEXT`, `target_machine_hint TEXT`, `exported_by_user_id TEXT REFERENCES users(id)`, `exported_at TEXT`, `imported_at TEXT`, `checksum_sha256 TEXT`, `rewrap_salt TEXT`, `status TEXT CHECK (status IN ('exported', 'verified', 'imported', 'failed'))`.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.export_migration_bundle` | Mutation | target_hint: Option<String> | MigrationExportResult | `system.manage` or Owner | 500 export error |
| `integrity.import_migration_bundle` | Mutation | bundle_path: String, recovery_phrase: String | MigrationImportResult | Owner | 401 invalid key, 409 checksum mismatch |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Export package | .coachify-bundle file and checksum | SQLite online backup plus vault files zipped and encrypted with Argon2id derived key |
| Import package | Restored database and new secrets | Unpacked archive decrypted with recovery key and re encrypted via local DPAPI |

**Key invariants**:
* An import attempt with an invalid recovery phrase must fail immediately without altering existing local files.
* Checksum mismatches on the bundle payload immediately halt the import process.

**Security model**:
* Exporting or importing migration bundles is strictly restricted to Administrator and Owner roles.
* Importing requires explicit entry of the owner 24 word recovery phrase.

**Critical test scenarios**:
* Clean migration export: Generates export package with test database and documents, verifies SHA256 manifest exists, verifies AC-1 and AC-2.
* Invalid key rejection: Attempts import with incorrect recovery phrase, asserts decryption error and zero file modification, verifies AC-3.
* Round trip import and rewrap: Imports valid bundle into new clean directory, verifies database tables readable, verifies credentials rewrapped under current machine DPAPI, verifies AC-3, AC-4, AC-5.

## Build plan

1. Create `pc_migrations` table in Migration 0017, satisfies **AC-5**.
2. Implement bundle packager with AES 256 GCM and Argon2id key derivation in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Implement bundle unpacker and DPAPI key rewrapper in `crates/core/src/integrity.rs`, satisfies **AC-3**, **AC-4**.
4. Create PC Migration Wizard dialog in `ui/src/components/setup/pc-migration-wizard.tsx`, satisfies **AC-1**, **AC-3**.

## Consequences

**Positive**:
* Turns hardware upgrades and PC replacements into a stress free 5 minute procedure.
* Guarantees zero credential lockout through automatic DPAPI rewrapping.

**Negative / tradeoffs**:
* Requires the owner to have their original recovery phrase available during machine setup.
