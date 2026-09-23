import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type OutboxLifecycleSummary,
  type OutboxItemDetail,
  type RetryResult,
  type FlushResult,
} from "@/gen/rpc";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Send,
  RefreshCw,
  RotateCcw,
  FastForward,
  ChevronDown,
  ChevronRight,
  CheckCircle2,
  XCircle,
  Clock,
  AlertTriangle,
  Mail,
  MessageSquare,
  Smartphone,
} from "lucide-react";
import { cn } from "@/lib/utils";

type StatusFilter = "all" | "queued" | "failed" | "uncertain";

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, { icon: typeof CheckCircle2; class: string }> = {
    queued: {
      icon: Clock,
      class: "bg-blue-500/10 text-blue-700 dark:text-blue-300 border-blue-500/20",
    },
    sending: {
      icon: Send,
      class: "bg-primary/10 text-primary border-primary/20",
    },
    sent: {
      icon: CheckCircle2,
      class:
        "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/20",
    },
    failed: {
      icon: XCircle,
      class: "bg-destructive/10 text-destructive border-destructive/20",
    },
    uncertain: {
      icon: AlertTriangle,
      class: "bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/20",
    },
  };
  const v = map[status] ?? {
    icon: Clock,
    class: "bg-muted text-muted-foreground border-muted",
  };
  const Icon = v.icon;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold border",
        v.class
      )}
    >
      <Icon className="h-3 w-3" aria-hidden="true" />
      {status.charAt(0).toUpperCase() + status.slice(1)}
    </span>
  );
}

function ChannelIcon({ channel }: { channel: string }) {
  if (channel === "email")
    return <Mail className="h-3.5 w-3.5 text-muted-foreground" aria-label="Email" />;
  if (channel === "sms")
    return <Smartphone className="h-3.5 w-3.5 text-muted-foreground" aria-label="SMS" />;
  return (
    <MessageSquare className="h-3.5 w-3.5 text-muted-foreground" aria-label={channel} />
  );
}

