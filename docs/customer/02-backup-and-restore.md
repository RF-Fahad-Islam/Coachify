# Backup and Restore Guide (ব্যাকআপ এবং রিস্টোর নির্দেশিকা)

Your coaching center's student records, fee ledgers, attendance sheets, and exam marks are your most valuable operational assets. Coachify uses an **Offline-First Crash-Safe SQLite WAL Database** combined with **Encrypted Backup Archives (`.cbackup`)** so your records remain safe even if your PC suffers a hard drive failure, virus attack, or sudden power cut.

---

## 1. Understanding the Recovery Sheet (রিকভারি শিট সম্পর্কে ধারণা)

Every backup file created by Coachify is encrypted using `ChaCha20-Poly1305` so that if a USB pen drive is lost or stolen, nobody else can read your financial or student data.

- During initial setup, Coachify generated your **Encryption Recovery Sheet** (রিকভারি শিট).
- **Keep this printed sheet inside your physical cash safe.**
- You will need the Recovery Key printed on that sheet if you ever move Coachify to a **brand new computer** after your old PC stops working.

---

## 2. Setting Up a Secondary Backup Drive (দ্বিতীয় ব্যাকআপ ড্রাইভ সেটআপ)

Storing backups only on `C:\` does not protect you if the Windows drive fails. Always configure a second location:

1. Plug in a dedicated USB flash drive (e.g., `E:\CoachifyBackups`) or select a separate disk partition (`D:\CoachifyBackups`).
2. Open **Settings → Data Safety & Backups** (সেটিংস → ডাটা নিরাপত্তা ও ব্যাকআপ).
3. Click **Change Backup Destination** (ব্যাকআপ ফোল্ডার পরিবর্তন করুন) and choose your USB or secondary drive folder.
4. Enable **Automated Daily Backup on Cash Close** (দৈনিক ক্যাশ ক্লোজে অটো ব্যাকআপ).

---

## 3. Running a Manual Backup (ম্যানুয়াল ব্যাকআপ নেওয়ার নিয়ম)

Before making major batch changes, importing hundreds of students from Excel, or closing the office for holidays:

1. Go to **Settings → Data Safety & Backups** (সেটিংস → ডাটা নিরাপত্তা ও ব্যাকআপ) or check the **Resilience Status Badge** in the top navigation bar.
2. Click **Create Verified Backup Now** (এখনই ব্যাকআপ নিন).
3. Coachify performs three safety steps automatically:
   - Checkpoints the live SQLite WAL ledger cleanly.
   - Encrypts and writes the `.cbackup` archive with a SHA-256 integrity manifest.
   - Runs a background verification check to confirm the backup file is 100% readable.
4. Wait for the green **Backup Verified** (ব্যাকআপ যাচাইকৃত) badge before unplugging your USB drive.

---

## 4. Running a Monthly Backup Verification Drill (মাসিক রিস্টোর মহড়া)

Coachify includes a non-destructive **Backup Verification Drill** (ব্যাকআপ যাচাই মহড়া) that proves your backup can be restored without touching your live database:

1. Open **Settings → Data Safety & Backups → Backup Verification Drill**.
2. Select your latest `.cbackup` file from your USB drive.
3. Enter your **Recovery Key** from your Recovery Sheet.
4. Click **Run Verification Drill** (রিস্টোর মহড়া চালান).
5. Coachify decrypts the backup into an isolated temporary sandbox, runs `PRAGMA integrity_check`, verifies the cryptographic audit hash chain, and compares student and receipt counts.
6. Upon completion, your **Disaster Recovery Score** (ডিজাস্টার রিকভারি স্কোর) updates to green.

---

## 5. Restoring Your Database in an Emergency (জরুরি অবস্থায় ডাটা রিস্টোর করার নিয়ম)

If your computer is replaced or Windows is reinstalled:

1. Install Coachify on the replacement PC.
2. On the welcome screen (or under **Settings → Restore Wizard**), click **Restore from Backup (`.cbackup`)** (ব্যাকআপ থেকে রিস্টোর করুন).
3. Browse and select the `.cbackup` file from your USB flash drive.
4. Enter your **Recovery Key** from your printed Recovery Sheet.
5. Review the **Backup Summary Preview** showing:
   - Institution Name & Snapshot Timestamp
   - Total Active Students & Receipts Count
   - Financial Ledger Hash Verification Status
6. Click **Confirm & Restore Database** (নিশ্চিত করুন এবং ডাটাবেস রিস্টোর করুন).
7. Coachify automatically backs up any existing local file before replacing the database, then restarts into your restored coaching center workspace.

---

## Daily Receptionist Checklist (দৈনিক চেকলিস্ট)

- [ ] Keep the backup USB pen drive plugged in before running **Daily Cash Close** (দৈনিক ক্যাশ ক্লোজ).
- [ ] Check that the top-bar **Resilience Status Badge** shows **Healthy (সবুজ)**.
- [ ] Run one **Backup Verification Drill** on the 1st of every month.
