# Review, feat/audit-log (spec 0006 migration engine only), 2026-09-21

**Reviewed by**: claude-opus-5 (author on a different model)
**Scope**: 17 files, branch vs `main` (merge base `b1b252f`), scoped to the migration engine feature (spec 0006). The audit-log and setup-wizard work on this branch is reviewed elsewhere.
**Verdict**: Blocked

## Summary

This adds a staging-copy migration engine (`crates/core/src/migrations.rs`, 1341 new lines) with a pre-flight disk check, a Windows `LockFileEx` process lock, per-migration transactions on an isolated `staging.db`, three validation gates, an atomic rename swap with archives, crash-swap recovery, two permission-gated RPC queries, and a React recovery screen. The architecture matches spec 0006 closely and the happy path is solid and well tested; the WAL-checkpoint-then-copy ordering and the `Err` path that purges staging without touching the live file are both correct.

Three things block merge. The released-migrations immutability guard protects **one** of nine migrations, and `xtask check-migrations` prints "All released migrations are intact" while a tampered `0005_financial_ledger.sql` sails through — I verified this empirically. Crash-swap recovery sorts archives lexicographically, so once version 10 exists it restores the **v9** archive instead of the v10 one, silently reverting the institute's database — also verified empirically. And the test that is supposed to prove AC-4's headline guarantee ("a failing migration leaves the live database unmodified") never induces a failure; it asserts nothing.

Below that sit a set of real Majors: the "Retry Upgrade" button cannot dismiss the recovery screen, `run_startup_checks` is now dead code so the documented startup integrity gate no longer runs, `open_database()` still applies all nine migration scripts in-place on every single RPC call (bypassing the engine entirely), and the recovery screen re-implements diagnostics sanitization in TypeScript, more weakly than the Rust function that exists for it and is never called.

## Blockers

### 🔴 Released-migration immutability guard covers 1 of 9 migrations and reports success anyway, `migrations/released.json:1-9`, `crates/core/src/migrations.rs:1232-1273`

**Problem**: `released.json` contains exactly one entry (`0001_scaffold.sql`). `verify_released_migrations` iterates `manifest.released` and hashes only the files listed there, so `0002`–`0009` are never read, never hashed, and never checked. `verify_migration_0009_schema` only does four `content.contains(...)` substring checks, which any edit that preserves those four strings passes.

Separately, the `sha256` constants in the compiled catalog (`MIGRATIONS`, lines 62-117) are **never** compared against the SQL they sit next to. Nothing at build time, test time, or run time computes the hash of `db::MIGRATION_0005_SQL` and checks it against `"8f3a3e..."`. The values happen to be correct today (I recomputed all nine), but that is an accident of authorship, not an invariant. `run_migrations_with_staging` then writes those unverified constants into `_migrations.sha256` as though they attested to the applied SQL.

I confirmed the bypass by appending `DROP TABLE IF EXISTS payments;` to a copy of `0005_financial_ledger.sql` and calling `verify_released_migrations` against that tree:

```
GUARD PASSED despite tampered 0005_financial_ledger.sql
```

**Why it matters**: `AGENTS.md` states "Released migrations are immutable: never edit an already released migration file", `crates/core/AGENTS.md` names `migrations.rs` as owning "the guard that keeps them unmodified", and spec 0006 lists "Checksum consistency" as a key invariant covering *both* `released.json` and the compiled catalog. A green `check-migrations` is currently evidence of nothing for eight of nine files. Worse than no guard, because the team will trust it. An edit to a shipped migration changes what runs on databases that already applied the old version, and `INSERT OR IGNORE` on `_migrations` (line 970) means the stale recorded hash is silently kept, so drift is undetectable after the fact too.

**Suggested fix**: Populate `released.json` with all nine entries. Then close the catalog gap: add a test (or a `check-migrations` step) that recomputes the SHA-256 of each `MigrationStep::Sql` payload with the same `\r\n` → `\n` normalization and asserts it equals `def.sha256`, so the constant and the embedded file can never disagree. Consider making the swap fail loudly when a `_migrations` row exists for a version with a different `sha256` than the catalog, rather than `INSERT OR IGNORE`.

