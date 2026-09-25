import {
  useCallback,
  useEffect,
  useState,
  type DragEvent,
  type ChangeEvent,
} from "react";
import {
  ShieldCheck,
  AlertTriangle,
  Lock,
  KeyRound,
  Upload,
  Copy,
  Check,
  QrCode,
  MonitorSmartphone,
  RefreshCw,
  FileCheck2,
} from "lucide-react";
import { rpc, type LicenseStatusResponse, type RequestCodeResponse } from "@/gen/rpc";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

const ERROR_EXPLANATIONS: Record<string, string> = {
  license_signature_invalid:
    "Cryptographic signature verification failed. This license file has been modified, corrupted, or was not signed by an authentic Coachify vendor key.",
  license_installation_mismatch:
    "Hardware binding mismatch. This license file was issued for a different computer installation ID and cannot activate this PC.",
  license_serial_downgrade:
    "Serial number downgrade prevented. This license file is older than the license currently active on this computer.",
  license_payload_malformed:
    "Malformed license file. Please select a valid, unmodified .coachifylicense file issued by Coachify Support.",
};

function formatTierLabel(tier: string): string {
  switch (tier) {
    case "local_pro":
      return "Coachify Local Pro (LAN)";
    case "local":
      return "Coachify Local";
    case "trial":
      return "14-Day Evaluation Trial";
    default:
      return tier;
  }
}

function formatStatusLabel(status: string): string {
  switch (status) {
    case "active":
      return "Active";
    case "expiring_soon":
      return "Expiring Soon";
    case "in_grace":
      return "In 14-Day Grace Period";
    case "expired_past_grace":
      return "Read-Only Mode (Expired)";
    case "moved_transitional":
      return "Moved PC (14-Day Transitional Grace)";
    default:
      return status;
  }
}

/**
 * Deterministic visual QR-like matrix SVG derived from the payload string
 * so offline users can photograph the code together with the human-readable request code.
 */
function OfflineQrGraphic({ payload }: { payload: string }) {
  const size = 21;
  const cells: boolean[] = [];
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      const inTopLeft = r < 7 && c < 7;
      const inTopRight = r < 7 && c >= size - 7;
      const inBottomLeft = r >= size - 7 && c < 7;
      if (inTopLeft || inTopRight || inBottomLeft) {
        const lr = inBottomLeft ? r - (size - 7) : r;
        const lc = inTopRight ? c - (size - 7) : c;
        const border = lr === 0 || lr === 6 || lc === 0 || lc === 6;
        const inner = lr >= 2 && lr <= 4 && lc >= 2 && lc <= 4;
        cells.push(border || inner);
      } else {
        const idx = (r * size + c) % Math.max(1, payload.length);
        const code = payload.charCodeAt(idx) + r * 17 + c * 31;
        cells.push(code % 2 === 0);
      }
    }
  }

  return (
    <div
      className="inline-flex flex-col items-center p-3 rounded-lg bg-white text-black border border-border shadow-xs"
      data-testid="offline-qr-code"
    >
      <svg
        width="126"
        height="126"
        viewBox={`0 0 ${size} ${size}`}
        aria-label={`QR Code for ${payload}`}
      >
        {cells.map((filled, idx) => {
          if (!filled) return null;
          const x = idx % size;
          const y = Math.floor(idx / size);
          return <rect key={idx} x={x} y={y} width={1} height={1} fill="#09090b" />;
        })}
      </svg>
      <span className="mt-1.5 text-[10px] font-mono text-zinc-600 max-w-[160px] truncate">
        {payload}
      </span>
    </div>
  );
}

export interface ReadOnlyBannerProps {
  status?: LicenseStatusResponse | null;
  onManageLicense?: () => void;
}

