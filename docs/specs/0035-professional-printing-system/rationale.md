# 0035. Professional Printing System: Rationale

> Decision record and rationale for Spec 0035 (Professional Printing System)

## Context

Coaching centers in Bangladesh operate in fast paced environments where paper documents remain essential for official communication with guardians and students. While Spec 0013 established receipt printing for counter cashiers, institutions also need to print student ID cards, exam admit cards, academic marksheets, report cards, attendance rosters, and due statements.

Counter staff frequently face challenges when generating documents. They must manage different physical printers (80mm thermal receipt printers, standard desktop laser printers, and specialized card printers). In addition, rendering Bengali script with complex vowel signs and conjunct ligatures often leads to garbled text when using external or uncalibrated tools. Batch operations (such as printing 200 exam admit cards right before an exam) can freeze computers with limited RAM.

Without a centralized, offline printing system, coaching centers rely on external printing shops or manual spreadsheet templates. This creates inconsistent branding, data security risks, and slow counter service.

## Options considered

### Option 1: Unified Document Print Architecture with Centralized Branding and Chunked Batch Processing

Expand the existing React and WebView2 printing engine with dedicated templates for all academic and financial document types. Store template branding and layout options in local database tables. Render large batch documents in memory efficient chunks of 50 records.

**Pros**:
* Reuses proven Edge WebView2 silent printing and PDF generation pipeline without adding heavy dependencies.
* Guarantees consistent typography and Bengali font ligature rendering across all documents.
* Prevents browser memory crashes during large print runs through chunked rendering.
* Operates completely offline without requiring internet access.

**Cons**:
* Requires implementing multiple custom print templates in React and CSS.

### Option 2: Server Side Headless Chrome or Rust PDF Generation

Introduce a backend PDF generation library or headless browser process inside the desktop shell to render PDF files in the background, then spool them directly to Windows printers.

**Pros**:
* Keeps rendering logic entirely in backend code.

**Cons**:
* Adds substantial binary bloat and memory overhead to the desktop bundle.
* Duplicates styling rules between UI views and server templates.
* Makes interactive print preview more difficult and slower.

### Option 3: Basic HTML Export Without Centralized Templates

Allow screens to trigger native window print with raw HTML without standardized template settings or batch chunking.

**Pros**:
* Minimal code to write upfront.

**Cons**:
* Inconsistent paper layouts, margins, and typography across screens.
* Counter workstations freeze when printing batches of more than 30 items.
* Lack of centralized branding and authorized signature controls.

## Rationale

We selected Option 1 because it builds naturally on the established print engine while solving the operational realities of coaching centers. Edge WebView2 is already bundled with Windows and provides native silent printing and PDF generation with zero added binary weight. Storing template options in the local database allows administrators to customize institution headers, logos, and signature titles once and have them reflected across all document types. Chunked rendering ensures that large batches of admit cards or report cards render smoothly without crashing entry level counter computers.
