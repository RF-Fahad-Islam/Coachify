# 0010 Rationale. Excel and CSV student import engine

**Date**: 2026-09-22
**Status**: Proposed

## Context

Coaching centers and academies in Bangladesh frequently conduct seasonal entrance examinations, batch intakes, and transition drives where hundreds of prospective students register at once. Admissions data is routinely collected across disparate external spreadsheets created by school staff, exam proctors, or third party test centers. These spreadsheets arrive in a variety of messy formats: Microsoft Excel workbooks (`.xlsx` and legacy binary `.xls`), comma separated values (`.csv`), files with mixed English and Bengali headers, dates entered in varying styles (such as `12/05/2008`, `2008-05-12`, or Excel serial integers), and phone numbers typed with Bengali numerals (`০১৭...`) or missing country prefixes.

Entering hundreds of student and guardian profiles manually through counter forms during admission week is error prone and overwhelms reception staff. Coaching centers require an automated, guided bulk import tool that eliminates manual typing while maintaining strict institutional data integrity.

Importing external data directly into a production database introduces serious operational hazards:
1. An unvalidated spreadsheet with bad rows could corrupt database constraints or insert broken phone numbers.
2. A crash or power cut midway through importing 500 rows could leave half the roster persisted, creating phantom students without guardians.
3. If an import fails completely because five rows have invalid dates, forcing the operator to fix the spreadsheet and restart from row 1 wastes valuable counter time.
4. Siblings appearing in the spreadsheet could easily be imported as duplicate guardian records with conflicting phone numbers, breaking family contact consistency.

To solve these problems safely on offline Windows office computers, Coachify requires an import engine that performs in memory dry run validation, captures an automated safety backup before database mutations, enforces atomic transaction commits, deduplicates sibling families, and exports rejected rows for correction.

## Options considered

### Option 1: Pure Rust In Process Parsing (`calamine` and `csv`), In Memory Validation Matrix, Automated SQLite Online Backup Snapshot, Atomic Transaction Commit, and Rejected Rows CSV Export (Recommended)

The entire import pipeline runs headless inside the pure Rust core library (`crates/core/src/import.rs`). Excel files are parsed using `calamine` and CSV files using `csv`. Column matching uses an in memory dictionary of English and Bengali header aliases with fuzzy scoring. Rows are parsed and validated in memory against Coachify domain constraints without writing to SQLite. Before any insertion, the engine uses the SQLite online backup API to write an automatic snapshot to `<institution_id>/backups/pre-import-<timestamp>.db`. Valid rows are inserted in a single `IMMEDIATE` database transaction, shared guardians are linked across sibling rows, and rejected rows are written to a downloadable CSV file annotated with explicit failure causes.

**Pros**:
- Pure Rust execution with zero C library dependencies ensures reliable cross compilation and deterministic memory safety on Windows.
- Parsing and validating entirely in memory guarantees that zero garbage data reaches the SQLite database file before validation passes.
- SQLite online backup snapshot provides an instant safety net against hardware failures or power disruptions.
- Single immediate transaction guarantees atomic all or nothing consistency: if an error occurs, the rollback leaves the database completely clean.
- Deduplicates sibling guardians by normalized phone number, preserving the family data invariant established in spec 0008.
- Generating a CSV file of rejected rows allows operators to import 95% of valid students immediately while reviewing only the problematic rows.

**Cons**:
- Adds `calamine` (approx 150KB compiled footprint) to `crates/core` dependencies.
- Large workbooks with tens of thousands of rows require streaming row iterators to keep memory consumption low.

### Option 2: Client Side JavaScript Parsing (`SheetJS` or `PapaParse` in Webview) with Raw Row RPC Ingestion

The web interface reads spreadsheet files in the browser webview using JavaScript libraries (`xlsx` or `PapaParse`), converts rows into JSON objects, and sends batches of student objects over the Tauri RPC bridge to `students.create`.

**Pros**:
- Core library remains untouched by spreadsheet parsing dependencies.
- JavaScript spreadsheet libraries are widely available.

