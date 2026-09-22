import { useState, useEffect } from "react";
import { rpc } from "@/gen/rpc";
import type {
  GradingSchemeSummary,
  BatchSummary,
  CreateExamPayload,
  CreateExamSubjectInput,
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Calendar, Plus, Trash2, BookOpen, AlertCircle } from "lucide-react";

interface SubjectOption {
  id: string;
  name: string;
  code: string;
}

interface CreateExamDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onExamCreated: () => void;
}

export function CreateExamDialog({
  open,
  onOpenChange,
  onExamCreated,
}: CreateExamDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form states
  const [name, setName] = useState("");
  const [nameBn, setNameBn] = useState("");
  const [sessionId] = useState("");
  const [examType, setExamType] = useState<string>("term");
  const [startDate, setStartDate] = useState("");
  const [endDate, setEndDate] = useState("");
  const [gradingSchemeId, setGradingSchemeId] = useState("");
  const [selectedBatchIds, setSelectedBatchIds] = useState<string[]>([]);
  const [subjects, setSubjects] = useState<CreateExamSubjectInput[]>([]);

  // Metadata options
  const [gradingSchemes, setGradingSchemes] = useState<GradingSchemeSummary[]>([]);
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [allSubjects, setAllSubjects] = useState<SubjectOption[]>([]);

  useEffect(() => {
    if (!open) return;

    // Reset fields
    setName("");
    setNameBn("");
    setExamType("term");
    const today = new Date().toISOString().split("T")[0];
    setStartDate(today);
    setEndDate(today);
    setError(null);
    setSelectedBatchIds([]);
    setSubjects([]);

    // Load grading schemes and batches
    async function fetchMetadata() {
      try {
        const [schemeRes, batchRes, subRes] = await Promise.all([
          rpc("grading_schemes.list", {}),
          rpc("batches.list", {}),
          rpc("subjects.list", {}),
        ]);

        if (schemeRes.ok && schemeRes.data) {
          setGradingSchemes(schemeRes.data);
          const defaultScheme =
            schemeRes.data.find((s) => s.id === "scheme_nctb_gpa5") || schemeRes.data[0];
          if (defaultScheme) {
            setGradingSchemeId(defaultScheme.id);
          }
        }

        if (batchRes.ok && batchRes.data) {
          setBatches(batchRes.data);
          if (batchRes.data.length > 0) {
            setSelectedBatchIds([batchRes.data[0].id]);
          }
        }

        if (subRes.ok && subRes.data) {
          setAllSubjects(subRes.data);
          // Pre-populate with first few subjects if available
          const initialSubjects: CreateExamSubjectInput[] = subRes.data
            .slice(0, 3)
            .map((s) => ({
              subject_id: s.id,
              full_marks: 10000, // 100.00
              pass_marks: 3300, // 33.00
              exam_date: today,
              start_time: "10:00",
              room_number: "Room 101",
              is_optional_allowed: false,
            }));
          setSubjects(initialSubjects);
        }
      } catch (err) {
        console.error("Failed to load exam metadata", err);
      }
    }

    void fetchMetadata();
  }, [open]);

  const handleAddSubject = () => {
    const unselected = allSubjects.find(
      (s) => !subjects.some((sub) => sub.subject_id === s.id)
    );
    const subId = unselected ? unselected.id : allSubjects[0]?.id || "";
    if (!subId) return;

    setSubjects((prev) => [
      ...prev,
      {
        subject_id: subId,
        full_marks: 10000,
        pass_marks: 3300,
        exam_date: startDate || new Date().toISOString().split("T")[0],
        start_time: "10:00",
        room_number: "Room 101",
        is_optional_allowed: false,
      },
    ]);
  };

  const handleRemoveSubject = (index: number) => {
    setSubjects((prev) => prev.filter((_, i) => i !== index));
  };

  const handleUpdateSubject = (
    index: number,
    updates: Partial<CreateExamSubjectInput>
  ) => {
    setSubjects((prev) =>
      prev.map((sub, i) => (i === index ? { ...sub, ...updates } : sub))
    );
  };

  const toggleBatch = (batchId: string) => {
    setSelectedBatchIds((prev) =>
      prev.includes(batchId) ? prev.filter((id) => id !== batchId) : [...prev, batchId]
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError("Exam name is required");
      return;
    }
    if (selectedBatchIds.length === 0) {
      setError("Please select at least one participating batch");
      return;
    }
    if (subjects.length === 0) {
      setError("Please add at least one subject to the exam");
      return;
    }

    setLoading(true);
    setError(null);

    const payload: CreateExamPayload = {
      name: name.trim(),
      name_bn: nameBn.trim() || null,
      session_id: sessionId || "sess-current",
      exam_type: examType,
      start_date: startDate,
      end_date: endDate,
      grading_scheme_id: gradingSchemeId,
      batch_ids: selectedBatchIds,
      subjects: subjects.map((s) => ({
        ...s,
        full_marks: Number(s.full_marks),
        pass_marks: Number(s.pass_marks),
      })),
    };

    try {
      const res = await rpc("exams.create", payload);
      if (res.ok) {
        onExamCreated();
        onOpenChange(false);
      } else {
        setError(res.error?.message || "Failed to create exam");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-xl font-bold">
            <BookOpen className="h-5 w-5 text-primary" />
            Create Examination Schedule (নতুন পরীক্ষা তৈরি)
          </DialogTitle>
        </DialogHeader>

        {error && (
          <div className="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 border border-destructive/20 rounded-md">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Basic Info */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label htmlFor="exam-name" className="text-xs font-semibold">
                Exam Title (English) *
              </label>
              <Input
                id="exam-name"
                placeholder="e.g. First Terminal Examination 2026"
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="exam-name-bn" className="text-xs font-semibold">
                Exam Title (বাংলা)
              </label>
              <Input
                id="exam-name-bn"
                placeholder="যেমন: ১ম সাময়িক পরীক্ষা ২০২৬"
                value={nameBn}
                onChange={(e) => setNameBn(e.target.value)}
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="exam-type" className="text-xs font-semibold">
                Exam Type *
              </label>
              <Select value={examType} onValueChange={setExamType}>
                <SelectTrigger id="exam-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="term">Terminal Exam (সাময়িক পরীক্ষা)</SelectItem>
                  <SelectItem value="class_test">Class Test (ক্লাস টেস্ট)</SelectItem>
                  <SelectItem value="model_test">Model Test (মডেল টেস্ট)</SelectItem>
                  <SelectItem value="quiz">Weekly Quiz (কুইজ)</SelectItem>
                  <SelectItem value="final">
                    Annual / Final Exam (বার্ষিক পরীক্ষা)
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <label htmlFor="grading-scheme" className="text-xs font-semibold">
                Grading Scheme *
              </label>
              <Select value={gradingSchemeId} onValueChange={setGradingSchemeId}>
                <SelectTrigger id="grading-scheme">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {gradingSchemes.map((scheme) => (
                    <SelectItem key={scheme.id} value={scheme.id}>
                      {scheme.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <label htmlFor="start-date" className="text-xs font-semibold">
                Start Date *
              </label>
              <Input
                id="start-date"
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                required
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="end-date" className="text-xs font-semibold">
                End Date *
              </label>
              <Input
                id="end-date"
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                required
              />
            </div>
          </div>

          {/* Batches Participating */}
          <div className="space-y-2">
            <label className="text-xs font-semibold">
              Participating Batches (অংশগ্রহণকারী ব্যাচসমূহ) *
            </label>
            <div className="flex flex-wrap gap-2 p-3 bg-muted/40 border border-border rounded-lg">
              {batches.length === 0 ? (
                <p className="text-xs text-muted-foreground">No active batches found.</p>
              ) : (
                batches.map((batch) => {
                  const isSelected = selectedBatchIds.includes(batch.id);
                  return (
                    <button
                      key={batch.id}
                      type="button"
                      onClick={() => toggleBatch(batch.id)}
                      className={`px-3 py-1.5 rounded-md text-xs font-medium border transition-colors ${
                        isSelected
                          ? "bg-primary text-primary-foreground border-primary"
                          : "bg-background text-foreground border-border hover:bg-muted"
                      }`}
                    >
                      {batch.name} ({batch.code})
                    </button>
                  );
                })
              )}
            </div>
          </div>

          {/* Subjects Table */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-xs font-semibold">
                Exam Subjects & Mark Rules (পরীক্ষার বিষয় ও পূর্ণমান) *
              </label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleAddSubject}
                className="gap-1.5 text-xs h-7"
              >
                <Plus className="h-3.5 w-3.5" />
                Add Subject
              </Button>
            </div>

            <div className="border border-border rounded-lg overflow-hidden">
              <table className="w-full text-xs">
                <thead className="bg-muted/60 text-muted-foreground font-medium border-b border-border">
                  <tr>
                    <th className="p-2 text-left">Subject</th>
                    <th className="p-2 text-center w-24">Full Marks</th>
                    <th className="p-2 text-center w-24">Pass Marks</th>
                    <th className="p-2 text-center w-28">Date</th>
                    <th className="p-2 text-center w-28">4th Subject?</th>
                    <th className="p-2 text-center w-12"></th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {subjects.length === 0 ? (
                    <tr>
                      <td colSpan={6} className="p-4 text-center text-muted-foreground">
                        No subjects added. Click "Add Subject" to configure exam papers.
                      </td>
                    </tr>
                  ) : (
                    subjects.map((sub, idx) => (
                      <tr key={idx} className="hover:bg-muted/30">
                        <td className="p-2">
                          <Select
                            value={sub.subject_id}
                            onValueChange={(val) =>
                              handleUpdateSubject(idx, { subject_id: val })
                            }
                          >
                            <SelectTrigger className="h-8 text-xs">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {allSubjects.map((s) => (
                                <SelectItem key={s.id} value={s.id}>
                                  {s.name} ({s.code})
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </td>
                        <td className="p-2">
                          <Input
                            type="number"
                            value={sub.full_marks / 100}
                            onChange={(e) =>
                              handleUpdateSubject(idx, {
                                full_marks: Math.round(Number(e.target.value) * 100),
                              })
                            }
                            className="h-8 text-center text-xs"
                            min={1}
                            max={1000}
                          />
                        </td>
                        <td className="p-2">
                          <Input
                            type="number"
                            value={sub.pass_marks / 100}
                            onChange={(e) =>
                              handleUpdateSubject(idx, {
                                pass_marks: Math.round(Number(e.target.value) * 100),
                              })
                            }
                            className="h-8 text-center text-xs"
                            min={0}
                            max={sub.full_marks / 100}
                          />
                        </td>
                        <td className="p-2">
                          <Input
                            type="date"
                            value={sub.exam_date || ""}
                            onChange={(e) =>
                              handleUpdateSubject(idx, { exam_date: e.target.value })
                            }
                            className="h-8 text-center text-xs"
                          />
                        </td>
                        <td className="p-2 text-center">
                          <input
                            type="checkbox"
                            checked={sub.is_optional_allowed}
                            onChange={(e) =>
                              handleUpdateSubject(idx, {
                                is_optional_allowed: e.target.checked,
                              })
                            }
                            className="rounded border-border text-primary focus:ring-primary h-4 w-4"
                            title="Check if this paper can be taken as 4th/Optional subject"
                          />
                        </td>
                        <td className="p-2 text-center">
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            onClick={() => handleRemoveSubject(idx)}
                            className="h-7 w-7 text-destructive hover:bg-destructive/10"
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </Button>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading} className="gap-1.5">
              <Calendar className="h-4 w-4" />
              {loading ? "Creating..." : "Create Examination"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
