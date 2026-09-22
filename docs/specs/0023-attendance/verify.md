# Verify: Attendance (spec 0023)

Steps derived from spec 0023 acceptance criteria.

## UI and Manual Verification

- [ ] Navigate to the Attendance view (`/academics/attendance`): select an active batch and choose today's date. Verify that the batch roster loads with all actively enrolled students (verifies AC-1).
- [ ] For a batch where attendance has not been taken, verify that the session header displays "Attendance Not Taken" badge and all marks are initialized to default state (verifies AC-1, AC-3).
- [ ] Use `Arrow Down` / `Arrow Up` to navigate through the student list. Press `P`, `A`, `L`, `E` on individual rows: verify that each key updates the student status badge instantly (verifies AC-2).
- [ ] Click `[Mark All Present]` (or press `Alt+P`): verify that all students are immediately set to Present (verifies AC-2).
- [ ] Click `[Mark All Absent]` (or press `Alt+A`): verify that all students are immediately set to Absent (verifies AC-2).
- [ ] In an 80 student batch, fill marks and click `[Save Attendance]`: verify that saving completes and UI displays confirmation in under 300 ms (verifies AC-2).
- [ ] After saving a batch with all absent marks, verify that the dashboard marks the session as "Taken (0% Present)" rather than "Not Taken" (verifies AC-3).
- [ ] Attempt to edit a past dated or locked session as a user without `attendance.edit_previous`: verify that the UI renders the session in read only mode and blocks saving with permission error (verifies AC-4).
- [ ] Log in as an administrator with `attendance.edit_previous`: edit a locked session and save. Verify that an entry is recorded in the audit log capturing previous and new marks along with the mandatory audit reason (verifies AC-4).
- [ ] Save attendance with a student marked `absent`: inspect the outbox and verify that outbox records are created for enabled channels (SMS and WhatsApp) with `scheduled_at` set 30 minutes in the future (verifies AC-5).
- [ ] Before the 30 minute delay elapses, edit the attendance session and change the absent student's status to `present`: verify that pending outbox messages across SMS and WhatsApp transition to `status = 'cancelled'` (verifies AC-6).
- [ ] Open the Student Profile Attendance tab and Batch Matrix view: verify that monthly session counts, present/absent totals, and attendance percentages match recorded marks (verifies AC-7).

## Commands

- [ ] `cargo test -p coachify_core --test attendance` (verifies batch roster retrieval, atomic session creation, sub 300ms 80 student benchmark, lock window enforcement, retroactive edit permission guard, audit logging, delayed alerts enqueue, and absence alert cancellation hook, satisfies AC-1 through AC-6).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contract freshness in `ui/src/gen/rpc.ts` for all `attendance.*` RPC payloads and responses).
- [ ] `cargo run -p xtask check-migrations` (verifies Migration 0014 schema validity and released migration immutability).
- [ ] `pnpm --dir ui test` (verifies AttendanceSheet keyboard navigation, shortcut keys, lock state indicators, and student attendance summary components).

## Acceptance Criteria Coverage

- AC-1 (Batch Roster and Session Inspection): verified by core session retrieval tests and UI roster loading.
- AC-2 (Rapid Keyboard Entry and Bulk Actions): verified by keyboard event tests, benchmark tests (under 300 ms for 80 students), and bulk mark shortcuts.
- AC-3 (Header Row Distinction): verified by core session presence tests asserting distinct states for missing header versus all absent records.
- AC-4 (Session Lock Window and Guarded Retroactive Edits): verified by lock evaluation tests, permission check enforcement, and audit log hash chain verification.
- AC-5 (Omnichannel Absence Automation Hook): verified by transactional outbox integration tests asserting delayed message creation with unique dedupe keys across SMS and WhatsApp.
- AC-6 (Mark Change Cancellation Hook): verified by outbox transition tests asserting cancellation of queued absence messages upon mark update.
- AC-7 (Monthly Student and Batch Attendance Metrics): verified by aggregation queries calculating monthly percentages and batch attendance matrices.
