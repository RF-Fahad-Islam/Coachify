import type { StudentIdCardData } from "@/gen/rpc";

export interface IdCardPrintOptions {
  layoutMode?: "a4_sheet" | "single_pvc";
  institutionName?: string;
  institutionNameBn?: string;
  subTitle?: string;
  signatureTitle?: string;
  showWatermark?: boolean;
}

const DEFAULT_INSTITUTION_NAME = "COACHIFY ACADEMY";
const DEFAULT_INSTITUTION_NAME_BN = "কোচিফাই একাডেমি";
const DEFAULT_SUBTITLE = "STUDENT IDENTITY CARD";
const DEFAULT_SIGNATURE_TITLE = "Principal / Director";

function escapeHtml(str?: string | null): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export const STUDENT_SILHOUETTE_SVG = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" style="width: 100%; height: 100%; display: block;">
  <rect width="64" height="64" rx="4" fill="#f1f5f9"/>
  <circle cx="32" cy="24" r="11" fill="#94a3b8"/>
  <path d="M14 54C14 44.0589 22.0589 36 32 36C41.9411 36 50 44.0589 50 54V56H14V54Z" fill="#94a3b8"/>
</svg>
`.trim();

export const INSTITUTION_LOGO_SVG = `
<svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg" style="width: 100%; height: 100%; display: block;">
  <rect width="32" height="32" rx="6" fill="#0f172a"/>
  <path d="M16 6L6 11V20C6 24 10 27 16 28C22 27 26 24 26 20V11L16 6Z" stroke="#ffffff" stroke-width="1.5" stroke-linejoin="round"/>
  <path d="M12 18C14 17 15.5 17.5 16 18C16.5 17.5 18 17 20 18V14C18 13 16.5 13.5 16 14C15.5 13.5 14 13 12 14V18Z" fill="#ffffff" fill-opacity="0.3" stroke="#ffffff" stroke-width="1"/>
  <circle cx="16" cy="10" r="1" fill="#38bdf8"/>
