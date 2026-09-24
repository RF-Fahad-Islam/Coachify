# Feature 50: Professional Printing System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand Coachify's printing engine into a unified, professional document generation system covering student ID cards (A4 multi-card sheet & CR80 plastic), exam admit cards with routines and QR codes, batch report cards, thermal 80mm due slips and cash close summaries, attendance sheets, and financial ledgers with centralized branding, Bangla typography, chunked batch rendering, and per-document configuration.

**Architecture:** A local-first print pipeline extending Edge WebView2 silent printing and browser printing. Per-document template defaults and print run audit trails are stored in SQLite via Migration 0019 (`print_template_configs` and `print_job_runs`). React print templates use `@media print` and `@page` rules with embedded Kalpurush/Noto Sans Bengali fonts. Heavy batch print jobs are processed in 50-item chunks with `document.fonts.ready` synchronization and vector silhouette fallbacks for missing assets.

**Tech Stack:** Rust (Tauri 2 native shell, `rusqlite`, `specta`, `serde`), TypeScript, React 19, Tailwind CSS v4, Lucide icons, Vitest.

---

## User Review Required

> [!IMPORTANT]
> - **Migration 0019 Additive Schema**: Migration 0019 introduces `print_template_configs` and `print_job_runs` tables with 11 standard seeds covering all institution document types (`receipt`, `due_statement`, `due_slip_thermal`, `attendance_sheet`, `id_card`, `admit_card`, `mark_sheet`, `tabulation_sheet`, `report_card`, `financial_ledger`, `cash_close_thermal`). Released migrations 0001-0018 remain completely untouched.
> - **Architect Decision Source**: Built in strict adherence to Spec 0035 (`0035-professional-printing-system`) authored during the architect pass in `develop_slice_8`.
> - **Hardware Decoupling Guarantee**: Any printer hardware stall, paper jam, or spooler failure is isolated from financial and academic database mutations; error states provide instant reprint and PDF download actions.

---

## Open Questions

None. The architecture review, scope requirements, and Spec 0035 fully specify the schema, layouts, DTOs, security controls, and batch rendering constraints.

---

## Proposed Changes

```
professional_printing_system/
├── migrations/
│   └── 0019_professional_printing.sql            # [NEW] Template configs & print job runs schema
├── crates/core/
│   ├── src/
│   │   ├── printers.rs                           # [NEW] Template configs, print job audit & bulk queries
│   │   ├── dispatcher.rs                         # [MODIFY] Register printing RPC endpoints
│   │   └── lib.rs                                # [MODIFY] Export printer types & modules
│   └── tests/
│       └── professional_printing_integration.rs  # [NEW] Rust integration test suite
├── ui/
│   └── src/
│       ├── components/
│       │   ├── print/
│       │   │   ├── id-card-template.tsx          # [NEW] CR80 and A4 8-card sheet templates
│       │   │   ├── admit-card-template.tsx        # [NEW] Single A5 and dual A4 admit card templates
│       │   │   ├── thermal-due-slip-template.tsx # [NEW] 80mm continuous due slip
│       │   │   ├── thermal-cash-close-template.tsx# [NEW] 80mm cash close summary
│       │   │   ├── financial-ledger-template.tsx # [NEW] A4 formal financial ledger
│       │   │   ├── attendance-sheet-printable.tsx# [NEW] A4 landscape attendance register
│       │   │   ├── document-print-preview-modal.tsx# [NEW] Universal preview modal with chunking
│       │   │   ├── print-preview-dialog.tsx      # [MODIFY] Backward-compatible wrapper
│       │   │   └── professional-printing.test.tsx# [NEW] Frontend print templates test suite
│       │   ├── settings/
│       │   │   └── printer-settings-card.tsx     # [MODIFY] Template configs and print history UI
│       │   ├── students/
│       │   │   └── student-directory-view.tsx    # [MODIFY] Add Print ID Cards action
│       │   ├── exams/
│       │   │   ├── exams-hub-view.tsx            # [MODIFY] Add Print Admit Cards action
│       │   │   └── result-publish-view.tsx       # [MODIFY] Add Batch Print Report Cards action
│       │   └── cashier/
│       │       └── dues-center-view.tsx          # [MODIFY] Add Batch Print Due Statements & Slips
│       └── lib/
│           └── printer.ts                        # [MODIFY] Batch chunking & asset preloading helpers
└── docs/
    ├── scope/
    │   ├── printing.md                           # [MODIFY] Mark Feature 50 milestones
    │   └── index.md                              # [MODIFY] Update status to in-progress
    └── specs/
        └── 0035-professional-printing-system/     # [NEW] Spec, rationale, and verify docs
            ├── index.md
            ├── rationale.md
            └── verify.md
```

