# 0033. Smart due collection queue

> Feature 39 (Slice 7: Owner intelligence)

## Summary

This specification defines the Smart due collection queue, prioritizing overdue students by recovery urgency. The priority algorithm weighs overdue months, outstanding balance, days since last contact attempt, and reminder frequency. Front desk staff can make collection calls, log promise dates, send SMS reminders, or launch the payment cashier in one click.

## Requirements

**User stories**:
* As a cashier or accountant, I want a prioritized queue of students with unpaid dues so that I know who to call first every morning.
* As an administrator, I want to record call outcomes and promised payment dates so that parents are not disturbed repeatedly by different staff members on the same day.
* As a cashier, I want a direct button from the collection queue to open the payment cashier prefilled for that student.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_smart_due_queue` outputs students with outstanding balances ordered by priority score.
* **AC-2**: Priority algorithm combines overdue duration (days since oldest unpaid charge due date), total due amount, days elapsed since last contact, and total reminder count (sum of SMS dispatches and manual call logs).
* **AC-3**: Queue displays primary guardian phone (joined from guardians via student_guardians where is_primary is true), last payment date, last contact outcome, and next promised payment date.
* **AC-4**: Command `intelligence.record_due_contact` records phone calls, in person visits, or messages, updating priority rankings immediately.
* **AC-5**: Each queue row provides direct action shortcuts to open the payment cashier or trigger an SMS reminder modal.

## Inline rationale

A weighted deterministic score calculated in Rust allows instant sorting without database locking. Logging contact attempts in `due_collection_contact_logs` prevents front desk harassment of guardians who already agreed to pay on a specific date, while deprioritizing them until that date arrives.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_smart_due_queue | query | batch_id: Option<String>, min_overdue_days: Option<u32>, limit: Option<u32>, offset: Option<u32> | SmartDueQueueResponse | Owner, Admin, Accounts | invalid_parameters |
| intelligence.record_due_contact | command | student_id: String, contact_channel: String, outcome: String, promised_payment_date: Option<String>, notes: String | DueContactLogItem | Owner, Admin, Accounts, Receptionist | student_not_found, invalid_outcome |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_smart_due_queue | outstanding due poisha | Sum of unpaid ledger charges for student |
| get_smart_due_queue | primary guardian phone | Joined from guardians via student_guardians where is_primary = 1 |
| get_smart_due_queue | overdue days count | Days between due_date of oldest unpaid charge and today |
| get_smart_due_queue | reminder count | Count of sms_outbox entries plus due_collection_contact_logs entries for student |
| get_smart_due_queue | priority ranking score | (overdue_days * 1.5) + (due_poisha / 100000.0) - (days_since_contact * 2.0) + (reminder_count * 5.0) |
| get_smart_due_queue | last contact outcome and promise date | Loaded from latest record in due_collection_contact_logs |
| record_due_contact | contact log entry | Inserted into due_collection_contact_logs with authenticated user ID |

## Build tasks

1. Implement due queue ranking query and priority scoring formula in Rust core, satisfies **AC-1**, **AC-2**, **AC-3**
2. Implement `intelligence.record_due_contact` command, satisfies **AC-4**
3. Create React Smart Due Queue interface with batch filters, contact logging modal, and cashier deep links, satisfies **AC-1** through **AC-5**
