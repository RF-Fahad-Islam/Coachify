# 0003. Design system and UI foundation

**Date**: 2026-09-20
**Status**: Accepted

## Summary

Coachify adopts a calm, dense, keyboard first design system built on Tailwind CSS v4 and accessible Radix primitives. The visual language matches your references for EduManage and the thirteen screen EduHub coaching suite, with an energetic Sunset Orange primary palette, clear data cards, attendance trends, fee collection gauges, and status pills. The system supports full customization through CSS variables, including six color palettes, dark and light modes, four font families, three information density levels, and global UI scaling for high resolution monitors. All Latin and Bangla fonts are bundled directly inside the application so offline coaching centers run smoothly without an internet connection.

## Requirements

**User stories**:
- As a coaching center administrator, I want a clean, professional interface with clear metrics, trend cards, and quick action shortcuts so that daily management is fast and intuitive.
- As a cashier handling hundreds of fee payments daily, I want a compact, high density view with keyboard shortcuts so that I can process payments rapidly without mouse fatigue.
- As a teacher entering exam marks or attendance, I want clear status badges and keyboard navigation across table cells so that grade entry is fast and error free.
- As an institute owner, I want to customize the institute brand color, typography, and UI scale to match my center identity and monitor size.
- As a Bengali speaker, I want crisp, correctly shaped Bangla typography alongside English text in student names, receipts, and navigation.

**Acceptance criteria**:
- **AC-1**: Tailwind CSS v4 is integrated with a pure CSS token architecture using `@theme` and semantic CSS variables in `ui/src/styles/tokens.css` without requiring legacy JavaScript config files.
- **AC-2**: The theme engine provides six curated color palette presets (Sunset Orange default, Ocean Blue, Emerald Growth, Royal Violet, Crimson Rose, Monochrome Slate) switchable in real time via root dataset attributes (`data-palette`).
- **AC-3**: Light and dark appearance modes render crisp contrast ratios exceeding WCAG 2.2 AA standards across all cards, text elements, status badges, and table rows.
- **AC-4**: The typography system bundles Latin fonts (Plus Jakarta Sans default, Inter, Outfit, System) and Bangla fonts (Noto Sans Bengali, Hind Siliguri) locally in `ui/public/fonts/`, supporting instant family switching and font size scaling.
- **AC-5**: Three information density modes (Compact with 36px table rows, Comfortable with 44px table rows, Spacious with 52px table rows) adjust cell padding, button heights, and font size for data intensive workflows.
- **AC-6**: Core accessible primitives in `ui/src/components/ui/` (Button, Input, Select, Dialog, DropdownMenu, Tabs, Table, Badge, Card, MetricCard, Progress, Avatar, Tooltip, Switch, Command) export named bindings only and handle focus rings and keyboard interaction.
- **AC-7**: The desktop app shell implements a collapsible left navigation sidebar, top search and session bar with `Ctrl+K` shortcut, and an interactive theme customizer dialog that persists preferences to local storage and syncs with the core settings database.

## Decision

**Chosen option**: Option 1: Tailwind CSS v4 semantic design tokens with accessible Radix primitives, bundled offline fonts, and real time CSS variable customization.

You build the Coachify design system and user interface foundation using Tailwind CSS v4, accessible headless primitives in `ui/src/components/ui/`, bundled offline Latin and Bangla fonts, and an application shell patterned directly after the EduManage and EduHub reference designs.