</svg>
`.trim();

export function generateSingleCardInnerHtml(
  card: StudentIdCardData,
  options?: IdCardPrintOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const subTitle = escapeHtml(options?.subTitle || DEFAULT_SUBTITLE);
  const signatureTitle = escapeHtml(options?.signatureTitle || DEFAULT_SIGNATURE_TITLE);

  const photoHtml = card.photo_data_uri
    ? `<img src="${escapeHtml(card.photo_data_uri)}" alt="${escapeHtml(card.full_name_en)}" style="width: 100%; height: 100%; object-fit: cover; border-radius: 4px;" />`
    : STUDENT_SILHOUETTE_SVG;

  return `
    <div class="id-card-frame" style="position: relative; width: 85.6mm; height: 53.98mm; box-sizing: border-box; background: #ffffff; border: 1px solid #0f172a; border-radius: 6px; overflow: hidden; display: flex; flex-direction: column; justify-content: space-between; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Kalpurush', 'Noto Sans Bengali', Arial, sans-serif;">
      <!-- Header Band -->
      <div style="background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%); color: #ffffff; padding: 4.5px 7px; display: flex; align-items: center; gap: 6px; border-bottom: 2px solid #38bdf8;">
        <div style="width: 22px; height: 22px; flex-shrink: 0;">
          ${INSTITUTION_LOGO_SVG}
        </div>
        <div style="flex-grow: 1; text-align: center; line-height: 1.1;">
          <div style="font-size: 8.5pt; font-weight: 800; letter-spacing: 0.5px; text-transform: uppercase;">${instName}</div>
          ${instNameBn ? `<div style="font-size: 6.5pt; color: #cbd5e1;">${instNameBn}</div>` : ""}
          <div style="font-size: 5.5pt; font-weight: 600; color: #38bdf8; letter-spacing: 0.8px;">${subTitle}</div>
        </div>
      </div>

      <!-- Watermark Background -->
      ${
        options?.showWatermark
          ? `<div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(-25deg); opacity: 0.05; font-size: 16pt; font-weight: 900; pointer-events: none; text-transform: uppercase; white-space: nowrap;">${instName}</div>`
          : ""
      }

      <!-- Main Body -->
      <div style="padding: 4px 7px; display: flex; gap: 8px; flex-grow: 1; align-items: center;">
        <!-- Left: Photo Frame & Barcode -->
        <div style="width: 23mm; display: flex; flex-direction: column; align-items: center; flex-shrink: 0;">
          <div style="width: 20mm; height: 23mm; border: 1.5px solid #cbd5e1; border-radius: 4px; overflow: hidden; background: #f8fafc; box-shadow: 0 1px 2px rgba(0,0,0,0.05);">
            ${photoHtml}
          </div>
          <div style="margin-top: 2px; font-family: monospace; font-size: 6pt; font-weight: 700; color: #0f172a; text-align: center; letter-spacing: 0.5px;">
            ${escapeHtml(card.student_code)}
          </div>
        </div>

        <!-- Right: Student Metadata -->
        <div style="flex-grow: 1; font-size: 6.8pt; line-height: 1.35; color: #1e293b;">
          <div style="font-size: 8.5pt; font-weight: 800; color: #0f172a; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 48mm;">
            ${escapeHtml(card.full_name_en)}
          </div>
          ${
            card.full_name_bn
              ? `<div style="font-size: 7pt; color: #475569; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 48mm;">${escapeHtml(card.full_name_bn)}</div>`
              : ""
          }

          <div style="margin-top: 3px; display: grid; grid-template-columns: auto 1fr; gap: 1px 5px; font-size: 6.5pt;">
            <span style="color: #64748b; font-weight: 600;">Batch:</span>
            <span style="font-weight: 700; color: #0f172a; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${escapeHtml(card.primary_batch_name || "General")}</span>

            ${
              card.academic_session
                ? `<span style="color: #64748b; font-weight: 600;">Session:</span>
                   <span style="font-weight: 600; color: #334155;">${escapeHtml(card.academic_session)}</span>`
                : ""
            }

            ${
              card.blood_group
                ? `<span style="color: #64748b; font-weight: 600;">Blood:</span>
                   <span style="font-weight: 800; color: #dc2626;">${escapeHtml(card.blood_group)}</span>`
                : ""
            }

            ${
              card.emergency_phone
                ? `<span style="color: #64748b; font-weight: 600;">Emergency:</span>
                   <span style="font-weight: 600; color: #0f172a; font-family: monospace;">${escapeHtml(card.emergency_phone)}</span>`
                : ""
            }
          </div>
        </div>
      </div>

      <!-- Footer Band with Barcode Stripe and Sign -->
      <div style="border-top: 1px solid #e2e8f0; background: #f8fafc; padding: 2px 7px; display: flex; justify-content: space-between; align-items: flex-end;">
        <div style="display: flex; flex-direction: column;">
          <div style="font-size: 5pt; color: #64748b; text-transform: uppercase;">Barcode</div>
          <div style="font-family: monospace; font-size: 6pt; font-weight: 700; letter-spacing: 1px; color: #0f172a;">
            *${escapeHtml(card.barcode_payload)}*
          </div>
        </div>
        <div style="text-align: right; width: 32mm;">
          <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 8px;"></div>
          <div style="font-size: 5pt; font-weight: 700; color: #334155; text-transform: uppercase; margin-top: 1px;">
            ${signatureTitle}
          </div>
        </div>
      </div>
    </div>
  `;
}

export function generateIdCardsHtml(
  cards: StudentIdCardData[],
  options?: IdCardPrintOptions
): string {
  const mode = options?.layoutMode || "a4_sheet";

  if (mode === "single_pvc") {
    const singleCard = cards[0];
    if (!singleCard) return "";

    return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>ID Card - ${escapeHtml(singleCard.student_code)}</title>
  <style>
    @page {
      size: 85.6mm 53.98mm;
      margin: 0;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      margin: 0;
      padding: 0;
      width: 85.6mm;
      height: 53.98mm;
      background: #ffffff;
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
    .id-card-frame {
      border: none !important;
      border-radius: 0 !important;
    }
  </style>
</head>
<body>
  ${generateSingleCardInnerHtml(singleCard, options)}
</body>
</html>
    `.trim();
  }

  // Multi-card A4 8-card sheet (2 columns x 4 rows) with crop marks
  // Chunking in pages of 8 cards
  const pages: StudentIdCardData[][] = [];
  for (let i = 0; i < cards.length; i += 8) {
    pages.push(cards.slice(i, i + 8));
  }

  const pagesHtml = pages
    .map(
      (pageCards, pageIdx) => `
    <div class="sheet-page" style="page-break-after: ${pageIdx < pages.length - 1 ? "always" : "auto"}; width: 210mm; min-height: 297mm; padding: 10mm 12mm; box-sizing: border-box; position: relative;">
      <!-- Page Header Label (hidden in clean print if desired, or small) -->
      <div style="display: flex; justify-content: space-between; font-size: 7pt; color: #94a3b8; margin-bottom: 4mm;">
        <span>Coachify Professional Printing - Student ID Card Sheet</span>
        <span>Sheet ${pageIdx + 1} of ${pages.length} (${pageCards.length} cards)</span>
      </div>

      <!-- 2 Columns x 4 Rows Grid with Crop Marks -->
      <div style="display: grid; grid-template-columns: repeat(2, 85.6mm); grid-template-rows: repeat(4, 53.98mm); gap: 4mm 6mm; justify-content: center; align-content: center;">
        ${pageCards
          .map(
            (card) => `
          <div style="position: relative; width: 85.6mm; height: 53.98mm;">
            <!-- Crop Mark Cut Guides (Corner Marks) -->
            <div style="position: absolute; top: -3mm; left: -3mm; width: 5mm; height: 5mm; border-top: 1px dashed #94a3b8; border-left: 1px dashed #94a3b8; pointer-events: none;"></div>
            <div style="position: absolute; top: -3mm; right: -3mm; width: 5mm; height: 5mm; border-top: 1px dashed #94a3b8; border-right: 1px dashed #94a3b8; pointer-events: none;"></div>
            <div style="position: absolute; bottom: -3mm; left: -3mm; width: 5mm; height: 5mm; border-bottom: 1px dashed #94a3b8; border-left: 1px dashed #94a3b8; pointer-events: none;"></div>
            <div style="position: absolute; bottom: -3mm; right: -3mm; width: 5mm; height: 5mm; border-bottom: 1px dashed #94a3b8; border-right: 1px dashed #94a3b8; pointer-events: none;"></div>

            ${generateSingleCardInnerHtml(card, options)}
          </div>
        `
          )
          .join("")}
      </div>
    </div>
  `
    )
    .join("");

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Student ID Cards Sheet (${cards.length} cards)</title>
  <style>
    @page {
      size: A4 portrait;
      margin: 0;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      margin: 0;
      padding: 0;
      background: #ffffff;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", "Noto Sans Bengali", Arial, sans-serif;
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
    @media print {
      .sheet-page {
        page-break-after: always;
      }
      .sheet-page:last-child {
        page-break-after: auto;
      }
    }
  </style>
</head>
<body>
  ${pagesHtml}
</body>
</html>
  `.trim();
}

/**
 * React Component for in-app Preview of ID Card Sheet or Single Card.
 */
export function IdCardSheet({
  cards,
  options,
}: {
  cards: StudentIdCardData[];
  options?: IdCardPrintOptions;
}) {
  const mode = options?.layoutMode || "a4_sheet";

  if (mode === "single_pvc") {
    const card = cards[0];
    if (!card)
      return (
        <div className="p-4 text-center text-muted-foreground">No card selected</div>
      );

    return (
      <div
        className="p-4 flex items-center justify-center bg-muted/40 rounded-lg overflow-auto"
        dangerouslySetInnerHTML={{ __html: generateSingleCardInnerHtml(card, options) }}
      />
    );
  }

  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-6">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing A4 8-card sheet layout ({cards.length} cards total)
      </div>
      <div
        className="bg-white p-6 shadow-sm border border-border rounded-md"
        dangerouslySetInnerHTML={{ __html: generateIdCardsHtml(cards, options) }}
      />
    </div>
  );
}
