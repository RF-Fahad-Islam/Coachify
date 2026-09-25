# 0039. CSV / Excel Export

> Status: Ratified
> Feature: 52 (Slice 9 — Printing & Localization)
> Date: 2026-09-25

## Overview

Coaching center owners and administrators frequently export operational tables into Microsoft Excel or Google Sheets for board meetings, accountant reconciliation, or offline analysis. Feature 52 expands the CSV/Excel export engine (`reports.export_csv` in `crates/core/src/reports.rs` and `ui/src/lib/export-csv.ts`) so every major list view and report supports filter-aware, Excel-compatible export with full Bangla Unicode preservation and formula-injection safety.

## Supported Export Domains

1. **Student Directory (`students_list`)**: Respects active `search_query`, `batch_id`, `status`, and `fee_status` filters. Columns: Student ID, Full Name, Name (Bangla), Gender, Phone, Primary Guardian, Guardian Phone, Active Batches, Status, Net Due (৳).
2. **Dues Center (`dues_list`)**: Respects active `batch_id`, `min_due_poisha`, and `search_query` filters. Columns: Student ID, Student Name, Name (Bangla), Batch, Guardian Name, Guardian Phone, Total Charged (৳), Total Paid (৳), Net Due (৳).
3. **Exam Results (`exam_results_list`)**: Respects active `exam_id` and `batch_id`. Columns: Merit Rank, Student ID, Student Name, Name (Bangla), Total Marks, Highest/Obtained Marks, GPA, Grade, Result Status.
4. **Attendance Reports (`attendance`)**: Respects `batch_id` and `month` (`YYYY-MM`). Columns: Student ID, Student Name, Name (Bangla), Batch, Sessions, Present, Late, Absent, Excused, Attendance %, Chronic Absentee.
5. **Fee & Collection Reports (`collection_summary`, `collector_summary`, `dues_aging`, `expenses`)**: Respects active date range, payment method, and collector filters.

## Acceptance Criteria

- **AC-1 (Universal List Export Buttons)**: Export buttons are present and functional on Student Directory, Dues Center, Exam Results, Attendance Reports, and Financial Reports.
- **AC-2 (Excel Bangla Unicode Preservation)**: Every exported CSV file begins with the UTF-8 Byte Order Mark (`\u{FEFF}`) and uses RFC 4180 CRLF (`\r\n`) line endings so Microsoft Excel on Windows opens Bangla names and `৳` symbols without mojibake.
- **AC-3 (Filter State Fidelity)**: Exporting from any list view passes the active filter state (`batch_id`, `status`, `search_query`, `exam_id`, date range) to `reports.export_csv` so only matching records are exported.
- **AC-4 (CSV Formula Injection Defense)**: Cell values starting with `=`, `+`, `-`, or `@` (unless a valid signed number) are sanitized with a leading single quote (`'`) to prevent spreadsheet formula execution.
- **AC-5 (High-Volume Performance)**: Exporting 10,000+ rows completes in under 500ms in Rust core without freezing the UI thread.

## Build Plan

1. Expand `crates/core/src/reports.rs` `export_csv` with `students_list`, `dues_list`, and `exam_results_list` handlers plus CSV formula-injection defense in `escape_csv_cell`.
2. Create `ui/src/lib/export-csv.ts` helper for triggering `reports.export_csv` and downloading `.csv` files with automatic filename timestamps.
3. Wire Export CSV/Excel buttons into `StudentDirectoryView`, `DuesCenterView`, `ResultPublishView`, and `ReportsView`.
4. Add Rust integration tests (including 10,000-row benchmark and Bangla + formula-injection verification) and Vitest UI tests.
