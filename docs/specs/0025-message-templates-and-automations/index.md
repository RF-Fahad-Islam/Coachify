# Spec 0025 (Multi Channel Templates and Automations: SMS, WhatsApp, Email)

> Feature 27 · Slice 4 (Academics and Messaging)

## Goal

Provide customizable English and Bengali message templates across SMS, WhatsApp, and Email, with precise GSM 03.38 and UCS 2 Unicode segment counting, turnkey automations for payment receipts, delayed and cancellable absence notifications, and period deduplicated fee due reminders, plus an auditable batch broadcast screen with channel selection, contact review tabs, and confirmation gates.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Multi Channel Catalog | `channel IN ('sms', 'whatsapp', 'email')` | Enables tailored formats: concise SMS, rich formatted WhatsApp (`*bold*`, emojis), and official HTML emails with subjects |
| Template Model | Normalized `message_templates` with `(key, channel, language)` unique key | Allows institutions to customize English and Bengali templates independently for each channel |
| Segment Math | Segment counting for SMS; character counting for WhatsApp and Email | Accurately models cellular billing units for SMS while giving full freedom for WhatsApp and Email |
| Absence Delay | 30 minute default delay window on absent mark notifications | Gives instructors time to correct student tardiness or clerical errors before alerts dispatch |
| Absence Cancellation | Updating mark from `absent` to `present`, `late`, or `excused` cancels pending alerts across SMS and WhatsApp | Guarantees false absence alerts are never delivered if corrected within the delay buffer |
| Due Review Gate | Batch review gate with candidate filtering and student exclusion toggles | Prevents sending due reminders to families who received verbal extensions or payment arrangements |
| Due Deduplication | Unique dedupe key scoped to student, period, and channel | Prevents spamming guardians with repeated reminders for the same month's dues |
| Broadcast Contact Tabs | Two tab preview: Ready to Dispatch versus Missing Contact Info | Guarantees broadcasts are not blocked while surfacing students needing updated phone numbers or emails |
| Batch Broadcast Gate | Modal confirmation displaying recipient count, channel, segment estimate, and sample preview | Prevents accidental mass sends and unintended credit depletion |

## Requirements

**User stories**:
- As an institute administrator, I want to edit our payment receipt template in Bengali, so that parents receive clear confirmation in their native language.
- As a batch teacher typing an SMS announcement, I want to see the exact segment count update live as I type, so that I do not accidentally trigger a second SMS segment with extended characters.
- As a cashier, I want fee collection to automatically send a receipt on WhatsApp and SMS without any extra buttons to click.
- As an academic coordinator, I want to send an exam reminder broadcast to an entire batch, review who is missing a phone number, and confirm the total segment cost before dispatching.
- As an institute owner, I want fee due reminders sent only once per monthly billing cycle per channel, skipping parents who agreed to an extension.

## Acceptance Criteria

### AC-1: Multi Channel Template Catalog and System Defaults
- Supports predefined system templates in English (`en`) and Bengali (`bn`) across SMS, WhatsApp, and Email:
  - `payment_received`:
    - SMS: Concise text with amount, receipt number, balance.
    - WhatsApp: Rich formatted card with bold labels (`*Amount Paid*`), emojis, and institute name.
    - Email: Formal payment receipt with subject line, greeting, and tabular breakdown.
  - `student_absent`:
    - SMS: Attendance absence alert with contact phone.
    - WhatsApp: Attendance alert card with date, batch, and immediate helpline link.
    - Email: Detailed daily absence notice with academic team signature.
  - `due_reminder`:
    - SMS: Fee due reminder with due date and amount.
    - WhatsApp: Formatted due statement card.
    - Email: Formal tuition fee statement with itemized period breakdown.
- System templates cannot be deleted (`is_system = 1`), but their title, subject, and body text can be customized.
- Custom templates can be created, updated, and deleted by users with `comms.manage` permission.

