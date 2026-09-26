# 0043. Online Pricing, Gateway Checkout, and Portal Device Management

**Date**: 2026-09-25
**Status**: Implemented & Verified

## Summary

Coachify adds self service online plan purchase and renewal (`Feature 67`) alongside customer portal PC device management (`Feature 68`) across `crates/cloud_api`, `crates/license_signer`, `Landing pages/`, `ui/`, and `crates/core`. Default commercial pricing is seeded with four admin configurable billing tiers: a 3 month plan at `1,500 BDT/month` (`4,500 BDT` total), a 6 month plan at `1,200 BDT/month` (`7,200 BDT` total), a 12 month annual plan at `1,000 BDT/month` (`12,000 BDT` total), and a full Lifetime license at `50,000 BDT` one time (`paid_through = '9999-12-31'`), all of which can be updated anytime from the Vendor Admin UI (`Plans & Pricing Studio`) to immediately update the website and checkout engine. Coaching center owners can pay via bKash, Nagad, or guided bank transfer to receive an instant 6 digit desktop pairing code, a signed `.coachifylicense` file, and a sequential `INV-YYYY-NNNNN` invoice, and can use `/portal` to view linked PCs, deactivate a lost PC into immediate read only mode, or replace a PC in one step.

## Structure

This umbrella specification coordinates two child specifications covering the remaining items of Commercial (Slice 12, Features 67 and 68):

1. [0043-pricing-and-checkout.md](0043-pricing-and-checkout.md): Feature 67 dynamic pricing catalog (`3 months` at `1,500 BDT/mo`, `6 months` at `1,200 BDT/mo`, `12 months` at `1,000 BDT/mo`, and `Lifetime` at `50,000 BDT`), easy Vendor Admin UI pricing configuration (`admin.prices.update_tier`, `GET /v1/pricing/catalog`), online checkout session creation (`POST /v1/checkout/create_session`), trait based Bangladesh payment gateway adapters (`SSLCommerz`, `bKash`, `Nagad`, and local sandbox), HMAC SHA256 webhook verification (`POST /v1/webhooks/payment/{provider}`), guided bank transfer submission and admin approval, and the interactive checkout flow in `Landing pages/`.
2. [0043-device-management.md](0043-device-management.md): Feature 68 portal device listing with last seen telemetry, step up OTP guarded device deactivation (`POST /v1/portal/devices/deactivate`), one step online or offline `REQ` replacement (`POST /v1/portal/devices/replace`), device label renaming (`POST /v1/portal/devices/rename`), `deactivated` refresh status propagation (`POST /v1/refresh`), and immediate `deactivated_read_only` enforcement in `crates/core/src/licensing.rs` (Migration 0023).

**Cross child contract**:
* **Admin configurable pricing catalog**: Every checkout session (`POST /v1/checkout/create_session`) and public pricing card (`GET /v1/pricing/catalog`) reads live pricing rows from `prices` in `crates/cloud_api` (`3_months` = `1,500 BDT/mo`, `6_months` = `1,200 BDT/mo`, `12_months` = `1,000 BDT/mo`, `lifetime` = `50,000 BDT`), editable with one click in Vendor Admin (`ui/src/components/vendor-admin/vendor-admin-view.tsx`).
* **Unified payment and license fulfillment pipeline**: Verified gateway webhooks (`bKash`, `Nagad`, `SSLCommerz`) and approved bank transfer orders in Feature 67 invoke the same transactional fulfillment function (`fulfill_verified_checkout_order`) as manual payments in Feature 63, writing to `payments`, `vendor_invoices` (`INV-YYYY-NNNNN`), `subscriptions` (setting `paid_through = '9999-12-31'` when `billing_cycle == 'lifetime'`), and `license_issues`, and calling `crates/license_signer` over `HMAC-SHA256`.
* **Shared installation slot and serial progression**: Both online checkout pre binding (`target_installation_id` in Feature 67) and portal device deactivation or replacement (Feature 68) mutate `licenses.bound_installation_id`, `licenses.current_serial`, `licenses.transfer_count_12m`, and `installations.status` atomically inside `crates/cloud_api`, ensuring at most one `active` host installation exists per `license_id` at any moment.
* **Immediate read only deactivation vs 14 day moved window**: When an owner replaces a PC via desktop self transfer (Feature 64), the old PC status is `moved` and it receives a 14 day transitional window; when an owner explicitly deactivates a lost or retired PC in `/portal` (Feature 68), the installation status becomes `deactivated` and `POST /v1/refresh` returns `status = 'deactivated'`, causing the desktop app (`crates/core`) to enter `deactivated_read_only` immediately with zero grace days while preserving full view, export, and backup access.

## Requirements

