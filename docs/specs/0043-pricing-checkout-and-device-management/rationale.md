# 0043. Rationale: Online Pricing, Gateway Checkout, and Portal Device Management

## Context

Coachify's Commercial epic (Slice 12) already provides desktop Ed25519 license verification and read only enforcement (Spec `0037`), a two factor Vendor Admin panel with manual payment recording and an isolated `crates/license_signer` service (Spec `0041`), daily desktop refresh and 3 per year desktop self transfers (Spec `0041`), and a customer portal (`/portal`) with `.coachifylicense` downloads, installer downloads, and invoice history (Spec `0042`). Two final commercial capabilities remain before general availability (`1.0`):

1. **Feature 67 (Pricing and checkout)**: Coaching center owners visiting the website or logged into `/portal` need to purchase or renew a Coachify plan online across four clear tiers (`3 months` at `1,500 BDT/month` = `4,500 BDT`, `6 months` at `1,200 BDT/month` = `7,200 BDT`, `12 months` at `1,000 BDT/month` = `12,000 BDT`, and a `Full Lifetime License` at `50,000 BDT` one time), using Bangladesh payment rails (`bKash`, `Nagad`, and bank transfer), with automatic invoice and Ed25519 license issuing once payment is verified. In addition, the founder or admin must be able to change and save any tier's price easily from the Vendor Admin UI (`Plans & Pricing Studio`) so the website and checkout update immediately without code changes.
2. **Feature 68 (Device management)**: Institution owners logged into `/portal` need to inspect every PC linked to their license (with last seen time, app version, and active student count), rename labels, deactivate a lost or retired PC so that it enters read only mode on its next refresh, or replace an old PC with a new machine (either online via pairing code or offline via a 20 character `REQ` code).

Without online checkout and an admin price studio, every price adjustment would require a code deploy and every purchase or renewal would require a human support agent on WhatsApp to verify a payment screenshot and manually click record in Vendor Admin. Without portal device management, an owner whose office computer dies or is stolen cannot release their bound license slot or force the missing machine into read only mode without waiting for vendor support.

## Options considered

### Option 1: Admin Configurable 4 Tier Pricing Catalog (`3m`, `6m`, `12m`, `Lifetime`), Transactional `crates/cloud_api` Checkout & Device Engine, Trait Based Bangladesh Gateway Adapters, and Immediate `deactivated_read_only` Desktop Enforcement (Recommended)

Extend `prices` in `crates/cloud_api` with `billing_cycle IN ('3_months', '6_months', '12_months', 'lifetime')`, `monthly_rate_poisha`, and `is_lifetime`, seeded with `1,500 BDT/mo` (`3_months`), `1,200 BDT/mo` (`6_months`), `1,000 BDT/mo` (`12_months`), and `50,000 BDT` (`lifetime`), paired with a one click `Plans & Pricing Studio` in Vendor Admin (`admin.prices.update_tier`) and public `GET /v1/pricing/catalog`. Add `checkout_orders`, `payment_webhook_events`, and `device_management_events`, implement a `PaymentGatewayProvider` trait supporting `SSLCommerz`, direct `bKash` and `Nagad` merchant APIs, and a deterministic `sandbox` adapter, and route both verified gateway webhooks and admin approved bank transfers through a single transactional `fulfill_verified_checkout_order` pipeline that calls `crates/license_signer`. In `/portal`, guard device deactivation and replacement with a step up 6 digit OTP, share the 3 transfers per 12 months cap, and add `RefreshStatus::Deactivated` plus Desktop Migration 0023 (`0023_license_deactivation_state.sql`) so a deactivated PC enters `deactivated_read_only` immediately on its next refresh without a 14 day grace window.

**Pros**:
* Gives the founder complete pricing control from the Vendor Admin UI: editing the 3 month (`1,500 BDT/mo`), 6 month (`1,200 BDT/mo`), 12 month (`1,000 BDT/mo`), or Lifetime (`50,000 BDT`) rate archives the old `prices` row (`active_to = now`), activates the new rate immediately on `GET /v1/pricing/catalog` and `Landing pages/`, and logs a hash chained entry in `admin_audit_log`.
* Zero divergence between manual admin payments (Feature 63) and online gateway or bank transfer checkouts (Feature 67): both produce identical `payments`, `vendor_invoices` (`INV-YYYY-NNNNN`), grace aware `paid_through` dates (`9999-12-31` for `lifetime`), and isolated `crates/license_signer` signatures.
* Distinguishes routine side by side PC migration (`moved` status, which keeps a 14 day transitional window so the front desk is not disrupted) from portal deactivation of a lost or stolen PC (`deactivated` status, which transitions the desktop into `deactivated_read_only` immediately on next refresh with zero grace days).
* Supports offline coaching centers during portal device replacement by accepting the new PC's 20 character request code (`REQ-XXXX-XXXX-XXXX-XXXX`) and returning the bound `.coachifylicense` download in a single step.

**Cons**:
* Adds Desktop SQLite Migration 0023 (`deactivated_at` and `deactivation_reason` in `license_refresh_state`) so the desktop distinguishes `deactivated` (0 day grace) from `moved` (14 day transitional window).
* Requires step up 6 digit OTP verification in `/portal` before deactivating or replacing a device, adding one confirmation step for the owner.

### Option 2: Hardcoded Frontend Pricing Constants and External SaaS Billing Provider

Hardcode the 3 month, 6 month, 12 month, and Lifetime prices in `Landing pages/src/data/appData.ts` and delegate checkout to an external international SaaS billing platform.

**Pros**:
* Requires no admin pricing editor UI.

**Cons**:
* Changing a price requires editing TypeScript code and redeploying the website instead of clicking Save in the Vendor Admin UI.
* International SaaS billing platforms lack native BDT `bKash`, `Nagad`, and local bank transfer slip workflows.

### Option 3: Reuse `moved` 14 Day Transitional Status for Portal Device Deactivation and Skip Step Up OTP

Implement portal device deactivation by simply setting `installations.status = 'moved'` without adding a `deactivated` state or Desktop Migration 0023.

**Pros**:
* Requires zero schema changes in `crates/core` (no Migration 0023).

**Cons**:
* Violates the security expectation of deactivating a lost or stolen PC: if `moved` is reused, the deactivated PC remains writable for 14 full days even when connected to the internet and calling `POST /v1/refresh`.

## Rationale

Option 1 is chosen because it puts the exact 4 tier BDT pricing structure (`3 months` at `1,500 BDT/month`, `6 months` at `1,200 BDT/month`, `12 months` at `1,000 BDT/month`, and `Full Lifetime License` at `50,000 BDT`) into the commercial database (`prices`) with an intuitive `Plans & Pricing Studio` in Vendor Admin (`admin.prices.update_tier`) while keeping the entire payment, invoice, and Ed25519 license signing pipeline inside a single ACID SQLite store (`crates/cloud_api`) and isolated signer (`crates/license_signer`). When an admin updates any tier's monthly or lifetime rate in Vendor Admin, `GET /v1/pricing/catalog` immediately serves the new rate to `Landing pages/src/components/Pricing.tsx` and the checkout modal, while existing in flight orders retain their locked `price_id`.

For Feature 68 (Device management), Option 1 cleanly resolves the difference between replacing an office PC side by side (`moved` with a 14 day transitional window from Spec `0037`) and deactivating a lost or retired PC in `/portal` (`deactivated` with immediate `deactivated_read_only` on next refresh via Migration 0023), guarded by a 6 digit step up OTP and the shared 3 transfers per 12 months policy.
