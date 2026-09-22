import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface ThermalReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

/**
 * Academic coaching center emblem for thermal receipts.
 */
function AcademicEmblemSvg({ className = "h-8 w-8" }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-hidden="true"
    >
      {/* Outer decorative shield */}
      <path
        d="M32 4L10 12V30C10 44 19.5 56 32 60C44.5 56 54 44 54 30V12L32 4Z"
        stroke="currentColor"
        strokeWidth="2.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      {/* Inner shield border */}
      <path
        d="M32 8L14 14.5V30C14 41.5 21.8 51.5 32 55C42.2 51.5 50 41.5 50 30V14.5L32 8Z"
        stroke="currentColor"
        strokeWidth="1"
        strokeDasharray="2 2"
      />
      {/* Open book of knowledge */}
      <path
        d="M22 38C26 36 30 36.5 32 38C34 36.5 38 36 42 38V27C38 25 34 25.5 32 27C30 25.5 26 25 22 27V38Z"
        fill="currentColor"
        fillOpacity="0.15"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
      <line x1="32" y1="27" x2="32" y2="38" stroke="currentColor" strokeWidth="1.5" />
      {/* Torch / Flame of Enlightenment */}
      <path
        d="M32 15C33.5 17.5 35 19 33.5 21.5C32.8 22.8 31.2 22.8 30.5 21.5C29 19 30.5 17.5 32 15Z"
        fill="currentColor"
      />
      <path
        d="M30 22H34L33 26H31L30 22Z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="0.5"
      />
      {/* Radiating knowledge stars */}
      <circle cx="21" cy="20" r="1.2" fill="currentColor" />
      <circle cx="43" cy="20" r="1.2" fill="currentColor" />
      <circle cx="32" cy="46" r="1.5" fill="currentColor" />
    </svg>
  );
}

const EMBLEM_SVG_STRING = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="34" height="34" style="margin: 0 auto; display: block; color: #000;">
  <path d="M32 4L10 12V30C10 44 19.5 56 32 60C44.5 56 54 44 54 30V12L32 4Z" stroke="#000" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <path d="M32 8L14 14.5V30C14 41.5 21.8 51.5 32 55C42.2 51.5 50 41.5 50 30V14.5L32 8Z" stroke="#000" stroke-width="1" stroke-dasharray="2 2"/>
  <path d="M22 38C26 36 30 36.5 32 38C34 36.5 38 36 42 38V27C38 25 34 25.5 32 27C30 25.5 26 25 22 27V38Z" fill="#000" fill-opacity="0.15" stroke="#000" stroke-width="1.5" stroke-linejoin="round"/>
  <line x1="32" y1="27" x2="32" y2="38" stroke="#000" stroke-width="1.5"/>
  <path d="M32 15C33.5 17.5 35 19 33.5 21.5C32.8 22.8 31.2 22.8 30.5 21.5C29 19 30.5 17.5 32 15Z" fill="#000"/>
  <path d="M30 22H34L33 26H31L30 22Z" fill="#000"/>
  <circle cx="21" cy="20" r="1.2" fill="#000"/>
  <circle cx="43" cy="20" r="1.2" fill="#000"/>
  <circle cx="32" cy="46" r="1.5" fill="#000"/>
