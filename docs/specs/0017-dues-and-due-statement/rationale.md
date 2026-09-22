# 0017. Dues and due statement (Rationale)

## Context

In Bangladeshi educational institutions, tuition recovery is the lifeline of the business. Typically, coaching centers allow students to attend classes while dues accrue, collecting fees on a rolling basis throughout the calendar month. However, tracking who owes what becomes unmanageable as enrollment grows beyond a few hundred students.

Without a centralized Dues Center, administrators suffer from severe blind spots:
- They cannot easily tell which batches have the lowest fee collection rates.
- Parents frequently dispute balances, claiming they paid fees that were never properly credited or allocated.
- When cashiers attempt to collect fees, they cannot provide a clear, printed statement itemizing which specific months remain unpaid.
- Staff spend hours manually tabulating balances from paper ledger books.

Coachify requires an aggregated **Dues & Due Statement** system that guarantees real-time mathematical consistency with the underlying ledger while providing professional, dispute-free printed statements.

## Options considered

### Option 1: Live Filtered Dues Index with On-Demand Statement Projection and WebView2 Bilingual Print

Calculate dues dynamically by joining open charges, concessions, payments, and adjustments, verified against the synchronous `student_balances` cache. Render official due statements on-demand using the existing WebView2 print engine.

**Pros**:
- Real-time accuracy: never suffers from stale due totals; any payment recorded instantly removes or reduces dues in the list.
- Guaranteed mathematical consistency: invariant assertions verify that total dues match the ledger balance cache.
- High-quality presentation: uses SolaimanLipi/Kalpurush typography for Bangla student names and institute branding, with amounts written in words.
- Smooth operational flow: cashier can click any student in the dues list to instantly jump to the Fast Cashier Desk with that student pre-loaded.

**Cons**:
- Dynamic joins across charges and payments require proper indexing (`idx_charges_student_status`, `idx_student_balances`) to maintain sub-100ms response times on large datasets.

### Option 2: Pre-Materialized Stored Monthly Invoice Records

Generate formal invoice header documents (`invoices` table) each month, storing immutable snapshots of dues.

**Pros**:
- Traditional enterprise ERP pattern.

**Cons**:
- Inflexible: coaching centers do not operate on rigid corporate invoicing; students join mid-month, transfer batches, or pay partial amounts across multiple months.
- High database bloat: creates thousands of redundant invoice header records.
- Stale numbers: if a charge is waived or adjusted, the invoice header must be mutated or reissued, complicating auditability.

### Option 3: Raw Balance Cache Export Only

Present only the raw `student_balances.balance_poisha` number in a simple list without itemized charge breakdowns or aging categorization.

**Pros**:
- Simplest query to write (`SELECT * FROM student_balances WHERE balance_poisha > 0`).

**Cons**:
- Poor usability: does not tell staff *which* months or fee heads are overdue.
- Cannot produce an explanatory due statement for parents showing line-by-line charges and payments.

## Rationale

Option 1 is selected because it directly aligns with how coaching centers operate in practice: students need clear breakdowns of overdue months, cashiers need immediate deep-linking to collect payments, and parents demand transparent, itemized statements. Reconciling the live dues projection with the `student_balances` cache guarantees that performance is stellar without sacrificing financial correctness.