export function ReadOnlyBanner({
  status: externalStatus,
  onManageLicense,
}: ReadOnlyBannerProps) {
  const [fetchedStatus, setFetchedStatus] = useState<LicenseStatusResponse | null>(null);
  const [movedModalOpen, setMovedModalOpen] = useState(false);

  const loadStatus = useCallback(async () => {
    if (externalStatus !== undefined) return;
    const res = await rpc("license.get_status", {});
    if (res.ok && res.data) {
      setFetchedStatus(res.data);
    }
  }, [externalStatus]);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  const status = externalStatus !== undefined ? externalStatus : fetchedStatus;
  if (!status) return null;

  if (status.is_moved && status.status === "moved_transitional") {
    return (
      <>
        <aside
          aria-label="Moved License Transitional Banner"
          data-testid="moved-license-banner"
          className="bg-amber-500/15 border-b border-amber-500/30 text-amber-950 dark:text-amber-200 px-4 py-2.5 flex flex-wrap items-center justify-between gap-3 text-xs font-medium"
        >
          <div className="flex items-center gap-2">
            <MonitorSmartphone className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
            <span>
              <strong>Activate this PC:</strong> Restored database detected on new
              hardware. Full counter access continues for{" "}
              <strong>{`${status.moved_days_remaining ?? 14} days`}</strong> while you
              re-bind your license. Request Code:{" "}
              <code className="font-mono font-bold">{status.request_code}</code>
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              className="h-7 text-xs border-amber-500/40 hover:bg-amber-500/20 cursor-pointer"
              onClick={() => setMovedModalOpen(true)}
            >
              Activate This PC
            </Button>
            {onManageLicense && (
              <Button
                size="sm"
                variant="ghost"
                className="h-7 text-xs cursor-pointer"
                onClick={onManageLicense}
              >
                License Settings
              </Button>
            )}
          </div>
        </aside>
        <MovedLicenseModal
          open={movedModalOpen}
          onOpenChange={setMovedModalOpen}
          status={status}
          onActivated={(updated) => {
            setFetchedStatus(updated);
            setMovedModalOpen(false);
          }}
        />
      </>
    );
  }

  if (status.is_read_only || status.status === "expired_past_grace") {
    return (
      <aside
        aria-label="License Expired Read Only Banner"
        data-testid="read-only-banner"
        className="bg-destructive/15 border-b border-destructive/30 text-destructive px-4 py-2.5 flex flex-wrap items-center justify-between gap-3 text-xs font-medium"
      >
        <div className="flex items-center gap-2">
          <Lock className="h-4 w-4 shrink-0" />
          <span>
            <strong>Read-Only Mode:</strong> Your subscription expired on{" "}
            <strong>{status.paid_through}</strong> (grace ended{" "}
            <strong>{status.valid_until}</strong>). All student searches, reports, receipt
            reprints, data exports, and backups remain open; new writes are paused until
            renewed.
          </span>
        </div>
        {onManageLicense && (
          <Button
            size="sm"
            variant="outline"
            className="h-7 text-xs border-destructive/40 hover:bg-destructive/20 text-destructive cursor-pointer"
            onClick={onManageLicense}
          >
            Import Renewed License
          </Button>
        )}
      </aside>
    );
  }

  if (status.status === "in_grace") {
    return (
      <aside
        aria-label="License Grace Period Banner"
        data-testid="grace-period-banner"
        className="bg-amber-500/15 border-b border-amber-500/30 text-amber-950 dark:text-amber-200 px-4 py-2 flex flex-wrap items-center justify-between gap-3 text-xs font-medium"
      >
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
          <span>
            <strong>14-Day Grace Period Active:</strong> Subscription paid-through date (
            {status.paid_through}) has passed. Full access continues until{" "}
            <strong>{status.valid_until}</strong> before Read-Only Mode begins.
          </span>
        </div>
        {onManageLicense && (
          <Button
            size="sm"
            variant="outline"
            className="h-7 text-xs border-amber-500/40 hover:bg-amber-500/20 cursor-pointer"
            onClick={onManageLicense}
          >
            Renew License
          </Button>
        )}
      </aside>
    );
  }

  if (status.status === "expiring_soon") {
    return (
      <aside
        aria-label="License Expiring Soon Banner"
        data-testid="expiring-soon-banner"
        className="bg-primary/10 border-b border-primary/20 text-foreground px-4 py-2 flex flex-wrap items-center justify-between gap-3 text-xs"
      >
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-3.5 w-3.5 text-primary shrink-0" />
          <span>
            Subscription renews in <strong>{`${status.days_remaining} days`}</strong> (
            {status.paid_through}). Prepare your offline request code (
            <code className="font-mono">{status.request_code}</code>) for seamless
            renewal.
          </span>
        </div>
        {onManageLicense && (
          <Button
            size="sm"
            variant="ghost"
            className="h-7 text-xs cursor-pointer"
            onClick={onManageLicense}
          >
            View License
          </Button>
        )}
      </aside>
    );
  }

  return null;
}

export interface MovedLicenseModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  status: LicenseStatusResponse | null;
  onActivated?: (updated: LicenseStatusResponse) => void;
}

