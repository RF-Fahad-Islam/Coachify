# Coachify UI

## Overview

React and TypeScript web interface running inside the Tauri desktop shell. Focuses strictly on presentation, user input, and receipt printing views.

## Key files

| File | Owns |
|---|---|
| ui/src/App.tsx | Root application view and connection verification |
| ui/src/gen/rpc.ts | Auto generated RPC contract types from Rust Specta |
| ui/src/main.tsx | React application bootstrap entrypoint |

## Commands

```bash
# Dev server
pnpm --dir ui dev

# Build and typecheck
pnpm --dir ui build
```

## Conventions

- No business logic in React components or hooks.
- Communicate with the backend only through the single typed `rpc` bridge.
- Use named exports only, no default exports.
- Use Tailwind CSS tokens and shadcn primitives for UI styling.

## Gotchas

- Do not edit `ui/src/gen/rpc.ts` by hand; generate it with `cargo xtask export-types`.

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
