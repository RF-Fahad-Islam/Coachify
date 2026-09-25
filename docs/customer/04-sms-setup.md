# SMS Setup and Messaging Guide (এসএমএস সেটআপ ও মেসেজিং নির্দেশিকা)

Coachify includes a **Durable Offline-Safe SMS Outbox** (অফলাইন-নিরাপদ এসএমএস আউটবক্স) designed for Bangladesh coaching centers. Even if your internet disconnects while you are taking attendance or collecting fees, Coachify queues every message safely on disk and sends them once connectivity returns — without ever sending duplicate SMS messages.

---

## 1. Configuring Your SMS Gateway (এসএমএস গেটওয়ে কনফিগারেশন)

Coachify encrypts your SMS API credentials on your Windows PC using **Windows DPAPI** (`Encrypted Sensitive Secrets`), so your API key cannot be copied or read in plain text.

1. Open **Settings → Communication & SMS Setup** (সেটিংস → যোগাযোগ ও এসএমএস সেটআপ).
2. Select your **SMS Provider Adapter** (এসএমএস প্রোভাইডার):
   - **Bangladesh Bulk SMS HTTP Gateway** (SSL Wireless, BulkSMSBD, GreenWeb, DianaHost, MimSMS, or custom REST endpoint).
   - **Local Simulation / Dry Run Mode** (for testing templates without spending SMS balance).
3. Enter your gateway details:
   - **API Endpoint URL** (এপিআই লিংক).
   - **API Key / Token** (এপিআই কী — stored encrypted via Windows DPAPI).
   - **Approved Sender ID / Masking Name** (সেন্ডার আইডি, e.g., `88096176XXXXX` or your BTRC-approved 11-character brand mask).
4. Click **Save & Test Connection** (সংরক্ষণ ও টেস্ট এসএমএস পাঠান) and enter your own mobile number (`017XXXXXXXX`) to verify delivery.

---

## 2. Managing SMS Templates & Character Encoding (এসএমএস টেমপ্লেট ও অক্ষর হিসাব)

Bangladesh telecom operators calculate SMS segments differently for English (`GSM-7`) and Bengali (`UCS-2 Unicode`):

| Encoding Type (ভাষা) | 1 SMS Limit (১টি এসএমএস) | Multi-Part Segment Limit (একাধিক অংশের সীমা) |
|---|---|---|
| **English (`GSM-7`)** | Up to **160 characters** | 153 characters per segment |
| **Bengali (`UCS-2 Unicode`)** | Up to **70 characters** | 67 characters per segment |

### Editing Templates (টেমপ্লেট সম্পাদনা):
1. Go to **Communication → Message Templates** (মেসেজ টেমপ্লেট).
2. Select a workflow template:
   - **Fee Payment Confirmation** (বেতন পরিশোধ নিশ্চিতকরণ): Uses placeholders `{{student_name}}`, `{{amount_paid}}`, `{{receipt_no}}`, `{{current_due}}`.
   - **Absentee Guardian Alert** (অনুপস্থিতির নোটিশ): Uses `{{student_name}}`, `{{batch_name}}`, `{{attendance_date}}`.
   - **Exam Result Notification** (পরীক্ষার ফলাফল): Uses `{{student_name}}`, `{{exam_name}}`, `{{obtained_marks}}`, `{{grade}}`, `{{merit_position}}`.
   - **Monthly Due Reminder** (বকেয়া বেতন তাগাদা): Uses `{{student_name}}`, `{{due_amount}}`, `{{due_month}}`.
3. Watch the live **SMS Segment & Cost Calculator** below the template box to keep messages within **1 segment** whenever possible to save SMS credits.

---

## 3. Handling Internet Blackouts & the Durable Outbox (ইন্টারনেট না থাকলে এসএমএস ব্যবস্থাপনা)

What happens when you mark attendance or collect fees during a broadband outage?

1. **Automatic Queuing**:
   - Every SMS triggered by fee collection, attendance marking, or exam results is saved immediately in the local **Durable Outbox** (`Pending Offline`).
   - Your cashier and teachers continue working at full speed with zero waiting or error popups.
2. **Inspecting the Outbox Queue**:
   - Open **Communication → SMS Outbox** (এসএমএস আউটবক্স) or click the **Resilience Status Badge** in the top bar.
   - You will see counts for **Queued (অপেক্ষমাণ)**, **Sent (প্রেরিত)**, and **Failed / Invalid Number (ব্যর্থ)**.
3. **Flushing Queued Messages When Internet Returns**:
   - Once internet connectivity is restored, click **Retry / Flush Outbox** (আউটবক্স সেন্ড করুন).
   - Every message carries a unique cryptographic `idempotency_key` so a student's guardian will **never receive the same payment or attendance SMS twice**, even if the network drops mid-batch.

---

## 4. Guardian Contact Health Check (অভিভাবকের মোবাইল নম্বর যাচাই)

If an SMS fails because of a mistyped phone number:
1. Open **Students → Guardian Contact Health** (অভিভাবক যোগাযোগ স্বাস্থ্য).
2. Review students flagged with missing or invalid BD mobile numbers (must be 11 digits starting with `013`, `014`, `015`, `016`, `017`, `018`, or `019`).
3. Click **Fix Number** (নম্বর সংশোধন করুন) directly from the list to update the guardian's mobile number and re-queue the message.
