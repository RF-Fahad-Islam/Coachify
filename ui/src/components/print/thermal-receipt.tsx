import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface ThermalReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

export function ThermalReceipt({ receipt, footerNote }: ThermalReceiptProps) {
  return (
    <div className="w-[76mm] max-w-[76mm] mx-auto bg-white text-black p-2 font-mono text-[11px] leading-tight select-text print:p-0 print:w-[72mm]">
      {/* Header */}
      <div className="text-center border-b border-dashed border-gray-400 pb-2 mb-2">
        {receipt.institution_name_bn && (
          <div className="font-bold text-sm tracking-wide mb-0.5">
            {receipt.institution_name_bn}
          </div>
        )}
        <div className="font-bold text-xs uppercase tracking-wider">
          {receipt.institution_name}
        </div>
        {receipt.institution_address && (
          <div className="text-[10px] text-gray-600 mt-0.5">
            {receipt.institution_address}
          </div>
        )}
        {receipt.institution_phone && (
          <div className="text-[10px] text-gray-600">
            Phone: {receipt.institution_phone}
          </div>
        )}
        <div className="inline-block border border-black font-bold px-2 py-0.5 mt-1.5 text-[10px] uppercase">
          {receipt.is_reprint ? "MONEY RECEIPT (REPRINT)" : "MONEY RECEIPT"}
        </div>
      </div>

      {/* Metadata */}
      <div className="space-y-0.5 mb-2 text-[10px] border-b border-dashed border-gray-400 pb-2">
        <div className="flex justify-between">
          <span className="text-gray-600">Receipt No:</span>
          <span className="font-bold">{receipt.receipt_no}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Date:</span>
          <span>{receipt.business_date}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Time:</span>
          <span>{receipt.received_at.slice(11, 19)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Cashier:</span>
          <span>{receipt.collector_name}</span>
        </div>
      </div>

      {/* Student Details */}
      <div className="space-y-0.5 mb-2 text-[10px] border-b border-dashed border-gray-400 pb-2">
        <div className="flex justify-between">
          <span className="text-gray-600">Student ID:</span>
          <span className="font-bold">{receipt.student_code}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Name:</span>
          <span className="font-bold text-right truncate max-w-[48mm]">
            {receipt.student_name}
          </span>
        </div>
        {receipt.student_name_bn && (
          <div className="flex justify-between">
            <span className="text-gray-600">বাংলা নাম:</span>
            <span className="font-medium text-right truncate max-w-[48mm]">
              {receipt.student_name_bn}
            </span>
          </div>
        )}
        {receipt.program_name && (
          <div className="flex justify-between">
            <span className="text-gray-600">Program:</span>
            <span className="text-right truncate max-w-[48mm]">
              {receipt.program_name}
            </span>
          </div>
        )}
        {receipt.batch_name && (
          <div className="flex justify-between">
            <span className="text-gray-600">Batch:</span>
            <span className="text-right truncate max-w-[48mm]">{receipt.batch_name}</span>
          </div>
        )}
        {receipt.roll_no && (
          <div className="flex justify-between">
            <span className="text-gray-600">Roll:</span>
            <span className="font-bold">{receipt.roll_no}</span>
          </div>
        )}
      </div>

      {/* Items Breakdown */}
      <div className="mb-2 border-b border-dashed border-gray-400 pb-2">
        <div className="flex justify-between font-bold text-[10px] border-b border-gray-300 pb-1 mb-1">
          <span>Fee Description</span>
          <span className="text-right">Amount (BDT)</span>
        </div>
        <div className="space-y-1 text-[10px]">
          {receipt.items.map((item, idx) => (
            <div key={idx} className="flex justify-between items-baseline">
              <div className="truncate max-w-[50mm]">
                <div>{item.fee_head_name}</div>
                {item.period && (
                  <div className="text-[9px] text-gray-500">Period: {item.period}</div>
                )}
              </div>
              <div className="font-bold text-right tabular-nums">
                {formatTaka(item.amount_poisha)}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Total & Words */}
      <div className="space-y-1 mb-2 border-b border-dashed border-gray-400 pb-2 text-[10px]">
        <div className="flex justify-between items-center text-xs font-bold pt-0.5">
          <span>TOTAL PAID:</span>
          <span className="text-sm tabular-nums">
            {formatTaka(receipt.amount_poisha)}
          </span>
        </div>
        <div className="text-[9px] text-gray-700 italic leading-snug">
          In Words: {receipt.amount_in_words_en}
        </div>
        {receipt.amount_in_words_bn && (
          <div className="text-[9px] text-gray-700 leading-snug">
            কথায়: {receipt.amount_in_words_bn}
          </div>
        )}
        <div className="flex justify-between text-[10px] pt-1">
          <span className="text-gray-600">Payment Method:</span>
          <span className="font-bold uppercase">{receipt.method}</span>
        </div>
        {receipt.reference && (
          <div className="flex justify-between text-[10px]">
            <span className="text-gray-600">Reference:</span>
            <span className="font-mono">{receipt.reference}</span>
          </div>
        )}
        <div className="flex justify-between text-[10px] pt-1 border-t border-dotted border-gray-300">
          <span className="text-gray-600">Student Balance After:</span>
          <span
            className={`font-bold tabular-nums ${
              receipt.balance_after_poisha > 0 ? "text-red-700" : "text-emerald-700"
            }`}
          >
            {receipt.balance_after_poisha > 0
              ? `${formatTaka(receipt.balance_after_poisha)} Due`
              : receipt.balance_after_poisha < 0
                ? `${formatTaka(-receipt.balance_after_poisha)} Advance`
                : "৳ 0 Cleared"}
          </span>
        </div>
      </div>

      {/* Footer Notes & Verification */}
      <div className="text-center text-[9px] text-gray-600 space-y-1 pt-1">
        {footerNote && <p className="leading-snug">{footerNote}</p>}
        <div className="pt-3 border-t border-dotted border-gray-300 mt-2 flex justify-between px-2">
          <span>Student / Guardian</span>
          <span>Cashier Signature</span>
        </div>
        <div className="pt-2 text-[8px] text-gray-400">
          Generated via Coachify • Computer System Verified
        </div>
      </div>

      {/* Feed line break for roll cutter */}
      <div className="h-6" />
    </div>
  );
}

/**
 * Generates self contained HTML for silent thermal printing.
 */
export function generateThermalReceiptHtml(
  receipt: PrintableReceiptData,
  footerNote?: string | null
): string {
  const itemsHtml = receipt.items
    .map(
      (item) => `
    <tr>
      <td style="padding: 2px 0; text-align: left;">
        <div>${item.fee_head_name}</div>
        ${item.period ? `<div style="font-size: 9px; color: #555;">Period: ${item.period}</div>` : ""}
      </td>
      <td style="padding: 2px 0; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums;">
        ${formatTaka(item.amount_poisha)}
      </td>
    </tr>
  `
    )
    .join("");

  const balanceText =
    receipt.balance_after_poisha > 0
      ? `${formatTaka(receipt.balance_after_poisha)} Due`
      : receipt.balance_after_poisha < 0
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance`
        : "৳ 0 Cleared";

  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Receipt ${receipt.receipt_no}</title>
  <style>
    @page {
      size: 80mm auto;
      margin: 0;
    }
    body {
      margin: 0;
      padding: 6px;
      font-family: monospace, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      font-size: 11px;
      color: #000;
      background: #fff;
      line-height: 1.25;
      width: 72mm;
    }
    .text-center { text-align: center; }
    .text-right { text-align: right; }
    .bold { font-weight: bold; }
    .border-b { border-bottom: 1px dashed #444; }
    .border-t { border-top: 1px dashed #444; }
    .flex-row { display: flex; justify-content: space-between; margin: 2px 0; font-size: 10px; }
    table { width: 100%; border-collapse: collapse; margin: 4px 0; font-size: 10px; }
    th { border-bottom: 1px solid #333; padding-bottom: 2px; }
    .badge { display: inline-block; border: 1px solid #000; font-weight: bold; padding: 1px 4px; font-size: 9px; margin-top: 4px; }
    @media print {
      body { width: 72mm; }
    }
  </style>
</head>
<body>
  <div class="text-center border-b" style="padding-bottom: 6px; margin-bottom: 6px;">
    ${receipt.institution_name_bn ? `<div class="bold" style="font-size: 13px;">${receipt.institution_name_bn}</div>` : ""}
    <div class="bold" style="font-size: 11px; text-transform: uppercase;">${receipt.institution_name}</div>
    ${receipt.institution_address ? `<div style="font-size: 9px; color: #444;">${receipt.institution_address}</div>` : ""}
    ${receipt.institution_phone ? `<div style="font-size: 9px; color: #444;">Phone: ${receipt.institution_phone}</div>` : ""}
    <div class="badge">${receipt.is_reprint ? "MONEY RECEIPT (REPRINT)" : "MONEY RECEIPT"}</div>
  </div>

  <div class="border-b" style="padding-bottom: 4px; margin-bottom: 4px;">
    <div class="flex-row"><span>Receipt No:</span><span class="bold">${receipt.receipt_no}</span></div>
    <div class="flex-row"><span>Date:</span><span>${receipt.business_date}</span></div>
    <div class="flex-row"><span>Time:</span><span>${receipt.received_at.slice(11, 19)}</span></div>
    <div class="flex-row"><span>Cashier:</span><span>${receipt.collector_name}</span></div>
  </div>

  <div class="border-b" style="padding-bottom: 4px; margin-bottom: 4px;">
    <div class="flex-row"><span>Student ID:</span><span class="bold">${receipt.student_code}</span></div>
    <div class="flex-row"><span>Name:</span><span class="bold">${receipt.student_name}</span></div>
    ${receipt.student_name_bn ? `<div class="flex-row"><span>বাংলা নাম:</span><span>${receipt.student_name_bn}</span></div>` : ""}
    ${receipt.program_name ? `<div class="flex-row"><span>Program:</span><span>${receipt.program_name}</span></div>` : ""}
    ${receipt.batch_name ? `<div class="flex-row"><span>Batch:</span><span>${receipt.batch_name}</span></div>` : ""}
    ${receipt.roll_no ? `<div class="flex-row"><span>Roll:</span><span class="bold">${receipt.roll_no}</span></div>` : ""}
  </div>

  <table class="border-b">
    <thead>
      <tr>
        <th style="text-align: left;">Description</th>
        <th style="text-align: right;">Amount</th>
      </tr>
    </thead>
    <tbody>
      ${itemsHtml}
    </tbody>
  </table>

  <div class="border-b" style="padding-bottom: 4px; margin-bottom: 4px;">
    <div class="flex-row bold" style="font-size: 12px; margin-top: 4px;">
      <span>TOTAL PAID:</span>
      <span>${formatTaka(receipt.amount_poisha)}</span>
    </div>
    <div style="font-size: 9px; font-style: italic; margin-top: 2px;">In Words: ${receipt.amount_in_words_en}</div>
    ${receipt.amount_in_words_bn ? `<div style="font-size: 9px; margin-top: 1px;">কথায়: ${receipt.amount_in_words_bn}</div>` : ""}
    <div class="flex-row" style="margin-top: 4px;"><span>Method:</span><span class="bold" style="text-transform: uppercase;">${receipt.method}</span></div>
    ${receipt.reference ? `<div class="flex-row"><span>Reference:</span><span>${receipt.reference}</span></div>` : ""}
    <div class="flex-row" style="margin-top: 4px; border-top: 1px dotted #ccc; padding-top: 2px;">
      <span>Balance After:</span>
      <span class="bold">${balanceText}</span>
    </div>
  </div>

  <div class="text-center" style="font-size: 9px; color: #555; margin-top: 8px;">
    ${footerNote ? `<div>${footerNote}</div>` : ""}
    <div style="display: flex; justify-content: space-between; margin-top: 20px; padding: 0 4px;">
      <span>Student / Guardian</span>
      <span>Cashier Signature</span>
    </div>
    <div style="font-size: 8px; color: #888; margin-top: 8px;">Generated via Coachify • Offline Verified</div>
  </div>
  <div style="height: 24px;"></div>
</body>
</html>
  `.trim();
}
