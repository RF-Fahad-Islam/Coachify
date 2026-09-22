# Spec 0029 (Role Dashboards & Zero Training Mode)

> Feature 31 · Slice 6 (Reports & Speed)

## Goal

Provide role-tailored daily cockpits for Receptionists, Cashiers, Teachers, Administrators, and Institution Owners that implement "Zero Training Mode"—a distraction-free UI presenting only the high-frequency daily actions, queues, and statistics relevant to that specific role, paired with an interactive "View As Role" preview mode allowing Owners to inspect any role's operational interface without re-authenticating.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Zero Training Mode Philosophy** | Action-first cards with high-contrast shortcut badges (`N S`, `N P`, `G A`) | Allows newly hired front-desk staff or non-technical teachers to execute daily operations on day one without software training |
| **Role-Specific Task Prioritization** | Receptionist = Admissions & Attendance; Cashier = Collections & Dues; Teacher = Classes & Marks; Admin = Operations; Owner = Full Cockpit | Eliminates cognitive overload by removing menu items and metrics that do not belong to that staff member's core duties |
| **Owner Impersonation / Preview** | Client-side transient role override ("View As") strictly for UI rendering | Empowers institute owners to verify workflows and troubleshoot counter issues without altering user credentials or security tokens |
| **Dynamic Sidebar Pruning** | Sidebar `NAV_ITEMS` filtered dynamically based on current user permissions | Defense-in-depth: non-authorized routes are not rendered in navigation, preventing UI clutter and unauthorized clicks |
| **Seeded Roles in Migration 0016** | Explicitly seed `Receptionist` and `Admin` in the `roles` table alongside `Owner`, `Cashier`, and `Teacher` | Completes the RBAC catalog and provides standard role templates out of the box for all coaching centers |

## Requirements

**User stories**:
- As a receptionist logging in at the front desk, I want to see prominent buttons for "Admit Student", "Search Student", and "Mark Attendance", with today's visitor and admission counts, without being confused by accounting or system settings.
- As a cashier arriving at the counter, I want to immediately see today's collection total, open cash drawer balance, and a 1-click button to collect fees or view the dues directory.
- As a batch teacher logging in, I want to see only my assigned batches for today with a single-click "Take Attendance" button, and drafts of exam papers awaiting mark entry.
- As an institute owner, I want a comprehensive financial and operational overview, plus a dropdown to preview the application as a Receptionist or Cashier so I can train my staff or verify their view.
- As an administrator, I want to manage academic scheduling, batch assignments, and bulk communications without access to root backup wipes or license configuration.

## Acceptance Criteria

### AC-1: Role Seeding & Permission Configuration
- Migration 0016 seeds `Receptionist` and `Admin` system roles.
- `Receptionist` permissions: `students.view`, `students.manage`, `academic.view`, `attendance.mark`, `search.global`.
- `Cashier` permissions: `students.view`, `payments.view`, `payments.collect`, `ledger.view`, `reports.financial`, `search.global`.
- `Teacher` permissions: `students.view`, `academic.view`, `attendance.mark`, `exam.view`, `exam.marks_entry`, `exam.verify`, `search.global`.
- `Admin` permissions: Full operational permissions excluding root cryptographic recovery and settings.
- `Owner`: All permissions including system administration and role preview mode.

### AC-2: Receptionist Dashboard View
- Dashboard displays:
  - 3 large primary action cards: `[Admit New Student (N S)]`, `[Search Student (Ctrl+K)]`, `[Batch Attendance (G A)]`.
  - Today's Metrics: New Admissions Today, Active Inquiries / Lookups, Batch Attendance Status (completed vs. pending for today's batches).
  - Navigation Rail restricted to: `Dashboard`, `Students`, `Attendance`.

### AC-3: Cashier / Accounts Dashboard View
- Dashboard displays:
  - 4 primary action cards: `[Receive Payment (N P)]`, `[Dues Directory (G F)]`, `[Record Expense]`, `[Close Cash Drawer]`.
  - Today's Metrics: Today's Total Collection (৳), Cash in Drawer, Unpaid Dues Count, Open Drawer Status.
  - Recent Transactions table: Last 10 payments with receipt number, student name, amount, and 1-click reprint.
  - Navigation Rail restricted to: `Dashboard`, `Payments`, `Reports` (Financial), `Students` (view-only).

### AC-4: Teacher Dashboard View
- Dashboard displays:
  - "My Batches Today" section: Lists batches assigned to the logged-in teacher profile, class time, room, and attendance status with `[Take Attendance]` action.
  - "Exams Pending Mark Entry": Lists exams with unverified marks for the teacher's subjects with `[Enter Marks]` action.
  - Today's attendance percentage across teacher's batches.
  - Navigation Rail restricted to: `Dashboard`, `Attendance`, `Exams`, `Classes`.

### AC-5: Administrator Dashboard View
- Dashboard displays:
  - Institution-wide operations pulse: Overall Attendance %, Daily Fee Collection, Total Active Students, Outbox Queue Status.
  - Quick action links for Batch Transfers, Exam Publishing, and Broadcast Messaging.
  - Full operational navigation rail.

### AC-6: Owner Executive Cockpit & "View As Role" Mode
- Dashboard displays:
  - Financial health cards (Revenue trend, Cash-in-hand, Overdue receivables).
  - Student enrollment growth trajectory.
  - System safety status: Last backup timestamp, verification badge, active license info.
  - **"View As Role" Preview Switcher**: Interactive selector in header/dashboard allowing the Owner to switch the interface rendering to `Receptionist`, `Cashier`, `Teacher`, or `Admin` mode without changing authentication state.
  - An amber banner indicates when preview mode is active, with a 1-click `[Exit Preview]` button.