### 🔴 Crash recovery restores the wrong archive from version 10 onward (silent data loss), `crates/core/src/migrations.rs:796` and `:849-858`

**Problem**: `list_archives` ends with `archives.sort_by(|a, b| b.cmp(a))` — a lexicographic descending sort on the whole `PathBuf`. Archive names are `pre-migrate-v{version}-{timestamp}.db`, so the version number is compared as *text*. `"pre-migrate-v9-"` sorts above `"pre-migrate-v10-"` because `'9' > '1'`. `check_and_recover_interrupted_swap` then takes `archives.first()` as "the newest archive" and copies it over the missing active database.

Verified against the real function:

```
newest-first order as the engine sees it:
  pre-migrate-v9-20260101000000.db
  pre-migrate-v8-20250101000000.db
  pre-migrate-v10-20261231000000.db
archives.first() (used for crash restore) = pre-migrate-v9-20260101000000.db
```

**Why it matters**: This is the exact scenario the feature exists to survive — a power cut during the rename swap. The engine restores a database snapshot from an *earlier* upgrade, discarding every payment, charge and enrolment recorded in between, and it does so silently: the app boots normally afterwards. Spec AC-7 says "identifies the **newest** archive file". It is latent only until the tenth migration, which is the next one anyone writes. The same misordering also drives `RecoveryInfoResponse.archive_paths`, so the recovery screen presents the wrong file as the most recent backup.

**Suggested fix**: Parse the `{timestamp}` segment (it is fixed-width `%Y%m%d%H%M%S`, so it sorts correctly on its own) and sort on that descending, falling back to file mtime when the name does not parse. Do not sort on the version segment as text. Add a regression test with `v8`, `v9` and `v10` archives asserting `first()` is the v10 one.

### 🔴 The AC-4 rollback test never induces a failure and asserts nothing, `crates/core/tests/migration_engine.rs:158-195`

**Problem**: `test_ac4_transaction_rollback_preserves_active_db` builds a database at version 8, records its file length, and then — under the comment `// 2. Corrupt or induce failure in a staging migration: // Insert an invalid row directly in staging or simulate a broken transaction` — does nothing at all. It never calls `run_migrations_with_staging`, never registers a failing migration, and never creates the `staging_path` it declares on line 180. It then asserts that the version is still 8, that `staging.db` does not exist, and that the file size is unchanged. All three are trivially true of a database nobody touched. The test passes and always will.

**Why it matters**: AC-4 ("If any migration step fails, the transaction rolls back, `staging.db` is purged, and the live database file remains completely unmodified") is the single promise this whole feature is built to make, and it is the promise the recovery screen repeats to the user in green text. It has zero coverage. The neighbouring path — a migration that fails *after* earlier ones in the same run already committed to staging — is also untested; nothing verifies that a partial apply is abandoned wholesale rather than leaving staging half-migrated and promotable. `verify.md:13` and `verify.md:22` both claim this suite verifies AC-4.

**Suggested fix**: Give the test a real failure injection. The cleanest seam is to make the migration catalog injectable for tests (a `run_migrations_with_staging_using(&[MigrationDefinition])`, or a `MigrationStep::Data` step that returns `Err`) so a deliberately broken step can be appended after several good ones. Assert: the call returns `ExecutionFailed { version }` naming the right version, `staging.db` is gone, the active file's byte length and `inspect_schema_version` are unchanged, and the active database still opens cleanly. Add a sibling test where the *validation gate* rejects (not the SQL) and assert the same invariants.

## Major

### 🟠 A dropped ledger table passes the financial-invariant gate, `crates/core/src/migrations.rs:580-590`

**Problem**: The comparison closure inside `assert_matches` is `if let (Some(b), Some(a)) = (before, after)`. `capture` returns `None` for a table that does not exist (`table_exists`, line 439). So if a migration **drops** `payments`, `before` is `Some(...)`, `after` is `None`, the `if let` does not match, and the closure returns `Ok(())`. Total loss of a ledger table is the one thing this gate cannot see.

**Why it matters**: Spec 0006 lists "Financial conservation" as a key invariant and AC-5 exists to catch "silent data divergence". A gate that passes on table deletion inverts its own purpose, and it is the last check before the live database is replaced and the original archived. Chained with the immutability blocker above, an edited-in-place migration that drops a table would be applied, validated, swapped in, and only discoverable from a 30-day archive. (Side note: `expenses` also stores poisha and is not in the baseline at all, while spec AC-5 names the table `allocations` where the code correctly uses `payment_allocations` — worth reconciling the spec text.)

