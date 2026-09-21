# 0007. Setup wizard and key management: Decision Record

## Context

Coachify is deployed as a standalone, offline-first Windows desktop application in coaching centers and educational institutions across Bangladesh. The operational environment at these customer sites presents specific, high-risk security and resilience challenges:

1. **Catastrophic Hardware Failures and Re-imaging**:
   Low-cost office PCs in coaching centers frequently suffer motherboard, power supply, or hard drive failures. Furthermore, when Windows slows down or contracts malware, local technicians routinely reformat the `C:\` system drive without backing up application files located under `C:\ProgramData` or user profile folders. If an application stores its sole database on the `C:\` drive, the coaching center's entire operational and financial history is permanently lost during routine maintenance.

2. **Shared Office PCs**:
   In most coaching centers, multiple staff members (cashiers, teachers, receptionists) share a single Windows user account that has no Windows password or runs under generic auto-login. If database encryption required typing a master database password every time the application launched, either the Owner's master password would be compromised to every counter clerk, or automated background services and casual restarts would constantly fail.

3. **Data Theft and Confidentiality**:
   Institution records contain sensitive personal data (student contact information, guardian phone numbers, academic marksheets) and confidential financial ledgers (tuition fee collections, discounts, daily cash totals). If an unencrypted SQLite database is copied to a USB stick by an employee or stolen during a burglary, the entire institute's records are exposed.

4. **The Disaster Recovery Paradox**:
   If a database is encrypted using machine-bound keys (such as Windows DPAPI tied to a specific motherboard/installation), when the computer hardware inevitably dies, backups copied to a new machine cannot be decrypted. Without a designed recovery mechanism, machine-bound encryption turns a hardware failure into permanent, fatal data loss.

Coachify requires an architecture that combines effortless daily operation on shared computers, strong at-rest encryption against unauthorized file copying, proactive protection against `C:\` drive reformats, and foolproof physical disaster recovery when hardware dies.

## Options considered

### Option 1: Two-Tier Key Envelope with Machine-Scope DPAPI Wrapping, Argon2id/AES-GCM Recovery Key Wrapping, Drive Recommendation, and Mandatory Printed Recovery Sheet (Chosen)

Under this option, Coachify creates a random 256-bit cryptographic master database key upon initial setup. The key is stored in a structured key envelope (`key.envelope`) with two independent cryptographic wrapping layers:
1. **Machine-Scope Windows DPAPI**: The master key is encrypted using `CryptProtectData` with `CRYPTPROTECT_LOCAL_MACHINE`. This allows any Windows user on that specific PC to launch the application and run daily operations without entering a master database password, while ensuring that copying the files to any other PC leaves the database completely unreadable.
2. **Owner Recovery Key Envelope**: The master key is also encrypted using authenticated AES-256-GCM, with the key-encryption key derived from a 24-character human-readable Owner Recovery Key (`CKREC-...`) using memory-hard Argon2id.

During initial setup, the wizard detects mounted volumes and explicitly recommends storing the database on a non-system drive (like `D:\Coachify Data`) to survive Windows reformatting. The wizard displays a printable **Recovery Sheet** complete with the recovery key, emergency instructions, and a scannable QR code. The wizard enforces a mandatory confirmation gate requiring the Owner to type the last 4 characters of the key before proceeding.

If the PC dies or the database is restored on a new machine, Coachify enters Emergency Recovery Mode, prompts for the Owner Recovery Key, unwraps the master key, re-wraps it with the new machine's DPAPI, and unlocks the database. The recovery key can also be used to reset a forgotten Owner password.

**Pros**:
- **Effortless Daily Use**: Counter staff and background tasks launch and run Coachify without needing the Owner's master password.
- **Portability Protection**: Copying the database or backups to an unauthorized machine fails; records cannot be viewed without the physical Recovery Sheet.
- **Reformat Resilience**: Recommending a non-system drive (`D:\`) shields coaching centers from the most prevalent cause of data loss in Bangladesh offices.
- **True Disaster Recovery**: When computer hardware burns out, the owner uses the printed Recovery Sheet to restore data onto any replacement PC without vendor assistance.
- **No Vendor Escrow / Zero Knowledge**: The vendor holds no keys, eliminating cloud breach risks and government surveillance vectors.
- **Credential Recovery**: If the institute owner forgets their login password, the physical recovery key unlocks the account locally.

**Cons**:
- If an owner loses *both* the computer hardware *and* their printed physical Recovery Sheet, data is permanently unrecoverable. This tradeoff is transparently communicated on the sheet itself.
- Setup requires 2 to 3 minutes of intentional onboarding to print and verify the sheet.

### Option 2: Single Password-Derived Key (Owner Password as Database Key)

Under this option, the master database key is derived directly from the Owner's login password using Argon2id. Whenever Coachify starts, the user must enter the Owner password to unlock SQLite.

**Pros**:
- Simple mental model for technical users; no separate recovery sheet or key envelope file needed.

**Cons**:
- **Fails on Shared PCs**: Receptionists, cashiers, and teachers cannot open the application without knowing the Owner's password, destroying administrative accountability.
- **Breaks Background Jobs**: Nightly backups and automated SMS runners cannot operate unless the Owner remains logged in.
- **Key Rotation Disaster**: Changing the Owner password requires decrypting and re-encrypting the entire SQLite database file, risking corruption during power outages.
- **Total Loss on Forgotten Password**: Forgetting the Owner password permanently destroys the entire institution database with zero recovery recourse.

### Option 3: Cloud Key Escrow / Vendor Recovery Service

Under this option, during setup, Coachify transmits a copy of the database key or recovery key to the vendor's cloud server. If the customer loses their PC or credentials, vendor support verifies ownership via SMS or phone and remotely unlocks or reissues the database key.

**Pros**:
- Support staff can rescue customers who lose their printed Recovery Sheets.

**Cons**:
- **Destroys the Standalone / Offline-First Guarantee**: Setup cannot complete without an active internet connection, violating Section 1 of the architecture review.
- **High-Value Cloud Target**: A centralized database of customer encryption keys creates an irresistible target for hackers and legal subpoenas.
- **Violates Architecture Decision 5**: Formally rejected in Section 20 Decision 5: *"Printed Recovery Sheet only, no vendor escrow. The vendor can never open customer data."*

## Rationale

Coachify adopts Option 1 because it directly addresses the realities of Bangladesh coaching centers without compromising security or architectural independence.

In coaching centers, Windows PCs are communal tools. Receptionists and cashiers operate the counter, while owners oversee operations and collect daily cash reports. Forcing daily password entry for database decryption would lead staff to write passwords on sticky notes attached to monitors, completely defeating security. Machine-scope DPAPI provides transparent convenience for authorized staff on the office PC while rendering stolen database files completely useless on foreign machines.

The physical Recovery Sheet solves the critical limitation of machine-bound encryption: hardware mortality. Computer motherboards in dusty, non-air-conditioned coaching centers fail regularly. By formatting the recovery key with Crockford Base32, distinct 4-character chunks, a checksum, and a high-density QR code, owners can store the sheet alongside their trade licenses and financial deeds in a physical safe.

Finally, proactive non-system drive detection (`D:\Coachify Data`) provides defensive engineering against common local IT practices. When Windows becomes sluggish, local repair technicians habitually format `C:\` and reinstall Windows. Guiding the user to place data on `D:\` from day one prevents catastrophic data loss before it can happen.
