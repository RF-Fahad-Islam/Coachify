# 0013. Print engine and receipts

**Date**: 2026-09-22
**Status**: Accepted

## Summary

Coachify delivers offline-first printing and receipt generation for educational institutes and coaching centers in Bangladesh. Counter cashiers print receipts instantly after collecting fees, while administrators and staff print due statements, admission forms, and financial reports. Because Windows PCs in coaching offices are often entry-level machines with disparate printer hardware—ranging from 80 mm thermal POS receipt printers to desktop laser and inkjet printers—the printing architecture must handle diverse paper sizes without external cloud dependencies or native font rendering quirks.

Print templates are defined as modular React components styled with print CSS (`@media print`, `@page`). In the Tauri desktop shell on Windows, the application leverages Microsoft Edge WebView2's native printing capabilities (`ICoreWebView2_16::PrintAsync` and `PrintToPdfAsync`) to achieve true silent printing for fee receipts and pixel-perfect PDF export. Open-licensed Bengali fonts are embedded directly into the application, ensuring that complex Bengali script shaping, conjunct ligatures, and numerals render cleanly across all Windows versions. Printer configurations are managed strictly per-workstation in local machine storage, and a decoupled error handling model guarantees that a printer paper jam, offline state, or missing driver never aborts or corrupts an already committed financial payment.

## Requirements

**User stories**:
- As a counter cashier, I want fee receipts to print silently and automatically to my connected receipt printer upon receiving payment, so that students in line are served without dialog delays.
- As an institute owner, I want receipts to support 80 mm thermal roll, A4 dual-copy (student and office copies), and A5 formats, so that we can use whichever paper format our center prefers.
- As a cashier, I want Bengali student names, institution addresses, and amounts in words to render cleanly without garbled characters, so that receipts look professional and legible.
- As an office administrator, I want to configure which physical printer handles receipts versus reports on my specific computer, so that counter workstations print to local thermal printers while back-office PCs print to laser printers.
- As a cashier whose printer ran out of paper, I want the recorded payment to remain safely saved in the database with an immediate option to reprint or save a PDF, so that hardware issues never create lost or duplicate transactions.
- As an accountant, I want to reprint any historical receipt with an automatic reprint indicator, so that duplicate copies can be issued when requested by parents.

**Acceptance criteria**:
- **AC-1**: Support three standardized receipt layouts:
  1. `thermal_80mm`: 80 mm continuous roll (72 mm printable width) with itemized fee table, receipt barcode/QR code, and compact Bengali/English typography.
  2. `a4_dual`: A4 sheet (210 mm × 297 mm) containing two identical copies—Student Copy on top and Office Copy on bottom—separated by a dotted perforation cut line with a scissors glyph (`✂ - - - - - - - -`).
  3. `a5_single`: A5 sheet (148 mm × 210 mm) formatted for single-voucher half-page laser printing.
- **AC-2**: Printer settings are stored per workstation in local device configuration (`%ProgramData%\Coachify\printer_settings.json` with web storage fallback), never in the shared institution database. Settings persist: `receipt_printer`, `report_printer`, `receipt_layout`, `silent_receipt_printing`, `auto_print_on_payment`, and `copies`.
- **AC-3**: Provide printer discovery (`printers.list`) returning installed Windows system printers and indicating the default system printer.
- **AC-4**: Support silent printing for fee receipts when `silent_receipt_printing = true`, printing directly to the designated `receipt_printer` without presenting an operating system print dialog.
- **AC-5**: Provide an in-app `PrintPreviewDialog` for reports, due statements, and manual reprints, featuring layout switching, zoom controls, a manual Print button, and PDF download.
- **AC-6**: Export pixel-perfect PDF documents via WebView2 `PrintToPdfAsync` (or browser download fallback) that match physical print layouts.
- **AC-7**: Embed open-licensed Bengali typography (`Kalpurush` and `Noto Sans Bengali`) via CSS `@font-face` so that complex Bengali glyph shaping, vowel signs, and Bengali numbers (০-৯) render consistently across all Windows installations.
- **AC-8**: Decouple printing from database mutations: a printer failure, offline status, or spooler exception must never rollback a posted payment. The UI displays an explicit warning with instant `[Reprint]` and `[Save PDF]` fallback actions.

## Decision

**Chosen option**: Option 1: React Print Components with Print CSS (`@media print`, `@page`), Edge WebView2 Silent Print Pipeline, Per-PC Local Configuration, and Bundled Web Fonts.

