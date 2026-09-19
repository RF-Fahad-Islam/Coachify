# Coachify (formerly NF Management Local): Architecture and Product Review

Reviewed 2026-09-20. Inputs: the plan in the `/scope` request and `Implementation_plan.md`.

I reviewed this as if I would own the product for five years and personally answer the support phone. Where I name a specific tool or library, treat it as my recommendation for the `/architect` spec to confirm, not a frozen choice. Every recommendation here is yours to accept or override.

Short version: the strategy is right, the skeleton is right, and the scope is about three times too large for a first sellable version. Five parts are underdesigned in ways that would cause real data loss or real support pain: encryption key recovery, PC clock handling, SMS duplicate sends, the LAN host lifecycle, and migration failure recovery. Everything below either fixes one of those or cuts scope.

---

## 1. Executive Assessment

### Is the standalone strategy sound?

Yes. A desktop app that owns institution data, next to a small cloud that owns only the commercial relationship, fits Bangladesh well: patchy internet, owners who are wary of putting student and money data "somewhere online", and price sensitivity that makes per seat cloud hosting costs painful. It also makes the business robust. If your cloud goes down for a week, no customer stops collecting fees.

### Biggest strengths

1. **Clean ownership split.** Desktop owns operations, cloud owns commerce. This is the most important decision and it is correct.
2. **Offline verifiable signed licenses.** Correct and cheap to run.
3. **Durable outbox for SMS.** Correct. A payment never waits on an SMS provider.
4. **A real ledger instead of a mutable `paid_total`.** Correct, and it is what makes owners trust the numbers.
5. **Backups treated as a product feature with verification.** Correct, and rare in this market.
6. **No SQLite on a network share.** Correct. That single rule prevents the most common corruption story in small office software.

### Biggest risks, ranked

1. **Scope.** `Implementation_plan.md` has 75 sections and reads like a three year ERP. A small team building all of it before selling anything will run out of money or patience first. Section 13 cuts it hard. I also recommend shipping 1.0 as single PC and LAN as 1.1, with the LAN seam built from day one (Section 5).
2. **Encryption key loss means total data loss.** SQLCipher with a key tied to one Windows machine, and backups encrypted with the same key, means that when the PC dies (the most common disaster you will see) the backups cannot be opened on the new PC. The plan does not mention this. It needs a printed recovery key (Section 11).
3. **The LAN host is "a desktop app somebody can close".** Add Windows sleep, Windows Update restarts, firewall prompts, networks marked Public, DHCP changing the host IP, and students on the same WiFi as the office PCs. This is where most of your support calls will come from if it ships unhardened.
4. **Printing is the riskiest technical bet in the stack.** Silent printing to a chosen printer with a chosen paper size is not a built in Tauri feature. It is solvable through the WebView2 print API, but it must be proven in a spike before any feature work (Section 2).
5. **Commercial assumptions imported from card based SaaS.** In Bangladesh most renewals will be bKash, Nagad, bank transfer, or cash, done by hand. Card autopay is rare, and Stripe does not serve Bangladeshi merchants. So `past_due`, automatic renewal, coupons, and self serve checkout matter little at launch, while manual payment recording and fast license issuance matter a lot. Also, gating SMS behind the higher tier will hurt sales, because SMS to guardians is the feature owners notice first.
6. **Real site conditions the plan does not design for:** power cuts without a UPS (load shedding), dead CMOS batteries that reset the PC clock to an old year, shared Windows accounts without passwords, local technicians who reformat drive C when "Windows is slow", aggressive or pirated antivirus, SmartScreen warnings on a new code signing certificate, and 4 GB RAM PCs with spinning hard disks.
7. **Leftover NF coupling.** `Implementation_plan.md` still mentions NF Cloud sync, NF Connect, NFStudio, and Porokh integration (its sections 1, 42, 68). Remove all of them. "Extension points" for them are exactly how coupling creeps back in. If a connected service ever exists, it will be designed then, as a new optional product capability.

### Would I install it tomorrow?

Not as written. With the changes below, yes: a single PC 1.0 covering students, fees and cashier, attendance, exams and results, SMS, backups, licensing, and updates, proven at three pilot sites for a month each, is something I would install and support.

---

## 2. Architecture Critique

| Decision | Verdict | Why | What to do |
|---|---|---|---|
| Tauri 2 shell | **KEEP** (after spike) | WebView2 is Chromium on Windows, so rendering and Bangla text shaping match Chrome. Small installer, low memory, Rust backend in the same process. | Phase 0 spike must prove silent printing, PDF, installer with offline WebView2, and updater. If printing fails, switch to Electron before writing features. |
| React + TypeScript + Vite + Tailwind | **KEEP** | Largest ecosystem, best AI agent output quality, fastest UI iteration. | No business logic in React. Generate TypeScript types from Rust so the contract cannot drift. |
| React Hook Form + Zod | **KEEP** | Good form UX. | Zod is for friendly messages only. Rust validation is the authority. |
| TanStack Query | **KEEP** | The right cache for command results, with invalidation on change events. | Treat it as a cache of server state, never as the source of truth. |
| Rust native backend | **KEEP**, restructure | Strong typing for money logic, single binary, easy SQLCipher bundling. | Put all domain logic in a `core` crate with no Tauri dependency, so it is testable headless and can later run as a Windows service. |
| SQLite | **KEEP**, change config | Right size for one institution. | WAL plus `synchronous=FULL` (power cuts), one writer, `BEGIN IMMEDIATE`, `foreign_keys=ON`, triggers that make financial rows immutable. |
| SQLCipher | **KEEP**, redesign keys | Protects copied database files and backups. Does not protect against anyone who can use the PC. | Random data key, wrapped by Windows DPAPI for daily use and by a printed owner recovery key for disaster recovery. Be honest in marketing about what it protects. |
| FTS5 search | **KEEP** | Fast enough for this size. | Use the trigram tokenizer for names, IDs and phone numbers (substring matching). Verify Bangla in the spike. |
| OS secure credential storage | **KEEP**, adjust | Offices share Windows accounts. | Use DPAPI machine scope so any Windows user on that PC can run the app and nightly jobs. |
| Local filesystem documents | **KEEP**, adjust | Correct. | Content addressed vault (file named by its hash), downscale photos on import, never trust names from users. |
| LAN host (Mode B) | **CHANGE** and **DEFER to 1.1** | Right idea, underdesigned, and roughly a third of the hardening effort. | One command dispatcher from day one, so LAN is only a transport. Host runs in the app with tray and no sleep. HTTPS JSON with pinned certificate, PAKE pairing, exact version match, host serves updates to workstations. |
| gRPC | **REMOVE** | No streaming or polyglot need. Adds a code generation toolchain and makes debugging harder. | JSON over HTTPS plus one WebSocket for change events. |
| Commercial backend | **KEEP**, shrink | Right boundary. Too many features for launch. | 1.0: admin panel, manual payments, license issuing, activation and refresh endpoint, releases and download page. Self serve checkout in 1.1. Remove coupons and the ticket system. |
| Licensing | **KEEP**, change model | Signed offline licenses are right. Device counting per PC is the wrong unit. | License the host installation. The host enforces the workstation count. 14 day grace. Renewals continue from the paid through date. |
| Billing states | **CHANGE** | `past_due` assumes card autopay. | Prepaid periods only: trial, active, grace, expired, suspended. |
| Updates (Tauri updater) | **KEEP**, add safety | Correct base. | Migrate a copy then swap, keep the previous installer, a blocked versions list, host mirrors updates to workstations. |
| Backups | **KEEP**, redesign | Right intent. A daily full archive with all photos does not scale and gets skipped. | Small daily snapshot plus incremental document vault, hourly local snapshots in business hours, weekly automatic restore drill, recovery key. |
| SMS via customer credentials | **KEEP**, change retry | A timed out SMS request may have been delivered. Auto retry sends duplicates, and most local providers have no idempotency keys. | Never auto retry an uncertain send. Quiet hours. Absence SMS delayed so teachers can correct mistakes. |
| Email | **DEFER to 1.1** | Guardians in this market rarely read email. SMS is what matters. | Keep the channel abstraction. Ship SMTP in 1.1. |
| Automations | **CHANGE** | No event bus needed. | Enqueue the message inside the same transaction as the business write, plus one small scheduler table for time based rules. |
| Printing | **KEEP** HTML templates, change engine | Rust PDF libraries shape Bangla poorly. | Render templates in a hidden WebView2 window, print through the WebView2 print API, make PDF through WebView2 print to PDF. Driver based thermal printing. Bundle fonts. |
| Plans Local / Local Pro | **CHANGE** | SMS and permissions are poor paywalls. | Tier by institution size (active students) with workstations included. Section 14. |
| Invoices + invoice items | **CHANGE** | Coaching centers think in "September fee due", not invoices. | Charges, payments, allocations, adjustments, refunds. Due statement is a printed view, not a stored invoice. |
| Expired means read only | **KEEP** | Fair, never holds data hostage. | Exports, backups, restore, receipt reprint always allowed. |
| Monorepo (Option A) | **CHANGE** | Nothing real to share between a Rust desktop and a TypeScript cloud. | Two repositories. Section 18. |
| QR / barcode attendance | **DEFER to 1.1** | Needs printed ID cards first. | A USB scanner types like a keyboard, so it is cheap once ID cards exist. |
| Telemetry | **CHANGE** | Privacy is the selling point. | Nothing automatic. Opt in diagnostics you review before sending. |

### Stack comparison

Scores are for this product (Windows only, heavy printing, Bangla, SQLite, LAN host, small team with AI agents). 5 is best.

