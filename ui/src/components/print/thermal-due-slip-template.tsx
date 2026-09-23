import type { DueStatementData } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";

export interface ThermalDueSlipOptions {
  institutionName?: string;
  institutionNameBn?: string;
  customFooter?: string;
  signatureTitle?: string;
}

const DEFAULT_INSTITUTION_NAME = "COACHIFY ACADEMY";
const DEFAULT_INSTITUTION_NAME_BN = "কোচিফাই একাডেমি";

function escapeHtml(str?: string | null): string {
  if (!str) return "";
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

export function generateThermalDueSlipHtml(
  data: DueStatementData,
  options?: ThermalDueSlipOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const footerText = escapeHtml(
    options?.customFooter ||
      "Please clear outstanding dues before the 10th of next month."
  );
  const signatureTitle = escapeHtml(options?.signatureTitle || "Counter Officer");

  const dueItems = data.itemized_charges.filter((c) => c.balance_poisha > 0);

  const dueRowsHtml =
    dueItems.length > 0
      ? dueItems
          .map(
            (c) => `
        <div style="display: flex; justify-content: space-between; padding: 2px 0; font-size: 8.5pt; border-bottom: 1px dotted #cbd5e1;">
          <div style="max-width: 48mm; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <div style="font-weight: 600; color: #0f172a;">${escapeHtml(c.fee_head_name)}</div>
            <div style="font-size: 7.5pt; color: #64748b;">${escapeHtml(c.billing_month)} (Due: ${formatBusinessDate(c.due_date)})</div>
          </div>
          <div style="font-family: monospace; font-weight: 700; color: #b91c1c; text-align: right;">
            ${formatTaka(c.balance_poisha)}
          </div>
        </div>
      `
          )
          .join("")
      : `<div style="text-align: center; padding: 6px 0; color: #059669; font-size: 8.5pt; font-weight: 600;">✓ All fees currently clear / কোনো বকেয়া নেই</div>`;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Due Slip - ${escapeHtml(data.student_code)}</title>
  <style>
    @page {
      size: 80mm auto;
      margin: 3mm 4mm;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", "Noto Sans Bengali", Arial, sans-serif;
      font-size: 9pt;
      line-height: 1.3;
      color: #0f172a;
      width: 72mm;
      margin: 0 auto;
      background: #ffffff;
      -webkit-print-color-adjust: exact;
      print-color-adjust: exact;
    }
  </style>
</head>
<body>
  <div style="text-align: center; border-bottom: 1.5px dashed #0f172a; padding-bottom: 6px; margin-bottom: 6px;">
    <div style="font-size: 11pt; font-weight: 900; text-transform: uppercase;">${instName}</div>
    ${instNameBn ? `<div style="font-size: 8pt; color: #475569;">${instNameBn}</div>` : ""}
    <div style="display: inline-block; background: #0f172a; color: #ffffff; padding: 1px 8px; border-radius: 2px; font-size: 7pt; font-weight: 800; margin-top: 3px; text-transform: uppercase;">
      FEE DUE REMINDER SLIP / বকেয়া নোটিশ
    </div>
  </div>

  <!-- Student Info Card -->
  <div style="border: 1px solid #0f172a; border-radius: 4px; padding: 4px 6px; margin-bottom: 6px; font-size: 8.5pt;">
    <div style="display: flex; justify-content: space-between;">
      <span style="color: #64748b;">Student ID:</span>
      <span style="font-family: monospace; font-weight: 800;">${escapeHtml(data.student_code)}</span>
    </div>
    <div style="display: flex; justify-content: space-between;">
      <span style="color: #64748b;">Name:</span>
      <span style="font-weight: 700; max-width: 45mm; text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
        ${escapeHtml(data.student_name)}
      </span>
    </div>
    ${
      data.student_name_bn
        ? `<div style="display: flex; justify-content: space-between;">
            <span style="color: #64748b;">নাম:</span>
            <span style="max-width: 45mm; text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              ${escapeHtml(data.student_name_bn)}
            </span>
          </div>`
        : ""
    }
    ${
      data.batch_name
        ? `<div style="display: flex; justify-content: space-between;">
            <span style="color: #64748b;">Batch:</span>
            <span style="font-weight: 600;">${escapeHtml(data.batch_name)}</span>
          </div>`
        : ""
    }
    ${
      data.guardian_phone
        ? `<div style="display: flex; justify-content: space-between;">
            <span style="color: #64748b;">Guardian:</span>
            <span style="font-family: monospace;">${escapeHtml(data.guardian_phone)}</span>
          </div>`
        : ""
    }
  </div>

  <!-- Itemized Due Breakdown -->
  <div style="margin-bottom: 6px;">
    <div style="font-size: 7.5pt; font-weight: 800; color: #475569; text-transform: uppercase; border-bottom: 1px solid #0f172a; padding-bottom: 2px; margin-bottom: 4px; display: flex; justify-content: space-between;">
      <span>Due Description</span>
      <span>Amount</span>
    </div>
    ${dueRowsHtml}
  </div>

  <!-- Total Outstanding Callout -->
  <div style="border: 1.5px solid #0f172a; background: #f8fafc; border-radius: 4px; padding: 6px; text-align: center; margin-bottom: 8px;">
    <div style="font-size: 8pt; font-weight: 700; text-transform: uppercase; color: #475569;">
      Total Outstanding / মোট বকেয়া
    </div>
    <div style="font-size: 15pt; font-weight: 900; font-family: monospace; color: ${data.net_due_poisha > 0 ? "#b91c1c" : "#059669"}; line-height: 1.2;">
      ${formatTaka(data.net_due_poisha)}
    </div>
  </div>

  <!-- Barcode for counter scanning -->
  <div style="text-align: center; margin-bottom: 6px;">
    <div style="font-family: monospace; font-size: 8pt; font-weight: 800; letter-spacing: 2px;">
      *${escapeHtml(data.student_code)}*
    </div>
    <div style="font-size: 6.5pt; color: #64748b;">Scan barcode at cashier desk</div>
  </div>

  <!-- Footer notice and signature -->
  <div style="font-size: 7pt; color: #475569; text-align: center; margin-bottom: 12px; line-height: 1.25;">
    ${footerText}
  </div>

  <div style="display: flex; justify-content: space-between; align-items: flex-end; padding-top: 4px;">
    <div style="font-size: 6.5pt; color: #94a3b8;">
      Printed: ${formatBusinessDate(new Date().toISOString())}
    </div>
    <div style="text-align: center; width: 30mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 10px;"></div>
      <div style="font-size: 6.5pt; font-weight: 700; margin-top: 2px;">${signatureTitle}</div>
    </div>
  </div>
</body>
</html>
  `.trim();
}

export function ThermalDueSlip({
  data,
  options,
}: {
  data: DueStatementData;
  options?: ThermalDueSlipOptions;
}) {
  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-4">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing 80mm Continuous Roll Due Slip
      </div>
      <div
        className="bg-white p-4 shadow-sm border border-border rounded-sm max-w-[320px] w-full"
        dangerouslySetInnerHTML={{ __html: generateThermalDueSlipHtml(data, options) }}
      />
    </div>
  );
}
