import type { StudentDetail, ChargeItem, PaymentReceipt } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";

export interface AdmissionEnrollmentItem {
  batch_name: string;
  roll?: string | number | null;
  start_date: string;
}

export interface AdmissionFormData {
  institutionName: string;
  institutionAddress?: string | null;
  institutionPhone?: string | null;
  student: StudentDetail;
  enrollments: AdmissionEnrollmentItem[];
  charges: ChargeItem[];
  payment?: PaymentReceipt | null;
  photoUrl?: string | null;
}

const ACADEMIC_EMBLEM_SVG = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="56" height="56" style="display: block; color: #111;">
  <path d="M32 4L10 12V30C10 44 19.5 56 32 60C44.5 56 54 44 54 30V12L32 4Z" stroke="#111" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M32 8L14 14.5V30C14 41.5 21.8 51.5 32 55C42.2 51.5 50 41.5 50 30V14.5L32 8Z" stroke="#111" stroke-width="1" stroke-dasharray="2 2"/>
  <path d="M22 38C26 36 30 36.5 32 38C34 36.5 38 36 42 38V27C38 25 34 25.5 32 27C30 25.5 26 25 22 27V38Z" fill="#111" fill-opacity="0.15" stroke="#111" stroke-width="1.5" stroke-linejoin="round"/>
  <line x1="32" y1="27" x2="32" y2="38" stroke="#111" stroke-width="1.5"/>
  <path d="M32 15C33.5 17.5 35 19 33.5 21.5C32.8 22.8 31.2 22.8 30.5 21.5C29 19 30.5 17.5 32 15Z" fill="#111"/>
  <path d="M30 22H34L33 26H31L30 22Z" fill="#111"/>
  <circle cx="21" cy="20" r="1.2" fill="#111"/>
  <circle cx="43" cy="20" r="1.2" fill="#111"/>
  <circle cx="32" cy="46" r="1.5" fill="#111"/>
