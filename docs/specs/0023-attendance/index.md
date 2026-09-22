# Spec 0023 (Attendance)

> Feature 25 · Slice 4 (Academics and Messaging)

## Goal

Provide coaching center teachers and administrative staff with a high speed, keyboard first batch attendance marking system that records 80 student marks in under 300 ms, strictly separates attendance not taken from all students absent, enforces session lock windows, requires audited permissions for past dated edits, and integrates seamlessly with absence alert automation across SMS and WhatsApp.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Session Header Model | Separate `attendance_sessions` record per `(batch_id, date)` | Distinguishes an unrecorded class session from a class where all students were absent |
| Mark Statuses | `present`, `absent`, `late`, `excused` | Standard statuses in Bangladeshi academic institutions |
| Keyboard Ergonomics | `Arrow Up/Down` for navigation; `P`, `A`, `L`, `E` single key entry | Allows teachers to mark an entire class of 80 students in seconds without touching the mouse |
| Bulk Operations | `Mark All Present` and `Mark All Absent` instant shortcuts | Reduces repetitive keystrokes when class attendance is near universal |
| Lock Window Policy | Midnight of session date (or 24 hours post creation) | Prevents retroactive attendance tampering while permitting same day corrections |
| Post Lock Edits | Requires `attendance.edit_previous` permission and writes SHA 256 audit entry | Guarantees accountability for administrative or disciplinary changes |
| Omnichannel Absence Hook | Synchronously enqueues delayed outbox messages across SMS and WhatsApp | Ensures parents receive notifications without coupling attendance saving to external network latency |
| Absence Correction Hook | Transitions pending delayed outbox messages to `cancelled` upon mark change | Prevents false absence alerts if a teacher corrects a clerical error before dispatch |
| Real Time Feedback Badges | Renders live badges on student rows showing pending vs sent alerts | Gives teachers total visibility into guardian notifications and warns if a mark is edited post dispatch |

## Requirements

**User stories**:
- As a batch teacher, I want to open today's class roster and mark attendance using only the keyboard (`P`, `A`, `L`, `E`), so that I can finish taking attendance in under 30 seconds.
- As an institute owner, I want to see which batches have not yet taken attendance today, so that I can remind teachers before classes dismiss.
- As an administrator, I want past dated attendance records to lock automatically at the end of the day, so that teachers cannot quietly alter historical attendance percentages.
- As an academic coordinator, I want to edit a locked attendance record when a parent submits an official medical excuse, requiring my explicit permission and leaving an immutable audit trail.
- As a guardian, I want to receive an alert if my child is marked absent, but only after a short delay in case the teacher made an initial mistake or the student arrived late.

## Acceptance Criteria

### AC-1: Batch Roster and Session Inspection
- RPC `attendance.get_session` returns the attendance sheet for a given `batch_id` and `date` (`YYYY-MM-DD`).
- Retrieves all students with an **active enrollment** in the batch on that date (`enrollments.start_date <= date AND (enrollments.end_date IS NULL OR enrollments.end_date >= date)`).
- If an `attendance_sessions` row exists:
  - Returns `session_exists: true`, `locked`, `taken_by`, `taken_at`, `notes`, and existing marks for each student.
- If no session exists:
  - Returns `session_exists: false`, `locked: false`, and an empty mark state for each active student (defaulting in UI to `present` or blank based on institute settings).

### AC-2: Rapid Keyboard Entry and Bulk Actions
- Keyboard navigation across student rows:
  - `Arrow Down` / `Enter`: Move focus to next student.
  - `Arrow Up`: Move focus to previous student.
  - `P` / `p`: Set status to `present`.
  - `A` / `a`: Set status to `absent`.
  - `L` / `l`: Set status to `late`.
  - `E` / `e`: Set status to `excused`.
  - `Tab`: Jump between status picker and optional note input.
- Instant bulk shortcuts:
  - `Mark All Present` helper button or keyboard trigger: Sets all students in roster to `present`.
  - `Mark All Absent` helper button: Sets all students in roster to `absent`.
- RPC `attendance.save_session` commits all marks in a single SQLite transaction and completes in under 300 ms for 80 students.

### AC-3: Header Row Distinction (Not Taken vs All Absent)
- A date and batch combination has two distinct states:
  - **Not Taken**: No row in `attendance_sessions`. Batch registers and dashboards flag this batch as pending attendance.
  - **All Absent**: A row exists in `attendance_sessions` where every student mark in `attendance_marks` has `status = 'absent'`.
- Institute dashboards and monthly reports never conflate an unrecorded class with 0% attendance.

### AC-4: Session Lock Window and Guarded Retroactive Edits
- A session is considered locked if:
  - Explicitly marked `locked = 1` via `attendance.lock_session`, OR
  - Both `date < current_business_date()` AND `now() > datetime(taken_at, '+12 hours')` (preventing the midnight trap for late evening classes while maintaining daily lock integrity), OR
  - More than 24 hours have elapsed since `taken_at`.
- Saving or updating marks for a locked session requires the `attendance.edit_previous` permission. Unauthorized attempts fail with error code `attendance_locked`.
- Every edit to a locked session writes an entry to `audit_logs` capturing `before_json` and `after_json` for modified student marks along with a mandatory audit reason.

### AC-5: Omnichannel Absence Automation Hook
- When an attendance session is saved and the `student_absent` automation rule is active:
  - For each student newly marked `absent`, delayed messages are enqueued for all enabled channels (SMS, WhatsApp):
    - SMS dedupe key: `attendance:<session_id>:<student_id>:absence_sms`
    - WhatsApp dedupe key: `attendance:<session_id>:<student_id>:absence_whatsapp`
  - Default delay window is 30 minutes (`scheduled_at = datetime('now', '+' || delay_minutes || ' minutes')`).
  - If an outbox row with that dedupe key already exists, it is not recreated.

### AC-6: Mark Change Cancellation Hook and Concurrency Guard
- If an attendance mark is changed from `absent` to `present`, `late`, or `excused`:
  - The core executes a strictly bounded cancellation query:
    ```sql
    UPDATE outbox 
    SET status = 'cancelled' 
    WHERE (
        dedupe_key = 'attendance:' || ?1 || ':' || ?2 || ':absence_sms'
        OR dedupe_key = 'attendance:' || ?1 || ':' || ?2 || ':absence_whatsapp'
    ) AND status = 'queued';
    ```
  - If 0 rows are affected (because the background worker has already transitioned the message to `sending` or `sent`), the system gracefully records that the cancellation window expired without corrupting the `sending` or `sent` status.
  - The UI warns the teacher with an informational toast that the absence alert was already sent to the guardian.
- If an entire attendance session is deleted by an administrator, trigger `trg_attendance_sessions_cancel_outbox` automatically cancels all associated pending `queued` outbox entries to prevent orphaned alerts.

### AC-7: Monthly Student and Batch Attendance Metrics
- RPC `attendance.get_student_summary` calculates:
  - Total class sessions held in the requested date range.
  - Counts of `present`, `absent`, `late`, and `excused`.
  - Attendance percentage: (present + late) / total sessions * 100.
- RPC `attendance.get_batch_matrix` returns a grid of all active students in a batch across all calendar dates in a requested month (`YYYY-MM`).

### AC-8: Structured Domain Errors
- Maps to typed `RpcError` with explicit error codes:
  - `attendance_not_found`: Requested session does not exist.
  - `attendance_locked`: Attempted to modify a locked session without `attendance.edit_previous`.
  - `student_not_enrolled`: Student is not actively enrolled in batch on session date.
  - `invalid_attendance_status`: Status is not one of `present`, `absent`, `late`, `excused`.
