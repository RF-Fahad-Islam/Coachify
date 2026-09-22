# 0026. Exams & Mark Entry (Rationale)

## Context

In Bangladeshi coaching centers, academic terms are punctuated by rigorous assessment cycles: weekly model tests, monthly evaluations, and comprehensive semester finals. A typical center evaluates hundreds of students across multiple batches and academic subjects (Bangla, English, Mathematics, Physics, Chemistry, Biology, Accounting, etc.).

Teachers often grade stacks of physical test papers in short turnaround times. Common failure modes in educational software include:
1. **Slow, mouse-dependent data entry**: Requiring a teacher or administrative assistant to click on individual form inputs or table cells to enter scores slows down data entry to hours, causing data entry backlogs.
2. **Absence of range boundaries**: Without strict database and UI bounds, clerical errors like entering `980` instead of `98` or `110` on a 100-mark paper distort class statistics and report cards.
3. **Incomplete verification leaks**: When marks are published with missing entries, absent students may be wrongly marked zero or omitted from tabulation, requiring awkward manual reprints of report cards.
4. **Silent retroactive tampering**: If teachers or staff can freely modify marks after results have been verified or announced, accusations of favoritism or clerical corruption arise.

Coachify solves these operational challenges with a keyboard-driven spreadsheet entry matrix, integer hundredths precision, dual-layer range validation, and strict complete-roster verification gates.

---

## Options Considered

### Option 1: Keyboard-First Matrix with Integer Hundredths, Complete-Roster Verification Gates, and Defense-in-Depth Triggers (Chosen)

Build a dedicated mark entry matrix with spreadsheet ergonomics:
- Keyboard-only navigation (`ArrowUp`, `ArrowDown`, `Enter`, `Tab`) and single-key `A` absent toggling.
- All scores represented as integer hundredths (e.g. `85.50` stored as `8550`).
- Range constraints enforced at the UI boundary and SQLite trigger level (`trg_marks_validate_range`).
- Verification blocked until 100% of enrolled students have marks or are marked absent.
- Verified marks locked against updates without `exam.edit_verified` permission.
- Publishing locks all underlying marks permanently against updates.

Pros:
- Unmatched entry speed: Teachers enter 80 scores in under a minute without touching a mouse.
- Absolute numerical precision with zero floating-point rounding quirks.
- Prevents clerical errors and out-of-range marks from ever entering the system.
- Enforces institutional integrity: published marks cannot be quietly altered.

Cons:
- Requires custom keyboard handling and focus management in React table cells.

### Option 2: Generic Form Inputs with Floating Point Decimals and Unconstrained Entry

Use standard HTML numeric form inputs (`<input type="number" step="0.1">`) with floating-point numbers in SQLite (`REAL`), allowing partial entry and unverified publishing.

Pros:
- Simpler initial UI implementation.

Cons:
- Floating point inaccuracies (e.g. `85.1` stored as `85.09999999999999`) cause display glitches and rounding errors during percentage calculation.
- Mouse-dependent entry creates user frustration during heavy grading sessions.
- Incomplete marks cause erroneous zero-scores and missing report cards.

### Option 3: Bulk CSV / Excel File Import Only

Require teachers to enter marks into offline spreadsheets and import them via CSV or Excel files, omitting direct in-app mark entry.

Pros:
- Offloads UI grid complexity to external spreadsheet software.

Cons:
- Broken workflow for centers where teachers grade and input marks directly on center PCs.
- Mismatched student codes or malformed rows in external spreadsheets create repeated import validation friction.
- Lacks real-time verification gates and instant range checks.
