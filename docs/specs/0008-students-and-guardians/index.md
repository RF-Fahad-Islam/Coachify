# 0008. Students and guardians directory

**Date**: 2026-09-21
**Status**: Accepted

## Summary

Coachify provides an offline first students and guardians directory that serves as the central operational registry for coaching centers and educational institutes. Receptionists and center staff can register students, link or create guardians, upload student photos and admission documents, and retrieve records instantly. Guardians are normalized entities shared across siblings, ensuring that family contact details stay synchronized and avoiding duplicate entries.

To deliver instantaneous search on low power office computers without an active internet connection, search is powered by an encrypted SQLite FTS5 trigram virtual table synchronized through database triggers. Substring and prefix queries across student codes, student names, emergency phones, and guardian contacts return in under 50 milliseconds. Search triggers bind directly to the SQLite table rowid, preventing unindexed table scans during record updates. Profile photos and document attachments are stored in an organized, content addressed local filesystem vault outside the database file, with photos automatically resized to preserve disk space and memory.

## Requirements

**User stories**:
- As a receptionist or cashier, I want to create a student profile and link or create their guardian offline, so that admissions can proceed without delay even when the internet is unavailable.
- As an office staff member, I want to find any student instantly by typing their student ID, partial name, emergency phone, or guardian phone, so that counter operations take less than a second.
- As a receptionist, I want to link a new student to an existing guardian when siblings enroll, so that family contact information is shared and tuition communication reaches the right parent.
- As a receptionist editing a shared guardian, I want clear visibility into all affected sibling students, with the ability to either update all siblings or detach into a separate guardian record.
- As an institute administrator, I want to store student photos and admission certificates in a local content addressed vault, so that document records stay organized without bloating the main SQLite database file.
- As an institute owner, I want students with past financial charges or attendance history protected from hard deletion, so that accounting ledgers and operational records remain untampered.

**Acceptance criteria**:
- **AC-1**: Creating a student auto generates a unique human readable student code from an atomic sequence (for example `S2026-0001` using prefix `S` and year `2026`), with support for optional manual code entry that validates uniqueness. The sequence is partitioned per academic year (`student:<year>`) and starts at 1.
- **AC-2**: A student record requires at least one linked guardian upon creation. The link specifies `is_primary` (exactly one primary guardian per student) and `receives_sms` (at least one guardian designated for automated SMS notifications).
- **AC-3**: Sibling enrollment allows searching existing guardians by phone or name with live auto complete, linking an existing guardian record to the new student without duplicating guardian data.
- **AC-4**: Phone numbers accept standard Bangladeshi mobile formats (`01XXXXXXXXX`, `+8801XXXXXXXXX`, `8801XXXXXXXXX`) and normalize into standard 13 digit format `8801XXXXXXXXX`. Landlines and international foreign numbers are preserved with symbols stripped. If another active student or guardian already uses the same phone, the API returns a `phone_conflict_warning` error that the client can bypass with an explicit `allow_duplicate_phone = true` confirmation flag.
- **AC-5**: Student photos and admission documents are processed and saved in the institution local vault directory (`<data_root>/<institution_id>/vault/<digest[0..2]>/<digest>.<ext>`). Photos are decoded safely with strict dimension and memory limits, downscaled to a maximum resolution of 800 by 800 pixels in JPEG or WebP format, named by their SHA256 digest, and referenced via `photo_vault_id` and the `documents` table. Non image documents such as PDF certificates are hashed and stored verbatim.
- **AC-6**: SQLite FTS5 trigram virtual table `student_search_index` indexes student code, name, Bengali name, phone, emergency phone, current school roll, guardian names, and guardian phones. Search triggers map directly to the SQLite integer `rowid` of the `students` table, ensuring constant time index synchronization without unindexed full table scans on student updates or deletes.
- **AC-7**: Substring and prefix search queries across the FTS5 index return matching student summary records in under 50 milliseconds for an institution directory of up to 10,000 students. The client search input enforces a minimum 3 character threshold for trigram lookups, falling back to exact code matching for short inputs.
- **AC-8**: Core exposes typed, authenticated RPC commands (`students.create`, `students.update`, `students.get`, `students.list`, `students.search`, `students.archive`, `guardians.search`, `guardians.get`, `guardians.update`, `vault.upload_file`, `vault.get_file`) protected by granular permissions (`students.view`, `students.manage`, `students.archive`).
- **AC-9**: Student records support status lifecycle transitions (`active`, `inactive`, `completed`, `transferred`, `archived`). Hard deletion of a student record is rejected by the database if associated enrollments, attendance marks, or financial ledger charges and payments exist.
- **AC-10**: The user interface provides a student directory table with instant debounced search (250ms), status filters, and a comprehensive student profile view featuring tabbed sections for Overview, Guardians and Siblings, Documents and Photos, Academic Enrollments, and Fee Ledger History. Editing a shared guardian displays a list of affected siblings with an option to unlink and create a distinct record.

