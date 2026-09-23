# 0032-guardian-contact-health. Guardian Contact Health

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 42

**Date**: 2026-09-23

## Summary

This specification introduces a guardian contact health analyzer and bulk correction facility. The analyzer scans the student and guardian directory to surface missing phone numbers, invalid Bangladeshi phone numbers (not matching 11 digits starting with 01), duplicates across unrelated student accounts, and shared sibling contacts. Staff can view an actionable contact health register and execute bulk inline phone number updates and normalizations directly from the view.

## Requirements

**User stories**:
* As a counter receptionist, I want to see a list of students with missing or invalid guardian phone numbers, so that I can collect and update correct numbers during admission follow ups.
* As an administrator, I want to see which students share the same guardian contact number, so that sibling accounts are recognized and families receive coordinated messages.
* As a communications officer, I want to clean and normalize a list of malformed numbers in bulk, so that automated SMS broadcasts do not fail at the gateway.

**Acceptance criteria**:
* **AC-1**: RPC `integrity.get_guardian_contact_health` inspects guardian records and returns categorized counts and lists of missing phones, malformed phone strings, duplicate phone numbers, and sibling groups.
* **AC-2**: Bangladeshi phone number validation strictly verifies the standard format of 11 digits starting with `013`, `014`, `015`, `016`, `017`, `018`, or `019` after stripping whitespace and international prefixes (`+88` or `88`).
* **AC-3**: Sibling detection groups all student records that share the same normalized primary guardian phone number.
* **AC-4**: RPC `integrity.bulk_update_guardian_contacts` updates multiple guardian phone records in a single immediate SQLite transaction, re normalizing phone index strings and logging changes to the audit log.

## Decision

**Chosen option**: On Demand SQL Diagnostic Aggregation with Immediate Bulk Normalization

We implement high performance SQL queries over `guardians`, `students`, and `student_guardians` utilizing trigram index columns and standard regex filters. Bulk updates apply validation rules in Rust core before committing rows within an atomic SQLite transaction.

## Rationale

SMS gateways in Bangladesh reject messages with non standard phone numbers, resulting in wasted delivery credits and missed parent notifications. Rather than only blocking invalid input on new forms, an on demand health view empowers staff to clean legacy imported student data. Sibling detection eliminates redundant outbound messages when broadcasting general center notices to parents.

## Feature design

**Data model sketch**:
* Utilizes existing `guardians`, `students`, and `student_guardians` tables.
* Re normalizes `phone_norm` field in `student_search_index`.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.get_guardian_contact_health` | Read | limit: Option<u32>, offset: Option<u32> | ContactHealthReport | `students.view` | 500 query failed |
| `integrity.bulk_update_guardian_contacts` | Mutation | updates: Vec<GuardianContactUpdate> | BulkUpdateResult | `students.manage` | 400 validation error |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Health report | missing, invalid, duplicate lists | Filtered SQL queries on guardians table matching phone IS NULL, length != 11, or COUNT > 1 |
| Sibling groups | shared guardian student lists | Self join on student_guardians grouped by guardian phone |
| Bulk update | updated guardian records | User provided new phone strings validated and updated in DB |

**Key invariants**:
* Normalized phone numbers must consist of exactly 11 digits starting with 01.
* Bulk phone updates must reindex the search table within the same transaction to maintain search consistency.

**Security model**:
* Receptionist, Admin, and Owner roles can view the contact health report.
* Receptionist, Admin, and Owner roles can perform bulk updates.

**Critical test scenarios**:
* Malformed phone detection: Inserts guardian with phone "0171122", asserts health check flags it as invalid format, verifies AC-1 and AC-2.
* Sibling grouping: Links two students to a guardian with phone "01711000000", asserts sibling group returns both students, verifies AC-3.
* Bulk normalization: Submits bulk update normalizing 5 guardian phones, asserts all 5 records updated and normalized search index updated, verifies AC-4.

## Build plan

1. Implement SQL contact health query logic and Bangladeshi phone validator in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**, **AC-3**.
2. Implement `integrity.bulk_update_guardian_contacts` transaction handler, satisfies **AC-4**.
3. Build Guardian Contact Health management view in `ui/src/components/communication/guardian-contact-health-view.tsx`, satisfies **AC-1**, **AC-3**, **AC-4**.

## Consequences

**Positive**:
* Eliminates failed SMS gateway charges caused by malformed numbers.
* Surfaces family relationships across multiple enrolled students.

**Negative / tradeoffs**:
* Running full contact health scans on large databases with over 15,000 records takes roughly 80 to 200 milliseconds.
