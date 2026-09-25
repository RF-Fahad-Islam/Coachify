# Verification — 0041. Marketing & Commercial Landing Pages

## Automated Verification Commands

```bash
# Typecheck landing pages
pnpm --dir "Landing pages" typecheck

# Lint landing pages (enforcing named exports and no explicit any)
pnpm --dir "Landing pages" lint

# Run Vitest unit & route integration tests
pnpm --dir "Landing pages" test

# Production build verification
pnpm --dir "Landing pages" build
```

## Test Coverage Checklist

- [x] All 11 routes (`/`, `/features`, `/how-it-works`, `/screens`, `/offline-lan`, `/pricing`, `/download`, `/help`, `/why-coachify`, `/contact`, `/faq`) render without runtime errors.
- [x] `EN | বাংলা` language toggle switches active locale (`en` ↔ `bn`), updates `document.documentElement.lang`, and persists in `localStorage`.
- [x] Product mockup screens and pricing display `৳` (`BDT`), Bangladesh coaching center batches (`HSC`, `Medical`, `BUET`, `SSC`), and commercial pricing (`Free`, `৳14,900`, `৳29,900`).
- [x] Download Demo modal displays version `v1.0.0`, SHA-256 checksum verification, and triggers installer download.
- [x] Request License modal and Contact form validate contact inputs and persist submitted leads.
