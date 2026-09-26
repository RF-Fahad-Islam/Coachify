# 0035. Professional Printing System

**Date**: 2026-09-23
**Status**: Accepted

## Summary

Coachify expands its printing engine to generate all institution documents with crisp Bengali typography and unified center branding. Staff can print standard A4 sheets, half page A5 sheets, narrow 80mm thermal rolls, and student ID cards from one shared system. Batch printing processes hundreds of admit cards, due notices, and report cards in chunks so the application stays fast and responsive. All document layouts work completely offline with instant print previews and direct PDF exports.

## Requirements

**User stories**:
* As a counter cashier or receptionist, I want to print student ID cards on standard A4 paper with cut guides or on single plastic cards, so that our center can issue identity badges easily.
* As an exam controller, I want to print exam admit cards in bulk for a whole batch with schedule details and student photos, so that students enter exam halls with official verification.
* As an academic coordinator, I want to print batch marksheets, tabulation sheets, and report cards with Bengali grading scales and teacher remarks, so that parents receive clear academic updates.
* As an accounts officer, I want to print batch due statements and quick thermal roll due slips for fee collection drives, so that counter collection runs smoothly.
* As an institution owner, I want consistent institution headers, logos, and authorized signatures across every document, so that all printed materials project a professional image.

**Acceptance criteria**:
* **AC-1**: Support student ID card printing in two selectable modes: an A4 multi card layout (8 cards per sheet arranged in 2 columns by 4 rows with crop marks) and a single CR80 plastic card layout (85.6 mm × 53.98 mm).
* **AC-2**: Support exam admit card printing with student photo, barcode or QR code, exam routine schedule table (sourced from `exam_subjects`), and center instructions in single A5 format and multi item A4 format.
* **AC-3**: Support batch printing across admit cards, report cards, and due statements using chunked background rendering (50 items per chunk) with a visual progress bar, image decode synchronization, and font readiness checks.
* **AC-4**: Provide thermal 80mm roll layouts for payment receipts, quick fee due reminders, and daily cash close summaries.
* **AC-5**: Store global and per document print configurations in SQLite table `print_template_configs`, persisting paper size, orientation, header visibility, logo visibility, watermark toggle, signature title, and custom notes with standard seeds.
* **AC-6**: Provide a centralized `printers.update_template_config` RPC command guarded by `settings.manage` permission (Administrator and Owner roles), while allowing all authenticated staff to apply temporary preview layout adjustments without altering saved defaults.
* **AC-7**: Provide a `print_job_runs` audit log table and RPC `printers.record_print_job` recording document type, batch mode, item count, printer name, timestamp, and session authenticated user ID.
* **AC-8**: Missing student photos gracefully fall back to a clean vector silhouette placeholder with the student code, preserving card geometry and grid alignment without layout shifts.
* **AC-9**: Embedded Bengali fonts (`Kalpurush` and `Noto Sans Bengali`) render complex conjunct ligatures, vowel signs, and Bengali numerals (০ to ৯) cleanly across all print outputs, waiting for `document.fonts.ready` before spooling.
* **AC-10**: Expand the universal in app `DocumentPrintPreviewModal` to support layout switching, orientation toggle, zoom controls, batch navigation sliders, and print triggers via the isolated print pipeline.

## Decision

**Chosen option**: Option 1: Unified Document Print Architecture with Centralized Branding and Chunked Batch Processing

We build a unified document print architecture that extends the existing print engine across all academic and financial document types. Layout templates are implemented as modular React components with dedicated print CSS rules (`@media print`, `@page`). Document rendering awaits font readiness and image decodes to prevent blank frames. Template styling rules and branding preferences persist in the local database with administrator access controls, while print execution remains completely offline and decoupled from data storage.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tailwind-4-docs` (`lombiq/tailwind-agent-skills`, `.claude/skills/tailwind-4-docs/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

**Data model sketch**:
Migration 0019 introduces dedicated printing tables and default templates:

```sql
-- Print template settings per document type
CREATE TABLE print_template_configs (
    id TEXT PRIMARY KEY,
    document_type TEXT NOT NULL UNIQUE CHECK (document_type IN (
        'receipt', 'due_statement', 'due_slip_thermal', 'attendance_sheet',
        'id_card', 'admit_card', 'mark_sheet', 'tabulation_sheet',
        'report_card', 'financial_ledger', 'cash_close_thermal'
    )),
    paper_size TEXT NOT NULL DEFAULT 'a4' CHECK (paper_size IN (
        'a4', 'a5', 'thermal_80mm', 'id_card_pvc'
    )),
    orientation TEXT NOT NULL DEFAULT 'portrait' CHECK (orientation IN (
        'portrait', 'landscape'
    )),
    show_header INTEGER NOT NULL DEFAULT 1 CHECK (show_header IN (0, 1)),
    show_logo INTEGER NOT NULL DEFAULT 1 CHECK (show_logo IN (0, 1)),
    show_watermark INTEGER NOT NULL DEFAULT 0 CHECK (show_watermark IN (0, 1)),
    custom_header_text TEXT,
    custom_footer_text TEXT,
    signature_title TEXT DEFAULT 'Authorized Signature',
    options_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL,
    updated_by_user_id TEXT REFERENCES users(id)
);

-- Print job run history
CREATE TABLE print_job_runs (
    id TEXT PRIMARY KEY,
    document_type TEXT NOT NULL,
    batch_mode INTEGER NOT NULL DEFAULT 0 CHECK (batch_mode IN (0, 1)),
    item_count INTEGER NOT NULL DEFAULT 1,
    printer_name TEXT,
    paper_size TEXT NOT NULL,
    created_at TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN (
        'completed', 'failed', 'cancelled'
    )),
    metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_print_job_runs_type_date ON print_job_runs(document_type, created_at DESC);
```

