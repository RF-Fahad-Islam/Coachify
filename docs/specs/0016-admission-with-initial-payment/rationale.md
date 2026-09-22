# 0016. Admission with initial payment (Rationale)

## Context

In Bangladesh coaching centers and institutes, admissions peak sharply before academic sessions and exam crash courses. During these periods, hundreds of students line up at the front desk. In legacy systems or generic ERPs, enrolling a student is fragmented across multiple unrelated screens:
1. Create student in the student directory.
2. Navigate to academic batches to enroll the student.
3. Open a billing module to manually create an admission fee invoice.
4. Open the cashier desk to collect the payment.
5. Print a receipt from one screen, then navigate elsewhere to print an admission slip.

This disconnected workflow takes 5 to 10 minutes per student, forces counter staff to repeat data lookups, and creates critical failure modes:
- A receptionist creates the student and collects cash, but the system crashes before batch enrollment is completed.
- A receptionist enrolls the student, but forgets to invoice the admission fee, causing silent revenue loss.
- Staff print an admission slip, but forget to record the cash receipt in the ledger.

Coachify solves this by making student admission an **atomic, single-transaction composite operation**: all personal data, guardian relationships, batch enrollments, charge calculations, and optional payments are committed together or not at all.

## Options considered

### Option 1: Unified Atomic Composite Admission in Core Transaction

Provide a dedicated `students.admit_with_payment` RPC command that takes a single comprehensive payload and executes the entire pipeline in a single `BEGIN IMMEDIATE` transaction.

**Pros**:
- Absolute atomicity: impossible to have orphan students without enrollments or collected money without a recorded student.
- Exceptional counter throughput: complete student admission with payment and print in under 90 seconds.
- Guarantees revenue capture: admission fees and tuition charges are automatically calculated based on system fee plans.
- Instant dual-print output: prints both the official admission contract and the payment receipt in one flow.

**Cons**:
- Slightly more complex backend transaction function orchestrating multiple domain models.

### Option 2: Client-Orchestrated Sequential RPC Calls

Keep backend commands fine-grained (`students.create`, then `enrollments.create`, then `charges.create`, then `payments.create`) and let the React UI call them sequentially in a `for` loop or `Promise.all`.

**Pros**:
- Avoids adding a new backend RPC command.

**Cons**:
- No atomicity: network drops, browser crashes, or a failure on step 4 leaves orphaned records and unallocated cash.
- High network latency: requires multiple roundtrips across the IPC boundary.
- High risk of client-side state divergence and race conditions.

### Option 3: Two-Phase "Draft Admission" Workflow

Create admissions as "draft" records, requiring a separate manager approval before finalizing charges and payments.

**Pros**:
- Allows multi-step review for large corporate schools.

**Cons**:
- Overengineered and inappropriate for coaching centers: front desks need immediate finalization while the parent stands at the counter.
- Adds unnecessary administrative friction.

## Rationale

Option 1 is selected because it strictly preserves transactional integrity and delivers the high-speed counter experience coaching centers demand. By treating admission and initial payment as an atomic unit in the Rust core, Coachify ensures that institute ledgers and student rosters remain 100% synchronized under all conditions.
