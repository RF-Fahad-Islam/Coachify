# 0018. Expenses and daily cash close

**Date**: 2026-09-22
**Status**: Accepted

## Summary

In coaching centers and small educational institutes, daily cash management is the most vulnerable operational seam. Tuition payments arrive in physical cash, bKash transfers, and bank deposits throughout the day, while cash is constantly drawn from the counter drawer for daily operating expenses: photocopier paper, whiteboard markers, teacher conveyance, utility bills, tea and refreshments, and minor repairs. Without a disciplined daily cash close, cash shortages go unnoticed, discrepancies compound across weeks, and institute owners cannot verify whether the drawer cash matches reported receipts.

Feature 20 establishes the **Daily Cash Close & Expense Desk**:
1. **Expense Management**: Categorized expense recording (`expense_categories`, `expenses`) with amount in poisha, payment method (`cash`, `bkash`, `nagad`, `bank`, `other`), payee, voucher reference, receipt photo attachments stored in the content-addressed document vault, and same-day voiding with audited reasons.
2. **Daily Cash Reconciliation**: Real-time aggregation of opening drawer cash, cash collected from student fees, cash paid out for expenses and refunds, and the mathematically expected closing balance:
   $$\text{expected closing} = \text{opening cash} + \text{cash in} - \text{cash out}$$
   Non-cash collections (bKash, Nagad, Bank, Card) are tracked and broken down separately to reconcile against merchant statement accounts without polluting drawer cash totals.
3. **Closing Transaction & Retroactive Mutation Lock**: Counter cashiers count the physical cash in the drawer and record the `counted_closing`. The system logs any shortage or surplus (`difference_poisha`), records an explanation note, and locks the business date. Once closed, database triggers and core validators **strictly reject** any payment, expense, refund, or void dated into that closed day or any earlier closed date.
4. **Next-Day Opening Balance Carry-Forward**: The subsequent business day automatically initializes its opening balance with the physical counted closing cash of the preceding closed day, ensuring continuous cash flow continuity.
5. **Printable Daily Cash Close Sheet**: Official A4 portrait or 80 mm thermal close summary sheet for institute director review and physical cashier sign-off.

## Requirements

**User stories**:
- As a counter cashier, I want to record small petty cash expenses (like markers, paper, or tea) directly from the desk, so that every taka leaving the drawer is accounted for.
- As a receptionist, I want to take a photo of an expense paper voucher or receipt and attach it to the expense entry, so that receipts are never lost.
- As an accountant, I want to close the daily cash drawer at the end of the shift, comparing the physical cash count against system-calculated expected cash.
- As an institute owner, I want the system to calculate any cash difference (shortage or surplus) and require an explanation note, so that discrepancies are investigated immediately.
- As an institute owner, I want a closed business day to be permanently locked, so that staff cannot back-date transactions or alter historical figures after money is counted and handed over.
- As a cashier starting the morning shift, I want the new day's opening drawer cash to match yesterday's counted closing cash automatically, so that drawer continuity is maintained.
- As a center manager, I want a printable Daily Cash Close Summary sheet with collector-wise breakdowns and signature lines to file in our physical records.

