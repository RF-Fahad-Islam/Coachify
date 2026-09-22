import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface A5ReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

export function A5Receipt({ receipt, footerNote }: A5ReceiptProps) {
  const balanceText =
    receipt.balance_after_poisha > 0
      ? `${formatTaka(receipt.balance_after_poisha)} Due`
      : receipt.balance_after_poisha < 0
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance Credit`
        : "৳ 0 Cleared";

  return (
    <div className="w-[140mm] max-w-[140mm] mx-auto border border-gray-400 rounded p-5 bg-white text-black font-sans text-xs select-text print:p-0 print:border-none print:w-[130mm]">
      {/* Header */}
      <div className="flex justify-between items-start border-b-2 border-gray-800 pb-3 mb-4">
        <div>
          {receipt.institution_name_bn && (
            <h2 className="text-base font-bold text-gray-900 leading-tight">
              {receipt.institution_name_bn}
            </h2>
          )}
          <h1 className="text-sm font-bold uppercase tracking-wider text-gray-800">
            {receipt.institution_name}
          </h1>
          {receipt.institution_address && (
            <p className="text-[11px] text-gray-600 mt-0.5">
              {receipt.institution_address}
            </p>
          )}
          {receipt.institution_phone && (
            <p className="text-[11px] text-gray-600">
              Phone: {receipt.institution_phone}
            </p>
          )}
        </div>

        <div className="text-right">
          <div className="inline-block border border-gray-800 px-2 py-0.5 font-bold uppercase text-[10px] bg-gray-100">
            MONEY RECEIPT
          </div>
          {receipt.is_reprint && (
            <div className="text-[10px] font-bold text-amber-700 mt-0.5">
              REPRINT COPY
            </div>
          )}
          <div className="mt-1.5 font-mono text-xs font-bold text-gray-900">
            {receipt.receipt_no}
          </div>
          <div className="text-[11px] text-gray-600">Date: {receipt.business_date}</div>
        </div>
      </div>

      {/* Info Grid */}
      <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-[11px] bg-gray-50 p-2.5 rounded border border-gray-200 mb-4">
        <div>
          <span className="text-gray-500 font-medium">Student Code: </span>
          <span className="font-bold">{receipt.student_code}</span>
        </div>
        <div>
          <span className="text-gray-500 font-medium">Cashier: </span>
          <span className="font-semibold">{receipt.collector_name}</span>
        </div>
        <div>
          <span className="text-gray-500 font-medium">Student Name: </span>
          <span className="font-bold">{receipt.student_name}</span>
          {receipt.student_name_bn && (
            <span className="text-gray-600 ml-1">({receipt.student_name_bn})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Method: </span>
          <span className="font-bold uppercase">{receipt.method}</span>
          {receipt.reference && (
            <span className="text-gray-600 ml-1 font-mono">({receipt.reference})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Program / Batch: </span>
          <span>{receipt.program_name || "General"}</span>
          {receipt.batch_name && <span> • {receipt.batch_name}</span>}
          {receipt.roll_no && (
            <span className="font-bold"> (Roll {receipt.roll_no})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Student Balance: </span>
          <span className="font-bold tabular-nums">{balanceText}</span>
        </div>
      </div>

      {/* Line items table */}
      <table className="w-full border-collapse text-[11px] mb-4">
        <thead>
          <tr className="border-b-2 border-t border-gray-300 bg-gray-100 text-gray-700">
            <th className="py-1.5 px-2 text-left font-semibold">#</th>
            <th className="py-1.5 px-2 text-left font-semibold">Fee Description</th>
            <th className="py-1.5 px-2 text-left font-semibold">Period</th>
            <th className="py-1.5 px-2 text-right font-semibold">Amount (BDT)</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {receipt.items.map((item, idx) => (
            <tr key={idx}>
              <td className="py-1.5 px-2 text-gray-500 w-8">{idx + 1}</td>
              <td className="py-1.5 px-2 font-medium">
                {item.fee_head_name}
                {item.fee_head_name_bn && (
                  <span className="text-gray-500 ml-1">({item.fee_head_name_bn})</span>
                )}
              </td>
              <td className="py-1.5 px-2 text-gray-600">{item.period || "—"}</td>
              <td className="py-1.5 px-2 text-right font-bold tabular-nums">
                {formatTaka(item.amount_poisha)}
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr className="border-t-2 border-gray-800 font-bold bg-gray-50">
            <td colSpan={3} className="py-2 px-2 text-right">
              TOTAL COLLECTED:
            </td>
            <td className="py-2 px-2 text-right text-sm tabular-nums text-gray-900">
              {formatTaka(receipt.amount_poisha)}
            </td>
          </tr>
        </tfoot>
      </table>

      {/* In Words */}
      <div className="space-y-1 mb-6 text-[10.5px] text-gray-700">
        <div>
          <span className="font-semibold">In Words: </span>
          <span className="italic">{receipt.amount_in_words_en}</span>
        </div>
        {receipt.amount_in_words_bn && (
          <div>
            <span className="font-semibold">কথায়: </span>
            <span>{receipt.amount_in_words_bn}</span>
          </div>
        )}
      </div>

      {/* Footer and Signatures */}
      <div className="pt-6 flex justify-between items-end text-[10px] text-gray-600 border-t border-dotted border-gray-300">
        <div className="text-center w-36">
          <div className="border-b border-gray-400 pb-1 mb-1"></div>
          <div>Student / Guardian</div>
        </div>
        <div className="text-[9px] text-gray-400 text-center">
          {footerNote || "Thank you for being with us."}
        </div>
        <div className="text-center w-36">
          <div className="border-b border-gray-400 pb-1 mb-1"></div>
          <div>Authorized Cashier</div>
        </div>
      </div>
    </div>
  );
}

/**
 * Generates self contained HTML for A5 single voucher printing.
 */
export function generateA5ReceiptHtml(
  receipt: PrintableReceiptData,
  footerNote?: string | null
): string {
  const itemsRows = receipt.items
    .map(
      (item, idx) => `
      <tr>
        <td style="padding: 4px 8px; text-align: left; color: #555;">${idx + 1}</td>
        <td style="padding: 4px 8px; text-align: left; font-weight: 500;">
          ${item.fee_head_name}
          ${item.fee_head_name_bn ? `<span style="color: #666; font-size: 10px;">(${item.fee_head_name_bn})</span>` : ""}
        </td>
        <td style="padding: 4px 8px; text-align: left; color: #555;">${item.period || "—"}</td>
        <td style="padding: 4px 8px; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums;">
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
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance Credit`
        : "৳ 0 Cleared";

  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Receipt ${receipt.receipt_no} - A5 Voucher</title>
  <style>
    @page {
      size: A5 portrait;
      margin: 10mm;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #fff;
      color: #000;
      line-height: 1.35;
      font-size: 11px;
    }
    .box {
      border: 1px solid #999;
      border-radius: 6px;
      padding: 16px;
    }
    .header {
      display: flex;
      justify-content: space-between;
      border-bottom: 2px solid #222;
      padding-bottom: 10px;
      margin-bottom: 12px;
    }
    .grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 6px 16px;
      background: #f8f8f8;
      padding: 8px 12px;
      border-radius: 4px;
      border: 1px solid #e0e0e0;
      margin-bottom: 12px;
      font-size: 11px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 11px;
      margin-bottom: 12px;
    }
    th {
      border-top: 1px solid #ccc;
      border-bottom: 2px solid #333;
      background: #eee;
      padding: 5px 8px;
    }
    tfoot tr {
      border-top: 2px solid #222;
      background: #f5f5f5;
      font-weight: bold;
    }
    .signatures {
      display: flex;
      justify-content: space-between;
      align-items: flex-end;
      font-size: 10px;
      color: #555;
      border-top: 1px dotted #ccc;
      padding-top: 14px;
      margin-top: 20px;
    }
    .sig-line {
      text-align: center;
      width: 130px;
    }
    .sig-line div:first-child {
      border-bottom: 1px solid #777;
      margin-bottom: 4px;
    }
  </style>
</head>
<body>
  <div class="box">
    <div class="header">
      <div>
        ${receipt.institution_name_bn ? `<div style="font-size: 16px; font-weight: bold; color: #111;">${receipt.institution_name_bn}</div>` : ""}
        <div style="font-size: 13px; font-weight: bold; text-transform: uppercase; color: #222;">${receipt.institution_name}</div>
        ${receipt.institution_address ? `<div style="font-size: 10px; color: #555;">${receipt.institution_address}</div>` : ""}
        ${receipt.institution_phone ? `<div style="font-size: 10px; color: #555;">Phone: ${receipt.institution_phone}</div>` : ""}
      </div>
      <div style="text-align: right;">
        <div style="display: inline-block; border: 1px solid #333; padding: 2px 8px; font-weight: bold; font-size: 10px; background: #f0f0f0;">
          MONEY RECEIPT
        </div>
        ${receipt.is_reprint ? `<div style="font-size: 10px; font-weight: bold; color: #b45309; margin-top: 2px;">REPRINT COPY</div>` : ""}
        <div style="font-size: 13px; font-weight: bold; font-family: monospace; margin-top: 4px;">${receipt.receipt_no}</div>
        <div style="font-size: 11px; color: #555;">Date: ${receipt.business_date}</div>
      </div>
    </div>

    <div class="grid">
      <div><span style="color: #666;">Student Code: </span><strong>${receipt.student_code}</strong></div>
      <div><span style="color: #666;">Cashier: </span><strong>${receipt.collector_name}</strong></div>
      <div><span style="color: #666;">Student Name: </span><strong>${receipt.student_name}</strong> ${receipt.student_name_bn ? `(${receipt.student_name_bn})` : ""}</div>
      <div><span style="color: #666;">Method: </span><strong style="text-transform: uppercase;">${receipt.method}</strong> ${receipt.reference ? `(${receipt.reference})` : ""}</div>
      <div><span style="color: #666;">Program / Batch: </span>${receipt.program_name || "General"} ${receipt.batch_name ? `• ${receipt.batch_name}` : ""} ${receipt.roll_no ? `(Roll ${receipt.roll_no})` : ""}</div>
      <div><span style="color: #666;">Student Balance: </span><strong>${balanceText}</strong></div>
    </div>

    <table>
      <thead>
        <tr>
          <th style="text-align: left; width: 28px;">#</th>
          <th style="text-align: left;">Fee Description</th>
          <th style="text-align: left; width: 100px;">Period</th>
          <th style="text-align: right; width: 100px;">Amount (BDT)</th>
        </tr>
      </thead>
      <tbody>
        ${itemsRows}
      </tbody>
      <tfoot>
        <tr>
          <td colspan="3" style="padding: 6px 8px; text-align: right;">TOTAL COLLECTED:</td>
          <td style="padding: 6px 8px; text-align: right; font-size: 13px; font-variant-numeric: tabular-nums;">${formatTaka(receipt.amount_poisha)}</td>
        </tr>
      </tfoot>
    </table>

    <div style="font-size: 11px; color: #444; margin-bottom: 12px;">
      <div><strong>In Words: </strong><em>${receipt.amount_in_words_en}</em></div>
      ${receipt.amount_in_words_bn ? `<div><strong>কথায়: </strong>${receipt.amount_in_words_bn}</div>` : ""}
    </div>

    <div class="signatures">
      <div class="sig-line">
        <div></div>
        <div>Student / Guardian</div>
      </div>
      <div style="font-size: 9px; color: #777;">${footerNote || "Thank you for being with us."}</div>
      <div class="sig-line">
        <div></div>
        <div>Authorized Cashier</div>
      </div>
    </div>
  </div>
</body>
</html>
  `.trim();
}