| Criterion | Tauri 2 + React + Rust | Electron + React + Node | .NET WPF | WinUI 3 | Flutter desktop | Avalonia | Local server + browser |
|---|---|---|---|---|---|---|---|
| Development speed | 4 | 5 | 3 | 2 | 3 | 3 | 4 |
| Windows reliability | 4 | 5 | 5 | 3 | 3 | 4 | 3 |
| Installer quality | 4 | 4 | 4 | 2 | 3 | 3 | 2 |
| Auto update | 4 | 5 | 3 | 2 | 2 | 3 | 5 (host only) |
| Printing control | 3 (needs WebView2 API work) | 5 | 5 | 3 | 2 | 3 | 1 (browser dialogs) |
| Bangla text in print and PDF | 5 | 5 | 4 | 4 | 2 | 3 | 5 |
| SQLite + SQLCipher | 5 | 4 (native module rebuilds) | 3 (community or paid builds) | 3 | 3 | 3 | 5 |
| LAN hosting in process | 5 | 4 | 5 | 3 | 2 | 4 | 5 |
| Memory on a 4 GB PC | 5 | 2 | 4 | 4 | 4 | 4 | 3 |
| Long term maintenance | 4 | 4 | 5 | 2 | 3 | 3 | 3 |
| AI agent friendliness | 4 | 5 | 3 | 2 | 3 | 2 | 4 |

Notes that decide it:

- **Electron** is the safest fallback. Its printing API (silent print to a named printer with page size) is the best of the web stacks. The cost is memory (a real concern on 4 GB workstations running a browser too), a bigger installer, and native SQLite modules that must be rebuilt on every Electron upgrade.
- **.NET WPF** is the strongest pure Windows choice for printing and reliability, but a modern SaaS look costs more work, SQLCipher on .NET means a community bundle or a commercial license, and AI agents write noticeably weaker XAML than React.
- **WinUI 3** has packaging and deployment friction for a small team. Reject.
- **Flutter desktop** is weak at complex script shaping in generated PDFs, which is fatal for Bangla receipts and marksheets. Reject.
- **Avalonia** is good engineering with a small ecosystem and few agent examples. Reject.
- **Local server + browser** is tempting because workstations would need no install, but you lose silent receipt printing, users get certificate warnings on the LAN, staff close the tab, and there is no tray or autostart. Reject as the main architecture.

**Final recommendation: Tauri 2 + React + TypeScript + Rust + SQLite with SQLCipher**, on the condition that the Phase 0 spike proves these five things on a clean Windows 10 PC:

1. Silent print to a named printer with A4, A5, and 80 mm paper, margins, and copies, through the WebView2 print API called from Rust.
2. PDF export through WebView2 with correct Bangla shaping.
3. SQLCipher plus FTS5 with the trigram tokenizer, built in Windows CI.
4. The NSIS installer installs on a PC with no WebView2 and no internet (offline WebView2 installer mode).
5. The updater downloads, verifies, and installs a signed update.

If item 1 cannot be made to work within the spike, switch to Electron with the same React UI and a Node core. That decision costs little in week 2 and a lot in month 6.

**Windows support:** Windows 10 (1809 or later) and Windows 11, 64 bit only. Windows 7 and 8 are out for every modern stack (WebView2 and Electron both dropped them). Check your pilot market for 32 bit Windows 10 before you commit; supporting it doubles the test matrix.

---

## 3. Final Recommended Architecture

```
CUSTOMER SITE (works with no internet)

 ┌──────────────────────────── Host PC (or the only PC) ─────────────────────────────┐
 │  React UI  (screens, forms, print templates)                                      │
 │     │  one typed IPC entry: rpc(command)                                          │
 │  Rust app shell  (windows, tray, printing via WebView2, updater, OS integration)  │
 │     │                                                                             │
 │  Rust core  (no UI dependency)                                                    │
 │     dispatcher: session → permission → license gate → transaction → audit        │
 │     ├─ SQLite + SQLCipher  (one writer, WAL, synchronous FULL)                    │
 │     ├─ document vault  (content addressed files)                                  │
 │     ├─ job runner  (outbox sender, backups, verify, fee generation, reminders)    │
 │     ├─ license + clock guard                                                      │
 │     └─ LAN server  (1.1): HTTPS JSON rpc + WebSocket events + update mirror       │
 └───────────────────────────────────────────────────────────────────────────────────┘
          ▲ pinned TLS · device token · user session
 ┌──── Workstation PC (1.1) ────┐
 │ same app in client mode:     │   no local database, no offline writes
 │ UI → rpc → forwarder → host  │   prints locally on its own printers
 └──────────────────────────────┘

 Backup targets: local folder · second drive · USB · NAS path
 Internet, only when present: SMS provider · license refresh · update check

VENDOR CLOUD (never receives institution data)

 Web app: website · customer portal · admin panel (admin behind MFA)
 License signer: isolated service, only holder of the private key, logs every signature
 Postgres: customers · plans · prices · subscriptions · payments · licenses · installations · releases
 Object storage + CDN: installers · update manifests per channel
 CI: tag → tests → build → Windows code signing → updater signing → internal → beta → stable
```

### Source of truth

| Data | Owner | Copies allowed |
|---|---|---|
| Students, guardians, money, attendance, exams, users, audit | Local database on the host | Backups only |
| Documents and photos | Local vault on the host | Backups only |
| Customers, subscriptions, vendor payments | Cloud database | None |
| Entitlement (what this install may do, until when) | The signed license file | Cached inside the local database; cloud issues new ones |
| Releases and update metadata | Object storage, written only by CI | CDN cache |
| Printer choice | Each PC's local config | None (printers are per PC) |

The cloud never holds a mutable copy of institution data. The desktop never holds a mutable copy of subscription state, only the signed license, which it cannot change.

---

## 4. Desktop Architecture

### Boundaries

- **React UI** renders screens, holds form state, and renders print templates. It calls exactly one function: a generated typed client that sends `rpc(command, payload, idempotency_key)`. It never sees file paths, SQL, secrets, or license internals.
- **Rust app shell** (the Tauri crate) handles windows, tray, autostart, printing through WebView2, file pickers, the updater, and passes `rpc` calls to the core. Keep it thin.
- **Rust core** (a plain library crate) holds all domain logic, the database, migrations, permissions, audit, jobs, backups, licensing checks, and the LAN server later. All tests for business rules run against the core with no UI.

Using one `rpc` entry point (rather than hundreds of separate Tauri commands) means the same request format travels over IPC today and over the LAN tomorrow, through the same permission and audit code. That is the whole trick that makes LAN mode affordable.

### What every command does, in order

1. Resolve the session into a context: user, roles, permissions, device, business date.
2. Check the permission the command declares (`payments.collect`, and so on).
3. Check the license gate the command declares (`write`, `sms`, `lan`). Read only mode blocks every `write` command here, in one place.
4. Validate input.
5. `BEGIN IMMEDIATE`, do the domain work, write audit rows, write outbox rows if a message is triggered, `COMMIT`.
6. Publish a change event (`students changed: [ids]`) so open screens refresh.

### SQLite access

- One writer connection owned by a writer thread (commands queue to it), plus a small pool of reader connections. SQLite allows one writer at a time anyway, and a single owner removes "database is locked" errors from normal operation.
- Pragmas: `journal_mode=WAL`, `synchronous=FULL` (a committed payment must survive a power cut, and load shedding is common), `foreign_keys=ON`, `busy_timeout=5000`, `temp_store=MEMORY`.
- Money is stored as integer poisha (`i64`). Never floating point.
- Time: store the UTC instant and a separate `business_date` (the local calendar date the office considers the transaction to belong to). Reports group by `business_date`. On LAN, the host clock stamps everything.
- IDs: UUIDv7 as text. Human facing numbers (student ID, receipt number) are separate columns from counters.
- Startup runs `PRAGMA quick_check`; backup verification runs the full `integrity_check`.

### Filesystem

```
<Data folder>\<institution_id>\
    db\main.db            (SQLCipher, plus WAL files)
    vault\ab\abcdef...    (documents and photos, named by SHA256)
    snapshots\            (hourly local snapshots, last 24)
    logs\                 (JSON lines, rotated, redacted)
    staging\              (temporary work for imports, backups, migrations)
%ProgramData%\<Product>\config.json   (data folder location, mode, host address; no secrets)
```

Install per machine, and let the setup wizard **recommend a data folder on a non system drive** (for example `D:\<Product> Data`) when one exists. Local technicians routinely reformat drive C, and data under `C:\ProgramData` disappears with it. The installer grants the Users group modify rights on the data folder, because offices share Windows accounts.

### Printing

- Print templates are React components with print CSS (`@page` sizes for A4, A5, and 80 mm).
- The shell renders a template in a hidden window, then calls the WebView2 print API with the chosen printer, paper, margins, and copies. Receipts print silently to the configured receipt printer; other documents show a preview first.
- PDF export uses WebView2 print to PDF, so the PDF matches the printed page exactly, including Bangla.
- Thermal 80 mm printers print through their Windows driver as a normal page (72 mm printable width). Do not send raw ESC/POS text, because it cannot render Bangla. Cash drawer kick can come later.
- Bundle open licensed Bangla and Latin fonts. Never depend on whatever fonts a PC happens to have.
- Printer settings are stored **per PC**, not in the institution database. Each workstation prints on its own printers.
- Offer an "A4 with two copies (student copy and office copy)" receipt layout, which is how many offices in this market work.

### Local jobs

One `jobs` table (kind, run_at, status, attempts, lease_until, last_error) and one runner thread on the host. Every job is idempotent, and on startup any job whose lease expired is picked up again. Jobs: send outbox, scheduled backup, verify backup, weekly restore drill, monthly fee generation, due reminders, license refresh, update check, database maintenance. No queue library, no event bus.

### Messaging outbox

