import type { PrintableReceiptData } from "../../gen/rpc";
import { formatTaka } from "../../lib/utils";

interface A4DualReceiptProps {
  receipt: PrintableReceiptData;
  footerNote?: string | null;
}

/**
 * Academic coaching center crest emblem for formal A4 receipts.
 */
function AcademicEmblemSvg({ className = "h-11 w-11" }: { className?: string }) {
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
<svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg" width="44" height="44" style="display: block; color: #111;">
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
        ? `${formatTaka(-receipt.balance_after_poisha)} Advance`
        : "৳ 0 Cleared";

  return (
    <div className="border border-gray-400 rounded p-3.5 bg-white text-black text-xs leading-tight">
      {/* Header with Academic Crest Logo and Coaching Branding */}
      <div className="flex justify-between items-start border-b-2 border-gray-900 pb-2 mb-2">
        <div className="flex items-center gap-2.5">
          <AcademicEmblemSvg className="h-11 w-11 text-gray-950 flex-shrink-0" />
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
              <p className="text-[10px] text-gray-700 mt-0.5">
                {receipt.institution_address}
              </p>
            )}
            <div className="flex flex-wrap items-center gap-x-2 text-[9.5px] text-gray-600">
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
          <div className="inline-flex items-center gap-1 border-2 border-gray-900 px-2 py-0.5 font-black uppercase text-[10px] bg-gray-100 tracking-wider">
            <span>MONEY RECEIPT</span>
            <span>•</span>
            <span>মানি রিসিট</span>
          </div>
          <div className="mt-1 text-[11px] font-black uppercase text-gray-900">
            {copyTitle} • {copyTitleBn}
          </div>
          {receipt.is_reprint && (
            <div className="text-[9.5px] font-bold text-amber-800 bg-amber-50 border border-amber-300 px-1 py-0.2 mt-0.5 inline-block">
              REPRINT COPY / অনুলিপি
            </div>
          )}
          <div className="mt-1 font-mono text-xs font-extrabold text-gray-950">
            {receipt.receipt_no}
          </div>
          <div className="text-[10px] text-gray-600">
            Date: {receipt.business_date} • Time: {receipt.received_at.slice(11, 19)}
          </div>
        </div>
      </div>

      {/* Student, Batch, and Payment Metadata Grid */}
      <div className="grid grid-cols-3 gap-x-3 gap-y-1 text-[10.5px] bg-gray-50/80 p-2 rounded border border-gray-300 mb-2">
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
          <span className="font-medium">{receipt.batch_name || "General"}</span>
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

      {/* Itemized Line Items Table */}
      <table className="w-full border-collapse text-[10.5px] mb-1.5 border border-gray-300">
        <thead>
          <tr className="border-b-2 border-gray-700 bg-gray-100 text-gray-800">
            <th className="py-1 px-2 text-left font-bold w-7">#</th>
            <th className="py-1 px-2 text-left font-bold">Fee Description / বিবরণ</th>
            <th className="py-1 px-2 text-left font-bold w-24">Period</th>
            <th className="py-1 px-2 text-right font-bold w-28">Amount (BDT)</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {receipt.items.map((item, idx) => (
            <tr key={idx}>
              <td className="py-1 px-2 text-gray-500">{idx + 1}</td>
              <td className="py-1 px-2 font-medium">
                {item.fee_head_name}
                {item.fee_head_name_bn && (
                  <span className="text-gray-600 ml-1">({item.fee_head_name_bn})</span>
                )}
              </td>
              <td className="py-1 px-2 text-gray-600">{item.period || "N/A"}</td>
              <td className="py-1 px-2 text-right font-bold tabular-nums">
                {formatTaka(item.amount_poisha)}
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr className="border-t-2 border-gray-800 font-bold bg-gray-50">
            <td colSpan={3} className="py-1 px-2 text-right text-gray-900">
              TOTAL RECEIVED / সর্বমোট আদায়:
            </td>
            <td className="py-1 px-2 text-right text-xs tabular-nums text-gray-950">
              {formatTaka(receipt.amount_poisha)}
            </td>
          </tr>
        </tfoot>
      </table>

      {/* Amount in Words */}
      <div className="space-y-0.5 mb-2 text-[10px] text-gray-800">
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

      {/* Coaching Center Rules and Terms */}
      <div className="border border-dotted border-gray-400 p-1 mb-2 text-[8.5px] text-gray-600 leading-tight">
        <span className="font-bold text-gray-800 mr-1">নির্দেশনাবলী / Rules:</span>
        <span>
          ১. মানি রিসিট ব্যতীত ফি সংক্রান্ত কোনো দাবি গ্রহণযোগ্য নহে। ২. প্রদেয় ফি
          সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য। ৩. ক্লাসে স্টুডেন্ট আইডি কার্ড প্রদর্শন
          আবশ্যক।
        </span>
      </div>

      {/* Triple Signatures with Official Seal Box */}
      <div className="pt-2 flex justify-between items-end text-[9.5px] text-gray-700 border-t border-gray-300">
        <div className="text-center w-36">
          <div className="border-b border-gray-500 pb-1 mb-1"></div>
          <div className="font-medium">Student / Guardian</div>
          <div className="text-[8.5px] text-gray-500">শিক্ষার্থী বা অভিভাবক</div>
        </div>

        {/* Official Seal Stamp Box */}
        <div className="border-2 border-dashed border-gray-400 rounded px-3 py-1 text-center bg-gray-50/50">
          <div className="text-[9px] font-bold uppercase tracking-wider text-gray-600">
            OFFICIAL SEAL
          </div>
          <div className="text-[8px] text-gray-500">অফিসিয়াল সিল</div>
        </div>

        <div className="text-center w-36">
          <div className="border-b border-gray-500 pb-1 mb-1"></div>
          <div className="font-medium">Authorized Officer</div>
          <div className="text-[8.5px] text-gray-500">অনুমোদিত কর্মকর্তা</div>
        </div>
      </div>

      {/* System Watermark */}
      <div className="text-[8px] text-gray-400 text-center mt-1">
        Coachify Academic ERP • Computer Generated Official Money Receipt •{" "}
        {footerNote || "Keep this receipt safe for future reference."}
      </div>
    </div>
  );
}

export function A4DualReceipt({ receipt, footerNote }: A4DualReceiptProps) {
  return (
    <div className="w-[200mm] max-w-[200mm] mx-auto p-3 bg-white text-black font-sans space-y-3 print:p-0 print:w-[190mm]">
      {/* Top Half: Student Copy */}
      <ReceiptSingleVoucher
        receipt={receipt}
        copyTitle="STUDENT COPY"
        copyTitleBn="শিক্ষার্থী কপি"
        footerNote={footerNote}
      />

      {/* Perforation Divider Line */}
      <div className="relative py-2 flex items-center justify-center">
        <div className="w-full border-b-2 border-dashed border-gray-400" />
        <div className="absolute bg-white px-3 text-xs text-gray-600 font-mono flex items-center gap-2 select-none border border-gray-300 rounded-full shadow-xs">
          <span>✂</span>
          <span className="font-semibold tracking-wider">
            CUT HERE / ছিঁড়ে নিন (শিক্ষার্থী ও অফিস কপি)
          </span>
          <span>✂</span>
        </div>
      </div>

      {/* Bottom Half: Office Copy */}
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
          <td style="padding: 2.5px 6px; text-align: left; color: #555;">${idx + 1}</td>
          <td style="padding: 2.5px 6px; text-align: left; font-weight: 500;">
            ${item.fee_head_name}
            ${item.fee_head_name_bn ? `<span style="color: #666; font-size: 9.5px; margin-left: 4px;">(${item.fee_head_name_bn})</span>` : ""}
          </td>
          <td style="padding: 2.5px 6px; text-align: left; color: #555;">${item.period || "N/A"}</td>
          <td style="padding: 2.5px 6px; text-align: right; font-weight: bold; font-variant-numeric: tabular-nums;">
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
      <div style="border: 1px solid #999; border-radius: 4px; padding: 10px 12px; background: #fff; font-size: 10.5px; line-height: 1.25;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; border-bottom: 2px solid #222; padding-bottom: 6px; margin-bottom: 6px;">
          <div style="display: flex; align-items: center; gap: 10px;">
            ${EMBLEM_SVG_STRING}
            <div>
              ${receipt.institution_name_bn ? `<div style="font-size: 15px; font-weight: 800; color: #111;">${receipt.institution_name_bn}</div>` : ""}
              <div style="font-size: 11px; font-weight: 800; text-transform: uppercase; color: #222; letter-spacing: 0.5px;">${receipt.institution_name}</div>
              ${receipt.institution_address ? `<div style="font-size: 9.5px; color: #444; margin-top: 1px;">${receipt.institution_address}</div>` : ""}
              <div style="font-size: 9px; color: #555;">
                ${receipt.institution_phone ? `Helpline: ${receipt.institution_phone} ` : ""}
                ${receipt.institution_email ? `• Email: ${receipt.institution_email} ` : ""}
                ${receipt.institution_website ? `• Web: ${receipt.institution_website}` : ""}
              </div>
            </div>
          </div>
          <div style="text-align: right; flex-shrink: 0;">
            <div style="display: inline-block; border: 1.5px solid #222; padding: 1.5px 6px; font-weight: 800; font-size: 9.5px; background: #f0f0f0; letter-spacing: 0.5px;">
              MONEY RECEIPT • মানি রিসিট
            </div>
            <div style="font-size: 10.5px; font-weight: 800; text-transform: uppercase; margin-top: 2px; color: #111;">
              ${copyTitle} • ${copyTitleBn}
            </div>
            ${receipt.is_reprint ? `<div style="font-size: 9px; font-weight: bold; color: #92400e; background: #fef3c7; border: 1px solid #fcd34d; padding: 1px 4px; margin-top: 2px; display: inline-block;">REPRINT COPY / অনুলিপি</div>` : ""}
            <div style="font-size: 11px; font-weight: 800; font-family: monospace; margin-top: 2px;">${receipt.receipt_no}</div>
            <div style="font-size: 9.5px; color: #555;">Date: ${receipt.business_date} • ${receipt.received_at.slice(11, 19)}</div>
          </div>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 3px 10px; background: #f8f8f8; padding: 5px 8px; border-radius: 4px; border: 1px solid #e0e0e0; margin-bottom: 6px; font-size: 10px;">
          <div><span style="color: #666;">Student ID: </span><strong>${receipt.student_code}</strong></div>
          <div><span style="color: #666;">Program: </span>${receipt.program_name || "General Academic"}</div>
          <div><span style="color: #666;">Mode: </span><strong style="text-transform: uppercase;">${receipt.method}</strong> ${receipt.reference ? `(${receipt.reference})` : ""}</div>

          <div><span style="color: #666;">Student: </span><strong>${receipt.student_name}</strong> ${receipt.student_name_bn ? `(${receipt.student_name_bn})` : ""}</div>
          <div><span style="color: #666;">Batch / Roll: </span>${receipt.batch_name || "General"} ${receipt.roll_no ? `(Roll ${receipt.roll_no})` : ""}</div>
          <div><span style="color: #666;">Cashier: </span><strong>${receipt.collector_name}</strong></div>

          <div style="grid-column: span 2;"><span style="color: #666;">Guardian: </span>${receipt.guardian_name || "N/A"} ${receipt.guardian_phone ? `(${receipt.guardian_phone})` : ""}</div>
          <div><span style="color: #666;">Balance: </span><strong>${balanceText}</strong></div>
        </div>

        <table style="width: 100%; border-collapse: collapse; font-size: 10px; margin-bottom: 6px; border: 1px solid #ccc;">
          <thead>
            <tr style="border-bottom: 1.5px solid #444; background: #ececec;">
              <th style="padding: 2.5px 6px; text-align: left; width: 22px;">#</th>
              <th style="padding: 2.5px 6px; text-align: left;">Fee Description / বিবরণ</th>
              <th style="padding: 2.5px 6px; text-align: left; width: 80px;">Period</th>
              <th style="padding: 2.5px 6px; text-align: right; width: 85px;">Amount (BDT)</th>
            </tr>
          </thead>
          <tbody>
            ${itemsRows}
          </tbody>
          <tfoot>
            <tr style="border-top: 1.5px solid #444; background: #f7f7f7; font-weight: bold;">
              <td colspan="3" style="padding: 3px 6px; text-align: right;">TOTAL RECEIVED / সর্বমোট আদায়:</td>
              <td style="padding: 3px 6px; text-align: right; font-size: 11px; font-variant-numeric: tabular-nums;">${formatTaka(receipt.amount_poisha)}</td>
            </tr>
          </tfoot>
        </table>

        <div style="font-size: 9.5px; color: #333; margin-bottom: 6px; line-height: 1.2;">
          <div><strong>In Words: </strong><em>${receipt.amount_in_words_en}</em></div>
          ${receipt.amount_in_words_bn ? `<div><strong>কথায়: </strong>${receipt.amount_in_words_bn}</div>` : ""}
        </div>

        <div style="border: 1px dotted #aaa; padding: 3px 6px; margin-bottom: 6px; font-size: 8.5px; color: #555;">
          <strong>নির্দেশনাবলী / Rules:</strong> ১. মানি রিসিট ব্যতীত ফি সংক্রান্ত কোনো দাবি গ্রহণযোগ্য নহে। ২. প্রদেয় ফি সম্পূর্ণ অফেরতযোগ্য ও অহস্তান্তরযোগ্য। ৩. ক্লাসে আইডি কার্ড প্রদর্শন আবশ্যক।
        </div>

        <div style="display: flex; justify-content: space-between; align-items: flex-end; font-size: 9px; color: #555; border-top: 1px solid #ccc; padding-top: 6px;">
          <div style="text-align: center; width: 120px;">
            <div style="border-bottom: 1px solid #777; margin-bottom: 3px;"></div>
            <div style="font-weight: 500;">Student / Guardian</div>
            <div style="font-size: 8px; color: #777;">শিক্ষার্থী বা অভিভাবক</div>
          </div>
          <div style="border: 1.5px dashed #999; border-radius: 3px; padding: 3px 8px; text-align: center; background: #fafafa;">
            <div style="font-size: 8.5px; font-weight: bold; text-transform: uppercase; color: #555;">OFFICIAL SEAL</div>
            <div style="font-size: 7.5px; color: #777;">অফিসিয়াল সিল</div>
          </div>
          <div style="text-align: center; width: 120px;">
            <div style="border-bottom: 1px solid #777; margin-bottom: 3px;"></div>
            <div style="font-weight: 500;">Authorized Officer</div>
            <div style="font-size: 8px; color: #777;">অনুমোদিত কর্মকর্তা</div>
          </div>
        </div>

        <div style="font-size: 8px; color: #888; text-align: center; margin-top: 4px;">
          Coachify Academic ERP • Computer Generated Official Money Receipt • ${footerNote || "Keep this receipt safe for future reference."}
        </div>
      </div>
    `;
  };

  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Receipt ${receipt.receipt_no} - A4 Dual Copy</title>
  <style>
    @page {
      size: A4 portrait;
      margin: 8mm 10mm;
    }
    body {
      margin: 0;
      padding: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Kalpurush", sans-serif;
      background: #fff;
      color: #000;
      line-height: 1.25;
    }
    .divider {
      position: relative;
      text-align: center;
      margin: 8px 0;
    }
    .divider-line {
      border-bottom: 2px dashed #888;
      width: 100%;
    }
    .divider-badge {
      position: absolute;
      top: -8px;
      left: 50%;
      transform: translateX(-50%);
      background: #fff;
      padding: 0 10px;
      font-size: 10px;
      color: #555;
      font-family: monospace;
      border: 1px solid #ccc;
      border-radius: 12px;
    }
  </style>
</head>
<body>
  ${renderHalf("STUDENT COPY", "শিক্ষার্থী কপি")}

  <div class="divider">
    <div class="divider-line"></div>
    <div class="divider-badge">✂ CUT HERE / ছিঁড়ে নিন (শিক্ষার্থী ও অফিস কপি) ✂</div>
  </div>

  ${renderHalf("OFFICE COPY", "অফিস কপি")}
</body>
</html>
  `.trim();
}
