import { useState, useEffect, useCallback, useRef } from "react";
import {
  rpc,
  type AttendanceSessionInfo,
  type AttendanceMarkItem,
  type BatchSummary,
} from "@/gen/rpc";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { useAuth } from "@/contexts/auth-context";
import {
  CalendarCheck,
  Lock,
  Users,
  CheckCircle2,
  Save,
  AlertTriangle,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import { cn } from "@/lib/utils";

export interface AttendanceSheetProps {
  initialBatchId?: string | null;
  onBatchChange?: (batchId: string) => void;
}

export function AttendanceSheet({
  initialBatchId,
  onBatchChange,
}: AttendanceSheetProps = {}) {
  const { hasPermission } = useAuth();
  const canEditPrevious = hasPermission("attendance.edit_previous");

  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>(initialBatchId || "");
  const [selectedDate, setSelectedDate] = useState<string>(
    () => new Date().toISOString().split("T")[0]
  );

  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const [sessionInfo, setSessionInfo] = useState<AttendanceSessionInfo | null>(null);
  const [marks, setMarks] = useState<AttendanceMarkItem[]>([]);
  const [sessionNotes, setSessionNotes] = useState<string>("");
  const [focusedIndex, setFocusedIndex] = useState<number>(0);

  // Retroactive edit dialog state
  const [auditDialogOpen, setAuditDialogOpen] = useState(false);
  const [auditReason, setAuditReason] = useState("");

  const tableContainerRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<(HTMLTableRowElement | null)[]>([]);

  // Load active batches on mount
  useEffect(() => {
    let mounted = true;
    async function loadBatches() {
      try {
        const res = await rpc("batches.list", { status: "active" });
        if (mounted && res.ok && res.data) {
          setBatches(res.data);
          if (!selectedBatchId && res.data.length > 0) {
            const firstId = res.data[0].id;
            setSelectedBatchId(firstId);
            onBatchChange?.(firstId);
          }
        }
      } catch (err: unknown) {
        if (mounted) {
          setError(err instanceof Error ? err.message : "Failed to load batches");
        }
      }
    }
    void loadBatches();
    return () => {
      mounted = false;
    };
  }, [selectedBatchId, onBatchChange]);

  // Load attendance session whenever batch or date changes
  const loadSession = useCallback(async () => {
    if (!selectedBatchId || !selectedDate) return;
    setLoading(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const res = await rpc("attendance.get_session", {
        batch_id: selectedBatchId,
        date: selectedDate,
      });

      if (res.ok && res.data) {
        setSessionInfo(res.data);
        setMarks(res.data.marks);
        setSessionNotes(res.data.notes || "");
        setFocusedIndex(0);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load attendance session");
    } finally {
      setLoading(false);
    }
  }, [selectedBatchId, selectedDate]);

  useEffect(() => {
    void loadSession();
  }, [loadSession]);

  // Scroll focused row into view
  useEffect(() => {
    if (rowRefs.current[focusedIndex]) {
      rowRefs.current[focusedIndex]?.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  }, [focusedIndex]);

  // Update mark status for a student
  const updateMarkStatus = useCallback((index: number, newStatus: string) => {
    setMarks((prev) => {
      const updated = [...prev];
      if (updated[index]) {
        updated[index] = {
          ...updated[index],
          status: newStatus,
        };
      }
      return updated;
    });
  }, []);

  // Update mark note for a student
  const updateMarkNote = useCallback((index: number, note: string) => {
    setMarks((prev) => {
      const updated = [...prev];
      if (updated[index]) {
        updated[index] = {
          ...updated[index],
          note: note || null,
        };
      }
      return updated;
    });
  }, []);

  // Mark all students
  const handleMarkAll = useCallback((status: string) => {
    setMarks((prev) =>
      prev.map((m) => ({
        ...m,
        status,
      }))
    );
  }, []);

  // Save session submission
  const executeSave = useCallback(
    async (reason?: string) => {
      if (!selectedBatchId || !selectedDate) return;
      setSaving(true);
      setError(null);
      setSuccessMessage(null);

      const startTime = performance.now();

      try {
        const payload = {
          batch_id: selectedBatchId,
          date: selectedDate,
          notes: sessionNotes.trim() ? sessionNotes.trim() : null,
          audit_reason: reason || null,
          marks: marks.map((m) => ({
            student_id: m.student_id,
            status: m.status,
            note: m.note || null,
          })),
        };

        const res = await rpc("attendance.save_session", payload);

        if (res.ok && res.data) {
          const elapsed = Math.round(performance.now() - startTime);
          setSessionInfo(res.data);
          setMarks(res.data.marks);
          setSuccessMessage(
            `Attendance saved in ${elapsed} ms for ${res.data.marks.length} students`
          );
          setAuditDialogOpen(false);
          setAuditReason("");
        } else if (res.error) {
          if (res.error.code === "attendance_locked") {
            setAuditDialogOpen(true);
          } else {
            setError(res.error.message);
          }
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : "Failed to save attendance");
      } finally {
        setSaving(false);
      }
    },
    [selectedBatchId, selectedDate, sessionNotes, marks]
  );

  const handleSaveClick = () => {
    if (sessionInfo?.locked) {
      setAuditDialogOpen(true);
    } else {
      void executeSave();
    }
  };

  // Lock session manually
  const handleLockSession = async () => {
    if (!sessionInfo?.id) return;
    try {
      const res = await rpc("attendance.lock_session", {
        session_id: sessionInfo.id,
      });
      if (res.ok) {
        void loadSession();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to lock attendance session");
    }
  };

  // High speed keyboard handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Avoid capturing when user is typing inside an input or textarea
      const target = e.target as HTMLElement;
      const isInput = target.tagName === "INPUT" || target.tagName === "TEXTAREA";

      if (isInput && target.getAttribute("data-attendance-note") === null) {
        return;
      }

      if (marks.length === 0) return;

      if (e.key === "ArrowDown" || (e.key === "j" && !isInput)) {
        e.preventDefault();
        setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
      } else if (e.key === "ArrowUp" || (e.key === "k" && !isInput)) {
        e.preventDefault();
        setFocusedIndex((prev) => Math.max(0, prev - 1));
      } else if (!isInput) {
        const key = e.key.toLowerCase();
        if (key === "p") {
          e.preventDefault();
          updateMarkStatus(focusedIndex, "present");
          setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
        } else if (key === "a") {
          e.preventDefault();
          updateMarkStatus(focusedIndex, "absent");
          setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
        } else if (key === "l") {
          e.preventDefault();
          updateMarkStatus(focusedIndex, "late");
          setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
        } else if (key === "e") {
          e.preventDefault();
          updateMarkStatus(focusedIndex, "excused");
          setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
        } else if (e.key === "Enter") {
          e.preventDefault();
          setFocusedIndex((prev) => Math.min(marks.length - 1, prev + 1));
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [marks.length, focusedIndex, updateMarkStatus]);

  // Compute counts
  const presentCount = marks.filter((m) => m.status === "present").length;
  const absentCount = marks.filter((m) => m.status === "absent").length;
  const lateCount = marks.filter((m) => m.status === "late").length;
  const excusedCount = marks.filter((m) => m.status === "excused").length;
  const totalCount = marks.length;
  const percentage =
    totalCount > 0 ? Math.round(((presentCount + lateCount) / totalCount) * 100) : 0;

  return (
    <div className="space-y-6">
      {/* Controls Bar */}
      <Card className="p-4 bg-card/60 backdrop-blur-xs border-border/70 shadow-xs">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <label
                htmlFor="batch-select"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Batch:
              </label>
              <select
                id="batch-select"
                value={selectedBatchId}
                onChange={(e) => {
                  setSelectedBatchId(e.target.value);
                  onBatchChange?.(e.target.value);
                }}
                className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
              >
                {batches.map((b) => (
                  <option key={b.id} value={b.id}>
                    {b.name} ({b.code})
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-2">
              <label
                htmlFor="attendance-date"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Date:
              </label>
              <Input
                id="attendance-date"
                type="date"
                value={selectedDate}
                onChange={(e) => setSelectedDate(e.target.value)}
                className="h-9 w-40 text-xs font-medium"
              />
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => void loadSession()}
              disabled={loading}
              className="h-9 text-xs gap-1.5"
            >
              <RotateCcw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
              Refresh
            </Button>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            {/* Session Status Badges */}
            {sessionInfo?.session_exists ? (
              <Badge variant="paid" className="gap-1 text-xs py-1">
                <CheckCircle2 className="h-3 w-3" />
                Recorded
              </Badge>
            ) : (
              <Badge variant="pending" className="gap-1 text-xs py-1">
                <AlertTriangle className="h-3 w-3" />
                Not Taken Yet
              </Badge>
            )}

            {sessionInfo?.locked ? (
              <Badge variant="overdue" className="gap-1 text-xs py-1">
                <Lock className="h-3 w-3" />
                Session Locked
              </Badge>
            ) : (
              sessionInfo?.session_exists && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => void handleLockSession()}
                  className="h-8 text-xs gap-1 text-muted-foreground hover:text-foreground"
                >
                  <Lock className="h-3 w-3" />
                  Lock Session
                </Button>
              )
            )}

            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveClick}
              disabled={saving || loading || marks.length === 0}
              className="h-9 gap-1.5 font-semibold text-xs bg-primary text-primary-foreground shadow-xs"
            >
              <Save className="h-3.5 w-3.5" />
              {saving ? "Saving..." : "Save Attendance"}
            </Button>
          </div>
        </div>
      </Card>

      {/* Summary Metrics & Fast Bulk Actions */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <div className="p-3 rounded-lg border border-border/80 bg-card/40 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-muted-foreground">
            Enrolled Students
          </span>
          <div className="text-xl font-bold text-foreground mt-1 flex items-center gap-1.5">
            <Users className="h-4 w-4 text-primary" />
            {totalCount}
          </div>
        </div>

        <div className="p-3 rounded-lg border border-emerald-500/20 bg-emerald-500/5 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-emerald-700 dark:text-emerald-400">
            Present
          </span>
          <div className="text-xl font-bold text-emerald-600 dark:text-emerald-300 mt-1 flex items-center justify-between">
            <span>{presentCount}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleMarkAll("present")}
              className="h-6 px-1.5 text-[10px] text-emerald-700 dark:text-emerald-400 hover:bg-emerald-500/20 font-semibold"
            >
              All P
            </Button>
          </div>
        </div>

        <div className="p-3 rounded-lg border border-rose-500/20 bg-rose-500/5 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-rose-700 dark:text-rose-400">
            Absent
          </span>
          <div className="text-xl font-bold text-rose-600 dark:text-rose-300 mt-1 flex items-center justify-between">
            <span>{absentCount}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleMarkAll("absent")}
              className="h-6 px-1.5 text-[10px] text-rose-700 dark:text-rose-400 hover:bg-rose-500/20 font-semibold"
            >
              All A
            </Button>
          </div>
        </div>

        <div className="p-3 rounded-lg border border-amber-500/20 bg-amber-500/5 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-amber-700 dark:text-amber-400">
            Late
          </span>
          <div className="text-xl font-bold text-amber-600 dark:text-amber-300 mt-1">
            {lateCount}
          </div>
        </div>

        <div className="p-3 rounded-lg border border-sky-500/20 bg-sky-500/5 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-sky-700 dark:text-sky-400">
            Excused
          </span>
          <div className="text-xl font-bold text-sky-600 dark:text-sky-300 mt-1">
            {excusedCount}
          </div>
        </div>

        <div className="p-3 rounded-lg border border-primary/20 bg-primary/5 flex flex-col justify-between">
          <span className="text-[11px] font-medium text-primary">Attendance Rate</span>
          <div className="text-xl font-bold text-primary mt-1 flex items-center gap-1">
            <Sparkles className="h-3.5 w-3.5" />
            {percentage}%
          </div>
        </div>
      </div>

      {/* Keyboard Guide Banner */}
      <div className="p-3 rounded-lg bg-muted/50 border border-border/70 flex flex-wrap items-center justify-between text-xs text-muted-foreground gap-2">
        <div className="flex items-center gap-4 flex-wrap">
          <span className="font-semibold text-foreground">Keyboard Controls:</span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              P
            </kbd>{" "}
            Present
          </span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              A
            </kbd>{" "}
            Absent
          </span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              L
            </kbd>{" "}
            Late
          </span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              E
            </kbd>{" "}
            Excused
          </span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              ↑
            </kbd>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              ↓
            </kbd>{" "}
            Navigate
          </span>
          <span>
            <kbd className="px-1.5 py-0.5 rounded bg-background border border-border font-mono text-[11px]">
              Enter
            </kbd>{" "}
            Next
          </span>
        </div>
        <div className="text-[11px] italic">
          High speed attendance takes 80 student marks in under 300 ms
        </div>
      </div>

      {/* Error & Success Messages */}
      {error && (
        <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}
      {successMessage && (
        <div className="p-3 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          <span>{successMessage}</span>
        </div>
      )}

      {/* Roster Table */}
      <Card className="overflow-hidden border-border/80 shadow-xs">
        <CardHeader className="py-3 px-4 bg-muted/20 border-b border-border/70 flex flex-row items-center justify-between">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <CalendarCheck className="h-4 w-4 text-primary" />
            Class Student Roster
          </CardTitle>
          <div className="text-xs text-muted-foreground">
            {marks.length} students enrolled on {selectedDate}
          </div>
        </CardHeader>

        <div ref={tableContainerRef} className="max-h-[520px] overflow-y-auto">
          <table className="w-full text-xs text-left border-collapse">
            <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
              <tr>
                <th className="py-2.5 px-3 w-12 text-center">#</th>
                <th className="py-2.5 px-3 w-20">Roll</th>
                <th className="py-2.5 px-3">Student Name</th>
                <th className="py-2.5 px-3 w-28">Student Code</th>
                <th className="py-2.5 px-3 w-64 text-center">Attendance Status</th>
                <th className="py-2.5 px-3">Session Note</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {marks.map((student, index) => {
                const isFocused = index === focusedIndex;
                const status = student.status;

                return (
                  <tr
                    key={student.student_id}
                    ref={(el) => {
                      rowRefs.current[index] = el;
                    }}
                    onClick={() => setFocusedIndex(index)}
                    className={cn(
                      "transition-colors cursor-pointer",
                      isFocused ? "bg-primary/10 font-medium" : "hover:bg-muted/40",
                      status === "absent" && "bg-rose-500/5",
                      status === "late" && "bg-amber-500/5"
                    )}
                  >
                    <td className="py-2 px-3 text-center text-muted-foreground font-mono">
                      {index + 1}
                    </td>
                    <td className="py-2 px-3 font-mono font-semibold text-foreground">
                      {student.roll ?? "-"}
                    </td>
                    <td className="py-2 px-3">
                      <div className="font-semibold text-foreground">
                        {student.student_name}
                      </div>
                    </td>
                    <td className="py-2 px-3 font-mono text-muted-foreground">
                      {student.student_code}
                    </td>
                    <td className="py-2 px-3">
                      <div className="flex items-center justify-center gap-1.5">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            updateMarkStatus(index, "present");
                            setFocusedIndex(index);
                          }}
                          className={cn(
                            "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1",
                            status === "present"
                              ? "bg-emerald-600 text-white shadow-xs"
                              : "bg-muted/80 text-muted-foreground hover:bg-emerald-500/20 hover:text-emerald-700"
                          )}
                        >
                          P
                        </button>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            updateMarkStatus(index, "absent");
                            setFocusedIndex(index);
                          }}
                          className={cn(
                            "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1",
                            status === "absent"
                              ? "bg-rose-600 text-white shadow-xs"
                              : "bg-muted/80 text-muted-foreground hover:bg-rose-500/20 hover:text-rose-700"
                          )}
                        >
                          A
                        </button>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            updateMarkStatus(index, "late");
                            setFocusedIndex(index);
                          }}
                          className={cn(
                            "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1",
                            status === "late"
                              ? "bg-amber-600 text-white shadow-xs"
                              : "bg-muted/80 text-muted-foreground hover:bg-amber-500/20 hover:text-amber-700"
                          )}
                        >
                          L
                        </button>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            updateMarkStatus(index, "excused");
                            setFocusedIndex(index);
                          }}
                          className={cn(
                            "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1",
                            status === "excused"
                              ? "bg-sky-600 text-white shadow-xs"
                              : "bg-muted/80 text-muted-foreground hover:bg-sky-500/20 hover:text-sky-700"
                          )}
                        >
                          E
                        </button>
                      </div>
                    </td>
                    <td className="py-2 px-3">
                      <input
                        type="text"
                        data-attendance-note="true"
                        placeholder="Note..."
                        value={student.note || ""}
                        onChange={(e) => updateMarkNote(index, e.target.value)}
                        className="h-7 w-full px-2 rounded bg-background/80 border border-input text-[11px] focus:outline-hidden focus:ring-1 focus:ring-ring"
                      />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {/* Footer with Session Notes */}
        <div className="p-4 bg-muted/10 border-t border-border/70 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex-1 max-w-xl">
            <label
              htmlFor="session-notes"
              className="text-xs font-semibold text-muted-foreground block mb-1"
            >
              Class Notes & Covered Curriculum:
            </label>
            <Input
              id="session-notes"
              value={sessionNotes}
              onChange={(e) => setSessionNotes(e.target.value)}
              placeholder="E.g., Physics Chapter 4 Newton's Laws and problem solving discussion"
              className="text-xs"
            />
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="primary"
              onClick={handleSaveClick}
              disabled={saving || loading || marks.length === 0}
              className="gap-1.5 font-semibold text-xs"
            >
              <Save className="h-3.5 w-3.5" />
              {saving ? "Saving..." : "Save Attendance"}
            </Button>
          </div>
        </div>
      </Card>

      {/* Retroactive Edit Dialog */}
      <Dialog open={auditDialogOpen} onOpenChange={setAuditDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base font-bold">
              <Lock className="h-4 w-4 text-rose-500" />
              Retroactive Attendance Edit Authorization
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              This attendance session was previously recorded or has exceeded the daily
              lock window. Modifying past attendance records requires authorized
              privileges and an explicit audit explanation.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2">
            <div className="p-3 rounded-md bg-amber-500/10 border border-amber-500/20 text-xs text-amber-900 dark:text-amber-200">
              An immutable SHA 256 entry will be committed to the institutional audit log
              capturing previous and updated student marks.
            </div>

            <div className="space-y-1.5">
              <label
                htmlFor="audit-reason"
                className="text-xs font-semibold text-foreground"
              >
                Mandatory Reason for Edit:
              </label>
              <textarea
                id="audit-reason"
                rows={3}
                value={auditReason}
                onChange={(e) => setAuditReason(e.target.value)}
                placeholder="E.g., Medical certificate verified by coordinator for Rahim Khan"
                className="w-full p-2.5 rounded-md border border-input bg-background text-xs focus:outline-hidden focus:ring-1 focus:ring-ring"
              />
            </div>

            {!canEditPrevious && (
              <div className="p-2.5 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs">
                Warning: Your current user account does not possess the
                attendance.edit_previous permission.
              </div>
            )}
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setAuditDialogOpen(false)}
              className="text-xs"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => void executeSave(auditReason)}
              disabled={saving || !auditReason.trim()}
              className="text-xs font-semibold"
            >
              {saving ? "Authorizing..." : "Confirm & Save Changes"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
