# 0028. Reports Center & Exports (Rationale)

## Context

In coaching center management, reports represent the core operational feedback loop for institution owners and branch managers. Daily questions include:
- "How much cash did Counter 1 collect today versus bKash digital transfers?"
- "Which students have unpaid dues older than 60 days so our receptionist can call their guardians?"
- "Are any students attending less than 75% of classes this month?"
- "Did Counter 2's cash drawer balance match counted money at the end of the shift?"

Traditional coaching software systems fail in three specific ways:
1. **Slow in-memory report generation**: Fetching unaggregated rows over client-server or ORM abstractions freezes the desktop application when datasets reach thousands of student records.
2. **Broken Excel exports for non-Latin text**: When exporting reports to CSV on Windows, standard UTF-8 text files open with garbled symbols (mojibake) in Microsoft Excel unless a UTF-8 Byte Order Mark (BOM) is explicitly prepended.
3. **Disconnected financial reconciliation**: Collections, refunds, and daily cash drawer closes are often tracked in separate silos, making cashier cash shrinkage difficult to detect.

---

## Options Considered

### Option 1: Indexed SQL Aggregations, UTF-8 BOM CSV, and Integrated Reconciliation (Chosen)

- Execute report summaries using direct SQLite queries leveraging covering indexes on `payments`, `charges`, `attendance_marks`, and `cash_days`.
- Prepend `\u{FEFF}` (UTF-8 BOM) to generated CSV text.
- Standardize aging buckets into 0–30, 31–60, 61–90, 90+ days.
- Include chronic absenteeism flag (< 75%) in monthly attendance registers.
- Provide clean CSS `@media print` styling alongside CSV export.

Pros:
- Guaranteed sub-2-second generation on 10,000+ student datasets.
- 100% Excel compatibility on Windows out of the box with flawless Bengali names and addresses.
- Direct mathematical integrity: report totals match the immutable ledger records to the exact poisha.
- Lightweight: requires no heavy server-side PDF generator dependencies.

Cons:
- Report layout logic is maintained in both SQL aggregation models and React presentation views.

### Option 2: Client-Side Aggregation (Fetch All Rows to UI)

Fetch raw payments, students, and marks to the React UI and perform grouping, sum, and aging calculations in TypeScript memory.

Pros:
- Simpler SQL endpoints.

Cons:
- Transferring 50,000 payment records over the Tauri IPC bridge causes significant latency and UI freezing (> 5s).
- Fails the reference PC performance benchmark on 4 GB RAM machines.

### Option 3: External PDF Library Binary Generation

Use native headless Chromium or wkhtmltopdf to generate binary PDF files on the backend.

Pros:
- Generates pixel-frozen PDF files on disk.

Cons:
- Heavyweight dependencies, increased installer size, and complex Bengali font shaping issues in third-party PDF engines.
- Desktop WebView2 print API and browser print stylesheets already provide high-quality printing with native Windows font shaping.
