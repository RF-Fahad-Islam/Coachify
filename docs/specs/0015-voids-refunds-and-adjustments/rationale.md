# 0015. Voids, refunds and adjustments (Rationale)

## Context

In high-volume coaching centers, front-desk counter staff process hundreds of payment transactions daily. Mistakes occur routinely:
- A cashier selects the wrong student account from search and issues a receipt.
- A cashier mistypes the amount (e.g., entering ৳25,000 instead of ৳2,500).
- A parent pays for a course and requests money back three weeks later after moving to another city.
- An institute director approves a 50% hardship waiver on outstanding exam fees for a student.

Traditional software solutions typically handle these scenarios in destructive ways: allowing users to edit or delete payment records, or leaving open database tables that invite embezzlement. In a cash-heavy environment, an unprincipled void mechanism allows a dishonest cashier to collect cash, print a receipt, pocket the money, and void the transaction days later without anyone noticing.

To solve this, Coachify establishes clear boundaries between **same-day operational voids**, **post-dated refunds**, and **accounting adjustments**, backed by immutable database triggers.

## Options considered

### Option 1: Strict Temporal Split (Same-Day Voids vs. Formal Refunds & Adjustments)

- **Voids**: Allowed *only* on the exact calendar day of receipt issuance (`business_date == today`), *only* while the daily cash drawer remains open (`closed_at IS NULL`), and requires the `payments.void` permission and an explicit reason. The original payment record is retained with `status = 'voided'`, and the monotonic receipt number remains permanently assigned to prevent gaps in sequence audits.
- **Refunds**: Allowed at any time for returning unallocated credit, creating an independent negative cash-flow record on the current date, tied to the original payment and limited to unallocated funds.
- **Adjustments**: Explicit waivers and fines attached to student accounts or specific charges, subject to trigger-enforced ceilings.

**Pros**:
- Closes the counter embezzlement vector: once a cash day is closed and money handed over, historical receipts cannot be cancelled.
- Preserves audit trails: no sequence numbers vanish, and every void/refund is logged with identity and timestamp.
- Unallocated ceiling triggers guarantee that refunds cannot exceed money actually received.
- Simple mental model for staff: "Made a typo today? Void it. Need to give money back next month? Issue a refund."

**Cons**:
- Inflexible if management discovers an error several days late: they must process an adjustment and refund rather than a simple void button.

### Option 2: Unrestricted Supervisor Voids at Any Time

Allow users with Administrator or Owner credentials to void any transaction in the system at any date in history.

**Pros**:
- Maximum convenience for resolving historical data errors.

**Cons**:
- Breaks the integrity of past cash drawer closures: past closed days become retroactively unbalanced.
- Facilitates collusive fraud between managers and cashiers.
- Complicates accounting reconciliation: historical reports would change retroactively when rerun.

### Option 3: Mutable Editable Records

Allow cashiers to simply click "Edit" on a payment or charge to fix mistakes in-place.

**Pros**:
- Lowest database row volume.

**Cons**:
- Catastrophic for auditability and trust: eliminates evidence of what actually happened at the counter.
- Strictly violates Coachify's core append-only architectural principle.

## Rationale

Option 1 is selected because it strictly enforces accounting integrity while providing practical operational tools. By restricting voids to open, same-day sessions, Coachify guarantees that closing a cash day locks its transactions permanently. For legitimate money returns on later dates, the dedicated `refunds` workflow tracks money flowing out of today's drawer without falsifying historical records.
