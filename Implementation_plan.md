You are the lead product engineer, desktop software architect, QA engineer, security engineer, database engineer, UX designer and release engineer responsible for building a complete commercial product called:

# NF Management Local

A modern, offline-first coaching-center and educational-institute management application for Windows.
Should use customizable themes, modern UI components, and a local-first architecture.
This must NOT be a prototype, mock dashboard, CRUD demo, hackathon app or partial proof of concept.

Build a complete product that can be:

* installed on a customer's Windows PC
* configured without technical knowledge
* operated without internet
* used daily by real coaching-center staff
* used by multiple computers over a local LAN
* backed up and restored safely
* licensed commercially
* printed from
* upgraded between versions
* tested automatically
* packaged into a real installer
* handed to paying customers

The target market is primarily coaching centers, academies and small-to-medium educational institutions in Bangladesh.

The software should feel like a polished modern SaaS application, but all core institution operations must work locally without depending on cloud services.

---

# 1. CORE PRODUCT PROMISE

The product promise is:

“Run your coaching center from one modern system—even when the internet is completely unavailable.”

Core areas:

Admission
→ Students
→ Guardians
→ Courses
→ Batches
→ Teachers
→ Attendance
→ Fees
→ Payments
→ Expenses
→ Exams
→ Results
→ Communication
→ Reports
→ Backups

Internet must NOT be required for:

* login
* viewing students
* admission
* attendance
* fee collection
* receipts
* expenses
* exams
* marks
* results
* reports
* document storage
* printing
* backup
* administration

Internet should only be required for optional external services such as:

* SMS APIs
* email delivery
* online license renewal
* application update checking
* optional future NF Cloud synchronization

When the internet is unavailable, these services must queue operations safely instead of blocking the application.

---

# 2. TECH STACK

Use a local-first architecture.

Preferred technology:

Frontend:

* React
* TypeScript
* Vite
* Tailwind CSS
* high-quality reusable component system
* TanStack Query where useful
* React Hook Form
* Zod

Desktop:

* Tauri 2

Native/backend layer:

* Rust

Database:

* SQLite
* WAL mode where appropriate
* SQLCipher stable production release for database encryption

Search:

* SQLite FTS5

Files:

* local filesystem
* securely organized application-data directories

Printing:

* native/system printer integration where possible
* print-ready HTML/PDF documents

Installer:

* professional Windows installer generated through Tauri

Do NOT use:

* Supabase
* Firebase
* Appwrite
* remote PostgreSQL
* Vercel
* Keycloak
* mandatory cloud APIs

for the local edition.

The local application must remain independent.

---

# 3. TWO OPERATION MODES

Support two modes from the same product.

## Mode A — Single Computer

One PC contains:

* application
* database
* documents
* backups
* users
* settings

Architecture:

Desktop App
→ Local Rust backend
→ SQLite
→ Local Filesystem

This should be the easiest installation.

---

## Mode B — Local Network / Multi-PC

One computer becomes:

NF Local Host

It owns:

* authoritative database
* files
* local API
* backup engine
* message queue
* licensing state

Other computers become workstations.

Example:

Main Office PC
→ NF Local Host

Reception PC
→ LAN
→ NF Local Host

Accounts PC
→ LAN
→ NF Local Host

Director PC
→ LAN
→ NF Local Host

Teacher PC
→ LAN
→ NF Local Host

Internet is still unnecessary.

Never place the SQLite database itself on a Windows shared/network folder.

Only the host process accesses SQLite directly.

LAN clients communicate through the authenticated local API.

---

# 4. LAN DISCOVERY & PAIRING

Make local networking easy for nontechnical customers.

Host setup:

Settings
→ Local Network
→ Enable NF Local Host

Show:

Host Name:
OFFICE-PC

Local Address:
192.168.x.x

Status:
Running

Connected Devices:
3

Provide:

[ Pair New Device ]

Generate a temporary pairing code or QR code.

Client installation flow:

Open NF Management
→ Join Existing Institution
→ Detect Local Hosts

If discovery works:

Atomic Coaching
OFFICE-PC

[ Connect ]

Otherwise:

Enter Host Address
[ __________ ]

Then:

Pairing Code
[ ______ ]

After pairing:

* issue device credentials
* create local trusted session
* register device
* audit pairing action

Administrators must be able to:

* view connected devices
* rename devices
* revoke a workstation
* block a device
* view last connection
* see device user activity

Do not expose an unauthenticated LAN API.

---

# 5. FIRST-RUN EXPERIENCE

