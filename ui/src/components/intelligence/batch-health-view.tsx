import { useState, useEffect, useCallback } from "react";
import { rpc, type BatchHealthItem, type ProgramSummary } from "@/gen/rpc";
import { formatTaka } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Activity,
  ShieldCheck,
  AlertTriangle,
  AlertOctagon,
  Users,
  CalendarCheck,
  CreditCard,
  RefreshCw,
  Loader2,
  Tag,
} from "lucide-react";

export interface BatchHealthViewProps {
  onNavigateBatch?: (batchId: string) => void;
  onOpenAttendance?: (batchId: string) => void;
  onNavigateTab?: (tabId: string) => void;
}

export function BatchHealthView({
  onNavigateBatch,
  onOpenAttendance,
  onNavigateTab,
}: BatchHealthViewProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [programs, setPrograms] = useState<ProgramSummary[]>([]);
  const [selectedProgramId, setSelectedProgramId] = useState<string>("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [batches, setBatches] = useState<BatchHealthItem[]>([]);

  // Load programs for dropdown filter
  useEffect(() => {
    void (async () => {
      try {
        const res = await rpc("programs.list", {});
        if (res.ok && res.data) {
          setPrograms(res.data);
        }
      } catch (err: unknown) {
        console.error("Failed to load programs list", err);
      }
    })();
  }, []);

  const loadBatchHealth = useCallback(async (progId: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("intelligence.get_batch_health_list", {
        program_id: progId || null,
        session: null,
      });

      if (res.ok && res.data) {
        setBatches(res.data);
      } else {
        setError(res.error?.message || "Failed to load batch health list");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadBatchHealth(selectedProgramId);
  }, [selectedProgramId, loadBatchHealth]);

  const filteredBatches = batches.filter((b) => {
    if (statusFilter === "all") return true;
    return b.health_status.toLowerCase() === statusFilter.toLowerCase();
  });

  const healthyCount = batches.filter(
    (b) => b.health_status.toLowerCase() === "healthy"
  ).length;
  const warningCount = batches.filter(
    (b) => b.health_status.toLowerCase() === "warning"
  ).length;
  const criticalCount = batches.filter(
    (b) => b.health_status.toLowerCase() === "critical"
  ).length;

  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case "healthy":
        return (
          <Badge variant="paid" size="sm" className="gap-1 font-bold">
            <ShieldCheck className="h-3 w-3" />
            Healthy
          </Badge>
        );
      case "warning":
        return (
          <Badge variant="pending" size="sm" className="gap-1 font-bold">
            <AlertTriangle className="h-3 w-3" />
            Warning
          </Badge>
        );
      case "critical":
        return (
          <Badge variant="overdue" size="sm" className="gap-1 font-bold">
            <AlertOctagon className="h-3 w-3" />
            Critical
          </Badge>
        );
      default:
        return (
          <Badge variant="outline" size="sm">
            {status}
          </Badge>
        );
    }
  };

  return (
    <div className="space-y-6">
      {/* Header & Filter Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            Batch Health & Operational Risk Matrix
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Holistic diagnostic evaluation: capacity utilization, 30-day attendance
            vigilance, overdue dues ratio, and dropout risks.
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <select
            aria-label="Filter by Academic Program"
            value={selectedProgramId}
            onChange={(e) => setSelectedProgramId(e.target.value)}
            className="h-8 px-3 py-1 text-xs rounded-[var(--radius-md)] bg-card border border-border text-foreground font-medium focus:outline-hidden cursor-pointer"
          >
            <option value="">All Programs ({programs.length})</option>
            {programs.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadBatchHealth(selectedProgramId)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh batch health data"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {/* Summary KPI Cards Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card
          onClick={() => setStatusFilter("all")}
          className={`border-border/80 bg-card shadow-2xs hover:shadow-xs cursor-pointer transition-all ${
            statusFilter === "all" ? "ring-2 ring-primary" : ""
          }`}
        >
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Monitored Batches
              </span>
              <div className="text-2xl font-bold tracking-tight text-foreground">
                {batches.length}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Active academic cohorts
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-primary-subtle text-primary flex items-center justify-center">
              <Activity className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card
          onClick={() => setStatusFilter("healthy")}
          className={`border-border/80 bg-card shadow-2xs hover:shadow-xs cursor-pointer transition-all ${
            statusFilter === "healthy" ? "ring-2 ring-emerald-500" : ""
          }`}
        >
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-emerald-700 dark:text-emerald-400">
                Healthy Batches
              </span>
              <div className="text-2xl font-bold tracking-tight text-emerald-600 dark:text-emerald-400">
                {healthyCount}
              </div>
              <span className="text-[11px] text-muted-foreground">
                High attendance &amp; on-track fees
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-emerald-500/10 text-emerald-600 flex items-center justify-center">
              <ShieldCheck className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card
          onClick={() => setStatusFilter("warning")}
          className={`border-border/80 bg-card shadow-2xs hover:shadow-xs cursor-pointer transition-all ${
            statusFilter === "warning" ? "ring-2 ring-amber-500" : ""
          }`}
        >
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-amber-700 dark:text-amber-400">
                Attention Required
              </span>
              <div className="text-2xl font-bold tracking-tight text-amber-600 dark:text-amber-400">
                {warningCount}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Moderate dues or dip in attendance
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-amber-500/10 text-amber-600 flex items-center justify-center">
              <AlertTriangle className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card
          onClick={() => setStatusFilter("critical")}
          className={`border-border/80 bg-card shadow-2xs hover:shadow-xs cursor-pointer transition-all ${
            statusFilter === "critical" ? "ring-2 ring-destructive" : ""
          }`}
        >
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-destructive">
                Critical Cohorts
              </span>
              <div className="text-2xl font-bold tracking-tight text-destructive">
                {criticalCount}
              </div>
              <span className="text-[11px] text-muted-foreground">
                High dues ratio &gt;30% or attendance &lt;60%
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-destructive/10 text-destructive flex items-center justify-center">
              <AlertOctagon className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>
      </div>

      {error && (
        <div className="p-4 rounded-[var(--radius-lg)] bg-destructive/10 border border-destructive/20 text-destructive text-xs">
          {error}
        </div>
      )}

      {/* Loading state */}
      {loading ? (
        <div className="py-20 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Computing multi-dimensional batch diagnostics...
          </span>
        </div>
      ) : filteredBatches.length === 0 ? (
        <Card className="border-dashed border-2 border-border">
          <CardContent className="py-16 text-center space-y-3">
            <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center mx-auto text-muted-foreground">
              <Activity className="h-6 w-6" />
            </div>
            <h3 className="text-sm font-bold text-foreground">
              No batches matching the current filter
            </h3>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              {statusFilter !== "all"
                ? `There are currently 0 batches marked as "${statusFilter}".`
                : "No batches registered in this program."}
            </p>
          </CardContent>
        </Card>
      ) : (
        /* Batches Health Cards Grid */
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {filteredBatches.map((batch) => (
            <Card
              key={batch.batch_id}
              className={`border-border/80 bg-card shadow-2xs hover:shadow-xs transition-all ${
                batch.health_status.toLowerCase() === "critical"
                  ? "border-destructive/40 bg-destructive/5"
                  : batch.health_status.toLowerCase() === "warning"
                    ? "border-amber-500/30 bg-amber-500/5"
                    : ""
              }`}
            >
              <CardHeader className="p-4 pb-3 border-b border-border/60 flex flex-row items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <CardTitle className="text-base font-bold text-foreground">
                      {batch.batch_name}
                    </CardTitle>
                    {getStatusBadge(batch.health_status)}
                  </div>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
                    <span>{batch.program_name}</span>
                    <span>•</span>
                    <span>Teacher: {batch.teacher_name || "Unassigned"}</span>
                  </div>
                </div>

                <div className="flex items-center gap-1.5">
                  {batch.total_due_poisha > 0 && onNavigateTab && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onNavigateTab("payments")}
                      className="h-7 text-xs font-semibold text-destructive hover:bg-destructive/10"
                    >
                      Dues
                    </Button>
                  )}
                  {onOpenAttendance && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onOpenAttendance(batch.batch_id)}
                      className="h-7 text-xs font-semibold"
                    >
                      Roll Call
                    </Button>
                  )}
                  {onNavigateBatch && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => onNavigateBatch(batch.batch_id)}
                      className="h-7 text-xs"
                    >
                      Details
                    </Button>
                  )}
                </div>
              </CardHeader>

              <CardContent className="p-4 space-y-4">
                {/* 3 Major Health Bar Gauges */}
                <div className="space-y-3 text-xs">
                  {/* Capacity Utilization */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-muted-foreground flex items-center gap-1">
                        <Users className="h-3 w-3" />
                        Capacity Utilization
                      </span>
                      <span className="font-semibold text-foreground font-mono">
                        {batch.enrolled_count} / {batch.capacity} (
                        {batch.capacity_utilization_pct}%)
                      </span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-muted overflow-hidden">
                      <div
                        className={`h-full rounded-full ${
                          batch.capacity_utilization_pct > 100
                            ? "bg-purple-600"
                            : batch.capacity_utilization_pct < 40
                              ? "bg-amber-500"
                              : "bg-primary"
                        }`}
                        style={{
                          width: `${Math.min(batch.capacity_utilization_pct, 100)}%`,
                        }}
                      />
                    </div>
                  </div>

                  {/* 30-Day Attendance Rate */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-muted-foreground flex items-center gap-1">
                        <CalendarCheck className="h-3 w-3" />
                        30-Day Attendance Rate
                      </span>
                      <span
                        className={`font-semibold font-mono ${
                          batch.thirty_day_attendance_pct < 60
                            ? "text-destructive font-bold"
                            : batch.thirty_day_attendance_pct < 75
                              ? "text-amber-600 dark:text-amber-400"
                              : "text-emerald-600 dark:text-emerald-400"
                        }`}
                      >
                        {batch.thirty_day_attendance_pct}%
                      </span>
                    </div>
                    <div className="w-full h-2 rounded-full bg-muted overflow-hidden">
                      <div
                        className={`h-full rounded-full ${
                          batch.thirty_day_attendance_pct < 60
                            ? "bg-destructive"
                            : batch.thirty_day_attendance_pct < 75
                              ? "bg-amber-500"
                              : "bg-emerald-500"
                        }`}
                        style={{
                          width: `${Math.min(batch.thirty_day_attendance_pct, 100)}%`,
                        }}
                      />
                    </div>
                  </div>

                  {/* Dues Ratio */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <span className="text-muted-foreground flex items-center gap-1">
                        <CreditCard className="h-3 w-3" />
                        Unpaid Dues Ratio
                      </span>
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-muted-foreground">
                          {formatTaka(batch.total_due_poisha)} due
                        </span>
                        <span
                          className={`font-semibold font-mono ${
                            batch.dues_ratio_pct > 30
                              ? "text-destructive font-bold"
                              : "text-foreground"
                          }`}
                        >
                          ({batch.dues_ratio_pct}%)
                        </span>
                      </div>
                    </div>
                    <div className="w-full h-2 rounded-full bg-muted overflow-hidden">
                      <div
                        className={`h-full rounded-full ${
                          batch.dues_ratio_pct > 30 ? "bg-destructive" : "bg-emerald-500"
                        }`}
                        style={{
                          width: `${Math.min(batch.dues_ratio_pct, 100)}%`,
                        }}
                      />
                    </div>
                  </div>
                </div>

                {/* Sub Metrics: GPA & Inactive Students */}
                <div className="grid grid-cols-2 gap-2 pt-2 border-t border-border/50 text-xs">
                  <div className="p-2 rounded bg-muted/30 border border-border/40">
                    <span className="text-[11px] text-muted-foreground block">
                      Inactive Students
                    </span>
                    <span
                      className={`font-bold font-mono ${
                        batch.inactive_students_count > 0
                          ? "text-destructive"
                          : "text-foreground"
                      }`}
                    >
                      {batch.inactive_students_count}
                    </span>
                  </div>
                  <div className="p-2 rounded bg-muted/30 border border-border/40">
                    <span className="text-[11px] text-muted-foreground block">
                      Avg Exam GPA
                    </span>
                    <span className="font-bold font-mono text-foreground">
                      {batch.average_exam_gpa !== null &&
                      batch.average_exam_gpa !== undefined
                        ? batch.average_exam_gpa.toFixed(2)
                        : "No exams yet"}
                    </span>
                  </div>
                </div>

                {/* Diagnostic Flags */}
                {batch.flags.length > 0 && (
                  <div className="flex items-center gap-1.5 flex-wrap pt-1">
                    <Tag className="h-3 w-3 text-muted-foreground shrink-0" />
                    {batch.flags.map((flag, fIdx) => (
                      <span
                        key={fIdx}
                        className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-muted text-muted-foreground border border-border"
                      >
                        {flag}
                      </span>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
