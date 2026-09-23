import type { CashDaySummary } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";

export interface ThermalCashCloseOptions {
  institutionName?: string;
  institutionNameBn?: string;
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

export function generateThermalCashCloseHtml(
  summary: CashDaySummary,
  options?: ThermalCashCloseOptions
): string {
  const instName = escapeHtml(options?.institutionName || DEFAULT_INSTITUTION_NAME);
  const instNameBn = escapeHtml(
    options?.institutionNameBn || DEFAULT_INSTITUTION_NAME_BN
  );
  const signatureTitle = escapeHtml(
    options?.signatureTitle || "Center Manager / Supervisor"
  );

  const diff = summary.difference_poisha ?? 0;
  const diffStatus =
    diff === 0
      ? "BALANCED"
      : diff > 0
        ? `+${formatTaka(diff)} SURPLUS`
        : `-${formatTaka(-diff)} SHORTAGE`;

  const collectorsHtml =
    summary.collector_breakdown.length > 0
      ? summary.collector_breakdown
          .map(
            (c) => `
        <div style="display: flex; justify-content: space-between; font-size: 8pt; padding: 1.5px 0;">
          <span style="color: #1e293b; font-weight: 500;">${escapeHtml(c.collector_name)} (${c.receipt_count})</span>
          <span style="font-family: monospace; font-weight: 600;">${formatTaka(c.total_amount_poisha)}</span>
        </div>
      `
          )
          .join("")
      : `<div style="font-size: 7.5pt; color: #64748b; text-align: center;">No collector receipts</div>`;

  const digitalHtml =
    summary.digital_collections.length > 0
      ? summary.digital_collections
          .map(
            (d) => `
        <div style="display: flex; justify-content: space-between; font-size: 8pt; padding: 1.5px 0;">
          <span style="color: #1e293b; font-weight: 600; text-transform: uppercase;">${escapeHtml(d.method)} (${d.count})</span>
          <span style="font-family: monospace; font-weight: 600; color: #0284c7;">${formatTaka(d.total_amount_poisha)}</span>
        </div>
      `
          )
          .join("")
      : `<div style="font-size: 7.5pt; color: #64748b; text-align: center;">No digital transactions</div>`;

  const totalDigitalPoisha = summary.digital_collections.reduce(
    (sum, d) => sum + d.total_amount_poisha,
    0
  );
  const totalIntakePoisha = summary.cash_in_poisha + totalDigitalPoisha;

  return `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Daily Cash Close Summary - ${escapeHtml(summary.business_date)}</title>
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
  <!-- Header -->
  <div style="text-align: center; border-bottom: 1.5px dashed #0f172a; padding-bottom: 6px; margin-bottom: 6px;">
    <div style="font-size: 11pt; font-weight: 900; text-transform: uppercase;">${instName}</div>
    ${instNameBn ? `<div style="font-size: 8pt; color: #475569;">${instNameBn}</div>` : ""}
    <div style="display: inline-block; background: #0f172a; color: #ffffff; padding: 1px 8px; border-radius: 2px; font-size: 7pt; font-weight: 800; margin-top: 3px; text-transform: uppercase;">
      DAILY CASH CLOSE / দৈনিক ক্যাশ ক্লোজ
    </div>
  </div>

  <!-- Session & Date Metadata -->
  <div style="border: 1px solid #0f172a; border-radius: 4px; padding: 4px 6px; margin-bottom: 6px; font-size: 8.5pt;">
    <div style="display: flex; justify-content: space-between;">
      <span style="color: #64748b;">Business Date:</span>
      <span style="font-weight: 700;">${formatBusinessDate(summary.business_date)}</span>
    </div>
    <div style="display: flex; justify-content: space-between;">
      <span style="color: #64748b;">Closed By:</span>
      <span style="font-weight: 700;">${escapeHtml(summary.closed_by || "Head Cashier")}</span>
    </div>
    <div style="display: flex; justify-content: space-between;">
      <span style="color: #64748b;">Closed At:</span>
      <span style="font-family: monospace; font-size: 7.5pt;">${escapeHtml(summary.closed_at || "--")}</span>
    </div>
  </div>

  <!-- Cash Reconciliation Box -->
  <div style="border: 1.5px solid #0f172a; background: #f8fafc; border-radius: 4px; padding: 6px; margin-bottom: 6px; font-size: 8.5pt;">
    <div style="font-size: 7.5pt; font-weight: 800; text-transform: uppercase; color: #475569; border-bottom: 1px solid #cbd5e1; padding-bottom: 2px; margin-bottom: 4px;">
      Physical Cash In Drawer
    </div>
    <div style="display: flex; justify-content: space-between; padding: 1px 0;">
      <span>Opening Float:</span>
      <span style="font-family: monospace; font-weight: 600;">${formatTaka(summary.opening_cash_poisha)}</span>
    </div>
    <div style="display: flex; justify-content: space-between; padding: 1px 0;">
      <span>Cash Collections (+):</span>
      <span style="font-family: monospace; font-weight: 700; color: #059669;">${formatTaka(summary.cash_in_poisha)}</span>
    </div>
    <div style="display: flex; justify-content: space-between; padding: 1px 0;">
      <span>Cash Expenses (-):</span>
      <span style="font-family: monospace; font-weight: 700; color: #b91c1c;">${formatTaka(summary.cash_out_poisha)}</span>
    </div>
    <div style="border-top: 1px dashed #0f172a; margin-top: 3px; padding-top: 3px; display: flex; justify-content: space-between; font-weight: 700;">
      <span>Expected Physical Cash:</span>
      <span style="font-family: monospace; font-size: 9.5pt;">${formatTaka(summary.expected_closing_poisha)}</span>
    </div>
    <div style="display: flex; justify-content: space-between; font-weight: 800; color: #0f172a; margin-top: 2px;">
      <span>Actual Counted Cash:</span>
      <span style="font-family: monospace; font-size: 10.5pt;">${formatTaka(summary.counted_closing_poisha ?? summary.expected_closing_poisha)}</span>
    </div>
    <div style="border-top: 1px solid #0f172a; margin-top: 3px; padding-top: 3px; display: flex; justify-content: space-between; font-weight: 800;">
      <span>Status:</span>
      <span style="font-size: 8pt; color: ${diff === 0 ? "#059669" : diff > 0 ? "#0284c7" : "#b91c1c"};">
        ${diffStatus}
      </span>
    </div>
  </div>

  <!-- Digital Payments Breakdown -->
  <div style="border: 1px solid #cbd5e1; border-radius: 4px; padding: 5px 6px; margin-bottom: 6px;">
    <div style="font-size: 7.5pt; font-weight: 800; text-transform: uppercase; color: #475569; border-bottom: 1px solid #e2e8f0; padding-bottom: 2px; margin-bottom: 3px; display: flex; justify-content: space-between;">
      <span>Digital & Bank Txns</span>
      <span>Amount</span>
    </div>
    ${digitalHtml}
    <div style="border-top: 1px dotted #cbd5e1; margin-top: 3px; padding-top: 2px; display: flex; justify-content: space-between; font-weight: 700; font-size: 8pt;">
      <span>Total Digital:</span>
      <span style="font-family: monospace; color: #0284c7;">${formatTaka(totalDigitalPoisha)}</span>
    </div>
  </div>

  <!-- Collector Summary -->
  <div style="border: 1px solid #cbd5e1; border-radius: 4px; padding: 5px 6px; margin-bottom: 8px;">
    <div style="font-size: 7.5pt; font-weight: 800; text-transform: uppercase; color: #475569; border-bottom: 1px solid #e2e8f0; padding-bottom: 2px; margin-bottom: 3px; display: flex; justify-content: space-between;">
      <span>Counter Staff</span>
      <span>Collected</span>
    </div>
    ${collectorsHtml}
  </div>

  <!-- Total Intake Grand Total -->
  <div style="border: 1.5px solid #0f172a; background: #0f172a; color: #ffffff; border-radius: 4px; padding: 6px; text-align: center; margin-bottom: 12px;">
    <div style="font-size: 7.5pt; font-weight: 700; text-transform: uppercase; color: #94a3b8;">
      Total Gross Inflow (Cash + Digital)
    </div>
    <div style="font-size: 13pt; font-weight: 900; font-family: monospace; line-height: 1.2;">
      ${formatTaka(totalIntakePoisha)}
    </div>
  </div>

  <!-- Dual Signatures -->
  <div style="display: flex; justify-content: space-between; align-items: flex-end; padding-top: 8px; margin-top: 10px;">
    <div style="text-align: center; width: 32mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 10px;"></div>
      <div style="font-size: 6.5pt; font-weight: 700; margin-top: 2px;">Cashier Signature</div>
    </div>
    <div style="text-align: center; width: 32mm;">
      <div style="border-bottom: 1px solid #0f172a; width: 100%; height: 10px;"></div>
      <div style="font-size: 6.5pt; font-weight: 700; margin-top: 2px;">${signatureTitle}</div>
    </div>
  </div>
</body>
</html>
  `.trim();
}

export function ThermalCashCloseSlip({
  summary,
  options,
}: {
  summary: CashDaySummary;
  options?: ThermalCashCloseOptions;
}) {
  return (
    <div className="p-4 bg-muted/40 rounded-lg overflow-auto flex flex-col items-center gap-4">
      <div className="text-xs text-muted-foreground font-medium">
        Previewing 80mm Continuous Roll Daily Cash Close Slip
      </div>
      <div
        className="bg-white p-4 shadow-sm border border-border rounded-sm max-w-[320px] w-full"
        dangerouslySetInnerHTML={{
          __html: generateThermalCashCloseHtml(summary, options),
        }}
      />
    </div>
  );
}