The first run must look like real commercial software.

Flow:

Welcome to NF Management

→ Create New Institution
OR
→ Restore Existing Institution
OR
→ Connect to Local Host

For new institution:

Institution Name

Institution Type:

* Coaching Center
* Academy
* School
* Training Institute
* Other

Logo

Address

Phone

Email

Currency:
BDT by default

Language:
Bangla
English

Academic Session

Student ID Format

Receipt Format

Administrator Account

Backup Location

Then:

Create Workspace

The application should initialize:

* database
* required folders
* default roles
* admin account
* receipt numbering
* initial academic session
* backup configuration
* institution settings

Then show:

Your institution is ready.

---

# 6. AUTHENTICATION

Build complete local authentication.

Support:

* username/password
* email/password where configured
* logout
* local sessions
* password change
* administrator password reset
* idle session timeout
* remembered device optional
* account disabling
* failed login protection
* secure password hashing

Roles should include:

Owner
Administrator
Receptionist
Accountant
Teacher
Operator

Roles must be customizable.

One user may have multiple roles.

---

# 7. GRANULAR PERMISSIONS

Implement permission-based authorization.

Examples:

students.read
students.create
students.update
students.archive

admissions.read
admissions.manage

attendance.read
attendance.manage
attendance.edit_previous

finance.read
payments.collect
payments.refund
expenses.manage

exams.read
exams.manage
marks.enter
marks.verify
results.publish

communications.send
communications.templates
communications.settings

reports.read
reports.export

users.manage
roles.manage

settings.manage

backup.manage
backup.restore

Do not rely on hiding buttons.

Backend/native operations must check authorization.

---

# 8. DASHBOARD

Create role-specific dashboards.

Admin/Owner:

* total active students
* new admissions
* today's attendance
* today's collection
* monthly collection
* outstanding dues
* active batches
* teachers
* upcoming exams
* recent payments
* recent admissions
* due alerts
* backup status
* communication status
* system health

Accountant:

* today's collection
* due amount
* recent payments
* overdue students
* expenses
* cash summary
* quick payment button

Receptionist:

* student search
* new admission
* recent admissions
* due lookup where permitted
* quick guardian/student lookup

Teacher:

* today's batches
* attendance shortcuts
* upcoming exams
* pending marks
* assigned students/batches

---

# 9. STUDENT MANAGEMENT

Create complete student records.

Fields:

* internal UUID
* generated student ID
* roll
* name
* Bangla name
* photo
* gender
* date of birth
* blood group optional
* phone
* email
* address
* school/college
* SSC/HSC information where relevant
* admission date
* academic session
* course/program
* batch
* subjects
* guardian
* guardian phone
* emergency phone
* status
* scholarship
* discount
* notes
* documents
* custom fields

Statuses:

Active
Pending
Completed
Transferred
Suspended
Inactive
Archived

Student profile tabs:

Overview
Enrollment
Attendance
Fees
Payments
Exams
Results
Guardian
Documents
Communication
Notes
Activity

Provide:

* advanced search
* filters
* sorting
* bulk selection
* bulk update
* CSV/Excel import
* export
* print
* pagination where required
* keyboard-friendly workflows

---

# 10. GUARDIANS

Guardian should be a separate entity.

One guardian may be associated with multiple students.

Store:

* name
* relationship
* mobile
* secondary mobile
* email
* address
* profession optional
* notes

Guardian profile:

Students
Payments
Communication History
Notes

Searching by guardian phone should immediately show related students.

---

# 11. ADMISSIONS

Build a complete admission workflow.

Inquiry
→ Applicant
→ Review
→ Admission
→ Student
→ Enrollment
→ Fee Assignment

Support:

* walk-in admission
* inquiry registration
* applicant records
* status tracking
* document collection
* previous institution
* class/course selection
* batch selection
* guardian
* admission fee
* initial payment
* student ID generation
* admission receipt
* printable admission form

Admission dashboard:

* inquiries
* pending applicants
* admitted today
* monthly admissions
* conversion trend

---

# 12. COURSES, PROGRAMS & BATCHES

Support coaching-style structures.

Program/Course
→ Batch
→ Subject

Example:

HSC 2028
→ Physics Batch A
→ Physics

Batch fields:

* title
* code
* academic session
* program/course
* subject
* teacher
* room
* days
* start time
* end time
* capacity
* start date
* end date
* status

Detect:

* teacher conflicts
* room conflicts
* overlapping schedules

Support:

* batch transfer
* batch merging
* student removal
* bulk enrollment
* capacity warnings