- The domain command that causes a message (a payment, an absence) inserts the outbox row **in the same transaction**. The message exists if and only if the payment exists.
- Each message has a `dedupe_key` with a unique constraint (for example `payment:<id>:guardian_sms`), so a retried command never creates a second message.
- Outbox statuses: `queued`, `sending`, `sent`, `failed`, `uncertain`, `cancelled`. Drop `draft` (that is compose screen state), `waiting_for_network` (that is sender state, not message state), and `delivered` (most local providers give no delivery report you can receive on a desktop; add it later only for adapters that support polling).
- **Retry rules:** retry automatically only when the request clearly never reached the provider (DNS failure, connection refused, provider says "try again"). If the request was sent and the response was lost (timeout), mark `uncertain` and **do not retry automatically**; staff can resend by hand. Duplicate SMS to a guardian costs the customer money and trust.
- Backoff: 1, 5, 15, 60 minutes, then every 6 hours up to 24 hours, then `failed`.
- Quiet hours (default 9 pm to 8 am): messages wait.
- Absence SMS waits 30 minutes (configurable) after attendance is saved and is cancelled if the teacher corrects the mark.
- Only the host sends. Workstations only enqueue through commands.

### Security inside the app

- Tauri capabilities: expose only the `rpc` command and the print and dialog functions the shell needs. No general file system or shell plugins available to the UI.
- Strict content security policy, no remote content, devtools disabled in release builds.
- Files are referenced by vault ID. The UI never passes a path into the core.
- Secrets (SMS token, SMTP password) are write only fields. The UI can set them but never read them back.
- Passwords hashed with Argon2id. Five failed logins lock the account for five minutes. Idle lock after 15 minutes (configurable).
- Logs pass through a redaction layer before writing. Phone numbers are masked in logs; tokens and passwords never appear.

---

## 5. LAN Architecture (ships in 1.1, seam built in 1.0)

**Recommendation: sell 1.0 as single PC and ship LAN as 1.1**, but build the single `rpc` dispatcher in Phase 1 so every feature is LAN ready by construction. When LAN work starts, it adds a transport and the pairing flow, not a rewrite. (This is decision 2 in Section 20; override it if multi PC is what you must sell first.)

### Host

- The host role runs inside the desktop app on the main PC. No separate server install.
- When hosting is on: closing the window hides it to the tray; quitting while workstations are connected asks for confirmation; the app starts when Windows starts; the app keeps Windows from sleeping while hosting; the LAN guide explains Windows Update active hours.
- The host TLS key and host ID are stored in the (encrypted) database. When the host PC is replaced and a backup restored, workstations reconnect without pairing again.
- A Windows service mode is a V2 option if field data shows people closing the app is a real problem. Keeping the core free of UI dependencies keeps that door open.

### Transport and API

- HTTPS (TLS 1.3) on one fixed default port, configurable.
- `POST /rpc`: the same command envelope as local IPC.
- `GET /events`: one WebSocket per workstation for change notifications (topic plus IDs). On reconnect, the client refetches everything visible.
- `GET /updates/...`: the host serves its own signed installer and update metadata to workstations.
- `GET /hello`: unauthenticated, returns only product, protocol version, host ID. Institution name only while a pairing window is open.
- Why not gRPC: no streaming need, adds tooling, harder to debug on a customer PC. Why not plain HTTP: students on the office WiFi.

### Discovery

- mDNS service advertisement with the host ID and protocol version.
- Manual fallback: type an IP address or PC name.
- Clients remember the host ID and last address. If the address stops working (DHCP gave the host a new IP), they rediscover by host ID. The guide suggests a DHCP reservation but nothing requires it.

### Pairing (improved)

1. On the host, an admin with `devices.manage` clicks **Pair new device**. A pairing window opens for 10 minutes and shows an 8 digit code.
2. On the workstation: **Join existing institution** → discovered hosts listed → **Connect** → type the code.
3. Both sides run a password authenticated key exchange (SPAKE2) over the code, and confirm the host certificate fingerprint and the new device key inside it. A person in the middle cannot learn the code offline or swap certificates. Five wrong attempts close the window.
4. The host registers the device (name, Windows PC name, first seen), writes an audit entry, and issues a 256 bit device token. The workstation stores it with DPAPI.
5. The workstation shows the normal login screen for that institution.

Drop QR pairing: office workstations have no camera. Keep it only if a phone or tablet client ever exists.

### Authentication on every request

- Client verifies the pinned host certificate.
- Request carries the device token (host stores only its hash) and a user session token (issued at login, bound to that device, expires on idle).
- The host builds the same context as local IPC and runs the same dispatcher. Permissions are always decided on the host.
- Revoked or blocked device: every request fails with a clear "This PC was removed from the institution" screen.
- Rate limits on login and pairing.

### Concurrency

- One writer on the host. Every write is serialized, so there is no lost write at the database level.
- Editable records (student profile, batch) carry `row_version`. A stale save returns "changed by Rahim at 10:42, review and save again".
- Every create command carries an idempotency key generated when the form opens. A double click or a retry after a lost connection returns the original result, not a second record.
- Payments carry the due the cashier saw. If it changed (another cashier just took money for the same fee), the host refuses and the screen refreshes with the new due.
- Receipt numbers, student IDs, and timestamps are assigned only by the host.

### Host restart and connection loss

- Workstation states: connected, reconnecting, host unreachable, removed.
- While not connected: views stay visible but marked stale, all save buttons are disabled, and typed form data stays. When the host is back, the user clicks save again with the same idempotency key.
- **No offline writes on workstations.** "Works offline" means without the internet, not without the host. Building a sync and merge engine for workstations would be the largest and riskiest code in the product for little value.

### Version compatibility

- Each build has a `protocol_version` integer, exchanged at connect.
- 1.1 policy: **the workstation must run the same app version as the host.** When they differ, the workstation offers a one click update downloaded from the host (no internet needed), verified by the updater signature, and refuses to downgrade unless an admin confirms.
- This removes the need to test old clients against new hosts. Loosen it to "one protocol version back" only if support data shows a real need.

### Firewall and network profile

- The installer adds an inbound firewall rule for the app on Private networks only.
- Diagnostics detects when the host's network is marked Public (common) and walks the user through changing it.
- Diagnostics detects WiFi client isolation (workstations cannot reach the host at all) and says so plainly. The LAN guide recommends a wired host.

### Limits

Test with 10 workstations; allow up to 25.

---

## 6. Database Design

### Conventions

- Primary keys `id` UUIDv7 text. `created_at`, `created_by`, `updated_at`, `row_version` on editable tables.
- Money: integer poisha. Percentages: integer basis points.
- Posted financial rows are never updated or deleted. Corrections are new rows. SQLite triggers `RAISE(ABORT)` on forbidden updates and deletes, so a bug or a curious admin with a database tool cannot silently rewrite history.
- Status fields have `CHECK` constraints. Uniqueness is enforced by the database, not by code.

### Institution, users, access

| Table | Key columns and rules |
|---|---|
| `institution` | single row: id, name, name_bn, logo_vault_id, address, phones, currency, timezone, id formats, fiscal_year_start |
| `academic_sessions` | id, name ("2026"), start_date, end_date, is_current |
| `users` | id, username unique, display_name, password_hash, status (active, disabled), must_change_password, staff_profile_id nullable |
| `roles` | id, name unique, is_system (Owner cannot be deleted) |
| `permissions` | code primary key (`payments.collect`), description; seeded by migrations |
| `role_permissions` | role_id, permission_code, primary key both |
| `user_roles` | user_id, role_id |
| `staff` | id, name, phone, designation, joining_date, status, notes |
| `sessions` | id, user_id, device_id, created_at, last_seen_at, expires_at |
| `devices` | id, name, windows_name, token_hash, status (active, blocked, revoked), paired_at, last_seen_at, app_version (1.1) |
| `settings` | key primary key, value_json, updated_by (typed keys defined in code) |

Rules: at least one active Owner must always exist (checked in the command). Changes to roles, permissions, and users are audited.

### People

| Table | Key columns and rules |
|---|---|
| `students` | id, student_code unique, name, name_bn, gender, dob, phone_norm, email, address, school, photo_vault_id, status (active, inactive, completed, transferred, archived), admission_date, custom_json, notes, row_version |
| `guardians` | id, name, relationship, phone_norm, phone2_norm, email, address, notes |
| `student_guardians` | student_id, guardian_id, is_primary, receives_sms |
| `documents` | id, owner_type, owner_id, kind, original_name, mime, size, sha256, uploaded_by |
| `custom_field_defs` | id, entity, key, label, type, options_json, required |
| `search_index` | FTS5 table (trigram) over student code, names, phones, guardian names and phones, roll; kept current by triggers |

Phones are normalized to one format (`8801XXXXXXXXX`) on write, so a search for "017..." or "+88017..." finds the same person.

### Academics structure

| Table | Key columns and rules |
|---|---|
| `programs` | id, name, session_id, status |
| `subjects` | id, name, code |
| `batches` | id, program_id, name, code unique per session, teacher_staff_id, room, schedule_json, capacity, start_date, end_date, status |
| `batch_subjects` | batch_id, subject_id, teacher_staff_id |
| `enrollments` | id, student_id, batch_id, start_date, end_date nullable, status (active, ended), end_reason (completed, transferred, left), roll; unique active enrollment per student and batch |

A transfer ends one enrollment and starts another in one transaction. Nothing is overwritten.

### Attendance

| Table | Key columns and rules |
|---|---|
| `attendance_sessions` | id, batch_id, date, taken_by, taken_at, locked; unique (batch_id, date) |
| `attendance_marks` | session_id, student_id, status (present, absent, late, excused), note; primary key (session_id, student_id) |

A separate session row means "attendance not taken" is different from "everyone absent". Edits after the lock time need `attendance.edit_previous` and are audited with the old value.

### Fees and money (the part that must be right)

