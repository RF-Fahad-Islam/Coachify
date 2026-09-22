# Academics & messaging (Slices 4 to 5)

Back to [index](index.md).

## Slice 4: Attendance & messaging

### 25. Attendance
Take attendance per batch and date with keyboard entry, mark all, notes, a lock window, and monthly percentages. Covers present, absent, late, and excused states with reports.
**Done when:** a teacher marks 80 students by keyboard and saves in under 300 ms; "not taken" differs from "all absent"; edits after the lock need permission and are audited.
spec [0023](../specs/0023-attendance/index.md)
- [x] Design it (spec): `/architect attendance`
- [x] Build it: `/develop attendance`
  - [x] Migration 0014 schema for attendance_sessions and attendance_marks with unique batch/date constraint and permissions, covers AC-1, AC-3
  - [x] Core attendance engine with atomic mark insertion, sub-300ms benchmark, and active enrollment roster resolution, covers AC-1, AC-2
  - [x] Session lock evaluation and permission-guarded past edit handler with SHA-256 audit log tracking, covers AC-4
  - [x] Omnichannel delayed absence alert hook and mark correction cancellation hook across SMS and WhatsApp, covers AC-5, AC-6
  - [x] Monthly student summary and batch attendance matrix calculations, covers AC-7
  - [x] High-speed keyboard attendance sheet UI and monthly register view, covers AC-1, AC-2, AC-7
- [x] Verify it: `/check verify attendance`
- [x] Test it: `/test attendance`

### 26. Omnichannel outbox & provider adapters
Durable outbox written in the same transaction as the business action, supporting SMS, WhatsApp (via OpenWA), and Email (via SMTP), with a sender job on the host, quiet hours, and no automatic resend of uncertain sends. Covers the foundation for the durable inbox/outbox, offline message queue, and templates.
**Done when:** messages queued offline send exactly once on reconnect; a timeout is marked uncertain and not resent; OpenWA QR pairing and session state work; SMTP sends test email; secrets never appear in the UI or logs.
spec [0024](../specs/0024-sms-outbox-and-provider-adapters/index.md)
- [x] Design it (spec): `/architect omnichannel outbox & provider adapters`
- [x] Build it: `/develop omnichannel outbox & provider adapters`
  - [x] Migration 0014 schema for provider_configs, outbox (SMS, WhatsApp, Email), and delivery_attempts with unique dedupe key, covers AC-1
  - [x] Core transactional outbox repository, multi-channel status lifecycle machine, and quiet hours scheduler, covers AC-1, AC-2, AC-4
  - [x] Network timeout handling marking uncertain status with zero automatic retry, covers AC-3
  - [x] Provider adapter abstraction with Mock, Generic HTTP, Greenweb, BulkSMS BD, OpenWA (WhatsApp), and SMTP (Email) adapters, covers AC-6
  - [x] Host sender background worker loop with DPAPI secret unwrap and offline reconnection draining, covers AC-5, AC-7
  - [x] Multi-channel outbox monitor UI, OpenWA QR pairing view, and manual resend actions, covers AC-3, AC-8
- [x] Verify it: `/check verify omnichannel outbox & provider adapters`
- [x] Test it: `/test omnichannel outbox & provider adapters`

### 27. Multi-channel templates & automations
Bangla and English templates for SMS, WhatsApp, and Email with placeholders, segment and character counts, rules for payment received, absence (delayed and cancellable), and due reminders, plus a batch broadcast screen with channel selection.
**Done when:** a corrected absence cancels its pending alerts across all channels; due reminders never repeat for the same period; email templates render with subjects; batch broadcast confirms recipient count and channel costs.
spec [0025](../specs/0025-message-templates-and-automations/index.md)
- [x] Design it (spec): `/architect multi-channel templates & automations`
- [x] Build it: `/develop multi-channel templates & automations`
  - [x] Migration 0014 schema for message_templates and automation_rules across SMS, WhatsApp, and Email with bilingual system seeds, covers AC-1
  - [x] Dual GSM-7 vs UCS-2 Unicode character analysis (with extended escape table) and UTF-8 multi-channel validation, covers AC-2
  - [x] Dynamic template placeholder interpolation engine across student, finance, and institution fields, covers AC-3
  - [x] Turnkey automations for payment received, delayed absence alerts, and period-deduplicated due reminders per channel, covers AC-4, AC-5, AC-6
  - [x] Multi-channel template management UI, automation toggles, and batch broadcast composer with confirmation modal, covers AC-1, AC-7
- [x] Verify it: `/check verify multi-channel templates & automations`
- [x] Test it: `/test multi-channel templates & automations`

## Slice 5: Exams & results

### 28. Exams & mark entry · needs a decision
Exams across batches and subjects with full and pass marks, a spreadsheet style mark grid with keyboard navigation, absence, drafts, and verification. Includes grade scales (national Bangla medium A+ to F with grade points, English medium letter scales, pass rules, optional 4th subject math), tabulation sheets, and class ranking with tie breaking rules.
**Done when:** a teacher enters a batch's marks by keyboard alone; out of range marks are rejected; a subject can be verified only when complete; GPA calculates correctly with optional 4th subject rules; fail in any mandatory subject caps GPA to 0.0.
- [ ] Design it (spec): `/architect exams & mark entry`

### 29. Results, grading & report cards · needs a decision
Configurable grading schemes, calculation preview, publishing that freezes results and rules, marksheets, report cards with Bangla printing, merit lists, and cross subject tabulation for an entire batch with total marks, GPA, and rank generation.
**Done when:** published results do not change when the grading scheme is edited later; ties rank consistently; report cards print correctly in Bangla; an entire batch tabulation sheet prints in landscape mode with all subjects and ranks without horizontal truncation.
- [ ] Design it (spec): `/architect results, grading & report cards`
