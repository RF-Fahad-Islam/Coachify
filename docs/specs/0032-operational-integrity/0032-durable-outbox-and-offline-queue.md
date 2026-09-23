# 0032-durable-outbox-and-offline-queue. Durable Outbox & Offline Message Queue

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Features 43 and 44

**Date**: 2026-09-23

## Summary

This specification establishes a durable outbox state machine and offline message queue. All outgoing SMS, WhatsApp, and email messages are persisted to the local SQLite database inside the same transaction as the triggering business event (such as payment receipts, attendance alerts, or published exam marks). When the computer is disconnected from the internet, messages remain safely queued on disk across restarts and system crashes. A background connectivity monitor watches network status using exponential backoff and automatically flushes pending queues upon verified reconnection with zero message loss and strict deduplication.

## Requirements

**User stories**:
* As a teacher publishing exam results during an internet outage, I want the report card messages to queue safely in the local database, so that I can proceed with my work knowing the system will send them once internet returns.
* As an administrator, I want full visibility into every message lifecycle state (queued, sending, sent, failed), so that I can diagnose and retry failed messages individually or in bulk.
* As an owner, I want absolute certainty that no duplicate SMS messages are sent to parents after power returns or after reconnecting to Wi-Fi.

**Acceptance criteria**:
* **AC-1**: Messages written to the `outbox` table during offline periods survive application restarts, power cuts, and crashes without loss or corruption.
* **AC-2**: Background worker periodically polls network health and flushes pending outbox messages in chronological batches once stable connectivity is confirmed.
* **AC-3**: Outbox records maintain strict deduplication keys (`dedupe_key UNIQUE`), preventing duplicate transmissions even if reconnect events trigger rapid repeated flushes.
* **AC-4**: RPC `integrity.get_outbox_lifecycle` returns aggregated outbox metrics and filtered message lists supporting status filters (queued, sending, sent, failed) and channel filters (sms, whatsapp, email).
* **AC-5**: RPC `integrity.retry_outbox_messages` allows staff to reset selected failed messages back to queued status for immediate re delivery.
* **AC-6**: RPC `integrity.flush_offline_queue` provides an immediate manual trigger for cashiers or administrators to force a queue flush when connectivity returns.

## Decision

**Chosen option**: Local Immediate Outbox Transaction with Idempotent Background Reconnect Worker

Outbox records are created atomically alongside business domain actions using SQLite immediate transactions. A dedicated background thread in the core manages HTTP delivery attempts, tracking attempt counts, HTTP status codes, and provider message IDs in `delivery_attempts`. Offline detection uses a lightweight DNS or socket ping before dispatching gateway requests.

## Rationale

Decoupling message creation from actual HTTP transmission ensures that front desk counter operations and classroom workflows remain completely unblocked by network slowdowns or gateway timeouts. Storing the outbox in the local SQLite database guarantees zero message loss through power cuts and machine restarts.

## Feature design

**Data model sketch**:
* Reuses existing `outbox` and `delivery_attempts` tables from Migration 0014.
* Outbox statuses: `queued`, `sending`, `sent`, `failed`, `uncertain`, `cancelled`.
* Delivery attempts log: `id`, `outbox_id`, `attempted_at`, `outcome`, `http_status`, `duration_ms`.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.get_outbox_lifecycle` | Read | status: Option<String>, channel: Option<String> | OutboxLifecycleSummary | Authenticated | 500 query error |
| `integrity.retry_outbox_messages` | Mutation | message_ids: Vec<String> | RetryResult | Authenticated | 403 forbidden |
| `integrity.flush_offline_queue` | Mutation | none | FlushResult | Authenticated | 503 network unavailable |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Queue summary | counts by status and channel | SQL GROUP BY on outbox table |
| Flush queue | sent count and failed count | Iteration over pending outbox rows with network dispatch |
| Retry message | reset status | UPDATE outbox SET status = 'queued', attempts = 0 |

**Key invariants**:
* A message cannot transition directly from failed to sent without an intermediate queued attempt.
* Deduplication keys are strictly unique across all outbox records.
* In flight messages marked as sending that fail to complete due to process termination are reset to queued upon startup recovery.

**Security model**:
* All counter roles (Cashier, Receptionist, Admin, Owner) can view the outbox status.
* Admin and Owner roles can trigger manual flushes and bulk retries.

**Critical test scenarios**:
* Offline persistence: Writes 10 messages with mock offline mode, closes database, reopens database, asserts all 10 messages remain queued, verifies AC-1.
* Reconnect auto flush: Queues 3 messages, switches mock network to online, triggers flush worker, asserts all 3 messages marked sent and delivery attempts recorded, verifies AC-2 and AC-3.
* Idempotent retry: Retries failed messages, asserts status transitions back to queued with reset attempt counter, verifies AC-5.

## Build plan

1. Implement outbox startup recovery routine resetting orphaned sending rows back to queued in `crates/core/src/integrity.rs`, satisfies **AC-1**.
2. Implement network connectivity check and batch outbox dispatcher with exponential backoff, satisfies **AC-2**, **AC-3**.
3. Implement `integrity.get_outbox_lifecycle`, `integrity.retry_outbox_messages`, and `integrity.flush_offline_queue` RPC endpoints, satisfies **AC-4**, **AC-5**, **AC-6**.
4. Build Outbox Monitor & Management view in `ui/src/components/communication/outbox-monitor-view.tsx`, satisfies **AC-4**, **AC-5**, **AC-6**.

## Consequences

**Positive**:
* Zero message loss when internet drops during high volume admissions or fee collection.
* Prevents costly duplicate SMS transmissions to parents.

**Negative / tradeoffs**:
* Requires a lightweight background polling timer to monitor network state.
