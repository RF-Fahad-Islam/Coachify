# Quick Start Guide (দ্রুত শুরু করার নির্দেশিকা)

Welcome to **Coachify**! This guide walks you through installing Coachify on your coaching center PC, completing the first-time setup wizard, creating your academic programs and batches, and admitting your first student with a fee receipt in under ten minutes.

---

## Before You Begin (প্রস্তুতি)

Make sure you have:
- A Windows 10 or Windows 11 desktop or laptop PC (64-bit).
- Your coaching center's official name, address, and contact mobile number.
- A USB flash drive or secondary drive folder (`D:\CoachifyBackups`) ready for automated backups.
- An A4 or thermal printer if you want to print your **Encryption Recovery Sheet** (উদ্ধার শিট) and student receipts immediately.

---

## Step 1: Run the Installer (ইনস্টলেশন)

1. Double-click the installer file `Coachify-Setup-x64.exe`.
2. If Windows displays a blue **Windows Protected Your PC (SmartScreen)** prompt:
   - Click **More info** (আরও তথ্য).
   - Click **Run anyway** (তবুও চালান).
3. Follow the installation prompt and launch **Coachify** from your Desktop shortcut or Start Menu.

---

## Step 2: Complete the First-Time Setup Wizard (প্রাথমিক সেটআপ উইজার্ড)

When Coachify starts on a new PC, the **Setup Wizard** opens automatically:

1. **Institution Profile (প্রতিষ্ঠানের তথ্য)**:
   - Enter your **Institution Name** (প্রতিষ্ঠানের নাম), e.g., *Dhaka Central Coaching*.
   - Enter your **Branch / Address** (ঠিকানা) and **Official Phone Number** (মোবাইল নম্বর) — these appear on every printed fee receipt and due statement.
   - Select your academic currency (`BDT ৳`) and timezone (`Asia/Dhaka`).
2. **Owner Account Creation (মালিকের অ্যাকাউন্ট তৈরি)**:
   - Choose a **Username** (ইউজারনেম) such as `owner` and a strong **Password** (পাসওয়ার্ড).
   - This Owner account has full administrative permissions over fee waivers, refunds, backups, and staff accounts.
3. **Encryption & Recovery Sheet (নিরাপত্তা ও রিকভারি শিট)**:
   - Coachify generates a unique **24-word Recovery Phrase / Recovery Key** to protect your encrypted backups.
   - Click **Print Recovery Sheet** (রিকভারি শিট প্রিন্ট করুন) or write down the recovery code on paper and store it in your office safe.
   - Check the confirmation box verifying you have stored the Recovery Sheet safely.
4. **Backup Folder Selection (ব্যাকআপ ফোল্ডার নির্বাচন)**:
   - Select a secondary drive (`D:\CoachifyBackups`) or external USB drive where Coachify will automatically store verified daily backups.

---

## Step 3: Create Academic Programs & Batches (প্রোগ্রাম এবং ব্যাচ তৈরি)

Before enrolling students, set up your classes and batches:

1. From the left sidebar, click **Programs & Batches** (প্রোগ্রাম ও ব্যাচ).
2. Click **+ New Program** (নতুন প্রোগ্রাম):
   - Example: *HSC 2026 Science Target Batch*.
   - Set the default **Monthly Tuition Fee** (মাসিক বেতন), e.g., `৳ 2,500`, and **Admission Fee** (ভর্তি ফি), e.g., `৳ 1,000`.
3. Inside the program, click **+ Add Batch** (নতুন ব্যাচ যোগ করুন):
   - Example: *Physics Morning Batch A (Sat-Mon-Wed 8:00 AM)*.
   - Set the **Batch Capacity** (আসন সংখ্যা), e.g., `60` students, and assign the instructor.

---

## Step 4: Admit Your First Student & Print a Receipt (প্রথম শিক্ষার্থী ভর্তি ও রশিদ প্রিন্ট)

1. Press `Ctrl + K` or click **Students** (শিক্ষার্থী) in the sidebar, then click **+ New Admission** (নতুন ভর্তি).
2. Enter the student's details:
   - **Full Name** (শিক্ষার্থীর নাম) and **Roll / Student ID** (রোল নম্বর).
   - **Guardian Name** (অভিভাবকের নাম) and **11-digit Guardian Mobile Number** (অভিভাবকের মোবাইল নম্বর, e.g., `017XXXXXXXX`).
3. Select the **Batch** (ব্যাচ) you created in Step 3.
4. In the **Initial Payment** (প্রাথমিক পেমেন্ট) section:
   - Enter the cash received today (e.g., `৳ 3,500` for Admission + First Month Tuition).
   - Select payment method: **Cash** (নগদ), **bKash** (বিকাশ), or **Nagad** (নগদ).
5. Click **Complete Admission & Print Receipt** (ভর্তি সম্পন্ন ও রশিদ প্রিন্ট করুন).
6. Coachify saves the ledger record atomically and opens the **Receipt Print Preview** (রশিদ প্রিন্ট প্রিভিউ) so you can print an 80mm thermal slip or A4/A5 receipt immediately.

---

## Quick Verification Checklist (যাচাইকরণ তালিকা)

- [ ] Coachify opens to the **Dashboard** (ড্যাশবোর্ড) without errors.
- [ ] Your **Recovery Sheet** is printed and locked in a safe place.
- [ ] At least one **Program** and **Batch** are active.
- [ ] Your first student appears in **Students** (`Ctrl + K` search works instantly).
- [ ] Today's collection total matches your cash drawer in **Cashier & Fees** (ক্যাশিয়ার ও ফি).
