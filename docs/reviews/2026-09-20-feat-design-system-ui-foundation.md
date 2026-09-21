# Review, branch, 2026-09-20

**Reviewed by**: Antigravity (author on <author-model>)
**Scope**: 43 files, branch
**Verdict**: Changes requested

## Summary
The change correctly establishes a robust Tailwind CSS v4 design system, theme switching logic, and a suite of accessible UI primitives. The foundation aligns beautifully with the Coachify branding and the offline requirements. However, the offline font files were omitted from the commit, which breaks the typography requirement for offline deployment.

## Major
### 🟠 Missing bundled font files, `ui/public/fonts/`
**Problem**: The `ui/public/fonts/` directory only contains a `README.md` file. The actual `.woff2` font files for Latin (Plus Jakarta Sans, Inter, Outfit) and Bangla (Noto Sans Bengali, Hind Siliguri) are missing.
**Why it matters**: Without the bundled font files, the app will fail to load the fonts and fallback to system defaults. The specification strictly requires offline typography support, which this change currently breaks.
**Suggested fix**: Download the `.woff2` font files as specified in the README and commit them to the `ui/public/fonts/` directory. Check `.gitignore` if they were accidentally excluded.

## Minor
### 🟡 Untested layout and feature components, `ui/src/components/layout/` and `ui/src/App.tsx`
**Problem**: While the core UI primitives in `ui/src/components/ui/` are well-tested, the major feature components including `app-shell.tsx`, `sidebar.tsx`, `top-bar.tsx`, `theme-customizer-dialog.tsx`, and `App.tsx` have no test coverage.
**Why it matters**: These components handle essential interactions like theme switching logic, tab navigation, and RPC calls. Lacking tests leaves them vulnerable to regression.
**Suggested fix**: Add a basic rendering and interaction test suite for the layout components and the main App shell to verify tab switching and dialog triggers.

## Nits
- ⚪ `ui/src/styles/tokens.css:26`, The spec suggested using `oklch` for the semantic core colors (e.g., `--color-primary: oklch(0.62 0.21 42)`). The current implementation uses hex values (`#ea580c`). Tailwind v4 natively supports `oklch`, but hex is functionally fine.

## Strengths
- The `ThemeCustomizerDialog` and CSS variable injection (`document.documentElement.setAttribute`) is cleanly implemented and avoids expensive React re-renders.
- Primitives use Radix UI, ensuring robust keyboard navigation and accessibility out-of-the-box.
- Strict adherence to the "named exports only" rule across the entire UI codebase.

## Test coverage
The core primitives (`components.test.tsx`), utility functions, i18n, and theme logic are well-covered. However, the higher-level layout components, dashboard view, and application shell (`App.tsx`) are completely untested.

## Fixes applied (2026-09-20)
- **Major (Bundled offline font assets)**: Downloaded and bundled `PlusJakartaSans.woff2` and `NotoSansBengali.woff2` directly into `ui/public/fonts/`. Full `vite build` now resolves both font files locally with zero warnings.
- **Minor (Layout and shell test coverage)**: Created `ui/src/components/layout/layout.test.tsx` providing test coverage for `Sidebar`, `TopBar`, `QuickSearchDialog`, `ThemeCustomizerDialog`, `AppShell`, and `DashboardView`. Total UI test count increased to 36 tests across 6 test suites, all passing.

