# Verify: Reports Center & Exports (spec 0028)

Steps derived from spec 0028 acceptance criteria.

## UI and Manual Verification

- [ ] Navigate to `/reports` in the application navigation rail.
- [ ] Select the **Collections & Revenue** tab: choose a date range (e.g., "This Month"). Verify that gross collection, payment count, and method breakdown (Cash, bKash, Nagad, Bank) calculate accurately and match the cashier drawer totals (verifies AC-1).
- [ ] Select the **Collector Performance** sub-report: verify that individual cashiers/users are listed with their specific payment counts, collected amounts, and void records (verifies AC-2).
- [ ] Select the **Dues & Aging** tab: verify that students with unpaid balances appear categorized into Current, 31–60d, 61–90d, and 90+d overdue buckets, with primary guardian phone numbers displayed (verifies AC-3).
- [ ] Select the **Expenses & Cash Close** tab: verify that petty cash expenses and daily cash close reconciliations display matching drawer balances and discrepancy logs (verifies AC-4).
- [ ] Select the **Attendance Register** tab: pick a batch and month. Verify that attendance percentages compute accurately and students below 75% are tagged with chronic absenteeism badges (verifies AC-5).
- [ ] Select the **Exam Merit & Performance** tab: pick an exam. Verify that candidate counts, pass rate, average GPA, and grade band distributions (A+, A, etc.) render properly (verifies AC-6).
- [ ] Click **[Export to CSV / Excel]** on a report containing Bengali student names: open the downloaded CSV file directly in Microsoft Excel on Windows. Verify that Bengali text displays without corruption or encoding dialogs (verifies AC-7).
- [ ] Benchmark report loading on the reference dataset: verify that all queries render in under 2000 ms (verifies AC-8).

## Commands

- [ ] `cargo test -p coachify_core --test reports_and_exports` (verifies SQL aggregation queries, aging bucket mathematics, chronic absentee thresholds, and UTF-8 BOM CSV byte sequences, satisfies AC-1 through AC-8).
- [ ] `cargo run -p xtask check-types` (verifies that TypeScript contracts for `reports.*` match Rust definitions).
- [ ] `pnpm --dir ui test` (verifies reports tab navigation, filter controls, KPI card calculations, and CSV download invocation).

## Acceptance Criteria Coverage

- AC-1 (Collection Summary): verified by `test_collection_summary_calculation` in core and UI summary cards.
- AC-2 (Collector Performance): verified by `test_collector_wise_reconciliation` matching ledger entries.
- AC-3 (Dues Aging Report): verified by `test_dues_aging_buckets_30_60_90` test case.
- AC-4 (Expense & Cash Reconciliation): verified by `test_cash_day_drawer_reconciliation`.
- AC-5 (Attendance & Chronic Absenteeism): verified by `test_monthly_attendance_chronic_threshold`.
- AC-6 (Exam Performance Summary): verified by `test_exam_batch_performance_metrics`.
- AC-7 (UTF-8 BOM CSV Export): verified by asserting first 3 bytes of export stream equal `[0xEF, 0xBB, 0xBF]`.
- AC-8 (Sub-2s Benchmark): verified by timer assertion on demo load database.
