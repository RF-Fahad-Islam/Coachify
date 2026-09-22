# 0033. Owner intelligence (umbrella architecture)

**Date**: 2026-09-23
**Status**: Proposed

## Summary

This specification establishes the architectural foundation for Owner Intelligence (Slice 7) in Coachify. It defines how Coachify synthesizes operational, academic, and financial data into immediate insights for coaching center owners and administrators without adding extra data entry. All intelligence views rely on on demand SQLite queries across existing local records, supported by Migration 0017 for configurable batch costs, student timeline notes, and due collection contact logs.

## Structure

This umbrella decision organizes the six operational intelligence features of Slice 7 into linked child specifications:

* [0033-student-360-timeline.md](0033-student-360-timeline.md): Feature 34 (Student 360 timeline). Chronological student history combining admissions, enrollment shifts, payments, attendance sessions, exam marks, outbound messages, and staff notes.
* [0033-today-operations-cockpit.md](0033-today-operations-cockpit.md): Feature 35 (Today operations cockpit). Single screen daily command center covering today's classes, missing attendance alerts, overdue balances, counter collections, absent teachers, upcoming exams, message delivery failures, and backup health.
* [0033-coaching-day-timeline.md](0033-coaching-day-timeline.md): Feature 36 (Coaching day timeline). Institution wide chronological event stream of all activities happening during the center day with actor attribution and category filters.
* [0033-batch-health-view.md](0033-batch-health-view.md): Feature 37 (Batch health view). Holistic batch diagnostics showing capacity utilization, thirty day attendance trends, dues ratios, exam grade distributions, and inactive student counts.
* [0033-batch-profitability.md](0033-batch-profitability.md): Feature 38 (Batch profitability). Financial sustainability analysis comparing collected and projected tuition revenue against teacher compensation, room rent allocation, and operational overhead.
* [0033-smart-due-collection-queue.md](0033-smart-due-collection-queue.md): Feature 39 (Smart due collection queue). Priority ranked collection queue scoring overdue students by aging duration, balance size, reminder history, and previous payment cadence.

## Cross child contract

All six child features share a common set of read models, database tables, and interface patterns:

1. **Zero separate event storage**: Historical event streams (Student 360 and Coaching Day Timeline) are computed through virtual SQL union queries directly from primary domain tables (`admissions`, `enrollments`, `payments`, `attendance_records`, `marks`, `sms_outbox`, `student_timeline_notes`, `due_collection_contact_logs`, and `audit_log`).
2. **Migration 0017 schema additions**:
   * `batch_cost_configs`: Stores monthly fixed or per class teacher costs, room allocations, and overhead parameters per batch.
   * `student_timeline_notes`: Stores staff observations, counseling remarks, and academic notes attached to a student.
   * `due_collection_contact_logs`: Records phone calls, in person visits, SMS reminders, and promised payment dates with actor tracking.
3. **Unified timeline event structure**: A shared Rust type `TimelineEvent` provides a normalized envelope (id, event_time, category, event_type, actor_name, title, description, metadata_json) for both student specific and center wide feeds.
4. **Sub second latency budget**: All queries must complete in under 1000 ms on the reference dataset (10,000 students, 350 batches, 50,000 ledger records).
5. **Integer poisha arithmetic**: All currency values (dues, collections, teacher costs, room allocations, profit margins) are computed and returned as 64 bit signed integers in poisha (100 poisha = 1 Taka).
6. **Cursor pagination**: Timeline feeds accept optional cursor parameters (`before_time: Option<String>`, `before_id: Option<String>`) to guarantee steady performance during deep scrolling without offset penalties.

## Requirements

**User stories**:
* As a coaching center owner, I want a single command center to see today's operations, revenue, and alerts without digging through multiple screens.
* As an owner, I want to inspect any student's full chronological journey from initial admission to recent exam results and payment vouchers on one page.
* As an academic coordinator, I want to see which batches have low attendance, high dues, or underutilized capacity so we can intervene early.
* As an administrator, I want to know which batches generate positive operating margins after paying teacher fees and room expenses.
* As a front desk accountant, I want a prioritized queue of overdue students showing who to contact first with one click links to send SMS or open the cashier.

