import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type BatchSummary,
  type MessageTemplateItem,
  type TextAnalysisResponse,
  type PreviewBatchBroadcastResponse,
} from "@/gen/rpc";
import { Card, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Radio,
  Send,
  Smartphone,
  MessageSquare,
  Mail,
  AlertTriangle,
  CheckCircle2,
  Users,
  Eye,
  Calculator,
} from "lucide-react";
import { cn } from "@/lib/utils";

const PLACEHOLDERS = [
  { key: "{student_name}", label: "Student Name" },
  { key: "{student_code}", label: "Student Code" },
  { key: "{roll}", label: "Roll" },
  { key: "{batch_name}", label: "Batch Name" },
  { key: "{program_name}", label: "Program Name" },
  { key: "{institution_name}", label: "Institution" },
  { key: "{institution_phone}", label: "Helpline" },
];

export function BatchBroadcastView() {
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>("");

  const [channel, setChannel] = useState<"sms" | "whatsapp" | "email">("sms");
  const [templates, setTemplates] = useState<MessageTemplateItem[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>("custom");

  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");

  const [loading, setLoading] = useState(false);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  // Live text analysis
  const [analysis, setAnalysis] = useState<TextAnalysisResponse | null>(null);

  // Recipient preview
  const [preview, setPreview] = useState<PreviewBatchBroadcastResponse | null>(null);
  const [selectedStudentIds, setSelectedStudentIds] = useState<Set<string>>(new Set());

  // Confirmation modal
  const [confirmModalOpen, setConfirmModalOpen] = useState(false);

  // Load active batches
  useEffect(() => {
    let mounted = true;
    async function loadBatches() {
      try {
        const res = await rpc("batches.list", { status: "active" });
        if (mounted && res.ok && res.data) {
          setBatches(res.data);
          if (res.data.length > 0 && !selectedBatchId) {
            setSelectedBatchId(res.data[0].id);
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
  }, [selectedBatchId]);

  // Load templates for channel
  useEffect(() => {
    let mounted = true;
    async function loadTemplates() {
      try {
        const res = await rpc("comms.list_templates", { channel });
        if (mounted && res.ok && res.data) {
          setTemplates(res.data);
        }
      } catch (err: unknown) {
        if (mounted) {
          console.error("Failed to load templates:", err);
        }
      }
    }
    void loadTemplates();
    return () => {
      mounted = false;
    };
  }, [channel]);

  // Handle template selection
  const handleTemplateChange = (tmplId: string) => {
    setSelectedTemplateId(tmplId);
    if (tmplId === "custom") {
      setBody("");
      setSubject("");
    } else {
      const tmpl = templates.find((t) => t.id === tmplId);
      if (tmpl) {
        setBody(tmpl.body);
        if (tmpl.subject) setSubject(tmpl.subject);
      }
    }
  };

  // Text analysis debounce
  useEffect(() => {
    if (!body.trim()) {
      setAnalysis(null);
      return;
    }

    const timer = setTimeout(async () => {
      try {
        const res = await rpc("comms.analyze_text", {
          text: body,
          channel,
        });
        if (res.ok && res.data) {
          setAnalysis(res.data);
        }
      } catch (e: unknown) {
        console.error("Failed to analyze text:", e);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [body, channel]);

  // Load recipient preview
  const loadPreview = useCallback(async () => {
    if (!selectedBatchId || !body.trim()) return;
    setLoading(true);
    setError(null);

    try {
      const res = await rpc("comms.preview_batch_broadcast", {
        batch_id: selectedBatchId,
        channel,
        custom_subject: channel === "email" ? subject : null,
        custom_body: body,
      });

      if (res.ok && res.data) {
        setPreview(res.data);
        // Default select all ready recipients
        const allReadyIds = new Set(res.data.ready_recipients.map((r) => r.student_id));
        setSelectedStudentIds(allReadyIds);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load broadcast preview");
    } finally {
      setLoading(false);
    }
  }, [selectedBatchId, channel, subject, body]);

  useEffect(() => {
    if (selectedBatchId && body.trim()) {
      void loadPreview();
    }
  }, [selectedBatchId, channel, body, loadPreview]);

  // Insert placeholder at current position
  const handleInsertPlaceholder = (placeholder: string) => {
    setBody((prev) => `${prev} ${placeholder} `);
  };

  // Toggle recipient selection
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

  // Toggle all ready students
  const toggleAllReady = () => {
    if (!preview) return;
    if (selectedStudentIds.size === preview.ready_recipients.length) {
      setSelectedStudentIds(new Set());
    } else {
      setSelectedStudentIds(new Set(preview.ready_recipients.map((r) => r.student_id)));
    }
  };

  // Execute broadcast send
  const executeSend = async () => {
    if (!selectedBatchId || selectedStudentIds.size === 0) return;
    setSending(true);
    setError(null);
    setFeedback(null);

    try {
      const res = await rpc("comms.send_batch_broadcast", {
        batch_id: selectedBatchId,
        channel,
        recipient_student_ids: Array.from(selectedStudentIds),
        custom_subject: channel === "email" ? subject : null,
        custom_body: body,
      });

      if (res.ok && res.data) {
        setFeedback(
          `Broadcast successfully scheduled. Enqueued ${res.data.enqueued_count} message(s) to outbox.`
        );
        setConfirmModalOpen(false);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to dispatch batch broadcast");
    } finally {
      setSending(false);
    }
  };

  const selectedBatch = batches.find((b) => b.id === selectedBatchId);
  const readyCount = preview?.ready_recipients.length ?? 0;
  const missingCount = preview?.missing_recipients.length ?? 0;
  const selectedCount = selectedStudentIds.size;
  const totalEstimatedCost = (preview?.segments_per_message ?? 1) * selectedCount;

  return (
    <div className="space-y-6">
      {/* Top Configuration Card */}
      <Card className="p-4 bg-card/60 backdrop-blur-xs border-border/70 shadow-xs">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <label
                htmlFor="broadcast-batch"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Target Batch:
              </label>
              <select
                id="broadcast-batch"
                value={selectedBatchId}
                onChange={(e) => setSelectedBatchId(e.target.value)}
                className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
              >
                {batches.map((b) => (
                  <option key={b.id} value={b.id}>
                    {b.name} ({b.code})
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-1.5 p-1 rounded-lg bg-muted border border-border">
              <button
                type="button"
                onClick={() => setChannel("sms")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  channel === "sms"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Smartphone className="h-3.5 w-3.5" />
                SMS
              </button>
              <button
                type="button"
                onClick={() => setChannel("whatsapp")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  channel === "whatsapp"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <MessageSquare className="h-3.5 w-3.5" />
                WhatsApp
              </button>
              <button
                type="button"
                onClick={() => setChannel("email")}
                className={cn(
                  "h-7 px-2.5 rounded text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5",
                  channel === "email"
                    ? "bg-primary text-primary-foreground shadow-xs"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <Mail className="h-3.5 w-3.5" />
                Email
              </button>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <label
              htmlFor="broadcast-template"
              className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
            >
              Template:
            </label>
            <select
              id="broadcast-template"
              value={selectedTemplateId}
              onChange={(e) => handleTemplateChange(e.target.value)}
              className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
            >
              <option value="custom">Custom Composition</option>
              {templates.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.title} ({t.language.toUpperCase()})
                </option>
              ))}
            </select>
          </div>
        </div>
      </Card>

      {/* Composition and Live Analysis Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left 2 Cols: Message Composer */}
        <div className="lg:col-span-2 space-y-4">
          <Card className="p-4 border-border/80 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-semibold text-foreground flex items-center gap-2">
                <Radio className="h-4 w-4 text-primary" />
                Compose Broadcast Message
              </span>
              <span className="text-xs text-muted-foreground">
                Channel: <strong className="uppercase">{channel}</strong>
              </span>
            </div>

            {channel === "email" && (
              <div className="space-y-1.5">
                <label
                  htmlFor="email-subject"
                  className="text-xs font-semibold text-foreground"
                >
                  Email Subject Line <span className="text-destructive">*</span>:
                </label>
                <Input
                  id="email-subject"
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  placeholder="E.g., Important Examination Notice for Class 10 Batch"
                  className="text-xs"
                />
              </div>
            )}

            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <label
                  htmlFor="broadcast-body"
                  className="text-xs font-semibold text-foreground"
                >
                  Message Body:
                </label>
                <div className="text-[11px] text-muted-foreground">
                  Insert dynamic placeholders below
                </div>
              </div>
              <textarea
                id="broadcast-body"
                rows={6}
                value={body}
                onChange={(e) => setBody(e.target.value)}
                placeholder="Type your announcement here. Click any placeholder badge below to insert student-specific data."
                className="w-full p-3 rounded-md border border-input bg-background font-mono text-xs focus:outline-hidden focus:ring-1 focus:ring-ring leading-relaxed"
              />
            </div>

            {/* Placeholder Pills */}
            <div className="space-y-1.5">
              <span className="text-[11px] font-semibold text-muted-foreground block">
                Available Placeholders:
              </span>
              <div className="flex flex-wrap gap-1.5">
                {PLACEHOLDERS.map((p) => (
                  <button
                    key={p.key}
                    type="button"
                    onClick={() => handleInsertPlaceholder(p.key)}
                    className="px-2 py-1 rounded bg-muted hover:bg-primary-subtle hover:text-primary border border-border text-[11px] font-mono transition-colors cursor-pointer"
                    title={`Click to insert ${p.label}`}
                  >
                    {p.key}
                  </button>
                ))}
              </div>
            </div>
          </Card>

          {/* Sample Interpolated Live Preview */}
          {preview?.sample_interpolated_body && (
            <Card className="p-4 border-border/70 bg-muted/20 shadow-xs space-y-2">
              <div className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <Eye className="h-3.5 w-3.5 text-primary" />
                Sample Student Preview (First Active Recipient):
              </div>
              <div className="p-3 rounded bg-background border border-border text-xs font-mono whitespace-pre-wrap leading-relaxed">
                {preview.sample_interpolated_body}
              </div>
            </Card>
          )}
        </div>

        {/* Right Col: Live Segment Math & Dispatch Card */}
        <div className="space-y-4">
          <Card className="p-4 border-border/80 shadow-xs space-y-4">
            <div className="text-sm font-semibold text-foreground flex items-center gap-2">
              <Calculator className="h-4 w-4 text-primary" />
              Live Segment & Encoding Math
            </div>

            {channel === "sms" ? (
              <div className="space-y-3 text-xs">
                <div className="flex items-center justify-between p-2.5 rounded bg-muted/50 border border-border">
                  <span className="text-muted-foreground">Encoding Format:</span>
                  <Badge
                    variant={analysis?.encoding === "GSM 7-bit" ? "paid" : "pending"}
                    className="font-mono text-[10px]"
                  >
                    {analysis?.encoding || "GSM 7-bit"}
                  </Badge>
                </div>

                <div className="space-y-1 p-2.5 rounded bg-muted/50 border border-border">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Total Characters:</span>
                    <span className="font-bold text-foreground font-mono">
                      {analysis?.char_count ?? 0}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Units per Segment:</span>
                    <span className="font-mono text-foreground">
                      {analysis?.units_per_segment ?? 160}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Remaining in Segment:</span>
                    <span className="font-mono text-emerald-600 dark:text-emerald-400 font-bold">
                      {analysis?.units_remaining_in_segment ?? 160}
                    </span>
                  </div>
                </div>

                <div className="p-3 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-between">
                  <span className="font-semibold text-primary">
                    Segments per Message:
                  </span>
                  <span className="text-lg font-bold text-primary font-mono">
                    {analysis?.segment_count ?? 1}
                  </span>
                </div>
              </div>
            ) : (
              <div className="space-y-2 text-xs">
                <div className="p-2.5 rounded bg-muted/50 border border-border flex items-center justify-between">
                  <span className="text-muted-foreground">Character Count:</span>
                  <span className="font-mono font-bold text-foreground">
                    {body.length}
                  </span>
                </div>
                <p className="text-[11px] text-muted-foreground leading-relaxed">
                  WhatsApp and Email channels support rich content and formatting without
                  cellular SMS segment limits.
                </p>
              </div>
            )}

            <div className="border-t border-border pt-3 space-y-2 text-xs">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Selected Recipients:</span>
                <span className="font-bold text-foreground">{selectedCount}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Est. Total SMS Units:</span>
                <span className="font-bold font-mono text-primary">
                  {totalEstimatedCost}
                </span>
              </div>
            </div>

            <Button
              variant="primary"
              size="sm"
              onClick={() => setConfirmModalOpen(true)}
              disabled={
                selectedCount === 0 ||
                !body.trim() ||
                (channel === "email" && !subject.trim())
              }
              className="w-full gap-2 font-semibold text-xs py-2 bg-primary text-primary-foreground shadow-xs"
            >
              <Send className="h-3.5 w-3.5" />
              Review & Send Broadcast
            </Button>
          </Card>
        </div>
      </div>

      {/* Feedback & Errors */}
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

      {/* Recipient Preview Tabs */}
      <Card className="overflow-hidden border-border/80 shadow-xs">
        <Tabs defaultValue="ready" className="w-full">
          <CardHeader className="py-2.5 px-4 bg-muted/20 border-b border-border/70 flex flex-row items-center justify-between">
            <TabsList className="h-8 p-0.5 bg-muted">
              <TabsTrigger value="ready" className="text-xs px-3 gap-1.5">
                <Users className="h-3.5 w-3.5 text-emerald-500" />
                Ready to Dispatch ({readyCount})
              </TabsTrigger>
              <TabsTrigger value="missing" className="text-xs px-3 gap-1.5">
                <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />
                Missing Contact Info ({missingCount})
              </TabsTrigger>
            </TabsList>
            <div className="text-xs text-muted-foreground">
              {selectedCount} of {readyCount} selected
            </div>
          </CardHeader>

          {/* Ready Tab */}
          <TabsContent value="ready" className="m-0">
            <div className="overflow-x-auto max-h-[380px]">
              <table className="w-full text-xs text-left border-collapse">
                <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
                  <tr>
                    <th className="py-2.5 px-3 w-10 text-center">
                      <input
                        type="checkbox"
                        checked={readyCount > 0 && selectedCount === readyCount}
                        onChange={toggleAllReady}
                        className="rounded border-input text-primary focus:ring-ring h-4 w-4"
                      />
                    </th>
                    <th className="py-2.5 px-3 w-16">Roll</th>
                    <th className="py-2.5 px-3">Student Name</th>
                    <th className="py-2.5 px-3 w-28">Student Code</th>
                    <th className="py-2.5 px-3">Destination Address</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border/60">
                  {preview?.ready_recipients.map((st) => {
                    const isSelected = selectedStudentIds.has(st.student_id);

                    return (
                      <tr
                        key={st.student_id}
                        onClick={() => toggleStudent(st.student_id)}
                        className={cn(
                          "hover:bg-muted/30 transition-colors cursor-pointer",
                          isSelected ? "bg-primary/5 font-medium" : "opacity-70"
                        )}
                      >
                        <td
                          className="py-2 px-3 text-center"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <input
                            type="checkbox"
                            checked={isSelected}
                            onChange={() => toggleStudent(st.student_id)}
                            className="rounded border-input text-primary focus:ring-ring h-4 w-4"
                          />
                        </td>
                        <td className="py-2 px-3 font-mono font-semibold text-foreground">
                          {st.roll ?? "-"}
                        </td>
                        <td className="py-2 px-3 text-foreground">{st.student_name}</td>
                        <td className="py-2 px-3 font-mono text-muted-foreground">
                          {st.student_code}
                        </td>
                        <td className="py-2 px-3 font-mono text-emerald-600 dark:text-emerald-400 font-medium">
                          {st.contact_address}
                        </td>
                      </tr>
                    );
                  })}

                  {readyCount === 0 && !loading && (
                    <tr>
                      <td
                        colSpan={5}
                        className="py-8 text-center text-muted-foreground text-xs"
                      >
                        No enrolled students with valid contact information for this
                        channel
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </TabsContent>

          {/* Missing Tab */}
          <TabsContent value="missing" className="m-0">
            <div className="overflow-x-auto max-h-[380px]">
              <table className="w-full text-xs text-left border-collapse">
                <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
                  <tr>
                    <th className="py-2.5 px-3 w-16">Roll</th>
                    <th className="py-2.5 px-3">Student Name</th>
                    <th className="py-2.5 px-3 w-28">Student Code</th>
                    <th className="py-2.5 px-3">Status Note</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border/60">
                  {preview?.missing_recipients.map((st) => (
                    <tr
                      key={st.student_id}
                      className="hover:bg-muted/20 text-muted-foreground"
                    >
                      <td className="py-2 px-3 font-mono font-semibold">
                        {st.roll ?? "-"}
                      </td>
                      <td className="py-2 px-3">{st.student_name}</td>
                      <td className="py-2 px-3 font-mono">{st.student_code}</td>
                      <td className="py-2 px-3 text-rose-500 text-[11px]">
                        Missing primary guardian phone or email for{" "}
                        {channel.toUpperCase()}
                      </td>
                    </tr>
                  ))}

                  {missingCount === 0 && !loading && (
                    <tr>
                      <td
                        colSpan={4}
                        className="py-8 text-center text-muted-foreground text-xs"
                      >
                        All enrolled students in this batch have valid contact information
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </TabsContent>
        </Tabs>
      </Card>

      {/* Confirmation Dialog */}
      <Dialog open={confirmModalOpen} onOpenChange={setConfirmModalOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base font-bold">
              <Send className="h-4 w-4 text-primary" />
              Confirm Mass Batch Broadcast
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Please verify your broadcast parameters before messages are enqueued to the
              outbox.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2 text-xs">
            <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-2">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Target Batch:</span>
                <span className="font-semibold text-foreground">
                  {selectedBatch?.name}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Channel:</span>
                <span className="font-bold uppercase text-foreground">{channel}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Recipients:</span>
                <span className="font-bold text-foreground">
                  {selectedCount} students
                </span>
              </div>
              {channel === "sms" && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Total SMS Segments:</span>
                  <span className="font-bold font-mono text-primary">
                    {totalEstimatedCost}
                  </span>
                </div>
              )}
            </div>

            <div className="p-2.5 rounded-md bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 text-[11px]">
              Broadcast messages are dispatched safely through the background queue
              respecting quiet hours.
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setConfirmModalOpen(false)}
              className="text-xs"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={executeSend}
              disabled={sending}
              className="text-xs font-semibold"
            >
              {sending ? "Enqueuing..." : "Confirm & Dispatch"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
