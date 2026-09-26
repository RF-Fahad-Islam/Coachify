import { useEffect, useState, useCallback } from "react";
import { ShieldAlert, AlertOctagon, LogOut } from "lucide-react";
import { rpc, type WorkstationClientStatus } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { useLocale } from "@/lib/i18n";

export interface WorkstationLockoutOverlayProps {
  initialStatus?: WorkstationClientStatus | null;
}

export function WorkstationLockoutOverlay({
  initialStatus = null,
}: WorkstationLockoutOverlayProps = {}) {
  const { locale } = useLocale();
  const [status, setStatus] = useState<WorkstationClientStatus | null>(initialStatus);
  const [unpairing, setUnpairing] = useState(false);

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
    }, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  if (!status || !status.is_client_mode) {
    return null;
  }

  const isLocked = status.state === "revoked" || status.state === "blocked";
  if (!isLocked) {
    return null;
  }

  const isRevoked = status.state === "revoked";

  const handleUnpair = async () => {
    setUnpairing(true);
    try {
      await rpc("lan.disconnect_workstation", {});
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    } catch {
      // Force reload anyway
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    } finally {
      setUnpairing(false);
    }
  };

  return (
    <div
      role="alertdialog"
      aria-modal="true"
      className="fixed inset-0 z-[100] bg-background/95 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in-0"
    >
      <div className="max-w-md w-full bg-card border border-destructive/30 rounded-xl p-6 shadow-2xl text-center space-y-4">
        <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-destructive/10 text-destructive">
          {isRevoked ? (
            <ShieldAlert className="h-8 w-8" />
          ) : (
            <AlertOctagon className="h-8 w-8" />
          )}
        </div>

        <div className="space-y-2">
          <h2 className="text-lg font-bold tracking-tight text-foreground">
            {isRevoked
              ? locale === "bn"
                ? "ওয়ার্কস্টেশন অনুমোদন বাতিল করা হয়েছে"
                : "Workstation Decommissioned by Administrator"
              : locale === "bn"
                ? "ওয়ার্কস্টেশন অ্যাক্সেস সাময়িকভাবে স্থগিত"
                : "Workstation Access Temporarily Suspended"}
          </h2>

          <p className="text-xs text-muted-foreground leading-relaxed">
            {isRevoked
              ? locale === "bn"
                ? "প্রধান সার্ভার অ্যাডমিনিস্ট্রেটর এই ডিভাইসের পেয়ারিং ও অ্যাক্সেস বাতিল করেছেন। বর্তমান সেশনটি নিরাপদভাবে মুছে ফেলা হয়েছে।"
                : "The primary institution administrator has revoked pairing authorization for this workstation. Local credentials have been purged for security."
              : locale === "bn"
                ? "প্রধান অ্যাডমিনিস্ট্রেটর সাময়িকভাবে এই কাউন্টারের অ্যাক্সেস স্থগিত করেছেন। পুনরায় সচল করতে অ্যাডমিনিস্ট্রেটরের সাথে যোগাযোগ করুন।"
                : "The primary administrator has temporarily suspended traffic from this counter. Please contact your host administrator to unblock this device."}
          </p>
        </div>

        <div className="p-3 bg-muted/60 rounded-lg text-left text-xs space-y-1">
          <div className="flex justify-between">
            <span className="text-muted-foreground">
              {locale === "bn" ? "ওয়ার্কস্টেশন:" : "Workstation:"}
            </span>
            <span className="font-semibold">
              {status.workstation_name || "Client PC"}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">
              {locale === "bn" ? "হোস্ট:" : "Host:"}
            </span>
            <span className="font-mono">
              {status.host_ip}:{status.host_port}
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">
              {locale === "bn" ? "ডিভাইস আইডি:" : "Device ID:"}
            </span>
            <span className="font-mono text-[10px] truncate max-w-[180px]">
              {status.device_id}
            </span>
          </div>
        </div>

        <div className="pt-2">
          <Button
            variant="destructive"
            className="w-full font-semibold gap-2"
            onClick={handleUnpair}
            disabled={unpairing}
          >
            <LogOut className="h-4 w-4" />
            {locale === "bn"
              ? "ওয়ার্কস্টেশন আনপেয়ার ও রিস্টার্ট করুন"
              : "Unpair & Reset Workstation"}
          </Button>
        </div>
      </div>
    </div>
  );
}
