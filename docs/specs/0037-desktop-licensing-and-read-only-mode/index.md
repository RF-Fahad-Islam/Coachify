# 0037. Desktop Licensing and Read Only Mode

**Date**: 2026-09-25
**Status**: Accepted

## Summary

Coachify introduces an offline first licensing engine that validates cryptographically signed license tokens without ever contacting a remote server during desk operations. The system binds licenses to the local host computer, guards against clock rollback, and grants a 14 day grace window before gracefully entering read only mode. In read only mode, staff and owners can still log in, search records, print reports, export data, and create backups, while write actions are securely blocked at the central dispatcher. If an institution restores a backup onto a replacement computer, a 14 day transitional window ensures that daily student admissions and fee collections continue uninterrupted.

## Requirements

**User stories**:
* As an institution owner, I want to activate my Coachify desktop installation by importing a signed license file or scanning an offline request code, so that my coaching center runs securely without requiring an internet connection.
* As a counter cashier or receptionist, I want the system to remain fully operational during temporary payment delays or internet outages, so that students can pay fees without desk stoppages.
* As an institution owner whose subscription has expired past the grace period, I want full read access to all past records, receipts, and backups, so that Coachify never holds my student or financial data hostage.
* As a support technician helping a coaching center recover from a dead computer on an exam morning, I want restored backups to provide a 14 day transitional window on the new PC, so that emergency migrations never halt critical operations.
* As the product vendor, I want tampered, expired, or swapped older license files to be rejected with clear diagnostic explanations, so that commercial agreements are respected while legitimate centers face zero friction.

**Acceptance criteria**:
* **AC-1**: Cryptographic signature verification: The core library validates Ed25519 signatures on imported `.coachifylicense` files against embedded public keys for recognized key identifiers (`k2026a`, `k2026b`), rejecting tampered or invalid signatures with error code `license_signature_invalid`.
* **AC-2**: Installation hardware binding: The verification engine confirms that the license payload `installation_id` matches the machine installation identifier stored at `%ProgramData%\Coachify\installation_id` and reflected on the local host device (`devices.id` where `is_host = 1`), rejecting foreign licenses with error code `license_installation_mismatch`.
* **AC-3**: Monotonic serial progression: Newly imported licenses must have a serial number equal to or greater than the currently active license serial, rejecting older or downgraded licenses with error code `license_serial_downgrade`.
* **AC-4**: Automatic evaluation trial: Setup wizard automatically generates and activates a 14 day full access evaluation trial bound to the host installation identifier with sentinel signature `LOCAL_TRIAL` and `key_id = 'trial'`, valid only for 14 days from initial institution creation (`institution.created_at`).
* **AC-5**: Expiration warnings and 14 day grace: A 14 day grace period extends past the `paid_through` date before read only restrictions apply; the user interface presents non blocking warnings starting 30 days prior to expiration and persistent reminders during grace.
* **AC-6**: Dispatcher read only enforcement: When license status enters `expired_past_grace`, the central command dispatcher rejects any command gated by `LicenseGate::Write`, `LicenseGate::Sms`, or `LicenseGate::Lan` with error code `license_expired_read_only`, while public and read commands (including searches, document reprints, reports, data exports, backups, and restores) remain permitted.
* **AC-7**: Moved license disaster recovery window: Restoring a database backup on a replacement computer with a different installation identifier grants a 14 day transitional full access window marked `moved_transitional`, tracking `moved_first_detected_at` to prevent repeated restore resets, and displaying an "Activate this PC" banner and request code rather than locking the counter immediately.
* **AC-8**: Deterministic offline request code: The core generates a compact 20 character Base32 request code (formatted as `REQ-XXXX-XXXX-XXXX-XXXX`) and QR payload (`COACHIFY-REQ\|v1\|<installation_id>\|<checksum>`) derived via HMAC SHA256 of the machine installation identifier.
* **AC-9**: Offline license file import: An authenticated Owner or Admin can import a signed `.coachifylicense` file through `license.import_file` RPC; upon successful validation, the new license is activated in the database, logged in the audit trail, and instantly refreshes client state.
* **AC-10**: Clock rollback defense and forward jump recovery: The monotonic clock guard verifies that system time does not drift backwards beyond 60 seconds relative to `clock_guard.high_water_at`; backward drift blocks write operations with error code `clock_out_of_sync`, while accidental forward clock shifts can be re-synchronized via `system.acknowledge_clock_jump` by the Owner.
* **AC-11**: Persistent schema and trigger safety: Migration 0021 establishes the `licenses` table storing license payloads, cryptographic signatures, verified claims, and active flags, protected by SQLite triggers that prevent record deletion.

## Decision

**Chosen option**: Option 1: Embedded Ed25519 Signature Verification with Dispatcher Level Gating and 14 Day Recovery Transition

Coachify implements offline desktop licensing using standard Ed25519 digital signatures verified against public keys compiled directly into the Rust core binary. The central command dispatcher acts as the single enforcement boundary, checking license status on all mutation commands while keeping query, report, export, backup, and restore pathways open. When moving to a replacement PC, a 14 day transitional grace window prevents exam day lockouts, and offline request codes enable activation over phone or messaging channels without server connectivity.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:
Migration 0021 creates the dedicated `licenses` table and associated audit triggers:

