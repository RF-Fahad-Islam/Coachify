# Verify: Stack and architecture (spec 0001, updated 2026-09-20)

Steps derived from spec 0001 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [ ] Start the desktop app (`cargo run -p coachify_desktop_lib --bin coachify`) -> Main window displays "Coachify Desktop" with "Core Connected" green badge -> AC-1
- [ ] Click the "Send RPC Ping" button -> Sends `system.ping` through the single `rpc` bridge and updates roundtrip timestamp -> AC-1

## Commands
- [x] `cargo test --workspace` -> All tests pass, including the `system.ping` database transaction -> AC-1 (Passed 2026-09-20)
- [x] `cargo run -p xtask export-types` -> Exports TypeScript contract into `ui/src/gen/rpc.ts` -> AC-1 (Passed 2026-09-20)
- [x] `pnpm --dir ui build` -> TypeScript typecheck passes and Vite bundles the web frontend -> AC-1 (Passed 2026-09-20)
- [x] `cargo build --workspace` -> Compiles all workspace members and produces the `coachify.exe` binary -> AC-1 (Passed 2026-09-20)

## Acceptance criteria coverage
- AC-1 (empty scaffold builds and runs with one `rpc` round trip to core and database): covered by unit tests, type export, UI build, and manual ping.
