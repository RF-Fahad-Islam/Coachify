# 0009. Academic programs, batches, staff directory, and student enrollments

**Date**: 2026-09-22
**Status**: Proposed

## Summary

Coachify provides an integrated academic operations subsystem that connects students and guardians to the institutional curriculum: **Programs**, **Subjects**, **Batches**, **Staff**, and **Student Enrollments**. 

In Bangladeshi coaching centers and educational institutions, academic offerings are structured around distinct programs (for example, "HSC 2026 Science", "Class 10 SSC Foundation", "Medical Admission Crash Course", or "IELTS Intensive"). Each program runs multiple batches partitioned by shift (Morning, Day, Evening), schedule, and target classroom capacity. Teachers and administrative personnel are managed in a unified Staff directory that links teachers to assigned batches and subjects, while optionally connecting to local system login accounts.

Student enrollment is modeled as a first-class, auditable lifecycle. Students can be enrolled in one or more batches with batch-specific roll numbers, transferred atomically between sections without loss of academic history, and protected against accidental deletion once enrolled. To maintain operational sanity during busy registration seasons, Coachify incorporates an offline-first **Schedule Conflict Detection Engine** that detects overlapping room allocations and teacher double-bookings in real time, accompanied by strict batch capacity guards with supervisor override workflows.

## Requirements

**User stories**:
- As a coaching center director or academic coordinator, I want to define programs tied to academic sessions and configure batches with schedules, assigned teachers, and capacities, so that our academic offerings are structured and predictable.
- As an academic coordinator, I want instant warnings when a proposed batch schedule creates a room conflict or double-books a teacher across multiple batches, so that scheduling clashes are prevented before classes begin.
- As a receptionist or cashier, I want to enroll students into one or more batches during or after admission, with automatic sequential roll number assignment, so that student rosters are formed immediately.
- As a receptionist, I want to transfer a student from one batch to another (for example, switching from Morning to Evening section) in a single atomic action, so that enrollment history is preserved without duplicate active entries.
- As a center manager, I want clear warnings when a batch reaches its maximum capacity, with the option to allow supervisor-approved overcapacity enrollment when necessary.
- As an institute administrator, I want to manage staff profiles (teachers, managers, accountants) with contact info, salary types, and qualifications, link teachers to batches, and optionally link staff records to software login accounts.
- As an institute owner, I want staff members actively assigned to batches and batches with enrolled students protected from hard deletion, so that audit logs, attendance history, and fee ledgers remain uncompromised.
- As a teacher, I want to view my assigned batches, schedules, and active student rosters with roll numbers and contact details, so that I can manage my classroom effectively.

**Acceptance criteria**:

- **AC-1 (Programs Directory & Lifecycle)**:
  - Programs belong to an `academic_sessions` record.
  - Core provides commands to create, update, list, and archive programs.
  - Program fields: `id` (UUIDv7), `session_id`, `name`, `name_bn`, `code`, `class_level`, `group_major` (`Science`, `Commerce`, `Humanities`, `General`), `description`, `status` (`active`, `inactive`, `archived`).
  - Cannot delete a program if batches reference it (enforced via database trigger and foreign keys).
  
- **AC-2 (Subjects Catalog & Curriculum Mapping)**:
  - Global catalog of academic subjects (`subjects`) with unique code (e.g., `PHYS-101`, `MATH-201`), English name, and Bengali name.
  - Batches link to one or more subjects through `batch_subjects(batch_id, subject_id, teacher_staff_id)`.
  - Core provides commands to manage subjects and assign/remove subjects and specialized subject teachers within a batch.
  
- **AC-3 (Batches & Schedule Structure)**:
  - Batches belong to a `program_id`.
  - Fields: `id`, `program_id`, `name`, `code` (unique within program), `teacher_staff_id` (lead class teacher), `room_number`, `shift` (`morning`, `day`, `evening`), `schedule_json`, `capacity` (default 40, check `capacity > 0`), `start_date`, `end_date`, `status` (`active`, `completed`, `cancelled`).
  - `schedule_json` stores an array of weekly recurring slots: `day_of_week` (`Sunday` through `Saturday`), `start_time` (`HH:MM` 24h format), `end_time` (`HH:MM` 24h format), and optional slot-specific `room` override.
  
