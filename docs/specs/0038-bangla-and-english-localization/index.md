# 0038. Bangla + English Localization

> Status: Ratified
> Feature: 51 (Slice 9 — Printing & Localization)
> Date: 2026-09-25

## Overview

Coachify serves coaching centers in Bangladesh where owners, receptionists, cashiers, and teachers operate across a mix of English and Bangla (`বাংলা`). While the UI foundation (Feature 4) introduced a basic `t(key)` lookup and font tokens (`Hind Siliguri` for Bangla and `Plus Jakarta Sans` for Latin), Feature 51 completes the localization architecture:

1. **Reactive Locale State & Persistence**: Instant, zero-reload language switching (`en` ↔ `bn`) via a reactive `useLocale()` React hook backed by `localStorage` (`coachify.locale`) and synchronized with `document.documentElement.lang`.
2. **Full Dictionary Coverage**: Comprehensive English and Bangla translation dictionaries covering navigation, top-bar controls, role dashboards, student directory, cashier & dues workflows, attendance & messaging, exams & results, reports, settings, common actions/modals, and RPC domain error codes.
3. **Bangladeshi Regional Formatting**:
   - **Numbers**: Bengali digit conversion (`০১২৩৪৫৬৭৮৯`) and South Asian lakh/crore grouping (`1,25,000` / `১,২৫,০০০`).
   - **Currency**: Poisha-to-Taka formatting with `৳` symbol and locale-aware numerals (`৳ 1,250.00` in English, `৳ ১,২৫০.০০` in Bangla).
   - **Dates**: Locale-aware date rendering (`25 Sep 2026` in English, `২৫ সেপ্টেম্বর ২০২৬` in Bangla).
4. **Localized Error Translation**: RPC error translator (`translateError(code, fallback, params)`) ensuring validation and domain error messages render in Bangla when Bangla mode is active.

## Acceptance Criteria

- **AC-1 (Instant Language Switching)**: Toggling between English (`EN`) and Bangla (`বাংলা`) in the top bar updates all visible navigation, labels, headings, buttons, and status badges immediately without reloading the window, and persists across app restarts.
- **AC-2 (Zero English Leakage in Key Workflows)**: Every key in `translations.en` has a non-empty, accurate Bangla counterpart in `translations.bn`, enforced by an automated parity test.
- **AC-3 (Bangladeshi Number & Currency Conventions)**: `formatNumberLocale`, `formatCurrencyLocale`, and `toBengaliDigits` format numbers using South Asian digit grouping (`1,25,000`) and convert numerals to Bengali digits (`০-৯`) when the active locale is `bn`.
- **AC-4 (Bangladeshi Date Formatting)**: `formatDateLocale` renders ISO dates (`YYYY-MM-DD`) using Bengali month names and Bengali numerals when the active locale is `bn`.
- **AC-5 (Localized Domain & Validation Errors)**: `translateError` maps core RPC error codes to natural Bangla messages in `bn` mode.

## Build Plan

1. Upgrade `ui/src/lib/i18n.ts` with persistent locale storage, `useLocale()` hook, comprehensive `en`/`bn` dictionaries, `toBengaliDigits`, `formatNumberLocale`, `formatCurrencyLocale`, `formatDateLocale`, and `translateError`.
2. Add a 1-click `EN | বাংলা` language toggle control to `TopBar` (`ui/src/components/layout/top-bar.tsx`) and wire `useLocale()` across `Sidebar`, `AppShell`, and primary views.
3. Add comprehensive unit and parity tests in `ui/src/lib/i18n.test.ts` verifying 100% key parity between `en` and `bn`, digit conversion, lakh/crore currency formatting, date formatting, and reactive hook updates.
