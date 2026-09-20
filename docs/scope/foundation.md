# Foundation

Back to [index](index.md).

### 1. Stack & architecture · in-progress · GA
Confirm the desktop stack by running the Phase 0 spikes on a clean Windows 10 PC, then scaffold a runnable app with the core logic in its own library, separate from the UI shell.
**Done when:** silent printing (A4 and A5, Bangla) and encrypted trigram search are proven on a real Windows 10 PC; the go or no go is recorded in spec 0001; the empty scaffold builds and runs with one `rpc` round trip.
spec [0001](../specs/0001-desktop-stack-architecture/index.md)
- [x] Decide the stack (spec): `/architect stack & architecture`
- [x] Scaffold from the decision: `/develop stack & architecture`
- [x] Verify it: `/check verify stack & architecture`
- [ ] Test it: `/test stack & architecture`
- [ ] Review it (fresh model): `/check review stack & architecture`
- [x] Document it: `/document stack & architecture`

### 2. Coding standards & tooling · done
Capture conventions (no business logic in the UI, one command entry point, no new abstraction without two callers) and install lint, format, type strictness, hooks, and Windows CI.
**Done when:** root `AGENTS.md` reflects the real project, CI runs on Windows, and a check fails any edit to an already released migration.
- [x] Capture conventions + tooling choices: `/audit`
- [x] Install the tooling: `/develop tooling`

### 3. Data model & database conventions · done · GA
Core entities, keys, money as integer poisha, business date versus timestamp, immutability triggers, and database settings for power cut safety.
**Done when:** the schema for institution, people, academics, and system tables is agreed; forbidden updates and deletes on posted rows are rejected by the database; settings survive a forced power off test.
spec [0002](../specs/0002-data-model-database-conventions/index.md)
code in [crates/core/src/db.rs](../../crates/core/src/db.rs), [migrations/](../../migrations/)
- [x] Design it (spec): `/architect data model & database conventions`
- [x] Build it: `/develop data model & database conventions`
  - [x] Configure database PRAGMAs (WAL, synchronous FULL, foreign keys, busy timeout) and startup quick check, covers AC-1, AC-2
  - [x] Purpose based migrations 0002 to 0006 creating core tables, people directories, academic structures, financial ledger, and triggers, covers AC-3, AC-5
  - [x] Implement SQLite immutability triggers for audit log, clock guard, posted payments, charges, allocations, adjustments, refunds, and cash days, covers AC-4
  - [x] Implement Rust database utilities for sequence allocation, transaction management, and business date handling, covers AC-1, AC-5, AC-6
  - [x] Integration tests verifying constraints, trigger abort rejections, and power crash resilience, covers AC-1, AC-2, AC-3, AC-4, AC-5, AC-6
- [x] Verify it: `/check verify data model & database conventions`
- [x] Test it: `/test data model & database conventions`
- [x] Review it (fresh model): `/check review data model & database conventions`
- [x] Document it: `/document data model & database conventions`

### 4. Design system & UI foundation · in-progress · GA
A calm, dense, keyboard first look, the app shell, tables that page in the core, bundled Bangla and Latin fonts, and a translation layer so a Bangla UI is only a translation later.
**Done when:** `design.md` covers type, color, spacing, tables, forms, and dialogs; base components handle focus and keyboard; every UI string goes through the translation layer.
spec [0003](../specs/0003-design-system-ui-foundation/index.md)
code in [design.md](../../design.md), [ui/src/styles/tokens.css](../../ui/src/styles/tokens.css), [ui/src/components/](../../ui/src/components/)
- [x] Design it (spec): `/architect design system & UI foundation`
- [x] Build it: `/develop design system & UI foundation`
  - [x] Configure Tailwind CSS v4 pure CSS token architecture and semantic variables, covers AC-1
  - [x] Implement ThemeProvider with 6 color palettes, dark and light modes, font switcher, density, and scaling, covers AC-2, AC-3, AC-4, AC-5
  - [x] Build accessible component primitives (Button, Input, Select, Dialog, DropdownMenu, Tabs, Table, Badge, Card, MetricCard, Progress, Avatar, Tooltip, Switch, Command), covers AC-6
  - [x] Build desktop AppShell with left navigation rail, top bar with Ctrl+K search, and reference EduManage dashboard layout, covers AC-7
