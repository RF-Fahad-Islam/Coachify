# 0032. Operational Integrity: Rationale & Decision Record

> Decision record and rationale for Spec 0032 (Slice 8 Operational Integrity Umbrella)

## Context

Coaching centers in Bangladesh operate in environments characterized by intermittent broadband, frequent grid power outages, shared counter computers, and high daily transaction volumes during peak admission and monthly fee collection cycles. Front desk counters handle high velocity cash and digital mobile financial service transactions (such as bKash and Nagad). Under these challenging conditions, operational software faces severe reliability risks.

First, paper fee vouchers and money receipts can be forged or altered by bad actors if receipts lack offline verifiable security markers. Guarding counter integrity requires receipts to be independently authenticatable even when reception staff or guardians have zero internet access.

Second, cashiers working rapidly during morning rushes are prone to accidental double billing, such as submitting the same payment twice when network latency causes UI hesitation, or re entering an already recorded bKash transaction reference.

Third, communications pipelines are vulnerable to connectivity drops. When internet disconnects while teachers are publishing marks or marking attendance, SMS notices must never vanish into the void. They must queue safely on disk and resume sending automatically once service restores.

Fourth, data safety requires more than passive snapshotting. Hardware failures on budget PCs can lead to catastrophic data loss if backups are corrupted or never tested. Center owners need proactive recovery health scores, weekly automated restore drills, and a simple mechanism to migrate the entire system to a replacement computer when hardware degrades.

Finally, database integrity must be continuously validated. Corrupt records or broken foreign keys must be caught on startup in under three seconds before bad writes compound.

## Options considered

### Option 1: Unified Operational Integrity Engine with Modular Child Specs

Establish a unified core integrity subsystem inside the application core with modular child specifications for finance safety, communication outbox resilience, resilience monitoring, disaster recovery drills, and machine migration. All verification, outbox queueing, and integrity testing runs directly against the local SQLite database and Windows DPAPI security context.

**Pros**:
* Single cohesive domain module with shared transaction and audit log conventions.
* Completely local and offline capable with zero external server dependencies.
* Clear child specifications allow focused development and incremental testing across features 40 through 49.
* Full compatibility with existing Coachify SQLite WAL mode and command dispatcher architecture.

**Cons**:
* Requires coordinating multiple related features across core, migrations, and UI.

### Option 2: Fragmented Feature Specific Logic Across Existing Modules

Implement each of the ten features inside their respective existing modules without a unified integrity domain. Receipt verification in ledger, duplicate checks in payments, outbox changes in comms, and recovery drills in backup.

**Pros**:
* Places each individual function in existing source files without creating a new integrity module.

**Cons**:
* Creates fragmented cross cutting logic across ledger, backup, comms, and system.
* Hard to compute unified recovery scores and health dashboards when metrics are scattered across four different subsystems.
* Duplicate validation logic and inconsistent error reporting across counter workflows.

### Option 3: Vendor Cloud Managed Operational Safety

Offload verification, outbox management, and integrity monitoring to a cloud based server, relying on cloud synchronisation to detect duplicates and verify receipts.

**Pros**:
* Offloads computation from budget coaching center computers.

**Cons**:
* Violates the foundational Coachify architectural rule that institution data lives locally.
* Fails completely during internet outages, leaving cashiers unable to verify receipts or queue notices.
* Adds recurring hosting costs and cloud dependency for features that should work offline.

## Rationale

Option 1 is chosen. It directly honors the Coachify offline first desktop architecture, where all coaching center data, payments, and communication outbox state remain strictly on the local machine. By organizing the integrity subsystem into focused child specifications under a shared umbrella, we maintain clean separation of concerns while delivering an integrated operational health and safety shield. Compact HMAC SHA256 signatures enable instant offline receipt verification using local machine keys. The outbox database queue ensures complete durability through power cuts, while automated restore drills and DPAPI key rewrapping deliver real disaster recovery confidence for non technical institution owners.
