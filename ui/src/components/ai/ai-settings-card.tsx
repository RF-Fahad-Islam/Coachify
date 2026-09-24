import { useState, useEffect, useCallback } from "react";
import {
  ShieldCheck,
  ShieldAlert,
  Coins,
  Cpu,
  History,
  Sparkles,
  RefreshCw,
  Plus,
  Key,
  Check,
  AlertCircle,
  Eye,
  EyeOff,
  Lock,
  CheckCircle2,
  Activity,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { rpc } from "@/gen/rpc";
import type {
  PrivacySettings,
  AiWalletBalance,
  CreditTransactionItem,
  AiGatewayConfig,
  GatewayTestResult,
  AiAuditLogItem,
} from "@/gen/rpc";
import { cn } from "@/lib/utils";

const MODEL_PRESETS = [
  {
    id: "anthropic/claude-3.5-sonnet",
    name: "Claude 3.5 Sonnet (Recommended)",
    provider: "Anthropic",
  },
  { id: "openai/gpt-4o-mini", name: "GPT-4o Mini (Fast & Low Cost)", provider: "OpenAI" },
  {
    id: "google/gemini-2.0-flash-001",
    name: "Gemini 2.0 Flash (Fast & Capable)",
    provider: "Google",
  },
  {
    id: "meta-llama/llama-3.3-70b-instruct",
    name: "Llama 3.3 70B (Open Weights)",
    provider: "Meta",
  },
];

export function AiSettingsCard() {
  const [activeSubTab, setActiveSubTab] = useState("privacy");
  const [feedbackMsg, setFeedbackMsg] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  // 1. Privacy Settings State
  const [privacy, setPrivacy] = useState<PrivacySettings>({
    allow_student_names: false,
    allow_phone_numbers: false,
    allow_guardian_info: false,
    allow_financial_figures: false,
    allow_exam_marks: false,
    allow_staff_names: false,
    updated_at: new Date().toISOString(),
  });
  const [privacyLoading, setPrivacyLoading] = useState(false);
  const [privacySaving, setPrivacySaving] = useState(false);

  // 2. Wallet & Credit Ledger State
  const [wallet, setWallet] = useState<AiWalletBalance | null>(null);
  const [ledger, setLedger] = useState<CreditTransactionItem[]>([]);
  const [walletLoading, setWalletLoading] = useState(false);
  const [rechargeOpen, setRechargeOpen] = useState(false);
  const [grantAmount, setGrantAmount] = useState<number>(500);
  const [grantDescription, setGrantDescription] = useState<string>(
    "Institutional promotional top-up"
  );
  const [grantRefId, setGrantRefId] = useState<string>("");
  const [grantLoading, setGrantLoading] = useState(false);

  // 3. Gateway Configuration State
  const [gatewayConfig, setGatewayConfig] = useState<AiGatewayConfig | null>(null);
  const [gatewayUrl, setGatewayUrl] = useState("https://openrouter.ai/api/v1");
  const [apiKey, setApiKey] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);
  const [selectedModel, setSelectedModel] = useState("anthropic/claude-3.5-sonnet");
  const [customModel, setCustomModel] = useState("");
  const [gatewaySaving, setGatewaySaving] = useState(false);
  const [testResult, setTestResult] = useState<GatewayTestResult | null>(null);
  const [testingConnection, setTestingConnection] = useState(false);

  // 4. Audit Log State
  const [auditLogs, setAuditLogs] = useState<AiAuditLogItem[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);

  const showNotification = useCallback((type: "success" | "error", text: string) => {
    setFeedbackMsg({ type, text });
    const timer = setTimeout(() => setFeedbackMsg(null), 4000);
    return () => clearTimeout(timer);
  }, []);

  // Fetch Privacy Settings
  const fetchPrivacySettings = useCallback(async () => {
    setPrivacyLoading(true);
    try {
      const res = await rpc("ai.get_privacy_settings", {});
      if (res.ok && res.data) {
        setPrivacy(res.data);
      }
    } catch (err: unknown) {
      console.error("Failed to load privacy settings:", err);
    } finally {
      setPrivacyLoading(false);
    }
  }, []);

  // Fetch Wallet & Ledger
  const fetchWalletAndLedger = useCallback(async () => {
    setWalletLoading(true);
    try {
      const [walletRes, ledgerRes] = await Promise.all([
        rpc("ai.get_wallet_balance", {}),
        rpc("ai.get_credit_ledger", { limit: 25 }),
      ]);
      if (walletRes.ok && walletRes.data) {
        setWallet(walletRes.data);
      }
      if (ledgerRes.ok && ledgerRes.data) {
        setLedger(ledgerRes.data);
      }
    } catch (err: unknown) {
      console.error("Failed to load AI wallet:", err);
    } finally {
      setWalletLoading(false);
    }
  }, []);

  // Fetch Gateway Configuration
  const fetchGatewayConfig = useCallback(async () => {
    try {
      const res = await rpc("ai.get_gateway_config", {});
      if (res.ok && res.data) {
        setGatewayConfig(res.data);
        setGatewayUrl(res.data.gateway_url);
      }
    } catch (err: unknown) {
      console.error("Failed to load gateway config:", err);
    }
  }, []);

  // Fetch Audit Logs
  const fetchAuditLogs = useCallback(async () => {
    setAuditLoading(true);
    try {
      const res = await rpc("ai.get_audit_log", { limit: 30 });
      if (res.ok && res.data) {
        setAuditLogs(res.data);
      }
    } catch (err: unknown) {
      console.error("Failed to load AI audit log:", err);
    } finally {
      setAuditLoading(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    void fetchPrivacySettings();
    void fetchWalletAndLedger();
    void fetchGatewayConfig();
    void fetchAuditLogs();
  }, [fetchPrivacySettings, fetchWalletAndLedger, fetchGatewayConfig, fetchAuditLogs]);

  // Save Privacy Settings
  const handleSavePrivacy = async (updated: PrivacySettings) => {
    setPrivacySaving(true);
    try {
      const res = await rpc("ai.update_privacy_settings", {
        allow_student_names: updated.allow_student_names,
        allow_phone_numbers: updated.allow_phone_numbers,
        allow_guardian_info: updated.allow_guardian_info,
        allow_financial_figures: updated.allow_financial_figures,
        allow_exam_marks: updated.allow_exam_marks,
        allow_staff_names: updated.allow_staff_names,
      });
      if (res.ok && res.data) {
        setPrivacy(res.data);
        showNotification("success", "Privacy policy updated and enforced immediately.");
      } else {
        showNotification(
          "error",
          res.error?.message ?? "Failed to save privacy settings."
        );
      }
    } catch (err: unknown) {
      showNotification("error", err instanceof Error ? err.message : String(err));
    } finally {
      setPrivacySaving(false);
    }
  };

  // Reset Privacy to Default (All OFF)
  const handleResetPrivacy = () => {
    const defaulted: PrivacySettings = {
      ...privacy,
      allow_student_names: false,
      allow_phone_numbers: false,
      allow_guardian_info: false,
      allow_financial_figures: false,
      allow_exam_marks: false,
      allow_staff_names: false,
    };
    setPrivacy(defaulted);
    void handleSavePrivacy(defaulted);
  };

  // Toggle single privacy category
  const handleTogglePrivacy = (
    key: keyof Omit<PrivacySettings, "updated_at" | "updated_by">
  ) => {
    const updated = {
      ...privacy,
      [key]: !privacy[key],
    };
    setPrivacy(updated);
    void handleSavePrivacy(updated);
  };

  // Sync Credits
  const handleSyncCredits = async () => {
    setWalletLoading(true);
    try {
      const res = await rpc("ai.sync_credits", {});
      if (res.ok && res.data) {
        setWallet(res.data);
        showNotification("success", "Institutional AI wallet balance synchronized.");
        void fetchWalletAndLedger();
      } else {
        showNotification("error", res.error?.message ?? "Failed to sync credits.");
      }
    } catch (err: unknown) {
      showNotification("error", err instanceof Error ? err.message : String(err));
    } finally {
      setWalletLoading(false);
    }
  };

  // Grant Credits
  const handleRecordGrant = async () => {
    if (grantAmount <= 0) return;
    setGrantLoading(true);
    try {
      const res = await rpc("ai.record_credit_grant", {
        amount: grantAmount,
        tx_type: "grant",
        description: grantDescription.trim() || "Manual institutional credit grant",
        reference_id: grantRefId.trim() || null,
      });
      if (res.ok) {
        showNotification("success", `Granted +${grantAmount} AI credits to wallet.`);
        setRechargeOpen(false);
        setGrantRefId("");
        void fetchWalletAndLedger();
      } else {
        showNotification("error", res.error?.message ?? "Failed to record credit grant.");
      }
    } catch (err: unknown) {
      showNotification("error", err instanceof Error ? err.message : String(err));
    } finally {
      setGrantLoading(false);
    }
  };

  // Save Gateway Config
  const handleSaveGateway = async () => {
    setGatewaySaving(true);
    try {
      const res = await rpc("ai.update_gateway_config", {
        gateway_url: gatewayUrl.trim() || "https://openrouter.ai/api/v1",
        openrouter_key: apiKey.trim() ? apiKey.trim() : null,
      });
      if (res.ok && res.data) {
        setGatewayConfig(res.data);
        setApiKey("");
        showNotification("success", "AI Gateway credentials saved successfully.");
      } else {
        showNotification("error", res.error?.message ?? "Failed to save gateway config.");
      }
    } catch (err: unknown) {
      showNotification("error", err instanceof Error ? err.message : String(err));
    } finally {
      setGatewaySaving(false);
    }
  };

  // Test Gateway Connection
  const handleTestConnection = async () => {
    setTestingConnection(true);
    setTestResult(null);
    try {
      const res = await rpc("ai.test_gateway_connection", {});
      if (res.ok && res.data) {
        setTestResult(res.data);
        if (res.data.ok) {
          showNotification("success", `Gateway responded in ${res.data.latency_ms}ms.`);
        } else {
          showNotification("error", `Gateway test failed: ${res.data.provider_status}`);
        }
      } else {
        showNotification("error", res.error?.message ?? "Gateway test failed.");
      }
    } catch (err: unknown) {
      showNotification("error", err instanceof Error ? err.message : String(err));
    } finally {
      setTestingConnection(false);
    }
  };

  const privacyItems = [
    {
      key: "allow_student_names" as const,
      label: "Student Full Names",
      desc: "When OFF, names are pseudonymized to Student #[code] before leaving this machine and re-hydrated locally.",
      badge: "Scrubbed to Student #[code]",
    },
    {
      key: "allow_phone_numbers" as const,
      label: "Bangladeshi Mobile Numbers",
      desc: "When OFF, all 11-digit numbers matching 01[3-9]\\d{8} are strictly scrubbed to [PHONE REDACTED].",
      badge: "Scrubbed to [PHONE REDACTED]",
    },
    {
      key: "allow_guardian_info" as const,
      label: "Guardian Names & Contacts",
      desc: "When OFF, parent relationships, professions, and contacts are redacted from AI tool responses.",
      badge: "Scrubbed to [GUARDIAN REDACTED]",
    },
    {
      key: "allow_financial_figures" as const,
      label: "Financial Figures & Dues Amounts",
      desc: "When OFF, exact fee amounts and tuition totals are scrubbed to [AMOUNT REDACTED].",
      badge: "Scrubbed to [AMOUNT REDACTED]",
    },
    {
      key: "allow_exam_marks" as const,
      label: "Exam Marks & Scores",
      desc: "When OFF, specific student exam marks and grades are masked to [EXAM MARKS REDACTED].",
      badge: "Scrubbed to [EXAM MARKS REDACTED]",
    },
    {
      key: "allow_staff_names" as const,
      label: "Teacher & Staff Names",
      desc: "When OFF, instructor and employee identities are scrubbed to Teacher #[code].",
      badge: "Scrubbed to Teacher #[code]",
    },
  ];

  return (
    <div className="space-y-6">
      {/* Feedback Toast Banner */}
      {feedbackMsg && (
        <div
          className={cn(
            "p-3 rounded-[var(--radius)] text-xs flex items-center justify-between border transition-all animate-in fade-in duration-200",
            feedbackMsg.type === "success"
              ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-900 dark:text-emerald-200"
              : "bg-destructive/10 border-destructive/30 text-destructive dark:text-red-400"
          )}
        >
          <div className="flex items-center gap-2">
            {feedbackMsg.type === "success" ? (
              <CheckCircle2 className="h-4 w-4 shrink-0" />
            ) : (
              <AlertCircle className="h-4 w-4 shrink-0" />
            )}
            <span className="font-medium">{feedbackMsg.text}</span>
          </div>
          <button
            type="button"
            onClick={() => setFeedbackMsg(null)}
            className="text-xs opacity-70 hover:opacity-100 cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Main Container Card */}
      <Card className="border-border">
        <CardHeader className="border-b border-border pb-4">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div>
              <CardTitle className="flex items-center gap-2 text-lg font-bold">
                <Sparkles className="h-5 w-5 text-primary" />
                <span>AI Foundation & Institutional Intelligence</span>
                <Badge
                  variant="outline"
                  className="text-[10px] font-mono border-primary/30 text-primary"
                >
                  Slice 10
                </Badge>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground mt-1">
                Zero-leaked-data privacy engine, institutional credit wallet, OpenRouter
                LLM gateway, and tool audit logging.
              </CardDescription>
            </div>

            {/* Quick Wallet Summary Pill */}
            {wallet && (
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary/10 border border-primary/20 text-xs font-medium text-foreground self-start sm:self-auto">
                <Coins className="h-3.5 w-3.5 text-primary" />
                <span>{wallet.balance_credits.toLocaleString()} Credits Available</span>
              </div>
            )}
          </div>

          {/* Sub Navigation Tabs */}
          <Tabs
            value={activeSubTab}
            onValueChange={setActiveSubTab}
            className="w-full mt-4"
          >
            <TabsList className="grid grid-cols-2 md:grid-cols-4 w-full h-auto p-1 bg-muted/60">
              <TabsTrigger value="privacy" className="gap-2 text-xs py-2 cursor-pointer">
                <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />
                <span>Privacy Shield</span>
              </TabsTrigger>
              <TabsTrigger value="wallet" className="gap-2 text-xs py-2 cursor-pointer">
                <Coins className="h-3.5 w-3.5 text-primary" />
                <span>Credit Wallet</span>
              </TabsTrigger>
              <TabsTrigger value="gateway" className="gap-2 text-xs py-2 cursor-pointer">
                <Cpu className="h-3.5 w-3.5 text-blue-500" />
                <span>OpenRouter Gateway</span>
              </TabsTrigger>
              <TabsTrigger value="audit" className="gap-2 text-xs py-2 cursor-pointer">
                <History className="h-3.5 w-3.5 text-amber-500" />
                <span>AI Audit Log</span>
              </TabsTrigger>
            </TabsList>

            {/* TAB 1: PRIVACY SHIELD */}
            <TabsContent value="privacy" className="mt-6 space-y-6">
              {/* Privacy Architecture Info Card */}
              <div className="rounded-[var(--radius-lg)] border border-emerald-500/20 bg-emerald-500/5 p-4 space-y-2">
                <div className="flex items-center gap-2 text-emerald-900 dark:text-emerald-300 font-semibold text-xs">
                  <ShieldCheck className="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
                  <span>Zero-Leaked-Data Privacy Engine (Feature 55)</span>
                  <Badge
                    variant="outline"
                    className="text-[10px] bg-emerald-500/10 border-emerald-500/30 text-emerald-700 dark:text-emerald-300"
                  >
                    Default: OFF (Strict)
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Coachify guarantees that student PII, mobile numbers, guardian
                  relationships, and ledger balances never enter cloud prompts without
                  explicit owner consent. All personal data is masked locally on this
                  workstation prior to calling OpenRouter, and names are de-anonymized
                  in-memory inside the Rust core before being returned to your screen.
                </p>
              </div>

              {/* Privacy Toggles Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {privacyItems.map((item) => {
                  const isAllowed = privacy[item.key];
                  return (
                    <div
                      key={item.key}
                      className={cn(
                        "p-4 rounded-[var(--radius)] border transition-all duration-200 flex flex-col justify-between gap-3",
                        isAllowed
                          ? "bg-amber-500/5 border-amber-500/30"
                          : "bg-card border-border hover:border-emerald-500/40"
                      )}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="space-y-1">
                          <div className="flex items-center gap-2">
                            <span className="font-semibold text-xs text-foreground">
                              {item.label}
                            </span>
                            {!isAllowed && (
                              <Badge
                                variant="outline"
                                className="text-[9px] bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                              >
                                Protected
                              </Badge>
                            )}
                            {isAllowed && (
                              <Badge
                                variant="outline"
                                className="text-[9px] bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/30"
                              >
                                Allowed to LLM
                              </Badge>
                            )}
                          </div>
                          <p className="text-[11px] text-muted-foreground leading-relaxed">
                            {item.desc}
                          </p>
                        </div>
                        <Switch
                          checked={isAllowed}
                          onCheckedChange={() => handleTogglePrivacy(item.key)}
                          disabled={privacyLoading || privacySaving}
                        />
                      </div>
                      <div className="pt-2 border-t border-border/40 text-[10px] text-muted-foreground flex items-center justify-between">
                        <span>Masking Rule:</span>
                        <code className="text-muted-foreground/90 font-mono text-[9px] bg-muted px-1.5 py-0.5 rounded">
                          {item.badge}
                        </code>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Privacy Actions & Reset */}
              <div className="flex flex-col sm:flex-row items-center justify-between gap-3 pt-4 border-t border-border">
                <div className="text-[11px] text-muted-foreground">
                  Last updated: {new Date(privacy.updated_at).toLocaleString()}
                </div>
                <div className="flex items-center gap-2 w-full sm:w-auto">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleResetPrivacy}
                    disabled={privacyLoading || privacySaving}
                    className="text-xs cursor-pointer gap-1.5"
                  >
                    <ShieldAlert className="h-3.5 w-3.5 text-amber-500" />
                    Reset to Maximum Privacy (All OFF)
                  </Button>
                </div>
              </div>
            </TabsContent>

            {/* TAB 2: CREDIT WALLET */}
            <TabsContent value="wallet" className="mt-6 space-y-6">
              {/* Wallet Summary Row */}
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="p-4 rounded-[var(--radius)] border border-border bg-card space-y-1">
                  <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
                    <span>Available Balance</span>
                    <Coins className="h-4 w-4 text-primary" />
                  </div>
                  <div className="text-2xl font-bold text-foreground">
                    {wallet ? wallet.balance_credits.toLocaleString() : "..."}
                    <span className="text-xs font-normal text-muted-foreground ml-1">
                      Credits
                    </span>
                  </div>
                  <div className="text-[10px] text-muted-foreground flex items-center gap-1.5 pt-1">
                    <span
                      className={cn(
                        "h-1.5 w-1.5 rounded-full",
                        wallet?.is_active ? "bg-emerald-500" : "bg-destructive"
                      )}
                    />
                    <span>
                      {wallet?.is_active ? "Wallet Active" : "Wallet Suspended"}
                    </span>
                  </div>
                </div>

                <div className="p-4 rounded-[var(--radius)] border border-border bg-card space-y-1">
                  <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
                    <span>Cost Invariant</span>
                    <Lock className="h-4 w-4 text-emerald-500" />
                  </div>
                  <div className="text-sm font-semibold text-foreground mt-1">
                    1 Turn = 1 Credit
                  </div>
                  <div className="text-[10px] text-muted-foreground pt-1 leading-relaxed">
                    Deducted strictly upon successful response delivery. Zero credits
                    billed on connection drops or errors.
                  </div>
                </div>

                <div className="p-4 rounded-[var(--radius)] border border-border bg-card flex flex-col justify-between">
                  <div>
                    <div className="text-[11px] font-medium text-muted-foreground flex items-center justify-between">
                      <span>Quick Actions</span>
                      <Activity className="h-4 w-4 text-blue-500" />
                    </div>
                    <div className="text-[10px] text-muted-foreground mt-1">
                      Manual reconciliation & institutional grants
                    </div>
                  </div>
                  <div className="flex items-center gap-2 mt-3">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleSyncCredits}
                      disabled={walletLoading}
                      className="text-xs h-8 flex-1 cursor-pointer gap-1"
                    >
                      <RefreshCw
                        className={cn("h-3 w-3", walletLoading && "animate-spin")}
                      />
                      Sync
                    </Button>
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => setRechargeOpen(true)}
                      className="text-xs h-8 flex-1 cursor-pointer gap-1"
                    >
                      <Plus className="h-3 w-3" />
                      Grant Credits
                    </Button>
                  </div>
                </div>
              </div>

              {/* Transactions Ledger Table */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <h4 className="text-xs font-semibold text-foreground uppercase tracking-wider">
                    Institutional Credit Ledger (Immutable Log)
                  </h4>
                  <span className="text-[11px] text-muted-foreground">
                    Showing latest {ledger.length} events
                  </span>
                </div>

                <div className="rounded-[var(--radius)] border border-border overflow-hidden bg-card">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="text-left text-xs">Date & Time</TableHead>
                        <TableHead className="text-left text-xs">Type</TableHead>
                        <TableHead className="text-right text-xs">Change</TableHead>
                        <TableHead className="text-right text-xs">
                          Balance After
                        </TableHead>
                        <TableHead className="text-left text-xs">Description</TableHead>
                        <TableHead className="text-left text-xs">Ref ID / User</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {ledger.length === 0 ? (
                        <TableRow>
                          <TableCell
                            colSpan={6}
                            className="text-center py-8 text-xs text-muted-foreground"
                          >
                            No transactions recorded yet in the AI credit ledger.
                          </TableCell>
                        </TableRow>
                      ) : (
                        ledger.map((item) => {
                          const isGrant = item.amount > 0;
                          return (
                            <TableRow key={item.id} className="text-xs">
                              <TableCell className="text-muted-foreground whitespace-nowrap">
                                {new Date(item.created_at).toLocaleString()}
                              </TableCell>
                              <TableCell>
                                <Badge
                                  variant="outline"
                                  className={cn(
                                    "text-[10px] uppercase font-mono",
                                    isGrant
                                      ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                                      : "bg-muted text-muted-foreground"
                                  )}
                                >
                                  {item.transaction_type}
                                </Badge>
                              </TableCell>
                              <TableCell className="text-right font-mono font-medium whitespace-nowrap">
                                <span
                                  className={
                                    isGrant
                                      ? "text-emerald-600 dark:text-emerald-400"
                                      : "text-muted-foreground"
                                  }
                                >
                                  {isGrant ? `+${item.amount}` : item.amount}
                                </span>
                              </TableCell>
                              <TableCell className="text-right font-mono text-foreground font-semibold">
                                {item.balance_after.toLocaleString()}
                              </TableCell>
                              <TableCell
                                className="max-w-[220px] truncate text-muted-foreground"
                                title={item.description}
                              >
                                {item.description}
                              </TableCell>
                              <TableCell className="text-muted-foreground text-[11px] truncate max-w-[120px]">
                                {item.reference_id || item.created_by_name || "System"}
                              </TableCell>
                            </TableRow>
                          );
                        })
                      )}
                    </TableBody>
                  </Table>
                </div>
              </div>
            </TabsContent>

            {/* TAB 3: OPENROUTER GATEWAY */}
            <TabsContent value="gateway" className="mt-6 space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Credentials & Provider Form */}
                <div className="space-y-4">
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">
                      Gateway Base Endpoint
                    </label>
                    <Input
                      type="text"
                      value={gatewayUrl}
                      onChange={(e) => setGatewayUrl(e.target.value)}
                      placeholder="https://openrouter.ai/api/v1"
                      className="text-xs font-mono h-9"
                    />
                    <p className="text-[11px] text-muted-foreground">
                      Defaults to OpenRouter. Custom OpenAI-compatible endpoints can also
                      be specified.
                    </p>
                  </div>

                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground flex items-center justify-between">
                      <span>OpenRouter API Key (BYOK)</span>
                      <button
                        type="button"
                        onClick={() => setShowApiKey(!showApiKey)}
                        className="text-[11px] text-primary hover:underline flex items-center gap-1 cursor-pointer"
                      >
                        {showApiKey ? (
                          <EyeOff className="h-3 w-3" />
                        ) : (
                          <Eye className="h-3 w-3" />
                        )}
                        <span>{showApiKey ? "Hide" : "Show"}</span>
                      </button>
                    </label>
                    <div className="relative">
                      <Input
                        type={showApiKey ? "text" : "password"}
                        value={apiKey}
                        onChange={(e) => setApiKey(e.target.value)}
                        placeholder={
                          gatewayConfig?.is_configured
                            ? "••••••••••••••••••••••••"
                            : "sk-or-v1-..."
                        }
                        className="text-xs font-mono h-9 pr-8"
                      />
                      <Key className="absolute right-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
                    </div>
                    <p className="text-[11px] text-muted-foreground">
                      Stored locally in machine app configuration. Never shared with third
                      parties.
                    </p>
                  </div>

                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">
                      Default Language Model
                    </label>
                    <div className="grid grid-cols-1 gap-2">
                      {MODEL_PRESETS.map((preset) => (
                        <button
                          key={preset.id}
                          type="button"
                          onClick={() => {
                            setSelectedModel(preset.id);
                            setCustomModel("");
                          }}
                          className={cn(
                            "p-2.5 rounded-[var(--radius)] border text-left text-xs transition-all cursor-pointer flex items-center justify-between",
                            selectedModel === preset.id && !customModel
                              ? "bg-primary/10 border-primary text-foreground font-medium"
                              : "bg-card border-border hover:bg-muted text-muted-foreground"
                          )}
                        >
                          <div>
                            <div className="text-xs font-semibold text-foreground">
                              {preset.name}
                            </div>
                            <div className="text-[10px] text-muted-foreground font-mono">
                              {preset.id}
                            </div>
                          </div>
                          {selectedModel === preset.id && !customModel && (
                            <Check className="h-4 w-4 text-primary shrink-0" />
                          )}
                        </button>
                      ))}
                    </div>

                    <div className="pt-2">
                      <Input
                        type="text"
                        value={customModel}
                        onChange={(e) => {
                          setCustomModel(e.target.value);
                          if (e.target.value.trim()) {
                            setSelectedModel(e.target.value.trim());
                          }
                        }}
                        placeholder="Or enter custom OpenRouter model ID..."
                        className="text-xs font-mono h-8"
                      />
                    </div>
                  </div>

                  <div className="flex items-center gap-2 pt-2">
                    <Button
                      onClick={handleSaveGateway}
                      disabled={gatewaySaving}
                      className="text-xs h-9 cursor-pointer gap-1.5 flex-1"
                    >
                      {gatewaySaving ? (
                        <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <Check className="h-3.5 w-3.5" />
                      )}
                      Save Configuration
                    </Button>
                    <Button
                      variant="outline"
                      onClick={handleTestConnection}
                      disabled={testingConnection}
                      className="text-xs h-9 cursor-pointer gap-1.5"
                    >
                      {testingConnection ? (
                        <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                      ) : (
                        <Activity className="h-3.5 w-3.5" />
                      )}
                      Test Connection
                    </Button>
                  </div>
                </div>

                {/* Gateway Status & Diagnostics Box */}
                <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-muted/30 space-y-4 flex flex-col justify-between">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between border-b border-border pb-3">
                      <h4 className="text-xs font-semibold text-foreground uppercase tracking-wider">
                        Connection Diagnostics
                      </h4>
                      <Badge
                        variant="outline"
                        className={cn(
                          "text-[10px]",
                          gatewayConfig?.is_configured
                            ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                            : "bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/30"
                        )}
                      >
                        {gatewayConfig?.is_configured
                          ? "Credentials Stored"
                          : "Not Configured"}
                      </Badge>
                    </div>

                    <div className="space-y-2 text-xs text-muted-foreground">
                      <div className="flex items-center justify-between">
                        <span>Provider:</span>
                        <span className="font-semibold text-foreground">
                          {gatewayConfig?.upstream_provider || "OpenRouter"}
                        </span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span>Endpoint:</span>
                        <code className="text-[11px] font-mono text-foreground truncate max-w-[200px]">
                          {gatewayUrl}
                        </code>
                      </div>
                      <div className="flex items-center justify-between">
                        <span>Mock Mode:</span>
                        <span className="font-medium text-foreground">
                          {gatewayConfig?.is_mock
                            ? "Enabled (Offline Simulation)"
                            : "Live (OpenRouter API)"}
                        </span>
                      </div>
                    </div>

                    {/* Test Results Box */}
                    {testResult && (
                      <div
                        className={cn(
                          "p-3 rounded-[var(--radius)] border text-xs space-y-1 animate-in fade-in duration-150",
                          testResult.ok
                            ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-900 dark:text-emerald-200"
                            : "bg-destructive/10 border-destructive/30 text-destructive dark:text-red-400"
                        )}
                      >
                        <div className="font-semibold flex items-center gap-1.5">
                          {testResult.ok ? (
                            <CheckCircle2 className="h-4 w-4" />
                          ) : (
                            <AlertCircle className="h-4 w-4" />
                          )}
                          <span>
                            {testResult.ok
                              ? "Gateway Online & Responsive"
                              : "Connection Test Failed"}
                          </span>
                        </div>
                        <div className="text-[11px] opacity-90">
                          Latency: <strong>{testResult.latency_ms} ms</strong> • Provider
                          Status: {testResult.provider_status}
                        </div>
                      </div>
                    )}
                  </div>

                  <div className="p-3 rounded-[var(--radius)] bg-card border border-border text-[11px] text-muted-foreground space-y-1">
                    <div className="font-semibold text-foreground flex items-center gap-1">
                      <Lock className="h-3 w-3 text-emerald-500" />
                      <span>Security & Zero SQL Access</span>
                    </div>
                    <p className="text-[10px] leading-relaxed">
                      Coachify does not allow the AI model to execute arbitrary SQL
                      statements. Only 6 strictly typed read-only tool functions are
                      exposed, clamping results to 50 rows maximum.
                    </p>
                  </div>
                </div>
              </div>
            </TabsContent>

            {/* TAB 4: AUDIT LOG */}
            <TabsContent value="audit" className="mt-6 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-xs font-semibold text-foreground uppercase tracking-wider">
                    AI Execution Audit Trail (Feature 57)
                  </h4>
                  <p className="text-[11px] text-muted-foreground mt-0.5">
                    Immutable log of user prompts, executed tools, latency, and redaction
                    counts.
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={fetchAuditLogs}
                  disabled={auditLoading}
                  className="text-xs h-8 cursor-pointer gap-1.5"
                >
                  <RefreshCw className={cn("h-3 w-3", auditLoading && "animate-spin")} />
                  Refresh
                </Button>
              </div>

              <div className="rounded-[var(--radius)] border border-border overflow-hidden bg-card">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="text-left text-xs">Date & Time</TableHead>
                      <TableHead className="text-left text-xs">
                        User Prompt Summary
                      </TableHead>
                      <TableHead className="text-left text-xs">Tools Invoked</TableHead>
                      <TableHead className="text-center text-xs">Redactions</TableHead>
                      <TableHead className="text-right text-xs">Credits</TableHead>
                      <TableHead className="text-right text-xs">Latency</TableHead>
                      <TableHead className="text-right text-xs">Tokens</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {auditLogs.length === 0 ? (
                      <TableRow>
                        <TableCell
                          colSpan={7}
                          className="text-center py-8 text-xs text-muted-foreground"
                        >
                          No AI Copilot queries recorded yet.
                        </TableCell>
                      </TableRow>
                    ) : (
                      auditLogs.map((log) => (
                        <TableRow key={log.id} className="text-xs">
                          <TableCell className="text-muted-foreground whitespace-nowrap">
                            {new Date(log.created_at).toLocaleString()}
                          </TableCell>
                          <TableCell
                            className="font-medium text-foreground max-w-[200px] truncate"
                            title={log.prompt_summary}
                          >
                            {log.prompt_summary}
                          </TableCell>
                          <TableCell>
                            <div className="flex flex-wrap gap-1">
                              {log.tools_invoked.length === 0 ? (
                                <span className="text-muted-foreground text-[10px]">
                                  —
                                </span>
                              ) : (
                                log.tools_invoked.map((tool, idx) => (
                                  <Badge
                                    key={idx}
                                    variant="outline"
                                    className="text-[9px] bg-muted font-mono"
                                  >
                                    {tool}
                                  </Badge>
                                ))
                              )}
                            </div>
                          </TableCell>
                          <TableCell className="text-center">
                            {log.redaction_count > 0 ? (
                              <Badge
                                variant="outline"
                                className="text-[9px] bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30"
                              >
                                {log.redaction_count} scrubbed
                              </Badge>
                            ) : (
                              <span className="text-muted-foreground text-[10px]">0</span>
                            )}
                          </TableCell>
                          <TableCell className="text-right font-mono text-muted-foreground">
                            {log.credits_used}
                          </TableCell>
                          <TableCell className="text-right font-mono text-muted-foreground">
                            {log.latency_ms} ms
                          </TableCell>
                          <TableCell className="text-right font-mono text-[11px] text-muted-foreground whitespace-nowrap">
                            {log.prompt_tokens + log.completion_tokens}
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </div>
            </TabsContent>
          </Tabs>
        </CardHeader>
      </Card>

      {/* Grant Credits Modal Dialog */}
      <Dialog open={rechargeOpen} onOpenChange={setRechargeOpen}>
        <DialogContent className="sm:max-w-[420px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base">
              <Coins className="h-4 w-4 text-primary" />
              <span>Grant Institutional Credits</span>
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Add prepaid AI credits to the institutional wallet ledger. All grants are
              permanently audited.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-2">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Credit Amount
              </label>
              <div className="flex gap-2 mb-2">
                {[100, 500, 1000].map((amt) => (
                  <Button
                    key={amt}
                    type="button"
                    variant={grantAmount === amt ? "primary" : "outline"}
                    size="sm"
                    onClick={() => setGrantAmount(amt)}
                    className="flex-1 text-xs cursor-pointer"
                  >
                    +{amt}
                  </Button>
                ))}
              </div>
              <Input
                type="number"
                value={grantAmount}
                onChange={(e) => setGrantAmount(parseInt(e.target.value, 10) || 0)}
                placeholder="Enter credit amount"
                className="text-xs h-9"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Description / Purpose
              </label>
              <Input
                type="text"
                value={grantDescription}
                onChange={(e) => setGrantDescription(e.target.value)}
                placeholder="e.g. Monthly institutional allocation"
                className="text-xs h-9"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Reference ID (Optional)
              </label>
              <Input
                type="text"
                value={grantRefId}
                onChange={(e) => setGrantRefId(e.target.value)}
                placeholder="e.g. INV-2026-001"
                className="text-xs h-9 font-mono"
              />
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setRechargeOpen(false)}
              className="text-xs cursor-pointer"
            >
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={handleRecordGrant}
              disabled={grantLoading || grantAmount <= 0}
              className="text-xs cursor-pointer gap-1.5"
            >
              {grantLoading ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Check className="h-3.5 w-3.5" />
              )}
              Confirm Credit Grant
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
