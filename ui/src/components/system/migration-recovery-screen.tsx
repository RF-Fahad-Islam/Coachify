import { useState } from "react";
import { type RecoveryInfoResponse, rpc } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  AlertTriangle,
  FolderOpen,
  Download,
  RotateCw,
  ShieldCheck,
  FileCode,
  HardDrive,
  Database,
  CheckCircle2,
} from "lucide-react";

interface MigrationRecoveryScreenProps {
  recoveryInfo: RecoveryInfoResponse;
  onRetry?: () => void;
}

export function MigrationRecoveryScreen({
  recoveryInfo,
  onRetry,
}: MigrationRecoveryScreenProps) {
  const [openingFolder, setOpeningFolder] = useState(false);
  const [exported, setExported] = useState(false);

  const handleOpenFolder = async () => {
    setOpeningFolder(true);
    try {
      await rpc("system.open_db_directory", {});
    } catch (e: unknown) {
      console.error("Failed to open database directory:", e);
    } finally {
      setOpeningFolder(false);
    }
  };

  const handleExportDiagnostics = () => {
    const rawError = recoveryInfo.error_message ?? "Unknown migration failure";
    // Sanitize any credentials or sensitive strings
    const sanitizedError = rawError.replace(
      /(password|token|secret|key|hash)=[^\s&,;]+/gi,
      "$1=[REDACTED]"
    );

    const diagnosticsBundle = {
      app_name: "Coachify",
      diagnostic_type: "database_migration_recovery",
      timestamp: new Date().toISOString(),
      is_in_recovery: recoveryInfo.is_in_recovery,
      failed_version: recoveryInfo.failed_version,
      error_message: sanitizedError,
      active_db_path: recoveryInfo.active_db_path,
      archive_count: recoveryInfo.archive_paths.length,
      archive_paths: recoveryInfo.archive_paths,
      system_note:
        "Coachify isolated the migration on staging.db; the production database file was preserved intact.",
    };

    const dataStr =
      "data:text/json;charset=utf-8," +
      encodeURIComponent(JSON.stringify(diagnosticsBundle, null, 2));
    const downloadAnchor = document.createElement("a");
    downloadAnchor.setAttribute("href", dataStr);
    downloadAnchor.setAttribute(
      "download",
      `coachify-migration-diagnostics-${Date.now()}.json`
    );
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();

    setExported(true);
    setTimeout(() => setExported(false), 4000);
  };

  const handleRetry = () => {
    if (onRetry) {
      onRetry();
    } else if (typeof window !== "undefined") {
      window.location.reload();
    }
  };

  return (
    <div className="min-h-screen w-full flex items-center justify-center bg-background p-4 sm:p-6 md:p-8">
      <Card className="w-full max-w-2xl border-destructive/30 shadow-2xl bg-card">
        <CardHeader className="space-y-3 pb-4 border-b border-border">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-destructive/15 text-destructive">
                <AlertTriangle className="h-5 w-5" />
              </div>
              <div>
                <CardTitle className="text-xl font-bold tracking-tight text-foreground">
                  Database Migration Recovery
                </CardTitle>
                <CardDescription className="text-xs text-muted-foreground mt-0.5">
                  An automatic safety guard protected your institution records.
                </CardDescription>
              </div>
            </div>
            <Badge variant="overdue" className="font-mono text-xs uppercase">
              Recovery Mode
            </Badge>
          </div>
        </CardHeader>

        <CardContent className="space-y-6 pt-6 text-foreground">
          {/* Safeguard Assurance Notice */}
          <div className="flex items-start gap-3 rounded-lg border border-emerald-500/20 bg-emerald-500/10 p-3.5 text-xs text-emerald-600 dark:text-emerald-400">
            <ShieldCheck className="h-4 w-4 shrink-0 mt-0.5" />
            <div className="space-y-1">
              <p className="font-semibold">Your live database remains completely safe</p>
              <p className="text-emerald-600/90 dark:text-emerald-400/90 leading-relaxed">
                Coachify executes all schema migrations on an isolated temporary staging
                copy. Because the upgrade did not pass validation, the active database was
                never modified or replaced.
              </p>
            </div>
          </div>

          {/* Error Diagnostics Box */}
          <div className="space-y-2">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="font-medium flex items-center gap-1.5">
                <FileCode className="h-3.5 w-3.5" />
                Technical Error Diagnostics
              </span>
              {recoveryInfo.failed_version != null && (
                <span className="font-mono text-[11px] bg-muted px-2 py-0.5 rounded text-foreground">
                  {`Failed at Migration v${recoveryInfo.failed_version}`}
                </span>
              )}
            </div>

            <div className="rounded-md bg-muted/60 p-3 border border-border font-mono text-xs text-destructive break-all whitespace-pre-wrap max-h-40 overflow-y-auto">
              {recoveryInfo.error_message ||
                "A validation gate or migration script rejected the staging database."}
            </div>
          </div>

          {/* Database & Archive Metadata */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <div className="rounded-lg border border-border p-3 space-y-1 bg-card/60">
              <span className="font-medium text-muted-foreground flex items-center gap-1.5">
                <Database className="h-3.5 w-3.5 text-primary" />
                Active Database Location
              </span>
              <p
                className="font-mono text-[11px] text-foreground truncate"
                title={recoveryInfo.active_db_path}
              >
                {recoveryInfo.active_db_path}
              </p>
            </div>

            <div className="rounded-lg border border-border p-3 space-y-1 bg-card/60">
              <span className="font-medium text-muted-foreground flex items-center gap-1.5">
                <HardDrive className="h-3.5 w-3.5 text-primary" />
                Pre-migration Safety Archives
              </span>
              <p className="text-[11px] text-foreground font-medium">
                {recoveryInfo.archive_paths.length > 0
                  ? `${recoveryInfo.archive_paths.length} historical backup(s) preserved`
                  : "No prior archives found in folder"}
              </p>
            </div>
          </div>

          {/* Historical Archives List if available */}
          {recoveryInfo.archive_paths.length > 0 && (
            <div className="space-y-1.5">
              <span className="text-xs text-muted-foreground font-medium">
                Available Safety Snapshots (retained for 30 days)
              </span>
              <div className="max-h-24 overflow-y-auto space-y-1 rounded border border-border p-2 bg-muted/30">
                {recoveryInfo.archive_paths.map((path, idx) => (
                  <div
                    key={idx}
                    className="font-mono text-[11px] text-muted-foreground flex items-center justify-between"
                  >
                    <span className="truncate" title={path}>
                      {path.split(/[/\\]/).pop()}
                    </span>
                    <span className="text-[10px] bg-background border border-border rounded px-1.5 py-0.2">
                      Backup
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>

        <CardFooter className="flex flex-wrap items-center justify-between gap-2.5 pt-4 pb-6 border-t border-border bg-muted/20">
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleExportDiagnostics}
              className="gap-1.5 text-xs"
              data-testid="export-diagnostics-button"
            >
              {exported ? (
                <>
                  <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />
                  Downloaded
                </>
              ) : (
                <>
                  <Download className="h-3.5 w-3.5" />
                  Export Diagnostics
                </>
              )}
            </Button>

            <Button
              variant="outline"
              size="sm"
              onClick={handleOpenFolder}
              disabled={openingFolder}
              className="gap-1.5 text-xs"
              data-testid="open-db-folder-button"
            >
              <FolderOpen className="h-3.5 w-3.5" />
              Open Database Folder
            </Button>
          </div>

          <Button
            variant="primary"
            size="sm"
            onClick={handleRetry}
            className="gap-1.5 text-xs"
            data-testid="retry-upgrade-button"
          >
            <RotateCw className="h-3.5 w-3.5" />
            Retry Upgrade
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
