# Coachify UI

## Overview

React and TypeScript web interface running inside the Tauri desktop shell. Focuses strictly on presentation, user input, and receipt printing views.

## Key files

| File | Owns |
|---|---|
| ui/src/App.tsx | Root application view, module tabs, and shell routing |
| ui/src/contexts/auth-context.tsx | Client auth state, session management, inactivity tracking, and heartbeat |
| ui/src/components/auth/startup-auth-page.tsx | Full-page first-run owner bootstrap and daily staff login |
| ui/src/components/auth/idle-lock-dialog.tsx | 15-minute inactivity lock overlay requiring password unlock |
| ui/src/components/cashier/cashier-desk-view.tsx | High-density keyboard-driven cashier desk for student lookups and fee collection |
| ui/src/components/attendance/ | Daily keyboard attendance sheet and monthly class attendance register |
| ui/src/components/communication/ | Outbox queue monitor, batch broadcast composer, and fee due reminders |
| ui/src/components/exams/ | Examination management hub, spreadsheet mark entry grid, and result publish preview |
| ui/src/components/backup/ | Encrypted backup dashboard, creation triggers, and verification status |
| ui/src/components/restore/ | Restore wizard and database corruption banner |
| ui/src/components/print/ | Printable document and receipt templates for WebView2 silent print engine |
| ui/src/gen/rpc.ts | Auto generated RPC contract types from Rust Specta |
| ui/src/main.tsx | React application bootstrap entrypoint |

## Commands

```bash
# Dev server
pnpm --dir ui dev

# Build and typecheck
pnpm --dir ui build

# Lint, format check, and tests
pnpm --dir ui lint
pnpm --dir ui format:check
pnpm --dir ui test
```

## Conventions

- No business logic in React components or hooks.
- Communicate with the backend only through the single typed `rpc` bridge.
- Use named exports only, no default exports.
- Use Tailwind CSS tokens and shadcn primitives for UI styling.

## Gotchas

- Do not edit `ui/src/gen/rpc.ts` by hand; generate it with `cargo run -p xtask export-types`.

_Drafted by /audit from the repo, worth a quick human pass. Edit freely: once a line stops matching this draft, later runs treat it as curated and will flag rather than overwrite it._
