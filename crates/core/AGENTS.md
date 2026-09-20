# Coachify Core

## Overview

Pure Rust domain library holding all business logic, local database operations, and data models. Runs headless with zero Tauri or UI dependencies.

## Key files

| File | Owns |
|---|---|
| crates/core/src/lib.rs | Single RPC command handler and Specta type definitions |
| crates/core/src/db.rs | SQLite connection pool, PRAGMA setup, and migrations |

## Commands

```bash
# Run unit tests
cargo test -p coachify_core

# Export TypeScript contract types
cargo run -p xtask export-types
```

## Conventions

- Domain logic stays in core, never in the UI or Tauri shell.
- Return errors as `thiserror` enums mapped to code, params, and optional field.
- Repositories use plain SQL mapped to Rust structs, no heavy ORM.
- Database access uses write ahead logging mode with immediate transactions.

## Gotchas

- Release builds fail to start without real encryption key management.
- The core has no async runtime, all operations run synchronously.

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
