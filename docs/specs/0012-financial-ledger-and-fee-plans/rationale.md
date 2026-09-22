# 0012. Financial ledger model and fee plans (Rationale)

## Context

Coaching centers in Bangladesh operate in high volume, cash heavy environments where tuition fees, admission charges, and model test fees are collected daily at the front desk. Power cuts are frequent and internet connectivity is erratic. Under these conditions, typical accounting software that relies on mutable total columns (such as a single `paid_amount` updated with each payment) frequently suffers from silent data corruption, lost audit trails, and cashier disputes.

When owners suspect cashier leakage or balance discrepancies, traditional software provides no reliable mathematical proof of what transpired. If an operator edits a student profile or changes a fee amount, previous balance calculations become untrustworthy. Furthermore, monthly fee generation across hundreds or thousands of students must be resilient against interruptions; if an administrator clicks generate twice during a power glitch, duplicate charges must not be created.

Finally, students often pay tuition in advance for multiple months or join an institute mid month. Handling advance credit manually at the counter slows down customer queues and introduces human math errors. The application requires an append only financial ledger with integer precision, automatic credit allocation, and mathematically proven consistency invariants that can survive power cuts and operating system crashes.

## Options considered

### Option 1: Append only relational ledger with synchronous balance cache and proptest verification

Store every financial action as an immutable row in distinct relational tables (charges, payments, allocations, adjustments, refunds). Enforce integrity with SQLite database triggers that reject raw SQL updates and deletes on posted rows. Maintain a dedicated balance cache table (`student_balances`) updated synchronously within the same immediate transaction. Prevent duplicate monthly billing with a unique partial database index on `(enrollment_id, fee_head_id, period)`. Apply advance credit automatically to open charges in oldest first order, prioritizing tuition. Verify the entire system using the `proptest` crate with 100,000 randomized state transitions.

**Pros**:
- Mathematical certainty that ledger entries match cached balances at all times.
- Database triggers protect against accidental data mutation even if third party SQLite tools access the database.
- Idempotent charge generation eliminates duplicate billing risk during power failures.
- Fast read performance for front desk cashier screens without recalculating full ledger histories on every keystroke.
- Property testing detects rare edge cases, race conditions, and rounding drift before release.

**Cons**:
- Corrections cannot be made by simple row editing; operators must post formal waiver or adjustment entries with reasons.
- Slightly higher database storage footprint compared to simple mutable balance rows.

### Option 2: Traditional double entry accounting ledger with journal entries and accounts

Implement a full double entry book keeping system with general ledger accounts, debits, and credits (Assets, Liabilities, Equity, Revenue, Expenses). Every student charge creates a debit to Accounts Receivable and a credit to Tuition Revenue. Every payment creates a debit to Cash and a credit to Accounts Receivable.

**Pros**:
- Conforms to standard corporate accounting principles and makes external financial auditing straightforward.
- Easily handles broader institute expenses, asset depreciation, and bank reconciliation within one unified structure.

**Cons**:
- Substantial engineering complexity for coaching center staff who are not trained accountants.
- Front desk cashiers find debit and credit terminology confusing, making everyday counter operations slower and prone to data entry errors.
- Out of scope for version 1.0 single PC requirements, violating the principle of building only what the coaching center requires today.

### Option 3: Event sourced ledger storing raw serialized mutation events

Persist every financial transaction as an immutable serialized event in an append only log table. Derive all balances, due statements, and charge records on demand or via asynchronous read model projections.

**Pros**:
- Complete historical event replay and time travel auditing capabilities.
- Simple write path consisting solely of appending event records to a single table.

**Cons**:
- Complex read queries on low power office hardware, requiring rebuilding state in memory or complex projection tables.
- Harder to enforce strict foreign key constraints and immediate referential integrity across students, batches, and programs.
- Overkill for a local desktop SQLite database where immediate ACID transactions provide equivalent auditability with simpler query patterns.

## Rationale

Option 1 is selected because it directly addresses the operating reality of coaching centers in Bangladesh without imposing the excessive cognitive and architectural overhead of enterprise double entry accounting or event sourcing.

A relational ledger with discrete tables for charges, payments, and allocations matches how institute owners think: a student is billed a specific fee for a specific month, pays cash or mobile money at the counter, and receives a printed receipt. By storing amounts as integer poisha and enforcing immutability through SQLite triggers, Coachify guarantees tamper proof history while retaining instantaneous querying on budget desktop computers.

The decision to apply advance credit automatically in oldest first order frees front desk receptionists from mental math, while the unique partial index on `(enrollment_id, fee_head_id, period)` makes monthly billing generation inherently crash safe. Verifying this model with 100,000 property test iterations provides the mathematical proof required for a GA tier financial core.