**User stories**:
* As a founder or operations administrator (`super_admin` or `ops_admin`), I want the commercial catalog pre configured with four default billing tiers (`3 months` at `1,500 BDT/month` = `4,500 BDT`, `6 months` at `1,200 BDT/month` = `7,200 BDT`, `12 months` at `1,000 BDT/month` = `12,000 BDT`, and `Full Lifetime License` at `50,000 BDT` one time) and an easy `Plans & Pricing Studio` in the Vendor Admin UI so I can change any monthly rate or lifetime price in Taka at any time and have the website and checkout update immediately.
* As a prospective coaching center owner on the Coachify website, I want to pick the 3 month, 6 month, 12 month, or Lifetime tier, enter my institution and owner details, pay via bKash or Nagad (or submit a bank transfer reference), and immediately receive my 6 digit desktop pairing code, downloadable `.coachifylicense` file, and paid `INV-YYYY-NNNNN` invoice.
* As an existing customer logged into `/portal`, I want to renew my subscription (3, 6, or 12 months, extending my existing `paid_through` without losing days) or upgrade to the `50,000 BDT` Lifetime license (or `Local Pro`) and have my office PC automatically pull the updated license on its next daily refresh or Check Now click.
* As an operations administrator (`ops_admin` or `super_admin`), I want pending bank transfer checkout orders to appear in a dedicated queue in Vendor Admin where I can verify the bank reference and slip and click Approve to trigger the exact same automatic license and invoice fulfillment pipeline.
* As an institution owner logged into `/portal`, I want to see all PCs linked to my license with their custom label, hostname, OS version, app version, active student count, and last seen timestamp, and rename any label for clarity.
* As an institution owner whose office PC was lost, damaged, or retired, I want to confirm a 6 digit OTP in `/portal` to either deactivate that PC (freeing the license slot and issuing a fresh 6 digit pairing code while forcing the old PC into immediate read only mode on next refresh) or replace it directly by pasting the new PC's offline request code (`REQ-XXXX-XXXX-XXXX-XXXX`) to download the bound `.coachifylicense` in one step.

**Acceptance criteria**:
* **AC-1**: Four default billing tiers (`3 months`, `6 months`, `12 months`, `Lifetime`) and easy Vendor Admin UI price editing: `crates/cloud_api` seeds the active `prices` catalog with `3_months` (`period_months = 3`, `monthly_rate_poisha = 150_000` [`1,500 BDT/mo`], `amount_poisha = 450_000` [`4,500 BDT`]), `6_months` (`period_months = 6`, `monthly_rate_poisha = 120_000` [`1,200 BDT/mo`], `amount_poisha = 720_000` [`7,200 BDT`]), `12_months` (`period_months = 12`, `monthly_rate_poisha = 100_000` [`1,000 BDT/mo`], `amount_poisha = 1_200_000` [`12,000 BDT`]), and `lifetime` (`period_months = 0`, `is_lifetime = 1`, `monthly_rate_poisha = 5_000_000`, `amount_poisha = 5_000_000` [`50,000 BDT`], unlocking full `local_pro` capabilities forever), while keeping `period_months IN (0, 1, 3, 6, 12)` compatible with legacy 1 month test records. The Vendor Admin `Plans & Pricing Studio` (`admin.prices.update_tier`) lets an admin edit any tier's monthly rate (`monthly_rate_bdt`, which auto calculates `total_bdt = monthly_rate_bdt * period_months`) or set a custom flat `total_bdt` override in place (archiving the previous price row with `active_to = now`, inserting the new active row, and logging to `admin_audit_log`), and `GET /v1/pricing/catalog` serves the updated tiers immediately to `Landing pages/` and checkout.
* **AC-2**: Public new purchase and authenticated portal renewal, lifetime conversion, or upgrade checkout sessions: `POST /v1/checkout/create_session` creates a `checkout_orders` row with a sequential `CHK-YYYY-NNNNN` order number and a 30 minute session timer (`expires_at`), computing integer `amount_poisha` from the active `prices` row for `3_months`, `6_months`, `12_months`, or `lifetime`, or computing the unused day credit from the customer's last confirmed payment (`remaining_paid_days = max(0, floor((subscriptions.paid_through - now_utc) / 86400))` excluding grace days; `unused_credit_poisha = floor(last_payment.amount_poisha * remaining_paid_days / (last_payment.period_months * 30))`; `amount_poisha = max(100, target_tier.amount_poisha - unused_credit_poisha)`) when upgrading an active subscription to `lifetime` or `local_pro`.
* **AC-3**: Trait based Bangladesh gateway integration, HMAC webhook verification, and mismatch audit logging: `crates/cloud_api` defines a `PaymentGatewayProvider` trait with adapters for `sslcommerz`, `bkash_direct`, `nagad_direct`, and `sandbox`; `POST /v1/webhooks/payment/{provider}` verifies the provider's `HMAC-SHA256` signature, records an immutable row in `payment_webhook_events` (`processed_status = 'applied'` when settled `amount_poisha` matches, `duplicate_ignored` on duplicate event/transaction ID, or `rejected` when `settled_amount_poisha != checkout_orders.amount_poisha`), and returns `WEBHOOK_AMOUNT_MISMATCH` (`422`) if the amount disagrees.
* **AC-4**: Idempotent automatic license, pairing code, and invoice fulfillment (including lifetime and late webhooks): When a verified webhook arrives for a `pending_payment` or `expired` order (so captured customer funds are never dropped), `fulfill_verified_checkout_order` executes in one SQLite transaction: creates or updates the customer and subscription, sets `paid_through = '9999-12-31'` when `billing_cycle == 'lifetime'` (or extends `paid_through` by `3`, `6`, or `12` months anchored to existing `paid_through` if before grace ends, else to payment date), writes `payments` and `vendor_invoices` (`INV-YYYY-NNNNN`), increments `current_serial`, generates a 6 digit `pairing_code`, requests an Ed25519 signature from `crates/license_signer`, stores the signed blob in `license_issues`, and marks the order `completed`. Duplicate webhooks return `200 OK` with `duplicate_ignored` and zero side effects.
* **AC-5**: Guided bank transfer submission and one click admin approval: `POST /v1/checkout/submit_bank_transfer` transitions a checkout order to `pending_bank_review` with `bank_reference` and `bank_slip_url`; when `ops_admin` or `super_admin` calls `admin.checkout.approve_bank_transfer`, it runs `fulfill_verified_checkout_order`, records a hash chained entry in `admin_audit_log`, and makes the signed `.coachifylicense` and `INV-YYYY-NNNNN` invoice immediately available in `/portal` and via `POST /v1/refresh`.
* **AC-6**: Portal device inventory with telemetry and label renaming: `GET /v1/portal/overview` (and `GET /v1/portal/devices`) returns all `installations` for the authenticated `customer_id` (`label`, `installation_id`, `hostname`, `os_version`, `app_version`, `active_students`, `first_seen_at`, `last_seen_at`, `status`) along with `transfer_count_12m`, `max_transfers_12m` (`3`), and recent `device_management_events`; `POST /v1/portal/devices/rename` updates `installations.label` and records a `rename_label` event.
* **AC-7**: Step up OTP portal device deactivation and slot release: `POST /v1/portal/devices/deactivate` requires an authenticated `customer_portal_sessions` bearer token plus a valid 6 digit OTP code (`customer_portal_credentials.otp_code_hash`), verifies `transfer_count_12m < 3`, marks the target installation `status = 'deactivated'` with `deactivated_at` and `deactivation_reason`, clears `licenses.bound_installation_id`, increments `transfer_count_12m` and `current_serial`, generates a fresh 72 hour 6 digit `pairing_code`, and appends an immutable row to `device_management_events`.
* **AC-8**: Immediate desktop read only mode upon `deactivated` refresh: When a desktop installation whose cloud row is `status = 'deactivated'` calls `POST /v1/refresh`, `crates/cloud_api` returns `RefreshStatus::Deactivated` with `deactivated_at` and `reason`; Desktop Migration 0023 (`migrations/0023_license_deactivation_state.sql`) and `crates/core/src/licensing.rs` persist `last_refresh_result = 'deactivated'` and `deactivated_at`, immediately transitioning the desktop dispatcher into `deactivated_read_only` mode (zero grace days) where mutations and outbound messages are blocked with error code `LICENSE_DEVICE_DEACTIVATED` while login, search, reports, printing, exports, and backups remain open.
* **AC-9**: One step portal device replacement (online pairing or offline `REQ` code) and 3 transfer cap enforcement: `POST /v1/portal/devices/replace` verifies the session bearer token, 6 digit step up OTP, and `transfer_count_12m < 3`, marks the old installation `deactivated`, increments `transfer_count_12m` and `current_serial`, and either binds a provided offline request code (`REQ-XXXX-XXXX-XXXX-XXXX` or `COACHIFY-REQ|v1|...`) via `crates/license_signer` to return a ready to download `.coachifylicense` file for the new PC, or issues a fresh 6 digit `pairing_code` for online activation. When `transfer_count_12m >= 3`, both `deactivate` and `replace` reject with `DEVICE_TRANSFER_LIMIT_EXCEEDED` (`remaining = 0`) and `/portal` offers a one click support reset request button.
* **AC-10**: Unified `design.md` and `Landing pages/` UI execution: The 4 tier Pricing Grid (`3 Months`, `6 Months`, `12 Months Annual`, `Full Lifetime`) and Checkout Modal in `Landing pages/`, the `Plans & Pricing Studio` and Pending Bank Queue in Vendor Admin (`ui/src/components/vendor-admin/vendor-admin-view.tsx`), and the Device Management studio in `/portal` use the `design.md` and `Landing pages/` visual system (`Sora` display typography, `Inter` body with `.tnum` tabular figures, `Noto Sans Bengali` for Bangla copy, warm cream `#faf7f2`, ink `#16110d`, ember `#f97316`, `WindowFrame`, `CtaButton`, `SectionTag`, and `Chip` primitives).

