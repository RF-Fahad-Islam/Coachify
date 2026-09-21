import { useState, useEffect, useRef, useCallback } from "react";
import {
  Database,
  Sparkles,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
  Play,
  ArrowRight,
  ShieldAlert,
} from "lucide-react";
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
import { Progress } from "@/components/ui/progress";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  rpc,
  type DemoProfile,
  type GenerationSummary,
  type GeneratorStatusResponse,
} from "@/gen/rpc";

interface DemoGeneratorDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDatabaseSwitched?: () => void;
}

export function DemoGeneratorDialog({
  open,
  onOpenChange,
  onDatabaseSwitched,
}: DemoGeneratorDialogProps) {
  const [profile, setProfile] = useState<DemoProfile>("demo");
  const [seed, setSeed] = useState<number>(42);
  const [outFilename, setOutFilename] = useState<string>("");
  const [isGenerating, setIsGenerating] = useState<boolean>(false);
  const [progressPercent, setProgressPercent] = useState<number>(0);
  const [progressStage, setProgressStage] = useState<string>("");
  const [summary, setSummary] = useState<GenerationSummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSwitching, setIsSwitching] = useState<boolean>(false);

  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const stopPolling = useCallback(() => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
  }, []);

  const startPolling = useCallback(() => {
    stopPolling();
    pollIntervalRef.current = setInterval(async () => {
      try {
        const res = await rpc("generator.get_status", {});
        if (res.ok && res.data) {
          setProgressPercent(res.data.percent);
          setProgressStage(res.data.stage);
        }
      } catch {
        // Ignore polling errors during busy execution
      }
    }, 250);
  }, [stopPolling]);

  useEffect(() => {
    return () => {
      stopPolling();
    };
  }, [stopPolling]);

  const handleStartGeneration = async () => {
    setIsGenerating(true);
    setError(null);
    setSummary(null);
    setProgressPercent(5);
    setProgressStage("Initializing generator...");
    startPolling();

    try {
      const filename = outFilename.trim() ? outFilename.trim() : null;
      const res = await rpc("generator.generate_dataset", {
        profile,
        seed: Number.isNaN(seed) ? null : seed,
        out_filename: filename,
      });

      stopPolling();
      setIsGenerating(false);

      if (res.ok && res.data) {
        setSummary(res.data);
        setProgressPercent(100);
        setProgressStage("Dataset Generated Successfully");
      } else {
        const msg = res.error?.message || "Failed to generate dataset";
        setError(msg);
      }
    } catch (err) {
      stopPolling();
      setIsGenerating(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSwitchToGenerated = async () => {
    if (!summary) return;
    setIsSwitching(true);
    setError(null);

    try {
      const parts = summary.db_path.split(/[\\/]/);
      const filename = parts[parts.length - 1] || "";
      const res = await rpc("generator.switch_database", {
        target_filename: filename,
      });

      setIsSwitching(false);
      if (res.ok) {
        onDatabaseSwitched?.();
        onOpenChange(false);
        if (typeof window !== "undefined") {
          window.location.reload();
        }
      } else {
        setError(res.error?.message || "Failed to switch database");
      }
    } catch (err) {
      setIsSwitching(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-amber-500" />
            Generate Demo & Benchmark Dataset
          </DialogTitle>
          <DialogDescription>
            Generate realistic synthetic institutional datasets with authentic Bangladeshi
            demographics, sibling grouping, academic rosters, and financial records.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {error && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {!summary ? (
            <>
              {/* Profile Selection */}
              <div className="space-y-2">
                <label className="text-xs font-semibold text-foreground">
                  Select Dataset Profile
                </label>
                <div className="grid grid-cols-2 gap-3">
                  <button
                    type="button"
                    disabled={isGenerating}
                    onClick={() => {
                      setProfile("demo");
                      setOutFilename("demo_500.db");
                    }}
                    className={`p-3 rounded-xl border text-left transition-all cursor-pointer ${
                      profile === "demo"
                        ? "border-primary bg-primary/5 ring-1 ring-primary"
                        : "border-border bg-card hover:bg-muted/50"
                    }`}
                  >
                    <div className="font-semibold text-xs text-foreground flex items-center justify-between">
                      <span>Demo Profile</span>
                      <Badge variant="outline" className="text-[10px]">
                        ~500 Students
                      </Badge>
                    </div>
                    <p className="text-[11px] text-muted-foreground mt-1">
                      Quick onboarding and testing. Generates in under 5 seconds.
                    </p>
                  </button>

                  <button
                    type="button"
                    disabled={isGenerating}
                    onClick={() => {
                      setProfile("load");
                      setOutFilename("load_10k.db");
                    }}
                    className={`p-3 rounded-xl border text-left transition-all cursor-pointer ${
                      profile === "load"
                        ? "border-primary bg-primary/5 ring-1 ring-primary"
                        : "border-border bg-card hover:bg-muted/50"
                    }`}
                  >
                    <div className="font-semibold text-xs text-foreground flex items-center justify-between">
                      <span>Load Benchmark</span>
                      <Badge variant="outline" className="text-[10px]">
                        25,000 Students
                      </Badge>
                    </div>
                    <p className="text-[11px] text-muted-foreground mt-1">
                      10k active + 15k archived for stress and FTS5 search testing.
                    </p>
                  </button>
                </div>
              </div>

              {/* Seed & Filename */}
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    Deterministic Seed
                  </label>
                  <Input
                    type="number"
                    value={seed}
                    disabled={isGenerating}
                    onChange={(e) => setSeed(parseInt(e.target.value, 10) || 0)}
                    placeholder="e.g. 42"
                    className="h-8 text-xs"
                  />
                  <p className="text-[10px] text-muted-foreground">
                    Same seed generates exact identical records.
                  </p>
                </div>

                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-foreground">
                    Database Filename (Optional)
                  </label>
                  <Input
                    type="text"
                    value={outFilename}
                    disabled={isGenerating}
                    onChange={(e) => setOutFilename(e.target.value)}
                    placeholder={profile === "demo" ? "demo_500.db" : "load_10k.db"}
                    className="h-8 text-xs"
                  />
                  <p className="text-[10px] text-muted-foreground">
                    Saved sandboxed inside the demo directory.
                  </p>
                </div>
              </div>

              {/* Progress indicator */}
              {isGenerating && (
                <div className="p-3 rounded-lg bg-muted/40 border border-border space-y-2">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-medium text-foreground flex items-center gap-2">
                      <RefreshCw className="h-3.5 w-3.5 animate-spin text-primary" />
                      {progressStage || "Generating dataset..."}
                    </span>
                    <span className="font-bold text-primary">{progressPercent}%</span>
                  </div>
                  <Progress value={progressPercent} className="h-2" />
                </div>
              )}
            </>
          ) : (
            /* Summary View */
            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-900 dark:text-emerald-200 flex items-center gap-2 text-xs font-medium">
                <CheckCircle2 className="h-4 w-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
                <span>Dataset successfully generated and verified!</span>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">
                    Students
                  </span>
                  <strong className="text-sm font-bold text-foreground">
                    {summary.student_count.toLocaleString()}
                  </strong>
                </div>
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">
                    Guardians
                  </span>
                  <strong className="text-sm font-bold text-foreground">
                    {summary.guardian_count.toLocaleString()}
                  </strong>
                </div>
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">Batches</span>
                  <strong className="text-sm font-bold text-foreground">
                    {summary.batch_count.toLocaleString()}
                  </strong>
                </div>
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">Charges</span>
                  <strong className="text-sm font-bold text-foreground">
                    {summary.charge_count.toLocaleString()}
                  </strong>
                </div>
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">
                    Payments
                  </span>
                  <strong className="text-sm font-bold text-foreground">
                    {summary.payment_count.toLocaleString()}
                  </strong>
                </div>
                <div className="p-2.5 rounded-lg border border-border bg-card text-xs">
                  <span className="text-muted-foreground block text-[10px]">
                    Generation Time
                  </span>
                  <strong className="text-sm font-bold text-foreground">
                    {(summary.duration_ms / 1000).toFixed(2)}s
                  </strong>
                </div>
              </div>

              <div className="p-2.5 rounded-lg border border-border bg-muted/30 text-xs font-mono break-all">
                <span className="text-muted-foreground block text-[10px] font-sans">
                  Target Database Location
                </span>
                {summary.db_path}
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          {!summary ? (
            <>
              <Button
                variant="outline"
                size="sm"
                disabled={isGenerating}
                onClick={() => onOpenChange(false)}
              >
                Cancel
              </Button>
              <Button
                size="sm"
                disabled={isGenerating}
                onClick={handleStartGeneration}
                className="gap-1.5"
              >
                {isGenerating ? (
                  <>
                    <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    <Play className="h-3.5 w-3.5 fill-current" />
                    Generate Dataset
                  </>
                )}
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="outline"
                size="sm"
                disabled={isSwitching}
                onClick={() => {
                  setSummary(null);
                  onOpenChange(false);
                }}
              >
                Close
              </Button>
              <Button
                size="sm"
                disabled={isSwitching}
                onClick={handleSwitchToGenerated}
                className="gap-1.5 bg-amber-600 hover:bg-amber-700 text-white font-semibold"
              >
                {isSwitching ? (
                  <>
                    <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                    Switching Database...
                  </>
                ) : (
                  <>
                    <ArrowRight className="h-3.5 w-3.5" />
                    Switch to Demo Database
                  </>
                )}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function DeveloperDemoCard() {
  const [status, setStatus] = useState<GeneratorStatusResponse | null>(null);
  const [dialogOpen, setDialogOpen] = useState<boolean>(false);
  const [isSwitching, setIsSwitching] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await rpc("generator.get_status", {});
      if (res.ok && res.data) {
        setStatus(res.data);
      }
    } catch {
      // Ignore in non-Tauri or testing environments
    }
  }, []);

  useEffect(() => {
    void fetchStatus();
  }, [fetchStatus]);

  const handleRevertToProduction = async () => {
    setIsSwitching(true);
    setError(null);
    try {
      const res = await rpc("generator.switch_to_production_database", {});
      setIsSwitching(false);
      if (res.ok) {
        if (typeof window !== "undefined") {
          window.location.reload();
        }
      } else {
        setError(res.error?.message || "Failed to switch to production");
      }
    } catch (err) {
      setIsSwitching(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between text-base">
            <span className="flex items-center gap-2">
              <Database className="h-4 w-4 text-primary" />
              Developer & Demo Datasets
            </span>
            {status?.is_demo_active ? (
              <Badge
                variant="outline"
                className="border-amber-500 text-amber-600 dark:text-amber-400 gap-1"
              >
                <span className="h-1.5 w-1.5 rounded-full bg-amber-500 animate-pulse" />
                Demo Mode Active
              </Badge>
            ) : (
              <Badge
                variant="outline"
                className="border-emerald-500 text-emerald-600 dark:text-emerald-400 gap-1"
              >
                <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                Production Mode
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-xs text-muted-foreground leading-relaxed">
            Generate synthetic datasets for staff onboarding, feature demonstration, and
            performance load benchmarking. Demo databases are safely sandboxed in their
            own directory and will never overwrite or alter your institutional production
            records.
          </p>

          {error && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {status && (
            <div className="p-3 rounded-lg bg-muted/40 border border-border space-y-1.5 text-xs">
              <div className="text-muted-foreground text-[11px]">
                Active Database Path:
              </div>
              <div className="font-mono text-xs break-all text-foreground">
                {status.active_database_path || "Default Production Database"}
              </div>
            </div>
          )}

          {status?.is_demo_active && (
            <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 text-xs flex items-center gap-2">
              <ShieldAlert className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
              <span>
                You are currently connected to a synthetic demo database. Restarting the
                application or clicking below will safely return to your primary
                production database.
              </span>
            </div>
          )}

          <div className="flex flex-wrap items-center gap-2.5 pt-1">
            <Button
              onClick={() => setDialogOpen(true)}
              className="gap-2 font-semibold text-xs"
            >
              <Sparkles className="h-3.5 w-3.5" />
              Open Demo Generator
            </Button>

            {status?.is_demo_active && (
              <Button
                variant="outline"
                onClick={handleRevertToProduction}
                disabled={isSwitching}
                className="gap-2 font-medium text-xs border-amber-500/40 text-amber-900 dark:text-amber-200 hover:bg-amber-500/10"
              >
                {isSwitching ? (
                  <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                ) : (
                  <ArrowRight className="h-3.5 w-3.5" />
                )}
                Revert to Production
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      <DemoGeneratorDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onDatabaseSwitched={fetchStatus}
      />
    </>
  );
}
