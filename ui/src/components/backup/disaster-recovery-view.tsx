import { useState, useCallback } from "react";
import { rpc, type RecoveryScoreResult, type DrillResult } from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableRow, TableCell } from "@/components/ui/table";
import {
  ShieldCheck,
  ShieldAlert,
  RefreshCw,
  Play,
  CheckCircle2,
  XCircle,
  AlertTriangle,
} from "lucide-react";
import { cn } from "@/lib/utils";

function ScoreBar({ value, max = 25 }: { value: number; max?: number }) {
  const pct = Math.min(100, (value / max) * 100);
  const color =
    pct >= 80
      ? "bg-emerald-500"
      : pct >= 60
        ? "bg-amber-500"
        : pct >= 40
          ? "bg-orange-500"
          : "bg-destructive";
  return (
    <div className="flex items-center gap-2 w-full">
      <div className="flex-1 h-2 rounded-full bg-muted overflow-hidden">
        <div
          className={cn("h-full rounded-full transition-all", color)}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-xs font-mono text-muted-foreground w-10 text-right">
        {value}/{max}
      </span>
    </div>
  );
}

function DrillOutcomeBadge({ outcome }: { outcome: string }) {
  if (outcome === "passed") {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20">
        <CheckCircle2 className="h-3 w-3" aria-hidden="true" />
        Passed
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-semibold bg-destructive/10 text-destructive border border-destructive/20">
      <XCircle className="h-3 w-3" aria-hidden="true" />
      Failed
    </span>
  );
}

