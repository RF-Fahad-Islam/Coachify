# Verify: Print engine and receipts (spec 0013, updated 2026-09-22)

Steps derived from spec 0013 acceptance criteria. The verification skill runs these; testing locks the durable steps.

## UI and manual

- [x] Open Settings → Printers: verify that installed system printers are listed and the default system printer is correctly marked (verifies AC-2, AC-3)
- [x] Configure `thermal_80mm` as the active receipt layout, select a receipt printer, and enable silent printing; record a test fee payment and verify that an 80 mm thermal receipt prints silently without an OS print dialog (verifies AC-1, AC-2, AC-4)
- [x] Switch active layout to `a4_dual`: print a payment receipt and verify that the sheet renders two equal halves (Student Copy and Office Copy) separated by a dotted line with scissors glyph (`✂`) (verifies AC-1)
- [x] Switch active layout to `a5_single`: print a payment receipt and verify that it fits on a single A5 sheet with compact formatting (verifies AC-1)
- [x] Inspect printed receipt and preview with a student having Bengali name (e.g., "মুহাম্মদ তানভীর আহমেদ") and complex Bengali numerals/words: verify that vowel signs and conjunct ligatures render with 100% fidelity without missing glyphs (verifies AC-7)
- [x] Open a posted receipt in `PrintPreviewDialog`: test zoom in/out, switch between 80 mm, A4, and A5 preview modes, and trigger manual print (verifies AC-5)
- [x] Click `Save as PDF` in the preview dialog: verify that the downloaded PDF file opens in an external PDF reader and visually matches the physical printout (verifies AC-6)
- [x] Simulate printer offline or disconnected state: record a payment and verify that the payment successfully commits to the database, while the UI displays a clear "Printer offline" notice with instant `[Reprint]` and `[Save PDF]` fallback buttons (verifies AC-8)
- [x] Click `[Reprint]` on a historical receipt: verify that the resulting printout carries a prominent `[DUPLICATE / REPRINT]` watermark or badge (verifies AC-1, AC-8)

## Commands

- [x] `cargo test -p coachify_core --test print_and_receipts` (verifies workstation printer settings serialization, system printer fallback resolution, receipt data aggregation with Bengali/English amount-in-words converter, and payment receipt data models, satisfies AC-1, AC-2, AC-3, AC-7, AC-8)
- [x] `cargo run -p xtask check-types` (verifies that TypeScript RPC bindings in `ui/src/gen/rpc.ts` match the Rust core contract for `printers.*` and `receipts.*`, satisfies AC-2, AC-3)
- [x] `pnpm --dir ui test` (verifies React print template rendering, layout classes for 80 mm, A4 dual-copy, and A5, and print preview dialog component tests, satisfies AC-1, AC-5, AC-7)

## Acceptance criteria coverage

- AC-1 (Three Standard Receipt Layouts): verified by template unit tests and manual print tests across 80 mm thermal, A4 dual-copy, and A5.
- AC-2 (Per-PC Workstation Printer Settings): verified by settings repository unit tests asserting persistence outside the institution database.
- AC-3 (System Printer Discovery): verified by core printer enumeration tests and Settings UI tests.
- AC-4 (Silent Receipt Printing): verified by desktop print trigger integration tests on Windows.
- AC-5 (Print Preview Dialog): verified by UI component tests checking zoom, layout switching, and manual print dispatch.
- AC-6 (PDF Export): verified by WebView2 PDF generation checks matching physical print DOM.
- AC-7 (Embedded Bengali Typography): verified by visual font rendering checks of Bengali conjuncts and amount-in-words converters.
- AC-8 (Decoupled Fault-Tolerant Printing): verified by transaction rollback prevention tests asserting payment persistence despite print errors.
