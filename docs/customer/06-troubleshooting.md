# Troubleshooting Guide: SmartScreen, Antivirus, Printers & Clock Warnings (সমস্যা ও সমাধান নির্দেশিকা)

This guide helps coaching center owners and receptionists quickly solve the most common Windows, antivirus, printer, and system clock issues without waiting for a technician.

---

## 1. Bypassing Windows SmartScreen During Installation (উইন্ডোজ স্মার্টস্ক্রিন সতর্কতা সমাধান)

When installing a new Coachify update on Windows 10 or Windows 11, you may see a blue popup saying **"Windows protected your PC — Microsoft Defender SmartScreen prevented an unrecognized app from starting."**

**Why this happens**: Windows displays this screen for newly released desktop installers until thousands of PCs have downloaded that specific version.

**How to proceed safely**:
1. Verify that the installer came from your official Coachify download or authorized USB installer.
2. Click the underlined text **More info** (আরও তথ্য) on the left side of the blue dialog.
3. Verify the publisher/application name shows **Coachify Desktop**.
4. Click the **Run anyway** (তবুও চালান) button at the bottom right.

---

## 2. Adding Windows Defender & Antivirus Exclusions (অ্যান্টিভাইরাস এক্সক্লুশন যুক্ত করা)

Some third-party antivirus tools (Avast, Kaspersky, ESET, QuickHeal) or Windows Defender Real-Time Scanning can slow down SQLite database writes or temporarily lock `.cbackup` files during daily cash close.

**How to exclude the Coachify data folder in Windows Security**:
1. Open the Start Menu and type **Windows Security** (উইন্ডোজ সিকিউরিটি).
2. Click **Virus & threat protection** → under *Virus & threat protection settings*, click **Manage settings**.
3. Scroll down to **Exclusions** (ব্যতিক্রমসমূহ) and click **Add or remove exclusions**.
4. Click **+ Add an exclusion → Folder** and add these two folders:
   - App installation folder: `C:\Program Files\Coachify` (or `%LOCALAPPDATA%\Coachify`)
   - Your external backup folder: `D:\CoachifyBackups`

---

## 3. Resolving "System Clock Jump Detected" Warning (কম্পিউটারের ঘড়ির তারিখ পরিবর্তনজনিত সতর্কতা)

If your desktop PC's CMOS battery (on the motherboard) becomes weak, the Windows clock may suddenly reset to `2020` or `1970` after a power cut.

Because financial receipts and audit logs depend on accurate timestamps, Coachify automatically pauses new fee transactions if it detects the system clock moved **backward** past the last recorded transaction timestamp.

**How to fix a Clock Jump Warning**:
1. Right-click the clock in the bottom-right corner of your Windows taskbar and click **Adjust date and time** (তারিখ ও সময় ঠিক করুন).
2. Set the **correct current Date and Time** (and Timezone: `(UTC+06:00) Astana, Dhaka`).
3. Return to Coachify. Once your Windows clock is set to the present date (after the last recorded receipt timestamp), the warning clears automatically, or the **Owner** can click **Verify & Acknowledge Clock Adjustment** (`acknowledge_clock_jump`) with their Owner password.
4. *Permanent hardware fix*: Ask your local computer shop to replace the motherboard **CR2032 CMOS coin battery** (costs ~৳50–৳100) so the PC keeps time during power cuts.

---

## 4. Clearing a Stuck Windows Printer Queue (আটকে থাকা প্রিন্ট কিউ পরিষ্কার করা)

If receipts stop printing after a paper jam or power cut:

1. Turn the thermal or laser printer **OFF**, check for jammed paper, and turn it back **ON**.
2. Press `Win + R` on your keyboard, type `control printers`, and press **Enter**.
3. Double-click your receipt printer to open the **Print Queue** window.
4. Click **Printer → Cancel All Documents** (সব প্রিন্ট বাতিল করুন).
5. If a document stays stuck on *Deleting...*:
   - Open Start Menu, type `cmd`, right-click **Command Prompt**, and click **Run as administrator**.
   - Run: `net stop spooler` followed by `net start spooler`.
6. Inside Coachify, go to **Cashier & Fees → Recent Receipts** and click **Reprint Receipt** (রশিদ পুনরায় প্রিন্ট করুন). Every reprinted slip is clearly marked `REPRINT / ডুপ্লিকেট কপি` for audit safety.

---

## 5. Exporting a Diagnostics Bundle for Support (সাপোর্ট টিমের জন্য ডায়াগনস্টিক ফাইল তৈরি)

If you ever need help from Coachify Support:
1. Go to **Settings → System Health & Diagnostics** (সেটিংস → সিস্টেম স্বাস্থ্য ও ডায়াগনস্টিকস).
2. Click **Export Redacted Diagnostics Bundle (`.zip`)**.
3. Coachify collects system logs, migration history, and database health checks while **automatically redacting all passwords, API keys, and recovery keys**.
4. Send that `.zip` file to your support representative via WhatsApp or email.