- **AC-4 (Real-time Schedule Conflict Detection Engine)**:
  - Core implements a high-performance in-memory schedule conflict detection engine.
  - Detects two distinct collision categories across active batches:
    1. **Teacher Conflict**: The same teacher (`teacher_staff_id` or subject teacher in `batch_subjects`) is scheduled in multiple active batches with overlapping time windows on the same day of the week.
    2. **Room Conflict**: The same room (`room_number` or slot `room`) is scheduled for multiple active batches with overlapping time windows on the same day of the week.
  - Collision detection handles time span overlaps: `start_a < end_b && end_a > start_b`.
  - Core exposes a dry-run check query `batches.check_conflicts` for instant client validation, and validates during `batches.create` / `batches.update` (returning `schedule_conflict` with details, bypassable with supervisor `allow_conflict = true` if permitted).

- **AC-5 (Staff & Teacher Directory with FTS5 Search)**:
  - Unified staff directory (`staff`) capturing name, Bengali name, normalized phone (`8801XXXXXXXXX`), email, designation, department, educational qualifications, NID, blood group, gender, DOB, joining date, resignation date, salary type (`monthly`, `per_class`), status (`active`, `on_leave`, `resigned`, `terminated`), and notes.
  - Dedicated SQLite FTS5 trigram virtual table `staff_search_index` synchronizes via rowid triggers for instant substring search across name, Bengali name, phone, designation, and department under 50ms.

- **AC-6 (Staff Auth Linking & Deletion Protection)**:
  - User accounts (`users`) link to staff records via `users.staff_profile_id`.
  - Database triggers reject hard deletion of staff members who are assigned as lead teacher of a batch, subject teacher of a batch subject, or linked to an active user account.

- **AC-7 (Student Enrollment & Sequential Roll Assignment)**:
  - Enrollment links a student to a batch in `enrollments`.
  - Active enrollment enforces unique `(student_id, batch_id)` where `status = 'active'`.
  - Active roll number is unique per batch: `idx_batch_active_roll` on `(batch_id, roll)` WHERE `status = 'active'`.
  - Auto-roll allocates next sequential integer (`coalesce(max(roll), 0) + 1`) within the batch, while allowing manual roll assignment with duplicate check.

- **AC-8 (Batch Capacity Management & Overflow Safeguard)**:
  - Active student count is computed dynamically: `count(*) WHERE batch_id = ? AND status = 'active'`.
  - Batch summary includes `enrolled_count` and `available_seats`.
  - If `enrolled_count >= capacity`, enrollment requests return `batch_capacity_reached` error, requiring explicit client confirmation flag `allow_overcapacity = true`.

- **AC-9 (Atomic Batch Transfer & Bulk Enrollment Workflows)**:
  - **Batch Transfer**: In a single immediate database transaction:
    1. Ends active enrollment in source batch: `status = 'ended'`, `end_reason = 'transferred'`, `end_date = current_date`.
    2. Creates new active enrollment in target batch with designated roll and start date.
    3. Emits audit log entries and change events for both batches and student.
  - **Bulk Enrollment**: Allows enrolling multiple selected students into a batch in a single atomic transaction, auto-allocating sequential roll numbers and checking capacity.

- **AC-10 (Typed RPC API & RBAC Permissions)**:
  - Core exposes typed RPC commands for all entities protected by granular permissions:
    - `academic.view`: View programs, batches, subjects, and enrollments (Owner, Admin, Cashier, Teacher).
    - `academic.manage`: Create/edit programs, batches, subjects, enrollments, transfers (Owner, Admin, Cashier).
    - `staff.view`: View staff profiles and directory (Owner, Admin, Cashier, Teacher).
    - `staff.manage`: Create/edit staff records and salary configurations (Owner, Admin).

- **AC-11 (UI Classes & Staff Modules)**:
  - **Classes View (`ui/src/components/academic/`)**:
    - Program filter and batch cards showing shift, room, lead teacher, capacity progress bar, and weekly schedule tags.
    - Batch Detail view / drawer displaying student roster table with roll, student name, phone, admission date, quick attendance shortcut, and "Add Student" / "Transfer Student" actions.
    - Add/Edit Batch dialog with real-time schedule conflict warning banner.
  - **Staff Directory View (`ui/src/components/staff/`)**:
    - High-density table with designation and status filters, FTS5 instant search, and staff profile modal showing assigned batches and curriculum.
  - **Student Profile Integration**:
    - "Academic Enrollments" tab in `student-profile-dialog.tsx` displaying current active batches, past completed batches, transfer history, and quick enroll action.

## Decision

**Chosen option**: Option 1: Relational Normalized Domain with Structured JSON Recurrence Schedules, High-Performance In-Memory Conflict Matrix, Atomic Transfer Transactions, and Rowid-Bound SQLite FTS5 Staff Search.