---

## Detailed Bite-Sized Implementation Tasks

### Task 1: Spec and Scope Synchronization
**Files:**
- Create: `docs/specs/0035-professional-printing-system/index.md`
- Create: `docs/specs/0035-professional-printing-system/rationale.md`
- Create: `docs/specs/0035-professional-printing-system/verify.md`
- Modify: `docs/scope/printing.md`
- Modify: `docs/scope/index.md`

- **Step 1:** Copy and reconcile Spec 0035 documentation from `develop_slice_8` into `docs/specs/0035-professional-printing-system/`.
- **Step 2:** Update `docs/scope/printing.md` with Feature 50 checklist and `docs/scope/index.md` status to in-progress.
- **Step 3:** Commit docs: `git commit -m "docs: import spec 0035 professional printing system and update scope"`.

---

### Task 2: Migration 0019 for Print Template Configurations and Job Audit Logs
**Files:**
- Create: `migrations/0019_professional_printing.sql`
- Test: `crates/core/tests/migration_immutability.rs`

- **Step 1:** Write `migrations/0019_professional_printing.sql`:
  - `print_template_configs` table with check constraints and standard seed rows for all 11 document types (`receipt`, `due_statement`, `due_slip_thermal`, `attendance_sheet`, `id_card`, `admit_card`, `mark_sheet`, `tabulation_sheet`, `report_card`, `financial_ledger`, `cash_close_thermal`).
  - `print_job_runs` table with indexing on `(document_type, created_at DESC)`.
  - Schema version bump to 19.
- **Step 2:** Verify immutability check: `cargo run -p xtask check-migrations`.
- **Step 3:** Commit: `git commit -m "feat(core): add migration 0019 for print template configs and job audit log"`.

---

### Task 3: Core Domain Types and Query Logic in `crates/core/src/printers.rs`
**Files:**
- Create: `crates/core/src/printers.rs`
- Modify: `crates/core/src/lib.rs`

- **Step 1: Write the failing integration test** in `crates/core/tests/professional_printing_integration.rs`:
  - Tests `get_template_configs` returns default seeds.
  - Tests `update_template_config` updates settings and persists to database.
  - Tests `record_print_job` and `list_print_jobs`.
- **Step 2: Run test to verify it fails**: `cargo test -p coachify_core --test professional_printing_integration`.
- **Step 3: Implement domain logic** in `crates/core/src/printers.rs`:
  - Structs: `PrintTemplateConfig`, `PrintTemplateConfigInput`, `PrintJobRun`, `RecordPrintJobInput`, `PrintJobListResult`.
  - Functions: `get_template_configs`, `update_template_config`, `record_print_job`, `list_print_jobs`.
- **Step 4: Run test to verify it passes**: `cargo test -p coachify_core --test professional_printing_integration`.
- **Step 5: Commit**: `git commit -m "feat(core): implement print template config and print job audit services"`.

---

### Task 4: Bulk Printable Data Query Services
**Files:**
- Modify: `crates/core/src/printers.rs`
- Test: `crates/core/tests/professional_printing_integration.rs`

- **Step 1: Write failing tests** for:
  - `students.get_id_card_data` (resolves student code, Bangla/English name, blood group, emergency contact, primary batch, session, barcode payload, fallback photo vector).
  - `exams.get_admit_card_data` (resolves exam title, schedule list ordered by date/time, enrolled students with QR verification payload).
  - `exams.get_batch_report_cards` (batch query for student report cards).
  - `finances.get_batch_due_statements` (batch query for student due statements).
- **Step 2: Run test to verify failure**: `cargo test -p coachify_core --test professional_printing_integration`.
- **Step 3: Implement bulk queries** in `crates/core/src/printers.rs`:
  - `get_id_card_data(conn, student_ids)`
  - `get_admit_card_data(conn, exam_id, batch_id, student_ids)`
  - `get_batch_report_cards(conn, exam_id, student_ids)`
  - `get_batch_due_statements(conn, student_ids)`
- **Step 4: Run test to verify it passes**: `cargo test -p coachify_core --test professional_printing_integration`.
- **Step 5: Commit**: `git commit -m "feat(core): add bulk query services for ID cards, admit cards, report cards, and dues"`.