**Suggested fix**: Treat `(Some(_), None)` as a hard `ValidationFailed` ("ledger table `X` disappeared during migration"). `(None, Some(_))` — a table appearing — is fine and should stay passing.

### 🟠 "Retry Upgrade" cannot ever leave the recovery screen, `ui/src/App.tsx:42-53` and `:88-95`

**Problem**: Two compounding issues. First, `checkRecovery` only calls `setRecoveryInfo` **inside** `if (res.ok && res.data && res.data.is_in_recovery)`. It never clears the state, so once `recoveryInfo` is set, the `if (recoveryInfo && recoveryInfo.is_in_recovery)` guard at line 88 holds forever regardless of what the backend now reports. Second, and more fundamentally, `system.recovery_info` just reads the in-process `RECOVERY_STATE` (`migrations.rs:204-228`), which is only ever cleared by `run_startup_lifecycle` succeeding — and that runs once, in the Tauri `setup` hook. Nothing re-runs the migration. The `window.location.reload()` fallback in `migration-recovery-screen.tsx:90` would only reload the webview, not the Rust process, and it is dead anyway because `App.tsx` always supplies `onRetry`.

**Why it matters**: The primary call to action on the recovery screen does nothing and gives no feedback. An operator who frees up disk space after an `InsufficientStorage` failure — the most likely and most recoverable failure mode — clicks Retry, sees nothing change, and concludes the product is broken. The real fix (quit and relaunch) is never suggested.

**Suggested fix**: Either add a core command that re-runs `run_startup_lifecycle` and have Retry call it, or drop the button and tell the user to restart Coachify. Whichever way, make `checkRecovery` set state unconditionally (`setRecoveryInfo(res.ok ? res.data : null)`) so a resolved state actually dismisses the screen.

### 🟠 `run_startup_checks` is no longer called, so the documented startup integrity gate does not run, `src-tauri/src/lib.rs:31-38`, `crates/core/src/db.rs:92-111`

**Problem**: The Tauri `setup` hook now calls only `migrations::run_startup_lifecycle()`. `db::run_startup_checks()` has no production caller left (`grep` finds it only in `crates/core/tests/crash_recovery.rs`). `run_startup_lifecycle` does its own ad-hoc `verify_integrity` on success (line 1064-1070) but never sets the `STARTUP_INTEGRITY_OK` `OnceLock`, so the memoization that exists specifically to keep the full-file scan off the per-command path is never primed.

**Why it matters**: `src-tauri/AGENTS.md` states "The setup hook calls `coachify_core::db::run_startup_checks()`, so a corrupt database stops the app before any command runs", and `crates/core/AGENTS.md` states "The integrity quick check runs once per process through `run_startup_checks`, never on a per command path". Both are now false. Any future caller of `run_startup_checks` will re-scan the whole file. The replacement path is also weaker: `if let Ok(conn) = db::open_connection_at(&db_path)` silently swallows a connection-open failure and reports startup success.

**Suggested fix**: Have `run_startup_lifecycle` delegate its post-migration integrity check to `db::run_startup_checks()` so the `OnceLock` is set and there is one integrity gate rather than two. Propagate the `open_connection_at` error instead of discarding it. Update both AGENTS.md files if the call shape genuinely changes.

### 🟠 `open_database()` applies all nine migrations in-place on every RPC, bypassing the engine, `crates/core/src/db.rs:165-171`, `:217`

**Problem**: `open_database()` is called on the hot path of every dispatched command (`lib.rs:309`) and ends with `apply_all_migrations(&conn)`, which `execute_batch`es all nine migration scripts directly against the **live** database and then runs nine `INSERT OR IGNORE` statements into `_migrations`. No staging copy, no `migration.active.lock`, no pre-flight disk check, no validation gates, no downgrade check, no archive.

