# Review, feat/data-model-database-conventions, 2026-09-20

**Reviewed by**: Gemini Pro (author on Gemini Flash)
**Scope**: 12 files, branch vs base
**Verdict**: Changes requested

## Summary
You did a solid job building the database foundation. The schema is well organized and the transaction wrappers are clean. We have a few important issues to fix before merging. The main problems are missing triggers that enforce immutability and a severe performance bottleneck when opening database connections.

## Major
### 🟠 Missing immutability triggers, `migrations/0002_foundation_schema.sql:462`
**Problem**: The spec states that adjustments and refunds are permanently protected by triggers. However these tables have no such triggers. Also the charges table has a delete trigger but lacks an update trigger.
**Why it matters**: Without these triggers the database allows silent modifications to financial history. This breaks the core ledger immutability guarantee.
**Suggested fix**: Add `BEFORE UPDATE` and `BEFORE DELETE` triggers for adjustments and refunds. Add a `BEFORE UPDATE` trigger for charges to prevent altering amounts after generation.

### 🟠 Heavy integrity check on every connection, `crates/core/src/db.rs:450`
**Problem**: You call `verify_integrity` inside `configure_connection`. This executes `PRAGMA quick_check` every time a new connection opens. 
**Why it matters**: The quick check command scans the entire database structure. As the database grows this will cause massive delays and block the application every time a ping or new query runs.
**Suggested fix**: Remove `verify_integrity` from the standard connection setup. Run it once during application startup or in a dedicated background task.

## Minor
### 🟡 Unnecessary illegal update, `crates/core/src/db.rs:562`
**Problem**: When `verify_clock` detects backwards time drift it attempts to update the database with the invalid time before returning an error.
**Why it matters**: The database trigger will always abort this update. The extra query wastes time and adds confusion to the code.
**Suggested fix**: Remove the `tx.execute` call inside the `diff_seconds < 0` block and just return the error directly.

## Strengths
- The transaction wrapper in Rust is clean and handles immediate transactions perfectly.
- You wrote strong tests for the foreign key constraints and unique indexes.
- The sequence allocation logic is robust and prevents gaps.

## Test coverage
The test coverage is solid for most of the core behaviors. You successfully covered clock drift, sequence allocation, and basic trigger rules. You missed tests for adjustment and refund immutability because the triggers themselves are missing. Once you add those triggers you should add matching tests to verify them.
