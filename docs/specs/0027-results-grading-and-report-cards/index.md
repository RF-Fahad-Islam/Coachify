# Spec 0027 (Results, Grading and Report Cards)

> Feature 29 · Slice 5 (Exams & Results)

## Goal

Provide coaching centers and educational institutions with a comprehensive grading calculation, immutable result snapshot, and professional reporting engine that accurately enforces the Bangladesh National Curriculum (NCTB GPA-5) and British letter scale standards, provides interactive calculation preview before freezing, generates deterministic merit positions, renders bilingual student report cards with authentic academic crests, and prints landscape cross-subject batch tabulation sheets without horizontal truncation.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Grading Scheme Structure** | Pre-seeded NCTB GPA-5 and British GPA-4 with customizable `grading_schemes` and `grade_bands` | Supports both mainstream Bangladeshi curricula (Bangla medium SSC/HSC and English medium Cambridge/Edexcel) |
| **NCTB 4th Subject Bonus** | `bonus = max(0, gp - 200)` added to mandatory GP sum; divisor = count of mandatory subjects | Matches Ministry of Education SSC/HSC mathematical rules; 4th subject failure does not fail the student |
| **Mandatory Fail Hard Gate** | Failing any mandatory subject caps overall GPA to 0.00 (`Grade F`) | Core rule of Bangladeshi secondary education |
| **Immutable Snapshot Model** | Publishing writes `result_snapshots` with frozen JSON schemas, rules, and `results` records | Guarantees that historical marks and report cards never change when grading schemes are edited months later |
| **Republishing Workflow** | Explicit state transition: revert `exams.status` to `verified` (requires `exam.edit_verified`), amend marks, then republish creating `version = 2` | Preserves audit trail while permitting authorized clerical corrections |
| **5-Tier Tie-Breaking** | GPA DESC → Total Marks DESC → Mandatory Marks DESC → Student Roll ASC → Student ID ASC | Guarantees deterministic merit positions across batches and prevents tie-breaking ambiguities |
| **Fixed-Layout Landscape Tabulation** | CSS `@page { size: landscape; margin: 8mm; }` with `table-layout: fixed` and ellipsis name truncation | Guarantees up to 10 subjects fit cleanly on standard A4 paper without horizontal clipping |
| **Bilingual Student Report Card** | A4 portrait layout with academic emblem, subject breakdown, 4th subject math, attendance summary, and 3-signature block | Produces presentation-ready credentials for guardians and parent-teacher meetings |
| **Omnichannel Notification Hook** | Automatic outbox enqueueing for `result_published` automation rule | Delivers student GPA, grade, and rank to guardians over SMS and WhatsApp upon publishing |

## Requirements

**User stories**:
- As an academic coordinator, I want to preview calculated GPAs, letter grades, and ranks before publishing, so that I can inspect class performance and spot any grading anomalies.
- As an institute owner, I want published results frozen into permanent snapshots, so that changing a grading band for a new semester never recalculates or corrupts last semester's student ranks.
- As a parent, I want to receive an authentic, beautifully printed report card in Bengali and English showing my child's subject marks, 4th subject contribution, and teacher signature lines.
- As a headmaster, I want an entire batch tabulation sheet printed on A4 landscape with all subjects visible on one page, so that teachers can review class performance in departmental meetings.
- As an administrator, I want ties broken deterministically using total marks and mandatory subject performance, so that prize winners and merit ranks are undisputed.

## Acceptance Criteria

### AC-1: Grading Scheme Configuration & NCTB Seed
- System pre-seeds the official Bangladesh National Curriculum GPA-5 scheme:
  - `A+` (80.00% – 100.00%, GP 5.00)
  - `A` (70.00% – 79.99%, GP 4.00)
  - `A-` (60.00% – 69.99%, GP 3.50)
  - `B` (50.00% – 59.99%, GP 3.00)
  - `C` (40.00% – 49.99%, GP 2.00)
  - `D` (33.00% – 39.99%, GP 1.00)
  - `F` (0.00% – 32.99%, GP 0.00, fail = 1)
