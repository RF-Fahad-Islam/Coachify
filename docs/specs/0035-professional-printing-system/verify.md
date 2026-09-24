# 0035. Professional Printing System: Verification Plan

> Verification suite and criteria for Spec 0035 (Professional Printing System)

## Automated test suite

Run the dedicated professional printing integration suite:

```bash
cargo test -p coachify_core --test professional_printing_integration
```

Verify contract freshness and run UI tests:

```bash
cargo run -p xtask check-types
cargo build --workspace
pnpm --dir ui test
```

## Test scenarios & criteria matrix

| Test scenario | Acceptance criteria | Verification method |
|---|---|---|
| ID card A4 multi card layout | AC-1, AC-8 | Generates grid of 8 student cards with cut guides, verifies intact geometry and fallback avatar |
| ID card single PVC format | AC-1 | Renders isolated CR80 card dimensions (85.6 mm × 53.98 mm) with barcodes and student codes |
| Admit card batch generation | AC-2, AC-9 | Renders admit cards with exam schedule tables, QR codes, and complex Bengali fonts |
| Batch print chunking | AC-3 | Queues 150 items, verifies processing in 50 item chunks with progress reporting and cancellation |
| Thermal due slip format | AC-4 | Generates 80mm continuous roll receipt layout for dues with barcode and Bengali currency figures |
| Template configuration persistence | AC-5, AC-6 | Updates header visibility, logo toggle, and signature title via RPC, asserting database persistence |
| Print job audit logging | AC-7 | Records print job run in database, confirming item count, user ID attribution, and timestamp |
| Missing photo fallback | AC-8 | Simulates student record with missing photo, asserting sharp vector avatar without grid collapse |
| Bengali ligature rendering | AC-9 | Tests complex conjunct Bengali strings, verifying correct glyph shaping and vowel mark positioning |
| Universal preview modal | AC-10 | Tests interactive preview modal with zoom controls, orientation toggle, and PDF export trigger |

## Manual verification checklist

1. Start the desktop application using `pnpm dev`.
2. Open the Student Directory, select several students, and choose Print ID Cards. Verify both A4 multi card sheet and single card preview options.
3. Open the Exams section, select an upcoming exam, and trigger Print Admit Cards. Confirm student details, exam routines, and QR codes appear correctly.
4. Open the Dues Center and trigger Print Due Slips for a selected batch. Confirm the 80mm thermal preview displays clear Bengali currency formatting.
5. In Settings, open Print Template Settings and toggle header visibility or update the signature title. Verify that subsequent print previews immediately reflect the updated settings.
6. Trigger a batch print job of over 50 items and verify that the progress bar displays chunk progress smoothly.
