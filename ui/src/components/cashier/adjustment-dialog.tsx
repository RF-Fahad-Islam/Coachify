import { useState } from "react";
import { rpc, type ChargeItem } from "@/gen/rpc";
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
import { Loader2, Sliders } from "lucide-react";

export interface AdjustmentDialogProps {
  studentId: string;
  studentName: string;
  charges?: ChargeItem[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
}

export function AdjustmentDialog({
  studentId,
  studentName,
  charges = [],
  open,
  onOpenChange,
  onSuccess,
}: AdjustmentDialogProps) {
  const [type, setType] = useState<"waiver" | "fine" | "correction">("waiver");
  const [selectedChargeId, setSelectedChargeId] = useState<string>("");
  const [amountTaka, setAmountTaka] = useState("");
  const [reason, setReason] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openCharges = charges.filter(
    (c) => c.status === "open" || c.status === "partial"
  );
  const selectedCharge = openCharges.find((c) => c.id === selectedChargeId);

  const handleAdjust = async () => {
    const numTaka = parseFloat(amountTaka);
    if (isNaN(numTaka) || numTaka <= 0) {
      setError("Please enter a valid adjustment amount");
      return;
    }

    const amountPoisha = Math.round(numTaka * 100);

    if (type === "waiver" && selectedCharge) {
      const remainingDue = selectedCharge.remaining_due_poisha;
      if (amountPoisha > remainingDue) {
        setError(
          `Waiver cannot exceed remaining charge due of ${formatTaka(remainingDue)}`
        );
        return;
      }
    }

    if (!reason.trim()) {
      setError("Please provide a justification for this adjustment");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      const res = await rpc("adjustments.create", {
        adjustment: {
          student_id: studentId,
          charge_id: selectedChargeId || null,
          type,
          amount_poisha: amountPoisha,
          reason: reason.trim(),
          business_date: null,
        },
      });

      if (res.ok) {
        setAmountTaka("");
        setReason("");
        setSelectedChargeId("");
        onOpenChange(false);
        onSuccess();
      } else {
        setError(res.error?.message || "Failed to record adjustment");
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
            <Sliders className="h-5 w-5 text-primary" />
            Financial Adjustment: {studentName}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2 text-xs">
          <div className="space-y-1.5">
            <label htmlFor="adjust-type" className="font-medium text-foreground">
              Adjustment Type
            </label>
            <div className="grid grid-cols-3 gap-2">
              {(["waiver", "fine", "correction"] as const).map((t) => (
                <button
                  key={t}
                  type="button"
                  onClick={() => setType(t)}
                  className={`py-2 px-3 rounded-[var(--radius-md)] border text-xs font-semibold capitalize cursor-pointer transition-all ${
                    type === t
                      ? "border-primary bg-primary-subtle text-primary shadow-xs"
                      : "border-border bg-card text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>

          {type === "waiver" && openCharges.length > 0 && (
            <div className="space-y-1.5">
              <label htmlFor="adjust-charge" className="font-medium text-foreground">
                Apply to Specific Charge (Optional)
              </label>
              <select
                id="adjust-charge"
                value={selectedChargeId}
                onChange={(e) => setSelectedChargeId(e.target.value)}
                className="w-full h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="">General Balance Adjustment</option>
                {openCharges.map((c) => {
                  const rem = c.remaining_due_poisha;
                  return (
                    <option key={c.id} value={c.id}>
                      {c.fee_head_name} ({c.period}) • Due {formatTaka(rem)}
                    </option>
                  );
                })}
              </select>
            </div>
          )}

          <div className="space-y-1.5">
            <label htmlFor="adjust-amount" className="font-medium text-foreground">
              Amount (৳) <span className="text-destructive">*</span>
            </label>
            <Input
              id="adjust-amount"
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
            <label htmlFor="adjust-reason" className="font-medium text-foreground">
              Justification / Reason <span className="text-destructive">*</span>
            </label>
            <Input
              id="adjust-reason"
              placeholder="e.g. Merit waiver approved by Director, late fine"
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
            onClick={() => void handleAdjust()}
            disabled={submitting || !amountTaka || !reason.trim()}
            className="gap-1.5"
          >
            {submitting ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
            Record Adjustment
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
