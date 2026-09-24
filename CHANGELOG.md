# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Keyboard first batch attendance sheet with single key entry (P, A, L, E), sub 300 ms save benchmark for 80 students, midnight session locking, and monthly attendance matrix (spec 0023).
- Offline first durable omnichannel outbox written in the same database transaction as domain actions, supporting cellular SMS, WhatsApp pairing via OpenWA, and SMTP email with quiet hours and timeout protection (spec 0024).
- Bilingual English and Bengali message templates with live GSM 7 and UCS 2 Unicode segment math, automatic payment receipt triggers, delayed absence alerts with cancellation buffer, and period deduplicated fee reminders (spec 0025).
- Examination management hub with spreadsheet style mark entry grid, NCTB five point GPA calculation, optional fourth subject bonus math, and fail rules for mandatory subjects (spec 0026).
- Results calculation and immutable snapshot publishing with five tier deterministic tie breaking, bilingual A4 student progress report cards, and fixed width landscape batch tabulation sheets (spec 0027).
- AI Foundation and Copilot Shell (Features 54–57, spec 0036) with zero-SQL access, 6 controlled read-only tools clamped to 50 rows, bounded multi-hop execution, client-side PII and 11-digit Bangladeshi mobile scrubbing, ephemeral in-core re-hydration, institutional credit ledger wallet with 1-turn/1-credit invariant, OpenRouter LLM gateway, and persistent audit logging.
- AI Copilot slide-over drawer accessible globally via Ctrl+J or TopBar icon with live credit pill, Shield Active protection badge, suggested questions, and tool execution chips.
- Institute AI & Privacy Settings card with 6 granular privacy toggles, credit wallet ledger table, recharge grant modal, BYOK gateway configuration, and tool execution audit trail.

### Fixed
- Recorded the acting user ID in the SHA 256 audit log whenever an administrator performs a retroactive edit on a locked attendance session.
- Allowed clearing saved gateway credentials by submitting an empty password, setting secret reference to null in the database.

### Security
- Gated attendance session saving and locking strictly behind the `attendance.mark` permission, preventing users with read only `attendance.view` access from altering or locking student attendance.
