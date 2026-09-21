# 0009 Rationale. Academic programs, batches, staff directory, and student enrollments

**Date**: 2026-09-22
**Status**: Proposed

## Context

Coaching centers, academies, and private educational institutes in Bangladesh operate under distinct operational conditions compared to traditional formal schools:

1. **Flexible Shift & Batch Scheduling**: Rather than having all students attend from 9 AM to 3 PM, coaching centers run parallel batches throughout the day (Morning, Day, and Evening shifts). Multiple batches of the same program (e.g., "Physics HSC 2026 Batch A, B, and C") share classrooms and instructors.
2. **Classroom and Teacher Bottlenecks**: Urban coaching centers in cities like Dhaka, Chittagong, and Sylhet often operate in commercial buildings with a limited number of physical classrooms. Instructors frequently teach across different grades and subjects. A single scheduling error—such as double-booking "Room 201" or assigning the same physics lecturer to two concurrent batches—creates chaos during rush hours.
3. **Frequent Section Transfers**: Students routinely request section changes when school schedules change or transportation shifts (e.g., transferring from an 8:00 AM morning batch to a 4:30 PM evening batch). If software overwrites the student's batch assignment, past attendance records and fee invoices lose their contextual anchor.
4. **Capacity Constraints**: Classrooms have hard physical seating capacities (typically 30 to 60 benches). Centers must track enrolled seat counts in real time, preventing inadvertent counter overcrowding while providing supervisor discretion for waitlisted or VIP admissions.
5. **Staff Diversity**: Staff includes full-time administrators, salaried permanent teachers, and guest lecturers paid per class. Software accounts must link to verified staff profiles without requiring every staff member to possess computer login credentials.

## Options considered

### Option 1: Relational Normalized Domain with Structured JSON Recurrence Schedules, In-Memory Collision Matrix, Atomic Transfer Transactions, and Rowid-Bound SQLite FTS5 Staff Search (Recommended)

Programs, Batches, Subjects, Staff, and Enrollments are modeled as first-class relational entities in SQLite. Weekly recurrence schedules are stored in a validated JSON array column (`schedule_json`) within the `batches` table. Schedule conflict detection is executed by an in-memory collision detection algorithm that fetches active batch time slots and tests interval overlap ($S_1.\text{start} < S_2.\text{end} \land S_1.\text{end} > S_2.\text{start}$) across shared teachers and rooms. Batch transfers are executed as atomic two-step transactions that close the previous enrollment with an audit reason and open a new one with a newly allocated roll number. Staff search uses an SQLite FTS5 trigram virtual table bound to the native integer `rowid`.

**Pros**:
- **Compact & High Performance**: Storing weekly recurrence in `schedule_json` avoids maintaining hundreds of synthetic calendar rows for static weekly routines while preserving full day-of-week and time-range fidelity.
- **Fast In-Memory Conflict Engine**: Evaluating 50–100 active batch schedules in memory takes less than 2 milliseconds on low-power Intel Celeron/Core i3 desktop PCs, eliminating complex and slow SQL self-joins.
- **Auditable Enrollment History**: Students never lose past enrollment records. Transferred, dropped, and completed states remain historically queryable, providing reliable backing data for future attendance and fee reconciliation.
- **Clean Separation of Concerns**: Staff records exist independently of software authentication accounts (`users`), allowing coaching centers to track all faculty and operational employees even if only 2 receptionists have system login access.
- **Sub-50ms Staff Search**: Trigram tokenizer enables instant substring matching across English and Bengali names, designations, and contact numbers.

**Cons**:
- Requires writing and maintaining dedicated in-memory schedule conflict validation logic in Rust.
- Schema modifications to weekly slots require JSON validation in domain code.

---

### Option 2: Rigid Relational Slot Schedule Tables with Discrete Time-Interval Rows

Every scheduled day and time slot is modeled as an individual row in a `batch_schedules` relational table (`id`, `batch_id`, `day_of_week`, `start_time`, `end_time`, `room_number`). Schedule conflicts are detected via complex SQL self-joins with inequality operators.

**Pros**:
- Normal form purism: time slots are stored in standard SQL columns.
- Basic foreign keys can link directly to a dedicated `rooms` table if one existed.

