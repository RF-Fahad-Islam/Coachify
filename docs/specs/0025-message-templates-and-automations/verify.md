# Verify: Multi channel templates and automations (spec 0025)

Steps derived from spec 0025 acceptance criteria.

## UI and Manual Verification

- [ ] Navigate to Settings -> Templates (`/settings/templates`): verify that system default templates for English and Bengali across SMS, WhatsApp, and Email are listed and marked as system templates (verifies AC-1).
- [ ] Open a template in the Template Editor in SMS mode: type 160 standard ASCII characters. Verify that the indicator displays `160/160 chars (1 segment, GSM)` (verifies AC-2).
- [ ] Type 1 additional ASCII character (161 chars): verify that the indicator transitions to `161/306 chars (2 segments, GSM)` (verifies AC-2).
- [ ] Type an extended GSM character such as `€` or `[`: verify that the character consumes 2 units of segment capacity (verifies AC-2).
- [ ] Clear input and type a Bengali character (for example `ক`): verify that the indicator immediately switches to Unicode mode: `1/70 chars (1 segment, Unicode)` (verifies AC-2).
- [ ] Switch channel selector to WhatsApp or Email: verify that the segment counter hides and displays standard UTF 8 character count without cellular segment restrictions (verifies AC-2).
- [ ] In the Template Editor preview tab: select a student and verify that `{student_name}`, `{roll}`, `{amount}`, `{balance}`, and `{institution_name}` placeholders interpolate correctly into the rendered preview text (verifies AC-3).
- [ ] Ensure `payment_received` automation rules are enabled for SMS, WhatsApp, and Email. Record a fee payment at the cashier desk: inspect the Outbox Monitor and verify that messages are queued for each enabled channel with distinct dedupe keys (`payment:<id>:receipt_sms`, `payment:<id>:receipt_whatsapp`, `payment:<id>:receipt_email`) (verifies AC-4).
- [ ] Ensure `student_absent` automation rules are enabled. Save batch attendance with a student marked `absent`: verify that absence messages are enqueued for SMS and WhatsApp with `scheduled_at` set 30 minutes in the future (verifies AC-5).
- [ ] Before the 30 minute delay elapses, edit the student mark to `present`: inspect the outbox and verify that messages on both channels transition to `cancelled` (verifies AC-5).
- [ ] Navigate to Due Reminders (`/communication/due-reminders`): filter candidates by batch and overdue threshold. Uncheck a student who requested an extension. Click `[Enqueue Due Reminders]`: verify that reminders are enqueued with dedupe keys (`due_reminder:<student_id>:<period>:<channel>`) skipping the unchecked student (verifies AC-6).
- [ ] Re run the due reminder trigger for the same billing period: verify that zero duplicate messages are enqueued (verifies AC-6).
- [ ] Open the Batch Broadcast view (`/communication/broadcast`): select a batch and select WhatsApp channel. Verify that the recipient preview displays two tabs: `Ready to Dispatch` and `Missing Contact Info` (verifies AC-7).
- [ ] In the `Missing Contact Info` tab, verify that students lacking phone numbers are clearly listed with a quick profile link (verifies AC-7).
- [ ] Click `[Send Broadcast]`: verify that a confirmation modal appears displaying recipient count, channel, and message preview. Click `[Confirm and Send]`: verify that all messages are queued into the outbox with appropriate dedupe keys (verifies AC-7).

## Commands

- [ ] `cargo test -p coachify_core --test message_templates_and_automations` (verifies GSM 7 versus UCS 2 segment math boundaries, Bengali UTF 8 integrity, placeholder interpolation, payment receipt trigger across channels, delayed absence trigger, absence cancellation hook, and period due deduplication, satisfies AC-1 through AC-6).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contract freshness in `ui/src/gen/rpc.ts` for all template and broadcast RPC types).
- [ ] `cargo run -p xtask check-migrations` (verifies Migration 0014 schema validity and released migration immutability).
- [ ] `pnpm --dir ui test` (verifies TemplateEditor character counter, Unicode toggle badge, placeholder pills, automation switches, two tab broadcast preview, and batch broadcast confirmation modal).

## Acceptance Criteria Coverage

- AC-1 (Multi Channel Template Management and System Defaults): verified by core template seed tests and UI template listing tests across SMS, WhatsApp, and Email.
- AC-2 (Strict GSM versus Unicode Segment Math): verified by exhaustive character boundary unit tests covering GSM 7 (160/153), GSM extended table, and UCS 2 Unicode (70/67).
- AC-3 (Placeholder Interpolation Engine): verified by template rendering tests across student, academic, financial, and institution placeholders.
- AC-4 (Payment Received Automation Rule): verified by ledger payment integration tests asserting atomic outbox insertion across enabled channels.
- AC-5 (Absence Automation and Delayed Cancellation Rule): verified by attendance integration tests asserting 30 minute delay scheduling and status cancellation upon mark correction across SMS and WhatsApp.
- AC-6 (Due Reminder Automation Rule and Period Deduplication): verified by reminder engine tests asserting single reminder per student per billing period per channel.
- AC-7 (Batch Broadcast Screen and Two Tab Preview): verified by broadcast recipient filtering tests, missing contact segregation, and UI confirmation modal workflow tests.
