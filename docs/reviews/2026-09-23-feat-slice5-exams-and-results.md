# Review: Slice 5 (Exams & Results), 2026-09-23

**Reviewed by**: Gemini Pro (author on Claude Sonnet)
**Scope**: Features 28 and 29 (Exams & Mark Entry, Results, Grading & Report Cards), Migration 0015, Core Exam Engine, Dispatcher Endpoints, Integration Tests, UI Components, Print Templates
**Verdict**: Blocked

## Summary

Slice 5 delivers the academic assessment foundation for Coachify. It adds keyboard spreadsheet mark entry with integer hundredths, NCTB GPA 5 calculation with optional fourth subject bonus math, five tier deterministic tie breaking, frozen immutable result snapshots, bilingual A4 student progress report cards, and fixed width landscape batch tabulation sheets.

The data model and database triggers in Migration 0015 provide exceptional defense in depth. However, our architectural inspection uncovered two critical blockers in the core exam engine: a flawed SQL scalar subquery that halts automatic exam verification for any exam with two or more subjects, and a missing verification check in the result publishing workflow that permits publishing unverified draft exams. Furthermore, the overall letter grade formula hard codes NCTB letter grades, corrupting results for British Curriculum and custom grading schemes.

## Blockers

### 🔴 Multi subject verification query fails to transition exam status, `crates/core/src/exam.rs:1038`
**Problem**: In `verify_subject`, the query determining if all subjects of an exam are verified runs this subquery:
```sql
SELECT
    (SELECT count(*) FROM exam_subjects WHERE exam_id = ?1),
    (SELECT count(distinct es.id)
     FROM exam_subjects es
     JOIN marks m ON m.exam_subject_id = es.id AND m.status = 'verified'
     WHERE es.exam_id = ?1
     GROUP BY es.id
     HAVING count(*) >= ?2)
```
**Why it matters**: In SQLite, evaluating a subquery with `GROUP BY es.id` in a scalar context returns multiple rows where each row has `count(distinct es.id) = 1`. SQLite selects only the first row, evaluating `verified_subjects_count` as 1. For any exam with two or more subjects, `verified_subjects_count >= all_subjects_count` evaluates to false. The parent exam status never advances to `verified`, blocking downstream workflows.
**Suggested fix**: Wrap the grouped subject subquery in an outer `SELECT count(*) FROM (...)`:
```sql
SELECT
    (SELECT count(*) FROM exam_subjects WHERE exam_id = ?1),
    (SELECT count(*) FROM (
        SELECT es.id
        FROM exam_subjects es
        JOIN marks m ON m.exam_subject_id = es.id AND m.status = 'verified'
        WHERE es.exam_id = ?1
        GROUP BY es.id
        HAVING count(*) >= ?2
    ))
```

### 🔴 Missing verification gate allows publishing unverified draft exams, `crates/core/src/exam.rs:1433`
**Problem**: In `publish_results`, the function checks only `if current_status == "published"`. It never checks whether `preview.unverified_subjects` is empty, nor does it require `current_status == "verified"`.
**Why it matters**: Spec 0027 AC-8 and `ExamError::ExamNotVerified` explicitly require that exams cannot be published until all subjects are verified. Without this check, a coordinator or user can publish an exam while marks are incomplete, permanently freezing unverified zero scores and broken rosters into immutable snapshots.
**Suggested fix**: Add an explicit verification guard before creating the snapshot:
```rust
if !preview.unverified_subjects.is_empty() || current_status != "verified" {
    return Err(ExamError::ExamNotVerified(payload.exam_id.clone()));
}
```

### 🔴 Overall student grade hard codes NCTB scale and caps GPA at 500, `crates/core/src/exam.rs:1333`
**Problem**: In `calculate_and_preview_results`, the overall letter grade is computed with a hard coded ladder:
```rust
let gpa_grade = if capped_gpa >= 500 {
    "A+".to_string()
} else if capped_gpa >= 400 {
    "A".to_string()
...
```
and caps GPA using `min(500, calculated_gpa)`.
**Why it matters**: Migration 0015 seeds the British Curriculum (`scheme_british_gpa4`, Cambridge / Edexcel GPA 4.0) with letter grades A*, A, B, C, D, E, U. When an exam uses this scheme, the overall student grade completely ignores the configured scheme bands. A top British curriculum student with GPA 4.00 receives grade A instead of A*, and students cannot receive valid British grades.
**Suggested fix**: Look up the overall grade and cap the GPA dynamically using the exam's assigned `grading_scheme` and its associated `grade_bands`.

