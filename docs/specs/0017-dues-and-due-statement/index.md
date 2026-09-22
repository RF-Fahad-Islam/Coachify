# 0017. Dues and due statement

**Date**: 2026-09-22
**Status**: Accepted

## Summary

Maintaining positive cash flow in a coaching center or private institute requires relentless visibility into outstanding receivables. At any point in the month, center directors and accountants must know exactly which students have unpaid fees, broken down by program, batch, billing month, and overdue age. When parents question their balances or request a fee breakdown before exams, the institute must provide an authoritative, bilingual **Student Due Statement** (Account Statement) itemizing all charges, concessions, payments, and adjustments.

Feature 19 establishes the **Dues Management & Statement Engine**:
1. **Dues Center View**: A high-performance, filterable directory of all active students with outstanding dues (`balance_poisha > 0`), filterable by Program, Batch, Academic Session, Billing Month, and overdue aging thresholds (Current, 30+ days, 60+ days overdue), with instant single-click transitions to the Fast Cashier Desk.
2. **Strict Financial Balance Invariant**: Guaranteed mathematical equivalence where the sum of balances in the Dues Center strictly equals the aggregate balance in the synchronous `student_balances` cache:
   $$\sum \text{Dues List Balances} = \sum \text{student\_balances.balance\_poisha}$$
3. **Printable Bilingual Due Statement**: An official document rendered via the WebView2 print engine (Spec 0013) in A4 portrait and A5 formats, featuring the institute's crest, student particulars, itemized charge ledger, amount in words in both English and Bengali (`words.rs`), payment account details (bKash, Nagad, Bank), and formal signature blocks.

## Requirements

**User stories**:
- As an institute accountant, I want to filter overdue fees by batch and billing month, so that I can identify students who have not paid tuition for the current session.
- As a cashier, I want to click a student in the dues list to open them directly in the Fast Cashier Desk, so that I can collect their payment immediately without re-typing their name.
- As an institute director, I want to see the total outstanding dues across the institute and an aging breakdown, so that I understand our cash collection efficiency.
- As a parent, I want a printed statement showing every fee charged, discount applied, and payment made, so that I can verify my child's account history before paying.
- As an administrator, I want to print due statements in bulk for an entire batch or send automated SMS reminders, so that parents are notified before exam deadlines.

**Acceptance criteria**:
- **AC-1**: Dues aggregation query (`ledger.list_dues`) retrieves all students with positive balances (`balance_poisha > 0`), supporting filtering by `program_id`, `batch_id`, `session_id`, `period` (`YYYY-MM`), minimum due threshold (`min_due_poisha`), and text search (Student Code, Name, Phone, Roll) in under 100 ms on a 10,000-student dataset.
- **AC-2**: Aging breakdown classifies dues into aging buckets: Current Month (due within 30 days), 30–60 Days Overdue, and 60+ Days Seriously Overdue, based on the oldest unpaid charge's `due_date`.
- **AC-3**: Strict financial invariant: the total due reported across all active students in the dues query must strictly equal the sum of net charges minus posted payments minus waivers plus fines plus refunds in `student_balances` with zero poisha drift.
- **AC-4**: Printable Due Statement query (`ledger.get_due_statement`) returns the student's complete financial profile: institute header info, student demographics, enrollment details, itemized historical ledger entries (charges with due dates, concessions, payments with receipt numbers, waivers, fines), total charges, total paid, net due, and amount in words in English and Bengali.
- **AC-5**: Due Statement template rendered in React (`ui/src/components/print/due-statement-template.tsx`) supports A4 portrait and A5 formats with SolaimanLipi / Kalpurush Bengali typography, institute crest, payment channel instructions (bKash merchant number, bank account details), and dual signature blocks (Accountant, Director).
- **AC-6**: Interactive Dues Center UI (`/dues` or integrated into Payments/Reports) displays summary cards (Total Dues BDT, Due Student Count, Overdue >30 Days Amount), a responsive data table with aging badges, batch filters, and quick action buttons: `[Collect Payment]` (deep-links to Cashier Desk), `[Print Statement]`, `[Send Reminder SMS]`.
- **AC-7**: Batch print and export: support batch printing of due statements for all filtered students and CSV/Excel export for offline accounting reviews.

## Decision

**Chosen option**: Option 1: Aggregated Dues View with Strict Cache Reconciliation, Embedded Aging Categorization, and WebView2 Printable Account Statements.

