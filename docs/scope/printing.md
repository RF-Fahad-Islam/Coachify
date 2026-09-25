# Printing & localization (Slice 9)

Back to [index](index.md). Professional output and language support for Bangladesh coaching centers.

### 50. Professional printing system
Expand the print engine (feature 15) to cover all document types: A4 reports, thermal receipts, due statements, attendance sheets, ID cards, admit cards, marksheets, report cards, and financial ledgers. Consistent branding, Bangla typography, and per document layout settings.
**Done when:** every document type prints with correct Bangla text and institution branding; thermal and A4 layouts are selectable; batch printing works for due statements and report cards; print preview is available for every document type.
spec [0035](../specs/0035-professional-printing-system/index.md) · code in `crates/core/src/print.rs`, `ui/src/components/print/`
- [x] Design it (spec): `/architect professional printing system`
- [x] Build it: `/develop professional printing system`
  - [x] Migration 0019 adding print_template_configs and print_job_runs tables with standard seeds, covers AC-5, AC-7
  - [x] Core RPC handlers for template configuration and print job audit logging, covers AC-5, AC-6, AC-7
  - [x] Bulk data query endpoints for ID cards, admit cards, report cards, and due statements, covers AC-1, AC-2, AC-3
  - [x] React print templates for student ID cards, exam admit cards, and thermal slips with photo fallback, covers AC-1, AC-2, AC-4, AC-8, AC-9
  - [x] Universal DocumentPrintPreviewModal with batch chunking progress and orientation controls, covers AC-3, AC-10
- [x] Verify it: `/check verify professional printing system`
- [x] Test it: `/test professional printing system`

### 51. Bangla + English localization
Full UI translation layer so every screen, label, error message, validation message, and system notification is available in both Bangla and English. The translation layer is already built (feature 4); this feature adds the actual Bangla translations and language switching.
**Done when:** switching to Bangla translates every visible UI string; no English leaks through in Bangla mode; date formats, number formats, and currency symbols follow Bangladeshi conventions; new features automatically use the translation layer.
spec [0038](../specs/0038-bangla-and-english-localization/index.md) · code in `ui/src/lib/i18n.ts`, `ui/src/components/layout/top-bar.tsx`, `ui/src/components/layout/sidebar.tsx`
- [x] Design it (spec): `/architect bangla + english localization`
- [x] Build it: `/develop bangla + english localization`
  - [x] Complete `en` and `bn` translation dictionaries with 100% key parity and reactive `useLocale()` hook (`coachify.locale`), covers AC-1, AC-2, AC-6
  - [x] Bangladeshi Lakh/Crore number grouping, Taka currency (`৳`), Bengali numerals (`০-৯`), and Bangla date formatter, covers AC-3, AC-4
  - [x] Core error code translation helper (`translateError`) with parameter interpolation and TopBar 1-click `EN | বাংলা` switcher, covers AC-1, AC-5
- [x] Verify it: `/check verify bangla + english localization`
- [x] Test it: `/test bangla + english localization`

### 52. CSV/Excel export
Export appropriate records and reports as CSV or Excel files. Export buttons on student lists, attendance reports, fee reports, dues lists, and exam results.
**Done when:** every major list view has an export button; exported files open correctly in Excel with Bangla text preserved; exports respect the current filter state; large exports (10,000+ rows) complete without freezing the UI.
spec [0039](../specs/0039-csv-excel-export/index.md) · code in `crates/core/src/reports.rs`, `ui/src/lib/export-csv.ts`
- [x] Design it (spec): `/architect csv/excel export`
- [x] Build it: `/develop csv/excel export`
  - [x] Extend `reports::export_csv` with `students_list`, `dues_list`, and `exam_results_list` handlers alongside existing financial, attendance, and academic exports, covers AC-1, AC-3, AC-6
  - [x] Prepend UTF-8 BOM (`\uFEFF`) and apply RFC 4180 + spreadsheet formula-injection neutralization (`'=` prefixing) in `escape_csv_cell`, covers AC-2, AC-4, AC-5
  - [x] Wire reusable `exportCsvFromCore` trigger buttons on Student Directory, Dues Center, Exam Results & Grading, and Reports Center, covers AC-1, AC-3
- [x] Verify it: `/check verify csv/excel export`
- [x] Test it: `/test csv/excel export`

### 53. Encrypted sensitive secrets · GA
SMS, email, API credentials, and provider configuration protected through the native OS layer (DPAPI on Windows). Secrets are never stored as plaintext in the database or config files, never appear in logs or error messages, and never leave the machine through AI or diagnostics.
**Done when:** all provider credentials are encrypted at rest using DPAPI; a database copy on another machine cannot read the secrets; the diagnostics export bundle contains no secrets; log redaction covers all secret patterns.
spec [0040](../specs/0040-encrypted-sensitive-secrets/index.md) · code in `crates/core/src/crypto.rs`, `crates/core/src/comms.rs`, `crates/core/src/diagnostics.rs`, `crates/core/src/ai/privacy.rs`
- [x] Design it (spec): `/architect encrypted sensitive secrets`
- [x] Build it: `/develop encrypted sensitive secrets`
  - [x] Centralized `crypto::protect_secret_dpapi` and `unprotect_secret_dpapi` with `DPAPI_V1:` envelope and `get_secrets_health` status RPC (`security.get_secrets_health`), covers AC-1, AC-2, AC-3
  - [x] Automatic credential stripping in `comms::save_provider_config` (`sanitize_provider_config_json`) encrypting sensitive JSON keys into `secret_ref` and replacing plaintext with `[REDACTED_TO_DPAPI]`, covers AC-1, AC-2
  - [x] Universal secret scrubber (`diagnostics::redact_secrets` and `redact_json_value`) across diagnostics bundles, logs, and AI prompts (`ai::privacy::scrub_text`), covers AC-4, AC-5, AC-6
- [x] Verify it: `/check verify encrypted sensitive secrets`
- [x] Test it: `/test encrypted sensitive secrets`
- [x] Review it: `/check review encrypted sensitive secrets`
- [x] Document it: `/document encrypted sensitive secrets`
