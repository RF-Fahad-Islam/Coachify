# 0043a. Pricing and Checkout (Feature 67)

## Summary

Feature 67 enables coaching center owners to purchase or renew a Coachify plan online across four admin configurable tiers (`3 Months` at `1,500 BDT/month` = `4,500 BDT`, `6 Months` at `1,200 BDT/month` = `7,200 BDT`, `12 Months Annual` at `1,000 BDT/month` = `12,000 BDT`, and `Full Lifetime License` at `50,000 BDT` one time) through `Landing pages/` and `/portal`, and gives administrators a `Plans & Pricing Studio` in the Vendor Admin UI (`ui/src/components/vendor-admin/vendor-admin-view.tsx`) to edit any tier's price easily at any time. It integrates Bangladesh payment methods (`bKash`, `Nagad`, `SSLCommerz`, and guided bank transfer slip submission) into `crates/cloud_api`, verifies gateway webhooks with `HMAC-SHA256`, and invokes `fulfill_verified_checkout_order` to generate a sequential `INV-YYYY-NNNNN` vendor invoice, a 6 digit desktop pairing code, and an Ed25519 signed `.coachifylicense` from `crates/license_signer` automatically.

## Requirements

Satisfies umbrella acceptance criteria **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, and **AC-10** in [index.md](index.md).

## Decision & Inline Rationale

We implement online pricing and checkout inside `crates/cloud_api` (`prices` extended with `billing_cycle` and `is_lifetime`, `checkout_orders`, and `payment_webhook_events`), `ui/src/components/vendor-admin/vendor-admin-view.tsx` (`Plans & Pricing Studio` + Pending Bank Transfers Queue), and `Landing pages/` (`Pricing.tsx`, `CheckoutModal`, and `/portal` renewal controls). Storing all four tiers (`3_months`, `6_months`, `12_months`, `lifetime`) in `prices` and exposing `admin.prices.update_tier` plus `GET /v1/pricing/catalog` ensures any price adjustment made in Vendor Admin takes effect immediately on the public pricing page and in checkout calculations without editing code, while locking the active `price_id` on each created `checkout_orders` row so in flight checkouts never break.

## Feature design

**Default Seeded Tiers & Admin Price Editing Rules**:
* **3 Month Plan (`billing_cycle = '3_months'`, `period_months = 3`, `is_lifetime = 0`)**: Default `1,500 BDT/month` (`monthly_rate_poisha = 150_000`), total billed `4,500 BDT` (`amount_poisha = 450_000`).
* **6 Month Plan (`billing_cycle = '6_months'`, `period_months = 6`, `is_lifetime = 0`)**: Default `1,200 BDT/month` (`monthly_rate_poisha = 120_000`), total billed `7,200 BDT` (`amount_poisha = 720_000`).
* **12 Month Annual Plan (`billing_cycle = '12_months'`, `period_months = 12`, `is_lifetime = 0`)**: Default `1,000 BDT/month` (`monthly_rate_poisha = 100_000`), total billed `12,000 BDT` (`amount_poisha = 1_200_000`).
* **Full Lifetime License (`billing_cycle = 'lifetime'`, `period_months = 0`, `is_lifetime = 1`)**: Default `50,000 BDT` one time (`monthly_rate_poisha = 5_000_000`, `amount_poisha = 5_000_000`). Sets `subscriptions.paid_through = '9999-12-31'` and `LicensePayload.paid_through = '9999-12-31'` so the desktop license never expires.
* **Easy Admin UI Pricing Configuration (`admin.prices.update_tier`)**: In the Vendor Admin `Plans & Pricing Studio`, `super_admin` or `ops_admin` edits the monthly rate in BDT for the 3, 6, or 12 month plan (the UI and backend automatically compute `total = monthly_rate * months`) or the one time BDT price for the Lifetime license and clicks Save. `crates/cloud_api` sets `active_to = now` on the previous active row for `(plan_id, billing_cycle)`, inserts the new active `prices` row (`active_from = now`, `active_to = NULL`), and writes a hash chained audit entry to `admin_audit_log`.

**Checkout & Fulfillment Rules**:
* **New purchase (`kind = 'new_purchase'` or `'lifetime_purchase'`)**: Visitor selects tier (`3_months`, `6_months`, `12_months`, or `lifetime`), enters `org_name`, `owner_name`, `phone`, optional `email`, and optional offline `target_request_code` (`REQ-XXXX-XXXX-XXXX-XXXX`). `amount_poisha` is read from the active `prices` row.
* **Renewal (`kind = 'renewal'`)**: Authenticated `/portal` customer selects `3_months`, `6_months`, `12_months`, or `lifetime`. When fulfilled for a periodic tier before `existing_paid_through + 14 days grace`, `new_paid_through = existing_paid_through + period_months`; after grace expiry, `new_paid_through = payment_date + period_months`. When fulfilled for `lifetime`, `new_paid_through = '9999-12-31'`.
* **Guided bank transfer (`method = 'bank_transfer'`)**: Order starts in `pending_payment`, displays Coachify's bank account details and `order_number` (`CHK-YYYY-NNNNN`), and transitions to `pending_bank_review` when the customer submits `bank_reference` and `bank_slip_url` via `POST /v1/checkout/submit_bank_transfer`. When `ops_admin` or `super_admin` executes `admin.checkout.approve_bank_transfer`, `fulfill_verified_checkout_order` runs atomically.

**API endpoints (`crates/cloud_api`)**:
* `get_pricing_catalog` (`GET /v1/pricing/catalog`)
* `admin_update_price_tier` (`admin.prices.update_tier`)
* `create_checkout_session` (`POST /v1/checkout/create_session`)
* `submit_bank_transfer_proof` (`POST /v1/checkout/submit_bank_transfer`)
* `get_checkout_order_status` (`GET /v1/checkout/status/{order_number}`)
* `handle_payment_webhook` (`POST /v1/webhooks/payment/{provider}`)
* `admin_approve_bank_transfer` (`admin.checkout.approve_bank_transfer`)
* `admin_list_pending_bank_transfers` (`admin.checkout.list_pending_bank_transfers`)
