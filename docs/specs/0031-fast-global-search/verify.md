# Verify: Fast Global Search (spec 0031)

Steps derived from spec 0031 acceptance criteria.

## UI and Manual Verification

- [ ] Focus the global search input in the top bar or press `Ctrl+K`.
- [ ] Type a student name in Latin script (e.g. `"Rahim"`): verify that student matches appear with student code, phone, and batch tag (verifies AC-1).
- [ ] Type a student name in Bengali script (e.g. `"রহিম"` or `"ফার"`): verify that trigram matching successfully locates students with Bengali names (verifies AC-1, AC-4).
- [ ] Type a guardian's phone number (e.g. `"01711234567"` or `"171123"`): verify that all enrolled students belonging to this guardian are returned with a badge `Guardian: <Name>` (verifies AC-2).
- [ ] Type a receipt number (e.g. `"2026-0001"`): verify that the payment receipt is found with student name, collected amount in ৳, and payment method (verifies AC-3).
- [ ] Click or press `Enter` on the matched receipt: verify that the receipt reprint preview dialog opens immediately (verifies AC-3).
- [ ] Type a batch name (e.g. `"HSC Physics"`): verify that matching batch records appear with program name and student enrollment count (verifies AC-1).
- [ ] Type a single character: verify that search does not fire and empty state is cleanly maintained (verifies AC-6).
- [ ] Type characters with special FTS syntax (e.g. `"` or `*` or `-`): verify that input is sanitized and no SQLite FTS5 syntax errors occur (verifies AC-6).
- [ ] Benchmark latency on the reference dataset: verify that `duration_ms` returned in `GlobalSearchResponse` is under 100 ms (verifies AC-5).

## Commands

- [ ] `cargo test -p coachify_core --test global_search_performance` (verifies sub-100ms multi-entity search, guardian phone reverse lookup, receipt index lookup, and Bengali trigram tolerance, satisfies AC-1 through AC-6).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contracts for `search.global` payload and response types).
- [ ] `pnpm --dir ui test` (verifies top-bar search debouncing, categorized result list rendering, and deep link navigation).

## Acceptance Criteria Coverage

- AC-1 (Unified Search RPC): verified by `test_global_search_multi_entity_response`.
- AC-2 (Guardian Phone Reverse Lookup): verified by `test_guardian_phone_finds_all_siblings`.
- AC-3 (Receipt Number Fast Matching): verified by `test_receipt_number_indexed_lookup`.
- AC-4 (Bengali Substring & Trigram): verified by `test_bengali_unicode_substring_search`.
- AC-5 (Sub-100ms Performance): verified by benchmark test asserting `duration_ms < 100`.
- AC-6 (Sanitization & Error Handling): verified by `test_fts_special_char_sanitization`.
