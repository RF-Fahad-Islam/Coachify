# Money (Slice 2)

Back to [index](index.md).

### 13. Financial ledger model · done · GA
Charges, payments, allocations, adjustments, refunds, receipt sequences, and a balance cache, with the invariants that keep them consistent. No mutable paid total, no invoice header in 1.0.
**Done when:** the invariants are written down and enforced by a property based test of 100,000 random operations with zero violations; posted rows cannot be changed.
spec [0012](../specs/0012-financial-ledger-and-fee-plans/index.md)
- [x] Design it (spec): `/architect financial ledger model`
- [x] Build it: `/develop financial ledger model`
  - [x] Migration 0012 refining concession batch scoping, unconditional generated charge uniqueness, and triggers, covers AC-1, AC-3, AC-7
  - [x] Atomic payment recording, annual monotonic receipt sequence generation, and synchronous balance cache updater, covers AC-7, AC-8, AC-9
  - [x] Adjustments (waivers with ceiling check, fines), voids, and refund handlers with trigger immutability, covers AC-7, AC-9
  - [x] Comprehensive proptest suite executing 100,000 random state transitions with zero invariant violations, covers AC-10
  - [x] RPC command dispatcher handlers for ledger operations with permissions and TypeScript contract export, covers AC-8
- [x] Verify it: `/check verify financial ledger model`
- [x] Test it: `/test financial ledger model`
- [ ] Review it (fresh model): `/check review financial ledger model`
- [x] Document it: `/document financial ledger model`

### 14. Fee plans & charge generation · done · GA
Fee heads, plans per program or batch with effective dates, student concessions, and an idempotent monthly charge generator that also applies credit.
**Done when:** running generation twice creates no duplicates; a concession changes only future charges; advance credit is applied oldest first; mid month joins follow the chosen rule.
spec [0012](../specs/0012-financial-ledger-and-fee-plans/index.md)
- [x] Design it (spec): `/architect fee plans & charge generation`
- [x] Build it: `/develop fee plans & charge generation`
  - [x] Fee head catalog and program/batch fee plans with integer poisha amounts and effective date validation, covers AC-1, AC-2
  - [x] Concession evaluation engine with half up rounding, batch override hierarchy, and gross clamping, covers AC-3
  - [x] Idempotent chunked monthly fee generation engine with mid month enrollment rules, covers AC-4, AC-5
  - [x] Automatic advance credit allocation engine applying unallocated payment funds oldest first by fee head priority, covers AC-6
  - [x] RPC command dispatcher handlers for fee plans, concessions, and charge generation with permissions, covers AC-1, AC-2, AC-3, AC-4
- [x] Verify it: `/check verify fee plans & charge generation`
- [x] Test it: `/test fee plans & charge generation`
- [ ] Review it (fresh model): `/check review fee plans & charge generation`
- [x] Document it: `/document fee plans & charge generation`

### 15. Print engine & receipts · done · GA
Print templates rendered and printed through the web runtime's print API, silent for receipts, preview for everything else; per PC printer settings; receipt layouts A4 with two copies, A5, and 80 mm thermal.
**Done when:** a receipt prints silently on the chosen printer in all three layouts with Bangla text, long names, and large amounts; PDF matches print; a missing printer never undoes a payment.
spec [0013](../specs/0013-print-engine-and-receipts/index.md)
- [x] Design it (spec): `/architect print engine & receipts`
- [x] Build it: `/develop print engine & receipts`
  - [x] Workstation printer settings repository and system printer enumeration RPCs, covers AC-2, AC-3
  - [x] Printable receipt aggregation query with Bengali and English amount in words, covers AC-1, AC-7
  - [x] React receipt components for 80 mm thermal, A4 dual-copy, and A5 with embedded Bengali fonts, covers AC-1, AC-7
  - [x] Print preview dialog with layout switching, zoom, manual print, and PDF export, covers AC-5, AC-6
  - [x] Tauri desktop silent print dispatch pipeline with fault-tolerant error handling, covers AC-4, AC-8
- [x] Verify it: `/check verify print engine & receipts`
- [x] Test it: `/test print engine & receipts`
- [x] Review it (fresh model): `/check review print engine & receipts`
- [x] Document it: `/document print engine & receipts`

### 16. Fast cashier & payments · done · GA
Search a student, see dues by month and recent payments, enter amount and method, receive, print. The core business loop.
**Done when:** payment to printed receipt takes under a second on the reference PC; a double click creates one payment; a changed due is refused with fresh numbers; a power cut leaves the payment fully saved or not at all.
spec [0014](../specs/0014-fast-cashier-and-payments/index.md)
- [x] Design it (spec): `/architect fast cashier & payments`
- [x] Build it: `/develop fast cashier & payments`
  - [x] Concurrency check with expected balance guard and cashier student overview RPC, covers AC-2, AC-7
  - [x] High-density cashier desk view with due breakdown, advance credit, and recent payments, covers AC-2, AC-3
  - [x] Instant FTS5 student search combobox with keyboard arrow selection and due badges, covers AC-1
  - [x] Quick-fill payment amount shortcuts (Alt+F, Alt+M) and numeric method selectors, covers AC-3, AC-4
  - [x] Atomic payment submission with double-click idempotency and post-payment receipt banner, covers AC-5, AC-6, AC-8, AC-9
- [x] Verify it: `/check verify fast cashier & payments`
- [x] Test it: `/test fast cashier & payments`
- [x] Review it (fresh model): `/check review fast cashier & payments`
- [x] Document it: `/document fast cashier & payments`

### 17. Voids, refunds & adjustments · GA
Same day void with reason and permission, later refunds as their own records, waivers and fines as adjustments, all audited.
**Done when:** a void is only possible on an open day by a permitted user; a refund never exceeds what was paid; every correction shows in the student ledger and the audit log.
- [ ] Build it: `/develop voids, refunds & adjustments`

### 18. Admission with initial payment
A direct admission form that creates student, guardian link, enrollments, admission charges, and an optional first payment in one transaction, then prints the admission receipt and form.
**Done when:** an admission with payment completes in one save and prints; a failure at any step leaves nothing behind.
- [ ] Build it: `/develop admission with initial payment`

### 19. Dues & due statement
Who owes what, filtered by batch, month, and amount, with a printable due statement per student.
**Done when:** the dues list total equals the ledger total; a due statement prints in English or Bangla.
- [ ] Build it: `/develop dues & due statement`

### 20. Expenses & daily cash close · needs a decision · GA
Expense categories and entries, a daily cash summary (opening, received, spent, expected, counted, difference), and closing a day so nothing can be dated into it.
**Done when:** a closed day rejects new payments, expenses, refunds, and voids dated into it; the next day opens with the counted closing; collector wise totals match.
- [ ] Design it (spec): `/architect expenses & daily cash close`
