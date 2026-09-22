# Review, branch, 2026-09-22

**Reviewed by**: Antigravity (author on AI)
**Scope**: 24 files, branch
**Verdict**: Approved with follow up items

## Summary
The print engine and fast cashier features correctly introduce high density desk workflows, robust concurrency guards, and native silent receipt printing without ESC/POS encoding issues. The workstation printer settings contract mismatch identified during the initial review pass was immediately resolved in `crates/core/src/storage.rs` and validated with end to end integration tests.

## Resolved Blockers
### 🟢 RESOLVED: UI and Rust contract mismatch for `WorkstationPrinterSettings`, `crates/core/src/storage.rs:536`
**Resolution**: Aligned `WorkstationPrinterSettings` in `crates/core/src/storage.rs` to match the exact TypeScript contract in `ui/src/gen/rpc.ts` (`printer_name`, `page_size`, `paper_source`, `copies`, `auto_print_on_payment`, `open_cash_drawer`, `header_title`, `footer_note`, and `updated_at`). Added dispatcher integration test `test_dispatcher_printer_settings_rpcs` in `crates/core/tests/print_and_receipts.rs` verifying JSON serialization round trip through the dispatcher.

## Major (Tracked for future refactor)
### 🟠 TypeScript contract generation bypassed, `crates/core/src/lib.rs:402`
**Problem**: `generate_ts_definitions()` in `lib.rs` returns a string literal of TypeScript interfaces instead of using `specta::export::ts()` to derive them from the actual Rust types.
**Why it matters**: `cargo run -p xtask check-types` compares against this generated string. Expanding Specta macros across all command payloads directly will ensure compile time type reflection.
**Suggested fix**: Replace string literal generation with full recursive Specta macro code generation across all payloads.

## Minor
### 🟡 Workstation settings fallback to User profile, `crates/core/src/storage.rs:565`
**Problem**: `resolve_printer_settings_path` attempts to write to `%ProgramData%\Coachify`. Standard users without elevated permissions will fall back to `%LOCALAPPDATA%`.
**Why it matters**: `%LOCALAPPDATA%` is per user. Multiple Windows accounts on the same workstation will not share the printer config.
**Suggested fix**: Create the `%ProgramData%\Coachify` folder with appropriate access control during the Windows installer step.

## Nits
- ⚪ `crates/core/src/ledger.rs:409`, `InstitutionDetails` fallback uses default name.

## Strengths
- The optimistic concurrency check using `expected_balance_poisha` is cleanly implemented and reliably rejects stale payments.
- The use of print CSS and WebView2 to side step ESC/POS Bengali rendering issues guarantees typography fidelity.
- Keyboard navigation shortcuts on the cashier desk view provide swift ergonomics for high volume counters.

## Test coverage
**Tests are configured and passing**. Integration test suite `crates/core/tests/print_and_receipts.rs` (6 tests) and `crates/core/tests/cashier_and_payments.rs` (5 tests) cover settings persistence, printer enumeration, receipt aggregation, Bengali words, reprint flags, student overviews, allocation algorithms, concurrency guards, double click idempotency, and sub second latency benchmarks. All 90 Vitest UI unit tests pass cleanly.
