# 0010. Excel and CSV student import engine

**Date**: 2026-09-22
**Status**: Accepted

## Summary

Coachify provides a guided bulk import engine that allows coaching center staff to import student and guardian records from external spreadsheets (modern Excel `.xlsx`, older Excel `.xls`, and comma separated `.csv` files). Center administrators often receive messy student rosters from entrance tests, school lists, or legacy management files during busy admission seasons. This engine guides staff through selecting sheets, mapping spreadsheet columns to Coachify fields, previewing validation warnings, taking an automatic database safety snapshot, and committing records in a single database transaction. Any rejected rows can be downloaded as a ready to edit file with plain explanation columns, ensuring zero data loss and clean records.

## Requirements

**User stories**:
- As a coaching center receptionist or cashier, I want to upload an Excel or CSV file of student admissions and preview how columns match our system, so that I do not have to type hundreds of student entries by hand.
- As an office administrator, I want intelligent auto mapping that recognizes English and Bengali column headers, so that setting up the import takes only a few seconds.
- As an intake coordinator, I want to see a clear preview of valid rows, warnings, and errors before anything touches the database, so that typos or missing required fields are caught early.
- As an intake coordinator, I want the option to import valid rows immediately while downloading failed rows as a new spreadsheet with reasons attached, so that I can fix mistakes without delaying valid admissions.
- As an intake coordinator, I want sibling students sharing the same guardian phone number in the spreadsheet to link automatically to a single shared guardian, so that family contact information stays unified.
- As an institute owner, I want an automatic database safety snapshot taken right before bulk import execution and a single transaction commit, so that an accidental failure or power cut never leaves half imported or corrupt data.
- As an academic coordinator, I want the option to enroll imported students into an academic batch in the same step with sequential roll numbers, so that class rosters are formed immediately upon import.

**Acceptance criteria**:

- **AC-1 (File Format Inspection & Sheet Selection)**:
  - Core inspects uploaded spreadsheet files in `.xlsx`, `.xls`, and `.csv` formats.
  - Multi sheet workbooks return a list of sheet names with row counts, allowing the operator to select the target sheet.
  - The inspector detects the header row, skips leading empty rows, and trims extraneous whitespace from header titles.
  - Memory consumption stays under 64MB even when inspecting workbooks with 10,000 rows.

- **AC-2 (Intelligent Bilingual Column Auto Mapping)**:
  - The engine matches spreadsheet header titles against Coachify student and guardian fields using case insensitive aliases and fuzzy scoring in both English and Bengali.
  - Recognizes standard aliases for Student Name (`name`, `student_name`, `নাম`, `ছাত্র/ছাত্রীর নাম`), Date of Birth (`dob`, `birth_date`, `জন্ম তারিখ`), Gender (`gender`, `sex`, `লিঙ্গ`), Student Phone (`phone`, `mobile`, `মোবাইল`), Present Address (`address`, `present_address`, `ঠিকানা`), Guardian Name (`father_name`, `mother_name`, `guardian_name`, `পিতা`, `মাতা`, `অভিভাবক`), Guardian Phone (`guardian_phone`, `parent_phone`, `অভিভাবকের মোবাইল`), and Class or Group.
  - Auto mapped columns display confidence scores in the user interface with manual override pickers for every field.

- **AC-3 (Bilingual Value Parsing & Normalization)**:
  - Bengali numerals (`০, ১, ২, ৩, ৪, ৫, ৬, ৭, ৮, ৯`) in phones, dates, and rolls are normalized into standard ASCII digits (`0-9`).
  - Dates parse common formats (`YYYY-MM-DD`, `DD/MM/YYYY`, `DD-MM-YYYY`, `MM/DD/YYYY`, and Excel numeric date serials).
  - Gender values parse standard variations (`male`, `female`, `other`, `m`, `f`, `ছেলে`, `মেয়ে`, `পুরুষ`, `মহিলা`).
  - Required fields (`name`, `gender`, `dob`, `present_address`) are validated; missing required fields flag the row as an error.

