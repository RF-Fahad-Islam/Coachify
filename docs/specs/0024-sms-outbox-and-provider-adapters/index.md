# Spec 0024 (Omnichannel Outbox and Provider Adapters: SMS, WhatsApp, Email)

> Feature 26 · Slice 4 (Academics and Messaging)

## Goal

Provide Coachify with an offline first, durable omnichannel outbox written in the exact same database transaction as domain business events, supporting SMS, WhatsApp (via OpenWA companion daemon), and Email (via SMTP), executed by a desktop background poller with quiet hours enforcement, anti ban throttling, fail safe handling of network timeouts (uncertain status with zero automatic duplicate resends), and pluggable adapters.

## Key Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Outbox Pattern | Append only `outbox` table in SQLite committed inside business transaction | Guarantees an outbox message is queued if and only if the underlying payment or attendance action commits |
| Supported Channels | `sms`, `whatsapp`, `email` | Meets coaching center needs: instant cellular SMS, rich WhatsApp messages, and official email receipts and statements |
| WhatsApp Integration | OpenWA child process bundled with desktop app | Runs OpenWA background process with session data saved in AppData so pairing persists across app restarts |
| Port Conflict Guard | Port 8000 default with automatic fallback scanning across ports 8001 to 8010 | Automatically resolves local port conflicts without crashing or requiring manual terminal troubleshooting |
| WhatsApp Anti Ban Pacing | Randomized 3 to 6 second pause between consecutive dispatches | Simulates natural human cadence and protects coaching center phone numbers from automated spam bans |
| Email Delivery Model | Synchronous SMTP with multipart MIME (HTML plus plain text fallback) | Delivers branded responsive receipts directly in the email body with zero external PDF dependencies in this slice |
| Worker Lease Model | 30 second background poller plus manual queue flush button | Dispatches scheduled messages reliably while letting operators click Send All Pending for instant delivery |
| Deduplication Key | `dedupe_key TEXT UNIQUE` on the outbox table | Guarantees idempotent commands never enqueue duplicate messages on any channel |
| State Machine | `queued`, `sending`, `sent`, `uncertain`, `failed`, `cancelled` | Clearly communicates delivery state across all channels without ambiguous waiting states |
| Timeout Policy | Requests timing out after dispatch are marked `uncertain` with zero automatic retry | Prevents duplicate billing and recipient spam because local gateways lack idempotency keys |
| Quiet Hours | 21:00 (9:00 PM) to 08:00 (8:00 AM) in institution timezone (UTC+6) | Prevents waking guardians with non urgent alerts late at night |
| Secrets Storage | Windows DPAPI machine scope key storage | API keys, passwords, and tokens are write only and never leak in UI, logs, or diagnostic bundles |

## Requirements

**User stories**:
- As an institute accountant collecting tuition fees, I want receipt alerts to queue in the exact same transaction as the payment, so that a payment is never recorded without its confirmation message.
- As an institute owner, I want to send WhatsApp alerts through our institute phone number with zero per message fees, pairing easily by scanning a QR code inside settings.
- As an institute administrator, I want official tuition receipts emailed to parents with clean HTML formatting showing receipt numbers and remaining balances.
- As a center manager, I want messages held safely during late night quiet hours, releasing automatically at 8:00 AM the next morning.
- As an operator experiencing internet drops, I want uncertain dispatches clearly highlighted without automatic duplicate retries, protecting our account from double billing.

## Acceptance Criteria

### AC-1: Same Transaction Multi Channel Outbox Insertion
- Any domain command that initiates messaging (such as payment collection, student admission, attendance absence) writes corresponding rows into the `outbox` table within the **same SQLite transaction**.
- Column `channel` supports `'sms'`, `'whatsapp'`, and `'email'`.
- Column `subject` stores subject lines for emails (NULL for SMS and WhatsApp).
- If the domain transaction fails or rolls back, no rows are written to `outbox`.
- Each record has a required `dedupe_key` with a unique constraint (for example `payment:<id>:receipt_sms`, `payment:<id>:receipt_whatsapp`, `payment:<id>:receipt_email`).
- Attempting to re insert with the same `dedupe_key` is safely ignored or handled idempotently without error.

### AC-2: Outbox Lifecycle and State Machine
- Supported statuses:
  - `queued`: Ready to be sent when scheduled time arrives and outside quiet hours.
  - `sending`: Claimed by the sender worker with an active lease.
  - `sent`: Successfully accepted by provider or mail server (HTTP 200 or SMTP 250 OK).
  - `uncertain`: Request was transmitted over the wire, but connection timed out before a definitive response was received.
  - `failed`: Definitive failure (invalid recipient address or number, insufficient credits, rejected authentication, or max retry attempts reached).
  - `cancelled`: Cancelled prior to dispatch (for example when an absence mark was corrected during the delay window).

