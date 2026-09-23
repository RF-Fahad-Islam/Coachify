import { useState } from "react";
import { rpc, type ReceiptVerificationResult } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CheckCircle2, AlertTriangle, XCircle, Loader2, ShieldCheck } from "lucide-react";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface ReceiptVerificationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatRelativeTime(isoString: string): string {
  const delta = Math.floor((Date.now() - new Date(isoString).getTime()) / 1000);
  if (delta < 60) return `${delta}s ago`;
  if (delta < 3600) return `${Math.floor(delta / 60)}m ago`;
  if (delta < 86400) return `${Math.floor(delta / 3600)}h ago`;
  return new Date(isoString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

// ---------------------------------------------------------------------------
// Result card sub-components
// ---------------------------------------------------------------------------

interface ValidCardProps {
  result: ReceiptVerificationResult;
}

function ValidCard({ result }: ValidCardProps) {
  return (
    <div className="rounded-[var(--radius-md)] border border-emerald-300 bg-emerald-50 dark:border-emerald-700/60 dark:bg-emerald-950/30 p-3 space-y-2">
      <div className="flex items-center gap-2 text-emerald-700 dark:text-emerald-400">
        <CheckCircle2 className="h-4 w-4 shrink-0" aria-hidden="true" />
        <span className="font-semibold text-xs">Authentic Receipt — Valid</span>
      </div>
      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
        <span className="text-muted-foreground">Receipt #</span>
        <span className="font-mono font-bold text-right text-foreground">
          {result.receipt_number}
        </span>
        {result.amount_poisha != null && (
          <>
            <span className="text-muted-foreground">Amount</span>
            <span className="font-bold text-right text-foreground tabular-nums">
              {formatTaka(result.amount_poisha)}
            </span>
          </>
        )}
        {result.student_name && (
          <>
            <span className="text-muted-foreground">Student</span>
            <span className="font-medium text-right text-foreground truncate">
              {result.student_name}
            </span>
          </>
        )}
        {result.business_date && (
          <>
            <span className="text-muted-foreground">Business date</span>
            <span className="text-right text-foreground">
              {formatBusinessDate(result.business_date)}
            </span>
          </>
        )}
      </div>
    </div>
  );
}

interface VoidedCardProps {
  result: ReceiptVerificationResult;
}

function VoidedCard({ result }: VoidedCardProps) {
  return (
    <div className="rounded-[var(--radius-md)] border border-amber-300 bg-amber-50 dark:border-amber-700/60 dark:bg-amber-950/30 p-3 space-y-2">
      <div className="flex items-center gap-2 text-amber-700 dark:text-amber-400">
        <AlertTriangle className="h-4 w-4 shrink-0" aria-hidden="true" />
        <span className="font-semibold text-xs">Receipt Voided</span>
      </div>
      <div className="space-y-1 text-xs">
        <div className="flex justify-between gap-4">
          <span className="text-muted-foreground">Receipt #</span>
          <span className="font-mono font-bold text-foreground">
            {result.receipt_number}
          </span>
        </div>
        {result.void_reason && (
          <div className="flex justify-between gap-4">
            <span className="text-muted-foreground">Void reason</span>
            <span className="text-right text-foreground max-w-[160px]">
              {result.void_reason}
            </span>
          </div>
        )}
        {result.voided_at && (
          <div className="flex justify-between gap-4">
            <span className="text-muted-foreground">Voided</span>
            <span className="text-foreground">
              {formatRelativeTime(result.voided_at)}
            </span>
          </div>
        )}
        {result.voided_by_user_name && (
          <div className="flex justify-between gap-4">
            <span className="text-muted-foreground">By</span>
            <span className="text-foreground">{result.voided_by_user_name}</span>
          </div>
        )}
      </div>
    </div>
  );
}

interface ErrorCardProps {
  status: string;
}

function ErrorCard({ status }: ErrorCardProps) {
  const messages: Record<string, string> = {
    signature_mismatch:
      "The receipt signature does not match. This receipt may have been tampered with.",
    not_found: "No receipt was found matching this code or QR payload.",
  };
  return (
    <div className="rounded-[var(--radius-md)] border border-red-300 bg-red-50 dark:border-red-700/60 dark:bg-red-950/30 p-3 space-y-1.5">
      <div className="flex items-center gap-2 text-red-700 dark:text-red-400">
        <XCircle className="h-4 w-4 shrink-0" aria-hidden="true" />
        <span className="font-semibold text-xs">Verification Failed</span>
      </div>
      <p className="text-xs text-red-700/80 dark:text-red-400/80 leading-relaxed">
        {messages[status] ??
          "This receipt could not be verified. Check the code and try again."}
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function ReceiptVerificationDialog({
  open,
  onOpenChange,
}: ReceiptVerificationDialogProps) {
  const [code, setCode] = useState("");
  const [verifying, setVerifying] = useState(false);
  const [result, setResult] = useState<ReceiptVerificationResult | null>(null);
  const [rpcError, setRpcError] = useState<string | null>(null);

  const handleClose = (nextOpen: boolean) => {
    if (!nextOpen) {
      setCode("");
      setResult(null);
      setRpcError(null);
    }
    onOpenChange(nextOpen);
  };

  const handleVerify = async () => {
    const trimmed = code.trim();
    if (!trimmed) return;

    setVerifying(true);
    setResult(null);
    setRpcError(null);

    try {
      const res = await rpc("integrity.verify_receipt", { code_or_payload: trimmed });
      if (res.ok && res.data) {
        setResult(res.data);
      } else {
        setRpcError(
          res.error?.message ?? "Verification request failed. Please try again."
        );
      }
    } catch (err: unknown) {
      setRpcError(err instanceof Error ? err.message : String(err));
    } finally {
      setVerifying(false);
    }
  };

  const canSubmit = code.trim().length > 0 && !verifying;

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && canSubmit) {
      void handleVerify();
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-foreground">
            <ShieldCheck className="h-5 w-5 text-primary" aria-hidden="true" />
            Verify Payment Receipt
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          {/* Input */}
          <div className="space-y-1.5">
            <label htmlFor="receipt-code" className="font-medium text-foreground">
              Receipt Code or QR Payload
            </label>
            <Input
              id="receipt-code"
              placeholder="e.g. RCV-20261023-ABCD or QR payload"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              onKeyDown={handleKeyDown}
              className="text-xs font-mono"
              autoFocus
              autoComplete="off"
              spellCheck={false}
            />
            <p className="text-[11px] text-muted-foreground leading-relaxed">
              Enter the 12-character verification code or scan the QR payload from the
              receipt.
            </p>
          </div>

          {/* RPC error */}
          {rpcError && (
            <div className="p-2.5 rounded-[var(--radius-sm)] bg-destructive/15 text-destructive border border-destructive/20">
              {rpcError}
            </div>
          )}

          {/* Result */}
          {result && (
            <>
              {result.is_authentic && result.status === "valid" && (
                <ValidCard result={result} />
              )}
              {result.status === "voided" && <VoidedCard result={result} />}
              {(result.status === "signature_mismatch" ||
                result.status === "not_found") && <ErrorCard status={result.status} />}
              {/* Fallback for unexpected statuses */}
              {result.status !== "valid" &&
                result.status !== "voided" &&
                result.status !== "signature_mismatch" &&
                result.status !== "not_found" && (
                  <div
                    className={cn(
                      "p-2.5 rounded-[var(--radius-sm)] border text-xs",
                      result.is_authentic
                        ? "bg-emerald-50 border-emerald-300 text-emerald-800 dark:bg-emerald-950/30 dark:border-emerald-700/60 dark:text-emerald-300"
                        : "bg-red-50 border-red-300 text-red-800 dark:bg-red-950/30 dark:border-red-700/60 dark:text-red-300"
                    )}
                  >
                    Status: <strong>{result.status}</strong>
                  </div>
                )}
            </>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => handleClose(false)}
            disabled={verifying}
          >
            Cancel
          </Button>
          <Button
            type="button"
            size="sm"
            onClick={() => void handleVerify()}
            disabled={!canSubmit}
            className="gap-1.5"
          >
            {verifying ? (
              <>
                <Loader2 className="h-3.5 w-3.5 animate-spin" aria-hidden="true" />
                Verifying…
              </>
            ) : (
              <>
                <ShieldCheck className="h-3.5 w-3.5" aria-hidden="true" />
                Verify
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
