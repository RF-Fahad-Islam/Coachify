# Privacy and Data Ownership Guide (তথ্যের গোপনীয়তা এবং মালিকানা নীতি)

As a coaching center owner, your student roster, guardian phone numbers, batch enrollments, and financial collections represent years of reputation and hard work. Coachify is engineered from the ground up around one strict rule:

> **Your institution data belongs 100% to you and lives locally on your computer.**

---

## 1. Offline-First Architecture: Where Does Your Data Live? (আপনার তথ্য কোথায় সংরক্ষিত থাকে?)

Unlike web-based SaaS apps that store your entire student database on a shared cloud server, Coachify is a **Local-First Native Desktop Application**:

| Data Category (তথ্যের ধরন) | Stored On Your Local PC? | Sent to Coachify Cloud? |
|---|---|---|
| Student Names, Rolls & Profiles (শিক্ষার্থীর তথ্য) | **Yes (Local SQLite DB)** | **Never (কখনই না)** |
| Guardian Phone Numbers & Addresses (অভিভাবকের মোবাইল নম্বর) | **Yes (Local SQLite DB)** | **Never (কখনই না)** |
| Fee Collections, Receipts, Dues & Expenses (আয়-ব্যয় ও রশিদ) | **Yes (Local SQLite DB)** | **Never (কখনই না)** |
| Attendance & Exam Marks (উপস্থিতি ও পরীক্ষার নম্বর) | **Yes (Local SQLite DB)** | **Never (কখনই না)** |
| Commercial License Status & AI Credits (লাইসেন্স ও এআই ক্রেডিট) | Cached Locally | Yes (Commercial account verification only) |

Even if Coachify's cloud servers are unreachable or your internet line is cut for weeks, your coaching center can admit students, collect tuition fees, print receipts, take attendance, and prepare exam report cards without any interruption.

---

## 2. How Your Local Data & Secrets Are Protected (ডাটা নিরাপত্তা ও এনক্রিপশন)

Coachify protects your institution's records using multiple independent defense layers:

1. **Argon2id Staff Password Hashing**:
   - Owner, Accountant, Cashier, and Teacher passwords are hashed locally using `Argon2id` (OWASP & RFC 9106 standard). Even someone inspecting the raw database file cannot read staff passwords.
2. **Windows DPAPI Secret Encryption**:
   - Sensitive credentials such as your **SMS Gateway API Key** and **Cloud AI Gateway Token** are encrypted using the Windows Data Protection API (`CryptProtectData` / DPAPI) bound to your Windows user account.
3. **ChaCha20-Poly1305 Backup Encryption**:
   - Every `.cbackup` file written to your USB drive is encrypted with `ChaCha20-Poly1305` using the key derived from your **Encryption Recovery Sheet**. If a staff member loses a backup USB stick on the street, the file is completely unreadable to outsiders.
4. **Tamper-Evident Cryptographic Audit Log**:
   - Every fee receipt, void, refund, discount, and mark change is recorded in an append-only SHA-256 hash chain (`audit_log`), allowing the Owner to detect any unauthorized alteration immediately.

---

## 3. AI Privacy Controls & Redaction (এআই গোপনীয়তা নিয়ন্ত্রণ)

If you optionally use **Coachify Copilot (AI Assistant)**:
- You have full control in **Settings → AI Privacy Center** (এআই গোপনীয়তা কেন্দ্র).
- Personal identifiers (student phone numbers, exact home addresses, and sensitive secrets) can be redacted before any prompt leaves your machine.
- The AI Assistant **never has direct write access** to your database — any consequential action (such as preparing a fee adjustment or message draft) requires explicit human confirmation by an authorized staff member.

---

## 4. Exporting Your Data Anytime (যেকোনো সময় নিজের ডাটা এক্সপোর্ট করার অধিকার)

You are never locked in. At any time — even if your subscription expires and Coachify enters **Safe Read-Only Mode** — you can:
- Export your complete **Student & Guardian Directory** to `.csv` / Microsoft Excel (`.xlsx`).
- Export your complete **Financial Ledger, Receipts, and Due Statements** to `.csv` / Excel or printable PDF.
- Create and verify full `.cbackup` archives of your database.