export function MovedLicenseModal({
  open,
  onOpenChange,
  status,
  onActivated,
}: MovedLicenseModalProps) {
  const [reqInfo, setReqInfo] = useState<RequestCodeResponse | null>(null);
  const [licenseInput, setLicenseInput] = useState("");
  const [importError, setImportError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);

  useEffect(() => {
    if (!open) return;
    void (async () => {
      const res = await rpc("license.get_request_code", {});
      if (res.ok && res.data) {
        setReqInfo(res.data);
      }
    })();
  }, [open]);

  const handleImport = async () => {
    if (!licenseInput.trim()) return;
    setImporting(true);
    setImportError(null);
    const res = await rpc("license.import_file", {
      license_content: licenseInput.trim(),
    });
    setImporting(false);
    if (res.ok && res.data) {
      onActivated?.(res.data);
      onOpenChange(false);
    } else if (res.error) {
      setImportError(
        ERROR_EXPLANATIONS[res.error.code] ?? `${res.error.message} (${res.error.code})`
      );
    }
  };

  const modalBody = (
    <>
      <div className="space-y-1.5">
        <h2 className="flex items-center gap-2 text-lg font-semibold leading-none tracking-tight">
          <MonitorSmartphone className="h-5 w-5 text-amber-500" />
          Activate This Replacement PC
        </h2>
        <p className="text-sm text-muted-foreground">
          Your database backup was restored onto a computer with a new installation ID.
          Coachify has enabled a{" "}
          <strong>{`${status?.moved_days_remaining ?? 14}-day full-access transitional window`}</strong>{" "}
          so admissions, exams, and fee collections continue without interruption.
        </p>
      </div>

      <div className="space-y-4 text-xs">
        <div className="p-3 rounded-lg bg-muted/60 border border-border flex flex-col sm:flex-row items-center justify-between gap-4">
          <div className="space-y-1.5">
            <div className="text-muted-foreground font-medium">New PC Request Code</div>
            <div
              className="font-mono text-sm font-bold tracking-wider text-foreground"
              data-testid="moved-modal-request-code"
            >
              {reqInfo?.request_code ?? status?.request_code}
            </div>
            <p className="text-[11px] text-muted-foreground">
              Send this code or photograph the QR code via WhatsApp to receive your
              re-bound .coachifylicense file.
            </p>
          </div>
          {reqInfo && <OfflineQrGraphic payload={reqInfo.qr_payload} />}
        </div>

        <div className="space-y-2">
          <label className="font-medium text-foreground block">
            Paste or Import Re-bound License (.coachifylicense)
          </label>
          <textarea
            rows={4}
            value={licenseInput}
            onChange={(e) => setLicenseInput(e.target.value)}
            placeholder="Paste signed .coachifylicense JSON content here..."
            className="w-full rounded-md border border-input bg-background px-3 py-2 text-xs font-mono"
          />
          {importError && (
            <div
              role="alert"
              className="p-2.5 rounded-md bg-destructive/15 border border-destructive/30 text-destructive text-xs"
            >
              {importError}
            </div>
          )}
          <div className="flex justify-end gap-2">
            <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
              Continue in Transitional Mode
            </Button>
            <Button
              size="sm"
              disabled={importing || !licenseInput.trim()}
              onClick={() => void handleImport()}
            >
              {importing ? "Verifying..." : "Verify & Activate PC"}
            </Button>
          </div>
        </div>
      </div>
    </>
  );

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Activate This Replacement PC"
      data-testid="moved-license-modal"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
    >
      <div className="w-full max-w-lg rounded-xl border border-border bg-background p-6 shadow-lg space-y-4">
        {modalBody}
      </div>
    </div>
  );
}

export interface LicenseSettingsCardProps {
  initialStatus?: LicenseStatusResponse | null;
  initialRequestInfo?: RequestCodeResponse | null;
  defaultShowQr?: boolean;
}