Implementation spans:
- `migrations/0011_academic_operations_and_triggers.sql`: Deletion safety triggers, roll unique partial index, staff FTS5 search index, and RBAC permissions.
- `crates/core/src/schedule.rs`: Schedule representation, time-slot overlap mathematics, and conflict detection engine.
- `crates/core/src/academic.rs`: Domain repositories, query models, and atomic transactions for programs, batches, subjects, staff, and enrollments.
- `crates/core/src/dispatcher.rs`: RPC routing, payload validation, permission verification, and audit logging.
- `ui/src/components/academic/`: Academic Classes and Batch management views.
- `ui/src/components/staff/`: Staff & Teacher directory views.
- `ui/src/components/students/student-profile-dialog.tsx`: Tabbed academic enrollment history and actions.

## Feature design

### Data model & migration sketch

The foundational tables exist in `0003_people_directory.sql` and `0004_academic_structure.sql`. Migration `0011_academic_operations_and_triggers.sql` adds the roll unique constraint, deletion protection triggers, staff FTS5 index, and RBAC permissions:

```sql
-- Migration 0011: Academic operations, roll unique constraint, staff search, and safety triggers

-- 1. Ensure roll numbers are unique among active students in the same batch
CREATE UNIQUE INDEX IF NOT EXISTS idx_batch_active_roll
ON enrollments(batch_id, roll)
WHERE status = 'active' AND roll IS NOT NULL;

-- 2. Prevent hard deletion of programs that have batches
CREATE TRIGGER IF NOT EXISTS trg_programs_prevent_delete_with_batches
BEFORE DELETE ON programs
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batches WHERE program_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete program with existing batches; archive the program instead')
    END;
END;

-- 3. Prevent hard deletion of batches with enrollment history
CREATE TRIGGER IF NOT EXISTS trg_batches_prevent_delete_with_enrollments
BEFORE DELETE ON batches
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM enrollments WHERE batch_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete batch with enrollment history; cancel or complete the batch instead')
    END;
END;

-- 4. Prevent hard deletion of staff assigned to batches or linked to user accounts
CREATE TRIGGER IF NOT EXISTS trg_staff_prevent_delete_with_assignments
BEFORE DELETE ON staff
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batches WHERE teacher_staff_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member assigned as lead teacher of a batch')
        WHEN EXISTS (SELECT 1 FROM batch_subjects WHERE teacher_staff_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member assigned to batch subjects')
        WHEN EXISTS (SELECT 1 FROM users WHERE staff_profile_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete staff member linked to an active software user account')
    END;
END;

-- 5. Prevent hard deletion of subjects linked to batches
CREATE TRIGGER IF NOT EXISTS trg_subjects_prevent_delete_with_batches
BEFORE DELETE ON subjects
BEGIN
    SELECT CASE
        WHEN EXISTS (SELECT 1 FROM batch_subjects WHERE subject_id = OLD.id)
            THEN RAISE(ABORT, 'Cannot delete subject assigned to active or past batches')
    END;
END;

-- 6. SQLite FTS5 trigram virtual table for Staff search
CREATE VIRTUAL TABLE IF NOT EXISTS staff_search_index USING fts5(
    name,
    name_bn,
    phone_norm,
    designation,
    department,
    tokenize = 'trigram'
);

-- Triggers to synchronize staff search index
CREATE TRIGGER IF NOT EXISTS trg_staff_fts_insert AFTER INSERT ON staff
BEGIN
    INSERT INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
    VALUES (NEW.rowid, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm, NEW.designation, coalesce(NEW.department, ''));
END;

CREATE TRIGGER IF NOT EXISTS trg_staff_fts_update AFTER UPDATE ON staff
BEGIN
    DELETE FROM staff_search_index WHERE rowid = OLD.rowid;
    INSERT INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
    VALUES (NEW.rowid, NEW.name, coalesce(NEW.name_bn, ''), NEW.phone_norm, NEW.designation, coalesce(NEW.department, ''));
END;

CREATE TRIGGER IF NOT EXISTS trg_staff_fts_delete AFTER DELETE ON staff
BEGIN
    DELETE FROM staff_search_index WHERE rowid = OLD.rowid;
END;

-- Backfill staff search index
INSERT OR IGNORE INTO staff_search_index (rowid, name, name_bn, phone_norm, designation, department)
SELECT rowid, name, coalesce(name_bn, ''), phone_norm, designation, coalesce(department, '')
FROM staff;

-- 7. Seed Academic & Staff RBAC Permissions
INSERT OR IGNORE INTO permissions (code, module, description) VALUES
    ('academic.view', 'academic', 'View programs, batches, subjects, timetable schedules, and student rosters'),
    ('academic.manage', 'academic', 'Create and modify programs, batches, subjects, curriculum assignments, enrollments, and transfers'),
    ('staff.view', 'staff', 'View staff directory, profiles, and teaching assignments'),
    ('staff.manage', 'staff', 'Create and modify staff records, designations, and salary structures');

-- Grant permissions to default roles
INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'academic.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'academic.manage' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'staff.view' FROM roles WHERE name IN ('Owner', 'Admin', 'Cashier', 'Teacher');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT id, 'staff.manage' FROM roles WHERE name IN ('Owner', 'Admin');

INSERT OR IGNORE INTO _schema_version (version, applied_at)
VALUES (11, datetime('now'));
```