**Why it matters**: This is a direct contradiction of the spec's central invariant ("Zero modification on error: the active database file is never touched or modified until the staging copy has completely passed all migration steps and validation gates"). It is harmless-looking today only because every script is `CREATE ... IF NOT EXISTS` / `INSERT OR IGNORE` — but the moment migration `0010` contains an `ALTER TABLE` or a data backfill, the first RPC after a *failed or blocked* engine run will apply it straight to production with none of the protections this feature was built to provide. It also means the recovery-mode gate at `lib.rs:296` is the only thing standing between a failed migration and unguarded in-place DDL. As a secondary cost, executing nine SQL scripts plus nine inserts per command is meaningful per-request work on what is supposed to be a connection open.

**Suggested fix**: Remove `apply_all_migrations` from `open_database`. Schema arrival should be the engine's job alone. Keep `apply_*_migrations` as test helpers (the test suites use them heavily) but have the production path assert `inspect_schema_version(conn) == TARGET_SCHEMA_VERSION` and fail into recovery if not.

### 🟠 The recovery screen re-implements sanitization in TypeScript, weaker than the unused Rust version, `ui/src/components/system/migration-recovery-screen.tsx:48-54`, `crates/core/src/migrations.rs:1102-1189`

**Problem**: `export_sanitized_diagnostics` and `sanitize_error_string` exist in core, are tested (`test_ac8_diagnostics_sanitization`), and have **no production caller** — no RPC command exposes them. Instead the React component does its own redaction with a single regex: `/(password|token|secret|key|hash)=[^\s&,;]+/gi`. Compared with the Rust function this misses the `keyword: value` form entirely, and drops the `argon2`, `guardian`, `phone` and `student` keywords.

**Why it matters**: Two violations of project rules — `AGENTS.md` ("domain logic lives in `crates/core`, never in the UI") and `ui/AGENTS.md` ("No business logic in React components") — but the substantive harm is AC-8's promise that the export "strictly redacts ... personal student identity fields". A constraint-violation error mentioning `student: Rahim Uddin` is redacted by the Rust path and passed through verbatim by the path that actually runs. The exported bundle also carries `active_db_path` and every archive path unredacted, which embed the operator's Windows username. Related: `RecoveryInfoResponse.error_message` is the raw `MigrationError::to_string()`, so unsanitized SQLite text and absolute paths already reach the client before the component sees them.

**Suggested fix**: Sanitize in core. Either apply `sanitize_error_string` to `error_message` when building `RecoveryInfoResponse`, or add a `system.export_diagnostics` command returning the `export_sanitized_diagnostics` JSON, and have the component download exactly what core hands it. Delete the TypeScript regex. If neither, delete the dead Rust function so the next reader is not misled about which sanitizer runs.

### 🟠 Crash-swap recovery and staging purge run outside the process lock, `crates/core/src/migrations.rs:880` vs `:908`

**Problem**: `check_and_recover_interrupted_swap` is step 1; `MigrationLock::acquire` is step 4. Between them the function already renames `staging.db` → active, deletes `staging.db`, purges sidecars, and copies an archive over active — all unserialized. The branch at lines 859-863 (`active exists` → `fs::remove_file(&staging_path)`) will attempt to delete the staging file of a *concurrently running* migration in another process. Two instances launching together on the AC-7 path both call `rename_with_retry(&staging_path, active_path)`; one wins and the other burns 15 retries with backoff before returning `Io`, pushing a perfectly healthy installation into recovery mode.

**Why it matters**: Spec key invariant "Single writer lock: **only one** migration engine process may operate at a time, guaranteed by an operating system level file lock". Right now the lock covers the copy-migrate-swap phase but not the recovery phase, which is the phase that deletes and renames files. On Windows the open-file handle makes the cross-process delete fail rather than succeed, which softens this considerably — but the result is silently ignored (`let _ = fs::remove_file`) and the guarantee is not actually held by the lock.

**Suggested fix**: Acquire `MigrationLock` first, before `check_and_recover_interrupted_swap`, and hold it for the whole function. The lock file lives next to the database and its acquisition does not depend on any of the recovery work.

## Minor

### 🟡 Every `MigrationError` variant collapses to one client code, `crates/core/src/dispatcher.rs:832-848`

