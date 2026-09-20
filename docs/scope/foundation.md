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

### 2. Coding standards & tooling
Capture conventions (no business logic in the UI, one command entry point, no new abstraction without two callers) and install lint, format, type strictness, hooks, and Windows CI.
**Done when:** root `AGENTS.md` reflects the real project, CI runs on Windows, and a check fails any edit to an already released migration.
- [ ] Capture conventions + tooling choices: `/audit`

### 3. Data model & database conventions · needs a decision · GA
Core entities, keys, money as integer poisha, business date versus timestamp, immutability triggers, and database settings for power cut safety.
**Done when:** the schema for institution, people, academics, and system tables is agreed; forbidden updates and deletes on posted rows are rejected by the database; settings survive a forced power off test.
- [ ] Design it (spec): `/architect data model & database conventions`

### 4. Design system & UI foundation · needs a decision
A calm, dense, keyboard first look, the app shell, tables that page in the core, bundled Bangla and Latin fonts, and a translation layer so a Bangla UI is only a translation later.
**Done when:** `design.md` covers type, color, spacing, tables, forms, and dialogs; base components handle focus and keyboard; every UI string goes through the translation layer.
- [ ] Design it (spec): `/architect design system & UI foundation`

### 5. Command dispatcher & local accounts · needs a decision · GA
One typed command entry point that resolves the session, checks permission and license gate, runs a transaction, and emits change events; plus users, roles, custom roles, permissions, login, idle lock. This is the seam LAN mode plugs into later.
**Done when:** the UI can only reach the core through the dispatcher; a denied permission is rejected in the core, not just hidden; a repeated command with the same idempotency key returns the first result; at least one active Owner always exists.
- [ ] Design it (spec): `/architect command dispatcher & local accounts`

### 6. Audit log · needs a decision · GA
Insert only audit trail with a hash chain, written inside the same transaction as the change, with sensitive fields removed.
**Done when:** every sensitive command writes an audit row; editing or deleting an audit row is rejected; a broken chain is detected by a diagnostics check.
- [ ] Design it (spec): `/architect audit log`

### 7. Migration engine · needs a decision · GA
Numbered forward migrations run on a copy, validated (integrity, foreign keys, money totals), then swapped in atomically, with the old file kept.
**Done when:** a failing migration leaves the original database untouched and shows the recovery screen; CI upgrades every released fixture database to the latest schema.
- [ ] Design it (spec): `/architect migration engine`

### 8. Setup wizard & key management · needs a decision · GA
First run: create institution, choose a data folder (non system drive recommended), Owner account, printed Recovery Sheet, backup destination, trial start. Random database key wrapped by the OS for daily use and by the recovery key for disaster recovery.
**Done when:** a fresh install reaches the dashboard offline; the database cannot be opened by copying the files elsewhere; the recovery key opens a copied database and resets the Owner password.
- [ ] Design it (spec): `/architect setup wizard & key management`