**Acceptance criteria**:
* **AC-1**: `intelligence.get_student_360` returns complete chronological history across academic, financial, attendance, communication, and note events for a student within 1000 ms.
* **AC-2**: `intelligence.get_today_cockpit` delivers immediate operational metrics for today, including classes scheduled, counter collections, overdue totals, upcoming exams, absent teachers whose scheduled classes ended without attendance, and missing attendance alerts.
* **AC-3**: `intelligence.get_coaching_day_timeline` provides a filterable chronological activity feed for any selected date with actor user attribution.
* **AC-4**: `intelligence.get_batch_health_list` computes capacity utilization percentage, thirty day attendance rate, dues ratio, inactive student counts, and flags batches failing defined thresholds (critical when two or more thresholds are breached or attendance is below 50 percent, warning when one threshold is breached).
* **AC-5**: `intelligence.get_batch_profitability` computes projected monthly tuition revenue, net collected revenue, teacher cost (including percentage shares calculated on net collections), room overhead, net margin poisha, and breakeven status.
* **AC-6**: `intelligence.update_batch_cost_config` allows Owner and Admin users to update cost assumptions for any batch, recalculating profitability dynamically.
* **AC-7**: `intelligence.get_smart_due_queue` outputs prioritized students using a multi factor score (overdue duration from oldest unpaid charge, unpaid balance, days since last contact, combined SMS and call reminder count), with primary guardian phones and one click cashier and reminder actions.
* **AC-8**: `intelligence.record_due_contact` logs contact attempts, channel, outcome, and optional promised payment date, updating the student's priority score immediately.

## Decision

**Chosen option**: On demand unified query view joining existing domain tables, paired with Migration 0017 dedicated operational configurations and collection tracking tables.

