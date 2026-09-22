import type { StudentReportCardData } from "@/gen/rpc";

function escapeHtml(text: string | null | undefined): string {
  if (!text) return "";
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateReportCardHtml(data: StudentReportCardData): string {
  const gpaFormatted = (data.gpa / 100).toFixed(2);
  const percentageFormatted = (data.percentage / 100).toFixed(1);

  const subjectsHtml = data.subjects
    .map((sub, index) => {
      const marksDisplay = sub.is_absent
        ? `<span style="color: #dc2626; font-weight: bold;">ABSENT</span>`
        : sub.marks !== null && sub.marks !== undefined
          ? (sub.marks / 100).toFixed(sub.marks % 100 === 0 ? 0 : 2)
          : "--";

      const gpFormatted = (sub.grade_point / 100).toFixed(2);
      const isFail = !sub.passed && !sub.is_optional;

      return `
        <tr style="border-bottom: 1px solid #cbd5e1; ${isFail ? "background-color: #fef2f2;" : ""}">
          <td style="padding: 7px 10px; text-align: center; color: #475569;">${index + 1}</td>
          <td style="padding: 7px 10px; text-align: center; font-family: monospace;">${escapeHtml(sub.subject_code)}</td>
          <td style="padding: 7px 10px; font-weight: 500;">
            ${escapeHtml(sub.subject_name)}
            ${sub.is_optional ? `<span style="font-size: 8pt; color: #0284c7; margin-left: 4px;">(4th Subject)</span>` : ""}
          </td>
          <td style="padding: 7px 10px; text-align: center;">${(sub.full_marks / 100).toFixed(0)}</td>
          <td style="padding: 7px 10px; text-align: center; font-weight: 600;">${marksDisplay}</td>
          <td style="padding: 7px 10px; text-align: center; font-weight: 700; color: ${isFail ? "#dc2626" : "#0f172a"};">${escapeHtml(sub.grade)}</td>
          <td style="padding: 7px 10px; text-align: center; font-weight: 600;">${gpFormatted}</td>
        </tr>
      `;
    })
    .join("");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Progress Report Card - ${escapeHtml(data.student_name)}</title>
  <style>
    @page {
      size: A4 portrait;
      margin: 12mm 15mm;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", "SolaimanLipi", Arial, sans-serif;
      font-size: 10pt;
      line-height: 1.4;
      color: #0f172a;
      background: #ffffff;
      padding: 10px;
    }
    .header-table {
      width: 100%;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 12px;
      margin-bottom: 14px;
    }
    .inst-title {
      font-size: 18pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: #0f172a;
    }
    .inst-subtitle {
      font-size: 11pt;
      font-weight: 600;
      color: #475569;
      margin-top: 2px;
    }
    .inst-meta {
      font-size: 8.5pt;
      color: #64748b;
      margin-top: 3px;
    }
    .report-badge {
      display: inline-block;
      border: 1.5px solid #0f172a;
      padding: 4px 14px;
      font-size: 11pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 1px;
      margin-top: 6px;
      border-radius: 4px;
      background: #f8fafc;
    }
    .student-card {
      width: 100%;
      border: 1px solid #cbd5e1;
      border-radius: 4px;
      margin-bottom: 14px;
      background: #f8fafc;
    }
    .student-card td {
      padding: 6px 12px;
      font-size: 9.5pt;
    }
    .label {
      color: #64748b;
      font-size: 8.5pt;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .val {
      font-weight: 600;
      color: #0f172a;
    }
    .content-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 14px;
    }
    .content-table th {
      background: #0f172a;
      color: #ffffff;
      padding: 8px 10px;
      font-size: 9pt;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      border: 1px solid #0f172a;
    }
    .content-table td {
      border: 1px solid #e2e8f0;
      font-size: 9pt;
    }
    .summary-grid {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    .summary-cell {
      border: 1px solid #cbd5e1;
      padding: 10px;
      text-align: center;
      background: #f8fafc;
    }
    .summary-cell .metric {
      font-size: 16pt;
      font-weight: 800;
      color: #0f172a;
      margin-top: 4px;
    }
    .scale-table {
      font-size: 7pt;
      border-collapse: collapse;
      margin-left: auto;
    }
    .scale-table th, .scale-table td {
      border: 1px solid #cbd5e1;
      padding: 2px 4px;
      text-align: center;
    }
    .signature-row {
      width: 100%;
      margin-top: 50px;
      display: table;
    }
    .sig-col {
      display: table-cell;
      width: 33.33%;
      text-align: center;
      vertical-align: bottom;
    }
    .sig-line {
      width: 80%;
      border-top: 1px dashed #475569;
      margin: 0 auto 6px auto;
    }
    .sig-label {
      font-size: 8.5pt;
      font-weight: 600;
      color: #334155;
    }
  </style>
</head>
<body>
  <!-- Header -->
  <table class="header-table">
    <tr>
      <td style="vertical-align: top; width: 70%;">
        <div class="inst-title">${escapeHtml(data.institution_name)}</div>
        ${data.institution_name_bn ? `<div class="inst-subtitle">${escapeHtml(data.institution_name_bn)}</div>` : ""}
        <div class="inst-meta">${escapeHtml(data.institution_address)}</div>
        <div class="inst-meta">Phone: ${escapeHtml(data.institution_phones)}</div>
        <div class="report-badge">Progress Report Card</div>
      </td>
      <td style="vertical-align: top; text-align: right; width: 30%;">
        <!-- Mini grading scale reference -->
        <table class="scale-table">
          <tr style="background: #f1f5f9; font-weight: bold;">
            <th>Marks</th><th>Grade</th><th>GP</th>
          </tr>
          <tr><td>80-100</td><td>A+</td><td>5.0</td></tr>
          <tr><td>70-79</td><td>A</td><td>4.0</td></tr>
          <tr><td>60-69</td><td>A-</td><td>3.5</td></tr>
          <tr><td>50-59</td><td>B</td><td>3.0</td></tr>
          <tr><td>40-49</td><td>C</td><td>2.0</td></tr>
          <tr><td>33-39</td><td>D</td><td>1.0</td></tr>
          <tr><td>00-32</td><td>F</td><td>0.0</td></tr>
        </table>
      </td>
    </tr>
  </table>

  <!-- Student Info Card -->
  <table class="student-card">
    <tr>
      <td style="width: 25%;">
        <div class="label">Student Name</div>
        <div class="val">${escapeHtml(data.student_name)}</div>
      </td>
      <td style="width: 25%;">
        <div class="label">Student ID</div>
        <div class="val" style="font-family: monospace;">${escapeHtml(data.student_code)}</div>
      </td>
      <td style="width: 25%;">
        <div class="label">Roll Number</div>
        <div class="val">${data.roll !== null && data.roll !== undefined ? data.roll : "--"}</div>
      </td>
      <td style="width: 25%;">
        <div class="label">Batch</div>
        <div class="val">${escapeHtml(data.batch_name)}</div>
      </td>
    </tr>
    <tr>
      <td>
        <div class="label">Program</div>
        <div class="val">${escapeHtml(data.program_name)}</div>
      </td>
      <td>
        <div class="label">Examination</div>
        <div class="val">${escapeHtml(data.exam_name)}</div>
      </td>
      <td>
        <div class="label">Academic Session</div>
        <div class="val">${escapeHtml(data.session_name)}</div>
      </td>
      <td>
        <div class="label">Merit Position</div>
        <div class="val" style="color: #2563eb; font-weight: 700;">#${data.rank} <span style="font-size: 8pt; color: #64748b; font-weight: 400;">(out of ${data.total_students})</span></div>
      </td>
    </tr>
  </table>

  <!-- Subjects Table -->
  <table class="content-table">
    <thead>
      <tr>
        <th style="width: 5%;">#</th>
        <th style="width: 12%;">Code</th>
        <th style="width: 43%; text-align: left;">Subject Name</th>
        <th style="width: 10%;">Full Marks</th>
        <th style="width: 10%;">Obtained</th>
        <th style="width: 10%;">Grade</th>
        <th style="width: 10%;">Grade Point</th>
      </tr>
    </thead>
    <tbody>
      ${subjectsHtml}
    </tbody>
  </table>

  <!-- Performance Summary -->
  <table class="summary-grid">
    <tr>
      <td class="summary-cell" style="width: 20%;">
        <div class="label">Total Marks</div>
        <div class="metric">${(data.total_marks / 100).toFixed(data.total_marks % 100 === 0 ? 0 : 2)} <span style="font-size: 10pt; color: #64748b; font-weight: normal;">/ ${(data.total_max_marks / 100).toFixed(0)}</span></div>
      </td>
      <td class="summary-cell" style="width: 20%;">
        <div class="label">Percentage</div>
        <div class="metric">${percentageFormatted}%</div>
      </td>
      <td class="summary-cell" style="width: 20%;">
        <div class="label">Grade Point Avg (GPA)</div>
        <div class="metric" style="color: ${data.passed ? "#059669" : "#dc2626"};">${gpaFormatted}</div>
      </td>
      <td class="summary-cell" style="width: 20%;">
        <div class="label">Final Grade</div>
        <div class="metric" style="color: ${data.passed ? "#059669" : "#dc2626"};">${escapeHtml(data.grade)}</div>
      </td>
      <td class="summary-cell" style="width: 20%;">
        <div class="label">Result Status</div>
        <div class="metric" style="color: ${data.passed ? "#059669" : "#dc2626"};">${data.passed ? "PASSED" : "FAILED"}</div>
      </td>
    </tr>
  </table>

  <!-- Remarks Box -->
  <div style="border: 1px solid #cbd5e1; border-radius: 4px; padding: 10px; margin-bottom: 24px; background: #fafafa;">
    <div class="label" style="margin-bottom: 4px;">Teacher's Remarks / মন্তব্য:</div>
    <div style="font-size: 9.5pt; color: #334155; font-style: italic;">
      ${
        data.grade === "A+"
          ? "Outstanding performance! Keep up the brilliant standard."
          : data.grade === "A" || data.grade === "A-"
            ? "Very good academic result with strong potential for further growth."
            : data.passed
              ? "Satisfactory performance. Regular self-study and revision recommended."
              : "Needs immediate academic attention and remedial guidance in failing subjects."
      }
    </div>
  </div>

  <!-- Signature Block -->
  <div class="signature-row">
    <div class="sig-col">
      <div class="sig-line"></div>
      <div class="sig-label">Class Teacher</div>
    </div>
    <div class="sig-col">
      <div class="sig-line"></div>
      <div class="sig-label">Academic In-Charge</div>
    </div>
    <div class="sig-col">
      <div class="sig-line"></div>
      <div class="sig-label">Principal / Headmaster</div>
    </div>
  </div>
</body>
</html>
`;
}
