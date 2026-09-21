# 0008 Rationale. Students and guardians directory

**Date**: 2026-09-21
**Status**: Proposed

## Context

Educational institutions and coaching centers in Bangladesh manage hundreds to thousands of students across multiple academic batches. The counter environment at a coaching center is fast paced, especially during morning admission rush hours and monthly tuition collection periods. Receptionists and counter staff must search for students by partial names, student roll numbers, or guardian phone numbers within fractions of a second while interacting with parents.

Furthermore, internet connectivity at local coaching centers is frequently unreliable or completely absent due to power cuts and local infrastructure disruptions. Therefore, student registration, search, document retrieval, and profile management must operate entirely offline without dependencies on external cloud APIs or online databases.

Families frequently enroll multiple siblings in the same coaching center. In legacy spreadsheet setups or rudimentary software, guardians are duplicated for every sibling, leading to inconsistent contact numbers, outdated emergency addresses, and fragmented tuition fee communications. A clean data model must treat guardians as first class people entities that can be linked to multiple sibling students. When modifying a shared guardian, the application must provide clear visibility into the siblings affected by the edit, while allowing staff to detach into a separate record when families separate.

Finally, managing physical paper records or storing high resolution uncompressed camera photos inside an encrypted SQLite database quickly causes database file bloat, slow page cache performance, and potential application out of memory crashes on low specification Windows workstations (such as 4GB RAM PCs with traditional hard drives). The architecture requires an efficient, local, content addressed document vault that keeps large binary payloads separate from the transactional SQLite ledger while preserving cryptographic data integrity.

## Options considered

### Option 1: Normalized Relational People Directory with Shared Guardians Across Siblings, Content Addressed Local Vault with Safe Image Decoding, and Rowid Mapped SQLite FTS5 Trigram Substring Search (Recommended)

Students and guardians are stored in dedicated normalized tables linked by a many to many junction table (`student_guardians`). A student can have multiple guardians (such as father, mother, or local guardian), with exactly one designated as primary and at least one flagged to receive automated SMS notices. Profile photos and admission documents are resized, hashed, and stored in a local filesystem vault (`<data_root>/<institution_id>/vault/<hash>.<ext>`), referenced in SQLite by metadata records. Search is accelerated using an encrypted SQLite FTS5 trigram virtual table maintained by database triggers bound directly to `students.rowid`, ensuring constant time updates without unindexed table scans.

**Pros**:
- Eliminates duplicate guardian records and ensures sibling phone updates propagate automatically across the family.
- Trigram tokenizer provides instant substring search matching names, student codes, and phone numbers even with partial inputs.
- Binding FTS5 index directly to SQLite table rowid guarantees that triggers never perform full table scans during student updates or deletes.
- Separating binary files into a content addressed filesystem vault keeps the SQLite database compact, fast, and easy to snapshot.
- Safe image decoding limits in the core vault manager prevent high resolution smartphone photos from causing memory exhaustion.

**Cons**:
- Requires building a local vault file storage manager in the Rust core with image resizing capabilities.
- Trigram indexing requires enforcing a minimum 3 character query guard in the user interface to avoid table scans on single character queries.

### Option 2: Embedded Single Guardian Fields Directly on the Student Table

Guardian fields (such as guardian name, relation, and phone) are stored as scalar columns or an embedded JSON object directly inside the `students` table.

**Pros**:
- Simpler single table schema with straightforward INSERT and UPDATE queries.
- No junction table required.

**Cons**:
- Severe data duplication when siblings enroll: updating a parent phone number on one child leaves sibling records with stale contact numbers.
- Inability to model multiple guardians per student (such as both father and mother) cleanly.
- Complicates future sibling discount calculations and consolidated family fee billing.

### Option 3: Separate Guardian Directory with Standard SQL B Tree Queries and In Database BLOB Storage

Guardians are normalized in a separate table, but student photos are stored directly as BLOBs in SQLite columns, and searches rely on standard SQL `LIKE` queries with B tree indexes.

**Pros**:
- Keeps all data inside a single database file without managing a separate filesystem directory for files.
- Familiar SQL syntax without needing SQLite FTS5 extension modules.

**Cons**:
- Storing hundreds of megabytes of image BLOBs inside SQLite severely degrades database backup speed, cache locality, and memory usage on 4GB RAM machines.
- SQL `LIKE '%query%'` queries cannot use standard B tree indexes, resulting in full table scans that take several seconds on large databases with 10,000 students.
- Inadequate search experience for counter receptionists who require instant response times.

## Rationale

Option 1 is selected because it directly aligns with the operational reality of coaching centers in Bangladesh and adheres to Coachify local first architecture principles. 

First, normalizing guardians and linking them through `student_guardians` matches real world family relationships. Parents often visit coaching centers to pay fees or check progress for multiple children. Having a shared guardian record allows the receptionist to look up a phone number once and immediately view all enrolled siblings. When an SMS notification module is added in future slices, a single SMS can reach the verified family contact without conflicting numbers. The user interface explicitly informs receptionists whenever an edit affects multiple siblings, providing peace of mind.

Second, SQLite FTS5 with the trigram tokenizer provides superior search capabilities for Bengali and English names. Unlike standard word tokenizers that depend on whitespace boundaries, the trigram tokenizer splits text into three character sequences, enabling instant substring matching. A receptionist typing "0171", "Karim", or a school roll number instantly matches the relevant student records in under 50 milliseconds, even across databases containing 10,000 students. By binding triggers to the native SQLite integer `rowid`, index updates remain constant time operations without table locking overhead.

Third, keeping photo and document binaries in a content addressed filesystem vault protects database health. On a Windows PC with spinning hard drives, a bloated SQLite file slows down checkpoint operations and increases backup times. By resizing photos to a crisp 800 by 800 resolution with strict memory decoding safeguards, and storing them on disk named by their SHA256 digest with their file extension, Coachify achieves deduplication, compact storage, and rapid page cache retrieval.
