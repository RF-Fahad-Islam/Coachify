# Owner intelligence (Slice 7)

Back to [index](index.md). The coaching center owner's operational brain. These features surface what matters without the owner having to dig. All read from existing data; no new data entry required.

### 34. Student 360 timeline · needs a decision
Entire student journey in chronological form: admission, enrollment changes, payments, attendance, exam results, messages sent, status changes, and notes.
**Done when:** the timeline loads within 1 second for a 3 year student; every financial, academic, and administrative event appears; the timeline can be printed as a one page student report.
- [ ] Design it (spec): `/architect student 360 timeline`

### 35. Today operations cockpit · needs a decision
Single view for the owner or admin showing today's classes, missing attendance flags, overdue dues, today's collections, absent teachers, upcoming exams, message failures, and backup status. The daily command center.
**Done when:** the cockpit loads fresh on every visit; each widget links to its detail view; the owner can see at a glance if anything needs immediate attention; missing attendance is flagged within 15 minutes of a batch's scheduled start.
- [ ] Design it (spec): `/architect today operations cockpit`

### 36. Coaching day timeline · needs a decision
Chronological feed of the day's events: admissions, payments, attendance submissions, exam publications, messages sent, settings changes, and staff activity. A living record of what happened at the center today.
**Done when:** every business event appears in the timeline within seconds of occurring; the feed can be filtered by type; past days can be viewed; the timeline shows who performed each action.
- [ ] Design it (spec): `/architect coaching day timeline`

### 37. Batch health view · needs a decision
Per batch dashboard showing capacity utilization, attendance trend, dues total and collection rate, exam performance averages, assigned teacher, schedule, and inactive student count. Surfaces unhealthy batches without the owner asking.
**Done when:** batch health loads for all batches on one screen; color coding flags batches with attendance below 60%, dues above 30% of revenue, or capacity below 50%; clicking a batch opens its detail.
- [ ] Design it (spec): `/architect batch health view`

### 38. Batch profitability · needs a decision
Estimated revenue (collected plus projected) versus teacher cost, room cost, and operational cost per batch. Helps the owner see which batches are financially sustainable.
**Done when:** profitability shows for each batch with revenue, costs, and margin; the owner can adjust cost assumptions; batches below breakeven are flagged; comparison across batches is on one screen.
- [ ] Design it (spec): `/architect batch profitability`

### 39. Smart due collection queue · needs a decision
Groups outstanding dues by urgency, age, reminder history, batch, and payment behavior. Prioritizes which students to contact first for due collection, with one click actions for sending reminders or opening the cashier.
**Done when:** the queue shows students ordered by collection priority; the priority algorithm considers overdue months, last payment date, total outstanding, and reminder count; one click opens the cashier or sends a reminder; batch filtering works.
- [ ] Design it (spec): `/architect smart due collection queue`
