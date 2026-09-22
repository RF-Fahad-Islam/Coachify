# Verify: Command Palette (spec 0030)

Steps derived from spec 0030 acceptance criteria.

## UI and Manual Verification

- [ ] Press `Ctrl+K` on Windows/Linux (or `Cmd+K` on macOS): verify that the Command Palette dialog opens immediately (under 50 ms) with focus trapped in the search input (verifies AC-1).
- [ ] Press `Escape`: verify that the Command Palette closes immediately and restores focus to the trigger (verifies AC-1).
- [ ] Open the palette and type `"pay"` or `"fee"`: verify that the "Receive Fee Payment" action appears at the top under Quick Actions with shortcut `N P` (verifies AC-2, AC-4).
- [ ] Press `Enter` with "Receive Fee Payment" selected: verify that the palette closes and the cashier counter opens (verifies AC-2).
- [ ] From any tab outside of an active input field, press `G` then `S`: verify that the application immediately navigates to the Students Directory (verifies AC-3).
- [ ] From any tab, press `N` then `S`: verify that the New Student admission modal opens immediately (verifies AC-3).
- [ ] Focus an input field (e.g. search box or student name input) and type `"gs"` or `"np"`: verify that literal letters are typed and chorded shortcuts do NOT trigger navigation (verifies AC-3).
- [ ] Type a student name or code (e.g. `"Rahim"` or `"STD-2026-0001"`): verify that matching student records appear under the "Live Search Entities" section with their student code and guardian name (verifies AC-2).
- [ ] Log in as a Receptionist: open the palette and verify that administrative actions (e.g. "Trigger Database Backup") are completely hidden (verifies AC-5).

## Commands

- [ ] `pnpm --dir ui test` (verifies CommandPalette component mounting, chorded shortcut detection, focus trapping, and permission filtering, satisfies AC-1 through AC-6).

## Acceptance Criteria Coverage

- AC-1 (Instant Mount & Dismissal): verified by `test_command_palette_mount_under_50ms` and `Escape` key handling tests.
- AC-2 (Categorized Groups): verified by group header rendering assertions (`Quick Actions`, `Navigation`, `Entities`).
- AC-3 (Chorded Shortcuts): verified by `useGlobalShortcuts` test suite covering `G S`, `N P`, `N S`, and input focus guards.
- AC-4 (Fuzzy Matching & Aliases): verified by alias matching tests for `"pay"`, `"admit"`, and `"due"`.
- AC-5 (Role-Aware Filtering): verified by asserting that unauthorized actions are omitted based on current role permissions.
- AC-6 (Accessibility): verified by ARIA attribute assertions on dialog container and items.
