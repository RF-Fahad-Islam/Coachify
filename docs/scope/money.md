# Money (Slice 2)

Back to [index](index.md).

### 13. Financial ledger model · needs a decision · GA
Charges, payments, allocations, adjustments, refunds, receipt sequences, and a balance cache, with the invariants that keep them consistent. No mutable paid total, no invoice header in 1.0.
**Done when:** the invariants are written down and enforced by a property based test of 100,000 random operations with zero violations; posted rows cannot be changed.
- [ ] Design it (spec): `/architect financial ledger model`

### 14. Fee plans & charge generation · needs a decision · GA
Fee heads, plans per program or batch with effective dates, student concessions, and an idempotent monthly charge generator that also applies credit.
**Done when:** running generation twice creates no duplicates; a concession changes only future charges; advance credit is applied oldest first; mid month joins follow the chosen rule.
- [ ] Design it (spec): `/architect fee plans & charge generation`

### 15. Print engine & receipts · needs a decision
Print templates rendered and printed through the web runtime's print API, silent for receipts, preview for everything else; per PC printer settings; receipt layouts A4 with two copies, A5, and 80 mm thermal.
**Done when:** a receipt prints silently on the chosen printer in all three layouts with Bangla text, long names, and large amounts; PDF matches print; a missing printer never undoes a payment.
- [ ] Design it (spec): `/architect print engine & receipts`

### 16. Fast cashier & payments · needs a decision · GA
Search a student, see dues by month and recent payments, enter amount and method, receive, print. The core business loop.
**Done when:** payment to printed receipt takes under a second on the reference PC; a double click creates one payment; a changed due is refused with fresh numbers; a power cut leaves the payment fully saved or not at all.
- [ ] Design it (spec): `/architect fast cashier & payments`

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