## Decision

**Chosen option**: Option 1: Admin Configurable 4 Tier Pricing Catalog (`3m`, `6m`, `12m`, `Lifetime`), Transactional `crates/cloud_api` Checkout & Device Engine, Trait Based Bangladesh Gateway Adapters, Isolated `crates/license_signer` Fulfillment, and Immediate `deactivated_read_only` Desktop Enforcement (Migration 0023)

We implement Features 67 and 68 inside `crates/cloud_api`, `crates/license_signer`, `Landing pages/`, `crates/core`, and `ui` by extending `prices` to support the 4 billing cycles (`3_months` at `1,500 BDT/mo`, `6_months` at `1,200 BDT/mo`, `12_months` at `1,000 BDT/mo`, and `lifetime` at `50,000 BDT`) with a live Vendor Admin editor (`admin.prices.update_tier`), adding `checkout_orders`, `payment_webhook_events`, and `device_management_events`, extending `installations` with `deactivated` status, adding Desktop SQLite Migration 0023 (`0023_license_deactivation_state.sql`), and routing every verified payment through a single transactional fulfillment pipeline.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`) · `vitest` (`antfu/skills`, `.claude/skills/vitest/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:

1. **Extended Pricing Catalog (`prices` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `plan_id TEXT NOT NULL REFERENCES plans(id)`
   * `billing_cycle TEXT NOT NULL CHECK (billing_cycle IN ('1_month', '3_months', '6_months', '12_months', 'lifetime'))`
   * `period_months INTEGER NOT NULL CHECK (period_months IN (0, 1, 3, 6, 12))` (`0` for `lifetime`, `1` for legacy monthly, `3` for 3 months, `6` for 6 months, `12` for annual)
   * `is_lifetime INTEGER NOT NULL DEFAULT 0 CHECK (is_lifetime IN (0, 1))`
   * `currency TEXT NOT NULL DEFAULT 'BDT'`
   * `monthly_rate_poisha INTEGER NOT NULL CHECK (monthly_rate_poisha > 0)` (seeded defaults: `150_000` [`1,500 BDT/mo`] for `3_months`, `120_000` [`1,200 BDT/mo`] for `6_months`, `100_000` [`1,000 BDT/mo`] for `12_months`, `5_000_000` [`50,000 BDT`] for `lifetime`)
   * `amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0)` (seeded defaults: `450_000` [`4,500 BDT`] for `3_months`, `720_000` [`7,200 BDT`] for `6_months`, `1_200_000` [`12,000 BDT`] for `12_months`, `5_000_000` [`50,000 BDT`] for `lifetime`; auto computed as `monthly_rate_poisha * period_months` by default or overridden with a custom flat `total_bdt` in `admin.prices.update_tier`)
   * `active_from TEXT NOT NULL`
   * `active_to TEXT` (`NULL` indicates the currently active price for `(plan_id, billing_cycle)`)

2. **Checkout Orders (`checkout_orders` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `order_number TEXT UNIQUE NOT NULL` (sequential `CHK-YYYY-NNNNN`)
   * `customer_id TEXT NOT NULL REFERENCES customers(id)`
   * `subscription_id TEXT REFERENCES subscriptions(id)`
   * `plan_id TEXT NOT NULL REFERENCES plans(id)`
   * `price_id TEXT NOT NULL REFERENCES prices(id)`
   * `kind TEXT NOT NULL CHECK (kind IN ('new_purchase', 'renewal', 'upgrade', 'lifetime_purchase'))`
   * `billing_cycle TEXT NOT NULL CHECK (billing_cycle IN ('3_months', '6_months', '12_months', 'lifetime'))`
   * `period_months INTEGER NOT NULL CHECK (period_months IN (0, 3, 6, 12))`
   * `method TEXT NOT NULL CHECK (method IN ('bkash', 'nagad', 'bank_transfer'))`
   * `amount_poisha INTEGER NOT NULL CHECK (amount_poisha > 0)`
   * `gateway_provider TEXT NOT NULL CHECK (gateway_provider IN ('sslcommerz', 'bkash_direct', 'nagad_direct', 'manual_bank', 'sandbox'))`
   * `gateway_session_id TEXT`
   * `gateway_trx_id TEXT`
   * `bank_reference TEXT`
   * `bank_slip_url TEXT`
   * `target_installation_id TEXT`
   * `target_request_code TEXT`
   * `status TEXT NOT NULL CHECK (status IN ('pending_payment', 'pending_bank_review', 'completed', 'expired', 'failed'))`
   * `resulting_payment_id TEXT UNIQUE REFERENCES payments(id)`
   * `resulting_invoice_id TEXT UNIQUE REFERENCES vendor_invoices(id)`
   * `resulting_license_id TEXT REFERENCES licenses(license_id)`
   * `expires_at TEXT NOT NULL`
   * `completed_at TEXT`
   * `created_at TEXT NOT NULL`
   * `UNIQUE(method, gateway_trx_id)`

3. **Immutable Payment Webhook Events (`payment_webhook_events` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `checkout_order_id TEXT NOT NULL REFERENCES checkout_orders(id)`
   * `gateway_provider TEXT NOT NULL`
   * `gateway_event_id TEXT NOT NULL`
   * `gateway_trx_id TEXT NOT NULL`
   * `amount_poisha INTEGER NOT NULL`
   * `signature_verified INTEGER NOT NULL CHECK (signature_verified = 1)`
   * `raw_payload_json TEXT NOT NULL`
   * `processed_status TEXT NOT NULL CHECK (processed_status IN ('applied', 'duplicate_ignored', 'rejected'))`
   * `received_at TEXT NOT NULL`
   * `UNIQUE(gateway_provider, gateway_event_id)`
   * Protected by `BEFORE UPDATE` and `BEFORE DELETE` abort triggers (`prevent_update_payment_webhook_events`, `prevent_delete_payment_webhook_events`).

4. **Extended Installations (`installations` in `crates/cloud_api`)**:
   * Existing columns (`id`, `installation_id`, `customer_id`, `license_id`, `label`, `role`, `app_version`, `active_students`, `first_seen_at`, `last_seen_at`) plus:
   * `hostname TEXT NOT NULL DEFAULT ''`
   * `os_version TEXT NOT NULL DEFAULT 'Windows'`
   * `deactivated_at TEXT`
   * `deactivation_reason TEXT`
   * `replaced_by_installation_id TEXT`
   * `status TEXT NOT NULL CHECK (status IN ('active', 'moved', 'deactivated', 'revoked'))`
   * Partial unique index `CREATE UNIQUE INDEX IF NOT EXISTS idx_installations_one_active_host_per_license ON installations(license_id) WHERE role = 'host' AND status = 'active'`.

5. **Append Only Device Management Events (`device_management_events` in `crates/cloud_api`)**:
   * `id TEXT PRIMARY KEY`
   * `license_id TEXT NOT NULL REFERENCES licenses(license_id)`
   * `customer_id TEXT NOT NULL REFERENCES customers(id)`
   * `action TEXT NOT NULL CHECK (action IN ('deactivate', 'replace_online', 'replace_offline_req', 'rename_label', 'support_reset_request'))`
   * `old_installation_id TEXT NOT NULL`
   * `new_installation_id TEXT`
   * `new_label TEXT`
   * `reason TEXT NOT NULL`
   * `initiated_by_type TEXT NOT NULL CHECK (initiated_by_type IN ('portal_owner', 'admin'))`
   * `initiated_by_id TEXT NOT NULL`
   * `resulting_serial INTEGER NOT NULL`
   * `issued_serial_id TEXT REFERENCES license_issues(id)`
   * `created_at TEXT NOT NULL`
   * Protected by `BEFORE UPDATE` and `BEFORE DELETE` abort triggers (`prevent_update_device_management_events`, `prevent_delete_device_management_events`).

6. **Desktop SQLite Migration 0023 (`crates/core/migrations/0023_license_deactivation_state.sql`)**:
   ```sql
   ALTER TABLE license_refresh_state ADD COLUMN deactivated_at TEXT;
   ALTER TABLE license_refresh_state ADD COLUMN deactivation_reason TEXT;
   ```

**State transitions**:
* **Admin Price Update Lifecycle (`prices`)**:
  * `active` (`active_to IS NULL`) -> (`admin.prices.update_tier` with new Taka rate) -> old row set to `active_to = now`, new row inserted with `active_from = now`, `active_to = NULL`, `monthly_rate_poisha`, and `amount_poisha` (`monthly_rate_poisha * period_months` for 3, 6, 12 months, or `amount_poisha` directly for `lifetime`), audited in `admin_audit_log`.
* **Checkout Order Lifecycle (`checkout_orders.status`)**:
  * `pending_payment` -> (verified `bKash` / `Nagad` / `SSLCommerz` webhook arrives) -> `completed` (`payments`, `vendor_invoices`, `subscriptions`, `licenses`, and signed `license_issues` created atomically; `paid_through = '9999-12-31'` if `billing_cycle == 'lifetime'`).
  * `pending_payment` -> (`now > expires_at` before payment) -> `expired`.
  * `expired` -> (late verified gateway webhook arrives with valid `HMAC-SHA256` signature and matching `amount_poisha`) -> `completed`.
  * `pending_payment` -> (`POST /v1/checkout/submit_bank_transfer` with `bank_reference` and `bank_slip_url`) -> `pending_bank_review`.
  * `pending_bank_review` -> (`admin.checkout.approve_bank_transfer` by `ops_admin` or `super_admin`) -> `completed`.
* **Portal Device Management Lifecycle (`installations.status` & desktop mode)**:
  * `active` (bound to `installation_id = A`) -> (owner confirms OTP on `POST /v1/portal/devices/deactivate`) -> `A` becomes `deactivated`, `licenses.bound_installation_id` becomes `NULL`, `current_serial` increments, fresh 6 digit `pairing_code` generated.
  * `active` (bound to `installation_id = A`) -> (owner confirms OTP on `POST /v1/portal/devices/replace` with offline `REQ` for `installation_id = B`) -> `A` becomes `deactivated` (`replaced_by_installation_id = B`), `B` becomes `active`, `licenses.bound_installation_id` becomes `B`, `current_serial` increments, signed `.coachifylicense` for `B` returned immediately.
  * When PC `A` (now `deactivated` in cloud) next calls `POST /v1/refresh`, `crates/cloud_api` returns `RefreshStatus::Deactivated`; PC `A` sets `deactivated_at` in `license_refresh_state` and enters `deactivated_read_only` immediately (0 grace days).

**API surface**:

| Endpoint / Command | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `GET /v1/pricing/catalog` | Public HTTP | `plan_code` (opt) | `PricingCatalogResponse` (`tiers`: `3_months` [`1,500 BDT/mo`], `6_months` [`1,200 BDT/mo`], `12_months` [`1,000 BDT/mo`], `lifetime` [`50,000 BDT`]) | Public | `CATALOG_UNAVAILABLE` |
| `admin.prices.update_tier` | Admin Command / API | `plan_code`, `billing_cycle` (`3_months`, `6_months`, `12_months`, `lifetime`), `monthly_rate_bdt_or_total_bdt` | `UpdatedPriceTierResult` (`price_id`, `monthly_rate_poisha`, `amount_poisha`, `audit_seq`) | `ops_admin` or `super_admin` | `ADMIN_PERMISSION_DENIED`, `INVALID_PRICE_AMOUNT` |
| `POST /v1/checkout/create_session` | Public or Portal Bearer HTTP | `plan_code`, `billing_cycle` (`3_months`, `6_months`, `12_months`, `lifetime`), `kind` (`new_purchase`, `renewal`, `upgrade`, `lifetime_purchase`), `method` (`bkash`, `nagad`, `bank_transfer`), `org_name`, `owner_name`, `phone`, `email` (opt), `target_request_code` (opt) | `CheckoutSessionResponse` (`order_number`, `monthly_rate_poisha`, `amount_poisha`, `gateway_redirect_url`, `bank_instructions`, `expires_at`) | Public for `new_purchase`; Portal Bearer for `renewal` / `upgrade` | `INVALID_PLAN_OR_PERIOD`, `UPGRADE_NOT_ELIGIBLE`, `PORTAL_AUTH_REQUIRED` |
| `POST /v1/checkout/submit_bank_transfer` | Public or Portal Bearer HTTP | `order_number`, `bank_reference`, `bank_slip_url` | `CheckoutOrderStatusView` (`status = 'pending_bank_review'`) | Matches order session / phone | `ORDER_NOT_FOUND`, `BANK_REFERENCE_REQUIRED`, `ORDER_ALREADY_COMPLETED` |
| `GET /v1/checkout/status/{order_number}` | Public or Portal Bearer HTTP | `order_number` | `CheckoutOrderStatusView` (`status`, `invoice_number`, `license_id`, `pairing_code`, `signed_license_blob` when `completed`) | Order token or Portal Bearer | `ORDER_NOT_FOUND` |
| `POST /v1/webhooks/payment/{provider}` | Public Webhook HTTP | `provider`, `hmac_signature`, `payload` (`order_number`, `gateway_event_id`, `gateway_trx_id`, `amount_poisha`, `status`) | `WebhookAckResponse` (`processed_status`, `order_number`, `invoice_number`) | `HMAC-SHA256` (`GATEWAY_WEBHOOK_SECRET`) | `WEBHOOK_SIGNATURE_INVALID`, `WEBHOOK_AMOUNT_MISMATCH`, `ORDER_NOT_FOUND` |
| `admin.checkout.approve_bank_transfer` | Admin Command / API | `order_number`, `verification_notes` | `CheckoutOrderStatusView` (`status = 'completed'`, `invoice`, `license_issue`) | `ops_admin` or `super_admin` | `ADMIN_PERMISSION_DENIED`, `ORDER_NOT_PENDING_BANK_REVIEW` |
| `POST /v1/portal/devices/request_action_otp` | Portal Bearer HTTP | `session_token`, `action` (`deactivate`, `replace`), `installation_id` | `PortalOtpChallenge` (`otp_expires_at`, `masked_destination`, `demo_otp_preview` in test/dev) | Portal Bearer (`customer_portal_sessions`) | `PORTAL_SESSION_INVALID`, `DEVICE_NOT_OWNED_BY_CUSTOMER` |
| `POST /v1/portal/devices/deactivate` | Portal Bearer HTTP | `session_token`, `installation_id`, `otp_code`, `reason` | `DeviceActionResult` (`deactivated_installation_id`, `new_pairing_code`, `pairing_expires_at`, `transfers_used_12m`, `transfers_remaining_12m`) | Portal Bearer + 6 digit step up OTP | `PORTAL_OTP_INVALID`, `DEVICE_TRANSFER_LIMIT_EXCEEDED`, `DEVICE_ALREADY_INACTIVE` |
| `POST /v1/portal/devices/replace` | Portal Bearer HTTP | `session_token`, `old_installation_id`, `otp_code`, `reason`, `new_label`, `new_request_code_or_qr` (opt) | `DeviceActionResult` (`deactivated_installation_id`, `new_installation_id`, `new_pairing_code`, `signed_license_blob`, `transfers_remaining_12m`) | Portal Bearer + 6 digit step up OTP | `PORTAL_OTP_INVALID`, `DEVICE_TRANSFER_LIMIT_EXCEEDED`, `REQUEST_CODE_CHECKSUM_INVALID` |
| `POST /v1/portal/devices/rename` | Portal Bearer HTTP | `session_token`, `installation_id`, `new_label` | `InstallationRecord` | Portal Bearer (`customer_portal_sessions`) | `PORTAL_SESSION_INVALID`, `DEVICE_NOT_OWNED_BY_CUSTOMER`, `INVALID_DEVICE_LABEL` |

**Value sourcing**:

| Action | Value produced / displayed | Source |
|---|---|---|
| `GET /v1/pricing/catalog` & `POST /v1/checkout/create_session` | Monthly rate and total BDT price (`3_months` = `1,500/mo` [`4,500 BDT`], `6_months` = `1,200/mo` [`7,200 BDT`], `12_months` = `1,000/mo` [`12,000 BDT`], `lifetime` = `50,000 BDT`) | Looked up from active `prices` rows (`active_to IS NULL`) in `crates/cloud_api`; editable live via `admin.prices.update_tier` in Vendor Admin |
| `fulfill_verified_checkout_order` | New `subscriptions.paid_through` date | If `billing_cycle == 'lifetime'`, sets `paid_through = '9999-12-31'`; if `kind == 'upgrade'`, preserves `paid_through`; if `kind == 'renewal'` before grace ends, sets `existing_paid_through + period_months`; otherwise sets `payment_date + period_months` |
| `fulfill_verified_checkout_order` | Sequential `order_number` (`CHK-YYYY-NNNNN`) and `invoice_number` (`INV-YYYY-NNNNN`) | Derived from current UTC year `YYYY` and `COUNT(*) + 1` in `checkout_orders` and `vendor_invoices` respectively inside the SQLite transaction |
| `POST /v1/portal/devices/deactivate` & `replace` | `transfers_remaining_12m` | Computed as `3 - licenses.transfer_count_12m` |
| `POST /v1/refresh` on a deactivated PC | `RefreshStatus::Deactivated` + desktop `deactivated_read_only` state | Read from `installations.status == 'deactivated'` and `installations.deactivated_at` in `crates/cloud_api`; persisted into `license_refresh_state.deactivated_at` (Migration 0023) in `crates/core/src/licensing.rs` |

**Key invariants**:
* **Four tier pricing formula and historical immutability**: For periodic plans (`3_months`, `6_months`, `12_months`), `amount_poisha` is always `monthly_rate_poisha * period_months` (`1,500 * 3 = 4,500 BDT`, `1,200 * 6 = 7,200 BDT`, `1,000 * 12 = 12,000 BDT`), while for `lifetime` (`period_months = 0`, `is_lifetime = 1`), `amount_poisha` is the one time lifetime fee (`50,000 BDT` = `5,000,000 poisha`). Updating a price in Vendor Admin sets `active_to = now` on the prior `prices` row and inserts a new row so existing `checkout_orders` keep their locked `price_id` and `amount_poisha`.
* **Money and webhook idempotency**: `payment_webhook_events` enforces `UNIQUE(gateway_provider, gateway_event_id)` and `checkout_orders` enforces `UNIQUE(method, gateway_trx_id)` so replayed gateway webhooks never double extend `paid_through` or generate duplicate `INV-YYYY-NNNNN` invoices.
* **Never strand captured funds**: A verified gateway webhook (`signature_verified = 1` and matching `amount_poisha`) for an order whose 30 minute UI timer (`expires_at`) has elapsed still transitions the order from `expired` to `completed` and issues the license and invoice automatically.
* **Single active host per license**: Partial unique index `idx_installations_one_active_host_per_license` guarantees at most one `installations` row with `role = 'host'` and `status = 'active'` exists per `license_id` at any point in time.
* **Immediate read only on portal deactivation**: Unlike `moved` (which grants a 14 day transitional window for side by side PC migration), `deactivated` grants zero grace days; on its next `POST /v1/refresh`, the deactivated PC immediately enters `deactivated_read_only` while keeping all read, report, export, print, and backup operations available.

**Security model**:
* **Admin pricing mutation control**: `admin.prices.update_tier` is restricted to authenticated `super_admin` and `ops_admin` sessions and writes a SHA256 hash chained record (with `before_json` and `after_json` rates) into `admin_audit_log`.
* **Payment webhook authenticity**: `POST /v1/webhooks/payment/{provider}` computes `HMAC-SHA256` over the canonical webhook payload using the provider's configured secret in constant time (`hmac::Mac::verify_slice`) and verifies `amount_poisha` before mutating any state.
* **Portal tenant isolation and step up OTP**: All `/v1/portal/devices/*` and renewal/upgrade endpoints authenticate the bearer token against `customer_portal_sessions` and verify `licenses.customer_id == session.customer_id`. Destructive device actions (`deactivate` and `replace`) additionally require a fresh 6 digit OTP code verified against `customer_portal_credentials.otp_code_hash`.

**Configuration required**:
* `SSLCOMMERZ_STORE_ID` and `SSLCOMMERZ_STORE_SECRET`: merchant credentials and HMAC webhook verification secret for SSLCommerz checkout sessions.
* `BKASH_APP_KEY`, `BKASH_APP_SECRET`, and `BKASH_WEBHOOK_SECRET`: credentials and webhook HMAC secret for bKash tokenized checkout.
* `NAGAD_MERCHANT_ID`, `NAGAD_PG_PUBLIC_KEY`, and `NAGAD_WEBHOOK_SECRET`: credentials and webhook verification secret for Nagad merchant checkout.
* `COACHIFY_BANK_ACCOUNT_DETAILS_JSON`: beneficiary bank name, account title, account number, routing number, and branch displayed during guided bank transfer checkout.

**Critical test scenarios**:
* Happy path (4 tier pricing seed & Vendor Admin price update): Verify default seeded tiers (`3_months` = `1,500 BDT/mo` [`4,500 BDT`], `6_months` = `1,200 BDT/mo` [`7,200 BDT`], `12_months` = `1,000 BDT/mo` [`12,000 BDT`], `lifetime` = `50,000 BDT`), call `admin.prices.update_tier` from Vendor Admin to change a tier rate, and verify `GET /v1/pricing/catalog` and new checkout sessions immediately reflect the new rate while archiving the old `prices` row, verifies **AC-1**, **AC-2**.
* Happy path (instant online purchase for periodic and `50,000 BDT` lifetime plans): Customer creates a `bKash` checkout session for `3_months`, `6_months`, `12_months`, or `lifetime` (`POST /v1/checkout/create_session`), `POST /v1/webhooks/payment/bkash_direct` arrives with a valid `HMAC-SHA256` signature, and the system atomically creates the `payments` row, `INV-YYYY-NNNNN` invoice, `subscriptions` extension (`paid_through = '9999-12-31'` for `lifetime`), 6 digit `pairing_code`, and Ed25519 signed `.coachifylicense`, verifies **AC-2**, **AC-3**, **AC-4**.
* Edge case (idempotent duplicate webhook and late webhook after expiry): Sending the identical `(gateway_provider, gateway_event_id)` webhook twice produces `duplicate_ignored`, while a valid webhook arriving after `expires_at` still transitions an `expired` order to `completed`, verifies **AC-3**, **AC-4**.
* Happy path (guided bank transfer and admin approval): Customer submits a bank reference (`POST /v1/checkout/submit_bank_transfer`), order enters `pending_bank_review`, and `ops_admin` runs `admin.checkout.approve_bank_transfer` to issue the invoice and signed license, verifies **AC-5**.
* Happy path (portal device deactivation and desktop immediate read only): Owner requests a step up OTP and calls `POST /v1/portal/devices/deactivate` for PC `A`, clearing `bound_installation_id` and issuing a new 6 digit pairing code; when PC `A` calls `POST /v1/refresh`, it receives `deactivated`, records `deactivated_at` in SQLite (Migration 0023), and blocks write commands with `LICENSE_DEVICE_DEACTIVATED` while keeping exports and backups open, verifies **AC-6**, **AC-7**, **AC-8**.
* Happy path and limit guard (one step offline `REQ` device replacement and 3 transfer cap): Owner calls `POST /v1/portal/devices/replace` with PC `B`'s `REQ-XXXX-XXXX-XXXX-XXXX` code to deactivate PC `A` and receive PC `B`'s signed `.coachifylicense` in one step; a 4th deactivation or replacement within 12 months returns `DEVICE_TRANSFER_LIMIT_EXCEEDED`, verifies **AC-9**, **AC-10**.

## Build plan

Ordered by the `Tracer Bullet` approach across `crates/cloud_api`, `crates/core`, `Landing pages/`, and `ui/`:

1. **Cloud 4 tier pricing catalog (`3m`, `6m`, `12m`, `Lifetime`), admin price editor, and checkout/webhook engine (`crates/cloud_api`)**: Seed `prices` with `3_months` (`1,500 BDT/mo` = `4,500 BDT`), `6_months` (`1,200 BDT/mo` = `7,200 BDT`), `12_months` (`1,000 BDT/mo` = `12,000 BDT`), and `lifetime` (`50,000 BDT` one time), implement `admin.prices.update_tier` and `GET /v1/pricing/catalog`, add `checkout_orders`, `payment_webhook_events`, `PaymentGatewayProvider` adapters (`sslcommerz`, `bkash_direct`, `nagad_direct`, `sandbox`), `POST /v1/checkout/create_session`, `POST /v1/checkout/submit_bank_transfer`, `POST /v1/webhooks/payment/{provider}`, `admin.checkout.approve_bank_transfer`, and transactional `fulfill_verified_checkout_order` (including `paid_through = '9999-12-31'` for `lifetime`), satisfies **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**.
2. **Cloud portal device management engine (`crates/cloud_api`)**: Extend `installations` (`hostname`, `os_version`, `deactivated_at`, `deactivation_reason`, `replaced_by_installation_id`, `status = 'deactivated'`, single active host unique index), add `device_management_events` (with append only triggers), implement `POST /v1/portal/devices/request_action_otp`, `POST /v1/portal/devices/deactivate`, `POST /v1/portal/devices/replace` (online pairing + offline `REQ` binding), `POST /v1/portal/devices/rename`, and update `POST /v1/refresh` to return `RefreshStatus::Deactivated`, satisfies **AC-6**, **AC-7**, **AC-9**.
3. **Desktop SQLite Migration 0023 and immediate `deactivated_read_only` enforcement (`crates/core`)**: Add `migrations/0023_license_deactivation_state.sql` (`deactivated_at`, `deactivation_reason` on `license_refresh_state`), handle `RefreshStatus::Deactivated` in `crates/core/src/licensing.rs` without grace days, enforce `LicenseMode::DeactivatedReadOnly` (`LICENSE_DEVICE_DEACTIVATED`) in the command dispatcher while keeping read/export/backup open, and clear deactivation when a newer serial bound to the installation is imported, satisfies **AC-8**.
4. **Website 4 Tier Pricing & Checkout Modal (`Landing pages/`), Vendor Admin `Plans & Pricing Studio` (`ui/`), and `/portal` Device Management Studio**: Update `Landing pages/src/components/Pricing.tsx` and `CheckoutModal` to render the live 3 Month (`1,500 BDT/mo`), 6 Month (`1,200 BDT/mo`), 12 Month Annual (`1,000 BDT/mo`), and Full Lifetime (`50,000 BDT`) plans, add the easy `Plans & Pricing Studio` editor and Pending Bank Transfers queue in `ui/src/components/vendor-admin/vendor-admin-view.tsx`, and expand `/portal` with device telemetry, Rename, Deactivate, and Replace modals plus Vitest and Rust test suites, satisfies **AC-1**, **AC-2**, **AC-5**, **AC-6**, **AC-7**, **AC-9**, **AC-10**.

## Consequences

**Positive**:
* Gives coaching center owners clear, transparent BDT pricing across 3 months (`1,500 BDT/mo`), 6 months (`1,200 BDT/mo`), 12 months (`1,000 BDT/mo`), and a full Lifetime license (`50,000 BDT`), while letting the founder or admin adjust any rate in seconds from the Vendor Admin UI without redeploying code.
* Online checkout and manual admin payment recording share one transactional fulfillment function (`fulfill_verified_checkout_order`), eliminating drift between gateway renewals, lifetime purchases, and staff recorded payments.
* Portal device management gives owners self service control over lost or replaced PCs while preventing license sharing abuse through the single active host index, immediate `deactivated_read_only` mode on the old PC, and the shared 3 transfers per 12 months cap.

**Negative / tradeoffs**:
* Sharing the 3 transfers per 12 months counter between desktop self transfers and portal device deactivations/replacements means an institution with frequent hardware crashes must request a one click support reset from `ops_admin` on their 4th change in a year.
* Because Coachify is offline first, a deactivated PC that is physically disconnected from the internet will remain active until it next reaches `POST /v1/refresh` or its local `paid_through` + grace date expires.

**Neutral**:
* Adds Desktop SQLite Migration `0023_license_deactivation_state.sql` (`crates/core/migrations/`) and three commercial cloud tables (`checkout_orders`, `payment_webhook_events`, `device_management_events`) plus the 4 tier `prices` catalog in `crates/cloud_api`.

## Follow-up

* [ ] Configure production merchant credentials (`SSLCOMMERZ_STORE_ID`, `SSLCOMMERZ_STORE_SECRET`, `BKASH_WEBHOOK_SECRET`, `NAGAD_WEBHOOK_SECRET`) in the cloud deployment secrets manager once merchant onboarding completes after pilots.