- **AC-4 (Phone Normalization, Conflict Detection & Sibling Grouping)**:
  - Phone numbers normalize to the standard 13 digit Bangladeshi format (`8801XXXXXXXXX`) via `crate::phone::normalize_phone`, while preserving valid landlines and foreign numbers.
  - Rows with identical normalized guardian phone numbers within the same file are detected as siblings and linked to a single shared guardian record, satisfying the shared family invariant from spec 0008.
  - When an imported phone matches a student or guardian already existing in the SQLite database, the row is flagged with a warning and respects the selected conflict policy (`link_existing_guardian`, `allow_duplicate_phone`, or `skip_row`).

- **AC-5 (In Memory Validation Matrix & Issue Categorization)**:
  - The engine validates all rows in memory without persisting changes to SQLite.
  - Rows are categorized into three distinct states:
    1. `Valid`: All required fields valid, no conflicts.
    2. `Warning`: Recoverable issues (for example phone conflict with existing record, guessed date format, auto generated student code needed, missing optional guardian income).
    3. `Error`: Fatal invalid data (for example missing student name, invalid gender string, unparseable date, phone with invalid digit count).
  - Returns total row count, valid count, warning count, error count, and a preview slice of rows with cell level issue explanations.

- **AC-6 (Pre Import Database Safety Snapshot)**:
  - Immediately prior to executing the database insert, the core uses the SQLite online backup API to write a point in time snapshot of the active database to `<data_root>/<institution_id>/backups/pre-import-<timestamp>.db`.
  - The snapshot path is verified to exist on disk before the write transaction begins.
  - If taking the safety snapshot fails (for example due to full disk space), the import halts immediately without modifying the database.

- **AC-7 (Atomic Database Transaction & Rollback Guarantee)**:
  - All student records, guardian records, junction links, sequence increments, and audit log entries are executed inside a single SQLite `IMMEDIATE` transaction.
  - If a fatal database constraint error occurs during execution, the transaction rolls back completely, guaranteeing that zero partial rows remain.
  - Student codes allocate sequentially using the institutional academic year sequence (`S2026-0001`), with support for manual code columns when specified and unique.

- **AC-8 (Selective Import Mode & Rejected Rows CSV Export)**:
  - The operator can choose between **Strict Rollback** (abort entire import if any row has an error) and **Import Valid Rows Only**.
  - Under Import Valid Rows Only, valid rows and approved warning rows commit to the database, while error rows are excluded from the transaction.
  - The engine generates a clean CSV file containing the exact original columns of the rejected rows plus an added `_import_error` column explaining the failure reason for each row.
  - The rejected file is saved to the local downloads folder or returned via RPC for immediate operator download.

- **AC-9 (Optional Academic Batch Enrollment)**:
  - The import command accepts an optional `target_batch_id`.
  - When provided, every successfully imported student is enrolled into that batch in the same transaction, allocating sequential roll numbers (`max(roll) + 1`) within the batch.
  - If the batch reaches capacity, enrollment respects the `allow_overcapacity` flag or logs a warning per spec 0009.

- **AC-10 (Typed RPC API & Desktop Import Wizard UX)**:
  - Core exposes typed RPC commands (`import.inspect_file`, `import.preview_validation`, `import.execute`) gated by `students.manage` permission.
  - The user interface provides a step by step modal wizard in the Student Directory with:
    1. File selection dropzone with file size and sheet selector.
    2. Visual column mapper with auto match tags, sample data previews, and confidence indicators.
    3. Validation preview table with filter tabs (`All`, `Valid`, `Warnings`, `Errors`) and stat summary cards.
    4. Execution screen with safety backup notification, progress indicators, and a one click "Download Failed Rows (CSV)" button.

## Decision