---

### Task 5: Dispatcher RPC Registrations & Contract Freshness
**Files:**
- Modify: `crates/core/src/dispatcher.rs`
- Modify: `crates/core/src/lib.rs`

- **Step 1: Register commands in dispatcher**:
  - `printers.get_template_configs` (Authenticated, Read)
  - `printers.update_template_config` (Permission `settings.manage`, Mutation)
  - `printers.record_print_job` (Authenticated, Mutation)
  - `printers.list_print_jobs` (Authenticated, Read)
  - `students.get_id_card_data` (Permission `students.view`, Read)
  - `exams.get_admit_card_data` (Permission `exams.view`, Read)
  - `exams.get_batch_report_cards` (Permission `exams.view`, Read)
  - `finances.get_batch_due_statements` (Permission `ledger.view`, Read)
- **Step 2: Export TypeScript types**: `cargo run -p xtask export-types`.
- **Step 3: Verify contract freshness**: `cargo run -p xtask check-types`.
- **Step 4: Commit**: `git commit -m "feat(core): register printing RPC endpoints and export TypeScript contract"`.

---

### Task 6: Student ID Card & Exam Admit Card React Print Templates
**Files:**
- Create: `ui/src/components/print/id-card-template.tsx`
- Create: `ui/src/components/print/admit-card-template.tsx`
- Create: `ui/src/components/print/professional-printing.test.tsx`

- **Step 1: Write failing UI test** for ID card rendering:
  - Tests Single CR80 Card layout (`85.6mm × 53.98mm`).
  - Tests A4 8-card sheet layout (`2 × 4` grid with crop marks).
  - Tests missing photo fallback renders clean vector avatar without layout break (AC-8).
  - Tests barcode SVG representation and Bangla numerals (AC-9).
  - Tests Admit card single A5 and multi-item A4 format with exam routines and QR payload (AC-2).
- **Step 2: Run test to verify it fails**: `pnpm test`.
- **Step 3: Implement `id-card-template.tsx` and `admit-card-template.tsx`**:
  - Full print styling with `@media print`, `@page`, `break-inside: avoid`.
  - Embedded Bengali font classes (`Kalpurush`, `Noto Sans Bengali`).
  - Crop marks and cutting guides for counter staff.
- **Step 4: Run test to verify it passes**: `pnpm test`.
- **Step 5: Commit**: `git commit -m "feat(ui): implement student ID card and exam admit card print templates"`.

---

### Task 7: Thermal Due Slips, Thermal Cash Close & Ledger Templates
**Files:**
- Create: `ui/src/components/print/thermal-due-slip-template.tsx`
- Create: `ui/src/components/print/thermal-cash-close-template.tsx`
- Create: `ui/src/components/print/financial-ledger-template.tsx`
- Create: `ui/src/components/print/attendance-sheet-printable.tsx`
- Modify: `ui/src/components/print/professional-printing.test.tsx`

- **Step 1: Write failing UI test** for thermal roll templates and formal ledger.
- **Step 2: Run test to verify it fails**: `pnpm test`.
- **Step 3: Implement templates**:
  - `thermal-due-slip-template.tsx`: 80mm roll layout with student dues breakdown, Bengali amounts in words, barcode.
  - `thermal-cash-close-template.tsx`: 80mm shift summary with collections, payments, and net balance.
  - `financial-ledger-template.tsx`: Formal A4 ledger report with institution header, debit/credit rows, net totals, and authorization lines.
  - `attendance-sheet-printable.tsx`: A4 landscape attendance register with date columns and student list.
- **Step 4: Run test to verify it passes**: `pnpm test`.
- **Step 5: Commit**: `git commit -m "feat(ui): implement thermal due slip, cash close summary, and printable ledger templates"`.

---

### Task 8: Universal `DocumentPrintPreviewModal` with Chunked Batch Rendering
**Files:**
- Create: `ui/src/components/print/document-print-preview-modal.tsx`
- Modify: `ui/src/components/print/print-preview-dialog.tsx`
- Modify: `ui/src/lib/printer.ts`
- Modify: `ui/src/components/print/professional-printing.test.tsx`

- **Step 1: Write failing UI test** for `DocumentPrintPreviewModal` and batch chunking:
  - Tests rendering single and multi-page documents.
  - Tests 50-item chunking with progress bar state.
  - Tests zoom controls (60% to 140%), orientation toggling, and layout switching.
  - Tests calling `printers.record_print_job` on print confirmation.
