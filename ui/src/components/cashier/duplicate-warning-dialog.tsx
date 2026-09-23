import { useState } from "react";
import { type DuplicateWarningResult } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AlertTriangle, ShieldAlert } from "lucide-react";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface DuplicateWarningDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  warning: DuplicateWarningResult | null;
  onConfirmOverride: (reason: string) => void;
  onCancel: () => void;
}

// ---------------------------------------------------------------------------
// Validation helper
// ---------------------------------------------------------------------------

function hasMinWords(text: string, min: number): boolean {
  return text.trim().split(/\s+/).filter(Boolean).length >= min;
}

function formatTimestamp(isoString: string): string {
  try {
    return new Date(isoString).toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return isoString;
  }
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function DuplicateWarningDialog({
  open,
  onOpenChange,
  warning,
  onConfirmOverride,
  onCancel,
}: DuplicateWarningDialogProps) {
  const [reason, setReason] = useState("");

  const isReasonValid = hasMinWords(reason, 2);

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
    setReason("");
  };

  const handleProceed = () => {
    if (!isReasonValid) return;
    onConfirmOverride(reason.trim());
    onOpenChange(false);
    setReason("");
  };

  // Guard: render nothing if no warning data
  if (!warning) return null;

  const existingPayment = warning.existing_payment ?? null;

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) handleCancel();
        else onOpenChange(true);
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-amber-700 dark:text-amber-400">
            <ShieldAlert className="h-5 w-5 shrink-0" aria-hidden="true" />
            Duplicate Payment Warning
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          {/* Warning message */}
          <div className="flex gap-3 p-3 rounded-[var(--radius-md)] border border-amber-300 bg-amber-50 dark:border-amber-700/60 dark:bg-amber-950/30">
            <AlertTriangle
              className="h-4 w-4 mt-0.5 shrink-0 text-amber-600 dark:text-amber-400"
              aria-hidden="true"
            />
            <p className="leading-relaxed text-amber-800 dark:text-amber-300 font-medium">
              {warning.message}
            </p>
          </div>

          {/* Existing payment mini-table */}
          {existingPayment && (
            <div className="space-y-1.5">
              <p className="font-semibold text-foreground text-[11px] uppercase tracking-wide">
                Existing Payment
              </p>
              <table className="w-full text-xs border-collapse">
                <thead>
                  <tr className="border-b border-border bg-muted/50">
                    <th className="text-left px-2 py-1.5 font-semibold text-muted-foreground">
                      Receipt #
                    </th>
                    <th className="text-right px-2 py-1.5 font-semibold text-muted-foreground">
                      Amount
                    </th>
                    <th className="text-right px-2 py-1.5 font-semibold text-muted-foreground">
                      Business Date
                    </th>
                    <th className="text-right px-2 py-1.5 font-semibold text-muted-foreground">
                      Created
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr className="border-b border-border/60">
                    <td className="px-2 py-1.5 font-mono font-bold text-foreground">
                      {existingPayment.receipt_no}
                    </td>
                    <td className="px-2 py-1.5 text-right tabular-nums font-semibold text-foreground">
                      {formatTaka(existingPayment.amount_poisha)}
                    </td>
                    <td className="px-2 py-1.5 text-right text-foreground">
                      {formatBusinessDate(existingPayment.business_date)}
                    </td>
                    <td className="px-2 py-1.5 text-right text-muted-foreground whitespace-nowrap">
                      {formatTimestamp(existingPayment.created_at)}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}

          {/* Override justification */}
          <div className="space-y-1.5">
            <label htmlFor="override-reason" className="font-medium text-foreground">
              Override Justification{" "}
              <span className="text-destructive" aria-hidden="true">
                *
              </span>
            </label>
            <textarea
              id="override-reason"
              rows={3}
              placeholder="e.g. Student confirmed the earlier payment failed at gateway — proceeding with re-collection"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="w-full resize-none rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground/70 focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-shadow"
              aria-describedby="override-reason-hint"
            />
            <p id="override-reason-hint" className="text-[11px] text-muted-foreground">
              Minimum 2 words required. This justification is recorded in the audit log.
            </p>
            {reason.trim().length > 0 && !isReasonValid && (
              <p className="text-[11px] text-destructive" role="alert">
                Please enter at least 2 words to justify the override.
              </p>
            )}
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button type="button" variant="outline" size="sm" onClick={handleCancel}>
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            size="sm"
            onClick={handleProceed}
            disabled={!isReasonValid}
            aria-label="Proceed with payment override"
          >
            Proceed Anyway
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
