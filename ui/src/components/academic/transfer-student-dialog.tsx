import { useState, useEffect } from "react";
import {
  rpc,
  type BatchRosterItem,
  type BatchSummary,
  type EnrollmentDetail,
} from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { ArrowRightLeft, AlertTriangle, Loader2 } from "lucide-react";

export interface TransferStudentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentBatchId: string;
  currentBatchName: string;
  rosterItem: BatchRosterItem | null;
  onTransferred: (enrollment: EnrollmentDetail) => void;
}

export function TransferStudentDialog({
  open,
  onOpenChange,
  currentBatchId,
  currentBatchName,
  rosterItem,
  onTransferred,
}: TransferStudentDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [availableBatches, setAvailableBatches] = useState<BatchSummary[]>([]);
  const [loadingBatches, setLoadingBatches] = useState(false);

  const [toBatchId, setToBatchId] = useState("");
  const [newRoll, setNewRoll] = useState("");
  const [transferDate, setTransferDate] = useState(
    () => new Date().toISOString().split("T")[0]
  );
  const [notes, setNotes] = useState("");
  const [allowOvercapacity, setAllowOvercapacity] = useState(false);

  useEffect(() => {
    if (open) {
      setError(null);
      setToBatchId("");
      setNewRoll("");
      setNotes("");
      setAllowOvercapacity(false);

      // Load active batches
      void (async () => {
        setLoadingBatches(true);
        try {
          const res = await rpc("batches.list", { status: "active" });
          if (res.ok && res.data) {
            // Exclude current batch
            setAvailableBatches(res.data.filter((b) => b.id !== currentBatchId));
          }
        } catch {
          // ignore
        } finally {
          setLoadingBatches(false);
        }
      })();
    }
  }, [open, currentBatchId]);

  const targetBatch = availableBatches.find((b) => b.id === toBatchId);
  const isTargetFull = targetBatch
    ? targetBatch.enrolled_count >= targetBatch.capacity
    : false;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!rosterItem) return;
    if (!toBatchId) {
      setError("Please select the destination batch");
      return;
    }

    if (isTargetFull && !allowOvercapacity) {
      setError(
        "Destination batch is at capacity limit. Please check the overcapacity override box to continue."
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const rollNum = newRoll ? parseInt(newRoll, 10) : null;
      const res = await rpc("enrollments.transfer", {
        transfer: {
          student_id: rosterItem.student_id,
          from_batch_id: currentBatchId,
          to_batch_id: toBatchId,
          new_roll: rollNum,
          transfer_date: transferDate,
          notes: notes.trim() || null,
        },
        allow_overcapacity: allowOvercapacity,
      });

      if (res.ok && res.data) {
        onOpenChange(false);
        onTransferred(res.data);
      } else {
        setError(res.error?.message || "Transfer failed");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <ArrowRightLeft className="h-5 w-5 text-primary" />
            Transfer Student to Another Batch
          </DialogTitle>
        </DialogHeader>

        {rosterItem && (
          <div className="p-3 rounded-[var(--radius-md)] bg-muted/40 border border-border/80 space-y-1.5 text-xs">
            <div className="flex justify-between items-center">
              <span className="font-semibold text-foreground text-sm">
                {rosterItem.student_name}
              </span>
              <span className="font-mono text-muted-foreground">
                {rosterItem.student_code}
              </span>
            </div>
            <div className="flex items-center gap-2 text-muted-foreground text-[11px]">
              <span>
                Current Batch: <strong>{currentBatchName}</strong>
              </span>
              <span>•</span>
              <span>
                Roll: <strong>#{rosterItem.roll || "None"}</strong>
              </span>
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Destination Batch */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Destination Batch <span className="text-destructive">*</span>
            </label>
            <select
              value={toBatchId}
              onChange={(e) => setToBatchId(e.target.value)}
              required
              disabled={loading || loadingBatches}
              aria-label="Destination Batch"
              className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="">Select destination batch...</option>
              {availableBatches.map((b) => (
                <option key={b.id} value={b.id}>
                  {b.name} ({b.code}) — {b.program_name} [{b.shift}] — ({b.enrolled_count}
                  /{b.capacity} seats)
                </option>
              ))}
            </select>
          </div>

          {/* Target Batch Info */}
          {targetBatch && (
            <div className="p-2.5 rounded-[var(--radius-md)] bg-primary-subtle/50 border border-primary/20 text-xs flex justify-between items-center">
              <div>
                <div className="font-semibold text-foreground">
                  {targetBatch.name} ({targetBatch.shift})
                </div>
                <div className="text-[11px] text-muted-foreground">
                  Teacher: {targetBatch.teacher_name || "Unassigned"} • Room:{" "}
                  {targetBatch.room_number || "TBD"}
                </div>
              </div>
              <Badge
                variant={isTargetFull ? "overdue" : "secondary"}
                className="font-mono text-[10px]"
              >
                {targetBatch.enrolled_count} / {targetBatch.capacity} Seats
              </Badge>
            </div>
          )}

          {/* Overcapacity Warning */}
          {isTargetFull && (
            <div className="p-3 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 space-y-1.5 text-xs">
              <div className="flex items-center gap-1.5 font-bold">
                <AlertTriangle className="h-4 w-4 text-amber-500" />
                Destination Batch Full
              </div>
              <p className="text-[11px]">
                Target batch has reached maximum capacity ({targetBatch?.capacity}{" "}
                students).
              </p>
              <label className="flex items-center gap-2 pt-1 font-medium cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowOvercapacity}
                  onChange={(e) => setAllowOvercapacity(e.target.checked)}
                  className="rounded border-amber-500 text-primary focus:ring-primary h-4 w-4"
                />
                <span>Allow Overcapacity Transfer</span>
              </label>
            </div>
          )}

          {/* New Roll & Transfer Date */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                New Roll in Target Batch
              </label>
              <Input
                type="number"
                min="1"
                placeholder="Auto-assigned"
                value={newRoll}
                onChange={(e) => setNewRoll(e.target.value)}
                disabled={loading}
              />
              <span className="text-[10px] text-muted-foreground block">
                Leave blank to auto-increment
              </span>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Effective Transfer Date
              </label>
              <Input
                type="date"
                value={transferDate}
                onChange={(e) => setTransferDate(e.target.value)}
                required
                disabled={loading}
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Transfer Reason / Notes
            </label>
            <Input
              placeholder="e.g. Schedule clash with college / morning shift preference"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              disabled={loading}
            />
          </div>

          <DialogFooter className="pt-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading || !toBatchId} className="gap-1.5">
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Transferring...
                </>
              ) : (
                "Execute Transfer"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
