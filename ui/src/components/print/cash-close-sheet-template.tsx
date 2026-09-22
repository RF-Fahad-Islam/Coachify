import type { CashDaySummary } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";

const ACADEMIC_EMBLEM_SVG = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="46" height="46" style="display: block; color: #111;">
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

export function generateCashCloseSheetHtml(
  summary: CashDaySummary,
  institutionName = "Coachify Institute"
): string {
  const diff = summary.difference_poisha ?? 0;
  const diffColor = diff === 0 ? "#059669" : diff > 0 ? "#2563eb" : "#b91c1c";
  const diffLabel =
    diff === 0
      ? "Balanced (No discrepancy)"
      : diff > 0
        ? `+${formatTaka(diff)} Surplus`
        : `-${formatTaka(-diff)} Shortage`;

  const collectorsHtml =
    summary.collector_breakdown.length > 0
      ? summary.collector_breakdown
          .map(
            (c) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 8.5pt;">
          <td style="padding: 4px 8px; font-weight: 500;">${escapeHtml(c.collector_name)}</td>
          <td style="padding: 4px 8px; text-align: center;">${c.receipt_count} receipts</td>
          <td style="padding: 4px 8px; text-align: right; font-family: monospace; font-weight: 600;">${formatTaka(c.total_amount_poisha)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="3" style="padding: 6px; text-align: center; color: #64748b; font-size: 8.5pt;">No collections recorded</td></tr>`;

  const digitalHtml =
    summary.digital_collections.length > 0
      ? summary.digital_collections
          .map(
            (d) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 8.5pt;">
          <td style="padding: 4px 8px; text-transform: uppercase; font-weight: 600;">${escapeHtml(d.method)}</td>
          <td style="padding: 4px 8px; text-align: center;">${d.count} txns</td>
          <td style="padding: 4px 8px; text-align: right; font-family: monospace; font-weight: 600; color: #0284c7;">${formatTaka(d.total_amount_poisha)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="3" style="padding: 6px; text-align: center; color: #64748b; font-size: 8.5pt;">No digital payments</td></tr>`;

  const expensesHtml =
    summary.expenses_by_category.length > 0
      ? summary.expenses_by_category
          .map(
            (e) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 8.5pt;">
          <td style="padding: 4px 8px; font-weight: 500;">${escapeHtml(e.category_name)}</td>
          <td style="padding: 4px 8px; text-align: center;">${e.count} vouchers</td>
          <td style="padding: 4px 8px; text-align: right; font-family: monospace; font-weight: 600; color: #b91c1c;">-${formatTaka(e.total_amount_poisha)}</td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="3" style="padding: 6px; text-align: center; color: #64748b; font-size: 8.5pt;">No expenses recorded</td></tr>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Daily Cash Close Sheet - ${escapeHtml(summary.business_date)}</title>
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
      font-size: 9.5pt;
      line-height: 1.35;
      color: #0f172a;
      background: #fff;
    }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 8px;
      margin-bottom: 12px;
    }
    .title-box {
      text-align: center;
      flex-grow: 1;
    }
    .inst-name {
      font-size: 16pt;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .sheet-badge {
      display: inline-block;
      margin-top: 3px;
      padding: 2px 10px;
      background: #0f172a;
      color: #fff;
      font-size: 8.5pt;
      font-weight: 700;
      letter-spacing: 0.5px;
      border-radius: 3px;
      text-transform: uppercase;
    }
    .status-badge {
      padding: 3px 8px;
      border-radius: 4px;
      font-weight: 700;
      font-size: 8pt;
      text-transform: uppercase;
    }
    .status-closed {
      background: #dcfce7;
      color: #166534;
      border: 1px solid #86efac;
    }
    .status-open {
      background: #fef9c3;
      color: #854d0e;
      border: 1px solid #fde047;
    }
    .drawer-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
      margin-bottom: 12px;
    }
    .reconciliation-card {
      background: #f8fafc;
      border: 1.5px solid #cbd5e1;
      border-radius: 4px;
      padding: 10px 14px;
    }
    .recon-row {
      display: flex;
      justify-content: space-between;
      padding: 3.5px 0;
      font-size: 9pt;
    }
    .recon-total {
      border-top: 1.5px solid #94a3b8;
      margin-top: 5px;
      padding-top: 5px;
      font-weight: 700;
      font-size: 10.5pt;
    }
    .discrepancy-card {
      border-top: 2px dashed #94a3b8;
      margin-top: 8px;
      padding-top: 6px;
    }
    .sec-head {
      font-size: 9pt;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      background: #f1f5f9;
      padding: 3px 8px;
      border-left: 3px solid #0f172a;
      margin-top: 10px;
      margin-bottom: 4px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 6px;
    }
    th {
      background: #f8fafc;
      padding: 4px 8px;
      text-align: left;
      font-size: 7.5pt;
      text-transform: uppercase;
      color: #475569;
      border-bottom: 1.5px solid #cbd5e1;
    }
    .note-box {
      background: #fafafa;
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 6px 10px;
      font-size: 8.5pt;
      margin-top: 8px;
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
        <div class="inst-name">${escapeHtml(institutionName)}</div>
        <div class="sheet-badge">Daily Cash Drawer Reconciliation Sheet</div>
      </div>
      <div style="text-align: right; min-width: 110px;">
        <div style="font-size: 9pt; font-weight: 600; margin-bottom: 4px;">${formatBusinessDate(summary.business_date)}</div>
        <span class="status-badge ${summary.is_closed ? "status-closed" : "status-open"}">
          ${summary.is_closed ? "DAY CLOSED & LOCKED" : "DAY IN PROGRESS (OPEN)"}
        </span>
      </div>
    </div>

    <!-- Main Drawer Reconciliation -->
    <div class="reconciliation-card">
      <div style="font-weight: 700; text-transform: uppercase; font-size: 9.5pt; color: #334155; margin-bottom: 6px;">
        Cash Drawer Balancing (Physical Cash Only)
      </div>
      <div class="recon-row">
        <span>Opening Drawer Cash (Carried Forward):</span>
        <span style="font-family: monospace; font-weight: 600;">${formatTaka(summary.opening_cash_poisha)}</span>
      </div>
      <div class="recon-row">
        <span style="color: #059669;">+ Cash Received (Student Tuition & Fees):</span>
        <span style="font-family: monospace; font-weight: 600; color: #059669;">+${formatTaka(summary.cash_in_poisha)}</span>
      </div>
      <div class="recon-row">
        <span style="color: #b91c1c;">- Cash Paid Out (Voucher Expenses & Refunds):</span>
        <span style="font-family: monospace; font-weight: 600; color: #b91c1c;">-${formatTaka(summary.cash_out_poisha)}</span>
      </div>

      <div class="recon-row recon-total">
        <span>Expected Closing Cash Drawer:</span>
        <span style="font-family: monospace; font-weight: 800; font-size: 11pt;">${formatTaka(summary.expected_closing_poisha)}</span>
      </div>

      ${
        summary.counted_closing_poisha !== null &&
        summary.counted_closing_poisha !== undefined
          ? `
        <div class="discrepancy-card">
          <div class="recon-row" style="font-size: 10pt; font-weight: 700;">
            <span>Actual Counted Cash in Drawer:</span>
            <span style="font-family: monospace; font-weight: 800; font-size: 11pt;">${formatTaka(summary.counted_closing_poisha)}</span>
          </div>
          <div class="recon-row" style="font-size: 10.5pt; font-weight: 800; color: ${diffColor};">
            <span>Cash Drawer Discrepancy:</span>
            <span>${diffLabel}</span>
          </div>
        </div>
      `
          : `
        <div style="font-size: 8.5pt; color: #64748b; font-style: italic; margin-top: 6px;">
          Drawer count not yet finalized.
        </div>
      `
      }
    </div>

    ${
      summary.note
        ? `
      <div class="note-box">
        <strong>Cashier Shift Note:</strong> ${escapeHtml(summary.note)}
      </div>
    `
        : ""
    }

    <!-- Breakdowns Grid -->
    <div class="drawer-grid" style="margin-top: 8px;">
      <!-- Digital Collections (bKash / Nagad / Bank) -->
      <div>
        <div class="sec-head">Digital & Bank Collections</div>
        <table>
          <thead>
            <tr>
              <th>Channel</th>
              <th style="text-align: center;">Transactions</th>
              <th style="text-align: right;">Amount (৳)</th>
            </tr>
          </thead>
          <tbody>
            ${digitalHtml}
          </tbody>
        </table>
      </div>

      <!-- Collector Breakdown -->
      <div>
        <div class="sec-head">Cashier Breakdown</div>
        <table>
          <thead>
            <tr>
              <th>Collector Name</th>
              <th style="text-align: center;">Receipts</th>
              <th style="text-align: right;">Collected (৳)</th>
            </tr>
          </thead>
          <tbody>
            ${collectorsHtml}
          </tbody>
        </table>
      </div>
    </div>

    <!-- Expenses Breakdown -->
    <div class="sec-head">Category Petty Cash Expenses & Disbursed Refunds</div>
    <table>
      <thead>
        <tr>
          <th>Category Name</th>
          <th style="text-align: center;">Vouchers Count</th>
          <th style="text-align: right;">Disbursed Total (৳)</th>
        </tr>
      </thead>
      <tbody>
        ${expensesHtml}
      </tbody>
    </table>

    <div style="font-size: 8pt; color: #64748b; margin-top: 10px;">
      ${summary.is_closed && summary.closed_at ? `Locked on ${formatBusinessDate(summary.closed_at.slice(0, 10))}. Modifications to transactions on this date are permanently barred.` : "Preliminary live drawer report. Final sign-off required upon shift close."}
    </div>

    <!-- Signatures -->
    <div class="signatures">
      <div class="sig-line">Counter Cashier Sign-off</div>
      <div class="sig-line">Accounts Officer Verification</div>
      <div class="sig-line">Director / Owner Approval</div>
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
