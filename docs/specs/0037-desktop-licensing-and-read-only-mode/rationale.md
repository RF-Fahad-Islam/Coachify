# 0037. Desktop Licensing and Read Only Mode: Rationale

> Decision record and rationale for Spec 0037 (Desktop Licensing and Read Only Mode)

## Context

Coaching centers in Bangladesh operate in environments where internet connectivity is intermittent and desktop computers frequently face abrupt shutdowns, hardware failures, or CMOS battery resets. (basis: project architecture review, Section 9). In this operational reality, software that requires real time online activation or periodic license server heartbeats inevitably locks centers out during critical morning admissions or examination seasons.

At the same time, the vendor commercial model requires a reliable mechanism to govern software subscriptions across two distinct plans: Coachify Local (single workstation) and Coachify Local Pro (LAN multi workstation). (basis: project architecture review, Section 20, Decision 3). The software must enforce subscription validity and prevent unauthorized copying across institutions without introducing cumbersome digital rights management or hardware dongles that frustrate honest educators.

When a coaching center faces computer hardware failure on an exam day, the center must be able to restore an encrypted backup onto a temporary replacement computer and continue admissions, mark entry, and fee collection immediately without waiting hours for support assistance. (basis: project architecture review, Section 9). Furthermore, if a subscription expires, student data, payment receipts, academic results, and financial reports must never be locked away. The application must transition smoothly into a safe read only mode that preserves all viewing, reporting, and export capabilities while preventing new data mutations.

## Options considered

### Option 1: Embedded Ed25519 Signature Verification with Dispatcher Level Gating and 14 Day Recovery Transition

Compile vendor public keys directly into the Rust core binary and verify incoming `.coachifylicense` files offline using Ed25519 digital signatures. Enforce licensing and read only mode at the central command dispatcher pipeline. Provide a 14 day transitional grace window when a restored database detects a new host computer.

**Pros**:
* Functions completely offline with zero reliance on cloud availability or local network stability.
* Cryptographic Ed25519 signatures make file tampering and manual database date modifications impossible.
* The 14 day transitional window guarantees that emergency computer replacements never disrupt desk operations during busy morning rushes.
* Enforcing access rules in the central command dispatcher ensures that all mutation pathways are defended consistently without repeating checks across individual screens.
* Read only mode allows complete access to historical records, reports, receipt reprints, and data backups without holding data hostage.

**Cons**:
* Updating an embedded public key in the event of private key compromise requires releasing a routine desktop software update.
* Revoking a license on an offline computer can only take effect when that computer connects to the internet or reaches its calendar expiration date.

### Option 2: Periodic Online Heartbeat Licensing with Server Polling

Require the desktop application to check in with a cloud license server at least once every 72 hours. If the application cannot verify its status online within this window, it automatically disables write operations.

**Pros**:
* License revocations and plan changes propagate quickly to customer computers.
* Simpler cloud license database without managing offline request codes.

**Cons**:
* Flaky internet connections in regional districts cause false positive lockouts during busy coaching hours.
* Any cloud server downtime or maintenance window immediately threatens local counter operations.
* Violates the foundational offline first architectural principle of Coachify.

### Option 3: Hardware Dongle or Strict Machine Fingerprinting

Tie software licenses strictly to physical motherboard, CPU, and disk serial numbers using proprietary hardware fingerprinting libraries or USB security dongles.

**Pros**:
* Prevents cloning an installation across multiple physical machines.

**Cons**:
* Restoring a backup onto a replacement PC on exam day halts all counter operations until a new license can be generated.
* Routine hardware upgrades (such as adding RAM or swapping a hard drive) trigger false invalidation alerts.
* USB dongles frequently fail, get lost, or suffer physical port damage in crowded reception desks.

## Rationale

We selected Option 1 because it directly addresses the operating conditions of coaching institutions while maintaining commercial integrity. (basis: project architecture review, Section 9). Ed25519 provides industry standard signature verification with tiny 64 byte signatures and high verification speed. (basis: RFC 8032 Ed25519 cryptography). Storing vendor public keys inside the compiled Rust binary means license files can be verified anywhere without network dependencies.

The 14 day grace window and the 14 day moved PC transitional window balance commercial protection with practical operational resilience. (basis: project architecture review, Section 9). If an institution experiences an internet outage, an Eid holiday banking delay, or an emergency motherboard replacement on exam day, staff can continue recording fee payments and student marks without interruption.

Placing the enforcement boundary directly within the command dispatcher (`crates/core/src/dispatcher.rs`) aligns with the defense in depth architecture established in Spec 0004. (basis: Spec 0004 Command Dispatcher and Local Accounts). Because the dispatcher evaluates every incoming command before transaction execution, read only mode cannot be bypassed by client side UI tampering. Read commands, reports, document reprints, and encrypted backups remain accessible, fulfilling our commitment that Coachify will never hold an institution's data hostage. (basis: project architecture review, Section 8 and Section 20, Decision 4).

## References

**Project sources**:
- `AGENTS.md`: Stack conventions, core domain separation, and command dispatcher rules
- `docs/architecture-review.md`: Section 8 (Commercial Rules), Section 9 (Licensing Model), Section 14 (Feature Tiers), and Section 20 (Decisions 1, 3, and 4)
- Spec 0004: `docs/specs/0004-command-dispatcher-local-accounts/index.md` (Command dispatcher pipeline and permission gating)
- Spec 0032: `docs/specs/0032-operational-integrity/0032-database-integrity-checks.md` (Safety read only locks and clock guard watermarks)

**Practices & standards**:
- **Offline first software design**: Institution data and validation engines operate autonomously without continuous cloud connectivity (basis: local first architecture)
- **Asymmetric signature verification**: Ed25519 high speed digital signatures for compact tamper proof license files (basis: IETF RFC 8032)
- **Centralized authorization and gating**: Single point of command interception and entitlement enforcement in the dispatcher (basis: NIST Role Based Access Control)
- **Monotonic clock guard**: Audit log high water marks prevent clock rollback exploitation (basis: NIST SP 800-92 Guide to Computer Security Log Management)
- **Graceful degradation and read only mode**: Expired licenses preserve full search, reprint, export, and backup capabilities (basis: customer data sovereignty)
