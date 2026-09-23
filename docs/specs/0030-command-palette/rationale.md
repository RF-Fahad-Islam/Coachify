# 0030. Command Palette (Rationale)

## Context

Coaching center operators and counter staff handle high volumes of students and parents throughout the day. Switching between mouse and keyboard slows down counter throughput, increases queue wait times, and causes repetitive strain.

Power users in desktop accounting and operating systems (such as VS Code, Linear, or macOS Spotlight) rely heavily on a global palette to trigger actions and navigate without mouse clicks. In Coachify, introducing `Ctrl+K` and chorded shortcuts transforms data entry from a multi-step mouse traversal into an instantaneous keystroke flow.

---

## Options Considered

### Option 1: Client-Side Cached Command Registry with Two-Key Chords and Integrated Search (Chosen)

- Maintain a fast, client-side registry of all commands, navigation targets, and keywords.
- Support chorded two-letter shortcuts (`G S`, `N P`, `N S`) inspired by Gmail and GitHub.
- Trigger dynamic `search.global` queries when input is not an action keyword.
- Display shortcut badges beside each command to promote discoverability.

Pros:
- Sub-50ms opening latency since static actions are indexed in memory.
- Natural learning curve: users see shortcuts in the palette and memorize them over time.
- Power users can execute counter operations in seconds without touching a mouse.
- Deep linking allows navigating directly to a student, batch, or receipt.

Cons:
- Must ensure chord listener does not conflict with normal typing inside form input fields.

### Option 2: Server-Driven Menu/Action Dispatcher

Send search queries to the Rust backend to resolve matched action items and navigation targets dynamically on every keystroke.

Pros:
- Backend manages the full list of application actions.

Cons:
- Introduces round-trip IPC latency (10–30ms) for simple navigation actions.
- Unnecessary overhead for static client-side navigation.

### Option 3: Standard Top Navigation Menu Only (No Command Palette)

Rely exclusively on the left sidebar and top navigation buttons with hover dropdowns.

Pros:
- Simpler UI implementation.

Cons:
- Completely misses the "Operational Speed" differentiator for Coachify.
- Fails the project's desktop ergonomics and keyboard-first design mandate in `design.md`.