Keep historical enrollment instead of overwriting old records.

---

# 13. ATTENDANCE

Teacher workflow:

Choose Batch
→ Choose Date
→ Mark Attendance
→ Save

Statuses:

Present
Absent
Late
Excused

Features:

* mark all present
* keyboard shortcuts
* quick toggles
* bulk update
* note per student
* previous-day edits with permission
* attendance lock
* attendance history
* monthly percentage
* low-attendance warning

Optional:

USB barcode/QR scanner support.

Allow generated student cards to contain:

* Student ID
* barcode or QR identifier

Scanning may open:

student profile

or:

attendance check-in

depending on mode.

---

# 14. FEES

Implement a serious financial module.

Support:

* admission fee
* monthly fee
* course fee
* subject fee
* exam fee
* registration fee
* material fee
* custom fee

Also:

* discounts
* scholarships
* fines
* waivers
* partial payment
* advance balance
* credit
* refunds

Entities should include:

fee_types
fee_structures
student_fee_assignments
invoices
invoice_items
payments
payment_allocations
discounts
refunds

Never store finance as only a single changing `paid_total`.

Financial history must remain auditable.

---

# 15. FAST CASHIER

This should be one of the best parts of the product.

Cashier screen:

Search using:

* Student ID
* name
* phone
* guardian phone
* roll

Immediately display:

Student
Batch
Current fees
Previous dues
Total outstanding
Recent payments

Example:

Rahim Hasan
HSC 2028 • Physics A

September Fee        ৳1,200
Exam Fee               ৳200
Previous Due            ৳500

Total Due             ৳1,900

Payment Amount
[ 1900 ]

Method
Cash

[ Receive Payment ]

After confirmation:

Payment received ✓

Receipt #RC-2026-009821

[ Print Receipt ]
[ Print Again ]
[ Send SMS ]

Payment creation must be transactional.

A crash during payment must not produce duplicate or corrupted financial records.

---

# 16. RECEIPTS

Create configurable professional receipts.

Support:

* A4
* half-page
* thermal 80mm where possible

Include:

* institution logo
* institution name
* address
* student
* ID
* batch
* receipt number
* payment items
* amount
* remaining due
* method
* collected by
* date/time
* signature area

Support:

* print
* PDF
* duplicate/reprint
* reprint watermark optional
* search by receipt number

Receipt numbers must not accidentally collide.

---

# 17. EXPENSES & CASHBOOK

Do not build a complete enterprise accounting suite.

Build a useful coaching-center cashbook.

Expense categories:

* rent
* utilities
* teacher payment
* salary
* printing
* supplies
* marketing
* maintenance
* miscellaneous
* custom

Expense record:

* category
* amount
* date
* vendor/payee
* method
* reference
* attachment
* note
* creator

Provide:

Daily Cash Summary

Opening Balance

* Cash Payments Received

- Cash Expenses
  = Expected Closing Cash

Allow administrator to record actual closing cash and differences.

Reports:

* income
* expenses
* net operational summary
* payment-method breakdown
* monthly trend

Clearly state this is operational finance, not full double-entry accounting.

---

# 18. TEACHERS & STAFF

Staff profiles:

* name
* photo
* phone
* email
* address
* role
* assigned subjects
* batches
* joining date
* status
* documents
* notes

Teacher view:

* assigned batches
* timetable
* attendance actions
* upcoming exams
* marks entry

Optional teacher payment ledger:

* fixed
* per class
* custom manual payment

Do not turn this into a full payroll/HR suite in V1.

---

# 19. EXAMS

Support:

* exam types
* subjects
* batches
* exam dates
* maximum marks
* pass marks
* grading system
* written/practical components if enabled
* mark entry
* mark review
* result publishing

Workflow:

Create Exam
→ Select Batches
→ Configure Subjects
→ Enter Marks
→ Verify
→ Calculate
→ Publish

---

# 20. MARK ENTRY

Create spreadsheet-like keyboard-friendly mark entry.

Columns:

Roll
Student
Marks
Status

Support:

* arrow/tab navigation
* bulk save
* validation
* absence
* missing mark highlighting
* draft mode
* verified state

Reject invalid marks greater than maximum unless configured otherwise.

---

# 21. RESULTS

Calculate:

* marks
* total
* average
* percentage
* grade
* GPA if configured
* pass/fail
* position optionally

Provide:

* individual marksheet
* batch result sheet
* merit list
* subject analysis
* student performance history
* printable report card

Grading rules must be configurable.