| Table | Key columns and rules |
|---|---|
| `fee_heads` | id, name, name_bn, kind (tuition, admission, exam, registration, material, fine, other) |
| `fee_plans` | id, program_id or batch_id, fee_head_id, amount, frequency (once, monthly), effective_from, effective_to |
| `concessions` | id, student_id, fee_head_id nullable (all), type (percent, fixed), value, from_period, to_period, reason, approved_by |
| `charges` | id, student_id, enrollment_id, fee_head_id, period (`2026-09`, nullable for one time), gross, concession, net, due_date, source (generated, manual), status (open, voided), void_reason; **unique (enrollment_id, fee_head_id, period)** for generated charges |
| `payments` | id, receipt_no **unique**, student_id, amount, method (cash, bkash, nagad, bank, card, other), reference, business_date, received_at, collector_user_id, device_id, **idempotency_key unique**, status (posted, voided), void_reason, voided_by |
| `payment_allocations` | id, payment_id, charge_id, amount; created in the payment transaction or when credit is applied |
| `adjustments` | id, student_id, charge_id nullable, type (waiver, fine, correction), amount (signed), reason required, approved_by, business_date |
| `refunds` | id, payment_id, amount, method, reason, business_date, refunded_by |
| `cash_days` | business_date primary key, opening_cash, cash_in, cash_out, expected_closing, counted_closing, difference, note, closed_by, closed_at |
| `expense_categories` | id, name |
| `expenses` | id, category_id, amount, method, payee, reference, business_date, attachment_vault_id, created_by, status (posted, voided), void_reason |
| `sequences` | name primary key (`receipt:2026`), prefix, next_value |
| `student_balances` | student_id primary key, total_charged, total_paid, total_adjusted, total_refunded, balance; **a cache**, updated in the same transaction, checked nightly |

**Invariants (tested on every release and checked nightly on customer PCs):**

1. For each payment: sum of allocations ≤ amount minus refunds.
2. For each charge: sum of allocations ≤ net plus fine adjustments minus waivers on that charge.
3. Student balance = open charges net + fines − waivers − posted payments + refunds, and equals the `student_balances` cache.
4. Receipt numbers are unique, and every number from a sequence exists as a posted or voided payment (no gaps without a record).
5. No payment, expense, refund, or void carries a `business_date` of a closed cash day.
6. Daily collection report total = sum of posted payments for that date = cash day figures for cash.

**Rules that make it auditable:**

- A charge's amount never changes once issued. Mistakes are fixed with a waiver or correction adjustment (reason required, permission checked, audited), or by voiding a charge with no allocations.
- A payment is never deleted. A same day mistake is voided (reason required, `payments.void` permission, only while the day is open). Money given back later is a refund on the day it is given back.
- Once a cash day is closed, nothing can be dated into it. Corrections go into today.
- Advance payment: the unallocated part of a payment stays as credit. When new charges are generated, credit is allocated oldest first in the same transaction. The cashier can override allocation order.
- Monthly charges are generated by an idempotent job for each active enrollment with a monthly plan. The unique key makes double generation impossible, even if the job runs twice or two PCs trigger it.
- Receipt numbers come from `sequences` inside the payment transaction. One writer means no collisions.

**Receive payment, as one transaction:**
check idempotency key (return the existing receipt if found) → check permission and license gate → verify the due the cashier saw still matches → allocate the next receipt number → insert payment → insert allocations → update balance cache → insert audit row → insert outbox row if the payment SMS rule is on → commit → print.

### Exams and results

| Table | Key columns and rules |
|---|---|
| `grading_schemes` | id, name; `grade_bands` (scheme_id, min_percent, grade, grade_point, is_fail) |
| `exams` | id, name, session_id, type, start_date, grading_scheme_id, status (draft, entry, verified, published) |
| `exam_batches` | exam_id, batch_id |
| `exam_subjects` | id, exam_id, subject_id, full_marks, pass_marks, date |
| `marks` | exam_subject_id, student_id, marks (integer hundredths), is_absent, status (draft, verified), entered_by, verified_by; primary key both; CHECK marks ≤ full_marks unless allowed |
| `result_snapshots` | id, exam_id, version, published_at, published_by, scheme_json (frozen copy) |
| `results` | snapshot_id, student_id, total, percent, grade, gpa, passed, rank |

Publishing freezes results and the grading rules used. Later edits to the scheme do not change published results; republishing creates a new version.

### Communications

| Table | Key columns and rules |
|---|---|
| `message_templates` | id, key, channel, language, body, is_system |
| `automation_rules` | id, trigger (payment_received, absent, due_reminder, exam_scheduled, result_published), enabled, template_id, options_json |
| `outbox` | id, channel, to_address, body, segments, template_key, related_type, related_id, **dedupe_key unique**, status, scheduled_at, attempts, last_error, provider_message_id, created_by |
| `delivery_attempts` | id, outbox_id, attempted_at, outcome (accepted, rejected, network_error, timeout), http_status, response_excerpt |
| `provider_configs` | id, channel, adapter, config_json (no secrets), secret_ref |

### Audit, backups, system

| Table | Key columns and rules |
|---|---|
| `audit_log` | seq integer primary key, at, user_id, device_id, action, entity, entity_id, before_json, after_json (sensitive fields removed), prev_hash, hash; insert only by trigger |
| `backup_runs` | id, kind (scheduled, manual, pre_update, pre_restore, hourly), destination, file, size, sha256, db_schema_version, app_version, created_at, verified_at, verify_result |
| `backup_destinations` | id, path, kind (local, usb, network), last_ok_at |
| `license_state` | single row: license_blob, installation_id, last_refresh_at, last_refresh_result |
| `clock_guard` | single row: high_water_at, last_ack_by, last_ack_at |
| `schema_migrations` | version, name, applied_at, app_version, checksum |
| `jobs` | id, kind, run_at, status, attempts, lease_until, last_error, payload_json |

The audit log is a hash chain (each row includes the hash of the previous row). The chain head is written into every backup manifest, so any rewrite of history after a backup is detectable. Cheap to build, and a real selling point to owners worried about staff fraud.

---

## 7. Commercial Backend Architecture

### Stack

Next.js plus managed Postgres is appropriate. Keep it boring:

- **One web app** for the website, customer portal, and admin panel. Admin lives under its own route group with separate admin accounts, required TOTP two factor login, and every action audited.
- **One isolated license signer.** A small separate function or service is the only thing holding the private signing key. The web app calls it with a narrow request ("sign this payload for license X") using its own service credential. It logs every signature. A bug or leak in the website cannot leak the key.
- Supabase Cloud is fine as managed Postgres and portal auth. Avoid building on vendor specific features you would struggle to leave (keep business logic in your own code, not in database functions tied to one provider).
- Object storage plus CDN for installers and update metadata. CI is the only writer.

### Main tables

| Table | Purpose |
|---|---|
| `customers` | organization name, owner name, phone, email, district, notes, status |
| `customer_users` | portal logins (email, password or magic link). Not related to local institution users. |
| `plans` | code, name, tier limits (active students, workstations), capabilities |
| `prices` | plan, period (1, 6, 12 months), currency BDT, amount, active_from, active_to |
| `subscriptions` | customer, plan, status, paid_through, grace_days, started_at, cancelled_at |
| `payments` | customer, subscription, amount, method (bkash, nagad, bank, cash, gateway), reference, proof file, recorded_by, status (confirmed, refunded) |
| `vendor_invoices` | numbered invoices for your own accounting and VAT |
| `licenses` | license_id, customer, subscription, current serial, bound installation, kid, last issued blob, status |
| `license_issues` | every issued license file (serial, payload, issued_by, reason) |
| `installations` | installation_id, customer, license, label, role (host), first_seen, last_seen, app_version, status (active, moved, revoked) |
| `activation_requests` | offline request codes and what was issued for them |
| `revocations` | license_id, reason, at; exported as a signed list |
| `releases` | version, channel, notes, artifact URLs, sha256, updater signature, min_upgrade_from, status (live, held, blocked) |
| `admin_users`, `admin_audit_log` | your staff and everything they did |

Removed from launch: coupons (hand out discounts by recording a lower price manually), support tickets (use WhatsApp Business and a shared helpdesk inbox; your customers will phone you anyway).

### Desktop facing API (small, versioned, rate limited)

- `POST /v1/activate` with license key and installation ID → a bound license, or "already active on another PC, transfer?"
- `POST /v1/refresh` with license ID, installation ID, current serial, app version, active student count → newer license, "no change", "moved", or "revoked" (signed)
- `GET /v1/updates/{channel}` → update metadata
- Everything a desktop sends is listed in the privacy page. No student data, ever.

### Customer portal (1.0 minimal)

Login, license status and download, installer downloads, vendor invoices, installations list with "this PC was replaced". Online renew button in 1.1.

---

## 8. Billing Model

**Recommendation: prepaid periods, no automatic renewal.** It matches how customers here pay and removes a whole class of edge cases.

### States

| State | Meaning | Desktop behavior |
|---|---|---|
| `trial` | 30 days from institution creation, created locally, no internet needed | Full features, banner with days left |
| `active` | today ≤ paid_through | Full features |
| `grace` | paid_through < today ≤ paid_through + 14 days | Full features, strong daily renewal banner |
| `expired` | after grace | Read only mode (below) |
| `suspended` | vendor action for fraud or chargeback, delivered at next online refresh | Same as expired |

`cancelled` is not a separate desktop state: a customer who cancels simply does not renew and reaches `expired` at the end of what they paid for. Drop `past_due`; there is no autopay for anything to be past due on.

### Rules

