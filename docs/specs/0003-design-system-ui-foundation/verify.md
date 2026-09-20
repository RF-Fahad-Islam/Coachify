# Verify: Design system and UI foundation (spec 0003, updated 2026-09-20)

Steps derived from spec 0003 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual
- [ ] Launch development server and inspect default visual styling, ensuring Sunset Orange primary tokens, clean white cards, and rounded borders render properly (verifies AC-1, AC-2)
- [ ] Open the theme customizer dialog and switch between all six color palettes, observing instant token updates across all controls (verifies AC-2)
- [ ] Toggle dark mode and verify contrast compliance on text, cards, muted badges, and borders (verifies AC-3)
- [ ] Switch font families between Plus Jakarta Sans, Inter, and Outfit, confirming font glyphs load cleanly without external network calls (verifies AC-4)
- [ ] Toggle between Compact and Comfortable density modes, verifying table row heights change from 36px to 44px (verifies AC-5)
- [ ] Press `Ctrl+K` from any screen to verify that the universal search dialog opens and traps focus, and press `Escape` to close it (verifies AC-6, AC-7)

## Commands
- [ ] `pnpm --dir ui typecheck` -> Confirms strict TypeScript type safety across all tokens, themes, and component props (verifies AC-1, AC-6)
- [ ] `pnpm --dir ui lint` -> Confirms code style and lint rules (verifies AC-1, AC-6)
- [ ] `pnpm --dir ui format:check` -> Confirms formatting consistency (verifies AC-1)
- [ ] `pnpm --dir ui test` -> Runs Vitest unit tests verifying theme provider state and atomic component rendering (verifies AC-2, AC-5, AC-6)

## Acceptance criteria coverage
- AC-1 (Tailwind CSS v4 pure CSS token architecture): verified by build check and token inspections.
- AC-2 (six curated color palette presets): verified by manual theme switcher test and unit tests.
- AC-3 (light and dark mode contrast): verified by dark mode visual inspection and contrast validation.
- AC-4 (offline bundled Latin and Bangla fonts): verified by offline network inspection and family switching.
- AC-5 (compact, comfortable, and spacious density modes): verified by table row height tests.
- AC-6 (accessible component primitives): verified by Radix keyboard and focus tests.
- AC-7 (desktop application shell and theme dialog): verified by shell rendering, search dialog, and setting persistence.
