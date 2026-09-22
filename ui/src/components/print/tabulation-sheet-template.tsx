import type { BatchTabulationSheetData } from "@/gen/rpc";

function escapeHtml(text: string | null | undefined): string {
  if (!text) return "";
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateTabulationSheetHtml(data: BatchTabulationSheetData): string {
  const subjectHeadersHtml = data.subjects
    .map(
      (sub) => `
        <th style="padding: 4px 6px; text-align: center; border: 1px solid #94a3b8; font-size: 7.5pt; min-width: 55px;" colspan="3">
          <div style="font-weight: 700; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 90px;" title="${escapeHtml(sub.subject_name)}">
            ${escapeHtml(sub.subject_name)}
          </div>
          <div style="font-size: 6.5pt; color: #cbd5e1; font-weight: normal;">
            (Full: ${(sub.full_marks / 100).toFixed(0)})
          </div>
        </th>
      `
    )
    .join("");

  const subjectSubHeadersHtml = data.subjects
    .map(
      () => `
        <th style="padding: 2px 4px; text-align: center; border: 1px solid #94a3b8; font-size: 6.5pt; width: 28px;">Marks</th>
        <th style="padding: 2px 4px; text-align: center; border: 1px solid #94a3b8; font-size: 6.5pt; width: 22px;">GP</th>
        <th style="padding: 2px 4px; text-align: center; border: 1px solid #94a3b8; font-size: 6.5pt; width: 20px;">Grd</th>
      `
    )
    .join("");

  const studentRowsHtml = data.rows
    .map((row, idx) => {
      const gpaFormatted = (row.gpa / 100).toFixed(2);
      const isAlt = idx % 2 === 1;

      const cellsHtml = data.subjects
        .map((sub) => {
          const cell = row.marks_by_subject[sub.exam_subject_id];
          if (!cell) {
            return `
              <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; color: #94a3b8; font-size: 7.5pt;">--</td>
              <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; color: #94a3b8; font-size: 7.5pt;">--</td>
              <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; color: #94a3b8; font-size: 7.5pt;">--</td>
            `;
          }

          if (cell.is_absent) {
            return `
              <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; color: #dc2626; font-size: 7pt; font-weight: bold;" colspan="3">ABS</td>
            `;
          }

          const markDisp =
            cell.marks !== null && cell.marks !== undefined
              ? (cell.marks / 100).toFixed(cell.marks % 100 === 0 ? 0 : 1)
              : "--";
          const gpDisp = (cell.grade_point / 100).toFixed(
            cell.grade_point % 100 === 0 ? 0 : 1
          );
          const isFail = !cell.passed && !cell.is_optional;

          return `
            <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; font-size: 7.5pt; ${isFail ? "color: #dc2626; font-weight: bold; background-color: #fee2e2;" : ""}">
              ${markDisp}
            </td>
            <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; font-size: 7.5pt; ${isFail ? "color: #dc2626; background-color: #fee2e2;" : ""}">
              ${gpDisp}
            </td>
            <td style="padding: 3px 2px; text-align: center; border: 1px solid #cbd5e1; font-size: 7.5pt; font-weight: bold; ${isFail ? "color: #dc2626; background-color: #fee2e2;" : ""}">
              ${escapeHtml(cell.grade)}
            </td>
          `;
        })
        .join("");

      return `
        <tr style="${isAlt ? "background-color: #f8fafc;" : "background-color: #ffffff;"} ${!row.passed ? "border-left: 3px solid #dc2626;" : ""}">
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-weight: 700; color: #2563eb;">#${row.rank}</td>
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-weight: 600;">${row.roll !== null && row.roll !== undefined ? row.roll : "--"}</td>
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-family: monospace; font-size: 7.5pt;">${escapeHtml(row.student_code)}</td>
          <td style="padding: 4px 6px; border: 1px solid #cbd5e1; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 130px;" title="${escapeHtml(row.student_name)}">
            ${escapeHtml(row.student_name)}
          </td>
          ${cellsHtml}
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-weight: 700;">${(row.total_marks / 100).toFixed(row.total_marks % 100 === 0 ? 0 : 1)}</td>
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-weight: 700; color: ${row.passed ? "#059669" : "#dc2626"};">${gpaFormatted}</td>
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-weight: 800; color: ${row.passed ? "#059669" : "#dc2626"};">${escapeHtml(row.grade)}</td>
          <td style="padding: 4px; text-align: center; border: 1px solid #cbd5e1; font-size: 7.5pt; font-weight: 600; color: ${row.passed ? "#059669" : "#dc2626"};">
            ${row.passed ? "PASSED" : "FAILED"}
          </td>
        </tr>
      `;
    })
    .join("");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Tabulation Sheet - ${escapeHtml(data.exam_name)} - ${escapeHtml(data.batch_name)}</title>
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
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", Arial, sans-serif;
      font-size: 8.5pt;
      line-height: 1.25;
      color: #0f172a;
      background: #ffffff;
    }
    .sheet-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-end;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 6px;
      margin-bottom: 8px;
    }
    .sheet-title {
      font-size: 14pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: #0f172a;
    }
    .meta-line {
      font-size: 8.5pt;
      color: #334155;
      margin-top: 2px;
    }
    .tab-table {
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
    }
    .tab-table th {
      background: #0f172a;
      color: #ffffff;
      font-size: 8pt;
      letter-spacing: 0.2px;
    }
    .sig-block {
      display: flex;
      justify-content: space-between;
      margin-top: 35px;
      padding: 0 30px;
    }
    .sig-item {
      text-align: center;
      width: 160px;
    }
    .sig-line {
      border-top: 1px dashed #475569;
      margin-bottom: 4px;
    }
    .sig-label {
      font-size: 8pt;
      font-weight: 600;
      color: #334155;
    }
  </style>
