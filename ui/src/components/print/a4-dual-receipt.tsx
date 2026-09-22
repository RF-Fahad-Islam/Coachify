import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface A4DualReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

function ReceiptSingleVoucher({
  receipt,
  copyTitle,
  copyTitleBn,
  footerNote,
}: {
  receipt: PrintableReceiptData;
  copyTitle: string;
  copyTitleBn: string;
  footerNote?: string | null;
}) {
  const balanceText =
    receipt.balance_after_poisha > 0
      ? `${formatTaka(receipt.balance_after_poisha)} Due`
      : receipt.balance_after_poisha < 0
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance Credit`
        : "৳ 0 Cleared";

  return (
    <div className="border border-gray-300 rounded p-4 bg-white text-black text-xs">
      {/* Header */}
      <div className="flex justify-between items-start border-b border-gray-300 pb-2 mb-3">
        <div>
          {receipt.institution_name_bn && (
            <h2 className="text-base font-bold text-gray-900 leading-snug">
              {receipt.institution_name_bn}
            </h2>
          )}
          <h1 className="text-sm font-bold uppercase tracking-wider text-gray-800">
            {receipt.institution_name}
          </h1>
          {receipt.institution_address && (
            <p className="text-[11px] text-gray-600">{receipt.institution_address}</p>
          )}
          {receipt.institution_phone && (
            <p className="text-[11px] text-gray-600">
              Phone: {receipt.institution_phone}
            </p>
          )}
        </div>

        <div className="text-right">
          <div className="inline-block border border-gray-800 px-2 py-0.5 font-bold uppercase text-[10px] bg-gray-50">
            {copyTitle} / {copyTitleBn}
          </div>
          {receipt.is_reprint && (
            <div className="text-[10px] font-bold text-amber-700 mt-0.5">
              REPRINT COPY
            </div>
          )}
          <div className="mt-1 font-mono text-xs font-bold text-gray-900">
            {receipt.receipt_no}
          </div>
          <div className="text-[11px] text-gray-600">Date: {receipt.business_date}</div>
        </div>
      </div>

      {/* Student & Payment Info Grid */}
      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-[11px] bg-gray-50/70 p-2 rounded border border-gray-200 mb-3">
        <div>
          <span className="text-gray-500 font-medium">Student Code: </span>
          <span className="font-bold">{receipt.student_code}</span>
        </div>
        <div>
          <span className="text-gray-500 font-medium">Cashier / Collector: </span>
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
          <span className="text-gray-500 font-medium">Payment Method: </span>
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

      {/* Line Items Table */}
      <table className="w-full border-collapse text-[11px] mb-2">
        <thead>
          <tr className="border-b border-t border-gray-300 bg-gray-100/70 text-gray-700">
            <th className="py-1 px-2 text-left font-semibold">#</th>
            <th className="py-1 px-2 text-left font-semibold">Fee Head / Description</th>
            <th className="py-1 px-2 text-left font-semibold">Period</th>
            <th className="py-1 px-2 text-right font-semibold">Amount (BDT)</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {receipt.items.map((item, idx) => (
            <tr key={idx}>
              <td className="py-1 px-2 text-gray-500 w-8">{idx + 1}</td>
              <td className="py-1 px-2 font-medium">
                {item.fee_head_name}
                {item.fee_head_name_bn && (
                  <span className="text-gray-500 ml-1">({item.fee_head_name_bn})</span>
                )}
              </td>
              <td className="py-1 px-2 text-gray-600">{item.period || "—"}</td>
              <td className="py-1 px-2 text-right font-bold tabular-nums">
                {formatTaka(item.amount_poisha)}
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr className="border-t-2 border-gray-400 font-bold bg-gray-50">
            <td colSpan={3} className="py-1.5 px-2 text-right">
              TOTAL COLLECTED:
            </td>
            <td className="py-1.5 px-2 text-right text-sm tabular-nums text-gray-900">
              {formatTaka(receipt.amount_poisha)}
            </td>
          </tr>
        </tfoot>
      </table>

      {/* Words and Signatures */}
      <div className="space-y-1 mb-4 text-[10px] text-gray-700">
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

      <div className="pt-4 flex justify-between items-end text-[10px] text-gray-600 border-t border-dotted border-gray-300">
        <div className="text-center w-36">
          <div className="border-b border-gray-400 pb-1 mb-1"></div>
          <div>Student / Guardian</div>
        </div>
        <div className="text-[9px] text-gray-400 text-center">
          {footerNote || "Thank you. Keep this receipt safe."}
        </div>
        <div className="text-center w-36">
          <div className="border-b border-gray-400 pb-1 mb-1"></div>
          <div>Authorized Cashier</div>
        </div>
      </div>
    </div>
  );
}

export function A4DualReceipt({ receipt, footerNote }: A4DualReceiptProps) {
  return (
    <div className="w-[200mm] max-w-[200mm] mx-auto p-4 bg-white text-black font-sans space-y-4 print:p-0 print:w-[190mm]">
      {/* Top Copy: Student Copy */}
      <ReceiptSingleVoucher
        receipt={receipt}
        copyTitle="STUDENT COPY"
        copyTitleBn="শিক্ষার্থী কপি"
        footerNote={footerNote}
      />

      {/* Perforation Divider Line */}
      <div className="relative py-2 flex items-center justify-center">
        <div className="w-full border-b-2 border-dashed border-gray-400" />
        <div className="absolute bg-white px-3 text-xs text-gray-500 font-mono flex items-center gap-1.5 select-none">
          <span>✂</span>
          <span>CUT HERE / ছিঁড়ে নিন</span>
          <span>✂</span>
        </div>
      </div>

      {/* Bottom Copy: Office Copy */}
      <ReceiptSingleVoucher
        receipt={receipt}
        copyTitle="OFFICE COPY"
        copyTitleBn="অফিস কপি"
        footerNote={footerNote}
      />
    </div>
  );
}

/**
 * Generates self contained HTML for A4 dual copy printing.
 */
export function generateA4DualReceiptHtml(
  receipt: PrintableReceiptData,
  footerNote?: string | null
): string {
  const renderHalf = (copyTitle: string, copyTitleBn: string) => {
    const itemsRows = receipt.items
      .map(
        (item, idx) => `
        <tr>
          <td style="padding: 3px 6px; text-align: left; color: #555;">${idx + 1}</td>
          <td style="padding: 3px 6px; text-align: left; font-weight: 500;">
            ${item.fee_head_name}
            ${item.fee_head_name_bn ? `<span style="color: #666; font-size: 10px;">(${item.fee_head_name_bn})</span>` : ""}
          </td>
          <td style="padding: 3px 6px; text-align: left; color: #555;">${item.period || "—"}</td>
          <td style="padding: 3px 6px; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums;">
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
      <div style="border: 1px solid #ccc; border-radius: 4px; padding: 12px; background: #fff; font-size: 11px;">
        <div style="display: flex; justify-content: space-between; border-bottom: 1px solid #ccc; padding-bottom: 8px; margin-bottom: 8px;">
          <div>
            ${receipt.institution_name_bn ? `<div style="font-size: 14px; font-weight: bold; color: #111;">${receipt.institution_name_bn}</div>` : ""}
            <div style="font-size: 12px; font-weight: bold; text-transform: uppercase; color: #222;">${receipt.institution_name}</div>
            ${receipt.institution_address ? `<div style="font-size: 10px; color: #555;">${receipt.institution_address}</div>` : ""}
            ${receipt.institution_phone ? `<div style="font-size: 10px; color: #555;">Phone: ${receipt.institution_phone}</div>` : ""}
          </div>
          <div style="text-align: right;">
            <div style="display: inline-block; border: 1px solid #333; padding: 2px 6px; font-weight: bold; font-size: 10px; background: #f9f9f9;">
              ${copyTitle} / ${copyTitleBn}
            </div>
            ${receipt.is_reprint ? `<div style="font-size: 10px; font-weight: bold; color: #b45309; margin-top: 2px;">REPRINT COPY</div>` : ""}
            <div style="font-size: 12px; font-weight: bold; font-family: monospace; margin-top: 4px;">${receipt.receipt_no}</div>
            <div style="font-size: 10px; color: #555;">Date: ${receipt.business_date}</div>
          </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 4px 12px; background: #f7f7f7; padding: 6px 10px; border-radius: 4px; border: 1px solid #e5e5e5; margin-bottom: 8px; font-size: 10px;">
          <div><span style="color: #666;">Student Code: </span><strong>${receipt.student_code}</strong></div>
          <div><span style="color: #666;">Cashier: </span><strong>${receipt.collector_name}</strong></div>
          <div><span style="color: #666;">Student Name: </span><strong>${receipt.student_name}</strong> ${receipt.student_name_bn ? `(${receipt.student_name_bn})` : ""}</div>
          <div><span style="color: #666;">Method: </span><strong style="text-transform: uppercase;">${receipt.method}</strong> ${receipt.reference ? `(${receipt.reference})` : ""}</div>
          <div><span style="color: #666;">Batch: </span>${receipt.program_name || ""} ${receipt.batch_name ? `• ${receipt.batch_name}` : ""} ${receipt.roll_no ? `(Roll ${receipt.roll_no})` : ""}</div>
          <div><span style="color: #666;">Balance After: </span><strong>${balanceText}</strong></div>
        </div>

        <table style="width: 100%; border-collapse: collapse; font-size: 10px; margin-bottom: 8px;">
          <thead>
            <tr style="border-top: 1px solid #ccc; border-bottom: 1px solid #ccc; background: #f0f0f0;">
              <th style="padding: 3px 6px; text-align: left; width: 24px;">#</th>
              <th style="padding: 3px 6px; text-align: left;">Fee Description</th>
              <th style="padding: 3px 6px; text-align: left; width: 90px;">Period</th>
              <th style="padding: 3px 6px; text-align: right; width: 90px;">Amount (BDT)</th>
            </tr>
          </thead>
          <tbody>
            ${itemsRows}
          </tbody>
          <tfoot>
            <tr style="border-top: 2px solid #555; background: #fafafa; font-weight: bold;">
              <td colspan="3" style="padding: 4px 6px; text-align: right;">TOTAL COLLECTED:</td>
              <td style="padding: 4px 6px; text-align: right; font-size: 12px; font-variant-numeric: tabular-nums;">${formatTaka(receipt.amount_poisha)}</td>
            </tr>
          </tfoot>
        </table>

        <div style="font-size: 10px; color: #444; margin-bottom: 14px;">
          <div><strong>In Words: </strong><em>${receipt.amount_in_words_en}</em></div>
          ${receipt.amount_in_words_bn ? `<div><strong>কথায়: </strong>${receipt.amount_in_words_bn}</div>` : ""}
        </div>

        <div style="display: flex; justify-content: space-between; align-items: flex-end; font-size: 9px; color: #666; border-top: 1px dotted #ccc; padding-top: 8px;">
          <div style="text-align: center; width: 120px;">
            <div style="border-bottom: 1px solid #888; margin-bottom: 3px;"></div>
            <div>Student / Guardian</div>
          </div>
          <div style="font-size: 8px; color: #888;">${footerNote || "Thank you. Keep this receipt safe."}</div>
          <div style="text-align: center; width: 120px;">
            <div style="border-bottom: 1px solid #888; margin-bottom: 3px;"></div>
            <div>Authorized Cashier</div>
          </div>
        </div>
      </div>
    `;
  };

  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Receipt ${receipt.receipt_no} - A4 Dual</title>
  <style>
    @page {
      size: A4 portrait;
      margin: 10mm;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: #fff;
      color: #000;
      line-height: 1.3;
    }
    .divider {
      position: relative;
      text-align: center;
      margin: 12px 0;
    }
    .divider-line {
      border-bottom: 2px dashed #999;
      width: 100%;
    }
    .divider-badge {
      position: absolute;
      top: -9px;
      left: 50%;
      transform: translateX(-50%);
      background: #fff;
      padding: 0 10px;
      font-size: 11px;
      color: #666;
      font-family: monospace;
    }
  </style>
</head>
<body>
  ${renderHalf("STUDENT COPY", "শিক্ষার্থী কপি")}

  <div class="divider">
    <div class="divider-line"></div>
    <div class="divider-badge">✂ CUT HERE / ছিঁড়ে নিন ✂</div>
  </div>

  ${renderHalf("OFFICE COPY", "অফিস কপি")}
</body>
</html>
  `.trim();
}
