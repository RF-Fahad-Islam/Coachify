import { useState, useEffect, useCallback } from "react";
import { rpc, type DueReminderCandidate, type BatchSummary } from "@/gen/rpc";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Bell,
  Smartphone,
  MessageSquare,
  Mail,
  AlertTriangle,
  CheckCircle2,
  RotateCcw,
  Send,
  ShieldCheck,
} from "lucide-react";
import { cn, formatTaka } from "@/lib/utils";

export function DueRemindersView() {
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>("all");

  const [selectedPeriod, setSelectedPeriod] = useState<string>(() => {
    const d = new Date();
    const yr = d.getFullYear();
    const mo = String(d.getMonth() + 1).padStart(2, "0");
    return `${yr}-${mo}`;
  });

  const [minDueTaka, setMinDueTaka] = useState<number>(500);

  const [candidates, setCandidates] = useState<DueReminderCandidate[]>([]);
  const [selectedStudentIds, setSelectedStudentIds] = useState<Set<string>>(new Set());

  const [dispatchChannel, setDispatchChannel] = useState<"sms" | "whatsapp" | "email">(
    "sms"
  );
  const [customBody, setCustomBody] = useState<string>("");

  const [loading, setLoading] = useState(false);
  const [dispatching, setDispatching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  // Load batches
  useEffect(() => {
    let mounted = true;
    async function loadBatches() {
      try {
        const res = await rpc("batches.list", { status: "active" });
        if (mounted && res.ok && res.data) {
          setBatches(res.data);
        }
      } catch (err: unknown) {
        if (mounted) {
          console.error("Failed to load batches:", err);
        }
      }
    }
    void loadBatches();
    return () => {
      mounted = false;
    };
  }, []);

  // Load candidates
  const loadCandidates = useCallback(async () => {
    setLoading(true);
    setError(null);
    setFeedback(null);

    try {
      const res = await rpc("comms.get_due_reminder_candidates", {
        batch_id: selectedBatchId === "all" ? null : selectedBatchId,
        period: selectedPeriod || null,
        min_due_poisha: minDueTaka > 0 ? minDueTaka * 100 : null,
      });

      if (res.ok && res.data) {
        setCandidates(res.data);
        // By default, select all candidates who have not yet been reminded on the selected channel
        const unreminded = new Set<string>();
        for (const c of res.data) {
          const already =
            dispatchChannel === "sms"
              ? c.already_reminded_sms
              : dispatchChannel === "whatsapp"
                ? c.already_reminded_whatsapp
                : c.already_reminded_email;
          if (!already) {
            unreminded.add(c.student_id);
          }
        }
        setSelectedStudentIds(unreminded);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(
        err instanceof Error ? err.message : "Failed to load due reminder candidates"
      );
    } finally {
      setLoading(false);
    }
  }, [selectedBatchId, selectedPeriod, minDueTaka, dispatchChannel]);

  useEffect(() => {
    void loadCandidates();
  }, [loadCandidates]);

  // Toggle selection
  const toggleStudent = (id: string) => {
    setSelectedStudentIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const toggleAll = () => {
    if (selectedStudentIds.size === candidates.length) {
      setSelectedStudentIds(new Set());
    } else {
      setSelectedStudentIds(new Set(candidates.map((c) => c.student_id)));
    }
  };

  // Enqueue reminders
  const handleEnqueue = async () => {
    if (selectedStudentIds.size === 0) return;
    setDispatching(true);
    setError(null);
    setFeedback(null);

    try {
      const res = await rpc("comms.enqueue_due_reminders", {
        student_ids: Array.from(selectedStudentIds),
        channel: dispatchChannel,
        period: selectedPeriod,
        custom_body: customBody.trim() ? customBody.trim() : null,
      });

      if (res.ok && res.data) {
        setFeedback(
          `Successfully enqueued ${res.data.enqueued_count} reminder(s). ${res.data.skipped_count} skipped due to existing reminders.`
        );
        void loadCandidates();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to enqueue due reminders");
    } finally {
      setDispatching(false);
    }
  };

  const selectedCount = selectedStudentIds.size;
  const totalDuePoisha = candidates.reduce((sum, c) => sum + c.due_amount_poisha, 0);

  return (
    <div className="space-y-6">
      {/* Filter Bar */}
      <Card className="p-4 bg-card/60 backdrop-blur-xs border-border/70 shadow-xs">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <label
                htmlFor="filter-batch"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Batch:
              </label>
              <select
                id="filter-batch"
                value={selectedBatchId}
                onChange={(e) => setSelectedBatchId(e.target.value)}
                className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
              >
                <option value="all">All Batches</option>
                {batches.map((b) => (
                  <option key={b.id} value={b.id}>
                    {b.name} ({b.code})
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-2">
              <label
                htmlFor="filter-period"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Period:
              </label>
              <Input
                id="filter-period"
                type="month"
                value={selectedPeriod}
                onChange={(e) => setSelectedPeriod(e.target.value)}
                className="h-9 w-36 text-xs font-medium"
              />
            </div>

            <div className="flex items-center gap-2">
              <label
                htmlFor="filter-min-due"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Min Due (৳):
              </label>
              <Input
                id="filter-min-due"
                type="number"
                min={0}
                step={100}
                value={minDueTaka}
                onChange={(e) => setMinDueTaka(Number(e.target.value))}
                className="h-9 w-28 text-xs font-medium"
              />
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => void loadCandidates()}
              disabled={loading}
              className="h-9 text-xs gap-1.5"
            >
              <RotateCcw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
              Refresh Candidates
            </Button>
          </div>

          <div className="flex items-center gap-3 text-xs">
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-muted/60 border border-border">
              <span className="text-muted-foreground">Candidates:</span>
              <span className="font-bold text-foreground">{candidates.length}</span>
            </div>
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-rose-500/10 border border-rose-500/20 text-rose-700 dark:text-rose-400 font-semibold">
              <span>Total Overdue:</span>
              <span className="font-bold">{formatTaka(totalDuePoisha)}</span>
            </div>
          </div>
        </div>
      </Card>

      {/* Dispatch Action Panel */}
      <Card className="p-4 border-border/80 shadow-xs space-y-4">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex items-center gap-3">
            <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
              <Bell className="h-4 w-4 text-primary" />
              Dispatch Channel:
            </span>
            <div className="flex items-center gap-1.5 p-1 rounded-lg bg-muted border border-border">
              <button
                type="button"
                onClick={() => setDispatchChannel("sms")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  dispatchChannel === "sms"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Smartphone className="h-3.5 w-3.5" />
                SMS
              </button>
              <button
                type="button"
                onClick={() => setDispatchChannel("whatsapp")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  dispatchChannel === "whatsapp"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <MessageSquare className="h-3.5 w-3.5" />
                WhatsApp
              </button>
              <button
                type="button"
                onClick={() => setDispatchChannel("email")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  dispatchChannel === "email"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Mail className="h-3.5 w-3.5" />
                Email
              </button>
            </div>
          </div>

          <div className="flex-1 max-w-sm">
            <Input
              value={customBody}
              onChange={(e) => setCustomBody(e.target.value)}
              placeholder="Optional custom reminder note..."
              className="h-8 text-xs"
            />
          </div>

          <div className="flex items-center gap-3">
            <span className="text-xs text-muted-foreground font-medium">
              {selectedCount} candidate(s) selected
            </span>
            <Button
              variant="primary"
              size="sm"
              onClick={handleEnqueue}
              disabled={dispatching || selectedCount === 0}
              className="gap-2 font-semibold text-xs bg-primary text-primary-foreground shadow-xs"
            >
              <Send className="h-3.5 w-3.5" />
              {dispatching ? "Scheduling..." : "Enqueue Due Reminders"}
            </Button>
          </div>
        </div>

        {/* Deduplication Guarantee Note */}
        <div className="p-3 rounded-lg bg-muted/40 border border-border/70 flex items-center gap-2.5 text-xs text-muted-foreground">
          <ShieldCheck className="h-4 w-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
          <span>
            <strong>Period Deduplication Guard:</strong> Each candidate receives at most
            one reminder per channel per monthly billing period ({selectedPeriod}).
            Re-running skips already reminded guardians automatically.
          </span>
        </div>
      </Card>

      {/* Feedback & Error */}
      {error && (
        <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}
      {feedback && (
        <div className="p-3 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          <span>{feedback}</span>
        </div>
      )}

      {/* Candidates Table */}
      <Card className="overflow-hidden border-border/80 shadow-xs">
        <CardHeader className="py-3 px-4 bg-muted/20 border-b border-border/70 flex flex-row items-center justify-between">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <Bell className="h-4 w-4 text-primary" />
            Candidate Review List ({candidates.length})
          </CardTitle>
          <div className="text-xs text-muted-foreground">
            {selectedCount} selected for dispatch
          </div>
        </CardHeader>

        <div className="overflow-x-auto max-h-[520px]">
          <table className="w-full text-xs text-left border-collapse">
            <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
              <tr>
                <th className="py-2.5 px-3 w-10 text-center">
                  <input
                    type="checkbox"
                    checked={candidates.length > 0 && selectedCount === candidates.length}
                    onChange={toggleAll}
                    className="rounded border-input text-primary focus:ring-ring h-4 w-4"
                  />
                </th>
                <th className="py-2.5 px-3 w-16">Roll</th>
                <th className="py-2.5 px-3">Student Name</th>
                <th className="py-2.5 px-3 w-28">Student Code</th>
                <th className="py-2.5 px-3">Batch</th>
                <th className="py-2.5 px-3 w-32 text-right">Overdue Balance</th>
                <th className="py-2.5 px-3">Guardian Contact</th>
                <th className="py-2.5 px-3 w-36 text-center">Reminded Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {candidates.map((c) => {
                const isSelected = selectedStudentIds.has(c.student_id);

                return (
                  <tr
                    key={c.student_id}
                    onClick={() => toggleStudent(c.student_id)}
                    className={cn(
                      "hover:bg-muted/30 transition-colors cursor-pointer",
                      isSelected ? "bg-primary/5 font-medium" : "opacity-75"
                    )}
                  >
                    <td
                      className="py-2.5 px-3 text-center"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleStudent(c.student_id)}
                        className="rounded border-input text-primary focus:ring-ring h-4 w-4"
                      />
                    </td>
                    <td className="py-2.5 px-3 font-mono font-semibold text-foreground">
                      {c.roll ?? "-"}
                    </td>
                    <td className="py-2.5 px-3 text-foreground">{c.student_name}</td>
                    <td className="py-2.5 px-3 font-mono text-muted-foreground">
                      {c.student_code}
                    </td>
                    <td className="py-2.5 px-3 text-muted-foreground">{c.batch_name}</td>
                    <td className="py-2.5 px-3 text-right font-mono font-bold text-rose-600 dark:text-rose-400">
                      {formatTaka(c.due_amount_poisha)}
                    </td>
                    <td className="py-2.5 px-3 text-muted-foreground">
                      <div>{c.guardian_name || "Guardian"}</div>
                      <div className="font-mono text-[11px] text-foreground">
                        {c.guardian_phone || c.guardian_email || "No contact info"}
                      </div>
                    </td>
                    <td className="py-2.5 px-3 text-center">
                      <div className="flex items-center justify-center gap-1">
                        <span
                          className={cn(
                            "px-1.5 py-0.5 rounded text-[10px] font-semibold",
                            c.already_reminded_sms
                              ? "bg-blue-500/20 text-blue-700 dark:text-blue-300"
                              : "bg-muted text-muted-foreground/40"
                          )}
                          title={c.already_reminded_sms ? "SMS Sent" : "SMS Not Sent"}
                        >
                          SMS
                        </span>
                        <span
                          className={cn(
                            "px-1.5 py-0.5 rounded text-[10px] font-semibold",
                            c.already_reminded_whatsapp
                              ? "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300"
                              : "bg-muted text-muted-foreground/40"
                          )}
                          title={
                            c.already_reminded_whatsapp
                              ? "WhatsApp Sent"
                              : "WhatsApp Not Sent"
                          }
                        >
                          WA
                        </span>
                        <span
                          className={cn(
                            "px-1.5 py-0.5 rounded text-[10px] font-semibold",
                            c.already_reminded_email
                              ? "bg-purple-500/20 text-purple-700 dark:text-purple-300"
                              : "bg-muted text-muted-foreground/40"
                          )}
                          title={
                            c.already_reminded_email ? "Email Sent" : "Email Not Sent"
                          }
                        >
                          Email
                        </span>
                      </div>
                    </td>
                  </tr>
                );
              })}

              {candidates.length === 0 && !loading && (
                <tr>
                  <td
                    colSpan={8}
                    className="py-12 text-center text-muted-foreground text-xs"
                  >
                    No overdue candidates found for the selected filters
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
