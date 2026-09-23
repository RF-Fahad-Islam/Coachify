import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type SmartDueQueueResponse,
  type SmartDueQueueItem,
  type BatchSummary,
} from "@/gen/rpc";
import { formatTaka, formatBusinessDate, formatPhone } from "@/lib/utils";
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
  AlertCircle,
  Phone,
  PhoneCall,
  CreditCard,
  RefreshCw,
  Loader2,
  Users,
  CheckCircle2,
} from "lucide-react";

export interface SmartDueQueueViewProps {
  onCollectDue?: (studentId: string) => void;
  onViewStudentProfile?: (studentId: string) => void;
}

export function SmartDueQueueView({
  onCollectDue,
  onViewStudentProfile,
}: SmartDueQueueViewProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [batches, setBatches] = useState<BatchSummary[]>([]);
  const [selectedBatchId, setSelectedBatchId] = useState<string>("");
  const [minOverdueDays, setMinOverdueDays] = useState<number>(0);
  const [queueData, setQueueData] = useState<SmartDueQueueResponse | null>(null);

  // Record Contact Log Modal
  const [contactTarget, setContactTarget] = useState<SmartDueQueueItem | null>(null);
  const [contactModalOpen, setContactModalOpen] = useState(false);
  const [channel, setChannel] = useState<string>("call");
  const [outcome, setOutcome] = useState<string>("promised");
  const [promisedDate, setPromisedDate] = useState<string>("");
  const [notes, setNotes] = useState<string>("");
  const [savingContact, setSavingContact] = useState(false);
  const [modalError, setModalError] = useState<string | null>(null);

  // Load batches for filter
  useEffect(() => {
    void (async () => {
      try {
        const res = await rpc("batches.list", {});
        if (res.ok && res.data) {
          setBatches(res.data);
        }
      } catch (err: unknown) {
        console.error("Failed to load batches list", err);
      }
    })();
  }, []);

  const loadQueue = useCallback(async (batchId: string, minDays: number) => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("intelligence.get_smart_due_queue", {
        batch_id: batchId || null,
        min_overdue_days: minDays > 0 ? minDays : null,
        limit: 100,
        offset: 0,
      });

      if (res.ok && res.data) {
        setQueueData(res.data);
      } else {
        setError(res.error?.message || "Failed to load smart due collection queue");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadQueue(selectedBatchId, minOverdueDays);
  }, [selectedBatchId, minOverdueDays, loadQueue]);

  const handleOpenContactModal = (item: SmartDueQueueItem) => {
    setContactTarget(item);
    setChannel("call");
    setOutcome("promised");
    setPromisedDate("");
    setNotes("");
    setModalError(null);
    setContactModalOpen(true);
  };

  const handleSaveContact = async () => {
    if (!contactTarget) return;
    setSavingContact(true);
    setModalError(null);

    try {
      const res = await rpc("intelligence.record_due_contact", {
        student_id: contactTarget.student_id,
        contact_channel: channel,
        outcome,
        promised_payment_date: promisedDate ? promisedDate : null,
        notes: notes.trim(),
      });

      if (res.ok) {
        setContactModalOpen(false);
        void loadQueue(selectedBatchId, minOverdueDays);
      } else {
        setModalError(res.error?.message || "Failed to log contact outcome");
      }
    } catch (err: unknown) {
      setModalError(err instanceof Error ? err.message : String(err));
    } finally {
      setSavingContact(false);
    }
  };

  const items = queueData?.items || [];
  const highPriorityCount = items.filter((i) => i.priority_score >= 60).length;

  const getPriorityScoreBadge = (score: number) => {
    if (score >= 70) {
      return (
        <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-destructive text-destructive-foreground font-mono">
          {score}
        </span>
      );
    }
    if (score >= 40) {
      return (
        <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-amber-500 text-white font-mono">
          {score}
        </span>
      );
    }
    return (
      <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-muted text-foreground font-mono">
        {score}
      </span>
    );
  };

  return (
    <div className="space-y-6">
      {/* Header & Filter Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            Smart Due Recovery Queue
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Prioritized recovery queue based on aging, total balance, previous contact
            history, and promised commitments.
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          {/* Batch Selector */}
          <select
            aria-label="Filter by Batch"
            value={selectedBatchId}
            onChange={(e) => setSelectedBatchId(e.target.value)}
            className="h-8 px-3 py-1 text-xs rounded-[var(--radius-md)] bg-card border border-border text-foreground font-medium focus:outline-hidden cursor-pointer"
          >
            <option value="">All Batches ({batches.length})</option>
            {batches.map((b) => (
              <option key={b.id} value={b.id}>
                {b.name}
              </option>
            ))}
          </select>

          {/* Aging Threshold */}
          <select
            aria-label="Filter by Minimum Overdue Days"
            value={minOverdueDays}
            onChange={(e) => setMinOverdueDays(parseInt(e.target.value, 10) || 0)}
            className="h-8 px-3 py-1 text-xs rounded-[var(--radius-md)] bg-card border border-border text-foreground font-medium focus:outline-hidden cursor-pointer"
          >
            <option value="0">All Overdue Days</option>
            <option value="7">&gt; 7 Days Overdue</option>
            <option value="15">&gt; 15 Days Overdue</option>
            <option value="30">&gt; 30 Days Overdue</option>
            <option value="60">&gt; 60 Days Overdue</option>
          </select>

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadQueue(selectedBatchId, minOverdueDays)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh due recovery queue"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {/* Summary KPI Cards Row */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Queued Defaulters
              </span>
              <div className="text-2xl font-bold tracking-tight text-foreground">
                {queueData?.total_students_due ?? items.length}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Accounts with unpaid balances
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-primary-subtle text-primary flex items-center justify-center">
              <Users className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                Total Recoverable Dues
              </span>
              <div className="text-2xl font-bold tracking-tight text-destructive">
                {formatTaka(queueData?.total_due_poisha ?? 0)}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Outstanding receivables
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-destructive/10 text-destructive flex items-center justify-center">
              <CreditCard className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card className="border-border/80 bg-card shadow-2xs">
          <CardContent className="p-4 flex items-center justify-between">
            <div className="space-y-1">
              <span className="text-xs font-semibold text-muted-foreground">
                High Urgent Priority
              </span>
              <div className="text-2xl font-bold tracking-tight text-amber-600 dark:text-amber-400">
                {highPriorityCount}
              </div>
              <span className="text-[11px] text-muted-foreground">
                Score &ge; 60 (Call today)
              </span>
            </div>
            <div className="h-10 w-10 rounded-[var(--radius-md)] bg-amber-500/10 text-amber-600 flex items-center justify-center">
              <AlertCircle className="h-5 w-5" />
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
            Scoring outstanding balances, overdue days, and contact urgency...
          </span>
        </div>
      ) : items.length === 0 ? (
        <Card className="border-dashed border-2 border-border">
          <CardContent className="py-16 text-center space-y-3">
            <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center mx-auto text-muted-foreground">
              <CheckCircle2 className="h-6 w-6 text-emerald-500" />
            </div>
            <h3 className="text-sm font-bold text-foreground">
              No outstanding dues matching filters
            </h3>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              All student balances are either fully cleared or below the selected overdue
              threshold.
            </p>
          </CardContent>
        </Card>
      ) : (
        /* Prioritized Due Queue Table */
        <Card className="border-border">
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-xs text-left border-collapse">
                <thead>
                  <tr className="border-b border-border bg-muted/40 font-semibold text-muted-foreground">
                    <th className="py-3 px-3 text-center">Score</th>
                    <th className="py-3 px-4">Student &amp; Batch</th>
                    <th className="py-3 px-4">Primary Guardian</th>
                    <th className="py-3 px-3 text-right">Total Due</th>
                    <th className="py-3 px-3 text-right">Aging</th>
                    <th className="py-3 px-3">Last Contact</th>
                    <th className="py-3 px-3">Outcome &amp; Promise</th>
                    <th className="py-3 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border/60">
                  {items.map((item) => (
                    <tr
                      key={item.student_id}
                      className="hover:bg-muted/30 transition-colors"
                    >
                      {/* Priority Score Badge */}
                      <td className="py-3 px-3 text-center">
                        {getPriorityScoreBadge(item.priority_score)}
                      </td>

                      {/* Student Details */}
                      <td className="py-3 px-4">
                        <span
                          className={`font-bold text-foreground block ${
                            onViewStudentProfile ? "hover:underline cursor-pointer" : ""
                          }`}
                          onClick={() => onViewStudentProfile?.(item.student_id)}
                        >
                          {item.student_name}
                        </span>
                        <div className="flex items-center gap-2 text-[11px] text-muted-foreground mt-0.5">
                          <span>{item.batch_name}</span>
                          {item.roll_no && (
                            <>
                              <span>•</span>
                              <span className="font-mono">Roll: {item.roll_no}</span>
                            </>
                          )}
                        </div>
                      </td>

                      {/* Guardian & Phone */}
                      <td className="py-3 px-4">
                        <span className="font-medium text-foreground block">
                          {item.primary_guardian_name || "None listed"}
                        </span>
                        {item.primary_guardian_phone ? (
                          <a
                            href={`tel:${item.primary_guardian_phone}`}
                            className="font-mono text-[11px] text-primary hover:underline flex items-center gap-1 mt-0.5"
                          >
                            <Phone className="h-3 w-3" />
                            {formatPhone(item.primary_guardian_phone)}
                          </a>
                        ) : (
                          <span className="text-[11px] text-muted-foreground">—</span>
                        )}
                      </td>

                      {/* Due Amount */}
                      <td className="py-3 px-3 text-right font-mono font-bold text-destructive">
                        {formatTaka(item.total_due_poisha)}
                      </td>

                      {/* Aging */}
                      <td className="py-3 px-3 text-right">
                        <span
                          className={`font-mono font-bold block ${
                            item.overdue_days > 30
                              ? "text-destructive"
                              : item.overdue_days > 15
                                ? "text-amber-600 dark:text-amber-400"
                                : "text-foreground"
                          }`}
                        >
                          {item.overdue_days}d overdue
                        </span>
                        <span className="text-[10px] text-muted-foreground block">
                          Since {formatBusinessDate(item.oldest_unpaid_due_date)}
                        </span>
                      </td>

                      {/* Last Contact */}
                      <td className="py-3 px-3 text-[11px]">
                        {item.days_since_last_contact === 999 ? (
                          <span className="text-muted-foreground italic">
                            Never contacted
                          </span>
                        ) : (
                          <div>
                            <span className="font-medium text-foreground">
                              {item.days_since_last_contact}d ago
                            </span>
                            <span className="text-[10px] text-muted-foreground block">
                              {item.reminder_count} reminder
                              {item.reminder_count === 1 ? "" : "s"}
                            </span>
                          </div>
                        )}
                      </td>

                      {/* Outcome & Promise */}
                      <td className="py-3 px-3 text-[11px]">
                        {item.last_contact_outcome ? (
                          <div className="space-y-0.5">
                            <Badge
                              variant="outline"
                              size="sm"
                              className="capitalize text-[10px]"
                            >
                              {item.last_contact_outcome.replace("_", " ")}
                            </Badge>
                            {item.promised_payment_date && (
                              <span className="text-[10px] text-amber-600 dark:text-amber-400 block font-mono">
                                Due: {formatBusinessDate(item.promised_payment_date)}
                              </span>
                            )}
                          </div>
                        ) : (
                          <span className="text-muted-foreground">—</span>
                        )}
                      </td>

                      {/* Action Buttons */}
                      <td className="py-3 px-4 text-right">
                        <div className="flex items-center justify-end gap-1.5">
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleOpenContactModal(item)}
                            className="h-7 text-xs font-semibold gap-1"
                            title="Log Guardian Call / Contact"
                          >
                            <PhoneCall className="h-3 w-3 text-primary" />
                            Log
                          </Button>

                          {onCollectDue && (
                            <Button
                              variant="primary"
                              size="sm"
                              onClick={() => onCollectDue(item.student_id)}
                              className="h-7 text-xs font-semibold gap-1"
                              title="Open Counter Cashier with this student"
                            >
                              <CreditCard className="h-3 w-3" />
                              Collect
                            </Button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Record Contact Log Modal */}
      <Dialog open={contactModalOpen} onOpenChange={setContactModalOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base font-bold flex items-center gap-2">
              <PhoneCall className="h-4 w-4 text-primary" />
              Log Guardian Follow-up Contact
            </DialogTitle>
            {contactTarget && (
              <p className="text-xs text-muted-foreground">
                Recording communication with{" "}
                <strong className="text-foreground">
                  {contactTarget.primary_guardian_name || contactTarget.student_name}
                </strong>{" "}
                ({formatPhone(contactTarget.primary_guardian_phone || "")}) regarding due
                balance of{" "}
                <span className="text-destructive font-mono font-bold">
                  {formatTaka(contactTarget.total_due_poisha)}
                </span>
                .
              </p>
            )}
          </DialogHeader>

          {modalError && (
            <div className="p-3 rounded bg-destructive/10 text-destructive text-xs">
              {modalError}
            </div>
          )}

          <div className="space-y-4 py-2 text-xs">
            {/* Channel */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">
                Communication Channel
              </label>
              <select
                value={channel}
                onChange={(e) => setChannel(e.target.value)}
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-medium focus:outline-hidden"
              >
                <option value="call">Phone Call</option>
                <option value="sms">SMS Notification</option>
                <option value="whatsapp">WhatsApp Message</option>
                <option value="in_person">In-Person at Counter</option>
              </select>
            </div>

            {/* Outcome */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Contact Outcome</label>
              <select
                value={outcome}
                onChange={(e) => setOutcome(e.target.value)}
                className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-medium focus:outline-hidden"
              >
                <option value="promised">Promised Payment Date</option>
                <option value="requested_time">Requested More Time</option>
                <option value="unreachable">Phone Unreachable / No Answer</option>
                <option value="disputed">Disputed Fee Calculation</option>
                <option value="refused">Refused to Pay</option>
              </select>
            </div>

            {/* Promised Date (conditional) */}
            {(outcome === "promised" || outcome === "requested_time") && (
              <div className="space-y-1.5">
                <label className="font-semibold text-foreground flex items-center justify-between">
                  <span>Promised Payment Date</span>
                  <span className="text-[11px] text-muted-foreground">Optional</span>
                </label>
                <input
                  type="date"
                  value={promisedDate}
                  onChange={(e) => setPromisedDate(e.target.value)}
                  className="w-full px-3 py-1.5 rounded-[var(--radius-md)] border border-border bg-card text-foreground font-mono focus:outline-hidden cursor-pointer"
                />
              </div>
            )}

            {/* Notes */}
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">
                Call / Conversation Notes
              </label>
              <textarea
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                rows={3}
                placeholder="e.g. Guardian will visit on Sunday after salary disbursement..."
                className="w-full px-3 py-2 rounded-[var(--radius-md)] border border-border bg-card text-foreground focus:outline-hidden"
              />
            </div>
          </div>

          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setContactModalOpen(false)}
              disabled={savingContact}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveContact}
              disabled={savingContact}
              className="gap-1.5"
            >
              {savingContact ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
              Save Outcome
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
