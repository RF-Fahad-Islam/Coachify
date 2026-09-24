import { formatBusinessDate } from "@/lib/utils";
import { INSTITUTION_LOGO_SVG } from "./id-card-template";

export interface AttendanceStudentRow {
  student_id: string;
  student_code: string;
  student_name: string;
  student_name_bn?: string | null;
  roll?: string | number | null;
}

export interface AttendanceSheetData {
  batch_name: string;
  program_name?: string;
  month: string; // e.g. "October 2026"
  days_in_month?: number; // 28 - 31, defaults to 31
  instructor_name?: string;
  students: AttendanceStudentRow[];
}

export interface AttendanceSheetPrintOptions {
  institutionName?: string;
  institutionNameBn?: string;
  signatureTitle?: string;
}

const DEFAULT_INSTITUTION_NAME = "COACHIFY ACADEMY";
const DEFAULT_INSTITUTION_NAME_BN = "কোচিফাই একাডেমি";
const DEFAULT_SIGNATURE_TITLE = "Academic Coordinator";

function escapeHtml(str?: string | null): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateAttendanceSheetHtml(
  data: AttendanceSheetData,
  options?: AttendanceSheetPrintOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const signatureTitle = escapeHtml(options?.signatureTitle || DEFAULT_SIGNATURE_TITLE);
  const daysCount = data.days_in_month || 31;

  // Generate date column headers (1..daysCount)
  const dayHeaders = Array.from({ length: daysCount }, (_, i) => i + 1)
    .map(
      (day) =>
        `<th style="width: 5.5mm; text-align: center; border: 1px solid #cbd5e1; font-size: 7pt; padding: 2px 0;">${day}</th>`
    )
    .join("");

  // Student rows
  const rowsHtml =
    data.students.length > 0
      ? data.students
          .map((st, idx) => {
            const emptyDayCells = Array.from({ length: daysCount }, () => "")
              .map(
                () =>
                  `<td style="border: 1px solid #cbd5e1; width: 5.5mm; height: 6mm;">&nbsp;</td>`
              )
              .join("");

            return `
        <tr style="font-size: 8pt; border-bottom: 1px solid #cbd5e1;">
          <td style="border: 1px solid #cbd5e1; padding: 2px 4px; text-align: center; color: #64748b;">${idx + 1}</td>
          <td style="border: 1px solid #cbd5e1; padding: 2px 4px; text-align: center; font-family: monospace; font-weight: 700;">${escapeHtml(st.student_code)}</td>
          <td style="border: 1px solid #cbd5e1; padding: 2px 6px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 48mm;">
            ${escapeHtml(st.student_name)}
            ${st.student_name_bn ? `<span style="font-size: 7pt; color: #64748b;"> (${escapeHtml(st.student_name_bn)})</span>` : ""}
          </td>
          <td style="border: 1px solid #cbd5e1; padding: 2px 4px; text-align: center; font-weight: 700;">${escapeHtml(st.roll ? String(st.roll) : "--")}</td>
          ${emptyDayCells}
          <td style="border: 1px solid #cbd5e1; width: 8mm; text-align: center; font-size: 7pt;"></td>
          <td style="border: 1px solid #cbd5e1; width: 8mm; text-align: center; font-size: 7pt;"></td>
          <td style="border: 1px solid #cbd5e1; width: 14mm; text-align: center; font-size: 7pt;"></td>
        </tr>
      `;
          })
          .join("")
      : `<tr><td colspan="${daysCount + 7}" style="text-align: center; padding: 12px; color: #64748b;">No students enrolled in this batch</td></tr>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Attendance Register - ${escapeHtml(data.batch_name)} (${escapeHtml(data.month)})</title>
  <style>
    @page {
      size: A4 landscape;
      margin: 8mm 10mm;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", "Noto Sans Bengali", Arial, sans-serif;
      font-size: 8.5pt;
      color: #0f172a;
      background: #ffffff;
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
  </style>
</head>
<body>
  <!-- Header Banner -->
  <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #0f172a; padding-bottom: 6px; margin-bottom: 8px;">
    <div style="width: 32px; height: 32px; flex-shrink: 0;">
      ${INSTITUTION_LOGO_SVG}
    </div>
    <div style="text-align: center; flex-grow: 1; padding: 0 10px;">
      <div style="font-size: 14pt; font-weight: 900; letter-spacing: 0.5px; text-transform: uppercase;">${instName}</div>
      ${instNameBn ? `<div style="font-size: 8.5pt; color: #475569;">${instNameBn}</div>` : ""}
      <div style="font-size: 8pt; font-weight: 800; color: #0284c7; text-transform: uppercase; margin-top: 1px;">
        BATCH ATTENDANCE REGISTER / হাজিরা খাতা
      </div>
    </div>
    <div style="text-align: right; font-size: 7.5pt; color: #64748b;">
      <div>Generated: ${formatBusinessDate(new Date().toISOString())}</div>
      <div style="font-weight: 700; color: #0f172a;">Coachify Academic Suite</div>
    </div>
  </div>

  <!-- Meta Info Grid -->
  <div style="display: flex; justify-content: space-between; background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 4px; padding: 4px 10px; margin-bottom: 8px; font-size: 8pt;">
    <div>
      <span style="color: #64748b; font-weight: 600;">Batch: </span>
      <span style="font-weight: 800; color: #0f172a;">${escapeHtml(data.batch_name)}</span>
      ${
        data.program_name
          ? `<span style="color: #64748b; margin-left: 12px; font-weight: 600;">Program: </span>
             <span style="font-weight: 700; color: #1e293b;">${escapeHtml(data.program_name)}</span>`
          : ""
      }
    </div>
    <div>
      <span style="color: #64748b; font-weight: 600;">Month & Year: </span>
      <span style="font-weight: 800; color: #0f172a;">${escapeHtml(data.month)}</span>
    </div>
    ${
      data.instructor_name
        ? `<div>
            <span style="color: #64748b; font-weight: 600;">Teacher / Mentor: </span>
            <span style="font-weight: 700; color: #0f172a;">${escapeHtml(data.instructor_name)}</span>
          </div>`
        : ""
    }
  </div>

  <!-- Attendance Table -->
  <table style="width: 100%; border-collapse: collapse; text-align: left; margin-bottom: 12px;">
    <thead>
      <tr style="background: #f1f5f9; color: #334155; font-size: 7pt; font-weight: 700; text-transform: uppercase;">
        <th style="border: 1px solid #cbd5e1; width: 6mm; text-align: center; padding: 3px 0;">#</th>
        <th style="border: 1px solid #cbd5e1; width: 22mm; text-align: center; padding: 3px 2px;">ID Code</th>
        <th style="border: 1px solid #cbd5e1; padding: 3px 6px;">Student Name</th>
        <th style="border: 1px solid #cbd5e1; width: 10mm; text-align: center; padding: 3px 0;">Roll</th>
        ${dayHeaders}
        <th style="border: 1px solid #cbd5e1; width: 8mm; text-align: center; padding: 3px 0; color: #059669;">P</th>
        <th style="border: 1px solid #cbd5e1; width: 8mm; text-align: center; padding: 3px 0; color: #b91c1c;">A</th>
        <th style="border: 1px solid #cbd5e1; width: 14mm; text-align: center; padding: 3px 0;">Remarks</th>
      </tr>
    </thead>
    <tbody>
      ${rowsHtml}
    </tbody>
  </table>

  <!-- Signatures -->
  <div style="display: flex; justify-content: space-between; align-items: flex-end; padding-top: 10px;">
    <div style="text-align: center; width: 50mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 12px;"></div>
      <div style="font-size: 7pt; font-weight: 700; margin-top: 2px;">Class Teacher / Instructor</div>
    </div>
    <div style="text-align: center; width: 50mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 12px;"></div>
      <div style="font-size: 7pt; font-weight: 700; margin-top: 2px; text-transform: uppercase;">${signatureTitle}</div>
    </div>
  </div>
</body>
</html>
  `.trim();
}

export function AttendanceSheetPrintable({
  data,
  options,
}: {
  data: AttendanceSheetData;
  options?: AttendanceSheetPrintOptions;
}) {
  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-4">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing A4 Landscape Attendance Register ({data.students.length} students)
      </div>
      <div
        className="bg-white p-6 shadow-sm border border-border rounded-md max-w-5xl w-full"
        dangerouslySetInnerHTML={{ __html: generateAttendanceSheetHtml(data, options) }}
      />
    </div>
  );
}