export function DisasterRecoveryView() {
  const [score, setScore] = useState<RecoveryScoreResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [drillResult, setDrillResult] = useState<DrillResult | null>(null);
  const [drillRunning, setDrillRunning] = useState(false);
  const [drillError, setDrillError] = useState<string | null>(null);

  const loadScore = useCallback(async () => {
    setLoading(true);
    setError(null);
    const res = await rpc("integrity.get_disaster_recovery_score", {});
    if (res.ok && res.data) {
      setScore(res.data);
    } else {
      setError(res.error?.message ?? "Failed to load recovery score");
    }
    setLoading(false);
  }, []);

  const runDrill = useCallback(async () => {
    setDrillRunning(true);
    setDrillError(null);
    setDrillResult(null);
    const res = await rpc("integrity.run_verification_drill", { backup_id: null });
    setDrillRunning(false);
    if (res.ok && res.data) {
      setDrillResult(res.data);
      void loadScore();
    } else {
      setDrillError(res.error?.message ?? "Drill failed");
    }
  }, [loadScore]);

  const totalScore = score?.total_score ?? 0;
  const scoreColor =
    totalScore >= 80
      ? "text-emerald-600 dark:text-emerald-400"
      : totalScore >= 60
        ? "text-amber-600 dark:text-amber-400"
        : totalScore >= 40
          ? "text-orange-600 dark:text-orange-400"
          : "text-destructive";
  const scoreLabel =
    totalScore >= 80
      ? "Excellent"
      : totalScore >= 60
        ? "Good"
        : totalScore >= 40
          ? "Fair"
          : "At Risk";

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-primary" />
            Disaster Recovery Readiness
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            100-point score assessing your backup strategy and recovery posture.
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void loadScore()}
          disabled={loading}
          className="gap-1.5 text-xs"
        >
          <RefreshCw className={cn("h-3.5 w-3.5", loading && "animate-spin")} />
          {loading ? "Loading..." : "Refresh"}
        </Button>
      </div>

      {error && (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Score display */}
      {score ? (
        <>
          <Card className="shadow-sm">
            <CardContent className="p-6">
              <div className="flex items-center gap-6">
                <div className="text-center">
                  <div className={cn("text-6xl font-bold tabular-nums", scoreColor)}>
                    {score.total_score}
                  </div>
                  <div className="text-sm text-muted-foreground mt-1">/ 100</div>
                  <div className={cn("text-sm font-semibold mt-1", scoreColor)}>
                    {scoreLabel}
                  </div>
                </div>
                <div className="flex-1 space-y-3">
                  <div>
                    <div className="flex justify-between text-xs text-muted-foreground mb-1">
                      <span>Backup Recency</span>
                    </div>
                    <ScoreBar value={score.backup_recency_score} />
                  </div>
                  <div>
                    <div className="flex justify-between text-xs text-muted-foreground mb-1">
                      <span>External Copy</span>
                    </div>
                    <ScoreBar value={score.external_copy_score} />
                  </div>
                  <div>
                    <div className="flex justify-between text-xs text-muted-foreground mb-1">
                      <span>Drill Success</span>
                    </div>
                    <ScoreBar value={score.drill_success_score} />
                  </div>
                  <div>
                    <div className="flex justify-between text-xs text-muted-foreground mb-1">
                      <span>DB Integrity</span>
                    </div>
                    <ScoreBar value={score.db_integrity_score} />
                  </div>
                </div>
              </div>
              <div className="text-[11px] text-muted-foreground mt-4 text-right">
                Calculated at {new Date(score.calculated_at).toLocaleString()}
              </div>
            </CardContent>
          </Card>

          {/* Warnings */}
          {score.warnings.length > 0 && (
            <Card className="shadow-sm border-amber-500/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-semibold text-amber-700 dark:text-amber-300 flex items-center gap-2">
                  <AlertTriangle className="h-4 w-4" aria-hidden="true" />
                  Warnings ({score.warnings.length})
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-1.5">
                {score.warnings.map((w, i) => (
                  <div
                    key={i}
                    className="flex items-start gap-2 text-xs text-amber-700 dark:text-amber-300 bg-amber-500/5 border border-amber-500/20 rounded px-3 py-2"
                  >
                    <AlertTriangle
                      className="h-3.5 w-3.5 shrink-0 mt-0.5"
                      aria-hidden="true"
                    />
                    {w}
                  </div>
                ))}
              </CardContent>
            </Card>
          )}
        </>
      ) : !loading && !error ? (
        <Card className="shadow-sm">
          <CardContent className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <ShieldAlert className="h-10 w-10 mb-3 opacity-30" aria-hidden="true" />
            <p className="text-sm font-medium">Score not loaded</p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => void loadScore()}
              className="mt-3 gap-1.5 text-xs"
            >
              Load Score
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {/* Verification Drill */}
      <Card className="shadow-sm">
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-semibold">Verification Drill</CardTitle>
          <CardDescription className="text-xs">
            Run a non-destructive integrity check against the most recent backup.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button
            onClick={() => void runDrill()}
            disabled={drillRunning}
            className="gap-1.5 text-xs"
          >
            <Play className="h-3.5 w-3.5" aria-hidden="true" />
            {drillRunning ? "Running Drill..." : "Run Verification Drill"}
          </Button>

          {drillError && (
            <div className="rounded bg-destructive/10 border border-destructive/20 px-3 py-2 text-xs text-destructive">
              {drillError}
            </div>
          )}

          {drillResult && (
            <div className="rounded-lg border border-border bg-muted/30 overflow-hidden">
              <div className="flex items-center justify-between px-4 py-2 bg-muted/50 border-b border-border">
                <span className="text-xs font-semibold">Drill Result</span>
                <DrillOutcomeBadge outcome={drillResult.outcome} />
              </div>
              <Table>
                <TableBody>
                  <TableRow>
                    <TableCell className="text-xs text-muted-foreground">
                      Duration
                    </TableCell>
                    <TableCell className="text-xs font-medium text-right">
                      {drillResult.duration_ms}ms
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="text-xs text-muted-foreground">
                      PRAGMA Integrity
                    </TableCell>
                    <TableCell className="text-right">
                      {drillResult.pragma_integrity_ok ? (
                        <CheckCircle2
                          className="h-4 w-4 text-emerald-500 ml-auto"
                          aria-label="OK"
                        />
                      ) : (
                        <XCircle
                          className="h-4 w-4 text-destructive ml-auto"
                          aria-label="Failed"
                        />
                      )}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="text-xs text-muted-foreground">
                      Foreign Keys
                    </TableCell>
                    <TableCell className="text-right">
                      {drillResult.foreign_keys_ok ? (
                        <CheckCircle2
                          className="h-4 w-4 text-emerald-500 ml-auto"
                          aria-label="OK"
                        />
                      ) : (
                        <XCircle
                          className="h-4 w-4 text-destructive ml-auto"
                          aria-label="Failed"
                        />
                      )}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="text-xs text-muted-foreground">
                      Balance Reconciled
                    </TableCell>
                    <TableCell className="text-right">
                      {drillResult.balance_reconciled ? (
                        <CheckCircle2
                          className="h-4 w-4 text-emerald-500 ml-auto"
                          aria-label="OK"
                        />
                      ) : (
                        <XCircle
                          className="h-4 w-4 text-destructive ml-auto"
                          aria-label="Failed"
                        />
                      )}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="text-xs text-muted-foreground">
                      Completed
                    </TableCell>
                    <TableCell className="text-xs font-medium text-right">
                      {new Date(drillResult.completed_at).toLocaleString()}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
              {drillResult.error_details && (
                <div className="px-4 py-2 text-xs text-destructive border-t border-border">
                  {drillResult.error_details}
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
