import type { AdmitCardBatchData, AdmitCardStudentItem } from "@/gen/rpc";
import { STUDENT_SILHOUETTE_SVG, INSTITUTION_LOGO_SVG } from "./id-card-template";

export interface AdmitCardPrintOptions {
  layoutMode?: "a5_single" | "a4_dual";
  institutionName?: string;
  institutionNameBn?: string;
  signatureTitle?: string;
  showWatermark?: boolean;
}

const DEFAULT_INSTITUTION_NAME = "COACHIFY ACADEMY";
const DEFAULT_INSTITUTION_NAME_BN = "কোচিফাই একাডেমি";
const DEFAULT_SIGNATURE_TITLE = "Controller of Examinations";

function escapeHtml(str?: string | null): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateSingleAdmitCardInnerHtml(
  batchData: AdmitCardBatchData,
  student: AdmitCardStudentItem,
  options?: AdmitCardPrintOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const signatureTitle = escapeHtml(options?.signatureTitle || DEFAULT_SIGNATURE_TITLE);

  const photoHtml = student.photo_data_uri
    ? `<img src="${escapeHtml(student.photo_data_uri)}" alt="${escapeHtml(student.student_name)}" style="width: 100%; height: 100%; object-fit: cover; border-radius: 4px;" />`
    : STUDENT_SILHOUETTE_SVG;

  // Subjects table rows
  const subjectRows = student.subjects
    .map(
      (s, idx) => `
      <tr style="border-bottom: 1px solid #cbd5e1; font-size: 8pt;">
        <td style="padding: 3.5px 6px; text-align: center; color: #64748b;">${idx + 1}</td>
        <td style="padding: 3.5px 6px; font-weight: 600; color: #0f172a;">${escapeHtml(s.subject_name)}</td>
        <td style="padding: 3.5px 6px; text-align: center; color: #334155;">${escapeHtml(s.exam_date || "--")}</td>
        <td style="padding: 3.5px 6px; text-align: center; color: #334155;">${escapeHtml(s.start_time || "--")}</td>
        <td style="padding: 3.5px 6px; text-align: center; font-weight: 600; color: #0284c7;">${escapeHtml(s.room_number || "--")}</td>
      </tr>
    `
    )
    .join("");

  // Instructions list
  const instructionsList = batchData.center_instructions
    .map((inst) => `<li style="margin-bottom: 1.5px;">${escapeHtml(inst)}</li>`)
    .join("");

  return `
    <div class="admit-card-container" style="position: relative; box-sizing: border-box; background: #ffffff; border: 1.5px solid #0f172a; border-radius: 6px; padding: 10px 14px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Kalpurush', 'Noto Sans Bengali', Arial, sans-serif; display: flex; flex-direction: column; justify-content: space-between; height: 100%;">
      <!-- Watermark Background -->
      ${
        options?.showWatermark
          ? `<div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(-25deg); opacity: 0.04; font-size: 28pt; font-weight: 900; pointer-events: none; text-transform: uppercase; white-space: nowrap;">${instName}</div>`
          : ""
      }

      <!-- Top Header with Center Branding -->
      <div>
        <div style="display: flex; align-items: center; justify-content: space-between; border-bottom: 2px solid #0f172a; padding-bottom: 6px; margin-bottom: 8px;">
          <div style="width: 36px; height: 36px; flex-shrink: 0;">
            ${INSTITUTION_LOGO_SVG}
          </div>
          <div style="text-align: center; flex-grow: 1; padding: 0 10px;">
            <div style="font-size: 13pt; font-weight: 900; letter-spacing: 0.5px; text-transform: uppercase; color: #0f172a; line-height: 1.1;">
              ${instName}
            </div>
            ${instNameBn ? `<div style="font-size: 8.5pt; color: #475569; font-weight: 500;">${instNameBn}</div>` : ""}
            <div style="display: inline-block; background: #0f172a; color: #ffffff; padding: 1.5px 12px; border-radius: 3px; font-size: 7.5pt; font-weight: 800; letter-spacing: 1px; margin-top: 3px; text-transform: uppercase;">
              ADMIT CARD / প্রবেশপত্র
            </div>
          </div>
          <!-- Verification QR Box -->
          <div style="width: 44px; height: 44px; border: 1px solid #cbd5e1; border-radius: 4px; padding: 2px; text-align: center; background: #f8fafc; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;">
            <div style="font-size: 5pt; font-weight: 700; color: #64748b; text-transform: uppercase;">QR VERIFY</div>
            <div style="font-size: 4pt; font-family: monospace; word-break: break-all; color: #0f172a; line-height: 1;">
              [QR]
            </div>
            <div style="font-size: 4pt; color: #0284c7; font-weight: 600; margin-top: 1px;">AUTHENTIC</div>
          </div>
        </div>

        <!-- Exam & Student Information Banner -->
        <div style="display: flex; justify-content: space-between; gap: 12px; margin-bottom: 8px;">
          <!-- Left: Student Info -->
          <div style="flex-grow: 1; display: grid; grid-template-columns: auto 1fr; gap: 2px 8px; font-size: 8.5pt; align-content: center;">
            <span style="font-weight: 600; color: #64748b;">Examination:</span>
            <span style="font-weight: 800; color: #0f172a;">${escapeHtml(batchData.exam_title)}</span>

            ${
              batchData.academic_session
                ? `<span style="font-weight: 600; color: #64748b;">Academic Session:</span>
                   <span style="font-weight: 600; color: #334155;">${escapeHtml(batchData.academic_session)}</span>`
                : ""
            }

            <span style="font-weight: 600; color: #64748b;">Candidate Name:</span>
            <span style="font-weight: 800; color: #0f172a;">${escapeHtml(student.student_name)}</span>

            <span style="font-weight: 600; color: #64748b;">Student Code / ID:</span>
            <span style="font-weight: 800; font-family: monospace; color: #0369a1;">${escapeHtml(student.student_code)}</span>

            <span style="font-weight: 600; color: #64748b;">Batch / Class:</span>
            <span style="font-weight: 700; color: #1e293b;">${escapeHtml(student.batch_name)}</span>

            ${
              student.roll_number
                ? `<span style="font-weight: 600; color: #64748b;">Exam Roll:</span>
                   <span style="font-weight: 800; color: #0f172a;">${escapeHtml(student.roll_number)}</span>`
                : ""
            }
          </div>

          <!-- Right: Candidate Photo -->
          <div style="width: 25mm; height: 30mm; border: 1.5px solid #cbd5e1; border-radius: 4px; overflow: hidden; background: #f8fafc; flex-shrink: 0; box-shadow: 0 1px 2px rgba(0,0,0,0.05);">
            ${photoHtml}
          </div>
        </div>

        <!-- Routine Schedule Table -->
        <div style="margin-bottom: 8px; border: 1px solid #cbd5e1; border-radius: 4px; overflow: hidden;">
          <table style="width: 100%; border-collapse: collapse; text-align: left;">
            <thead>
              <tr style="background: #f1f5f9; border-bottom: 1.5px solid #cbd5e1; font-size: 7.5pt; font-weight: 700; color: #475569; text-transform: uppercase;">
                <th style="padding: 4px 6px; width: 24px; text-align: center;">#</th>
                <th style="padding: 4px 6px;">Subject Name</th>
                <th style="padding: 4px 6px; text-align: center; width: 75px;">Date</th>
                <th style="padding: 4px 6px; text-align: center; width: 70px;">Start Time</th>
                <th style="padding: 4px 6px; text-align: center; width: 65px;">Room</th>
              </tr>
            </thead>
            <tbody>
              ${subjectRows}
            </tbody>
          </table>
        </div>

        <!-- Center Instructions (Bilingual) -->
        <div style="background: #f8fafc; border: 1px dashed #cbd5e1; border-radius: 4px; padding: 5px 8px; margin-bottom: 8px;">
          <div style="font-size: 6.8pt; font-weight: 800; color: #475569; text-transform: uppercase; margin-bottom: 2px;">
            General Instructions for Candidate / পরীক্ষার্থীদের জন্য নির্দেশাবলী:
          </div>
          <ol style="margin: 0; padding-left: 14px; font-size: 6.5pt; color: #334155; line-height: 1.35;">
            ${instructionsList}
          </ol>
        </div>
      </div>

      <!-- Signatures Footer -->
      <div style="display: flex; justify-content: space-between; align-items: flex-end; padding-top: 6px; border-top: 1px solid #e2e8f0; margin-top: auto;">
        <div style="text-align: center; width: 45mm;">
          <div style="border-bottom: 1px dashed #94a3b8; width: 100%; height: 14px;"></div>
          <div style="font-size: 6.5pt; font-weight: 600; color: #64748b; margin-top: 2px;">
            Candidate Signature
          </div>
        </div>

        <div style="text-align: center; width: 55mm;">
          <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 14px;"></div>
          <div style="font-size: 7pt; font-weight: 800; color: #0f172a; margin-top: 2px; text-transform: uppercase;">
            ${signatureTitle}
          </div>
        </div>
      </div>
    </div>
  `;
}

export function generateAdmitCardsHtml(
  batchData: AdmitCardBatchData,
  options?: AdmitCardPrintOptions
): string {
  const mode = options?.layoutMode || "a4_dual";

  if (mode === "a5_single") {
    const pagesHtml = batchData.students
      .map(
        (student, idx) => `
      <div class="a5-page" style="page-break-after: ${idx < batchData.students.length - 1 ? "always" : "auto"}; width: 148mm; min-height: 210mm; padding: 8mm 10mm; box-sizing: border-box;">
        ${generateSingleAdmitCardInnerHtml(batchData, student, options)}
      </div>
    `
      )
      .join("");

    return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Admit Cards - ${escapeHtml(batchData.exam_title)} (A5 Single)</title>
  <style>
    @page {
      size: A5 portrait;
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
      .a5-page {
        page-break-after: always;
      }
      .a5-page:last-child {
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

  // A4 Dual (2 admit cards stacked vertically per A4 sheet with horizontal cut guide)
  const studentPairs: [AdmitCardStudentItem, AdmitCardStudentItem | null][] = [];
  for (let i = 0; i < batchData.students.length; i += 2) {
    studentPairs.push([
      batchData.students[i],
      batchData.students[i + 1] ? batchData.students[i + 1] : null,
    ]);
  }

  const pagesHtml = studentPairs
    .map(
      (pair, pageIdx) => `
    <div class="a4-dual-page" style="page-break-after: ${pageIdx < studentPairs.length - 1 ? "always" : "auto"}; width: 210mm; height: 297mm; padding: 8mm 10mm; box-sizing: border-box; display: flex; flex-direction: column; justify-content: space-between;">
      <!-- Top Admit Card -->
      <div style="height: 135mm; width: 100%;">
        ${generateSingleAdmitCardInnerHtml(batchData, pair[0], options)}
      </div>

      <!-- Cut Guide Divider -->
      <div style="height: 7mm; display: flex; align-items: center; justify-content: center; position: relative;">
        <div style="border-top: 1px dashed #94a3b8; width: 100%; position: absolute;"></div>
        <span style="background: #ffffff; padding: 0 8px; font-size: 6.5pt; color: #94a3b8; position: relative; z-index: 1;">
          ✂ Cut here to separate admit cards
        </span>
      </div>

      <!-- Bottom Admit Card (if available in pair) -->
      <div style="height: 135mm; width: 100%;">
        ${
          pair[1]
            ? generateSingleAdmitCardInnerHtml(batchData, pair[1], options)
            : `<div style="height: 100%; border: 1px dashed #cbd5e1; border-radius: 6px; display: flex; align-items: center; justify-content: center; color: #94a3b8; font-size: 9pt;">[ Intentionally blank ]</div>`
        }
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
  <title>Admit Cards - ${escapeHtml(batchData.exam_title)} (A4 Dual)</title>
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
      .a4-dual-page {
        page-break-after: always;
      }
      .a4-dual-page:last-child {
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
 * React Component for in-app Preview of Admit Card.
 */
export function AdmitCardSheet({
  batchData,
  options,
}: {
  batchData: AdmitCardBatchData;
  options?: AdmitCardPrintOptions;
}) {
  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-6">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing Admit Cards for {batchData.exam_title} ({batchData.students.length}{" "}
        students)
      </div>
      <div
        className="bg-white p-6 shadow-sm border border-border rounded-md max-w-4xl w-full"
        dangerouslySetInnerHTML={{ __html: generateAdmitCardsHtml(batchData, options) }}
      />
    </div>
  );
}
