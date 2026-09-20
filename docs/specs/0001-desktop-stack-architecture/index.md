# 0001. Desktop stack and architecture

**Date**: 2026-09-20
**Status**: In Progress

## Summary

Coachify's desktop app will be built with Tauri 2 (a small Windows app shell that shows a web UI through the built in Edge WebView2), a React and TypeScript interface, and all business logic in a plain Rust library (the "core") that stores data in an encrypted SQLite database (SQLCipher). The choice holds only if a spike of about one week on a real low end Windows 10 PC proves the two risky bets: silent printing with Bangla, and encrypted search. The rest gets proven later, inside the features that need it. If printing fails, only the shell changes to Electron: the same React UI and the same Rust core stay. The UI talks to the core through exactly one typed `rpc` call, which is what makes LAN mode in 1.1 a transport and not a rewrite.

## Decision

**Chosen option**: Option 1: Tauri 2 + React + Rust core + SQLCipher, spike gated (reasoning and all options in [rationale.md](rationale.md)).

You build the 1.0 desktop app as one process with three layers (React UI, thin Tauri shell, Rust core library) on the stack below. You commit to it only after the go or no go gate passes. If the print proof fails, you swap the shell to Electron and load the same Rust core through napi-rs (a bridge that lets Node call Rust).

**Implementation skills**: `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`). Where a skill conflicts with this spec, this spec wins. For example, the `tauri-v2` skill's pattern of many `#[tauri::command]` functions does not apply here: this app exposes one `rpc` command.

## Proposed stack

Versions: take the latest stable release of each at scaffold time, then pin it (see *Toolchain*). As of 2026-09-20 that means Tauri 2.11.x.

