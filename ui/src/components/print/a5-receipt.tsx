import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface A5ReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

/**
 * Academic coaching center crest emblem for formal A5 receipts.
 */
function AcademicEmblemSvg({ className = "h-12 w-12" }: { className?: string }) {
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
      {/* Open textbook of knowledge */}
      <path
        d="M22 38C26 36 30 36.5 32 38C34 36.5 38 36 42 38V27C38 25 34 25.5 32 27C30 25.5 26 25 22 27V38Z"
        fill="currentColor"
        fillOpacity="0.15"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinejoin="round"
      />
      <line x1="32" y1="27" x2="32" y2="38" stroke="currentColor" strokeWidth="1.5" />
      {/* Torch flame of wisdom */}
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
      {/* Radiating academic stars */}
      <circle cx="21" cy="20" r="1.2" fill="currentColor" />
      <circle cx="43" cy="20" r="1.2" fill="currentColor" />
      <circle cx="32" cy="46" r="1.5" fill="currentColor" />
    </svg>
  );
}

const EMBLEM_SVG_STRING = `
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="48" height="48" style="display: block; color: #111;">
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

export function A5Receipt({ receipt, footerNote }: A5ReceiptProps) {
  const balanceText =
    receipt.balance_after_poisha > 0
      ? `${formatTaka(receipt.balance_after_poisha)} Due`
      : receipt.balance_after_poisha < 0
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance`
        : "৳ 0 Cleared";

  return (
    <div className="w-[140mm] max-w-[140mm] mx-auto border-2 border-gray-900 rounded p-5 bg-white text-black font-sans text-xs select-text print:p-0 print:border-none print:w-[130mm] leading-tight">
      {/* Header with Academic Crest Logo and Coaching Branding */}
      <div className="flex justify-between items-start border-b-2 border-gray-900 pb-3 mb-3">
        <div className="flex items-center gap-3">
          <AcademicEmblemSvg className="h-12 w-12 text-gray-950 flex-shrink-0" />
          <div>
            {receipt.institution_name_bn && (
              <h2 className="text-base font-extrabold text-gray-950 leading-tight">
                {receipt.institution_name_bn}
              </h2>
            )}
            <h1 className="text-xs font-black uppercase tracking-wider text-gray-900">
              {receipt.institution_name}
            </h1>
            {receipt.institution_address && (
              <p className="text-[10.5px] text-gray-700 mt-0.5">
                {receipt.institution_address}
              </p>
            )}
            <div className="flex flex-wrap items-center gap-x-2 text-[10px] text-gray-600 mt-0.5">
              {receipt.institution_phone && (
                <span>{`Helpline: ${receipt.institution_phone}`}</span>
              )}
              {receipt.institution_email && (
                <span>{`Email: ${receipt.institution_email}`}</span>
              )}
              {receipt.institution_website && (
                <span>{`Web: ${receipt.institution_website}`}</span>
              )}
            </div>
          </div>
        </div>

        <div className="text-right flex-shrink-0">
          <div className="inline-flex items-center gap-1 border-2 border-gray-900 px-2.5 py-0.5 font-black uppercase text-[10.5px] bg-gray-100 tracking-wider">
            <span>MONEY RECEIPT</span>
            <span>•</span>
            <span>মানি রিসিট</span>
          </div>
          {receipt.is_reprint && (
            <div className="mt-1 text-[9.5px] font-bold text-amber-800 bg-amber-50 border border-amber-300 px-1.5 py-0.2 inline-block">
              REPRINT COPY / অনুলিপি
            </div>
          )}
          <div className="mt-1.5 font-mono text-xs font-extrabold text-gray-950">
            {receipt.receipt_no}
          </div>
          <div className="text-[10px] text-gray-600">
            Date: {receipt.business_date} • {receipt.received_at.slice(11, 19)}
          </div>
        </div>
      </div>

      {/* Student and Payment Metadata Card */}
      <div className="grid grid-cols-3 gap-x-3 gap-y-1.5 text-[10.5px] bg-gray-50 p-2.5 rounded border border-gray-300 mb-3">
        <div>
          <span className="text-gray-500 font-medium">Student ID: </span>
          <span className="font-bold font-mono">{receipt.student_code}</span>
        </div>
        <div>
          <span className="text-gray-500 font-medium">Program: </span>
          <span>{receipt.program_name || "General Academic"}</span>
        </div>
        <div>
          <span className="text-gray-500 font-medium">Payment Mode: </span>
          <span className="font-bold uppercase">{receipt.method}</span>
          {receipt.reference && (
            <span className="text-gray-600 font-mono text-[9.5px] ml-1">
              ({receipt.reference})
            </span>
          )}
        </div>

        <div>
          <span className="text-gray-500 font-medium">Student Name: </span>
          <span className="font-bold">{receipt.student_name}</span>
          {receipt.student_name_bn && (
            <span className="text-gray-700 ml-1">({receipt.student_name_bn})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Batch / Roll: </span>
          <span>{receipt.batch_name || "General Batch"}</span>
          {receipt.roll_no && (
            <span className="font-bold ml-1">(Roll {receipt.roll_no})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Cashier: </span>
          <span className="font-semibold">{receipt.collector_name}</span>
        </div>

        <div className="col-span-2">
          <span className="text-gray-500 font-medium">Guardian: </span>
          <span>{receipt.guardian_name || "N/A"}</span>
          {receipt.guardian_phone && (
            <span className="text-gray-700 ml-1">({receipt.guardian_phone})</span>
          )}
        </div>
        <div>
          <span className="text-gray-500 font-medium">Balance Status: </span>
          <span className="font-bold tabular-nums">{balanceText}</span>
        </div>
      </div>

      {/* Itemized Fee Breakdown Table */}
      <table className="w-full border-collapse text-[10.5px] mb-2 border border-gray-300">
        <thead>
          <tr className="border-b-2 border-gray-800 bg-gray-100 text-gray-800">
            <th className="py-1.5 px-2 text-left font-bold w-8">#</th>
            <th className="py-1.5 px-2 text-left font-bold">Fee Description / বিবরণ</th>
            <th className="py-1.5 px-2 text-left font-bold w-24">Period</th>
            <th className="py-1.5 px-2 text-right font-bold w-28">Amount (BDT)</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {receipt.items.map((item, idx) => (
            <tr key={idx}>
              <td className="py-1.5 px-2 text-gray-500">{idx + 1}</td>
              <td className="py-1.5 px-2 font-medium">
                {item.fee_head_name}
                {item.fee_head_name_bn && (
                  <span className="text-gray-600 ml-1">({item.fee_head_name_bn})</span>
                )}
              </td>
              <td className="py-1.5 px-2 text-gray-600">{item.period || "N/A"}</td>
              <td className="py-1.5 px-2 text-right font-bold tabular-nums">
                {formatTaka(item.amount_poisha)}
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr className="border-t-2 border-gray-800 font-bold bg-gray-50">
            <td colSpan={3} className="py-2 px-2 text-right text-gray-900">
              TOTAL RECEIVED / সর্বমোট আদায়:
            </td>
            <td className="py-2 px-2 text-right text-sm tabular-nums text-gray-950">
              {formatTaka(receipt.amount_poisha)}
            </td>
          </tr>
        </tfoot>
      </table>

      {/* In Words & Due Status */}
      <div className="space-y-0.5 mb-3 text-[10.5px] text-gray-800">
        <div>
          <span className="font-bold">In Words: </span>
          <span className="italic">{receipt.amount_in_words_en}</span>
        </div>
        {receipt.amount_in_words_bn && (
          <div>
            <span className="font-bold">কথায়: </span>
            <span>{receipt.amount_in_words_bn}</span>
          </div>
        )}
      </div>

      {/* Authentic Terms and Conditions */}
      <div className="border border-dotted border-gray-400 p-1.5 mb-3 text-[9px] text-gray-600 leading-tight">
        <div className="font-bold text-gray-800 mb-0.5">নির্দেশনাবলী / Terms:</div>
        <div>
          ১. মানি রিসিট ব্যতীত কোনো অভিযোগ বা রিফান্ড সংক্রান্ত দাবি গ্রহণযোগ্য হইবে না।
        </div>
        <div>২. পরিশোধিত ফি সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য।</div>
        <div>
          ৩. প্রতিটি ক্লাস ও পরীক্ষায় অংশ নিতে স্টুডেন্ট আইডি কার্ড সাথে রাখা আবশ্যক।
        </div>
        <div>৪. যাবতীয় বকেয়া নির্ধারিত তারিখের মধ্যে পরিশোধযোগ্য।</div>
      </div>

      {/* Triple Signature Block with Official Seal Box */}
      <div className="pt-3 flex justify-between items-end text-[10px] text-gray-700 border-t border-gray-300">
        <div className="text-center w-36">
          <div className="border-b border-gray-500 pb-1 mb-1"></div>
          <div className="font-medium">Student / Guardian</div>
          <div className="text-[9px] text-gray-500">শিক্ষার্থী বা অভিভাবক</div>
        </div>

        {/* Official Seal Box */}
        <div className="border-2 border-dashed border-gray-400 rounded px-3 py-1.5 text-center bg-gray-50/50">
          <div className="text-[9.5px] font-bold uppercase tracking-wider text-gray-600">
            OFFICIAL SEAL
          </div>
          <div className="text-[8.5px] text-gray-500">অফিসিয়াল সিল</div>
        </div>

        <div className="text-center w-36">
          <div className="border-b border-gray-500 pb-1 mb-1"></div>
          <div className="font-medium">Authorized Officer</div>
          <div className="text-[9px] text-gray-500">অনুমোদিত কর্মকর্তা</div>
        </div>
      </div>

      {/* Watermark and Footer */}
      <div className="text-[8.5px] text-gray-400 text-center mt-2">
        Coachify Academic ERP • Computer Generated Official Money Receipt •{" "}
        {footerNote || "Thank you for partnering with us for quality academic education."}
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
        <td style="padding: 3.5px 8px; text-align: left; color: #555;">${idx + 1}</td>
        <td style="padding: 3.5px 8px; text-align: left; font-weight: 500;">
          ${item.fee_head_name}
          ${item.fee_head_name_bn ? `<span style="color: #666; font-size: 10px; margin-left: 4px;">(${item.fee_head_name_bn})</span>` : ""}
        </td>
        <td style="padding: 3.5px 8px; text-align: left; color: #555;">${item.period || "N/A"}</td>
        <td style="padding: 3.5px 8px; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums;">
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
  <title>Receipt ${receipt.receipt_no} - A5 Voucher</title>
  <style>
    @page {
      size: A5 portrait;
      margin: 8mm 10mm;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", sans-serif;
      background: #fff;
      color: #000;
      line-height: 1.3;
      font-size: 11px;
    }
    .box {
      border: 2px solid #222;
      border-radius: 6px;
      padding: 16px;
    }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      border-bottom: 2px solid #222;
      padding-bottom: 8px;
      margin-bottom: 10px;
    }
    .grid {
      display: grid;
      grid-template-columns: 1fr 1fr 1fr;
      gap: 4px 12px;
      background: #f8f8f8;
      padding: 8px 12px;
      border-radius: 4px;
      border: 1px solid #e0e0e0;
      margin-bottom: 10px;
      font-size: 10.5px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 10.5px;
      margin-bottom: 10px;
      border: 1px solid #ccc;
    }
    th {
      border-bottom: 2px solid #333;
      background: #eee;
      padding: 4px 8px;
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
      font-size: 9.5px;
      color: #555;
      border-top: 1px solid #ccc;
      padding-top: 10px;
      margin-top: 12px;
    }
    .sig-line {
      text-align: center;
      width: 125px;
    }
    .sig-line div:first-child {
      border-bottom: 1px solid #777;
      margin-bottom: 3px;
    }
  </style>
</head>
<body>
  <div class="box">
    <div class="header">
      <div style="display: flex; align-items: center; gap: 12px;">
        ${EMBLEM_SVG_STRING}
        <div>
          ${receipt.institution_name_bn ? `<div style="font-size: 16px; font-weight: 800; color: #111;">${receipt.institution_name_bn}</div>` : ""}
          <div style="font-size: 12px; font-weight: 800; text-transform: uppercase; color: #222; letter-spacing: 0.5px;">${receipt.institution_name}</div>
          ${receipt.institution_address ? `<div style="font-size: 10px; color: #444; margin-top: 1px;">${receipt.institution_address}</div>` : ""}
          <div style="font-size: 9.5px; color: #555;">
            ${receipt.institution_phone ? `Helpline: ${receipt.institution_phone} ` : ""}
            ${receipt.institution_email ? `• Email: ${receipt.institution_email} ` : ""}
            ${receipt.institution_website ? `• Web: ${receipt.institution_website}` : ""}
          </div>
        </div>
      </div>
      <div style="text-align: right; flex-shrink: 0;">
        <div style="display: inline-block; border: 1.5px solid #222; padding: 2px 8px; font-weight: 800; font-size: 10px; background: #f0f0f0; letter-spacing: 0.5px;">
          MONEY RECEIPT • মানি রিসিট
        </div>
        ${receipt.is_reprint ? `<div style="font-size: 9.5px; font-weight: bold; color: #92400e; background: #fef3c7; border: 1px solid #fcd34d; padding: 1px 4px; margin-top: 2px; display: inline-block;">REPRINT COPY / অনুলিপি</div>` : ""}
        <div style="font-size: 12px; font-weight: 800; font-family: monospace; margin-top: 3px;">${receipt.receipt_no}</div>
        <div style="font-size: 10px; color: #555;">Date: ${receipt.business_date} • ${receipt.received_at.slice(11, 19)}</div>
      </div>
    </div>

    <div class="grid">
      <div><span style="color: #666;">Student ID: </span><strong>${receipt.student_code}</strong></div>
      <div><span style="color: #666;">Program: </span>${receipt.program_name || "General Academic"}</div>
      <div><span style="color: #666;">Mode: </span><strong style="text-transform: uppercase;">${receipt.method}</strong> ${receipt.reference ? `(${receipt.reference})` : ""}</div>

      <div><span style="color: #666;">Student: </span><strong>${receipt.student_name}</strong> ${receipt.student_name_bn ? `(${receipt.student_name_bn})` : ""}</div>
      <div><span style="color: #666;">Batch / Roll: </span>${receipt.batch_name || "General Batch"} ${receipt.roll_no ? `(Roll ${receipt.roll_no})` : ""}</div>
      <div><span style="color: #666;">Cashier: </span><strong>${receipt.collector_name}</strong></div>

      <div style="grid-column: span 2;"><span style="color: #666;">Guardian: </span>${receipt.guardian_name || "N/A"} ${receipt.guardian_phone ? `(${receipt.guardian_phone})` : ""}</div>
      <div><span style="color: #666;">Balance: </span><strong>${balanceText}</strong></div>
    </div>

    <table>
      <thead>
        <tr>
          <th style="text-align: left; width: 24px;">#</th>
          <th style="text-align: left;">Fee Description / বিবরণ</th>
          <th style="text-align: left; width: 90px;">Period</th>
          <th style="text-align: right; width: 95px;">Amount (BDT)</th>
        </tr>
      </thead>
      <tbody>
        ${itemsRows}
      </tbody>
      <tfoot>
        <tr>
          <td colspan="3" style="padding: 4px 8px; text-align: right;">TOTAL RECEIVED / সর্বমোট আদায়:</td>
          <td style="padding: 4px 8px; text-align: right; font-size: 12px; font-variant-numeric: tabular-nums;">${formatTaka(receipt.amount_poisha)}</td>
        </tr>
      </tfoot>
    </table>

    <div style="font-size: 10px; color: #333; margin-bottom: 8px;">
      <div><strong>In Words: </strong><em>${receipt.amount_in_words_en}</em></div>
      ${receipt.amount_in_words_bn ? `<div><strong>কথায়: </strong>${receipt.amount_in_words_bn}</div>` : ""}
    </div>

    <div style="border: 1px dotted #aaa; padding: 4px 8px; margin-bottom: 8px; font-size: 9px; color: #555;">
      <strong>নির্দেশনাবলী / Terms:</strong> ১. মানি রিসিট ব্যতীত কোনো অভিযোগ বা রিফান্ড সংক্রান্ত দাবি গ্রহণযোগ্য হইবে না। ২. পরিশোধিত ফি সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য। ৩. ক্লাসে আইডি কার্ড প্রদর্শন আবশ্যক। ৪. যাবতীয় বকেয়া নির্ধারিত তারিখের মধ্যে পরিশোধযোগ্য।
    </div>

    <div class="signatures">
      <div class="sig-line">
        <div></div>
        <div style="font-weight: 500;">Student / Guardian</div>
        <div style="font-size: 8px; color: #777;">শিক্ষার্থী বা অভিভাবক</div>
      </div>
      <div style="border: 1.5px dashed #999; border-radius: 3px; padding: 3px 10px; text-align: center; background: #fafafa;">
        <div style="font-size: 9px; font-weight: bold; text-transform: uppercase; color: #555;">OFFICIAL SEAL</div>
        <div style="font-size: 8px; color: #777;">অফিসিয়াল সিল</div>
      </div>
      <div class="sig-line">
        <div></div>
        <div style="font-weight: 500;">Authorized Officer</div>
        <div style="font-size: 8px; color: #777;">অনুমোদিত কর্মকর্তা</div>
      </div>
    </div>

    <div style="font-size: 8px; color: #888; text-align: center; margin-top: 6px;">
      Coachify Academic ERP • Computer Generated Official Money Receipt • ${footerNote || "Thank you for partnering with us for quality academic education."}
    </div>
  </div>
</body>
</html>
  `.trim();
}