- **Periods:** 1, 6, and 12 months. Price 12 months attractively. Monthly plans work well only for customers who are online at least monthly (license refreshes by itself); for offline customers, steer them to 6 or 12 months, because monthly file imports create support work every month.
- **Renewal date:** if renewed before the grace period ends, the new period starts at the old `paid_through` (grace gives no free days). If renewed after expiry, the new period starts on the payment date.
- **Grace is 14 days, not 7.** A 7 day grace colliding with Eid holidays, a slow bank transfer, or your staff being unavailable on a weekend turns into an angry call from a center that cannot collect fees. Warnings start 30 days before the end date.
- **Manual payments (the main path at launch):** customer pays by bKash, Nagad, bank, or cash → your admin records it with reference and proof → paid_through extends → a new license is issued → online customers receive it at their next refresh (within a day, or immediately with "Check now"); offline customers get the file by WhatsApp or email.
- **Online payments (1.1):** a Bangladeshi payment gateway that supports bKash, Nagad, and cards. The gateway's confirmation calls the same code path as a manual payment.
- **Refunds:** pro rata refund of unused whole months within an agreed window, admin action only, license revoked at the next online refresh. Accept that offline licenses stay valid until their end date; the loss is small.
- **Upgrades:** take effect immediately; charge the difference for the remaining days (admin screen calculates it). Downgrades take effect at renewal.
- **Read only mode (expired) allows:** login, viewing and searching everything, all reports, receipt and document reprints, CSV and Excel exports, full data export, backup, restore, verification, license import, update installation. **Blocks:** anything that creates or changes records (admissions, payments, attendance, marks, expenses, settings except license and backup), and all outgoing SMS and email.

Read only is the right commercial lever. It never holds data hostage, and it is the moment the owner calls you, which is the point.

---

## 9. Licensing Model

### Format

A `.nflicense` file (rename with the brand, see Section 20) is small JSON:

```json
{ "v": 1, "kid": "k2026a", "payload": "<base64url of canonical JSON>", "sig": "<base64url Ed25519 signature of payload bytes>" }
```

Payload fields: `license_id`, `customer_id`, `customer_name`, `installation_id` (the host it is bound to), `tier`, `limits` (active students, workstations), `capabilities`, `paid_through`, `grace_days`, `valid_until` (paid_through plus grace), `issued_at`, `serial` (increases with every issue for this license), `kind` (paid, extension, trial_conversion).

The app embeds the public keys for the current and the next `kid`.

### Verification rules in the app

1. Signature valid under a known, non retired `kid`.
2. `installation_id` matches this installation (or the moved license window applies, below).
3. `serial` is not lower than the serial already stored. You cannot swap back to an older or cheaper license.
4. The state is computed from `paid_through`, `grace_days`, and the clock guard.
5. Capabilities feed one entitlement service. Commands declare what they need; nothing else in the code checks editions.

### License the host, not each device

Only the host (or the single PC) holds a license. The host enforces the workstation limit when pairing. That makes the whole device story simple: customers see one installation per institution in the portal, and "OFFICE PC, ACCOUNTS PC" in the portal is just a listing the host reports at refresh.

### Online activation

Purchase or trial conversion → the owner enters a license key in the app → `activate` binds it to this installation → license stored. If another installation is active for that license, the app asks "Move the license to this PC?" Yes marks the old installation `moved`; the old one learns this at its next refresh and becomes read only after 7 days.

### Offline activation and renewal

- The app shows a short **request code** (installation ID plus license key checksum, about 30 characters, also as a QR code the owner can photograph with a phone).
- The customer sends it by WhatsApp or reads it over the phone. Your admin pastes it in, records the payment, clicks **Issue license**, and sends back the file.
- The customer imports it from Settings → License → Import. The file only works on that installation, so a leaked renewal file is useless to anyone else.

### Moving to a new PC

Restoring a backup on a new PC brings the license with it, but it is bound to the old installation. The new installation runs with full features for 14 days while unbound, and shows "Activate this PC". Online, one click. Offline, the request code flow. A dead PC on exam day never stops the institution.

### Refresh, downtime, long offline periods

- When online, the app refreshes once a day in the background. Failures back off quietly. Nothing ever waits on the license server.
- Server down for a week: nothing happens on any customer PC.
- Customer offline for months: the license works until `valid_until`. Yearly customers need nothing online all year.

### Clock rollback, reframed as a data safety feature

- The app keeps a high water mark: the latest trusted time it has seen (latest recorded transaction or refresh).
- If the PC clock is more than 48 hours behind that mark, the app blocks new financial entries with: "Your PC's date looks wrong (shows 1 January 2015). Please correct the date and time." This protects receipts and reports from wrong dates, which happens constantly when a CMOS battery dies, and it happens to also stop clock rollback abuse.
- If someone set the clock too far forward by mistake and the mark is stuck in the future, the Owner can acknowledge the correction with their password. It is audited.
- On LAN, only the host clock matters.

### Tampering and cracking

The signature stops edited files. A determined cracker can patch any offline binary; do not chase that with hardware locks or obfuscation that break legitimate customers. Make paying easy, renewal fast, and support good. That beats DRM in this market.

### Revocation and key compromise

- Revocation reaches a PC only when it goes online (signed revocation list in the refresh response). Offline licenses run to their end date. Accept this.
- Keep the private key only in the isolated signer, with two offline sealed backups. If it leaks: generate a new `kid`, ship an app update that retires the old one, and reissue every license automatically at refresh.

### Trial

Created locally at institution setup, 30 days, full features, then read only. A paid license imported into a trial institution keeps all its data. Reinstalling to reset the trial means starting a new, empty institution, which is its own deterrent.

---

## 10. Update Architecture

### CI and build

- Protected `main`. Release is a signed git tag. The release workflow runs in a protected environment that needs manual approval.
- Pipeline: lint and typecheck → Rust and UI unit tests → database and migration tests (every historical fixture upgraded to head) → financial invariant tests → integration tests → E2E critical flows on Windows → build → Windows code signing of the executable and installer → updater signature → publish to storage → metadata updated for `internal`.
- Pin third party CI actions to exact versions, lock dependency files, run dependency audits, and fail on known vulnerabilities in the release job.

### Signing

- **Windows code signing:** an OV certificate from a CA that offers cloud or hardware backed signing usable from CI. Since 2023, private keys must live on hardware or a cloud HSM, so a file in a CI secret is not an option. EV no longer skips SmartScreen warnings, so expect "Windows protected your PC" on early installs until reputation builds. Document the "More info → Run anyway" step with screenshots, sign every release with the same certificate, and submit early builds to major antivirus vendors as false positive checks. Start procurement early; identity validation for a Bangladeshi company can take weeks.
- **Updater signing key:** stored as an encrypted CI secret with its password in a separate secret, plus two offline backups. **If you lose this key, installed apps can never auto update again.** Treat it like the license key.
- Only the release job can read either secret. Pull request builds never can.

### Channels and metadata

- Channels: `internal` → `beta` → `stable`. Promotion copies metadata; it never rebuilds. A build spends at least 7 days in beta with at least one real site before stable.
- Metadata per channel: version, notes, URL, signature, `min_upgrade_from`, and a `blocked_versions` list that makes affected apps show "Please update now, this version has a known problem".
- Staged percentage rollout can come in 1.1 through a tiny endpoint that answers per installation.

### In the app

1. Update check (daily when online, or manual). The host checks; workstations follow the host.
2. Admin sees version and notes and chooses **Install now** or **Install when I close the app**. Never restart without being asked.
3. Download in the background, verify the signature.
4. Take a pre update backup (a full verified snapshot).
5. Install, restart, run the migration procedure below.

### Manual installer

One signed installer handles fresh install and upgrade. Download on any PC, copy by USB, run. It never touches the data folder. There is also a larger "offline" variant that includes the WebView2 runtime for PCs that never see the internet.

### Rollback strategy (honest version)

Database migrations go forward only, so "rollback" means:

1. **Migration failure:** the original database was never touched (see below). The app shows "The update could not finish. Your data has not been changed." with **Reinstall previous version** (the previous installer is kept in the data folder) and **Export diagnostics**.
2. **Bad release discovered later:** mark it blocked and ship a fix forward. Restoring the pre update backup is the last resort and loses work entered since the update, so the app must say so plainly.

### Migration procedure

1. On startup the app sees `schema_version` is older than the build expects.
2. Checks free space (at least 2.5 times the database size); refuses with a clear message if short.
3. Checkpoints WAL, closes all connections, copies `main.db` to `staging`.
4. Runs the migrations on the copy, each in a transaction.
5. Validates the copy: `integrity_check`, `foreign_key_check`, all financial invariants, row counts and money totals equal to before (unless a migration declares an expected change).
6. Atomically swaps the copy into place and keeps the old file as `pre-migrate-<version>.db` for 30 days.
7. Any failure before the swap leaves the original untouched. The database is never half migrated and writable.

Migration rules for developers: numbered SQL files plus an optional Rust data step, never edited once released, each release adds a fixture database created by the previous version, and CI upgrades every fixture since 1.0.

---

## 11. Backup and Disaster Recovery

### The encryption key problem, fixed

- At setup, the app creates a random database key.
- The key is stored twice: wrapped by Windows DPAPI (machine scope) for daily use, and wrapped by an **owner recovery key**.
- The setup wizard shows the recovery key once and prints a **Recovery Sheet** (key plus QR code plus instructions). The wizard does not continue until the owner confirms by typing the last 4 characters. The Owner can reprint it later after entering their password.
- Every backup contains the database key wrapped by the recovery key. Restoring on the same PC needs nothing. Restoring on a new PC needs the Recovery Sheet.
- The recovery key also resets a forgotten Owner password.
- **Decision for you (Section 20):** offer optional key escrow with the vendor for owners who fear losing the sheet. It lowers support pain and slightly weakens the privacy story. Opt in only if offered.

Tell customers plainly: "If you lose both your PC and your Recovery Sheet, nobody, including us, can open your backups."

### What gets backed up, and how

- **Snapshot** (daily, and before updates and restores): SQLite online backup of the database into a `.nfbackup` file with `manifest.json` (app version, schema version, institution ID, time, record counts, money totals, audit chain head, file list with SHA256, the wrapped key) and an HMAC over the manifest so tampering is detectable. Small, usually tens of MB.
- **Document vault mirror:** the backup destination keeps a copy of the content addressed vault. Each backup copies only new files. Photos resized on import keep this small.
- **Full export** (manual): one single file with snapshot plus all documents, for moving house or sending to support.
- **Hourly local snapshots** during business hours on the same PC (database only, keep 24). A corrupted database then loses at most an hour, not a day.