### Schedule model & conflict detection

The `schedule_json` column stores an array of structured weekly recurrence slots:

```json
[
  {
    "day_of_week": "Sunday",
    "start_time": "08:00",
    "end_time": "09:30",
    "room": "Room 101"
  },
  {
    "day_of_week": "Tuesday",
    "start_time": "08:00",
    "end_time": "09:30",
    "room": "Room 101"
  },
  {
    "day_of_week": "Thursday",
    "start_time": "08:00",
    "end_time": "09:30",
    "room": "Room 101"
  }
]
```

#### Collision algorithm

Two slots $S_1$ and $S_2$ conflict if and only if:
1. $S_1.\text{day\_of\_week} == S_2.\text{day\_of\_week}$
2. Times overlap: $S_1.\text{start\_time} < S_2.\text{end\_time} \land S_1.\text{end\_time} > S_2.\text{start\_time}$
3. And either:
   - **Teacher Collision**: Both batches share a teacher ($\text{teacher}_1 == \text{teacher}_2$)
   - **Room Collision**: Both slots share a room ($\text{room}_1 == \text{room}_2$)

### State lifecycles

```
Program:
  active ──► inactive ──► archived

Batch:
  active ──► completed
    │
    └──► cancelled

Staff:
  active ◄──► on_leave
    │
    ├──► resigned
    └──► terminated

Enrollment:
  active ──► ended (end_reason: completed | transferred | dropped | left)
```

### API surface