All seven variants map to `code: "migration_error"` with the raw `Display` string as `message`. `AGENTS.md` asks for `thiserror` errors mapped to `{ code, params, field }`. A client cannot distinguish `InsufficientStorage { available, required }` (user-fixable: free up disk) from `DowngradePrevented { current, target }` (user-fixable: update the app) from `Sql` (not user-fixable), and the structured fields those variants carry are thrown away into English prose. The recovery screen is consequently forced to render backend English, which also forecloses localization for a Bangladesh-facing product. Consider distinct codes per variant with the variant's fields as `params`.

### 🟡 The recovery screen asserts the live database is untouched even when it is not, `ui/src/components/system/migration-recovery-screen.tsx:124-129`

The green assurance panel ("the active database was never modified or replaced") is unconditional. If `rename_with_retry` fails on the *second* rename (`migrations.rs:1037`), the active file has already been renamed to the archive and is genuinely gone at that moment. The next launch repairs it via AC-7, but the screen shown in that window states the opposite of the truth. Gate the panel on the error variant, or reword it to describe what the engine attempts rather than asserting an outcome it cannot know.

### 🟡 `inspect_schema_version` swallows query errors as version 0, `crates/core/src/migrations.rs:670-673`, `:680-686`

Both `query_row(...).unwrap_or(None)` calls treat a genuine SQLite failure identically to an empty table. On a partially-damaged database this reports version 0 and the engine proceeds to apply all nine migrations to a database that is actually at 9. The current scripts are idempotent enough to survive it and the staging isolation limits the blast radius, but silently downgrading a *read failure* to "brand new database" on the decision input that drives the entire engine deserves an explicit error.

### 🟡 `get_migration_history` silently drops unmappable rows, `crates/core/src/migrations.rs:738`

`.filter_map(|r| r.ok())` discards any row that fails to map (a `NULL` `execution_time_ms`, a type surprise) without a trace. The admin sees a shorter history and no indication anything was hidden. Propagate the error or count the skips.

### 🟡 `system.open_db_directory` is unauthenticated and discards its error, `crates/core/src/dispatcher.rs:198-203`, `:856-864`

Registered as `AuthRequirement::Public` and handled in `lib.rs:57-71` ahead of the recovery gate, so any caller of the RPC bridge can spawn `explorer.exe`. It is a local desktop app and the surface is small, but the command is not in spec 0006's API table at all, and `OpenDbDirectoryResponse { success: res.is_ok() }` throws away the actual failure while the UI never reads `success` anyway. Consider gating it behind the recovery state or `settings.view`, and surfacing the error.

### 🟡 Archive pruning does not run on startup, only after a successful upgrade, `crates/core/src/migrations.rs:900-902`, `:1041`

`prune_old_archives` is the last statement of `run_migrations_with_staging`, but the function returns early at line 900 when `current_version == TARGET_SCHEMA_VERSION` — which is every normal boot. So archives are only pruned on the boot that performs an upgrade. AC-6 says "automatically pruned from the directory **on startup**". Also uses `meta.modified()` where the spec says creation timestamp, and will happily delete the last remaining archive.

### 🟡 Interrupted-swap promotion accepts any structurally-valid staging file, `crates/core/src/migrations.rs:829-841`

The validity test for promoting `staging.db` to active is `PRAGMA quick_check` only. That checks file structure, not whether the migration run that produced the file ever finished. A staging file left by a process killed mid-run passes and gets promoted. The engine does then continue and re-apply the remaining migrations (each migration commits atomically, so the file sits on a version boundary), so it self-heals — but the promotion criterion should say so deliberately. Consider also asserting `inspect_schema_version(staging) >= inspect_schema_version_of_newest_archive` before promoting.

### 🟡 Hand-rolled Win32 FFI with a misaligned `OVERLAPPED` and no safety comments, `crates/core/src/migrations.rs:259`, `:311`, `:261-270`, `:379-386`

`let mut overlapped = [0u8; 32];` is passed as `LPOVERLAPPED`. The size matches `OVERLAPPED` on x64 but `[u8; N]` has alignment 1, and the struct requires pointer alignment — the compiler gives no guarantee here. `LockFileEx`, `UnlockFileEx` and `GetDiskFreeSpaceExW` are also declared by hand rather than taken from `windows-sys`, and none of the three `unsafe` blocks carries a `// SAFETY:` comment (the `rust-best-practices` skill named in the spec asks for these). Use a properly-aligned zeroed struct or pull the declarations from `windows-sys`.

