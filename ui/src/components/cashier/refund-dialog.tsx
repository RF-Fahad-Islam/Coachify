import { useState } from "react";
import { rpc, type PaymentReceipt } from "@/gen/rpc";
import { formatTaka } from "@/lib/utils";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Loader2, ArrowDownLeft } from "lucide-react";

export interface RefundDialogProps {
  payment: PaymentReceipt | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function RefundDialog({
  payment,
  open,
  onOpenChange,
  onSuccess,
}: RefundDialogProps) {
  const [amountTaka, setAmountTaka] = useState("");
  const [method, setMethod] = useState("cash");
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!payment) return null;

  const maxRefundPoisha = payment.unallocated_amount_poisha;
  const maxRefundTaka = (maxRefundPoisha / 100).toFixed(2);

  const handleRefund = async () => {
    const numTaka = parseFloat(amountTaka);
    if (isNaN(numTaka) || numTaka <= 0) {
      setError("Please enter a valid refund amount");
      return;
    }

    const amountPoisha = Math.round(numTaka * 100);
    if (amountPoisha > maxRefundPoisha) {
      setError(
        `Refund amount cannot exceed unallocated credit of ${formatTaka(maxRefundPoisha)}`
      );
      return;
    }

    if (!reason.trim()) {
      setError("Please provide a reason for the refund");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const res = await rpc("refunds.create", {
        refund: {
          payment_id: payment.id,
          amount_poisha: amountPoisha,
          method,
          reason: reason.trim(),
          business_date: null,
        },
      });

      if (res.ok) {
        setAmountTaka("");
        setReason("");
        onOpenChange(false);
        onSuccess();
      } else {
        setError(res.error?.message || "Failed to process refund");
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
          <DialogTitle className="flex items-center gap-2 text-foreground">
            <ArrowDownLeft className="h-5 w-5 text-primary" />
            Issue Payment Refund
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          <div className="p-3 bg-muted/60 rounded-[var(--radius-md)] space-y-1.5 border border-border">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Original Payment:</span>
              <span className="font-mono font-bold text-foreground">
                {payment.receipt_no}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total Paid:</span>
              <span className="font-semibold text-foreground">
                {formatTaka(payment.amount_poisha)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Available Advance Credit:</span>
              <span className="font-bold text-success font-mono">
                {formatTaka(maxRefundPoisha)}
              </span>
            </div>
          </div>

          <div className="space-y-1.5">
            <div className="flex justify-between items-center">
              <label htmlFor="refund-amount" className="font-medium text-foreground">
                Refund Amount (৳) <span className="text-destructive">*</span>
              </label>
              <button
                type="button"
                onClick={() => setAmountTaka(maxRefundTaka)}
                className="text-[11px] text-primary hover:underline font-medium cursor-pointer"
              >
                Max ({formatTaka(maxRefundPoisha)})
              </button>
            </div>
            <Input
              id="refund-amount"
              type="number"
              step="any"
              min="1"
              placeholder="e.g. 500"
              value={amountTaka}
              onChange={(e) => setAmountTaka(e.target.value)}
              className="text-xs font-mono font-medium"
              autoFocus
            />
          </div>

          <div className="space-y-1.5">
            <label htmlFor="refund-method" className="font-medium text-foreground">
              Disbursement Method
            </label>
            <select
              id="refund-method"
              value={method}
              onChange={(e) => setMethod(e.target.value)}
              className="w-full h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="cash">Cash (from drawer)</option>
              <option value="bkash">bKash</option>
              <option value="nagad">Nagad</option>
              <option value="bank">Bank Transfer</option>
              <option value="other">Other</option>
            </select>
          </div>

          <div className="space-y-1.5">
            <label htmlFor="refund-reason" className="font-medium text-foreground">
              Reason <span className="text-destructive">*</span>
            </label>
            <Input
              id="refund-reason"
              placeholder="e.g. Course withdrawal, excess advance payment"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="text-xs"
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
            size="sm"
            onClick={() => void handleRefund()}
            disabled={submitting || !amountTaka || !reason.trim()}
            className="gap-1.5"
          >
            {submitting ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
            Disburse Refund
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