## Major

### 🟠 Missing SHA 256 audit log attribution on sensitive exam operations, `crates/core/src/exam.rs:1063`
**Problem**: The functions `unverify_subject` and `revert_published_to_verified` both accept user ID as `_user_id: &str` without calling `crate::audit::record_audit_entry`.
**Why it matters**: Reverting a published exam and unlocking verified marks are sensitive administrative actions with legal and institutional ramifications. In coaching centers where grade tampering is a concern, unverified mark edits must leave an unbroken audit trail identifying who re-opened the marks.
**Suggested fix**: Call `crate::audit::record_audit_entry` in both functions, recording the action name, actor user ID, target entity ID, and timestamp.

### 🟠 Verified mark edits in save_marks fail with untyped SQL error, `crates/core/src/exam.rs:953`
**Problem**: When a user attempts to edit verified marks without un-verifying, the SQLite trigger `trg_marks_prevent_edit_when_verified` fires. In `exam.rs`, this is caught as a generic `rusqlite::Error` and returned as `exam_sql_error` instead of the typed `unauthorized_verified_edit` domain code specified in Spec 0026 AC-5.
**Why it matters**: The client UI receives an unhelpful database error message rather than a structured error that directs the user to request permission or re-open verification.
**Suggested fix**: Check if the subject is verified before running mark updates in `save_marks`, returning `Err(ExamError::UnauthorizedVerifiedEdit)` immediately.

## Minor

### 🟡 Scope table status discrepancy in index document, `docs/scope/index.md:95`
**Problem**: In `docs/scope/index.md`, line 95 reads `· 3 done · 2 in-progress`, whereas the summary table and `docs/scope/academics.md` list both Features 28 and 29 as done.
**Why it matters**: Inconsistent progress counts between the summary table and the epic breakdown create ambiguity during release audits.
**Suggested fix**: Update line 95 of `docs/scope/index.md` to `· 5 done`.

## Strengths

- **Integer hundredths precision**: Storing all scores, full marks, pass marks, and GPAs as integers (`8550` for 85.50, `500` for 5.00) completely prevents floating point rounding drift.
- **Robust database immutability triggers**: Triggers `trg_results_immutable_update`, `trg_results_immutable_delete`, `trg_result_subject_details_immutable_update`, and `trg_result_subject_details_immutable_delete` provide strict defense in depth for historical records.
- **Accurate NCTB fourth subject math**: The bonus calculation `max(0, gp - 200)` correctly adds bonus points to the mandatory sum and divides by mandatory subjects alone, ensuring fourth subject failures do not fail the student.
- **High speed keyboard spreadsheet UX**: The mark entry matrix in `mark-entry-grid.tsx` provides fluent arrow key navigation, single key `A` absent toggling, and instant range validation.
- **Print layout safety**: Landscape CSS `@page { size: landscape; margin: 8mm; }` with `table-layout: fixed` guarantees up to 10 subjects fit cleanly on standard A4 paper without horizontal clipping.

## Verification & Test Gate Status

- **Unit and Integration Tests**: All 6 tests in `crates/core/tests/exams_and_results_integration.rs` passed.
- **UI Test Suite**: All 120 tests across 31 test files passed, including all 6 exams UI component tests.
- **TypeScript RPC Contract**: Verified fresh and in sync via `cargo run -p xtask check-types`.
- **Database Migrations**: Migration 0015 verified intact via `cargo run -p xtask check-migrations`.
- **Code Formatting and Linter**: `cargo clippy` passed cleanly under `-D warnings`, `cargo fmt` clean, `pnpm lint` and `pnpm format:check` clean.
