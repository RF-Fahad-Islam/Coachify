# 0025. Multi channel templates and automations (Rationale)

## Context

Institutes and coaching centers in Bangladesh interact with parents across a spectrum of linguistic preferences: some parents prefer concise English messages, while many prefer native Bengali script (`বাংলা`).

However, telecommunications architecture creates a severe trap regarding text encoding and billing:
1. The Unicode character limit cliff:
   - Standard Latin text adhering to the GSM 03.38 7 bit alphabet allows 160 characters in a single SMS message (153 characters per part for multi segment messages).
   - If a message contains even a single Bengali character (`অ`, `ক`, `।`) or non GSM punctuation, the entire message must be encoded using UCS 2 Unicode. Under UCS 2, the limit drops drastically to 70 characters for a single message, and 67 characters per part for multi segment messages.
   - If software naively assumes all text is 160 characters, an institute composing a 140 character Bengali announcement will be billed for 3 separate SMS segments per recipient, multiplying costs unexpectedly.
2. False absence alerts:
   - Students frequently arrive 10 to 15 minutes late due to traffic or weather. If an absence alert sends immediately, parents call the coaching center in panic while the student is already sitting in class.
3. Repeated due reminders:
   - Cashiers collecting dues need automated assistance to remind parents, but sending reminders multiple times in the same billing cycle irritates parents and damages institute reputation.
4. Channel suitability:
   - Different messages fit different channels: SMS is ideal for immediate urgent absence alerts, WhatsApp delivers free rich formatted payment cards with emojis and institution helplines, and Email delivers official tuition fee statements with itemized breakdowns.

Coachify addresses these realities with native GSM versus Unicode segment detection, a 30 minute delay and cancellation buffer for absences across SMS and WhatsApp, period deduplicated fee reminders with batch review gates, and two tab broadcast recipient previews.

---

## Options Considered

### Option 1: Native Dual Engine Segment Math with Triggered Transactional Enqueueing, Review Gates, and Absence Cancellation Buffer (Chosen)

Build a dedicated encoding and template engine:
- Real time character analyzer scanning for non GSM codepoints, dynamically updating segment count and remaining characters as staff type for SMS.
- Standard character counting with UTF 8 formatting for WhatsApp and Email.
- System seed templates in both English and Bengali with dynamic placeholders across SMS, WhatsApp, and Email.
- Absence alerts enqueued with a 30 minute delay and tied to the student mark; updating the mark cancels the queued messages.
- Due reminders reviewed via a batch review gate with student exclusions and deduplicated using a unique database constraint per student, billing period, and channel.
- Batch broadcasts previewed with a two tab view separating ready recipients from students with missing contact info, guarded by an explicit confirmation modal.

Pros:
- Complete transparency: institute staff see exact segment counts, channel choices, and credit usage before sending.
- High operational safety: eliminates false absence alerts and duplicate due reminders.
- Administrative control: review gates let staff honor verbal payment extensions before issuing fee reminders.
- Omnichannel versatility: allows tailoring message formats to the strengths of each channel.
- Fully offline capable: templates and automation triggers are evaluated locally inside SQLite.

Cons:
- Slightly more frontend validation and preview rendering logic.

### Option 2: Restrict SMS to Transliterated Latin Script ("Banglish")

Enforce that all SMS templates be written in Latin characters (such as `Apnar shontan ajke absent chilo`) to force GSM 7 bit 160 character limits.

Pros:
- Always fits 160 characters.

Cons:
- Highly unprofessional: parents expect proper Bengali script or standard English from reputable academic institutes.
- Dialects and phonetic spelling vary widely across staff, causing inconsistent messaging.
- Unacceptable compromise for premier institutions.

### Option 3: Immediate Unbuffered Automation Triggers

Fire automations immediately upon saving records with no cancellation delay window.

Pros:
- Simpler scheduling logic.

Cons:
- Guarantees frequent support friction when late arriving students or clerical typos trigger erroneous absence alerts to parents.
