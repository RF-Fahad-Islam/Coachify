# 0011 Rationale. Demo and load data generator

**Date**: 2026-09-22
**Status**: Proposed

## Context

Coachify is engineered as a standalone, offline first desktop application designed to run on standard office hardware in Bangladesh. Institutional decision makers, including coaching center proprietors, headmasters, and administrative directors, expect a realistic demonstration of the software before committing to purchasing a license. Presenting an empty system with zero records forces sales representatives to manually type sample students, create synthetic courses, and enter fake fee collections during sales calls. This manual process is slow, tedious, and fails to showcase the rich reporting, automated due statements, and lightning fast search capabilities of the system.

Simultaneously, Section 16 of the architecture review mandates strict performance targets on a modest reference personal computer (a fourth generation Intel Core i3 or similar processor, 4 gigabytes of memory, a mechanical spinning hard drive, and Windows 10). The defined reference load dataset requires 10,000 active students, 15,000 archived students, 60,000 payments, and multi year historical records. To verify that student searches return results in under 150 milliseconds (and under 50 milliseconds on trigram lookups), that cashier views open within 300 milliseconds, and that database operations never exceed the 400 megabyte total application memory ceiling, the engineering team requires a high speed, reproducible data generation tool.

Allowing arbitrary test data generation in an operational desktop application introduces critical risks:
1. An operator or developer could accidentally target the active production database, permanently corrupting or overwriting real student admissions and financial receipts.
2. Inconsistent pseudo random generators can produce varying data distributions across operating systems, preventing reproducible performance benchmarks.
3. Unoptimized bulk inserts of tens of thousands of records can lock database connections, cause write ahead log bloating, or exhaust memory on low specification computers.
4. Generically generated data (such as "Student 1" with generic phone numbers) looks artificial during sales demonstrations and fails to validate realistic Bengali script searching and mobile phone normalization.
5. In memory client side caches (such as TanStack Query in React) could retain stale production data or leak demo records into production sessions when databases are switched dynamically.

To solve these requirements safely, Coachify requires a dedicated dataset generator capable of producing both a 500 student sales demo and a 25,000 student hardware stress dataset with authentic demographics, deterministic repeatability, strict path sandboxing, and complete cache invalidation.

## Options considered

### Option 1: Pure Rust Headless Generator in Core with Deterministic PRNG (`ChaCha8Rng`), Embedded Bilingual Dictionaries, Chunked Bulk Transactions, Path Sandboxing, and Dynamic Database Switching (Recommended)

The generator logic is implemented directly in the core Rust library (`crates/core/src/generator.rs`). Random generation uses `ChaCha8Rng` with an explicit 64 bit seed: headless CLI runs default to fixed seed `42` for deterministic benchmark comparisons, while desktop UI demo creation defaults to system entropy so each click generates a fresh, varied cohort. Authentic Bengali student names, guardian names, addresses, and realistic phone prefixes are stored in static embedded string tables. Sibling relationships model shared family surnames and shared guardians across separate age cohorts. The generator writes exclusively to temporary files in a sandboxed `<data_root>/demo/` directory using chunked transactions of 1,000 rows with prepared statements, running with `PRAGMA synchronous = OFF` and `PRAGMA journal_mode = OFF` during write phase to maximize speed on spinning disks before performing integrity checks, transitioning to WAL mode, and atomically renaming. A command line runner in `crates/xtask` provides headless benchmark generation, while an in app developer dialog allows instant demonstration database switching with complete TanStack Query cache invalidation and clean return to production.

**Pros**:
- Pure Rust execution guarantees cross platform memory safety, rapid compilation, and zero external runtime dependencies on client machines.
- Deterministic seeding ensures that automated performance tests produce identical records and checksums every time, while UI entropy gives sales teams fresh demonstration data on demand.
- Embedded bilingual dictionaries produce authentic Bangladeshi names and phone numbers, validating both English and Bengali FTS5 search queries.
- Strict path sandboxing, canonicalization, and temporary file creation guarantee that a production database can never be overwritten by synthetic data.
- Chunked bulk transactions with temporary bulk PRAGMAs allow generating 25,000 students and multi year histories in under 60 seconds even on spinning hard drives.
- Dynamic connection pool switching with TanStack Query cache clearing prevents memory state pollution between demo and production databases.
- Application restarts always default back to the real production database, preventing accidental operational work in demo mode.

**Cons**:
- Embedded name dictionaries increase core compiled binary size by approximately 40 kilobytes.
- Requires maintenance when new database tables or schema migrations are introduced in future development slices.

### Option 2: Pre Generated Static SQL Dump Fixtures

The repository stores pre generated `.sql` database dumps or compressed SQLite files that are unpacked into the target directory when demo mode is requested.

**Pros**:
- Near instantaneous setup time since no synthetic generation calculations are executed at runtime.
- Zero extra random number generation code needed in the application binary.

**Cons**:
- Massive repository bloat: storing compressed SQLite files or SQL dumps for 25,000 students and 60,000 payments would add tens of megabytes of binary data to git history.
- Extremely brittle: every schema migration requires rebuilding and re committing large binary database dump files.
- Zero flexibility: operators cannot adjust student counts, dates, or random seeds for custom benchmark profiles.

### Option 3: External Script in Python or Node.js

A sidecar script written in Python (using libraries like Faker) or JavaScript executes outside the core application to populate the database.

**Pros**:
- Leverages existing third party mock data libraries.
- Keeps data generation logic out of the Rust core crate.

**Cons**:
- Heavy operational dependency: target Windows office computers in Bangladesh do not have Python or Node.js runtimes installed.
- Cannot easily invoke internal core logic, such as phone normalization, password hashing, and cryptographic audit hashing.
- High risk of schema drift between external scripts and SQLite table definitions.

## Rationale

Option 1 is selected because it directly satisfies both the sales demonstration requirements and the automated hardware benchmarking targets while maintaining Coachify's offline first architectural integrity.

First, performance and determinism are paramount. By utilizing `ChaCha8Rng`, automated test runners and benchmarks on spinning hard disks produce bit for bit identical data distributions, eliminating variance in query latency measurements. Generating 25,000 student records and multi year histories in pure Rust with prepared statements in under 60 seconds achieves the high throughput needed for continuous integration and release qualification without bloating git repositories with static dump fixtures.

Second, cultural authenticity directly impacts sales conversion. Demonstrating software to coaching center proprietors with genuine names like "Rahat Chowdhury" or "তানভির আহমেদ", genuine neighborhoods like Mirpur and Farmgate, and authentic phone prefixes like `017` and `018` builds immediate trust. It also exercises the SQLite FTS5 trigram search index under realistic linguistic conditions, ensuring that both English and Bengali substring lookups perform smoothly.

Third, safety is non negotiable. By strictly prohibiting writes to the active production database path, sandboxing all target files inside the demo directory, enforcing canonical path resolution, clearing client side caches on database switches, and defaulting to production on application restart, Coachify eliminates any possibility of accidental data loss during staff training or sales demonstrations.
