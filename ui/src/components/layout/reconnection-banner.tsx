import { useEffect, useState, useCallback } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { rpc, type WorkstationClientStatus } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { useLocale } from "@/lib/i18n";

export interface ReconnectionBannerProps {
  initialStatus?: WorkstationClientStatus | null;
}

export function ReconnectionBanner({
  initialStatus = null,
}: ReconnectionBannerProps = {}) {
  const { locale } = useLocale();
  const [status, setStatus] = useState<WorkstationClientStatus | null>(initialStatus);
  const [retrying, setRetrying] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await rpc("lan.get_client_status", {});
      if (res.ok && res.data) {
        setStatus(res.data);
      }
    } catch {
      // Ignore
    }
  }, []);

  useEffect(() => {
    void fetchStatus();
    const interval = setInterval(() => {
      void fetchStatus();
    }, 4000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  if (!status || !status.is_client_mode) {
    return null;
  }

  const isReconnecting = status.state === "reconnecting" || status.state === "connecting";

  if (!isReconnecting) {
    return null;
  }

  const handleRetry = async () => {
    setRetrying(true);
    try {
      await rpc("lan.reconnect", {});
    } catch {
      // Ignore
    } finally {
      setRetrying(false);
      void fetchStatus();
    }
  };

  return (
    <div
      role="status"
      aria-live="polite"
      className="bg-amber-500/15 border-b border-amber-500/30 text-amber-900 dark:text-amber-200 px-4 py-2 text-xs flex items-center justify-between sticky top-0 z-40 backdrop-blur-sm"
    >
      <div className="flex items-center gap-2">
        <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0 animate-pulse" />
        <span>
          {locale === "bn"
            ? `হোস্ট সার্ভারের সাথে সংযোগ বিচ্ছিন্ন (${status.host_ip || "host"}:${status.host_port || ""})। স্বয়ংক্রিয়ভাবে পুনরায় সংযোগের চেষ্টা করা হচ্ছে...`
            : `Connection to host server lost (${status.host_ip || "host"}:${status.host_port || ""}). Reconnecting automatically...`}
        </span>
      </div>

      <Button
        variant="outline"
        size="sm"
        onClick={handleRetry}
        disabled={retrying}
        className="h-6 px-2.5 text-[11px] font-semibold bg-amber-50 dark:bg-amber-950/50 border-amber-300 dark:border-amber-700 hover:bg-amber-100 dark:hover:bg-amber-900 text-amber-900 dark:text-amber-200 cursor-pointer"
      >
        <RefreshCw className={`h-3 w-3 mr-1 ${retrying ? "animate-spin" : ""}`} />
        {locale === "bn" ? "এখনই চেষ্টা করুন" : "Retry Now"}
      </Button>
    </div>
  );
}