</head>
<body>
  <!-- Header -->
  <div class="sheet-header">
    <div>
      <div class="sheet-title">Academic Tabulation Register (মার্কশীট ট্যাবুলেশন শিট)</div>
      <div class="meta-line">
        <strong>Examination:</strong> ${escapeHtml(data.exam_name)} &nbsp;|&nbsp;
        <strong>Session:</strong> ${escapeHtml(data.session_name)} &nbsp;|&nbsp;
        <strong>Batch:</strong> ${escapeHtml(data.batch_name)}
      </div>
    </div>
    <div style="text-align: right; font-size: 8pt; color: #64748b;">
      Total Students: <strong>${data.rows.length}</strong><br>
      Printed: ${new Date().toLocaleDateString("en-GB")}
    </div>
  </div>

  <!-- Tabulation Table -->
  <table class="tab-table">
    <thead>
      <tr>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 34px;" rowspan="2">Rank</th>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 34px;" rowspan="2">Roll</th>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 55px;" rowspan="2">ID</th>
        <th style="padding: 4px 6px; text-align: left; border: 1px solid #94a3b8; width: 110px;" rowspan="2">Student Name</th>
        ${subjectHeadersHtml}
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 44px;" rowspan="2">Total</th>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 38px;" rowspan="2">GPA</th>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 34px;" rowspan="2">Grade</th>
        <th style="padding: 4px; text-align: center; border: 1px solid #94a3b8; width: 48px;" rowspan="2">Status</th>
      </tr>
      <tr>
        ${subjectSubHeadersHtml}
      </tr>
    </thead>
    <tbody>
      ${studentRowsHtml}
    </tbody>
  </table>

  <!-- Signatures -->
  <div class="sig-block">
    <div class="sig-item">
      <div class="sig-line"></div>
      <div class="sig-label">Tabulated by (Examiner)</div>
    </div>
    <div class="sig-item">
      <div class="sig-line"></div>
      <div class="sig-label">Scrutinized by (Committee)</div>
    </div>
    <div class="sig-item">
      <div class="sig-line"></div>
      <div class="sig-label">Approved by (Head of Institution)</div>
    </div>
  </div>
</body>
</html>
`;
}