Do not hard-code one grading system.

---

# 22. COMMUNICATION CENTER

Create:

Communication

→ New Message
→ Outbox
→ Templates
→ Scheduled
→ Failed
→ History
→ Automations
→ Settings

Channels:

SMS
Email

Architecture:

Business Event
→ Message Created
→ Durable Local Outbox
→ Internet Available?
→ Provider Adapter
→ Delivery Attempt
→ Result Stored

The core operation must NEVER fail because the messaging provider is unavailable.

---

# 23. SMS PROVIDER SYSTEM

Create provider adapters.

Interface concept:

sendSms()
checkBalance()
checkDeliveryStatus()

Allow multiple providers to be added later.

V1 should include:

* generic HTTP SMS API adapter
* at least one concrete Bangladesh-provider adapter
* second adapter if feasible

Settings:

Provider
API endpoint
API key/token
Sender ID
Unicode/Bangla toggle where required

Button:

Send Test SMS

Store credentials encrypted.

Do not expose them in frontend logs or application logs.

---

# 24. EMAIL SYSTEM

Support:

Generic SMTP

Fields:

SMTP host
port
encryption
username
password
from email
from name

Also make provider architecture compatible with future API-based services.

Allow:

[ Send Test Email ]

Support:

* HTML emails
* text fallback
* PDF attachment where appropriate

---

# 25. MESSAGE TEMPLATES

Support placeholders.

Example payment SMS:

{institution_name}
Payment received: ৳{amount}
Student: {student_name}
Receipt: {receipt_no}
Due: ৳{due_amount}

Example Bangla:

প্রিয় অভিভাবক,
{student_name}-এর ৳{amount} টাকা গ্রহণ করা হয়েছে।
রসিদ: {receipt_no}
বকেয়া: ৳{due_amount}

Templates:

* admission confirmation
* payment received
* due reminder
* absence notification
* class cancellation
* exam reminder
* result notification
* announcement

Preview the final rendered message before sending.

For SMS show:

* Unicode/non-Unicode detection
* character count
* estimated segments
* recipient count

where technically feasible.

---

# 26. COMMUNICATION AUTOMATIONS

Allow toggles such as:

Payment received
→ SMS guardian

Student absent
→ SMS guardian

Fee due in 3 days
→ SMS

Fee overdue
→ SMS

Admission complete
→ Welcome SMS/email

Exam scheduled
→ Notify students

Result published
→ Notify students/guardians

Class cancelled
→ Batch announcement

Automations must generate outbox records.

They must not directly call the external provider from core business transactions.

---

# 27. BULK MESSAGING

Build recipient filters.

Examples:

All Active Students

Specific Batch

HSC 2028

Physics Batch A

Students With Due > ৳1000

Absent Today

Exam Candidates

Recipients:
127

Channel:
SMS
Email
Both

Allow:

Preview
Schedule
Send

Require confirmation before large sends.

Show estimated SMS count/cost when configured.

---

# 28. MESSAGE RELIABILITY

Message statuses:

draft
queued
waiting_for_network
sending
sent
delivered
failed
cancelled

Store each delivery attempt.

Retry failures with:

* bounded retry count
* exponential backoff
* jitter

Do not create duplicate messages on retry.

Use message/idempotency identifiers where providers support them.

---

# 29. LOCAL DOCUMENT STORAGE

Store:

* student photos
* guardian documents
* admission documents
* certificates
* expense receipts
* attachments

Never store large documents as SQLite BLOBs unless there is a strong reason.

Store metadata in the database and content in managed filesystem directories.

The application owns the directories.

Users should not need to manually manipulate them.

Validate:

* extensions
* MIME type where possible
* size
* file names
* access permission

---

# 30. GLOBAL SEARCH

Ctrl/Cmd + K

Search:

Students
Guardians
Teachers
Batches
Payments
Invoices
Receipts
Exams

Allow quick navigation.

Student search must feel instantaneous for realistic coaching-center datasets.

Use proper indexing and FTS where appropriate.

---

# 31. REPORTING

Create Reporting Center.

Student:

* active students
* admission report
* batch roster
* enrollment history

Attendance:

* daily
* monthly
* individual student
* batch percentage
* low attendance

Finance:

* daily collection
* date-range collection
* outstanding dues
* overdue dues
* payment method
* fee category
* refunds
* expenses
* cash summary

Academic:

* exam
* marks
* result
* student performance
* batch performance
* subject performance

Communication:

* messages sent
* failures
* channel
* template usage

System:

* user activity
* audit history

Allow:

* filter
* print
* PDF
* CSV
* Excel where appropriate

---

# 32. ID CARDS & PRINTABLE DOCUMENTS

Generate:

Student ID Card
Admission Form
Attendance Sheet
Money Receipt
Due Statement
Student Ledger
Exam Admit Card
Marksheet
Report Card
Batch List
Daily Collection
Monthly Collection

Allow institution logo and custom header/footer.

Templates should print consistently on common Windows printers.

---

# 33. AUDIT LOG

Audit sensitive activity.

Track:

actor
device
timestamp
action
resource type
resource ID
previous value where appropriate
new value where appropriate

Audit:

* payments
* refunds
* expenses
* fee changes
* student edits
* attendance edits
* marks edits
* result publication
* role changes
* permission changes
* backup restores
* device pairing
* settings
* communication sending

Audit history must not be silently editable by ordinary administrators.

---

# 34. DATABASE ENGINEERING

Use UUIDs where appropriate.

Tables should include:

created_at
updated_at
created_by where relevant

Use:

* foreign keys
* unique constraints
* check constraints
* indexes
* transactions
* migrations

Important operations must be atomic.

Examples:

receivePayment()
refundPayment()
admitStudent()
restoreBackup()

Do not spread critical business invariants only across UI code.

---

# 35. DATABASE MIGRATIONS

Create versioned migrations.

Every application upgrade must:

* detect schema version
* backup before destructive migration
* run migration transactionally where possible
* validate success
* provide clear recovery path

Test upgrades from previously supported versions.

Never instruct customers to manually edit SQLite.

---

# 36. ENCRYPTION & SECRETS

Encrypt sensitive local data at rest.

Use:

* SQLCipher for SQLite
* OS/native secure secret storage where appropriate

Store:

SMS API credentials
SMTP password
license information
local host secrets

outside ordinary React/frontend-readable configuration.

Never log:

* database encryption keys
* passwords
* SMTP passwords
* API tokens
* license signing secrets

---

# 37. BACKUP SYSTEM

This is a critical commercial feature.

Backup must include:

* consistent database snapshot
* student documents
* institution assets
* settings
* metadata
* version manifest
* checksums

Produce:

InstitutionName-Date-Time.nfbackup

Example:

Atomic-Coaching-2026-09-17-180500.nfbackup

Use a safe SQLite backup/snapshot mechanism.

Do not copy an active database file naïvely.

---

# 38. BACKUP DESTINATIONS

Support:

Local backup folder
Secondary drive
USB drive
Network/NAS folder if available

Allow administrators to configure:

Automatic Daily Backup
Keep Last 30
External Backup Destination

Dashboard:

Last Backup
Today 6:05 PM ✓

Last Verified
Today ✓

Backup Location
D:\NFBackups ✓

Warn clearly when:

* no recent backup
* backup drive disconnected
* storage full
* backup verification failed

---

# 39. RESTORE EXPERIENCE

First-run:

Restore Existing Institution

Choose:
.nfbackup

Then:

Verify Backup
→ Show Institution
→ Backup Date
→ Product Version
→ Integrity Status

[ Restore ]

Before restoring over an existing installation:

create safety backup

Then restore.

After restore:

run integrity checks
run schema migration if required
verify documents
verify database

Display:

Restore Successful ✓

---

# 40. BACKUP VERIFICATION

A backup existing is not enough.

Provide scheduled integrity verification.

Validate:

* archive readability
* manifest
* checksums
* SQLite integrity
* essential tables
* referenced files

The UI should differentiate:

Backup created

from:

Backup verified

---

# 41. COMMERCIAL LICENSING

Build an offline-capable licensing system.

NF owns an offline signing private key.

The distributed application contains only the verification public key.

License file:

Atomic-Coaching.nflicense

Signed contents may include:

license_id
institution
edition
issued_at
expires_at
maximum_devices
features
support_period

Never ship the private signing key.

The application validates licenses locally.

---

# 42. LICENSE EDITIONS

Architect for editions such as:

Local

Local Pro

Local + Connect

Possible capabilities:

Single-PC
LAN
Maximum Devices
SMS
Email
Advanced Reports
Backup Features
Future NF Connect

Do not scatter edition checks everywhere.

Use a capability/entitlement service.

Example:

lan_mode = true
sms = true
advanced_reports = true

---

# 43. LICENSE UX

Settings
→ License

Licensed to:
Atomic Coaching

Edition:
NF Management Local Pro

Devices:
4 / 10

