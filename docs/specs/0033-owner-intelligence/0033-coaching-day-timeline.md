# 0033. Coaching day timeline

> Feature 36 (Slice 7: Owner intelligence)

## Summary

This specification defines the Coaching day timeline, an institution wide chronological feed of daily events occurring at the center. Owners and supervisors can inspect admissions, payment receipts, attendance submissions, exam publications, SMS broadcasts, settings changes, and staff activities for any selected date, with actor attribution and category filters.

## Requirements

**User stories**:
* As an owner, I want a running feed of what is happening across the center today so that I can keep track of staff actions and counter throughput.
* As an auditor, I want to review past center days to reconstruct who accepted a specific payment or published exam marks.
* As an administrator, I want to filter today's feed by category (admissions, finance, attendance, exams, messages, settings) to focus on one operational area.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_coaching_day_timeline` returns all institution events for the specified date in under 500 ms.
* **AC-2**: Each event identifies the actor username and display name who performed the action.
* **AC-3**: Feed can be filtered by category (all, admissions, finance, attendance, exams, messages, settings).
* **AC-4**: Supports date navigation allowing inspection of any historical date.

## Inline rationale

Direct virtual queries combining audit log entries with domain transaction tables (`payments`, `admissions`, `exams`) guarantee an unbroken, chronological view of the coaching day without requiring an asynchronous event bus or background message store.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_coaching_day_timeline | query | date: String, filter_category: Option<String>, before_time: Option<String>, before_id: Option<String>, limit: Option<u32> | CoachingDayTimelineResponse | Owner, Admin | invalid_date_format |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_coaching_day_timeline | event timestamp and action | Extracted from source record timestamp and event classification |
| get_coaching_day_timeline | actor display name | Joined from users table using actor or created_by user ID |
| get_coaching_day_timeline | event summary text | Derived from domain entity metadata and payment or exam details |

## Build tasks

1. Construct unified daily event SQL query joining audit log with domain tables, satisfies **AC-1**, **AC-2**
2. Add category filter and cursor pagination routines in core, satisfies **AC-3**, **AC-4**
3. Create React coaching day feed UI with date selector and category tabs, satisfies **AC-1** through **AC-4**
