# Scope: Coachify

An intelligent, offline first operating system for coaching centers. Standalone Windows app that runs students, fees and cashier, attendance, exams, messaging, backups, and owner intelligence, plus a small vendor cloud that handles only the commercial relationship: licensing, AI credits, downloads, updates, and support. Built for Bangladesh coaching centers with patchy internet and ordinary PCs and printers.

**Build approach:** Tracer Bullet (prove one real thread through every layer first, then thicken it slice by slice).
**Workflow:** Beta (after `/develop`: `/check verify`, then `/test`). Money, safety, licensing, migrations, release, finance integrity, data safety, communication delivery, and security features are tagged `· GA` and also get a fresh model `/check review` and `/document`. `/architect` is the recommended first stop for a feature with a real decision, but skippable when you already know the build. Any feature can carry its own tag to do more or less.

_These are recommendations to keep your build orderly, not requirements. Skip anything that does not fit: if you already know how to build a feature, use `/develop` and skip `/architect`. You decide when a feature is `done`._

Decision input for every `/architect` run: [architecture-review.md](../architecture-review.md). Version 1.0 is single PC; LAN multi PC is 1.1, but feature 5 builds the seam for it from day one. Pilot 1 can start fee collection once Slice 3 is done. Plans: Coachify Local and Coachify Local Pro (Pro adds only LAN). Decisions made 2026-09-20 are logged in Section 20 of the review.

The four strongest differentiators: **operational speed**, **offline/LAN reliability**, **financial/data safety**, and **AI powered operational intelligence**.

The architectural rule: **Institution data lives locally. Coachify Cloud handles the commercial relationship, licensing, AI credits, cloud AI, downloads, updates, and support.**

## At a glance

| # | Feature | Phase | Status |
|---|---------|-------|--------|
| 1 | Stack & architecture | Foundation | in-progress |
| 2 | Coding standards & tooling | Foundation | done |
| 3 | Data model & database conventions | Foundation | done |
| 4 | Design system & UI foundation | Foundation | done |
| 5 | Command dispatcher & local accounts | Foundation | done |
| 6 | Audit log | Foundation | done |
| 7 | Migration engine | Foundation | done |
| 8 | Setup wizard & key management | Foundation | done |
| 9 | Students & guardians | Slice 1 | done |
| 10 | Programs, batches, staff & enrollments | Slice 1 | done |
| 11 | Excel student import | Slice 1 | done |
| 12 | Demo & load data generator | Slice 1 | done |
| 13 | Financial ledger model | Slice 2 | done |
| 14 | Fee plans & charge generation | Slice 2 | done |
| 15 | Print engine & receipts | Slice 2 | done |
| 16 | Fast cashier & payments | Slice 2 | done |
| 17 | Voids, refunds & adjustments | Slice 2 | done |
| 18 | Admission with initial payment | Slice 2 | done |
| 19 | Dues & due statement | Slice 2 | done |
| 20 | Expenses & daily cash close | Slice 2 | done |
| 21 | Backup & verification | Slice 3 | done |
| 22 | Restore & corruption recovery | Slice 3 | done |
| 23 | Diagnostics & logs | Slice 3 | done |
| 24 | Installer & update pipeline | Slice 3 | done |
| 25 | Attendance | Slice 4 | done |
| 26 | SMS outbox & provider adapters | Slice 4 | done |
| 27 | Message templates & automations | Slice 4 | done |
| 28 | Exams & mark entry | Slice 5 | done |
| 29 | Results, grading & report cards | Slice 5 | done |
| 30 | Reports center & exports | Slice 6 | done |
| 31 | Role dashboards & zero training mode | Slice 6 | done |
| 32 | Command palette | Slice 6 | done |
| 33 | Fast global search | Slice 6 | done |
| 34 | Student 360 timeline | Slice 7 | done |
| 35 | Today operations cockpit | Slice 7 | done |
| 36 | Coaching day timeline | Slice 7 | done |
| 37 | Batch health view | Slice 7 | done |
| 38 | Batch profitability | Slice 7 | done |
| 39 | Smart due collection queue | Slice 7 | done |
| 40 | Receipt verification | Slice 8 | done |
| 41 | Duplicate payment protection | Slice 8 | done |
| 42 | Guardian contact health | Slice 8 | done |
| 43 | Durable inbox/outbox | Slice 8 | done |
| 44 | Offline message queue | Slice 8 | done |
| 45 | Offline resilience center | Slice 8 | done |
| 46 | Disaster recovery score | Slice 8 | done |
| 47 | Backup verification drill | Slice 8 | done |
| 48 | One click PC migration | Slice 8 | done |
| 49 | Database integrity checks | Slice 8 | done |
| 50 | Professional printing system | Slice 9 | done |
| 51 | Bangla + English localization | Slice 9 | done |
| 52 | CSV/Excel export | Slice 9 | done |
| 53 | Encrypted sensitive secrets | Slice 9 | done |
| 54 | AI copilot shell | Slice 10 | done |
| 55 | Cloud AI gateway | Slice 10 | done |
| 56 | AI credit wallet & ledger | Slice 10 | done |
| 57 | AI privacy center | Slice 10 | done |
| 58 | Crash & failure safety suite | Slice 11 | done |
| 59 | Performance on the reference PC | Slice 11 | deferred |
| 60 | Customer docs & support runbook | Slice 11 | in-progress |
| 61 | 1.0 readiness & pilots | Slice 11 | in-progress |
| 62 | Desktop licensing & read only mode | Slice 12 | done |
| 63 | Vendor admin & license issuing | Slice 12 | planned |
| 64 | Activation & refresh service | Slice 12 | planned |
| 65 | Website, downloads & customer portal | Slice 12 | planned |
| 66 | Code signing & release channels | Slice 12 | planned |
| 67 | Pricing & checkout | Slice 12 | planned |
| 68 | Device management | Slice 12 | planned |

