import { useState, type FormEvent } from "react";
import { rpc, type InspectDatabaseResponse } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  KeyRound,
  ShieldAlert,
  CheckCircle2,
  AlertTriangle,
  Lock,
  Search,
} from "lucide-react";

export interface EmergencyRecoveryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRecoverySuccess: (message: string) => void;
}

export function EmergencyRecoveryDialog({
  open,
  onOpenChange,
  onRecoverySuccess,
}: EmergencyRecoveryDialogProps) {
  const [dbDir, setDbDir] = useState("");
  const [recoveryKey, setRecoveryKey] = useState("");
  const [resetPassword, setResetPassword] = useState("");
  const [resetPasswordConfirm, setResetPasswordConfirm] = useState("");

  const [inspecting, setInspecting] = useState(false);
  const [inspectData, setInspectData] = useState<InspectDatabaseResponse | null>(null);

  const [submitting, setSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Inspect database at specified path
  const handleInspect = async () => {
    setErrorMsg(null);
    setInspecting(true);
    try {
      const res = await rpc("recovery.inspect_database", {
        db_dir: dbDir.trim() || null,
      });
      if (res.ok && res.data) {
        setInspectData(res.data);
      } else {
        setErrorMsg(res.error?.message ?? "Database inspection failed.");
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setInspecting(false);
    }
  };

  // Unlock database
  const handleUnlock = async (e: FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);

    const cleanKey = recoveryKey.trim().toUpperCase();
    if (!cleanKey) {
      setErrorMsg(
        "Please enter the 24-character Owner Recovery Key from your printed sheet."
      );
      return;
    }

    if (resetPassword && resetPassword.length < 8) {
      setErrorMsg("New password must be at least 8 characters long.");
      return;
    }

    if (resetPassword && resetPassword !== resetPasswordConfirm) {
      setErrorMsg("New passwords do not match.");
      return;
    }

    setSubmitting(true);
    try {
      const res = await rpc("recovery.unlock_with_key", {
        db_dir: dbDir.trim() || null,
        recovery_key: cleanKey,
        reset_owner_password: resetPassword.trim() || null,
      });

      if (res.ok && res.data) {
        onRecoverySuccess(
          `Database for "${res.data.institution_name}" has been successfully unlocked and bound to this computer. You can now log in.`
        );
        onOpenChange(false);
      } else {
        setErrorMsg(
          res.error?.message ??
            "Unlocking database failed. Please verify your recovery key."
        );
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center gap-2 text-primary font-semibold text-sm mb-1">
            <KeyRound className="h-4 w-4" />
            <span>Disaster Recovery &amp; Key Re-wrapping</span>
          </div>
          <DialogTitle className="text-xl font-bold">
            Unlock Database on Replacement Workstation
          </DialogTitle>
          <DialogDescription className="text-xs">
            If your previous computer suffered a hardware crash, was formatted, or you
            copied your database from another PC, use your printed Owner Recovery Sheet to
            unwrap the master key and seal it with this machine&apos;s Windows DPAPI.
          </DialogDescription>
        </DialogHeader>

        {errorMsg && (
          <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{errorMsg}</span>
          </div>
        )}

        <form onSubmit={handleUnlock} className="space-y-4 pt-2">
          {/* Database Path Field */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground flex items-center justify-between">
              <span>Database Directory (Leave empty for default app path)</span>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={handleInspect}
                disabled={inspecting}
                className="h-6 text-[11px] gap-1 px-2 text-primary"
              >
                <Search className="h-3 w-3" />
                {inspecting ? "Inspecting..." : "Inspect Folder"}
              </Button>
            </label>
            <Input
              placeholder="e.g. D:\Coachify Data\inst_xxxx\db"
              value={dbDir}
              onChange={(e) => setDbDir(e.target.value)}
              className="font-mono text-xs"
            />
          </div>

          {/* Database Inspection Status */}
          {inspectData && (
            <div className="p-3 rounded-lg border bg-muted/40 text-xs space-y-1.5">
              <div className="flex items-center justify-between font-semibold">
                <span>Database Status:</span>
                {inspectData.needs_recovery ? (
                  <span className="text-amber-600 dark:text-amber-400 flex items-center gap-1">
                    <ShieldAlert className="h-3.5 w-3.5" />
                    Foreign Machine Key (Recovery Required)
                  </span>
                ) : inspectData.can_unwrap_dpapi ? (
                  <span className="text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                    <CheckCircle2 className="h-3.5 w-3.5" />
                    Already Unlocked &amp; Bound
                  </span>
                ) : (
                  <span className="text-destructive">Database Missing</span>
                )}
              </div>
              {inspectData.institution_name && (
                <div className="text-muted-foreground">
                  Institution:{" "}
                  <strong className="text-foreground">
                    {inspectData.institution_name}
                  </strong>
                </div>
              )}
            </div>
          )}

          {/* Recovery Key Input */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Owner Recovery Key (Printed on Sheet) *
            </label>
            <Input
              placeholder="CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX"
              value={recoveryKey}
              onChange={(e) => setRecoveryKey(e.target.value.toUpperCase())}
              className="font-mono font-bold tracking-widest text-sm text-center uppercase"
              required
            />
            <p className="text-[11px] text-muted-foreground">
              Format: 24 characters in 6 blocks (e.g. CKREC-7M9K-W4BX-...).
            </p>
          </div>

          {/* Optional Owner Password Reset */}
          <div className="p-3 rounded-lg border border-border/60 bg-card/40 space-y-3">
            <div className="flex items-center gap-1.5 text-xs font-semibold text-foreground">
              <Lock className="h-3.5 w-3.5 text-primary" />
              <span>Reset Owner Password (Optional)</span>
            </div>
            <p className="text-[11px] text-muted-foreground">
              If you have also forgotten the administrator password, enter a new password
              below to reset it alongside the database unlock.
            </p>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <Input
                type="password"
                placeholder="New password (min 8 chars)"
                value={resetPassword}
                onChange={(e) => setResetPassword(e.target.value)}
                className="text-xs"
              />
              <Input
                type="password"
                placeholder="Confirm new password"
                value={resetPasswordConfirm}
                onChange={(e) => setResetPasswordConfirm(e.target.value)}
                className="text-xs"
              />
            </div>
          </div>

          <DialogFooter className="pt-2 gap-2">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={submitting || !recoveryKey.trim()}
              className="gap-2 bg-primary text-primary-foreground font-semibold"
            >
              {submitting ? (
                <>
                  <div className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                  Unlocking &amp; Binding...
                </>
              ) : (
                <>
                  <KeyRound className="h-4 w-4" />
                  Unlock Database
                </>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
