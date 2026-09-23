# 0034. UI/UX Design Refinements, Typography Scale, Spacing, and Responsiveness

**Date**: 2026-09-23
**Status**: Accepted

## Summary

This enhancement standardizes Coachify's design system across typography, sizing, surface contrast, fluid spacing, and universal responsive ergonomics. Grounded in the principles of `design.md`, the UI/UX Pro Max intelligence framework, and the `/architect` spec methodology, it resolves readability and layout degradation across mobile and tablet devices while retaining high-density keyboard-driven efficiency for desktop workstations.

## Requirements

**User stories**:
- As a coaching administrator using a tablet or mobile device, I want a responsive slide-over drawer navigation and hamburger menu so that I can manage classes and students without screen clipping or horizontal overflow.
- As a cashier processing hundreds of payments on different displays, I want high contrast text, crisp card borders, and large touch-friendly payment method buttons so that I can collect fees quickly and accurately.
- As a teacher entering attendance on an iPad or laptop, I want comfortable line heights, legible font sizing without sub-11px microcopy, and responsive data tables with smooth horizontal scrolling.

**Acceptance criteria**:
- **AC-1**: Standardized modular typography scale with Display (24–28px), Headings (18–20px), Subheadings (14–15px), Body (13–14px with `leading-relaxed` 1.5–1.625), Captions (12px), and Micro-badges (11px minimum). Sub-11px text is completely eliminated.
- **AC-2**: Elevated light-mode contrast: Muted text set to `#475569` (slate-600, 7.0:1 contrast on surface) and card borders set to `#e2e8f0` (slate-200) for crisp definition on all displays.
- **AC-3**: Fluid layout spacing system based on an 8pt grid with responsive main workspace padding (`p-3 sm:p-4 md:p-6 lg:p-8`) and grid gaps (`gap-3 sm:gap-4 md:gap-6`).
- **AC-4**: Responsive application shell: Persistent 240px sidebar rail on desktop viewports (`>= 1024px`) and an accessible off-canvas slide-over drawer with backdrop overlay on mobile and tablet viewports (`< 1024px`).
- **AC-5**: Adaptive top navigation bar featuring a mobile hamburger toggle button, fluid search bar width (`max-w-[220px] sm:max-w-xs md:max-w-md`), and minimum 40–44px touch tap targets.
- **AC-6**: Responsive Dashboard view wrapping KPI cards (1-col mobile, 2-col tablet, 4-col desktop), stacked middle section, and responsive tables.
- **AC-7**: Tablet-ready Cashier Desk POS layout stacking cleanly into a single-column flow on viewports `< 1280px` and maintaining 7/5 split on desktop, with multi-row touch payment chips and quick-fill buttons.

## Decision

**Chosen option**: Refactor core tokens, shell layout, and primary view layouts to support responsive breakpoints (375px mobile, 768px tablet, 1024px desktop, 1440px wide) while strengthening WCAG 2.2 AA contrast and modular typography.

**Implementation skills**: `ui-ux-pro-max`, `tailwind-4-docs`, `shadcn`, `react-patterns`.

## Build Plan

Follows the Tracer Bullet delivery approach:
1. **Tokens & CSS Foundation**: Update `tokens.css` with `#475569` muted text and `#e2e8f0` borders. Enhance `index.css` with modular typography line-heights and heading tracking.
2. **Responsive Shell**: Add `mobileSidebarOpen` state to `AppShell`, build mobile drawer and backdrop in `Sidebar`, and add hamburger menu toggle with fluid search bar in `TopBar`.
3. **Primary Views**: Apply responsive grid breakpoints and modular font sizes in `DashboardView` and `CashierDeskView`.
4. **Verification**: Run unit test suites, typechecks, and responsive viewport tests.

## Consequences

- **What becomes easier**: Using Coachify on tablet workstations, point-of-sale touch screens, or compact monitors is intuitive and overflow-free.
- **What becomes harder**: Arbitrary hardcoded Tailwind text sizes (like `text-[10px]`) are forbidden; components must strictly reference modular typography tokens and classes.