**Cons**:
- Severe memory bloat: deserializing large spreadsheets into thousands of JSON objects in the webview web thread easily causes high memory pressure on low power 4GB RAM machines.
- Violates the foundational architectural rule: domain logic, data normalization, and integrity validation must live strictly in `crates/core`, never in the UI.
- No ability to execute a pre import database backup snapshot or atomic bulk transaction from the client side without multiple round trips.

### Option 3: External Python or Standalone CLI Script

A separate CLI binary or Python script converts spreadsheets into SQLite dumps or runs a sidecar process.

**Pros**:
- Keeps the main Coachify application binary smaller.

**Cons**:
- Heavy operational burden: coaching centers run standard Windows PCs without Python or external script runtimes installed.
- Fragile inter process communication, complicated error reporting, and complex updater requirements.

### Option 4: Staged Temporary SQLite Tables with SQL Triggers

The file is loaded raw into a temporary SQLite staging table (`_import_staging`), followed by SQL triggers and stored procedures that validate and migrate rows into `students` and `guardians`.

**Pros**:
- Everything is expressed in SQL inside the database.

**Cons**:
- SQLite SQL dialect lacks sophisticated string manipulation, regex, fuzzy matching, and Bengali numeral transliteration.
- Writing unvalidated, messy raw text into SQLite tables can bloat the database and cause unindexed table locks.
- Constructing dynamic error messages and exporting clean rejected CSV files from SQLite triggers is clumsy and brittle.

## Rationale

Option 1 is selected because it preserves the architectural integrity of Coachify: headless pure Rust domain logic, strict type safety, local first performance, and maximal operational resilience.

First, placing spreadsheet parsing in the core library using `calamine` and `csv` guarantees that data normalization is executed with compiled Rust speed and deterministic safety. In our target market of Bangladesh, spreadsheets frequently combine English and Bengali characters and numbers. A Rust parser equipped with explicit Unicode character mapping transforms Bengali digits (`০-৯`) into standard ASCII numerals (`0-9`) in nanoseconds, normalizes mobile numbers using the battle tested `crate::phone::normalize_phone` module, and parses multiple date patterns without client side ambiguity.

Second, the combination of an automated pre import safety backup and a single immediate SQLite transaction delivers unmatched reliability. Coaching centers in regional districts frequently suffer from erratic electricity and unexpected power outages. If power drops during a bulk admission of 500 students, the immediate transaction rolls back cleanly on restart. Furthermore, having an explicit snapshot file (`pre-import-<timestamp>.db`) stored in the institution backup directory gives administrators a physical recovery point that can be inspected or restored if an operator accidentally imports the wrong academic year roster.

Third, selective import mode with downloadable rejected rows solves the real world admission desk dilemma. When counter staff import a class roster of 200 applicants, 192 rows are usually valid while 8 rows might have missing phone digits or invalid birthdates. Enforcing strict all or nothing rejection forces staff to track down parents or manually edit the spreadsheet before registering a single student. By committing the 192 valid students and producing a clean CSV containing only the 8 failed rows with an added `_import_error` explanation column, staff can register the vast majority immediately and address the exceptions with zero duplicate work.

Fourth, sibling family deduplication in memory preserves family relationships. When two children from the same family enroll together, their rows in the spreadsheet share the father or mother mobile number. The import engine groups rows by normalized guardian phone, creates a single primary guardian entity, and links both students to it. This prevents duplicate guardian records in the database, ensuring that tuition fee invoices and SMS alerts stay unified.

## References

- Calamine pure Rust spreadsheet reader: `https://crates.io/crates/calamine` (basis: zero C dependency Excel reading)
- Csv fast streaming CSV reader and writer for Rust: `https://crates.io/crates/csv` (basis: memory efficient CSV parsing and error export)
- SQLite Online Backup API: `https://www.sqlite.org/backup.html` and `rusqlite::backup::Backup` (basis: live database snapshot without table lock)
- Spec 0008: Students and guardians directory (basis: normalized people schema, phone normalization, and shared guardian invariant)
- Spec 0009: Academic programs, batches, staff directory, and student enrollments (basis: batch structure and sequential roll allocation)
