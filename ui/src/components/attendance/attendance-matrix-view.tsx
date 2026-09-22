import { useState, useEffect, useCallback } from "react";
import { rpc, type BatchAttendanceMatrixResponse, type BatchSummary } from "@/gen/rpc";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Calendar, RotateCcw, AlertTriangle } from "lucide-react";
import { cn } from "@/lib/utils";

export interface AttendanceMatrixViewProps {
  initialBatchId?: string | null;
  onBatchChange?: (batchId: string) => void;
}

export function AttendanceMatrixView({
  initialBatchId,
  onBatchChange,
}: AttendanceMatrixViewProps = {}) {
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>(initialBatchId || "");

  // Current year and month in YYYY-MM
  const [selectedMonth, setSelectedMonth] = useState<string>(() => {
    const d = new Date();
    const yr = d.getFullYear();
    const mo = String(d.getMonth() + 1).padStart(2, "0");
    return `${yr}-${mo}`;
  });

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [matrixData, setMatrixData] = useState<BatchAttendanceMatrixResponse | null>(
    null
  );

  // Load batches
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

  // Load batch matrix
  const loadMatrix = useCallback(async () => {
    if (!selectedBatchId || !selectedMonth) return;
    setLoading(true);
    setError(null);

    try {
      const res = await rpc("attendance.get_batch_matrix", {
        batch_id: selectedBatchId,
        month: selectedMonth,
      });

      if (res.ok && res.data) {
        setMatrixData(res.data);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load attendance matrix");
    } finally {
      setLoading(false);
    }
  }, [selectedBatchId, selectedMonth]);

  useEffect(() => {
    void loadMatrix();
  }, [loadMatrix]);

  // Calculate batch overall average percentage
  const totalStudents = matrixData?.students.length ?? 0;
  const averagePercentage =
    totalStudents > 0 && matrixData
      ? Math.round(
          matrixData.students.reduce((acc, s) => acc + s.attendance_percentage, 0) /
            totalStudents
        )
      : 0;

  const dates = matrixData?.dates ?? [];

  return (
    <div className="space-y-6">
      {/* Controls */}
      <Card className="p-4 bg-card/60 backdrop-blur-xs border-border/70 shadow-xs">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <label
                htmlFor="matrix-batch-select"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Batch:
              </label>
              <select
                id="matrix-batch-select"
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
                htmlFor="matrix-month-input"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Month:
              </label>
              <Input
                id="matrix-month-input"
                type="month"
                value={selectedMonth}
                onChange={(e) => setSelectedMonth(e.target.value)}
                className="h-9 w-40 text-xs font-medium"
              />
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => void loadMatrix()}
              disabled={loading}
              className="h-9 text-xs gap-1.5"
            >
              <RotateCcw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
              Refresh
            </Button>
          </div>

          <div className="flex items-center gap-3 text-xs">
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-muted/60 border border-border">
              <span className="text-muted-foreground">Sessions Held:</span>
              <span className="font-bold text-foreground">{dates.length}</span>
            </div>
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-primary/10 border border-primary/20 text-primary">
              <span className="font-semibold">Batch Rate:</span>
              <span className="font-bold">{averagePercentage}%</span>
            </div>
          </div>
        </div>
      </Card>

      {/* Error Banner */}
      {error && (
        <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Matrix Grid */}
      <Card className="overflow-hidden border-border/80 shadow-xs">
        <CardHeader className="py-3 px-4 bg-muted/20 border-b border-border/70 flex flex-row items-center justify-between">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <Calendar className="h-4 w-4 text-primary" />
            Monthly Attendance Register ({selectedMonth})
          </CardTitle>
          <div className="text-xs text-muted-foreground">
            {totalStudents} students registered
          </div>
        </CardHeader>

        <div className="overflow-x-auto max-h-[580px]">
          <table className="w-full text-xs text-left border-collapse">
            <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-20">
              <tr>
                <th className="py-2.5 px-3 w-12 text-center sticky left-0 bg-card z-30 shadow-xs">
                  #
                </th>
                <th className="py-2.5 px-3 w-16 sticky left-12 bg-card z-30 shadow-xs">
                  Roll
                </th>
                <th className="py-2.5 px-3 min-w-44 sticky left-28 bg-card z-30 shadow-xs">
                  Student Name
                </th>
                {dates.map((dateStr) => {
                  const dayNum = dateStr.slice(8);
                  return (
                    <th
                      key={dateStr}
                      className="py-2.5 px-1 text-center w-8 border-l border-border/50 text-[10px]"
                      title={dateStr}
                    >
                      {dayNum}
                    </th>
                  );
                })}
                <th className="py-2.5 px-3 w-24 text-center border-l border-border/70">
                  Rate
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {matrixData?.students.map((student, idx) => {
                return (
                  <tr
                    key={student.student_id}
                    className="hover:bg-muted/30 transition-colors"
                  >
                    <td className="py-2 px-3 text-center text-muted-foreground font-mono sticky left-0 bg-card/95 z-10">
                      {idx + 1}
                    </td>
                    <td className="py-2 px-3 font-mono font-semibold text-foreground sticky left-12 bg-card/95 z-10">
                      {student.roll ?? "-"}
                    </td>
                    <td className="py-2 px-3 sticky left-28 bg-card/95 z-10">
                      <div className="font-semibold text-foreground">
                        {student.student_name}
                      </div>
                      <div className="text-[10px] text-muted-foreground font-mono">
                        {student.student_code}
                      </div>
                    </td>
                    {dates.map((dateStr) => {
                      const mark = student.marks[dateStr];
                      let cellContent = (
                        <span className="text-muted-foreground/40 font-mono text-[10px]">
                          ·
                        </span>
                      );

                      if (mark === "present") {
                        cellContent = (
                          <span
                            className="inline-flex items-center justify-center h-5 w-5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-700 dark:text-emerald-300"
                            title={`${student.student_name} on ${dateStr}: Present`}
                          >
                            P
                          </span>
                        );
                      } else if (mark === "absent") {
                        cellContent = (
                          <span
                            className="inline-flex items-center justify-center h-5 w-5 rounded text-[10px] font-bold bg-rose-500/20 text-rose-700 dark:text-rose-300"
                            title={`${student.student_name} on ${dateStr}: Absent`}
                          >
                            A
                          </span>
                        );
                      } else if (mark === "late") {
                        cellContent = (
                          <span
                            className="inline-flex items-center justify-center h-5 w-5 rounded text-[10px] font-bold bg-amber-500/20 text-amber-700 dark:text-amber-300"
                            title={`${student.student_name} on ${dateStr}: Late`}
                          >
                            L
                          </span>
                        );
                      } else if (mark === "excused") {
                        cellContent = (
                          <span
                            className="inline-flex items-center justify-center h-5 w-5 rounded text-[10px] font-bold bg-sky-500/20 text-sky-700 dark:text-sky-300"
                            title={`${student.student_name} on ${dateStr}: Excused`}
                          >
                            E
                          </span>
                        );
                      }

                      return (
                        <td
                          key={dateStr}
                          className="py-1 px-1 text-center border-l border-border/40"
                        >
                          {cellContent}
                        </td>
                      );
                    })}
                    <td className="py-2 px-3 text-center border-l border-border/70 font-semibold">
                      <span
                        className={cn(
                          "px-2 py-0.5 rounded text-[11px] font-bold",
                          student.attendance_percentage >= 80
                            ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300"
                            : student.attendance_percentage >= 60
                              ? "bg-amber-500/15 text-amber-700 dark:text-amber-300"
                              : "bg-rose-500/15 text-rose-700 dark:text-rose-300"
                        )}
                      >
                        {student.attendance_percentage}%
                      </span>
                    </td>
                  </tr>
                );
              })}

              {(!matrixData || matrixData.students.length === 0) && !loading && (
                <tr>
                  <td
                    colSpan={dates.length + 4}
                    className="py-12 text-center text-muted-foreground text-xs"
                  >
                    No students found or no attendance recorded for this batch in{" "}
                    {selectedMonth}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {/* Legend */}
        <div className="p-3 bg-muted/20 border-t border-border/70 flex flex-wrap items-center justify-between gap-3 text-xs">
          <div className="flex items-center gap-4 flex-wrap">
            <span className="font-semibold text-foreground">Status Legend:</span>
            <div className="flex items-center gap-1.5">
              <span className="h-4 w-4 rounded bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-[10px] font-bold flex items-center justify-center">
                P
              </span>
              <span className="text-muted-foreground">Present</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="h-4 w-4 rounded bg-rose-500/20 text-rose-700 dark:text-rose-300 text-[10px] font-bold flex items-center justify-center">
                A
              </span>
              <span className="text-muted-foreground">Absent</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="h-4 w-4 rounded bg-amber-500/20 text-amber-700 dark:text-amber-300 text-[10px] font-bold flex items-center justify-center">
                L
              </span>
              <span className="text-muted-foreground">Late</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="h-4 w-4 rounded bg-sky-500/20 text-sky-700 dark:text-sky-300 text-[10px] font-bold flex items-center justify-center">
                E
              </span>
              <span className="text-muted-foreground">Excused</span>
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-muted-foreground/60 font-mono">·</span>
              <span className="text-muted-foreground">No Session Held</span>
            </div>
          </div>
          <div className="text-[11px] text-muted-foreground">
            Calculated rate formula: (Present + Late) / Total Sessions * 100
          </div>
        </div>
      </Card>
    </div>
  );
}