### When

- Daily at a configured time if the app is open, and at startup when the last good backup is more than 24 hours old. Do not rely on backup at shutdown; many offices switch off at the wall.
- Before every update, restore, and bulk import.

### Where

- Default: a folder on a different physical drive when one exists, otherwise a local folder plus a strong warning.
- USB and second drive: detected by a marker file with the institution ID, so a different USB stick is never mistaken for the backup drive.
- NAS or network folder: supported as a plain path. Never the live database.
- The dashboard warns when the destination is missing, full, or the last verification failed. After 3 days without a good external backup, the warning appears on every login for Owner and Admin.

### Writing safely

Write to a temporary file, flush, verify, then rename. A removed USB drive or a power cut leaves either the previous good backup or a clearly incomplete temp file that is cleaned up next time.

### Verification

After every backup: reopen the snapshot with the key, `integrity_check`, compare record counts and money totals with the manifest, check manifest checksums, confirm referenced vault files exist at the destination. Mark "created" and "verified" separately in the UI.

**Weekly restore drill:** in the background, restore the latest backup into `staging`, run the app's invariant checks against it, delete it, and record the result. This proves backups can actually be restored, which is the only thing that matters.

### Retention

Keep 7 daily, 4 weekly, 12 monthly, plus pre update and pre restore backups for 30 days. Prune only after a newer verified backup exists. Never prune the last verified one.

### Restore

1. Choose a backup file or backup folder.
2. The app verifies it and shows institution name, date, app version, record counts, and integrity status.
3. If a newer schema than this app supports: "Update the app first."
4. On a new PC, ask for the Recovery Sheet.
5. Take a safety backup of the current data if any exists.
6. Restore into `staging`, migrate if older, validate, then swap in atomically.
7. Show the result and a summary of counts and totals to compare with what the owner expects.

### Corruption recovery

If the startup check fails, the app opens read only and offers: restore the latest verified backup (showing what time it is from), or run salvage (the SQLite recover procedure into a new file) for support to examine. Never keep writing to a database that failed its check.

### Off site

Theft, fire, and ransomware take the PC and the USB stick that stays plugged in. The guide recommends two USB drives rotated weekly, one kept at home. An encrypted off site backup (encrypted on the PC, so the vendor cannot read it) is a good paid add on for V2.

---

## 12. Security Threat Model

| Threat | Example | Mitigations | Remaining risk |
|---|---|---|---|
| Local attacker with the PC | Someone boots the office PC or copies the data folder | SQLCipher, DPAPI wrapped key, app login, idle lock | Anyone who can log in to Windows can run the app and reach the key. Say so honestly. |
| Malicious staff | Cashier voids payments, pockets cash; teacher changes marks | Permissions checked in the core, voids need reason and permission and only on open days, daily cash close, collector wise reports, audit hash chain, immutable financial rows, Owner alerts on voids and refunds | Collusion with the Owner account. |
| Stolen database or backup | USB stick lost | Encrypted database inside backups, key wrapped by recovery key, HMAC on manifest | Weak recovery sheet storage. |
| Tampered license | Edit expiry date | Ed25519 signature, installation binding, serial check | Binary patching (accepted). |
| LAN attacker | Student on office WiFi scans the network | TLS, no unauthenticated data endpoints, PAKE pairing only while the window is open, device tokens, rate limits, host not reachable on Public networks | A compromised paired PC (see next). |
| Malicious workstation | Stolen device token | Tokens stored with DPAPI, bound to device, revocable, every request still needs a user session and permissions, last seen shown to admins | Session hijack on a PC already compromised. |
| Malformed import | Crafted Excel file | Parse in the core with size and row limits, strict types, preview before commit, reject formulas, one transaction per import with a pre import backup | None significant. |
| Path traversal | Document named `..\..\x` | Vault stores by hash, original names only as metadata, UI passes IDs never paths, backup extraction rejects absolute paths and `..` | None significant. |
| Credential theft | SMS token read from config | Secrets wrapped by DPAPI, write only in UI, redacted from logs and diagnostics | Local admin level malware. |
| Update supply chain | Poisoned dependency or stolen signing key | Pinned dependencies, audits, release only from protected tag with approval, keys readable only by the release job, updater verifies signature, code signing | A compromised developer machine with release rights. Use two person approval for releases. |
| Backup tampering | Edited backup restored to hide fraud | Manifest HMAC, checksums, audit chain head comparison, restore is audited and needs `backup.restore` | Owner restoring an old backup on purpose (visible in audit). |
| Clock manipulation | Backdating receipts | Clock guard, host time only on LAN, business date locks after day close | None significant. |

### Privacy, telemetry, crash reporting

- **Nothing is sent automatically** except the license refresh and update check, which contain only installation ID, license ID, app version, and an active student count (needed for size tiers). List exactly this on the privacy page and in the app.
- **No analytics SDK** in 1.0. If you want product analytics later, make it opt in, counts only (which screens are used, never content), and show the payload.
- **Crash reports:** the app writes crash details and redacted recent logs locally. The next start offers "Send this report to support?" with a view of exactly what will be sent. Nothing leaves without a click.
- **Diagnostics bundle:** versions, settings (secrets removed), redacted logs, database stats (counts, sizes, integrity result), backup history, license state, LAN state. Never records, names, or phone numbers.

---

## 13. Product Feature Scope

### V1.0 must have (single PC)

- Setup wizard: new institution, restore, trial start, data folder choice, Recovery Sheet
- Users, default roles plus custom roles, permissions, login, idle lock, Owner recovery
- Students and guardians, profiles, documents, photo, custom fields, fast search
- Programs, subjects, batches, teachers (staff profiles), enrollments with history, transfers
- Simple admission form (direct admission with initial payment and admission receipt)
- Student import from Excel with preview and error file (every customer starts from Excel)
- Fee heads, fee plans, concessions, monthly charge generation
- Fast cashier, payments, receipts (A4 two copy, A5, 80 mm), reprint, void, refund, adjustments
- Dues list and due statement
- Expenses and daily cash close
- Attendance by batch and date, keyboard entry, lock, monthly percentage
- Exams, spreadsheet style mark entry, verification, grading schemes, results, marksheet, report card, merit list
- Reports: daily and range collection, collector wise, dues, expenses, cash days, attendance monthly, admissions, results; print, PDF, CSV, Excel
- Role dashboards (owner, accountant, receptionist, teacher), kept simple
- SMS outbox with a generic HTTP adapter and one Bangladeshi provider adapter; templates; automations for payment, absence, due reminder; send to a batch
- Backups, verification, restore drill, restore, retention, corruption recovery
- Audit log with hash chain; record activity tab
- License: trial, activation online and offline, refresh, grace, read only mode, clock guard
- Updates: online and offline installer, safe migrations
- Diagnostics bundle, logs
- English UI with full Bangla data entry, Bangla print templates and Bangla SMS; the translation layer ready for a Bangla UI
- Demo institution (built from the same generator used for performance tests)

### V1.1

- LAN multi PC (host, pairing, workstations, update mirror)
- Bangla UI translation
- Email channel (SMTP)
- Inquiry and applicant pipeline
- Student ID cards and admit cards with barcode; barcode scan for attendance and cashier search
- Online renewal checkout in the portal
- Teacher payment ledger
- Per counter cash sessions
- Bulk tools (bulk status change, bulk concessions)
- SMS delivery status polling for adapters that support it
- Staged update rollout

### V2

- Encrypted off site backup (paid add on)
- Managed SMS credits (paid add on)
- Multi branch consolidated reporting
- Windows service host mode
- Guardian payment links by bKash or Nagad (needs a cloud component; design carefully against the privacy promise)
- Timetable conflict detection, batch merging

### Do not build yet

Full LMS, video, live classes, AI tutor, social features, hostel, transport, library, inventory, payroll, double entry accounting, student or guardian mobile app, cloud student portal, any NF ecosystem link or SSO, macOS or Linux builds, coupons, an in house ticket system.

---

## 14. Pricing and Packaging Recommendation

> **Decided 2026-09-20: feature tiers.** Two plans, **Coachify Local** (single PC) and **Coachify Local Pro**. The only feature on Pro alone is **LAN multi PC** (host plus N workstations). SMS, automations, reports, bulk tools, custom roles, and every item in "Never paywall" below stay on both plans. The size tier table below is kept as the alternative that was considered. Because only one feature is gated, a later move to size tiers stays cheap: the entitlement service already reads capabilities and limits from the license.

### What is wrong with Local / Local Pro as proposed

- **SMS behind Pro:** customers bring their own SMS account and pay for every message themselves. Gating it costs you nothing to give and loses deals where the owner's first question is "does it send SMS to parents?"
- **Advanced permissions and audit behind Pro:** these are fraud controls. An owner who gets robbed by a cashier on the cheap plan blames the product.
- **Features are a weak value measure here.** Size is what owners understand and what matches their ability to pay.

### Recommended structure

One product, every core feature on every plan. Tiers by **active students** with workstations included:

| Tier | Active students | PCs |
|---|---|---|
| Starter | small (set after pilots, for example up to 300) | 1 PC |
| Standard | medium | host plus 2 workstations (from 1.1) |
| Plus | large | host plus 5 workstations |
| Enterprise | above that | quoted |

- Extra workstation as a small add on.
- Limits are soft. At 100% of the student limit the app warns; above 110% it blocks **new admissions only**, never payments or anything else, until the owner upgrades or archives old students.
- Annual price clearly cheaper than 12 months paid monthly.
- Consider a one time setup and data import fee, done by your staff or a partner. In this market, assisted onboarding sells software, and it pays for your onsite visit.
- Paid add ons later: off site backup, managed SMS credits, priority support.

### Never paywall

