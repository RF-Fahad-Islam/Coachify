# 0038. Bangla + English Localization: Verification Plan

## Automated Verification

```bash
pnpm --dir ui test i18n.test.ts --run
pnpm --dir ui test layout.test.tsx --run
```

## Verification Matrix

| Criterion | Test | Expected Outcome |
|---|---|---|
| AC-1 | `useLocale` and `TopBar` language switcher test | Clicking `বাংলা` switches locale to `bn`, updates `document.documentElement.lang`, persists to `localStorage`, and re-renders components |
| AC-2 | `en` / `bn` dictionary parity test | Every key in `translations.en` exists in `translations.bn` with a non-empty string containing Bengali script or appropriate bilingual terms |
| AC-3 | `formatCurrencyLocale` & `formatNumberLocale` | `12500000` poisha (`1,25,000` Taka) renders as `৳ 1,25,000.00` in `en` and `৳ ১,২৫,০০০.০০` in `bn` |
| AC-4 | `formatDateLocale` | `2026-09-25` renders as `25 Sep 2026` in `en` and `২৫ সেপ্টেম্বর ২০২৬` in `bn` |
| AC-5 | `translateError` | Core RPC error codes map to localized Bangla descriptions when `bn` is active |
