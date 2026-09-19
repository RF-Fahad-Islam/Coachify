# Scope: Coachify

A standalone, offline first Windows app that runs a coaching center or small institute (students, fees and cashier, attendance, exams, SMS, backups), plus a small vendor cloud that only handles customers, licenses, and releases. Built for Bangladesh offices with patchy internet and ordinary PCs and printers.

**Build approach:** Tracer Bullet (prove one real thread through every layer first, then thicken it slice by slice).
**Workflow:** Beta (after `/develop`: `/check verify`, then `/test`). Money, safety, licensing, migrations, and release features are tagged `· GA` and also get a fresh model `/check review` and `/document`. `/architect` is the recommended first stop for a feature with a real decision, but skippable when you already know the build. Any feature can carry its own tag to do more or less.

_These are recommendations to keep your build orderly, not requirements. Skip anything that does not fit: if you already know how to build a feature, use `/develop` and skip `/architect`. You decide when a feature is `done`._

Decision input for every `/architect` run: [architecture-review.md](../architecture-review.md). Version 1.0 is single PC; LAN multi PC is 1.1, but feature 5 builds the seam for it from day one. Pilot 1 can start fee collection once Slice 3 is done. Plans: Coachify Local and Coachify Local Pro (Pro adds only LAN). Decisions made 2026-09-20 are logged in Section 20 of the review.

## At a glance

| # | Feature | Phase | Status |
|---|---------|-------|--------|
| 1 | Stack & architecture | Foundation | in-progress |
| 2 | Coding standards & tooling | Foundation | planned |
| 3 | Data model & database conventions | Foundation | planned |
| 4 | Design system & UI foundation | Foundation | planned |
| 5 | Command dispatcher & local accounts | Foundation | planned |
| 6 | Audit log | Foundation | planned |
| 7 | Migration engine | Foundation | planned |
| 8 | Setup wizard & key management | Foundation | planned |
| 9 | Students & guardians | Slice 1 | planned |
| 10 | Programs, batches, staff & enrollments | Slice 1 | planned |
| 11 | Excel student import | Slice 1 | planned |
| 12 | Demo & load data generator | Slice 1 | planned |
| 13 | Financial ledger model | Slice 2 | planned |
| 14 | Fee plans & charge generation | Slice 2 | planned |
| 15 | Print engine & receipts | Slice 2 | planned |
| 16 | Fast cashier & payments | Slice 2 | planned |
| 17 | Voids, refunds & adjustments | Slice 2 | planned |
| 18 | Admission with initial payment | Slice 2 | planned |
| 19 | Dues & due statement | Slice 2 | planned |
| 20 | Expenses & daily cash close | Slice 2 | planned |
| 21 | Backup & verification | Slice 3 | planned |
| 22 | Restore & corruption recovery | Slice 3 | planned |
| 23 | Diagnostics & logs | Slice 3 | planned |
| 24 | Installer & update pipeline | Slice 3 | planned |
| 25 | Attendance | Slice 4 | planned |
| 26 | SMS outbox & provider adapters | Slice 4 | planned |
| 27 | Message templates & automations | Slice 4 | planned |
| 28 | Exams & mark entry | Slice 5 | planned |
| 29 | Results & report cards | Slice 5 | planned |
| 30 | Reports center & exports | Slice 6 | planned |
| 31 | Role dashboards | Slice 6 | planned |
| 32 | Desktop licensing & read only mode | Slice 7 | planned |
| 33 | Vendor admin & license issuing | Slice 7 | planned |
| 34 | Activation & refresh service | Slice 7 | planned |
| 35 | Website, downloads & customer portal | Slice 7 | planned |
| 36 | Code signing & release channels | Slice 7 | planned |
| 37 | Crash & failure safety suite | Slice 8 | planned |
| 38 | Performance on the reference PC | Slice 8 | planned |
| 39 | Customer docs & support runbook | Slice 8 | planned |
| 40 | 1.0 readiness & pilots | Slice 8 | planned |

## Epics

- [Foundation](foundation.md): features 1 to 8 · 1 in-progress · 7 planned
- [Core records](records.md): features 9 to 12 (Slice 1, the walking skeleton) · 4 planned
- [Money](money.md): features 13 to 20 (Slice 2) · 8 planned
- [Safety & delivery](safety.md): features 21 to 24 (Slice 3, pilot gate) · 4 planned
- [Academics & messaging](academics.md): features 25 to 31 (Slices 4 to 6) · 7 planned
- [Commercial](commercial.md): features 32 to 36 (Slice 7, can run in parallel from Slice 3) · 5 planned
- [Release hardening](release.md): features 37 to 40 (Slice 8) · 4 planned

