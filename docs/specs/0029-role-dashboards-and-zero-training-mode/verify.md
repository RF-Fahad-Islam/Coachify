# Verify: Role Dashboards & Zero Training Mode (spec 0029)

Steps derived from spec 0029 acceptance criteria.

## UI and Manual Verification

- [x] Log in as a user with the **Receptionist** role:
  - Verify that the dashboard presents 3 prominent action cards: `[Admit New Student]`, `[Search Student]`, and `[Batch Attendance]`.
  - Verify that the left navigation sidebar shows only `Dashboard`, `Students`, and `Attendance`. All financial, settings, and exam management items are hidden (verifies AC-1, AC-2).
- [x] Log in as a user with the **Cashier** role:
  - Verify that the dashboard presents action cards for `[Receive Payment]`, `[Dues Directory]`, `[Record Expense]`, and `[Close Cash Drawer]`.
  - Verify that today's collection total, cash in drawer, and recent 10 payments are displayed with a 1-click receipt reprint button.
  - Verify that the sidebar displays only `Dashboard`, `Payments`, `Reports`, and `Students` (verifies AC-3).
- [x] Log in as a user with the **Teacher** role:
  - Verify that the dashboard shows "My Batches Today" with direct `[Take Attendance]` shortcuts and draft exams awaiting mark entry with `[Enter Marks]`.
  - Verify that the sidebar shows only `Dashboard`, `Attendance`, `Exams`, and `Classes` (verifies AC-4).
- [x] Log in as an **Admin** user:
  - Verify that the operations pulse shows center attendance %, daily intake, and outbox queue status (verifies AC-5).
- [x] Log in as the **Owner**:
  - Verify that the executive dashboard displays full financial metrics, student growth chart, and backup safety status.
  - Open the **"View As Role"** dropdown: select "Receptionist". Verify that the UI instantly transforms into the Receptionist dashboard, the sidebar prunes accordingly, and a visible amber preview banner appears.
  - Click `[Exit Preview]`: verify that the full Owner interface restores immediately (verifies AC-6).

## Commands

- [x] `cargo test -p coachify_core --test reports_and_search_integration` (verifies role seeding for Receptionist and Admin, permission checks, and role resolution, satisfies AC-1).
- [x] `cargo run -p xtask check-types` (verifies role dashboard contract types).
- [x] `pnpm --dir ui test` (verifies role-adaptive dashboard rendering, action card shortcuts, and Owner role preview state transitions).

## Acceptance Criteria Coverage

- AC-1 (Role Seeding & Permissions): verified by Migration 0016 tests and core role-permission assertions.
- AC-2 (Receptionist View): verified by `RoleDashboardView` receptionist test case and sidebar filtering.
- AC-3 (Cashier View): verified by cashier transaction list and drawer status card tests.
- AC-4 (Teacher View): verified by teacher assigned batch roster and mark entry action links.
- AC-5 (Admin View): verified by operations pulse and academic action cards.
- AC-6 (Owner Cockpit & Preview): verified by executive KPI rendering and "View As" switcher state handling.