**Acceptance criteria**:
- **AC-1**: Expense categories catalog (`expense_categories`) supports default categories (*Office Rent*, *Teacher Honorarium*, *Staff Salaries*, *Printing & Stationery*, *Utilities*, *Student Refreshments*, *Marketing*, *Maintenance*, *Miscellaneous*) and custom categories, preventing deletion of categories referenced by active expenses.
- **AC-2**: Expense recording (`expenses.create`) validates positive integer poisha amount, payee name, payment method (`cash`, `bkash`, `nagad`, `bank`, `other`), valid category, and open cash day status. Allows optional receipt image attachment stored in the SHA-256 local document vault.
- **AC-3**: Same-day expense voiding (`expenses.void`) is permitted only on an open cash day (`cash_days.closed_at IS NULL`) for transactions dated today, requiring a mandatory `void_reason` and permission check.
- **AC-4**: Real-time daily cash summary query (`cash_days.get_summary`) aggregates:
  - `opening_cash_poisha`: carried forward from previous day's `counted_closing_poisha`.
  - `cash_in_poisha`: total cash received from posted payments (`method = 'cash' AND status = 'posted'`).
  - `cash_out_poisha`: total cash disbursed for posted expenses (`method = 'cash' AND status = 'posted'`) plus cash refunds (`method = 'cash'`).
  - `expected_closing_poisha`: $\text{opening} + \text{cash in} - \text{cash out}$.
  - Non-cash breakdown: itemized receipt totals and transaction counts for bKash, Nagad, Bank, and Card.
  - Collector-wise breakdown: itemized receipt count and cash collected per counter cashier user.
- **AC-5**: Closing a cash day (`cash_days.close`) requires the `cash_day.close` permission, captures the physical `counted_closing_poisha`, calculates `difference_poisha = counted - expected`, records an optional closing explanation note, and stamps `closed_by` and `closed_at`.
- **AC-6**: **Retroactive Mutation Lock (Invariant 5)**: SQLite database trigger `trg_cash_days_closed_immutable` and core validation prevent inserting, updating, or voiding any payment, expense, refund, or adjustment whose `business_date` belongs to a closed cash day, strictly rejecting mutations with error code `closed_cash_day`.
- **AC-7**: Opening a new business day (`cash_days.get_or_open_today`) automatically initializes `opening_cash_poisha` with the `counted_closing_poisha` of the most recently closed cash day. If no prior cash day exists (first day of institute operation), opening cash defaults to zero or an initial float amount.
- **AC-8**: Printable Daily Cash Close Sheet rendered in React (`ui/src/components/print/cash-close-sheet-template.tsx`) in A4 portrait and 80 mm thermal formats, displaying opening balance, collections by channel, expenses by category, refunds, expected vs. counted cash, shortage/surplus, collector breakdown, and dual signature lines (Cashier, Owner/Director).
- **AC-9**: High-density Cash Close & Expense Desk UI in `/expenses` or under `/payments` with tabbed views for Expense Entry, Expense History, and Daily Cash Close Reconciliation.

## Decision

**Chosen option**: Option 1: Immutable Cash Day State Machine with Automatic Previous-Day Balance Carry-Forward, Strict Database Retroactive Mutation Lock, and Multi-Channel Reconciliation.

Implementation lives in:
- Backend: `crates/core/src/ledger.rs` (expense methods, cash day state machine, summary aggregator), `crates/core/src/dispatcher.rs` (RPC endpoints `expenses.create`, `expenses.list`, `expenses.void`, `expense_categories.list`, `expense_categories.create`, `cash_days.get_summary`, `cash_days.close`, `cash_days.get_or_open_today`).
- Database: Triggers in `migrations/0006_audit_and_triggers.sql` and refinement migration for cash-day mutation locks.
- Frontend: `ui/src/components/cashier/cash-close-view.tsx`, `ui/src/components/cashier/expense-entry-dialog.tsx`, `ui/src/components/print/cash-close-sheet-template.tsx`.

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Daily cash close state machine & drawer lifecycle