export function LicenseSettingsCard({
  initialStatus = null,
  initialRequestInfo = null,
  defaultShowQr = false,
}: LicenseSettingsCardProps = {}) {
  const [status, setStatus] = useState<LicenseStatusResponse | null>(initialStatus);
  const [requestInfo, setRequestInfo] = useState<RequestCodeResponse | null>(
    initialRequestInfo
  );
  const [showQr, setShowQr] = useState(defaultShowQr);
  const [copied, setCopied] = useState(false);
  const [licenseText, setLicenseText] = useState("");
  const [importing, setImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [importSuccess, setImportSuccess] = useState<string | null>(null);
  const [dragActive, setDragActive] = useState(false);

  const fetchLicenseData = useCallback(async () => {
    const [statusRes, reqRes] = await Promise.all([
      rpc("license.get_status", {}),
      rpc("license.get_request_code", {}),
    ]);
    if (statusRes.ok && statusRes.data) {
      setStatus(statusRes.data);
    }
    if (reqRes.ok && reqRes.data) {
      setRequestInfo(reqRes.data);
    }
  }, []);

  useEffect(() => {
    void fetchLicenseData();
  }, [fetchLicenseData]);

  const handleCopyRequestCode = async () => {
    const code = requestInfo?.request_code ?? status?.request_code;
    if (!code) return;
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard fallback ignored in headless test environments
    }
  };

  const submitLicenseContent = async (rawContent: string) => {
    const trimmed = rawContent.trim();
    if (!trimmed) {
      setImportError(
        "Please provide a .coachifylicense file or paste its signed JSON content."
      );
      return;
    }
    setImporting(true);
    setImportError(null);
    setImportSuccess(null);

    const res = await rpc("license.import_file", {
      license_content: trimmed,
    });
    setImporting(false);

    if (res.ok && res.data) {
      setStatus(res.data);
      setLicenseText("");
      setImportSuccess(
        `Successfully verified and activated ${formatTierLabel(res.data.tier)} (valid through ${res.data.paid_through}).`
      );
    } else if (res.error) {
      const friendly =
        ERROR_EXPLANATIONS[res.error.code] ?? `${res.error.message} (${res.error.code})`;
      setImportError(friendly);
    }
  };

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const content = typeof reader.result === "string" ? reader.result : "";
      setLicenseText(content);
      void submitLicenseContent(content);
    };
    reader.readAsText(file);
  };

  const handleDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragActive(false);
    const file = e.dataTransfer.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const content = typeof reader.result === "string" ? reader.result : "";
      setLicenseText(content);
      void submitLicenseContent(content);
    };
    reader.readAsText(file);
  };

  return (
    <div className="space-y-6" data-testid="license-settings-card">
      <ReadOnlyBanner status={status} />

      <div className="grid gap-6 md:grid-cols-2">
        {/* Current Subscription & Entitlements Card */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-3">
            <CardTitle className="flex items-center gap-2 text-base">
              <ShieldCheck className="h-4 w-4 text-primary" />
              Desktop License & Entitlements
            </CardTitle>
            {status && (
              <Badge
                variant={
                  status.is_read_only
                    ? "overdue"
                    : status.status === "active"
                      ? "default"
                      : "secondary"
                }
                data-testid="license-status-badge"
              >
                {formatStatusLabel(status.status)}
              </Badge>
            )}
          </CardHeader>
          <CardContent className="space-y-4 text-xs">
            {status ? (
              <>
                <div className="grid grid-cols-2 gap-3 p-3 rounded-lg bg-muted/50 border border-border">
                  <div>
                    <span className="text-muted-foreground block">Institution</span>
                    <strong
                      className="text-foreground text-sm"
                      data-testid="license-customer-name"
                    >
                      {status.customer_name}
                    </strong>
                  </div>
                  <div>
                    <span className="text-muted-foreground block">Subscription Plan</span>
                    <strong
                      className="text-foreground text-sm"
                      data-testid="license-tier-label"
                    >
                      {formatTierLabel(status.tier)}
                    </strong>
                  </div>
                  <div>
                    <span className="text-muted-foreground block">Paid Through</span>
                    <span className="font-mono font-medium text-foreground">
                      {status.paid_through} ({status.days_remaining} days)
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block">
                      Valid Until (incl. {status.grace_days}d grace)
                    </span>
                    <span className="font-mono font-medium text-foreground">
                      {status.valid_until}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block">
                      Active Enrollments
                    </span>
                    <span className="font-medium text-foreground">
                      {status.active_students}
                      {status.student_limit
                        ? ` / ${status.student_limit}`
                        : " (Unlimited)"}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block">Workstations</span>
                    <span className="font-medium text-foreground">
                      {status.workstation_limit ?? 1} PC(s)
                    </span>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-foreground flex items-center gap-1.5">
                      <KeyRound className="h-3.5 w-3.5 text-primary" />
                      Offline Activation Request Code
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 text-xs gap-1 cursor-pointer"
                      onClick={() => setShowQr((prev) => !prev)}
                      data-testid="toggle-request-qr-btn"
                    >
                      <QrCode className="h-3.5 w-3.5" />
                      {showQr ? "Hide QR Code" : "Show Request Code & QR"}
                    </Button>
                  </div>

                  <div className="flex items-center justify-between p-2.5 rounded-md bg-background border border-border">
                    <code
                      className="font-mono text-xs font-bold tracking-wider text-foreground"
                      data-testid="request-code-display"
                    >
                      {requestInfo?.request_code ?? status.request_code}
                    </code>
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs gap-1 cursor-pointer"
                      onClick={() => void handleCopyRequestCode()}
                    >
                      {copied ? (
                        <>
                          <Check className="h-3 w-3 text-emerald-500" />
                          Copied
                        </>
                      ) : (
                        <>
                          <Copy className="h-3 w-3" />
                          Copy
                        </>
                      )}
                    </Button>
                  </div>

                  {showQr && requestInfo && (
                    <div className="pt-2 flex flex-col items-center gap-2 p-3 rounded-lg bg-muted/40 border border-border">
                      <OfflineQrGraphic payload={requestInfo.qr_payload} />
                      <span className="text-[11px] text-muted-foreground text-center">
                        Installation ID:{" "}
                        <code className="font-mono">{status.installation_id}</code>
                      </span>
                    </div>
                  )}
                </div>
              </>
            ) : (
              <div className="py-6 flex items-center justify-center gap-2 text-muted-foreground">
                <RefreshCw className="h-4 w-4 animate-spin" />
                Loading license status...
              </div>
            )}
          </CardContent>
        </Card>

        {/* Offline License Import Card */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Upload className="h-4 w-4 text-primary" />
              Import Offline License File (.coachifylicense)
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 text-xs">
            <p className="text-muted-foreground leading-relaxed">
              Coachify verifies Ed25519 digital signatures 100% offline. Drag and drop
              your signed <code className="font-mono">.coachifylicense</code> file below
              or paste its contents to activate or renew this computer immediately.
            </p>

            <div
              onDragOver={(e) => {
                e.preventDefault();
                setDragActive(true);
              }}
              onDragLeave={() => setDragActive(false)}
              onDrop={handleDrop}
              data-testid="license-dropzone"
              className={`border-2 border-dashed rounded-lg p-5 text-center transition-colors ${
                dragActive
                  ? "border-primary bg-primary/5"
                  : "border-border hover:border-primary/50 bg-muted/30"
              }`}
            >
              <FileCheck2 className="h-6 w-6 mx-auto mb-2 text-primary" />
              <div className="font-medium text-foreground mb-1">
                Drop .coachifylicense file here or browse
              </div>
              <p className="text-[11px] text-muted-foreground mb-3">
                Verified offline against embedded vendor public keys (k2026a / k2026b)
              </p>
              <label className="inline-flex items-center justify-center rounded-md text-xs font-medium border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3 cursor-pointer">
                Choose License File
                <input
                  type="file"
                  accept=".coachifylicense,.json,application/json"
                  onChange={handleFileChange}
                  className="hidden"
                  data-testid="license-file-input"
                />
              </label>
            </div>

            <div className="space-y-2">
              <label
                htmlFor="license-json-textarea"
                className="font-medium text-foreground block"
              >
                Or paste signed license JSON
              </label>
              <textarea
                id="license-json-textarea"
                data-testid="license-json-textarea"
                rows={4}
                value={licenseText}
                onChange={(e) => setLicenseText(e.target.value)}
                placeholder='{"license_id":"LIC-...","signature":"..."}'
                className="w-full rounded-md border border-input bg-background px-3 py-2 text-xs font-mono"
              />
            </div>

            {importError && (
              <div
                role="alert"
                data-testid="license-import-error"
                className="p-3 rounded-lg bg-destructive/15 border border-destructive/30 text-destructive text-xs font-medium"
              >
                {importError}
              </div>
            )}

            {importSuccess && (
              <div
                role="status"
                data-testid="license-import-success"
                className="p-3 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-emerald-800 dark:text-emerald-200 text-xs font-medium"
              >
                {importSuccess}
              </div>
            )}

            <Button
              className="w-full font-semibold cursor-pointer"
              disabled={importing || !licenseText.trim()}
              onClick={() => void submitLicenseContent(licenseText)}
              data-testid="import-license-submit-btn"
            >
              {importing
                ? "Verifying Cryptographic Signature..."
                : "Verify & Activate License"}
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