| Layer | Choice | Reason |
|---|---|---|
| Target OS | 64 bit Windows 10 (1809+) and Windows 11 only | Decision 6 in the review. One build target, one test matrix. |
| Architecture pattern | One process, three layers: React UI → Tauri shell → Rust `core` crate. A modular monolith (one program split into clear modules) | Small team, one PC per install. The core has no Tauri dependency, so it runs headless in tests, can become a Windows service in V2, and can serve LAN in 1.1. |
| Desktop shell | Tauri 2, kept thin: windows, tray, file dialogs, printing, updater, forwarding `rpc` to the core | Small installer and low memory on 4 GB PCs. Rust runs in the same process as the core. |
| Shell fallback | Electron shell + the same Rust core via napi-rs + the same React UI | Triggered only by a failed print proof. The core and UI written during the spike carry over, so nothing is thrown away. |
| UI language & build | React + TypeScript (strict mode) + Vite | Largest ecosystem, best agent output. Vite is Tauri's default frontend tooling. |
| Styling | Tailwind CSS v4, design tokens as CSS variables | Fast iteration. Print CSS (`@page`) for receipts sits alongside it. The look itself is feature 4. |
| Component primitives | shadcn/ui on Radix (source copied into `ui/`, owned by you) | Accessible focus and keyboard handling, with no heavy runtime library to fight. |
| Data tables | TanStack Table + TanStack Virtual | Headless, so it fits the design system. Row virtualization fits the rule that the core pages and the UI gets one page. |
| Forms | React Hook Form + Zod | Handles long forms well. Zod gives friendly early messages only: Rust validation is the authority. |
| Routing | TanStack Router (hash or memory history) | Typed routes and search params, so filters, paging, and the selected record live in typed state. |
| UI data cache | TanStack Query, keyed by command. Core change events invalidate matching keys | A cache of core results, never the source of truth. Works unchanged over LAN in 1.1. |
| Translation layer | i18next + react-i18next, typed keys, JSON catalogs | Every UI string goes through it from day one, so the Bangla UI in 1.1 is only a translation. |
| UI ↔ core contract | One Tauri command, `rpc`. Request and response types derived with `specta` in Rust and exported to a generated TS file in a build step. CI fails if the generated file is stale | One entry point is the LAN seam. Stable `specta` avoids depending on `tauri-specta`, which is still a release candidate. |
| Error contract | `thiserror` enums in the core, mapped to `{ code, params, field? }` (for example `due_changed`, `permission_denied`), generated into TS. The UI translates codes through i18n | The UI can react to specific errors, and the Bangla UI later needs no change in the core. |
| Core language & model | Rust, synchronous. One writer thread owns the write connection and takes commands from a queue. A small pool of reader connections. One job runner thread. No async runtime (`tokio`) in the core | SQLite allows one writer anyway. A single owner removes "database is locked" from normal operation. It is also the simplest model to test. |
| Database | SQLite through `rusqlite` with `bundled-sqlcipher-vendored-openssl` | Bundles SQLCipher with FTS5 and trigram, exposes the online backup API that snapshots need, and nothing needs installing on customer PCs or CI. |
| Search | SQLite FTS5 with the trigram tokenizer (matches any 3+ character substring) for names, IDs, and phone numbers | Built in, and fast enough at the 25,000 student ceiling. Bangla behavior is proven in the spike. |
| Query style | Plain SQL in one repository module per domain area, mapped to Rust structs | Transparent and fast, and agents write it well. No ORM (a library that writes SQL for you). |
| Migrations | Own small runner: numbered SQL files plus optional Rust data steps, run on a copy, validated, then swapped in | Off the shelf runners migrate in place, which the review rules out. Designed in feature 7. |
| Background jobs | One `jobs` table (with a lease, so a crashed job is picked up again) plus the runner thread | Database backed. No queue library, no event bus. |
| Document storage | Content addressed vault on disk (each file named by its SHA256 hash) under the data folder | Files stay out of the database. Covered in the review, Section 4. |
| Outbound HTTP | `ureq` with `rustls` and bundled root certificates | Synchronous, so it fits the runner thread. It doesn't depend on a PC's broken Windows certificate store or pull in `tokio`. |
| Date & time | `jiff` with its bundled time zone database. Store the UTC instant plus a separate `business_date` (Asia/Dhaka civil date) | Its types separate an instant in time from a calendar date. The bundled tz data doesn't rely on Windows zone names. |
| IDs | UUIDv7 as text (`uuid` crate, `v7` feature). Human facing numbers are separate columns | Time ordered, so inserts stay index friendly. Settled in the review. |
| Password hashing | Argon2id (`argon2` crate) | Current best practice. Details in feature 5. |
| License signature | Ed25519 verification (`ed25519-dalek`) | Matches the license format in the review, Section 9. Details in feature 32. |
| Windows integration | `windows` crate (official Microsoft bindings): DPAPI in machine scope, power requests (keep the PC awake), and `webview2-com` for printing | One dependency family covers COM, DPAPI, and WebView2. |
| Printing | Templates are React components with print CSS, rendered in a hidden window, then printed through WebView2 `PrintAsync` with `ICoreWebView2PrintSettings` (printer name, paper, margins, copies) via `webview2-com`. Thermal printers go through their Windows driver as normal pages | No maintained Tauri print plugin exists, so the shell calls WebView2 directly. Raw ESC/POS (the thermal printer command language) cannot render Bangla. |
| PDF | WebView2 print to PDF | The PDF matches the printed page exactly, Bangla included. |
| Fonts | Bundled open licensed Bangla and Latin fonts. The exact choice is feature 4 | Never depend on fonts a customer's PC happens to have. |
| Logging | `tracing` with a span per `rpc` command, a redaction layer, and JSON lines rotated daily under `logs\` | Structured and redacted. Diagnostics export reads the same files. |
| Observability | Local logs plus an opt in diagnostics export you review before sending. No automatic telemetry or crash upload | Privacy is a selling point (review Section 12). |
| Serialization | `serde` + `serde_json` | Standard. The same envelope travels over IPC now and over HTTPS in 1.1. |
| Installer | MVP: one Tauri NSIS installer, per machine install, copied by USB. Later (feature 24): an offline variant with `webviewInstallMode: offlineInstaller` | Offices share Windows accounts. One installer handles fresh install and upgrade. |
| Updates | MVP: none, you install new builds by hand. Later (feature 24): `tauri-plugin-updater` with its signature check always on, plus channels and a pre update backup | You install the pilots yourself anyway. Deferring it removes key management from the MVP. |
| Code signing | None for pilot builds. OV certificate with cloud or hardware backed signing before the first sale (feature 36) | Decision 10 in the review. |
| Security posture | Tauri capabilities expose only `rpc`, print, and the file dialog. No fs, shell, or http plugins for the UI. Strict CSP, no remote content, devtools off in release builds | The UI can reach nothing except the core. Enforced from the scaffold, never loosened. |
| Package managers | Cargo workspace + pnpm | pnpm is strict about dependencies and fast on Windows. |
| Toolchain | `rust-toolchain.toml` pins an exact stable Rust. Node LTS is pinned via `.nvmrc` and `packageManager`. `Cargo.lock` and `pnpm-lock.yaml` are committed | Reproducible release builds. |
| CI | GitHub Actions on hosted Windows runners, actions pinned to exact commit SHAs. Caches for Rust and the vendored OpenSSL build | Builds, tests, and bundles on the real target OS. |
| Rust tests | `cargo-nextest` + `proptest` | Fast, isolated runs. `proptest` powers the property based ledger test (random sequences checked against invariants). |
| UI tests | Vitest + Testing Library against a mocked `rpc` | Fast component tests with no shell needed. |
| End to end tests | MVP: none, you check the real app by hand (`/check verify`). Before 1.0 release hardening: WebdriverIO driving the built app on Windows through `tauri-driver` | Core tests cover the business rules. Real app automation is only required by the 1.0 release criteria. |
| Lint & format | `rustfmt`, `clippy` with warnings as errors, and a TS linter and formatter chosen in feature 2 | Feature 2 (`/audit`) captures the exact tools and rules. |

### Repository layout

This folder becomes the `desktop` repository. The vendor cloud lives in a separate `cloud` repository with its own spec (Slice 7). The only contract between the two is the license and update metadata formats, pinned by shared test vectors committed to both.

```
Coachify/                 (desktop repo root)
  AGENTS.md
  docs/scope/  docs/specs/  docs/architecture-review.md
  crates/core/            package `coachify_core`: domain, db, migrations runner, permissions, audit, jobs, backup, license checks
  xtask/                  dev tasks, starting with `export-types`
  src-tauri/              thin shell: windows, tray, rpc bridge, printing, updater
  ui/                     React app and print templates
  migrations/             numbered SQL, never edited after release
  tests/fixtures/         one database per released version, test vectors
