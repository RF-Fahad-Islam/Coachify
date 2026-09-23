# 0032-offline-resilience-center. Offline Resilience Center

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 45

**Date**: 2026-09-23

## Summary

This specification establishes the Offline Resilience Center, a persistent status monitoring panel embedded directly in the Coachify desktop application header. The panel tracks four vital system pulses: real time internet connectivity, outbox message queue depth, backup snapshot recency, and SQLite database storage health. The widget reflects state transitions within 5 seconds using clear color coded indicators (green for healthy, amber for warning, red for critical). Clicking the indicator expands a diagnostic drawer that explains current conditions and provides one click corrective actions.

## Requirements

**User stories**:
* As a counter receptionist, I want to glance at the top bar and see whether the internet is working and how many SMS notices are waiting, so that I know whether parents are receiving instant receipts.
* As an administrator, I want immediate visual alerts if a scheduled backup fails or if database integrity requires attention, so that technical issues are resolved before they cause data loss.

**Acceptance criteria**:
* **AC-1**: Persistent status widget in the application top bar updates its indicators within 5 seconds of any network disconnect, reconnect, or backup completion.
* **AC-2**: RPC `integrity.get_resilience_status` aggregates internet connectivity, pending outbox message count, last successful backup timestamp, disk free space, and database integrity state.
* **AC-3**: Traffic light indicators summarize health:
  * Green: Online, outbox clear or sending, backup under 24 hours old, database healthy.
  * Amber: Offline with messages queued, backup between 24 and 72 hours old.
  * Red: Database integrity error, backup older than 72 hours, or outbox failure threshold exceeded.
* **AC-4**: Clicking the resilience indicator opens a slide over drawer displaying itemized diagnostics, network latency, queue depth breakdown by channel, and a direct "Send Queued Now" button.

## Decision

**Chosen option**: Lightweight Core Status Aggregator with 5 Second Reactive Client Polling

The Rust core exposes `integrity.get_resilience_status` which inspects SQLite metadata, file system statistics, and recent outbox rows in under 15 milliseconds. The React UI utilizes a TanStack Query hook with a 5 second polling interval, automatically pausing when the application window is minimized or idle.

## Rationale

Front desk staff should never have to wonder if the system is properly communicating with SMS gateways or if backups are being taken. Placing an unobtrusive, always visible pulse indicator in the header provides peace of mind and proactive notification without interrupting daily cashier tasks.

## Feature design

**Data model sketch**:
* No dedicated storage table required. Reads dynamically from `backup_history`, `outbox`, and SQLite PRAGMAs.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.get_resilience_status` | Read | none | ResilienceStatus | Authenticated | 500 query failed |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Resilience status | internet_connected | Socket connect probe to 1.1.1.1 or gateway |
| Outbox depth | queued_messages_count | COUNT from outbox WHERE status = 'queued' |
| Backup recency | seconds_since_last_backup | NOW() minus completed_at from latest backup_history row |
| DB health | db_status string | Result of cached startup check or last integrity run |

**Key invariants**:
* The resilience check must complete in under 20 milliseconds to prevent UI sluggishness during 5 second polling.
* The widget must remain fully interactive and render accurate local state even when completely offline.

**Security model**:
* All authenticated roles (Receptionist, Cashier, Teacher, Admin, Owner) have permission to view the resilience widget.

**Critical test scenarios**:
* Fast state transition: Disconnects mock internet, polls status, asserts internet_connected is false within 5 seconds, verifies AC-1 and AC-2.
* Amber warning trigger: Queues 10 messages with mock offline mode, verifies indicator status changes to amber, verifies AC-3.
* Drawer details: Opens resilience drawer, verifies outbox counts and last backup age are rendered accurately, verifies AC-4.

## Build plan

1. Implement `integrity.get_resilience_status` query aggregator in `crates/core/src/integrity.rs`, satisfies **AC-2**, **AC-3**.
2. Create TanStack Query hook `useResilienceStatus` in `ui/src/hooks/use-resilience-status.ts`, satisfies **AC-1**.
3. Build header status badge and diagnostic slide over drawer in `ui/src/components/layout/resilience-status-badge.tsx`, satisfies **AC-1**, **AC-3**, **AC-4**.

## Consequences

**Positive**:
* Continuous operational awareness for all counter staff.
* Immediate alert when internet or backup pipelines degrade.

**Negative / tradeoffs**:
* Adds a background RPC query every 5 seconds while the application window is active.