**Core Data Transfer Objects**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StudentIdCardData {
    pub student_id: String,
    pub student_code: String,
    pub full_name_en: String,
    pub full_name_bn: Option<String>,
    pub gender: String,
    pub blood_group: Option<String>,
    pub emergency_phone: Option<String>,
    pub primary_batch_name: Option<String>,
    pub academic_session: Option<String>,
    pub barcode_payload: String,
    pub photo_data_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExamSubjectScheduleItem {
    pub subject_id: String,
    pub subject_name: String,
    pub exam_date: String,
    pub start_time: String,
    pub room_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmitCardStudentItem {
    pub student_id: String,
    pub student_code: String,
    pub student_name: String,
    pub batch_name: String,
    pub roll_number: Option<String>,
    pub photo_data_uri: Option<String>,
    pub qr_verification_payload: String,
    pub subjects: Vec<ExamSubjectScheduleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdmitCardBatchData {
    pub exam_id: String,
    pub exam_title: String,
    pub academic_session: Option<String>,
    pub center_instructions: Vec<String>,
    pub students: Vec<AdmitCardStudentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PrintTemplateConfigInput {
    pub document_type: String,
    pub paper_size: String,
    pub orientation: String,
    pub show_header: bool,
    pub show_logo: bool,
    pub show_watermark: bool,
    pub custom_header_text: Option<String>,
    pub custom_footer_text: Option<String>,
    pub signature_title: Option<String>,
    pub options_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RecordPrintJobInput {
    pub document_type: String,
    pub batch_mode: bool,
    pub item_count: u32,
    pub printer_name: Option<String>,
    pub paper_size: String,
    pub status: String,
    pub metadata_json: Option<String>,
}
```

**State transitions**:
Batch Print Job State:
`idle` &rarr; `chunking` &rarr; `awaiting_assets` &rarr; `rendering` &rarr; `spooling` &rarr; `completed` (or `cancelled`, `failed`)

Print Template Update:
`active` &rarr; modified by Admin with `settings.manage` &rarr; `active` (with audit log record)

**API surface**:
| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `printers.get_template_configs` | Read | none | Vec<PrintTemplateConfig> | Authenticated | 500 query failed |
| `printers.update_template_config` | Mutation | config: PrintTemplateConfigInput | PrintTemplateConfig | `settings.manage` | 400 validation error, 403 forbidden |
| `printers.record_print_job` | Mutation | job: RecordPrintJobInput | PrintJobRun | Authenticated | 400 validation error |
| `printers.list_print_jobs` | Read | limit: Option<u32>, offset: Option<u32>, document_type: Option<String> | PrintJobListResult | Authenticated | 500 query failed |
| `students.get_id_card_data` | Read | student_ids: Vec<String> | Vec<StudentIdCardData> | `students.view` | 404 student not found |
| `exams.get_admit_card_data` | Read | exam_id: String, batch_id: Option<String>, student_ids: Option<Vec<String>> | AdmitCardBatchData | `exams.view` | 404 exam not found |
| `exams.get_batch_report_cards` | Read | exam_id: String, student_ids: Vec<String> | Vec<StudentReportCardData> | `exams.view` | 404 exam not found |
| `finances.get_batch_due_statements` | Read | student_ids: Vec<String> | Vec<DueStatementData> | `payments.view` | 500 query failed |

**Value sourcing**:
| Action | Value produced / displayed | Source |
|---|---|---|
| Print ID cards | Student name, code, gender, blood group | Table `students` |
| Print ID cards | Primary batch and academic session | Active enrollment in `enrollments` joined with `batches` and `academic_sessions` |
| Print ID cards | Emergency contact phone | `students.emergency_phone_norm`, falling back to primary `guardians.phone_norm` |
| Print ID cards | Student photograph | Vault file referenced by `students.photo_vault_id` encoded as data URI |
| Print ID cards | Barcode payload | `student_code` formatted for standard 1D or 2D barcode scanner input |
| Print admit cards | Exam title, academic session | Table `exams` |
| Print admit cards | Subject routines, dates, times, rooms | Table `exam_subjects` ordered by `exam_date` and `start_time` ascending |
| Print admit cards | Center instructions | `options_json` in `print_template_configs` for `admit_card` |
| Print admit cards | Verification QR payload | Format `ADM\|<exam_id>\|<student_code>\|<hmac_hash>` |
| Thermal due slip | Student code, total due, batch name | Computed balance from financial ledger matching current business date |
| Thermal cash close | Total income, expenses, net cash, cashier | Aggregated payments and expenses for current open cash day |
| Document headers | Institute name, Bengali name, logo, address, phone | Table `institution` (`id = 1`) plus template header overrides |
| Footer signature | Signatory designation and signature line | `signature_title` column in `print_template_configs` |

**Key invariants**:
* Document rendering operates completely offline without calling external font or image services.
* Missing student photographs never abort print execution or cause grid misalignment.
* Large batch jobs render in chunks of 50 records with asset preloading (`document.fonts.ready` and image decoding) to prevent blank frames.
* Physical printer hardware errors never rollback or invalidate persisted academic or financial records.
* Print template configuration changes require `settings.manage` permission and write audit log entries.
* Transient preview settings in the UI modal do not mutate saved database templates.

**Security model**:
* All authenticated roles (Receptionist, Cashier, Teacher, Admin, Owner) can view and print documents.
* Only roles with `settings.manage` permission can modify print templates, header logos, and signature titles.
* Print job execution records the operator user ID derived from the authenticated session context.

**Configuration required**:
No new environment variables required. Configurations are stored in SQLite table `print_template_configs` and local `%ProgramData%\Coachify\printer_settings.json`.

**Critical test scenarios**:
* ID card A4 multi card layout: Generates grid of 8 student cards on A4 sheet with crop marks, verifying AC-1.
* ID card single PVC card layout: Generates isolated CR80 card dimension (85.6 mm × 53.98 mm), verifying AC-1.
* Admit card generation: Generates exam admit cards with subject routines from `exam_subjects` and student QR codes, verifying AC-2.
* Batch print chunking: Queues 150 student report cards, verifies chunked rendering and progress reporting, verifying AC-3.
* Thermal due slip: Renders 80mm continuous roll slip with barcode and Bangla currency formatting, verifying AC-4.
* Thermal cash close summary: Renders 80mm compact cash close summary, verifying AC-4.
* Template settings update: Updates header visibility and signature title via RPC, confirms persistence, verifying AC-5 and AC-6.
* Print audit logging: Records print job run in `print_job_runs` table, confirms user attribution, verifying AC-7.
* Missing photo fallback: Renders ID card for student without photo, confirms vector avatar and intact layout, verifying AC-8.
* Bengali ligature rendering: Tests complex Bengali text string rendering in print layout, verifying AC-9.
* Universal preview modal: Opens preview modal, switches zoom, toggles orientation, and triggers export, verifying AC-10.

## Build plan

Following the Tracer Bullet build approach, Feature 50 progresses in five vertical end to end layers:

1. Create Migration 0019 adding `print_template_configs` and `print_job_runs` tables with default configurations and standard seeds, satisfies **AC-5**, **AC-7**.
2. Implement core RPC handlers `printers.get_template_configs`, `printers.update_template_config`, `printers.record_print_job`, and `printers.list_print_jobs` in `crates/core/src/printers.rs`, satisfies **AC-5**, **AC-6**, **AC-7**.
3. Implement bulk data query endpoints `students.get_id_card_data`, `exams.get_admit_card_data`, `exams.get_batch_report_cards`, and `finances.get_batch_due_statements`, satisfies **AC-1**, **AC-2**, **AC-3**.
4. Build reusable React print templates for student ID cards (A4 8 card sheet and single PVC) and exam admit cards with photo fallback, satisfies **AC-1**, **AC-2**, **AC-8**, **AC-9**.
5. Build thermal 80mm due slip and cash close summary templates, batch print pagination chunker with asset preloading, and universal `DocumentPrintPreviewModal`, satisfies **AC-3**, **AC-4**, **AC-10**.
6. Implement comprehensive integration tests in `crates/core/tests/professional_printing_integration.rs` and UI component tests, satisfies **AC-1** through **AC-10**.

## Consequences

**Positive**:
* Coaching centers can produce professional ID cards, admit cards, and report cards in house without external graphic designers.
* Batch printing runs smoothly without freezing the counter workstation during busy exam or admission seasons.
* Branding is consistent and customizable across all formal documents.
* Offline support guarantees that print tasks work regardless of internet availability.

**Negative / tradeoffs**:
* Rendering hundreds of complex cards with QR codes and photos requires careful DOM recycling to prevent client memory bloat.
* Plastic card printing requires counter workstations to have properly calibrated card printer drivers.

**Neutral**:
* Migration 0019 introduces two new tables to manage document settings and print logs.
* Template customizations are managed per document type rather than with raw visual code editors.

## Follow-up

* [ ] Coordinate with front desk staff during pilot testing to verify barcode scanning contrast on student ID cards.
* [ ] Verify silent printing behavior on popular dual side plastic card printer drivers.

## Migration plan

**Strategy**: no migration needed (additive schema enhancement via Migration 0019).
**Phases**:
1. Run Migration 0019 to establish `print_template_configs` and `print_job_runs` with standard defaults.
2. Deploy expanded print engine components and RPC methods.
**Rollback**: Drop tables `print_job_runs` and `print_template_configs` if rollback is required.
**Risks**: Low risk since existing receipt printing paths continue using established printer settings.
