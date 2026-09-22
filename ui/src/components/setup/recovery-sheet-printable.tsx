import { useEffect, useState } from "react";
import QRCode from "qrcode";
import {
  Printer,
  ShieldAlert,
  KeyRound,
  Building,
  Calendar,
  User,
  HelpCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export interface RecoverySheetPrintableProps {
  institutionName: string;
  ownerUsername: string;
  academicSession: string;
  createdAt: string;
  recoveryKey: string;
  checksum: string;
  onPrintComplete?: () => void;
  showPrintButton?: boolean;
}

export function RecoverySheetPrintable({
  institutionName,
  ownerUsername,
  academicSession,
  createdAt,
  recoveryKey,
  checksum,
  onPrintComplete,
  showPrintButton = true,
}: RecoverySheetPrintableProps) {
  const [qrDataUrl, setQrDataUrl] = useState<string>("");

  useEffect(() => {
    if (!recoveryKey) return;
    QRCode.toDataURL(recoveryKey, {
      width: 180,
      margin: 1,
      color: {
        dark: "#000000",
        light: "#ffffff",
      },
      errorCorrectionLevel: "H",
    })
      .then((url) => setQrDataUrl(url))
      .catch((err) => console.error("Failed to generate QR code:", err));
  }, [recoveryKey]);

  const keyChunks = recoveryKey.split("-");

  const handlePrint = () => {
    window.print();
    onPrintComplete?.();
  };

  const formattedDate = new Date(createdAt || Date.now()).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  return (
    <div className="recovery-sheet-root relative w-full max-w-2xl mx-auto bg-white text-neutral-900 font-sans p-4 sm:p-6 md:p-8 rounded-xl border border-neutral-300 shadow-sm print:shadow-none print:border-none print:p-0 print:m-0 print:max-w-none">
      {/* Top action bar (hidden during print) */}
      {showPrintButton && (
        <div className="flex items-center justify-between pb-4 sm:pb-6 mb-4 sm:mb-6 border-b border-neutral-200 print:hidden">
          <div>
            <h3 className="text-sm font-semibold text-neutral-900">
              Physical Recovery Sheet
            </h3>
            <p className="text-xs text-neutral-500">
              Print this sheet and store it in your institution safe.
            </p>
          </div>
          <Button
            type="button"
            onClick={handlePrint}
            className="gap-2 bg-neutral-900 text-white hover:bg-neutral-800 text-xs sm:text-sm h-9 sm:h-10"
          >
            <Printer className="h-4 w-4" />
            Print Recovery Sheet
          </Button>
        </div>
      )}

      {/* Printable Sheet Container */}
      <div className="printable-content border-2 border-neutral-800 rounded-lg p-4 sm:p-6 md:p-8 bg-white print:border print:p-6 print:border-neutral-900">
        {/* Header Section */}
        <div className="flex items-start justify-between border-b-2 border-neutral-800 pb-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2 text-neutral-900">
              <KeyRound className="h-6 w-6 text-neutral-800" />
              <h1 className="text-xl font-bold tracking-tight uppercase">
                Coachify Disaster Recovery Sheet
              </h1>
            </div>
            <p className="text-xs text-neutral-600 font-medium">
              Confidential Institutional Master Key Envelope • Keep in Safe
            </p>
          </div>
          <div className="text-right">
            <span className="inline-block px-2 py-0.5 text-[11px] font-mono font-bold uppercase tracking-wider bg-neutral-900 text-white rounded">
              Tier-2 Offline
            </span>
          </div>
        </div>

        {/* Institution & Metadata Grid */}
        <div className="grid grid-cols-2 gap-4 py-4 border-b border-neutral-200 text-xs">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Building className="h-3.5 w-3.5 text-neutral-500" />
              <span className="text-neutral-500">Institution:</span>
              <span className="font-semibold text-neutral-900">
                {institutionName || "N/A"}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <User className="h-3.5 w-3.5 text-neutral-500" />
              <span className="text-neutral-500">Owner Username:</span>
              <span className="font-mono font-semibold text-neutral-900">
                {ownerUsername || "owner"}
              </span>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Calendar className="h-3.5 w-3.5 text-neutral-500" />
              <span className="text-neutral-500">Academic Session:</span>
              <span className="font-semibold text-neutral-900">
                {academicSession || "Current"}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-neutral-500">Initialized On:</span>
              <span className="font-semibold text-neutral-900">{formattedDate}</span>
            </div>
          </div>
        </div>

        {/* Warning Alert Banner */}
        <div className="my-4 p-3 bg-neutral-100 rounded border border-neutral-300 flex items-start gap-2.5">
          <ShieldAlert className="h-5 w-5 text-neutral-800 shrink-0 mt-0.5" />
          <div className="text-xs leading-relaxed text-neutral-800">
            <p className="font-bold">IMPORTANT: NO CLOUD VENDOR ESCROW</p>
            <p className="text-neutral-600 mt-0.5">
              Coachify operates entirely offline. Neither developers nor support staff
              have copies of your master database encryption key. If your computer
              hardware fails or Windows is re-installed, this physical paper is the{" "}
              <strong>ONLY</strong> method to recover student records and financial
              transactions.
            </p>
          </div>
        </div>

        {/* 24-Character Recovery Key Display */}
        <div className="my-5 p-4 bg-neutral-50 border-2 border-dashed border-neutral-400 rounded-md">
          <div className="text-center mb-3">
            <span className="text-xs font-bold uppercase tracking-widest text-neutral-600">
              Owner Recovery Key (24 Characters)
            </span>
          </div>

          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-2 text-center">
            {keyChunks.map((chunk, idx) => {
              const isLast = idx === keyChunks.length - 1;
              return (
                <div
                  key={idx}
                  className={`p-2 rounded-lg border font-mono text-xs sm:text-sm md:text-base font-bold tracking-wider ${
                    isLast
                      ? "bg-neutral-900 text-white border-neutral-900 ring-2 ring-neutral-900/20"
                      : "bg-white text-neutral-900 border-neutral-300"
                  }`}
                >
                  <div className="text-[9px] uppercase tracking-wider text-neutral-400 font-sans">
                    Block {idx + 1}
                  </div>
                  <div className="mt-0.5">{chunk}</div>
                </div>
              );
            })}
          </div>

          {/* Checksum callout */}
          <div className="mt-3 flex items-center justify-between px-2 text-xs text-neutral-600">
            <span>
              Confirmation Checksum (Last 4):{" "}
              <strong className="font-mono text-neutral-900">{checksum}</strong>
            </span>
            <span className="text-[11px] text-neutral-500">
              Crockford Base32 + CRC-16
            </span>
          </div>
        </div>

        {/* QR Code and Step-by-Step Recovery Instructions */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2 items-center">
          {/* QR Code */}
          <div className="flex flex-col items-center justify-center p-3 border border-neutral-200 rounded bg-white">
            {qrDataUrl ? (
              <img
                src={qrDataUrl}
                alt="Recovery Key QR Code"
                className="w-32 h-32 object-contain"
              />
            ) : (
              <div className="w-32 h-32 flex items-center justify-center bg-neutral-100 text-xs text-neutral-400">
                Generating QR...
              </div>
            )}
            <span className="text-[10px] text-neutral-500 font-mono mt-1 text-center">
              Scan during Emergency Unlock
            </span>
          </div>

          {/* Recovery Instructions */}
          <div className="sm:col-span-2 text-xs text-neutral-700 space-y-2">
            <div className="flex items-center gap-1.5 font-bold text-neutral-900">
              <HelpCircle className="h-3.5 w-3.5" />
              <span>How to Recover on a New Computer:</span>
            </div>
            <ol className="list-decimal list-inside space-y-1 text-neutral-600 leading-relaxed text-[11px]">
              <li>Install Coachify on the replacement Windows workstation.</li>
              <li>Connect your external backup drive or copy your data folder.</li>
              <li>
                Launch Coachify and select{" "}
                <strong>&quot;Disaster Recovery / Unlock Database&quot;</strong>.
              </li>
              <li>Enter or scan the 24-character key printed on this sheet.</li>
              <li>Set a new Owner password and resume operations immediately.</li>
            </ol>
            <div className="pt-2 text-[10px] text-neutral-500 border-t border-neutral-200">
              বাংলা: এই রিকভারি শীটটি সুরক্ষিত স্থানে রাখুন। হার্ডওয়্যার নষ্ট হলে ডাটা
              পুনরুদ্ধারের জন্য এটি আবশ্যক।
            </div>
          </div>
        </div>

        {/* Footer Sign-off Block */}
        <div className="mt-6 pt-4 border-t border-neutral-300 grid grid-cols-2 gap-8 text-[11px] text-neutral-600">
          <div>
            <div className="border-b border-neutral-400 pb-8 mb-1"></div>
            <span>Authorized Signature &amp; Seal</span>
          </div>
          <div>
            <div className="border-b border-neutral-400 pb-8 mb-1"></div>
            <span>Date Placed in Vault</span>
          </div>
        </div>
      </div>
    </div>
  );
}
