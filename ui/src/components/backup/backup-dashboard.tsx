import { useCallback, useEffect, useState } from "react";
import {
  rpc,
  type BackupDashboard as BackupDashboardData,
  type BackupHistoryItem,
  type VerificationResult,
} from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { RestoreWizard } from "@/components/restore/restore-wizard";
import { formatBytes, formatBusinessDate } from "@/lib/utils";
import {
  ShieldCheck,
  ShieldAlert,
  ArchiveRestore,
  Camera,
  FolderPlus,
  RefreshCw,
  HardDrive,
  Clock,
  CheckCircle2,
  AlertTriangle,
  FileCheck,
  Trash2,
  Plus,
} from "lucide-react";

export function BackupDashboard() {
  const [dashboard, setDashboard] = useState<BackupDashboardData | null>(null);
  const [history, setHistory] = useState<BackupHistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Snapshot modal
  const [snapshotOpen, setSnapshotOpen] = useState(false);
  const [selectedDestId, setSelectedDestId] = useState<string>("");
  const [snapshotKind, setSnapshotKind] = useState<string>("manual");
  const [snapshotLoading, setSnapshotLoading] = useState(false);
  const [snapshotError, setSnapshotError] = useState<string | null>(null);

  // Add destination modal
  const [destModalOpen, setDestModalOpen] = useState(false);
  const [destPath, setDestPath] = useState("");
  const [destLabel, setDestLabel] = useState("");
  const [destKind, setDestKind] = useState("local_folder");
  const [destSaving, setDestSaving] = useState(false);
  const [destError, setDestError] = useState<string | null>(null);

  // Restore wizard
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [restoreFilePath, setRestoreFilePath] = useState<string | undefined>(undefined);

  // Verification result modal
  const [verifyModalOpen, setVerifyModalOpen] = useState(false);
  const [verifyResult, setVerifyResult] = useState<VerificationResult | null>(null);
  const [verifyingId, setVerifyingId] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [dashRes, histRes] = await Promise.all([
        rpc("backup.get_dashboard", {}),
        rpc("backup.list_history", { limit: 25 }),
      ]);

      if (dashRes.ok && dashRes.data) {
        setDashboard(dashRes.data);
        if (dashRes.data.destinations.length > 0 && !selectedDestId) {
          const defaultDest = dashRes.data.destinations.find((d) => d.is_default);
          setSelectedDestId(
            defaultDest ? defaultDest.id : dashRes.data.destinations[0].id
          );
        }
      } else if (dashRes.error) {
        setError(dashRes.error.message);
      }

      if (histRes.ok && histRes.data) {
        setHistory(histRes.data);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [selectedDestId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const handleCreateSnapshot = async () => {
    if (!selectedDestId) return;
    setSnapshotLoading(true);
    setSnapshotError(null);
    try {
      const res = await rpc("backup.create_snapshot", {
        destination_id: selectedDestId,
        kind: snapshotKind,
      });
      if (res.ok) {
        setSnapshotOpen(false);
        void loadData();
      } else {
        setSnapshotError(res.error?.message || "Failed to create snapshot.");
      }
    } catch (e: unknown) {
      setSnapshotError(e instanceof Error ? e.message : String(e));
    } finally {
      setSnapshotLoading(false);
    }
  };

  const handleVerify = async (backupId: string) => {
    setVerifyingId(backupId);
    try {
      const res = await rpc("backup.verify", { backup_id: backupId });
      if (res.ok && res.data) {
        setVerifyResult(res.data);
        setVerifyModalOpen(true);
        void loadData();
      }
    } catch (e: unknown) {
      console.error("Verification failed:", e);
    } finally {
      setVerifyingId(null);
    }
  };

  const handleSaveDestination = async () => {
    if (!destPath.trim() || !destLabel.trim()) return;
    setDestSaving(true);
    setDestError(null);
    try {
      const res = await rpc("backup.save_destination", {
        path: destPath.trim(),
        label: destLabel.trim(),
        kind: destKind,
      });
      if (res.ok) {
        setDestModalOpen(false);
        setDestPath("");
        setDestLabel("");
        void loadData();
      } else {
        setDestError(res.error?.message || "Failed to save backup destination.");
      }
    } catch (e: unknown) {
      setDestError(e instanceof Error ? e.message : String(e));
    } finally {
      setDestSaving(false);
    }
  };

  const handleRemoveDestination = async (id: string) => {
    try {
      const res = await rpc("backup.remove_destination", { destination_id: id });
      if (res.ok) {
        void loadData();
      }
    } catch (e: unknown) {
      console.error("Failed to remove destination:", e);
    }
  };

  const handleToggleSchedule = async (scheduleKind: string, currentEnabled: boolean) => {
    const existing = dashboard?.schedules.find((s) => s.kind === scheduleKind);
    try {
      await rpc("backup.save_schedule", {
        kind: scheduleKind,
        destination_id:
          existing?.destination_id || (dashboard?.destinations[0]?.id ?? null),
        time_of_day: existing?.time_of_day || "23:00",
        enabled: !currentEnabled,
      });
      void loadData();
    } catch (e: unknown) {
      console.error("Failed to toggle schedule:", e);
    }
  };

  if (loading && !dashboard) {
    return (
      <div className="p-8 text-center text-xs text-muted-foreground">
        <RefreshCw className="h-6 w-6 animate-spin mx-auto mb-2 text-primary" />
        Loading backup status and storage history...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Top Action Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            Backup & Snapshot Management
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Automated hot backups, SHA-256 HMAC manifests, integrity verification, and
            recovery.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => void loadData()}
            className="gap-1.5 h-8 text-xs font-semibold"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => {
              setRestoreFilePath(undefined);
              setRestoreOpen(true);
            }}
            className="gap-1.5 h-8 text-xs font-semibold text-foreground"
          >
            <ArchiveRestore className="h-3.5 w-3.5 text-primary" />
            Restore Database
          </Button>
          <Button
            size="sm"
            onClick={() => setSnapshotOpen(true)}
            className="gap-1.5 h-8 text-xs font-semibold"
            disabled={!dashboard?.destinations.length}
          >
            <Camera className="h-3.5 w-3.5" />
            Create Snapshot Now
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-3.5 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-xs">
          {error}
        </div>
      )}

      {/* Warnings Banner */}
      {dashboard && dashboard.warnings.length > 0 && (
        <div className="p-3.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 text-xs space-y-1">
          <div className="font-semibold flex items-center gap-1.5 text-xs">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            Backup Health Warnings
          </div>
          <ul className="list-disc pl-5 space-y-0.5 text-[11px]">
            {dashboard.warnings.map((w, idx) => (
              <li key={idx}>{w}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Metric Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Last Backup Snapshot
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>
                {dashboard?.last_backup?.completed_at
                  ? formatBusinessDate(dashboard.last_backup.completed_at.slice(0, 10))
                  : "Never"}
              </span>
              {dashboard?.last_backup && (
                <Badge
                  variant={
                    dashboard.last_backup.status === "verified"
                      ? "paid"
                      : dashboard.last_backup.status === "completed"
                        ? "active"
                        : "overdue"
                  }
                  size="sm"
                >
                  {dashboard.last_backup.status}
                </Badge>
              )}
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            {dashboard?.last_backup ? (
              <span>
                {formatBytes(dashboard.last_backup.file_size_bytes)} •{" "}
                {dashboard.last_backup.record_counts.students} students
              </span>
            ) : (
              <span>No snapshots recorded yet</span>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Last Verified Snapshot
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>
                {dashboard?.last_verified?.verified_at
                  ? formatBusinessDate(dashboard.last_verified.verified_at.slice(0, 10))
                  : "None"}
              </span>
              <ShieldCheck className="h-4 w-4 text-emerald-500" />
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            {dashboard?.last_verified ? (
              <span className="text-emerald-600 dark:text-emerald-400 font-medium">
                PRAGMA integrity check passed
              </span>
            ) : (
              <span className="text-amber-600 dark:text-amber-400">
                Needs verification
              </span>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Live Database File Size
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>{dashboard ? formatBytes(dashboard.data_size_bytes) : "0 B"}</span>
              <HardDrive className="h-4 w-4 text-primary" />
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>Primary SQLite store (dev.db)</span>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Retained Backups
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>{dashboard?.backup_count ?? 0}</span>
              <Clock className="h-4 w-4 text-muted-foreground" />
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>Retention: 7 daily + 30-day tagged</span>
          </CardContent>
        </Card>
      </div>

      {/* Destinations and Schedules Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Destinations */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <div>
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <HardDrive className="h-4 w-4 text-primary" />
                Backup Destinations
              </CardTitle>
              <CardDescription className="text-xs">
                Local folders, external USB drives, or LAN network shares.
              </CardDescription>
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setDestModalOpen(true)}
              className="gap-1 h-7 text-xs font-semibold"
            >
              <Plus className="h-3 w-3" />
              Add Target
            </Button>
          </CardHeader>
          <CardContent className="space-y-2">
            {dashboard?.destinations.length === 0 ? (
              <div className="p-4 text-center text-xs text-muted-foreground border border-dashed rounded-lg">
                No backup destination configured. Add a destination to enable automated
                snapshots.
              </div>
            ) : (
              dashboard?.destinations.map((dest) => (
                <div
                  key={dest.id}
                  className="p-3 rounded-lg border border-border bg-card/60 flex items-center justify-between text-xs"
                >
                  <div className="space-y-1 min-w-0 pr-3">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-foreground">{dest.label}</span>
                      <Badge variant="secondary" size="sm" className="text-[10px]">
                        {dest.kind}
                      </Badge>
                      {dest.is_default && (
                        <Badge variant="active" size="sm" className="text-[10px]">
                          Default
                        </Badge>
                      )}
                    </div>
                    <div className="font-mono text-[11px] text-muted-foreground truncate">
                      {dest.path}
                    </div>
                    {dest.last_error && (
                      <div className="text-[10px] text-destructive">
                        {dest.last_error}
                      </div>
                    )}
                  </div>

                  <Button
                    size="sm"
                    variant="ghost"
                    className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive shrink-0"
                    onClick={() => void handleRemoveDestination(dest.id)}
                    title="Remove destination"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        {/* Schedules */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-semibold flex items-center gap-2">
              <Clock className="h-4 w-4 text-primary" />
              Automated Snapshot Schedules
            </CardTitle>
            <CardDescription className="text-xs">
              Background job runner creates verified snapshots without stopping counter
              operations.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Daily Schedule */}
            <div className="p-3 rounded-lg border border-border bg-card/60 flex items-center justify-between">
              <div className="space-y-0.5">
                <div className="font-semibold text-xs text-foreground">
                  Daily Full Snapshot (Nightly)
                </div>
                <div className="text-[11px] text-muted-foreground">
                  Runs daily at 23:00 UTC with full verification and manifest signing.
                </div>
              </div>
              <Switch
                checked={
                  dashboard?.schedules.some((s) => s.kind === "daily" && s.enabled) ??
                  false
                }
                onCheckedChange={(checked) =>
                  void handleToggleSchedule("daily", !checked)
                }
              />
            </div>

            {/* Hourly Schedule */}
            <div className="p-3 rounded-lg border border-border bg-card/60 flex items-center justify-between">
              <div className="space-y-0.5">
                <div className="font-semibold text-xs text-foreground">
                  Hourly Snapshot (Counter Hours)
                </div>
                <div className="text-[11px] text-muted-foreground">
                  Captures hot transactional checkpoints during active cashier desk hours.
                </div>
              </div>
              <Switch
                checked={
                  dashboard?.schedules.some((s) => s.kind === "hourly" && s.enabled) ??
                  false
                }
                onCheckedChange={(checked) =>
                  void handleToggleSchedule("hourly", !checked)
                }
              />
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Backup History Table */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-semibold flex items-center gap-2">
            <FileCheck className="h-4 w-4 text-primary" />
            Snapshot History & Verification Log
          </CardTitle>
          <CardDescription className="text-xs">
            Review snapshots, verify HMAC checksums, and trigger restorations.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {history.length === 0 ? (
            <div className="py-8 text-center text-xs text-muted-foreground">
              No backup snapshots recorded in database history.
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-xs">Timestamp</TableHead>
                  <TableHead className="text-xs">Kind</TableHead>
                  <TableHead className="text-xs">Size</TableHead>
                  <TableHead className="text-xs">Students</TableHead>
                  <TableHead className="text-xs">Payments</TableHead>
                  <TableHead className="text-xs">Status</TableHead>
                  <TableHead className="text-xs text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {history.map((item) => (
                  <TableRow key={item.id}>
                    <TableCell className="text-xs font-medium">
                      {item.completed_at || item.started_at}
                    </TableCell>
                    <TableCell className="text-xs">
                      <Badge
                        variant="secondary"
                        size="sm"
                        className="font-mono text-[10px]"
                      >
                        {item.kind}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {formatBytes(item.file_size_bytes)}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {item.record_counts.students}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {item.record_counts.payments}
                    </TableCell>
                    <TableCell className="text-xs">
                      <Badge
                        variant={
                          item.status === "verified"
                            ? "paid"
                            : item.status === "completed"
                              ? "active"
                              : "overdue"
                        }
                        size="sm"
                      >
                        {item.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs text-right space-x-1.5">
                      <Button
                        size="sm"
                        variant="outline"
                        className="h-7 text-[11px] px-2 font-semibold"
                        disabled={verifyingId === item.id}
                        onClick={() => void handleVerify(item.id)}
                      >
                        {verifyingId === item.id ? (
                          <RefreshCw className="h-3 w-3 animate-spin" />
                        ) : (
                          <ShieldCheck className="h-3 w-3 text-emerald-500 mr-1" />
                        )}
                        Verify
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 text-[11px] px-2 font-semibold text-primary"
                        onClick={() => {
                          setRestoreFilePath(item.file_path);
                          setRestoreOpen(true);
                        }}
                      >
                        Restore
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Modal: Create Snapshot Now */}
      <Dialog open={snapshotOpen} onOpenChange={setSnapshotOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Create Backup Snapshot</DialogTitle>
            <DialogDescription className="text-xs">
              Captures a consistent hot snapshot of the database using SQLite online
              backup.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-2 text-xs">
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Target Destination</label>
              <Select value={selectedDestId} onValueChange={setSelectedDestId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select destination" />
                </SelectTrigger>
                <SelectContent>
                  {dashboard?.destinations.map((d) => (
                    <SelectItem key={d.id} value={d.id}>
                      {d.label} ({d.path})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Snapshot Kind</label>
              <Select value={snapshotKind} onValueChange={setSnapshotKind}>
                <SelectTrigger>
                  <SelectValue placeholder="Select kind" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="manual">Manual Snapshot</SelectItem>
                  <SelectItem value="daily">Daily Snapshot</SelectItem>
                  <SelectItem value="pre_update">Pre-Update Checkpoint</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {snapshotError && (
              <p className="text-destructive text-xs font-medium">{snapshotError}</p>
            )}
          </div>

          <DialogFooter>
            <Button variant="ghost" size="sm" onClick={() => setSnapshotOpen(false)}>
              Cancel
            </Button>
            <Button
              size="sm"
              disabled={!selectedDestId || snapshotLoading}
              onClick={() => void handleCreateSnapshot()}
              className="gap-1.5"
            >
              {snapshotLoading ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Camera className="h-3.5 w-3.5" />
              )}
              {snapshotLoading ? "Capturing..." : "Create Snapshot"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: Add Destination */}
      <Dialog open={destModalOpen} onOpenChange={setDestModalOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add Backup Destination</DialogTitle>
            <DialogDescription className="text-xs">
              Configure a local directory or removable storage drive for backup storage.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-2 text-xs">
            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Label</label>
              <Input
                value={destLabel}
                onChange={(e) => setDestLabel(e.target.value)}
                placeholder="e.g. External USB Drive (D:)"
              />
            </div>

            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Storage Path</label>
              <Input
                value={destPath}
                onChange={(e) => setDestPath(e.target.value)}
                placeholder="e.g. D:\CoachifyBackups"
                className="font-mono"
              />
            </div>

            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">Storage Kind</label>
              <Select value={destKind} onValueChange={setDestKind}>
                <SelectTrigger>
                  <SelectValue placeholder="Kind" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="local_folder">Local Directory</SelectItem>
                  <SelectItem value="usb_drive">Removable USB Drive</SelectItem>
                  <SelectItem value="network_share">Network Share</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {destError && (
              <p className="text-destructive text-xs font-medium">{destError}</p>
            )}
          </div>

          <DialogFooter>
            <Button variant="ghost" size="sm" onClick={() => setDestModalOpen(false)}>
              Cancel
            </Button>
            <Button
              size="sm"
              disabled={!destPath.trim() || !destLabel.trim() || destSaving}
              onClick={() => void handleSaveDestination()}
              className="gap-1.5"
            >
              {destSaving ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <FolderPlus className="h-3.5 w-3.5" />
              )}
              Save Destination
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: Verification Result */}
      <Dialog open={verifyModalOpen} onOpenChange={setVerifyModalOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <div className="flex items-center gap-2">
              <div
                className={`p-2 rounded-lg ${
                  verifyResult?.passed
                    ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                    : "bg-destructive/10 text-destructive"
                }`}
              >
                {verifyResult?.passed ? (
                  <CheckCircle2 className="h-5 w-5" />
                ) : (
                  <ShieldAlert className="h-5 w-5" />
                )}
              </div>
              <div>
                <DialogTitle>
                  {verifyResult?.passed ? "Verification Passed" : "Verification Failed"}
                </DialogTitle>
                <DialogDescription className="text-xs">
                  Cryptographic HMAC and SQLite PRAGMA validation result.
                </DialogDescription>
              </div>
            </div>
          </DialogHeader>

          {verifyResult && (
            <div className="space-y-3 py-2 text-xs">
              <div className="grid grid-cols-2 gap-2">
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <span className="text-[10px] text-muted-foreground block">
                    Integrity Check
                  </span>
                  <span
                    className={`font-semibold ${
                      verifyResult.integrity_ok
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {verifyResult.integrity_ok ? "OK" : "FAILED"}
                  </span>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <span className="text-[10px] text-muted-foreground block">
                    Foreign Keys
                  </span>
                  <span
                    className={`font-semibold ${
                      verifyResult.foreign_keys_ok
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {verifyResult.foreign_keys_ok ? "OK" : "FAILED"}
                  </span>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <span className="text-[10px] text-muted-foreground block">
                    HMAC Signature
                  </span>
                  <span
                    className={`font-semibold ${
                      verifyResult.manifest_hmac_ok
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {verifyResult.manifest_hmac_ok ? "VERIFIED" : "MISMATCH"}
                  </span>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <span className="text-[10px] text-muted-foreground block">
                    Record Counts
                  </span>
                  <span
                    className={`font-semibold ${
                      verifyResult.record_counts_match
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {verifyResult.record_counts_match ? "MATCHED" : "MISMATCH"}
                  </span>
                </div>
              </div>

              {verifyResult.errors.length > 0 && (
                <div className="p-2.5 rounded bg-destructive/10 border border-destructive/20 text-destructive text-[11px] space-y-1">
                  <span className="font-semibold">Errors detected:</span>
                  <ul className="list-disc pl-4 space-y-0.5">
                    {verifyResult.errors.map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          <DialogFooter>
            <Button size="sm" onClick={() => setVerifyModalOpen(false)}>
              Dismiss
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Restore Wizard */}
      <RestoreWizard
        open={restoreOpen}
        onOpenChange={setRestoreOpen}
        initialFilePath={restoreFilePath}
        onRestoreSuccess={() => void loadData()}
      />
    </div>
  );
}
