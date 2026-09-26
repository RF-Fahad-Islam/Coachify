import { useEffect, useState, useCallback } from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import {
  Laptop,
  Wifi,
  WifiOff,
  RefreshCw,
  AlertCircle,
  ShieldAlert,
  Server,
  Activity,
  CheckCircle2,
} from "lucide-react";
import {
  rpc,
  type WorkstationClientStatus,
  type WorkstationConnectionState,
} from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useLocale } from "@/lib/i18n";

export interface WorkstationPillProps {
  initialStatus?: WorkstationClientStatus | null;
}

export function WorkstationPill({ initialStatus = null }: WorkstationPillProps = {}) {
  const { locale } = useLocale();
  const [status, setStatus] = useState<WorkstationClientStatus | null>(initialStatus);
  const [isTesting, setIsTesting] = useState(false);
  const [isReconnecting, setIsReconnecting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [open, setOpen] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await rpc("lan.get_client_status", {});
      if (res.ok && res.data) {
        setStatus(res.data);
      }
    } catch {
      // Ignore if offline
    }
  }, []);

  useEffect(() => {
    void fetchStatus();
    const interval = setInterval(() => {
      void fetchStatus();
    }, 8000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const res = await rpc("lan.test_connection", {});
      if (res.ok && res.data) {
        if (res.data.success) {
          setTestResult(
            locale === "bn"
              ? `সফল (${res.data.latency_ms ?? 0} ms)`
              : `Connected (${res.data.latency_ms ?? 0} ms)`
          );
        } else {
          setTestResult(
            res.data.message || (locale === "bn" ? "সংযোগ ব্যর্থ" : "Failed")
          );
        }
      } else {
        setTestResult(res.error?.message || (locale === "bn" ? "ত্রুটি" : "Error"));
      }
    } catch {
      setTestResult(locale === "bn" ? "হোস্টে পৌঁছানো যায়নি" : "Host Unreachable");
    } finally {
      setIsTesting(false);
      void fetchStatus();
    }
  };

  const handleReconnect = async () => {
    setIsReconnecting(true);
    try {
      await rpc("lan.reconnect", {});
    } catch {
      // Ignore
    } finally {
      setIsReconnecting(false);
      void fetchStatus();
    }
  };

  if (!status || !status.is_client_mode) {
    return null;
  }

  const state: WorkstationConnectionState = status.state;

  const stateConfig = {
    connected: {
      color: "bg-emerald-500",
      textColor: "text-emerald-700 dark:text-emerald-400",
      bgColor:
        "bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-800",
      labelEn: `Workstation • ${status.rtt_ms ?? 0}ms`,
      labelBn: `ওয়ার্কস্টেশন • ${status.rtt_ms ?? 0}ms`,
      icon: Wifi,
    },
    connecting: {
      color: "bg-amber-500",
      textColor: "text-amber-700 dark:text-amber-400",
      bgColor: "bg-amber-50 dark:bg-amber-950/40 border-amber-200 dark:border-amber-800",
      labelEn: "Connecting...",
      labelBn: "সংযোগ হচ্ছে...",
      icon: RefreshCw,
    },
    reconnecting: {
      color: "bg-amber-500",
      textColor: "text-amber-700 dark:text-amber-400",
      bgColor: "bg-amber-50 dark:bg-amber-950/40 border-amber-200 dark:border-amber-800",
      labelEn: "Reconnecting...",
      labelBn: "পুনরায় সংযোগ...",
      icon: RefreshCw,
    },
    disconnected: {
      color: "bg-slate-400",
      textColor: "text-slate-700 dark:text-slate-400",
      bgColor: "bg-slate-50 dark:bg-slate-900 border-slate-200 dark:border-slate-800",
      labelEn: "Offline",
      labelBn: "অফলাইন",
      icon: WifiOff,
    },
    revoked: {
      color: "bg-red-500",
      textColor: "text-red-700 dark:text-red-400",
      bgColor: "bg-red-50 dark:bg-red-950/40 border-red-200 dark:border-red-800",
      labelEn: "Revoked",
      labelBn: "বাতিল",
      icon: ShieldAlert,
    },
    blocked: {
      color: "bg-red-500",
      textColor: "text-red-700 dark:text-red-400",
      bgColor: "bg-red-50 dark:bg-red-950/40 border-red-200 dark:border-red-800",
      labelEn: "Blocked",
      labelBn: "স্থগিত",
      icon: AlertCircle,
    },
  }[state] || {
    color: "bg-slate-400",
    textColor: "text-slate-600",
    bgColor: "bg-muted border-border",
    labelEn: "Workstation",
    labelBn: "ওয়ার্কস্টেশন",
    icon: Laptop,
  };

  const isReconnectingState = state === "connecting" || state === "reconnecting";

  return (
    <PopoverPrimitive.Root open={open} onOpenChange={setOpen}>
      <PopoverPrimitive.Trigger asChild>
        <button
          type="button"
          data-testid="workstation-status-pill"
          className={cn(
            "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border transition-colors cursor-pointer focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
            stateConfig.bgColor,
            stateConfig.textColor
          )}
          title={
            locale === "bn"
              ? "ওয়ার্কস্টেশন সংযোগ বিবরণ"
              : "Workstation Connection Details"
          }
        >
          <span className="relative flex h-2 w-2 shrink-0">
            {isReconnectingState && (
              <span
                className={cn(
                  "absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping",
                  stateConfig.color
                )}
              />
            )}
            <span
              className={cn(
                "relative inline-flex h-2 w-2 rounded-full",
                stateConfig.color
              )}
            />
          </span>
          <Laptop className="h-3 w-3 shrink-0" />
          <span className="tabular-nums font-semibold">
            {locale === "bn" ? stateConfig.labelBn : stateConfig.labelEn}
          </span>
        </button>
      </PopoverPrimitive.Trigger>

      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align="end"
          sideOffset={8}
          className="z-50 w-80 rounded-lg border bg-popover p-4 text-popover-foreground shadow-lg outline-none animate-in fade-in-0 zoom-in-95"
        >
          <div className="space-y-3">
            {/* Header */}
            <div className="flex items-center justify-between border-b pb-2">
              <div className="flex items-center gap-2">
                <Laptop className="h-4 w-4 text-primary" />
                <span className="font-semibold text-sm">
                  {status.workstation_name ||
                    (locale === "bn" ? "ওয়ার্কস্টেশন" : "Workstation")}
                </span>
              </div>
              <span
                className={cn(
                  "text-[10px] font-semibold uppercase px-2 py-0.5 rounded-full border",
                  stateConfig.bgColor,
                  stateConfig.textColor
                )}
              >
                {state}
              </span>
            </div>

            {/* Connection Details */}
            <div className="space-y-2 text-xs">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground flex items-center gap-1.5">
                  <Server className="h-3 w-3" />
                  {locale === "bn" ? "হোস্ট সার্ভার" : "Host Server"}
                </span>
                <span className="font-medium">
                  {status.host_name || "Host"} ({status.host_ip}:{status.host_port})
                </span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-muted-foreground flex items-center gap-1.5">
                  <Activity className="h-3 w-3" />
                  {locale === "bn" ? "লেটেন্সি (RTT)" : "Latency (RTT)"}
                </span>
                <span className="font-medium tabular-nums">
                  {status.rtt_ms != null ? `${status.rtt_ms} ms` : "—"}
                </span>
              </div>

              {status.last_heartbeat_at && (
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">
                    {locale === "bn" ? "সর্বশেষ সিঙ্ক" : "Last Sync"}
                  </span>
                  <span className="font-medium text-muted-foreground">
                    {new Date(status.last_heartbeat_at).toLocaleTimeString()}
                  </span>
                </div>
              )}

              {testResult && (
                <div className="mt-2 p-2 rounded bg-muted text-[11px] flex items-center gap-1.5 font-medium">
                  <CheckCircle2 className="h-3.5 w-3.5 text-primary shrink-0" />
                  <span>{testResult}</span>
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="pt-2 border-t flex gap-2">
              <Button
                variant="outline"
                size="sm"
                className="flex-1 text-xs h-8"
                onClick={handleTestConnection}
                disabled={isTesting}
              >
                {isTesting ? (
                  <RefreshCw className="h-3 w-3 animate-spin mr-1.5" />
                ) : (
                  <Activity className="h-3 w-3 mr-1.5" />
                )}
                {locale === "bn" ? "পরীক্ষা করুন" : "Ping Host"}
              </Button>

              <Button
                variant="secondary"
                size="sm"
                className="flex-1 text-xs h-8"
                onClick={handleReconnect}
                disabled={isReconnecting}
              >
                {isReconnecting ? (
                  <RefreshCw className="h-3 w-3 animate-spin mr-1.5" />
                ) : (
                  <RefreshCw className="h-3 w-3 mr-1.5" />
                )}
                {locale === "bn" ? "পুনরায় সংযোগ" : "Reconnect"}
              </Button>
            </div>
          </div>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
