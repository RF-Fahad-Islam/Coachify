import { useState, useEffect, useCallback } from "react";
import { rpc } from "@/gen/rpc";
import type { ExamResultPreviewData, ResultPreviewStudentRow } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  ArrowLeft,
  Award,
  Users,
  CheckCircle,
  XCircle,
  TrendingUp,
  AlertTriangle,
  Printer,
  FileSpreadsheet,
  Send,
  Search,
} from "lucide-react";
import { PrintPreviewDialog } from "@/components/print/print-preview-dialog";
import { generateTabulationSheetHtml } from "@/components/print/tabulation-sheet-template";
import { generateReportCardHtml } from "@/components/print/report-card-template";

interface ResultPublishViewProps {
  examId: string;
  onBack: () => void;
}

export function ResultPublishView({ examId, onBack }: ResultPublishViewProps) {
  const [loading, setLoading] = useState(true);
  const [publishing, setPublishing] = useState(false);
  const [preview, setPreview] = useState<ExamResultPreviewData | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [publishDialogOpen, setPublishDialogOpen] = useState(false);
  const [sendSms, setSendSms] = useState(true);
  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  // Print Preview state
  const [printDialogOpen, setPrintDialogOpen] = useState(false);
  const [printHtml, setPrintHtml] = useState<string | null>(null);
  const [printTitle, setPrintTitle] = useState("");
  const [printPageSize, setPrintPageSize] = useState("a4");

  const loadPreview = useCallback(async () => {
    setLoading(true);
    try {
      const res = await rpc("exams.preview_results", { exam_id: examId });
      if (res.ok && res.data) {
        setPreview(res.data);
      } else {
        setStatusMessage({
          type: "error",
          text: res.error?.message || "Failed to calculate results preview",
        });
      }
    } catch (err: unknown) {
      setStatusMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoading(false);
    }
  }, [examId]);

  useEffect(() => {
    void loadPreview();
  }, [loadPreview]);

  const handlePublishConfirm = async () => {
    setPublishing(true);
    setStatusMessage(null);
    try {
      const res = await rpc("exams.publish_results", {
        exam_id: examId,
        send_notifications: sendSms,
      });

      if (res.ok && res.data) {
        setPublishDialogOpen(false);
        setStatusMessage({
          type: "success",
          text: `Exam results published successfully (Version ${res.data.version})! ${
            res.data.notifications_enqueued > 0
              ? `${res.data.notifications_enqueued} guardian notifications queued to outbox.`
              : ""
          }`,
        });
        await loadPreview();
      } else {
        setStatusMessage({
          type: "error",
          text: res.error?.message || "Failed to publish exam results",
        });
      }
    } catch (err: unknown) {
      setStatusMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setPublishing(false);
    }
  };

  // Open Tabulation Sheet print preview
  const handlePrintTabulation = async () => {
    if (!preview || preview.students.length === 0) return;
    const batchId = preview.students[0].batch_id;

    try {
      const res = await rpc("exams.get_tabulation_sheet", {
        exam_id: examId,
        batch_id: batchId,
      });

      if (res.ok && res.data) {
        const html = generateTabulationSheetHtml(res.data);
        setPrintHtml(html);
        setPrintTitle(`Tabulation Sheet - ${preview.exam_name}`);
        setPrintPageSize("a4 landscape");
        setPrintDialogOpen(true);
      }
    } catch (err) {
      console.error("Failed to load tabulation sheet", err);
    }
  };

  // Open Individual Student Report Card print preview
  const handlePrintReportCard = async (student: ResultPreviewStudentRow) => {
    try {
      const res = await rpc("exams.get_student_report_card", {
        exam_id: examId,
        student_id: student.student_id,
      });

      if (res.ok && res.data) {
        const html = generateReportCardHtml(res.data);
        setPrintHtml(html);
        setPrintTitle(`Report Card - ${student.name}`);
        setPrintPageSize("a4");
        setPrintDialogOpen(true);
      }
    } catch (err) {
      console.error("Failed to load student report card", err);
    }
  };

  const hasUnverified = Boolean(preview && preview.unverified_subjects.length > 0);

  const filteredStudents = preview
    ? preview.students.filter(
        (s) =>
          s.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          s.student_code.toLowerCase().includes(searchTerm.toLowerCase()) ||
          (s.roll && s.roll.toString().includes(searchTerm))
      )
    : [];

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            onClick={onBack}
            className="h-8 w-8 text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-bold tracking-tight text-foreground">
                {preview?.exam_name || "Exam Results & Grading"}
              </h1>
              <Badge variant="outline" className="text-xs">
                {preview?.grading_scheme_name || "NCTB GPA-5"}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Comprehensive results tabulation, 5-tier deterministic merit ranks, and
              report cards.
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handlePrintTabulation}
            disabled={loading || !preview}
            className="gap-1.5 text-xs h-9"
          >
            <FileSpreadsheet className="h-3.5 w-3.5 text-primary" />
            Tabulation Sheet
          </Button>

          <Button
            variant="primary"
            size="sm"
            onClick={() => setPublishDialogOpen(true)}
            disabled={loading || !preview || hasUnverified}
            className="gap-1.5 text-xs h-9"
          >
            <Award className="h-3.5 w-3.5" />
            Publish Results
          </Button>
        </div>
      </div>

      {statusMessage && (
        <div
          className={`flex items-center gap-2 p-3 text-sm rounded-md border ${
            statusMessage.type === "success"
              ? "bg-emerald-500/10 text-emerald-700 border-emerald-500/20"
              : "bg-destructive/10 text-destructive border-destructive/20"
          }`}
        >
          {statusMessage.type === "success" ? (
            <CheckCircle className="h-4 w-4 shrink-0" />
          ) : (
            <AlertTriangle className="h-4 w-4 shrink-0" />
          )}
          <span>{statusMessage.text}</span>
        </div>
      )}

      {/* Unverified Warning Banner */}
      {hasUnverified && (
        <div className="flex items-start gap-3 p-3.5 bg-amber-500/10 border border-amber-500/30 rounded-lg text-amber-800 dark:text-amber-300">
          <AlertTriangle className="h-5 w-5 shrink-0 mt-0.5 text-amber-600" />
          <div className="text-xs space-y-1">
            <div className="font-semibold text-sm">
              Incomplete Verification Gate (যাচাইকরণ অসম্পূর্ণ)
            </div>
            <div>
              Results cannot be published until all examination subjects are verified. The
              following subjects require mark entry and verification:
            </div>
            <div className="flex flex-wrap gap-1.5 mt-1.5">
              {preview?.unverified_subjects.map((sub, i) => (
                <Badge
                  key={i}
                  className="bg-amber-500/20 text-amber-800 dark:text-amber-200 border-amber-500/40 text-[11px]"
                >
                  {sub}
                </Badge>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Metric Cards */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div className="p-4 bg-card border border-border rounded-xl">
          <div className="flex items-center justify-between text-muted-foreground text-xs">
            <span>Total Candidates</span>
            <Users className="h-4 w-4 text-primary" />
          </div>
          <div className="text-2xl font-bold mt-2 text-foreground">
            {preview ? preview.total_students : "--"}
          </div>
          <div className="text-[11px] text-muted-foreground mt-1">Enrolled students</div>
        </div>

        <div className="p-4 bg-card border border-border rounded-xl">
          <div className="flex items-center justify-between text-muted-foreground text-xs">
            <span>Passed (উত্তীর্ণ)</span>
            <CheckCircle className="h-4 w-4 text-emerald-600" />
          </div>
          <div className="text-2xl font-bold mt-2 text-emerald-600">
            {preview ? preview.passed_students : "--"}
          </div>
          <div className="text-[11px] text-muted-foreground mt-1">
            {preview && preview.total_students > 0
              ? `${Math.round((preview.passed_students / preview.total_students) * 100)}% Pass Rate`
              : "--"}
          </div>
        </div>

        <div className="p-4 bg-card border border-border rounded-xl">
          <div className="flex items-center justify-between text-muted-foreground text-xs">
            <span>Failed (অনুত্তীর্ণ)</span>
            <XCircle className="h-4 w-4 text-destructive" />
          </div>
          <div className="text-2xl font-bold mt-2 text-destructive">
            {preview ? preview.failed_students : "--"}
          </div>
          <div className="text-[11px] text-muted-foreground mt-1">
            {preview && preview.total_students > 0
              ? `${Math.round((preview.failed_students / preview.total_students) * 100)}% Failed`
              : "--"}
          </div>
        </div>

        <div className="p-4 bg-card border border-border rounded-xl">
          <div className="flex items-center justify-between text-muted-foreground text-xs">
            <span>Average GPA (গড় জিপিএ)</span>
            <TrendingUp className="h-4 w-4 text-indigo-500" />
          </div>
          <div className="text-2xl font-bold mt-2 text-indigo-500">
            {preview ? (preview.average_gpa / 100).toFixed(2) : "--"}
          </div>
          <div className="text-[11px] text-muted-foreground mt-1">
            Batch academic average
          </div>
        </div>
      </div>

      {/* Roster & Filter */}
      <div className="space-y-3">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="relative w-full sm:w-72">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="text"
              placeholder="Search by name, roll, or ID..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-9 h-9 text-xs"
            />
          </div>
          <div className="text-xs text-muted-foreground">
            Showing <strong>{filteredStudents.length}</strong> of{" "}
            <strong>{preview?.students.length || 0}</strong> candidates
          </div>
        </div>

        {/* Results Table */}
        <div className="border border-border rounded-lg overflow-hidden bg-card">
          <table className="w-full text-xs">
            <thead className="bg-muted/60 text-muted-foreground font-medium border-b border-border">
              <tr>
                <th className="p-2.5 text-center w-14">Rank</th>
                <th className="p-2.5 text-center w-14">Roll</th>
                <th className="p-2.5 text-center w-24">ID</th>
                <th className="p-2.5 text-left">Student Name</th>
                <th className="p-2.5 text-center w-24">Total Marks</th>
                <th className="p-2.5 text-center w-20">Percentage</th>
                <th className="p-2.5 text-center w-20">GPA</th>
                <th className="p-2.5 text-center w-16">Grade</th>
                <th className="p-2.5 text-center w-24">Status</th>
                <th className="p-2.5 text-center w-28">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filteredStudents.length === 0 ? (
                <tr>
                  <td colSpan={10} className="p-8 text-center text-muted-foreground">
                    {loading
                      ? "Calculating and sorting candidates..."
                      : "No candidates found matching filter."}
                  </td>
                </tr>
              ) : (
                filteredStudents.map((st) => {
                  const gpaFormatted = (st.gpa / 100).toFixed(2);
                  const isTopRank = st.rank <= 3 && st.passed;

                  return (
                    <tr
                      key={st.student_id}
                      className={`hover:bg-muted/30 transition-colors ${
                        !st.passed ? "bg-destructive/5" : ""
                      }`}
                    >
                      <td className="p-2.5 text-center font-bold">
                        <span
                          className={`inline-flex items-center justify-center h-6 w-6 rounded-full text-xs ${
                            isTopRank
                              ? "bg-amber-500/20 text-amber-700 font-extrabold"
                              : "text-muted-foreground"
                          }`}
                        >
                          #{st.rank}
                        </span>
                      </td>
                      <td className="p-2.5 text-center font-semibold text-muted-foreground">
                        {st.roll !== null && st.roll !== undefined ? st.roll : "--"}
                      </td>
                      <td className="p-2.5 text-center font-mono text-muted-foreground">
                        {st.student_code}
                      </td>
                      <td className="p-2.5 font-medium text-foreground">
                        <div>{st.name}</div>
                        {st.name_bn && (
                          <div className="text-[11px] text-muted-foreground font-normal">
                            {st.name_bn}
                          </div>
                        )}
                      </td>
                      <td className="p-2.5 text-center font-semibold">
                        {(st.total_marks / 100).toFixed(
                          st.total_marks % 100 === 0 ? 0 : 1
                        )}
                        <span className="text-[10px] text-muted-foreground">
                          {" "}
                          / {(st.total_max_marks / 100).toFixed(0)}
                        </span>
                      </td>
                      <td className="p-2.5 text-center font-medium">
                        {(st.percentage / 100).toFixed(1)}%
                      </td>
                      <td
                        className={`p-2.5 text-center font-bold text-sm ${
                          st.passed ? "text-emerald-600" : "text-destructive"
                        }`}
                      >
                        {gpaFormatted}
                      </td>
                      <td
                        className={`p-2.5 text-center font-extrabold ${
                          st.passed ? "text-emerald-600" : "text-destructive"
                        }`}
                      >
                        {st.grade}
                      </td>
                      <td className="p-2.5 text-center">
                        <Badge
                          className={
                            st.passed
                              ? "bg-emerald-500/15 text-emerald-700 border-emerald-500/30 text-[10px]"
                              : "bg-destructive/15 text-destructive border-destructive/30 text-[10px]"
                          }
                        >
                          {st.passed ? "PASSED" : "FAILED"}
                        </Badge>
                      </td>
                      <td className="p-2.5 text-center">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handlePrintReportCard(st)}
                          className="h-7 text-xs gap-1 text-primary hover:bg-primary/10"
                        >
                          <Printer className="h-3.5 w-3.5" />
                          Report Card
                        </Button>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Publish Confirmation Dialog */}
      <Dialog open={publishDialogOpen} onOpenChange={setPublishDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-lg font-bold">
              <Award className="h-5 w-5 text-primary" />
              Publish Examination Results
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4 py-2 text-sm text-foreground">
            <p>
              Are you sure you want to publish the results for{" "}
              <strong>{preview?.exam_name}</strong>?
            </p>
            <p className="text-xs text-muted-foreground">
              Publishing freezes an immutable snapshot of all student marks, GPAs, and
              merit ranks. Mark entry will be locked.
            </p>

            <div className="p-3 bg-muted/40 border border-border rounded-lg space-y-2">
              <label className="flex items-center gap-2 cursor-pointer text-xs">
                <input
                  type="checkbox"
                  checked={sendSms}
                  onChange={(e) => setSendSms(e.target.checked)}
                  className="rounded border-border text-primary focus:ring-primary h-4 w-4"
                />
                <span className="font-medium text-foreground">
                  Queue SMS result notification to guardians
                </span>
              </label>
              <p className="text-[11px] text-muted-foreground pl-6">
                Sends personalized SMS with GPA, Grade, and Merit Position to guardians
                with SMS alerts enabled.
              </p>
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              onClick={() => setPublishDialogOpen(false)}
              disabled={publishing}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handlePublishConfirm}
              disabled={publishing}
              className="gap-1.5"
            >
              <Send className="h-4 w-4" />
              {publishing ? "Publishing..." : "Confirm & Publish"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Print Preview Dialog */}
      {printHtml && (
        <PrintPreviewDialog
          open={printDialogOpen}
          onOpenChange={setPrintDialogOpen}
          htmlContent={printHtml}
          title={printTitle}
          pageSize={printPageSize}
        />
      )}
    </div>
  );
}
