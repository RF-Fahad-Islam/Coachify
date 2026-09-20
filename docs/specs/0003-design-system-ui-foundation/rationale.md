# 0003. Design system and UI foundation: rationale

Decision record for [index.md](index.md). Read by engineers and by `/architect` on update or supersede, not during a normal build.

## Context

Coachify is a desktop coaching management system built for single PC and local network use in education institutes. Operating in intensive administrative environments like busy coaching centers means staff work long hours at computers, processing student admissions, batch attendance, exam results, and daily cash transactions.

Key forces that shape the design system:

- **Heavy data throughput**: Cashiers and reception staff process dozens of students and fee receipts every hour. They need high information density, fast keyboard navigation, and zero visual clutter.
- **Varying hardware and monitor resolutions**: Customer centers run older desktop PCs with 720p or 1080p monitors, as well as modern wide displays. A rigid layout or tiny unchangeable font causes immediate eye strain. Users must be able to adjust UI scale, density, and typography freely.
- **Brand pride and visual warmth**: Institutes want software that feels modern, energetic, and clean, rather than looking like an outdated accounting utility. Your reference designs demonstrate how warm orange accents, rounded metric cards, and clean status badges create an inspiring workspace.
- **Offline reliability**: Many centers experience intermittent or zero internet access. No remote Google Fonts or external CDN stylesheets can be loaded. Everything, including Latin and Bengali fonts, must be bundled locally inside the desktop installer.
- **Bilingual reality**: Institute staff, students, and parents in Bangladesh read English and Bangla simultaneously. The design system must support bilingual text pairs, Bangla currency symbols, and proper font rendering out of the box.

If we do not establish a solid token foundation now, every subsequent screen will invent its own ad hoc colors, paddings, and font sizes, resulting in visual drift and technical debt that takes weeks to unwind.

## Options considered

### Option 1: Tailwind CSS v4 CSS variables with accessible Radix primitives (Chosen)

A design token architecture using modern Tailwind CSS v4 with `@theme` and custom CSS variables, paired with accessible headless primitives from Radix UI.

**Pros**:
- Pure CSS configuration without bulky JavaScript configuration files.
- Instant, zero cost theme switching in real time by simply modifying attributes on the root HTML element.
- Accessible focus management, keyboard traversal, and screen reader labels built into Radix primitives.
- Code is fully owned in the repository, making customization simple.

**Cons**:
- Requires setting up token variables carefully across light and dark modes.

### Option 2: Prebuilt full UI component library (MUI or Ant Design)

Adopt a heavy off the shelf component framework like Material UI or Ant Design.

**Pros**:
- Ready made components with extensive default widget sets.
- Built in data table grids.

**Cons**:
- Heavy bundle size and runtime performance overhead on low memory 4 GB PCs.
- Fighting the default Material or enterprise styles to match your warm EduManage reference aesthetic takes more effort than building clean custom primitives.
- Harder to customize with dynamic density and bundled offline Bengali fonts.

### Option 3: Traditional CSS modules or styled components

Write individual CSS module files or use CSS in JS libraries for each component.

**Pros**:
- Total isolation of style rules.

**Cons**:
- Slower development velocity for AI agents and human developers alike.
- Runtime performance cost with CSS in JS libraries in React 19.
- Inconsistent spacing and color usage across different views without a shared utility framework.

## Rationale

Option 1 is the clear winner for Coachify. Tailwind CSS v4 provides exceptional developer velocity and minimal bundle size. By leveraging CSS variables for colors, typography, border radius, and spacing density, the entire application can switch palettes (Sunset Orange, Ocean Blue, Emerald, Violet, Rose, Slate) and density modes (Compact, Comfortable, Spacious) instantly without triggering React component remounts.

Accessible Radix primitives guarantee that keyboard navigation (`Tab`, `Shift+Tab`, `Arrow` keys, `Escape`, and `Enter`) works reliably for busy cashiers who prefer not to use the mouse. Bundling offline font files inside `ui/public/fonts/` ensures Coachify works in completely air gapped coaching centers across Bangladesh.

## References

- Tailwind CSS v4 Documentation and CSS Variables Architecture (basis: `lombiq/tailwind-agent-skills`)
- Accessible Component Primitives (basis: `shadcn/ui`, Radix UI)
- Human Interface Guidelines for Desktop Utility Systems