```

One `core` crate with a module per area. Split it only when compile times hurt. `crates/lan/` arrives in 1.1.

### Boundary contract (the LAN seam)

- The UI calls exactly one generated function, `rpc(command, payload, idempotencyKey)`. It never sees file paths, SQL, secrets, or license internals.
- The response is `{ ok: true, data }` or `{ ok: false, error: { code, params, field? } }`.
- The core publishes change events (a topic plus IDs, for example `students changed: [ids]`) that the shell forwards to the UI.
- The dispatcher pipeline (session, permission, license gate, transaction, audit, events) is designed in feature 5. This spec fixes only its entry shape.
- The empty scaffold proves the seam end to end. One trivial command (for example `system.ping`) travels UI → `rpc` → core → SQLCipher database and back, with the TS types generated from Rust.

### Scaffold settings (throwaway until feature 8)

- **Database key**: a fixed dev key compiled only into debug builds (`cfg(debug_assertions)`). A release build refuses to start until feature 8 provides real key management.
- **Database path**: `%LOCALAPPDATA%\Coachify\dev\dev.db`, clearly a dev location. The real data folder (non system drive recommended) comes from the setup wizard in feature 8.
- **Type export**: `cargo xtask export-types` writes `ui/src/gen/rpc.ts` from the `specta` types. The `pnpm gen` script calls it. CI runs it and fails on any git diff, so the contract can't drift.
- **Crate name**: the core package is `coachify_core` (folder `crates/core`), which avoids clashing with Rust's built in `core` crate.

## Go or no go gate (Phase 0 spike)

The decision is committed only when both proofs below pass on a **real low end Windows 10 22H2 PC** (a physical 4 GB RAM, older Core i3 class PC). Start from a clean Windows install with no WebView2. Let the app's NSIS installer install WebView2 through its bootstrapper, then run G1, so the proof tests what a customer PC actually gets. **Timebox: about 1 week.** Keep it to these two. They are the only bets that could force a different stack.

| # | Proof | Passes when |
|---|---|---|
| G1 | Silent print | On your laser printer: no dialog appears, the job goes to the named printer, A4 and A5 are each honored, and Bangla text is shaped correctly (conjuncts and vowel signs) with a bundled font. |
| G2 | Encrypted search | A SQLCipher database with FTS5 trigram builds in Windows CI. A copied database file will not open without the key. Substring search works on a Latin and a Bangla name. |

**Proven later, inside the feature that needs it** (not part of this gate): 80 mm thermal, margins, and copies (feature 15, print engine). Bangla PDF (feature 15). DPAPI key wrapping (feature 8). Offline installer and signed updates (feature 24).

**Fallback rule.** If G1 is not solid by the end of the timebox, switch the shell to Electron (`webContents.print` with `silent`, `deviceName`, and `pageSize`), load the Rust core through napi-rs, and rerun G1 on Electron. A G2 failure is fixed within the stack, not by switching. Record the outcome and date in the Go or no go record below before any Slice 1 work starts.

### Go or no go record

- Result: _pending_
- Date and PC:
- Notes (per proof):

## Consequences

**Positive**:
- The same `rpc` envelope serves IPC now and HTTPS in 1.1. LAN adds a transport, not a rewrite.
- All business rules are testable headless against the core. Money logic gets Rust's strict typing.
- The installer is small and memory stays low, which fits 4 GB office PCs running a browser alongside.
- The fallback keeps the core and UI, so a failed print spike costs days, not months.

**Negative / tradeoffs**:
- Printing is custom COM work (Windows component interfaces) against WebView2. No plugin maintains it for you, so it is your code to own.
- Two languages (Rust and TypeScript) and a type generation step. A stale generated file must fail CI or the contract drifts.
- The vendored OpenSSL build makes cold CI builds slow until the cache warms.
- WebView2 updates itself through Windows, so print or render behavior can change under you. The print matrix must rerun on each release.
- A synchronous core means the 1.1 LAN server needs a thread or small runtime of its own at the edge.
- Pilot builds are unsigned, so SmartScreen warnings appear until feature 36.

**Neutral**:
- `specta`, `jiff`, and TanStack Router are newer than their best known alternatives. All three are stable, and each is replaceable behind a small surface.
- Feature 2 (`/audit`) writes `AGENTS.md` from this stack. Features 3 to 8 each design their part inside it.

## Follow-up

- [ ] Get one common 80 mm USB thermal printer (Xprinter XP-80 class, or borrow a pilot's) before feature 15, the print engine.
- [ ] Fill the Go or no go record above when the spike ends. If the fallback runs, update the Decision and Proposed stack rows through `/architect` (supersede or update).
- [ ] Root `AGENTS.md` doesn't exist yet. Feature 2 (`/audit`) should capture this stack and the guardrails: no business logic in the UI, only `rpc`, no new dependency without a spec line, locked Tauri capabilities, no edits to released migrations.
- [ ] Move `Implementation_plan.md` to `docs/` (or mark it historical). Its NF Cloud and NF Connect sections conflict with this spec and the standalone rule.
- [ ] The vendor cloud stack needs its own spec in the `cloud` repository when Slice 7 starts.
- [ ] Feature 2 (`/audit`) should list the six installed skills in root `AGENTS.md` `## Agent skills`. All are project wide. Offered but not picked: `vercel-react-best-practices`, `tanstack-table`.
- [ ] Connect the two MCP servers you chose (a config step you run yourself, following each project's README): `mcp-server-tauri` (github.com/hypothesi/mcp-server-tauri), which lets the agent see the running app, and a SQLite MCP. Note that the SQLite MCP cannot read SQLCipher encrypted files without the key, so it only helps with unencrypted test copies.

## Rationale

Reasoning, options, and the landscape check: see [rationale.md](rationale.md).
