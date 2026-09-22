# Verify: Omnichannel outbox and provider adapters (spec 0024)

Steps derived from spec 0024 acceptance criteria.

## UI and Manual Verification

- [ ] Navigate to Settings -> Communication (`/settings/communication`): verify that tabs for SMS, WhatsApp, and Email are present (verifies AC-6, AC-7).
- [ ] In WhatsApp Settings, verify OpenWA status indicator: if disconnected or unpaired, click `[Connect WhatsApp]` and verify that the pairing QR code displays live in the modal with automatic refresh (verifies AC-6, AC-8).
- [ ] Scan the QR code using WhatsApp on a mobile phone: verify that status transitions to `Connected` and persists across app restarts (verifies AC-6, AC-8).
- [ ] In Email Settings, enter SMTP host, port 587, username, and password: click `[Send Test Email]`. Verify that an email arrives in the recipient inbox with clean HTML formatting (verifies AC-6, AC-8).
- [ ] Enter an API key in SMS config and save: verify that upon refresh the API key is masked as write only (`••••••••1234`) and not visible in inspect logs (verifies AC-7).
- [ ] Disable network connectivity: record a student tuition fee payment. Verify that the payment completes with zero latency and outbox records appear for enabled channels with status `queued` (verifies AC-1, AC-5).
- [ ] Restore network connectivity: trigger the outbox poller. Verify that queued messages transition from `queued` -> `sending` -> `sent` with single entries in `delivery_attempts` and are sent exactly once (verifies AC-2, AC-5).
- [ ] Dispatch a batch of WhatsApp messages: verify that consecutive sends pause 3 to 6 seconds between each recipient to avoid spam flags (verifies AC-5).
- [ ] Configure Mock Adapter to simulate a request timeout: trigger outbox delivery. Verify that the message transitions to status `uncertain` with an attempt outcome of `timeout` (verifies AC-2, AC-3).
- [ ] Run the background worker loop multiple times: verify that the `uncertain` message is never automatically retried (verifies AC-3).
- [ ] In the Outbox Monitor view (`/communication/outbox`), locate the `uncertain` message and click `[Resend Message]`: verify that the message is manually requeued and successfully dispatched (verifies AC-3).
- [ ] Simulate quiet hours (set local clock between 21:00 and 08:00 Asia/Dhaka): enqueue a standard non urgent outbox message. Verify that the worker keeps the message `queued` and does not dispatch it until quiet hours close (verifies AC-4).
- [ ] Idempotency verification: trigger a command with an existing `dedupe_key`. Verify that the database unique constraint blocks duplicate row creation and returns cleanly (verifies AC-1).

## Commands

- [ ] `cargo test -p coachify_core --test comms_outbox_and_adapters` (verifies transactional enqueueing, dedupe key uniqueness, state transitions, mock provider failure modes, timeout marking as uncertain, quiet hours scheduling, OpenWA endpoints, SMTP mailer, and secret redaction, satisfies AC-1 through AC-8).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contract freshness in `ui/src/gen/rpc.ts` for all `comms.*` and `whatsapp.*` RPC payloads and responses).
- [ ] `cargo run -p xtask check-migrations` (verifies Migration 0014 schema validity and released migration immutability).
- [ ] `pnpm --dir ui test` (verifies OutboxList status badges, provider config forms, masked secret inputs, WhatsApp QR modal, and test message dispatch).

## Acceptance Criteria Coverage

- AC-1 (Same Transaction Multi Channel Outbox Insertion): verified by core ledger and attendance transaction integration tests and dedupe key constraint checks.
- AC-2 (Outbox Lifecycle and State Machine): verified by state transition unit tests covering queued, sending, sent, uncertain, failed, and cancelled.
- AC-3 (Strict Timeout and Uncertain Handling): verified by mock timeout tests asserting status `uncertain` and zero automatic reschedules.
- AC-4 (Timezone Aware Quiet Hours and Scheduled Delivery): verified by scheduler time evaluation tests asserting message deferral during quiet hours.
- AC-5 (Background Poller and Manual Trigger): verified by poller loop tests and rate limiting delay tests.
- AC-6 (Provider Adapter Abstraction): verified by unit tests for Mock, GenericHttp, Greenweb, BulkSmsBd, OpenWA, and SMTP adapters.
- AC-7 (Secret Protection and Write Only Credentials): verified by DPAPI serialization tests, secret redaction checks, and log scanner tests.
- AC-8 (Test Message Dispatch and WhatsApp Pairing Flow): verified by test dispatch RPC tests, QR payload checks, and settings persistence tests.
