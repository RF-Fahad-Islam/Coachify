# Verify: Results, Grading and Report Cards (spec 0027)

Steps derived from spec 0027 acceptance criteria.

## UI and Manual Verification

- [ ] In the Exams view (`/exams`), verify that system default grading schemes are present: "NCTB Bangladesh National Curriculum (GPA 5.0)" and "British Curriculum (GPA 4.0)" with complete grade bands (verifies AC-1).
- [ ] For an exam with verified marks across 5 subjects (including 1 optional 4th subject): open `[Preview Results]`. Verify that calculated GPAs, letter grades, total marks, and ranks render accurately in the preview table (verifies AC-2).
- [ ] Test 4th subject math: verify a student who scored A+ (GP 5.0) in the 4th subject receives +3.00 bonus points added to their mandatory GP sum, and the divisor is 4 (the count of mandatory subjects) (verifies AC-2).
- [ ] Test mandatory fail rule: verify a student who scored A+ in 4 subjects but scored F (under 33 marks) in mandatory Physics has an overall GPA of 0.00 and Grade F (verifies AC-2).
- [ ] Test 4th subject fail rule: verify a student who scored A in all mandatory subjects but failed the optional 4th subject STILL PASSES with GPA calculated from mandatory subjects alone (verifies AC-2).
- [ ] Click `[Publish Results]`: confirm in the confirmation modal. Verify that the exam transitions to `Published` status and generates a new snapshot (verifies AC-3).
- [ ] Attempt to edit the underlying grading scheme: edit a percentage threshold in Settings -> Grading Schemes. Re-open the published exam and verify that published GPAs, grades, and ranks remain 100% identical to the frozen snapshot (verifies AC-3).
- [ ] Inspect merit rankings for two students with identical GPAs: verify ties are broken deterministically by Total Marks, then Mandatory Marks, then Roll, then Student ID (verifies AC-4).
- [ ] Click `[Print Report Card]` for a student: verify that the report card dialog renders an authentic A4 portrait layout with academic emblem, bilingual Bengali/English headers, 4th subject bonus explanation, and 3-signature block (verifies AC-5).
- [ ] Click `[Print Batch Tabulation Sheet]`: verify that the tabulation sheet renders in Landscape mode with all participating subjects, GPA, total marks, and ranks displayed cleanly without horizontal table clipping (verifies AC-6).
- [ ] With `result_published` SMS automation rule enabled: verify that publishing an exam enqueues SMS messages into the outbox for all students' guardians with their GPA and merit rank (verifies AC-7).

## Commands

- [ ] `cargo test -p coachify_core --test exams_and_results_integration` (verifies NCTB GPA-5 calculation, 4th subject bonus math, mandatory fail propagation, 5-tier deterministic tie-breaking, immutable result snapshots, and trigger protections, satisfies AC-1 through AC-4).
- [ ] `cargo run -p xtask check-types` (verifies TypeScript contract freshness in `ui/src/gen/rpc.ts` for all result and grading RPC types).
- [ ] `cargo run -p xtask check-migrations` (verifies Migration 0015 schema validity and released migration immutability).
- [ ] `pnpm --dir ui test` (verifies ResultPreviewView, ReportCardTemplate, TabulationSheetTemplate, and landscape styling defenses).

## Acceptance Criteria Coverage

- AC-1 (Grading Scheme Configuration & NCTB Seed): verified by database migration seeds and scheme retrieval tests.
- AC-2 (Calculation Preview and 4th Subject Rule Engine): verified by grading math unit tests for 4th subject bonus, divisor math, and mandatory fail gates.
- AC-3 (Result Publishing and Immutable Freezing): verified by snapshot insertion tests, frozen JSON verification, and SQLite immutability abort triggers.
- AC-4 (Deterministic 5-Tier Tie-Breaking): verified by ranking algorithm tests asserting identical rank stability across multiple runs and cross-batch roll collisions.
- AC-5 (Printable Student Report Cards): verified by report card data retrieval tests and UI template rendering with Kalpurush font.
- AC-6 (Printable Landscape Batch Tabulation Sheet): verified by tabulation sheet matrix generator and CSS `table-layout: fixed` checks.
- AC-7 (Omnichannel Automation Hook): verified by outbox insertion tests on result publish action.
