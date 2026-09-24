# 0035. AI privacy center

> Feature 57 (Slice 10: AI foundation)

## Summary

This specification defines the AI Privacy Center and its client-side redaction engine. While cloud AI provides powerful analytical synthesis, coaching center records contain sensitive family contacts, academic evaluations, and financial balances. 

The AI Privacy Center puts the institution owner in complete command of data boundaries. A dedicated settings screen provides granular toggles for sensitive data categories. Crucially, the system ships with a **conservative default: personal identifiers (names, phones, guardians) and financial figures are disabled by default**. 

When a category is disabled, Coachify's client-side redaction engine automatically strips or replaces those values with pseudonyms (e.g. `Student #STU-1042`, `[PHONE REDACTED]`, `*** poisha`) *before* any payload leaves the PC. The local desktop UI maintains an ephemeral session dictionary to re-hydrate pseudonyms on the user's screen without leaking sensitive data to cloud servers.

## Requirements

**User stories**:
* As an institution owner, I want to control exactly what data leaves our local PC for cloud AI processing so that we protect student privacy and comply with data safety expectations.
* As an owner, I want the initial system state to be secure and conservative by default, so that nobody accidentally leaks phone numbers or financial figures without conscious configuration.
* As an administrator, I want to toggle student names or financial amounts on or off at will with immediate effect on all subsequent AI requests.
* As an auditor, I want a permanent log recording what categories were allowed and what categories were redacted for every single AI inquiry.
* As a user, I want answers in the copilot drawer to show recognizable student names on my screen even if the cloud model only received anonymized pseudonyms.

**Acceptance criteria**:
* **AC-1**: `ai_privacy_settings` table stores category toggles: `allow_student_names`, `allow_phone_numbers`, `allow_guardian_info`, `allow_financial_figures`, `allow_exam_marks`, and `allow_staff_names`.
* **AC-2**: Default configuration is strictly conservative: all toggles are initialized to `0` (disabled/redacted).
* **AC-3**: Client-side redaction engine in `crates/core/src/ai/privacy.rs` inspects all user prompts and controlled tool outputs before transmission, replacing disabled fields with deterministic pseudonyms or standard masks:
  * Student Name $\to$ `Student #[student_code]`
  * Phone Number $\to$ `[PHONE REDACTED]`
  * Guardian Info $\to$ `[GUARDIAN REDACTED]`
  * Financial Figures $\to$ `*** poisha` (or bucketed relative tiers)
  * Exam Marks $\to$ `*** marks` (or grade letters only)
  * Staff Name $\to$ `Teacher #[teacher_code]`
* **AC-4**: Ephemeral de-anonymization dictionary: The local copilot maintains an in-memory session map per request (`"Student #STU-1042" -> "Rahim Uddin"`). When displaying the final synthesized response in the UI, pseudonyms are re-hydrated to their real names for the local user.
* **AC-5**: Command `ai.update_privacy_settings` permits Owner and Admin users to update toggles, logging a security audit event in `audit_log` detailing changed categories.
* **AC-6**: Every AI inquiry records the active allowed categories, redacted categories, and total replacement count in `ai_audit_logs`.
* **AC-7**: Dedicated AI Privacy Center view in UI renders toggle switches with descriptive warnings, risk indicators, and preview of redacted vs original data.

## Inline rationale

Performing redaction entirely inside `crates/core` on the client PC guarantees that network eavesdroppers, cloud proxies, or upstream AI providers can never access sensitive student PII or confidential tuition balances. The ephemeral de-anonymization dictionary ensures an optimal user experience locally without compromising cloud confidentiality.

## Redaction rules & pseudonyms

```rust
pub struct PrivacySettings {
    pub allow_student_names: bool,       // Default: false
    pub allow_phone_numbers: bool,       // Default: false
    pub allow_guardian_info: bool,       // Default: false
    pub allow_financial_figures: bool,   // Default: false
    pub allow_exam_marks: bool,          // Default: false
    pub allow_staff_names: bool,         // Default: false
}

pub struct RedactionContext {
    /// In-memory ephemeral map: Pseudonym -> Original Text
    pub token_map: HashMap<String, String>,
    pub redacted_categories: Vec<String>,
    pub total_redactions: usize,
}
```

### Example transformation

| Field | Original value | When Allowed | When Redacted (Default) |
|---|---|---|---|
| Student Name | Tanvir Hasan | `"Tanvir Hasan"` | `"Student #STU-0089"` |
| Student Phone | 01711223344 | `"01711223344"` | `"[PHONE REDACTED]"` |
| Guardian Name | Kamal Hasan | `"Kamal Hasan (Father)"` | `"[GUARDIAN REDACTED]"` |
| Outstanding Dues | 650000 poisha (6,500 Tk) | `650000` | `"*** poisha"` |
| Exam Marks | 88.5 marks | `88.5` | `"*** marks"` |
| Teacher Name | Farhan Ahmed | `"Farhan Ahmed"` | `"Teacher #TCH-0012"` |

## API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| `ai.get_privacy_settings` | query | _None_ | `AiPrivacySettings` (all toggles, updated_at, updated_by_name) | Any authenticated | _None_ |
| `ai.update_privacy_settings` | command | `allow_student_names: bool`, `allow_phone_numbers: bool`, `allow_guardian_info: bool`, `allow_financial_figures: bool`, `allow_exam_marks: bool`, `allow_staff_names: bool` | `AiPrivacySettings` | Owner, Admin | `unauthorized` |
| `ai.get_audit_log` | query | `limit: Option<u32>`, `before_time: Option<String>` | `Vec<AiAuditLogItem>` | Owner, Admin | `unauthorized` |

## Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| `get_privacy_settings` | toggle states | Read from `ai_privacy_settings` singleton row |
| `update_privacy_settings` | updated settings row | Mutated in `ai_privacy_settings` and logged to `audit_log` |
| `ask_copilot` | redacted tool payloads | Filtered by `PrivacyRedactionEngine` in memory |
| `get_audit_log` | audit entries | Read from `ai_audit_logs` joined with `users` |

## Build tasks

1. Implement `crates/core/src/ai/privacy.rs` with `PrivacyRedactionEngine`, token pseudonym generator, and ephemeral de-anonymization re-hydration mapper, satisfies **AC-2**, **AC-3**, **AC-4**
2. Wire `ai.get_privacy_settings` and `ai.update_privacy_settings` commands in `dispatcher.rs` with audit logging, satisfies **AC-1**, **AC-5**
3. Implement `crates/core/src/ai/audit.rs` logging transmitted vs redacted categories, token metrics, and latency in `ai_audit_logs`, satisfies **AC-6**
4. Build `AiPrivacyCenterView` in React with category toggles, risk badges, conservative preset reset button, and live preview simulator, satisfies **AC-5**, **AC-7**
