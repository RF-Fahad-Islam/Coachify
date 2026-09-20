---
name: coachify-design-system
source: spec-0003
character: "A calm, dense, keyboard first look for coaching centers. Warm Sunset Orange accent with high contrast neutrals, crisp data tables, quick action shortcuts, and bundled bilingual Latin and Bangla typography."
tokens: "real values live in ui/src/styles/tokens.css; read them there, never duplicated here"
contrast: "body 14.1:1 light / 15.6:1 dark; on accent 4.8:1 light / 5.2:1 dark; border 3.2:1 against surface"
---

# Coachify Design System

## Build mandate

You are a senior product designer shipping a real desktop software product. Every page and dialog must leave as a complete, professional product surface with brand presence, real coaching center copy, clear visual hierarchy, all feedback states (empty, loading, error), supporting shortcuts, and contextual summaries. Never ship a bare input, an unstyled table, or an isolated button on an empty canvas.

## Character & direction

Coachify combines the productivity of a dense desktop accounting tool with the visual clarity of a modern academy suite. It draws direct inspiration from reference coaching platforms like EduManage and EduHub:

1. **Energetic warmth with serious utility**: Sunset Orange acts as the default brand accent, bringing energy to statistics and primary actions, while slate neutrals keep data entry calm and legible.
2. **Dense by design**: Coaching center cashiers and teachers process hundreds of records each day. Controls are compact, padding is disciplined, and key information is visible without excessive scrolling.
3. **Keyboard first fluidity**: Frequent workflows (taking attendance, collecting fee payments, opening search via `Ctrl+K`) provide explicit keyboard shortcuts and obvious focus rings.
4. **Offline bilingual typography**: Offline coaching centers in Bangladesh require dependable rendering for both English and Bangla. Plus Jakarta Sans and Noto Sans Bengali provide clean pairing without remote web fonts.

## Single source of token values

All concrete token values (colors, fonts, row heights, button dimensions, cell padding, radius sizes) are defined in [ui/src/styles/tokens.css](file:///c:/Users/User/Documents/Coachify/ui/src/styles/tokens.css) and exposed through Tailwind CSS v4 `@theme` variables in [ui/src/index.css](file:///c:/Users/User/Documents/Coachify/ui/src/index.css). Do not duplicate or hardcode raw color hex values in component files or in this document.

The design system supports:
- **Six palette presets**: `orange` (Sunset Orange default), `blue` (Ocean Blue), `emerald` (Emerald Growth), `violet` (Royal Violet), `rose` (Crimson Rose), `slate` (Monochrome Slate).
- **Two appearance modes**: `light` and `dark` with WCAG AA compliance.
- **Three density levels**: `compact` (36px table rows, 32px inputs), `comfortable` (44px table rows, 38px inputs), `spacious` (52px table rows, 44px inputs, default).
- **Four font options**: `system` (OS Native default), `plus-jakarta`, `inter`, `outfit`.
- **Three corner styles**: `sharp` (4px), `rounded` (8px default), `soft` (14px).

## Composition patterns

### 1. Desktop application shell
- **Left navigation rail**: Fixed 240px wide sidebar containing institute branding, active academic year tag, primary module navigation links with notification counts, and quick theme toggle banner.
- **Top utility bar**: 56px high utility bar housing the global search shortcut (`Ctrl+K`), active date range indicator, notifications trigger, and cashier profile dropdown.
- **Main workspace**: Standard padded area (`p-6`) containing breadcrumb context, title block with primary action, and responsive grid layouts.

### 2. High density data tables
- **Header**: Search filter, batch dropdown selector, export button, and item count badge.
- **Table container**: Rounded border container with subtle background for the header row. Columns align numbers to the right and text to the left.
- **Rows**: Explicit hover feedback, subtle cell border dividers, and status pills for payment or attendance state.
- **Footer**: Pagination controls showing current page, total records, and page size selector.

### 3. Financial and statistics cards
- **KPI MetricCard**: Built with default elevation shadow (`shadow-sm`), smooth hover depth (`hover:shadow-md`, `hover:border-primary/30`, `hover:-translate-y-0.5`), tinted icon box in top right, large bold metric value, delta indicator badge (example: `+12% from last month`), and explanatory subtitle.
- **Progress Card**: Dual track progress bar comparing received amount against total expected fees, with clear remaining balance callout.

### 4. Modal dialogs and drawers
- **Header**: Dialog title and descriptive subtitle explaining the action.
- **Body**: Grouped form fields with clear labels, placeholder hints, and inline validation messages.
- **Footer**: Secondary cancel button on the left, primary confirm button on the right.

## Component and usage rules

### Button
- **Primary**: Reserved for the single most important action on a surface (for example, Receive Payment, Save Attendance).
- **Secondary / Outline**: Used for filters, exports, and secondary actions.
- **Ghost**: Used for icon buttons, table row actions, and top bar shortcuts.
- **Destructive**: Reserved for irreversible actions (for example, Delete Student, Void Voucher) and always requires confirmation.

### Badge
- Status indicators must use semantic variants: `paid` (green), `pending` (amber), `overdue` (red), `active` (blue), `draft` (gray). Never use arbitrary custom colors for standard statuses.

### Input and Select
- Every input and select element must include an accessible label, explicit placeholder, and support density scaling. Focus rings use the active primary token with a 2px offset.

### Table
- Always use semantic `table`, `thead`, `tbody`, `tr`, `th`, `td` elements. Do not replace tables with nested generic divs for tabular data.
- Numbers, currency values, and dates align right. Text, student names, and roll numbers align left.

### Dialog and Sheet
- Trap keyboard focus inside the dialog while open. Pressing `Escape` closes the dialog. Returning focus to the trigger button on close is mandatory.

### Command and Search
- Global search is triggered by `Ctrl+K` or clicking the top bar search bar. Arrow keys navigate results, `Enter` selects an item, and `Escape` dismisses the palette.

## Responsive and accessibility direction

1. **Focus management**: Every interactive element must be reachable by keyboard navigation in natural visual order. Focus indicators use `ring-2 ring-primary` with clear separation from the element boundary.
2. **Color contrast**: Text on surface must exceed 4.5:1 ratio in both light and dark modes. Status pills use colored text on tinted subtle backgrounds to ensure readability.
3. **Screen readers**: Icon only buttons must provide an explicit `aria-label` attribute. Decorative icons must be hidden with `aria-hidden="true"`.
4. **Bilingual text rendering**: Use the font family token `--font-bangla` for Bengali script elements. Ensure line height accommodates ascenders and descenders in conjunct Bengali characters without vertical clipping.
