# Excel Student Import Guide (এক্সেল থেকে শিক্ষার্থী ইমপোর্ট নির্দেশিকা)

Moving from handwritten registers or Microsoft Excel spreadsheets to Coachify takes only a few minutes. The **Student Import Wizard** (শিক্ষার্থী ইমপোর্ট উইজার্ড) validates phone numbers, links siblings who share the same guardian mobile number, and enrolls students directly into their target batches.

---

## 1. Preparing Your Spreadsheet (এক্সেল ফাইল প্রস্তুত করার নিয়ম)

Coachify accepts both `.xlsx` (Excel) and `.csv` (UTF-8 Comma Separated Values) files, supporting English and Bengali (`Unicode`) student names.

### Required & Optional Columns (প্রয়োজনীয় ও ঐচ্ছিক কলামসমূহ):

| Column Name (কলামের নাম) | Required? | Format & Example (উদাহরণ) |
|---|---|---|
| **Student Name** (`student_name`) | **Yes (আবশ্যক)** | Full name in English or Bengali, e.g., `Tanvir Hasan` or `তানভীর হাসান` |
| **Guardian Name** (`guardian_name`) | **Yes (আবশ্যক)** | Father/Mother/Guardian name, e.g., `Md. Rafiqul Islam` |
| **Guardian Phone** (`guardian_phone`) | **Yes (আবশ্যক)** | 11-digit BD mobile number, e.g., `01711223344` (or `+8801711223344`) |
| **Student Code / Roll** (`student_code`) | Optional | Custom roll number, e.g., `HSC26-104` (auto-generated if left blank) |
| **School / College** (`institution_name`) | Optional | Current school or college, e.g., `Notre Dame College` |
| **Batch Name** (`batch_name`) | Optional | Must match an existing batch name if auto-enrolling |
| **Opening Due / Balance** (`opening_due`) | Optional | Previous unpaid balance in BDT, e.g., `1500` |

> **Tip for Excel formatting**: If Excel strips the leading `0` from mobile numbers (turning `01711223344` into `1711223344`), Coachify's smart normalizer automatically restores the leading `0` during import!

---

## 2. Step-by-Step Import Process (ইমপোর্ট করার ধাপসমূহ)

1. Open **Students** (শিক্ষার্থী) from the left navigation bar.
2. Click **Import from Excel / CSV** (এক্সেল / সিএসভি ইমপোর্ট করুন).
3. Click **Download Sample Template** (নমুনা টেমপ্লেট ডাউনলোড করুন) if you want a pre-formatted header sheet, or select your existing spreadsheet file.
4. **Column Mapping (কলাম মেলানো)**:
   - Coachify automatically detects columns like *Name*, *Father's Name*, *Mobile*, and *Roll*.
   - Verify each dropdown mapping and select the **Target Batch** (টার্গেট ব্যাচ) if all rows belong to the same batch.
5. **Validation & Duplicate Preview (যাচাই ও ডুপ্লিকেট প্রিভিউ)**:
   - Coachify scans every row before saving anything to the database.
   - Rows with invalid mobile numbers or missing names are highlighted in **Amber/Red** with clear line-by-line explanations.
6. **Handling Sibling & Shared Guardian Numbers (ভাই-বোন ও অভিন্ন অভিভাবক নম্বর)**:
   - If two students share the same `guardian_phone` (for example, a brother and sister studying in your center), Coachify links both students to a **Single Shared Guardian Profile** (`Edit Shared Guardian`) rather than creating duplicate guardian records.
   - If the exact same student name + guardian mobile already exists in the batch, Coachify flags it as a **Duplicate Row** (ডুপ্লিকেট) and skips it so you never double-charge a student.
7. Click **Confirm & Import Valid Students** (নিশ্চিত করুন ও ইমপোর্ট সম্পন্ন করুন).

---

## 3. Verifying Imported Enrollments (ইমপোর্ট পরবর্তী যাচাইকরণ)

After the import finishes:
1. Check the **Import Completion Summary** showing *Imported Count*, *Linked Guardians*, and *Skipped Rows*.
2. Open **Programs & Batches** and select your batch to verify the updated active student count.
3. Press `Ctrl + K` and search for two or three imported students by name or guardian mobile number to verify their profiles and batch fee plans.
