# 0016. Admission with initial payment

**Date**: 2026-09-22
**Status**: Accepted

## Summary

In coaching centers and private educational institutes throughout Bangladesh, student admission is the foundational customer-facing event. A parent and student arrive at the front desk to enroll in one or more academic programs or exam preparation batches. In that single visit, the receptionist must gather student and guardian demographics, link shared family guardians to prevent duplicate contact records, assign the student to target batches, compute standard admission charges and initial tuition fees, apply any sibling or merit concessions, collect an initial upfront payment, mint a formal receipt sequence, and hand the parent both an official printed Admission Form / Enrollment Card and a payment voucher.

Feature 18 delivers the **Direct Admission Pipeline**: an atomic, multi-entity transaction orchestrator in the Rust core paired with a high-density, multi-step intake wizard in the desktop interface. All entity writes—student profile, guardian linkage, batch enrollments, concession records, fee charges, payment receipt, payment allocations, synchronous balance cache update, and audit log entries—execute inside a single SQLite `BEGIN IMMEDIATE` transaction.

If any failure occurs (e.g. invalid phone number, program batch capacity exceeded, closed cash day, or invalid payment parameters), the entire transaction aborts immediately with zero partial records. Upon successful commit, Coachify immediately dispatches the dual print trigger: rendering the Student Admission Form / Enrollment Card and the official payment receipt via the WebView2 print engine.

## Requirements

**User stories**:
- As a front-desk receptionist, I want to capture student particulars, guardian contact, batch enrollments, fee charges, and initial payment in one continuous workflow, so that an admission takes less than two minutes.
- As a parent, I want to pay the admission fee and first month's tuition upon registration and immediately receive both a signed admission enrollment form and an official receipt voucher.
- As an institute owner, I want the admission process to be strictly atomic, so that a software crash, network glitch, or payment failure never leaves behind half-created students with no enrollment or charges.
- As an administrator, I want fee plans from selected batches to automatically calculate total admission dues and allow custom percentage or fixed concessions to be configured right in the admission dialog.
- As a cashier, I want initial payment collection to be optional, so that students can be admitted with zero upfront payment and pay their dues later.

**Acceptance criteria**:
- **AC-1**: Direct admission executes inside an immediate SQLite transaction (`BEGIN IMMEDIATE`): atomically creates the student record (`students`), creates or links the guardian (`guardians`, `student_guardians`), creates batch enrollments (`enrollments`), inserts student concessions (`concessions`), generates initial admission and tuition charges (`charges`), records upfront payment (`payments`, `payment_allocations`), updates the balance cache (`student_balances`), and logs audit events. If any step fails, the entire transaction rolls back completely.
- **AC-2**: Fee calculation automatically derives charges from active `fee_plans` for the selected programs/batches, populating one-time admission fees, registration fees, and first-month tuition charges. Concessions entered during admission immediately adjust gross amounts to calculate net initial payable dues.
- **AC-3**: Upfront payment collection is optional. When enabled, the engine validates that the cash day is open, validates payment method (`cash`, `bkash`, `nagad`, `bank`, `card`, `other`), mints the next strictly monotonic annual receipt sequence (`REC-YYYY-####`), and allocates funds to the generated admission charges oldest/priority first. Any excess payment is credited as advance balance.
- **AC-4**: Sibling deduplication: entering a guardian phone number automatically searches existing guardians via normalized phone lookup. If matched, the admission links to the existing guardian without creating duplicate records.
- **AC-5**: Monotonic student code generation automatically issues the next annual student ID (`STU-YYYY-####`) unless a validated custom student code is provided.
- **AC-6**: User interface provides a high-density, multi-step Direct Admission Wizard (`AddStudentDialog` enhanced or dedicated `/admission` route) with:
  1. Personal & contact details (with photo capture/upload via vault).
  2. Guardian details (with instant phone search and relationship selector).
  3. Academic batch selection (multi-batch enrollment with schedule clash warnings).
  4. Fee & Concession overview (showing standard dues, concession inputs, and net payable).
  5. Initial payment counter (amount, method, reference, and quick-fill buttons).
- **AC-7**: Post-save dual print trigger: immediately offers one-click silent or preview printing of both the Student Admission Form / Identity Card and the standard Payment Receipt Voucher (A4 dual-copy, A5, or 80 mm thermal).

## Decision

