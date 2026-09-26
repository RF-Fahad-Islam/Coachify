import { useState, useRef, useCallback } from "react";
import {
  Camera,
  AlertTriangle,
  Volume2,
  VolumeX,
  UserCheck,
  Clock,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export interface AttendanceRecord {
  id: string;
  studentId: string;
  studentName: string;
  rollNumber: string;
  className: string;
  timestamp: string;
  status: "present" | "late" | "duplicate";
}

export interface MobileAttendanceScannerProps {
  onScanBadge?: (barcode: string) => Promise<{
    studentName: string;
    rollNumber: string;
    className: string;
  }>;
  audioFeedbackEnabled?: boolean;
}

export function MobileAttendanceScanner({
  onScanBadge,
  audioFeedbackEnabled = true,
}: MobileAttendanceScannerProps) {
  const [manualInput, setManualInput] = useState("");
  const [soundEnabled, setSoundEnabled] = useState(audioFeedbackEnabled);
  const [lastScanned, setLastScanned] = useState<AttendanceRecord | null>(null);
  const [recentScans, setRecentScans] = useState<AttendanceRecord[]>([]);
  const [flashSuccess, setFlashSuccess] = useState(false);
  const [flashDuplicate, setFlashDuplicate] = useState(false);
  const [scanLatencyMs, setScanLatencyMs] = useState<number | null>(null);

  // 5-second duplicate debounce guard (barcode -> last scan timestamp)
  const debounceMap = useRef<Map<string, number>>(new Map());

  // Simulate audio beep
  const playBeep = useCallback(
    (isSuccess: boolean) => {
      if (!soundEnabled || typeof window === "undefined" || !window.AudioContext) return;
      try {
        const ctx = new (
          window.AudioContext ||
          (window as unknown as { webkitAudioContext: typeof AudioContext })
            .webkitAudioContext
        )();
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.connect(gain);
        gain.connect(ctx.destination);

        if (isSuccess) {
          osc.frequency.setValueAtTime(1046.5, ctx.currentTime); // C6 bright high chime
          gain.gain.setValueAtTime(0.2, ctx.currentTime);
          gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.12);
          osc.start(ctx.currentTime);
          osc.stop(ctx.currentTime + 0.12);
        } else {
          osc.frequency.setValueAtTime(300, ctx.currentTime); // Low buzz for duplicate
          gain.gain.setValueAtTime(0.2, ctx.currentTime);
          gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.2);
          osc.start(ctx.currentTime);
          osc.stop(ctx.currentTime + 0.2);
        }
      } catch {
        // AudioContext muted or blocked by policy
      }
    },
    [soundEnabled]
  );

  // Simulate haptic vibration
  const triggerHaptic = useCallback((isSuccess: boolean) => {
    if (typeof navigator !== "undefined" && navigator.vibrate) {
      navigator.vibrate(isSuccess ? 40 : [80, 50, 80]);
    }
  }, []);

  const handleProcessScan = useCallback(
    async (rawCode: string) => {
      const code = rawCode.trim();
      if (!code) return;

      const startTime = performance.now();
      const nowMs = Date.now();
      const lastTime = debounceMap.current.get(code) || 0;

      // AC-6: 5-second duplicate debounce guard
      if (nowMs - lastTime < 5000) {
        setFlashDuplicate(true);
        playBeep(false);
        triggerHaptic(false);
        setTimeout(() => setFlashDuplicate(false), 500);

        const dupRecord: AttendanceRecord = {
          id: `dup_${nowMs}`,
          studentId: code,
          studentName: lastScanned?.studentName || `Student ${code}`,
          rollNumber: lastScanned?.rollNumber || code,
          className: lastScanned?.className || "Class 10",
          timestamp: new Date().toLocaleTimeString(),
          status: "duplicate",
        };
        setLastScanned(dupRecord);
        return;
      }

      debounceMap.current.set(code, nowMs);

      try {
        let studentDetails = {
          studentName: `Student ${code.substring(code.length - 4)}`,
          rollNumber: `R-${code.substring(code.length - 3)}`,
          className: "Class 10 - Science",
        };

        if (onScanBadge) {
          studentDetails = await onScanBadge(code);
        }

        const latency = Math.round(performance.now() - startTime);
        setScanLatencyMs(latency);

        const record: AttendanceRecord = {
          id: `att_${nowMs}`,
          studentId: code,
          studentName: studentDetails.studentName,
          rollNumber: studentDetails.rollNumber,
          className: studentDetails.className,
          timestamp: new Date().toLocaleTimeString(),
          status: "present",
        };

        setLastScanned(record);
        setRecentScans((prev) => [record, ...prev.slice(0, 19)]);
        setFlashSuccess(true);
        playBeep(true);
        triggerHaptic(true);
        setTimeout(() => setFlashSuccess(false), 300);
      } catch {
        playBeep(false);
      }
    },
    [onScanBadge, playBeep, triggerHaptic, lastScanned]
  );

  return (
    <div className="flex flex-col gap-4 max-w-md mx-auto w-full p-3">
      {/* Viewfinder & Optical Feed */}
      <div
        data-testid="attendance-viewfinder"
        className={`relative aspect-[4/3] w-full rounded-2xl bg-slate-950 flex flex-col items-center justify-center overflow-hidden border-2 transition-all duration-150 ${
          flashSuccess
            ? "border-emerald-500 shadow-[0_0_30px_rgba(16,185,129,0.5)] bg-emerald-950/20"
            : flashDuplicate
              ? "border-amber-500 shadow-[0_0_30px_rgba(245,158,11,0.5)] bg-amber-950/20"
              : "border-primary/40"
        }`}
      >
        {/* Optical Scanning Target Crosshair */}
        <div className="absolute w-48 h-32 border-2 border-emerald-400/80 rounded-xl pointer-events-none flex items-center justify-center">
          <div className="w-full h-0.5 bg-emerald-400/80 animate-bounce" />
        </div>

        <Camera className="w-10 h-10 text-slate-600 mb-2" />
        <span className="text-xs text-slate-400 font-medium">Scanning Student Badge</span>

        {/* Top Floating Controls */}
        <div className="absolute top-3 right-3 flex items-center gap-2">
          <Button
            size="icon"
            variant="ghost"
            className="w-8 h-8 rounded-full bg-slate-900/80 text-slate-300 hover:text-white border border-slate-700"
            onClick={() => setSoundEnabled(!soundEnabled)}
            data-testid="toggle-sound-btn"
          >
            {soundEnabled ? (
              <Volume2 className="w-4 h-4" />
            ) : (
              <VolumeX className="w-4 h-4" />
            )}
          </Button>
        </div>

        {/* Bottom Latency Indicator */}
        <div className="absolute bottom-3 left-3 flex items-center gap-2">
          <Badge
            variant="outline"
            className="text-[10px] bg-slate-900/80 text-emerald-400 border-slate-700 flex items-center gap-1 font-mono"
            data-testid="scan-sla-badge"
          >
            <Zap className="w-3 h-3 text-amber-400" />
            {scanLatencyMs !== null ? `${scanLatencyMs}ms SLA` : "<150ms Engine"}
          </Badge>
        </div>
      </div>

      {/* Manual Input / Hardware Scanner Fallback */}
      <div className="flex gap-2">
        <Input
          placeholder="Scan barcode or type Student ID..."
          value={manualInput}
          onChange={(e) => setManualInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              handleProcessScan(manualInput);
              setManualInput("");
            }
          }}
          className="text-xs font-mono"
          data-testid="barcode-input"
        />
        <Button
          size="sm"
          onClick={() => {
            handleProcessScan(manualInput);
            setManualInput("");
          }}
          disabled={!manualInput.trim()}
          data-testid="barcode-submit-btn"
        >
          Mark
        </Button>
      </div>

      {/* Active Scan Notification Banner */}
      {lastScanned && (
        <Card
          data-testid="last-scanned-card"
          className={`border transition-all ${
            lastScanned.status === "duplicate"
              ? "border-amber-500/30 bg-amber-500/10 text-amber-900 dark:text-amber-200"
              : "border-emerald-500/30 bg-emerald-500/10 text-emerald-900 dark:text-emerald-200"
          }`}
        >
          <CardContent className="p-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className={`w-9 h-9 rounded-full flex items-center justify-center shrink-0 ${
                  lastScanned.status === "duplicate"
                    ? "bg-amber-500/20 text-amber-600"
                    : "bg-emerald-500/20 text-emerald-600"
                }`}
              >
                {lastScanned.status === "duplicate" ? (
                  <AlertTriangle className="w-5 h-5" />
                ) : (
                  <UserCheck className="w-5 h-5" />
                )}
              </div>
              <div>
                <div className="text-xs font-bold">{lastScanned.studentName}</div>
                <div className="text-[11px] opacity-80 font-mono">
                  {lastScanned.rollNumber} • {lastScanned.className}
                </div>
              </div>
            </div>
            <Badge
              variant={lastScanned.status === "duplicate" ? "outline" : "secondary"}
              className="text-[10px] capitalize font-semibold"
            >
              {lastScanned.status === "duplicate" ? "Debounced (5s)" : "Marked Present"}
            </Badge>
          </CardContent>
        </Card>
      )}

      {/* Recent Scans History Roll */}
      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs text-muted-foreground font-medium px-1">
          <span>Recent Scans ({recentScans.length})</span>
          <span className="flex items-center gap-1 font-mono text-[11px]">
            <Clock className="w-3 h-3" /> Live Feed
          </span>
        </div>

        <div className="space-y-1.5 max-h-48 overflow-y-auto pr-1">
          {recentScans.length === 0 ? (
            <div className="text-center py-6 text-xs text-muted-foreground border border-dashed rounded-lg">
              No students scanned in this session yet.
            </div>
          ) : (
            recentScans.map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between p-2 rounded-lg bg-card border border-border/80 text-xs"
                data-testid={`scan-row-${item.id}`}
              >
                <div className="flex items-center gap-2">
                  <span className="font-semibold">{item.studentName}</span>
                  <span className="text-muted-foreground font-mono text-[11px]">
                    ({item.rollNumber})
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] text-muted-foreground font-mono">
                    {item.timestamp}
                  </span>
                  <Badge
                    variant="outline"
                    className="text-[9px] text-emerald-600 border-emerald-500/30"
                  >
                    Present
                  </Badge>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