**Chosen option**: Option 1: Pure Rust In Process Parsing (`calamine` and `csv`), In Memory Validation Matrix, Automated SQLite Online Backup Snapshot, Atomic Transaction Commit, and Rejected Rows CSV Export.

Implementation spans:
- `crates/core/src/import.rs`: File inspection, column mapping heuristics, value parsing, validation matrix, pre import backup helper, atomic bulk inserter, and failed rows CSV generator.
- `crates/core/src/lib.rs` and `crates/core/src/dispatcher.rs`: RPC contracts, Specta exports, and `students.manage` permission gating.
- `ui/src/components/students/import-students-dialog.tsx`: Step by step wizard dialog using shadcn primitives and Tailwind CSS.
- `ui/src/components/students/students-page.tsx`: Import action trigger button in the main student directory toolbar.

**Implementation skills**: `rust-best-practices` (`apollographql/skills`, `.claude/skills/rust-best-practices/`) · `tauri-v2` (`nodnarbnitram/claude-code-extensions`, `.claude/skills/tauri-v2/`) · `react-patterns` (`.claude/skills/react-patterns/`) · `shadcn` (`shadcn/ui`, `.claude/skills/shadcn/`) · `tanstack-query` (`tanstack-skills/tanstack-skills`, `.claude/skills/tanstack-query/`)

## Rationale

Reasoning and options: see [rationale.md](rationale.md).

## Feature design

### Data model sketch

