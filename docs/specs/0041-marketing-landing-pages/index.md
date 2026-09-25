# 0041. Marketing & Commercial Landing Pages

> Status: Ratified
> Feature: Slice 12 — Commercial Web Presence, Installer Distribution & Lead Capture
> Date: 2026-09-25

## Overview

Slice 12 delivers the official commercial web presence (`Landing pages`) for **Coachify (NF Management Local)**, designed specifically for coaching centers, academies, and educational institutions in Bangladesh. The landing web application is integrated into the Coachify pnpm workspace and adheres to the repository's strict TypeScript, ESLint (`no-explicit-any`, named exports only), and design system standards (`design.md`).

Key capabilities:

1. **First-Class Monorepo Package**: Integrated into `pnpm-workspace.yaml` with workspace scripts for development, type-checking, linting, formatting, building, and Vitest verification.
2. **Bangladesh Coaching Center Identity (`৳ BDT`)**:
   - All product screen mockups (`appData.ts` & `ProductUI.tsx`) showcase realistic Bangladeshi coaching center workflows: `Atomic Coaching Center` (`অ্যাটমিক কোচিং সেন্টার`), `HSC 2026–A`, `Medical Admission 2026`, `BUET Engineering 2026`, `SSC Foundation`, `৳ BDT` currency with South Asian lakh/thousand formatting, and local payment rails (`Cash`, `bKash`, `Nagad`, `Bank`).
3. **Bilingual English + Bangla (`EN` / `বাংলা`) Switcher**:
   - Reactive `useLandingLocale()` hook backed by `localStorage` (`coachify.landing.locale`) and `document.documentElement.lang` synchronization.
   - Paired Latin (`Sora` / `Inter`) and Bengali (`Noto Sans Bengali`) typography with instant language toggling in the top navigation bar and footer.
4. **Transparent BDT Commercial Pricing**:
   - Clear one-time commercial tiers: **Demo (`Free / ৳0`)** for 30 days, **Full Version (`৳14,900` one-time per-PC license)**, and **Pro Multi-PC (`৳29,900` one-time LAN license up to 5 PCs)**.
5. **Verified Release Installer Distribution & Lead Capture**:
   - Configurable Windows installer download (`VITE_INSTALLER_URL`, version badge `v1.0.0`, file size, SHA-256 checksum `sha256` with one-click copy, and direct `.exe` download trigger).
   - Validated license/demo request modal and contact forms with Bangladesh phone/WhatsApp/email validation, configurable webhook submission (`VITE_LEAD_WEBHOOK_URL`), local storage audit persistence (`coachify.landing.leads`), and direct WhatsApp/Email links.

## Acceptance Criteria

- **AC-1 (Monorepo & Strict Code Standards)**: `Landing pages` is registered in `pnpm-workspace.yaml`, exports all modules via named exports only (zero default exports), and passes `tsc --noEmit` and ESLint with zero warnings or errors.
- **AC-2 (Bangladesh Market Data & BDT Pricing)**: All product previews (`Dashboard`, `Students`, `Attendance`, `Payments`, `Dues`, `Exams`, `Results`, `Reports`, `Notifications`) render `৳ BDT` currency, Bangladeshi coaching center batches (`HSC`, `Medical`, `BUET`, `SSC`), `bKash`/`Nagad` payment methods, and transparent BDT pricing (`Free`, `৳14,900`, `৳29,900`).
- **AC-3 (Instant EN / বাংলা Switching)**: Clicking the `EN | বাংলা` switcher in the navbar immediately updates navigation links, hero headlines, CTA buttons, pricing headers, and modals, and persists across reloads.
- **AC-4 (Release Installer & SHA-256 Checksum)**: Clicking Download Demo opens the release installer flow showing version `v1.0.0`, SHA-256 checksum verification, and triggers a real downloadable installer artifact.
- **AC-5 (Validated Lead & Contact Submission)**: Submitting the License Request modal or Contact page form validates required fields and contact format, persists the lead record, posts to the configured webhook when set, and provides instant WhatsApp/email follow-up options.
