# 0018. Expenses and daily cash close (Rationale)

## Context

In Bangladesh, coaching centers and local academic institutes operate predominantly on physical cash transactions. Counter receptionists take cash for monthly fees, hand out paper receipts, and use the same drawer cash to pay for daily operational expenses—printer paper, whiteboard markers, drinking water jars, tea, electric bills, and conveyance for visiting teachers.

Without a disciplined daily closing procedure, this setup inevitably leads to operational chaos:
1. **Cash Bleed**: Small unrecorded expenses accumulate. By month's end, the institute director finds a large discrepancy between recorded fee income and actual bank/cash deposits.
2. **Back-Dating Fraud**: A dishonest cashier who made a mistake or took money can back-date an expense entry into last week's records, masking the shortage.
3. **Digital vs. Physical Confusion**: Parents frequently pay via bKash, Nagad, or direct bank transfer. If digital payments are lumped into "daily revenue" without separating physical cash, cashiers cannot reconcile their physical drawer balance.
4. **Discontinuous Morning Floats**: Cashiers start each day guessing how much cash was left in the drawer from yesterday, destroying continuous audit accountability.

Coachify addresses this with a formal **Daily Cash Close & Expense Desk** that treats daily drawer reconciliation as a first-class business invariant.

## Options considered

### Option 1: Immutable Cash Day State Machine with Previous-Day Carry-Forward and Multi-Channel Reconciliation

Structure daily accounting around an explicit `cash_days` record for each calendar date:
- Cash-in strictly counts cash payments (`method = 'cash'`). Digital methods (bKash, Nagad, Bank, Card) are tracked in parallel buckets.
- Cash-out sums cash expenses and cash refunds.
- Closing requires entering the counted cash, calculates variance, and permanently locks the business date against retroactive mutations.
- The next day's opening cash automatically inherits the previous day's counted closing cash.

**Pros**:
- Rock-solid physical drawer accountability: cashiers reconcile what is physically in the box against what the system calculated.
- Digital clarity: owners see total collections across bKash, Nagad, and cash without confusing cash drawer counts.
- Strict immutability: closing a day creates an unalterable historical record. Embezzlement via back-dating is mathematically impossible.
- Continuous balance chain: day \(N+1\)'s opening cash is anchored in day \(N\)'s closing count.

**Cons**:
- Requires staff to perform a count at shift end. If a day is forgotten, it must be closed before or during the next morning's opening.

### Option 2: Continuous Running Balance without Formal Daily Closes

Maintain a single un-segmented cash ledger without daily closing boundaries, allowing cashiers to record expenses whenever they occur.

**Pros**:
- Less ceremony; staff never have to perform an explicit "close day" action.

**Cons**:
- Disastrous for cash accountability: shortages cannot be isolated to a specific day or cashier shift.
- Leaves historical records open to retroactive tampering at any time.
- Fails the core requirement of providing daily sign-off sheets for institute owners.

### Option 3: Full Double-Entry Accounting General Ledger

Implement standard chart of accounts, debit/credit journal entries, and general ledger reconciliation.

**Pros**:
- Matches large enterprise ERP accounting software.

**Cons**:
- Vastly over-scoped and inappropriate for coaching centers: front-desk receptionists are not certified accountants and will not enter double-entry debits and credits.
- Violates Coachify's explicit scope boundary: *"Do not build yet: double entry accounting"* (`docs/scope/index.md`).

## Rationale

Option 1 is selected because it strikes the perfect balance between uncompromising financial rigor and front-desk practicality. By modeling the natural daily cash drawer lifecycle (Open -> Transact -> Count -> Close -> Lock), Coachify protects institute owners against cash leaks while keeping front-desk workflows fast, intuitive, and error-free.