Implementation lives in:
- Backend: `crates/core/src/ledger.rs` (queries `list_dues` and `get_due_statement`), `crates/core/src/dispatcher.rs` (RPC endpoints `ledger.list_dues`, `ledger.get_due_statement`).
- Frontend: `ui/src/components/cashier/dues-center-view.tsx` (dues directory with aging badges and filters), `ui/src/components/print/due-statement-template.tsx` (printable account statement component).

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Dues Center layout & workflow

```
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │ DUES & RECEIVABLES CENTER                                                       │
 ├───────────────────┬───────────────────┬───────────────────┬─────────────────────┤
 │ Total Institute   │ Due Students      │ Overdue (>30 Days)│ Collection Rate     │
 │ ৳ 8,42,500        │ 342 Students      │ ৳ 2,15,000        │ 88.4% This Month    │
 └───────────────────┴───────────────────┴───────────────────┴─────────────────────┘
 ┌─────────────────────────────────────────────────────────────────────────────────┐
 │ Filters: [ Program: HSC Science v ] [ Batch: Physics A v ] [ Min Due: ৳1,000 v ]│
 │ Search: [ Tanvir, 0171...                                 ] [ Export CSV / Print]│
 ├───────────┬──────────────┬──────────────────┬───────────┬───────────┬───────────┤
 │ ID        │ Student Name │ Batch & Roll     │ Overdue   │ Total Due │ Action    │
 ├───────────┼──────────────┼──────────────────┼───────────┼───────────┼───────────┤
 │ STU-0042  │ Tanvir Ahmed │ HSC Phys A (#12) │ ৳400 [30d]│ ৳ 2,200   │ [Pay] [🖨] │
 │ STU-0089  │ Sumaiya Akter│ HSC Chem B (#05) │ --        │ ৳ 1,500   │ [Pay] [🖨] │
 └───────────┴──────────────┴──────────────────┴───────────┴───────────┴───────────┘
```

### Data model & RPC DTOs

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListDuesFilter {
    pub program_id: Option<String>,
    pub batch_id: Option<String>,
    pub period: Option<String>, // "2026-09"
    pub min_due_poisha: Option<i64>,
    pub overdue_only: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentDueSummaryItem {
    pub student_id: String,
    pub student_code: String,
    pub name: String,
    pub name_bn: Option<String>,
    pub phone: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub roll_no: Option<String>,
    pub current_month_due_poisha: i64,
    pub overdue_poisha: i64,
    pub total_due_poisha: i64,
    pub oldest_unpaid_due_date: Option<String>,
    pub last_payment_date: Option<String>,
    pub last_payment_amount_poisha: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DueStatementData {
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub institution_address: String,
    pub institution_phone: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub statement_date: String,
    pub itemized_charges: Vec<StatementChargeLine>,
    pub recent_payments: Vec<StatementPaymentLine>,
    pub total_charged_poisha: i64,
    pub total_concession_poisha: i64,
    pub total_paid_poisha: i64,
    pub total_adjusted_poisha: i64,
    pub net_due_poisha: i64,
    pub amount_in_words_en: String,
    pub amount_in_words_bn: String,
    pub payment_instructions: Option<String>,
}
```

## Build plan

1. Implement `list_dues` query in `crates/core/src/ledger.rs` with batch, period, and threshold filtering plus overdue aging calculation, satisfies **AC-1**, **AC-2**.
2. Assert strict mathematical reconciliation between `list_dues` aggregate totals and `student_balances` cache, satisfies **AC-3**.
3. Implement `get_due_statement` query in `crates/core/src/ledger.rs` assembling itemized charges, payments, concessions, and bilingual words-in-BDT, satisfies **AC-4**.
4. Build printable React Due Statement component `ui/src/components/print/due-statement-template.tsx` with SolaimanLipi typography, crest, and signature blocks, satisfies **AC-5**.
5. Build `DuesCenterView` in `ui/src/components/cashier/dues-center-view.tsx` with summary cards, aging badges, and quick-pay deep link, satisfies **AC-6**.
6. Implement batch statement printing and CSV export actions, satisfies **AC-7**.
7. Write unit and performance tests verifying sub-100ms response time on 10,000 student demo datasets, satisfies **AC-1**, **AC-3**.

## Consequences

**Positive**:
- Clear institutional receivables visibility: management instantly spots chronic non-payers.
- Professional account statements: parents receive unambiguous, bilingual statements of account.
- Instant cash collection link: eliminates friction between spotting a due and taking money at the counter.

**Negative / Tradeoffs**:
- Bulk printing for hundreds of students requires careful pagination to avoid browser memory spikes.