### AC-2: Strict GSM 03.38 versus Unicode Segment Math and Extended Table (for SMS)
- Real time character and segment calculation algorithm for SMS channel:
  - **GSM 7 bit Mode** (all characters present in GSM 03.38 standard alphabet):
    - Basic GSM characters (A to Z, a to z, 0 to 9, standard punctuation, newline) count as 1 character.
    - **GSM Extended Table characters** (`\`, `^`, `{`, `}`, `[`, `]`, `~`, `|`, and `€`) require an escape byte (`0x1B`) and count as 2 characters of segment capacity.
    - 1 segment: 1 to 160 character units.
    - Multi segment: 153 character units per segment (7 bytes reserved for UDH concatenation header).
  - **Unicode Mode** (contains any character outside GSM 03.38, including Bengali script `ক`, `খ`, emojis):
    - All UTF 16 code units count as 1 character.
    - 1 segment: 1 to 70 characters.
    - Multi segment: 67 characters per segment.
- For WhatsApp and Email: Returns standard character count with UTF 8 encoding without segment limits.

### AC-3: Placeholder Interpolation Engine
- Interpolates named placeholders enclosed in `{placeholder_name}`:
  - Student: `{student_name}`, `{student_code}`, `{roll}`
  - Academic: `{batch_name}`, `{program_name}`, `{date}`
  - Financial: `{amount}`, `{receipt_no}`, `{period}`, `{due_amount}`, `{balance}`
  - Institution: `{institution_name}`, `{institution_phone}`
- Unrecognized or missing placeholders are replaced with an empty string or left formatted safely without throwing errors.

### AC-4: Payment Received Automation Rule
- Rule `payment_received` can be toggled on or off independently per channel (`sms`, `whatsapp`, `email`).
- When enabled for a channel, executing `payment.create` in `ledger.rs` automatically inserts corresponding message into `outbox` within the **same transaction**.
- Deduplication keys: `payment:<id>:receipt_sms`, `payment:<id>:receipt_whatsapp`, `payment:<id>:receipt_email`.
- Target recipient: Primary guardian with corresponding contact info (phone for SMS and WhatsApp, email for Email).

### AC-5: Absence Automation and Delayed Cancellation Rule
- Rule `student_absent` can be toggled on or off independently per channel with configurable `delay_minutes` (default 30).
- When enabled, saving attendance in `attendance.rs` with an `absent` mark inserts outbox messages with `scheduled_at = now() + delay_minutes`.
- Deduplication keys: `attendance:<session_id>:<student_id>:absence_sms`, `attendance:<session_id>:<student_id>:absence_whatsapp`.
- If the mark is updated to `present`, `late`, or `excused` before `scheduled_at` arrives, all pending absence messages for that mark transition to `status = 'cancelled'`.

### AC-6: Due Reminder Automation Rule and Period Deduplication
- Rule `due_reminder` can be configured independently per channel with `min_due_poisha` threshold.
- The UI exposes a Batch Review Gate where administrators:
  - Filter active students by batch or minimum overdue balance.
  - Review the candidate list and uncheck specific students granted verbal extensions.
  - Preview message preview, total recipients, and channel costs.
  - Confirm dispatch to enqueue due reminders.
- Deduplication keys: `due_reminder:<student_id>:<period>:<channel>`.
- The unique constraint strictly guarantees a student never receives more than one due reminder on a given channel for the same billing period.

### AC-7: Batch Broadcast Screen and Two Tab Preview
- Dedicated UI view for sending custom batch messages:
  - Batch selector.
  - Channel selector: SMS, WhatsApp, or Email.
  - Template selector (or freeform message input; email mode exposes Subject Line field).
  - Live preview with placeholder interpolation for sample student.
  - Recipient preview split into two tabs:
    - **Ready to Dispatch (N)**: Enrolled students with valid phone or email for the selected channel, checked by default.
    - **Missing Contact Info (M)**: Enrolled students lacking contact details, displaying student roll and quick profile link so receptionists can collect details.
  - Metrics banner: Total Recipients, Segments per Message, Estimated Delivery Cost.
- Confirmation Modal:
  - Explicit warning displaying recipient count and channel.
  - Dispatches to `comms.send_batch_broadcast` only upon explicit user confirmation.

### AC-8: Structured Domain Errors
- Maps to typed `RpcError` with explicit codes:
  - `template_not_found`: Requested template ID does not exist.
  - `template_is_system`: Cannot delete a protected system template.
  - `duplicate_due_reminder`: Reminder has already been issued for this student, channel, and period.
  - `batch_no_valid_recipients`: No active students in the selected batch have valid contact information for the chosen channel.
  - `email_subject_required`: Email messages require a non empty subject line.
