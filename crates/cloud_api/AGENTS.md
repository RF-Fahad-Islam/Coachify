# Coachify Cloud API

## Overview

Commercial licensing backend, vendor administration engine, customer portal API, payment gateway fulfillment, and desktop update distribution service.

## Key files

| File | Owns |
|---|---|
| `crates/cloud_api/src/lib.rs` | Schema definitions, admin auth (Argon2id + TOTP), activation and refresh endpoints, billing ledger, customer portal, payment gateway adapters, and hash-chained audit log |

## Commands

```bash
# Run unit and integration tests
cargo test -p coachify_cloud_api
```

## Conventions

- Private Ed25519 signing keys never enter this crate; all cryptographic license signing delegates to `coachify_license_signer` over authenticated HMAC service calls.
- Admin actions are recorded in a SHA-256 hash-chained `admin_audit_log` with SQLite immutability abort triggers.
- Versioned desktop endpoints (`/v1/activate`, `/v1/refresh`, `/v1/updates/{channel}`) transmit minimal telemetry only (`installation_id`, `license_id`, `current_serial`, `app_version`, `active_students`). No student or financial records ever leave customer PCs.
- Self-service PC hardware transfers are capped at 3 per 12-month rolling window per license.
- Payment gateway providers implement the trait-based `PaymentGatewayProvider` with HMAC-SHA256 webhook signature verification.

_Drafted by /sync from the introducing change, worth a quick human pass._
