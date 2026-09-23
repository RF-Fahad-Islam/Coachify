# 0033. Batch health view

> Feature 37 (Slice 7: Owner intelligence)

## Summary

This specification defines the Batch health view, providing a comprehensive diagnostic dashboard across all batches. It highlights capacity utilization, thirty day attendance trends, dues totals, dues collection percentages, exam grade point averages, assigned teachers, class schedules, and inactive student counts on a single screen with automatic visual warning indicators.

## Requirements

**User stories**:
* As an owner, I want to see which batches are underperforming in attendance or accumulating excessive dues so that we can intervene before student dropout increases.
* As an academic coordinator, I want to view capacity utilization across classrooms so that we can consolidate underfilled batches or schedule additional sections.
* As a head teacher, I want to compare exam averages across batches within the same program to ensure curriculum consistency.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_batch_health_list` computes diagnostics across all batches in under 500 ms.
* **AC-2**: Health status assigns critical when two or more thresholds fail (attendance below 60 percent, dues above 30 percent, capacity below 50 percent) or attendance is below 50 percent; assigns warning when one threshold fails; assigns healthy otherwise.
* **AC-3**: Tracks inactive student counts per batch, defined as students with inactive enrollment status or zero attendance sessions over the last thirty days.
* **AC-4**: Clicking any batch navigates directly to its detailed health profile and class roster.
* **AC-5**: Supports filtering by academic program and school session year.

## Inline rationale

Direct SQL aggregations over `enrollments`, `attendance_records`, `ledger_charges`, and `exam_results` give instantaneous calculations without stale materialized cache rows. Safe division helpers prevent divide by zero errors when batches have zero enrolled students or zero scheduled classes.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_batch_health_list | query | program_id: Option<String>, session: Option<String> | Vec<BatchHealthItem> | Owner, Admin, Teacher | program_not_found |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_batch_health_list | capacity utilization percentage | Active student count divided by batch capacity |
| get_batch_health_list | thirty day attendance rate | Total present marks divided by total scheduled attendance opportunities in last 30 days |
| get_batch_health_list | dues ratio percentage | Unpaid charges divided by total billed charges for batch |
| get_batch_health_list | inactive student count | Students with enrollment status inactive or zero attendance sessions in last 30 days |
| get_batch_health_list | health flag status | critical if 2 or more thresholds fail or attendance < 50%; warning if 1 fails; healthy otherwise |

## Build tasks

1. Implement core aggregation query computing capacity, attendance, dues, inactive counts, and exam metrics per batch, satisfies **AC-1**, **AC-2**, **AC-3**
2. Add threshold evaluator returning health flag string (healthy, warning, critical), satisfies **AC-2**
3. Create React batch health dashboard with summary cards, warning badges, and program filters, satisfies **AC-1** through **AC-5**
