import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type OutboxItem,
  type OutboxStatsResponse,
  type OutboxDetailResponse,
} from "@/gen/rpc";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Send,
  RotateCcw,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Clock,
  Mail,
  MessageSquare,
  Smartphone,
  Info,
  ShieldAlert,
  FastForward,
} from "lucide-react";
import { cn } from "@/lib/utils";

export function OutboxMonitorView() {
  const [loading, setLoading] = useState(false);
  const [flushing, setFlushing] = useState(false);
  const [actionLoadingId, setActionLoadingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  const [stats, setStats] = useState<OutboxStatsResponse>({
    total_queued: 0,
    total_sent: 0,
    total_uncertain: 0,
    total_failed: 0,
    total_cancelled: 0,
  });

  const [items, setItems] = useState<OutboxItem[]>([]);
  const [totalCount, setTotalCount] = useState(0);

  // Filters
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [channelFilter, setChannelFilter] = useState<string>("all");
  const [bypassQuietHours, setBypassQuietHours] = useState(false);

  // Detail Modal
  const [detailModalOpen, setDetailModalOpen] = useState(false);
  const [detailData, setDetailData] = useState<OutboxDetailResponse | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);

  // Load stats and outbox items
  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [statsRes, listRes] = await Promise.all([
        rpc("comms.get_outbox_stats", {}),
        rpc("comms.list_outbox", {
          status: statusFilter === "all" ? null : statusFilter,
          channel: channelFilter === "all" ? null : channelFilter,
          limit: 100,
          offset: 0,
        }),
      ]);

      if (statsRes.ok && statsRes.data) {
        setStats(statsRes.data);
      }
      if (listRes.ok && listRes.data) {
        setItems(listRes.data.items);
        setTotalCount(listRes.data.total_count);
      } else if (listRes.error) {
        setError(listRes.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load outbox data");
    } finally {
      setLoading(false);
    }
  }, [statusFilter, channelFilter]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  // Flush queue
  const handleFlushQueue = async () => {
    setFlushing(true);
    setFeedback(null);
    setError(null);

    try {
      const res = await rpc("comms.flush_queue", {
        channel: channelFilter === "all" ? null : channelFilter,
        force_bypass_quiet: bypassQuietHours,
      });

      if (res.ok && res.data) {
        setFeedback(
          `Queue flushed successfully. Dispatched ${res.data.processed_count} message(s).`
        );
        void loadData();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to flush queue");
    } finally {
      setFlushing(false);
    }
  };

  // Resend message
  const handleResend = async (id: string) => {
    setActionLoadingId(id);
    try {
      const res = await rpc("comms.resend_message", { id });
      if (res.ok) {
        setFeedback("Message queued for resend");
        void loadData();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to resend message");
    } finally {
      setActionLoadingId(null);
    }
  };

  // Cancel message
  const handleCancel = async (id: string) => {
    setActionLoadingId(id);
    try {
      const res = await rpc("comms.cancel_message", { id });
      if (res.ok) {
        setFeedback("Pending message cancelled");
        void loadData();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to cancel message");
    } finally {
      setActionLoadingId(null);
    }
  };

  // View detail
  const handleViewDetail = async (id: string) => {
    setDetailLoading(true);
    setDetailModalOpen(true);
    setDetailData(null);

    try {
      const res = await rpc("comms.get_outbox_detail", { id });
      if (res.ok && res.data) {
        setDetailData(res.data);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load message details");
    } finally {
      setDetailLoading(false);
    }
  };

  const getChannelIcon = (channel: string) => {
    switch (channel) {
      case "sms":
        return <Smartphone className="h-3.5 w-3.5 text-blue-500" />;
      case "whatsapp":
        return <MessageSquare className="h-3.5 w-3.5 text-emerald-500" />;
      case "email":
        return <Mail className="h-3.5 w-3.5 text-purple-500" />;
      default:
        return <Send className="h-3.5 w-3.5 text-muted-foreground" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "queued":
        return (
          <Badge
            variant="outline"
            className="gap-1 text-[11px] border-amber-500/30 text-amber-600 dark:text-amber-400"
          >
            <Clock className="h-3 w-3" />
            Queued
          </Badge>
        );
      case "sending":
        return (
          <Badge variant="secondary" className="gap-1 text-[11px] animate-pulse">
            <Send className="h-3 w-3" />
            Sending
          </Badge>
        );
      case "sent":
        return (
          <Badge variant="paid" className="gap-1 text-[11px]">
            <CheckCircle2 className="h-3 w-3" />
            Sent
          </Badge>
        );
      case "uncertain":
        return (
          <Badge
            variant="pending"
            className="gap-1 text-[11px] bg-amber-500/20 text-amber-700 dark:text-amber-300 font-bold border border-amber-500/30"
          >
            <AlertTriangle className="h-3 w-3" />
            Uncertain
          </Badge>
        );
      case "failed":
        return (
          <Badge variant="overdue" className="gap-1 text-[11px]">
            <XCircle className="h-3 w-3" />
            Failed
          </Badge>
        );
      case "cancelled":
        return (
          <Badge variant="outline" className="gap-1 text-[11px] text-muted-foreground">
            Cancelled
          </Badge>
        );
      default:
        return <Badge variant="outline">{status}</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      {/* Metrics Row */}
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
        <button
          type="button"
          onClick={() => setStatusFilter("queued")}
          className={cn(
            "p-3 rounded-lg border text-left transition-all cursor-pointer",
            statusFilter === "queued"
              ? "border-amber-500/60 bg-amber-500/10 shadow-xs"
              : "border-border/80 bg-card/40 hover:bg-muted/50"
          )}
        >
          <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
            <span>Queued</span>
            <Clock className="h-3.5 w-3.5 text-amber-500" />
          </div>
          <div className="text-2xl font-bold text-foreground mt-1">
            {stats.total_queued}
          </div>
        </button>

        <button
          type="button"
          onClick={() => setStatusFilter("sent")}
          className={cn(
            "p-3 rounded-lg border text-left transition-all cursor-pointer",
            statusFilter === "sent"
              ? "border-emerald-500/60 bg-emerald-500/10 shadow-xs"
              : "border-border/80 bg-card/40 hover:bg-muted/50"
          )}
        >
          <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
            <span>Delivered</span>
            <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />
          </div>
          <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 mt-1">
            {stats.total_sent}
          </div>
        </button>

        <button
          type="button"
          onClick={() => setStatusFilter("uncertain")}
          className={cn(
            "p-3 rounded-lg border text-left transition-all cursor-pointer",
            statusFilter === "uncertain"
              ? "border-amber-500 bg-amber-500/20 shadow-xs"
              : stats.total_uncertain > 0
                ? "border-amber-500/40 bg-amber-500/10 hover:bg-amber-500/20"
                : "border-border/80 bg-card/40 hover:bg-muted/50"
          )}
        >
          <div className="text-[11px] font-medium text-amber-700 dark:text-amber-300 flex items-center justify-between">
            <span>Uncertain</span>
            <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />
          </div>
          <div className="text-2xl font-bold text-amber-600 dark:text-amber-400 mt-1">
            {stats.total_uncertain}
          </div>
        </button>

        <button
          type="button"
          onClick={() => setStatusFilter("failed")}
          className={cn(
            "p-3 rounded-lg border text-left transition-all cursor-pointer",
            statusFilter === "failed"
              ? "border-rose-500/60 bg-rose-500/10 shadow-xs"
              : "border-border/80 bg-card/40 hover:bg-muted/50"
          )}
        >
          <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
            <span>Failed</span>
            <XCircle className="h-3.5 w-3.5 text-rose-500" />
          </div>
          <div className="text-2xl font-bold text-rose-600 dark:text-rose-400 mt-1">
            {stats.total_failed}
          </div>
        </button>

        <button
          type="button"
          onClick={() => setStatusFilter("cancelled")}
          className={cn(
            "p-3 rounded-lg border text-left transition-all cursor-pointer",
            statusFilter === "cancelled"
              ? "border-slate-500/60 bg-slate-500/10 shadow-xs"
              : "border-border/80 bg-card/40 hover:bg-muted/50"
          )}
        >
          <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
            <span>Cancelled</span>
            <Info className="h-3.5 w-3.5 text-slate-500" />
          </div>
          <div className="text-2xl font-bold text-foreground mt-1">
            {stats.total_cancelled}
          </div>
        </button>
      </div>

      {/* Prominent Amber Warning for Uncertain Status */}
      {stats.total_uncertain > 0 && (
        <div className="p-4 rounded-lg bg-amber-500/15 border border-amber-500/30 text-amber-950 dark:text-amber-200 flex items-start gap-3 shadow-xs">
          <ShieldAlert className="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
          <div className="space-y-1 text-xs">
            <div className="font-bold text-sm">
              Uncertain Delivery Warning ({stats.total_uncertain} Message
              {stats.total_uncertain === 1 ? "" : "s"})
            </div>
            <p className="leading-relaxed">
              One or more network requests timed out during dispatch before the gateway
              confirmed delivery. In accordance with the zero automated duplicate retry
              policy, these messages remain in uncertain status to safeguard your center
              from duplicate cellular charges or spamming guardians. Please inspect the
              recipient details and click Resend only after verifying if the message
              reached the guardian.
            </p>
          </div>
        </div>
      )}

      {/* Controls & Queue Actions */}
      <Card className="p-4 bg-card/60 backdrop-blur-xs border-border/70 shadow-xs">
        <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <label
                htmlFor="outbox-channel-select"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Channel:
              </label>
              <select
                id="outbox-channel-select"
                value={channelFilter}
                onChange={(e) => setChannelFilter(e.target.value)}
                className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
              >
                <option value="all">All Channels</option>
                <option value="sms">SMS</option>
                <option value="whatsapp">WhatsApp</option>
                <option value="email">Email</option>
              </select>
            </div>

            <div className="flex items-center gap-2">
              <label
                htmlFor="outbox-status-select"
                className="text-xs font-semibold text-muted-foreground whitespace-nowrap"
              >
                Status:
              </label>
              <select
                id="outbox-status-select"
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="h-9 px-3 rounded-md border border-input bg-background text-xs font-medium focus:outline-hidden focus:ring-1 focus:ring-ring"
              >
                <option value="all">All Statuses</option>
                <option value="queued">Queued</option>
                <option value="sending">Sending</option>
                <option value="sent">Sent</option>
                <option value="uncertain">Uncertain</option>
                <option value="failed">Failed</option>
                <option value="cancelled">Cancelled</option>
              </select>
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => void loadData()}
              disabled={loading}
              className="h-9 text-xs gap-1.5"
            >
              <RotateCcw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
              Refresh
            </Button>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
              <input
                type="checkbox"
                checked={bypassQuietHours}
                onChange={(e) => setBypassQuietHours(e.target.checked)}
                className="rounded border-input text-primary focus:ring-ring h-4 w-4"
              />
              Bypass Quiet Hours (Urgent)
            </label>

            <Button
              variant="primary"
              size="sm"
              onClick={handleFlushQueue}
              disabled={flushing || loading}
              className="h-9 gap-1.5 font-semibold text-xs bg-primary text-primary-foreground shadow-xs"
            >
              <FastForward className={cn("h-3.5 w-3.5", flushing && "animate-spin")} />
              {flushing ? "Dispatching..." : "Send All Pending"}
            </Button>
          </div>
        </div>
      </Card>

      {/* Feedbacks */}
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

      {/* Outbox Table */}
      <Card className="overflow-hidden border-border/80 shadow-xs">
        <CardHeader className="py-3 px-4 bg-muted/20 border-b border-border/70 flex flex-row items-center justify-between">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <Send className="h-4 w-4 text-primary" />
            Outbox Queue Records
          </CardTitle>
          <div className="text-xs text-muted-foreground">
            Displaying {items.length} of {totalCount} records
          </div>
        </CardHeader>

        <div className="overflow-x-auto max-h-[560px]">
          <table className="w-full text-xs text-left border-collapse">
            <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
              <tr>
                <th className="py-2.5 px-3 w-10 text-center">Type</th>
                <th className="py-2.5 px-3 w-36">Destination</th>
                <th className="py-2.5 px-3">Message Excerpt / Subject</th>
                <th className="py-2.5 px-3 w-28 text-center">Cost Units</th>
                <th className="py-2.5 px-3 w-36">Scheduled / Sent</th>
                <th className="py-2.5 px-3 w-28 text-center">Status</th>
                <th className="py-2.5 px-3 w-32 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {items.map((item) => {
                const isActionLoading = actionLoadingId === item.id;

                return (
                  <tr
                    key={item.id}
                    className={cn(
                      "hover:bg-muted/30 transition-colors",
                      item.status === "uncertain" && "bg-amber-500/5",
                      item.status === "failed" && "bg-rose-500/5"
                    )}
                  >
                    <td className="py-2.5 px-3 text-center">
                      <div
                        className="flex justify-center"
                        title={`Channel: ${item.channel}`}
                      >
                        {getChannelIcon(item.channel)}
                      </div>
                    </td>
                    <td className="py-2.5 px-3 font-mono font-medium text-foreground">
                      {item.to_address}
                    </td>
                    <td className="py-2.5 px-3 max-w-md">
                      {item.subject && (
                        <div className="font-semibold text-foreground truncate mb-0.5">
                          {item.subject}
                        </div>
                      )}
                      <div className="text-muted-foreground truncate text-[11px]">
                        {item.body}
                      </div>
                    </td>
                    <td className="py-2.5 px-3 text-center font-mono text-[11px] text-muted-foreground">
                      {item.channel === "sms" ? (
                        <span>
                          {item.segments} seg ({item.encoding})
                        </span>
                      ) : (
                        <span>{item.body.length} chars</span>
                      )}
                    </td>
                    <td className="py-2.5 px-3 text-muted-foreground text-[11px]">
                      <div>{item.sent_at || item.scheduled_at}</div>
                      {item.attempts > 0 && (
                        <div className="text-[10px] text-muted-foreground/80">
                          {item.attempts} of {item.max_attempts} attempt(s)
                        </div>
                      )}
                    </td>
                    <td className="py-2.5 px-3 text-center">
                      {getStatusBadge(item.status)}
                    </td>
                    <td className="py-2.5 px-3 text-right">
                      <div className="flex items-center justify-end gap-1.5">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => void handleViewDetail(item.id)}
                          className="h-7 px-2 text-[11px]"
                        >
                          Detail
                        </Button>

                        {(item.status === "uncertain" || item.status === "failed") && (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => void handleResend(item.id)}
                            disabled={isActionLoading}
                            className="h-7 px-2 text-[11px] text-amber-600 hover:bg-amber-500/10 font-semibold"
                          >
                            Resend
                          </Button>
                        )}

                        {item.status === "queued" && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => void handleCancel(item.id)}
                            disabled={isActionLoading}
                            className="h-7 px-2 text-[11px] text-rose-600 hover:bg-rose-500/10"
                          >
                            Cancel
                          </Button>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}

              {items.length === 0 && !loading && (
                <tr>
                  <td
                    colSpan={7}
                    className="py-12 text-center text-muted-foreground text-xs"
                  >
                    No outbox messages found matching current filters
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Message Detail Modal */}
      <Dialog open={detailModalOpen} onOpenChange={setDetailModalOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base font-bold">
              <Send className="h-4 w-4 text-primary" />
              Outbox Message Details
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Full message body and delivery audit attempt history.
            </DialogDescription>
          </DialogHeader>

          {detailLoading || !detailData ? (
            <div className="py-8 text-center text-xs text-muted-foreground">
              Loading message details...
            </div>
          ) : (
            <div className="space-y-4 py-2 text-xs">
              <div className="grid grid-cols-2 gap-2 p-3 rounded-md bg-muted/40 border border-border">
                <div>
                  <span className="text-muted-foreground">Channel:</span>
                  <div className="font-semibold text-foreground uppercase">
                    {detailData.item.channel}
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Recipient:</span>
                  <div className="font-mono font-semibold text-foreground">
                    {detailData.item.to_address}
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Status:</span>
                  <div className="mt-0.5">{getStatusBadge(detailData.item.status)}</div>
                </div>
                <div>
                  <span className="text-muted-foreground">Dedupe Key:</span>
                  <div className="font-mono text-[10px] text-muted-foreground truncate">
                    {detailData.item.dedupe_key || "None"}
                  </div>
                </div>
              </div>

              {detailData.item.subject && (
                <div>
                  <span className="font-semibold text-muted-foreground block mb-1">
                    Subject:
                  </span>
                  <div className="p-2 rounded bg-background border border-border font-medium">
                    {detailData.item.subject}
                  </div>
                </div>
              )}

              <div>
                <span className="font-semibold text-muted-foreground block mb-1">
                  Message Body:
                </span>
                <div className="p-3 rounded-md bg-background border border-border font-mono text-[11px] whitespace-pre-wrap max-h-48 overflow-y-auto leading-relaxed">
                  {detailData.item.body}
                </div>
              </div>

              {/* Delivery Attempts List */}
              <div>
                <span className="font-semibold text-muted-foreground block mb-1.5">
                  Delivery Attempts ({detailData.attempts.length}):
                </span>
                {detailData.attempts.length === 0 ? (
                  <div className="text-muted-foreground italic text-[11px]">
                    No delivery attempts yet
                  </div>
                ) : (
                  <div className="space-y-2 max-h-40 overflow-y-auto">
                    {detailData.attempts.map((att, idx) => (
                      <div
                        key={att.id}
                        className="p-2.5 rounded border border-border/80 bg-muted/20 flex flex-col gap-1 text-[11px]"
                      >
                        <div className="flex items-center justify-between font-semibold">
                          <span>
                            Attempt #{idx + 1} ({att.duration_ms} ms)
                          </span>
                          <span
                            className={cn(
                              "font-bold uppercase text-[10px] px-1.5 py-0.5 rounded",
                              att.outcome === "success"
                                ? "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300"
                                : att.outcome === "timeout"
                                  ? "bg-amber-500/20 text-amber-700 dark:text-amber-300"
                                  : "bg-rose-500/20 text-rose-700 dark:text-rose-300"
                            )}
                          >
                            {att.outcome} (Status {att.http_status ?? "-"})
                          </span>
                        </div>
                        <div className="text-[10px] text-muted-foreground">
                          {att.attempted_at}
                        </div>
                        {att.response_excerpt && (
                          <div className="font-mono text-[10px] text-muted-foreground truncate">
                            {att.response_excerpt}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}

          <DialogFooter>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setDetailModalOpen(false)}
              className="text-xs"
            >
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
