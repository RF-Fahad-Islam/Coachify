# Coachify

## Stack

- **Language / Runtime**: Rust 1.85+, TypeScript 5.7+, Node 22+
- **Framework**: Tauri 2, React 19, Vite
- **Key dependencies**: `rusqlite`, `specta`, `serde`, `tailwindcss`
- **Package manager**: Cargo (Rust workspace), pnpm (UI)

## Build approach

Tracer Bullet (prove one real thread through every layer first, then thicken it slice by slice).

## Commands

```bash
# Dev app
pnpm dev

# Build UI and core
cargo build --workspace

# Run tests
cargo test --workspace

# Sync TypeScript contract
cargo run -p xtask export-types

# Verify contract freshness without writing
cargo run -p xtask check-types

# Verify released migrations immutability
cargo run -p xtask check-migrations

# Lint and format
pnpm lint
pnpm format:check
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Specs

Stored in `docs/specs/`. Format: `docs/specs/NNNN-title.md`.

## Rules

- Organized around domain areas; domain logic lives in `crates/core`, never in the UI.
- The UI reaches the core exclusively through one typed `rpc` call.
- Strict type safety everywhere, no `any` in TypeScript and exhaustive error handling in Rust.
- No new abstraction without two callers.
- Released migrations are immutable: never edit an already released migration file.
- Map errors with `thiserror` in core to `{ code, params, field }` for client consumption.
- Repositories use plain SQL mapped to Rust models, no ORMs.
- Conventional commit messages (`feat:`, `fix:`, `docs:`, `chore:`).
- Document public library APIs and boundary contracts.
- Named exports only in TypeScript, no default exports.
- Design system: build all UI to `design.md` (art direction and the maximalist product bar); token values live in CSS.

## Git

- integration: on
- branch prefix: feat/
- commit: per-milestone

## Agent skills

- [rust-best-practices](.claude/skills/rust-best-practices/): `apollographql/skills`, Rust idioms, patterns, and safety
- [shadcn](.claude/skills/shadcn/): `shadcn/ui`, accessible UI component primitives
- [tailwind-4-docs](.claude/skills/tailwind-4-docs/): `lombiq/tailwind-agent-skills`, Tailwind CSS v4 styling
- [tanstack-query](.claude/skills/tanstack-query/): `tanstack-skills/tanstack-skills`, query caching and state sync
- [tauri-v2](.claude/skills/tauri-v2/): `nodnarbnitram/claude-code-extensions`, desktop shell and window management
- [vitest](.claude/skills/vitest/): `antfu/skills`, UI unit and component testing

Declined: `vercel-react-best-practices`, `tanstack-table`
MCP servers: `mcp-server-tauri` (recommended), `sqlite` (recommended)

## Context files

- [crates/core/AGENTS.md](crates/core/AGENTS.md): Pure Rust domain library, database connection, and migrations
- [ui/AGENTS.md](ui/AGENTS.md): React and TypeScript interface and view rendering
- [src-tauri/AGENTS.md](src-tauri/AGENTS.md): Thin Tauri 2 native shell and single RPC command bridge
- [crates/license_signer/AGENTS.md](crates/license_signer/AGENTS.md): Isolated Ed25519 signer service and audit log
- [crates/cloud_api/AGENTS.md](crates/cloud_api/AGENTS.md): Cloud commercial API, activation, portal, and checkout service
- [Landing pages/AGENTS.md](Landing%20pages/AGENTS.md): Commercial website, marketing pages, customer portal, and downloads

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
