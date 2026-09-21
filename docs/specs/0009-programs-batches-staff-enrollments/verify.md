# Verify: Academic programs, batches, staff directory, and student enrollments (spec 0009, updated 2026-09-22)

Steps derived from spec 0009 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [ ] **Program Setup**: Log in as Admin, navigate to "Classes", click "Add Program", fill in Program name (e.g. "HSC 2026 Science"), code ("HSC26-SCI"), select Academic Session, select Group Major ("Science"), save, and confirm that the new program appears in the programs list (verifies AC-1, AC-11).
- [ ] **Staff & Teacher Registration**: Navigate to "Teachers" / "Staff", click "Add Staff", enter name (e.g. "Dr. Kamrul Hasan"), designation ("Senior Physics Lecturer"), valid Bangladeshi phone number ("01812345678"), salary type ("per_class"), and save. Verify that the staff member appears in the directory table and is searchable via partial name in under 50ms (verifies AC-5, AC-11).
- [ ] **Batch Creation with Schedule**: Under the program, click "Add Batch", enter Title ("Morning Physics A"), code ("MP-A"), capacity (30), room ("Room 201"), select "Dr. Kamrul Hasan" as lead teacher, add schedule slots (Sunday, Tuesday, Thursday 08:00–09:30 in Room 201), and submit. Verify batch is created with 0/30 seats occupied (verifies AC-3, AC-8, AC-11).
- [ ] **Teacher Schedule Conflict Warning**: Click "Add Batch" again to create "Chemistry Batch A", assign the same teacher "Dr. Kamrul Hasan", and set schedule to Sunday 08:30–10:00. Verify that the UI displays a real-time warning banner identifying the overlapping schedule with "Morning Physics A", and requires supervisor confirmation before proceeding (verifies AC-4, AC-11).
- [ ] **Room Schedule Conflict Warning**: Attempt to create a third batch with a different teacher in "Room 201" on Tuesday 08:30–10:00. Verify that the UI flags the room double-booking in real time (verifies AC-4, AC-11).
- [ ] **Student Enrollment & Auto-Roll**: Open the batch detail view for "Morning Physics A", click "Enroll Student", search for an existing active student, submit without entering a roll number, and verify that roll 1 is assigned automatically and capacity updates to 1/30 (verifies AC-7, AC-8, AC-11).
- [ ] **Sequential Roll Number Allocation**: Enroll a second student into the same batch without roll number; verify roll 2 is allocated. Attempt to enroll a third student manually specifying roll 2; confirm the system blocks the request with a duplicate roll error (verifies AC-7).
- [ ] **Batch Capacity Overflow Guard**: In a test batch with capacity 2, enroll 2 students. Attempt to enroll a 3rd student without overcapacity flag; confirm the system rejects with `batch_capacity_reached`. Check the "Allow Overcapacity" supervisor box; confirm enrollment succeeds with 3/2 seats indicated (verifies AC-8, AC-11).
- [ ] **Atomic Batch Transfer**: Open the Student Profile for an enrolled student, click the "Academic History" tab, click "Transfer Section" next to "Morning Physics A", select target batch "Evening Physics B", confirm transfer. Verify that "Morning Physics A" status switches to "Transferred" with today's date, and the student is now actively enrolled in "Evening Physics B" with an allocated roll number (verifies AC-9, AC-11).
- [ ] **Deletion Safety Guards**: Attempt to delete a staff member assigned to an active batch; verify rejection by database trigger. Attempt to delete a batch with enrollment history; verify rejection by database trigger (verifies AC-1, AC-6).
- [ ] **Role Permissions Verification**: Log in as a Teacher; verify read-only access to assigned batches and student rosters, with "Add Program", "Add Batch", and "Transfer Student" buttons hidden or disabled (verifies AC-10).

## Commands

- [ ] `cargo test -p coachify_core --test academic_operations` (runs comprehensive core integration tests verifying program creation and archiving, subject management, batch schedule parsing and recurrence modeling, in-memory teacher and room schedule collision detection, staff FTS5 search performance under 50ms, auto-roll sequencing, duplicate roll rejection, capacity overflow guards, atomic batch transfer transactions, delete rejection triggers, and RBAC permission checks, verifies AC-1 through AC-10).
- [ ] `cargo run -p xtask check-types` (verifies that generated TypeScript contract in `ui/src/gen/rpc.ts` is up to date with core RPC payload and response structs for `programs.*`, `batches.*`, `subjects.*`, `staff.*`, and `enrollments.*`, verifies AC-10).
- [ ] `cargo run -p xtask check-migrations` (verifies released migration hashes and confirms migration 0011 executes cleanly on fresh and upgraded databases, verifies AC-1 through AC-7).
- [ ] `pnpm --dir ui test` (runs Vitest UI component and integration tests for Academic Classes view, Batch cards and roster table, schedule conflict dialog, Staff Directory view, and Student Profile Academic History tab, verifies AC-11).

## Acceptance criteria coverage

- **AC-1 (Programs Directory & Lifecycle)**: Verified by `test_ac1_program_crud_and_lifecycle` asserting program creation, session binding, status transitions, and delete rejection when batches exist.
- **AC-2 (Subjects Catalog & Curriculum Mapping)**: Verified by `test_ac2_subjects_and_curriculum_mapping` asserting subject catalog operations and linking subjects and subject teachers to batches.
- **AC-3 (Batches & Schedule Modeling)**: Verified by `test_ac3_batch_creation_and_schedule_json` asserting batch code uniqueness within program, shift settings, and weekly recurrence serialization.
- **AC-4 (Real-time Schedule Conflict Detection Engine)**: Verified by `test_ac4_teacher_and_room_schedule_conflicts` asserting exact interval overlap detection for both shared instructors and shared rooms across active batches.
- **AC-5 (Staff Directory & FTS5 Trigram Search)**: Verified by `test_ac5_staff_crud_and_fts5_search` asserting staff profile persistence, phone normalization, and substring search matching under 50ms.
- **AC-6 (Staff Auth Linking & Deletion Protection)**: Verified by `test_ac6_staff_deletion_safety_triggers` asserting database trigger rejection when staff is referenced in batches, batch subjects, or user accounts.
- **AC-7 (Student Enrollment & Auto-Roll Assignment)**: Verified by `test_ac7_enrollment_auto_roll_and_uniqueness` asserting sequential roll assignment and partial index unique enforcement for active batch rolls.
- **AC-8 (Batch Capacity Management & Overflow Safeguard)**: Verified by `test_ac8_batch_capacity_guard_and_override` asserting capacity computation, seat availability, overflow rejection, and supervisor bypass.
- **AC-9 (Atomic Batch Transfer & Bulk Enrollment)**: Verified by `test_ac9_atomic_batch_transfer_and_bulk_enroll` asserting single-transaction state changes, audit entries, and change event emission.
- **AC-10 (Typed RPC API & RBAC Permissions)**: Verified by `test_ac10_academic_rbac_permissions` asserting permission enforcement for `academic.view`, `academic.manage`, `staff.view`, and `staff.manage`.
- **AC-11 (Maximalist UI for Classes & Staff Directory)**: Verified by UI component tests asserting responsive layout, conflict warning modals, timetable views, and student profile enrollment tabs.
