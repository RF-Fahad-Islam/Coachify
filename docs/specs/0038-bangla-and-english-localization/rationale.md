# 0038. Bangla + English Localization: Rationale

## Key Decisions

### 1. Zero-Dependency Reactive Store vs. Heavy i18n Framework
We keep the lightweight pub-sub dictionary architecture in `ui/src/lib/i18n.ts` enhanced with `useSyncExternalStore` (`useLocale()`) rather than pulling in `i18next`. This guarantees zero bundle bloat, sub-millisecond synchronous lookups, strict TypeScript safety, and instant re-rendering across React 19 components.

### 2. Deterministic South Asian Number & Currency Formatting
Browser `Intl.NumberFormat('bn-BD')` implementations vary subtly across WebView2 / Chromium versions regarding symbol spacing and negative signs. Implementing deterministic South Asian grouping (`#,##,###`) and explicit ASCII-to-Bengali numeral mapping (`0-9` → `০-৯`) guarantees identical output across Windows 10/11 installations and Vitest environments.