- Supports British Letter Scale (A*, A, B, C, D, E, U; GPA 4.00) and custom institute schemes.

### AC-2: Calculation Preview and 4th Subject Rule Engine
- RPC `exams.preview_results` computes candidate grades:
  - Subject percentage: `(marks / full_marks) * 10000`.
  - Subject pass check: `marks >= pass_marks AND is_absent == 0`.
  - 4th Subject logic:
    - If `is_optional == 1`, `bonus = max(0, gp - 200)`.
    - Fail in 4th subject does not fail student overall.
  - Mandatory fail check:
    - If any mandatory subject has `passed == false`, overall result is `passed = false`, `gpa = 0`, `grade = 'F'`.
  - Overall GPA formula: `min(500, (sum(mandatory_gp) + bonus) / count(mandatory_subjects))`.
- Returns calculation warnings for unverified subjects or missing student records.

### AC-3: Result Publishing and Immutable Freezing
- RPC `exams.publish_results` atomically creates:
  - `result_snapshots`: Captures `exam_id`, `version`, `grading_scheme_json`, `calculation_rules_json`, `published_by`, and `published_at`.
  - `results`: Stores final `total_marks`, `percentage`, `gpa`, `grade`, `passed`, `rank`, and `tie_breaker_json` per student.
  - `result_subject_details`: Stores subject-wise marks, full marks, pass marks, letter grade, and grade points.
  - Updates `exams.status = 'published'`.
- SQLite triggers `trg_results_immutable_update` and `trg_results_immutable_delete` abort any direct modification to published result rows.
- Edits to `grading_schemes` or `grade_bands` after publication have zero effect on existing snapshots.

### AC-4: Deterministic 5-Tier Tie-Breaking
- Class ranking sorts students strictly by:
  1. `gpa` DESC
  2. `total_marks` DESC
  3. `mandatory_total_marks` DESC
  4. `student_roll` ASC
  5. `student_id` ASC
- Guarantees completely deterministic ranks without ties or order shuffling.

### AC-5: Printable Student Report Cards (A4 Bilingual)
- RPC `exams.get_student_report_card` returns complete marksheet payload for single student or batch.
- HTML/CSS template renders standard A4 portrait marksheet:
  - Header: Academic crest emblem, institution name (English & Bengali), contact details, exam title, and academic session.
  - Student block: Student Name, Bengali Name, Roll Number, Student Code, Batch Name, Gender.
  - Grade summary table: Subject, Full Marks, Pass Marks, Marks Obtained, Letter Grade, Grade Point.
  - 4th subject row highlighted with bonus points calculation note.
  - Final tally: Total Marks, Percentage, Final GPA, Final Grade, Merit Position.
  - Signature block: Class Teacher, Academic In-Charge, Principal / Director.
  - Rendered with open-licensed `Kalpurush` and `Noto Sans Bengali` fonts.

### AC-6: Printable Landscape Batch Tabulation Sheet
- RPC `exams.get_tabulation_sheet` returns batch matrix of all students across all exam subjects.
- HTML/CSS template renders A4 Landscape `@page { size: landscape; margin: 8mm; }`:
  - Columns: Roll, Student Name (fixed width with ellipsis), each Subject (Obtained / Grade), Total Marks, GPA, Grade, Merit Rank.
  - Guaranteed zero horizontal truncation for up to 10 participating subjects.

### AC-7: Omnichannel Automation Hook
- When results are published and `result_published` automation rule is active:
  - Automatically enqueues delayed or immediate SMS and WhatsApp messages to primary guardians with student name, exam title, GPA, grade, and rank.
  - Deduplication key: `result:<snapshot_id>:<student_id>:<channel>`.

### AC-8: Structured Domain Errors
- Maps to typed `RpcError` with explicit codes:
  - `grading_scheme_not_found`: Referenced grading scheme does not exist.
  - `exam_not_verified`: Cannot publish exam because subjects are not yet verified.
  - `snapshot_not_found`: Requested result snapshot does not exist.
  - `results_immutable`: Attempted to modify or delete published results.
