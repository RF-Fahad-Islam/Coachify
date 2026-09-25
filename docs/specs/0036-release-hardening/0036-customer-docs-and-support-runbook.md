# 0036. Customer Documentation and Support Runbook (Feature 60)

**Date**: 2026-09-24

## Summary

This specification establishes customer facing documentation, an internal support runbook, and a lightweight in app help viewer for Coachify. It equips coaching center staff to install the software, configure thermal and laser printers, import student records from Excel, configure SMS gateways, and perform backup restore drills without technical assistance. It also provides support specialists with clear step by step procedures to resolve antivirus alerts, printer configuration problems, emergency clock jumps, and forgotten recovery keys. Everything is available completely offline inside the desktop application.

## Requirements

**User stories**:
* As a non technical coaching center receptionist, I want clear illustrated guides accessible inside the application, so that I can configure receipt printers, import student rosters, and run daily backups without calling technical support.
* As an institution owner, I want a plain language guide explaining how our data is stored, encrypted, and backed up, so that I can be certain our student and financial information never leaves our office.
* As a customer support specialist, I want a structured troubleshooting runbook with concrete resolution steps for Windows SmartScreen, antivirus blocks, printer errors, and clock jump recovery, so that I can solve owner emergencies calmly and quickly.

**Acceptance criteria**:
* **AC-8**: The desktop shell provides a lightweight in app Help dialog accessible from the top navigation bar, rendering bundled markdown documentation loaded directly at build time by Vite with zero internet dependency.
* **AC-9**: Comprehensive customer documentation is authored in `docs/customer/` with printable PDF versions covering Quick Start, Backup and Restore, Thermal and Laser Printer Setup, SMS Gateway Configuration, Excel Student Import, Troubleshooting (including Windows SmartScreen and antivirus exceptions), Privacy and Data Ownership, and the Owner License Guide.
* **AC-10**: An internal support runbook in `docs/customer/support-runbook.md` provides Tier 1 and Tier 2 resolution procedures for lost encryption recovery sheets, database corruption recovery, emergency clock jump recovery, damaged thermal printers, and forgotten staff credentials.

## Decision

**Chosen option**: Markdown Source Guides with In App Help Modal and Printable PDFs

We author all customer guides in plain GitHub Flavored Markdown within `docs/customer/`. These markdown files are bundled directly into the React application bundle at compile time using Vite raw glob imports (`import.meta.glob('../../../docs/customer/*.md', { query: '?raw', eager: true })`). This allows the in app Help modal to load and display them instantly with syntax highlighting and searchable topics without network calls or core RPC overhead. Printable A4 PDF versions are rendered from the same source files using the desktop print engine. The internal support runbook remains in `docs/customer/support-runbook.md` as an operational guide for support personnel.

## Rationale

Coaching centers in Bangladesh often have unreliable internet connectivity. Storing documentation exclusively on a remote website would leave staff stranded during connectivity outages. Packaging the guides inside the desktop application guarantees that help is always accessible. Importing markdown directly in Vite avoids adding unnecessary RPC handlers or duplicating static files into separate asset directories.

## Feature design

**Documentation catalog**:

* `docs/customer/01-quick-start.md`: First time installation, setup wizard walkthrough, creating programs and batches, admitting first students.
* `docs/customer/02-backup-and-restore.md`: Setting up secondary backup drives, understanding the Recovery Sheet, running daily backups, testing restore drills.
* `docs/customer/03-printer-setup.md`: Connecting 80mm thermal receipt printers, setting paper widths, laser printer settings for report cards and due statements.
* `docs/customer/04-sms-setup.md`: Configuring SMS gateway credentials, managing message templates, reviewing outbox queues and handling internet blackouts.
* `docs/customer/05-excel-student-import.md`: Template format requirements, column mapping, handling duplicate mobile numbers, verifying imported enrollments.
* `docs/customer/06-troubleshooting.md`: Handling Windows SmartScreen warnings, adding Windows Defender exclusions, clearing printer queues, resolving date mismatch warnings.
* `docs/customer/07-privacy-and-ownership.md`: Offline first architecture explanation, local database encryption details, commercial licensing boundaries.
* `docs/customer/08-license-guide.md`: Activating trial licenses, importing offline license keys, machine transfers, subscription renewal steps.
* `docs/customer/support-runbook.md`: Internal procedures for field technicians: restoring corrupt databases from WAL copies, DPAPI key rewrapping, emergency clock jump recovery via `acknowledge_clock_jump`, printer driver diagnosis, owner password resets.

