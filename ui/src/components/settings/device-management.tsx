import { useCallback, useEffect, useState } from "react";
import QRCode from "qrcode";
import {
  Laptop,
  Server,
  Plus,
  RefreshCw,
  MoreVertical,
  Edit2,
  Ban,
  CheckCircle2,
  Trash2,
  History,
  Copy,
  Check,
  AlertTriangle,
  Clock,
  Shield,
  ShieldAlert,
  Key,
  Activity,
  Users,
  Smartphone,
  Tablet,
  QrCode,
} from "lucide-react";
import {
  rpc,
  type WorkstationSummary,
  type WorkstationQuota,
  type PairingWindowInfo,
  type MobilePairingWindowInfo,
  type DeviceEventItem,
} from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useLocale } from "@/lib/i18n";

export interface DeviceManagementProps {
  initialDevices?: WorkstationSummary[];
  initialQuota?: WorkstationQuota | null;
}

export function DeviceManagement({
  initialDevices = [],
  initialQuota = null,
}: DeviceManagementProps = {}) {
  const { t } = useLocale();

  const [devices, setDevices] = useState<WorkstationSummary[]>(initialDevices);
  const [quota, setQuota] = useState<WorkstationQuota | null>(initialQuota);
  const [loading, setLoading] = useState(initialDevices.length === 0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Pairing Modal State
  const [pairingModalOpen, setPairingModalOpen] = useState(false);
  const [pairingInfo, setPairingInfo] = useState<PairingWindowInfo | null>(null);
  const [pairingLoading, setPairingLoading] = useState(false);
  const [copiedCode, setCopiedCode] = useState(false);
  const [countdownSeconds, setCountdownSeconds] = useState<number>(0);

  // Mobile Pairing Modal State
  const [mobilePairingModalOpen, setMobilePairingModalOpen] = useState(false);
  const [mobilePairingInfo, setMobilePairingInfo] =
    useState<MobilePairingWindowInfo | null>(null);
  const [mobilePairingLoading, setMobilePairingLoading] = useState(false);
  const [mobileCountdown, setMobileCountdown] = useState<number>(0);
  const [mobileQrDataUrl, setMobileQrDataUrl] = useState<string>("");

  // Rename Dialog State
  const [renameTarget, setRenameTarget] = useState<WorkstationSummary | null>(null);
  const [newName, setNewName] = useState("");
  const [renameLoading, setRenameLoading] = useState(false);

  // Block Dialog State
  const [blockTarget, setBlockTarget] = useState<WorkstationSummary | null>(null);
  const [blockReason, setBlockReason] = useState("");
  const [blockLoading, setBlockLoading] = useState(false);

  // Revoke Dialog State
  const [revokeTarget, setRevokeTarget] = useState<WorkstationSummary | null>(null);
  const [revokeReason, setRevokeReason] = useState("");
  const [revokeLoading, setRevokeLoading] = useState(false);

  // Activity History Dialog State
  const [activityTarget, setActivityTarget] = useState<WorkstationSummary | null>(null);
  const [eventsList, setEventsList] = useState<DeviceEventItem[]>([]);
  const [eventsLoading, setEventsLoading] = useState(false);

  const fetchWorkstations = useCallback(async () => {
    try {
      setLoading(true);
      const res = await rpc("lan.list_workstations", {});
      if (res.ok && res.data) {
        setDevices(res.data.devices);
        setQuota(res.data.quota);
        setErrorMsg(null);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchWorkstations();
  }, [fetchWorkstations]);

  // Countdown timer for pairing code
  useEffect(() => {
    if (!pairingModalOpen || countdownSeconds <= 0) return;
    const interval = setInterval(() => {
      setCountdownSeconds((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [pairingModalOpen, countdownSeconds]);

  // Open Pairing Window Handlers
  const handleOpenPairing = async () => {
    try {
      setPairingLoading(true);
      setErrorMsg(null);
      const res = await rpc("lan.open_pairing_window", { duration_seconds: 600 });
      if (res.ok && res.data) {
        setPairingInfo(res.data);
        setCountdownSeconds(res.data.expires_in_seconds || 600);
        setPairingModalOpen(true);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setPairingLoading(false);
    }
  };

  const handleClosePairing = async () => {
    try {
      await rpc("lan.close_pairing_window", {});
    } catch {
      // Ignore
    } finally {
      setPairingModalOpen(false);
      setPairingInfo(null);
      fetchWorkstations();
    }
  };

  useEffect(() => {
    if (!mobilePairingInfo?.qr_uri) {
      setMobileQrDataUrl("");
      return;
    }
    QRCode.toDataURL(mobilePairingInfo.qr_uri, {
      width: 256,
      margin: 1,
      color: { dark: "#0f172a", light: "#ffffff" },
      errorCorrectionLevel: "M",
    })
      .then((url) => setMobileQrDataUrl(url))
      .catch((err) => console.error("QR render error:", err));
  }, [mobilePairingInfo?.qr_uri]);

  useEffect(() => {
    if (!mobilePairingModalOpen || mobileCountdown <= 0) return;
    const interval = setInterval(() => {
      setMobileCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          handleOpenMobilePairing();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [mobilePairingModalOpen, mobileCountdown]);

  const handleOpenMobilePairing = async () => {
    try {
      setMobilePairingLoading(true);
      setErrorMsg(null);
      const res = await rpc("lan.open_mobile_pairing_window", { duration_seconds: 600 });
      if (res.ok && res.data) {
        setMobilePairingInfo(res.data);
        setMobileCountdown(res.data.expires_in_seconds || 600);
        setMobilePairingModalOpen(true);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setMobilePairingLoading(false);
    }
  };

  const handleCloseMobilePairing = async () => {
    try {
      await rpc("lan.close_pairing_window", {});
    } catch {
      // Ignore
    } finally {
      setMobilePairingModalOpen(false);
      setMobilePairingInfo(null);
      fetchWorkstations();
    }
  };

  const getDeviceIcon = (dev: WorkstationSummary) => {
    if (dev.is_host) return <Server className="h-4 w-4 text-primary shrink-0" />;
    const platform = (dev.platform || "").toLowerCase();
    const type = (dev.device_type || "").toLowerCase();
    if (platform === "android" || platform === "ios" || type === "mobile_phone") {
      return <Smartphone className="h-4 w-4 text-emerald-500 shrink-0" />;
    }
    if (type === "tablet") {
      return <Tablet className="h-4 w-4 text-indigo-500 shrink-0" />;
    }
    return <Laptop className="h-4 w-4 text-muted-foreground shrink-0" />;
  };

  const handleCopyCode = (code: string) => {
    navigator.clipboard.writeText(code);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 2000);
  };

  // Rename Workstation Handlers
  const handleRename = async () => {
    if (!renameTarget || !newName.trim()) return;
    try {
      setRenameLoading(true);
      const res = await rpc("lan.rename_workstation", {
        device_id: renameTarget.id,
        new_name: newName.trim(),
      });
      if (res.ok) {
        setRenameTarget(null);
        setNewName("");
        setSuccessMsg(`Workstation renamed to "${newName.trim()}"`);
        setTimeout(() => setSuccessMsg(null), 3000);
        fetchWorkstations();
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setRenameLoading(false);
    }
  };

  // Block Workstation Handlers
  const handleBlock = async () => {
    if (!blockTarget) return;
    try {
      setBlockLoading(true);
      const res = await rpc("lan.block_workstation", {
        device_id: blockTarget.id,
        reason: blockReason.trim() ? blockReason.trim() : null,
      });
      if (res.ok) {
        setBlockTarget(null);
        setBlockReason("");
        setSuccessMsg(`Workstation "${blockTarget.name}" has been blocked.`);
        setTimeout(() => setSuccessMsg(null), 3000);
        fetchWorkstations();
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setBlockLoading(false);
    }
  };

  // Unblock Workstation Handlers
  const handleUnblock = async (device: WorkstationSummary) => {
    try {
      const res = await rpc("lan.unblock_workstation", { device_id: device.id });
      if (res.ok) {
        setSuccessMsg(`Workstation "${device.name}" unblocked successfully.`);
        setTimeout(() => setSuccessMsg(null), 3000);
        fetchWorkstations();
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    }
  };

  // Revoke Workstation Handlers
  const handleRevoke = async () => {
    if (!revokeTarget) return;
    try {
      setRevokeLoading(true);
      const res = await rpc("lan.revoke_workstation", {
        device_id: revokeTarget.id,
        reason: revokeReason.trim() ? revokeReason.trim() : null,
      });
      if (res.ok) {
        setRevokeTarget(null);
        setRevokeReason("");
        setSuccessMsg(`Workstation "${revokeTarget.name}" permanently revoked.`);
        setTimeout(() => setSuccessMsg(null), 3000);
        fetchWorkstations();
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setRevokeLoading(false);
    }
  };

  // Activity History Handlers
  const handleViewEvents = async (device: WorkstationSummary) => {
    setActivityTarget(device);
    setEventsLoading(true);
    setEventsList([]);
    try {
      const res = await rpc("lan.list_device_events", {
        device_id: device.id,
        limit: 50,
      });
      if (res.ok && res.data) {
        setEventsList(res.data);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setEventsLoading(false);
    }
  };

  const formatCountdown = (secs: number) => {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  };

  const isQuotaFull =
    quota?.workstation_limit !== null &&
    quota?.workstation_limit !== undefined &&
    quota.active_workstations >= quota.workstation_limit;

  const quotaPercent =
    quota?.workstation_limit && quota.workstation_limit > 0
      ? Math.min(100, (quota.active_workstations / quota.workstation_limit) * 100)
      : 0;

  return (
    <div className="space-y-6" data-testid="device-management">
      {/* Alert Banners */}
      {errorMsg && (
        <div
          data-testid="error-banner"
          className="flex items-center justify-between rounded-md bg-destructive/15 p-3 text-sm text-destructive border border-destructive/30"
        >
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            <span>{errorMsg}</span>
          </div>
          <Button variant="ghost" size="sm" onClick={() => setErrorMsg(null)}>
            Dismiss
          </Button>
        </div>
      )}

      {successMsg && (
        <div
          data-testid="success-banner"
          className="flex items-center justify-between rounded-md bg-emerald-500/15 p-3 text-sm text-emerald-600 dark:text-emerald-400 border border-emerald-500/30"
        >
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-4 w-4" />
            <span>{successMsg}</span>
          </div>
          <Button variant="ghost" size="sm" onClick={() => setSuccessMsg(null)}>
            Dismiss
          </Button>
        </div>
      )}

      {/* Quota & Actions Card */}
      <Card>
        <CardHeader>
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div>
              <CardTitle className="flex items-center gap-2 text-base">
                <Laptop className="h-5 w-5 text-primary" />
                {t("devices.title")}
              </CardTitle>
              <CardDescription>{t("devices.subtitle")}</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={fetchWorkstations}
                disabled={loading}
                className="gap-1.5"
              >
                <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
                {t("action.refresh")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleOpenMobilePairing}
                disabled={isQuotaFull || mobilePairingLoading}
                className="gap-1.5 border-primary/40 text-primary hover:bg-primary/10"
                data-testid="pair-mobile-button"
              >
                <QrCode className="h-3.5 w-3.5" />
                {t("devices.pairMobile")}
              </Button>
              <Button
                size="sm"
                onClick={handleOpenPairing}
                disabled={isQuotaFull || pairingLoading}
                className="gap-1.5"
                data-testid="pair-workstation-button"
              >
                <Plus className="h-3.5 w-3.5" />
                {t("devices.pairNew")}
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {quota && (
            <div className="rounded-lg border border-border p-4 bg-muted/30 space-y-3">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-xs">
                <div className="flex items-center gap-2 font-medium">
                  <Users className="h-4 w-4 text-primary" />
                  <span>{t("devices.seatsUsed")}</span>
                  <Badge variant="outline" className="text-xs">
                    {quota.license_tier}
                  </Badge>
                </div>
                <div className="text-muted-foreground font-medium">
                  {quota.workstation_limit !== null &&
                  quota.workstation_limit !== undefined
                    ? t("devices.quota", {
                        used: quota.active_workstations.toString(),
                        limit: quota.workstation_limit.toString(),
                      })
                    : t("devices.unlimited", {
                        used: quota.active_workstations.toString(),
                      })}
                </div>
              </div>
              {quota.workstation_limit !== null &&
                quota.workstation_limit !== undefined && (
                  <Progress
                    value={quotaPercent}
                    className="h-2"
                    indicatorClassName={
                      isQuotaFull
                        ? "bg-destructive"
                        : quotaPercent > 80
                          ? "bg-amber-500"
                          : "bg-primary"
                    }
                  />
                )}
              {isQuotaFull && (
                <p className="text-xs text-destructive font-medium flex items-center gap-1.5">
                  <ShieldAlert className="h-3.5 w-3.5" />
                  Workstation limit reached for your license tier. Please upgrade license
                  to pair additional workstations.
                </p>
              )}
            </div>
          )}

          {/* Workstations Inventory Table */}
          <div className="rounded-md border border-border overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("devices.tableName")}</TableHead>
                  <TableHead>{t("devices.tableWindowsName")}</TableHead>
                  <TableHead>{t("devices.tableIp")}</TableHead>
                  <TableHead>{t("devices.tableStatus")}</TableHead>
                  <TableHead>{t("devices.tablePairedAt")}</TableHead>
                  <TableHead>{t("devices.tableLastSeen")}</TableHead>
                  <TableHead className="text-center">
                    {t("devices.tableActiveSessions")}
                  </TableHead>
                  <TableHead className="text-right">
                    {t("devices.tableActions")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {devices.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={8}
                      className="text-center py-8 text-muted-foreground text-xs"
                    >
                      No devices paired yet.
                    </TableCell>
                  </TableRow>
                ) : (
                  devices.map((dev) => (
                    <TableRow key={dev.id} data-testid={`device-row-${dev.id}`}>
                      <TableCell className="font-medium text-xs">
                        <div className="flex items-center gap-2">
                          {getDeviceIcon(dev)}
                          <span className="truncate max-w-[140px]">{dev.name}</span>
                          {dev.is_host && (
                            <Badge
                              variant="outline"
                              className="text-[10px] bg-primary/10 text-primary border-primary/20"
                            >
                              {t("devices.host")}
                            </Badge>
                          )}
                          {dev.paired_via === "qr" && (
                            <Badge
                              variant="secondary"
                              className="text-[10px] font-mono uppercase bg-emerald-500/10 text-emerald-600 border border-emerald-500/20"
                              data-testid={`badge-paired-via-${dev.id}`}
                            >
                              QR
                            </Badge>
                          )}
                          {dev.platform && (
                            <Badge
                              variant="outline"
                              className="text-[10px] font-medium capitalize"
                              data-testid={`badge-platform-${dev.id}`}
                            >
                              {dev.platform}
                            </Badge>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="text-xs font-mono text-muted-foreground">
                        {dev.windows_name}
                      </TableCell>
                      <TableCell className="text-xs font-mono text-muted-foreground">
                        {dev.ip_address || "—"}
                      </TableCell>
                      <TableCell>
                        {dev.status === "revoked" ? (
                          <Badge
                            variant="outline"
                            className="text-[10px] bg-destructive/10 text-destructive border-destructive/30"
                          >
                            {dev.status}
                          </Badge>
                        ) : dev.status === "blocked" ? (
                          <Badge
                            variant="outline"
                            className="text-[10px] bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30"
                          >
                            {dev.status}
                          </Badge>
                        ) : dev.is_online ? (
                          <Badge
                            variant="outline"
                            className="text-[10px] bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30 flex items-center gap-1 w-fit"
                          >
                            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 inline-block animate-pulse" />
                            {t("devices.online")}
                          </Badge>
                        ) : (
                          <Badge
                            variant="secondary"
                            className="text-[10px] flex items-center gap-1 w-fit text-muted-foreground"
                          >
                            <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground inline-block" />
                            {t("devices.offline")}
                          </Badge>
                        )}
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {dev.paired_at ? dev.paired_at.substring(0, 10) : "—"}
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {dev.last_seen_at ? dev.last_seen_at.substring(11, 19) : "—"}
                      </TableCell>
                      <TableCell className="text-xs text-center font-semibold">
                        {dev.active_sessions_count}
                      </TableCell>
                      <TableCell className="text-right">
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-8 w-8 p-0"
                              data-testid={`actions-menu-${dev.id}`}
                            >
                              <MoreVertical className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end" className="text-xs">
                            <DropdownMenuItem
                              onClick={() => handleViewEvents(dev)}
                              className="gap-2 cursor-pointer"
                            >
                              <History className="h-3.5 w-3.5 text-primary" />
                              {t("devices.viewEvents")}
                            </DropdownMenuItem>

                            {!dev.is_host && dev.status !== "revoked" && (
                              <>
                                <DropdownMenuItem
                                  onClick={() => {
                                    setRenameTarget(dev);
                                    setNewName(dev.name);
                                  }}
                                  className="gap-2 cursor-pointer"
                                >
                                  <Edit2 className="h-3.5 w-3.5 text-muted-foreground" />
                                  {t("devices.rename")}
                                </DropdownMenuItem>

                                <DropdownMenuSeparator />

                                {dev.status === "blocked" ? (
                                  <DropdownMenuItem
                                    onClick={() => handleUnblock(dev)}
                                    className="gap-2 cursor-pointer text-emerald-600"
                                  >
                                    <CheckCircle2 className="h-3.5 w-3.5" />
                                    {t("devices.unblock")}
                                  </DropdownMenuItem>
                                ) : (
                                  <DropdownMenuItem
                                    onClick={() => {
                                      setBlockTarget(dev);
                                      setBlockReason("");
                                    }}
                                    className="gap-2 cursor-pointer text-amber-600"
                                  >
                                    <Ban className="h-3.5 w-3.5" />
                                    {t("devices.block")}
                                  </DropdownMenuItem>
                                )}

                                <DropdownMenuItem
                                  onClick={() => {
                                    setRevokeTarget(dev);
                                    setRevokeReason("");
                                  }}
                                  className="gap-2 cursor-pointer text-destructive"
                                >
                                  <Trash2 className="h-3.5 w-3.5" />
                                  {t("devices.revoke")}
                                </DropdownMenuItem>
                              </>
                            )}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      {/* ------------------------------------------------------------- */}
      {/* 1. Pairing Code Modal (AC-1, AC-2)                            */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={pairingModalOpen}
        onOpenChange={(open) => !open && handleClosePairing()}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Key className="h-5 w-5 text-primary" />
              {t("devices.pairingModalTitle")}
            </DialogTitle>
            <DialogDescription>{t("devices.pairingModalDesc")}</DialogDescription>
          </DialogHeader>

          {pairingInfo && (
            <div className="space-y-5 py-3">
              {/* Big 6-Digit Display */}
              <div className="flex flex-col items-center justify-center p-6 bg-muted/50 rounded-xl border border-border">
                <span className="text-4xl font-mono font-bold tracking-[0.4em] text-primary select-all">
                  {pairingInfo.code}
                </span>
                <div className="flex items-center gap-1.5 mt-4 text-xs font-medium text-muted-foreground">
                  <Clock className="h-3.5 w-3.5 animate-pulse text-amber-500" />
                  <span>
                    {t("devices.codeExpiresIn")}:{" "}
                    <strong className="text-foreground">
                      {formatCountdown(countdownSeconds)}
                    </strong>
                  </span>
                </div>
              </div>

              {/* Host Fingerprint */}
              <div className="space-y-1.5 text-xs">
                <label className="text-muted-foreground font-medium flex items-center gap-1">
                  <Shield className="h-3.5 w-3.5 text-primary" />
                  {t("devices.hostFingerprint")}
                </label>
                <div className="p-2.5 rounded bg-muted/60 font-mono text-[11px] break-all border border-border select-all">
                  {pairingInfo.host_fingerprint}
                </div>
              </div>

              <div className="flex items-center justify-between text-xs text-muted-foreground pt-1">
                <span className="flex items-center gap-1.5">
                  <Activity className="h-3.5 w-3.5 text-emerald-500" />
                  Listening on port 50443 (TLS 1.3)
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleCopyCode(pairingInfo.code)}
                  className="gap-1.5"
                >
                  {copiedCode ? (
                    <>
                      <Check className="h-3.5 w-3.5 text-emerald-500" />
                      {t("devices.copied")}
                    </>
                  ) : (
                    <>
                      <Copy className="h-3.5 w-3.5" />
                      {t("devices.copyCode")}
                    </>
                  )}
                </Button>
              </div>
            </div>
          )}

          <DialogFooter>
            <Button variant="secondary" onClick={handleClosePairing}>
              {t("devices.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ------------------------------------------------------------- */}
      {/* 1b. Mobile QR Pairing Modal (Spec 0046, AC-1)                */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={mobilePairingModalOpen}
        onOpenChange={(open) => !open && handleCloseMobilePairing()}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <QrCode className="h-5 w-5 text-primary" />
              {t("devices.mobilePairingModalTitle")}
            </DialogTitle>
            <DialogDescription>{t("devices.mobilePairingModalDesc")}</DialogDescription>
          </DialogHeader>

          {mobilePairingInfo && (
            <div className="space-y-4 py-2">
              {/* QR Code Canvas/Image */}
              <div className="flex flex-col items-center justify-center p-4 bg-white dark:bg-white rounded-xl shadow-inner border border-border">
                {mobileQrDataUrl ? (
                  <img
                    src={mobileQrDataUrl}
                    alt="Pairing QR Code"
                    className="w-56 h-56 object-contain"
                    data-testid="mobile-qr-image"
                  />
                ) : (
                  <div className="w-56 h-56 flex items-center justify-center text-muted-foreground">
                    <RefreshCw className="h-8 w-8 animate-spin" />
                  </div>
                )}
                <div className="mt-2 text-center text-xs text-slate-700 font-semibold">
                  {mobilePairingInfo.institution_name}
                </div>
                <div className="text-[11px] text-slate-500 font-mono">
                  {mobilePairingInfo.host_ips.join(", ")}:{mobilePairingInfo.port}
                </div>
              </div>

              {/* Countdown Alert */}
              <div className="flex items-center justify-between rounded-lg border border-border/70 bg-muted/40 px-3 py-2 text-xs">
                <span className="flex items-center gap-1.5 text-muted-foreground font-medium">
                  <Clock className="h-3.5 w-3.5 text-primary" />
                  {t("devices.codeExpiresIn")}
                </span>
                <span
                  className={`font-mono font-bold ${
                    mobileCountdown < 60
                      ? "text-destructive animate-pulse"
                      : "text-primary"
                  }`}
                >
                  {formatCountdown(mobileCountdown)}
                </span>
              </div>

              {/* Fallback 6-digit Code */}
              <div className="rounded-lg border border-border bg-card p-3 space-y-1.5">
                <div className="text-xs text-muted-foreground font-medium">
                  {t("devices.enterCode")}
                </div>
                <div className="flex items-center justify-between bg-muted/60 p-2 rounded border border-border/80">
                  <span className="font-mono text-xl font-bold tracking-widest text-primary pl-2 select-all">
                    {mobilePairingInfo.code}
                  </span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleCopyCode(mobilePairingInfo.code)}
                    className="gap-1 text-xs"
                  >
                    {copiedCode ? (
                      <Check className="h-3.5 w-3.5 text-emerald-500" />
                    ) : (
                      <Copy className="h-3.5 w-3.5" />
                    )}
                    {copiedCode ? t("devices.copied") : t("devices.copyCode")}
                  </Button>
                </div>
              </div>

              {/* Web / Local Testing Companion Launcher */}
              <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 space-y-2">
                <div className="flex items-center justify-between">
                  <div className="text-xs font-semibold text-primary flex items-center gap-1.5">
                    <Smartphone className="h-3.5 w-3.5" />
                    Mobile Companion Test (Browser / LAN)
                  </div>
                  <Badge
                    variant="outline"
                    className="text-[10px] bg-background text-emerald-600 border-emerald-500/30"
                  >
                    Ready to Test
                  </Badge>
                </div>
                <p className="text-[11px] text-muted-foreground">
                  Open on any smartphone browser on your Wi-Fi, or test directly in a
                  separate browser tab with one click.
                </p>
                <div className="flex items-center gap-2 pt-1">
                  <Button
                    size="sm"
                    variant="primary"
                    className="h-8 text-xs font-semibold flex-1 gap-1.5"
                    onClick={() => {
                      const pairUrl = `${window.location.origin}/?mode=mobile&pair=${encodeURIComponent(
                        mobilePairingInfo.qr_uri
                      )}`;
                      window.open(pairUrl, "_blank");
                    }}
                    data-testid="launch-mobile-companion-tab-btn"
                  >
                    <span>Launch Mobile View</span>
                    <span className="text-[10px] font-mono opacity-80">(New Tab)</span>
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    className="h-8 text-xs gap-1.5"
                    onClick={() => {
                      void navigator.clipboard.writeText(mobilePairingInfo.qr_uri);
                      setCopiedCode(true);
                      setTimeout(() => setCopiedCode(false), 2000);
                    }}
                    title="Copy QR Pairing URI"
                    data-testid="copy-qr-uri-btn"
                  >
                    <Copy className="h-3 w-3" />
                    <span>Copy URI</span>
                  </Button>
                </div>
              </div>

              {/* TLS Fingerprint */}
              <div className="space-y-1 text-xs">
                <label className="text-muted-foreground font-medium flex items-center gap-1">
                  <Shield className="h-3.5 w-3.5 text-primary" />
                  {t("devices.hostFingerprint")}
                </label>
                <div className="p-2 rounded bg-muted/60 font-mono text-[11px] break-all border border-border select-all">
                  {mobilePairingInfo.host_fingerprint}
                </div>
              </div>
            </div>
          )}

          <DialogFooter className="sm:justify-between">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleOpenMobilePairing}
              disabled={mobilePairingLoading}
              className="gap-1.5 text-xs"
            >
              <RefreshCw
                className={`h-3.5 w-3.5 ${mobilePairingLoading ? "animate-spin" : ""}`}
              />
              {t("action.refresh")}
            </Button>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={handleCloseMobilePairing}
            >
              {t("devices.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ------------------------------------------------------------- */}
      {/* 2. Rename Workstation Dialog (AC-6)                           */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={renameTarget !== null}
        onOpenChange={(open) => !open && setRenameTarget(null)}
      >
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>{t("devices.rename")}</DialogTitle>
            <DialogDescription>
              Rename workstation "{renameTarget?.name}".
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2">
            <label className="text-xs font-medium text-foreground">
              {t("devices.tableName")}
            </label>
            <Input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              maxLength={60}
              placeholder="e.g. Front Desk PC"
              className="text-xs"
            />
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setRenameTarget(null)}
              disabled={renameLoading}
            >
              {t("devices.cancel")}
            </Button>
            <Button
              size="sm"
              onClick={handleRename}
              disabled={renameLoading || !newName.trim()}
            >
              {t("devices.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ------------------------------------------------------------- */}
      {/* 3. Block Workstation Dialog (AC-7)                            */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={blockTarget !== null}
        onOpenChange={(open) => !open && setBlockTarget(null)}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-amber-600">
              <Ban className="h-5 w-5" />
              {t("devices.block")}
            </DialogTitle>
            <DialogDescription className="text-xs pt-1">
              {t("devices.confirmBlock")}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2">
            <label className="text-xs font-medium text-foreground">
              {t("devices.reason")} (Optional)
            </label>
            <Input
              value={blockReason}
              onChange={(e) => setBlockReason(e.target.value)}
              placeholder="e.g. Suspicious terminal activity"
              className="text-xs"
            />
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setBlockTarget(null)}
              disabled={blockLoading}
            >
              {t("devices.cancel")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleBlock}
              disabled={blockLoading}
            >
              {t("devices.block")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ------------------------------------------------------------- */}
      {/* 4. Revoke Workstation Dialog (AC-8)                           */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={revokeTarget !== null}
        onOpenChange={(open) => !open && setRevokeTarget(null)}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-destructive">
              <Trash2 className="h-5 w-5" />
              {t("devices.revoke")}
            </DialogTitle>
            <DialogDescription className="text-xs pt-1">
              {t("devices.confirmRevoke")}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2">
            <label className="text-xs font-medium text-foreground">
              {t("devices.reason")} (Optional)
            </label>
            <Input
              value={revokeReason}
              onChange={(e) => setRevokeReason(e.target.value)}
              placeholder="e.g. Workstation decommissioned"
              className="text-xs"
            />
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setRevokeTarget(null)}
              disabled={revokeLoading}
            >
              {t("devices.cancel")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleRevoke}
              disabled={revokeLoading}
            >
              {t("devices.revoke")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ------------------------------------------------------------- */}
      {/* 5. Device Activity Audit Trail Drawer/Modal (AC-10)           */}
      {/* ------------------------------------------------------------- */}
      <Dialog
        open={activityTarget !== null}
        onOpenChange={(open) => !open && setActivityTarget(null)}
      >
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <History className="h-5 w-5 text-primary" />
              {t("devices.activityTitle")}
            </DialogTitle>
            <DialogDescription className="text-xs">
              Audit log for <strong>{activityTarget?.name}</strong> (
              {activityTarget?.windows_name})
            </DialogDescription>
          </DialogHeader>

          <div className="overflow-y-auto flex-1 my-3 pr-1">
            {eventsLoading ? (
              <div className="py-8 text-center text-xs text-muted-foreground flex items-center justify-center gap-2">
                <RefreshCw className="h-4 w-4 animate-spin" />
                Loading activity history...
              </div>
            ) : eventsList.length === 0 ? (
              <div className="py-8 text-center text-xs text-muted-foreground">
                {t("devices.noEvents")}
              </div>
            ) : (
              <div className="space-y-2">
                {eventsList.map((ev) => (
                  <div
                    key={ev.id}
                    className="p-3 rounded-lg border border-border bg-card/60 space-y-1.5 text-xs"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 font-medium">
                        <Badge variant="outline" className="text-[10px] capitalize">
                          {ev.event_type}
                        </Badge>
                        <span className="text-foreground">
                          {ev.actor_user_name || ev.actor_user_id || "System"}
                        </span>
                      </div>
                      <span className="text-muted-foreground text-[11px] font-mono">
                        {ev.created_at}
                      </span>
                    </div>
                    {ev.details_json && ev.details_json !== "{}" && (
                      <div className="p-2 rounded bg-muted/40 font-mono text-[10px] break-all border border-border/50 text-muted-foreground">
                        {ev.details_json}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="secondary" onClick={() => setActivityTarget(null)}>
              {t("devices.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