## Decision

**Chosen option**: Option 1: Normalized Relational People Directory with Shared Guardians Across Siblings, Content Addressed Local Vault with Safe Image Decoding, and Rowid Mapped SQLite FTS5 Trigram Substring Search.

You build the Coachify students and guardians subsystem across `crates/core/src/students.rs` (people repositories and queries), `crates/core/src/vault.rs` (content addressed document and photo management with `image` crate processing), migration `0010_student_search_and_triggers.sql` (FTS5 search table, rowid triggers, and permissions), and `ui/src/components/students/` (directory table, creation modal with guardian lookup, and tabbed profile view).

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `react-patterns` (`.claude/skills/react-patterns/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

The foundational schema for `students`, `guardians`, and `student_guardians` exists in migration `0003_people_directory.sql`. Migration `0004_academic_structure.sql` defines `documents`. Migration `0010_student_search_and_triggers.sql` adds the FTS5 trigram virtual table with rowid bindings, synchronization triggers, and RBAC permissions.

```sql
-- Migration 0010: Student search index, rowid triggers, and permissions

-- FTS5 trigram virtual table for lightning fast substring matching
-- Note: Uses the native SQLite integer rowid to link with students(rowid)
CREATE VIRTUAL TABLE IF NOT EXISTS student_search_index USING fts5(
    student_code,
    name,
    name_bn,
    phone_norm,
    emergency_phone_norm,
    current_school_roll,
    guardian_names,
    guardian_phones,
    tokenize = 'trigram'
);

-- Triggers to maintain student_search_index on student insert
CREATE TRIGGER IF NOT EXISTS trg_students_fts_insert AFTER INSERT ON students
BEGIN
    INSERT INTO student_search_index (
        rowid, student_code, name, name_bn, phone_norm, emergency_phone_norm,
        current_school_roll, guardian_names, guardian_phones
    )
    VALUES (
        NEW.rowid, NEW.student_code, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm,
        coalesce(NEW.emergency_phone_norm, ''), coalesce(NEW.current_school_roll, ''),
        (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id),
        (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id)
    );
END;

-- Triggers to update search index on student update using direct rowid index lookup
CREATE TRIGGER IF NOT EXISTS trg_students_fts_update AFTER UPDATE ON students
BEGIN
    DELETE FROM student_search_index WHERE rowid = OLD.rowid;
    INSERT INTO student_search_index (
        rowid, student_code, name, name_bn, phone_norm, emergency_phone_norm,
        current_school_roll, guardian_names, guardian_phones
    )
    VALUES (
        NEW.rowid, NEW.student_code, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm,
        coalesce(NEW.emergency_phone_norm, ''), coalesce(NEW.current_school_roll, ''),
        (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id),
        (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.id)
    );
END;

-- Triggers to remove from search index on student delete using direct rowid lookup
CREATE TRIGGER IF NOT EXISTS trg_students_fts_delete AFTER DELETE ON students
BEGIN
    DELETE FROM student_search_index WHERE rowid = OLD.rowid;
END;

-- Triggers to refresh search index when student guardians change
CREATE TRIGGER IF NOT EXISTS trg_student_guardians_fts_insert AFTER INSERT ON student_guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.student_id),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = NEW.student_id)
    WHERE rowid = (SELECT rowid FROM students WHERE id = NEW.student_id);
END;

