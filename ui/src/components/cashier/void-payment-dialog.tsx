import { useState } from "react";
import { rpc, type PaymentReceipt } from "@/gen/rpc";
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
import { AlertTriangle, Loader2 } from "lucide-react";

export interface VoidPaymentDialogProps {
  payment: PaymentReceipt | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function VoidPaymentDialog({
  payment,
  open,
  onOpenChange,
  onSuccess,
}: VoidPaymentDialogProps) {
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!payment) return null;

  const handleVoid = async () => {
    if (!reason.trim()) {
      setError("Please provide a reason for voiding this payment");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const res = await rpc("payments.void", {
        payment_id: payment.id,
        reason: reason.trim(),
      });

      if (res.ok) {
        setReason("");
        onOpenChange(false);
        onSuccess();
      } else {
        setError(res.error?.message || "Failed to void payment");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-destructive">
            <AlertTriangle className="h-5 w-5" />
            Void Payment Receipt
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          <div className="p-3 bg-muted/60 rounded-[var(--radius-md)] space-y-1.5 border border-border">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Receipt Number:</span>
              <span className="font-mono font-bold text-foreground">
                {payment.receipt_no}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Amount:</span>
              <span className="font-bold text-foreground">
                {formatTaka(payment.amount_poisha)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Method:</span>
              <span className="capitalize font-medium text-foreground">
                {payment.method}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Date:</span>
              <span className="text-foreground">
                {formatBusinessDate(payment.business_date)}
              </span>
            </div>
          </div>

          <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-[var(--radius-md)] text-destructive">
            <p className="font-semibold mb-1">Important:</p>
            <p className="leading-relaxed">
              Voiding cancels this payment receipt and automatically restores any
              previously satisfied charges to unpaid status. This operation is recorded in
              the immutable audit log.
            </p>
          </div>

          <div className="space-y-1.5">
            <label htmlFor="void-reason" className="font-medium text-foreground">
              Reason for voiding <span className="text-destructive">*</span>
            </label>
            <Input
              id="void-reason"
              placeholder="e.g. Incorrect amount typed, wrong student selected"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="text-xs"
              autoFocus
            />
          </div>

          {error && (
            <div className="p-2.5 rounded-[var(--radius-sm)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onOpenChange(false)}
            disabled={submitting}
          >
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            size="sm"
            onClick={() => void handleVoid()}
            disabled={submitting || !reason.trim()}
            className="gap-1.5"
          >
            {submitting ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
            Confirm Void
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
