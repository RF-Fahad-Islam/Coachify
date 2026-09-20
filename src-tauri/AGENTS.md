# Coachify Desktop Shell

## Overview

Thin Tauri 2 desktop shell providing native window management, system tray, and OS level dialogs. Exposes a single command bridge to the core library.

## Key files

| File | Owns |
|---|---|
| src-tauri/src/lib.rs | Single `rpc` command bridge and application runner |
| src-tauri/tauri.conf.json | Window configuration and strict content security policy |
| src-tauri/capabilities/default.json | Locked permission set |

## Commands

```bash
# Run desktop dev
cargo run -p coachify_desktop_lib --bin coachify
```

## Conventions

- Keep the shell thin; do not place institution business rules here.
- Tauri permissions expose only the single `rpc` command, print, and file dialogs.

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
