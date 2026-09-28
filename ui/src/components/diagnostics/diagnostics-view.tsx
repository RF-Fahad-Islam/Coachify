import { useCallback, useEffect, useState, useMemo } from "react";
import { rpc, type DiagnosticsInfo, type IntegrityCheckResult } from "@/gen/rpc";
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
import { formatBytes } from "@/lib/utils";
import {
  ShieldCheck,
  ShieldAlert,
  HardDrive,
  Download,
  Search,
  RefreshCw,
  Copy,
  Check,
  Database,
  Layers,
  CheckCircle2,
} from "lucide-react";

export function DiagnosticsView() {
  const [info, setInfo] = useState<DiagnosticsInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copiedDbPath, setCopiedDbPath] = useState(false);
  const [tableSearch, setTableSearch] = useState("");

  // Integrity Check Dialog
  const [checkingIntegrity, setCheckingIntegrity] = useState(false);
  const [integrityResult, setIntegrityResult] = useState<IntegrityCheckResult | null>(
    null
  );
  const [integrityDialogOpen, setIntegrityDialogOpen] = useState(false);

  // Bundle Export Dialog
  const [bundleDialogOpen, setBundleDialogOpen] = useState(false);
  const [bundleDestPath, setBundleDestPath] = useState("coachify-diagnostics-bundle.zip");
  const [exportingBundle, setExportingBundle] = useState(false);
  const [bundleResultPath, setBundleResultPath] = useState<string | null>(null);
  const [bundleError, setBundleError] = useState<string | null>(null);

  const loadInfo = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("diagnostics.get_info", {});
      if (res.ok && res.data) {
        setInfo(res.data);
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadInfo();
  }, [loadInfo]);

  const handleCopyDbPath = () => {
    if (!info?.database_path) return;
    void navigator.clipboard.writeText(info.database_path);
    setCopiedDbPath(true);
    setTimeout(() => setCopiedDbPath(false), 2000);
  };

  const handleRunIntegrityCheck = async () => {
    setCheckingIntegrity(true);
    try {
      const res = await rpc("diagnostics.run_integrity_check", {});
      if (res.ok && res.data) {
        setIntegrityResult(res.data);
        setIntegrityDialogOpen(true);
      }
    } catch (e: unknown) {
      console.error("Integrity check failed:", e);
    } finally {
      setCheckingIntegrity(false);
    }
  };

  const handleExportBundle = async () => {
    if (!bundleDestPath.trim()) return;
    setExportingBundle(true);
    setBundleError(null);
    setBundleResultPath(null);
    try {
      const res = await rpc("diagnostics.export_bundle", {
        destination_path: bundleDestPath.trim(),
      });
      if (res.ok && res.data) {
        setBundleResultPath(res.data.bundle_path);
      } else {
        setBundleError(res.error?.message || "Export failed.");
      }
    } catch (e: unknown) {
      setBundleError(e instanceof Error ? e.message : String(e));
    } finally {
      setExportingBundle(false);
    }
  };

  const filteredTables = useMemo(() => {
    if (!info?.table_stats) return [];
    if (!tableSearch.trim()) return info.table_stats;
    const term = tableSearch.toLowerCase();
    return info.table_stats.filter((t) => t.name.toLowerCase().includes(term));
  }, [info?.table_stats, tableSearch]);

  if (loading && !info) {
    return (
      <div className="p-8 text-center text-xs text-muted-foreground">
        <RefreshCw className="h-6 w-6 animate-spin mx-auto mb-2 text-primary" />
        Gathering system metrics and SQLite engine diagnostics...
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            System Diagnostics & Engine Health
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Real-time storage telemetry, SQLite low-level metrics, and sanitized support
            bundles.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => void loadInfo()}
            className="gap-1.5 h-8 text-xs font-semibold"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => void handleRunIntegrityCheck()}
            disabled={checkingIntegrity}
            className="gap-1.5 h-8 text-xs font-semibold text-foreground"
          >
            {checkingIntegrity ? (
              <RefreshCw className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />
            )}
            Run Integrity Check
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setBundleResultPath(null);
              setBundleError(null);
              setBundleDialogOpen(true);
            }}
            className="gap-1.5 h-8 text-xs font-semibold"
          >
            <Download className="h-3.5 w-3.5" />
            Export Diagnostics Bundle
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-3.5 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-xs">
          {error}
        </div>
      )}

      {/* Overview Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Application & Engine
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>v{info?.app_version}</span>
              <Badge variant="secondary" size="sm" className="font-mono text-[10px]">
                Schema v{info?.schema_version}
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>Operating System: {info?.os_version}</span>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Database Storage (WAL)
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>{info ? formatBytes(info.database_size_bytes) : "0 B"}</span>
              <HardDrive className="h-4 w-4 text-primary" />
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>
              WAL: {info ? formatBytes(info.wal_size_bytes) : "0 B"} • Freelist:{" "}
              {info?.freelist_pages ?? 0}
            </span>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Storage Free Space
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>{info ? formatBytes(info.data_drive_free_bytes) : "0 B"}</span>
              <Database className="h-4 w-4 text-muted-foreground" />
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>
              Backup Drive:{" "}
              {info?.backup_drive_free_bytes
                ? formatBytes(info.backup_drive_free_bytes)
                : "N/A"}
            </span>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardDescription className="text-[11px] font-medium uppercase tracking-wider">
              Engine Integrity Status
            </CardDescription>
            <CardTitle className="text-base font-bold flex items-center justify-between">
              <span>{info?.integrity_status}</span>
              <Badge
                variant={info?.integrity_status === "ok" ? "paid" : "overdue"}
                size="sm"
              >
                {info?.integrity_status === "ok" ? "Healthy" : "Attention"}
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent className="text-[11px] text-muted-foreground">
            <span>
              Destination:{" "}
              {info?.backup_status.destination_healthy ? "Accessible" : "Inaccessible"}
            </span>
          </CardContent>
        </Card>
      </div>

      {/* Database File Path Bar */}
      {info?.database_path && (
        <div className="p-3 rounded-lg border border-border bg-card/60 flex items-center justify-between gap-3 text-xs">
          <div className="flex items-center gap-2 min-w-0">
            <Database className="h-4 w-4 text-muted-foreground shrink-0" />
            <span className="text-muted-foreground shrink-0 font-medium">
              Database Path:
            </span>
            <code className="font-mono text-[11px] text-foreground truncate select-all">
              {info.database_path}
            </code>
          </div>
          <Button
            size="sm"
            variant="ghost"
            onClick={handleCopyDbPath}
            className="gap-1.5 h-7 text-xs font-semibold shrink-0"
          >
            {copiedDbPath ? (
              <Check className="h-3.5 w-3.5 text-emerald-500" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
            {copiedDbPath ? "Copied" : "Copy"}
          </Button>
        </div>
      )}

      {/* Table Row Statistics */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3">
          <div>
            <CardTitle className="text-sm font-semibold flex items-center gap-2">
              <Layers className="h-4 w-4 text-primary" />
              Database Tables & Row Counts
            </CardTitle>
            <CardDescription className="text-xs">
              Live row counts across all domain entity stores in SQLite.
            </CardDescription>
          </div>

          <div className="relative w-full sm:w-64">
            <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
            <Input
              value={tableSearch}
              onChange={(e) => setTableSearch(e.target.value)}
              placeholder="Search table name..."
              className="pl-8 h-8 text-xs"
            />
          </div>
        </CardHeader>
        <CardContent>
          {filteredTables.length === 0 ? (
            <div className="py-6 text-center text-xs text-muted-foreground">
              No matching database tables found.
            </div>
          ) : (
            <div className="max-h-96 overflow-y-auto border rounded-lg">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="text-xs">Table Name</TableHead>
                    <TableHead className="text-xs text-right">Row Count</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredTables.map((t) => (
                    <TableRow key={t.name}>
                      <TableCell className="font-mono text-xs text-foreground font-medium">
                        {t.name}
                      </TableCell>
                      <TableCell className="font-mono text-xs text-right text-muted-foreground">
                        {t.row_count.toLocaleString()}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Modal: Integrity Check Result */}
      <Dialog open={integrityDialogOpen} onOpenChange={setIntegrityDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <div className="flex items-center gap-2">
              <div
                className={`p-2 rounded-lg ${
                  integrityResult?.passed
                    ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                    : "bg-destructive/10 text-destructive"
                }`}
              >
                {integrityResult?.passed ? (
                  <CheckCircle2 className="h-5 w-5" />
                ) : (
                  <ShieldAlert className="h-5 w-5" />
                )}
              </div>
              <div>
                <DialogTitle>
                  {integrityResult?.passed
                    ? "Integrity Check Passed"
                    : "Integrity Errors Detected"}
                </DialogTitle>
                <DialogDescription className="text-xs">
                  Low-level B-tree and page allocation check via SQLite PRAGMA
                  integrity_check.
                </DialogDescription>
              </div>
            </div>
          </DialogHeader>

          <div className="space-y-3 py-2 text-xs">
            <div className="p-3 rounded-lg bg-muted/60 border border-border font-mono text-[11px] space-y-1 max-h-48 overflow-y-auto">
              {integrityResult?.details.map((line: string, idx: number) => (
                <div
                  key={idx}
                  className={
                    line === "ok"
                      ? "text-emerald-600 dark:text-emerald-400"
                      : "text-destructive"
                  }
                >
                  {line}
                </div>
              ))}
            </div>
          </div>

          <DialogFooter>
            <Button size="sm" onClick={() => setIntegrityDialogOpen(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: Export Diagnostics Bundle */}
      <Dialog open={bundleDialogOpen} onOpenChange={setBundleDialogOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Export Diagnostics Bundle</DialogTitle>
            <DialogDescription className="text-xs">
              Exports a redacted ZIP archive with system info, logs, and database metrics.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-2 text-xs">
            <div className="p-3 rounded-lg bg-primary/10 border border-primary/20 text-xs space-y-1">
              <span className="font-semibold block text-primary">
                Privacy & PII Guarantee
              </span>
              <p className="text-[11px] text-muted-foreground leading-relaxed">
                All Bangladeshi phone numbers (e.g. 017XXXXXXXX), student names, guardian
                details, fee ledger amounts, and credentials are automatically stripped or
                redacted from the bundle.
              </p>
            </div>

            <div className="space-y-1.5">
              <label className="font-semibold text-foreground">
                Target Archive Path (.zip)
              </label>
              <Input
                value={bundleDestPath}
                onChange={(e) => setBundleDestPath(e.target.value)}
                placeholder="coachify-diagnostics-bundle.zip"
                className="font-mono"
              />
            </div>

            {bundleResultPath && (
              <div className="p-2.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-900 dark:text-emerald-200 text-xs">
                <span className="font-semibold block">Bundle exported successfully:</span>
                <span className="font-mono text-[11px] truncate block">
                  {bundleResultPath}
                </span>
              </div>
            )}

            {bundleError && (
              <p className="text-destructive text-xs font-medium">{bundleError}</p>
            )}
          </div>

          <DialogFooter>
            <Button variant="ghost" size="sm" onClick={() => setBundleDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              size="sm"
              disabled={!bundleDestPath.trim() || exportingBundle}
              onClick={() => void handleExportBundle()}
              className="gap-1.5"
            >
              {exportingBundle ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Download className="h-3.5 w-3.5" />
              )}
              {exportingBundle ? "Exporting..." : "Generate ZIP"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