Valid Through:
30 June 2027

Status:
Active ✓

Allow:

[ Import License ]
[ View License Details ]

A missing internet connection must not randomly invalidate a valid offline license.

---

# 44. PRODUCT UPDATES

The application must work indefinitely without checking online on every startup.

When internet exists:

Settings
→ Updates
→ Check for Updates

Provide optional automatic update checking.

For commercial release:

* version packages
* release notes
* migration notes
* rollback planning

Never silently apply risky database migrations.

---

# 45. APPLICATION VERSIONING

Use semantic versioning.

Example:

1.0.0
1.0.1
1.1.0

Show version in:

About
Diagnostics
Backup manifests
Logs

Maintain a changelog.

---

# 46. DIAGNOSTICS

Create:

Settings
→ System
→ Diagnostics

Show:

Application Version

Database Version

Database Integrity

Data Directory

Backup Status

Host Mode

Local IP

Connected Devices

Messaging Queue

Free Disk Space

Logs

Provide:

[ Export Diagnostic Bundle ]

Diagnostic bundle must redact secrets.

---

# 47. UX DESIGN

Do not make this look like old-school ERP software.

Use:

* clean neutral background
* restrained brand accent
* excellent typography
* compact but comfortable density
* clear data tables
* keyboard navigation
* fast forms
* drawers
* dialogs
* command palette
* professional empty states

Avoid:

* excessive gradients
* large flashy cards everywhere
* glassmorphism overload
* animations that slow cashier workflows

Administrative software should feel:

fast
calm
precise
trustworthy

---

# 48. APP SHELL

Sidebar:

Dashboard

Admissions

Students

Guardians

Academics

* Programs
* Batches
* Subjects
* Schedule

Attendance

Finance

* Overview
* Fees
* Payments
* Dues
* Expenses
* Receipts
* Cashbook

Exams

* Exams
* Marks
* Results

Communication

Reports

Staff & Access

Backup

Settings

Only show areas the current user may access.

---

# 49. TABLE UX

All major tables should support:

search
filters
sorting
pagination where needed
selection
bulk actions
column visibility
status badges
context actions

Preserve useful filters between navigations where appropriate.

---

# 50. RESPONSIVENESS

Primary target:
Windows desktop/laptop.

LAN client machines may use smaller displays.

The UI should work from approximately tablet-size upward where practical.

Do not sacrifice desktop information density for unnecessary mobile-first patterns.

---

# 51. BANGLADESH LOCALIZATION

Support:

English
Bangla

Use Unicode correctly.

Support BDT formatting.

Bangladesh phone-number normalization should be considered.

Allow institution-specific labels.

Do not hardcode English-only receipt fields.

Provide professional Bangla text rendering.

---

# 52. IMPORT / MIGRATION TOOLS

Many customers will already have Excel files.

Build a guided import wizard.

Students:

Upload CSV/Excel
→ Map Columns
→ Preview
→ Validate
→ Import

Show:

Valid: 489
Warnings: 8
Errors: 3

Never silently import corrupt records.

Allow export of failed rows.

Support import for:

Students
Guardians
Batches
Outstanding balances where safely modeled

Provide sample templates.

---

# 53. DEMO MODE

Create realistic demo data.

Institution:
Atomic Coaching Demo

Include:

500+ students
guardians
teachers
courses
batches
attendance history
fees
payments
dues
expenses
exams
results
communications

Demo mode should allow prospective customers to understand the product immediately.

Never mix demo data into a real institution database.

---

# 54. SECURITY

Protect against:

* SQL injection
* directory traversal
* arbitrary file access
* malformed imports
* unauthorized LAN access
* privilege escalation
* tampered license files
* credential leakage
* unsafe backup archives
* session hijacking
* unrestricted local API binding

LAN host should:

* bind only as intentionally configured
* authenticate every request
* validate client/device
* have revocable credentials
* rate-limit sensitive authentication endpoints

---

# 55. PERFORMANCE

Target smooth operation on ordinary office hardware.

Test with at least realistic datasets such as:

10,000 students
50,000+ payment records
hundreds of batches
large attendance history
large messaging history

Student search should remain fast.

Do not load an entire large database table into the browser.

Use:

indexes
pagination
virtualization where appropriate
efficient SQL

---

# 56. FAILURE HANDLING

Explicitly test:

PC loses power during payment

PC loses power during backup

LAN cable disconnects

client reconnects

internet disappears during SMS sending

provider times out

printer unavailable

USB backup drive removed

disk nearly full

