# Spec 0031 (Fast Global Search)

> Feature 33 · Slice 6 (Reports & Speed)

## Goal

Provide a lightning-fast, unified global search engine across Students, Guardians, Batches, Staff, and Payment Receipts, returning categorized results in under 100 ms on reference hardware with 10,000+ records, supporting Bengali phonetic variations and partial substrings, and enabling 1-click deep navigation directly to student profiles, cashier counters, or receipt reprints.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **Multi-Entity Unified Endpoint** | Single RPC `search.global` returning grouped entity hits | Prevents client-side waterfall requests; backend executes targeted index queries in a single database read transaction |
| **Trigram FTS5 Tokenizer** | SQLite FTS5 `tokenize = 'trigram'` for student names (Latin and Bengali) and codes | Trigram tokenization allows true substring matching across complex Bengali conjuncts without word-boundary restrictions |
| **Phone Number Normalization** | Digits-only phone normalization stripping `+88`, `88`, spaces, and dashes | Guarantees that typing any fragment of a phone number (e.g. `01711` or `1711`) matches both student and guardian records |
| **Guardian Phone Reverse Lookup** | Match guardian phone in `student_search_index` and return child with guardian badge | Coaching staff frequently search by guardian phone when parents call; showing the student immediately solves the lookup problem |
| **Receipt Composite Index** | Composite index on `payments(receipt_number, student_id, paid_at)` in Migration 0016 | Enables instant sub-millisecond lookup by receipt number prefix (e.g. `2026-0042` or `RCP-2026-0042`) |
| **Sub-100ms Budget** | Hard query latency budget of 100 ms on 10,000 students and 50,000 receipts | Fulfills Coachify's core differentiator of operational speed on ordinary coaching center PCs |

## Requirements

**User stories**:
- As a front-desk receptionist, I want to type a guardian's phone number and immediately see all children registered under that guardian, so that I can answer parent queries without asking for student IDs.
- As a cashier, I want to type a receipt number from a paper slip (e.g. `2026-0123`) and have the payment open immediately, so that I can process refunds or reprint receipts.
- As an academic supervisor, I want to search for a batch by name (e.g. "HSC Physics") or code and jump directly to its class roster.
- As an institute owner, I want to search in Bengali (e.g. "রহিম" or "ফারহানা") and get accurate results even with partial words or minor spelling variations.

## Acceptance Criteria

### AC-1: Unified `search.global` RPC Endpoint
- RPC `search.global` accepts `{ query: String, limit: Option<u32> }`.
- Returns `GlobalSearchResponse`:
  - `students`: Vec of student hits (`id`, `student_code`, `name`, `name_bn`, `phone_norm`, `batch_name`, `guardian_names`, `guardian_phones`).
  - `batches`: Vec of batch hits (`id`, `name`, `code`, `program_name`, `student_count`).
  - `staff`: Vec of staff hits (`id`, `staff_code`, `name`, `phone_norm`, `designation`).
  - `receipts`: Vec of payment hits (`payment_id`, `receipt_number`, `student_id`, `student_name`, `student_code`, `amount`, `method`, `paid_at`).
  - `total_hits`: Total number of matching records.
  - `duration_ms`: Server execution time in milliseconds.

### AC-2: Guardian Phone Reverse Lookup
- When a numeric string (6+ digits) is queried:
  - Searches `guardian_phones` in `student_search_index`.
  - Returns student record with primary badge: `Guardian: <Name> (<Phone>)`.
  - Supports multiple children belonging to the same guardian phone.

### AC-3: Receipt Number Fast Matching
- When a receipt number pattern is searched (digits or prefixed e.g. `2026-`):
  - Uses `idx_payments_search_composite` to locate matching `payments` rows.
  - Returns receipt number, student name, collected amount in ৳ (converted from poisha), payment method, and formatted date.
  - Selecting the receipt opens the reprint preview modal directly.

### AC-4: Bengali Substring & Trigram Tolerance
- Searches containing Bengali Unicode characters match against `name_bn` and Bengali guardian fields via trigram FTS5 index.
- Handles partial substring matching (e.g. `"রাহি"` matches `"রাহিম"`).

### AC-5: Sub-100ms Query Latency
- On a database loaded with 10,000 students, 350+ batches, and 50,000 payments:
  - The `search.global` query executes in under 100 ms on reference hardware.
  - UI debounces input at 150 ms to avoid unnecessary keystroke queries.

### AC-6: Structured Error & Empty Handling
- If query is less than 2 characters, returns empty result set without executing database queries.
- Sanitizes FTS5 special characters (quotes, asterisks, hyphens) to prevent SQLite FTS5 query syntax errors.