### AC-3: Strict Timeout and Uncertain Handling
- HTTP client and SMTP socket dispatch with strict timeouts (10 seconds).
- If a request times out, the outcome is logged in `delivery_attempts` with `outcome = 'timeout'`.
- The outbox record transitions to `status = 'uncertain'`.
- **Zero Automatic Retry**: The system strictly refrains from automatically rescheduling `uncertain` messages.
- Staff with `comms.manage` permission can inspect uncertain messages in the outbox UI and manually trigger `comms.resend_message`.

### AC-4: Timezone Aware Quiet Hours and Scheduled Delivery
- All outbox timestamps (`scheduled_at`, `created_at`, `sent_at`, `attempted_at`) are stored in **strict UTC** (`YYYY-MM-DDTHH:MM:SSZ`).
- Default quiet hours window: `21:00` (9:00 PM) to `08:00` (8:00 AM) in the institution configured timezone (default `Asia/Dhaka`, UTC+6).
- The worker converts UTC time to institute local time to evaluate the quiet hours boundary:
  - If current local time is within quiet hours, non urgent SMS and WhatsApp messages remain in `status = 'queued'`.
  - `scheduled_at` is automatically evaluated against the 08:00 AM morning boundary.
- Urgent messages and transactional emails bypass quiet hours when `is_urgent = 1`.

### AC-5: Background Poller and Manual Trigger
- The desktop app runs a periodic 30 second background poller that leases eligible queued messages.
- Operators can click `[Send All Pending]` on the Outbox Monitor view to trigger immediate dispatch.
- When dispatching WhatsApp batches, the poller enforces a randomized 3 to 6 second pause between consecutive messages to prevent phone bans.

### AC-6: Provider Adapter Abstraction
- Unified trait `ChannelAdapter`:
  - **SMS Adapters**: `MockSmsAdapter`, `GenericHttpSmsAdapter`, `GreenwebAdapter`, `BulkSmsBdAdapter`.
  - **WhatsApp Adapter (`OpenWaAdapter`)**: Connects to the local OpenWA REST endpoint (default port 8000, scanning through 8010 on conflict), formats recipient to `<digits>@c.us`, dispatches `POST /sendText`, checks connection state via `GET /getConnectionState`, and fetches QR code via `GET /getQrCode`.
  - **Email Adapter (`SmtpEmailAdapter`)**: Synchronous SMTP client supporting port 587 (STARTTLS), port 465 (TLS), and custom mail servers, generating multipart MIME with responsive HTML receipt table and plain text fallback.
- Adapters normalize phone numbers to international standard (`8801XXXXXXXXX`).

### AC-7: Secret Protection and Write Only Credentials
- Provider configurations are stored in `provider_configs` unique on `(channel, adapter)`.
- Passwords, API keys, and auth tokens are encrypted using Windows DPAPI (machine scope) and referenced via `secret_ref`.
- RPC `comms.get_provider_config` redacts secrets (`secret_is_set: true`, masked value `•••••••1234`).
- Raw secrets never appear in log files, database exports, or diagnostic support bundles.

### AC-8: Test Message Dispatch and WhatsApp Pairing Flow
- RPC `comms.send_test_message` accepts a channel (`sms`, `whatsapp`, `email`), destination address, and test payload, dispatches immediately, and returns the provider response.
- RPC `whatsapp.get_status` queries OpenWA state (`connected`, `qr_required`, `disconnected`, `unreachable`).
- RPC `whatsapp.get_qr` retrieves the base64 pairing QR code from OpenWA for display in the settings modal.

### AC-9: Orphaned Message Lifecycle Triggers
- Automatic database triggers cancel pending `queued` outbox entries when parent entities change:
  - When a payment is voided (`UPDATE payments SET status = 'voided'`), trigger `trg_payments_cancel_outbox` cancels any pending receipt messages on all channels (`sms`, `whatsapp`, `email`).
  - When an attendance session is deleted, trigger `trg_attendance_sessions_cancel_outbox` cancels any pending absence messages linked to that session.

### AC-10: Structured Domain Errors
- Maps to typed `RpcError` with explicit codes:
  - `provider_not_configured`: No active provider configured for requested channel.
  - `provider_auth_failed`: Gateway or mail server rejected credentials.
  - `openwa_server_unreachable`: OpenWA companion daemon is offline or not responding.
  - `phone_normalization_failed`: Phone number cannot be formatted to valid Bangladeshi mobile.
  - `invalid_email_address`: Destination email address format is invalid.
  - `quiet_hours_active`: Message held in queue due to active quiet hours window.
  - `comms_uncertain`: Dispatch timed out; message marked uncertain and held from automated retry.