## Epics

- [Foundation](foundation.md): features 1 to 8 · 1 in-progress · 7 done
- [Core records](records.md): features 9 to 12 (Slice 1, the walking skeleton) · 4 done
- [Money](money.md): features 13 to 20 (Slice 2) · 8 done
- [Safety & delivery](safety.md): features 21 to 24 (Slice 3, pilot gate) · 4 done
- [Academics & messaging](academics.md): features 25 to 29 (Slices 4 to 5) · 5 done
- [Reports & speed](speed.md): features 30 to 33 (Slice 6) · 4 done
- [Owner intelligence](intelligence.md): features 34 to 39 (Slice 7) · 6 done
- [Operational integrity](integrity.md): features 40 to 49 (Slice 8, finance + communication + offline + data safety) · 10 done
- [Printing & localization](printing.md): features 50 to 53 (Slice 9) · 4 done
- [AI foundation](ai.md): features 54 to 57 (Slice 10, minimal AI for early customers) · 4 done
- [Release hardening](release.md): features 58 to 61 (Slice 11) · 1 done · 2 in-progress · 1 deferred
- [Commercial](commercial.md): features 62 to 68 (Slice 12, parallel from Slice 3) · 1 done · 6 planned

## Deferred

Out of scope for 1.0, kept so the plan stays honest.

**1.1**
- **Performance on the reference PC**: Measure the targets on a low end PC (4 GB RAM, spinning disk) with the large generated dataset and fix what misses (deferred from Slice 11)
- **LAN host & pairing** (Local Pro only): host role, discovery, PAKE pairing, device management, firewall checks · needs a decision · GA
- **LAN workstation client mode**: rpc over pinned TLS, reconnect behavior, update from host · needs a decision · GA
- **Multi PC LAN mode**: reception, accounts, admin, and other PCs work through a local Coachify host · needs a decision · GA
- **Schedule conflict engine**: detect room, teacher, batch, and availability conflicts instantly · needs a decision
- **Substitute teacher workflow**: find available replacement, assign, update schedule, optionally notify students · needs a decision
- **What if batch planner**: simulate batch splitting, capacity changes, costs, room needs, and schedule impact · needs a decision
- **Email channel**: SMTP delivery through the outbox · needs a decision
- **Inquiry & applicant pipeline**: leads before admission
- **ID cards, admit cards & barcode scan**: printed cards, scan for attendance and cashier search · needs a decision
- **Online renewal checkout**: gateway payment in the portal · needs a decision · GA
- **Teacher payment ledger**
- **Per counter cash sessions** · GA
- **Bulk tools**: bulk status and concession changes
- **SMS delivery status polling**
- **Staged update rollout**
- **Morning intelligence brief**: explains today's important changes, issues, and suggested actions
- **Natural language reports**: example "Show Physics A students below 75% attendance with dues"
- **AI communication writer**: draft Bangla/English SMS, email, and notices
- **AI Excel migration**: understand messy column names and suggest mappings/duplicate resolutions
- **Financial anomaly guard**: detect unusual refunds, discounts, duplicate references, and cash discrepancies
- **Smart batch intelligence**: explain unhealthy, overcrowded, underfilled, or financially weak batches
- **Student attention signals**: transparent attendance/result/dues signals rather than opaque risk scores
- **AI schedule assistant**: suggest schedules and explain conflicts/compromises
- **AI result remarks**: draft guardian friendly performance summaries
- **Coachify help assistant**: RAG over Coachify documentation and support guides
- **Local AI option**: optional downloaded AI pack for selected offline AI features · needs a decision
- **Cloud AI option (advanced)**: higher capability AI through Coachify's hosted AI gateway
- **AI audit history**: track feature, user, data category, model/tool usage, and approved actions
- **No direct AI DB access**: AI uses controlled tools instead of arbitrary database access · GA
- **Human approval for consequential AI actions**: AI prepares refunds, discounts, transfers; normal logic performs after confirmation · GA
- **Buy more from desktop**: button opens the authenticated Coachify web account checkout
- **Automatic credit sync**: purchased credits become available to the linked desktop account

