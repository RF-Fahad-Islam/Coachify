import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type StudentDueSummaryItem,
  type ProgramSummary,
  type BatchSummary,
  type DueStatementData,
} from "@/gen/rpc";
import { formatTaka } from "@/lib/utils";
import { executeHtmlPrint } from "@/lib/printer";
import { generateDueStatementHtml } from "../print/due-statement-template";
import { PrintPreviewDialog } from "../print/print-preview-dialog";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Search,
  Printer,
  CreditCard,
  Download,
  AlertCircle,
  Loader2,
  RefreshCw,
  Users,
  Clock,
  Sparkles,
} from "lucide-react";

export interface DuesCenterViewProps {
  onCollectPayment?: (studentId: string) => void;
  onOpenSmartDueQueue?: () => void;
}

export function DuesCenterView({
  onCollectPayment,
  onOpenSmartDueQueue,
}: DuesCenterViewProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dues, setDues] = useState<StudentDueSummaryItem[]>([]);

  // Filter options
  const [programs, setPrograms] = useState<ProgramSummary[]>([]);
  const [batches, setBatches] = useState<BatchSummary[]>([]);

  // Active filters
  const [search, setSearch] = useState("");
  const [selectedProgramId, setSelectedProgramId] = useState<string>("");
  const [selectedBatchId, setSelectedBatchId] = useState<string>("");
  const [overdueOnly, setOverdueOnly] = useState(false);

  // Print statement preview state
  const [previewOpen, setPreviewOpen] = useState(false);
  const [previewHtml, setPreviewHtml] = useState("");
  const [statementLoadingId, setStatementLoadingId] = useState<string | null>(null);

  // Load programs and batches
  useEffect(() => {
    void (async () => {
      try {
        const [progRes, batchRes] = await Promise.all([
          rpc("programs.list", {}),
          rpc("batches.list", {}),
        ]);
        if (progRes.ok && progRes.data) setPrograms(progRes.data);
        if (batchRes.ok && batchRes.data) setBatches(batchRes.data);
      } catch (err: unknown) {
        console.error("Failed to load academic programs/batches", err);
      }
    })();
  }, []);

  const loadDues = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("ledger.list_dues", {
        filter: {
          program_id: selectedProgramId || null,
          batch_id: selectedBatchId || null,
          search: search.trim() || null,
          overdue_only: overdueOnly ? true : null,
          period: null,
          min_due_poisha: null,
          limit: 100,
          offset: 0,
        },
      });

      if (res.ok && res.data) {
        setDues(res.data);
      } else {
        setError(res.error?.message || "Failed to load dues");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [selectedProgramId, selectedBatchId, search, overdueOnly]);

  useEffect(() => {
    const timer = setTimeout(() => {
      void loadDues();
    }, 200);
    return () => clearTimeout(timer);
  }, [loadDues]);

  // Totals calculations
  const totalDuesPoisha = dues.reduce((sum, d) => sum + d.total_due_poisha, 0);
  const totalOverduePoisha = dues.reduce((sum, d) => sum + d.overdue_poisha, 0);
  const dueStudentCount = dues.length;

  const handlePrintStatement = async (studentId: string) => {
    setStatementLoadingId(studentId);
    try {
      const res = await rpc("ledger.get_due_statement", {
        student_id: studentId,
        period_filter: null,
      });

      if (res.ok && res.data) {
        const html = generateDueStatementHtml(res.data as DueStatementData);
        setPreviewHtml(html);
        setPreviewOpen(true);
      } else {
        alert(res.error?.message || "Failed to generate due statement");
      }
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      setStatementLoadingId(null);
    }
  };

  const exportCsv = () => {
    if (dues.length === 0) return;
    const headers = [
      "Student ID",
      "Student Name",
      "Phone",
      "Batch",
      "Roll",
      "Current Month Due (BDT)",
      "Overdue (BDT)",
      "Total Due (BDT)",
    ];
    const rows = dues.map((d) => [
      d.student_code,
      `"${d.name.replace(/"/g, '""')}"`,
      d.phone,
      `"${(d.batch_name || "").replace(/"/g, '""')}"`,
      d.roll_no || "",
      (d.current_month_due_poisha / 100).toFixed(2),
      (d.overdue_poisha / 100).toFixed(2),
      (d.total_due_poisha / 100).toFixed(2),
    ]);

    const csvContent =
      "data:text/csv;charset=utf-8," +
      [headers.join(","), ...rows.map((r) => r.join(","))].join("\n");
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute(
      "download",
      `Dues_Directory_${new Date().toISOString().slice(0, 10)}.csv`
    );
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className="space-y-6">
      {/* Metric Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="border-l-4 border-l-destructive">
          <CardContent className="p-4 flex items-center justify-between">
            <div>
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Total Outstanding Dues
              </p>
              <h3 className="text-2xl font-extrabold text-foreground mt-1 tracking-tight">
                {formatTaka(totalDuesPoisha)}
              </h3>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Across all active coaching batches
              </p>
            </div>
            <div className="h-12 w-12 rounded-[var(--radius-md)] bg-destructive/10 text-destructive flex items-center justify-center">
              <CreditCard className="h-6 w-6" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-l-4 border-l-amber-500">
          <CardContent className="p-4 flex items-center justify-between">
            <div>
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Seriously Overdue (&gt;30 Days)
              </p>
              <h3 className="text-2xl font-extrabold text-foreground mt-1 tracking-tight text-amber-600 dark:text-amber-400">
                {formatTaka(totalOverduePoisha)}
              </h3>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Requires immediate follow-up
              </p>
            </div>
            <div className="h-12 w-12 rounded-[var(--radius-md)] bg-amber-500/10 text-amber-600 dark:text-amber-400 flex items-center justify-center">
              <Clock className="h-6 w-6" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-l-4 border-l-primary">
          <CardContent className="p-4 flex items-center justify-between">
            <div>
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                Due Students Count
              </p>
              <h3 className="text-2xl font-extrabold text-foreground mt-1 tracking-tight">
                {dueStudentCount} Students
              </h3>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Matching active search filters
              </p>
            </div>
            <div className="h-12 w-12 rounded-[var(--radius-md)] bg-primary/10 text-primary flex items-center justify-center">
              <Users className="h-6 w-6" />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filter and Action Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3 p-3 bg-card rounded-[var(--radius-lg)] border border-border">
        <div className="flex flex-wrap items-center gap-2.5 w-full sm:w-auto">
          {/* Search Box */}
          <div className="relative w-full sm:w-64">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search name, code, phone..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8 text-xs h-9"
            />
          </div>

          {/* Program filter */}
          <select
            value={selectedProgramId}
            onChange={(e) => {
              setSelectedProgramId(e.target.value);
              setSelectedBatchId("");
            }}
            className="h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option value="">All Academic Programs</option>
            {programs.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>

          {/* Batch filter */}
          <select
            value={selectedBatchId}
            onChange={(e) => setSelectedBatchId(e.target.value)}
            className="h-9 px-3 rounded-[var(--radius-md)] border border-input bg-background text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option value="">All Batches</option>
            {batches
              .filter((b) => !selectedProgramId || b.program_id === selectedProgramId)
              .map((b) => (
                <option key={b.id} value={b.id}>
                  {b.name}
                </option>
              ))}
          </select>

          {/* Overdue toggle */}
          <label className="flex items-center gap-1.5 text-xs text-foreground cursor-pointer select-none ml-1">
            <input
              type="checkbox"
              checked={overdueOnly}
              onChange={(e) => setOverdueOnly(e.target.checked)}
              className="rounded border-input text-primary focus:ring-primary h-4 w-4"
            />
            <span className="font-medium text-amber-600 dark:text-amber-400">
              Overdue only
            </span>
          </label>
        </div>

        <div className="flex items-center gap-2 self-end sm:self-auto">
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadDues()}
            disabled={loading}
            className="h-9 text-xs gap-1.5"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          {onOpenSmartDueQueue && (
            <Button
              variant="outline"
              size="sm"
              onClick={onOpenSmartDueQueue}
              className="h-9 text-xs gap-1.5 border-primary/40 text-primary hover:bg-primary-subtle"
            >
              <Sparkles className="h-3.5 w-3.5" />
              Smart Due Queue
            </Button>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={exportCsv}
            disabled={dues.length === 0}
            className="h-9 text-xs gap-1.5"
          >
            <Download className="h-3.5 w-3.5" />
            Export CSV
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20 flex items-center gap-2">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Dues Directory Table */}
      <div className="bg-card rounded-[var(--radius-lg)] border border-border overflow-hidden shadow-xs">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className="border-b border-border bg-muted/40 text-muted-foreground font-semibold uppercase tracking-wider text-[10px]">
                <th className="py-3 px-4">Student</th>
                <th className="py-3 px-4">Guardian / Contact</th>
                <th className="py-3 px-4">Batch & Roll</th>
                <th className="py-3 px-4 text-right">Current Month Due</th>
                <th className="py-3 px-4 text-right">Overdue</th>
                <th className="py-3 px-4 text-right">Total Outstanding</th>
                <th className="py-3 px-4 text-center">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {loading && dues.length === 0 ? (
                <tr>
                  <td colSpan={7} className="py-12 text-center text-muted-foreground">
                    <Loader2 className="h-6 w-6 animate-spin mx-auto mb-2 text-primary" />
                    Loading dues directory...
                  </td>
                </tr>
              ) : dues.length === 0 ? (
                <tr>
                  <td colSpan={7} className="py-12 text-center text-muted-foreground">
                    No outstanding dues found matching current filters.
                  </td>
                </tr>
              ) : (
                dues.map((d) => (
                  <tr key={d.student_id} className="hover:bg-muted/30 transition-colors">
                    <td className="py-3 px-4">
                      <div className="font-semibold text-foreground">{d.name}</div>
                      <div className="flex items-center gap-1.5 mt-0.5">
                        <span className="font-mono text-[11px] text-primary font-medium">
                          {d.student_code}
                        </span>
                        {d.name_bn && (
                          <span className="text-[10px] text-muted-foreground font-bengali">
                            ({d.name_bn})
                          </span>
                        )}
                      </div>
                    </td>

                    <td className="py-3 px-4">
                      <div className="text-foreground">{d.guardian_name || "Self"}</div>
                      <div className="font-mono text-[11px] text-muted-foreground">
                        {d.phone}
                      </div>
                    </td>

                    <td className="py-3 px-4">
                      <div className="font-medium text-foreground">
                        {d.batch_name || "--"}
                      </div>
                      {d.roll_no && (
                        <div className="text-[11px] text-muted-foreground">
                          Roll: {d.roll_no}
                        </div>
                      )}
                    </td>

                    <td className="py-3 px-4 text-right font-mono font-medium text-foreground">
                      {formatTaka(d.current_month_due_poisha)}
                    </td>

                    <td className="py-3 px-4 text-right">
                      {d.overdue_poisha > 0 ? (
                        <span className="font-mono font-bold text-amber-600 dark:text-amber-400">
                          {formatTaka(d.overdue_poisha)}
                        </span>
                      ) : (
                        <span className="text-muted-foreground">--</span>
                      )}
                    </td>

                    <td className="py-3 px-4 text-right">
                      <div className="font-mono font-bold text-destructive text-sm">
                        {formatTaka(d.total_due_poisha)}
                      </div>
                      {d.overdue_poisha > 0 ? (
                        <Badge
                          variant="outline"
                          className="text-[9px] px-1 py-0 text-amber-600 border-amber-300"
                        >
                          Overdue
                        </Badge>
                      ) : (
                        <Badge
                          variant="outline"
                          className="text-[9px] px-1 py-0 text-success border-success/30"
                        >
                          Current
                        </Badge>
                      )}
                    </td>

                    <td className="py-3 px-4 text-center">
                      <div className="flex items-center justify-center gap-1.5">
                        {onCollectPayment && (
                          <Button
                            size="sm"
                            variant="primary"
                            onClick={() => onCollectPayment(d.student_id)}
                            className="h-7 px-2 text-[11px] gap-1"
                          >
                            <CreditCard className="h-3 w-3" />
                            Collect
                          </Button>
                        )}
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => void handlePrintStatement(d.student_id)}
                          disabled={statementLoadingId === d.student_id}
                          className="h-7 px-2 text-[11px] gap-1"
                        >
                          {statementLoadingId === d.student_id ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                          ) : (
                            <Printer className="h-3 w-3" />
                          )}
                          Statement
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Statement Print Preview Dialog */}
      <PrintPreviewDialog
        open={previewOpen}
        onOpenChange={setPreviewOpen}
        title="Student Due Statement / Account Statement"
        htmlContent={previewHtml}
        pageSize="a4"
        onPrintConfirmed={async () => {
          await executeHtmlPrint(previewHtml);
          setPreviewOpen(false);
        }}
      />
    </div>
  );
}
