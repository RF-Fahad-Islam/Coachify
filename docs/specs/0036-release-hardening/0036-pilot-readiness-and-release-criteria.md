# 0036. Pilot Readiness and Release Criteria (Feature 61)

**Date**: 2026-09-24

## Summary

This specification defines the 1.0 release readiness qualification protocol and field pilot validation process for Coachify. It establishes an evidence based evaluation matrix verifying fourteen critical release criteria across automated test suites, financial ledger stability, backup restore drills, clean Windows installations, and printer compatibility. It coordinates real world trials across three active coaching centers for four consecutive weeks each, ensuring that software is sold commercially only after real operational proof with zero data loss.

## Requirements

**User stories**:
* As the product owner, I want a rigorous readiness checklist tracking real verification logs across all critical subsystems, so that we know exactly when version 1.0 meets commercial standards.
* As an institution owner participating in the beta pilot program, I want our daily fee collection, attendance, and exam cycles verified with zero disruptions, so that our coaching center operates smoothly while validating the software.
* As an engineer preparing the release candidate, I want automated and physical sign off templates for printer compatibility, Windows 10 and 11 installers, and offline license activation, so that nothing is overlooked prior to public launch.

**Acceptance criteria**:
* **AC-11**: A structured release qualification document in `docs/release/1.0-readiness-report.md` tracks and validates the release acceptance criteria defined in Section 17 of the architecture review, excluding reference hardware performance benchmarks which are deferred to 1.1.
* **AC-12**: The pilot verification program tracks at least 3 live pilot institutions operating for 4 or more consecutive weeks each, verifying at least one monthly collection cycle, one examination cycle, a successful restore drill at each location, and zero recorded data loss incidents.

## Decision

**Chosen option**: Evidence Based Readiness Matrix with Structured Pilot Validation Protocol

We establish an auditable release readiness framework centered on `docs/release/1.0-readiness-report.md`. This report records test evidence, git commit hashes, automated test run outputs, and physical device test records across fourteen specific release criteria. For the pilot program, we define a structured tracking template in `docs/release/pilot-matrix.json` that monitors weekly operational milestones, backup verification drills, and site sign offs for three pilot coaching centers.

## Rationale

Software that manages money and student records in a coaching center must be proven in real office conditions before being sold. A product that appears flawless in a developer environment may encounter unexpected printer driver quirks, power cuts, or unusual accounting habits in the field. Requiring three active institutions to run the software live for a full month ensures that monthly billing cycles, exam marks, and restore drills work reliably before wide commercial rollout.

## Feature design

**1.0 Release criteria matrix**:

| # | Criterion | Verification method | Target standard |
|---|---|---|---|
| 1 | Scope completion | Scope audit | All V1.0 features marked done in docs/scope/ |
| 2 | Automated test suite | CI execution | 100% passing across unit, integration, and DB tests |
| 3 | Financial property tests | Core test suite | 100,000 randomized operations with zero invariant violations |
| 4 | Crash safety suite | xtask crash runner | 20 payment kills, 10 backup kills, 10 migration kills with zero corruption |
| 5 | Backup and restore drill | Automated drill | Wipe, restore, compare test passes with identical counts and balances |
| 6 | Installation and upgrade | Windows test matrix | Clean install, upgrade, offline install pass on Windows 10 and 11 |
| 7 | Printer compatibility | Physical verification | Tested on 2 laser printers, 1 inkjet, and 2 thermal 80mm printers |
| 8 | Multi week pilot trials | Field log verification | 3 institutions live for 4+ weeks with zero data loss incidents |
| 9 | Field restore drills | On site verification | Successful restore drill conducted on site at all 3 pilot locations |
| 10 | Defect status | Issue tracker audit | Zero open P0 or P1 bugs; all P2 bugs have documented workarounds |
| 11 | License issuance | Commercial test | Online and offline license issuance and machine transfer verified |
| 12 | Code signing | Packaging audit | Authenticode binary signature and updater signature verified |
| 13 | Documentation suite | Documentation audit | Customer guides and support runbook complete and verified |
| 14 | Security audit | Threat model check | DPAPI protection, injection prevention, and redaction verified |

Note: Performance benchmarking on 4 GB RAM spinning disk hardware is deferred to version 1.1 per user decision.

**Pilot tracking template (`docs/release/pilot-matrix.json`)**:

```json
{
  "pilot_program": {
    "target_duration_weeks": 4,
    "required_sites_count": 3,
    "sites": [
      {
        "id": "pilot-01",
        "name": "Dhaka Central Coaching",
        "student_count": 420,
        "start_date": "2026-10-01",
        "end_date": "2026-10-29",
        "printer_models": ["Xprinter XP-N160I", "Canon LBP2900"],
        "monthly_fee_cycle_completed": true,
        "exam_cycle_completed": true,
        "restore_drill_completed": true,
        "data_loss_incidents": 0,
        "data_verification_hash": "a8f3b2c1e4d5... (latest verified audit_chain_head)",
        "owner_sign_off": "Verified"
      }
    ]
  }
}
```

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| Audit release status | Readiness report table | `docs/release/1.0-readiness-report.md` |
| Track pilot progress | Site milestone booleans | `docs/release/pilot-matrix.json` |
| Record restore drill | Drill timestamp and hash | On site technician drill protocol log |
| Verify printer matrix | Device test checkmarks | Physical test logs from pilot coaching centers |

**Key invariants**:
* A pilot site cannot be marked qualified without completing at least one full monthly tuition billing cycle and one examination cycle.
* Any data loss incident occurring at a pilot site immediately resets the qualification clock for that site.
* The readiness report must reference concrete test output files and git commit hashes for every passing check.

**Critical test scenarios**:
* Scenario 1 (Readiness Matrix Audit): The release engineer validates that all fourteen criteria in `docs/release/1.0-readiness-report.md` are marked complete with attached logs, verifying **AC-11**.
* Scenario 2 (Pilot Milestone Tracking): The pilot coordinator verifies that each pilot institution in `docs/release/pilot-matrix.json` achieves four consecutive weeks of live operation with zero data loss, verifying **AC-12**.
* Scenario 3 (On Site Restore Drill Verification): A field technician conducts a restore drill using a backup from pilot site 1 on a spare laptop, proving that all student balances and receipt counts match exactly, verifying **AC-12**.

## Build plan

1. Create `docs/release/1.0-readiness-report.md` template listing all fourteen criteria with verification evidence fields, satisfies **AC-11**
2. Create `docs/release/pilot-matrix.json` and tracking protocol for pilot coaching centers, satisfies **AC-12**
3. Establish printer testing protocol sheet covering receipt, report card, and due slip printing across thermal and laser models, satisfies **AC-11**
4. Execute financial ledger property tests (100,000 operations) and link test run output into the readiness report, satisfies **AC-11**
5. Execute automated backup wipe and restore test and link verification hashes, satisfies **AC-11**
6. Enroll three pilot institutions into the tracking matrix and review weekly milestone checklists, satisfies **AC-12**
7. Conduct on site restore drills at each pilot institution and record signed verification logs, satisfies **AC-12**
8. Finalize and sign off the 1.0 readiness report upon completing pilot cycles, satisfies **AC-11**, **AC-12**

## Consequences

**Positive**:
* Eliminates guesswork from the 1.0 release decision by basing launch approval on objective test evidence and real customer usage.
* Proves that printing, licensing, backups, and fees work under authentic field conditions.

**Negative / tradeoffs**:
* Conducting four week pilots requires approximately one to two months of calendar time before final public launch.

**Neutral**:
* The pilot tracking and readiness documents are version controlled in the repository, providing a permanent historical record for future releases.
