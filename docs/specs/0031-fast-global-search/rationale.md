# 0031. Fast Global Search (Rationale)

## Context

In an active coaching center, staff frequently receive inquiries from walk-in guardians, students without their ID cards, or telephone callers holding paper receipt slips.
Common scenarios include:
1. A mother calls from home asking about her two children's attendance and fee status, providing only her mobile number.
2. A student arrives without their ID card but knows their school roll or part of their name.
3. A parent brings a faded paper receipt slip asking for duplicate verification, showing only the receipt number.
4. An administrator wants to locate a specific batch or teacher between classes.

If the software requires staff to navigate to separate pages (Students, Batches, Cashier, Staff Directory) and use separate search filters on each page, counter throughput drops dramatically. A unified global search bar accessible from any screen that searches across all entity types simultaneously solves this bottleneck.

---

## Options Considered

### Option 1: Multi-Entity Trigram Indexing with Reverse Guardian Matching (Chosen)

- Provide a single `search.global` RPC endpoint executing targeted queries across Students, Batches, Staff, and Receipts.
- Use SQLite FTS5 with `tokenize = 'trigram'` for substring and Bengali character matching.
- Normalize phone numbers to digits-only for fast prefix and partial matching.
- Link guardian phone numbers to student records directly in the search index, returning the student with guardian attribution.
- Add a composite index on `payments(receipt_number, student_id, paid_at)` in Migration 0016 for sub-millisecond receipt matching.

Pros:
- True sub-100ms response time on reference PC hardware with 10,000+ students and 50,000+ receipts.
- Searching a guardian phone number instantly finds all enrolled siblings.
- Typing a receipt number immediately matches the payment transaction without opening the ledger.
- Full tolerance for Bengali typography and partial substrings.
- Zero client-side waterfall queries.

Cons:
- Multi-entity queries require careful SQLite index design and FTS query sanitization.

### Option 2: Client-Side Federated Query (Call 4 RPCs in Parallel)

The frontend dispatches separate requests to `students.search`, `batches.search`, `staff.search`, and `payments.search`, and joins the results in the browser.

Pros:
- Reuses existing individual entity search endpoints.

Cons:
- Quadruples IPC message serialization overhead over the Tauri bridge.
- Results appear at different times (UI jumping/layout shift).
- Slower on 4 GB RAM reference machines.

### Option 3: Unified Single Search Virtual Table (FTS5 Table Indexing Everything)

Create a single global FTS5 table containing all text tokens from students, parents, staff, batches, and receipts in one giant text corpus.

Pros:
- Single SQL query `SELECT * FROM global_fts WHERE global_fts MATCH ?`.

Cons:
- Storing denormalized rows for 50,000 receipts and financial transactions balloons database file size.
- Complex triggers required on every insert/update/delete across 5 different tables.
- Trigram indexing on short numeric receipt numbers is less efficient than a B-tree index on `receipt_number`.
