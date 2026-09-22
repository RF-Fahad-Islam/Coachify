# 0027. Results, Grading and Report Cards (Rationale)

## Context

Publishing academic results is a high-stakes operational event for coaching centers and private academic institutes in Bangladesh. Parents judge the institute's credibility by the visual quality and accuracy of the report cards distributed after exams. Furthermore, coaching centers post batch tabulation sheets and merit lists on notice boards, where any ambiguity or calculation error invites intense scrutiny and disputes.

Key complexities in the Bangladeshi grading ecosystem:
1. **The 4th Subject Bonus Formula**: In the Bangladesh National Curriculum (NCTB) for Class 9–10 (SSC) and Class 11–12 (HSC), students choose an optional 4th subject (e.g. Higher Math, Biology, Agriculture). The student's grade points above 2.00 in the 4th subject are added as bonus points to the mandatory grade points sum, and the total is divided by the *number of mandatory subjects* (not total subjects). Crucially, failing the 4th subject does not cause overall failure.
2. **Hard Fail Propagation**: If a student fails even a single mandatory subject (e.g. General Math or English), their overall result is an automatic F (GPA 0.00), regardless of high marks in other subjects.
3. **Historical Result Mutation Hazard**: If grading rules are evaluated on-the-fly dynamically against live grading scheme tables, an administrator who changes grade boundary percentages for a subsequent semester will unintentionally alter historical results from prior years, corrupting student transcripts.
4. **Landscape Print Clipping**: Cross-subject tabulation sheets frequently feature 6 to 10 subjects side-by-side. On entry-level desktop laser printers, naive table layouts cause the rightmost columns (GPA, Total, Rank) to clip off the edge of the paper or wrap unpredictably.

Coachify addresses these requirements with an immutable snapshot model (`result_snapshots`), NCTB-compliant 4th subject bonus math, deterministic 5-tier tie-breaking, and CSS-hardened landscape print templates.

---

## Options Considered

### Option 1: Immutable Frozen Snapshots with NCTB 4th Subject Math, Fixed Landscape Layout, and Omnichannel Publishing Hooks (Chosen)

Publishing results creates a dedicated `result_snapshots` record containing frozen JSON representations of the grading scheme and calculation parameters, accompanied by immutable child rows in `results` and `result_subject_details`.
- Historical permanence: Even if grading schemes or bands are updated or deleted later, existing published results are untouched.
- Full compliance with Bangladeshi SSC/HSC NCTB calculation guidelines.
- 5-tier deterministic tie-breaking eliminates disputed rankings.
- Fixed-width CSS layout (`table-layout: fixed`) guarantees landscape tabulation sheets print without horizontal clipping on standard A4 paper.
- Automatic integration with SMS and WhatsApp outbox to announce results to guardians.

Pros:
- Complete auditability and legal defensibility of historical academic records.
- Matches parent and educator expectations across Bangladesh.
- Zero floating-point drift or report card clipping.

Cons:
- Consumes slightly more disk space due to frozen JSON copies and detailed result rows (negligible for local SQLite).

### Option 2: Dynamic Live Calculation on Read

Do not store calculated GPAs, grades, or ranks; calculate them on-the-fly whenever a report card or tabulation sheet is opened by querying `marks` and `grading_schemes`.

Pros:
- Fewer database tables.

Cons:
- Catastrophic historical instability: changing a grading scheme immediately rewrites all past report cards.
- Performance penalty: computing ranks and tie-breakers across hundreds of students on every page view or report generation freezes the UI.
- Impossible to prove what grade a student held at the time of publication during academic disputes.

### Option 3: External PDF Generator CLI Tool

Delegate report card and tabulation sheet generation to external command-line tools like wkhtmltopdf or headless Chrome binaries.

Pros:
- Decouples rendering from the frontend.

Cons:
- Heavy binary dependencies that complicate the NSIS installer.
- Fragile Bengali font rendering and text shaping.
- WebView2 in Tauri already provides native Chromium printing and PDF rendering with zero added binary size.