## Deferred

Out of scope for 1.0, kept so the plan stays honest.

**1.1**
- **LAN host & pairing** (Local Pro only): host role, discovery, PAKE pairing, device management, firewall checks · needs a decision · GA
- **LAN workstation client mode**: rpc over pinned TLS, reconnect behavior, update from host · needs a decision · GA
- **Bangla UI**: full interface translation · needs a decision
- **Email channel**: SMTP delivery through the outbox · needs a decision
- **Inquiry & applicant pipeline**: leads before admission
- **ID cards, admit cards & barcode scan**: printed cards, scan for attendance and cashier search · needs a decision
- **Online renewal checkout**: gateway payment in the portal · needs a decision · GA
- **Teacher payment ledger**
- **Per counter cash sessions** · GA
- **Bulk tools**: bulk status and concession changes
- **SMS delivery status polling**
- **Staged update rollout**

**V2**
- **Encrypted off site backup** (paid add on) · needs a decision · GA
- **Managed SMS credits** (paid add on) · needs a decision
- **Multi branch consolidated reporting** · needs a decision
- **Windows service host mode** · needs a decision
- **Guardian payment links** · needs a decision · GA
- **Timetable conflicts & batch merging**

**Do not build yet:** LMS, video, live classes, AI tutor, social features, hostel, transport, library, inventory, payroll, double entry accounting, mobile apps, cloud student portal, any NF ecosystem link or SSO, macOS or Linux builds, coupons, in house ticketing.

## Legend

**The decision box.** Every feature carries exactly one, the sub-task whose label ends with `(spec)`. Its wording varies (`Design it (spec)` normally, `Decide the stack (spec)` on Stack & architecture), so skills locate it by that `(spec)` suffix, never by an exact label. Every other box is an execution box and `/architect` never ticks one.

**Feature lifecycle**: the scope updates as a feature moves; each row is what it shows and who sets it:

| State | Set by | The feature shows |
|---|---|---|
| `planned` · needs a decision | `/scope` | one box: `Design it (spec): /architect <feature>` |
| `in-progress` (designed) | **`/architect` at spec capture** | `Design it` ticked; spec linked; `Build it: /develop <feature>` + **2 to 5 milestones**; the tier's closing boxes (`Verify it` Alpha+, `Test it` Beta+, `Review it` + `Document it` GA); any surfaced follow-up enrolled |
| `in-progress` (building) | `/develop` | milestone sub-boxes tick one by one; code pointer filled |
| `in-progress` (verified) | `/check verify` | `Build it` + milestones ticked; `Verify it` ticked |
| `done` | **you, when you decide it is** (any skill sets it when you say so); `/sync` reconciles | boxes you ran ticked, skipped ones marked skipped; the tier's last stage (`Prototype` → after `/develop`; `Alpha` → after `/check verify`; `Beta`/`GA` → after `/test`) is the suggested point to call it done; `/sync` captures conventions |

- **Next step** = the first unticked box (always a command or a tracked milestone).
- **needs a decision** = run `/architect` first; otherwise straight to `/develop` (or `/audit` for standards & tooling). The tag drops once the spec is captured.
- **Atomic build tasks live in the spec's `## Build plan`, not here**: the scope carries only the milestone rollup.
- **Status** `planned` → `in-progress` → `done`, plus `existing` (pre-workflow) and `dropped` (de-scoped, kept for history).
- **Approach tag** beside a heading (e.g. `· Facade`) overrides the project default for that feature; no tag = inherits it.
- **Workflow tier tag** beside a heading (e.g. `· GA`, `· Prototype`) sets that one feature's rigor above or below the project default; no tag inherits the default. It decides the feature's check boxes and each skill's next suggestion.
- **Workflow** (header line) is the project default, what runs after `/develop`: **Prototype** = nothing (trust develop's own build time self check); **Alpha** = `/check verify`; **Beta** = `/check verify` then `/test`; **GA** = adds a fresh model `/check review` then `/document`. A feature built on an unratified decision (an `Assumed` spec) stays flagged, but that never blocks `done`.
- **Pointer line** (`spec <n> · code in <path>`): the spec link added by `/architect`, the code path by `/develop`.
