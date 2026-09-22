import { useCallback, useEffect, useState } from "react";
import { rpc, type HealthCheckResult } from "@/gen/rpc";
import { Button } from "@/components/ui/button";
import { ShieldAlert, RefreshCw, ArchiveRestore, Stethoscope } from "lucide-react";

interface CorruptionBannerProps {
  onOpenRestore?: () => void;
  onOpenDiagnostics?: () => void;
}

export function CorruptionBanner({
  onOpenRestore,
  onOpenDiagnostics,
}: CorruptionBannerProps) {
  const [health, setHealth] = useState<HealthCheckResult | null>(null);
  const [checking, setChecking] = useState(false);

  const checkHealth = useCallback(async () => {
    setChecking(true);
    try {
      const res = await rpc("system.health_check", {});
      if (res.ok && res.data) {
        setHealth(res.data);
      }
    } catch (e: unknown) {
      console.error("Failed to run system health check:", e);
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => {
    void checkHealth();
  }, [checkHealth]);

  if (!health || (health.healthy && !health.is_read_only)) {
    return null;
  }

  return (
    <aside
      aria-label="Database health alert"
      className="mb-6 rounded-[var(--radius-lg)] border border-destructive/40 bg-destructive/10 p-4 text-destructive dark:border-destructive/30 dark:bg-destructive/15 shadow-sm"
    >
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-destructive/20 text-destructive shrink-0 mt-0.5">
            <ShieldAlert className="h-5 w-5" />
          </div>
          <div>
            <h2 className="text-sm font-bold tracking-tight text-destructive">
              Database Integrity Warning &mdash; Running in Read-Only Mode
            </h2>
            <p className="text-xs text-muted-foreground mt-1 leading-relaxed max-w-2xl">
              {health.error_message ||
                "Database integrity or foreign key check failed. To prevent irreversible data loss, Coachify has locked counter operations into read-only mode."}
            </p>
            <div className="flex flex-wrap items-center gap-3 mt-2 text-[11px] text-muted-foreground font-mono">
              <span>Integrity: {health.integrity_ok ? "OK" : "FAILED"}</span>
              <span>•</span>
              <span>Foreign Keys: {health.foreign_keys_ok ? "OK" : "FAILED"}</span>
              <span>•</span>
              <span>Mode: {health.is_read_only ? "READ-ONLY" : "READ-WRITE"}</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0 self-end md:self-center">
          {onOpenRestore && (
            <Button
              size="sm"
              variant="destructive"
              className="gap-1.5 text-xs font-semibold"
              onClick={onOpenRestore}
            >
              <ArchiveRestore className="h-3.5 w-3.5" />
              Restore from Backup
            </Button>
          )}
          {onOpenDiagnostics && (
            <Button
              size="sm"
              variant="outline"
              className="gap-1.5 text-xs font-semibold"
              onClick={onOpenDiagnostics}
            >
              <Stethoscope className="h-3.5 w-3.5" />
              View Diagnostics
            </Button>
          )}
          <Button
            size="sm"
            variant="ghost"
            className="h-8 w-8 p-0 text-muted-foreground"
            title="Re-check health status"
            disabled={checking}
            onClick={() => void checkHealth()}
          >
            <RefreshCw className={`h-3.5 w-3.5 ${checking ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>
    </aside>
  );
}