```
 [ Morning Start ]
         │
 ┌───────▼──────────────────────────────────────────────────┐
 │ GET OR OPEN TODAY'S CASH DAY                             │
 │ Opening Cash = Counted Closing of Previous Closed Day    │
 └───────┬──────────────────────────────────────────────────┘
         │
 ┌───────▼──────────────────────────────────────────────────┐
 │ BUSINESS OPERATIONS (Day is OPEN)                        │
 │ • Receive payments (Cash in drawer, bKash, Bank)         │
 │ • Pay expenses (Cash out of drawer)                      │
 │ • Issue refunds (Cash out of drawer)                     │
 │ • Same-day voids allowed with reason                     │
 └───────┬──────────────────────────────────────────────────┘
         │ End of shift / Evening
 ┌───────▼──────────────────────────────────────────────────┐
 │ CASH DRAWER RECONCILIATION                               │
 │ Expected = Opening + Cash In - Cash Out                  │
 │ Staff counts physical cash: Counted Closing              │
 │ Difference = Counted - Expected (Shortage / Surplus)     │
 │ Cashier enters closing note                              │
 └───────┬──────────────────────────────────────────────────┘
         │ Action: cash_days.close
 ┌───────▼──────────────────────────────────────────────────┐
 │ DAY CLOSED & PERMANENTLY LOCKED                          │
 │ closed_at = now(), closed_by = user_id                   │
 │ 🚫 Trigger strictly rejects any new/edited/voided rows   │
 │    dated on or before this business date!                │
 │ 🖨 Print Daily Cash Close Sign-Off Sheet                 │
 └──────────────────────────────────────────────────────────┘
```

### Data model & RPC DTOs

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateExpenseInput {
    pub category_id: String,
    pub amount_poisha: i64,
    pub method: String,
    pub payee: String,
    pub reference: Option<String>,
    pub business_date: Option<String>,
    pub attachment_vault_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CloseCashDayInput {
    pub business_date: String,
    pub counted_closing_poisha: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CashDaySummary {
    pub business_date: String,
    pub is_closed: bool,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub opening_cash_poisha: i64,
    pub cash_in_poisha: i64,
    pub cash_out_poisha: i64,
    pub expected_closing_poisha: i64,
    pub counted_closing_poisha: Option<i64>,
    pub difference_poisha: Option<i64>,
    pub note: Option<String>,
    pub digital_collections: Vec<MethodCollectionSummary>,
    pub collector_breakdown: Vec<CollectorSummary>,
    pub expenses_by_category: Vec<CategoryExpenseSummary>,
}
```

## Build plan

1. Implement expense category management (`create_expense_category`, `list_expense_categories`) and expense repository (`create_expense`, `list_expenses`, `void_expense`) in `crates/core/src/ledger.rs`, satisfies **AC-1**, **AC-2**, **AC-3**.
2. Implement cash day aggregation query (`get_cash_day_summary`) computing opening, cash-in, cash-out, expected closing, digital channel separation, and collector-wise totals, satisfies **AC-4**.
3. Implement cash day closing transaction (`close_cash_day`) with counted closing input, variance calculation, and closing timestamp, satisfies **AC-5**.
4. Enforce strict retroactive mutation lock via SQLite trigger and core validation rejecting writes on closed days, satisfies **AC-6**.
5. Implement `get_or_open_today_cash_day` carrying forward the prior closed day's counted balance, satisfies **AC-7**.
6. Build printable React Cash Close Sheet template `ui/src/components/print/cash-close-sheet-template.tsx` supporting A4 and 80 mm thermal layouts, satisfies **AC-8**.
7. Build `CashCloseView` and `ExpenseEntryDialog` in `ui/src/components/cashier/`, satisfies **AC-9**.
8. Write comprehensive integration tests verifying balance carry-forward, variance calculation, and mutation rejection on closed days, satisfies **AC-4**, **AC-5**, **AC-6**, **AC-7**.

## Consequences

**Positive**:
- Eliminates unaccounted cash loss: every taka entering or leaving the physical drawer is tracked and balanced daily.
- Permanent audit security: closing a day creates an immutable financial boundary that cannot be tampered with.
- Seamless morning starts: next-day opening balance is always mathematically tied to yesterday's physical close.
- Separates physical cash from digital wallets: bKash and bank payments are tracked without distorting cash drawer counts.

**Negative / Tradeoffs**:
- Requires discipline from counter staff to close the day before leaving. If staff forget, the day remains open until closed the following morning.