</svg>
`.trim();

export function generateAdmissionFormHtml(data: AdmissionFormData): string {
  const {
    institutionName,
    institutionAddress,
    institutionPhone,
    student,
    enrollments,
    charges,
    payment,
    photoUrl,
  } = data;

  const totalBilled = charges.reduce((sum, c) => sum + c.net_amount_poisha, 0);
  const totalPaid = payment ? payment.amount_poisha : 0;
  const balanceDue = totalBilled - totalPaid;

  const primaryGuardian =
    student.guardians.find((g) => g.is_primary) || student.guardians[0];

  const batchesHtml =
    enrollments.length > 0
      ? enrollments
          .map(
            (enr, idx) => `
        <tr style="border-bottom: 1px solid #e5e7eb;">
          <td style="padding: 6px 8px; font-weight: 500;">${idx + 1}. ${escapeHtml(enr.batch_name)}</td>
          <td style="padding: 6px 8px; text-align: center; font-family: monospace;">${enr.roll ? `Roll ${enr.roll}` : "Auto"}</td>
          <td style="padding: 6px 8px; text-align: right; color: #4b5563;">${formatBusinessDate(enr.start_date)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="3" style="padding: 8px; text-align: center; color: #6b7280;">No academic batches enrolled</td></tr>`;

  const feesHtml =
    charges.length > 0
      ? charges
          .map(
            (ch) => `
        <tr style="border-bottom: 1px solid #e5e7eb;">
          <td style="padding: 6px 8px;">${escapeHtml(ch.fee_head_name)}</td>
          <td style="padding: 6px 8px; text-align: right; font-family: monospace;">${formatTaka(ch.gross_amount_poisha)}</td>
          <td style="padding: 6px 8px; text-align: right; font-family: monospace; color: #059669;">${ch.concession_amount_poisha > 0 ? `-${formatTaka(ch.concession_amount_poisha)}` : "0"}</td>
          <td style="padding: 6px 8px; text-align: right; font-family: monospace; font-weight: 600;">${formatTaka(ch.net_amount_poisha)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="4" style="padding: 8px; text-align: center; color: #6b7280;">No admission charges generated</td></tr>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Student Admission Form - ${escapeHtml(student.student_code)}</title>
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
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
      font-size: 11pt;
      line-height: 1.4;
      color: #111827;
      background: #fff;
    }
    .container {
      max-width: 100%;
      margin: 0 auto;
    }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 2px solid #111827;
      padding-bottom: 12px;
      margin-bottom: 16px;
    }
    .title-box {
      text-align: center;
      flex-grow: 1;
      padding: 0 16px;
    }
    .inst-name {
      font-size: 18pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: #0f172a;
    }
    .inst-sub {
      font-size: 9pt;
      color: #4b5563;
      margin-top: 2px;
    }
    .form-title {
      display: inline-block;
      margin-top: 6px;
      padding: 3px 12px;
      background: #0f172a;
      color: #fff;
      font-size: 10pt;
      font-weight: 700;
      letter-spacing: 1px;
      border-radius: 4px;
      text-transform: uppercase;
    }
    .section-title {
      font-size: 10pt;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      background: #f1f5f9;
      padding: 4px 8px;
      border-left: 4px solid #0f172a;
      margin-top: 14px;
      margin-bottom: 8px;
    }
    .meta-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 8px;
      font-size: 9.5pt;
    }
    .meta-row {
      display: flex;
    }
    .meta-label {
      width: 130px;
      color: #64748b;
      font-weight: 500;
      flex-shrink: 0;
    }
    .meta-value {
      font-weight: 600;
      color: #0f172a;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 9.5pt;
      margin-top: 4px;
    }
    th {
      background: #f8fafc;
      padding: 6px 8px;
      text-align: left;
      font-size: 8.5pt;
      text-transform: uppercase;
      color: #475569;
      border-bottom: 1.5px solid #cbd5e1;
    }
    .photo-frame {
      width: 90px;
      height: 110px;
      border: 1.5px dashed #94a3b8;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      text-align: center;
      font-size: 8pt;
      color: #94a3b8;
      overflow: hidden;
      flex-shrink: 0;
    }
    .photo-img {
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
    .finance-summary {
      display: flex;
      justify-content: flex-end;
      margin-top: 8px;
    }
    .finance-card {
      width: 260px;
      background: #f8fafc;
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 8px 12px;
      font-size: 9pt;
    }
    .finance-row {
      display: flex;
      justify-content: space-between;
      padding: 3px 0;
    }
    .declaration {
      margin-top: 14px;
      padding: 8px 10px;
      background: #fafafa;
      border: 1px solid #e5e7eb;
      border-radius: 4px;
      font-size: 8pt;
      color: #4b5563;
      line-height: 1.35;
    }
    .signatures {
      display: flex;
      justify-content: space-between;
      margin-top: 40px;
      padding-top: 8px;
    }
    .sig-line {
      width: 180px;
      text-align: center;
      border-top: 1px solid #111827;
      font-size: 8.5pt;
      font-weight: 600;
      color: #1f2937;
      padding-top: 4px;
    }
  </style>
</head>
<body>
  <div class="container">
    <!-- Header with Crest, Name, and Photo Box -->
    <div class="header">
      <div style="flex-shrink: 0;">
        ${ACADEMIC_EMBLEM_SVG}
      </div>
      <div class="title-box">
        <div class="inst-name">${escapeHtml(institutionName)}</div>
        <div class="inst-sub">${escapeHtml(institutionAddress || "Academic Excellence & Learning Center")}${institutionPhone ? ` • Phone: ${escapeHtml(institutionPhone)}` : ""}</div>
        <div class="form-title">Student Admission Record</div>
      </div>
      <div class="photo-frame">
        ${photoUrl ? `<img src="${photoUrl}" class="photo-img" alt="Student Photo" />` : "Affix Passport Size Photo"}
      </div>
    </div>

    <!-- Admission Meta -->
    <div style="display: flex; justify-content: space-between; margin-bottom: 8px; font-size: 9.5pt; background: #f8fafc; padding: 6px 10px; border-radius: 4px; border: 1px solid #e2e8f0;">
      <div><strong>Student ID:</strong> <span style="font-family: monospace; font-size: 11pt; color: #0284c7; font-weight: 700;">${escapeHtml(student.student_code)}</span></div>
      <div><strong>Date of Admission:</strong> ${formatBusinessDate(student.admission_date)}</div>
      <div><strong>Status:</strong> <span style="text-transform: capitalize; color: #059669; font-weight: 600;">${student.status}</span></div>
    </div>

    <!-- Personal Particulars -->
    <div class="section-title">1. Student Information</div>
    <div class="meta-grid">
      <div class="meta-row"><span class="meta-label">Full Name:</span><span class="meta-value">${escapeHtml(student.name)} ${student.name_bn ? `(${escapeHtml(student.name_bn)})` : ""}</span></div>
      <div class="meta-row"><span class="meta-label">Gender / DOB:</span><span class="meta-value" style="text-transform: capitalize;">${escapeHtml(student.gender)}, ${student.dob}</span></div>
      <div class="meta-row"><span class="meta-label">Contact Phone:</span><span class="meta-value" style="font-family: monospace;">${escapeHtml(student.phone_norm)}</span></div>
      <div class="meta-row"><span class="meta-label">Class Level:</span><span class="meta-value">${escapeHtml(student.class_level || "Not specified")}</span></div>
      <div class="meta-row"><span class="meta-label">Blood Group:</span><span class="meta-value">${escapeHtml(student.blood_group || "--")}</span></div>
      <div class="meta-row"><span class="meta-label">Institution Roll:</span><span class="meta-value">${escapeHtml(student.current_school_roll || "--")}</span></div>
      <div class="meta-row" style="grid-column: span 2;"><span class="meta-label">Present Address:</span><span class="meta-value">${escapeHtml(student.present_address)}</span></div>
    </div>

    <!-- Guardian Information -->
    <div class="section-title">2. Primary Guardian Information</div>
    <div class="meta-grid">
      <div class="meta-row"><span class="meta-label">Guardian Name:</span><span class="meta-value">${primaryGuardian ? escapeHtml(primaryGuardian.name) : "Not recorded"}</span></div>
      <div class="meta-row"><span class="meta-label">Relationship:</span><span class="meta-value" style="text-transform: capitalize;">${primaryGuardian ? escapeHtml(primaryGuardian.relationship) : "--"}</span></div>
      <div class="meta-row"><span class="meta-label">Primary Mobile:</span><span class="meta-value" style="font-family: monospace;">${primaryGuardian ? escapeHtml(primaryGuardian.phone_norm) : "--"}</span></div>
      <div class="meta-row"><span class="meta-label">SMS Alerts:</span><span class="meta-value">${primaryGuardian && primaryGuardian.receives_sms ? "Enabled" : "Disabled"}</span></div>
    </div>

    <!-- Enrolled Batches -->
    <div class="section-title">3. Academic Enrollment</div>
    <table>
      <thead>
        <tr>
          <th>Batch / Course Name</th>
          <th style="text-align: center;">Roll Number</th>
          <th style="text-align: right;">Commencement Date</th>
        </tr>
      </thead>
      <tbody>
        ${batchesHtml}
      </tbody>
    </table>

    <!-- Financial Schedule & Payment -->
    <div class="section-title">4. Admission Fees & Payment Summary</div>
    <table>
      <thead>
        <tr>
          <th>Fee Description</th>
          <th style="text-align: right;">Gross Fee</th>
          <th style="text-align: right;">Concession</th>
          <th style="text-align: right;">Payable Due</th>
        </tr>
      </thead>
      <tbody>
        ${feesHtml}
      </tbody>
    </table>

    <div class="finance-summary">
      <div class="finance-card">
        <div class="finance-row"><span>Total Payable:</span><span style="font-weight: 700;">${formatTaka(totalBilled)}</span></div>
        <div class="finance-row"><span>Paid at Admission:</span><span style="font-weight: 700; color: #059669;">${formatTaka(totalPaid)}</span></div>
        <div class="finance-row" style="border-top: 1px solid #cbd5e1; margin-top: 4px; padding-top: 4px; font-weight: 700;">
          <span>Net Balance:</span>
          <span style="color: ${balanceDue > 0 ? "#b91c1c" : "#059669"};">${balanceDue > 0 ? `${formatTaka(balanceDue)} Due` : "Fully Paid"}</span>
        </div>
        ${payment ? `<div style="font-size: 8pt; color: #64748b; margin-top: 4px; text-align: right;">Receipt: ${escapeHtml(payment.receipt_no)} (${escapeHtml(payment.method)})</div>` : ""}
      </div>
    </div>

    <!-- Declaration -->
    <div class="declaration">
      <strong>Guardian & Student Declaration:</strong> I certify that all particulars given above are correct to the best of my knowledge. We agree to abide by the academic rules, attendance standards, and fee policies of ${escapeHtml(institutionName)}.
    </div>

    <!-- Signatures -->
    <div class="signatures">
      <div class="sig-line">Student Signature</div>
      <div class="sig-line">Parent / Guardian Signature</div>
      <div class="sig-line">Authorized Center Director</div>
    </div>
  </div>
</body>
</html>
  `.trim();
}

function escapeHtml(str: string | null | undefined): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
