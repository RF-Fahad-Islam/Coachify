# 0013. Print engine and receipts (Rationale)

## Context

Coaching centers in Bangladesh operate under challenging physical and hardware conditions. Administrative counters frequently rely on older 64-bit Windows 10 computers connected to diverse printers: inexpensive 80 mm thermal POS receipt printers (e.g. Xprinter, Rongta, POS-80C) via USB, standard desktop laser printers (HP LaserJet, Canon) printing A4 or A5 paper, and occasional inkjets.

Receipts must display bilingual information: institution name and address in Bengali, student names in English and Bengali, fee heads, numerical amounts in Taka, and formal amount-in-words text in both English and Bengali. Printing must be instantaneous; during peak admission or monthly fee collection hours, long queues form at the counter, and cashiers cannot wait 10 seconds for a heavy print dialog or deal with garbled fonts.

Furthermore, power cuts and paper jams are everyday occurrences. If a printer runs out of paper mid-receipt or gets jammed, the underlying payment transaction must already be securely recorded in the database, and the cashier must be able to reprint immediately upon clearing the jam without duplicating the transaction.

## Options considered

### Option 1: React Print Components styled with `@media print` / `@page` and Windows Edge WebView2 native printing

Design print templates as modular React components with dedicated print CSS rules (`@page { size: ...; margin: 0; }`). In the desktop shell, silent printing uses Edge WebView2's COM API (`ICoreWebView2_16::PrintAsync`) targeting the configured receipt printer, and PDF generation uses `PrintToPdfAsync`. In standard web or preview mode, printing falls back to `window.print()`. Open-licensed Bengali fonts (`Kalpurush` and `Noto Sans Bengali`) are bundled into the app bundle.

**Pros**:
- Flawless Bengali text shaping: Chromium's HarfBuzz text shaping engine renders Bengali vowel signs (kar), consonant ligatures (juktakkhors), and Bengali numerals with 100% fidelity.
- Shared templates: Preview dialog, physical printout, and exported PDF share the exact same React DOM layout, eliminating visual discrepancies.
- Flexible paper formats: CSS `@page` seamlessly supports 80 mm continuous roll, A4 dual-copy, and A5 without separate rendering engines.
- Per-workstation independence: Each PC configures its own local receipt and report printers without database contention.

**Cons**:
- True silent printing without dialogs requires Windows WebView2 COM interfaces in the desktop shell.
- Bundled web fonts add ~1.2 MB to the distribution package.

### Option 2: Raw ESC/POS direct USB printing to thermal printers

Send raw ESC/POS byte commands directly to the receipt printer over USB/spooler for fast thermal printing, with a separate HTML/PDF pipeline for A4 laser printers.

**Pros**:
- Extremely fast printing on thermal printers (tens of milliseconds).
- Direct control over hardware cash drawers and paper cutters.

**Cons**:
- Raw ESC/POS hardware fonts cannot render Bengali Unicode scripts; Bengali text appears as broken glyphs, question marks, or boxes unless rendered as a bitmap on the host PC first.
- Requires maintaining two completely distinct template architectures: ESC/POS byte streams for thermal and HTML/PDF for laser printers.
- Fragile across different printer models and driver emulation modes.

### Option 3: Headless Rust PDF generation with print spooler dispatch

Generate PDF documents directly in Rust using libraries such as `printpdf` or `genpdf`, and send the resulting PDF to the Windows print spooler via Win32 APIs.

**Pros**:
- Pure Rust implementation that runs headless in core without web engine dependencies.
- Generates standalone PDF files easily.

**Cons**:
- Complex text layout, word wrapping, and dynamic pagination are tedious and error-prone to code in imperative Rust compared to CSS.
- Poor or absent Bengali text shaping in most Rust PDF libraries, resulting in broken character ordering and disconnected conjuncts.
- Slower iteration cycle for design tweaks compared to React/Tailwind print templates.

## Rationale

Option 1 is selected because it delivers superior typography and visual fidelity for the bilingual requirements of Bangladeshi institutes while maintaining a single, unified template architecture.

Because WebView2 is the foundational desktop shell in Tauri, using Chromium's mature print engine provides pixel-perfect font shaping for Bengali without the maintenance overhead of raw ESC/POS drivers. Storing printer settings per PC in local configuration satisfies the operational reality of multi-counter centers where counter workstations have dedicated thermal printers while back-office staff use laser printers. Finally, committing financial mutations before initiating print dispatch ensures that accounting ledgers remain untampered and accurate even if a printer experiences a hardware fault.
