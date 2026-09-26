# Front Desk & Operations (Slices 14 and 15)

Back to [index](index.md). High speed front counter workflows, student cards, barcode scanning, visitor inquiry tracking, email delivery, and bulk actions.

## Slice 14: Counter operations

### 73. Per counter cash sessions · needs a decision · GA
Cashier shift control with opening float records, counter cash tracking, shift handovers, blind drop reconciliations, and discrepancy logs.
**Done when:** a cashier opens a shift with an opening float, collects payments assigned to their session, enters cash counts for blind reconciliation, and the owner reviews cash discrepancies.
- [ ] Design it (spec): `/architect per counter cash sessions`

### 74. Student ID cards & admit cards print studio · needs a decision · GA
Design and print student ID cards and exam admit cards with photos, coaching branding, and scannable barcodes in batch.
**Done when:** staff can customize ID and admit card templates, preview student cards with barcodes, and print single cards or full batch sheets on thermal or standard card paper.
- [ ] Design it (spec): `/architect student id cards & admit cards print studio`

### 75. Fast barcode scanner workflows · needs a decision · GA
Hardware barcode scanner acceleration for instant student cashier lookup and hands free rapid attendance scanning with sound feedback.
**Done when:** scanning a student barcode instantly opens their fee payment screen or logs attendance with audio confirmation, without requiring mouse clicks.
- [ ] Design it (spec): `/architect fast barcode scanner workflows`

## Slice 15: Intake & communications

### 76. Inquiry & applicant pipeline · needs a decision · GA
Track prospective students from walk in or phone inquiry through follow up reminders, and convert qualified leads into enrolled students with one click.
**Done when:** receptionists log visitor inquiries with course interest, set follow up reminders, track stage progression, and convert an inquiry into an enrolled student without retyping.
- [ ] Design it (spec): `/architect inquiry & applicant pipeline`

### 77. Email SMTP delivery channel · needs a decision · GA
Deliver notices, payment receipts, and report cards to guardian email addresses through a configurable SMTP outbox with retry logic.
**Done when:** staff configure an SMTP email provider, test the connection, queue automated email receipts and report cards, and monitor delivery status in the outbox.
- [ ] Design it (spec): `/architect email smtp delivery channel`

### 78. Bulk student operations & tools · needs a decision · GA
Execute batch actions across multiple students including batch transfers, bulk status promotions, and batch fee concessions with safety checks and audit logs.
**Done when:** staff select students in bulk, apply batch transfers or status changes with preview confirmation, and review a clear audit log of all updated records.
- [ ] Design it (spec): `/architect bulk student operations & tools`