- [x] Verify it: `/check verify design system & UI foundation`
- [ ] Test it: `/test design system & UI foundation`
- [x] Review it (fresh model): `/check review design system & UI foundation`
- [x] Document it: `/document design system & UI foundation`

### 5. Command dispatcher & local accounts · in-progress · GA
One typed command entry point that resolves the session, checks permission and license gate, runs a transaction, and emits change events; plus users, roles, custom roles, permissions, login, idle lock. This is the seam LAN mode plugs into later.
**Done when:** the UI can only reach the core through the dispatcher; a denied permission is rejected in the core, not just hidden; a repeated command with the same idempotency key returns the first result; at least one active Owner always exists.
spec [0004](../specs/0004-command-dispatcher-local-accounts/index.md)
- [x] Design it (spec): `/architect command dispatcher & local accounts`
- [x] Build it: `/develop command dispatcher & local accounts`
  - [x] Migration 0007 creating accounts, roles, permissions, devices, sessions, and idempotency tables with defense in depth triggers, covers AC-1, AC-3, AC-7
  - [x] Password hashing with Argon2id, secure token generation, and account repository queries, covers AC-2, AC-3, AC-4
  - [x] Core command dispatcher pipeline with session resolution, permission checks, license gating, transaction separation, and idempotency cache, covers AC-1, AC-4, AC-5, AC-6
  - [x] Typed RPC command handlers and contract generation for UI bridge, covers AC-1, AC-5, AC-8
  - [x] React AuthContext, session management, activity heartbeat, and idle lock screen, covers AC-2, AC-8
- [x] Verify it: `/check verify command dispatcher & local accounts`
- [ ] Test it: `/test command dispatcher & local accounts`
- [ ] Review it (fresh model): `/check review command dispatcher & local accounts`
- [ ] Document it: `/document command dispatcher & local accounts`

### 6. Audit log · in-progress · GA
Insert only audit trail with a hash chain, written inside the same transaction as the change, with sensitive fields removed.
**Done when:** every sensitive command writes an audit row; editing or deleting an audit row is rejected; a broken chain is detected by a diagnostics check.
spec [0005](../specs/0005-audit-log/index.md)
- [x] Design it (spec): `/architect audit log`
- [x] Build it: `/develop audit log`
  - [x] Migration 0008 adding lookup indexes, registering audit.view permission, and mapping to Owner, covers AC-4, AC-6
  - [x] Core audit module with SHA-256 hash chain calculation, recursive sensitive field redaction, and diagnostics verification, covers AC-2, AC-3, AC-5
  - [x] Command dispatcher integration wiring in-transaction audit logging for mutation commands and security events, covers AC-1, AC-3
  - [x] Typed RPC commands (audit.list, audit.verify_chain, audit.get_head) and TypeScript boundary contract, covers AC-5, AC-6
  - [x] React Audit Log view with timeline, filter controls, before/after diff inspect modal, and verification badge, covers AC-6, AC-7
- [x] Verify it: `/check verify audit log`
- [ ] Test it: `/test audit log`
- [ ] Review it (fresh model): `/check review audit log`
- [ ] Document it: `/document audit log`

### 7. Migration engine · needs a decision · GA
Numbered forward migrations run on a copy, validated (integrity, foreign keys, money totals), then swapped in atomically, with the old file kept.
**Done when:** a failing migration leaves the original database untouched and shows the recovery screen; CI upgrades every released fixture database to the latest schema.
- [ ] Design it (spec): `/architect migration engine`

### 8. Setup wizard & key management · needs a decision · GA
First run: create institution, choose a data folder (non system drive recommended), Owner account, printed Recovery Sheet, backup destination, trial start. Random database key wrapped by the OS for daily use and by the recovery key for disaster recovery.
**Done when:** a fresh install reaches the dashboard offline; the database cannot be opened by copying the files elsewhere; the recovery key opens a copied database and resets the Owner password.
- [ ] Design it (spec): `/architect setup wizard & key management`