This decision avoids duplicate event storage, guarantees absolute consistency with primary records, and introduces minimal write overhead while fulfilling sub second analytics requirements.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`)

## Rationale

Reasoning and options: see rationale.md

## Feature design

### Data model sketch (Migration 0017)

```sql
-- Batch operational and cost configurations for profitability
CREATE TABLE IF NOT EXISTS batch_cost_configs (
    id TEXT PRIMARY KEY NOT NULL,
    batch_id TEXT NOT NULL UNIQUE REFERENCES batches(id) ON DELETE CASCADE,
    teacher_monthly_cost INTEGER NOT NULL DEFAULT 0,
    teacher_cost_type TEXT NOT NULL CHECK (teacher_cost_type IN ('monthly_fixed', 'per_class', 'percentage_share')),
    room_monthly_cost INTEGER NOT NULL DEFAULT 0,
    operational_overhead_cost INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Staff notes and counseling observations on students
CREATE TABLE IF NOT EXISTS student_timeline_notes (
    id TEXT PRIMARY KEY NOT NULL,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    author_id TEXT NOT NULL REFERENCES users(id),
    note_type TEXT NOT NULL CHECK (note_type IN ('academic', 'behavior', 'financial', 'general')),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_student_notes_student ON student_timeline_notes(student_id, created_at DESC);

-- Follow up logs and promise tracking for overdue due collection
CREATE TABLE IF NOT EXISTS due_collection_contact_logs (
    id TEXT PRIMARY KEY NOT NULL,
    student_id TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    contacted_by_user_id TEXT NOT NULL REFERENCES users(id),
    contact_channel TEXT NOT NULL CHECK (contact_channel IN ('phone_call', 'sms', 'in_person', 'whatsapp')),
    outcome TEXT NOT NULL CHECK (outcome IN ('promised_payment', 'dispute', 'unreachable', 'wrong_number', 'follow_up_needed', 'resolved')),
    promised_payment_date TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_contact_logs_student ON due_collection_contact_logs(student_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contact_logs_created ON due_collection_contact_logs(created_at DESC);
```

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_student_360 | query | student_id: String, category: Option<String>, before_time: Option<String>, before_id: Option<String>, limit: Option<u32> | Student360Response | Any authenticated | student_not_found, invalid_category |
| intelligence.add_student_note | command | student_id: String, note_type: String, title: String, content: String | StudentTimelineNote | Owner, Admin, Teacher, Receptionist | student_not_found, invalid_note_type |
| intelligence.get_today_cockpit | query | target_date: Option<String> | TodayCockpitResponse | Owner, Admin | invalid_date_format |
| intelligence.get_coaching_day_timeline | query | date: String, filter_category: Option<String>, before_time: Option<String>, before_id: Option<String>, limit: Option<u32> | CoachingDayTimelineResponse | Owner, Admin | invalid_date_format |
| intelligence.get_batch_health_list | query | program_id: Option<String>, session: Option<String> | Vec<BatchHealthItem> | Owner, Admin, Teacher | program_not_found |
| intelligence.get_batch_profitability | query | batch_id: Option<String>, session: Option<String> | Vec<BatchProfitabilityItem> | Owner, Admin | batch_not_found, unauthorized |
| intelligence.update_batch_cost_config | command | batch_id: String, teacher_monthly_cost: i64, teacher_cost_type: String, room_monthly_cost: i64, operational_overhead_cost: i64 | BatchCostConfig | Owner, Admin | batch_not_found, negative_cost |
| intelligence.get_smart_due_queue | query | batch_id: Option<String>, min_overdue_days: Option<u32>, limit: Option<u32>, offset: Option<u32> | SmartDueQueueResponse | Owner, Admin, Accounts | invalid_parameters |
| intelligence.record_due_contact | command | student_id: String, contact_channel: String, outcome: String, promised_payment_date: Option<String>, notes: String | DueContactLogItem | Owner, Admin, Accounts, Receptionist | student_not_found, invalid_outcome |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_student_360 | timeline events list | SQL union over payments, admissions, marks, attendance, outbox, and notes |
| get_student_360 | student academic GPA average | Derived from published exam results where student participated |
| get_student_360 | total paid and outstanding dues | Computed from ledger charges and payments allocations |
| get_today_cockpit | missing attendance alert | Active batches scheduled today whose end time has passed without attendance session |
| get_today_cockpit | absent teachers list | Teachers assigned to batches in batch_teachers whose scheduled class ended today without attendance submission |
| get_today_cockpit | upcoming exams list | Exams table where exam_date >= target_date and exam_date <= target_date + 7 days |
| get_today_cockpit | counter collections today | Sum of valid payments where paid_at >= start_of_day AND paid_at < end_of_day |
| get_today_cockpit | overdue dues total | Aggregated unpaid balance where charge due date is before today |
| get_coaching_day_timeline | event actor display name | Joined from users table using created_by or actor user ID |
| get_batch_health_list | capacity utilization percentage | Active enrollments count divided by batch capacity multiplied by 100 |
| get_batch_health_list | thirty day attendance rate | Present attendance records divided by total scheduled sessions in last 30 days |
| get_batch_health_list | inactive student count | Students with enrollment status inactive or zero attendance in last 30 days |
| get_batch_health_list | health flag status | critical if 2 or more thresholds fail or attendance < 50%; warning if 1 fails; healthy otherwise |
| get_batch_profitability | projected monthly revenue | Sum of active monthly tuition fee plan charge amounts for enrolled students |
| get_batch_profitability | teacher cost calculation | monthly_fixed uses fixed poisha; per_class multiplies by classes held; percentage_share calculates percentage of net collections |
| get_batch_profitability | net margin poisha | Net collected revenue minus sum of teacher, room, and operational overhead costs |
| get_smart_due_queue | primary guardian phone | Guardians table joined via student_guardians where is_primary = 1 |
| get_smart_due_queue | overdue days count | Days between due_date of student's oldest unpaid charge and today |
| get_smart_due_queue | reminder count | Count of sms_outbox messages plus due_collection_contact_logs records for student |
| get_smart_due_queue | priority score | (overdue_days * 1.5) + (due_poisha / 100000.0) - (days_since_contact * 2.0) + (reminder_count * 5.0) |

### Key invariants

1. **Deterministic timeline sorting**: All timeline feeds sort strictly by `event_time DESC, id DESC`, ensuring stable pagination with cursor tuples `(event_time, id)` without duplicate or missed items.
2. **Zero mutation of primary ledger**: Profitability calculations and cost projections never modify ledger charges or payments. Cost parameters are purely analytical projections.
3. **Index friendly range queries**: All date filtering utilizes ISO range comparisons (`paid_at >= ? AND paid_at < ?`) rather than SQL scalar wrappers like `date(paid_at)`, preserving B-tree index scans.
4. **Safe division guards**: Any ratio calculation (attendance percentage, capacity utilization, dues ratio, profit margin) guards against zero denominators, defaulting safely to zero percent.
5. **Timezone consistency**: All date comparisons utilize the coaching center's local date (`YYYY-MM-DD`) derived from application system time.
6. **Access control**: Batch profitability figures and operational margins are accessible strictly to Owner and Admin roles.

### Security model

* **Owner**: Full access to all intelligence queries, profitability figures, cost configurations, notes, and collection logs.
* **Admin**: Full access to operational cockpit, timelines, batch health, profitability, and due queue management.
* **Cashier / Accounts**: Access to Smart Due Queue and Student 360 financial history. No access to batch profitability margins.
* **Teacher**: Read access to assigned batch health and student academic history. No access to financial health, profitability, or billing queues.
* **Receptionist**: Read access to Student 360 and write access to student timeline notes and contact logs.

### Critical test scenarios

* Happy path: Loading Student 360 returns ordered events across admission, payment, attendance, exam, and note categories, verifies **AC-1**
* Operational check: Today Cockpit flags batch whose class end time passed without attendance record submitted today, verifies **AC-2**
* Upcoming exams: Today Cockpit surfaces exams scheduled in the upcoming week with candidate counts, verifies **AC-2**
* Timeline filtering: Coaching Day Timeline returns chronological events filtered by category for selected historical date, verifies **AC-3**
* Batch health threshold: Batch with 55% attendance and 35% dues displays warning and critical badges, verifies **AC-4**
* Cost modeling: Updating batch cost config updates projected margin and breakeven indicator immediately, verifies **AC-5**, **AC-6**
* Priority queue order: Student with older dues and no recent contact ranks higher than recently contacted student, verifies **AC-7**
* Contact logging: Recording a promised payment date logs contact history and updates queue item immediately, verifies **AC-8**
* Permission denial: Cashier or Teacher attempting to fetch batch profitability receives unauthorized error, verifies **AC-5**

## Build plan

1. Create Migration 0017 defining `batch_cost_configs`, `student_timeline_notes`, and `due_collection_contact_logs` tables with composite indexes, satisfies **AC-6**, **AC-8**
2. Implement core timeline aggregation engine in `crates/core/src/intelligence.rs` with unified SQL query across domain tables and cursor pagination, satisfies **AC-1**, **AC-3**
3. Implement today operational cockpit query engine detecting completed batch schedules with missing attendance, upcoming exams, collections and dues totals, satisfies **AC-2**
4. Implement batch health and profitability calculation routines with safe division guards, percentage share math, and margin calculations, satisfies **AC-4**, **AC-5**, **AC-6**
5. Implement smart due collection ranking algorithm in Rust with contact log integration and cashier navigation payloads, satisfies **AC-7**, **AC-8**
6. Wire dispatcher RPC endpoints under `intelligence.*` with strict role checks, export TypeScript contracts, satisfies **AC-1** through **AC-8**
7. Build integration test suite in `crates/core/tests/owner_intelligence_integration.rs` covering all calculation scenarios and benchmarks, satisfies **AC-1** through **AC-8**
8. Build UI views for Student 360, Today Cockpit, Coaching Day Feed, Batch Diagnostics, and Smart Due Queue using Tailwind CSS v4, satisfies **AC-1** through **AC-8**

## Consequences

**Positive**:
* Owners obtain immediate visibility into center health, missing attendance, and daily cash flow without manual auditing.
* Front desk staff can recover dues systematically based on transparent priority scores rather than guesswork.
* Batches operating at financial losses are identified automatically, allowing schedule adjustments or fee updates.
* Zero data duplication ensures intelligence views never drift out of sync with operational tables.

**Negative / tradeoffs**:
* On demand union queries across multiple heavy tables require well tuned SQLite indexes to maintain sub second performance.
* Profitability estimates rely on owner entered cost parameters rather than double entry accounting expense allocations.

**Neutral**:
* Adds Migration 0017 with three new tables and two composite indexes.
* Introduces `intelligence.*` RPC namespace in core dispatcher.

## Follow-up

- [ ] Run `/develop` to implement Migration 0017 and core intelligence routines.
- [ ] Connect printable student summary view with existing print engine for one page report card and timeline printouts.
