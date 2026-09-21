import { useState } from "react";
import { rpc, type BatchRosterItem, type EnrollmentDetail } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { UserMinus, Loader2 } from "lucide-react";

export interface EndEnrollmentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rosterItem: BatchRosterItem | null;
  onEnded: (enrollment: EnrollmentDetail) => void;
}

export function EndEnrollmentDialog({
  open,
  onOpenChange,
  rosterItem,
  onEnded,
}: EndEnrollmentDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [endReason, setEndReason] = useState("completed");
  const [endDate, setEndDate] = useState(() => new Date().toISOString().split("T")[0]);
  const [notes, setNotes] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!rosterItem) return;

    setLoading(true);
    setError(null);

    try {
      const res = await rpc("enrollments.end", {
        enrollment: {
          enrollment_id: rosterItem.enrollment_id,
          end_reason: endReason,
          end_date: endDate,
          notes: notes.trim() || null,
        },
      });

      if (res.ok && res.data) {
        onOpenChange(false);
        onEnded(res.data);
      } else {
        setError(res.error?.message || "Failed to end enrollment");
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
            <UserMinus className="h-5 w-5 text-destructive" />
            End Batch Enrollment
          </DialogTitle>
        </DialogHeader>

        {rosterItem && (
          <div className="p-3 rounded-[var(--radius-md)] bg-muted/40 border border-border/80 text-xs space-y-1">
            <div className="flex justify-between font-semibold text-foreground">
              <span>{rosterItem.student_name}</span>
              <span className="font-mono text-muted-foreground">
                {rosterItem.student_code}
              </span>
            </div>
            <div className="text-muted-foreground text-[11px]">
              Roll: #{rosterItem.roll || "None"} • Enrolled on {rosterItem.start_date}
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              End Reason <span className="text-destructive">*</span>
            </label>
            <select
              value={endReason}
              onChange={(e) => setEndReason(e.target.value)}
              disabled={loading}
              aria-label="End Reason"
              className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="completed">Course Completed (কোর্স সমাপ্ত)</option>
              <option value="transferred">Transferred out (স্থানান্তরিত)</option>
              <option value="dropped">Dropped out (বাদ পড়েছে)</option>
              <option value="left">Left voluntarily (স্বেচ্ছায় অব্যাহতি)</option>
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Termination Date
            </label>
            <Input
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              required
              disabled={loading}
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Remarks / Feedback
            </label>
            <Input
              placeholder="e.g. Session ended or moved out of town"
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
            <Button
              type="submit"
              variant="destructive"
              disabled={loading}
              className="gap-1.5"
            >
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Updating...
                </>
              ) : (
                "End Enrollment"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