```sql
-- Migration 0021: Desktop Licensing and Entitlements

CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    license_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    customer_name TEXT NOT NULL,
    installation_id TEXT NOT NULL,
    tier TEXT NOT NULL CHECK (tier IN ('trial', 'local', 'local_pro')),
    kind TEXT NOT NULL CHECK (kind IN ('trial', 'paid', 'extension', 'trial_conversion')),
    serial INTEGER NOT NULL DEFAULT 0,
    paid_through TEXT NOT NULL,
    grace_days INTEGER NOT NULL DEFAULT 14,
    valid_until TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    limits_json TEXT NOT NULL DEFAULT '{}',
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    key_id TEXT NOT NULL,
    raw_payload TEXT NOT NULL,
    signature TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    moved_first_detected_at TEXT,
    created_at TEXT NOT NULL,
    imported_by_user_id TEXT REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_licenses_active ON licenses(is_active) WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_licenses_serial ON licenses(license_id, serial DESC);

-- Active license view for rapid single row retrieval
CREATE VIEW IF NOT EXISTS active_license AS
SELECT * FROM licenses WHERE is_active = 1 LIMIT 1;

-- Immutability trigger: licenses table is append only, records cannot be deleted
CREATE TRIGGER IF NOT EXISTS prevent_delete_licenses
BEFORE DELETE ON licenses
BEGIN
    SELECT RAISE(ABORT, 'Historical license records cannot be deleted');
END;
```

**State transitions**:
License Lifecycle States:
`active_trial` &rarr; (14 days pass) &rarr; `expired_past_grace` (read only mode)
`active_trial` &rarr; (import paid license) &rarr; `active_paid`
`active_paid` &rarr; (30 days before paid_through) &rarr; `expiring_soon` (active, warning banners)
`expiring_soon` &rarr; (paid_through date reached) &rarr; `in_grace` (active, 14 day grace countdown)
`in_grace` &rarr; (valid_until passed) &rarr; `expired_past_grace` (read only mode)
`in_grace` or `expired_past_grace` &rarr; (import renewed license) &rarr; `active_paid`

Hardware Migration State:
Backup restored on new host &rarr; `moved_transitional` (active, 14 day countdown anchored to `moved_first_detected_at`)
`moved_transitional` &rarr; (import re-bound license matching new installation ID) &rarr; `active_paid`
`moved_transitional` &rarr; (14 days pass without rebind) &rarr; `expired_past_grace` (read only mode)

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `license.get_status` | Read | none | LicenseStatusResponse | Authenticated | 500 query failed |
| `license.get_request_code` | Read | none | RequestCodeResponse | Authenticated | 500 device resolution failed |
| `license.import_file` | Mutation | license_content: String | LicenseStatusResponse | `settings.manage` | `license_signature_invalid`, `license_installation_mismatch`, `license_serial_downgrade`, `license_payload_malformed` |