export function OutboxLifecycleMonitor() {
  const [summary, setSummary] = useState<OutboxLifecycleSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [retrying, setRetrying] = useState(false);
  const [flushing, setFlushing] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    const res = await rpc("integrity.get_outbox_lifecycle", {
      status: null,
      channel: null,
      limit: 100,
      offset: 0,
    });
    if (res.ok && res.data) {
      setSummary(res.data);
    } else {
      setError(res.error?.message ?? "Failed to load outbox lifecycle");
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const filteredItems =
    summary?.items.filter((item) => {
      if (statusFilter === "all") return true;
      return item.status === statusFilter;
    }) ?? [];

  const retryableItems = filteredItems.filter(
    (i) => i.status === "failed" || i.status === "uncertain"
  );

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleAll = () => {
    if (selectedIds.size === retryableItems.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(retryableItems.map((i) => i.id)));
    }
  };

  const toggleExpand = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleRetry = async (ids: string[]) => {
    setRetrying(true);
    setFeedback(null);
    const res = await rpc("integrity.retry_outbox_messages", {
      message_ids: ids,
    });
    setRetrying(false);
    if (res.ok && res.data) {
      const r: RetryResult = res.data;
      setFeedback(`Retried ${r.retried_count} message(s). ${r.message}`);
      setSelectedIds(new Set());
      void load();
    } else {
      setFeedback(`Error: ${res.error?.message ?? "Retry failed"}`);
    }
  };

  const handleFlush = async () => {
    setFlushing(true);
    setFeedback(null);
    const res = await rpc("integrity.flush_offline_queue", {});
    setFlushing(false);
    if (res.ok && res.data) {
      const f: FlushResult = res.data;
      setFeedback(
        `Flushed: processed ${f.processed_count}, sent ${f.sent_count}, failed ${f.failed_count}. ${f.message}`
      );
      void load();
    } else {
      setFeedback(`Error: ${res.error?.message ?? "Flush failed"}`);
    }
  };

  if (loading) {
    return (
      <div className="space-y-4">
        <div className="h-7 w-64 bg-muted animate-pulse rounded" />
        <div className="flex gap-3">
          {[...Array(6)].map((_, i) => (
            <div key={i} className="h-16 w-24 bg-muted animate-pulse rounded-lg" />
          ))}
        </div>
        <div className="h-64 bg-muted animate-pulse rounded-lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
        <p className="font-semibold">Failed to load outbox</p>
        <p className="text-xs mt-1 opacity-80">{error}</p>
        <Button
          variant="outline"
          size="sm"
          className="mt-3 gap-1.5 text-xs"
          onClick={() => void load()}
        >
          <RefreshCw className="h-3.5 w-3.5" /> Retry
        </Button>
      </div>
    );
  }

  const stats = [
    { label: "Total", value: summary?.total_count ?? 0, color: "text-foreground" },
    {
      label: "Queued",
      value: summary?.queued_count ?? 0,
      color: "text-blue-600 dark:text-blue-400",
    },
    { label: "Sending", value: summary?.sending_count ?? 0, color: "text-primary" },
    {
      label: "Sent",
      value: summary?.sent_count ?? 0,
      color: "text-emerald-600 dark:text-emerald-400",
    },
    { label: "Failed", value: summary?.failed_count ?? 0, color: "text-destructive" },
    {
      label: "Uncertain",
      value: summary?.uncertain_count ?? 0,
      color: "text-amber-600 dark:text-amber-400",
    },
  ];

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2">
            <Send className="h-5 w-5 text-primary" />
            Outbox Lifecycle Monitor
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Inspect message delivery status, retry failures, and flush offline queue.
          </p>
        </div>
        <div className="flex gap-2 flex-wrap">
          <Button
            variant="outline"
            size="sm"
            onClick={() => void load()}
            className="gap-1.5 text-xs"
          >
            <RefreshCw className="h-3.5 w-3.5" /> Refresh
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void handleFlush()}
            disabled={flushing}
            className="gap-1.5 text-xs"
          >
            <FastForward className="h-3.5 w-3.5" />
            {flushing ? "Flushing..." : "Flush Offline Queue"}
          </Button>
          {selectedIds.size > 0 && (
            <Button
              size="sm"
              onClick={() => void handleRetry(Array.from(selectedIds))}
              disabled={retrying}
              className="gap-1.5 text-xs"
            >
              <RotateCcw className="h-3.5 w-3.5" />
              {retrying ? "Retrying..." : `Retry ${selectedIds.size} Selected`}
            </Button>
          )}
        </div>
      </div>

      {/* Feedback */}
      {feedback && (
        <div className="rounded-lg bg-muted/60 border border-border px-4 py-2 text-xs text-foreground">
          {feedback}
        </div>
      )}

      {/* KPI Stats */}
      <div className="flex flex-wrap gap-3">
        {stats.map((s) => (
          <div
            key={s.label}
            className="rounded-lg border border-border bg-card px-4 py-2.5 min-w-[80px]"
          >
            <div className="text-[11px] text-muted-foreground">{s.label}</div>
            <div className={cn("text-xl font-bold", s.color)}>{s.value}</div>
          </div>
        ))}
      </div>

      {/* Filter Buttons */}
      <div className="flex gap-1.5 p-1 rounded-lg bg-muted border border-border w-fit">
        {(["all", "queued", "failed", "uncertain"] as StatusFilter[]).map((f) => (
          <Button
            key={f}
            variant={statusFilter === f ? "primary" : "ghost"}
            size="sm"
            className="h-7 text-xs font-medium px-3"
            onClick={() => setStatusFilter(f)}
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </Button>
        ))}
      </div>

      {/* Table */}
      <Card className="shadow-sm">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-semibold">
            Messages ({filteredItems.length})
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          {filteredItems.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
              <Send className="h-8 w-8 mb-3 opacity-30" aria-hidden="true" />
              <p className="text-sm font-medium">No messages</p>
              <p className="text-xs mt-1">
                {statusFilter === "all"
                  ? "Outbox is empty."
                  : `No ${statusFilter} messages.`}
              </p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50">
                    <TableHead className="w-8">
                      <input
                        type="checkbox"
                        checked={
                          retryableItems.length > 0 &&
                          selectedIds.size === retryableItems.length
                        }
                        onChange={toggleAll}
                        aria-label="Select all retryable messages"
                        className="rounded"
                      />
                    </TableHead>
                    <TableHead className="text-xs">Ch.</TableHead>
                    <TableHead className="text-xs">Recipient</TableHead>
                    <TableHead className="text-xs">Status</TableHead>
                    <TableHead className="text-xs text-right">Tries</TableHead>
                    <TableHead className="text-xs">Last Error</TableHead>
                    <TableHead className="text-xs">Created</TableHead>
                    <TableHead className="text-xs text-right">Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredItems.map((item: OutboxItemDetail) => {
                    const isRetryable =
                      item.status === "failed" || item.status === "uncertain";
                    const expanded = expandedIds.has(item.id);
                    return (
                      <>
                        <TableRow key={item.id} className="group">
                          <TableCell className="w-8">
                            {isRetryable && (
                              <input
                                type="checkbox"
                                checked={selectedIds.has(item.id)}
                                onChange={() => toggleSelect(item.id)}
                                aria-label={`Select message ${item.id}`}
                                className="rounded"
                              />
                            )}
                          </TableCell>
                          <TableCell>
                            <ChannelIcon channel={item.channel} />
                          </TableCell>
                          <TableCell className="text-xs max-w-[140px] truncate">
                            {item.recipient}
                          </TableCell>
                          <TableCell>
                            <StatusBadge status={item.status} />
                          </TableCell>
                          <TableCell className="text-xs text-right text-muted-foreground">
                            {item.attempts}
                          </TableCell>
                          <TableCell className="text-xs text-muted-foreground max-w-[160px] truncate">
                            {item.last_error ?? "—"}
                          </TableCell>
                          <TableCell className="text-xs text-muted-foreground whitespace-nowrap">
                            {new Date(item.created_at).toLocaleString()}
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex items-center justify-end gap-1">
                              {isRetryable && (
                                <Button
                                  size="icon"
                                  variant="ghost"
                                  className="h-7 w-7 text-primary hover:bg-primary/10"
                                  onClick={() => void handleRetry([item.id])}
                                  disabled={retrying}
                                  aria-label={`Retry message ${item.id}`}
                                >
                                  <RotateCcw className="h-3.5 w-3.5" />
                                </Button>
                              )}
                              {item.attempts_history.length > 0 && (
                                <Button
                                  size="icon"
                                  variant="ghost"
                                  className="h-7 w-7 text-muted-foreground hover:bg-muted"
                                  onClick={() => toggleExpand(item.id)}
                                  aria-label={`${expanded ? "Collapse" : "Expand"} history for ${item.id}`}
                                  aria-expanded={expanded}
                                >
                                  {expanded ? (
                                    <ChevronDown className="h-3.5 w-3.5" />
                                  ) : (
                                    <ChevronRight className="h-3.5 w-3.5" />
                                  )}
                                </Button>
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                        {expanded && item.attempts_history.length > 0 && (
                          <TableRow key={`${item.id}-history`} className="bg-muted/30">
                            <TableCell colSpan={8} className="py-2 px-4">
                              <div className="space-y-1">
                                <div className="text-[11px] font-semibold text-muted-foreground mb-1">
                                  Delivery Attempts
                                </div>
                                {item.attempts_history.map((attempt, idx) => (
                                  <div
                                    key={idx}
                                    className="flex items-center gap-3 text-xs"
                                  >
                                    <span className="text-muted-foreground whitespace-nowrap">
                                      {new Date(attempt.attempted_at).toLocaleString()}
                                    </span>
                                    <StatusBadge status={attempt.outcome} />
                                    {attempt.http_status && (
                                      <span className="text-muted-foreground">
                                        HTTP {attempt.http_status}
                                      </span>
                                    )}
                                    {attempt.response_excerpt && (
                                      <span className="text-muted-foreground truncate max-w-[200px]">
                                        {attempt.response_excerpt}
                                      </span>
                                    )}
                                  </div>
                                ))}
                              </div>
                            </TableCell>
                          </TableRow>
                        )}
                      </>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
