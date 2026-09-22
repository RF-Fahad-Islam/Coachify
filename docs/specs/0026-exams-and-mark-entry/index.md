# Spec 0026 (Exams & Mark Entry)

> Feature 28 · Slice 5 (Exams & Results)

## Goal

Provide coaching center teachers and academic coordinators with a high-speed, keyboard-first exam and mark entry system across batches and subjects, supporting custom full and pass marks, draft saves, instant range validation, complete-roster verification gates, and optional 4th subject designation.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Numeric Representation** | Integer hundredths (`INTEGER`, e.g. 85.50 → `8550`) | Prevents floating-point rounding errors and matches financial poisha convention across SQLite, Rust, and TypeScript |
| **Keyboard Ergonomics** | Arrow-key matrix navigation (`ArrowUp`/`ArrowDown`/`Enter`/`Tab`) with single-key absent toggle (`A`) | Enables batch teachers to enter 80 student marks in under 1 minute without touching a mouse |
| **Strict Range Enforcement** | Immediate UI validation + SQLite trigger `trg_marks_validate_range` rejecting marks > `full_marks` or < 0 | Prevents clerical errors and guarantees invalid scores never persist in draft or verified states |
| **Subject Verification Gate** | `exams.verify_subject` requires 100% of enrolled students to have a mark or be marked absent | Prevents premature result calculation with missing student scores |
| **Immutability of Verified Marks** | `trg_marks_prevent_edit_when_verified` rejects updates unless explicitly unverified with `exam.edit_verified` | Ensures academic accountability and prevents unauthorized retroactive mark alterations |
| **Publishing Lock** | `trg_marks_prevent_edit_when_published` strictly rejects mark updates once an exam is published | Preserves historical exam integrity; modifications require creating a new result snapshot |
| **Optional 4th Subject Tag** | `is_optional` flag stored per student mark with default derived from program/batch curriculum | Enables NCTB 4th subject bonus calculation during grading |

## Requirements

**User stories**:
- As a batch teacher, I want to navigate the mark entry spreadsheet using only the keyboard (`ArrowUp`, `ArrowDown`, `Enter`), so that I can input an entire batch's test scores rapidly.
- As a teacher entering marks, I want an immediate error if I type a number higher than the exam's full marks (e.g. 105 on a 100-mark exam), so that typos are caught on the spot.
- As an academic coordinator, I want to mark a student absent with a single keystroke (`A`), so that absentee records are clearly distinguished from zero marks.
- As an institute owner, I want subjects to be verified only when all students have marks recorded, so that no student is accidentally left unassessed.
- As an administrator, I want verified marks locked against casual edits, requiring explicit permission and leaving an audit trail if changes are necessary before publishing.

## Acceptance Criteria

### AC-1: Exam Scheduling and Batch Assignment
- RPC `exams.create` creates a new exam linked to an academic session, grading scheme, exam type (`term`, `class_test`, `model_test`, `final`, `quiz`), start date, end date, and participating batches.
- RPC `exams.add_subject` / `exams.configure_subjects` sets up participating subjects with custom `full_marks` (e.g. 100.00 = 10000) and `pass_marks` (e.g. 33.00 = 3300), exam date, and optional subject allowance.
- RPC `exams.list` returns exams with status badges (`draft`, `entry`, `verified`, `published`), batch counts, subject counts, and completion metrics.

### AC-2: Rapid Keyboard Mark Entry Grid
- RPC `exams.get_mark_sheet` returns all actively enrolled students in the participating batches for the given subject, along with existing marks, full marks, pass marks, and verification status.
- UI renders a high-performance spreadsheet grid:
  - `ArrowDown` / `Enter`: Focus next student mark cell.
  - `ArrowUp`: Focus previous student mark cell.
  - `A` / `a`: Toggle student absence (`is_absent = true`, marks disabled/cleared).
  - Decimal support: Allows entering decimal marks (e.g. `85.5`) automatically scaled to integer hundredths (`8550`).
  - Real-time client-side validation flags out-of-range marks (`marks > full_marks` or `marks < 0`) with red highlights and blocks saving.
- RPC `exams.save_marks` saves draft marks for the entire roster in a single transaction in under 300 ms for 100 students.

### AC-3: Complete Roster Verification Gate
- RPC `exams.verify_subject` inspects all active enrollments for the participating batches.
- If any active student lacks a mark (and is not marked absent), verification fails with `marks_incomplete` error detailing unrecorded students.
- When 100% complete, updates all mark rows for that subject to `status = 'verified'` and sets `verified_by` and `verified_at`.
- When all subjects for an exam are verified, the exam status automatically transitions to `verified`.

### AC-4: Guarded Retroactive Edits
- Editing verified marks requires `exam.edit_verified` permission or an explicit unverify call (`exams.unverify_subject`).
- Once an exam is published (`status = 'published'`), direct mark updates are rejected by database triggers.

### AC-5: Structured Domain Errors
- Maps to typed `RpcError` with explicit error codes:
  - `exam_not_found`: Requested exam does not exist.
  - `exam_subject_not_found`: Subject not configured for this exam.
  - `mark_out_of_range`: Mark exceeds subject full marks or is negative.
  - `marks_incomplete`: Cannot verify subject because some enrolled students are missing marks.
  - `exam_already_published`: Cannot edit marks for a published exam.
  - `unauthorized_verified_edit`: Cannot modify verified marks without `exam.edit_verified`.
