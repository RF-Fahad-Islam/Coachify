# 0032. Operational Integrity: Verification Plan

> Verification suite and criteria for Spec 0032 (Slice 8 Operational Integrity Umbrella)

## Automated test suite

Run the dedicated operational integrity integration suite:

```bash
cargo test -p coachify_core --test operational_integrity_integration
```

Verify contract freshness and compile all workspace packages:

```bash
cargo run -p xtask check-types
cargo build --workspace
pnpm --dir ui test
```

## Test scenarios & criteria matrix

| Test scenario | Acceptance criteria | Verification method |
|---|---|---|
| Receipt verification offline | AC-1, AC-2 | Signs payment receipt with HMAC SHA256 key, verifies valid status, modifies amount string and asserts signature mismatch error |
| Duplicate payment warning | AC-3, AC-4 | Submits duplicate reference within 24 hours, verifies warning returned, submits with override reason and confirms persistence in database and audit log |
| Guardian contact hygiene | AC-5, AC-6 | Inserts students with invalid phone formats and shared numbers, runs contact health check, executes bulk normalization and asserts updated values |
| Outbox restart durability | AC-7, AC-8 | Queues messages while offline, restarts core engine, restores network mock, verifies queue flushes without duplicating messages |
| Resilience center status | AC-9 | Polls `integrity.get_resilience_status`, verifies network status, outbox depth, backup recency, and DB integrity within 5 seconds |
| Disaster recovery score | AC-10 | Calculates composite score across backup age, external copy presence, drill pass rate, and DB integrity, verifying thresholds and alert generation |
| Backup verification drill | AC-11 | Restores snapshot into in memory SQLite instance, checks PRAGMA integrity, validates foreign keys, reconciles balance totals without touching production DB |
| PC migration round trip | AC-12, AC-13 | Exports encrypted migration package, validates SHA256 checksum, imports into clean environment, rewraps DPAPI credentials and checks data fidelity |
| Startup integrity check | AC-14 | Executes full integrity check suite on database with 10,000 records, measures execution time under 3000 ms, verifies corruption detection locks DB in read only mode |

## Manual verification checklist

1. Start desktop application with `pnpm dev`.
2. Navigate to Cashier desk and issue a receipt. Observe the printed thermal voucher displaying the 12 character verification code and QR code.
3. Open the receipt verification modal, scan or type the code, and confirm that receipt details and validity render instantly without network access.
4. Attempt to collect another payment with the identical transaction reference within 24 hours. Confirm the amber duplicate warning dialog appears with the reason prompt.
5. In the top application bar, observe the Offline Resilience Center status indicator. Disconnect network cable or Wi-Fi, and confirm the indicator updates to offline within 5 seconds.
6. Trigger a manual backup verification drill from settings, and verify the drill log records a successful test in the drill history table.