**Implementation skills**: `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `react-patterns` (`.claude/skills/react-patterns/`)

## Rationale

Reasoning and evaluated options: see [rationale.md](rationale.md).

## Design System Specifications

### 1. Token Architecture

All design tokens are defined as CSS variables under root and dark selectors. Changing a theme or density modifies the attributes on `document.documentElement` without requiring a full page refresh.

```css
:root {
  /* Font Families */
  --font-sans: "Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-bangla: "Noto Sans Bengali", "Hind Siliguri", sans-serif;
  
  /* Sizing and Density Defaults */
  --ui-scale: 1;
  --radius: 0.5rem; /* 8px default */
  --table-row-height: 2.75rem; /* 44px comfortable */
  --btn-py: 0.5rem;
  --btn-px: 1rem;
  --card-padding: 1.25rem;

  /* Semantic Core Colors (Default Sunset Orange) */
  --color-primary: oklch(0.62 0.21 42); /* #ea580c */
  --color-primary-hover: oklch(0.56 0.22 40);
  --color-primary-foreground: oklch(0.99 0 0);
  --color-primary-subtle: oklch(0.97 0.03 48); /* #fff7ed */
  --color-primary-border: oklch(0.92 0.06 50);

  /* Neutrals and Surfaces */
  --color-background: oklch(0.985 0.002 247); /* #f8fafc */
  --color-foreground: oklch(0.20 0.03 260); /* #0f172a */
  --color-card: oklch(1 0 0); /* #ffffff */
  --color-card-foreground: oklch(0.20 0.03 260);
  --color-muted: oklch(0.96 0.005 250);
  --color-muted-foreground: oklch(0.55 0.02 255); /* #64748b */
  --color-border: oklch(0.92 0.005 250); /* #e2e8f0 */

  /* Semantic Status Indicators */
  --color-success: oklch(0.60 0.18 145); /* green */
  --color-success-subtle: oklch(0.96 0.04 145);
  --color-warning: oklch(0.70 0.17 65); /* amber */
  --color-warning-subtle: oklch(0.97 0.05 65);
  --color-danger: oklch(0.60 0.22 25); /* red */
  --color-danger-subtle: oklch(0.97 0.04 25);
  --color-info: oklch(0.60 0.18 240); /* blue */
  --color-info-subtle: oklch(0.96 0.04 240);
}
```

### 2. Palette Presets

| Palette Key | Name | Primary Color | Tone and Character |
|---|---|---|---|
| `orange` | Sunset Orange (Default) | `#ea580c` / `#f97316` | Warm, energetic education coaching aesthetic matching reference |
| `blue` | Ocean Blue | `#2563eb` / `#3b82f6` | Calm, professional corporate academy feel |
| `emerald` | Emerald Growth | `#059669` / `#10b981` | Fresh, reassuring progress and growth |
| `violet` | Royal Violet | `#7c3aed` / `#8b5cf6` | Prestigious, refined modern institute |
| `rose` | Crimson Rose | `#e11d48` / `#f43f5e` | Vibrant, punchy and bold academic branding |
| `slate` | Monochrome Slate | `#334155` / `#64748b` | Ultra minimal, focused high contrast data utility |

### 3. Density and Sizing Matrix

| Density Level | Key | Table Row Height | Input Height | Base Text Size | Typical Use Case |
|---|---|---|---|---|---|
| Compact | `compact` | 36px | 32px | 13px / 0.8125rem | Cashier fee collection, bulk mark entry, attendance sheets |
| Comfortable | `comfortable` | 44px | 38px | 14px / 0.875rem | General administrative work, student profile, settings (default) |
| Spacious | `spacious` | 52px | 44px | 15px / 0.9375rem | Executive dashboard, high resolution displays, touch devices |

### 4. Component Catalog (`ui/src/components/ui/`)

Every component is authored with strict TypeScript types, named exports only, and zero default exports:

1. **Button**: Variants include `primary`, `secondary`, `outline`, `ghost`, `destructive`, `subtle`. Sizes include `sm`, `md`, `lg`, `icon`. Sizing adapts to density.
2. **Badge**: Status pill indicators with semantic variants `paid` (green), `pending` (amber), `overdue` (red), `active` (blue), and `draft` (gray).
3. **MetricCard**: KPI card featuring an icon with tinted background box, primary metric value, delta indicator badge (example: `+12%`), subtitle note, and optional navigation arrow.
4. **ProgressBar**: Dual track indicator showing current collected amount versus total target, with percentage label and remaining balance note.
5. **Table**: Clean data table with sticky header support, zebra row option, compact density styles, and cell truncation.
6. **Input & Select**: Accessible text, number, and search inputs with clear focus rings and error message slots.
7. **Tabs**: Segmented tab strip for sub view navigation and admissions pipeline stages (Inquiries, Applicants, Admitted, Rejected).
8. **Dialog & Sheet**: Accessible modal windows and side drawers for quick payment creation, new admissions, and settings.
9. **Command & Kbd**: Keyboard driven search palette accessible via `Ctrl+K`.
10. **Avatar**: Circular student and staff profile image with initials fallback.

### 5. Application Shell Structure

The desktop shell organizes navigation and workspace views:
- **Left Sidebar**: 240px wide collapsible rail. Houses the Coachify logo, coaching center name, nine primary navigation items with count badges, and an institute promo or quick help card at the bottom.
- **Top Application Bar**: 56px high utility bar. Houses the global search input with keyboard shortcut hint (`Ctrl+K`), date range and academic session filter, notification bell with unread badge, and user profile menu.
- **Main Workspace Container**: Scrollable area with standard padding (`p-6`), containing the page header, action bar, and responsive layout grids.
- **Theme Customizer Modal**: Instant preview and configuration dialog to switch palette, mode, font, density, scale, and border radius.

## Value Sourcing