**V2**
- **Encrypted off site backup** (paid add on) · needs a decision · GA
- **Managed SMS credits** (paid add on) · needs a decision
- **Multi branch consolidated reporting** · needs a decision
- **Windows service host mode** · needs a decision
- **Guardian payment links** · needs a decision · GA
- **Coachify Account cloud commercial account**: separate from institution local staff accounts
- **Customer portal (full)**: subscription, license, devices, AI credits, invoices, downloads, and support
- **Monthly / longer term plans**: support monthly, 6 month, and yearly commercial plans
- **Grace period**: temporary payment/internet problems don't immediately disable the product
- **Safe expired mode**: historical data, reports, export, backup, and restore remain accessible after expiry
- **Signed auto updater**: secure application updates
- **Stable / beta channels**: controlled rollout before full release
- **Pre update backup**: automatically protect data before upgrading
- **Safe database migrations (updater)**: versioned, recoverable migrations through the updater
- **Offline installer updates**: allow USB/manual installer upgrades while preserving data

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

## References

### Project sources
- `AGENTS.md` and `docs/architecture-review.md`: stack decisions, build approach, LAN seam, feature ordering, and the pilot gate
- Spec 0001 through 0025: recorded decisions for features 1 through 27
- `design.md`: visual language, component primitives, and Bangla typography rules

### Practices & standards
- **Tracer Bullet development** (basis: vertical slices ship real value early, from The Pragmatic Programmer): each feature is built end to end through every layer before the next starts
- **Foundations before features** (basis: the data model is the costliest thing to redo): foundation features land first, then slices build on stable ground
- **Immutable financial records** (basis: append only audit trails for financial systems): posted payments, charges, and allocations cannot be modified, only corrected by new records
- **RBAC with defense in depth** (basis: NIST role based access control): permission checks in the dispatcher, not just hidden UI
- **Offline first architecture** (basis: institution data lives locally): every feature works without internet; connectivity is a bonus
- **Crash safe database** (basis: SQLite WAL mode with synchronous FULL): power cuts during writes leave the database consistent
- **Argon2id password hashing** (basis: IETF RFC 9106, OWASP recommendation): strongest available password hashing for local accounts
- **DPAPI credential storage** (basis: Windows Data Protection API): OS managed encryption for secrets that must not leave the machine

### Links (web verified)
- [The Pragmatic Programmer: 20th Anniversary Edition](https://pragprog.com/titles/tpp20/the-pragmatic-programmer-20th-anniversary-edition/) — Tracer Bullet approach origin
- [SQLite Write-Ahead Logging](https://www.sqlite.org/wal.html) — WAL mode for crash safety
- [SQLite PRAGMA synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous) — synchronous FULL setting
- [Tauri 2.0 Documentation](https://v2.tauri.app/) — desktop framework
- [Windows Data Protection (DPAPI)](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata) — credential storage API
- [RFC 9106: Argon2](https://datatracker.ietf.org/doc/html/rfc9106) — password hashing standard
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) — Argon2id recommendation
- [Local-First Software](https://www.inkandswitch.com/local-first/) — offline first architecture principles (Ink & Switch)
- [NIST Role-Based Access Control](https://csrc.nist.gov/projects/role-based-access-control) — RBAC standard
- [NIST SP 800-92: Guide to Computer Security Log Management](https://csrc.nist.gov/pubs/sp/800/92/final) — immutable audit trail guidance
- [Twilio SMS Character Limits (GSM-7 vs UCS-2)](https://help.twilio.com/articles/223181508-What-are-the-character-limits-for-SMS-and-MMS-messages) — SMS encoding reference
- [3GPP TS 23.038: SMS Character Sets](https://portal.3gpp.org/desktopmodules/Specifications/SpecificationDetails.aspx?specificationId=747) — GSM-7 encoding specification
