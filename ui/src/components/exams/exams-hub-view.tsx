import { useState, useEffect } from "react";
import { rpc } from "@/gen/rpc";
import type { ExamSummary } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/ui/card";
import {
  BookOpen,
  Calendar,
  Plus,
  Edit,
  Award,
  Trash2,
  Unlock,
  Layers,
  Search,
  CheckCircle2,
  Sparkles,
} from "lucide-react";
import { CreateExamDialog } from "./create-exam-dialog";
import { MarkEntryGrid } from "./mark-entry-grid";
import { ResultPublishView } from "./result-publish-view";

type HubViewMode = "list" | "mark_entry" | "publish";

export function ExamsHubView() {
  const [viewMode, setViewMode] = useState<HubViewMode>("list");
  const [selectedExamId, setSelectedExamId] = useState<string>("");
  const [exams, setExams] = useState<ExamSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [actionMessage, setActionMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const loadExams = async () => {
    setLoading(true);
    setActionMessage(null);
    try {
      const res = await rpc("exams.list", {});
      if (res.ok && res.data) {
        setExams(res.data);
      }
    } catch (err) {
      console.error("Failed to load exams", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadExams();
  }, []);

  const handleOpenMarkEntry = (examId: string) => {
    setSelectedExamId(examId);
    setViewMode("mark_entry");
  };

  const handleOpenPublish = (examId: string) => {
    setSelectedExamId(examId);
    setViewMode("publish");
  };

  const handleDeleteExam = async (exam: ExamSummary) => {
    if (exam.status === "published") {
      alert("Published exams cannot be deleted.");
      return;
    }
    if (
      !confirm(
        `Are you sure you want to delete exam "${exam.name}"? All associated mark entries will be permanently removed.`
      )
    ) {
      return;
    }

    try {
      const res = await rpc("exams.delete", { exam_id: exam.id });
      if (res.ok) {
        setActionMessage({ type: "success", text: `Exam "${exam.name}" was deleted.` });
        await loadExams();
      } else {
        setActionMessage({
          type: "error",
          text: res.error?.message || "Failed to delete exam",
        });
      }
    } catch (err: unknown) {
      setActionMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const handleRevertExam = async (exam: ExamSummary) => {
    if (
      !confirm(
        `Revert "${exam.name}" to verified state? This will unfreeze marks for corrections. Existing published snapshots will be archived.`
      )
    ) {
      return;
    }

    try {
      const res = await rpc("exams.revert_to_verified", { exam_id: exam.id });
      if (res.ok) {
        setActionMessage({
          type: "success",
          text: `Exam "${exam.name}" reverted to verified. You may now unverify subjects to make mark adjustments.`,
        });
        await loadExams();
      } else {
        setActionMessage({
          type: "error",
          text: res.error?.message || "Failed to revert exam",
        });
      }
    } catch (err: unknown) {
      setActionMessage({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    }
  };

  // Sub-views
  if (viewMode === "mark_entry" && selectedExamId) {
    return (
      <MarkEntryGrid
        examId={selectedExamId}
        onBack={() => {
          setViewMode("list");
          void loadExams();
        }}
      />
    );
  }

  if (viewMode === "publish" && selectedExamId) {
    return (
      <ResultPublishView
        examId={selectedExamId}
        onBack={() => {
          setViewMode("list");
          void loadExams();
        }}
      />
    );
  }

  // Filtered exams
  const filteredExams = exams.filter((exam) => {
    const matchesSearch =
      exam.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (exam.name_bn && exam.name_bn.toLowerCase().includes(searchQuery.toLowerCase())) ||
      exam.session_name.toLowerCase().includes(searchQuery.toLowerCase());

    if (!matchesSearch) return false;

    if (statusFilter === "all") return true;
    if (statusFilter === "draft")
      return exam.status === "draft" || exam.status === "entry";
    if (statusFilter === "verified") return exam.status === "verified";
    if (statusFilter === "published") return exam.status === "published";
    return true;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2">
            <BookOpen className="h-6 w-6 text-primary" />
            Exams & Results Center (পরীক্ষা ও ফলাফল)
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Examination scheduling, rapid keyboard mark entry, NCTB GPA-5 calculation, and
            tabulation sheets.
          </p>
        </div>

        <Button
          onClick={() => setCreateDialogOpen(true)}
          className="gap-1.5 text-xs font-semibold h-9"
        >
          <Plus className="h-4 w-4" />
          Create Examination
        </Button>
      </div>

      {actionMessage && (
        <div
          className={`flex items-center gap-2 p-3 text-sm rounded-md border ${
            actionMessage.type === "success"
              ? "bg-emerald-500/10 text-emerald-700 border-emerald-500/20"
              : "bg-destructive/10 text-destructive border-destructive/20"
          }`}
        >
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          <span>{actionMessage.text}</span>
        </div>
      )}

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <Tabs value={statusFilter} onValueChange={setStatusFilter} className="w-auto">
          <TabsList className="grid grid-cols-4 h-9 p-1 bg-muted border border-border">
            <TabsTrigger value="all" className="text-xs font-semibold">
              All ({exams.length})
            </TabsTrigger>
            <TabsTrigger value="draft" className="text-xs font-semibold">
              In Progress (
              {exams.filter((e) => e.status === "draft" || e.status === "entry").length})
            </TabsTrigger>
            <TabsTrigger value="verified" className="text-xs font-semibold">
              Verified ({exams.filter((e) => e.status === "verified").length})
            </TabsTrigger>
            <TabsTrigger value="published" className="text-xs font-semibold">
              Published ({exams.filter((e) => e.status === "published").length})
            </TabsTrigger>
          </TabsList>
        </Tabs>

        <div className="relative w-full sm:w-72">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search exams by name..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 h-9 text-xs"
          />
        </div>
      </div>

      {/* Exam Cards Grid */}
      {loading ? (
        <div className="flex items-center justify-center p-12 border border-border rounded-xl bg-card text-muted-foreground text-sm">
          Loading examinations...
        </div>
      ) : filteredExams.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-12 border border-dashed border-border rounded-xl text-center bg-card">
          <div className="p-3 rounded-full bg-primary/10 text-primary mb-3">
            <BookOpen className="h-6 w-6" />
          </div>
          <h3 className="text-base font-semibold text-foreground">
            No Examinations Found
          </h3>
          <p className="text-xs text-muted-foreground max-w-sm mt-1 mb-4">
            {searchQuery
              ? "No exams match your search criteria. Try a different search term."
              : "Create an exam schedule to start recording subject marks and publishing report cards."}
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCreateDialogOpen(true)}
            className="gap-1.5 text-xs"
          >
            <Plus className="h-3.5 w-3.5" />
            Create First Examination
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredExams.map((exam) => {
            const isPublished = exam.status === "published";
            const isVerified = exam.status === "verified";

            return (
              <Card
                key={exam.id}
                className="flex flex-col justify-between hover:border-primary/40 transition-colors shadow-sm"
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <CardTitle className="text-base font-bold text-foreground leading-snug">
                        {exam.name}
                      </CardTitle>
                      {exam.name_bn && (
                        <CardDescription className="text-xs font-normal text-muted-foreground mt-0.5">
                          {exam.name_bn}
                        </CardDescription>
                      )}
                    </div>
                    {isPublished ? (
                      <Badge className="bg-emerald-500/15 text-emerald-700 border-emerald-500/30 text-[11px] shrink-0 font-semibold">
                        Published
                      </Badge>
                    ) : isVerified ? (
                      <Badge className="bg-indigo-500/15 text-indigo-700 border-indigo-500/30 text-[11px] shrink-0 font-semibold">
                        Verified
                      </Badge>
                    ) : (
                      <Badge className="bg-amber-500/15 text-amber-700 border-amber-500/30 text-[11px] shrink-0 font-semibold">
                        {exam.status === "entry" ? "In Entry" : "Draft"}
                      </Badge>
                    )}
                  </div>
                </CardHeader>

                <CardContent className="space-y-3 pb-3 text-xs">
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Calendar className="h-3.5 w-3.5 shrink-0" />
                    <span>
                      {exam.start_date} to {exam.end_date} &bull; {exam.session_name}
                    </span>
                  </div>

                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Sparkles className="h-3.5 w-3.5 shrink-0 text-amber-500" />
                    <span>{exam.grading_scheme_name}</span>
                  </div>

                  <div className="flex items-center gap-4 pt-1 border-t border-border/60">
                    <div className="flex items-center gap-1.5 text-muted-foreground">
                      <Layers className="h-3.5 w-3.5 text-primary" />
                      <span>{exam.batch_count} Batches</span>
                    </div>
                    <div className="flex items-center gap-1.5 text-muted-foreground">
                      <BookOpen className="h-3.5 w-3.5 text-primary" />
                      <span>{exam.subject_count} Subjects</span>
                    </div>
                  </div>
                </CardContent>

                <CardFooter className="pt-3 border-t border-border flex items-center justify-between gap-2 bg-muted/20">
                  <div className="flex items-center gap-1">
                    {!isPublished ? (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleDeleteExam(exam)}
                        className="h-8 w-8 text-destructive hover:bg-destructive/10"
                        title="Delete Exam"
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    ) : (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRevertExam(exam)}
                        className="h-8 w-8 text-amber-600 hover:bg-amber-500/10"
                        title="Reopen / Revert to Verified"
                      >
                        <Unlock className="h-3.5 w-3.5" />
                      </Button>
                    )}
                  </div>

                  <div className="flex items-center gap-1.5">
                    <Button
                      variant={isPublished ? "outline" : "primary"}
                      size="sm"
                      onClick={() => handleOpenMarkEntry(exam.id)}
                      className="gap-1 text-xs h-8"
                    >
                      <Edit className="h-3.5 w-3.5" />
                      {isPublished ? "View Marks" : "Enter Marks"}
                    </Button>

                    <Button
                      variant={isPublished ? "primary" : "outline"}
                      size="sm"
                      onClick={() => handleOpenPublish(exam.id)}
                      className="gap-1 text-xs h-8"
                    >
                      <Award className="h-3.5 w-3.5" />
                      {isPublished ? "Results & Print" : "Preview Results"}
                    </Button>
                  </div>
                </CardFooter>
              </Card>
            );
          })}
        </div>
      )}

      {/* Create Exam Dialog */}
      <CreateExamDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onExamCreated={loadExams}
      />
    </div>
  );
}
