# Owner intelligence (Slice 7)

Back to [index](index.md). The coaching center owner's operational brain. These features surface what matters without the owner having to dig. All read from existing data; no new data entry required.
Umbrella spec [0033](../specs/0033-owner-intelligence/index.md)

### 34. Student 360 timeline
Entire student journey in chronological form: admission, enrollment changes, payments, attendance, exam results, messages sent, status changes, and notes.
**Done when:** the timeline loads within 1 second for a 3 year student; every financial, academic, and administrative event appears; the timeline can be printed as a one page student report.
spec [0033](../specs/0033-owner-intelligence/0033-student-360-timeline.md)
- [x] Design it (spec): `/architect student 360 timeline`
- [x] Build it: `/develop student 360 timeline`
  - [x] Migration 0017 `student_timeline_notes` table and indexes, covers AC-4
  - [x] Core SQLite timeline union query across admissions, payments, marks, attendance, outbox, and notes, covers AC-1, AC-2, AC-3
  - [x] React Student 360 timeline UI with category filters, cursor pagination, and printable summary, covers AC-1, AC-5
- [x] Verify it: `/check verify student 360 timeline`
- [x] Test it: `/test student 360 timeline`

### 35. Today operations cockpit
Single view for the owner or admin showing today's classes, missing attendance flags, overdue dues, today's collections, absent teachers, upcoming exams, message failures, and backup status. The daily command center.
**Done when:** the cockpit loads fresh on every visit; each widget links to its detail view; the owner can see at a glance if anything needs immediate attention; missing attendance is flagged when scheduled class end time has passed.
spec [0033](../specs/0033-owner-intelligence/0033-today-operations-cockpit.md)
- [x] Design it (spec): `/architect today operations cockpit`
- [x] Build it: `/develop today operations cockpit`
  - [x] Core scheduled class and missing attendance evaluator for classes whose end time has passed today, covers AC-1, AC-2, AC-4
  - [x] Aggregator for collections today, overdue totals, upcoming exams, and backup health, covers AC-3, AC-5, AC-6
  - [x] React Today Operations Cockpit dashboard with live refresh and quick action links, covers AC-1 through AC-6
- [x] Verify it: `/check verify today operations cockpit`
- [x] Test it: `/test today operations cockpit`

### 36. Coaching day timeline
Chronological feed of the day's events: admissions, payments, attendance submissions, exam publications, messages sent, settings changes, and staff activity. A living record of what happened at the center today.
**Done when:** every business event appears in the timeline within seconds of occurring; the feed can be filtered by type; past days can be viewed; the timeline shows who performed each action.
spec [0033](../specs/0033-owner-intelligence/0033-coaching-day-timeline.md)
- [x] Design it (spec): `/architect coaching day timeline`
- [x] Build it: `/develop coaching day timeline`
  - [x] Core daily event aggregation joining audit logs with domain transactions and actor resolution, covers AC-1, AC-2
  - [x] Category filtering and historical date navigation routines, covers AC-3, AC-4
  - [x] React coaching day feed UI with date picker and category tabs, covers AC-1 through AC-4
- [x] Verify it: `/check verify coaching day timeline`
- [x] Test it: `/test coaching day timeline`

### 37. Batch health view
Per batch dashboard showing capacity utilization, attendance trend, dues total and collection rate, exam performance averages, assigned teacher, schedule, and inactive student count. Surfaces unhealthy batches without the owner asking.
**Done when:** batch health loads for all batches on one screen; color coding flags batches with attendance below 60%, dues above 30% of revenue, or capacity below 50%; clicking a batch opens its detail.
spec [0033](../specs/0033-owner-intelligence/0033-batch-health-view.md)
- [x] Design it (spec): `/architect batch health view`
- [x] Build it: `/develop batch health view`
  - [x] Core SQL aggregation computing capacity utilization, thirty day attendance rate, dues ratio, and inactive student counts, covers AC-1, AC-3
  - [x] Threshold evaluator mapping metrics to healthy, warning, and critical flags, covers AC-2
  - [x] React batch health dashboard with summary cards, badges, and program filters, covers AC-1 through AC-5
- [x] Verify it: `/check verify batch health view`
- [x] Test it: `/test batch health view`

### 38. Batch profitability
Estimated revenue (collected plus projected) versus teacher cost, room cost, and operational cost per batch. Helps the owner see which batches are financially sustainable.
**Done when:** profitability shows for each batch with revenue, costs, and margin; the owner can adjust cost assumptions; batches below breakeven are flagged; comparison across batches is on one screen.
spec [0033](../specs/0033-owner-intelligence/0033-batch-profitability.md)
- [x] Design it (spec): `/architect batch profitability`
- [x] Build it: `/develop batch profitability`
  - [x] Migration 0017 `batch_cost_configs` table and core update command, covers AC-2, AC-4
  - [x] Profitability calculation routine comparing projected tuition revenues against teacher costs, room rent, and overhead with breakeven flags, covers AC-1, AC-3
  - [x] React batch profitability view with inline cost adjustment sliders and margin indicators, covers AC-1 through AC-4
- [x] Verify it: `/check verify batch profitability`
- [x] Test it: `/test batch profitability`

### 39. Smart due collection queue
Groups outstanding dues by urgency, age, reminder history, batch, and payment behavior. Prioritizes which students to contact first for due collection, with one click actions for sending reminders or opening the cashier.
**Done when:** the queue shows students ordered by collection priority; the priority algorithm considers overdue months, last payment date, total outstanding, and reminder count; one click opens the cashier or sends a reminder; batch filtering works.
spec [0033](../specs/0033-owner-intelligence/0033-smart-due-collection-queue.md)
- [x] Design it (spec): `/architect smart due collection queue`
- [x] Build it: `/develop smart due collection queue`
  - [x] Migration 0017 `due_collection_contact_logs` table and logging command, covers AC-4
  - [x] Core due ranking query with multi factor priority formula and primary guardian phone lookup, covers AC-1, AC-2, AC-3
  - [x] React Smart Due Queue UI with contact logging modal, reminder trigger, and cashier deep links, covers AC-1 through AC-5
- [x] Verify it: `/check verify smart due collection queue`
- [x] Test it: `/test smart due collection queue`
