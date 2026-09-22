import type { DueStatementData } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";

const ACADEMIC_EMBLEM_SVG = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="50" height="50" style="display: block; color: #111;">
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

export function generateDueStatementHtml(data: DueStatementData): string {
  const chargesHtml =
    data.itemized_charges.length > 0
      ? data.itemized_charges
          .map(
            (c, idx) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 9pt;">
          <td style="padding: 5px 8px; text-align: center;">${idx + 1}</td>
          <td style="padding: 5px 8px; font-weight: 500;">
            ${escapeHtml(c.fee_head_name)}
            ${c.batch_name ? `<span style="color: #64748b; font-size: 8pt;"> (${escapeHtml(c.batch_name)})</span>` : ""}
          </td>
          <td style="padding: 5px 8px; text-align: center; font-family: monospace;">${escapeHtml(c.billing_month)}</td>
          <td style="padding: 5px 8px; text-align: center; color: #475569;">${formatBusinessDate(c.due_date)}</td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace;">${formatTaka(c.amount_poisha)}</td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace; color: #059669;">${formatTaka(c.paid_poisha)}</td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace; font-weight: 600; color: ${c.balance_poisha > 0 ? "#b91c1c" : "#059669"};">
            ${formatTaka(c.balance_poisha)}
          </td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="7" style="padding: 10px; text-align: center; color: #64748b;">No fees or charges billed</td></tr>`;

  const paymentsHtml =
    data.recent_payments.length > 0
      ? data.recent_payments
          .map(
            (p) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 8.5pt;">
          <td style="padding: 4px 8px; font-family: monospace; font-weight: 600;">${escapeHtml(p.receipt_number)}</td>
          <td style="padding: 4px 8px;">${formatBusinessDate(p.payment_date)}</td>
          <td style="padding: 4px 8px; text-transform: capitalize;">${escapeHtml(p.method)}</td>
          <td style="padding: 4px 8px; color: #64748b;">${escapeHtml(p.reference || "--")}</td>
          <td style="padding: 4px 8px; text-align: right; font-family: monospace; font-weight: 600; color: #059669;">${formatTaka(p.amount_poisha)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="5" style="padding: 8px; text-align: center; color: #64748b; font-size: 8.5pt;">No payments recorded yet</td></tr>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Student Due Statement - ${escapeHtml(data.student_code)}</title>
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
      background: #fff;
    }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 10px;
      margin-bottom: 12px;
    }
    .title-box {
      text-align: center;
      flex-grow: 1;
      padding: 0 12px;
    }
    .inst-name {
      font-size: 17pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: #0f172a;
    }
    .inst-sub {
      font-size: 8.5pt;
      color: #475569;
      margin-top: 1px;
    }
    .doc-badge {
      display: inline-block;
      margin-top: 4px;
      padding: 2px 10px;
      background: #0f172a;
      color: #fff;
      font-size: 9pt;
      font-weight: 700;
      letter-spacing: 0.5px;
      border-radius: 3px;
      text-transform: uppercase;
    }
    .info-card {
      background: #f8fafc;
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 8px 12px;
      margin-bottom: 12px;
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 6px 16px;
      font-size: 9pt;
    }
    .info-row {
      display: flex;
    }
    .info-label {
      width: 110px;
      color: #64748b;
      font-weight: 500;
      flex-shrink: 0;
    }
    .info-val {
      font-weight: 600;
      color: #0f172a;
    }
    .sec-head {
      font-size: 9.5pt;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      background: #f1f5f9;
      padding: 3px 8px;
      border-left: 3px solid #0f172a;
      margin-top: 12px;
      margin-bottom: 6px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin-top: 2px;
    }
    th {
      background: #f8fafc;
      padding: 5px 8px;
      text-align: left;
      font-size: 8pt;
      text-transform: uppercase;
      color: #475569;
      border-bottom: 1.5px solid #cbd5e1;
    }
    .summary-grid {
      display: flex;
      justify-content: space-between;
      margin-top: 10px;
      gap: 16px;
    }
    .words-box {
      flex: 1;
      background: #f8fafc;
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 8px 12px;
      font-size: 8.5pt;
    }
    .balance-card {
      width: 250px;
      background: #f8fafc;
      border: 1.5px solid #cbd5e1;
      border-radius: 4px;
      padding: 8px 12px;
      font-size: 9pt;
    }
    .bal-row {
      display: flex;
      justify-content: space-between;
      padding: 2.5px 0;
    }
    .net-due {
      border-top: 2px solid #0f172a;
      margin-top: 4px;
      padding-top: 4px;
      font-size: 11pt;
      font-weight: 800;
    }
    .instructions {
      margin-top: 12px;
      padding: 6px 10px;
      background: #fffbeb;
      border: 1px solid #fde68a;
      border-radius: 4px;
      font-size: 8pt;
      color: #92400e;
    }
    .signatures {
      display: flex;
      justify-content: space-between;
      margin-top: 45px;
      padding-top: 6px;
    }
    .sig-line {
      width: 170px;
      text-align: center;
      border-top: 1px solid #0f172a;
      font-size: 8pt;
      font-weight: 600;
      color: #1e293b;
      padding-top: 3px;
    }
  </style>
</head>
<body>
  <div class="container">
    <!-- Header -->
    <div class="header">
      <div style="flex-shrink: 0;">
        ${ACADEMIC_EMBLEM_SVG}
      </div>
      <div class="title-box">
        <div class="inst-name">${escapeHtml(data.institution_name)}</div>
        <div class="inst-sub">${escapeHtml(data.institution_address)} • Phone: ${escapeHtml(data.institution_phone)}</div>
        <div class="doc-badge">Statement of Account / বকেয়া বিবরণী</div>
      </div>
      <div style="text-align: right; font-size: 8pt; color: #64748b; min-width: 100px;">
        <div><strong>Date:</strong> ${formatBusinessDate(data.statement_date)}</div>
      </div>
    </div>

    <!-- Student Demographics Card -->
    <div class="info-card">
      <div class="info-row"><span class="info-label">Student ID:</span><span class="info-val" style="font-family: monospace; color: #0284c7;">${escapeHtml(data.student_code)}</span></div>
      <div class="info-row"><span class="info-label">Batch:</span><span class="info-val">${escapeHtml(data.batch_name || "--")}</span></div>
      <div class="info-row"><span class="info-label">Student Name:</span><span class="info-val">${escapeHtml(data.student_name)} ${data.student_name_bn ? `(${escapeHtml(data.student_name_bn)})` : ""}</span></div>
      <div class="info-row"><span class="info-label">Roll Number:</span><span class="info-val">${escapeHtml(data.roll_no || "--")}</span></div>
      <div class="info-row"><span class="info-label">Guardian:</span><span class="info-val">${escapeHtml(data.guardian_name || "--")} (${escapeHtml(data.guardian_phone || "--")})</span></div>
      <div class="info-row"><span class="info-label">Account Status:</span><span class="info-val" style="color: ${data.net_due_poisha > 0 ? "#b91c1c" : "#059669"};">${data.net_due_poisha > 0 ? "Dues Pending" : "Current / Paid"}</span></div>
    </div>

    <!-- Charges Table -->
    <div class="sec-head">Itemized Billed Dues / ফিস ও বকেয়া তালিকা</div>
    <table>
      <thead>
        <tr>
          <th style="width: 30px; text-align: center;">#</th>
          <th>Fee Description</th>
          <th style="text-align: center;">Period</th>
          <th style="text-align: center;">Due Date</th>
          <th style="text-align: right;">Billed (৳)</th>
          <th style="text-align: right;">Paid (৳)</th>
          <th style="text-align: right;">Outstanding (৳)</th>
        </tr>
      </thead>
      <tbody>
        ${chargesHtml}
      </tbody>
    </table>

    <!-- Payments Table -->
    <div class="sec-head" style="margin-top: 10px;">Recent Payment Vouchers / পরিশোধিত রসিদ সমূহ</div>
    <table>
      <thead>
        <tr>
          <th>Receipt No</th>
          <th>Date</th>
          <th>Method</th>
          <th>Reference / TrxID</th>
          <th style="text-align: right;">Amount (৳)</th>
        </tr>
      </thead>
      <tbody>
        ${paymentsHtml}
      </tbody>
    </table>

    <!-- Ledger Balances & Amount in Words -->
    <div class="summary-grid">
      <div class="words-box">
        <div style="font-weight: 600; color: #475569; margin-bottom: 2px;">Total Payable in Words (English):</div>
        <div style="font-style: italic; color: #0f172a; margin-bottom: 6px;">${escapeHtml(data.amount_in_words_en)}</div>

        <div style="font-weight: 600; color: #475569; margin-bottom: 2px;">কথায় (বাংলা):</div>
        <div style="font-weight: 600; color: #0f172a; font-family: 'Kalpurush', 'SolaimanLipi', sans-serif;">${escapeHtml(data.amount_in_words_bn)}</div>
      </div>

      <div class="balance-card">
        <div class="bal-row"><span>Total Billed:</span><span style="font-weight: 600;">${formatTaka(data.total_charged_poisha)}</span></div>
        <div class="bal-row"><span>Total Paid:</span><span style="font-weight: 600; color: #059669;">-${formatTaka(data.total_paid_poisha)}</span></div>
        ${data.total_concession_poisha > 0 ? `<div class="bal-row"><span>Concessions:</span><span style="font-weight: 600; color: #059669;">-${formatTaka(data.total_concession_poisha)}</span></div>` : ""}
        ${data.total_adjusted_poisha !== 0 ? `<div class="bal-row"><span>Adjustments:</span><span style="font-weight: 600;">${formatTaka(data.total_adjusted_poisha)}</span></div>` : ""}
        <div class="bal-row net-due">
          <span>Net Due Payable:</span>
          <span style="color: ${data.net_due_poisha > 0 ? "#b91c1c" : "#059669"};">${formatTaka(data.net_due_poisha)}</span>
        </div>
      </div>
    </div>

    ${
      data.payment_instructions
        ? `
      <div class="instructions">
        <strong>Payment Channels:</strong> ${escapeHtml(data.payment_instructions)}
      </div>
    `
        : `
      <div class="instructions">
        <strong>Payment Notice:</strong> Please pay outstanding fees at the counter desk or via official bKash / Nagad merchant accounts before the 10th of each calendar month.
      </div>
    `
    }

    <!-- Signatures -->
    <div class="signatures">
      <div class="sig-line">Prepared By (Accountant)</div>
      <div class="sig-line">Verified By (Cashier)</div>
      <div class="sig-line">Authorized Signature</div>
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
