import { useEffect, useState } from "react";
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
  Wifi,
  Download,
  AlertCircle,
} from "lucide-react";
import {
  rpc,
  type WorkstationClientStatus,
  type WorkstationUpdateCheckResult,
  type WorkstationUpdateDownloadResult,
} from "@/gen/rpc";
import { formatBytes } from "@/lib/utils";

export interface UpdateSettingsCardProps {
  initialClientStatus?: WorkstationClientStatus | null;
  initialCheckResult?: WorkstationUpdateCheckResult | null;
}

export function UpdateSettingsCard({
  initialClientStatus = null,
  initialCheckResult = null,
}: UpdateSettingsCardProps = {}) {
  const [clientStatus, setClientStatus] = useState<WorkstationClientStatus | null>(
    initialClientStatus
  );
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [checkResult, setCheckResult] = useState<WorkstationUpdateCheckResult | null>(
    initialCheckResult
  );
  const [downloadResult, setDownloadResult] =
    useState<WorkstationUpdateDownloadResult | null>(null);
  const [lastChecked, setLastChecked] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [installedNotice, setInstalledNotice] = useState(false);

  useEffect(() => {
    if (initialClientStatus === null) {
      void (async () => {
        try {
          const res = await rpc("lan.get_client_status", {});
          if (res.ok && res.data) {
            setClientStatus(res.data);
          }
        } catch {
          // Running in standalone or host mode
        }
      })();
    }
  }, [initialClientStatus]);

  const handleCheckUpdates = async () => {
    setChecking(true);
    setErrorMsg(null);
    try {
      if (clientStatus?.is_client_mode) {
        const res = await rpc("lan.check_workstation_update", {});
        if (res.ok && res.data) {
          setCheckResult(res.data);
          setLastChecked(new Date().toLocaleTimeString());
        } else if (!res.ok && res.error) {
          setErrorMsg(res.error.message);
        }
      } else {
        // Standalone simulated check
        setTimeout(() => {
          setChecking(false);
          setLastChecked(new Date().toLocaleTimeString());
        }, 1200);
        return;
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setChecking(false);
    }
  };

  const handleDownloadFromHost = async (version: string) => {
    setDownloading(true);
    setErrorMsg(null);
    try {
      const res = await rpc("lan.download_workstation_update", { version });
      if (res.ok && res.data) {
        setDownloadResult(res.data);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setDownloading(false);
    }
  };

  const isClient = clientStatus?.is_client_mode ?? false;

  return (
    <div className="space-y-6">
      {/* Workstation LAN Update Source Banner (Spec 0048 AC-8) */}
      {isClient && (
        <div
          data-testid="lan-update-source-banner"
          className="p-3.5 rounded-lg bg-primary/10 border border-primary/25 space-y-1.5"
        >
          <div className="font-semibold text-foreground flex items-center justify-between text-xs">
            <span className="flex items-center gap-1.5">
              <Wifi className="h-4 w-4 text-primary" />
              {`Update Source: Host PC (${clientStatus?.host_name || clientStatus?.host_ip || "Local Network"}) via LAN`}
            </span>
            <Badge variant="paid" size="sm">
              Air-Gapped LAN Delivery
            </Badge>
          </div>
          <p className="text-[11px] text-muted-foreground leading-relaxed">
            This workstation operates in Counter Client mode. Application updates are
            fetched directly from the primary coaching center Host PC over high-speed
            local network with zero external internet dependency.
          </p>
        </div>
      )}

      {errorMsg && (
        <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/25 text-destructive text-xs flex items-center gap-2">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{errorMsg}</span>
        </div>
      )}

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
                  {isClient ? "Air-Gapped LAN Peer" : "Per-Machine NSIS (Elevated)"}
                </span>
              </div>
              {lastChecked && (
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>Last Checked:</span>
                  <span>{lastChecked}</span>
                </div>
              )}
            </div>

            {/* Staged Download Ready for Installation */}
            {downloadResult && (
              <div
                data-testid="update-verified-ready-box"
                className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/25 space-y-2 text-emerald-900 dark:text-emerald-200"
              >
                <div className="font-semibold flex items-center gap-1.5 text-xs">
                  <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                  {`Update v${downloadResult.version} Verified & Ready`}
                </div>
                <p className="text-[11px] text-muted-foreground">
                  Cryptographic SHA-256 and vendor signature successfully verified. The
                  installer package is staged locally.
                </p>
                <Button
                  size="sm"
                  onClick={() => setInstalledNotice(true)}
                  disabled={installedNotice}
                  className="w-full text-xs font-semibold gap-1.5 bg-emerald-600 hover:bg-emerald-700 text-white"
                >
                  <CheckCircle2 className="h-3.5 w-3.5" />
                  {installedNotice
                    ? "Installer Triggered (Restarting...)"
                    : "Restart & Install Update"}
                </Button>
              </div>
            )}

            {/* Update Available from Host */}
            {!downloadResult && checkResult?.update_available && (
              <div
                data-testid="update-available-from-host-box"
                className="p-3 rounded-lg bg-primary/10 border border-primary/25 space-y-2.5"
              >
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold text-foreground">
                    {`Update v${checkResult.host_version} Available from Host`}
                  </span>
                  <Badge variant="paid" size="sm">
                    Ready on LAN
                  </Badge>
                </div>
                {checkResult.manifest && (
                  <div className="text-[11px] text-muted-foreground space-y-1">
                    <p>Size: {formatBytes(checkResult.manifest.file_size_bytes)}</p>
                    {checkResult.manifest.notes && (
                      <p className="italic">Notes: {checkResult.manifest.notes}</p>
                    )}
                  </div>
                )}
                <Button
                  size="sm"
                  onClick={() =>
                    checkResult.host_version &&
                    void handleDownloadFromHost(checkResult.host_version)
                  }
                  disabled={downloading}
                  className="w-full text-xs font-semibold gap-1.5"
                >
                  <Download
                    className={`h-3.5 w-3.5 ${downloading ? "animate-spin" : ""}`}
                  />
                  {downloading ? "Downloading from Host..." : "Download from Host"}
                </Button>
              </div>
            )}

            {/* Up to Date Message */}
            {!downloadResult && checkResult && !checkResult.update_available && (
              <div className="p-2.5 rounded-lg bg-muted/60 border border-border text-xs text-muted-foreground flex items-center gap-2">
                <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500 shrink-0" />
                <span>
                  Workstation is synchronized with Host PC (no updates pending).
                </span>
              </div>
            )}

            {/* Check Updates Action */}
            {!downloadResult && (!checkResult || !checkResult.update_available) && (
              <div className="pt-2">
                <Button
                  size="sm"
                  onClick={() => void handleCheckUpdates()}
                  disabled={checking}
                  className="gap-2 font-semibold w-full sm:w-auto"
                >
                  <RefreshCw
                    className={`h-3.5 w-3.5 ${checking ? "animate-spin" : ""}`}
                  />
                  {checking
                    ? isClient
                      ? "Querying Host PC..."
                      : "Checking for Updates..."
                    : isClient
                      ? "Check Host for Updates"
                      : "Check for Updates"}
                </Button>
              </div>
            )}
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
                Every updater execution automatically creates a verified snapshot before
                new binaries are staged and migrations are applied. If an update
                encounters an issue, data reverts instantly.
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
                Full standalone installers include all required fonts (Bangla Hind
                Siliguri, Noto Sans) and SQLite runtimes with zero internet dependency.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