### 🟡 `sanitize_error_string` can panic on non-ASCII input, `crates/core/src/migrations.rs:1130-1189`

`find_ascii_case_insensitive` returns a byte offset from `as_bytes().windows(...)`, which need not land on a UTF-8 char boundary; `&s[search_from..]` and `replace_range(pos..val_end)` then panic if it does not. A Windows username or institute name in Bengali inside an error string containing `key` is enough. Currently unreachable because the function has no production caller — but that changes the moment the Major above is fixed.

### 🟡 `check_free_space` and `MigrationLock` are no-ops off Windows, `crates/core/src/migrations.rs:368-411`, `:324-340`

Outside `#[cfg(windows)]`, `check_free_space` returns `Ok(())` unconditionally and `MigrationLock::acquire` opens a file and returns `Ok` without taking any lock. The product is Windows-only so this is defensible, but it means AC-2 and AC-3 pass vacuously on a Linux CI runner, and `test_ac3_process_lock_exclusion` guards its only meaningful assertion behind `#[cfg(windows)]`. Worth a `compile_error!` or an explicit comment stating the platform assumption.

### 🟡 `test_startup_check_passes_and_repeats_cheaply` touches the developer's real database, `crates/core/tests/crash_recovery.rs:87-91`

It calls `db::run_startup_checks()` with no `LOCALAPPDATA` override, so it resolves and opens `%LOCALAPPDATA%\Coachify\dev\dev.db` on the machine running the suite, creating the directory if absent. Tests should not reach outside their tempdir. (Relatedly, `migration_engine.rs:446` calls `std::env::set_var("LOCALAPPDATA", ...)` in a binary whose other tests run concurrently — safe today only because no other test in that file resolves the dev path.)

## Nits

- ⚪ `crates/core/src/migrations.rs:765`, the trailing `Ok(())` in `rename_with_retry` is unreachable: the `attempt == 14` arm always returns.
- ⚪ `crates/core/src/migrations.rs:12`, `TARGET_SCHEMA_VERSION` is a hardcoded `9` while the spec's value sourcing says it derives from `MIGRATIONS.last().version`. A `const` assertion or a test tying the two together would stop them drifting.
- ⚪ `crates/core/src/migrations.rs:988-1008`, the `_migrations` backfill runs outside a transaction and stamps versions 1-8 with `execution_time_ms = 0` and a synthetic `applied_at`. Harmless (it is staging-only), but AC-10's execution-time field is fiction for those rows.
- ⚪ `crates/core/src/migrations.rs:1091`, `app_version: "0.1.0"` here vs `app_version: "1.0.0"` in `dispatcher.rs:827`. Two hardcoded versions disagreeing; both should read `env!("CARGO_PKG_VERSION")`.
- ⚪ `crates/core/src/migrations.rs:1097`, `free_disk_bytes` is always `0` despite the spec listing disk metrics as diagnostic content.
- ⚪ `crates/core/src/migrations.rs:768-779`, `purge_sidecars` derives sidecar names via `with_extension`, which is correct for `*.db` but produces `name.db-wal` for an extensionless path where SQLite would use `name-wal`. Fine for current call sites; brittle if a path ever lacks an extension.
- ⚪ `crates/core/src/migrations.rs:50-53`, `MigrationStep::Data` has zero callers, which brushes against the "no new abstraction without two callers" rule — though spec 0006 explicitly designs it, so this is the spec's call, not the author's.
- ⚪ `crates/core/tests/crash_recovery.rs`, the filename promises migration crash recovery but the contents are database integrity plus a plain transaction rollback. The `// covers: AC-1, AC-2` markers refer to a different spec's numbering than 0006's, which is confusing next to `migration_engine.rs`.
- ⚪ `crates/core/src/lib.rs:296`, the recovery-mode gate sits after the `setup.*` and `recovery.*` handlers, so `setup.initialize_institution` can still write to disk while the app is in migration recovery mode. Probably intentional given spec 0007, but worth a comment stating so.

## Strengths

