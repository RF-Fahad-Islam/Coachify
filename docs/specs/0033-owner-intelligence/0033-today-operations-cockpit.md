# 0033. Today operations cockpit

> Feature 35 (Slice 7: Owner intelligence)

## Summary

This specification defines the Today operations cockpit, a daily command center for coaching center owners and administrators. It consolidates today's scheduled classes, missing attendance alerts for completed batches, daily fee collections, total overdue balances, absent teachers, upcoming exams, message failures, and database backup health into a single live dashboard.

## Requirements

**User stories**:
* As an owner, I want to open Coachify in the morning and see every class scheduled for today, so that I know what to expect.
* As an administrator, I want to be alerted immediately when a batch class end time has passed without attendance submitted, so that I can remind the teacher.
* As an owner, I want to monitor today's counter cash and digital collections against daily expectations in real time.
* As an academic coordinator, I want to see which exams are scheduled over the upcoming week so we can prepare question papers.
* As an operations manager, I want to see if any SMS messages failed or if database backups are overdue before closing the center for the day.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_today_cockpit` delivers all summary widgets in under 500 ms for the current local date.
* **AC-2**: Batches scheduled for today whose class end time has passed without any attendance session recorded are flagged in the missing attendance alert list.
* **AC-3**: Total payments collected today are broken down by payment method (cash, bKash, Nagad, bank) using index friendly range filters (`paid_at >= ? AND paid_at < ?`).
* **AC-4**: Dashboard surfaces absent teachers whose scheduled classes ended today without an attendance submission.
* **AC-5**: Dashboard lists exams scheduled within the next seven days with date, subject count, and enrolled candidate totals.
* **AC-6**: Dashboard alerts when the last verified database backup is older than 24 hours.

## Inline rationale

Checking missing attendance after the scheduled class end time provides teachers realistic time to finish instruction and submit marks without generating premature false alarms during class. Sourcing upcoming exams directly from scheduled exams ensures coordinators have advance notice for logistics.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_today_cockpit | query | target_date: Option<String> | TodayCockpitResponse | Owner, Admin | invalid_date_format |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_today_cockpit | classes scheduled today | Filtered from batch_schedules by current day of week |
| get_today_cockpit | missing attendance alert list | Batches with schedule today where end_time < current_time and no attendance record exists |
| get_today_cockpit | absent teachers list | Teachers assigned in batch_teachers whose scheduled batch class ended today without attendance |
| get_today_cockpit | upcoming exams list | Exams table where exam_date >= target_date and exam_date <= target_date + 7 days |
| get_today_cockpit | collections today sum | Sum of valid payments where paid_at >= start_of_day AND paid_at < end_of_day |
| get_today_cockpit | total overdue balance | Sum of unpaid ledger charges where due_date < current_date |
| get_today_cockpit | message failure count | Count of sms_outbox entries with status = 'failed' in last 24 hours |
| get_today_cockpit | backup health indicator | Query from backup metadata table comparing last_backup_at to current_time |

## Build tasks

1. Implement SQL routine in core querying batch schedules against attendance submissions for today, satisfies **AC-1**, **AC-2**, **AC-4**
2. Aggregate today collections, overdue balances, upcoming exams, and system health indicators, satisfies **AC-3**, **AC-5**, **AC-6**
3. Create React Today Cockpit dashboard with quick action links and background refresh, satisfies **AC-1** through **AC-6**
