# Rationale: 0036 Release Hardening

## Context

Coaching centers in Bangladesh operate under demanding physical conditions. Electrical load shedding causes sudden desktop computer power cuts multiple times each day, often without an uninterruptible power supply (UPS) connected. Staff frequently use older office computers running Windows 10 with limited storage, shared local accounts, and varied thermal receipt and laser printers. Under these real world constraints, an unexpected computer shutdown during payment processing, database migration, or daily backup creation could cause database corruption, lost audit records, or duplicate student billing if not rigorously hardened.

Furthermore, coaching center receptionists and cashier staff rarely possess advanced technical training. When an operating system antivirus program flags an unsigned executable, or when a thermal printer disconnects, staff require clear offline guidance immediately accessible from within the application. Support specialists also need clear standardized runbooks to guide owners through disaster recovery drills, lost recovery sheets, and hardware swaps without guessing.

Finally, commercial release readiness requires objective verification. Selling software to local institutions without proven multi week field trials risks reputation damage and costly on site support emergencies. Version 1.0 must be qualified through automated crash testing, comprehensive customer documentation, and verified pilot deployments across active institutions.

## Options considered

### Option 1: Modular Release Hardening with Headless xtask Harness and Bundled Documentation (Recommended)

This approach implements a dedicated cargo xtask crash runner (`cargo run -p xtask crash-suite`) that spawns child processes and issues Windows `TerminateProcess` calls at deterministic transaction checkpoints behind a compile time `fault-injection` feature. It pairs this with offline markdown customer guides bundled directly into the user interface via an in app Help dialog, and tracks 1.0 release readiness across three pilot centers in a structured markdown audit report. Low end reference PC performance benchmarking (Feature 59) is deferred to the 1.1 release milestone to maintain velocity and focus on core stability.

**Pros**:
* Real Windows process termination accurately reproduces sudden hardware power cuts.
* Test harness remains completely decoupled from production binaries via compile time feature gating.
* Bundled offline documentation ensures immediate staff assistance even during internet outages.
* Deferring reference hardware benchmarks avoids lengthy synthetic dataset generation while prioritizing data safety.

**Cons**:
* Running the full crash suite takes several minutes on local developer machines.
* Bundled markdown assets add approximately 1 to 2 megabytes to the desktop build.

### Option 2: Full Scope Release Hardening Including 4 GB Reference Hardware Benchmarking

This option retains all four features of Slice 11, including Feature 59 (Performance on the reference PC). It builds synthetic data generators for 10,000 active students and 6 million attendance marks, running automated benchmark sweeps against 4 GB RAM and spinning hard disk hardware budgets.

**Pros**:
* Verifies performance bounds on older hardware before any commercial customer installation.
* Establishes automated baseline metrics for memory usage and query latencies.

**Cons**:
* Generating millions of synthetic records and running spinning disk simulation requires significant build time and tooling overhead.
* The user explicitly requested excluding the 4 GB performance benchmarks and heavy synthetic test suite from 1.0.

### Option 3: Pure Integration Testing Without Child Process Termination

This option verifies crash resilience through standard in memory SQLite rollback tests and unit assertions, without launching separate child processes or invoking operating system process termination APIs. Documentation is published as external online pages only.

**Pros**:
* Executes very quickly within standard `cargo test` runs.
* Requires no platform specific process management code in xtask.

**Cons**:
* In memory tests fail to test SQLite write ahead log recovery, OS file locks, and actual process crash behavior.
* Online only documentation leaves coaching center staff stranded when internet connections fail.

## Rationale

We chose Option 1 because real world power cuts on Windows cannot be simulated accurately through standard unit tests alone. When a computer loses power, the operating system halts instantly without running process exit hooks or flushing user space buffers. Spawning actual worker processes and terminating them via Windows `TerminateProcess` provides authentic proof that SQLite write ahead logging and immediate transactions preserve ledger integrity.

Limiting the crash suite to 20 payment kills, 10 backup kills, and 10 migration kills strikes the right balance between exhaustive verification and fast developer iteration, removing the heavy overhead of the original 200 kill requirement.

Excluding Feature 59 aligns directly with user instructions, recognizing that memory and query performance in Coachify have already been optimized with pagination and indexed queries across previous slices. Bundling offline customer documentation into an in app Help dialog ensures that non technical staff can solve daily operational hurdles without waiting for remote assistance.

## References

**Project sources**:
* `AGENTS.md`: Stack conventions, pure Rust domain logic in crates/core, and Tracer Bullet build approach.
* `crates/core/AGENTS.md`: Database access conventions, write ahead logging mode with immediate transactions, and synchronous FULL pragmas.
* `docs/architecture-review.md`: Section 16 (Testing matrix and failure behaviors) and Section 17 (1.0 release acceptance criteria).
* `docs/scope/release.md`: Slice 11 epic definition and milestone acceptance requirements.
* `docs/specs/0032-operational-integrity/index.md`: Prior integrity patterns, offline resilience, and restore drills.

**Practices & standards**:
* SQLite WAL mode durability (basis: SQLite documentation on crash proof transactions and PRAGMA synchronous FULL).
* Windows process management (basis: Microsoft Win32 TerminateProcess API for ungraceful process abort testing).
* NIST Role Based Access Control (basis: dispatcher permission verification for administrative diagnostic endpoints).
* Tiered incident management runbooks (basis: ITIL operational support best practices for service desk escalation).
