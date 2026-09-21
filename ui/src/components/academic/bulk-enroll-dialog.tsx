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
import { Users, Search, Check, X, AlertTriangle, Loader2 } from "lucide-react";
import { formatPhone } from "@/lib/utils";

export interface BulkEnrollDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  batch: BatchDetail | null;
  roster: BatchRosterItem[];
  onBulkEnrolled: (enrollments: EnrollmentDetail[]) => void;
}

export function BulkEnrollDialog({
  open,
  onOpenChange,
  batch,
  roster,
  onBulkEnrolled,
}: BulkEnrollDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<StudentSearchHit[]>([]);
  const [searching, setSearching] = useState(false);

  // Selected students list
  const [selectedStudents, setSelectedStudents] = useState<StudentSearchHit[]>([]);

  const [startDate, setStartDate] = useState(
    () => new Date().toISOString().split("T")[0]
  );
  const [allowOvercapacity, setAllowOvercapacity] = useState(false);

  // Calculate sequential rolls preview
  const maxRoll = roster.reduce(
    (max, r) => (r.status === "active" && r.roll ? Math.max(max, r.roll) : max),
    0
  );

  const currentEnrolled = batch?.enrolled_count || 0;
  const capacity = batch?.capacity || 40;
  const willExceedCapacity = currentEnrolled + selectedStudents.length > capacity;

  useEffect(() => {
    if (!open) {
      setSearchQuery("");
      setSearchResults([]);
      setSelectedStudents([]);
      setAllowOvercapacity(false);
      setError(null);
    }
  }, [open]);

  useEffect(() => {
    const q = searchQuery.trim();
    if (q.length < 2) {
      setSearchResults([]);
      return;
    }

    const timer = setTimeout(async () => {
      setSearching(true);
      try {
        const res = await rpc("students.search", { query: q, limit: 15 });
        if (res.ok && res.data) {
          const activeIds = new Set(
            roster.filter((r) => r.status === "active").map((r) => r.student_id)
          );
          const alreadySelectedIds = new Set(selectedStudents.map((s) => s.id));
          setSearchResults(
            (res.data.matches || []).filter(
              (m) => !activeIds.has(m.id) && !alreadySelectedIds.has(m.id)
            )
          );
        }
      } catch {
        // ignore
      } finally {
        setSearching(false);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [searchQuery, roster, selectedStudents]);

  const addStudent = (student: StudentSearchHit) => {
    setSelectedStudents((prev) => [...prev, student]);
    setSearchResults((prev) => prev.filter((s) => s.id !== student.id));
  };

  const removeStudent = (id: string) => {
    setSelectedStudents((prev) => prev.filter((s) => s.id !== id));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!batch) return;
    if (selectedStudents.length === 0) {
      setError("Please select at least one student");
      return;
    }

    if (willExceedCapacity && !allowOvercapacity) {
      setError(
        "Enrollment count exceeds batch capacity limit. Please enable the overcapacity override."
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await rpc("enrollments.bulk_create", {
        bulk: {
          batch_id: batch.id,
          student_ids: selectedStudents.map((s) => s.id),
          start_date: startDate,
        },
        allow_overcapacity: allowOvercapacity,
      });

      if (res.ok && res.data) {
        onOpenChange(false);
        onBulkEnrolled(res.data);
      } else {
        setError(res.error?.message || "Failed to bulk enroll");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <Users className="h-5 w-5 text-primary" />
            Bulk Enroll Students
          </DialogTitle>
        </DialogHeader>

        {batch && (
          <div className="p-3 rounded-[var(--radius-md)] bg-muted/40 border border-border/80 flex items-center justify-between text-xs">
            <div>
              <span className="text-muted-foreground">Target Batch:</span>
              <span className="font-semibold text-foreground ml-1.5">{batch.name}</span>
              <span className="font-mono text-primary font-bold ml-1">
                ({batch.code})
              </span>
            </div>
            <span className="font-mono text-[11px] text-muted-foreground">
              Current: {currentEnrolled} / {capacity} seats
            </span>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Search to add */}
          <div className="space-y-2">
            <label className="text-xs font-semibold text-foreground">
              Search & Add Students
            </label>
            <div className="relative">
              <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search by student name, code, or phone..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9 text-xs"
              />
              {searching && (
                <Loader2 className="absolute right-3 top-2.5 h-4 w-4 animate-spin text-muted-foreground" />
              )}
            </div>

            {searchResults.length > 0 && (
              <div className="max-h-40 overflow-y-auto rounded-[var(--radius-md)] border border-border bg-card divide-y divide-border/60 shadow-sm">
                {searchResults.map((hit) => (
                  <button
                    key={hit.id}
                    type="button"
                    onClick={() => addStudent(hit)}
                    className="w-full text-left p-2 hover:bg-muted/70 flex items-center justify-between text-xs transition-colors"
                  >
                    <div>
                      <span className="font-semibold text-foreground mr-2">
                        {hit.name}
                      </span>
                      <span className="font-mono text-muted-foreground text-[11px]">
                        {hit.student_code} • {formatPhone(hit.phone_norm)}
                      </span>
                    </div>
                    <span className="text-xs font-medium text-primary flex items-center gap-1">
                      Add <Check className="h-3.5 w-3.5" />
                    </span>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Selected Students List with Roll Preview */}
          <div className="space-y-2">
            <div className="flex items-center justify-between text-xs">
              <span className="font-semibold text-foreground">
                Selected for Enrollment ({selectedStudents.length})
              </span>
              {selectedStudents.length > 0 && (
                <button
                  type="button"
                  onClick={() => setSelectedStudents([])}
                  className="text-[11px] text-destructive hover:underline"
                >
                  Clear all
                </button>
              )}
            </div>

            {selectedStudents.length === 0 ? (
              <div className="p-4 border border-dashed rounded-[var(--radius-md)] text-center text-xs text-muted-foreground">
                No students selected. Use the search box above to add students.
              </div>
            ) : (
              <div className="max-h-48 overflow-y-auto rounded-[var(--radius-md)] border border-border divide-y divide-border/60">
                {selectedStudents.map((s, idx) => {
                  const assignedRoll = maxRoll + idx + 1;
                  return (
                    <div
                      key={s.id}
                      className="p-2.5 flex items-center justify-between bg-card text-xs"
                    >
                      <div className="flex items-center gap-2.5">
                        <span className="font-mono px-2 py-0.5 rounded bg-primary/10 text-primary font-bold text-[11px]">
                          Roll #{assignedRoll}
                        </span>
                        <div>
                          <div className="font-semibold text-foreground">{s.name}</div>
                          <div className="text-[10px] text-muted-foreground font-mono">
                            {s.student_code}
                          </div>
                        </div>
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => removeStudent(s.id)}
                        className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                      >
                        <X className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* Start Date */}
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

          {/* Overcapacity Warning */}
          {willExceedCapacity && (
            <div className="p-3 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 space-y-1.5 text-xs">
              <div className="flex items-center gap-1.5 font-bold">
                <AlertTriangle className="h-4 w-4 text-amber-500" />
                Overcapacity Limit Warning
              </div>
              <p className="text-[11px]">
                Enrolling {selectedStudents.length} new students will bring total
                occupancy to {currentEnrolled + selectedStudents.length} (capacity is{" "}
                {capacity}).
              </p>
              <label className="flex items-center gap-2 pt-1 font-medium cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowOvercapacity}
                  onChange={(e) => setAllowOvercapacity(e.target.checked)}
                  className="rounded border-amber-500 text-primary focus:ring-primary h-4 w-4"
                />
                <span>Allow Overcapacity Bulk Enrollment</span>
              </label>
            </div>
          )}

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
              disabled={loading || selectedStudents.length === 0}
              className="gap-1.5"
            >
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Enrolling {selectedStudents.length}...
                </>
              ) : (
                `Enroll ${selectedStudents.length} Students`
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
