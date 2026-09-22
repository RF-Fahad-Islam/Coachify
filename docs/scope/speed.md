# Reports & speed (Slice 6)

Back to [index](index.md). Speed and UX features that make Coachify feel instant for daily operations.

### 30. Reports center & exports · needs a decision
Student, attendance, fees, dues, finance, exam, batch, and operational reports. Covers the full reporting needs for a coaching center owner, plus CSV/Excel export from the report views.
**Done when:** every report loads within 2 seconds on the reference PC with the large dataset; filters (batch, date range, session) apply correctly; totals match the source data; exports open cleanly in Excel.
- [ ] Design it (spec): `/architect reports center & exports`

### 31. Role dashboards & zero training mode · needs a decision
Reception, Accounts, Teacher, and Admin get simplified role-specific interfaces. Each role sees only their relevant features, arranged for their daily workflow. The Owner sees the full interface.
**Done when:** a receptionist logging in sees admissions, student search, and attendance; an accounts user sees cashier, dues, and expenses; a teacher sees only their assigned batches' attendance and marks; switching to the Owner role reveals the full menu.
- [ ] Design it (spec): `/architect role dashboards & zero training mode`

### 32. Command palette · needs a decision
`Ctrl+K` palette for payment, admission, attendance, student search, printing, and navigation. Every keyboard shortcut in the app is discoverable through the palette.
**Done when:** `Ctrl+K` opens instantly; typing "pay" shows the cashier shortcut; typing a student name navigates to their profile; every registered shortcut appears in the palette.
- [ ] Design it (spec): `/architect command palette`

### 33. Fast global search · needs a decision
Search by name, ID, guardian phone, batch, receipt number, or payment information. Unified search bar that searches across all entity types with clear result categories.
**Done when:** searching a guardian phone number finds all their children; searching a receipt number opens the payment; results appear within 100 ms on the reference PC; search handles partial matches and common misspellings in Bangla.
- [ ] Design it (spec): `/architect fast global search`
