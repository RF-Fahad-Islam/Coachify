import { formatTaka, formatBusinessDate } from "@/lib/utils";
import { INSTITUTION_LOGO_SVG } from "./id-card-template";

export interface LedgerEntry {
  id: string;
  entry_date: string;
  reference_no?: string | null;
  description: string;
  debit_poisha: number; // Charges / dues
  credit_poisha: number; // Payments / credits
  running_balance_poisha: number;
}

export interface FinancialLedgerData {
  account_name: string;
  account_code: string;
  phone?: string | null;
  batch_name?: string | null;
  date_from?: string | null;
  date_to?: string | null;
  entries: LedgerEntry[];
  total_debit_poisha: number;
  total_credit_poisha: number;
  net_balance_poisha: number;
}

export interface FinancialLedgerPrintOptions {
  institutionName?: string;
  institutionNameBn?: string;
  signatureTitle?: string;
  showWatermark?: boolean;
}

const DEFAULT_INSTITUTION_NAME = "COACHIFY ACADEMY";
const DEFAULT_INSTITUTION_NAME_BN = "কোচিফাই একাডেমি";
const DEFAULT_SIGNATURE_TITLE = "Accounts & Finance Officer";

function escapeHtml(str?: string | null): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateFinancialLedgerHtml(
  data: FinancialLedgerData,
  options?: FinancialLedgerPrintOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const signatureTitle = escapeHtml(options?.signatureTitle || DEFAULT_SIGNATURE_TITLE);

  const rowsHtml =
    data.entries.length > 0
      ? data.entries
          .map(
            (entry, idx) => `
        <tr style="border-bottom: 1px solid #e2e8f0; font-size: 8.5pt;">
          <td style="padding: 5px 6px; text-align: center; color: #64748b;">${idx + 1}</td>
          <td style="padding: 5px 6px; text-align: center; color: #334155;">${formatBusinessDate(entry.entry_date)}</td>
          <td style="padding: 5px 6px; text-align: center; font-family: monospace; font-weight: 600;">${escapeHtml(entry.reference_no || "--")}</td>
          <td style="padding: 5px 8px; font-weight: 500; color: #0f172a;">${escapeHtml(entry.description)}</td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace; color: #b91c1c; font-weight: 600;">
            ${entry.debit_poisha > 0 ? formatTaka(entry.debit_poisha) : "--"}
          </td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace; color: #059669; font-weight: 600;">
            ${entry.credit_poisha > 0 ? formatTaka(entry.credit_poisha) : "--"}
          </td>
          <td style="padding: 5px 8px; text-align: right; font-family: monospace; font-weight: 700; color: ${entry.running_balance_poisha > 0 ? "#b91c1c" : "#0f172a"};">
            ${formatTaka(entry.running_balance_poisha)}
          </td>
        </tr>
      `
          )
          .join("")
      : `<tr><td colspan="7" style="padding: 16px; text-align: center; color: #64748b; font-size: 9pt;">No ledger records found for this period</td></tr>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Financial Ledger - ${escapeHtml(data.account_code)}</title>
  <style>
    @page {
      size: A4 portrait;
      margin: 12mm 14mm;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", "Noto Sans Bengali", Arial, sans-serif;
      font-size: 9pt;
      line-height: 1.4;
      color: #0f172a;
      background: #ffffff;
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
  </style>
</head>
<body>
  <!-- Header -->
  <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #0f172a; padding-bottom: 8px; margin-bottom: 10px;">
    <div style="width: 36px; height: 36px; flex-shrink: 0;">
      ${INSTITUTION_LOGO_SVG}
    </div>
    <div style="text-align: center; flex-grow: 1; padding: 0 10px;">
      <div style="font-size: 15pt; font-weight: 900; letter-spacing: 0.5px; text-transform: uppercase;">${instName}</div>
      ${instNameBn ? `<div style="font-size: 9pt; color: #475569;">${instNameBn}</div>` : ""}
      <div style="font-size: 8.5pt; font-weight: 800; color: #0284c7; text-transform: uppercase; margin-top: 2px;">
        FINANCIAL LEDGER STATEMENT / হিসাব বিবরণী
      </div>
    </div>
    <div style="text-align: right; font-size: 7.5pt; color: #64748b;">
      <div>Generated: ${formatBusinessDate(new Date().toISOString())}</div>
      <div style="font-weight: 700; color: #0f172a;">Confidential Statement</div>
    </div>
  </div>

  <!-- Watermark Background -->
  ${
    options?.showWatermark
      ? `<div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(-25deg); opacity: 0.04; font-size: 32pt; font-weight: 900; pointer-events: none; text-transform: uppercase; white-space: nowrap;">${instName}</div>`
      : ""
  }

  <!-- Account Meta Card -->
  <div style="display: flex; justify-content: space-between; background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 4px; padding: 8px 12px; margin-bottom: 12px; font-size: 8.5pt;">
    <div>
      <div style="margin-bottom: 2px;">
        <span style="color: #64748b; font-weight: 600;">Account / Name: </span>
        <span style="font-weight: 800; color: #0f172a;">${escapeHtml(data.account_name)}</span>
      </div>
      <div>
        <span style="color: #64748b; font-weight: 600;">Account ID: </span>
        <span style="font-family: monospace; font-weight: 700; color: #0369a1;">${escapeHtml(data.account_code)}</span>
        ${
          data.batch_name
            ? `<span style="color: #64748b; margin-left: 12px; font-weight: 600;">Batch: </span>
               <span style="font-weight: 600; color: #1e293b;">${escapeHtml(data.batch_name)}</span>`
            : ""
        }
      </div>
    </div>
    <div style="text-align: right;">
      <div style="margin-bottom: 2px;">
        <span style="color: #64748b; font-weight: 600;">Period: </span>
        <span style="font-weight: 600;">
          ${data.date_from ? formatBusinessDate(data.date_from) : "All-time"} &mdash; ${data.date_to ? formatBusinessDate(data.date_to) : "Present"}
        </span>
      </div>
      ${
        data.phone
          ? `<div>
              <span style="color: #64748b; font-weight: 600;">Phone: </span>
              <span style="font-family: monospace;">${escapeHtml(data.phone)}</span>
            </div>`
          : ""
      }
    </div>
  </div>

  <!-- Transactions Table -->
  <table style="width: 100%; border-collapse: collapse; text-align: left; margin-bottom: 12px;">
    <thead>
      <tr style="background: #0f172a; color: #ffffff; font-size: 8pt; font-weight: 700; text-transform: uppercase;">
        <th style="padding: 6px; width: 28px; text-align: center;">#</th>
        <th style="padding: 6px; width: 75px; text-align: center;">Date</th>
        <th style="padding: 6px; width: 85px; text-align: center;">Reference</th>
        <th style="padding: 6px;">Particulars / বিবরণ</th>
        <th style="padding: 6px 8px; width: 85px; text-align: right;">Debit (Due)</th>
        <th style="padding: 6px 8px; width: 85px; text-align: right;">Credit (Paid)</th>
        <th style="padding: 6px 8px; width: 95px; text-align: right;">Balance</th>
      </tr>
    </thead>
    <tbody>
      ${rowsHtml}
    </tbody>
  </table>

  <!-- Financial Summary Strip -->
  <div style="display: flex; justify-content: flex-end; margin-bottom: 20px;">
    <div style="width: 75mm; border: 1.5px solid #0f172a; border-radius: 4px; overflow: hidden; background: #f8fafc;">
      <div style="display: flex; justify-content: space-between; padding: 4px 8px; font-size: 8.5pt; border-bottom: 1px solid #e2e8f0;">
        <span style="color: #64748b; font-weight: 600;">Total Debited (Billed):</span>
        <span style="font-family: monospace; font-weight: 700; color: #b91c1c;">${formatTaka(data.total_debit_poisha)}</span>
      </div>
      <div style="display: flex; justify-content: space-between; padding: 4px 8px; font-size: 8.5pt; border-bottom: 1px solid #e2e8f0;">
        <span style="color: #64748b; font-weight: 600;">Total Credited (Paid):</span>
        <span style="font-family: monospace; font-weight: 700; color: #059669;">${formatTaka(data.total_credit_poisha)}</span>
      </div>
      <div style="display: flex; justify-content: space-between; padding: 6px 8px; font-size: 9.5pt; font-weight: 900; background: #0f172a; color: #ffffff;">
        <span>NET OUTSTANDING:</span>
        <span style="font-family: monospace;">${formatTaka(data.net_balance_poisha)}</span>
      </div>
    </div>
  </div>

  <!-- Signatures -->
  <div style="display: flex; justify-content: space-between; align-items: flex-end; padding-top: 14px; margin-top: auto;">
    <div style="text-align: center; width: 50mm;">
      <div style="border-bottom: 1px dashed #94a3b8; width: 100%; height: 14px;"></div>
      <div style="font-size: 7pt; font-weight: 600; color: #64748b; margin-top: 2px;">
        Account Holder / Student Signature
      </div>
    </div>
    <div style="text-align: center; width: 60mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 14px;"></div>
      <div style="font-size: 7.5pt; font-weight: 800; color: #0f172a; margin-top: 2px; text-transform: uppercase;">
        ${signatureTitle}
      </div>
    </div>
  </div>
</body>
</html>
  `.trim();
}

export function FinancialLedgerPrintable({
  data,
  options,
}: {
  data: FinancialLedgerData;
  options?: FinancialLedgerPrintOptions;
}) {
  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-4">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing A4 Financial Ledger Statement ({data.entries.length} entries)
      </div>
      <div
        className="bg-white p-6 shadow-sm border border-border rounded-md max-w-4xl w-full"
        dangerouslySetInnerHTML={{ __html: generateFinancialLedgerHtml(data, options) }}
      />
    </div>
  );
}
