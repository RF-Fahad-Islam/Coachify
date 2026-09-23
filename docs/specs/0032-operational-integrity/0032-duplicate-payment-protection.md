# 0032-duplicate-payment-protection. Duplicate Payment Protection

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 41

**Date**: 2026-09-23

## Summary

This specification implements proactive duplicate payment protection at the cashier desk. Before a payment is posted to the ledger, the system inspects recent transaction records to detect repeated payment reference numbers within 24 hours (such as identical bKash or Nagad transaction IDs), as well as suspicious payments with the exact same student and amount on the same business date. If a collision is found, a prominent confirmation dialog warns the cashier and requires an explicit justification reason to proceed. All overrides are recorded directly on the payment row and in the audit log.

## Requirements

**User stories**:
* As a cashier processing payments during busy morning counter hours, I want the system to warn me if I enter a transaction reference that was already recorded today, so that duplicate entries from mobile banking SMS slips are stopped before billing.
* As an owner, I want every duplicate payment override to capture the cashier's stated reason, so that suspicious counter activity can be reviewed during daily audits.

**Acceptance criteria**:
* **AC-1**: Submitting a payment with a reference string that matches another posted payment within the previous 24 hours triggers a high visibility duplicate warning.
* **AC-2**: Submitting a payment for the same student with the exact same poisha amount on the same business date triggers a duplicate confirmation prompt.
* **AC-3**: The cashier can proceed only by entering a mandatory override reason string (minimum 5 characters).
* **AC-4**: The override reason is stored in the `duplicate_override_reason` column of the `payments` record and emits a structured security audit event.

## Decision

**Chosen option**: Proactive Pre Submission Warning with Cashier Reason Gate and Audit Trail

We implement an inspection RPC `integrity.check_duplicate_payment` that runs prior to transaction dispatch, plus server side validation during `ledger.collect_payment`. If a duplicate is detected and no valid override reason is supplied, the dispatch fails with a structured duplicate warning code. Once an override reason is provided, the payment posts with the explanation permanently attached.

## Rationale

Cashiers frequently transcribe ten digit transaction IDs from customer phones or paper slips. In high stress, fast paced environments, entering the same slip twice or double clicking submit buttons are common human errors. Stopping the transaction before ledger posting prevents messy void and adjustment workflows later. Requiring an audit logged reason ensures cashiers only bypass the warning for legitimate cases (such as twins paying identical fees on the same afternoon).

## Feature design

**Data model sketch**:
* `payments`: Column `duplicate_override_reason TEXT` added in Migration 0017.
* Composite index `idx_payments_ref_created ON payments(reference, created_at)` to accelerate duplicate lookups.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.check_duplicate_payment` | Read | student_id: String, amount_poisha: i64, reference: Option<String> | DuplicateWarningResult | `payments.collect` | 400 invalid input |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Duplicate check | matched_payments list | Query on payments filtering by reference in 24 hours or (student_id, amount, business_date) |
| Post payment | duplicate_override_reason | Cashier input string passed from the confirmation dialog |

**Key invariants**:
* Submissions with an identical reference in 24 hours must be rejected unless an override reason is present.
* The override reason cannot be whitespace only and must contain at least 5 characters.

**Security model**:
* Cashier, Admin, and Owner roles can override duplicate warnings with a valid explanation.
* An audit event of type `payment.duplicate_override` is recorded in `audit_log` with the cashier user ID.

**Critical test scenarios**:
* Reference match warning: Creates payment with reference BK123, submits duplicate check with BK123 within 2 hours, asserts duplicate found, verifies AC-1.
* Daily amount collision: Submits payment for Student A with amount 1500 on today date, checks duplicate for Student A with 1500, asserts collision warning, verifies AC-2.
* Override reason audit: Posts payment with duplicate reference and reason "Guardian paid for two siblings using same transaction ID", asserts reason saved on payment and audit log contains event, verifies AC-3 and AC-4.

## Build plan

1. Add `duplicate_override_reason` column and composite index to `payments` in Migration 0017, satisfies **AC-4**.
2. Implement duplicate detection query logic in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Wire duplicate guard into `ledger.collect_payment` in `crates/core/src/ledger.rs`, satisfies **AC-3**, **AC-4**.
4. Create Cashier Duplicate Warning modal in `ui/src/components/cashier/duplicate-warning-dialog.tsx`, satisfies **AC-1**, **AC-2**, **AC-3**.

## Consequences

**Positive**:
* Prevents accidental double collection and duplicate mobile money reference recording.
* Preserves complete accountability through mandatory cashier justification logs.

**Negative / tradeoffs**:
* Adds a lightweight pre submission database query to counter payment workflows.
