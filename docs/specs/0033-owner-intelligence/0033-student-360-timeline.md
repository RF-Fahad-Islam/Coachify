# 0033. Student 360 timeline

> Feature 34 (Slice 7: Owner intelligence)

## Summary

This specification defines the Student 360 timeline, presenting a student's full institutional journey in chronological order. Staff can view admission details, batch enrollments, fee charges, payments, attendance records, exam results, SMS messages, and counseling notes on one screen, with category filters and printable one page summaries.

## Requirements

**User stories**:
* As a center owner or guardian counselor, I want to review a student's complete history before meeting their parents so that I can speak with full context.
* As an administrator, I want to filter the student's timeline by category (financial, academic, attendance, communication, notes) to investigate specific discrepancies quickly.
* As a receptionist, I want to record academic and behavioral notes directly onto the student's timeline so future staff can see counseling observations.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_student_360` returns all timeline events for a student within 1000 ms, sorted newest first.
* **AC-2**: Events include correct categorization: Admission, Enrollment, Payment, Attendance, Exam Result, SMS Dispatch, and Staff Note.
* **AC-3**: Query supports category filtering and windowed pagination (limit and cursor) for high volume multi year histories.
* **AC-4**: Command `intelligence.add_student_note` inserts a note into `student_timeline_notes` with author attribution, immediately appearing in the timeline.
* **AC-5**: The student profile includes a printable summary view formatted for standard A4 paper.

## Inline rationale

Direct union queries over indexed primary tables ensure that newly recorded payments, exam marks, and attendance sessions show up instantly without batch synchronization or trigger latency. Staff notes are stored in a lightweight dedicated table to avoid polluting core student records.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_student_360 | query | student_id: String, category: Option<String>, before_time: Option<String>, before_id: Option<String>, limit: Option<u32> | Student360Response | Any authenticated | student_not_found, invalid_category |
| intelligence.add_student_note | command | student_id: String, note_type: String, title: String, content: String | StudentTimelineNote | Owner, Admin, Teacher, Receptionist | student_not_found, invalid_note_type |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_student_360 | event id and timestamp | Primary key and created_at column of source record |
| get_student_360 | event category and title | Formatted from source event type and related metadata |
| get_student_360 | actor display name | Joined from users table via created_by or author_id |
| get_student_360 | academic GPA and rank | Calculated from published exam results |
| add_student_note | persisted note record | Inserted into student_timeline_notes table |

## Build tasks

1. Construct unified SQLite union query across student payments, attendance, marks, messages, and notes, satisfies **AC-1**, **AC-2**, **AC-3**
2. Implement `intelligence.add_student_note` core command with validation, satisfies **AC-4**
3. Create React timeline component with category pill filters, cursor based pagination, and printable view, satisfies **AC-1**, **AC-5**
