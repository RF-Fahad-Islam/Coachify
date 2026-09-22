# Verify: Exams & Mark Entry (spec 0026)

Steps derived from spec 0026 acceptance criteria.

## UI and Manual Verification

- [ ] Navigate to the Exams view (`/exams`): click `[Create Exam]`. Fill in exam name, select academic session, exam type (`term`), grading scheme (`NCTB National Curriculum`), and attach participating batches (verifies AC-1).
- [ ] Configure exam subjects: add Physics (full marks 100, pass marks 33) and Chemistry (full marks 75, pass marks 25). Verify subjects appear in the exam schedule (verifies AC-1).
- [ ] Click `[Enter Marks]` for Physics: verify that the mark entry sheet loads with all actively enrolled batch students in roll order (verifies AC-2).
- [ ] Use `Arrow Down` / `Arrow Up` / `Enter` to navigate through student mark cells. Type decimal marks (e.g. `82.5`, `91`, `64.5`): verify values format cleanly (verifies AC-2).
- [ ] Press `A` on an individual row: verify that the student is toggled to `Absent` and the mark input is safely disabled (verifies AC-2).
- [ ] Attempt to enter an out-of-range mark (e.g. `110` or `-5`): verify that an immediate error badge appears, saving is prevented, and database rejects invalid values (verifies AC-2).
- [ ] In an 80-student roster, fill marks and click `[Save Draft]`: verify save completes with confirmation in under 300 ms (verifies AC-2).
- [ ] With 2 students missing marks, attempt to click `[Verify Subject Marks]`: verify that verification fails with a clear warning listing the students without marks (verifies AC-3).
- [ ] Complete all missing marks and click `[Verify Subject Marks]`: verify that the subject transitions to `Verified` status and rows lock against casual edits (verifies AC-3).
- [ ] Attempt to edit verified marks as a user without `exam.edit_verified`: verify that the UI renders the grid in read-only mode and blocks updates (verifies AC-4).

## Commands

- [ ] `cargo test -p coachify_core --test exams_and_results_integration` (verifies exam creation, subject scheduling, sub-300ms mark batch save, range validation triggers, complete-roster verification gates, and immutability rules, satisfies AC-1 through AC-5).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contract freshness in `ui/src/gen/rpc.ts` for all `exams.*` payloads and responses).
- [ ] `cargo run -p xtask check-migrations` (verifies Migration 0015 schema validity and released migration immutability).
- [ ] `pnpm --dir ui test` (verifies MarkEntryGrid keyboard navigation, absent key toggle, out-of-range validation rings, and subject verification workflows).

## Acceptance Criteria Coverage

- AC-1 (Exam Scheduling and Batch Assignment): verified by exam creation tests, subject setup tests, and exam list queries.
- AC-2 (Rapid Keyboard Mark Entry Grid): verified by keyboard event handlers, decimal-to-hundredths conversion tests, range check validation, and sub-300ms benchmark.
- AC-3 (Complete Roster Verification Gate): verified by core verification logic asserting rejection when any active student lacks marks, and status transition upon full completion.
- AC-4 (Guarded Retroactive Edits): verified by permission checks on verified mark modifications and SQLite trigger immutability enforcement.
- AC-5 (Structured Domain Errors): verified by typed error response mappings for `mark_out_of_range`, `marks_incomplete`, and `exam_already_published`.
