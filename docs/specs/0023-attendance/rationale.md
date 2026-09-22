# 0023. Attendance (Rationale)

## Context

In Bangladeshi coaching centers and private academic institutes, batch sizes typically range from 30 to 100 students per classroom. Instructors have limited time between consecutive class periods (often only 5 to 10 minutes) to transition subjects and record attendance. In many institutes, attendance is taken by the batch teacher directly on the office PC or laptop, or collected by a receptionist from paper rosters and entered into the computer in batches.

Common failure modes in legacy coaching software include:
1. Conflating unrecorded attendance with all students absent: Many systems simply check whether attendance records exist for a student on a given date. If an institute fails to take attendance on a holiday or teacher absence, students appear as absent, corrupting monthly attendance percentages and alarming parents.
2. Slow mouse heavy entry: Requiring clicks on checkboxes or radio buttons for 80 students takes several minutes. In a rush, staff often abandon attendance entry entirely.
3. Retroactive attendance tampering: Without a strict lock window and permission model, staff or teachers can quietly modify historical attendance records to cover for unexcused student absences or alter eligibility for exams.
4. Immediate premature absence alerts: If an absence alert is fired the millisecond an instructor hits save, parents receive panic alerts even when a student was simply 5 minutes late or the teacher accidentally pressed the wrong key.

To solve these problems, Coachify requires a dedicated, keyboard driven attendance sheet with a header based session model, automated lock windows, audited permissions for retroactive changes, and a delayed absence notification workflow across SMS and WhatsApp.

---

## Options Considered

### Option 1: Two Tier Header Detail Architecture with Keyboard Ergonomics, Session Lock Window, and Delayed Automation Hooks (Chosen)

Model attendance as an explicit `attendance_sessions` header table with child `attendance_marks` records.
- The existence of a session header record explicitly signals that class was held and attendance was taken.
- Keyboard first interaction model: single keystrokes (`P`, `A`, `L`, `E`) and arrow navigation cycle through 80 students in under 30 seconds.
- Session automatically locks at midnight of the session date or after 24 hours. Modifying locked sessions requires `attendance.edit_previous` and generates an audit log entry.
- Saving absent marks queues SMS and WhatsApp alerts with a 30 minute delay, cancelled automatically if the mark is updated to present, late, or excused before dispatch.

Pros:
- Unambiguous operational semantics: Not taken versus all absent is crystal clear.
- Fast entry speed: sub 300 ms save time and keyboard only operation.
- Protects academic integrity: prevents silent retroactive tampering.
- Eliminates false absence alerts to guardians through the cancellation buffer.

Cons:
- Slightly more database tables than a flattened single table approach.
- Requires building UI support for both active entry and locked historical viewing.

### Option 2: Flattened Single Table Student Log (`student_id`, `batch_id`, `date`, `status`)

Store individual attendance marks directly with a composite key of `(student_id, batch_id, date)` without a parent session table.

Pros:
- Single table schema.

Cons:
- Impossible to distinguish between a day when attendance was never taken and a day when zero students attended.
- Queries calculating batch attendance rates must perform complex joins across student enrollment history to determine expected versus actual attendees.
- Session level metadata (who took attendance, when it was taken, whether the batch session was formally locked) must be duplicated across dozens of student rows.

### Option 3: Immediate Real Time Check In Event Stream

Record individual timestamped check in events (such as RFID or barcode scans at the door) rather than batch roll calls.

Pros:
- Hands free for teachers if hardware is available.

Cons:
- Hardware dependent: requires ID cards and barcode or RFID readers (deferred to later releases).
- Does not accommodate traditional coaching classrooms where teachers call rolls or circulate sign in sheets.
- Does not solve batch level recording for ordinary desktop installations on reference PCs.