You build the Coachify print engine across `ui/src/components/print/` (templates for 80 mm thermal, A4 dual-copy, and A5, plus print preview dialog), `ui/src/lib/printer.ts` (printer settings management and print trigger orchestration), `crates/core/src/storage.rs` / `printers.rs` (workstation printer settings repository), and the Tauri native bridge in `src-tauri/src/lib.rs`.

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model & DTOs

Printer settings are kept local to the PC:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkstationPrinterSettings {
    pub receipt_printer: Option<String>,
    pub report_printer: Option<String>,
    pub receipt_layout: String, // "thermal_80mm" | "a4_dual" | "a5_single"
    pub silent_receipt_printing: bool,
    pub auto_print_on_payment: bool,
    pub copies: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SystemPrinterInfo {
    pub name: String,
    pub is_default: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ReceiptAllocationItem {
    pub fee_head_name: String,
    pub fee_head_name_bn: Option<String>,
    pub period: Option<String>,
    pub amount_poisha: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrintableReceiptData {
    pub receipt_no: String,
    pub business_date: String,
    pub received_at: String,
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub student_name_bn: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub batch_name: Option<String>,
    pub program_name: Option<String>,
    pub roll_no: Option<String>,
    pub amount_poisha: i64,
    pub amount_in_words_en: String,
    pub amount_in_words_bn: String,
    pub method: String,
    pub reference: Option<String>,
    pub collector_name: String,
    pub institution_name: String,
    pub institution_name_bn: Option<String>,
    pub institution_address: Option<String>,
    pub institution_phone: Option<String>,
    pub balance_after_poisha: i64,
    pub items: Vec<ReceiptAllocationItem>,
    pub is_reprint: bool,
}
```

### Layout specifications

1. **80 mm Thermal (`thermal_80mm`)**:
   - Width: 80 mm (72 mm printable area), auto-height.
   - Typography: 11px body, bold headings, tabular numbers (`font-variant-numeric: tabular-nums`).
   - Content: Institution header, receipt number, date, student details, 2-column fee breakdown, total paid in BDT and Bengali words, remaining balance, collector signature line, QR code encoding receipt verification string.
2. **A4 Dual-Copy (`a4_dual`)**:
   - Dimensions: 210 mm × 297 mm, portrait.
   - Division: Two equal 140 mm high cards (Student Copy on top, Office Copy on bottom).
   - Perforation Divider: Dotted border with centered scissors icon (`✂ - - - - - - - - - - - -`).
3. **A5 Single-Copy (`a5_single`)**:
   - Dimensions: 148 mm × 210 mm, landscape or portrait.
   - Clean tabular invoice style with formal signature blocks for Cashier and Guardian.

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `printers.list` | RPC | none | `printers: Vec<SystemPrinterInfo>` | `settings.view` | `enumeration_failed` |
| `printers.get_settings` | RPC | none | `settings: WorkstationPrinterSettings` | Authenticated | none |
| `printers.save_settings` | RPC | `settings: WorkstationPrinterSettings` | `settings: WorkstationPrinterSettings` | `settings.manage` | `invalid_layout` |
| `receipts.get_printable` | RPC | `payment_id: string`, `is_reprint?: boolean` | `receipt: PrintableReceiptData` | `ledger.view` | `payment_not_found` |

## Build plan

1. Implement workstation printer settings repository and system printer enumeration in `crates/core/src/storage.rs` and expose RPCs `printers.list`, `printers.get_settings`, `printers.save_settings`, satisfies **AC-2**, **AC-3**.
2. Implement printable receipt aggregation query `receipts.get_printable` in `crates/core/src/ledger.rs` with Bengali/English amount-in-words converters, satisfies **AC-1**, **AC-7**.
3. Create React receipt components in `ui/src/components/print/` for 80 mm thermal, A4 dual-copy, and A5 single-copy with embedded Bengali fonts, satisfies **AC-1**, **AC-7**.
4. Build `PrintPreviewDialog` with layout switching, zoom controls, manual print triggering, and PDF download fallback, satisfies **AC-5**, **AC-6**.
5. Implement silent print dispatch pipeline in Tauri shell with error handling so print errors never undo payments, satisfies **AC-4**, **AC-8**.

## Consequences

**Positive**:
- Edge WebView2 ensures flawless Bengali Unicode text shaping without missing glyph boxes.
- Per-workstation printer settings allow multi-counter flexibility on LAN without database coupling.
- Dual-copy A4 and 80 mm thermal support cover 100% of Bangladeshi coaching center paper choices.
- Decoupled transactions protect accounting records from hardware failures.

**Negative / tradeoffs**:
- Silent printing requires OS-level WebView2 integration; headless testing must use preview or mock dispatch.
- Bundled Bengali fonts add ~1.2 MB to application assets.
