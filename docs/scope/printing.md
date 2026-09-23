# Printing & localization (Slice 9)

Back to [index](index.md). Professional output and language support for Bangladesh coaching centers.

### 50. Professional printing system
Expand the print engine (feature 15) to cover all document types: A4 reports, thermal receipts, due statements, attendance sheets, ID cards, admit cards, marksheets, report cards, and financial ledgers. Consistent branding, Bangla typography, and per document layout settings.
**Done when:** every document type prints with correct Bangla text and institution branding; thermal and A4 layouts are selectable; batch printing works for due statements and report cards; print preview is available for every document type.
spec [0035](../specs/0035-professional-printing-system/index.md)
- [x] Design it (spec): `/architect professional printing system`
- [ ] Build it: `/develop professional printing system`
  - [ ] Migration 0019 adding print_template_configs and print_job_runs tables with standard seeds, covers AC-5, AC-7
  - [ ] Core RPC handlers for template configuration and print job audit logging, covers AC-5, AC-6, AC-7
  - [ ] Bulk data query endpoints for ID cards, admit cards, report cards, and due statements, covers AC-1, AC-2, AC-3
  - [ ] React print templates for student ID cards, exam admit cards, and thermal slips with photo fallback, covers AC-1, AC-2, AC-4, AC-8, AC-9
  - [ ] Universal DocumentPrintPreviewModal with batch chunking progress and orientation controls, covers AC-3, AC-10
- [ ] Verify it: `/check verify professional printing system`
- [ ] Test it: `/test professional printing system`

### 51. Bangla + English localization · needs a decision
Full UI translation layer so every screen, label, error message, validation message, and system notification is available in both Bangla and English. The translation layer is already built (feature 4); this feature adds the actual Bangla translations and language switching.
**Done when:** switching to Bangla translates every visible UI string; no English leaks through in Bangla mode; date formats, number formats, and currency symbols follow Bangladeshi conventions; new features automatically use the translation layer.
- [ ] Design it (spec): `/architect bangla + english localization`

### 52. CSV/Excel export · needs a decision
Export appropriate records and reports as CSV or Excel files. Export buttons on student lists, attendance reports, fee reports, dues lists, and exam results.
**Done when:** every major list view has an export button; exported files open correctly in Excel with Bangla text preserved; exports respect the current filter state; large exports (10,000+ rows) complete without freezing the UI.
- [ ] Design it (spec): `/architect csv/excel export`

### 53. Encrypted sensitive secrets · needs a decision · GA
SMS, email, API credentials, and provider configuration protected through the native OS layer (DPAPI on Windows). Secrets are never stored as plaintext in the database or config files, never appear in logs or error messages, and never leave the machine through AI or diagnostics.
**Done when:** all provider credentials are encrypted at rest using DPAPI; a database copy on another machine cannot read the secrets; the diagnostics export bundle contains no secrets; log redaction covers all secret patterns.
- [ ] Design it (spec): `/architect encrypted sensitive secrets`