- The core sequencing is genuinely right and is the hard part: checkpoint the active WAL with `TRUNCATE` **before** copying, purge sidecars, run every migration in its own `TransactionBehavior::Immediate` transaction on the staging file, and on any `Err` delete staging and return without ever having opened the active file for writing (lines 917-1023). The `run_res` closure-plus-cleanup shape makes the "live file untouched on error" property visible in the control flow rather than scattered across early returns.
- Capturing the financial baseline from *staging* rather than from the active database (line 937) is the correct choice and avoids a whole class of comparison races. The `TableBaseline` derive of `PartialEq` makes the comparison total over the captured columns rather than field-by-field and easy to extend.
- `test_ac7_interrupted_swap_crash_recovery` is a good test: it constructs both real crash topologies by hand (valid staging with no active; corrupt staging with an archive present) and asserts the outcome of each, rather than mocking the filesystem.
- `test_ac9_rpc_migration_commands_and_permissions` exercises the permission boundary end to end through the real dispatcher — bootstrap an Owner, create a Teacher, log in as each, assert `forbidden` for the unprivileged one — instead of unit-testing the permission table. That is the right altitude for an authorization test.
- `verify_released_migrations` normalizes `\r\n` → `\n` before hashing (line 1259), and `check-types` does the same for the contract comparison. On a Windows-first repo with a mixed-platform CI that detail is easy to miss and expensive to debug later.
- The recovery screen is well built as UI: `data-testid` hooks throughout, sensible empty-state handling for zero archives, `title` attributes on truncated paths, and a genuinely reassuring tone for an operator staring at a failed upgrade.

## Test coverage

**Covered well**: forward migration 0 → 9 and the no-op repeat call (AC-1); downgrade prevention with an injected version-99 row (AC-1); disk-space rejection plus the assertion that no staging file is created (AC-2); lock acquire / conflict / release-and-reacquire (AC-3, Windows only); the foreign-key gate and the poisha-divergence gate, both with real injected violations (AC-5); archive creation, naming and the archived database's schema version (AC-6); both interrupted-swap topologies (AC-7); `_migrations` UPDATE and DELETE trigger rejection with message assertions (AC-10); permission gating end to end (AC-9); tamper detection for `0001` (the one migration the manifest lists). All 13 tests pass, and `check-types` and `check-migrations` are green.

**Untested new logic**, roughly in order of risk:

- **Migration failure and rollback (AC-4)** — no coverage at all; the test that claims it is vacuous (see Blocker 3). Nothing exercises `MigrationError::ExecutionFailed`, nothing exercises a partial apply where migrations 1-4 commit and 5 fails, and nothing asserts staging is purged on failure.
- **Validation gate 1 (`PRAGMA integrity_check`)** — gates 2 and 3 have real rejection tests; the integrity gate only ever runs against clean databases, so its failure branch (lines 626-635) is unexecuted.
- **Checksum drift** — no test computes a migration's hash and compares it to the catalog constant, which is why Blocker 1 went unnoticed. Migrations `0002`-`0009` have no tamper test because `released.json` does not list them.
- **Archive ordering** — `list_archives` is only ever exercised with a single archive (`test_ac6`), which is why the version-10 sort bug is invisible. Any test with two archives at different versions would have caught it.
- **`run_startup_lifecycle`** — the production entry point has no test. Recovery-state set/clear, the `set_recovery_state` → `is_in_recovery` → RPC-gate chain, and the `failed_version` extraction per error variant are all uncovered, so the recovery screen's inputs are never verified against a real failure.
- **`check_and_recover_interrupted_swap` third branch** — active missing, staging absent, *and* no archives (line 865 falls through to `Ok(())`), leaving the caller to create a fresh empty database. Untested, and arguably should be an error rather than a silent new database.
- **UI** — `migration-recovery-screen.test.tsx` uses `renderToString` and only asserts substrings in static markup. Nothing clicks anything, so `handleExportDiagnostics` (including its sanitization regex), `handleOpenFolder` and `handleRetry` have zero behavioural coverage — the `verify.md:16` claim that `pnpm test` verifies "the diagnostics export button and explorer trigger" does not hold. Switching to `@testing-library/react` with `userEvent` and asserting on the produced bundle would cover the sanitization gap in Major 5, and an `App.tsx` test would have caught the stuck-screen bug in Major 2.
