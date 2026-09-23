import { useState, useEffect, useCallback } from "react";
import { rpc, type Student360Response, type TimelineEvent } from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Clock,
  CreditCard,
  CalendarCheck,
  GraduationCap,
  MessageSquare,
  School,
  FileText,
  Plus,
  RefreshCw,
  Loader2,
  User,
} from "lucide-react";

export interface Student360TimelineProps {
  studentId: string;
}

export type StudentTimelineCategory =
  "all" | "academic" | "attendance" | "finance" | "comms" | "exams" | "notes";

const CATEGORIES: { id: StudentTimelineCategory; label: string }[] = [
  { id: "all", label: "Full Journey" },
  { id: "finance", label: "Payments & Dues" },
  { id: "attendance", label: "Attendance" },
  { id: "exams", label: "Exams & Results" },
  { id: "academic", label: "Enrollments" },
  { id: "comms", label: "SMS & Outbox" },
  { id: "notes", label: "Teacher Notes" },
];

export function Student360Timeline({ studentId }: Student360TimelineProps) {
  const [category, setCategory] = useState<StudentTimelineCategory>("all");
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [timelineData, setTimelineData] = useState<Student360Response | null>(null);
  const [events, setEvents] = useState<TimelineEvent[]>([]);

  // Add Note Modal
  const [noteModalOpen, setNoteModalOpen] = useState(false);
  const [noteType, setNoteType] = useState<string>("behavioral");
  const [noteTitle, setNoteTitle] = useState("");
  const [noteContent, setNoteContent] = useState("");
  const [savingNote, setSavingNote] = useState(false);
  const [noteError, setNoteError] = useState<string | null>(null);

  const loadTimeline = useCallback(
    async (cat: StudentTimelineCategory) => {
      if (!studentId) return;
      setLoading(true);
      setError(null);
      try {
        const res = await rpc("intelligence.get_student_360", {
          student_id: studentId,
          category: cat === "all" ? null : cat,
          limit: 50,
        });

        if (res.ok && res.data) {
          setTimelineData(res.data);
          setEvents(res.data.events);
        } else {
          setError(res.error?.message || "Failed to load student 360 journey");
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    },
    [studentId]
  );

  useEffect(() => {
    void loadTimeline(category);
  }, [category, loadTimeline]);

  const handleLoadMore = async () => {
    if (!timelineData?.next_cursor || loadingMore) return;
    setLoadingMore(true);
    try {
      const res = await rpc("intelligence.get_student_360", {
        student_id: studentId,
        category: category === "all" ? null : category,
        limit: 50,
        before_time: timelineData.next_cursor.before_time,
        before_id: timelineData.next_cursor.before_id,
      });

      if (res.ok && res.data) {
        const nextData = res.data;
        setTimelineData(nextData);
        setEvents((prev) => [...prev, ...nextData.events]);
      }
    } catch (err: unknown) {
      console.error("Failed to load more student timeline events", err);
    } finally {
      setLoadingMore(false);
    }
  };

  const handleSaveNote = async () => {
    if (!noteTitle.trim() || !noteContent.trim()) {
      setNoteError("Title and content are required.");
      return;
    }

    setSavingNote(true);
    setNoteError(null);
    try {
      const res = await rpc("intelligence.add_student_note", {
        student_id: studentId,
        note_type: noteType,
        title: noteTitle.trim(),
        content: noteContent.trim(),
      });

      if (res.ok) {
        setNoteModalOpen(false);
        setNoteTitle("");
        setNoteContent("");
        void loadTimeline(category);
      } else {
        setNoteError(res.error?.message || "Failed to save student note");
      }
    } catch (err: unknown) {
      setNoteError(err instanceof Error ? err.message : String(err));
    } finally {
      setSavingNote(false);
    }
  };

  const getCategoryIcon = (cat: string) => {
    switch (cat.toLowerCase()) {
      case "finance":
      case "payment":
        return <CreditCard className="h-3.5 w-3.5 text-emerald-600" />;
      case "attendance":
        return <CalendarCheck className="h-3.5 w-3.5 text-purple-600" />;
      case "exams":
      case "exam":
        return <GraduationCap className="h-3.5 w-3.5 text-amber-600" />;
      case "academic":
      case "admissions":
        return <School className="h-3.5 w-3.5 text-blue-600" />;
      case "comms":
      case "message":
        return <MessageSquare className="h-3.5 w-3.5 text-indigo-600" />;
      case "notes":
      case "note":
        return <FileText className="h-3.5 w-3.5 text-rose-600" />;
      default:
        return <Clock className="h-3.5 w-3.5 text-muted-foreground" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Student 360 Header KPI Strip */}
      {timelineData && (
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div className="p-3.5 rounded-[var(--radius-lg)] border border-border/80 bg-card shadow-2xs">
            <span className="text-xs text-muted-foreground block font-medium">
              Total Paid
            </span>
            <div className="text-xl font-bold text-foreground font-mono mt-0.5">
              {formatTaka(timelineData.total_paid_poisha)}
            </div>
            <span className="text-[11px] text-muted-foreground">
              Historical fee counter payments
            </span>
          </div>

          <div className="p-3.5 rounded-[var(--radius-lg)] border border-border/80 bg-card shadow-2xs">
            <span className="text-xs text-muted-foreground block font-medium">
              Current Dues
            </span>
            <div className="text-xl font-bold text-destructive font-mono mt-0.5">
              {formatTaka(timelineData.total_due_poisha)}
            </div>
            <span className="text-[11px] text-muted-foreground">Outstanding charges</span>
          </div>

          <div className="p-3.5 rounded-[var(--radius-lg)] border border-border/80 bg-card shadow-2xs">
            <span className="text-xs text-muted-foreground block font-medium">
              Exam Performance
            </span>
            <div className="text-xl font-bold text-foreground font-mono mt-0.5">
              {timelineData.gpa_average !== null && timelineData.gpa_average !== undefined
                ? `GPA ${timelineData.gpa_average.toFixed(2)}`
                : "No exams evaluated"}
            </div>
            <span className="text-[11px] text-muted-foreground">
              Cumulative GPA across published terms
            </span>
          </div>
        </div>
      )}

      {/* Control Bar: Categories & Add Note */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-border pb-3">
        {/* Category Pills */}
        <div className="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-thin">
          {CATEGORIES.map((cat) => {
            const isActive = category === cat.id;
            return (
              <button
                key={cat.id}
                type="button"
                onClick={() => setCategory(cat.id)}
                className={`px-3 py-1 rounded-[var(--radius-md)] text-xs font-semibold whitespace-nowrap transition-colors cursor-pointer border ${
                  isActive
                    ? "bg-primary text-primary-foreground border-primary shadow-2xs"
                    : "bg-card text-muted-foreground border-border hover:text-foreground hover:bg-muted/60"
                }`}
              >
                {cat.label}
              </button>
            );
          })}
        </div>

        <div className="flex items-center gap-2 self-end sm:self-center">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setNoteModalOpen(true)}
            className="text-xs font-semibold gap-1.5 h-8 text-primary border-primary/30 hover:bg-primary-subtle"
          >
            <Plus className="h-3.5 w-3.5" />
            Add Note
          </Button>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadTimeline(category)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh student timeline"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-3 text-xs rounded bg-destructive/10 text-destructive border border-destructive/20">
          {error}
        </div>
      )}

      {/* Timeline Stream */}
      {loading ? (
        <div className="py-16 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Stitching Student 360 life cycle events...
          </span>
        </div>
      ) : events.length === 0 ? (
        <Card className="border-dashed border-2 border-border">
          <CardContent className="py-12 text-center space-y-2">
            <Clock className="h-8 w-8 mx-auto text-muted-foreground/60" />
            <h4 className="text-sm font-bold text-foreground">
              No events recorded in this category
            </h4>
            <p className="text-xs text-muted-foreground max-w-xs mx-auto">
              Select &quot;Full Journey&quot; to see all admissions, attendances, fee
              receipts, and marks.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          <div className="relative pl-6 sm:pl-8 before:absolute before:left-3 sm:before:left-4 before:top-2 before:bottom-2 before:w-0.5 before:bg-border">
            <div className="space-y-3.5">
              {events.map((event, idx) => {
                const datePart = event.event_time.slice(0, 10);
                const timePart = event.event_time.slice(11, 16);

                return (
                  <div
                    key={event.id || `${event.event_time}-${idx}`}
                    className="relative group"
                  >
                    {/* Node Dot */}
                    <div className="absolute -left-[27px] sm:-left-[31px] top-1 h-5 w-5 rounded-full bg-card border-2 border-primary flex items-center justify-center shadow-2xs group-hover:scale-110 transition-transform">
                      {getCategoryIcon(event.category)}
                    </div>

                    {/* Event Box */}
                    <div className="p-3.5 rounded-[var(--radius-lg)] border border-border/80 bg-card hover:border-border transition-all space-y-1.5 shadow-2xs">
                      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="font-bold text-xs sm:text-sm text-foreground">
                            {event.title}
                          </span>
                          <Badge
                            variant="outline"
                            size="sm"
                            className="capitalize text-[10px]"
                          >
                            {event.category}
                          </Badge>
                        </div>
                        <span className="text-[11px] font-mono text-muted-foreground">
                          {formatBusinessDate(datePart)} {timePart && `• ${timePart}`}
                        </span>
                      </div>

                      <p className="text-xs text-foreground/80 leading-relaxed">
                        {event.description}
                      </p>

                      <div className="flex items-center justify-between pt-1 border-t border-border/40 text-[10px] text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <User className="h-3 w-3" />
                          <span>
                            {event.actor_name ? `By ${event.actor_name}` : "System event"}
                          </span>
                        </div>
                        <span className="font-mono">{event.event_type}</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {timelineData?.has_more && (
            <div className="text-center pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleLoadMore}
                disabled={loadingMore}
                className="text-xs font-semibold gap-1.5"
              >
                {loadingMore ? (
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <Clock className="h-3.5 w-3.5" />
                )}
                Load Earlier Journey Events
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Add Student Note Modal */}
      <Dialog open={noteModalOpen} onOpenChange={setNoteModalOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base font-bold flex items-center gap-2">
              <FileText className="h-4 w-4 text-primary" />
              Add Student Journey Note
            </DialogTitle>
          </DialogHeader>

          {noteError && (
            <div className="p-2.5 rounded bg-destructive/10 text-destructive text-xs">
              {noteError}
            </div>
          )}

          <div className="space-y-3 py-2 text-xs">
            <div className="space-y-1">
              <label className="font-semibold text-foreground">Note Category</label>
              <select
                value={noteType}
                onChange={(e) => setNoteType(e.target.value)}
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground focus:outline-hidden cursor-pointer"
              >
                <option value="behavioral">Behavioral Observation</option>
                <option value="academic">Academic Feedback</option>
                <option value="guardian_meeting">Guardian Meeting / Counseling</option>
                <option value="medical">Health / Medical Note</option>
                <option value="administrative">Administrative Notice</option>
                <option value="general">General Remarks</option>
              </select>
            </div>

            <div className="space-y-1">
              <label className="font-semibold text-foreground">
                Note Subject / Title
              </label>
              <input
                type="text"
                value={noteTitle}
                onChange={(e) => setNoteTitle(e.target.value)}
                placeholder="e.g. Discussed irregular attendance with mother"
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground focus:outline-hidden"
              />
            </div>

            <div className="space-y-1">
              <label className="font-semibold text-foreground">
                Detailed Observation / Plan
              </label>
              <textarea
                value={noteContent}
                onChange={(e) => setNoteContent(e.target.value)}
                rows={4}
                placeholder="Enter counseling summary, teacher observation, or follow-up plan..."
                className="w-full px-3 py-2 rounded-[var(--radius-md)] border border-border bg-card text-foreground focus:outline-hidden"
              />
            </div>
          </div>

          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setNoteModalOpen(false)}
              disabled={savingNote}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveNote}
              disabled={savingNote}
              className="gap-1.5"
            >
              {savingNote ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
              Save Note
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