The import engine uses domain data transfer objects (DTOs) in memory and persists records directly into the existing normalized schema (`students`, `guardians`, `student_guardians`, `enrollments`, `sequences`, `audit_log`). No new database migration or table is required.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SheetInspection {
    pub name: String,
    pub row_count: usize,
    pub sample_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FileInspectionResult {
    pub file_name: String,
    pub file_size_bytes: u64,
    pub format: String, // "xlsx", "xls", "csv"
    pub sheets: Vec<SheetInspection>,
    pub suggested_mappings: Vec<ColumnMappingSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ColumnMappingSuggestion {
    pub source_column: String,
    pub target_field: Option<String>,
    pub confidence: f32, // 0.0 to 1.0
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ColumnMappingInput {
    pub source_column: String,
    pub target_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum RowIssueSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RowIssue {
    pub column: Option<String>,
    pub field: Option<String>,
    pub message: String,
    pub severity: RowIssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportPreviewRow {
    pub row_index: usize,
    pub raw_data: HashMap<String, String>,
    pub parsed_name: Option<String>,
    pub parsed_phone: Option<String>,
    pub parsed_guardian_name: Option<String>,
    pub parsed_guardian_phone: Option<String>,
    pub issues: Vec<RowIssue>,
    pub is_valid: bool,
    pub has_warnings: bool,
    pub has_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ValidationPreviewResult {
    pub total_rows: usize,
    pub valid_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub rows_sample: Vec<ImportPreviewRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExecuteImportInput {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub mappings: Vec<ColumnMappingInput>,
    pub skip_error_rows: bool, // true = import valid & export failed; false = strict all or nothing
    pub phone_conflict_mode: String, // "link_guardian", "allow_duplicate", "skip_row"
    pub target_batch_id: Option<String>,
    pub default_present_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportSummaryResult {
    pub total_rows: usize,
    pub imported_students_count: usize,
    pub imported_guardians_count: usize,
    pub skipped_errors_count: usize,
    pub backup_path: String,
    pub rejected_rows_csv_path: Option<String>,
}
```

### State transitions

The import workflow follows a strictly gated state machine:

1. `Unloaded`: No file selected. Operator drops or selects a spreadsheet file.
2. `Inspecting`: Core inspects format and reads headers; returns sheet choices and auto mapping suggestions.
3. `Mapping`: Operator reviews and adjusts column mappings.
4. `Validating`: Core performs in memory dry run validation; computes valid, warning, and error rows.
5. `Reviewed`: Operator reviews preview table, filters by issue status, and selects execution mode.
6. `Executing`: Core captures pre import database backup snapshot, starts immediate transaction, writes records, and commits.
7. `Completed`: Operator views total counts, has direct link to download rejected rows CSV, and views imported records in directory.

### API surface

| Command | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `import.inspect_file` | RPC | `file_path: String` | `FileInspectionResult` | `students.manage` | `FileNotFound`, `UnsupportedFormat`, `CorruptFile` |
| `import.preview_validation` | RPC | `file_path`, `sheet_name`, `mappings`, `conflict_mode` | `ValidationPreviewResult` | `students.manage` | `InvalidMappings`, `ParsingFailed` |
| `import.execute` | RPC | `ExecuteImportInput` | `ImportSummaryResult` | `students.manage` | `BackupFailed`, `TransactionAborted`, `AllRowsInvalid` |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| `import.inspect_file` | Sheet names and estimated row count | Read from workbook metadata via `calamine` |
| `import.inspect_file` | Column mapping suggestions and confidence | Fuzzy alias dictionary matching headers in English and Bengali |
| `import.preview_validation` | Normalized phone numbers | Derived via `crate::phone::normalize_phone` from mapped phone column |
| `import.preview_validation` | Phone conflict warnings | Looked up from active `students` and `guardians` tables via SQLite query |
| `import.preview_validation` | Sibling detection flags | Grouped by identical normalized guardian phone within the current sheet |
| `import.execute` | Pre import database backup file | Created via SQLite online backup API to `<institution_id>/backups/` |
| `import.execute` | Sequential student code | Generated from `sequences` table via `crate::db::allocate_sequence` |
| `import.execute` | Batch roll number | Calculated as `coalesce(max(roll), 0) + 1` within the target batch |
| `import.execute` | Audit log record | Cryptographic SHA256 audit entry logged via `crate::audit::log_event` |
| `import.execute` | Rejected rows CSV file | Written to local disk containing un imported rows plus `_import_error` column |

### Key invariants

- **Safety Backup Prior to Mutation**: No database row can be inserted or modified until the pre import backup snapshot file is fully written and verified on disk.
- **Transaction Atomicity**: All insertions for an import run happen within a single SQLite transaction. If strict mode is chosen and one error occurs, or if a database crash happens, zero rows persist.
- **Shared Guardian Integrity**: Sibling rows sharing a normalized guardian phone number within the same import file must resolve to a single primary guardian entity.
- **Phone Number Consistency**: Every phone number must be normalized to standard 13 digit Bangladeshi format (`8801XXXXXXXXX`) or validated landline format before database insertion.
- **Audit Immutability**: Every completed bulk import must log a tamper evident audit event recording operator ID, source filename, backup path, imported count, and skipped count.

### Security model

- **Permission Gating**: All import RPC commands require `students.manage` permission, restricted to Institution Owner, Administrator, and designated Admissions staff. Cashiers with read only permissions are blocked.
- **File System Boundary**: The file path passed to `inspect_file` must resolve to a readable local file and cannot traverse outside allowed system directories.
- **Memory Safety**: Parsing uses bounded chunk reading to prevent out of memory crashes on low resource Windows machines.

### Critical test scenarios

- Happy path: Importing a valid 100 row `.xlsx` file with full student and guardian details, auto generating student codes (`S2026-0001` through `S2026-0100`), verifies **AC-1, AC-2, AC-3, AC-6, AC-7, AC-10**.
- Sibling grouping: Importing a spreadsheet where three rows share the same guardian mobile number, verifying that one guardian entity is created and all three students link to it, verifies **AC-4**.
- Bengali numeral and date conversion: Importing rows with Bengali digits (`০১৭...`) and mixed date formats (`১২/০৫/২০০৮`, `2008-05-12`), verifying standard normalization, verifies **AC-3**.
- Rollback on database crash: Simulating a fatal constraint error during bulk insert in strict mode, verifying complete rollback and zero leftover rows, verifies **AC-7**.
- Partial import with error download: Importing a file with 90 valid rows and 10 invalid rows under partial mode, verifying 90 students are saved and the 10 failed rows download as a CSV with explicit reasons, verifies **AC-5, AC-8**.
- Academic batch assignment: Importing students with target batch selected, verifying sequential batch roll allocation, verifies **AC-9**.
- Authorization check: Attempting import as a read only receptionist, verifying rejection with permission denied error, verifies **AC-10**.

## Build plan

Following the project Tracer Bullet approach, we prove a thin thread end to end through every layer first, then thicken slice by slice:

- **Milestone 1: Core spreadsheet inspection and column mapper** (satisfies AC-1, AC-2, AC-10)
  - Add `calamine` and `csv` crates to `crates/core/Cargo.toml`.
  - Implement `crates/core/src/import.rs` inspection functions: detect format, read sheets, extract headers, and calculate auto mapping suggestions with English and Bengali aliases.
  - Expose `import.inspect_file` RPC command and write unit tests for `.xlsx` and `.csv` inspection.

- **Milestone 2: Normalization and in memory validation matrix** (satisfies AC-3, AC-4, AC-5, AC-10)
  - Implement Bengali numeral conversion and flexible date parsers.
  - Implement phone normalization, duplicate conflict lookups, and intra file sibling grouping by guardian phone.
  - Build the validation matrix categorizing rows into Valid, Warning, and Error with cell level issue diagnostics.
  - Expose `import.preview_validation` RPC command and write comprehensive validation tests.

- **Milestone 3: Pre import backup, atomic transaction execution, and batch enrollment** (satisfies AC-6, AC-7, AC-9)
  - Implement SQLite online backup snapshot helper creating `<institution_id>/backups/pre-import-<timestamp>.db`.
  - Implement `execute_import` running in a single `IMMEDIATE` SQLite transaction: insert students, link shared guardians, generate atomic student codes, enroll in academic batch, and record audit log.
  - Verify complete transaction rollback on failure.

- **Milestone 4: Rejected rows CSV export generator** (satisfies AC-8)
  - Implement CSV writer in `crates/core/src/import.rs` that outputs un imported rows annotated with `_import_error`.
  - Wire partial import mode to commit valid rows and save the rejected CSV file.
  - Write tests verifying error CSV content matches original input columns plus accurate reason strings.

- **Milestone 5: Desktop import wizard UI and student directory integration** (satisfies AC-10)
  - Build `ui/src/components/students/import-students-dialog.tsx` step by step wizard using shadcn dialog, table, badge, and tabs.
  - Connect wizard to RPC contract via TanStack Query.
  - Add "Import from Excel" action button in `ui/src/components/students/students-page.tsx`.
  - Add automated UI component tests and export TypeScript contracts.

## Consequences

- **What becomes easier**:
  - Educational institutions can migrate hundreds of student records from legacy systems or entrance test lists in minutes without manual data entry.
  - Sibling students are automatically deduplicated into shared families, eliminating phone inconsistencies.
  - Operators get peace of mind knowing that an automatic safety snapshot is taken before every bulk import.
  - Failed rows can be fixed and re imported incrementally using the generated error spreadsheet.

- **What becomes harder**:
  - The Rust core bundle size slightly increases due to including the `calamine` workbook parser.
  - Memory usage must be monitored to ensure multi megabyte files do not exceed memory constraints on 4GB RAM workstations.

## Follow-up

- **Fee Ledger Bulk Allocation**: In Slice 2 (Money), extend the import wizard to allow assigning initial admission fee plans and tuition charges to imported students.
- **Custom Mapping Templates**: In a future update, allow saving and reusing column mapping presets for institutions that receive regular roster files from specific partner schools.
