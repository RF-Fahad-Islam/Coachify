import { useCallback, useEffect, useState } from "react";
import {
  Server,
  Power,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Copy,
  Check,
  RefreshCw,
  Search,
  Activity,
  Terminal,
  Moon,
  Laptop,
  Radio,
  Wifi,
  AlertCircle,
} from "lucide-react";
import {
  rpc,
  type LanHostStatus,
  type FirewallGuidance,
  type DiscoveredHost,
  type HostPingResult,
} from "@/gen/rpc";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";

export interface LanHostSettingsCardProps {
  initialStatus?: LanHostStatus | null;
  initialGuidance?: FirewallGuidance | null;
}

export function LanHostSettingsCard({
  initialStatus = null,
  initialGuidance = null,
}: LanHostSettingsCardProps = {}) {
  const [status, setStatus] = useState<LanHostStatus | null>(initialStatus);
  const [guidance, setGuidance] = useState<FirewallGuidance | null>(initialGuidance);
  const [loading, setLoading] = useState(initialStatus === null);
  const [toggling, setToggling] = useState(false);
  const [savingConfig, setSavingConfig] = useState(false);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Editable configuration fields
  const [portInput, setPortInput] = useState<number>(initialStatus?.port ?? 50443);
  const [bindAddressInput, setBindAddressInput] = useState<string>(
    initialStatus?.bind_address ?? "0.0.0.0"
  );
  const [autoStart, setAutoStart] = useState<boolean>(
    initialStatus?.auto_start_on_boot ?? false
  );
  const [preventSleep, setPreventSleep] = useState<boolean>(
    initialStatus?.prevent_sleep ?? true
  );

  // Network discovery scanner
  const [scanning, setScanning] = useState(false);
  const [discoveredHosts, setDiscoveredHosts] = useState<DiscoveredHost[]>([]);

  // Manual IP Ping test
  const [pingIp, setPingIp] = useState<string>("");
  const [pingPort, setPingPort] = useState<number>(50443);
  const [pingExpectedFp, setPingExpectedFp] = useState<string>("");
  const [pinging, setPinging] = useState(false);
  const [pingResult, setPingResult] = useState<HostPingResult | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await rpc("lan.get_host_status", {});
      if (res.ok && res.data) {
        setStatus(res.data);
        setPortInput(res.data.port);
        setBindAddressInput(res.data.bind_address);
        setAutoStart(res.data.auto_start_on_boot);
        setPreventSleep(res.data.prevent_sleep);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const fetchGuidance = useCallback(async () => {
    try {
      const res = await rpc("lan.get_firewall_guidance", {});
      if (res.ok && res.data) {
        setGuidance(res.data);
      }
    } catch {
      // Ignored non-critical failure
    }
  }, []);

  const refreshAll = useCallback(async () => {
    setLoading(true);
    await Promise.all([fetchStatus(), fetchGuidance()]);
    setLoading(false);
  }, [fetchStatus, fetchGuidance]);

  useEffect(() => {
    if (initialStatus === null) {
      void refreshAll();
    } else {
      setLoading(false);
    }

    // Auto-refresh interval while host is active
    const timer = setInterval(() => {
      void fetchStatus();
    }, 10000);

    return () => clearInterval(timer);
  }, [refreshAll, fetchStatus, initialStatus]);

  const handleCopy = (key: string, text: string) => {
    void navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  const handleToggleHost = async () => {
    if (!status) return;
    setToggling(true);
    setErrorMsg(null);
    setSuccessMsg(null);

    try {
      if (status.is_running) {
        const res = await rpc("lan.stop_host", {});
        if (res.ok && res.data) {
          setStatus(res.data);
          setSuccessMsg("LAN Host service stopped successfully.");
        } else if (!res.ok && res.error) {
          setErrorMsg(res.error.message);
        }
      } else {
        const res = await rpc("lan.start_host", {
          port: portInput,
          bind_address: bindAddressInput,
        });
        if (res.ok && res.data) {
          setStatus(res.data);
          setSuccessMsg(`LAN Host service is now running on port ${res.data.port}.`);
        } else if (!res.ok && res.error) {
          setErrorMsg(res.error.message);
        }
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setToggling(false);
      void fetchGuidance();
    }
  };

  const handleSaveConfig = async () => {
    setSavingConfig(true);
    setErrorMsg(null);
    setSuccessMsg(null);
    try {
      const res = await rpc("lan.update_host_config", {
        port: portInput,
        bind_address: bindAddressInput,
        auto_start_on_boot: autoStart,
        prevent_sleep: preventSleep,
      });
      if (res.ok && res.data) {
        setStatus(res.data);
        setSuccessMsg("LAN host settings saved successfully.");
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingConfig(false);
    }
  };

  const handleScanHosts = async () => {
    setScanning(true);
    setErrorMsg(null);
    try {
      const res = await rpc("lan.discover_hosts", { timeout_ms: 2500 });
      if (res.ok && res.data) {
        setDiscoveredHosts(res.data.hosts);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setScanning(false);
    }
  };

  const handlePingHost = async () => {
    if (!pingIp.trim()) {
      setErrorMsg("Please enter an IP address to ping.");
      return;
    }
    setPinging(true);
    setPingResult(null);
    setErrorMsg(null);
    try {
      const res = await rpc("lan.ping_host", {
        ip: pingIp.trim(),
        port: pingPort,
        expected_fingerprint: pingExpectedFp.trim() ? pingExpectedFp.trim() : null,
      });
      if (res.ok && res.data) {
        setPingResult(res.data);
      } else if (!res.ok && res.error) {
        setErrorMsg(res.error.message);
      }
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setPinging(false);
    }
  };

  const formatUptime = (secs: number) => {
    if (secs < 60) return `${secs}s`;
    const mins = Math.floor(secs / 60);
    const remSecs = secs % 60;
    if (mins < 60) return `${mins}m ${remSecs}s`;
    const hrs = Math.floor(mins / 60);
    const remMins = mins % 60;
    return `${hrs}h ${remMins}m`;
  };

  return (
    <div className="space-y-6">
      {/* Messages */}
      {errorMsg && (
        <div
          data-testid="lan-error-banner"
          className="flex items-center gap-2 p-3 text-xs rounded-lg bg-destructive/10 border border-destructive/20 text-destructive font-medium"
        >
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{errorMsg}</span>
        </div>
      )}

      {successMsg && (
        <div
          data-testid="lan-success-banner"
          className="flex items-center gap-2 p-3 text-xs rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-600 dark:text-emerald-400 font-medium"
        >
          <Check className="h-4 w-4 shrink-0" />
          <span>{successMsg}</span>
        </div>
      )}

      {/* License Entitlement Banner (if not entitled) */}
      {status && !status.license_entitled && (
        <div
          data-testid="lan-license-warning"
          className="p-4 rounded-xl bg-amber-500/10 border border-amber-500/25 flex flex-col sm:flex-row sm:items-center justify-between gap-4"
        >
          <div className="flex items-start gap-3">
            <ShieldAlert className="h-5 w-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
            <div className="space-y-1">
              <h4 className="text-sm font-semibold text-amber-900 dark:text-amber-200">
                Multi-PC LAN Hosting Requires License Upgrade
              </h4>
              <p className="text-xs text-amber-700 dark:text-amber-300/90 leading-relaxed">
                Your current license does not include the Multi-PC LAN Host capability. To
                allow secondary front-desk workstations to connect and sync offline over
                your local Wi-Fi / Ethernet network, upgrade to{" "}
                <strong>Coachify Local Pro</strong>.
              </p>
            </div>
          </div>
          <Button
            variant="outline"
            size="sm"
            className="shrink-0 font-semibold border-amber-500/30 text-amber-900 dark:text-amber-100 hover:bg-amber-500/15"
            onClick={() => {
              const licenseTab = document.querySelector(
                '[data-value="license"], [value="license"]'
              ) as HTMLElement | null;
              if (licenseTab) licenseTab.click();
            }}
          >
            Upgrade License
          </Button>
        </div>
      )}

      {/* Main Host Status & Controls Card */}
      <Card className="border-border">
        <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Server className="h-5 w-5 text-primary" />
              <CardTitle className="text-lg font-bold">
                Authoritative LAN Host Service
              </CardTitle>
              {status?.is_running ? (
                <Badge
                  data-testid="host-status-badge"
                  className="bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border-emerald-500/30 font-semibold text-xs flex items-center gap-1.5"
                >
                  <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
                  Running
                </Badge>
              ) : (
                <Badge
                  data-testid="host-status-badge"
                  variant="outline"
                  className="text-muted-foreground font-semibold text-xs"
                >
                  Stopped
                </Badge>
              )}
            </div>
            <CardDescription className="text-xs">
              Broadcasts encrypted local authority for secondary counter and reception
              workstations without internet dependency.
            </CardDescription>
          </div>

          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => void refreshAll()}
              disabled={loading}
              className="h-8 gap-1.5 text-xs"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
              Refresh
            </Button>

            <Button
              data-testid="lan-toggle-button"
              variant={status?.is_running ? "destructive" : "primary"}
              size="sm"
              onClick={() => void handleToggleHost()}
              disabled={toggling || !status?.license_entitled}
              className="h-8 gap-1.5 text-xs font-semibold"
            >
              <Power className="h-3.5 w-3.5" />
              {toggling
                ? status?.is_running
                  ? "Stopping..."
                  : "Starting..."
                : status?.is_running
                  ? "Stop Host"
                  : "Start Host"}
            </Button>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Status Metric Grid */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div className="p-3 rounded-lg bg-muted/50 border border-border space-y-1">
              <span className="text-[11px] font-medium text-muted-foreground">
                Host Machine
              </span>
              <p
                className="text-sm font-semibold text-foreground truncate"
                title={status?.hostname}
              >
                {status?.hostname || "—"}
              </p>
            </div>

            <div className="p-3 rounded-lg bg-muted/50 border border-border space-y-1">
              <span className="text-[11px] font-medium text-muted-foreground">
                Active Port
              </span>
              <p className="text-sm font-semibold text-foreground">
                {status?.port ? `TCP :${status.port}` : "—"}
              </p>
            </div>

            <div className="p-3 rounded-lg bg-muted/50 border border-border space-y-1">
              <span className="text-[11px] font-medium text-muted-foreground">
                Service Uptime
              </span>
              <p className="text-sm font-semibold text-foreground">
                {status?.is_running ? formatUptime(status.uptime_seconds) : "0s"}
              </p>
            </div>

            <div className="p-3 rounded-lg bg-muted/50 border border-border space-y-1">
              <span className="text-[11px] font-medium text-muted-foreground">
                mDNS Discovery
              </span>
              <p className="text-sm font-semibold text-foreground flex items-center gap-1.5">
                {status?.mdns_broadcasting ? (
                  <>
                    <Radio className="h-3.5 w-3.5 text-emerald-500 animate-pulse" />
                    Broadcasting
                  </>
                ) : (
                  <span className="text-muted-foreground">Offline</span>
                )}
              </p>
            </div>
          </div>

          {/* Bound Local IP Addresses */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <Wifi className="h-3.5 w-3.5 text-primary" />
                Local IP Endpoints
              </span>
              <span className="text-[11px] text-muted-foreground">
                Share any of these addresses with workstations
              </span>
            </div>
            <div className="flex flex-wrap gap-2">
              {status?.bound_addresses && status.bound_addresses.length > 0 ? (
                status.bound_addresses.map((ip) => (
                  <button
                    key={ip}
                    type="button"
                    onClick={() => handleCopy(`ip-${ip}`, ip)}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-mono font-medium bg-muted hover:bg-muted/80 border border-border transition-colors cursor-pointer"
                    title="Click to copy IP"
                  >
                    <span>{ip}</span>
                    {copiedKey === `ip-${ip}` ? (
                      <Check className="h-3 w-3 text-emerald-500" />
                    ) : (
                      <Copy className="h-3 w-3 text-muted-foreground" />
                    )}
                  </button>
                ))
              ) : (
                <span className="text-xs text-muted-foreground">
                  No active network adapters found
                </span>
              )}
            </div>
          </div>

          {/* TLS Certificate Fingerprint */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <ShieldCheck className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
                TLS 1.3 Certificate Fingerprint (SHA-256)
              </span>
              {copiedKey === "fingerprint" && (
                <span className="text-[11px] font-medium text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                  <Check className="h-3 w-3" /> Copied to clipboard
                </span>
              )}
            </div>
            <div className="relative">
              <div
                data-testid="tls-fingerprint"
                className="p-2.5 rounded-lg bg-muted/60 border border-border font-mono text-[11px] text-foreground break-all select-all pr-10"
              >
                {status?.fingerprint || "Generating certificate..."}
              </div>
              <Button
                variant="ghost"
                size="sm"
                className="absolute right-1 top-1 h-7 w-7 p-0"
                onClick={() =>
                  status?.fingerprint && handleCopy("fingerprint", status.fingerprint)
                }
                title="Copy Fingerprint"
              >
                {copiedKey === "fingerprint" ? (
                  <Check className="h-3.5 w-3.5 text-emerald-500" />
                ) : (
                  <Copy className="h-3.5 w-3.5 text-muted-foreground" />
                )}
              </Button>
            </div>
            <p className="text-[11px] text-muted-foreground">
              Workstations compare this cryptographic fingerprint on initial pairing to
              block man-in-the-middle attacks over untrusted Wi-Fi routers.
            </p>
          </div>

          {/* Host Service Settings (Port, Bind, Sleep, Boot) */}
          <div className="pt-4 border-t border-border space-y-4">
            <h4 className="text-xs font-semibold text-foreground uppercase tracking-wider">
              Host Listener Settings
            </h4>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  Listener Port
                </label>
                <Input
                  type="number"
                  min={1024}
                  max={65535}
                  value={portInput}
                  onChange={(e) => setPortInput(Number(e.target.value))}
                  disabled={status?.is_running}
                  className="font-mono text-xs"
                />
                <span className="text-[11px] text-muted-foreground">
                  Default: 50443. (Cannot be modified while the host is active)
                </span>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-medium text-foreground">
                  Bind Address
                </label>
                <Input
                  type="text"
                  value={bindAddressInput}
                  onChange={(e) => setBindAddressInput(e.target.value)}
                  disabled={status?.is_running}
                  className="font-mono text-xs"
                />
                <span className="text-[11px] text-muted-foreground">
                  Use 0.0.0.0 to bind all network adapters, or 127.0.0.1 for local machine
                  testing.
                </span>
              </div>
            </div>

            <div className="space-y-3 pt-2">
              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/40 border border-border">
                <div className="space-y-0.5">
                  <div className="text-xs font-medium text-foreground flex items-center gap-1.5">
                    <Moon className="h-3.5 w-3.5 text-primary" />
                    Inhibit Windows Sleep
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Prevents the host computer from going to sleep while workstations are
                    active.
                  </p>
                </div>
                <Switch checked={preventSleep} onCheckedChange={setPreventSleep} />
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg bg-muted/40 border border-border">
                <div className="space-y-0.5">
                  <div className="text-xs font-medium text-foreground flex items-center gap-1.5">
                    <Activity className="h-3.5 w-3.5 text-primary" />
                    Auto-Start on Windows Startup
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Automatically starts the LAN authoritative host listener when Coachify
                    opens.
                  </p>
                </div>
                <Switch checked={autoStart} onCheckedChange={setAutoStart} />
              </div>
            </div>

            <div className="flex justify-end pt-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => void handleSaveConfig()}
                disabled={savingConfig}
                className="text-xs font-semibold"
              >
                {savingConfig ? "Saving..." : "Save Configuration"}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Windows Firewall & Network Category Guidance (AC-8) */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Shield className="h-4 w-4 text-primary" />
              <CardTitle className="text-base font-bold">
                Windows Firewall & Network Profile
              </CardTitle>
            </div>
            <div className="flex items-center gap-2">
              {guidance?.is_private ? (
                <Badge
                  data-testid="network-profile-badge"
                  className="bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border-emerald-500/30 text-xs font-medium"
                >
                  Profile: Private (Recommended)
                </Badge>
              ) : (
                <Badge
                  data-testid="network-profile-badge"
                  className="bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-500/30 text-xs font-medium"
                >
                  Profile: Public (Blocked by Windows)
                </Badge>
              )}

              {guidance?.rule_exists ? (
                <Badge
                  data-testid="firewall-rule-badge"
                  className="bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border-emerald-500/30 text-xs font-medium"
                >
                  Firewall: Allowed
                </Badge>
              ) : (
                <Badge
                  data-testid="firewall-rule-badge"
                  variant="outline"
                  className="text-amber-600 dark:text-amber-400 border-amber-500/30 text-xs font-medium"
                >
                  Firewall: Rule Missing
                </Badge>
              )}
            </div>
          </div>
          <CardDescription className="text-xs">
            Windows Firewall protects your machine by blocking inbound connections on
            Public networks. If workstations cannot reach this host, ensure your Wi-Fi
            network is set to Private and the inbound firewall rule is configured.
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Switch to Private Network Guidance */}
          {!guidance?.is_private && (
            <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/25 space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold text-amber-900 dark:text-amber-200 flex items-center gap-1.5">
                  <Terminal className="h-3.5 w-3.5" />
                  PowerShell Command: Switch Network Category to Private
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-6 text-[11px] gap-1 px-2 font-medium"
                  onClick={() =>
                    guidance?.powershell_command &&
                    handleCopy("powershell", guidance.powershell_command)
                  }
                >
                  {copiedKey === "powershell" ? (
                    <Check className="h-3 w-3 text-emerald-500" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                  Copy Command
                </Button>
              </div>
              <pre className="p-2 rounded bg-black/60 text-emerald-400 font-mono text-[11px] overflow-x-auto whitespace-pre-wrap select-all">
                {guidance?.powershell_command}
              </pre>
            </div>
          )}

          {/* Add Firewall Inbound Rule Guidance */}
          <div className="p-3 rounded-lg bg-muted/50 border border-border space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                <Terminal className="h-3.5 w-3.5 text-primary" />
                Windows Command: Allow Port 50443 Through Firewall
              </span>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-[11px] gap-1 px-2 font-medium"
                onClick={() =>
                  guidance?.netsh_command && handleCopy("netsh", guidance.netsh_command)
                }
              >
                {copiedKey === "netsh" ? (
                  <Check className="h-3 w-3 text-emerald-500" />
                ) : (
                  <Copy className="h-3 w-3" />
                )}
                Copy Command
              </Button>
            </div>
            <pre className="p-2 rounded bg-black/60 text-emerald-400 font-mono text-[11px] overflow-x-auto whitespace-pre-wrap select-all">
              {guidance?.netsh_command}
            </pre>
            <p className="text-[11px] text-muted-foreground">
              Run in Windows Terminal / Command Prompt as Administrator if client
              workstations cannot ping this PC.
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Discovered LAN Hosts & Ping Testing (AC-6, AC-7) */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Network Scanner */}
        <Card className="border-border">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Radio className="h-4 w-4 text-primary" />
                <CardTitle className="text-base font-bold">
                  LAN Discovery Scanner
                </CardTitle>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void handleScanHosts()}
                disabled={scanning}
                className="h-8 gap-1.5 text-xs font-semibold"
              >
                <Search className={`h-3.5 w-3.5 ${scanning ? "animate-spin" : ""}`} />
                {scanning ? "Scanning..." : "Scan Network"}
              </Button>
            </div>
            <CardDescription className="text-xs">
              Detects Coachify host peers broadcasting on the local subnet via DNS-SD
              (_coachify._tcp).
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {discoveredHosts.length === 0 ? (
              <div className="text-center py-8 text-xs text-muted-foreground border border-dashed border-border rounded-lg">
                <Laptop className="h-8 w-8 mx-auto mb-2 opacity-30" />
                {scanning
                  ? "Listening for mDNS beacons on LAN..."
                  : "No external Coachify hosts detected on this subnet."}
              </div>
            ) : (
              <div className="space-y-2">
                {discoveredHosts.map((host) => (
                  <div
                    key={host.host_id}
                    className="p-3 rounded-lg bg-muted/60 border border-border flex items-center justify-between text-xs"
                  >
                    <div className="space-y-0.5">
                      <div className="font-semibold text-foreground flex items-center gap-1.5">
                        <Server className="h-3.5 w-3.5 text-primary" />
                        {host.name}
                      </div>
                      <div className="font-mono text-[11px] text-muted-foreground">
                        {host.ip}:{host.port} · v{host.app_version}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        setPingIp(host.ip);
                        setPingPort(host.port);
                        setPingExpectedFp(host.fingerprint);
                      }}
                      className="h-7 text-[11px]"
                    >
                      Fill Ping
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Manual IP Ping Test Tool */}
        <Card className="border-border">
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <Activity className="h-4 w-4 text-primary" />
              <CardTitle className="text-base font-bold">
                Manual IP Connectivity Ping
              </CardTitle>
            </div>
            <CardDescription className="text-xs">
              Test end-to-end TLS handshake and /hello response when mDNS broadcasts are
              filtered by managed network switches.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2 space-y-1">
                <label className="text-[11px] font-medium text-foreground">
                  Target IP Address
                </label>
                <Input
                  placeholder="192.168.1.100"
                  value={pingIp}
                  onChange={(e) => setPingIp(e.target.value)}
                  className="font-mono text-xs h-8"
                />
              </div>
              <div className="space-y-1">
                <label className="text-[11px] font-medium text-foreground">Port</label>
                <Input
                  type="number"
                  value={pingPort}
                  onChange={(e) => setPingPort(Number(e.target.value))}
                  className="font-mono text-xs h-8"
                />
              </div>
            </div>

            <div className="space-y-1">
              <label className="text-[11px] font-medium text-foreground">
                Expected Fingerprint (Optional)
              </label>
              <Input
                placeholder="SHA256:..."
                value={pingExpectedFp}
                onChange={(e) => setPingExpectedFp(e.target.value)}
                className="font-mono text-xs h-8"
              />
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => void handlePingHost()}
              disabled={pinging}
              className="w-full h-8 text-xs font-semibold gap-1.5"
            >
              <Activity className={`h-3.5 w-3.5 ${pinging ? "animate-spin" : ""}`} />
              {pinging ? "Connecting..." : "Test Connection (Ping)"}
            </Button>

            {pingResult && (
              <div
                data-testid="ping-result"
                className={`p-3 rounded-lg border text-xs space-y-1 ${
                  pingResult.reachable
                    ? "bg-emerald-500/10 border-emerald-500/25 text-emerald-800 dark:text-emerald-300"
                    : "bg-destructive/10 border-destructive/25 text-destructive"
                }`}
              >
                <div className="font-semibold flex items-center justify-between">
                  <span>
                    {pingResult.reachable ? "Host Reachable" : "Host Unreachable"}
                  </span>
                  <span className="font-mono text-[11px]">
                    {pingResult.rtt_ms} ms RTT
                  </span>
                </div>
                {pingResult.reachable && (
                  <div className="text-[11px] space-y-0.5 text-muted-foreground dark:text-emerald-300/80">
                    <p>
                      Product: {pingResult.product} (Protocol v
                      {pingResult.protocol_version})
                    </p>
                    <p>Institution: {pingResult.institution_name || "Unknown"}</p>
                    <p>
                      Certificate Verification:{" "}
                      {pingResult.fingerprint_matched ? "Verified (Match)" : "Mismatch"}
                    </p>
                  </div>
                )}
                {pingResult.error && (
                  <p className="text-[11px] text-destructive">{pingResult.error}</p>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
