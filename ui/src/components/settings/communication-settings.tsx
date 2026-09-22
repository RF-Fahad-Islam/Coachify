import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type ProviderConfigItem,
  type MessageTemplateItem,
  type AutomationRuleItem,
  type WhatsAppStatusResponse,
} from "@/gen/rpc";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Smartphone,
  MessageSquare,
  Mail,
  KeyRound,
  FileText,
  Zap,
  Send,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Lock,
  Plus,
  Trash2,
  Edit2,
  ShieldCheck,
} from "lucide-react";
import { cn } from "@/lib/utils";

export function CommunicationSettings() {
  const [activeTab, setActiveTab] = useState<string>("providers");
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  // Provider configs
  const [providerConfigs, setProviderConfigs] = useState<ProviderConfigItem[]>([]);
  const [smsAdapter, setSmsAdapter] = useState("greenweb");
  const [smsEndpoint, setSmsEndpoint] = useState("https://api.greenweb.com.bd/api.php");
  const [smsSenderId, setSmsSenderId] = useState("");
  const [smsSecret, setSmsSecret] = useState("");

  const [emailHost, setEmailHost] = useState("smtp.gmail.com");
  const [emailPort, setEmailPort] = useState(587);
  const [emailUsername, setEmailUsername] = useState("");
  const [emailFromAddress, setEmailFromAddress] = useState("");
  const [emailFromName, setEmailFromName] = useState("");
  const [emailPassword, setEmailPassword] = useState("");

  // WhatsApp status
  const [whatsappStatus, setWhatsappStatus] = useState<WhatsAppStatusResponse | null>(
    null
  );
  const [whatsappQr, setWhatsappQr] = useState<string | null>(null);

  // Test Message
  const [testChannel, setTestChannel] = useState<"sms" | "whatsapp" | "email">("sms");
  const [testRecipient, setTestRecipient] = useState("");
  const [testSubject, setTestSubject] = useState("Coachify Test Notice");
  const [testBody, setTestBody] = useState(
    "Hello from Coachify. This is a gateway test message."
  );
  const [testSending, setTestSending] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  // Templates
  const [templates, setTemplates] = useState<MessageTemplateItem[]>([]);
  const [templateDialogOpen, setTemplateDialogOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<MessageTemplateItem | null>(
    null
  );
  const [tmplKey, setTmplKey] = useState("");
  const [tmplChannel, setTmplChannel] = useState("sms");
  const [tmplLang, setTmplLang] = useState("en");
  const [tmplTitle, setTmplTitle] = useState("");
  const [tmplSubject, setTmplSubject] = useState("");
  const [tmplBody, setTmplBody] = useState("");
  const [tmplSaving, setTmplSaving] = useState(false);

  // Automation Rules
  const [rules, setRules] = useState<AutomationRuleItem[]>([]);
  const [ruleSavingId, setRuleSavingId] = useState<string | null>(null);

  // Load all comms settings
  const loadSettings = useCallback(async () => {
    setError(null);

    try {
      const [configsRes, tmplsRes, rulesRes, waRes] = await Promise.all([
        rpc("comms.get_provider_configs", {}),
        rpc("comms.list_templates", {}),
        rpc("comms.list_automation_rules", {}),
        rpc("whatsapp.get_status", {}),
      ]);

      if (configsRes.ok && configsRes.data) {
        setProviderConfigs(configsRes.data);
        // Prepopulate SMS
        const smsConfig = configsRes.data.find((c) => c.channel === "sms" && c.is_active);
        if (smsConfig) {
          setSmsAdapter(smsConfig.adapter);
          try {
            const parsed = JSON.parse(smsConfig.config_json);
            if (parsed.api_url) setSmsEndpoint(parsed.api_url);
            if (parsed.sender_id) setSmsSenderId(parsed.sender_id);
          } catch {
            // Ignore parse errors
          }
        }
        // Prepopulate Email
        const emailConfig = configsRes.data.find(
          (c) => c.channel === "email" && c.is_active
        );
        if (emailConfig) {
          try {
            const parsed = JSON.parse(emailConfig.config_json);
            if (parsed.host) setEmailHost(parsed.host);
            if (parsed.port) setEmailPort(parsed.port);
            if (parsed.username) setEmailUsername(parsed.username);
            if (parsed.from_address) setEmailFromAddress(parsed.from_address);
            if (parsed.from_name) setEmailFromName(parsed.from_name);
          } catch {
            // Ignore parse errors
          }
        }
      }

      if (tmplsRes.ok && tmplsRes.data) {
        setTemplates(tmplsRes.data);
      }

      if (rulesRes.ok && rulesRes.data) {
        setRules(rulesRes.data);
      }

      if (waRes.ok && waRes.data) {
        setWhatsappStatus(waRes.data);
      }
    } catch (err: unknown) {
      setError(
        err instanceof Error ? err.message : "Failed to load communication settings"
      );
    }
  }, []);

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  // Load WhatsApp QR
  const loadWhatsAppQr = async () => {
    try {
      const res = await rpc("whatsapp.get_qr", {});
      if (res.ok && res.data) {
        setWhatsappQr(res.data.qr_code_base64 || null);
      }
    } catch (e: unknown) {
      console.error("Failed to load WhatsApp QR:", e);
    }
  };

  // Save SMS Config
  const handleSaveSms = async () => {
    setFeedback(null);
    setError(null);
    try {
      const configJson = JSON.stringify({
        api_url: smsEndpoint,
        sender_id: smsSenderId,
      });

      const res = await rpc("comms.save_provider_config", {
        channel: "sms",
        adapter: smsAdapter,
        is_active: true,
        config_json: configJson,
        secret: smsSecret.trim() ? smsSecret.trim() : null,
      });

      if (res.ok) {
        setFeedback("SMS provider settings and DPAPI encrypted credentials saved.");
        setSmsSecret("");
        void loadSettings();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to save SMS provider");
    }
  };

  // Save Email Config
  const handleSaveEmail = async () => {
    setFeedback(null);
    setError(null);
    try {
      const configJson = JSON.stringify({
        host: emailHost,
        port: emailPort,
        username: emailUsername,
        from_address: emailFromAddress,
        from_name: emailFromName,
      });

      const res = await rpc("comms.save_provider_config", {
        channel: "email",
        adapter: "smtp",
        is_active: true,
        config_json: configJson,
        secret: emailPassword.trim() ? emailPassword.trim() : null,
      });

      if (res.ok) {
        setFeedback("SMTP email settings and DPAPI encrypted credentials saved.");
        setEmailPassword("");
        void loadSettings();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to save Email provider");
    }
  };

  // Send Test Message
  const handleSendTestMessage = async () => {
    if (!testRecipient.trim() || !testBody.trim()) return;
    setTestSending(true);
    setTestResult(null);

    try {
      const res = await rpc("comms.send_test_message", {
        channel: testChannel,
        to_address: testRecipient,
        subject: testChannel === "email" ? testSubject : null,
        body: testBody,
      });

      if (res.ok && res.data) {
        if (res.data.success) {
          setTestResult(`Success! Dispatched via provider (Status: ${res.data.status})`);
        } else {
          setTestResult(`Error: ${res.data.error_message || res.data.status}`);
        }
      } else if (res.error) {
        setTestResult(`Error: ${res.error.message}`);
      }
    } catch (err: unknown) {
      setTestResult(`Error: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setTestSending(false);
    }
  };

  // Open Template Dialog
  const handleOpenTemplate = (tmpl?: MessageTemplateItem) => {
    if (tmpl) {
      setEditingTemplate(tmpl);
      setTmplKey(tmpl.key);
      setTmplChannel(tmpl.channel);
      setTmplLang(tmpl.language);
      setTmplTitle(tmpl.title);
      setTmplSubject(tmpl.subject || "");
      setTmplBody(tmpl.body);
    } else {
      setEditingTemplate(null);
      setTmplKey("");
      setTmplChannel("sms");
      setTmplLang("en");
      setTmplTitle("");
      setTmplSubject("");
      setTmplBody("");
    }
    setTemplateDialogOpen(true);
  };

  // Save Template
  const handleSaveTemplate = async () => {
    if (!tmplKey.trim() || !tmplTitle.trim() || !tmplBody.trim()) return;
    setTmplSaving(true);
    setError(null);

    try {
      const res = await rpc("comms.save_template", {
        id: editingTemplate?.id || null,
        key: tmplKey,
        channel: tmplChannel,
        language: tmplLang,
        title: tmplTitle,
        subject:
          tmplChannel === "email" && tmplSubject.trim() ? tmplSubject.trim() : null,
        body: tmplBody,
      });

      if (res.ok) {
        setFeedback("Message template saved successfully");
        setTemplateDialogOpen(false);
        void loadSettings();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to save template");
    } finally {
      setTmplSaving(false);
    }
  };

  // Delete Template
  const handleDeleteTemplate = async (id: string) => {
    try {
      const res = await rpc("comms.delete_template", { id });
      if (res.ok) {
        setFeedback("Custom template deleted");
        void loadSettings();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to delete template");
    }
  };

  // Toggle Automation Rule
  const handleToggleRule = async (rule: AutomationRuleItem, enabled: boolean) => {
    setRuleSavingId(rule.id);
    try {
      const res = await rpc("comms.update_automation_rule", {
        id: rule.id,
        enabled,
        template_id: rule.template_id,
        options_json: rule.options_json,
      });

      if (res.ok) {
        setFeedback(`Rule ${rule.trigger} for ${rule.channel} updated.`);
        void loadSettings();
      } else if (res.error) {
        setError(res.error.message);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to update automation rule");
    } finally {
      setRuleSavingId(null);
    }
  };

  // Find active configs for secret badges
  const activeSmsConfig = providerConfigs.find((c) => c.channel === "sms" && c.is_active);
  const activeEmailConfig = providerConfigs.find(
    (c) => c.channel === "email" && c.is_active
  );

  return (
    <div className="space-y-6">
      {/* Sub Header & Tabs */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <h2 className="text-xl font-bold tracking-tight text-foreground">
            Messaging & Gateway Settings
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Configure omnichannel SMS, WhatsApp, and SMTP providers, template catalog, and
            automation triggers.
          </p>
        </div>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-auto">
          <TabsList className="grid grid-cols-3 h-9 p-1 bg-muted border border-border">
            <TabsTrigger value="providers" className="gap-1.5 text-xs font-semibold">
              <KeyRound className="h-3.5 w-3.5" />
              Gateway Providers
            </TabsTrigger>
            <TabsTrigger value="templates" className="gap-1.5 text-xs font-semibold">
              <FileText className="h-3.5 w-3.5" />
              Templates Catalog
            </TabsTrigger>
            <TabsTrigger value="automations" className="gap-1.5 text-xs font-semibold">
              <Zap className="h-3.5 w-3.5" />
              Automation Rules
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* Notifications */}
      {error && (
        <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}
      {feedback && (
        <div className="p-3 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-emerald-700 dark:text-emerald-300 text-xs flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          <span>{feedback}</span>
        </div>
      )}

      {/* TAB 1: GATEWAY PROVIDERS */}
      {activeTab === "providers" && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* SMS Provider Card */}
            <Card className="p-5 border-border/80 shadow-xs space-y-4">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-bold flex items-center gap-2">
                  <Smartphone className="h-4 w-4 text-primary" />
                  Cellular SMS Gateway
                </CardTitle>
                {activeSmsConfig?.secret_is_set && (
                  <Badge variant="paid" className="gap-1 text-[10px]">
                    <ShieldCheck className="h-3 w-3" />
                    DPAPI Protected
                  </Badge>
                )}
              </div>

              <div className="space-y-3 text-xs">
                <div className="space-y-1.5">
                  <label
                    htmlFor="sms-adapter-select"
                    className="font-semibold text-foreground"
                  >
                    SMS Provider:
                  </label>
                  <select
                    id="sms-adapter-select"
                    value={smsAdapter}
                    onChange={(e) => setSmsAdapter(e.target.value)}
                    className="w-full h-9 px-3 rounded-md border border-input bg-background text-xs font-medium"
                  >
                    <option value="greenweb">Greenweb Bangladesh</option>
                    <option value="bulksms_bd">BulkSMS BD</option>
                    <option value="generic_http">Generic HTTP SMS Gateway</option>
                    <option value="mock">Local Development Mock</option>
                  </select>
                </div>

                <div className="space-y-1.5">
                  <label
                    htmlFor="sms-endpoint-input"
                    className="font-semibold text-foreground"
                  >
                    API Endpoint URL:
                  </label>
                  <Input
                    id="sms-endpoint-input"
                    value={smsEndpoint}
                    onChange={(e) => setSmsEndpoint(e.target.value)}
                    className="h-9 text-xs font-mono"
                  />
                </div>

                <div className="space-y-1.5">
                  <label
                    htmlFor="sms-sender-input"
                    className="font-semibold text-foreground"
                  >
                    Approved Masking / Sender ID:
                  </label>
                  <Input
                    id="sms-sender-input"
                    value={smsSenderId}
                    onChange={(e) => setSmsSenderId(e.target.value)}
                    placeholder="E.g. COACHIFY or non-masking number"
                    className="h-9 text-xs"
                  />
                </div>

                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <label
                      htmlFor="sms-secret-input"
                      className="font-semibold text-foreground"
                    >
                      API Token / Secret Key:
                    </label>
                    {activeSmsConfig?.masked_secret && (
                      <span className="font-mono text-[10px] text-muted-foreground">
                        Current: {activeSmsConfig.masked_secret}
                      </span>
                    )}
                  </div>
                  <Input
                    id="sms-secret-input"
                    type="password"
                    value={smsSecret}
                    onChange={(e) => setSmsSecret(e.target.value)}
                    placeholder={
                      activeSmsConfig?.secret_is_set
                        ? "Leave blank to keep existing secret"
                        : "Enter write-only API key"
                    }
                    className="h-9 text-xs"
                  />
                </div>

                <Button
                  variant="primary"
                  size="sm"
                  onClick={handleSaveSms}
                  className="w-full gap-2 font-semibold text-xs mt-2"
                >
                  <KeyRound className="h-3.5 w-3.5" />
                  Save SMS Provider Settings
                </Button>
              </div>
            </Card>

            {/* WhatsApp OpenWA Card */}
            <Card className="p-5 border-border/80 shadow-xs space-y-4">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-bold flex items-center gap-2">
                  <MessageSquare className="h-4 w-4 text-emerald-500" />
                  WhatsApp OpenWA Companion
                </CardTitle>
                <Badge
                  variant={whatsappStatus?.status === "connected" ? "paid" : "pending"}
                  className="text-[10px] capitalize"
                >
                  {whatsappStatus?.status || "disconnected"}
                </Badge>
              </div>

              <div className="space-y-3 text-xs">
                <p className="text-muted-foreground leading-relaxed">
                  Coachify pairs with an OpenWA companion daemon running locally on Port{" "}
                  <strong>{whatsappStatus?.port ?? 8000}</strong>. This lets your coaching
                  center dispatch alerts with zero per-message costs using your institute
                  phone.
                </p>

                <div className="p-3 rounded-lg bg-muted/60 border border-border flex items-center justify-between">
                  <div>
                    <div className="font-semibold text-foreground">Companion Status:</div>
                    <div className="text-muted-foreground text-[11px]">
                      {whatsappStatus?.phone_number
                        ? `Paired with ${whatsappStatus.phone_number}`
                        : "Not paired with phone"}
                    </div>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      void loadSettings();
                      void loadWhatsAppQr();
                    }}
                    className="h-8 text-xs gap-1.5"
                  >
                    <RotateCcw className="h-3 w-3" />
                    Refresh
                  </Button>
                </div>

                {whatsappQr && (
                  <div className="p-4 rounded-lg bg-card border border-border flex flex-col items-center gap-2 text-center">
                    <span className="font-semibold text-foreground text-xs">
                      Scan QR in WhatsApp Linked Devices:
                    </span>
                    <img
                      src={whatsappQr}
                      alt="WhatsApp Pairing QR Code"
                      className="h-40 w-40 rounded border border-border bg-white p-2"
                    />
                  </div>
                )}
              </div>
            </Card>
          </div>

          {/* SMTP Email Configuration */}
          <Card className="p-5 border-border/80 shadow-xs space-y-4">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-bold flex items-center gap-2">
                <Mail className="h-4 w-4 text-purple-500" />
                SMTP Transactional Email Provider
              </CardTitle>
              {activeEmailConfig?.secret_is_set && (
                <Badge variant="paid" className="gap-1 text-[10px]">
                  <ShieldCheck className="h-3 w-3" />
                  DPAPI Protected
                </Badge>
              )}
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 text-xs">
              <div className="space-y-1.5">
                <label
                  htmlFor="email-host-input"
                  className="font-semibold text-foreground"
                >
                  SMTP Host:
                </label>
                <Input
                  id="email-host-input"
                  value={emailHost}
                  onChange={(e) => setEmailHost(e.target.value)}
                  placeholder="smtp.mailgun.org or smtp.gmail.com"
                  className="h-9 text-xs font-mono"
                />
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="email-port-input"
                  className="font-semibold text-foreground"
                >
                  Port:
                </label>
                <Input
                  id="email-port-input"
                  type="number"
                  value={emailPort}
                  onChange={(e) => setEmailPort(Number(e.target.value))}
                  className="h-9 text-xs font-mono"
                />
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="email-user-input"
                  className="font-semibold text-foreground"
                >
                  SMTP Username:
                </label>
                <Input
                  id="email-user-input"
                  value={emailUsername}
                  onChange={(e) => setEmailUsername(e.target.value)}
                  placeholder="user@institute.edu.bd"
                  className="h-9 text-xs font-mono"
                />
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="email-from-address"
                  className="font-semibold text-foreground"
                >
                  From Address:
                </label>
                <Input
                  id="email-from-address"
                  value={emailFromAddress}
                  onChange={(e) => setEmailFromAddress(e.target.value)}
                  placeholder="accounts@institute.edu.bd"
                  className="h-9 text-xs font-mono"
                />
              </div>

              <div className="space-y-1.5">
                <label
                  htmlFor="email-from-name"
                  className="font-semibold text-foreground"
                >
                  From Display Name:
                </label>
                <Input
                  id="email-from-name"
                  value={emailFromName}
                  onChange={(e) => setEmailFromName(e.target.value)}
                  placeholder="Coachify Academic Academy"
                  className="h-9 text-xs"
                />
              </div>

              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <label
                    htmlFor="email-pass-input"
                    className="font-semibold text-foreground"
                  >
                    SMTP Password:
                  </label>
                  {activeEmailConfig?.masked_secret && (
                    <span className="font-mono text-[10px] text-muted-foreground">
                      Current: {activeEmailConfig.masked_secret}
                    </span>
                  )}
                </div>
                <Input
                  id="email-pass-input"
                  type="password"
                  value={emailPassword}
                  onChange={(e) => setEmailPassword(e.target.value)}
                  placeholder={
                    activeEmailConfig?.secret_is_set
                      ? "Leave blank to keep current password"
                      : "Enter SMTP app password"
                  }
                  className="h-9 text-xs"
                />
              </div>
            </div>

            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveEmail}
              className="gap-2 font-semibold text-xs"
            >
              <KeyRound className="h-3.5 w-3.5" />
              Save SMTP Email Settings
            </Button>
          </Card>

          {/* Test Dispatcher Card */}
          <Card className="p-5 border-border/80 shadow-xs space-y-4">
            <CardTitle className="text-sm font-bold flex items-center gap-2">
              <Send className="h-4 w-4 text-primary" />
              Send Immediate Test Dispatch
            </CardTitle>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs">
              <div className="space-y-1.5">
                <label
                  htmlFor="test-channel-select"
                  className="font-semibold text-foreground"
                >
                  Test Channel:
                </label>
                <select
                  id="test-channel-select"
                  value={testChannel}
                  onChange={(e) =>
                    setTestChannel(e.target.value as "sms" | "whatsapp" | "email")
                  }
                  className="w-full h-9 px-3 rounded-md border border-input bg-background text-xs font-medium"
                >
                  <option value="sms">Cellular SMS</option>
                  <option value="whatsapp">WhatsApp</option>
                  <option value="email">Email</option>
                </select>
              </div>

              <div className="space-y-1.5 sm:col-span-2">
                <label
                  htmlFor="test-recipient-input"
                  className="font-semibold text-foreground"
                >
                  Test Destination Address:
                </label>
                <Input
                  id="test-recipient-input"
                  value={testRecipient}
                  onChange={(e) => setTestRecipient(e.target.value)}
                  placeholder={
                    testChannel === "email"
                      ? "admin@coaching.com"
                      : "017XXXXXXXX or 88017XXXXXXXX"
                  }
                  className="h-9 text-xs font-mono"
                />
              </div>
            </div>

            {testChannel === "email" && (
              <div className="space-y-1.5 text-xs">
                <label
                  htmlFor="test-subject-input"
                  className="font-semibold text-foreground"
                >
                  Email Subject:
                </label>
                <Input
                  id="test-subject-input"
                  value={testSubject}
                  onChange={(e) => setTestSubject(e.target.value)}
                  className="h-9 text-xs"
                />
              </div>
            )}

            <div className="space-y-1.5 text-xs">
              <label htmlFor="test-body-input" className="font-semibold text-foreground">
                Test Message Text:
              </label>
              <Input
                id="test-body-input"
                value={testBody}
                onChange={(e) => setTestBody(e.target.value)}
                className="h-9 text-xs font-mono"
              />
            </div>

            <div className="flex items-center gap-3">
              <Button
                variant="primary"
                size="sm"
                onClick={handleSendTestMessage}
                disabled={testSending || !testRecipient.trim()}
                className="gap-2 font-semibold text-xs"
              >
                <Send className="h-3.5 w-3.5" />
                {testSending ? "Transmitting..." : "Send Test Message"}
              </Button>
              {testResult && (
                <span className="text-xs font-medium text-foreground">{testResult}</span>
              )}
            </div>
          </Card>
        </div>
      )}

      {/* TAB 2: TEMPLATES CATALOG */}
      {activeTab === "templates" && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              Predefined system templates and custom message formats in English and
              Bengali.
            </span>
            <Button
              variant="primary"
              size="sm"
              onClick={() => handleOpenTemplate()}
              className="gap-1.5 text-xs font-semibold"
            >
              <Plus className="h-3.5 w-3.5" />
              Create Custom Template
            </Button>
          </div>

          <Card className="overflow-hidden border-border/80 shadow-xs">
            <div className="overflow-x-auto max-h-[560px]">
              <table className="w-full text-xs text-left border-collapse">
                <thead className="sticky top-0 bg-card border-b border-border/80 text-muted-foreground font-semibold text-[11px] tracking-wide uppercase z-10">
                  <tr>
                    <th className="py-2.5 px-3 w-16 text-center">Lang</th>
                    <th className="py-2.5 px-3 w-24">Channel</th>
                    <th className="py-2.5 px-3 w-40">Template Key</th>
                    <th className="py-2.5 px-3">Title & Body Preview</th>
                    <th className="py-2.5 px-3 w-28 text-center">System Protected</th>
                    <th className="py-2.5 px-3 w-24 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border/60">
                  {templates.map((tmpl) => (
                    <tr key={tmpl.id} className="hover:bg-muted/30 transition-colors">
                      <td className="py-2.5 px-3 text-center font-bold uppercase text-[10px] text-muted-foreground">
                        {tmpl.language}
                      </td>
                      <td className="py-2.5 px-3 uppercase font-semibold text-[11px]">
                        {tmpl.channel}
                      </td>
                      <td className="py-2.5 px-3 font-mono text-foreground font-medium">
                        {tmpl.key}
                      </td>
                      <td className="py-2.5 px-3 max-w-md">
                        <div className="font-semibold text-foreground">{tmpl.title}</div>
                        <div className="text-[11px] text-muted-foreground truncate font-mono">
                          {tmpl.body}
                        </div>
                      </td>
                      <td className="py-2.5 px-3 text-center">
                        {tmpl.is_system ? (
                          <Badge
                            variant="outline"
                            className="gap-1 text-[10px] text-emerald-600 dark:text-emerald-400 border-emerald-500/30"
                          >
                            <Lock className="h-2.5 w-2.5" />
                            System
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="text-[10px]">
                            Custom
                          </Badge>
                        )}
                      </td>
                      <td className="py-2.5 px-3 text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleOpenTemplate(tmpl)}
                            className="h-7 w-7 p-0"
                            title="Edit template"
                          >
                            <Edit2 className="h-3.5 w-3.5 text-muted-foreground" />
                          </Button>
                          {!tmpl.is_system && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => void handleDeleteTemplate(tmpl.id)}
                              className="h-7 w-7 p-0 text-rose-500 hover:text-rose-700"
                              title="Delete template"
                            >
                              <Trash2 className="h-3.5 w-3.5" />
                            </Button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Card>
        </div>
      )}

      {/* TAB 3: AUTOMATION RULES */}
      {activeTab === "automations" && (
        <div className="space-y-4">
          <Card className="overflow-hidden border-border/80 shadow-xs">
            <CardHeader className="py-3 px-4 bg-muted/20 border-b border-border/70">
              <CardTitle className="text-sm font-semibold flex items-center gap-2">
                <Zap className="h-4 w-4 text-primary" />
                Turnkey Event Automation Rules
              </CardTitle>
            </CardHeader>

            <div className="divide-y divide-border/60">
              {rules.map((rule) => {
                const isSaving = ruleSavingId === rule.id;

                return (
                  <div
                    key={rule.id}
                    className="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-4 hover:bg-muted/20 transition-colors"
                  >
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <span className="font-bold text-sm text-foreground capitalize">
                          {rule.trigger.replace(/_/g, " ")}
                        </span>
                        <Badge variant="outline" className="uppercase text-[10px]">
                          {rule.channel}
                        </Badge>
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Assigned Template: <strong>{rule.template_title}</strong>
                      </div>
                    </div>

                    <div className="flex items-center gap-3">
                      <span className="text-xs font-semibold text-muted-foreground">
                        {rule.enabled ? "Active" : "Disabled"}
                      </span>
                      <Button
                        variant={rule.enabled ? "primary" : "outline"}
                        size="sm"
                        onClick={() => void handleToggleRule(rule, !rule.enabled)}
                        disabled={isSaving}
                        className={cn(
                          "h-8 text-xs font-semibold",
                          rule.enabled
                            ? "bg-emerald-600 hover:bg-emerald-700 text-white"
                            : ""
                        )}
                      >
                        {rule.enabled ? "Enabled" : "Enable"}
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          </Card>
        </div>
      )}

      {/* Template Edit / Create Dialog */}
      <Dialog open={templateDialogOpen} onOpenChange={setTemplateDialogOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-base font-bold">
              <FileText className="h-4 w-4 text-primary" />
              {editingTemplate ? "Edit Message Template" : "New Message Template"}
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Define template text with named placeholders.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2 text-xs">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1.5">
                <label htmlFor="tmpl-key" className="font-semibold text-foreground">
                  Template Key:
                </label>
                <Input
                  id="tmpl-key"
                  value={tmplKey}
                  onChange={(e) => setTmplKey(e.target.value)}
                  disabled={Boolean(editingTemplate?.is_system)}
                  placeholder="e.g. payment_received"
                  className="h-8 text-xs font-mono"
                />
              </div>

              <div className="space-y-1.5">
                <label htmlFor="tmpl-channel" className="font-semibold text-foreground">
                  Channel:
                </label>
                <select
                  id="tmpl-channel"
                  value={tmplChannel}
                  onChange={(e) => setTmplChannel(e.target.value)}
                  disabled={Boolean(editingTemplate?.is_system)}
                  className="w-full h-8 px-2 rounded-md border border-input bg-background text-xs font-medium"
                >
                  <option value="sms">SMS</option>
                  <option value="whatsapp">WhatsApp</option>
                  <option value="email">Email</option>
                </select>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1.5">
                <label htmlFor="tmpl-lang" className="font-semibold text-foreground">
                  Language:
                </label>
                <select
                  id="tmpl-lang"
                  value={tmplLang}
                  onChange={(e) => setTmplLang(e.target.value)}
                  disabled={Boolean(editingTemplate?.is_system)}
                  className="w-full h-8 px-2 rounded-md border border-input bg-background text-xs font-medium"
                >
                  <option value="en">English (en)</option>
                  <option value="bn">Bengali (bn)</option>
                </select>
              </div>

              <div className="space-y-1.5">
                <label htmlFor="tmpl-title" className="font-semibold text-foreground">
                  Title:
                </label>
                <Input
                  id="tmpl-title"
                  value={tmplTitle}
                  onChange={(e) => setTmplTitle(e.target.value)}
                  placeholder="Display title"
                  className="h-8 text-xs"
                />
              </div>
            </div>

            {tmplChannel === "email" && (
              <div className="space-y-1.5">
                <label htmlFor="tmpl-subject" className="font-semibold text-foreground">
                  Email Subject Line:
                </label>
                <Input
                  id="tmpl-subject"
                  value={tmplSubject}
                  onChange={(e) => setTmplSubject(e.target.value)}
                  placeholder="Subject line with {placeholders}"
                  className="h-8 text-xs"
                />
              </div>
            )}

            <div className="space-y-1.5">
              <label htmlFor="tmpl-body" className="font-semibold text-foreground">
                Template Body:
              </label>
              <textarea
                id="tmpl-body"
                rows={5}
                value={tmplBody}
                onChange={(e) => setTmplBody(e.target.value)}
                placeholder="Template text with {placeholders}..."
                className="w-full p-2.5 rounded-md border border-input bg-background font-mono text-xs focus:outline-hidden focus:ring-1 focus:ring-ring leading-relaxed"
              />
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setTemplateDialogOpen(false)}
              className="text-xs"
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSaveTemplate}
              disabled={
                tmplSaving || !tmplKey.trim() || !tmplTitle.trim() || !tmplBody.trim()
              }
              className="text-xs font-semibold"
            >
              {tmplSaving ? "Saving..." : "Save Template"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
