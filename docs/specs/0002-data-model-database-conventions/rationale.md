# Rationale: Data model and database conventions

## Context

Coachify is an offline first Windows desktop application designed specifically for coaching centers, academies, and private education institutions in Bangladesh. These institutions operate in demanding physical environments where electrical power cuts (load shedding) occur frequently, often multiple times per day. Most offices lack enterprise uninterrupted power supplies (UPS), meaning desktop computers lose power instantly while staff enter admissions or process payments. If a database relies on standard operating system disk caching without immediate synchronization, a sudden power cut can leave the database in a corrupted state or lose committed transactions.

Furthermore, coaching centers face internal trust issues. Institute owners are frequently absent from the front desk, delegating fee collection and student registration to salaried cashiers and front desk staff. If staff can edit or delete historical rows in the database, either through the user interface or by opening the database with an external SQLite tool, owners lose confidence in the numbers. Financial history and audit records must be strictly immutable at the database engine level.

Finally, Bangladeshi educational institutions track specific demographic and academic details that differ from generic international school software. Students require birth registration or National ID numbers, blood groups for emergency safety, emergency contact phone numbers normalized to standard Bangladeshi mobile formats (starting with `8801`), guardian relationships, and previous school background. Establishing both foundational entities and the complete financial ledger schema in migration 0002 ensures that later slices can implement cashier and fee workflows without needing disruptive schema restructuring.

## Options considered

### Option 1: Comprehensive core and financial schema in migration 0002 with defense in depth SQLite triggers and synchronous = FULL (Chosen)

This approach delivers the complete schema for institution profile, people directories, academic structure, and the entire financial ledger (charges, payments, allocations, adjustments, refunds, cash days, and expenses) in migration 0002. SQLite write ahead logging (WAL) runs with `synchronous = FULL`, requiring the operating system to flush WAL writes to physical media before a transaction returns success. SQLite triggers enforce immutability directly in the database engine using `RAISE(ABORT)` on audit logs, clock guards, posted payments, and closed cash days, while Rust repository code performs identical validation checks before executing SQL statements.

**Pros**:
- Prepares the complete database table structure and relationships upfront, preventing churn and schema mutations across Slice 1 and Slice 2.
- Guarantees zero data loss and prevents database corruption during unexpected power outages.
- Prevents tampering with audit trails and posted rows even if an administrator opens the file directly with a third party database browser.
- Slices domain logic implementation cleanly while keeping the underlying storage schema stable.

**Cons**:
- Causes an extra disk synchronization call on every transaction commit, introducing a minor write latency on spinning hard disks.
- Financial tables exist in the schema before the Slice 2 user interface code is built.

### Option 2: Sliced rollout deferring financial ledger tables to Slice 2

This approach creates only foundational tables (institution, people, academics, system) in migration 0002 and defers all financial tables to a future migration when Slice 2 begins.

**Pros**:
- Only creates tables that have immediate user interface consumers in Slice 1.

**Cons**:
- Requires multiple schema migrations in rapid succession as soon as financial features begin.
- Splits database trigger definitions and testing across multiple migration scripts.

### Option 3: Application only validation with synchronous = NORMAL

This approach relies exclusively on Rust domain logic to validate constraints and prevent updates to posted records. The database runs with `synchronous = NORMAL`, which flushes WAL writes during checkpointing rather than on every individual commit.

**Pros**:
- Delivers maximum write throughput on low specification computers with spinning magnetic disks.
- Keeps SQL migration scripts simpler by omitting database trigger definitions.

**Cons**:
- Exposes committed transactions to loss or rollback if the computer loses power within a few seconds of a write.
- Allows historical audit and financial records to be modified or deleted if someone accesses the SQLite file outside the application.

## Rationale

Option 1 is selected because delivering the complete core and financial ledger schema in migration 0002 establishes an unshakeable storage contract for both Slice 1 (People and Academics) and Slice 2 (Money). By laying out the tables, foreign keys, unique indexes, and immutability triggers upfront, subsequent feature development can focus on business logic and cashier workflows without altering database definitions.

In Bangladesh, power outages are an everyday occurrence rather than an exceptional disaster. The slight throughput reduction of `synchronous = FULL` on spinning hard drives is negligible for single institution desktop workloads, which typically process between ten and fifty transactions per hour, while the protection against corrupted databases is indispensable.

Enforcing immutability via SQLite engine triggers alongside Rust domain checks provides defense in depth. If a software bug in a future feature inadvertently issues an `UPDATE` or `DELETE` statement on a posted payment or audit row, the database engine itself aborts the query immediately.