| Command | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `programs.create` | Mutation | `session_id: string`, `name: string`, `code?: string`, `class_level?: string`, `group_major?: string`, `description?: string` | `program: ProgramDetail` | `academic.manage` | `program_code_exists`, `session_not_found` |
| `programs.update` | Mutation | `id: string`, `name: string`, `code?: string`, `class_level?: string`, `group_major?: string`, `description?: string`, `status?: string` | `program: ProgramDetail` | `academic.manage` | `not_found`, `program_code_exists` |
| `programs.list` | Query | `session_id?: string`, `status?: string` | `programs: ProgramSummary[]` | `academic.view` | none |
| `programs.get` | Query | `id: string` | `program: ProgramDetail`, `batches: BatchSummary[]` | `academic.view` | `not_found` |
| `batches.create` | Mutation | `batch: CreateBatchInput`, `allow_conflict?: boolean` | `batch: BatchDetail` | `academic.manage` | `batch_code_exists`, `schedule_conflict`, `teacher_not_found` |
| `batches.update` | Mutation | `id: string`, `batch: UpdateBatchInput`, `allow_conflict?: boolean` | `batch: BatchDetail` | `academic.manage` | `not_found`, `batch_code_exists`, `schedule_conflict` |
| `batches.get` | Query | `id: string` | `batch: BatchDetail`, `subjects: BatchSubjectItem[]`, `enrolled_count: number`, `available_seats: number` | `academic.view` | `not_found` |
| `batches.list` | Query | `program_id?: string`, `teacher_id?: string`, `status?: string` | `batches: BatchSummary[]` | `academic.view` | none |
| `batches.check_conflicts` | Query | `batch_id?: string`, `teacher_staff_id?: string`, `room_number?: string`, `schedule: ScheduleSlot[]` | `conflicts: ScheduleConflictItem[]` | `academic.view` | none |
| `batches.set_subjects` | Mutation | `batch_id: string`, `subjects: BatchSubjectInput[]` | `success: boolean` | `academic.manage` | `not_found`, `subject_not_found` |
| `subjects.list` | Query | none | `subjects: SubjectItem[]` | `academic.view` | none |
| `subjects.create` | Mutation | `name: string`, `name_bn?: string`, `code: string` | `subject: SubjectItem` | `academic.manage` | `subject_code_exists` |
| `staff.create` | Mutation | `staff: CreateStaffInput` | `staff: StaffDetail` | `staff.manage` | `invalid_phone`, `phone_conflict_warning` |
| `staff.update` | Mutation | `id: string`, `staff: UpdateStaffInput` | `staff: StaffDetail` | `staff.manage` | `not_found`, `invalid_phone` |
| `staff.get` | Query | `id: string` | `staff: StaffDetail`, `assigned_batches: BatchSummary[]`, `linked_user_id?: string` | `staff.view` | `not_found` |
| `staff.list` | Query | `status?: string`, `department?: string`, `limit?: number`, `offset?: number` | `items: StaffSummary[]`, `total: number` | `staff.view` | none |
| `staff.search` | Query | `query: string`, `limit?: number` | `matches: StaffSearchHit[]` | `staff.view` | none |
| `enrollments.create` | Mutation | `student_id: string`, `batch_id: string`, `roll?: number`, `start_date?: string`, `allow_overcapacity?: boolean` | `enrollment: EnrollmentDetail` | `academic.manage` | `student_not_found`, `batch_not_found`, `already_enrolled`, `roll_taken`, `batch_capacity_reached` |
| `enrollments.transfer` | Mutation | `student_id: string`, `from_batch_id: string`, `to_batch_id: string`, `new_roll?: number`, `transfer_date?: string`, `allow_overcapacity?: boolean` | `enrollment: EnrollmentDetail` | `academic.manage` | `not_enrolled`, `roll_taken`, `batch_capacity_reached` |
| `enrollments.end` | Mutation | `enrollment_id: string`, `end_reason: string`, `end_date?: string`, `notes?: string` | `enrollment: EnrollmentDetail` | `academic.manage` | `not_found`, `already_ended` |
| `enrollments.bulk_create` | Mutation | `batch_id: string`, `student_ids: string[]`, `start_date?: string`, `allow_overcapacity?: boolean` | `enrolled_count: number`, `enrollments: EnrollmentDetail[]` | `academic.manage` | `batch_not_found`, `batch_capacity_reached` |
| `enrollments.list_by_batch` | Query | `batch_id: string`, `status?: string` | `roster: BatchRosterItem[]` | `academic.view` | `not_found` |
| `enrollments.list_by_student` | Query | `student_id: string` | `enrollments: StudentEnrollmentHistoryItem[]` | `academic.view` | `not_found` |

### Value sourcing

| Value | Origin / Algorithm |
|---|---|
| `enrollments.roll` | If provided, validated for uniqueness in active batch. If omitted, assigned as `max(roll) + 1` among active students in the batch, starting at 1. |
| `enrollments.start_date` | Input date or current local business date (`Asia/Dhaka` format `YYYY-MM-DD`). |
| `batches.available_seats` | `max(0, capacity - active_enrolled_count)` |
| `schedule_conflict` details | Generated by `schedule::detect_conflicts(conn, candidate_batch)` comparing candidate slots against all other active batch schedules. |
| `staff.phone_norm` | Normalized via Bangladeshi mobile parser (`8801XXXXXXXXX`) from `phone.rs`. |

### Key invariants

- Active enrollment in a batch must be unique per student (`(student_id, batch_id)` unique when `status = 'active'`).
- Roll number must be unique among active students in the same batch (`(batch_id, roll)` unique when `status = 'active'`).
- Programs cannot be deleted if referenced by any batch.
- Batches cannot be deleted if referenced by any past or current enrollment.
- Staff members cannot be deleted if referenced as lead teacher in a batch, assigned to a batch subject, or linked to a user account.
- Subjects cannot be deleted if assigned to any batch.
- Batch capacity must be strictly greater than 0 (`capacity > 0`). Overcapacity enrollment requires explicit confirmation flag.
- Batch transfers are atomic: ending previous enrollment and creating target enrollment occurs in a single database transaction.

## Critical test scenarios

