import { useState, useEffect, type FormEvent } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Laptop,
  Server,
  KeyRound,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Wifi,
} from "lucide-react";
import { rpc, type DiscoveredHost } from "@/gen/rpc";
import { useLocale } from "@/lib/i18n";

export interface WorkstationPairDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onPairSuccess?: () => void;
}

export function WorkstationPairDialog({
  open,
  onOpenChange,
  onPairSuccess,
}: WorkstationPairDialogProps) {
  const { locale } = useLocale();
  const [hostIp, setHostIp] = useState("");
  const [hostPort, setHostPort] = useState(50443);
  const [pairingCode, setPairingCode] = useState("");
  const [workstationName, setWorkstationName] = useState("Front Desk Counter");
  const [submitting, setSubmitting] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [discoveredHosts, setDiscoveredHosts] = useState<DiscoveredHost[]>([]);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const discoverHosts = async () => {
    setDiscovering(true);
    try {
      const res = await rpc("lan.discover_hosts", { timeout_ms: 3000 });
      if (res.ok && res.data) {
        setDiscoveredHosts(res.data.hosts);
      }
    } catch {
      // Ignore discovery errors
    } finally {
      setDiscovering(false);
    }
  };

  useEffect(() => {
    if (open) {
      setErrorMsg(null);
      setSuccessMsg(null);
      void discoverHosts();
    }
  }, [open]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);
    setSuccessMsg(null);

    const cleanIp = hostIp.trim();
    const cleanCode = pairingCode.trim();
    const cleanName = workstationName.trim();

    if (!cleanIp) {
      setErrorMsg(locale === "bn" ? "হোস্ট আইপি প্রয়োজন।" : "Host IP is required.");
      return;
    }
    if (!cleanCode || cleanCode.length !== 6) {
      setErrorMsg(
        locale === "bn"
          ? "৬-সংখ্যার সঠিক পেয়ারিং কোড লিখুন।"
          : "Please enter a valid 6-digit numeric pairing code."
      );
      return;
    }
    if (!cleanName) {
      setErrorMsg(
        locale === "bn"
          ? "ওয়ার্কস্টেশনের নাম লিখুন।"
          : "Please enter a workstation name."
      );
      return;
    }

    setSubmitting(true);
    try {
      const res = await rpc("lan.pair_with_host", {
        host_ip: cleanIp,
        host_port: Number(hostPort) || 50443,
        pairing_code: cleanCode,
        workstation_name: cleanName,
      });

      if (res.ok && res.data) {
        setSuccessMsg(
          locale === "bn"
            ? `সফলভাবে ${res.data.institution_name} এর সাথে সংযুক্ত হয়েছে!`
            : `Successfully paired with ${res.data.institution_name}!`
        );
        setTimeout(() => {
          onPairSuccess?.();
          if (typeof window !== "undefined") {
            window.location.reload();
          }
        }, 1200);
      } else {
        const code = res.error?.code;
        if (code === "PAIRING_CODE_INVALID") {
          setErrorMsg(
            locale === "bn"
              ? "ভুল পেয়ারিং কোড। আবার চেষ্টা করুন।"
              : "Invalid pairing code. Please try again."
          );
        } else if (code === "PAIRING_LOCKED") {
          setErrorMsg(
            locale === "bn"
              ? "পেয়ারিং লক করা আছে। অনুগ্রহ করে কিছুক্ষণ অপেক্ষা করুন।"
              : "Pairing is temporarily locked due to failed attempts."
          );
        } else if (code === "WORKSTATION_LIMIT_REACHED") {
          setErrorMsg(
            locale === "bn"
              ? "লাইসেন্সকৃত ওয়ার্কস্টেশন সীমা অতিক্রান্ত হয়েছে।"
              : "Workstation device quota limit exceeded on host."
          );
        } else {
          setErrorMsg(
            res.error?.message ||
              (locale === "bn"
                ? "হোস্টে সংযোগ করা সম্ভব হয়নি।"
                : "Failed to connect to host.")
          );
        }
      }
    } catch (e: unknown) {
      setErrorMsg(e instanceof Error ? e.message : "Connection failed");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md w-full">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-primary/10 text-primary">
              <Laptop className="h-5 w-5" />
            </div>
            <div>
              <DialogTitle>
                {locale === "bn"
                  ? "ওয়ার্কস্টেশন হিসেবে যুক্ত হন"
                  : "Connect as Workstation"}
              </DialogTitle>
              <DialogDescription className="text-xs">
                {locale === "bn"
                  ? "প্রধান পিসির সাথে সুরক্ষিত TLS 1.3 এর মাধ্যমে এই কম্পিউটারটি যুক্ত করুন।"
                  : "Connect this PC to your primary Coachify server over secure local TLS 1.3."}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        {/* Auto-discovered hosts */}
        {discoveredHosts.length > 0 && (
          <div className="space-y-1.5 p-2.5 bg-muted/50 rounded-lg border text-xs">
            <div className="flex items-center justify-between text-muted-foreground font-medium text-[11px]">
              <span className="flex items-center gap-1">
                <Wifi className="h-3 w-3 text-primary" />
                {locale === "bn"
                  ? "পাওয়া গেছে এমন সার্ভার:"
                  : "Discovered Host Servers:"}
              </span>
              <button
                type="button"
                onClick={discoverHosts}
                disabled={discovering}
                className="hover:text-foreground text-[10px] underline cursor-pointer"
              >
                {discovering
                  ? locale === "bn"
                    ? "খোঁজা হচ্ছে..."
                    : "Scanning..."
                  : locale === "bn"
                    ? "রিফ্রেশ"
                    : "Refresh"}
              </button>
            </div>
            <div className="space-y-1">
              {discoveredHosts.map((h, i) => (
                <button
                  key={i}
                  type="button"
                  onClick={() => {
                    setHostIp(h.ip);
                    setHostPort(h.port);
                  }}
                  className="w-full flex items-center justify-between p-1.5 rounded hover:bg-card border border-transparent hover:border-border text-left cursor-pointer transition-colors"
                >
                  <div className="font-semibold">{h.name || h.ip}</div>
                  <div className="font-mono text-muted-foreground text-[10px]">
                    {h.ip}:{h.port}
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-3 pt-1">
          {errorMsg && (
            <div className="p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span>{errorMsg}</span>
            </div>
          )}

          {successMsg && (
            <div className="p-2.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300 text-xs flex items-center gap-2 font-medium">
              <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-600" />
              <span>{successMsg}</span>
            </div>
          )}

          <div className="space-y-1">
            <label className="text-xs font-semibold">
              {locale === "bn" ? "হোস্ট আইপি অ্যাড্রেস" : "Host IP Address"}
            </label>
            <div className="flex gap-2">
              <Input
                placeholder="192.168.1.50"
                value={hostIp}
                onChange={(e) => setHostIp(e.target.value)}
                disabled={submitting}
                className="font-mono text-xs flex-1"
                required
              />
              <Input
                type="number"
                placeholder="50443"
                value={hostPort}
                onChange={(e) => setHostPort(Number(e.target.value))}
                disabled={submitting}
                className="font-mono text-xs w-24"
              />
            </div>
          </div>

          <div className="space-y-1">
            <label className="text-xs font-semibold">
              {locale === "bn" ? "৬-সংখ্যার পেয়ারিং কোড" : "6-Digit Pairing Code"}
            </label>
            <div className="relative">
              <Input
                type="text"
                maxLength={6}
                placeholder="123456"
                value={pairingCode}
                onChange={(e) => setPairingCode(e.target.value.replace(/\D/g, ""))}
                disabled={submitting}
                className="font-mono tracking-widest text-center text-sm font-bold pl-8"
                required
              />
              <KeyRound className="h-4 w-4 text-muted-foreground absolute left-2.5 top-2.5" />
            </div>
            <p className="text-[11px] text-muted-foreground">
              {locale === "bn"
                ? "প্রধান পিসির সেটিংস > ডিভাইস > নতুন ওয়ার্কস্টেশন যুক্ত করুন এ কোডটি পাবেন।"
                : "Obtain this 6-digit code on the Host PC under Settings > Devices > Pair New Workstation."}
            </p>
          </div>

          <div className="space-y-1">
            <label className="text-xs font-semibold">
              {locale === "bn" ? "এই ওয়ার্কস্টেশনের নাম" : "Workstation / Counter Name"}
            </label>
            <Input
              placeholder="Front Desk Counter 1"
              value={workstationName}
              onChange={(e) => setWorkstationName(e.target.value)}
              disabled={submitting}
              className="text-xs"
              required
            />
          </div>

          <DialogFooter className="pt-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={submitting}
              className="text-xs"
            >
              {locale === "bn" ? "বাতিল" : "Cancel"}
            </Button>
            <Button
              type="submit"
              disabled={submitting}
              className="text-xs font-semibold gap-1.5"
            >
              {submitting ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Server className="h-3.5 w-3.5" />
              )}
              {locale === "bn" ? "সংযুক্ত করুন" : "Pair & Connect"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
