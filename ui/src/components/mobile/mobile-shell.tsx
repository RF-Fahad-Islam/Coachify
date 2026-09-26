import { useState, useEffect, useCallback } from "react";
import {
  QrCode,
  UserCheck,
  DollarSign,
  ShieldAlert,
  Wifi,
  WifiOff,
  RefreshCw,
  LogOut,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { MobilePairingScreen } from "./mobile-pairing-screen";
import { MobileAttendanceScanner } from "./mobile-attendance-scanner";
import { MobileCashierView } from "./mobile-cashier-view";

export interface MobileShellProps {
  initialHostIp?: string;
  initialHostPort?: number;
  initialDeviceToken?: string;
  initialHostFingerprint?: string;
  onSeverance?: (reason: string) => void;
}

export function MobileShell({
  initialHostIp,
  initialHostPort,
  initialDeviceToken,
  initialHostFingerprint,
  onSeverance,
}: MobileShellProps) {
  // Mobile Navigation Tab: 'scanner' | 'cashier' | 'pairing'
  const [activeTab, setActiveTab] = useState<"scanner" | "cashier" | "pairing">(
    initialDeviceToken ? "scanner" : "pairing"
  );

  const [deviceToken, setDeviceToken] = useState<string | null>(
    initialDeviceToken || null
  );
  const [hostIp, setHostIp] = useState<string | null>(initialHostIp || null);
  const [hostPort, setHostPort] = useState<number | null>(initialHostPort || null);
  const [hostFingerprint, setHostFingerprint] = useState<string | null>(
    initialHostFingerprint || null
  );

  // Connection & Remote Revocation States
  const [isConnected, setIsConnected] = useState(true);
  const [isSevered, setIsSevered] = useState(false);
  const [severanceReason, setSeveranceReason] = useState<string | null>(null);

  useEffect(() => {
    const handleOnline = () => setIsConnected(true);
    const handleOffline = () => setIsConnected(false);
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, []);

  // AC-9: Sub-500ms SSE control channel listener simulation & event handler
  const handleRemoteEvent = useCallback(
    (eventType: string, payload?: Record<string, unknown>) => {
      if (eventType === "device.revoked" || eventType === "device.blocked") {
        const reason =
          (payload?.reason as string) ||
          (eventType === "device.revoked"
            ? "Workstation device has been revoked by host administrator."
            : "Workstation device has been blocked by host.");

        setIsSevered(true);
        setSeveranceReason(reason);
        setDeviceToken(null);
        onSeverance?.(reason);
      }
    },
    [onSeverance]
  );

  // Expose mock event listener to window for automated testing / integration
  useEffect(() => {
    if (typeof window !== "undefined") {
      (
        window as unknown as {
          __coachify_emit_mobile_event?: (type: string, data?: unknown) => void;
        }
      ).__coachify_emit_mobile_event = (type: string, data?: unknown) => {
        handleRemoteEvent(type, data as Record<string, unknown>);
      };
    }
  }, [handleRemoteEvent]);

  // If severed by host, show security severance lockdown screen
  if (isSevered) {
    return (
      <div
        data-testid="mobile-severance-screen"
        className="min-h-screen bg-slate-950 text-white flex flex-col items-center justify-center p-6 text-center"
      >
        <div className="w-16 h-16 rounded-full bg-destructive/20 text-destructive flex items-center justify-center mb-4 animate-pulse">
          <ShieldAlert className="w-10 h-10" />
        </div>
        <h2 className="text-xl font-bold tracking-tight text-destructive">
          Device Access Terminated
        </h2>
        <p className="text-xs text-slate-400 max-w-sm mt-2 font-medium">
          {severanceReason ||
            "This mobile workstation has been revoked or blocked by the LAN host."}
        </p>

        <div className="mt-6 p-3 rounded-lg bg-slate-900 border border-slate-800 text-left text-xs font-mono text-slate-400 space-y-1 w-full max-w-xs">
          <div>
            Status: <span className="text-destructive font-semibold">Severed</span>
          </div>
          <div>
            Credentials: <span className="text-slate-300">Purged</span>
          </div>
          {hostFingerprint && (
            <div className="truncate text-[10px]">
              Host FP: {hostFingerprint.substring(0, 16)}...
            </div>
          )}
          <div>Action: Re-pairing required</div>
        </div>

        <Button
          variant="outline"
          className="mt-6 border-slate-700 text-slate-300 hover:text-white"
          onClick={() => {
            setIsSevered(false);
            setSeveranceReason(null);
            setActiveTab("pairing");
          }}
          data-testid="re-pair-btn"
        >
          <RefreshCw className="w-4 h-4 mr-2" />
          Pair Again
        </Button>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col">
      {/* Top Mobile Status Header */}
      <header className="sticky top-0 z-20 border-b border-border bg-card/95 backdrop-blur px-4 py-2.5 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse" />
          <span className="font-bold text-xs tracking-tight">Coachify Mobile</span>
        </div>

        <div className="flex items-center gap-2">
          {isConnected ? (
            <Badge
              variant="outline"
              className="text-[10px] bg-emerald-500/10 text-emerald-600 border-emerald-500/30 gap-1 font-mono"
            >
              <Wifi className="w-3 h-3" />
              <span>{hostIp && hostPort ? `${hostIp}:${hostPort}` : "LAN Live"}</span>
            </Badge>
          ) : (
            <Badge
              variant="outline"
              className="text-[10px] bg-amber-500/10 text-amber-600 border-amber-500/30 gap-1 animate-pulse"
            >
              <WifiOff className="w-3 h-3" />
              <span>Reconnecting...</span>
            </Badge>
          )}

          {deviceToken && (
            <Button
              size="icon"
              variant="ghost"
              className="w-7 h-7 text-muted-foreground hover:text-destructive"
              onClick={() => {
                setDeviceToken(null);
                setActiveTab("pairing");
              }}
              title="Disconnect"
            >
              <LogOut className="w-3.5 h-3.5" />
            </Button>
          )}
        </div>
      </header>

      {/* Main Content Body */}
      <main className="flex-1 pb-16">
        {activeTab === "pairing" && (
          <MobilePairingScreen
            onPairingSuccess={(resp, parsed) => {
              setDeviceToken(resp.device_token);
              setHostIp(parsed.hosts[0]);
              setHostPort(parsed.port);
              setHostFingerprint(parsed.fingerprint);
            }}
            onStaffLoginSuccess={() => {
              setActiveTab("scanner");
            }}
          />
        )}

        {activeTab === "scanner" && <MobileAttendanceScanner />}

        {activeTab === "cashier" && <MobileCashierView />}
      </main>

      {/* Fixed Bottom Touch Navigation Bar */}
      <nav className="fixed bottom-0 left-0 right-0 z-20 border-t border-border bg-card/95 backdrop-blur flex justify-around py-2">
        <Button
          variant="ghost"
          size="sm"
          className={`flex-col h-auto py-1 px-3 gap-0.5 ${
            activeTab === "scanner" ? "text-primary font-bold" : "text-muted-foreground"
          }`}
          onClick={() => setActiveTab("scanner")}
          data-testid="tab-scanner-btn"
        >
          <UserCheck className="w-4 h-4" />
          <span className="text-[10px]">Attendance</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          className={`flex-col h-auto py-1 px-3 gap-0.5 ${
            activeTab === "cashier" ? "text-primary font-bold" : "text-muted-foreground"
          }`}
          onClick={() => setActiveTab("cashier")}
          data-testid="tab-cashier-btn"
        >
          <DollarSign className="w-4 h-4" />
          <span className="text-[10px]">Cashier</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          className={`flex-col h-auto py-1 px-3 gap-0.5 ${
            activeTab === "pairing" ? "text-primary font-bold" : "text-muted-foreground"
          }`}
          onClick={() => setActiveTab("pairing")}
          data-testid="tab-pairing-btn"
        >
          <QrCode className="w-4 h-4" />
          <span className="text-[10px]">Pairing</span>
        </Button>
      </nav>
    </div>
  );
}