1. **Happy Path Academic Setup**: Admin creates Program "HSC 2026", configures Batch "Batch A - Morning" with 40 capacity, assigns a teacher and Sunday/Tuesday/Thursday 08:00-09:30 schedule, links subjects, and retrieves batch details.
2. **Schedule Conflict Detection (Teacher)**: Admin attempts to schedule "Batch B" with the same teacher on Sunday 08:30-10:00. The engine flags a teacher collision with detailed overlapping batch info; saving succeeds only if supervisor bypass is provided.
3. **Schedule Conflict Detection (Room)**: Admin attempts to schedule "Batch C" with a different teacher in "Room 101" on Sunday 09:00-10:30. The engine flags a room collision; blocked without override.
4. **Auto-Roll Sequence & Roll Collision**: Receptionist enrolls three students into Batch A; rolls 1, 2, and 3 are allocated sequentially. A fourth enrollment manually requests roll 2; system rejects with `roll_taken`.
5. **Capacity Overflow Guard**: Batch capacity is set to 2. Two students are enrolled. Attempting to enroll a third student without `allow_overcapacity` returns `batch_capacity_reached`. Passing `allow_overcapacity = true` succeeds.
6. **Atomic Batch Transfer**: Student is enrolled in Batch A. Cashier invokes `enrollments.transfer` to Batch B. Verification confirms Batch A enrollment has `status = 'ended'`, `end_reason = 'transferred'`, and Batch B has active enrollment with new roll.
7. **Deletion Safety Triggers**: Direct SQL DELETE statements against a Program with batches, a Batch with enrollments, or Staff assigned to a batch abort with explicit error messages.
8. **FTS5 Staff Search**: Searching staff by partial name, Bengali name, phone, or designation returns accurate results under 50ms.
9. **RBAC Permission Enforcement**: A user with only the `Teacher` role can view batches and rosters, but cannot create programs, batches, or enroll students.

## Build plan

### Phase 1: Database foundation & migration
- Task 1.1: Write migration `0011_academic_operations_and_triggers.sql` with roll unique index, delete prevention triggers, FTS5 staff search virtual table with rowid triggers, and RBAC permissions.
- Task 1.2: Register migration 0011 in `crates/core/src/migrations.rs` and verify catalog checks.

### Phase 2: Core domain logic & conflict engine
- Task 2.1: Implement `crates/core/src/schedule.rs` for schedule parsing, slot time-interval comparison, and in-memory collision detection.
- Task 2.2: Implement `crates/core/src/academic.rs` containing programs, batches, subjects, staff, and enrollment repositories, including auto-roll allocation and atomic transfer transactions.
- Task 2.3: Implement FTS5 staff search queries in `crates/core/src/academic.rs`.

### Phase 3: RPC command dispatcher integration
- Task 3.1: Wire all `programs.*`, `batches.*`, `subjects.*`, `staff.*`, and `enrollments.*` commands in `crates/core/src/dispatcher.rs` and `crates/core/src/lib.rs`.
- Task 3.2: Export updated TypeScript contract via `cargo run -p xtask export-types`.
- Task 3.3: Write comprehensive integration test suite `crates/core/tests/academic_operations.rs` covering all acceptance criteria.

### Phase 4: User interface implementation
- Task 4.1: Build Academic Classes view (`ui/src/components/academic/academic-classes-view.tsx`) with Program filtering, batch cards, capacity indicators, timetable overview, and Add Batch modal with real-time conflict checking.
- Task 4.2: Build Batch Detail view and Roster table (`ui/src/components/academic/batch-detail-view.tsx`) with quick enroll, transfer, and roster export.
- Task 4.3: Build Staff Directory view (`ui/src/components/staff/staff-directory-view.tsx`) with high-density table, designation filters, FTS5 instant search, and staff edit modal.
- Task 4.4: Integrate Academic Enrollments tab into Student Profile dialog (`ui/src/components/students/student-profile-dialog.tsx`) to view enrolled batches and initiate batch transfers.
- Task 4.5: Add UI unit and component tests in `ui/src/components/academic/` and `ui/src/components/staff/`.

## Consequences

- Academic offerings, batches, and class rosters become first-class, reliable local data assets.
- Teacher double-bookings and classroom collisions are eliminated before term schedules are published.
- Roll numbering is consistent and automated, removing human error during busy registration weeks.
- Batch transfer history is permanently preserved, maintaining clean accounting and attendance records.
- Staff members cannot be deleted if active academic or financial commitments depend on them.
- All operations run 100% offline on low-power Windows hardware with sub-50ms responsiveness.

## Follow-up

- Feature 10 (Attendance): Will use the batch roster and schedules defined here to drive daily high-speed attendance matrices.
- Feature 11 (Fees & Billing): Will bind fee plans (admission fee, monthly tuition, course fee) to programs and batches for automated billing.