CREATE TRIGGER IF NOT EXISTS trg_student_guardians_fts_delete AFTER DELETE ON student_guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = OLD.student_id),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = OLD.student_id)
    WHERE rowid = (SELECT rowid FROM students WHERE id = OLD.student_id);
END;

-- Triggers to refresh search index when guardian contact info changes
CREATE TRIGGER IF NOT EXISTS trg_guardians_fts_update AFTER UPDATE ON guardians
BEGIN
    UPDATE student_search_index
    SET guardian_names = (SELECT group_concat(g.name, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = (SELECT id FROM students WHERE rowid = student_search_index.rowid)),
        guardian_phones = (SELECT group_concat(g.phone_norm, ' ') FROM guardians g JOIN student_guardians sg ON sg.guardian_id = g.id WHERE sg.student_id = (SELECT id FROM students WHERE rowid = student_search_index.rowid))
    WHERE rowid IN (
        SELECT s.rowid FROM students s
        JOIN student_guardians sg ON sg.student_id = s.id
        WHERE sg.guardian_id = NEW.id
    );
END;

-- Prevent hard delete of students with financial ledger records or enrollments
CREATE TRIGGER IF NOT EXISTS trg_students_prevent_delete_with_history
BEFORE DELETE ON students
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM charges WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with existing financial charges; archive the student instead')
        WHEN EXISTS (SELECT 1 FROM payments WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with recorded payments; archive the student instead')
        WHEN EXISTS (SELECT 1 FROM enrollments WHERE student_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete student with enrollment history; archive the student instead')
    END;
END;

-- Seed student permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('students.view', 'students', 'View student directory, profiles, documents, and guardian records'),
    ('students.manage', 'students', 'Create and modify students, guardians, and profile documents'),
    ('students.archive', 'students', 'Change student operational status and archive student profiles');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.manage' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'students.archive' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (10, datetime('now'));
```

### State transitions

Student profile status follows an explicit lifecycle:

```
          ┌─────────────┐
          │   active    │◄───────────┐
          └──────┬──────┘            │
                 │                   │
         ┌───────┼──────────┐        │
         ▼       ▼          ▼        │
  ┌──────────┐ ┌─────────┐ ┌─────────┴┐
  │ inactive │ │completed│ │transferred
  └──────────┘ └─────────┘ └──────────┘
         │
         ▼
  ┌──────────┐
  │ archived │
  └──────────┘
```

- `active`: Current enrolled student participating in classes and fee billings.
- `inactive`: Temporarily suspended or absent student; charges can pause.
- `completed`: Successfully completed assigned courses or graduated.
- `transferred`: Moved to another school, institution, or branch.
- `archived`: Historical record preserved for auditing; excluded from active listings.

### API surface

| Command | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `students.create` | Mutation | `student: CreateStudentInput`, `guardian: GuardianLinkInput`, `allow_duplicate_phone?: boolean`, `photo_vault_id?: string` | `student: StudentDetail` | `students.manage` | `student_code_exists`, `invalid_phone`, `phone_conflict_warning`, `primary_guardian_missing` |
| `students.update` | Mutation | `id: string`, `expected_version: number`, `student: UpdateStudentInput`, `guardians?: GuardianLinkInput[]`, `allow_duplicate_phone?: boolean` | `student: StudentDetail` | `students.manage` | `not_found`, `concurrency_conflict`, `invalid_phone`, `phone_conflict_warning` |
| `students.get` | Query | `id: string` | `student: StudentDetail`, `guardians: GuardianDetail[]`, `documents: DocumentSummary[]` | `students.view` | `not_found` |
| `students.list` | Query | `status?: string`, `limit: number`, `offset: number` | `items: StudentSummary[]`, `total: number` | `students.view` | `invalid_pagination` |
| `students.search` | Query | `query: string`, `limit?: number` | `matches: StudentSearchHit[]` | `students.view` | none (returns empty array on short or empty query) |
| `students.archive` | Mutation | `id: string`, `reason: string`, `expected_version: number` | `student: StudentDetail` | `students.archive` | `not_found`, `concurrency_conflict` |
| `guardians.search` | Query | `query: string`, `limit?: number` | `matches: GuardianSearchHit[]` | `students.view` | none |
| `guardians.get` | Query | `id: string` | `guardian: GuardianDetail`, `siblings: StudentSummary[]` | `students.view` | `not_found` |
| `guardians.update` | Mutation | `id: string`, `guardian: UpdateGuardianInput`, `split_from_siblings?: boolean` | `guardian: GuardianDetail` | `students.manage` | `not_found`, `invalid_phone` |
| `vault.upload_file` | Mutation | `owner_type: string`, `owner_id: string`, `kind: string`, `file_name: string`, `mime_type: string`, `data_base64: string` | `document: DocumentSummary` | `students.manage` | `invalid_mime`, `file_too_large`, `image_decode_failed`, `io_error` |
| `vault.get_file` | Query | `id: string` | `data_base64: string`, `mime_type: string`, `file_name: string` | `students.view` | `not_found`, `io_error` |

### Value sourcing

| Action | Value produced or displayed | Source |
|---|---|---|
| `students.create` | `student_code` | Input parameter if specified, otherwise generated from `sequences` table (`student:<year>`) |
| `students.create` | `phone_norm` | Input student phone normalized via Bangladeshi mobile normalization helper (`8801XXXXXXXXX`) or raw clean string |
| `students.create` | `photo_vault_id` | Input parameter referencing previously uploaded photo in `documents` or null |
| `students.create` | `created_by`, `updated_by` | Session user ID resolved by command dispatcher from active token |
| `students.create` | `row_version` | Initialized to 1 |
| `students.create` | `created_at`, `updated_at` | Current UTC timestamp formatted as ISO 8601 string |
| `students.search` | `matches[].student_code` | Column `student_code` from `student_search_index` virtual table |
| `students.search` | `matches[].guardian_names` | Column `guardian_names` aggregated by FTS5 trigger |
| `guardians.get` | `siblings` | Query on `student_guardians` joined with `students` filtering by `guardian_id` |
| `vault.upload_file` | `sha256` | SHA256 digest calculated over incoming binary byte buffer |
| `vault.upload_file` | File path on disk | Derived path `<data_root>/<institution_id>/vault/<sha256[0..2]>/<sha256>.<ext>` |

### Key invariants

- Every student record must have at least one associated guardian.
- Exactly one guardian associated with a student must be designated as primary (`is_primary = 1`).
- At least one guardian associated with a student must be designated to receive SMS (`receives_sms = 1`).
- Student code must be unique across the entire institution.
- Mobile numbers starting with 01 or 8801 normalize to standard 13 digits (`8801XXXXXXXXX`). Foreign numbers and landlines preserve country and area codes with symbols stripped.
- Duplicate phone numbers generate a prompt warning before persistence, bypassable with `allow_duplicate_phone = true`.
- Profile photos in the vault must be resized to a maximum dimension of 800 by 800 pixels with maximum file decoding limit of 20MB and dimensions under 6000 by 6000 pixels.
- Deletion of a student record with existing financial charges, payments, or enrollment history is rejected by database triggers.

### Security model

Access control is enforced by the centralized command dispatcher:
- `students.view`: Permits read only access to the student directory, profile views, guardian contact information, and vault documents. Assigned to Owner, Admin, Cashier, and Teacher.
- `students.manage`: Permits creating students, editing profile information, linking guardians, updating guardian records, and uploading documents to the vault. Assigned to Owner, Admin, and Cashier.
- `students.archive`: Permits changing student operational status to archived or inactive. Assigned to Owner and Admin.

Files in the vault are accessed exclusively through core RPC commands, verifying session authentication and permissions before streaming binary content.

### Configuration required

No new external environment variables or third party cloud credentials are required. All storage is local to the host machine using SQLite and the local institution vault directory.

### Critical test scenarios

- Happy path: Receptionist creates a new student with a newly registered guardian and photo, queries the student via partial name in FTS5 search, and opens the full profile. Verifies AC-1, AC-2, AC-5, AC-6, AC-7, AC-10.
- Sibling reuse path: Receptionist registers a second student, searches the existing guardian by phone number, links the existing guardian as primary, and confirms that both student profiles link to the same guardian ID. Verifies AC-3, AC-4.
- Sibling guardian update dialog: Receptionist updates a shared guardian address; the user interface notifies that 2 students share this guardian and updates both records upon confirmation. Verifies AC-3, AC-10.
- Phone duplicate warning: Receptionist enters a phone number already registered to another active student; the system responds with `phone_conflict_warning`, prompting the user before saving when `allow_duplicate_phone = true`. Verifies AC-4.
- Concurrency conflict: Two staff members edit the same student profile simultaneously; the second update provides a stale `expected_version` and receives a concurrency error. Verifies AC-8.
- Ledger safety guard: Attempting to execute a SQL DELETE on a student who has an active enrollment or tuition fee charge is aborted by SQLite triggers with an informative error. Verifies AC-9.
- Permission enforcement: A user possessing only the `Teacher` role attempts to invoke `students.create` or `students.archive` and receives a permission denied error. Verifies AC-8.

## Build plan

The build plan follows the Tracer Bullet approach, standing up a thin functional thread from the database schema and Rust core through the RPC bridge to the React user interface, then thickening features.

### Phase 1: Search index and database foundation
- [ ] Task 1.1: Migration 0010 creating `student_search_index` FTS5 trigram table with rowid mapping, sync triggers, delete prevention trigger, and permissions, satisfies AC-6, AC-8, AC-9
- [ ] Task 1.2: Phone number normalization and validation utility in `crates/core/src/phone.rs` supporting Bangladeshi mobile formatting and duplicate conflict checks, satisfies AC-4
- [ ] Task 1.3: People repository implementation in `crates/core/src/students.rs` supporting CRUD, atomic student code generation with yearly partitioning (`student:<year>`), guardian linking, and status transitions, satisfies AC-1, AC-2, AC-3, AC-9

### Phase 2: Local vault and document storage
- [ ] Task 2.1: Vault storage manager in `crates/core/src/vault.rs` with content addressed file storage, SHA256 verification, safe image decoding limits, and image resizing (max 800x800), satisfies AC-5
- [ ] Task 2.2: FTS5 trigram query execution and ranking in `crates/core/src/students.rs` with minimum 3 character query guard delivering search results under 50ms, satisfies AC-6, AC-7

### Phase 3: RPC command dispatcher integration
- [ ] Task 3.1: Register student, guardian, and vault RPC commands in `crates/core/src/dispatcher.rs` and export TypeScript contract types via xtask, satisfies AC-8
- [ ] Task 3.2: Integration test suite in `crates/core/tests/students_and_guardians.rs` validating AC-1 through AC-9

### Phase 4: User interface directory and search
- [ ] Task 4.1: React Student Directory view with debounced real time FTS5 search input (250ms), status filters, and pagination, satisfies AC-7, AC-10
- [ ] Task 4.2: Add Student dialog with multi step sections: Basic Information, Guardian Search & Link (supporting sibling reuse), and Photo Upload, satisfies AC-1, AC-2, AC-3, AC-4, AC-5, AC-10
- [ ] Task 4.3: Comprehensive Student Profile view with tabbed sections (Overview, Guardians & Siblings, Documents, Academic History, Fee Ledger History) and shared guardian edit modal, satisfies AC-3, AC-10
- [ ] Task 4.4: Component test suite in `ui/src/components/students/` verifying search interactions, validation feedback, and form submissions, satisfies AC-10

## Consequences

- Receptionists can register students and verify guardians completely offline during power or internet outages.
- Shared guardian records prevent duplicate contacts and simplify future family billing or sibling tuition concession calculations.
- Using FTS5 trigram indexing with rowid bindings provides instant substring searching across English and Bengali text with zero unindexed table scans on updates.
- Resizing photos before persisting to the local vault keeps disk usage under 100 kilobytes per student while preventing high resolution photos from exhausting WebView2 memory.
- Sibling guardian sharing means updating a guardian address or phone number updates the contact information for all associated children immediately, with an explicit confirmation dialog informing staff of the blast radius.

## Follow-up

- [ ] Feature 10 will connect student profiles to academic batches, subject selections, and enrollment history.
- [ ] Feature 11 will build the guided Excel student import engine to batch populate students and guardians from legacy spreadsheets.
