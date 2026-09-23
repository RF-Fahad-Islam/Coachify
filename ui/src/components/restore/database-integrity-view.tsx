import { useState, useCallback } from "react";
import { rpc, type DbIntegrityResult } from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  CheckCircle2,
  XCircle,
  Database,
  ShieldAlert,
  Link as LinkIcon,
  Scale,
  History,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface CheckCardProps {
  label: string;
  ok: boolean | undefined;
  icon: typeof CheckCircle2;
}

function CheckCard({ label, ok, icon: Icon }: CheckCardProps) {
  if (ok === undefined) {
    return (
      <div className="rounded-lg border border-border bg-muted/40 p-4 flex flex-col items-center gap-2">
        <Icon className="h-5 w-5 text-muted-foreground/40" aria-hidden="true" />
        <span className="text-xs text-muted-foreground text-center">{label}</span>
        <span className="text-[11px] text-muted-foreground">—</span>
      </div>
    );
  }
  return (
    <div
      className={cn(
        "rounded-lg border p-4 flex flex-col items-center gap-2",
        ok
          ? "border-emerald-500/30 bg-emerald-500/5"
          : "border-destructive/30 bg-destructive/5"
      )}
    >
      <Icon
        className={cn("h-5 w-5", ok ? "text-emerald-500" : "text-destructive")}
        aria-hidden="true"
      />
      <span
        className={cn(
          "text-xs font-medium text-center",
          ok ? "text-emerald-700 dark:text-emerald-300" : "text-destructive"
        )}
      >
        {label}
      </span>
      <span
        className={cn(
          "text-[11px] font-semibold",
          ok ? "text-emerald-600 dark:text-emerald-400" : "text-destructive"
        )}
      >
        {ok ? "✓ OK" : "✗ Failed"}
      </span>
    </div>
  );
}

export function DatabaseIntegrityView() {
  const [result, setResult] = useState<DbIntegrityResult | null>(null);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runCheck = useCallback(async (full: boolean) => {
    setRunning(true);
    setError(null);
    const res = await rpc("integrity.run_database_integrity_checks", { full });
    setRunning(false);
    if (res.ok && res.data) {
      setResult(res.data);
    } else {
      setError(res.error?.message ?? "Integrity check failed");
    }
  }, []);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2">
            <Database className="h-5 w-5 text-primary" />
            Database Integrity Checks
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Run 4-layer integrity checks: PRAGMA, foreign keys, balance reconciliation,
            audit chain.
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => void runCheck(false)}
            disabled={running}
            className="gap-1.5 text-xs"
          >
            <Database className="h-3.5 w-3.5" />
            {running ? "Running..." : "Quick Check"}
          </Button>
          <Button
            size="sm"
            onClick={() => void runCheck(true)}
            disabled={running}
            className="gap-1.5 text-xs"
          >
            <Database className="h-3.5 w-3.5" />
            {running ? "Running..." : "Full Check"}
          </Button>
        </div>
      </div>

      {/* Safety lock emergency banner */}
      {result?.is_read_only_locked && (
        <div className="rounded-lg border-2 border-destructive bg-destructive/10 p-4 flex items-start gap-3">
          <ShieldAlert
            className="h-5 w-5 text-destructive shrink-0 mt-0.5"
            aria-hidden="true"
          />
          <div>
            <p className="text-sm font-bold text-destructive">
              ⚠️ Safety Lock Active — Database is in Read-Only Mode
            </p>
            <p className="text-xs text-destructive/80 mt-1">
              Corruption was detected and the safety lock has been engaged to prevent
              further data loss. Export your data immediately and contact support.
            </p>
          </div>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* No result yet */}
      {!result && !running && !error && (
        <Card className="shadow-sm">
          <CardContent className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <Database className="h-10 w-10 mb-3 opacity-30" aria-hidden="true" />
            <p className="text-sm font-medium">No check run yet</p>
            <p className="text-xs mt-1">
              Click "Quick Check" or "Full Check" to run a database integrity check.
            </p>
          </CardContent>
        </Card>
      )}

      {running && (
        <Card className="shadow-sm">
          <CardContent className="flex flex-col items-center justify-center py-12">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mb-3" />
            <p className="text-sm text-muted-foreground">Running integrity checks...</p>
            <p className="text-xs text-muted-foreground mt-1">
              This may take a few seconds.
            </p>
          </CardContent>
        </Card>
      )}

      {/* Results */}
      {result && !running && (
        <>
          {/* Overall status banner */}
          <div
            className={cn(
              "rounded-lg border p-4 flex items-center justify-between",
              result.overall_ok
                ? "border-emerald-500/30 bg-emerald-500/10"
                : "border-destructive/30 bg-destructive/10"
            )}
          >
            <div className="flex items-center gap-3">
              {result.overall_ok ? (
                <CheckCircle2 className="h-6 w-6 text-emerald-500" aria-hidden="true" />
              ) : (
                <XCircle className="h-6 w-6 text-destructive" aria-hidden="true" />
              )}
              <div>
                <p
                  className={cn(
                    "text-sm font-bold",
                    result.overall_ok
                      ? "text-emerald-700 dark:text-emerald-300"
                      : "text-destructive"
                  )}
                >
                  {result.overall_ok ? "All Checks Passed" : "Integrity Checks Failed"}
                </p>
                <p className="text-xs text-muted-foreground">
                  {result.check_type === "full" ? "Full" : "Quick"} check ·{" "}
                  {result.duration_ms}ms ·{" "}
                  {new Date(result.completed_at).toLocaleString()}
                </p>
              </div>
            </div>
          </div>

          {/* 4-layer check cards */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
            <CheckCard
              label="PRAGMA Integrity"
              ok={result.pragma_integrity_ok}
              icon={Database}
            />
            <CheckCard label="Foreign Keys" ok={result.foreign_keys_ok} icon={LinkIcon} />
            <CheckCard
              label="Balance Reconciliation"
              ok={result.balance_reconciliation_ok}
              icon={Scale}
            />
            <CheckCard label="Audit Chain" ok={result.audit_chain_ok} icon={History} />
          </div>

          {/* Error details */}
          {result.errors.length > 0 && (
            <Card className="shadow-sm border-destructive/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold text-destructive">
                  Errors ({result.errors.length})
                </CardTitle>
                <CardDescription className="text-xs">
                  Details of integrity violations detected.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-1.5">
                {result.errors.map((e, i) => (
                  <div
                    key={i}
                    className="text-xs text-destructive bg-destructive/5 border border-destructive/20 rounded px-3 py-1.5 font-mono"
                  >
                    {e}
                  </div>
                ))}
              </CardContent>
            </Card>
          )}
        </>
      )}
    </div>
  );
}
