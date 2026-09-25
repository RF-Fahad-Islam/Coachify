# Owner License & Activation Guide (লাইসেন্স ও অ্যাক্টিভেশন নির্দেশিকা)

Coachify uses an offline-resilient cryptographic license system (`Ed25519` signed license certificates) bound to your coaching center's computer. This guide explains how trial mode, offline license activation, subscription renewals, and Safe Read-Only Mode work.

---

## 1. Checking Your Current License Status (লাইসেন্স স্ট্যাটাস যাচাই)

1. Open **Settings → License & Subscription** (সেটিংস → লাইসেন্স ও সাবস্ক্রিপশন).
2. Review your license card details:
   - **Plan Tier**: `Coachify Local` (Single PC) or `Coachify Local Pro`.
   - **License State**: `Active (সক্রিয়)`, `Trial (ট্রায়াল)`, `Grace Period (গ্রেস পিরিয়ড)`, or `Expired / Read-Only (মেয়াদোত্তীর্ণ)`.
   - **Device Fingerprint (`Machine ID`)**: A unique hardware identifier for your coaching center PC (e.g., `CFY-WIN-8A4F-92C1`).
   - **Valid Until Date**: Your current subscription expiry date.

---

## 2. Activating Your License Key (লাইসেন্স অ্যাক্টিভেশন করার নিয়ম)

Coachify supports **100% Offline License Activation**, so you can activate your PC even if the coaching center does not have internet access:

1. Copy your **Device Fingerprint (`Machine ID`)** from **Settings → License & Subscription**.
2. Share this `Machine ID` with Coachify Commercial Support when completing your subscription purchase or renewal.
3. You will receive a signed **Coachify License Key / Certificate File (`.cfylic` or Base64 License Token)** via email or WhatsApp.
4. In Coachify, open **Settings → License & Subscription** and click **Activate / Import License Key** (লাইসেন্স কী ইমপোর্ট করুন).
5. Paste the license token (or select the `.cfylic` file) and click **Verify & Activate** (যাচাই ও সক্রিয় করুন).
6. Coachify verifies the `Ed25519` cryptographic signature locally in milliseconds and unlocks full operational access immediately.

---

## 3. Grace Period & Safe Read-Only Mode (গ্রেস পিরিয়ড এবং নিরাপদ রিড-অনলি মোড)

We know that coaching centers are busiest at the beginning of the month. Coachify is designed so that a billing delay never traps your data:

1. **Renewal Warning & Grace Period**:
   - Before your license expires, Coachify shows a polite reminder banner so you can renew smoothly.
   - During the **Grace Period**, all fee collection, admission, attendance, and printing features continue working normally.
2. **What Happens in Safe Read-Only Mode (`Expired`)?**:
   - If a license remains unrenewed past the grace window, Coachify enters **Safe Read-Only Mode** (নিরাপদ রিড-অনলি মোড).
   - **What remains 100% available**:
     - Viewing all students, guardians, batches, attendance records, and exam results.
     - Viewing all historical receipts, financial ledgers, and due statements.
     - **Exporting all reports to CSV / Excel and printing PDFs.**
     - **Creating and restoring encrypted `.cbackup` database backups.**
   - **What is paused until renewal**:
     - Creating new admissions, recording new payments, or modifying records.
   - As soon as you import a renewed license key, normal write access resumes immediately with zero data loss.

---

## 4. Moving Your License to a New PC (নতুন কম্পিউটারে লাইসেন্স স্থানান্তর)

If you upgrade your office computer or replace a damaged PC:

1. On your old PC (if still operational), go to **Settings → Data Safety & Backups → One-Click PC Migration** (এক-ক্লিকে পিসি মাইগ্রেশন) to package your encrypted database and settings onto a USB drive.
2. Install Coachify on your new PC and restore your migration package using your **Recovery Sheet**.
3. Open **Settings → License & Subscription** on the new PC and copy its new **Device Fingerprint (`Machine ID`)**.
4. Contact Coachify Support to issue a **Machine Transfer License Certificate** for your new `Machine ID` at no extra license fee.
