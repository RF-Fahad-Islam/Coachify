# Verification — 0042. Website, Customer Portal, Code Signing, and Release Channels

## Automated Verification Commands

```bash
# Verify isolated signer (Ed25519 updater release signing & dual key recovery drills)
cargo test -p coachify_license_signer

# Verify cloud API (customer portal authentication, OTP lockout, tenant isolation, zero-rebuild promotion, and blocked releases)
cargo test -p coachify_cloud_api

# Verify core dispatcher and RPC commands
cargo test -p coachify_core --lib

# Sync & check TypeScript contract freshness
cargo run -p xtask check-types

# Verify released migrations immutability
cargo run -p xtask check-migrations

# Lint and check formatting for UI and Landing pages
pnpm lint
pnpm format:check

# Run full Vitest suites (UI & Landing pages)
pnpm test

# Rust formatting and clippy clean checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Test Coverage Checklist

- [x] **AC-1**: Customer portal login via phone/email + Argon2id password + 6-digit OTP (10m expiry, 5-attempt lockout for 15m) and SHA256 hashed 24h bearer tokens (`customer_portal_sessions`).
- [x] **AC-2**: Authenticated customer portal access (`/portal`) strictly scoped to `session.customer_id`, offering `.coachifylicense` download, verified Windows installer download, paid BDT invoices (`INV-YYYY-NNNNN`), and bound PC installations.
- [x] **AC-3**: Telemetry and privacy transparency page (`/privacy`) displaying exact 5 activation/refresh fields (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`) and 2 update query fields, certifying zero student PII leaves the PC.
- [x] **AC-4**: Download center (`/downloads`, `/download`) and help center (`/help`) with live channel release metadata, SHA256 checksum with copy button, and Authenticode signer badge.
- [x] **AC-5**: CI-gated Windows Authenticode and Tauri updater Ed25519 signing (`u2026a`) to `internal` channel in `.github/workflows/release.yml` with `release-internal` environment human approval gate.
- [x] **AC-6**: Zero-rebuild channel promotion (`POST /v1/releases/promote`) from `internal` to `beta` and `stable` (requiring step-up TOTP for `stable`), copying immutable release properties into `release_channel_heads`.
- [x] **AC-7**: Blocked and held release enforcement (`RELEASE_NOT_PROMOTABLE`, omitted as latest from `GET /v1/updates/{channel}`, and reported in `blocked_versions`).
- [x] **AC-8**: Dual-custody recorded recovery drills (`POST /v1/releases/recovery_drill` + `coachify_license_signer::verify_key_recovery`) stored in append-only `key_recovery_drills` with SQLite abort triggers.
- [x] **AC-9**: Strict `design.md` adherence across `Landing pages/` and `ui/src/components/vendor-admin/vendor-admin-view.tsx` with Sora typography, Inter tabular numbers, warm cream/ink/ember palette, and reusable primitives.
