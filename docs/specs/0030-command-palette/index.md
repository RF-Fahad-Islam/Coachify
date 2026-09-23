# Spec 0030 (Command Palette)

> Feature 32 · Slice 6 (Reports & Speed)

## Goal

Provide a sub-50ms keyboard command palette accessible via `Ctrl+K` (or clicking the search box) that accelerates daily operations for power users, supports two-key chorded navigation and action hotkeys, enables instant entity discovery, and exposes a comprehensive in-app shortcut cheat sheet for full discoverability.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Activation Key** | `Ctrl+K` (Windows/Linux) and `Cmd+K` (macOS) with TopBar trigger | Universal convention across modern desktop productivity applications; familiar to users |
| **Instant Mounting Target** | Render dialog in under 50 ms with client-side cached command index | Ensures the palette feels instantaneous and fluid without UI stutter |
| **Chorded Keyboard Shortcuts** | Global listener supporting two-key sequences (`G` then `S` for Students, `N` then `P` for New Payment) | Enables power users to navigate the entire app without leaving the home row or reaching for a mouse |
| **Fuzzy Action Matching** | Action aliases (e.g. typing "pay", "fee", "bill" matches `Receive Fee Payment`; "admit", "new" matches `Enroll Student`) | Accommodates colloquial coaching center terminology and natural typing habits |
| **Role-Aware Command Filtering** | Filter action and navigation items based on current authenticated role permissions | Prevents users from seeing or triggering actions they do not have authority to execute |
| **Integrated Entity Search** | Delegating entity queries to `search.global` when input length ≥ 2 characters | Allows users to search and jump directly to a student, batch, receipt, or staff profile from the same palette |

## Requirements

**User stories**:
- As a cashier, I want to press `Ctrl+K`, type "pay", and hit `Enter` to immediately jump to the payment counter, so that I can process fees in seconds.
- As a receptionist, I want to press `N S` anywhere in the app to open the new student admission form without multiple mouse clicks.
- As an institute owner, I want every keyboard shortcut in the application clearly shown in the command palette, so that my team can discover and learn high-speed workflows.
- As a power user, I want arrow keys (`ArrowDown`/`ArrowUp`) to cycle smoothly through commands and `Enter` to execute immediately.

## Acceptance Criteria

### AC-1: Instant Palette Mount & Dismissal
- Pressing `Ctrl+K` (or `Cmd+K`) or clicking the top-bar search bar opens the Command Palette in under 50 ms.
- Pressing `Escape` or clicking outside dismisses the palette and restores focus to the previously active element.

### AC-2: Categorized Command Groups
- Commands are organized into distinct visual groups:
  - **Quick Actions**: Top mutation actions (`Receive Fee Payment`, `Admit New Student`, `Mark Batch Attendance`, `Record Petty Expense`, `Close Cash Drawer`, `Create New Exam`, `Trigger Database Backup`, `Customize Theme`).
  - **Navigation**: Instant jump to all accessible tabs and sub-tabs (`Dashboard`, `Students`, `Attendance`, `Communication`, `Payments: Counter / Dues / Cash Close`, `Exams`, `Classes`, `Teachers`, `Reports`, `Settings`).
  - **Live Search Entities**: Dynamic hits returned by `search.global` (Students, Batches, Receipts, Staff) displayed with colored badges.
  - **Keyboard Shortcuts**: Built-in shortcut cheat sheet displaying registered keys.

### AC-3: Global Chorded Shortcuts
- The application registers and listens for chorded navigation and action hotkeys:
  - `G D`: Go to Dashboard
  - `G S`: Go to Students Directory
  - `G A`: Go to Batch Attendance
  - `G P` / `G F`: Go to Payments & Finance
  - `G E`: Go to Exams Hub
  - `G R`: Go to Reports Center
  - `G T`: Go to Settings
  - `N S`: New Student Admission
  - `N P`: New Fee Payment
  - `Alt T`: Open Theme & Appearance Customizer
  - `?`: Open Shortcut Discoverability Sheet
- Chords do not trigger when the user is actively typing inside an `<input>`, `<textarea>`, or contenteditable field.

### AC-4: Fuzzy Keyword & Alias Matching
- Searching matches both command labels and predefined search keywords/aliases:
  - `"pay"`, `"fee"`, `"money"`, `"taka"` → `Receive Fee Payment`
  - `"admit"`, `"enroll"`, `"new student"`, `"vorti"` → `Admit New Student`
  - `"attend"`, `"hajira"`, `"presence"` → `Mark Batch Attendance`
  - `"due"`, `"baki"` → `Dues Directory`
  - `"drawer"`, `"close"`, `"reconcile"` → `Daily Cash Close`
  - `"backup"`, `"snapshot"` → `Trigger Database Backup`

### AC-5: Role-Aware Filtering
- Command items are filtered by active user permissions:
  - A user without `payments.collect` will not see `Receive Fee Payment`.
  - A user without `backup.manage` will not see `Trigger Database Backup`.
  - A user with `Owner` role sees all actions and administrative shortcuts.

### AC-6: Visual Feedback & Accessibility
- Full ARIA compliance (`role="dialog"`, `aria-expanded`, `aria-activedescendant`).
- High-contrast visual focus ring on selected item.
- Explicit `<kbd>` shortcut badges rendered on the right side of each command item.