migration fails

corrupt backup

duplicate payment submission

app crash

All critical flows should fail safely.

---

# 57. TESTING STRATEGY

Build tests as part of development, not afterward.

Unit tests:

* fee calculations
* discounts
* dues
* grade calculation
* permissions
* templates
* phone normalization
* licensing verification
* backup manifest validation

Database tests:

* constraints
* migrations
* transaction rollback
* duplicate prevention
* financial consistency

Integration tests:

* admission
* student enrollment
* attendance
* payment
* refund
* expense
* exam
* result
* messaging queue
* backup
* restore
* LAN pairing

E2E:

* first setup
* admin login
* receptionist admission
* accountant payment
* teacher attendance
* exam workflow
* backup and restore
* workstation pairing

---

# 58. MULTI-USER CONCURRENCY TESTING

Test LAN scenarios.

Examples:

Reception creates student while accountant searches.

Accountant collects payment while director views reports.

Two staff attempt to collect the same invoice simultaneously.

Teacher marks attendance while receptionist transfers student batch.

Ensure:

no corruption
no duplicated payments
no lost updates
clear conflict behavior

---

# 59. BACKUP / RESTORE TESTING

Automated release test:

Create realistic DB
→ Add documents
→ Backup
→ Delete local installation data
→ Fresh installation
→ Restore backup
→ Validate record counts
→ Validate key financial totals
→ Validate documents
→ Validate login/users

A release cannot be considered production-ready if this fails.

---

# 60. OFFLINE TESTING

Automated/manual release testing must include:

Disable internet completely.

Verify:

login ✓
students ✓
admissions ✓
attendance ✓
payments ✓
printing ✓
exams ✓
results ✓
reports ✓
backup ✓

Then queue:

SMS
Email

Reconnect internet.

Verify queue automatically resumes without duplicates.

---

# 61. PRINT TESTING

Test:

A4 printer
PDF printer
80mm receipt layout

Check:

page breaks
Bangla text
long student names
large amounts
logo sizing
margins
duplicate receipt

---

# 62. INSTALLATION TESTING

Test on clean Windows installation.

The installer should:

install app
create required application folders
configure permissions safely
not require development tools
not require Node.js
not require Rust
not require SQLite installation

Uninstall should:

remove application binaries

but must NOT silently delete customer institutional data/backups without explicit confirmation.

---

# 63. DELIVERY PACKAGE

The commercial release must contain:

Windows installer

Demo installer/data

Release notes

Administrator Quick Start Guide

Backup & Restore Guide

LAN Setup Guide

SMS/Email Setup Guide

Excel Import Guide

Printer Setup Guide

Troubleshooting Guide

License instructions

Privacy/Data Ownership documentation

Support diagnostics instructions

Do not ship only source code.

---

# 64. SOURCE CODE QUALITY

Use clean architecture without unnecessary enterprise abstraction.

Keep separation around:

UI

domain logic

database

files

communications

backup

licensing

LAN/networking

printing

Avoid giant files.

Avoid hidden global mutable state.

Avoid business logic directly inside components.

Use typed APIs and typed models.

---

# 65. LOGGING

Structured local logs.

Log:

startup
shutdown
migration
backup
restore
LAN status
message delivery errors
unexpected exceptions

Never log:

passwords
API keys
SMTP credentials
encryption keys
complete sensitive student records unnecessarily

Provide log rotation.

---

# 66. DATA OWNERSHIP

Make this explicit in the product:

“Your institution's data is stored locally on your system.”

Give administrators:

Open Data Folder
where safe

Export Data

Create Backup

Restore Backup

Do not secretly send student data to NF servers.

Future optional cloud synchronization must require explicit activation.

---

# 67. OUT OF SCOPE FOR V1

Do not bloat the initial product with:

full LMS
video hosting
live classes
student social network
AI tutor
biometrics hardware integration
GPS transport
hostel management
library system
inventory ERP
full HR/payroll
full double-entry accounting
cloud dependency

Create extension points without delaying V1.

---

# 68. OPTIONAL FUTURE NF CONNECT

Architect but do not require:

NF Management Local
↕
NF Connect

Future capabilities:

encrypted off-site backup
remote dashboard
central SMS
central email
NFStudio integration
Porokh integration
sync between branches

Local database remains authoritative unless a future architecture explicitly changes that.

---

# 69. BUILD PHASES

Phase 1:
Repository, architecture, database, migrations, local auth, roles, permissions, settings

Phase 2:
Students, guardians, admissions, courses, batches and enrollment

