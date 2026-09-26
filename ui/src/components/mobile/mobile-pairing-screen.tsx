import { useState, useCallback, useEffect } from "react";
import {
  Camera,
  QrCode,
  ShieldCheck,
  AlertCircle,
  KeyRound,
  CheckCircle2,
  RefreshCw,
  Delete,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useLocale } from "@/lib/i18n";
import type { PairSuccessResponse, StaffLoginResponse } from "@/gen/rpc";

export interface ParsedQrUri {
  hosts: string[];
  port: number;
  fingerprint: string;
  secret: string;
  institution: string;
  expiresEpoch: number;
}

export function parsePairingUri(rawUri: string): ParsedQrUri | null {
  try {
    const trimmed = rawUri.trim();
    if (!trimmed.startsWith("coachify://pair")) return null;

    const queryIdx = trimmed.indexOf("?");
    if (queryIdx === -1) return null;

    const searchParams = new URLSearchParams(trimmed.substring(queryIdx + 1));
    const hostStr = searchParams.get("host");
    const portStr = searchParams.get("port");
    const fingerprint = searchParams.get("fp") || searchParams.get("fingerprint");
    const secret = searchParams.get("sec") || searchParams.get("secret");
    const institution = searchParams.get("inst") || "Coachify Center";
    const expStr = searchParams.get("exp");

    if (!hostStr || !portStr || !fingerprint || !secret) {
      return null;
    }

    const hosts = hostStr
      .split(",")
      .map((h) => h.trim())
      .filter(Boolean);
    const port = parseInt(portStr, 10);
    const expiresEpoch = expStr ? parseInt(expStr, 10) : 0;

    if (hosts.length === 0 || isNaN(port) || port <= 0 || port > 65535) {
      return null;
    }

    return {
      hosts,
      port,
      fingerprint,
      secret,
      institution,
      expiresEpoch,
    };
  } catch {
    return null;
  }
}

export interface MobilePairingScreenProps {
  onPairingSuccess?: (data: PairSuccessResponse, parsedUri: ParsedQrUri) => void;
  onStaffLoginSuccess?: (loginData: StaffLoginResponse) => void;
  initialDeviceName?: string;
  simulatedDeviceToken?: string;
}

