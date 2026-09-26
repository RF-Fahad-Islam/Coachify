# Coachify Commercial Website & Landing Pages

## Overview

Standalone React 19 and Tailwind CSS v4 web application hosting Coachify's marketing website, customer portal (`/portal`), downloads center (`/downloads`), help and support guides (`/help`), telemetry disclosure (`/privacy`), and interactive checkout modal.

## Key files

| File | Owns |
|---|---|
| `Landing pages/src/App.tsx` | Main application shell, client-side routing, and modal dialog state |
| `Landing pages/src/components/Pricing.tsx` | 4-tier pricing catalog cards, billing duration toggle, and checkout modal launcher |
| `Landing pages/src/components/PagePrimitives.tsx` | Shared UI tokens and accessible primitives (`WindowFrame`, `CtaButton`, `SectionTag`, `Chip`, `Stat`) |
| `Landing pages/src/pages/PortalAndPrivacyPages.tsx` | Customer portal overview, license file download, verified installers, paid invoices, and telemetry disclosure |
| `Landing pages/src/pages/DownloadPage.tsx` | Channel release notes, installer download, and SHA-256 checksum verification |
| `Landing pages/src/pages/SupportPages.tsx` | Searchable knowledge base, installation guide, and support links |
| `Landing pages/src/landing.test.tsx` | Unit and integration tests for landing views and portals |

## Commands

```bash
# Start local dev server
pnpm --dir "Landing pages" dev

# Build single-file production distribution
pnpm --dir "Landing pages" build

# Run unit tests
pnpm --dir "Landing pages" test

# Run linter
pnpm --dir "Landing pages" lint
```

## Conventions

- Named exports only in TypeScript, no default exports.
- Adhere strictly to the design system in `design.md` and semantic tokens: warm cream canvas (`#faf7f2`), ink headers (`#16110d`), and ember orange action elements (`#f97316`).
- Customer portal interactions use session OTP tokens without persistent cookies.
- Telemetry disclosure on `/privacy` must stay strictly accurate to the 5 fields transmitted by the desktop client.

_Drafted by /sync from the introducing change, worth a quick human pass._
