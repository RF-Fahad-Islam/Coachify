# Core records (Slice 1, the walking skeleton)

Back to [index](index.md).

### 9. Students & guardians · needs a decision
The first real thread through every layer: create a student with a guardian, find them instantly by ID, name, roll, or any phone, and open their profile. Guardians are shared across siblings.
**Done when:** a receptionist creates a student and guardian offline; search by partial name, ID, or phone in any common format finds them; siblings share one guardian; the profile shows documents and a photo from the vault.
- [ ] Design it (spec): `/architect students & guardians`

### 10. Programs, batches, staff & enrollments
Programs, subjects, batches with teacher and schedule, staff profiles, and enrollments that keep history (a transfer ends one enrollment and starts another).
**Done when:** a student can be enrolled, transferred, and ended without losing past enrollments; a batch shows its current roster and capacity.
- [ ] Build it: `/develop programs, batches, staff & enrollments`

### 11. Excel student import · needs a decision
Guided import from Excel or CSV: map columns, preview, validate, import in one transaction after a safety backup, download failed rows.
**Done when:** a real messy Excel file imports with valid, warning, and error counts shown; nothing partial is saved on failure; failed rows download as a file.
- [ ] Design it (spec): `/architect excel student import`

### 12. Demo & load data generator
One generator that builds the demo institution for sales and the large dataset for performance tests.
**Done when:** a demo institution with 500 students and a full year of activity opens in a separate database; the large profile (10,000 active students, 3 years of history) generates repeatably.
- [ ] Build it: `/develop demo & load data generator`
