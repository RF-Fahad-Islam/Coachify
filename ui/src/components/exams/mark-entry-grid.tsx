import { useState, useEffect, useRef } from "react";
import { rpc } from "@/gen/rpc";
import type {
  ExamDetails,
  MarkSheetData,
  SaveMarksPayload,
  SaveMarksRowInput,
} from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  ArrowLeft,
  Save,
  CheckCircle2,
  Lock,
  Unlock,
  AlertTriangle,
  Info,
} from "lucide-react";

interface MarkEntryGridProps {
  examId: string;
  initialSubjectId?: string;
  onBack: () => void;
}

interface StudentRowState {
  student_id: string;
  roll?: number | null;
  student_code: string;
  name: string;
  name_bn?: string | null;
  marksStr: string; // string for fluid input typing
  is_absent: boolean;
  is_optional: boolean;
  note: string;
  status: string;
  hasError: boolean;
  errorMessage?: string;
}

export function MarkEntryGrid({ examId, initialSubjectId, onBack }: MarkEntryGridProps) {
  const [examDetails, setExamDetails] = useState<ExamDetails | null>(null);
  const [selectedExamSubjectId, setSelectedExamSubjectId] = useState<string>(
    initialSubjectId || ""
  );
  const [markSheet, setMarkSheet] = useState<MarkSheetData | null>(null);
  const [rows, setRows] = useState<StudentRowState[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  // Load exam details
  useEffect(() => {
    async function loadExam() {
      setLoading(true);
      try {
        const res = await rpc("exams.get", { exam_id: examId });
        if (res.ok && res.data) {
          setExamDetails(res.data);
          if (res.data.subjects.length > 0) {
            setSelectedExamSubjectId((prev) => prev || res.data!.subjects[0].id);
          }
        }
      } catch (err) {
        console.error("Failed to load exam details", err);
      } finally {
        setLoading(false);
      }
    }
    void loadExam();
  }, [examId]);

  // Load subject mark sheet when selected subject changes
  useEffect(() => {
    if (!selectedExamSubjectId) return;

    async function loadMarkSheet() {
      setLoading(true);
      setMessage(null);
      try {
        const res = await rpc("exams.get_mark_sheet", {
          exam_subject_id: selectedExamSubjectId,
        });
        if (res.ok && res.data) {
          setMarkSheet(res.data);
          const mappedRows: StudentRowState[] = res.data.rows.map((r) => {
            const marksStr =
              r.marks !== null && r.marks !== undefined ? (r.marks / 100).toString() : "";
            return {
              student_id: r.student_id,
              roll: r.roll,
              student_code: r.student_code,
              name: r.name,
              name_bn: r.name_bn,
              marksStr,
              is_absent: r.is_absent,
              is_optional: r.is_optional,
              note: r.note || "",
              status: r.status,
              hasError: false,
            };
          });
          setRows(mappedRows);
        }
      } catch (err) {
        console.error("Failed to load mark sheet", err);
      } finally {
        setLoading(false);
      }
    }
    void loadMarkSheet();
  }, [selectedExamSubjectId]);

  // Validate single mark row
  const validateMark = (
    marksStr: string,
    isAbsent: boolean,
    fullMarksHundredths: number
  ) => {
    if (isAbsent) return { hasError: false };
    if (!marksStr.trim()) return { hasError: false };

    const val = parseFloat(marksStr);
    if (isNaN(val)) {
      return { hasError: true, errorMessage: "Must be a valid number" };
    }
    if (val < 0) {
      return { hasError: true, errorMessage: "Marks cannot be negative" };
    }
    if (Math.round(val * 100) > fullMarksHundredths) {
      return {
        hasError: true,
        errorMessage: `Marks cannot exceed full marks (${fullMarksHundredths / 100})`,
      };
    }
    return { hasError: false };
  };

  const handleMarkChange = (index: number, val: string) => {
    if (!markSheet) return;
    setRows((prev) =>
      prev.map((row, i) => {
        if (i !== index) return row;
        const validation = validateMark(val, row.is_absent, markSheet.full_marks);
        return {
          ...row,
          marksStr: val,
          hasError: validation.hasError,
          errorMessage: validation.errorMessage,
        };
      })
    );
  };

  const handleToggleAbsent = (index: number) => {
    if (!markSheet) return;
    setRows((prev) =>
      prev.map((row, i) => {
        if (i !== index) return row;
        const nextAbsent = !row.is_absent;
        const nextMarksStr = nextAbsent ? "" : row.marksStr;
        const validation = validateMark(nextMarksStr, nextAbsent, markSheet.full_marks);
        return {
          ...row,
          is_absent: nextAbsent,
          marksStr: nextMarksStr,
          hasError: validation.hasError,
          errorMessage: validation.errorMessage,
        };
      })
    );
  };

  const handleToggleOptional = (index: number) => {
    setRows((prev) =>
      prev.map((row, i) =>
        i === index ? { ...row, is_optional: !row.is_optional } : row
      )
    );
  };

  // Keyboard navigation & shortcuts
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>, index: number) => {
    if (e.key === "ArrowDown" || e.key === "Enter") {
      e.preventDefault();
      if (index < rows.length - 1) {
        inputRefs.current[index + 1]?.focus();
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (index > 0) {
        inputRefs.current[index - 1]?.focus();
      }
    } else if (e.key === "a" || e.key === "A") {
      // If pressing 'a' when input is empty, toggle absent
      if (rows[index].marksStr === "") {
        e.preventDefault();
        handleToggleAbsent(index);
      }
    }
  };

  // Save marks
  const handleSaveMarks = async () => {
    if (!markSheet) return;

    // Check errors
    const anyError = rows.some((r) => r.hasError);
    if (anyError) {
      setMessage({
        type: "error",
        text: "Please resolve all mark input errors before saving.",
      });
      return;
    }

    setSaving(true);
    setMessage(null);

    const inputs: SaveMarksRowInput[] = rows.map((r) => {
      const marksHundredths =
        !r.is_absent && r.marksStr.trim() !== ""
          ? Math.round(parseFloat(r.marksStr) * 100)
          : null;

      return {
        student_id: r.student_id,
        marks: marksHundredths,
        is_absent: r.is_absent,
        is_optional: r.is_optional,
        note: r.note.trim() || null,
      };
    });

    const payload: SaveMarksPayload = {
      exam_subject_id: selectedExamSubjectId,
      marks: inputs,
    };

    try {
      const res = await rpc("exams.save_marks", payload);
      if (res.ok && res.data) {
        setMarkSheet(res.data);
        setMessage({ type: "success", text: "Marks successfully saved as draft." });
      } else {
        setMessage({ type: "error", text: res.error?.message || "Failed to save marks" });
      }
    } catch (err: unknown) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setSaving(false);
    }
  };

  // Verify Subject Marks
  const handleVerifySubject = async () => {
    // Check if unentered students exist
    const unrecordedCount = rows.filter(
      (r) => !r.is_absent && r.marksStr.trim() === ""
    ).length;

    if (unrecordedCount > 0) {
      setMessage({
        type: "error",
        text: `Cannot verify subject: ${unrecordedCount} student(s) have neither marks nor absence recorded.`,
      });
      return;
    }

    // Save first to ensure latest marks are persisted
    await handleSaveMarks();

    setSaving(true);
    try {
      const res = await rpc("exams.verify_subject", {
        exam_subject_id: selectedExamSubjectId,
      });
      if (res.ok) {
        setMessage({
          type: "success",
          text: "Subject marks verified and locked! You can now proceed to results preview.",
        });
        // Reload mark sheet
        const refresh = await rpc("exams.get_mark_sheet", {
          exam_subject_id: selectedExamSubjectId,
        });
        if (refresh.ok && refresh.data) {
          setMarkSheet(refresh.data);
        }
      } else {
        setMessage({
          type: "error",
          text: res.error?.message || "Failed to verify subject marks",
        });
      }
    } catch (err: unknown) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setSaving(false);
    }
  };

  // Unverify Subject Marks
  const handleUnverifySubject = async () => {
    setSaving(true);
    try {
      const res = await rpc("exams.unverify_subject", {
        exam_subject_id: selectedExamSubjectId,
      });
      if (res.ok) {
        setMessage({
          type: "success",
          text: "Subject marks unverified. Marks are now editable.",
        });
        const refresh = await rpc("exams.get_mark_sheet", {
          exam_subject_id: selectedExamSubjectId,
        });
        if (refresh.ok && refresh.data) {
          setMarkSheet(refresh.data);
        }
      } else {
        setMessage({
          type: "error",
          text: res.error?.message || "Failed to unverify subject marks",
        });
      }
    } catch (err: unknown) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setSaving(false);
    }
  };

  const isVerified = markSheet?.is_verified ?? false;
  const isExamPublished = examDetails?.exam.status === "published";

  const fullMarks = markSheet ? markSheet.full_marks / 100 : 100;
  const passMarks = markSheet ? markSheet.pass_marks / 100 : 33;
  const enteredCount = rows.filter((r) => r.is_absent || r.marksStr.trim() !== "").length;
  const totalCount = rows.length;

  return (
    <div className="space-y-5">
      {/* Header and Back button */}
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
                {examDetails?.exam.name || "Exam Mark Entry"}
              </h1>
              {isExamPublished ? (
                <Badge className="bg-emerald-500/15 text-emerald-600 border-emerald-500/30 text-xs">
                  Published
                </Badge>
              ) : isVerified ? (
                <Badge className="bg-indigo-500/15 text-indigo-600 border-indigo-500/30 text-xs">
                  Verified
                </Badge>
              ) : (
                <Badge className="bg-amber-500/15 text-amber-600 border-amber-500/30 text-xs">
                  Entry in Progress
                </Badge>
              )}
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Subject-wise mark entry with keyboard navigation and instant bounds
              validation.
            </p>
          </div>
        </div>

        {/* Subject Switcher & Actions */}
        <div className="flex flex-wrap items-center gap-2.5">
          <div className="w-56">
            <Select
              value={selectedExamSubjectId}
              onValueChange={setSelectedExamSubjectId}
              disabled={loading}
            >
              <SelectTrigger className="h-9 text-xs">
                <SelectValue placeholder="Select Subject" />
              </SelectTrigger>
              <SelectContent>
                {examDetails?.subjects.map((s) => (
                  <SelectItem key={s.id} value={s.id}>
                    {s.subject_name} ({s.subject_code}){s.is_verified ? " ✓" : ""}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {!isExamPublished && (
            <>
              {isVerified ? (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleUnverifySubject}
                  disabled={saving}
                  className="gap-1.5 text-xs h-9 border-amber-500/40 text-amber-600 hover:bg-amber-500/10"
                >
                  <Unlock className="h-3.5 w-3.5" />
                  Unverify Marks
                </Button>
              ) : (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleSaveMarks}
                    disabled={saving || loading}
                    className="gap-1.5 text-xs h-9"
                  >
                    <Save className="h-3.5 w-3.5" />
                    Save Draft
                  </Button>
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={handleVerifySubject}
                    disabled={saving || loading || enteredCount < totalCount}
                    className="gap-1.5 text-xs h-9"
                    title={
                      enteredCount < totalCount
                        ? `Cannot verify: ${totalCount - enteredCount} students missing marks`
                        : "Verify and lock subject marks"
                    }
                  >
                    <Lock className="h-3.5 w-3.5" />
                    Verify Subject Marks
                  </Button>
                </>
              )}
            </>
          )}
        </div>
      </div>

      {/* Info Strip */}
      <div className="flex flex-wrap items-center justify-between gap-4 p-3 bg-muted/40 border border-border rounded-lg text-xs">
        <div className="flex items-center gap-6">
          <div>
            <span className="text-muted-foreground">Subject: </span>
            <span className="font-semibold text-foreground">
              {markSheet?.subject_name} ({markSheet?.subject_code})
            </span>
          </div>
          <div>
            <span className="text-muted-foreground">Full Marks: </span>
            <span className="font-semibold text-foreground">{fullMarks}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Pass Marks: </span>
            <span className="font-semibold text-foreground">{passMarks}</span>
          </div>
          <div>
            <span className="text-muted-foreground">Entry Progress: </span>
            <span
              className={`font-semibold ${enteredCount === totalCount ? "text-emerald-600" : "text-amber-600"}`}
            >
              {enteredCount} / {totalCount} Recorded
            </span>
          </div>
        </div>

        <div className="flex items-center gap-3 text-muted-foreground">
          <Info className="h-3.5 w-3.5" />
          <span>
            Use <strong>↑ / ↓ / Enter</strong> to move &bull; Type <strong>A</strong> for
            Absent
          </span>
        </div>
      </div>

      {message && (
        <div
          className={`flex items-center gap-2 p-3 text-sm rounded-md border ${
            message.type === "success"
              ? "bg-emerald-500/10 text-emerald-700 border-emerald-500/20"
              : "bg-destructive/10 text-destructive border-destructive/20"
          }`}
        >
          {message.type === "success" ? (
            <CheckCircle2 className="h-4 w-4 shrink-0" />
          ) : (
            <AlertTriangle className="h-4 w-4 shrink-0" />
          )}
          <span>{message.text}</span>
        </div>
      )}

      {/* Roster Grid */}
      <div className="border border-border rounded-lg overflow-hidden bg-card">
        <table className="w-full text-xs">
          <thead className="bg-muted/60 text-muted-foreground font-medium border-b border-border">
            <tr>
              <th className="p-2.5 text-center w-16">Roll</th>
              <th className="p-2.5 text-center w-28">Student ID</th>
              <th className="p-2.5 text-left">Student Name</th>
              <th className="p-2.5 text-center w-36">Marks (পূর্ণমান {fullMarks})</th>
              <th className="p-2.5 text-center w-24">Absent (অনুপস্থিত)</th>
              <th className="p-2.5 text-center w-24">4th Subject</th>
              <th className="p-2.5 text-left w-48">Note</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {rows.length === 0 ? (
              <tr>
                <td colSpan={7} className="p-8 text-center text-muted-foreground">
                  {loading
                    ? "Loading student roster..."
                    : "No active students enrolled in participating batches."}
                </td>
              </tr>
            ) : (
              rows.map((row, idx) => {
                const markVal = parseFloat(row.marksStr);
                const isFail = !row.is_absent && !isNaN(markVal) && markVal < passMarks;

                return (
                  <tr
                    key={row.student_id}
                    className={`hover:bg-muted/30 transition-colors ${
                      row.is_absent ? "bg-muted/40" : isFail ? "bg-destructive/5" : ""
                    }`}
                  >
                    <td className="p-2.5 text-center font-semibold text-muted-foreground">
                      {row.roll !== null && row.roll !== undefined ? row.roll : "--"}
                    </td>
                    <td className="p-2.5 text-center font-mono text-muted-foreground">
                      {row.student_code}
                    </td>
                    <td className="p-2.5 font-medium text-foreground">
                      <div>{row.name}</div>
                      {row.name_bn && (
                        <div className="text-[11px] text-muted-foreground font-normal">
                          {row.name_bn}
                        </div>
                      )}
                    </td>
                    <td className="p-2.5">
                      <div className="relative">
                        <Input
                          ref={(el) => {
                            inputRefs.current[idx] = el;
                          }}
                          type="number"
                          step="any"
                          value={row.marksStr}
                          onChange={(e) => handleMarkChange(idx, e.target.value)}
                          onKeyDown={(e) => handleKeyDown(e, idx)}
                          disabled={row.is_absent || isVerified || isExamPublished}
                          placeholder={row.is_absent ? "ABSENT" : "0.00"}
                          className={`h-8 text-center text-xs font-semibold ${
                            row.hasError
                              ? "border-destructive text-destructive focus-visible:ring-destructive"
                              : isFail
                                ? "text-destructive font-bold"
                                : ""
                          }`}
                        />
                        {row.hasError && row.errorMessage && (
                          <div className="text-[10px] text-destructive mt-0.5 text-center">
                            {row.errorMessage}
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="p-2.5 text-center">
                      <button
                        type="button"
                        onClick={() => handleToggleAbsent(idx)}
                        disabled={isVerified || isExamPublished}
                        className={`px-2.5 py-1 rounded text-[11px] font-bold border transition-colors ${
                          row.is_absent
                            ? "bg-destructive text-destructive-foreground border-destructive"
                            : "bg-background text-muted-foreground border-border hover:bg-muted"
                        }`}
                      >
                        {row.is_absent ? "ABSENT" : "Present"}
                      </button>
                    </td>
                    <td className="p-2.5 text-center">
                      <input
                        type="checkbox"
                        checked={row.is_optional}
                        onChange={() => handleToggleOptional(idx)}
                        disabled={isVerified || isExamPublished}
                        className="rounded border-border text-primary focus:ring-primary h-4 w-4"
                        title="Mark as 4th subject for bonus calculation"
                      />
                    </td>
                    <td className="p-2.5">
                      <Input
                        type="text"
                        value={row.note}
                        onChange={(e) =>
                          setRows((prev) =>
                            prev.map((r, i) =>
                              i === idx ? { ...r, note: e.target.value } : r
                            )
                          )
                        }
                        disabled={isVerified || isExamPublished}
                        placeholder="Optional remarks..."
                        className="h-8 text-xs"
                      />
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
