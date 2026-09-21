import { useState, useEffect } from "react";
import {
  rpc,
  type BatchDetail,
  type BatchRosterItem,
  type StudentSearchHit,
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
import { UserPlus, Search, AlertTriangle, Loader2, Check } from "lucide-react";
import { formatPhone } from "@/lib/utils";

export interface EnrollStudentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  batch: BatchDetail | null;
  roster: BatchRosterItem[];
  onEnrolled: (enrollment: EnrollmentDetail) => void;
}

export function EnrollStudentDialog({
  open,
  onOpenChange,
  batch,
  roster,
  onEnrolled,
}: EnrollStudentDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Student search
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<StudentSearchHit[]>([]);
  const [searching, setSearching] = useState(false);
  const [selectedStudent, setSelectedStudent] = useState<StudentSearchHit | null>(null);

  // Enrollment fields
  const [manualRoll, setManualRoll] = useState("");
  const [startDate, setStartDate] = useState(
    () => new Date().toISOString().split("T")[0]
  );
  const [notes, setNotes] = useState("");
  const [allowOvercapacity, setAllowOvercapacity] = useState(false);

  // Calculate next auto-roll
  const maxRoll = roster.reduce(
    (max, r) => (r.status === "active" && r.roll ? Math.max(max, r.roll) : max),
    0
  );
  const nextRoll = maxRoll + 1;

  // Capacity check
  const isFull = batch ? batch.enrolled_count >= batch.capacity : false;

  useEffect(() => {
    if (!open) {
      setSearchQuery("");
      setSearchResults([]);
      setSelectedStudent(null);
      setManualRoll("");
      setNotes("");
      setAllowOvercapacity(false);
      setError(null);
    }
  }, [open]);

  // Search students
  useEffect(() => {
    const q = searchQuery.trim();
    if (q.length < 2) {
      setSearchResults([]);
      return;
    }

    const timer = setTimeout(async () => {
      setSearching(true);
      try {
        const res = await rpc("students.search", { query: q, limit: 10 });
        if (res.ok && res.data) {
          // Filter out students already active in this batch
          const activeStudentIds = new Set(
            roster.filter((r) => r.status === "active").map((r) => r.student_id)
          );
          setSearchResults(
            (res.data.matches || []).filter((m) => !activeStudentIds.has(m.id))
          );
        }
      } catch {
        // ignore
      } finally {
        setSearching(false);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [searchQuery, roster]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!batch) return;
    if (!selectedStudent) {
      setError("Please select a student to enroll");
      return;
    }

    if (isFull && !allowOvercapacity) {
      setError(
        "Batch is currently at full capacity. Please check the overcapacity override box to proceed."
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const rollNum = manualRoll ? parseInt(manualRoll, 10) : null;
      const res = await rpc("enrollments.create", {
        enrollment: {
          student_id: selectedStudent.id,
          batch_id: batch.id,
          roll: rollNum,
          start_date: startDate,
          notes: notes.trim() || null,
        },
        allow_overcapacity: allowOvercapacity,
      });

      if (res.ok && res.data) {
        onOpenChange(false);
        onEnrolled(res.data);
      } else {
        setError(res.error?.message || "Failed to enroll student");
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
            <UserPlus className="h-5 w-5 text-primary" />
            Enroll Student in Batch
          </DialogTitle>
        </DialogHeader>

        {batch && (
          <div className="p-3 rounded-[var(--radius-md)] bg-muted/50 border border-border/80 flex items-center justify-between text-xs">
            <div>
              <span className="text-muted-foreground">Target Batch:</span>
              <span className="font-semibold text-foreground ml-1.5">{batch.name}</span>
              <span className="font-mono text-primary font-bold ml-1">
                ({batch.code})
              </span>
            </div>
            <Badge
              variant={isFull ? "overdue" : "secondary"}
              className="font-mono text-[10px]"
            >
              {batch.enrolled_count} / {batch.capacity} Enrolled
            </Badge>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Student Search & Select */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Select Student <span className="text-destructive">*</span>
            </label>

            {selectedStudent ? (
              <div className="flex items-center justify-between p-2.5 rounded-[var(--radius-md)] bg-primary-subtle border border-primary/30 text-xs">
                <div className="flex items-center gap-2.5">
                  <div className="h-8 w-8 rounded-full bg-primary/20 text-primary font-bold flex items-center justify-center text-xs">
                    {selectedStudent.name.slice(0, 2).toUpperCase()}
                  </div>
                  <div>
                    <div className="font-semibold text-foreground">
                      {selectedStudent.name}
                    </div>
                    <div className="text-[11px] text-muted-foreground font-mono">
                      {selectedStudent.student_code} •{" "}
                      {formatPhone(selectedStudent.phone_norm)}
                    </div>
                  </div>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedStudent(null)}
                  className="h-7 text-xs text-muted-foreground hover:text-foreground"
                >
                  Change
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                <div className="relative">
                  <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search by name, student code, or phone..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-9 text-xs"
                    autoFocus
                  />
                  {searching && (
                    <Loader2 className="absolute right-3 top-2.5 h-4 w-4 animate-spin text-muted-foreground" />
                  )}
                </div>

                {searchResults.length > 0 && (
                  <div className="max-h-48 overflow-y-auto rounded-[var(--radius-md)] border border-border bg-card divide-y divide-border/60 shadow-md">
                    {searchResults.map((hit) => (
                      <button
                        key={hit.id}
                        type="button"
                        onClick={() => setSelectedStudent(hit)}
                        className="w-full text-left p-2.5 hover:bg-muted/70 flex items-center justify-between text-xs transition-colors"
                      >
                        <div>
                          <div className="font-semibold text-foreground">
                            {hit.name}{" "}
                            {hit.name_bn && (
                              <span className="text-muted-foreground font-normal">
                                ({hit.name_bn})
                              </span>
                            )}
                          </div>
                          <div className="text-[11px] text-muted-foreground font-mono">
                            {hit.student_code} • {formatPhone(hit.phone_norm)}
                          </div>
                        </div>
                        <Check className="h-4 w-4 text-muted-foreground/30 hover:text-primary" />
                      </button>
                    ))}
                  </div>
                )}
                {searchQuery.length >= 2 && !searching && searchResults.length === 0 && (
                  <div className="p-3 text-center text-xs text-muted-foreground border border-dashed rounded-[var(--radius)]">
                    No matching active students found
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Roll Number & Date */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Batch Roll Number
              </label>
              <Input
                type="number"
                min="1"
                placeholder={`Auto (#${nextRoll})`}
                value={manualRoll}
                onChange={(e) => setManualRoll(e.target.value)}
                disabled={loading}
              />
              <span className="text-[10px] text-muted-foreground block">
                Leave empty for sequential #{nextRoll}
              </span>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Admission / Start Date
              </label>
              <Input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                required
                disabled={loading}
              />
            </div>
          </div>

          {/* Overcapacity Warning */}
          {isFull && (
            <div className="p-3 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 space-y-1.5 text-xs">
              <div className="flex items-center gap-1.5 font-bold">
                <AlertTriangle className="h-4 w-4 text-amber-500" />
                Batch Capacity Reached
              </div>
              <p className="text-[11px]">
                This section currently has {batch?.enrolled_count} active enrollments out
                of {batch?.capacity} capacity limit.
              </p>
              <label className="flex items-center gap-2 pt-1 font-medium cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowOvercapacity}
                  onChange={(e) => setAllowOvercapacity(e.target.checked)}
                  className="rounded border-amber-500 text-primary focus:ring-primary h-4 w-4"
                />
                <span>Allow Overcapacity Enrollment</span>
              </label>
            </div>
          )}

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Remarks / Notes
            </label>
            <Input
              placeholder="e.g. Regular monthly admission"
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
              disabled={loading || !selectedStudent}
              className="gap-1.5"
            >
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Enrolling...
                </>
              ) : (
                "Confirm Enrollment"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
