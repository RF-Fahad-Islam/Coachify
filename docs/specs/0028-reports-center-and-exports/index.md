# Spec 0028 (Reports Center & Exports)

> Feature 30 · Slice 6 (Reports & Speed)

## Goal

Provide coaching center owners, accountants, and academic administrators with a centralized analytical reporting suite that delivers sub-2-second insights across financial collections, dues aging buckets, cash drawer reconciliation, batch attendance registers with chronic absenteeism detection, and exam merit distributions, accompanied by one-click RFC 4180 CSV exports prepended with UTF-8 BOM for seamless Microsoft Excel compatibility on Windows, and high-fidelity print layouts.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **SQL-First Aggregations** | Direct SQLite indexed aggregations over in-memory dataset scanning | Guarantees sub-2s query latency on reference hardware even with 10,000+ students and 50,000+ ledger entries |
| **Bilingual CSV Encoding** | Prepend UTF-8 Byte Order Mark (`\u{FEFF}`) to all CSV export streams | Windows Excel defaults to legacy ANSI/system code page unless BOM is present; BOM guarantees Bengali script renders cleanly without mojibake |
| **Aging Bucket Segmentation** | Standard 30-day tiers: Current (0–30d), 31–60d, 61–90d, 90+d overdue | Aligns with coaching fee collection cycles where monthly fees become increasingly difficult to collect past 60 days |
| **Chronic Absenteeism Threshold** | Fixed 75% attendance threshold with visual warning badges | Matches national educational standards in Bangladesh where < 75% attendance bars students from board exams |
| **Cash Drawer Reconciliation** | Explicit expected vs. counted variance calculation with notes | Prevents cashier shrinkage, tracks petty expenses against collections, and produces audited daily close statements |
| **Print Styling** | Dedicated `@media print` CSS rules paired with `--font-bangla` typography | Allows instant printing to A4 laser or thermal printers directly from the reports interface without external PDF generation overhead |

## Requirements

**User stories**:
- As a coaching center owner, I want to view daily and date-range fee collection summaries broken down by payment method (cash, bKash, Nagad, bank), so that I can audit daily cash flows against digital mobile wallets.
- As an accountant, I want an aging dues report categorized into 30-day buckets with guardian phone numbers, so that front-desk staff can conduct targeted telephone recovery calls.
- As an administrator, I want to export any report table to CSV and have it open immediately in Excel with perfect Bengali names, so that I can prepare custom off-app spreadsheets for board meetings.
- As a head teacher, I want a monthly attendance register highlighting students with less than 75% attendance, so that we can intervene with guardians before semester exams.
- As a cashier supervisor, I want collector-wise breakdown of payments and voids, so that discrepancies at the counter can be traced to the specific logged-in user.

## Acceptance Criteria

### AC-1: Collection Summary & Method Breakdown
- RPC `reports.get_collection_summary` accepts date range (`start_date`, `end_date`), optional `collector_id`, and optional `payment_method`.
- Returns aggregated metrics:
  - `total_collected`: Gross collected in integer hundredths.
  - `net_collected`: Gross minus refunded and voided transactions.
  - `payment_count`: Total valid payments.
  - `void_count`: Number of voided receipts.
  - Method distribution: Cash, bKash, Nagad, Bank transfer amounts.
  - Daily breakdown series for charting/tabular timeline.

### AC-2: Collector & Cashier Performance Breakdown
- RPC `reports.get_collector_summary` groups payment records by `created_by_user_id`.
- Returns username, display name, total payments taken, total cash collected, digital payments collected, voided count, and voided amount.
- Reconciles 100% with ledger entries in tests.

### AC-3: Dues Aging & Recovery Report
- RPC `reports.get_dues_aging_report` computes outstanding student balances from charges, payments, and adjustments.
- Categorizes overdue balances into:
  - Current (0–30 days)
  - 31–60 days
  - 61–90 days
  - 90+ days
- Includes student code, student name (English & Bengali), active batch name, and primary guardian phone number for direct contact.

### AC-4: Expense & Cash Drawer Reconciliation
- RPC `reports.get_expenses_summary` aggregates petty cash and center expenses by category across a date range.
- Integrates with `cash_days` records to present:
  - Starting cash.
  - Cash collections.
  - Cash expenses.
  - Expected drawer balance vs. actual counted cash.
  - Cashier discrepancy/over-short amount.

### AC-5: Academic Attendance & Chronic Absenteeism
- RPC `reports.get_attendance_summary` aggregates student attendance by batch and month (`YYYY-MM`).
- Calculates present %, absent count, late count, and excused count.
- Flags any student with attendance < 75.0% as `is_chronic_absentee = true`.

### AC-6: Exam Performance & Merit Summary
- RPC `reports.get_exam_performance` analyzes published result snapshots.
- Computes batch GPA average, total candidates, passed count, failed count, pass percentage, and letter grade distribution (`A+`, `A`, `A-`, `B`, `C`, `D`, `F`).
- Lists top 5 rankers with GPA and total marks.

### AC-7: UTF-8 BOM CSV Export Engine
- RPC `reports.export_csv` serializes report datasets into RFC 4180 compliant CSV format.
- Prefixes byte stream with standard UTF-8 Byte Order Mark (`0xEF, 0xBB, 0xBF`).
- Dates formatted as `YYYY-MM-DD`, monetary values converted from poisha to standard decimal format (`0.00`).
- Tested to open cleanly in Microsoft Excel on Windows without character corruption of Bengali script.

### AC-8: Sub-2-Second Performance Benchmark
- Every reporting query executes in under 2000 ms on the reference dataset (10,000 students, 50,000 ledger entries) on reference PC hardware.