Backups, verification, restore, encryption, audit log, voids and cash close controls, roles and permissions, data export, receipt reprints, read only access after expiry, security updates, the Recovery Sheet.

Exact prices: set after pilots, based on what three to five real centers say they would pay. Do not guess them now.

---

## 15. User Journeys

### New institution setup

1. Run the installer (UAC prompt once). Launch.
2. Welcome: **Create new institution**, **Restore from backup**, **Connect to a host PC** (1.1).
3. Create: name (English and Bangla), logo, address, phones, session, student ID format, receipt prefix.
4. Data folder: the app recommends a non system drive when one exists.
5. Owner account: username, password.
6. Recovery Sheet: shown, printed or saved as PDF, confirmed by typing the last 4 characters.
7. Backup destination: pick a second drive or USB; the app writes a marker and tests it.
8. License: continue trial, or enter a license key (online) or import a file (offline).
9. Optional: load sample programs and fee heads, or import students from Excel now.
10. Dashboard with a short checklist: create batches, fee plans, users, printer setup, SMS setup.

### Student admission

1. Receptionist opens **New admission** (Ctrl+N).
2. Types guardian phone first; the app shows existing guardians and siblings to reuse.
3. Student details, photo, documents.
4. Chooses program and batches; the app shows fee plans that will apply and capacity.
5. Optional concession (needs `concessions.manage`, reason required).
6. Save creates student, guardian link, enrollments, one time charges (admission fee), and the current month's charges, in one transaction.
7. If an initial payment is entered, it goes through the same payment transaction; receipt prints.
8. Admission SMS goes to the outbox if the rule is on.

### Payment collection

1. Cashier presses F2 (cashier screen) and types part of an ID, name, roll, or phone.
2. Results appear as they type; Enter selects.
3. Screen shows student, batch, open charges by month, total due, credit, last 5 payments.
4. Amount is prefilled with total due; the cashier edits it and picks the method (and reference for bKash or Nagad).
5. **Receive** (Enter). The button locks until the command returns.
6. Receipt prints silently on the configured receipt printer. Options: reprint, PDF, send SMS now.
7. If the printer is offline: "Payment saved. Receipt not printed." with Reprint. The payment is never rolled back because of a printer.

### Attendance

1. Teacher opens **Today's batches** and picks one.
2. All students listed as present by default (or blank, a setting).
3. Arrow keys move; A, L, E, P set absent, late, excused, present. Notes optional.
4. Save. Absence SMS queued with a 30 minute delay.
5. Later edits within the lock window are allowed; after it they need permission and are audited.

### Exam and result

1. Admin creates exam: name, session, batches, subjects with full and pass marks, grading scheme.
2. Teachers enter marks in the grid: keyboard navigation, `ab` for absent, red for out of range, autosave as draft.
3. A verifier checks each subject and marks it verified.
4. Admin clicks **Calculate**: preview of totals, grades, ranks, failures, missing marks.
5. **Publish** (needs `results.publish`): results and rules frozen as a snapshot.
6. Print marksheets and report cards per batch; merit list; optional result SMS.

### Backup

1. Automatic daily. Dashboard tile shows last backup, last verified, destination state.
2. Manual: **Backup now** shows progress, then "Backup created and verified".
3. Warnings when USB is missing, disk is full, or verification failed, with the fix.

### Restore

1. New PC: install, **Restore from backup**, choose file or folder.
2. Details and integrity shown; enter Recovery Sheet key.
3. Restore, migrate if needed, validate, summary of counts and totals.
4. License shows "Activate this PC" with 14 days of full use.

### LAN workstation setup (1.1)

1. Host: Settings → Local network → **Enable host**. The app checks firewall and network profile and fixes or explains.
2. Host: **Pair new device**, code shown.
3. Workstation: install, **Connect to a host PC**, pick the host found on the network (or type its address), enter code.
4. Paired. Login screen. Printer setup for that PC.
5. Admin sees the device in Settings → Devices, can rename, block, revoke.

### Subscription renewal (online)

1. 30 days before end: banner "Your subscription ends on 15 October".
2. Owner pays (bKash, bank) and sends the reference, or pays in the portal (1.1).
3. Vendor admin records payment; new license issued.
4. Within a day, or immediately with **Check now**, the app refreshes and the banner disappears.

### Offline license renewal

1. Owner pays and sends proof plus the request code from Settings → License (a photo of the QR works).
2. Vendor admin records payment and issues a license bound to that installation.
3. File sent by WhatsApp or email, copied by USB.
4. Settings → License → **Import**. The app shows the new end date.

### Software update

1. Update available: admin sees version and notes.
2. **Install now** or **Install when I close**.
3. Pre update backup, download, signature check, install, restart, migrate on a copy, validate, swap.
4. "Updated to 1.2.0". Failure shows the recovery screen with the previous version available.
5. On LAN (1.1): workstations see "Host updated, update this PC (1 minute)".

---

## 16. Testing Matrix

| Area | Critical tests |
|---|---|
| Unit | Fee generation for joins mid month, concessions, allocation order, balance math, receipt numbering, grading and GPA, rank ties, permission resolution, SMS segment counting (GSM 160/153, Unicode 70/67), Bangla template rendering, phone normalization, license verification cases, clock guard, manifest validation |
| Integration (core, no UI) | Every command: permission denied path, license gate path, idempotent retry, audit row written, outbox row written when expected |
| Database | Constraints and triggers reject forbidden updates and deletes; foreign keys; migrations from every released fixture; migration failure leaves the original untouched |
| Financial | Property based test: 100,000 random sequences of charges, payments, voids, refunds, adjustments, day closes, with all invariants checked after every step; report totals equal ledger totals |
| LAN (1.1) | Pairing success, wrong code, expired code, man in the middle rejected, revoked device, host restart mid request, two cashiers same student, version mismatch update from host; the whole integration suite rerun over the LAN transport on localhost |
| Backup | Snapshot while writes happen, verification catches a flipped byte, USB removed mid write, disk full, retention pruning never deletes the last verified |
| Restore | Fresh PC restore with recovery key, wrong key, older version backup then migrate, newer version backup refused, counts and totals equal |
| Upgrade | Install previous release, create realistic data, upgrade with the installer and with the updater, validate |
| Installer | Clean Windows 10 22H2 and Windows 11, with and without internet, without WebView2, non admin Windows user using the app, uninstall keeps data, reinstall finds data |
| Offline | Full daily workflow with the network adapter disabled; SMS queued; reconnect sends each exactly once |
| SMS and email | Mock provider: success, rejection, timeout (marked uncertain, not resent), slow responses, quiet hours, absence delay and cancel |
| Licensing | Tampered payload, wrong key ID, other installation, lower serial, expired, grace boundaries, clock back 3 days, clock forward then corrected, moved license window |
| Security | Path traversal in documents and backups, malformed Excel, SQL injection attempts in search, secrets absent from logs and diagnostics, UI cannot call anything except `rpc` |
| Performance | See targets below, on the reference low end PC |

### Required failure tests and expected behavior

| Failure | Expected behavior |
|---|---|
| Power loss during payment | After reboot: either the payment exists fully (payment, allocations, receipt number, audit) or not at all. Retrying with the same idempotency key never duplicates. |
| Power loss during migration | Original database untouched; migration reruns on the next start. |
| Power loss during backup | Previous backup intact; partial temp file cleaned up; next backup runs. |
| App crash | Same as power loss; jobs with expired leases resume; unsent SMS stays queued. |
| Database locked | Cannot happen in normal operation with a single writer; if a reader times out, the screen shows "Busy, try again" and nothing is lost. |
| Host crash or restart (1.1) | Workstations show reconnecting, keep form data, block saves, reconnect automatically. |
| LAN loss and client reconnect | Same; after reconnect, all visible data is refetched. |
| Duplicate payment click | One payment. The second click returns the same receipt. |
| Two cashiers, same student | One succeeds; the other gets "Due changed, please review" with fresh numbers. |
| SMS or email timeout | Business action already committed; message marked uncertain; no automatic resend. |
| Internet outage | Nothing visible except the outbox counter and a small offline indicator. |
| Printer offline | Payment saved; "Receipt not printed" with Reprint. |
| USB drive removed | Backup fails cleanly, warning on dashboard, next attempt when the drive returns. |
| Disk low | Warning at 2 GB free; backups and updates refuse with a clear message at thresholds; normal work continues. |
| Corrupt backup | Verification marks it failed; restore refuses it and suggests the previous verified one. |
| Older client, newer host (1.1) | Client offered the update from the host; cannot write until updated. |
| Invalid license | Import refused with a reason; the current license stays active. |
| Clock changed backwards | Financial entries blocked with a friendly "fix the date" message; everything else works. |
| Update server down | Nothing happens; manual installer still works. |
| Update download interrupted | Resumes or restarts; signature verified before install; nothing installed from a partial file. |

### Performance targets

Reference PC: 4th generation Core i3 or similar, 4 GB RAM, spinning hard disk, Windows 10. Dataset: 10,000 active students plus 15,000 archived, 60,000 payments, 3 years of attendance (about 6 million marks), 200,000 messages, 30,000 documents.

| Measure | Target |
|---|---|
| Cold start to login screen | under 6 s |
| Student search results (p95) | under 150 ms |
| Open cashier for a student (p95) | under 300 ms |
| Receive payment to receipt on screen (p95) | under 800 ms |
| Save attendance for 80 students | under 300 ms |
| Monthly collection report | under 2 s |
| Daily backup snapshot and verify | under 60 s |
| App memory in normal use | under 400 MB total |

All lists paginate or virtualize in the core. React never receives more than one page of rows. Design ceiling: 25,000 active students, 10 million attendance marks, 25 workstations. Beyond that is not your market.

---

## 17. Release Acceptance Criteria for 1.0

1.0 may be sold when **all** of these are true:

1. **Scope:** every V1.0 must have feature in Section 13 is `done` in the scope, with its specs, tests, and verification.
2. **Automated suites green on the release commit:** unit, integration, database, migration from every released fixture, E2E critical flows (setup, admission, payment, attendance, exam, backup and restore, license, update) on Windows.
3. **Financial:** the property based ledger test runs 100,000 operations with zero invariant violations.
4. **Crash safety:** 200 forced kills during payments, 50 during backups, and 50 during migrations, with zero corrupted databases, zero duplicate payments, and zero partial records.
5. **Backup:** the automated backup, wipe, fresh install, restore, compare test passes with identical counts and money totals.
6. **Performance:** every target in Section 16 met on the reference PC with the reference dataset.
7. **Install:** clean install, upgrade from the previous release, offline install without WebView2, and uninstall keeping data, all pass on Windows 10 22H2 and Windows 11.
8. **Print:** receipts, marksheet, report card, due statement, and collection report checked on at least 2 laser printers, 1 inkjet, and 2 common thermal 80 mm models, including Bangla text, long names, and large amounts.
9. **Pilots:** at least 3 institutions used it live for at least 4 weeks each, covering at least one monthly fee cycle and one exam cycle, with zero data loss incidents, and a real restore drill done at each site.
10. **Bugs:** zero open P0 or P1 bugs; every P2 has a documented workaround.
11. **Commercial:** a license issued online and one offline, renewed both ways, and a license move to a new PC, all done end to end by someone other than the developer.
12. **Signing:** code signing and updater signing in place; key backups stored and a recovery test done.
13. **Docs:** quick start, backup and restore, printer setup, SMS setup, Excel import, troubleshooting, privacy and data ownership, license guide; plus an internal support runbook.
14. **Security:** threat model mitigations in Section 12 each have a test or a review record.
15. **Readiness report** states what was tested, on which PCs and printers, with results. No "production ready" claims beyond that.

LAN (1.1) adds: pairing, revocation, 10 workstations concurrency test, host restart test, update from host test, and one pilot running LAN for 4 weeks.

---

## 18. Repository and Agent Workflow

### Repositories: two, not one

- **`desktop`** (this folder): Tauri app, Rust core, UI, migrations, print templates, license verification, desktop docs and specs.
- **`cloud`**: website, portal, admin, license signer, update metadata publisher.

Why not the monorepo: the desktop is Rust plus React and the cloud is TypeScript; there is no code worth sharing. A shared repo gives agents a bigger context to wander through, makes it easy for an agent working on the desktop to touch signing code, and couples release cadences. The only contract between them is the license format and the update metadata format. Write those once in a short versioned document in each repo, with **shared test vectors** (sample payloads, signatures, and expected verification results) committed to both. Both test suites check the vectors, so drift fails CI.

Drop the separate docs repo. Customer docs live in the cloud repo (they are published on the website); engineering docs live with the code they describe.

### Desktop layout

```
desktop/
  AGENTS.md              conventions agents must follow
  docs/scope/            living scope (this workflow)
  docs/specs/            one spec per decision
  docs/architecture-review.md
  crates/core/           domain, database, migrations, permissions, audit, jobs, backup, license checks
  crates/lan/            (1.1) host server and client transport
  src-tauri/             thin shell: windows, tray, IPC, printing, updater
  ui/                    React app and print templates
  migrations/            numbered SQL, never edited after release
  tests/fixtures/        a database per released version, test vectors
```

### Workflow

- **Trunk based.** Short feature branches, one pull request per feature milestone, CI required, squash merge.
- **Plan:** `docs/scope/` is the master plan. Each feature runs `/architect` (spec with decisions and a build plan), `/develop`, `/check verify`, `/test`; money, backup, license, migration, and LAN features also get `/check review` by a fresh model.
- **Architecture documents:** this review, then one spec per load bearing decision. Specs record decisions; code follows specs.
- **Migrations:** one new numbered file per change, generated with a fixture update, never edits to released migrations. An agent that edits a released migration has made an error; add a CI check that fails on it.
- **Testing requirements:** every command has a permission test, an idempotency test where it creates records, and an audit test. Money code adds invariant tests.
- **Definition of done:** UI, core logic, database constraints, permission check, audit, error states, tests, verified in the running app on Windows, and no mock data in production paths. UI existing is never "done".
- **Release checklist:** Section 17, as a checklist file in the repo, filled in per release.

### Keeping agents from overengineering

- AGENTS.md states: no new abstraction without two real callers; no new dependency without a line in the spec; no generic plugin systems, event buses, repositories of repositories, or microservices.
- One feature per session. Clear the context between features; the scope and specs carry state.
- Agents never touch signing, release workflows, or migrations that are already released without explicit human approval.

---

## 19. Implementation Phases

Each phase ends with something that runs. Phases 3 and 4 produce a build a friendly pilot center can use for fee collection alone, which is where you learn the most.

| Phase | Content | Depends on | Exit criteria |
|---|---|---|---|
| 0. Spikes | Tauri plus SQLCipher plus FTS5 trigram in Windows CI; WebView2 silent print A4, A5, 80 mm; Bangla PDF; offline WebView2 installer; signed update; DPAPI | nothing | Each proven on a clean Windows 10 PC, go or no go on the stack recorded in a spec |
| 1. Foundation | Repo, tooling, core crate, `rpc` dispatcher, migration engine with copy and swap, database conventions, users, roles, permissions, audit chain, settings, setup wizard, key management and Recovery Sheet, UI shell and design system, translation layer | 0 | Fresh install creates an institution, Owner logs in, a permission denied path and an audit entry are visible, a migration upgrade test passes |
| 2. Core records (walking skeleton) | Students, guardians, search, programs, batches, staff, enrollments, Excel import, demo and load data generator | 1 | 10,000 generated students searchable under 150 ms; import of a real Excel file with error rows |
| 3. Money | Ledger model, fee plans, concessions, charge generation, cashier, payments, receipts and print engine, voids, refunds, adjustments, dues, expenses, cash close | 2 | Property based ledger test green; receipts printed on real printers; power kill test on payments |
| 4. Safety and delivery | Backups, verification, restore drill, restore, retention, corruption recovery, diagnostics, installer, update pipeline (unsigned internal at first) | 3 | Backup, wipe, restore test green; installer and upgrade tests green. **Pilot 1 starts on fee collection.** |
| 5. Daily academics and messaging | Attendance, SMS outbox, adapters, templates, automations (payment, absence, due), batch SMS | 4 | Offline then reconnect sends exactly once; uncertain sends not repeated |
| 6. Exams | Exams, mark entry grid, verification, grading, results, marksheets, report cards, merit list | 5 | Full exam cycle at a pilot site |
| 7. Reports and dashboards | Reports center, exports, role dashboards, admission form print | 6 | Every report total matches the ledger in tests |
| 8. Commercial | License verification and read only mode, clock guard, cloud admin, manual payments, license issuing, activation and refresh endpoints, minimal portal and website, code signing | 4 (can run in parallel with 5 to 7) | Online and offline license flows end to end, done by a non developer |
| 9. Hardening and 1.0 | Failure tests, performance on the reference PC, print matrix, docs, three pilots, release criteria | all | Section 17 fully met |
| 10. 1.1 | LAN, Bangla UI, email, ID cards and barcode, inquiries, online checkout | 9 | Section 17 LAN additions met |

---

## 20. Decisions (resolved 2026-09-20)

All thirteen open decisions were answered. These now override anything earlier in this document that disagrees.

| # | Decision | Answer | What it changes |
|---|---|---|---|
| 1 | Product name | **Coachify** | Files become `.coachifybackup` and `.coachifylicense`; data folder, mDNS service, update domain, and certificate subject all use Coachify. "NF" disappears from the product. |
| 2 | LAN timing | **1.1**, seam built now | 1.0 is single PC. The command dispatcher (scope feature 5) stays LAN ready from day one. |
| 3 | Pricing | **Feature tiers**: Coachify Local and Coachify Local Pro | Pro adds only LAN multi PC. Everything else is on both plans. See Section 14. |
| 4 | Expired behavior | **Fully read only** after 14 day grace | As in Section 8. |
| 5 | Recovery key | **Printed Recovery Sheet only**, no vendor escrow | The vendor can never open customer data. Setup must make the sheet impossible to skip. |
| 6 | Windows | **64 bit Windows 10 (1809+) and 11** | One build target. |
| 7 | UI language | **English UI**, Bangla data, print, and SMS | Translation layer built in 1.0; Bangla UI in 1.1. |
| 8a | Monthly fee basis | **Calendar month** | Charges generated for `YYYY-MM` periods. |
| 8b | Mid month join | **Institution setting** (full, half, or from next month, with a cutoff day) | A setting read by charge generation; cashier can still waive with a reason. |
| 8c | Late fines | **Manual only** | No fine automation in 1.0. |
| 8d | Receipt numbers | **Reset each calendar year** | Sequence name `receipt:<year>`. |
| 9 | Company, merchant accounts, VAT | **After pilots** | Pilots run free. Online checkout stays 1.1 anyway; manual bKash/Nagad to a business account needs the paperwork before the first sale. |
| 10 | Code signing certificate | **After pilots** (with item 9) | Pilot builds are unsigned: you install them yourself and click through SmartScreen. Because identity validation takes weeks, the first sale waits on it. Scope feature 36 moves to just before 1.0. |
| 11 | Support | **Phone, WhatsApp Business, remote desktop**, diagnostics export | No ticket system. Docs include a remote help page. |
| 12 | Branches | **One installation and license per branch** | Consolidated reporting stays V2. |
| 13 | Grading schemes | **National GPA 5.0 scale** preloaded | Institutions can copy and edit it into custom schemes. |

### Still open, but not blocking

- Exact prices for Local and Local Pro: set after pilots.
- How many workstations Local Pro includes, and the price of an extra one: set before 1.1.
- VAT treatment of subscriptions: ask an accountant when the company paperwork starts.