**In app help modal layout**:

```
+-------------------------------------------------------------+
|  Coachify Help & Documentation                     [Search] |
+-------------------------------+-----------------------------+
|  Topics                       |  Quick Start Guide          |
|  ---------------------------  |  -------------------------  |
|  > Quick Start Guide          |                             |
|    Backup and Restore         |  Welcome to Coachify! This  |
|    Printer Setup              |  guide will help you set up |
|    SMS Setup                  |  your coaching center in    |
|    Excel Import               |  less than ten minutes.     |
|    Troubleshooting            |                             |
|    Privacy & Ownership        |  Step 1: Run the installer  |
|    License & Activation       |  ...                        |
|                               |                             |
|  [Print Guide as PDF]         |  [Back to Top]              |
+-------------------------------+-----------------------------+
```

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| Open help topic | Markdown article text | `docs/customer/*.md` loaded directly via Vite raw import |
| Filter help topics | Matching topic titles | Local client search over topic titles and markdown headings |
| Print topic PDF | Formatted printable document | Print engine template `ui/src/components/print/` |

**Key invariants**:
* Help content must be fully readable and searchable with network interfaces disabled.
* Support runbook files must not contain sensitive master encryption keys or hardcoded administrative credentials.
* Documentation must use plain simple words in English, with key interface terms also noted in Bengali where relevant.

**Security model**:
* Customer guides contain zero proprietary trade secrets or sensitive cryptographic parameters.
* The internal support runbook is intended for technical operators and includes reminders to verify owner identity before assisting with data recovery or password resets.

**Critical test scenarios**:
* Scenario 1 (Offline Help Navigation): A user clicks Help in the top navigation bar while disconnected from the internet, successfully opening and reading all eight customer guides, verifying **AC-8**.
* Scenario 2 (First Time Onboarding Simulation): A tester follows only `01-quick-start.md` on a clean Windows machine to install Coachify, complete the setup wizard, and enroll a sample batch, verifying **AC-9**.
* Scenario 3 (Printer Troubleshooting Verification): A tester follows `03-printer-setup.md` to resolve a simulated paper jam and offline thermal printer status, verifying **AC-9**.
* Scenario 4 (Support Runbook Execution): A technician follows `support-runbook.md` to simulate recovering a database after an emergency clock jump, verifying **AC-10**.

## Build plan

1. Author the eight customer guides in `docs/customer/` with clear step by step instructions and checklists, satisfies **AC-9**
2. Author internal technical support runbook in `docs/customer/support-runbook.md`, satisfies **AC-10**
3. Configure Vite raw markdown import pipeline for `docs/customer/*.md` in `ui/src/lib/docs.ts`, satisfies **AC-8**
4. Build `HelpDialog` component in `ui/src/components/help/help-dialog.tsx` with sidebar navigation, search bar, and markdown renderer, satisfies **AC-8**
5. Add Help trigger button to top navigation bar in `ui/src/components/shell/top-bar.tsx`, satisfies **AC-8**
6. Wire printable PDF export action to the existing WebView2 print engine, satisfies **AC-9**
7. Perform manual documentation walkthrough on a clean computer to verify clarity, satisfies **AC-9**, **AC-10**

## Consequences

**Positive**:
* Coaching center personnel can resolve everyday configuration questions without waiting for support phone calls.
* First time installation friction is drastically lowered with clear SmartScreen and printer setup steps.
* Field support agents follow standardized protocols, preventing mistakes during emergency recovery drills.

**Negative / tradeoffs**:
* Documentation files must be maintained as features evolve to prevent out of date screenshots or instructions.

**Neutral**:
* Documentation source files live in the desktop repository, making documentation edits part of everyday pull requests.
