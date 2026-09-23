# 0033. Owner intelligence (verification plan)

## Overview

This verification plan specifies the automated tests, benchmark assertions, and manual verification steps required to validate Slice 7 Owner Intelligence before marking the slice complete.

## Automated tests

### 1. Database schema and migration verification

* Run `cargo run -p xtask check-migrations` to verify Migration 0017 integrity.
* Verify tables `batch_cost_configs`, `student_timeline_notes`, and `due_collection_contact_logs` are created with foreign key cascades and check constraints.

### 2. Timeline aggregation and query correctness

* Integration suite `crates/core/tests/owner_intelligence_integration.rs`:
  * `test_student_360_timeline_aggregates_all_categories`: Verifies events across admission, enrollments, fee payments, attendance sessions, exam marks, outbound messages, and staff notes appear in chronological order.
  * `test_student_360_category_filtering_and_cursor_paging`: Verifies filtering by financial or academic categories returns only matching event types with cursor pagination.
  * `test_coaching_day_timeline_with_actor_attribution`: Verifies events recorded during the day include the performing user display name.

### 3. Today operations cockpit alerts

* `test_today_cockpit_missing_attendance_detection`: Seeds batch scheduled for today with class end time in the past. Asserts batch appears in `missing_attendance_batches` when no attendance session exists, and disappears immediately once attendance is submitted.
* `test_today_cockpit_upcoming_exams`: Verifies exams scheduled in the upcoming week appear with enrolled candidate counts.
* `test_today_cockpit_counter_collections_reconciliation`: Verifies sum of collections today matches payment ledger totals exactly.

### 4. Batch health and profitability calculations

* `test_batch_health_diagnostics_and_thresholds`: Verifies capacity utilization percentage, thirty day attendance rate, and health flag string (healthy, warning, critical) for batches crossing warning thresholds.
* `test_batch_profitability_margin_and_breakeven`: Verifies net margin poisha and breakeven flag when teacher and room costs exceed or remain under collected revenues.

### 5. Smart due collection queue ranking

* `test_smart_due_queue_weighted_scoring`: Verifies student with longer overdue balance and no recent contact ranks ahead of student contacted yesterday with promised payment date.
* `test_record_due_contact_updates_queue_priority`: Verifies logging a contact event updates priority score and last contact date immediately.

## Performance benchmarks

* **Sub second student timeline**: Querying a student with three years of history (over 500 total events) executes in under 1000 ms on reference hardware.
* **Sub 500 ms cockpit and batch health**: Cockpit summary and batch health calculations execute in under 500 ms on the 10,000 student reference dataset.

## Manual verification

1. Navigate to Student 360 view from student profile; verify timeline renders chronological history with category badges and print button.
2. Open Today Operations Cockpit; confirm missing attendance alert shows batches whose class end time has passed.
3. Open Batch Health dashboard; verify capacity and attendance progress bars reflect accurate batch figures.
4. Open Batch Profitability; test adjusting teacher cost slider and verify net margin updates instantly.
5. Open Smart Due Collection Queue; click cashier shortcut and verify cashier counter opens with student preselected.
