# Coachify License Signer

## Overview

Isolated cryptographic service holding private Ed25519 signing keys for desktop license envelopes, revocations, and Tauri updater release manifests. Runs in key isolation with HMAC-SHA256 service authentication and an append-only audit log.

## Key files

| File | Owns |
|---|---|
| `crates/license_signer/src/lib.rs` | Key custody, HMAC caller authentication, Ed25519 signing routines, and append-only `signer_audit_log` |

## Commands

```bash
# Run tests
cargo test -p coachify_license_signer
```

## Conventions

- The private Ed25519 signing keys (`SEED_K2026A`, `SEED_K2026B`, `SEED_U2026A`, `SEED_U2026B`) live exclusively in this crate, never in `cloud_api` or the web app.
- All signing requests require HMAC-SHA256 authentication over the service boundary.
- Every signature operation is audited into the append-only `signer_audit_log` table protected by SQLite abort triggers.
- Public keys and wire envelopes (`LicenseEnvelope`, `SignedRevocationEnvelope`, `UpdaterManifestEnvelope`) use canonical UTF-8 JSON payloads and hex signatures.

_Drafted by /sync from the introducing change, worth a quick human pass._
