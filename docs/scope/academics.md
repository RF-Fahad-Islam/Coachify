# Academics & messaging (Slices 4 to 6)

Back to [index](index.md).

## Slice 4: Attendance & messaging

### 25. Attendance · needs a decision
Take attendance per batch and date with keyboard entry, mark all, notes, a lock window, and monthly percentages.
**Done when:** a teacher marks 80 students by keyboard and saves in under 300 ms; "not taken" differs from "all absent"; edits after the lock need permission and are audited.
- [ ] Design it (spec): `/architect attendance`

### 26. SMS outbox & provider adapters · needs a decision
Durable outbox written in the same transaction as the business action, a sender job on the host, a generic HTTP adapter plus one Bangladeshi provider, quiet hours, and no automatic resend of uncertain sends.
**Done when:** SMS queued offline sends exactly once on reconnect; a timeout is marked uncertain and not resent; secrets never appear in the UI or logs; a test SMS works from settings.
- [ ] Design it (spec): `/architect SMS outbox & provider adapters`

### 27. Message templates & automations · needs a decision
Bangla and English templates with placeholders and segment counts, rules for payment received, absence (delayed and cancellable), and due reminders, plus a send to batch screen with confirmation.
**Done when:** a corrected absence cancels its pending SMS; due reminders never repeat for the same period; the preview shows characters, segments, and recipient count.
- [ ] Design it (spec): `/architect message templates & automations`

## Slice 5: Exams

### 28. Exams & mark entry · needs a decision
Exams across batches and subjects with full and pass marks, a spreadsheet style mark grid with keyboard navigation, absence, drafts, and verification.
**Done when:** a teacher enters a batch's marks by keyboard alone; out of range marks are rejected; a subject can be verified only when complete.
- [ ] Design it (spec): `/architect exams & mark entry`

### 29. Results & report cards · needs a decision
Configurable grading schemes, calculation preview, publishing that freezes results and rules, marksheets, report cards, and merit lists.
**Done when:** published results do not change when the grading scheme is edited later; ties rank consistently; report cards print correctly in Bangla.
- [ ] Design it (spec): `/architect results & report cards`

## Slice 6: Reports

### 30. Reports center & exports · needs a decision
Collection (daily, range, collector wise, method), dues, expenses, cash days, attendance, admissions, and results reports with print, PDF, CSV, and Excel.
**Done when:** every finance report total matches the ledger in tests; CSV opens correctly with Bangla text in Excel; large reports page in the core.
- [ ] Design it (spec): `/architect reports center & exports`

### 31. Role dashboards
Simple dashboards for Owner, Accountant, Receptionist, and Teacher, including backup health and outbox status for Owner.
**Done when:** each role sees only figures their permissions allow; the Owner dashboard flags missing or failed backups.
- [ ] Build it: `/develop role dashboards`