| Action / Element | Value Produced or Displayed | Source |
|---|---|---|
| Theme Switcher | Current palette name and active color | Local state in `ThemeProvider`, synced with `system_settings` table |
| Typography Switcher | Active font family and size scale | Local state in `ThemeProvider`, synced with `system_settings` table |
| Density Selector | Compact, comfortable, or spacious class attributes | Local state in `ThemeProvider`, synced with `system_settings` table |
| Metric Stat Card | Count, total amount, delta percent | Props passed from view model or RPC query result |
| Currency Formatting | Formatted Taka string with poisha conversion | Shared utility `formatTaka(poisha)` in `ui/src/lib/utils.ts` |
| Date Formatting | Localized civil date in Asia/Dhaka | Shared utility `formatBusinessDate(dateStr)` in `ui/src/lib/utils.ts` |
| Student Avatar | Photo image or initials abbreviation | Student record `photo_vault_id` or initials derived from `name` |

## Build Plan

Follows the Tracer Bullet delivery approach. It proves one thin end to end thread through token definitions, component primitives, layout shell, and theme switching first, then expands component variants.

### Slice 1: Core Dependencies and Token Infrastructure
- Install Tailwind CSS v4, `@tailwindcss/vite`, Radix primitives, Lucide icons, and utility packages, covers **AC-1**.
- Configure Vite plugin and path aliases in `ui/vite.config.ts`, covers **AC-1**.
- Create `ui/src/styles/tokens.css` with six palette presets, light and dark themes, density variables, and typography rules, covers **AC-1**, **AC-2**, **AC-3**.
- Create `ui/src/lib/utils.ts` with `cn()` and formatting helpers, covers **AC-1**.

### Slice 2: Theme Provider and Real Time Switching Engine
- Create `ui/src/theme/theme-provider.tsx` and `use-theme.ts` hook managing palette, mode, font, density, scale, and radius, covers **AC-2**, **AC-3**, **AC-4**, **AC-5**.
- Build `ThemeCustomizerDialog` allowing users to preview and toggle themes interactively, covers **AC-7**.

### Slice 3: Core Component Primitives Library
- Build atomic components in `ui/src/components/ui/`: `Button`, `Badge`, `Card`, `MetricCard`, `ProgressBar`, `Input`, `Select`, `Table`, `Tabs`, `Avatar`, `Skeleton`, `Dialog`, `DropdownMenu`, covers **AC-6**.
- Add unit tests for component rendering and density classes in `ui/src/components/ui/components.test.tsx`, covers **AC-6**.

### Slice 4: Desktop Application Shell and Reference Dashboard Layout
- Build `AppShell`, `Sidebar`, `TopBar` conforming to EduManage navigation structure, covers **AC-7**.
- Wire up interactive reference dashboard demonstrating KPI metric cards, attendance trend chart mockup, fee collection progress bar, and recent students table, covers **AC-6**, **AC-7**.
- Verify complete offline operation, keyboard navigation, and theme customization, covers **AC-1**, **AC-2**, **AC-3**, **AC-4**, **AC-5**, **AC-6**, **AC-7**.

## Critical Test Scenarios

- **Happy path theme switching**: Changing theme palette from Sunset Orange to Ocean Blue immediately updates CSS variables on `document.documentElement` without page reload, verifies **AC-2**.
- **Dark mode contrast**: Toggling dark mode maintains high contrast borders on cards (`#334155`), dark surface background (`#0f172a`), and readable muted text, verifies **AC-3**.
- **Density adjustment**: Switching from Comfortable to Compact reduces table row height to 36px and button padding to py-1.5, allowing greater data density on screen, verifies **AC-5**.
- **Keyboard navigation**: Pressing `Ctrl+K` opens the search command dialog from anywhere in the app, and `Escape` closes it, verifies **AC-7**.
- **Bangla text display**: Student names and currency amounts in Bengali script render with correct conjunct shaping using bundled offline fonts, verifies **AC-4**.

## Consequences

- **What becomes easier**: Every subsequent feature (Admissions, Attendance, Payments, Exams, Reports) builds rapidly using standard, battle tested primitives with consistent padding, colors, and keyboard accessibility.
- **What becomes harder**: Custom one off styling in individual screens is discouraged. All colors and dimensions must reference semantic tokens rather than arbitrary Tailwind hex values.
- **Offline boundary**: No external font CDNs (such as Google Fonts) may be linked in `index.html`. All assets must reside locally in `ui/public/`.

## Follow-up

- In Feature 5 (Command Dispatcher and Local Accounts), connect the theme preferences to persistent user profiles in SQLite via an RPC command `system.update_preferences`.
- In Feature 13 (Internationalization and Receipt Templates), wire `i18next` catalogs into all navigation labels and table headers.