Phase 3:
Attendance and teacher workflows

Phase 4:
Fees, invoices, payments, dues, refunds, receipts, expenses and cashbook

Phase 5:
Exams, marks, grading and results

Phase 6:
Reports, printing, ID cards and imports

Phase 7:
SMS, email, templates, outbox, automations

Phase 8:
Backup, restore, integrity checking and migration safety

Phase 9:
LAN host, device pairing, multi-user mode

Phase 10:
Licensing, installer and update architecture

Phase 11:
UX polish, Bangla/English localization and accessibility

Phase 12:
Automated tests, performance, security, recovery tests and release hardening

---

# 70. BEFORE IMPLEMENTATION

First produce:

Architecture

Database ERD

Complete table list

Filesystem structure

Permission matrix

User journeys

LAN architecture

Backup specification

License architecture

Communication architecture

Printing architecture

Migration strategy

Security threat model

Test matrix

Release checklist

Then implement.

Do not remain indefinitely in planning.

After architectural validation, continue building the complete application.

---

# 71. AGENT WORKING RULES

Do not claim a feature is complete merely because its UI exists.

A feature is complete only when:

UI
+
validation
+
backend/domain logic
+
database
+
authorization
+
error handling
+
tests
+
real workflow

all work.

Do not replace working polished UI with crude generated components.

Do not remove functionality to make tests pass.

Do not use mock data in production paths.

Do not silently swallow errors.

Do not leave TODO placeholders in critical workflows.

Do not fake integration success.

When an external provider cannot be tested without credentials, create the adapter, mock provider tests and explicit manual verification instructions.

---

# 72. BROWSER / UI VERIFICATION

After implementing each major workflow, visually inspect it.

Check:

alignment
overflow
dialogs
forms
tables
Bangla text
empty states
loading
errors
print previews
low-resolution displays

Fix visually broken UI before marking the feature complete.

---

# 73. RELEASE BLOCKERS

The product must NOT be released if any of these are failing:

payment consistency

backup creation

backup restore

database migration

user authorization

LAN authentication

license validation

installer

offline operation

receipt printing

financial totals

duplicate payment protection

database integrity

critical E2E tests

---

# 74. SELL-READY ACCEPTANCE SCENARIO

Perform this final realistic test.

Start with a completely clean Windows computer.

Install NF Management Local.

Create:

Atomic Coaching

Create users:

Owner
Receptionist
Accountant
Teacher

Add:

Programs
Batches
Teachers

Import 500 students.

Receptionist admits a new student.

Teacher marks attendance.

Accountant receives a monthly fee.

Receipt prints correctly.

Payment confirmation SMS enters outbox.

Internet disabled:
application continues working.

Internet restored:
SMS sends exactly once.

Create exam.

Teacher enters marks.

Publish result.

Print report card.

Create daily financial report.

Create backup.

Install NF Management on another clean environment.

Restore that backup.

Verify:

student count
payment totals
attendance
results
documents
users
settings

Then enable LAN mode.

Connect second workstation.

Perform concurrent operations.

Restart host.

Reconnect client.

Verify no data loss.

Only after this full scenario passes should the build be considered delivery-ready.

---

# 75. FINAL DELIVERABLES

At completion provide:

1. production source repository
2. Windows installer
3. database migrations
4. demo data
5. tests
6. test results
7. build instructions
8. release instructions
9. backup/restore documentation
10. LAN deployment guide
11. SMS/email configuration guide
12. administrator manual
13. customer quick-start guide
14. security notes
15. known limitations
16. versioned release notes
17. signed-license generation utility kept separate from customer software
18. sample license
19. demo institution
20. final production-readiness report

The production-readiness report must explicitly state what was actually tested.

Do not use vague claims such as:

“production ready”

unless the required acceptance tests were executed successfully.

---

# FINAL PRODUCT STANDARD

The completed product should feel like something NF can install at a real coaching center tomorrow.

The receptionist should understand it without technical training.

The accountant should trust the payment records.

The owner should trust the reports.

Teachers should be able to take attendance quickly.

The institution should remain operational with no internet.

Data should survive crashes, upgrades and computer replacement through tested backups.

SMS/email failures should never break core operations.

Multiple office computers should work safely through LAN mode.

The installer, documentation, backup system, licensing system and support diagnostics are part of the product—not optional finishing work.

The final objective is not:

“build a coaching-center dashboard.”

The objective is:

**Build, test, package and deliver a complete commercial Windows coaching-management product that NF can confidently sell to real institutions.**