**Chosen option**: Option 1: Unified Multi-Entity Atomic Admission Transaction in Rust Core with Synchronous Fee Generation and Dual Print Trigger.

Implementation lives in:
- Backend: `crates/core/src/students.rs` & `crates/core/src/ledger.rs` (orchestrator function `admit_student_with_payment`), `crates/core/src/dispatcher.rs` (RPC command `students.admit_with_payment`).
- Frontend: `ui/src/components/students/add-student-dialog.tsx` (extended with batch selector, fee breakdown, and payment collection tabs) and `ui/src/components/print/admission-form-template.tsx` (printable A4 admission form with academic branding).

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Direct admission transaction pipeline

```
 [ Front Desk Intake Form ]
             │
 ┌───────────▼───────────────────────────────────────────────┐
 │ BEGIN IMMEDIATE TRANSACTION                               │
 │                                                           │
 │ 1. Validate demographics & generate STU-YYYY-####         │
 │ 2. Deduplicate/link guardian & normalize phone            │
 │ 3. Insert student & student_guardians records             │
 │ 4. Insert batch enrollments & verify capacity             │
 │ 5. Evaluate fee plans -> generate admission & tuition     │
 │ 6. Apply concessions -> compute net initial payable       │
 │ 7. IF initial payment collected:                          │
 │    - Verify open cash day                                 │
 │    - Mint next receipt number REC-YYYY-####               │
 │    - Insert payment row (status = 'posted')               │
 │    - Allocate payment funds to charges                    │
 │ 8. Synchronously update student_balances cache            │
 │ 9. Insert cryptographic audit log entries                 │
 │ 10. Enqueue welcome SMS in outbox (if rule enabled)       │
 └───────────────────────────┬───────────────────────────────┘
                             │ COMMIT
 ┌───────────────────────────▼───────────────────────────────┐
 │ DUAL PRINT DISPATCH                                       │
 │ • Payment Receipt Voucher (Spec 0013)                     │
 │ • Student Admission Form & Roster Card (Spec 0016)         │
 └───────────────────────────────────────────────────────────┘
```

### Data model & RPC inputs

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmissionConcessionInput {
    pub fee_head_id: String,
    pub batch_id: Option<String>,
    pub r#type: String, // "percent" | "fixed"
    pub value: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmissionPaymentInput {
    pub amount_poisha: i64,
    pub method: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmitStudentInput {
    pub student: CreateStudentInput,
    pub guardian: GuardianLinkInput,
    pub batch_ids: Vec<String>,
    pub concessions: Vec<AdmissionConcessionInput>,
    pub payment: Option<AdmissionPaymentInput>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmitStudentResponse {
    pub student_id: String,
    pub student_code: String,
    pub guardian_id: String,
    pub enrollment_ids: Vec<String>,
    pub charge_ids: Vec<String>,
    pub payment_receipt: Option<PaymentReceipt>,
    pub total_due_poisha: i64,
    pub balance_poisha: i64,
}
```

## Build plan

1. Implement atomic `admit_student_with_payment` in `crates/core/src/students.rs` combining student creation, guardian linking, batch enrollments, charge generation, and payment recording in a single immediate SQLite transaction, satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**.
2. Register RPC command `students.admit_with_payment` in `crates/core/src/dispatcher.rs` with `students.manage` permission and write license gate, satisfies **AC-1**.
3. Create printable React admission form template `ui/src/components/print/admission-form-template.tsx` with student photo, barcode, batch schedules, institute crest, and guardian signature lines, satisfies **AC-7**.
4. Extend `ui/src/components/students/add-student-dialog.tsx` with multi-batch selector, automatic fee breakdown, concession toggles, and optional payment collection desk, satisfies **AC-6**.
5. Connect post-admission dialog actions to dual silent/preview printing of receipt and admission form, satisfies **AC-7**.
6. Write integration tests in `crates/core/tests/admission_with_payment.rs` asserting atomic rollback on payment failure and complete balance consistency, satisfies **AC-1**, **AC-3**.

## Consequences

**Positive**:
- Unifies student intake: replaces a cumbersome 4-step disconnected workflow with a single cohesive dialog.
- 100% atomic integrity: parents never leave with a receipt for a student record that failed to save.
- Instant professional presentation: prints both formal enrollment card and receipt voucher immediately at the counter.

**Negative / Tradeoffs**:
- A larger payload requires thorough frontend form validation before dispatching to the core.
