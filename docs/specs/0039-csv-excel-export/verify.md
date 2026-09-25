# 0039. CSV / Excel Export: Verification Plan

## Automated Verification

```bash
cargo test -p coachify_core --test reports_and_search_integration
pnpm --dir ui test
```

## Verification Matrix

| Criterion | Test | Expected Outcome |
|---|---|---|
| AC-1, AC-3 | `students_list`, `dues_list`, `exam_results_list` export tests | Exports only filtered rows matching `batch_id`, `status`, `search_query`, or `exam_id` |
| AC-2 | UTF-8 BOM and Bangla text preservation test | Output starts with `\u{FEFF}`, uses `\r\n`, and preserves Bangla characters (`আরিফ হোসেন`, `৳`) |
| AC-4 | Formula injection defense test | Cells beginning with `=CMD()`, `+cmd`, `-cmd`, `@SUM` are prefixed with `'` and quoted |
| AC-5 | 10,000-row export performance test | Exporting 10,000 student rows completes in < 500ms |
