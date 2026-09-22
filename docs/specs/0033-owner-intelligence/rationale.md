# 0033. Owner intelligence (decision record)

## Context

Coaching center owners in Bangladesh balance demanding operational days. They oversee student admissions, schedule classes across crowded rooms, track teacher payments, recover monthly tuition dues, monitor daily counter collections, and resolve parent complaints. In typical centers, operational data is fragmented across separate physical registers or disconnected spreadsheet tabs. When an owner wants to know whether a batch is financially viable or which students have chronic dues, they must spend hours manually cross referencing records.

Coachify already records rich primary data across prior slices: students, batches, admissions, fee plans, ledger charges, payments, attendance sessions, exam marks, SMS notifications, and audit logs. However, without dedicated intelligence features, the owner must navigate between multiple screens to synthesize what happened today or evaluate the overall health of a batch.

Furthermore, Coachify operates as an offline first desktop application installed on standard Windows PCs at local centers. The system has no cloud server dependency for day to day operations. Any analytical solution must execute directly against the local SQLite database without introducing memory heavy background daemons, complex caching layers, or significant write latency.

## Options considered

### Option 1: On demand unified query view joining existing domain tables

In this approach, timeline events and operational metrics are synthesized at query time by joining primary domain tables (`payments`, `admissions`, `attendance_records`, `marks`, `sms_outbox`, `audit_log`) and filtering them using indexed date and entity columns. New lightweight tables are added only for data that does not already exist in the system (batch cost parameters, staff student notes, and collection contact logs).

**Pros**:
* Single source of truth with zero data duplication.
* Absolute consistency: any payment, attendance record, or exam result reflects in intelligence views immediately.
* Zero write penalty on counter transactions and attendance marking.
* Simple maintenance with no synchronization jobs or cache invalidation logic.

**Cons**:
* Analytical queries must be carefully indexed to maintain sub second execution times on larger databases.
* More complex SQL queries with multiple union clauses.

### Option 2: Materialized intelligence events table maintained by SQLite triggers

In this design, every business event across all domains triggers an insert into a dedicated `intelligence_events` table via SQLite database triggers or core dispatcher hooks. Intelligence queries read directly from this single flat table.

**Pros**:
* Simple query logic on a single chronological table.
* Very fast read latency on event timeline queries.

**Cons**:
* Significant write amplification on every payment, attendance mark, and SMS record.
* Triggers add substantial schema complexity and make database migrations fragile.
* Risk of synchronization drift if an update or correction bypasses the trigger or requires historical backfill.

### Option 3: Event sourcing or background analytical daemon

This approach adopts full event sourcing or runs a local background worker process that periodically computes and updates analytical aggregates in memory or in cache tables.

**Pros**:
* Decouples analytics processing completely from user operations.
* Enables asynchronous complex computations.

**Cons**:
* Violates Coachify's architecture principle of keeping the system simple and robust on local PCs.
* Background workers consume CPU and memory continuously on modest coaching center hardware.
* Introduces eventual consistency where the owner sees stale counters after recording a transaction.

## Rationale

We selected Option 1: On demand unified query view joining existing domain tables, paired with Migration 0017 for operational configurations and contact tracking.

This choice aligns directly with Coachify's core design values: simple beats clever, and boring technology is a feature. SQLite handles indexed aggregations and multi table unions exceptionally fast on local NVMe and SSD drives. Testing in Slice 6 proved that SQLite executes complex multi table searches and reporting aggregations in under 50 ms on datasets with 10,000 students and 50,000 transactions. By querying primary tables directly, we eliminate the danger of stale data, avoid write amplification during busy counter hours, and prevent migration fragility.

Lightweight dedicated tables in Migration 0017 (`batch_cost_configs`, `student_timeline_notes`, and `due_collection_contact_logs`) provide the missing operational dimensions without polluting financial ledger tables or student registries.