- **Step 2: Run test to verify it fails**: `pnpm test`.
- **Step 3: Implement `DocumentPrintPreviewModal`**:
  - Multi-item slider / navigation bar ("Item 1 of 65").
  - Chunked DOM rendering: renders batches in chunks of 50 to avoid browser memory freeze.
  - Preload assets: `await document.fonts.ready` and image decoding before spooling.
  - Toolbar for zoom, paper size, orientation, header/logo toggles, and direct print / PDF actions.
- **Step 4: Run test to verify it passes**: `pnpm test`.
- **Step 5: Commit**: `git commit -m "feat(ui): implement universal DocumentPrintPreviewModal with batch chunking engine"`.

---

### Task 9: Settings UI for Template Configurations & Print History
**Files:**
- Modify: `ui/src/components/settings/printer-settings-card.tsx`
- Modify: `ui/src/components/settings/printer-settings-card.test.tsx`

- **Step 1: Write test** for print template configuration editor and print history log.
- **Step 2: Run test to verify it fails**: `pnpm test`.
- **Step 3: Implement settings UI**:
  - Template config tab: select document type, customize default paper size, orientation, show header, show logo, watermark, and signature title.
  - Print run history table: displays past print jobs with document type, item count, printer name, timestamp, and user.
- **Step 4: Run test to verify it passes**: `pnpm test`.
- **Step 5: Commit**: `git commit -m "feat(ui): add document template configuration and print history to settings"`.

---

### Task 10: View Integrations Across Academic & Financial Modules
**Files:**
- Modify: `ui/src/components/students/student-directory-view.tsx` (Add "Print ID Cards" button with single and multi-select support)
- Modify: `ui/src/components/exams/exams-hub-view.tsx` (Add "Print Admit Cards" action)
- Modify: `ui/src/components/exams/result-publish-view.tsx` (Add "Batch Print Report Cards" action)
- Modify: `ui/src/components/cashier/dues-center-view.tsx` (Add "Batch Print Due Statements" and "Print 80mm Due Slips" actions)
- Modify: `ui/src/components/attendance/attendance-view.tsx` (Add "Print Attendance Register" action)

- **Step 1: Integrate `DocumentPrintPreviewModal`** into all view action menus and toolbars.
- **Step 2: Run tests**: `pnpm test`.
- **Step 3: Commit**: `git commit -m "feat(ui): integrate professional printing modal across student, exam, cashier, and attendance views"`.

---

### Task 11: End-to-End Verification & Scope Closure
**Files:**
- Modify: `docs/scope/printing.md`
- Modify: `docs/scope/index.md`

- **Step 1: Run complete automated test suite**:
  - `cargo test --workspace`
  - `cargo run -p xtask check-types`
  - `cargo run -p xtask check-migrations`
  - `pnpm test`
  - `pnpm lint`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- **Step 2: Update scope document checkboxes** in `docs/scope/printing.md`.
- **Step 3: Commit**: `git commit -m "feat: complete professional printing system (feature 50, spec 0035)"`.

---

## Verification Plan

### Automated Tests
1. **Rust integration test**:
   ```bash
   cargo test -p coachify_core --test professional_printing_integration
   ```
2. **Workspace Rust test suite**:
   ```bash
   cargo test --workspace
   ```
3. **Contract and Migration Checks**:
   ```bash
   cargo run -p xtask check-types
   cargo run -p xtask check-migrations
   ```
4. **UI test suite**:
   ```bash
   pnpm test
   ```
5. **Linting and formatting**:
   ```bash
   pnpm lint
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all -- --check
   ```

### Manual Verification
1. Start the desktop application with `pnpm dev`.
2. Navigate to **Students > Directory**, select students, and click **Print ID Cards**. Verify both **A4 Sheet (8 cards with cut marks)** and **Single CR80 Card** previews with fallback avatar for students without photos.
3. Navigate to **Exams**, select an exam, and click **Print Admit Cards**. Verify student routines, date/time table, and QR code.
4. In **Cashier > Dues Center**, select a batch and click **Batch Print Due Statements** (A4) and **Print 80mm Due Slips** (Thermal roll).
5. In **Settings > Printer Settings**, open **Print Template Settings**, change a signature title or toggle header, and confirm it persists and updates previews.
6. Verify batch printing progress bar on 50+ items.