**Data transfer structures**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LicenseStatusResponse {
    pub status: String, // "active", "expiring_soon", "in_grace", "expired_past_grace", "moved_transitional"
    pub tier: String,   // "trial", "local", "local_pro"
    pub customer_name: String,
    pub license_id: String,
    pub installation_id: String,
    pub paid_through: String,
    pub valid_until: String,
    pub days_remaining: i32,
    pub grace_days: u32,
    pub is_read_only: bool,
    pub is_moved: bool,
    pub moved_days_remaining: Option<u32>,
    pub request_code: String,
    pub active_students: u32,
    pub student_limit: Option<u32>,
    pub workstation_limit: Option<u32>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RequestCodeResponse {
    pub installation_id: String,
    pub request_code: String,
    pub qr_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportLicenseInput {
    pub license_content: String,
}
```

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| `license.get_status` | Status string and read only flag | Computed from `active_license.valid_until`, `grace_days`, `is_moved`, and current business date |
| `license.get_status` | Days remaining count | Computed calendar days between current business date and `active_license.paid_through` |
| `license.get_status` | Active student count | `SELECT COUNT(*) FROM enrollments WHERE status = 'active'` |
| `license.get_status` | Offline request code | HMAC SHA256 of machine `installation_id` formatted into uppercase 20 character Base32 grouped text |
| `license.get_request_code` | QR verification payload | Formatted string `COACHIFY-REQ\|v1\|<installation_id>\|<sha256_checksum>` |
| `license.import_file` | Verified claims and limits | Decoded canonical JSON payload validated against Ed25519 signature |
| `license.import_file` | Host hardware match | Local machine installation ID read from `%ProgramData%\Coachify\installation_id` |
| Read only banner | Remaining grace days or expired message | `license.get_status` response consumed by React shell state |

**Key invariants**:
* License verification operates 100 percent offline with zero network blocking.
* A tampered, forged, or unverified signature is rejected before any database modification occurs.
* Newly imported license serial numbers must be greater than or equal to the active stored serial number.
* Read only mode strictly blocks state mutations while leaving search, view, receipt reprint, reports, data export, backup, and restore fully usable.
* Restoring a backup on a replacement computer initiates a 14 day full access transitional window rather than immediately halting counter work.
* System clock backwards drift beyond 60 seconds relative to the audit log high water mark blocks write mutations with `clock_out_of_sync`.
* Accidental forward clock drift can be reconciled through `system.acknowledge_clock_jump` by the Owner role.
* The `licenses` table is append only; historical records are never deleted.

**Security model**:
* All authenticated users (Receptionist, Cashier, Teacher, Admin, Owner) can query license status.
* Only users with `settings.manage` permission (Admin and Owner roles) can import licenses or view technical cryptographic details.
* Verification public keys are compiled into the core binary and verified using Ed25519 cryptography.

**Configuration required**:
No external environment variables or third party cloud tokens required. Public keys for `k2026a` and `k2026b` are compiled into `crates/core/src/licensing.rs`. Machine installation ID is stored locally in `%ProgramData%\Coachify\installation_id`.

**Critical test scenarios**:
* Signature verification happy path: Verifies authentic Ed25519 signature and activates valid `.coachifylicense`, verifying **AC-1**, **AC-9**.
* Tampered payload rejection: Modifies payload bytes or signature, asserts refusal with `license_signature_invalid`, verifying **AC-1**.
* Installation mismatch rejection: Attempts import of license created for different installation ID, asserts refusal with `license_installation_mismatch`, verifying **AC-2**.
* Serial downgrade rejection: Attempts import of older serial number license, asserts refusal with `license_serial_downgrade`, verifying **AC-3**.
* Automatic 14 day trial generation: Executes initial setup bootstrap, confirms 14 day active trial in database with sentinel signature, verifying **AC-4**.
* Grace period active state: Sets clock past `paid_through` but before `valid_until`, verifies full operational access and warning status, verifying **AC-5**.
* Dispatcher read only enforcement: Sets clock past `valid_until`, asserts mutation commands return `license_expired_read_only` while queries succeed, verifying **AC-6**.
* Restored backup transitional grace: Simulates restored database on new host ID, verifies 14 day `moved_transitional` full access state, verifying **AC-7**.
* Offline request code generation: Asserts 20 character formatted request code and valid QR payload, verifying **AC-8**.
* Clock rollback detection and jump recovery: Advances audit watermark, verifies rollback rejection with `clock_out_of_sync`, and confirms recovery via `system.acknowledge_clock_jump`, verifying **AC-10**.
* Append only trigger defense: Attempts SQL delete on `licenses` table, confirms SQLite trigger abortion, verifying **AC-11**.

## Build plan

Following the Tracer Bullet build approach, Feature 62 progresses in five vertical end to end layers:

1. Create Migration 0021 establishing the `licenses` table, active license view, and append only triggers, satisfies **AC-11**.
2. Implement cryptographic verification engine in `crates/core/src/licensing.rs` with embedded Ed25519 public keys, canonical JSON parser, installation matching, and serial validation, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-8**.
3. Wire setup bootstrap trial generation for 14 days and connect `license.get_status`, `license.get_request_code`, and `license.import_file` RPC commands to the core dispatcher, satisfies **AC-4**, **AC-8**, **AC-9**.
4. Update dispatcher license check pipeline to evaluate expiration, 14 day grace, moved machine transitional window, and clock guard, rejecting writes in read only mode while keeping queries and exports open, satisfies **AC-5**, **AC-6**, **AC-7**, **AC-10**.
5. Build React user interface components in `ui/src/components/settings/license-settings-card.tsx`, top navigation `ReadOnlyBanner`, and `MovedLicenseModal` with QR preview and drag and drop import, satisfies **AC-5**, **AC-6**, **AC-7**, **AC-9**.
6. Implement comprehensive automated integration test suite in `crates/core/tests/licensing_integration.rs` and UI component tests, satisfies **AC-1** through **AC-11**.

## Consequences

**Positive**:
* Coaching centers run completely offline with total operational independence from cloud servers.
* Counter operations are protected against sudden lockouts on exam mornings through the 14 day transitional and grace windows.
* Student records, receipt reprints, reports, and financial ledgers are never held hostage when a subscription expires.
* Cryptographic signatures prevent unauthorized file tampering or manual SQLite editing of license expiration dates.

**Negative / tradeoffs**:
* Offline licensing means revoked licenses cannot be canceled on an offline PC until that PC connects to the internet or reaches its natural expiration date.
* Institution owners must communicate their request code to vendor support via phone or WhatsApp when renewing offline.

**Neutral**:
* Migration 0021 adds one new table and an immutable trigger to the database.
* Compiling public keys into the core binary requires releasing a routine software update if a key identifier is ever retired.

## Follow-up

* [ ] Coordinate with customer support team to establish the WhatsApp Business response template for incoming offline license request codes.
* [ ] Commit shared test vector JSON files (`valid_license.json`, `tampered_license.json`, `mismatched_installation.json`) to verify parity between the desktop core and future vendor cloud issuing service.
