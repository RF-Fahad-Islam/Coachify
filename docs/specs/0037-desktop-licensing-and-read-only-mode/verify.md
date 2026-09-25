# 0037. Desktop Licensing and Read Only Mode: Verification Plan

> Verification suite and criteria for Spec 0037 (Desktop Licensing and Read Only Mode)

## Automated test suite

Run the dedicated licensing integration suite:

```bash
cargo test -p coachify_core --test licensing_integration
```

Verify contract freshness and run UI tests:

```bash
cargo run -p xtask check-types
cargo build --workspace
pnpm --dir ui test
```

## Test scenarios & criteria matrix

| Test scenario | Acceptance criteria | Verification method |
|---|---|---|
| Authentic Ed25519 signature verification | AC-1, AC-9 | Verifies that a valid `.coachifylicense` payload signed with vendor test private key validates and activates cleanly |
| Tampered payload rejection | AC-1 | Modifies a single character in the license payload, confirming rejection with error code `license_signature_invalid` |
| Installation hardware binding | AC-2 | Imports a valid license signed for a different installation ID, asserting refusal with `license_installation_mismatch` |
| Monotonic serial check | AC-3 | Imports a license with an older serial number than the active record, asserting refusal with `license_serial_downgrade` |
| Automatic 14 day trial generation | AC-4 | Executes setup wizard bootstrap, verifying active 14 day evaluation trial in `licenses` table |
| Grace period warnings | AC-5 | Sets virtual date past `paid_through` but within 14 days, verifying operational access and warning indicators |
| Dispatcher read only mode enforcement | AC-6 | Sets date past `valid_until`, confirming mutations fail with `license_expired_read_only` while reads succeed |
| Moved machine transitional grace | AC-7 | Restores database on a simulated new host device ID, verifying 14 day full access transitional status |
| Offline request code generation | AC-8 | Calls `license.get_request_code`, asserting 30 character grouped code and valid QR payload |
| Monotonic clock rollback guard | AC-10 | Advances audit log watermark then rolls system clock backward, confirming mutation rejection with `clock_out_of_sync` |
| SQLite trigger immutability defense | AC-11 | Attempts direct SQL deletion from `licenses` table, confirming operation is aborted by database trigger |

## Manual verification checklist

1. Launch Coachify in development mode using `pnpm dev`.
2. Navigate to Settings and select the License card. Confirm that current institution name, plan tier, and days remaining are shown accurately.
3. Click "Show Request Code" and verify that both the formatted 30 character text and the scannable QR code display cleanly.
4. Test importing an invalid or corrupted license file. Confirm that the interface displays a clear error explanation without modifying the current license.
5. In development test tools, advance the simulation clock beyond the 14 day grace period. Confirm that a prominent read only banner appears at the top of the application.
6. In read only mode, verify that student searches, marks viewing, report card printing, and database backup continue working, while attempting to record a fee payment displays the read only explanation.
7. Import a valid renewed license file and confirm that write capabilities are immediately restored and the read only banner dismisses.
