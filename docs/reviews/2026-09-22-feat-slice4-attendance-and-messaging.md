# Review, feat-slice4-attendance-and-messaging, 2026-09-22

**Reviewed by**: Gemini Pro (author on Claude Sonnet)
**Scope**: 4 core files + tests + migrations + UI, branch
**Verdict**: Blocked

## Summary
The slice successfully introduces robust omnichannel messaging and a clean attendance module with solid lock-window rules. The DPAPI integration for provider secrets and GSM 03.38 analysis are well implemented. However, critical authorization flaws and an inability to clear provider secrets block this PR.

## Blockers
### 🔴 Incorrect authorization gate for mutating attendance records, `crates/core/src/dispatcher.rs:791`
**Problem**: The RPC endpoints `attendance.save_session` and `attendance.lock_session` are both gated with `AuthRequirement::Permission("attendance.view")`.
**Why it matters**: Any user with read-only view access can overwrite and lock daily attendance records, bypassing the intended `attendance.mark` and `attendance.edit_previous` role checks entirely.
**Suggested fix**: Update both commands to require `AuthRequirement::Permission("attendance.mark")`.

## Major
### 🟠 Missing user ID in attendance retroactive edit audit log, `crates/core/src/attendance.rs:417`
**Problem**: When a user performs a retroactive edit on a locked session, the audit log entry is written with `None` instead of the actor's `user_id`.
**Why it matters**: The audit log loses attribution for highly sensitive edits (retroactive attendance changes), defeating the purpose of the security requirement (AC-4). 
**Suggested fix**: Pass `Some(user_id.to_string())` instead of `None` into `crate::audit::record_audit_entry`.

### 🟠 Cannot clear provider secrets once set, `crates/core/src/comms.rs:857`
**Problem**: In `save_provider_config`, if the incoming payload provides an empty string as a secret, it evaluates to `None` for `secret_ref_b64`. The `else` branch query completely omits `secret_ref` in its `ON CONFLICT DO UPDATE SET` clause.
**Why it matters**: Administrators cannot remove a compromised or unwanted password for a gateway; the system stubbornly preserves the old secret in the DB.
**Suggested fix**: Use a single UPSERT query where `secret_ref` is always updated. If `payload.secret` is `Some("")`, explicitly update the database column to `NULL`.

## Minor
### 🟡 Missing unit test for provider secret clearing, `crates/core/tests/comms_and_outbox_integration.rs:1`
**Problem**: The tests cover DPAPI encryption and masking but do not verify the lifecycle of deleting/clearing a previously set secret.
**Why it matters**: It allowed the secret-clearing bug to slip through.
**Suggested fix**: Add a quick test case asserting that an empty secret payload successfully updates the DB column to null.

## Strengths
- The `evaluate_is_locked` function accurately handles both the 24-hour limit and the midnight trap guard.
- Clever implementation of GSM 03.38 alphabet evaluation vs Unicode limits for SMS segments.
- High test coverage for outbox status lifecycles and SMS deduplication.

## Test coverage
Overall excellent integration coverage. The core features are well-exercised, but coverage is missing for the edge case of clearing a provider configuration secret.
