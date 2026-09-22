import { useCallback, useEffect, useState } from "react";
import {
  rpc,
  type BackupHistoryItem,
  type BackupPreview,
  type RestoreResult,
} from "@/gen/rpc";
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
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { formatTaka, formatBusinessDate } from "@/lib/utils";
import {
  ArchiveRestore,
  FileCheck,
  AlertTriangle,
  CheckCircle2,
  Lock,
  ArrowRight,
  RefreshCw,
  Database,
  CreditCard,
  ShieldCheck,
} from "lucide-react";

interface RestoreWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  initialFilePath?: string;
  onRestoreSuccess?: () => void;
}

type WizardStep = "select" | "preview" | "confirm" | "result";

export function RestoreWizard({
  open,
  onOpenChange,
  initialFilePath,
  onRestoreSuccess,
}: RestoreWizardProps) {
  const [step, setStep] = useState<WizardStep>("select");
  const [filePath, setFilePath] = useState(initialFilePath || "");
  const [recentBackups, setRecentBackups] = useState<BackupHistoryItem[]>([]);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [preview, setPreview] = useState<BackupPreview | null>(null);
  const [recoveryKey, setRecoveryKey] = useState("");
  const [confirmText, setConfirmText] = useState("");
  const [executing, setExecuting] = useState(false);
  const [executeError, setExecuteError] = useState<string | null>(null);
  const [restoreResult, setRestoreResult] = useState<RestoreResult | null>(null);

  // Sync initialFilePath when changed
  useEffect(() => {
    if (initialFilePath) {
      setFilePath(initialFilePath);
    }
  }, [initialFilePath]);

  const handleInspect = useCallback(
    async (pathOverride?: string) => {
      const pathToInspect = pathOverride || filePath;
      if (!pathToInspect.trim()) return;

      setPreviewLoading(true);
      setPreviewError(null);
      try {
        const res = await rpc("restore.preview_backup", {
          file_path: pathToInspect.trim(),
        });
        if (res.ok && res.data) {
          setPreview(res.data);
          setFilePath(pathToInspect.trim());
          setStep("preview");
        } else {
          setPreviewError(
            res.error?.message || "Failed to inspect candidate backup file."
          );
        }
      } catch (e: unknown) {
        setPreviewError(e instanceof Error ? e.message : String(e));
      } finally {
        setPreviewLoading(false);
      }
    },
    [filePath]
  );

  // Load recent backups on open to allow quick selection
  const loadRecentBackups = useCallback(async () => {
    try {
      const res = await rpc("backup.list_history", { limit: 5 });
      if (res.ok && res.data) {
        setRecentBackups(
          res.data.filter((b) => b.status === "completed" || b.status === "verified")
        );
      }
    } catch {
      // Non-critical if list history fails
    }
  }, []);

  useEffect(() => {
    if (open) {
      void loadRecentBackups();
      if (initialFilePath) {
        void handleInspect(initialFilePath);
      }
    } else {
      // Reset state when closed
      setStep("select");
      setPreview(null);
      setPreviewError(null);
      setExecuteError(null);
      setRestoreResult(null);
      setConfirmText("");
      setRecoveryKey("");
    }
  }, [open, initialFilePath, loadRecentBackups, handleInspect]);

  const handleExecuteRestore = async () => {
    if (confirmText !== "RESTORE") return;

    setExecuting(true);
    setExecuteError(null);
    try {
      const res = await rpc("restore.execute", {
        file_path: filePath.trim(),
        recovery_key: recoveryKey.trim() || null,
      });

      if (res.ok && res.data) {
        setRestoreResult(res.data);
        setStep("result");
        onRestoreSuccess?.();
      } else {
        setExecuteError(res.error?.message || "Restoration failed.");
      }
    } catch (e: unknown) {
      setExecuteError(e instanceof Error ? e.message : String(e));
    } finally {
      setExecuting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-primary/10 text-primary">
              <ArchiveRestore className="h-5 w-5" />
            </div>
            <div>
              <DialogTitle>Database Restoration Wizard</DialogTitle>
              <DialogDescription className="text-xs">
                Safely inspect, verify, and restore institution records from a snapshot.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        {/* Step 1: Select File */}
        {step === "select" && (
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <label className="text-xs font-semibold text-foreground">
                Backup File Path (.db)
              </label>
              <div className="flex gap-2">
                <Input
                  value={filePath}
                  onChange={(e) => setFilePath(e.target.value)}
                  placeholder="C:\CoachifyBackups\coachify_snapshot_*.db"
                  className="font-mono text-xs"
                />
                <Button
                  disabled={!filePath.trim() || previewLoading}
                  onClick={() => void handleInspect()}
                  className="gap-1.5 shrink-0"
                >
                  {previewLoading ? (
                    <RefreshCw className="h-4 w-4 animate-spin" />
                  ) : (
                    <FileCheck className="h-4 w-4" />
                  )}
                  Inspect Candidate
                </Button>
              </div>
              {previewError && (
                <p className="text-xs text-destructive font-medium mt-1">
                  {previewError}
                </p>
              )}
            </div>

            {recentBackups.length > 0 && (
              <div className="space-y-2 pt-2 border-t border-border">
                <span className="text-xs font-semibold text-muted-foreground">
                  Or select from recent local snapshots:
                </span>
                <div className="space-y-1.5">
                  {recentBackups.map((item) => (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => {
                        setFilePath(item.file_path);
                        void handleInspect(item.file_path);
                      }}
                      className="w-full text-left p-2.5 rounded-lg border border-border hover:bg-muted/60 transition-colors flex items-center justify-between text-xs"
                    >
                      <div className="space-y-0.5 min-w-0 pr-2">
                        <div className="font-mono text-[11px] truncate text-foreground">
                          {item.file_path}
                        </div>
                        <div className="text-[10px] text-muted-foreground">
                          {item.completed_at
                            ? formatBusinessDate(item.completed_at.slice(0, 10))
                            : "—"}{" "}
                          • {item.kind} • {item.record_counts.students} students •{" "}
                          {item.record_counts.payments} payments
                        </div>
                      </div>
                      <Badge
                        variant={item.status === "verified" ? "paid" : "active"}
                        size="sm"
                      >
                        {item.status}
                      </Badge>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Step 2: Preview & Verification */}
        {step === "preview" && preview && (
          <div className="space-y-4 py-2">
            <div className="grid grid-cols-2 gap-3">
              <Card>
                <CardContent className="p-3 space-y-1 text-xs">
                  <span className="text-[10px] uppercase font-semibold text-muted-foreground">
                    Target Institution
                  </span>
                  <div className="font-bold text-foreground text-sm">
                    {preview.institution_name}
                  </div>
                  <div className="font-mono text-[10px] text-muted-foreground truncate">
                    ID: {preview.institution_id}
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-3 space-y-1 text-xs">
                  <span className="text-[10px] uppercase font-semibold text-muted-foreground">
                    Snapshot Schema
                  </span>
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-foreground text-sm">
                      v{preview.schema_version}
                    </span>
                    <span className="text-muted-foreground text-[11px]">
                      (Current: v{preview.current_schema_version})
                    </span>
                  </div>
                  <div className="flex items-center gap-1.5 pt-0.5">
                    <Badge variant={preview.integrity_ok ? "paid" : "overdue"} size="sm">
                      {preview.integrity_ok ? "Integrity Passed" : "Integrity Failed"}
                    </Badge>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Migration / Update notices */}
            {preview.needs_migration && (
              <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 text-xs flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                <div>
                  <span className="font-semibold">Forward Migration Required:</span> This
                  snapshot was created with schema v{preview.schema_version}. Coachify
                  will automatically and transactionally apply pending schema migrations
                  to bring it up to v{preview.current_schema_version}.
                </div>
              </div>
            )}

            {preview.needs_update && (
              <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-xs flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                <div>
                  <span className="font-semibold">Application Update Required:</span> This
                  snapshot is from a newer schema version (v{preview.schema_version}) than
                  your installed Coachify app (v{preview.current_schema_version}). You
                  must update Coachify before restoring this database.
                </div>
              </div>
            )}

            {/* Record Counts Grid */}
            <div className="space-y-1.5">
              <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <Database className="h-3.5 w-3.5 text-primary" />
                Record Summary in Candidate
              </span>
              <div className="grid grid-cols-3 gap-2 text-xs">
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">Students</div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.students}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">Payments</div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.payments}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">Fee Charges</div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.charges}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">Expenses</div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.expenses}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">Enrollments</div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.enrollments}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">
                    Audit Trail Rows
                  </div>
                  <div className="font-semibold text-foreground">
                    {preview.record_counts.audit_rows}
                  </div>
                </div>
              </div>
            </div>

            {/* Financial Ledger Totals */}
            <div className="space-y-1.5">
              <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <CreditCard className="h-3.5 w-3.5 text-primary" />
                Ledger Totals in Candidate
              </span>
              <div className="grid grid-cols-2 gap-2 text-xs">
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">
                    Total Collections (Paid)
                  </div>
                  <div className="font-semibold text-emerald-600 dark:text-emerald-400">
                    {formatTaka(preview.money_totals.total_paid_poisha)}
                  </div>
                </div>
                <div className="p-2 rounded bg-muted/60 border border-border">
                  <div className="text-[10px] text-muted-foreground">
                    Total Outstanding Balance
                  </div>
                  <div className="font-semibold text-foreground">
                    {formatTaka(preview.money_totals.total_balance_poisha)}
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Step 3: Confirmation & Recovery Key */}
        {step === "confirm" && preview && (
          <div className="space-y-4 py-2">
            <div className="p-3.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-900 dark:text-emerald-200 text-xs space-y-1.5">
              <div className="font-semibold flex items-center gap-1.5">
                <ShieldCheck className="h-4 w-4" />
                Automated Safety Pre-Backup Guarantee
              </div>
              <p className="text-[11px] leading-relaxed">
                Before swapping the database, Coachify will automatically create a full,
                verified safety snapshot of your current live database. If anything goes
                wrong, you can always revert to your safety snapshot.
              </p>
            </div>

            <div className="space-y-2">
              <label className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <Lock className="h-3.5 w-3.5 text-muted-foreground" />
                Emergency Recovery Key (Optional / New Machine)
              </label>
              <Input
                type="password"
                value={recoveryKey}
                onChange={(e) => setRecoveryKey(e.target.value)}
                placeholder="Enter 24-word or hex recovery key if restoring to new machine"
                className="font-mono text-xs"
              />
              <p className="text-[11px] text-muted-foreground">
                Leave blank if restoring on the same workstation where Windows DPAPI
                credentials remain intact.
              </p>
            </div>

            <div className="space-y-2 pt-2 border-t border-border">
              <label className="text-xs font-semibold text-destructive">
                Safety Confirmation: Type &quot;RESTORE&quot; to proceed
              </label>
              <Input
                value={confirmText}
                onChange={(e) => setConfirmText(e.target.value)}
                placeholder="RESTORE"
                className="font-mono text-xs uppercase"
              />
            </div>

            {executeError && (
              <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/30 text-destructive text-xs">
                <span className="font-semibold">Restoration Failed:</span> {executeError}
              </div>
            )}
          </div>
        )}

        {/* Step 4: Result */}
        {step === "result" && restoreResult && (
          <div className="space-y-4 py-4 text-center">
            <div className="mx-auto h-12 w-12 rounded-full bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 flex items-center justify-center">
              <CheckCircle2 className="h-6 w-6" />
            </div>

            <div className="space-y-1">
              <h3 className="text-base font-bold text-foreground">
                Database Restored Successfully
              </h3>
              <p className="text-xs text-muted-foreground max-w-md mx-auto">
                {restoreResult.message}
              </p>
            </div>

            {restoreResult.safety_backup_id && (
              <div className="p-2.5 rounded-lg bg-muted/60 border border-border text-left text-xs font-mono">
                <span className="text-muted-foreground text-[10px] block">
                  Safety Backup Created:
                </span>
                <span className="text-foreground text-[11px] truncate block">
                  ID: {restoreResult.safety_backup_id}
                </span>
              </div>
            )}

            <div className="p-3 rounded-lg bg-primary/10 border border-primary/20 text-xs text-primary-foreground">
              Please restart Coachify or refresh your active view to load the newly
              restored data.
            </div>
          </div>
        )}

        <DialogFooter className="flex items-center justify-between sm:justify-between pt-2 border-t border-border">
          {step === "select" && (
            <>
              <Button variant="ghost" size="sm" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button
                size="sm"
                disabled={!filePath.trim() || previewLoading}
                onClick={() => void handleInspect()}
                className="gap-1.5"
              >
                Inspect Backup
                <ArrowRight className="h-3.5 w-3.5" />
              </Button>
            </>
          )}

          {step === "preview" && (
            <>
              <Button variant="ghost" size="sm" onClick={() => setStep("select")}>
                Back
              </Button>
              <Button
                size="sm"
                disabled={preview?.needs_update || !preview?.integrity_ok}
                onClick={() => setStep("confirm")}
                className="gap-1.5"
              >
                Continue to Restore
                <ArrowRight className="h-3.5 w-3.5" />
              </Button>
            </>
          )}

          {step === "confirm" && (
            <>
              <Button
                variant="ghost"
                size="sm"
                disabled={executing}
                onClick={() => setStep("preview")}
              >
                Back
              </Button>
              <Button
                size="sm"
                variant="destructive"
                disabled={confirmText !== "RESTORE" || executing}
                onClick={() => void handleExecuteRestore()}
                className="gap-1.5"
              >
                {executing ? (
                  <RefreshCw className="h-4 w-4 animate-spin" />
                ) : (
                  <ArchiveRestore className="h-4 w-4" />
                )}
                {executing ? "Restoring Database..." : "Execute In-Place Restore"}
              </Button>
            </>
          )}

          {step === "result" && (
            <Button
              size="sm"
              className="w-full sm:w-auto ml-auto"
              onClick={() => {
                onOpenChange(false);
                window.location.reload();
              }}
            >
              Reload Application
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
