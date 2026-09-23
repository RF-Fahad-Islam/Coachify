# Reports & speed (Slice 6)

Back to [index](index.md). Speed and UX features that make Coachify feel instant for daily operations.

### 30. Reports center & exports
Student, attendance, fees, dues, finance, exam, batch, and operational reports. Covers the full reporting needs for a coaching center owner, plus CSV/Excel export from the report views.
**Done when:** every report loads within 2 seconds on the reference PC with the large dataset; filters (batch, date range, session) apply correctly; totals match the source data; exports open cleanly in Excel.
spec [0028](../specs/0028-reports-center-and-exports/index.md)
- [x] Design it (spec): `/architect reports center & exports`
- [x] Build it: `/develop reports center & exports`
  - [x] Core SQL aggregation engine for daily collections, cashier performance, dues aging (0–30, 31–60, 61–90, 90+), expenses, attendance, and exam merit summaries, covers AC-1, AC-2, AC-3, AC-4, AC-5, AC-6
  - [x] RFC 4180 CSV export serializer with UTF-8 BOM (`\u{FEFF}`) prefix and poisha-to-decimal currency conversion, covers AC-7
  - [x] Sub-2-second query performance benchmarks on reference dataset, covers AC-8
  - [x] High-density reports hub UI with tabbed sub-reports, KPI summary cards, date range picker, and print stylesheet, covers AC-1 through AC-7
- [x] Verify it: `/check verify reports center & exports`
- [x] Test it: `/test reports center & exports`

### 31. Role dashboards & zero training mode
Reception, Accounts, Teacher, and Admin get simplified role-specific interfaces. Each role sees only their relevant features, arranged for their daily workflow. The Owner sees the full interface.
**Done when:** a receptionist logging in sees admissions, student search, and attendance; an accounts user sees cashier, dues, and expenses; a teacher sees only their assigned batches' attendance and marks; switching to the Owner role reveals the full menu.
spec [0029](../specs/0029-role-dashboards-and-zero-training-mode/index.md)
- [x] Design it (spec): `/architect role dashboards & zero training mode`
- [x] Build it: `/develop role dashboards & zero training mode`
  - [x] Migration 0016 schema seeding `Receptionist` and `Admin` system roles with granular permission mappings, covers AC-1
  - [x] Role-tailored Receptionist, Cashier/Accounts, Teacher, and Admin dashboards with high-contrast daily action cards and hotkey badges, covers AC-2, AC-3, AC-4, AC-5
  - [x] Dynamic sidebar navigation pruning based on authenticated permissions, covers AC-2, AC-3, AC-4
  - [x] Owner executive cockpit with financial health indicators and non-destructive "View As Role" preview switcher, covers AC-6
- [x] Verify it: `/check verify role dashboards & zero training mode`
- [x] Test it: `/test role dashboards & zero training mode`

### 32. Command palette
`Ctrl+K` palette for payment, admission, attendance, student search, printing, and navigation. Every keyboard shortcut in the app is discoverable through the palette.
**Done when:** `Ctrl+K` opens instantly; typing "pay" shows the cashier shortcut; typing a student name navigates to their profile; every registered shortcut appears in the palette.
spec [0030](../specs/0030-command-palette/index.md)
- [x] Design it (spec): `/architect command palette`
- [x] Build it: `/develop command palette`
  - [x] Sub-50ms command palette modal dialog with keyboard trap and focus restoration, covers AC-1, AC-6
  - [x] Categorized command groups (Quick Actions, Navigation, Entities, Shortcuts Cheat Sheet) with fuzzy keyword alias matching, covers AC-2, AC-4
  - [x] Global chorded shortcut dispatcher (`G D`, `G S`, `G A`, `G P`, `N S`, `N P`, `Alt T`) with input field typing guard, covers AC-3
  - [x] Role-aware action filtering omitting unauthorized commands, covers AC-5
- [x] Verify it: `/check verify command palette`
- [x] Test it: `/test command palette`

### 33. Fast global search
Search by name, ID, guardian phone, batch, receipt number, or payment information. Unified search bar that searches across all entity types with clear result categories.
**Done when:** searching a guardian phone number finds all their children; searching a receipt number opens the payment; results appear within 100 ms on the reference PC; search handles partial matches and common misspellings in Bangla.
spec [0031](../specs/0031-fast-global-search/index.md)
- [x] Design it (spec): `/architect fast global search`
- [x] Build it: `/develop fast global search`
  - [x] Migration 0016 composite indexing on `payments(receipt_number, student_id, paid_at)`, covers AC-3
  - [x] Unified `search.global` core query engine across Students, Guardians, Batches, Staff, and Receipts with sub-100ms benchmark, covers AC-1, AC-5
  - [x] Digits-only phone normalization and reverse guardian phone lookup finding enrolled siblings, covers AC-2
  - [x] FTS5 trigram Unicode query sanitization and Bengali substring tolerance, covers AC-4, AC-6
  - [x] Top-bar search bar integration with 150ms debouncing and 1-click deep navigation to student profile and receipt reprint, covers AC-1, AC-3
- [x] Verify it: `/check verify fast global search`
- [x] Test it: `/test fast global search`
