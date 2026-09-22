# 0032-receipt-verification. Receipt Verification

> Child specification under Spec 0032 (Slice 8 Operational Integrity Umbrella) · Supporting Feature 40

**Date**: 2026-09-23

## Summary

This specification establishes offline cryptographic receipt verification for printed fee vouchers and payment receipts. Each printed receipt carries a compact 12 character HMAC SHA256 verification code and a QR code encoding payment metadata and signature. Receptionists, cashiers, or auditors can scan the QR code or type the verification code into the application to instantly confirm payment authenticity, student details, and current validity status (valid, voided, or refunded) completely offline.

## Requirements

**User stories**:
* As a receptionist or cashier, I want to scan a receipt QR code presented by a student or guardian, so that I can immediately confirm the voucher is genuine and matches our financial ledger.
* As an institution administrator, I want to detect if an incoming receipt was voided or refunded after printing, so that fraudulent re use of paper receipts is eliminated.

**Acceptance criteria**:
* **AC-1**: Thermal and A4 printed receipts include a compact 12 character HMAC SHA256 verification code and a QR code holding receipt number, poisha amount, business date, student code, and signature hash.
* **AC-2**: Scanning or entering the code via RPC `integrity.verify_receipt` validates the signature offline and returns current status (valid, voided, or refunded) along with student name and payment method.
* **AC-3**: A voided or refunded payment shows a prominent warning indicating the date and time of the void or refund and the cashier who processed it.

## Decision

**Chosen option**: Local HMAC SHA256 Truncated Signature with Offline Verification RPC

We generate a deterministic HMAC SHA256 signature using the institution secret key over a canonical string `receipt_number|amount_poisha|business_date|student_code`. The first 12 hexadecimal characters form the human readable verification code, while the QR code embeds the full signature payload. The core RPC verifies the signature mathematically before performing a local database lookup.

## Rationale

Because coaching centers in Bangladesh often operate with patchy internet, receipt verification must not depend on cloud lookups or external certificate authorities. Using the institution local key with HMAC SHA256 ensures instant, zero latency verification directly on the desktop. Truncating to 12 characters allows easy telephone or manual typing verification when barcode scanners are unavailable.

## Feature design

**Data model sketch**:
* `payments`: Column `verification_code TEXT` added in Migration 0017.
* `receipt_verifications`: Table logging all scan attempts with columns `id TEXT PRIMARY KEY`, `receipt_number TEXT`, `verification_code TEXT`, `status_at_check TEXT`, `verified_by_user_id TEXT REFERENCES users(id)`, `scanned_payload TEXT`, `checked_at TEXT`.

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `integrity.verify_receipt` | Mutation | code_or_payload: String | ReceiptVerificationResult | Authenticated | 400 invalid format, 404 not found |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Issue payment | verification_code | HMAC SHA256 of receipt data keyed by local secret |
| Verify scan | verification result and status | Decoded signature check and status from payments table |

**Key invariants**:
* Changing any field in the QR payload (such as altering the amount from 500 to 5000) produces an immediate signature mismatch.
* Voided and refunded receipts retain their original verification code but return their updated status upon verification.

**Security model**:
* Any authenticated user (Receptionist, Cashier, Teacher, Admin, Owner) can verify receipts.
* The HMAC secret key is stored in DPAPI encrypted system settings and never exposed in the UI.

**Critical test scenarios**:
* Valid offline verification: Scans genuine payload, asserts is_authentic is true and status is valid, verifies AC-1 and AC-2.
* Tampered amount detection: Modifies amount in payload string, asserts signature mismatch error, verifies AC-2.
* Voided status display: Voids an existing payment, verifies the receipt code, asserts status returned is voided, verifies AC-3.

## Build plan

1. Add `verification_code` column to `payments` table and create `receipt_verifications` table in Migration 0017, satisfies **AC-1**.
2. Implement HMAC SHA256 canonical signer and verifier in `crates/core/src/integrity.rs`, satisfies **AC-1**, **AC-2**.
3. Implement `integrity.verify_receipt` RPC endpoint returning payment status, student details, and verification audit logging, satisfies **AC-2**, **AC-3**.
4. Update receipt print template in `ui/src/components/print/receipt-printable.tsx` to render the verification code and QR code, satisfies **AC-1**.
5. Build Receipt Verification dialog modal in `ui/src/components/cashier/receipt-verification-dialog.tsx`, satisfies **AC-2**, **AC-3**.

## Consequences

**Positive**:
* Guarantees 100% offline verification capability for counter staff and parents.
* Eliminates altered or forged fee vouchers.

**Negative / tradeoffs**:
* Requires allocating visual space on printed thermal receipts for the QR code and alphanumeric string.