</svg>
`.trim();

export function ThermalReceipt({ receipt, footerNote }: ThermalReceiptProps) {
  const balanceText =
    receipt.balance_after_poisha > 0
      ? `${formatTaka(receipt.balance_after_poisha)} Due`
      : receipt.balance_after_poisha < 0
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance`
        : "৳ 0 Cleared";

  return (
    <div className="w-[76mm] max-w-[76mm] mx-auto bg-white text-black p-2 font-mono text-[11px] leading-tight select-text print:p-0 print:w-[72mm]">
      {/* Institution Logo & Header */}
      <div className="text-center border-b-2 border-black pb-2 mb-2">
        <div className="flex justify-center mb-1">
          <AcademicEmblemSvg className="h-9 w-9 text-black" />
        </div>

        {receipt.institution_name_bn && (
          <div className="font-bold text-sm tracking-wide leading-tight text-black">
            {receipt.institution_name_bn}
          </div>
        )}
        <div className="font-bold text-[11px] uppercase tracking-wider text-black mt-0.5">
          {receipt.institution_name}
        </div>
        {receipt.institution_address && (
          <div className="text-[10px] text-gray-700 mt-0.5">
            {receipt.institution_address}
          </div>
        )}
        {receipt.institution_phone && (
          <div className="text-[10px] text-gray-700">
            {`Helpline: ${receipt.institution_phone}`}
          </div>
        )}
        {receipt.institution_website && (
          <div className="text-[9px] text-gray-600">{receipt.institution_website}</div>
        )}

        <div className="inline-flex items-center gap-1 border-2 border-black font-extrabold px-2.5 py-0.5 mt-1.5 text-[10.5px] uppercase tracking-wider bg-gray-50">
          <span>MONEY RECEIPT</span>
          <span>•</span>
          <span>মানি রিসিট</span>
        </div>

        {receipt.is_reprint && (
          <div className="mt-1 text-[10px] font-extrabold text-black uppercase bg-gray-200 py-0.5 px-2 border border-black inline-block">
            *** REPRINT COPY / অনুলিপি ***
          </div>
        )}
      </div>

      {/* Metadata Card */}
      <div className="border border-black p-1.5 mb-2 text-[10px] bg-gray-50/50 space-y-0.5">
        <div className="flex justify-between items-center">
          <span className="font-bold">Receipt No:</span>
          <span className="font-bold font-mono text-xs">{receipt.receipt_no}</span>
        </div>
        <div className="flex justify-between">
          <span>Business Date:</span>
          <span>{receipt.business_date}</span>
        </div>
        <div className="flex justify-between">
          <span>Time:</span>
          <span>{receipt.received_at.slice(11, 19)}</span>
        </div>
        <div className="flex justify-between border-t border-dotted border-gray-400 pt-0.5 mt-0.5">
          <span>Cashier:</span>
          <span className="font-semibold">{receipt.collector_name}</span>
        </div>
      </div>

      {/* Student Details Card */}
      <div className="border border-black p-1.5 mb-2 text-[10px] space-y-0.5">
        <div className="flex justify-between">
          <span className="font-semibold">Student ID:</span>
          <span className="font-bold font-mono">{receipt.student_code}</span>
        </div>
        <div className="flex justify-between">
          <span className="font-semibold">Student Name:</span>
          <span className="font-bold text-right truncate max-w-[46mm]">
            {receipt.student_name}
          </span>
        </div>
        {receipt.student_name_bn && (
          <div className="flex justify-between">
            <span className="text-gray-700">বাংলা নাম:</span>
            <span className="font-medium text-right truncate max-w-[46mm]">
              {receipt.student_name_bn}
            </span>
          </div>
        )}
        {receipt.program_name && (
          <div className="flex justify-between">
            <span>Program:</span>
            <span className="text-right truncate max-w-[46mm]">
              {receipt.program_name}
            </span>
          </div>
        )}
        {receipt.batch_name && (
          <div className="flex justify-between">
            <span>Batch:</span>
            <span className="text-right truncate max-w-[46mm]">{receipt.batch_name}</span>
          </div>
        )}
        {receipt.roll_no && (
          <div className="flex justify-between">
            <span>Roll:</span>
            <span className="font-bold">{receipt.roll_no}</span>
          </div>
        )}
        {receipt.guardian_phone && (
          <div className="flex justify-between border-t border-dotted border-gray-400 pt-0.5 mt-0.5">
            <span>Guardian:</span>
            <span>{receipt.guardian_phone}</span>
          </div>
        )}
      </div>

      {/* Items Breakdown Table */}
      <div className="mb-2 border border-black">
        <div className="flex justify-between font-bold text-[10px] bg-black text-white px-1.5 py-1">
          <span>Fee Description / বিবরণ</span>
          <span className="text-right">Amount (BDT)</span>
        </div>
        <div className="divide-y divide-gray-300 px-1.5 text-[10px]">
          {receipt.items.map((item, idx) => (
            <div key={idx} className="py-1 flex justify-between items-baseline">
              <div className="truncate max-w-[48mm]">
                <div className="font-medium">{item.fee_head_name}</div>
                {item.fee_head_name_bn && (
                  <div className="text-[9px] text-gray-600">{item.fee_head_name_bn}</div>
                )}
                {item.period && (
                  <div className="text-[9px] text-gray-600">Month: {item.period}</div>
                )}
              </div>
              <div className="font-bold text-right tabular-nums">
                {formatTaka(item.amount_poisha)}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Total & Words Summary */}
      <div className="border border-black p-1.5 mb-2 text-[10px] bg-gray-50/70 space-y-1">
        <div className="flex justify-between items-center text-xs font-black border-b border-black pb-1">
          <span>TOTAL PAID / মোট জমা:</span>
          <span className="text-sm tabular-nums">
            {formatTaka(receipt.amount_poisha)}
          </span>
        </div>
        <div className="text-[9.5px] leading-snug pt-0.5">
          <span className="font-bold">In Words: </span>
          <span className="italic">{receipt.amount_in_words_en}</span>
        </div>
        {receipt.amount_in_words_bn && (
          <div className="text-[9.5px] leading-snug">
            <span className="font-bold">কথায়: </span>
            <span>{receipt.amount_in_words_bn}</span>
          </div>
        )}
        <div className="flex justify-between border-t border-dotted border-gray-400 pt-1">
          <span className="font-semibold">Method:</span>
          <span className="font-bold uppercase">{receipt.method}</span>
        </div>
        {receipt.reference && (
          <div className="flex justify-between">
            <span className="font-semibold">Ref/TrxID:</span>
            <span className="font-mono">{receipt.reference}</span>
          </div>
        )}
        <div className="flex justify-between border-t border-dashed border-gray-400 pt-1 font-bold">
          <span>Balance After / বকেয়া:</span>
          <span className="tabular-nums">{balanceText}</span>
        </div>
      </div>

      {/* Standard Terms of Bangladesh Coaching Centers */}
      <div className="border border-dotted border-gray-400 p-1 mb-2 text-[8.5px] text-gray-700 leading-tight">
        <div className="font-bold text-center mb-0.5">নির্দেশনাবলী / Terms:</div>
        <div>১. মানি রিসিট ব্যতীত কোনো দাবি গ্রহণযোগ্য নহে।</div>
        <div>২. প্রদেয় ফি সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য।</div>
        <div>৩. ক্লাসে আইডি কার্ড বা মানি রিসিট প্রদর্শন আবশ্যক।</div>
      </div>

      {/* Signatures & Seal */}
      <div className="text-center text-[9px] text-gray-700 space-y-1 pt-1">
        {footerNote && <p className="leading-snug italic mb-2">{footerNote}</p>}
        <div className="pt-4 border-t border-black mt-2 flex justify-between px-1 text-[9px]">
          <div className="text-center">
            <div className="border-b border-gray-500 w-24 mb-1"></div>
            <span>Student / Guardian</span>
          </div>
          <div className="text-center">
            <div className="border-b border-gray-500 w-24 mb-1"></div>
            <span>Cashier Signature</span>
          </div>
        </div>
        <div className="pt-2 text-[8px] text-gray-500 uppercase tracking-wider">
          Coachify Academic ERP • Computer Generated Receipt
        </div>
      </div>

      {/* Feed line break for roll cutter */}
      <div className="h-6" />
    </div>
  );
}

/**
 * Generates self contained HTML for silent thermal printing with institutional branding.
 */
export function generateThermalReceiptHtml(
  receipt: PrintableReceiptData,
  footerNote?: string | null
): string {
  const itemsHtml = receipt.items
    .map(
      (item) => `
    <tr>
      <td style="padding: 3px 0; text-align: left; border-bottom: 1px dotted #ccc;">
        <div style="font-weight: 500;">${item.fee_head_name}</div>
        ${item.fee_head_name_bn ? `<div style="font-size: 9px; color: #444;">${item.fee_head_name_bn}</div>` : ""}
        ${item.period ? `<div style="font-size: 9px; color: #555;">Month: ${item.period}</div>` : ""}
      </td>
      <td style="padding: 3px 0; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums; border-bottom: 1px dotted #ccc;">
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
    .box { border: 1px solid #000; padding: 5px; margin-bottom: 6px; }
    .header-box { text-align: center; border-bottom: 2px solid #000; padding-bottom: 6px; margin-bottom: 6px; }
    .flex-row { display: flex; justify-content: space-between; margin: 2px 0; font-size: 10px; }
    table { width: 100%; border-collapse: collapse; margin: 4px 0; font-size: 10px; }
    th { background: #000; color: #fff; padding: 3px 4px; }
    .badge { display: inline-block; border: 2px solid #000; font-weight: 800; padding: 2px 6px; font-size: 10px; margin-top: 4px; background: #f5f5f5; }
    @media print {
      body { width: 72mm; }
    }
  </style>
</head>
<body>
  <div class="header-box">
    ${EMBLEM_SVG_STRING}
    ${receipt.institution_name_bn ? `<div class="bold" style="font-size: 14px; margin-top: 4px;">${receipt.institution_name_bn}</div>` : ""}
    <div class="bold" style="font-size: 11px; text-transform: uppercase; margin-top: 1px;">${receipt.institution_name}</div>
    ${receipt.institution_address ? `<div style="font-size: 9.5px; color: #333; margin-top: 2px;">${receipt.institution_address}</div>` : ""}
    ${receipt.institution_phone ? `<div style="font-size: 9.5px; color: #333;">Helpline: ${receipt.institution_phone}</div>` : ""}
    ${receipt.institution_website ? `<div style="font-size: 8.5px; color: #555;">${receipt.institution_website}</div>` : ""}
    <div class="badge">MONEY RECEIPT • মানি রিসিট</div>
    ${receipt.is_reprint ? `<div style="font-size: 9px; font-weight: bold; margin-top: 3px; border: 1px solid #000; padding: 1px 4px; display: inline-block;">*** REPRINT COPY / অনুলিপি ***</div>` : ""}
  </div>

  <div class="box" style="background: #fafafa;">
    <div class="flex-row"><span>Receipt No:</span><span class="bold" style="font-size: 11px;">${receipt.receipt_no}</span></div>
    <div class="flex-row"><span>Date:</span><span>${receipt.business_date}</span></div>
    <div class="flex-row"><span>Time:</span><span>${receipt.received_at.slice(11, 19)}</span></div>
    <div class="flex-row" style="border-top: 1px dotted #999; padding-top: 2px; margin-top: 2px;"><span>Cashier:</span><span class="bold">${receipt.collector_name}</span></div>
  </div>

  <div class="box">
    <div class="flex-row"><span>Student ID:</span><span class="bold">${receipt.student_code}</span></div>
    <div class="flex-row"><span>Student Name:</span><span class="bold">${receipt.student_name}</span></div>
    ${receipt.student_name_bn ? `<div class="flex-row"><span>বাংলা নাম:</span><span>${receipt.student_name_bn}</span></div>` : ""}
    ${receipt.program_name ? `<div class="flex-row"><span>Program:</span><span>${receipt.program_name}</span></div>` : ""}
    ${receipt.batch_name ? `<div class="flex-row"><span>Batch:</span><span>${receipt.batch_name}</span></div>` : ""}
    ${receipt.roll_no ? `<div class="flex-row"><span>Roll:</span><span class="bold">${receipt.roll_no}</span></div>` : ""}
    ${receipt.guardian_phone ? `<div class="flex-row" style="border-top: 1px dotted #999; padding-top: 2px; margin-top: 2px;"><span>Guardian:</span><span>${receipt.guardian_phone}</span></div>` : ""}
  </div>

  <div style="border: 1px solid #000; margin-bottom: 6px;">
    <table>
      <thead>
        <tr>
          <th style="text-align: left;">Particulars / বিবরণ</th>
          <th style="text-align: right;">Amount (BDT)</th>
        </tr>
      </thead>
      <tbody>
        ${itemsHtml}
      </tbody>
    </table>
  </div>

  <div class="box" style="background: #fafafa;">
    <div class="flex-row bold" style="font-size: 12px; border-bottom: 1px solid #000; padding-bottom: 3px; margin-bottom: 3px;">
      <span>TOTAL PAID / মোট জমা:</span>
      <span>${formatTaka(receipt.amount_poisha)}</span>
    </div>
    <div style="font-size: 9.5px; margin-top: 2px;"><strong>In Words:</strong> <em>${receipt.amount_in_words_en}</em></div>
    ${receipt.amount_in_words_bn ? `<div style="font-size: 9.5px; margin-top: 1px;"><strong>কথায়:</strong> ${receipt.amount_in_words_bn}</div>` : ""}
    <div class="flex-row" style="margin-top: 4px; border-top: 1px dotted #999; padding-top: 2px;"><span>Method:</span><span class="bold" style="text-transform: uppercase;">${receipt.method}</span></div>
    ${receipt.reference ? `<div class="flex-row"><span>Ref / TrxID:</span><span>${receipt.reference}</span></div>` : ""}
    <div class="flex-row" style="margin-top: 3px; border-top: 1px dashed #999; padding-top: 2px;">
      <span>Balance After / বকেয়া:</span>
      <span class="bold">${balanceText}</span>
    </div>
  </div>

  <div style="border: 1px dotted #666; padding: 4px; font-size: 8.5px; line-height: 1.25; margin-bottom: 6px;">
    <div style="font-weight: bold; text-align: center; margin-bottom: 2px;">নির্দেশনাবলী / Instructions:</div>
    <div>১. মানি রিসিট ব্যতীত কোনো দাবি গ্রহণযোগ্য নহে।</div>
    <div>২. প্রদেয় ফি সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য।</div>
    <div>৩. ক্লাসে আইডি কার্ড বা মানি রিসিট প্রদর্শন আবশ্যক।</div>
  </div>

  <div class="text-center" style="font-size: 9px; color: #444; margin-top: 6px;">
    ${footerNote ? `<div style="margin-bottom: 8px;">${footerNote}</div>` : ""}
    <div style="display: flex; justify-content: space-between; margin-top: 18px; padding: 0 4px;">
      <div style="text-align: center; width: 35%;">
        <div style="border-bottom: 1px solid #000; margin-bottom: 2px;"></div>
        <span>Student / Guardian</span>
      </div>
      <div style="text-align: center; width: 35%;">
        <div style="border-bottom: 1px solid #000; margin-bottom: 2px;"></div>
        <span>Cashier Signature</span>
      </div>
    </div>
    <div style="font-size: 8px; color: #666; margin-top: 8px; text-transform: uppercase;">Coachify Academic ERP • Verified Receipt</div>
  </div>
  <div style="height: 20px;"></div>
</body>
</html>
  `.trim();
}
