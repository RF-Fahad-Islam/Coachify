import { useState, type FormEvent } from "react";
import { rpc, type ReprintSheetResponse } from "@/gen/rpc";
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
import { RecoverySheetPrintable } from "@/components/setup/recovery-sheet-printable";
import { Printer, Lock, AlertTriangle, ShieldCheck } from "lucide-react";

export interface ReprintRecoveryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ReprintRecoveryDialog({
  open,
  onOpenChange,
}: ReprintRecoveryDialogProps) {
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [sheetData, setSheetData] = useState<ReprintSheetResponse | null>(null);

  const handleFetchSheet = async (e: FormEvent) => {
    e.preventDefault();
    if (!password) return;

    setErrorMsg(null);
    setSubmitting(true);
    try {
      const res = await rpc("recovery.reprint_sheet", {
        owner_password: password,
      });

      if (res.ok && res.data) {
        setSheetData(res.data);
      } else {
        setErrorMsg(
          res.error?.message ?? "Incorrect owner password or permission denied."
        );
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const handleClose = () => {
    setPassword("");
    setSheetData(null);
    setErrorMsg(null);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center gap-2 text-primary font-semibold text-sm mb-1">
            <Printer className="h-4 w-4" />
            <span>Institutional Key Vault</span>
          </div>
          <DialogTitle className="text-xl font-bold">
            Reprint Owner Recovery Sheet
          </DialogTitle>
          <DialogDescription className="text-xs">
            Reprinting the recovery sheet requires confirming the active Owner password.
            Keep the newly printed copy in your institutional safe.
          </DialogDescription>
        </DialogHeader>

        {errorMsg && (
          <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{errorMsg}</span>
          </div>
        )}

        {!sheetData ? (
          <form onSubmit={handleFetchSheet} className="space-y-4 pt-2">
            <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 flex items-start gap-2.5 text-xs">
              <ShieldCheck className="h-4 w-4 text-amber-600 shrink-0 mt-0.5" />
              <span>
                Only the Institute Owner may reprint the physical recovery sheet. This
                action is recorded permanently in the tamper-evident audit log.
              </span>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Confirm Owner Password *
              </label>
              <Input
                type="password"
                placeholder="Enter owner password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                autoFocus
                required
              />
            </div>

            <DialogFooter className="pt-2">
              <Button type="button" variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={submitting || !password}
                className="gap-2 bg-primary text-primary-foreground font-semibold"
              >
                {submitting ? (
                  <>
                    <div className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
                    Verifying...
                  </>
                ) : (
                  <>
                    <Lock className="h-4 w-4" />
                    Verify &amp; Display Sheet
                  </>
                )}
              </Button>
            </DialogFooter>
          </form>
        ) : (
          <div className="space-y-4 pt-2">
            <RecoverySheetPrintable
              institutionName={sheetData.institution_name}
              ownerUsername={sheetData.owner_username}
              academicSession="Current"
              createdAt={sheetData.created_at}
              recoveryKey={`CKREC-XXXX-XXXX-XXXX-XXXX-XXXX-${sheetData.recovery_key_checksum}`}
              checksum={sheetData.recovery_key_checksum}
              showPrintButton={true}
            />

            <DialogFooter className="print:hidden">
              <Button type="button" variant="outline" onClick={handleClose}>
                Close
              </Button>
            </DialogFooter>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
