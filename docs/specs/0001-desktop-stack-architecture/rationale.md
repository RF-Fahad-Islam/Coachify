# 0001. Desktop stack and architecture: rationale

Decision record for [index.md](index.md). Read by people and by `/architect` on update or supersede, not during a build.

## Context

Coachify is a standalone, offline first Windows app that runs a coaching center: students, fees and cashier, attendance, exams, SMS, backups (basis: `docs/scope/index.md`). Version 1.0 is single PC. LAN multi PC ships in 1.1 for Local Pro, and the seam for it must exist from day one (basis: review Section 20, decision 2). It must not depend on any NF product.

Forces that shape the stack:

- **Site conditions**: 4 GB RAM PCs with spinning disks, often running a browser too. Power cuts without a UPS. Shared Windows accounts. Technicians who reformat drive C. PCs that never see the internet (basis: review Section 1, risk 6).
- **Printing is the product**: receipts print silently to a chosen printer on A4, A5, or 80 mm thermal paper, and Bangla must be shaped correctly on paper and in PDF. This is the riskiest technical bet (basis: review Section 1, risk 4).
- **Money and data safety**: a committed payment must survive a power cut, a copied database must not open elsewhere, and business rules must be testable without a UI (basis: review Sections 4 and 11).
- **Team**: small, working with AI agents, so the stack must be one agents write well and one person can operate at 2am.
- **Performance targets** on the reference PC: cold start under 6 s, search under 150 ms at p95, memory under 400 MB (basis: review Section 16).

Not deciding means feature work starts on an unproven print path. Finding out in month 6 that silent printing does not work would force a rewrite of every screen that prints.

## Options considered

### Option 1: Tauri 2 + React + Rust core + SQLCipher (spike gated)

Tauri shell over WebView2, React UI, a plain Rust core crate, and rusqlite with bundled SQLCipher.

**Pros**: smallest installer and memory. Rust in process for strictly typed money logic and easy SQLCipher bundling. The core runs headless and hosts LAN later.
**Cons**: no maintained print plugin, so silent printing is custom WebView2 COM work. Two languages.

### Option 2: Electron + React + Rust core via napi-rs

The same React UI and Rust core, with Electron as the shell and the core loaded as a native Node module.

**Pros**: `webContents.print` supports silent printing to a named printer with a page size out of the box (still true in Electron 44). The same core is kept.
**Cons**: roughly twice the memory and a much bigger installer. A native module rebuild on every Electron upgrade. Two runtimes (Node and Rust) in one app.

### Option 3: Electron + React + Node core

An all TypeScript stack, with SQLCipher through a native Node SQLite module.

**Pros**: one language, the largest ecosystem, the best agent output, the best web stack printing.
**Cons**: money logic without Rust's strict typing. Native SQLite modules must be rebuilt on each upgrade. Memory cost on 4 GB PCs. The core cannot easily become a lean Windows service.

### Option 4: .NET WPF

A native Windows UI in C# with SQLite.

**Pros**: the strongest native printing and Windows reliability. One language.
**Cons**: a modern dense UI costs more work. SQLCipher needs a community or commercial bundle. Agents write noticeably weaker XAML. No reuse for any future non Windows client.

WinUI 3, Flutter desktop, Avalonia, and a local server with a browser were rejected in the review's stack comparison. The reasons: packaging friction, weak Bangla shaping in PDF, a small ecosystem, and no silent printing (basis: review Section 2, Stack comparison).

## Rationale

Option 1 wins on the constraints that are hardest to change later: memory on 4 GB office PCs, a small offline installer, and a Rust core that holds money logic, runs headless in tests, and can serve the LAN in 1.1 without a rewrite. Its one real weakness is printing, and that is a known, bounded risk. WebView2 exposes `PrintAsync` with full print settings, and `webview2-com` reaches it from Rust. So you prove it in a short spike on real hardware instead of assuming it.

The spike and the stack are cut to MVP size on purpose (your call, 2026-09-20: MVP first, no overengineering). The gate proves only the two bets that could change the stack: silent print and encrypted search. Thermal, PDF, DPAPI, the offline installer, the auto updater, and end to end automation are all real 1.0 needs, but none of them can change the stack choice. They are proven inside the features that build them.

Option 2 is the fallback rather than Option 3. When the core is a separate Rust library behind one `rpc` entry point, swapping the shell costs days and keeps every line of core and UI. Falling back to a Node core would throw away the core and the typing advantage for money. Option 4 gives up the React ecosystem and agent quality, which matter more for this team than native polish.

The finer picks follow two rules: reuse what the chosen layers already pair with, and prefer boring, stable versions. That's why the core is synchronous (a single writer makes async pointless for SQLite), `specta` is used without `tauri-specta` (which is still a release candidate), `ureq` with `rustls` is chosen (no `tokio`, no reliance on a broken Windows certificate store), vendored OpenSSL is used (nothing to install anywhere), and the migration runner is your own (off the shelf runners migrate in place) (basis: review Sections 4 and 10).

## Landscape check (2026-09-20)

Run once during the design conversation. Cached at `docs/.agent-cache/research/desktop-stack-landscape.md`.

- Tauri 2.11.5 (July 2026) has no built in silent printing. The community `tauri-plugin-printer-v2` has been unmaintained since May 2024. Direct WebView2 `ICoreWebView2_16.PrintAsync` through `webview2-com` is the path.
- rusqlite's bundled SQLCipher includes FTS5. The trigram tokenizer needs SQLite 3.34 or later, which current SQLCipher includes. Confirm in G2.
- NSIS `webviewInstallMode: offlineInstaller` is current. The `tauri-plugin-updater` signature check is mandatory.
- `tauri-specta` v2 is still a release candidate (rc.25, May 2026). `specta` and `ts-rs` are stable.
- Electron 44.4.3 (September 2026) still supports silent print with `deviceName` and `pageSize`. napi-rs is stable.

## References

**Project sources**:
- `docs/architecture-review.md`: Sections 2 (critique and stack comparison), 4 (desktop architecture), 5 (LAN seam), 10 (updates), 16 (performance targets), 18 (repositories and layout), 20 (decisions)
- `docs/scope/index.md` and `docs/scope/foundation.md`: feature 1 intent and done when

**Practices & standards**:
- Modular monolith with a headless core (one program split into clear modules, business logic independent of the UI)
- Single writer SQLite with WAL
- Spike before commit for the riskiest technical bet
- Reproducible builds through pinned toolchains and lockfiles

**Links** (web verified during the landscape check):
- Tauri releases: https://v2.tauri.app/release/
- Tauri updater plugin: https://v2.tauri.app/plugin/updater/
- SQLCipher: https://github.com/sqlcipher/sqlcipher
- tauri-specta releases: https://github.com/specta-rs/tauri-specta/releases
- Electron releases: https://releases.electronjs.org/release