**Cons**:
- **Query Complexity & Performance**: Checking for overlapping time ranges across teachers and rooms in pure SQL requires multiple self-joins with range checks (`start < other.end AND end > other.start`), which performs poorly in SQLite when batches scale and lacks clear diagnostic error reporting.
- **Operational Overhead**: Extra relational tables and cascade triggers increase migration complexity and database write overhead for simple weekly recurring schedules.
- **Rigidity**: Does not easily accommodate ad-hoc slot-specific teacher overrides or special room assignments without further relational proliferation.

---

### Option 3: Denormalized Course and Batch Tracking Directly on Student Records

Student records store `current_program` and `current_batch` as text columns or JSON arrays directly inside `students`, without standalone batch rosters or enrollment transaction logs.

**Pros**:
- Minimal initial schema with fewer tables.
- Fast student profile reading.

**Cons**:
- **Zero Historical Auditability**: Overwriting a batch destroys all historical knowledge of when the student joined, what roll they held, or why they transferred.
- **No Schedule Integrity**: Impossible to detect teacher double-bookings, track room capacity, or generate class attendance sheets.
- **Fatal for Institutional Operations**: Completely incapable of supporting real coaching center operations.

---

## Comparison matrix

| Quality Dimension | Option 1 (JSON Recurrence + In-Memory Engine) | Option 2 (Relational Slot Tables) | Option 3 (Denormalized on Student) |
|---|---|---|---|
| **Conflict Detection Performance** | **< 2ms** (in-memory interval arithmetic) | 15–40ms (complex SQL self-joins) | None (unsupported) |
| **Audit & Historical Integrity** | **High** (immutable enrollment records) | **High** | Poor (overwrites history) |
| **Offline Desktop Performance** | **Exceptional** (minimal disk I/O, compact DB) | Moderate (higher row count) | Fast but functionally broken |
| **Capacity Management** | **Real-time** with supervisor bypass | Real-time | None |
| **Implementation Complexity** | Balanced (Rust domain logic + clean SQL) | High (complex SQL joins & triggers) | Low (unusable) |

## Rationale

Option 1 is selected because it directly serves the operational needs of coaching centers in Bangladesh while maintaining Coachify's desktop-first performance standards:

1. **Why In-Memory Schedule Collision Detection**:
   In a typical coaching center, there are between 10 and 150 active batches during an academic term. Fetching active batch schedules into memory and running an interval overlap comparison takes less than 2 milliseconds in Rust. More importantly, an in-memory engine can construct rich, human-friendly diagnostic errors that explain *exactly* why a collision occurred (e.g., *"Teacher 'Rafiqul Islam' is already assigned to 'HSC Physics Batch A' in Room 101 on Sunday from 08:00 to 09:30"*). Providing this feedback in real time as the user types in the UI prevents scheduling errors before they are committed.

2. **Why Atomic Batch Transfers**:
   Section changes are one of the most frequent counter requests. When a student transfers, receptionists need to know their previous roll in the old section and allocate a new roll in the new section. By wrapping the transition in an immediate SQLite transaction, Coachify guarantees that a student cannot be left without an active batch, nor can they be actively enrolled in two conflicting sections of the same program simultaneously.

3. **Why Roll Number Uniqueness via Partial Index**:
   Roll numbers are local to a batch. Student A might be Roll 1 in Morning Physics, while Student B is Roll 1 in Evening Chemistry. However, two active students in the *same* batch must never share a roll number. Migration `0011` enforces this with `CREATE UNIQUE INDEX idx_batch_active_roll ON enrollments(batch_id, roll) WHERE status = 'active'`, allowing past ended enrollments to retain their historical roll without blocking future students from reusing that roll number.

4. **Why Decoupled Staff and User Accounts**:
   Coaching centers have dozens of teachers and support staff, but typically only 2 to 4 desktop computers at the reception and accounts desk. Decoupling `staff` from `users` ensures that all instructors can be assigned to batches and scheduled on timetables without needlessly creating software login accounts for individuals who never operate the computer. When a teacher or manager does require system access, their `users` account simply references `users.staff_profile_id`.
