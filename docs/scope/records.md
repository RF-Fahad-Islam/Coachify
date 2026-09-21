# Core records (Slice 1, the walking skeleton)

Back to [index](index.md).

### 9. Students & guardians · in-progress
The first real thread through every layer: create a student with a guardian, find them instantly by ID, name, roll, or any phone, and open their profile. Guardians are shared across siblings.
**Done when:** a receptionist creates a student and guardian offline; search by partial name, ID, or phone in any common format finds them; siblings share one guardian; the profile shows documents and a photo from the vault.
spec [0008](../specs/0008-students-and-guardians/index.md)
- [x] Design it (spec): `/architect students & guardians`
- [x] Build it: `/develop students & guardians`
  - [x] Migration 0010 adding FTS5 trigram search index, rowid sync triggers, and student permissions, covers AC-6, AC-8, AC-9
  - [x] Core people repository, phone normalization, and atomic code generator, covers AC-1, AC-2, AC-3, AC-4, AC-9
  - [x] Local filesystem vault manager with image resizing and SHA-256 content addressing, covers AC-5
  - [x] Command dispatcher RPC handlers and typed TypeScript contract, covers AC-7, AC-8
  - [x] Student directory table with instant debounced search and profile view with shared guardian management, covers AC-3, AC-7, AC-10
- [x] Verify it: `/check verify students & guardians`
- [x] Test it: `/test students & guardians`

### 10. Programs, batches, staff & enrollments · done
Programs, subjects, batches with teacher and schedule, staff profiles, and enrollments that keep history (a transfer ends one enrollment and starts another).
**Done when:** a student can be enrolled, transferred, and ended without losing past enrollments; a batch shows its current roster and capacity.
spec [0009](../specs/0009-programs-batches-staff-enrollments/index.md) · code in [crates/core/src/academic.rs](../../crates/core/src/academic.rs)
- [x] Design it (spec): `/architect programs, batches, staff & enrollments`
- [x] Build it: `/develop programs, batches, staff & enrollments`
  - [x] Migration 0011 adding programs, subjects, batches, staff, enrollments, delete triggers, and FTS5 staff index
  - [x] Schedule collision detection algorithm and pure Rust academic core
  - [x] 23 command dispatcher RPC handlers, audit logs, and typed TypeScript contract
  - [x] High-density academic classes & batch roster UI with live conflict warnings
  - [x] Staff directory view with FTS5 search and student profile enrollment history
- [x] Verify it: `/check verify programs, batches, staff & enrollments`
- [x] Test it: `/test programs, batches, staff & enrollments`

### 11. Excel student import · done
Guided import from Excel or CSV: map columns, preview, validate, import in one transaction after a safety backup, download failed rows.
**Done when:** a real messy Excel file imports with valid, warning, and error counts shown; nothing partial is saved on failure; failed rows download as a file.
spec [0010](../specs/0010-excel-student-import/index.md)
- [x] Design it (spec): `/architect excel student import`
- [x] Build it: `/develop excel student import`
  - [x] Core spreadsheet inspection and bilingual column auto mapper, covers AC-1, AC-2, AC-10
  - [x] In memory validation matrix, phone normalization, date parsing, and sibling grouping, covers AC-3, AC-4, AC-5
  - [x] Pre import database safety backup snapshot and atomic transaction bulk import, covers AC-6, AC-7, AC-9
  - [x] Rejected rows CSV file generator with failure reason annotations, covers AC-8
  - [x] Multi step desktop import wizard dialog and student directory integration, covers AC-10
- [x] Verify it: `/check verify excel student import`
- [x] Test it: `/test excel student import`

### 12. Demo & load data generator · done
One generator that builds the demo institution for sales and the large dataset for performance tests.
**Done when:** a demo institution with 500 students and a full year of activity opens in a separate database; the large profile (10,000 active students, 3 years of history) generates repeatably.
spec [0011](../specs/0011-demo-load-data-generator/index.md)
- [x] Design it (spec): `/architect demo & load data generator`
- [x] Build it: `/develop demo & load data generator`
  - [x] Core deterministic generator engine, embedded bilingual dictionaries, and entity synthesis, covers AC-1, AC-2, AC-3, AC-4
  - [x] Chunked bulk database insertion pipeline, PRAGMA speed optimizations, and integrity checkpoints, covers AC-5, AC-6, AC-7
  - [x] Disk space pre flight checks, path sandboxing, atomic temp files, and production safety guards, covers AC-8, AC-9, AC-10
  - [x] Dynamic connection pool switching, query cache invalidation, and CLI xtask runner, covers AC-11, AC-12, AC-14
  - [x] Desktop developer UI dialog, progress reporting, and Demo Mode status banner, covers AC-13
- [x] Verify it: `/check verify demo & load data generator`
- [x] Test it: `/test demo & load data generator`
- [x] Document it: `/document pr and commit`
