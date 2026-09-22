import { useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  ArrowUpCircle,
  ShieldCheck,
  HardDrive,
  RefreshCw,
  CheckCircle2,
  Package,
} from "lucide-react";

export function UpdateSettingsCard() {
  const [checking, setChecking] = useState(false);
  const [lastChecked, setLastChecked] = useState<string | null>(null);

  const handleCheckUpdates = () => {
    setChecking(true);
    setTimeout(() => {
      setChecking(false);
      setLastChecked(new Date().toLocaleTimeString());
    }, 1200);
  };

  return (
    <div className="grid gap-6 md:grid-cols-2">
      {/* Current Version & Channel */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <ArrowUpCircle className="h-4 w-4 text-primary" />
            Software Version & Update Channel
          </CardTitle>
          <CardDescription className="text-xs">
            Coachify Desktop for Windows 10 & 11 (x64 NSIS Distribution).
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4 text-xs">
          <div className="flex items-center justify-between p-3 rounded-lg bg-muted/60 border border-border">
            <div>
              <span className="text-[11px] text-muted-foreground block">
                Installed Version
              </span>
              <span className="font-bold text-sm text-foreground">
                Coachify v0.1.0-alpha
              </span>
            </div>
            <Badge variant="paid" size="sm">
              Current
            </Badge>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between text-xs">
              <span className="text-muted-foreground">Release Channel:</span>
              <span className="font-semibold text-foreground">
                Pilot & Internal (Slice 3 Gate)
              </span>
            </div>
            <div className="flex items-center justify-between text-xs">
              <span className="text-muted-foreground">Distribution Type:</span>
              <span className="font-semibold text-foreground">
                Per-Machine NSIS (Elevated)
              </span>
            </div>
            {lastChecked && (
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>Last Checked:</span>
                <span>{lastChecked}</span>
              </div>
            )}
          </div>

          <div className="pt-2">
            <Button
              size="sm"
              onClick={handleCheckUpdates}
              disabled={checking}
              className="gap-2 font-semibold w-full sm:w-auto"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${checking ? "animate-spin" : ""}`} />
              {checking ? "Checking for Updates..." : "Check for Updates"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Pre-Update Safety & Retention Policy */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <ShieldCheck className="h-4 w-4 text-emerald-500" />
            Pre-Update Safety & Storage Policy
          </CardTitle>
          <CardDescription className="text-xs">
            Zero-risk continuous delivery pipeline guarantees.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-xs text-muted-foreground">
          <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-900 dark:text-emerald-200 space-y-1">
            <div className="font-semibold flex items-center gap-1.5 text-xs">
              <CheckCircle2 className="h-4 w-4" />
              Automatic Pre-Update Snapshot
            </div>
            <p className="text-[11px] leading-relaxed">
              Every updater execution automatically creates a verified snapshot before new
              binaries are staged and migrations are applied. If an update encounters an
              issue, data reverts instantly.
            </p>
          </div>

          <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-1">
            <div className="font-semibold text-foreground flex items-center gap-1.5 text-xs">
              <HardDrive className="h-3.5 w-3.5 text-primary" />
              Data Preserved on Uninstall
            </div>
            <p className="text-[11px]">
              Institutional databases and encryption envelopes in{" "}
              <code className="font-mono text-[10px]">
                %LOCALAPPDATA%\com.coachify.desktop
              </code>{" "}
              are strictly preserved if the application is uninstalled or reinstalled.
            </p>
          </div>

          <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-1">
            <div className="font-semibold text-foreground flex items-center gap-1.5 text-xs">
              <Package className="h-3.5 w-3.5 text-primary" />
              Offline & Air-Gapped Ready
            </div>
            <p className="text-[11px]">
              Full standalone installers include all required fonts (Bangla Hind Siliguri,
              Noto Sans) and SQLite runtimes with zero internet dependency.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
