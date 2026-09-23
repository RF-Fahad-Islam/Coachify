import { useEffect, useRef, useState } from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { rpc, type ResilienceStatus } from "@/gen/rpc";
import { useAuth } from "@/contexts/auth-context";
import { cn } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatRelativeTime(isoString: string | null | undefined): string {
  if (!isoString) return "Never";
  const delta = Math.floor((Date.now() - new Date(isoString).getTime()) / 1000);
  if (delta < 60) return `${delta}s ago`;
  if (delta < 3600) return `${Math.floor(delta / 60)}m ago`;
  if (delta < 86400) return `${Math.floor(delta / 3600)}h ago`;
  return `${Math.floor(delta / 86400)}d ago`;
}

function formatTimestamp(isoString: string): string {
  try {
    return new Date(isoString).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return isoString;
  }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

type HealthLevel = "green" | "amber" | "red" | "checking";

interface DotProps {
  health: HealthLevel;
}

function StatusDot({ health }: DotProps) {
  const colorMap: Record<HealthLevel, string> = {
    green: "bg-emerald-500",
    amber: "bg-amber-500",
    red: "bg-red-500",
    checking: "bg-muted-foreground/50",
  };
  const pulse = health !== "checking";

  return (
    <span className="relative flex h-2 w-2 shrink-0">
      {pulse && (
        <span
          className={cn(
            "absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping",
            colorMap[health]
          )}
          aria-hidden="true"
        />
      )}
      <span
        className={cn("relative inline-flex h-2 w-2 rounded-full", colorMap[health])}
        aria-hidden="true"
      />
    </span>
  );
}

interface DetailRowProps {
  label: string;
  value: string;
  valueClassName?: string;
}

function DetailRow({ label, value, valueClassName }: DetailRowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-muted-foreground">{label}</span>
      <span className={cn("font-medium tabular-nums", valueClassName)}>{value}</span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function ResilienceStatusBadge() {
  const { hasPermission } = useAuth();
  const [status, setStatus] = useState<ResilienceStatus | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const canView = hasPermission("integrity.view");

  useEffect(() => {
    if (!canView) return;

    let mounted = true;

    const poll = async () => {
      try {
        const res = await rpc("integrity.get_resilience_status", {});
        if (mounted && res.ok && res.data) {
          setStatus(res.data);
        }
        // On failure: stay silent, keep last known status (or null)
      } catch {
        // Silently ignore RPC errors per spec
      }
    };

    void poll();
    intervalRef.current = setInterval(() => void poll(), 5_000);

    return () => {
      mounted = false;
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
      }
    };
  }, [canView]);

  // Permission gate — render nothing if not authorized
  if (!canView) return null;

  const health: HealthLevel =
    status === null
      ? "checking"
      : status.overall_health === "green"
        ? "green"
        : status.overall_health === "amber"
          ? "amber"
          : "red";

  const labelMap: Record<HealthLevel, string> = {
    green: "Online",
    amber: "Degraded",
    red: "Alert",
    checking: "Checking",
  };

  const dbStatusColor =
    status?.db_status === "Healthy"
      ? "text-emerald-600 dark:text-emerald-400"
      : status?.db_status === "Warning"
        ? "text-amber-600 dark:text-amber-400"
        : status?.db_status === "Corrupted"
          ? "text-red-600 dark:text-red-400"
          : undefined;

  return (
    <PopoverPrimitive.Root>
      <PopoverPrimitive.Trigger asChild>
        <button
          type="button"
          aria-label={`System resilience status: ${labelMap[health]}`}
          className={cn(
            "inline-flex items-center gap-1.5 px-2 py-1 rounded-full border text-[11px] font-medium",
            "bg-card/80 border-border text-foreground",
            "hover:bg-muted/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background",
            "cursor-pointer transition-colors select-none"
          )}
        >
          <StatusDot health={health} />
          <span>{labelMap[health]}</span>
        </button>
      </PopoverPrimitive.Trigger>

      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          side="bottom"
          align="end"
          sideOffset={8}
          collisionPadding={12}
          className={cn(
            "z-50 w-64 rounded-[var(--radius-md)] border border-border bg-card shadow-lg",
            "p-3 space-y-3 text-xs",
            "animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
            "data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2"
          )}
        >
          {/* Header */}
          <div className="flex items-center justify-between">
            <span className="font-semibold text-foreground text-[11px] uppercase tracking-wide">
              System Status
            </span>
            <span
              className={cn(
                "inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-semibold",
                health === "green" &&
                  "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400",
                health === "amber" &&
                  "bg-amber-500/15 text-amber-700 dark:text-amber-400",
                health === "red" && "bg-red-500/15 text-red-700 dark:text-red-400",
                health === "checking" && "bg-muted text-muted-foreground"
              )}
            >
              <StatusDot health={health} />
              {labelMap[health]}
            </span>
          </div>

          <div className="h-px bg-border" aria-hidden="true" />

          {status === null ? (
            <p className="text-muted-foreground text-center py-2">Fetching status…</p>
          ) : (
            <div className="space-y-1.5">
              <DetailRow
                label="Database"
                value={status.db_status}
                valueClassName={dbStatusColor}
              />
              <DetailRow
                label="Queued messages"
                value={String(status.queued_outbox_count)}
                valueClassName={
                  status.queued_outbox_count > 0
                    ? "text-amber-600 dark:text-amber-400"
                    : undefined
                }
              />
              <DetailRow
                label="Failed messages"
                value={String(status.failed_outbox_count)}
                valueClassName={
                  status.failed_outbox_count > 0
                    ? "text-red-600 dark:text-red-400"
                    : undefined
                }
              />
              <DetailRow
                label="Disk free"
                value={`${status.disk_free_gb.toFixed(1)} GB`}
                valueClassName={
                  status.disk_free_gb < 1
                    ? "text-red-600 dark:text-red-400"
                    : status.disk_free_gb < 5
                      ? "text-amber-600 dark:text-amber-400"
                      : undefined
                }
              />
              <DetailRow
                label="Last backup"
                value={formatRelativeTime(status.last_backup_at)}
              />
            </div>
          )}

          {status && (
            <>
              <div className="h-px bg-border" aria-hidden="true" />
              <p className="text-[10px] text-muted-foreground text-right">
                Checked {formatTimestamp(status.checked_at)}
              </p>
            </>
          )}
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
