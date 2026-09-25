# Thermal and Laser Printer Setup Guide (প্রিন্টার সেটআপ নির্দেশিকা)

Coachify supports both **80mm POS Thermal Receipt Printers** (for fast counter fee collection and daily cash close slips) and **A4 / A5 Laser & Inkjet Printers** (for dual-copy fee receipts, due statements, admit cards, and student report cards).

---

## 1. Supported Printer Types (সমর্থিত প্রিন্টারসমূহ)

| Document Type (ডকুমেন্টের ধরন) | Recommended Printer (প্রস্তাবিত প্রিন্টার) | Paper Size (কাগজের মাপ) |
|---|---|---|
| Instant Fee Receipt (তাৎক্ষণিক বেতন রশিদ) | 80mm POS Thermal Printer (Xprinter, Rongta, Epson) | `80mm Roll` (72mm printable width) |
| Dual-Copy Office + Student Receipt (ডুয়াল কপি রশিদ) | Monochrome Laser Printer (Canon LBP2900, HP LaserJet, Brother) | `A4 Portrait` (Top: Student, Bottom: Office) |
| Single-Copy Compact Receipt (হাফ পেজ রশিদ) | Laser or Inkjet Tank Printer (Epson EcoTank, Canon Pixma) | `A5 Landscape / Portrait` |
| Report Cards, Tabulation & Due Statements (রেজাল্ট ও বকেয়া তালিকা) | Laser or Inkjet Printer | `A4 Portrait / Landscape` |

---

## 2. Connecting an 80mm Thermal Receipt Printer (৮০ মিমি থার্মাল প্রিন্টার সংযোগ)

1. **Physical Connection**:
   - Connect the 80mm thermal printer power cable and plug the USB cable directly into a back USB port on your desktop PC (avoid loose front panel USB hubs).
   - Load an **80mm thermal paper roll** with the heat-sensitive shiny side facing the print head.
2. **Windows Driver Installation**:
   - Run the manufacturer driver installer (e.g., *Xprinter Driver Setup v7.77* or *Rongta POS Printer Driver*).
   - Select `USB001 (Virtual Printer Port for USB)` and model `XP-80C` (with auto-cutter) or `XP-80`.
   - Open **Windows Settings → Bluetooth & devices → Printers & scanners**, click your thermal printer, and print a **Windows Test Page** to verify USB communication.
3. **Configure Paper Width in Windows**:
   - In **Printer Properties → Preferences → Advanced**, set **Paper Size** to `80(72.1) x 297 mm`.
   - Set margins to `0 mm` (None) so receipts do not shrink or clip on the right edge.

---

## 3. Configuring Printers Inside Coachify (কোচিফাই প্রিন্টার সেটিংস)

1. In Coachify, open **Settings → Printer & Receipt Settings** (সেটিংস → প্রিন্টার ও রশিদ সেটিংস).
2. Click **Refresh System Printers** (প্রিন্টার তালিকা রিফ্রেশ করুন) to detect all installed Windows printers.
3. **Select Your Default Receipt Format** (রশিদের ফরম্যাট নির্বাচন করুন):
   - Choose **Thermal 80mm (`thermal_80mm`)** for POS roll printers.
   - Choose **A4 Dual Copy (`a4_dual`)** to print both Student Copy and Office Copy on a single A4 sheet with a cut line in the middle.
   - Choose **A5 Single (`a5_single`)** for half-sheet laser printing.
4. **Custom Header & Footer Note**:
   - Add an optional subtitle or footer notice (e.g., *"Fees once paid are non-refundable. Please preserve this receipt."* / *"প্রদত্ত বেতন অফেরতযোগ্য। ভবিষ্যতের জন্য রশিদটি সংরক্ষণ করুন।"*).
5. Click **Save Printer Settings** (সেটিংস সংরক্ষণ করুন) and click **Print Sample Receipt** (নমুনা রশিদ প্রিন্ট করুন) to verify alignment and Bengali font rendering (`Kalpurush` / `Noto Sans Bengali`).

---

## 4. Laser Printer Settings for Report Cards & Due Statements (লেজার প্রিন্টার সেটিংস)

When printing A4 documents such as **Academic Report Cards** (একাডেমিক রিপোর্ট কার্ড), **Attendance Sheets** (উপস্থিতি শিট), or **Batch Due Statements** (বকেয়া বিবরণী):

1. Open any report or receipt and click **Print / Export PDF** (প্রিন্ট / পিডিএফ).
2. In the **Document Print Preview Modal**, verify:
   - **Paper Size**: `A4`
   - **Orientation**: `Portrait` for Due Statements and Report Cards; `Landscape` for Wide Tabulation Sheets.
   - **Show Institution Header & Logo**: Enabled (`চালু`).
3. If your printer runs out of toner or is temporarily offline, choose **Save as PDF** in the print dialog to save the formatted document to your desktop for later printing.

---

## 5. Resolving Common Printer Issues (প্রিন্টারের সাধারণ সমস্যার সমাধান)

- **Receipt prints blank paper on thermal printer**: The thermal paper roll is loaded upside down. Flip the roll so the paper feeds from the bottom of the roll toward the front cutter.
- **Bengali text appears broken or overlapping**: Coachify automatically waits for embedded Bengali fonts (`document.fonts.ready`) before printing. Always use the built-in **Print Receipt** button inside Coachify rather than pressing `Ctrl + P` on the raw screen.
- **Printer shows "Offline" after USB unplugging**: Reconnect the USB cable to the **exact same USB port** on the PC, or click **Refresh System Printers** in **Settings → Printer & Receipt Settings**.