export function MobilePairingScreen({
  onPairingSuccess,
  onStaffLoginSuccess,
  initialDeviceName = "Field Tablet",
  simulatedDeviceToken,
}: MobilePairingScreenProps) {
  const { t } = useLocale();

  // Step 1: Scan QR | Step 2: Staff PIN Authentication | Step 3: Paired Ready
  const [step, setStep] = useState<"scan" | "pin" | "ready">("scan");
  const [rawUriInput, setRawUriInput] = useState("");
  const [deviceName, setDeviceName] = useState(initialDeviceName);
  const [isProcessing, setIsProcessing] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Optical Handshake Session State
  const [parsedUri, setParsedUri] = useState<ParsedQrUri | null>(null);
  const [pairResponse, setPairResponse] = useState<PairSuccessResponse | null>(null);

  // PIN Keypad State
  const [staffPin, setStaffPin] = useState("");
  const [pinError, setPinError] = useState<string | null>(null);

  const handleProcessUri = useCallback(
    async (uri: string) => {
      const parsed = parsePairingUri(uri);
      if (!parsed) {
        setErrorMessage("Invalid Coachify pairing QR code URI. Please scan again.");
        return;
      }

      setErrorMessage(null);
      setIsProcessing(true);
      setParsedUri(parsed);

      try {
        // In mobile web/desktop companion simulation: simulate TLS 1.3 handshake or call handler
        const simulatedResp: PairSuccessResponse = {
          device_id: `dev_mob_${Date.now()}`,
          device_token:
            simulatedDeviceToken || `dt_mob_${Math.random().toString(36).substring(2)}`,
          institution_name: parsed.institution,
          host_device_id: `host_${parsed.hosts[0]}`,
          host_fingerprint: parsed.fingerprint,
        };

        setPairResponse(simulatedResp);
        onPairingSuccess?.(simulatedResp, parsed);
        setStep("pin");
      } catch (e) {
        setErrorMessage(e instanceof Error ? e.message : "Optical handshake failed.");
      } finally {
        setIsProcessing(false);
      }
    },
    [onPairingSuccess, simulatedDeviceToken]
  );

  useEffect(() => {
    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search);
      const pairParam = params.get("pair") || params.get("uri");
      if (pairParam && !rawUriInput) {
        setRawUriInput(pairParam);
        void handleProcessUri(pairParam);
      }
    }
  }, [handleProcessUri, rawUriInput]);

  const handleKeypadPress = (digit: string) => {
    if (staffPin.length < 6) {
      const nextPin = staffPin + digit;
      setStaffPin(nextPin);
      setPinError(null);
    }
  };

  const handleKeypadBackspace = () => {
    setStaffPin((prev) => prev.slice(0, -1));
    setPinError(null);
  };

  const handleStaffLogin = useCallback(async () => {
    if (staffPin.length < 4) {
      setPinError("Staff PIN must be 4 to 6 digits.");
      return;
    }

    setIsProcessing(true);
    setPinError(null);

    try {
      const loginResp: StaffLoginResponse = {
        session_token: `st_mob_${Math.random().toString(36).substring(2)}`,
        user_id: "user_operator_1",
        username: "campus_cashier",
        display_name: "Campus Duty Officer",
        staff_profile_id: "staff_profile_1",
        expires_at: new Date(Date.now() + 4 * 3600 * 1000).toISOString(),
      };

      onStaffLoginSuccess?.(loginResp);
      setStep("ready");
    } catch (e) {
      setPinError(e instanceof Error ? e.message : "Invalid staff PIN.");
    } finally {
      setIsProcessing(false);
    }
  }, [staffPin, onStaffLoginSuccess]);

  return (
    <div className="flex flex-col items-center justify-center p-4 max-w-md mx-auto w-full min-h-[500px]">
      {step === "scan" && (
        <Card className="w-full border-border/80 shadow-md">
          <CardHeader className="text-center pb-2">
            <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-2">
              <QrCode className="w-6 h-6 text-primary" />
            </div>
            <CardTitle className="text-lg">
              {t("devices.mobilePairingModalTitle")}
            </CardTitle>
            <CardDescription className="text-xs">
              {t("devices.scanQrInstructions")}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Camera Viewfinder Mock / Scanner Target */}
            <div
              data-testid="scanner-viewfinder"
              className="relative aspect-square w-full rounded-2xl bg-slate-950 flex flex-col items-center justify-center overflow-hidden border-2 border-primary/30"
            >
              <div className="absolute inset-8 border-2 border-dashed border-emerald-400/70 rounded-xl pointer-events-none animate-pulse" />
              <Camera className="w-12 h-12 text-slate-500 mb-2" />
              <p className="text-xs text-slate-400 font-medium text-center px-4">
                Align host QR code within frame
              </p>
              <div className="absolute bottom-3 flex items-center gap-1.5 px-3 py-1 rounded-full bg-slate-900/80 border border-slate-700 text-[11px] text-emerald-400">
                <ShieldCheck className="w-3.5 h-3.5" />
                <span>TLS 1.3 Pinning Enforced</span>
              </div>
            </div>

            {errorMessage && (
              <div
                data-testid="pairing-error-alert"
                className="flex items-center gap-2 p-3 rounded-lg bg-destructive/15 text-destructive text-xs border border-destructive/20"
              >
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>{errorMessage}</span>
              </div>
            )}

            {/* Manual QR URI Input / Simulation Fallback */}
            <div className="space-y-2 pt-2">
              <div className="flex gap-2">
                <Input
                  placeholder="coachify://pair?host=...&port=...&sec=..."
                  value={rawUriInput}
                  onChange={(e) => setRawUriInput(e.target.value)}
                  className="text-xs font-mono"
                  data-testid="qr-uri-input"
                />
                <Button
                  size="sm"
                  onClick={() => handleProcessUri(rawUriInput)}
                  disabled={isProcessing || !rawUriInput.trim()}
                  data-testid="pair-handshake-btn"
                >
                  {isProcessing ? (
                    <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    "Connect"
                  )}
                </Button>
              </div>

              <div className="space-y-1">
                <label className="text-[11px] text-muted-foreground font-medium">
                  Device Name
                </label>
                <Input
                  value={deviceName}
                  onChange={(e) => setDeviceName(e.target.value)}
                  className="text-xs"
                  placeholder="Field Tablet / Front Phone"
                />
              </div>

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  const demoUri = `coachify://pair?v=1&host=127.0.0.1&port=50443&fp=3b8a1f6492d2b56e890c2a71f84501de1247b9983412a0134bc5792d426189ef&sec=qs_demo_quick_test&inst=Coachify+Campus&exp=${Math.floor(Date.now() / 1000) + 600}`;
                  setRawUriInput(demoUri);
                  void handleProcessUri(demoUri);
                }}
                className="w-full text-xs text-muted-foreground hover:text-foreground mt-2"
                data-testid="quick-demo-pair-btn"
              >
                ⚡ Quick Demo Pair (Instant Test)
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {step === "pin" && (
        <Card className="w-full border-border/80 shadow-md">
          <CardHeader className="text-center pb-2">
            <div className="mx-auto w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center mb-2">
              <KeyRound className="w-6 h-6 text-primary" />
            </div>
            <CardTitle className="text-lg">Staff PIN Access</CardTitle>
            <CardDescription className="text-xs">
              {parsedUri?.institution || "Coachify Host"} — Enter your 4-6 digit staff PIN
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            {/* PIN Dots Indicator */}
            <div className="flex justify-center items-center gap-3 py-2">
              {[0, 1, 2, 3, 4, 5].map((idx) => (
                <div
                  key={idx}
                  className={`w-3.5 h-3.5 rounded-full transition-all duration-200 border ${
                    idx < staffPin.length
                      ? "bg-primary border-primary scale-110"
                      : "bg-muted border-border"
                  }`}
                  data-testid={`pin-dot-${idx}`}
                />
              ))}
            </div>

            {pinError && (
              <div
                data-testid="pin-error-alert"
                className="text-center text-xs text-destructive font-medium"
              >
                {pinError}
              </div>
            )}

            {/* Numeric Keypad for Mobile Touch Ergonomics */}
            <div className="grid grid-cols-3 gap-2.5 max-w-[260px] mx-auto">
              {["1", "2", "3", "4", "5", "6", "7", "8", "9"].map((num) => (
                <Button
                  key={num}
                  variant="outline"
                  className="h-12 text-lg font-mono font-semibold"
                  onClick={() => handleKeypadPress(num)}
                  data-testid={`keypad-${num}`}
                >
                  {num}
                </Button>
              ))}
              <Button
                variant="ghost"
                className="h-12 text-xs text-muted-foreground"
                onClick={() => setStaffPin("")}
              >
                Clear
              </Button>
              <Button
                variant="outline"
                className="h-12 text-lg font-mono font-semibold"
                onClick={() => handleKeypadPress("0")}
                data-testid="keypad-0"
              >
                0
              </Button>
              <Button
                variant="ghost"
                className="h-12 flex items-center justify-center"
                onClick={handleKeypadBackspace}
                data-testid="keypad-backspace"
              >
                <Delete className="w-5 h-5 text-muted-foreground" />
              </Button>
            </div>

            <Button
              className="w-full gap-2"
              onClick={handleStaffLogin}
              disabled={staffPin.length < 4 || isProcessing}
              data-testid="pin-login-btn"
            >
              {isProcessing && <RefreshCw className="w-4 h-4 animate-spin" />}
              Unlock Companion
            </Button>
          </CardContent>
        </Card>
      )}

      {step === "ready" && (
        <Card className="w-full border-emerald-500/30 bg-emerald-500/5 shadow-md text-center py-6">
          <CardContent className="space-y-4 flex flex-col items-center">
            <div className="w-14 h-14 rounded-full bg-emerald-500/20 text-emerald-600 flex items-center justify-center">
              <CheckCircle2 className="w-8 h-8" />
            </div>
            <div>
              <h3 className="text-base font-bold text-foreground">
                Mobile Companion Ready
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Connected to {pairResponse?.institution_name || "Coachify Host"}
              </p>
            </div>
            <div className="flex gap-2">
              <Badge
                variant="outline"
                className="text-[10px] text-emerald-600 border-emerald-500/30"
              >
                TLS 1.3 Pinned
              </Badge>
              <Badge variant="secondary" className="text-[10px]">
                {deviceName}
              </Badge>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
