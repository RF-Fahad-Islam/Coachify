# 0014. Fast cashier and payments (Rationale)

## Context

Tuition collection in Bangladeshi coaching centers peaks during two specific windows: the first week of every calendar month when monthly tuition fees fall due, and the 15-minute rush before and after class batches start. During these peak windows, dozens of students and parents arrive at the counter simultaneously to pay tuition, exam fees, or purchase lecture sheets.

In typical legacy desktop software or sluggish web apps, receiving a payment requires navigating through multiple menus: opening a student directory, selecting the student, clicking an "Invoices" tab, opening a payment dialog, manually calculating partial balances, clicking save, waiting for an OS print prompt, and clicking print. This process takes 30 to 60 seconds per student, resulting in counter gridlock and frustrated parents.

To keep counter queues moving rapidly, Coachify requires a dedicated **Fast Cashier Desk** that unifies student search, due inspection, payment entry, and silent receipt printing into a single high-density screen operable entirely from the keyboard in under 5 seconds per transaction.

Furthermore, coaching centers with multiple counters or those running automated background fee generation risk race conditions: two cashiers might attempt to collect fees from the same student at the same moment, or fees might change while a payment is being entered. The system must guarantee that concurrent conflicts are detected and rejected cleanly without corrupting financial balances or producing duplicate payments.

## Options considered

### Option 1: Dedicated High-Density Fast Cashier Desk with Optimistic Concurrency Balance Checking and Keyboard-First Ergonomics

Build a dedicated full-screen counter module (`/payments`) featuring:
- Persistent search input with keyboard focus (`F2` or `/`) and live due status badges.
- Split-pane layout: student & due summary on the left, payment inputs and quick-fill action buttons on the right.
- Single-keystroke shortcuts for quick amounts (`Alt+F` for Full, `Alt+M` for Current Month) and methods (`1` to `6`).
- Client-side UUIDv7 idempotency key and immediate button disablement preventing double-clicks.
- Optimistic concurrency guard via `expected_balance_poisha` checking for intervening mutations before commit.
- Chained silent printing dispatch triggering the receipt printer immediately upon database commit.

**Pros**:
- Maximum counter throughput: complete transaction executed in under 1 second, reducing customer wait times.
- Ergonomic keyboard navigation eliminates mouse friction for experienced counter staff.
- Prevents overcollection and race conditions in multi-counter or automated billing scenarios.
- Complete auditability: transaction commits payment, allocations, receipt number, balance update, and audit log atomically.

**Cons**:
- Requires dedicated UI design and screen space rather than reusing standard modal dialogs.
- Cashier must be informed if a concurrent transaction changes dues while their screen is open.

### Option 2: Generic Modal Payment Dialog triggered from Student Directory

Retain payment collection solely as a modal dialog accessible from the student directory table or student profile.

**Pros**:
- Reuses existing student directory table and avoids building a dedicated counter screen.
- Simpler initial frontend implementation.

**Cons**:
- Severely hinders counter throughput: cashiers must search, click table row, open modal, enter details, close modal, and re-search for every student.
- Inconvenient for counter staff who spend their entire shift collecting fees.
- Poor keyboard ergonomics and lack of quick-fill shortcuts.

### Option 3: Traditional Enterprise Multi-Step Invoice Billing

Model fee collection around formal invoices: cashier creates an invoice, adds invoice line items, finalizes the invoice, creates a separate payment record, and marks the invoice paid.

**Pros**:
- Standard corporate ERP accounting pattern.

**Cons**:
- Unsuitable for coaching center business models: parents think in terms of "paying September tuition fee and model test fee", not paying formal invoice numbers.
- Creates excessive clicks, slow counter turnaround, and confusion for front-desk staff.
- Violates the principle of building specifically for the realities of educational institutes in Bangladesh.

## Rationale

Option 1 is selected because it directly solves the highest-stress problem faced by coaching center staff: managing long queues quickly and accurately without mental calculation errors.

By providing instant FTS5 student lookups with visible due badges, intelligent oldest-first fee allocation, quick-fill keyboard shortcuts, and immediate silent printing, the Fast Cashier Desk empowers counter staff to process transactions in seconds. Incorporating the `expected_balance_poisha` concurrency check ensures that even when multiple counters operate on the LAN or background fee generation runs, financial invariants remain mathematically sound and tamper-proof.
