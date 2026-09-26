import { useState, useEffect, useCallback } from "react";
import { Laptop, Printer, Activity, LogOut, RefreshCw, CheckCircle2 } from "lucide-react";
import {
  rpc,
  type WorkstationClientStatus,
  type SystemPrinterInfo,
  type WorkstationPrinterSettings,
} from "@/gen/rpc";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useLocale } from "@/lib/i18n";

export interface WorkstationSettingsCardProps {
  initialStatus?: WorkstationClientStatus | null;
  initialPrinters?: SystemPrinterInfo[];
  initialPrinterSettings?: WorkstationPrinterSettings | null;
}

export function WorkstationSettingsCard({
  initialStatus = null,
  initialPrinters = [],
  initialPrinterSettings = null,
}: WorkstationSettingsCardProps = {}) {
  const { locale } = useLocale();
  const [status, setStatus] = useState<WorkstationClientStatus | null>(initialStatus);
  const [printers, setPrinters] = useState<SystemPrinterInfo[]>(initialPrinters);
  const [printerSettings, setPrinterSettings] =
    useState<WorkstationPrinterSettings | null>(initialPrinterSettings);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [savingPrinters, setSavingPrinters] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [unpairing, setUnpairing] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const [statusRes, printRes, settingsRes] = await Promise.all([
        rpc("lan.get_client_status", {}),
        rpc("printers.list", {}),
        rpc("printers.get_settings", {}),
      ]);

      if (statusRes.ok && statusRes.data) {
        setStatus(statusRes.data);
      }
      if (printRes.ok && printRes.data) {
        setPrinters(printRes.data);
      }
      if (settingsRes.ok && settingsRes.data) {
        setPrinterSettings(settingsRes.data);
      }
    } catch {
      // Ignore
    }
  }, []);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  if (!status || !status.is_client_mode) {
    return null;
  }

  const handleTestLatency = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const res = await rpc("lan.test_connection", {});
      if (res.ok && res.data) {
        if (res.data.success) {
          setTestResult(
            locale === "bn"
              ? `হোস্ট সক্রিয় ও পৌঁছানো সম্ভব (${res.data.latency_ms ?? 0} ms)`
              : `Host reachable (${res.data.latency_ms ?? 0} ms)`
          );
        } else {
          setTestResult(
            res.data.message || (locale === "bn" ? "সংযোগ ব্যর্থ" : "Connection failed")
          );
        }
      } else {
        setTestResult(res.error?.message || (locale === "bn" ? "ত্রুটি" : "Error"));
      }
    } catch {
      setTestResult(locale === "bn" ? "হোস্টে পৌঁছানো যায়নি" : "Host unreachable");
    } finally {
      setTesting(false);
      void loadData();
    }
  };

  const handleSavePrinterSettings = async () => {
    if (!printerSettings) return;
    setSavingPrinters(true);
    setSaveSuccess(false);
    try {
      const res = await rpc("printers.save_settings", { settings: printerSettings });
      if (res.ok) {
        setSaveSuccess(true);
        setTimeout(() => setSaveSuccess(false), 2000);
      }
    } catch {
      // Ignore
    } finally {
      setSavingPrinters(false);
    }
  };

  const handleUnpairWorkstation = async () => {
    const confirmed = window.confirm(
      locale === "bn"
        ? "আপনি কি নিশ্চিতভাবে এই ওয়ার্কস্টেশনটি আনপেয়ার করতে চান? এই কম্পিউটারটি স্ট্যান্ডঅ্যালোন মোডে ফিরে যাবে।"
        : "Are you sure you want to unpair this workstation? It will disconnect from the host and return to standalone mode."
    );
    if (!confirmed) return;

    setUnpairing(true);
    try {
      await rpc("lan.disconnect_workstation", {});
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    } catch {
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    } finally {
      setUnpairing(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Workstation Identity Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Laptop className="h-5 w-5 text-primary" />
              <div>
                <CardTitle className="text-base">
                  {locale === "bn"
                    ? "ওয়ার্কস্টেশন সংযোগ বিবরণ"
                    : "Workstation Client Mode"}
                </CardTitle>
                <CardDescription className="text-xs">
                  {locale === "bn"
                    ? "এই কম্পিউটারটি একটি ক্যাশিয়ার/ফ্রন্ট-ডেস্ক ওয়ার্কস্টেশন হিসেবে হোস্টের সাথে সংযুক্ত রয়েছে।"
                    : "This PC operates as a secondary workstation client connected to the primary host."}
                </CardDescription>
              </div>
            </div>
            <span className="text-xs font-semibold px-2.5 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800">
              {status.state}
            </span>
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
            <div className="p-3 bg-muted/50 rounded-lg space-y-1">
              <div className="text-muted-foreground">
                {locale === "bn" ? "ওয়ার্কস্টেশনের নাম" : "Workstation Name"}
              </div>
              <div className="font-semibold text-sm">
                {status.workstation_name || "Front Desk PC"}
              </div>
            </div>

            <div className="p-3 bg-muted/50 rounded-lg space-y-1">
              <div className="text-muted-foreground">
                {locale === "bn" ? "হোস্ট সার্ভার" : "Primary Host Server"}
              </div>
              <div className="font-semibold text-sm">
                {status.host_name || "Primary Host"} ({status.host_ip}:{status.host_port})
              </div>
            </div>

            <div className="p-3 bg-muted/50 rounded-lg space-y-1">
              <div className="text-muted-foreground">
                {locale === "bn" ? "নেটওয়ার্ক লেটেন্সি (RTT)" : "Network Latency (RTT)"}
              </div>
              <div className="font-semibold font-mono text-sm">
                {status.rtt_ms != null ? `${status.rtt_ms} ms` : "—"}
              </div>
            </div>

            <div className="p-3 bg-muted/50 rounded-lg space-y-1">
              <div className="text-muted-foreground">
                {locale === "bn" ? "ডিভাইস হার্ডওয়্যার আইডি" : "Hardware Device ID"}
              </div>
              <div className="font-mono text-[11px] truncate">
                {status.device_id || "—"}
              </div>
            </div>
          </div>

          {testResult && (
            <div className="p-2.5 rounded-lg bg-muted text-xs flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-primary shrink-0" />
              <span>{testResult}</span>
            </div>
          )}

          <div className="flex gap-2 pt-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleTestLatency}
              disabled={testing}
              className="text-xs"
            >
              {testing ? (
                <RefreshCw className="h-3.5 w-3.5 animate-spin mr-1.5" />
              ) : (
                <Activity className="h-3.5 w-3.5 mr-1.5" />
              )}
              {locale === "bn" ? "লেটেন্সি পরীক্ষা করুন" : "Ping & Test Host"}
            </Button>

            <Button
              variant="destructive"
              size="sm"
              onClick={handleUnpairWorkstation}
              disabled={unpairing}
              className="text-xs ml-auto gap-1.5"
            >
              <LogOut className="h-3.5 w-3.5" />
              {locale === "bn" ? "ওয়ার্কস্টেশন আনপেয়ার করুন" : "Unpair Workstation"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Strictly Local Printing Settings */}
      {printerSettings && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Printer className="h-5 w-5 text-primary" />
              <div>
                <CardTitle className="text-base">
                  {locale === "bn"
                    ? "স্থানীয় প্রিন্টার কনফিগারেশন"
                    : "Local Workstation Printer Setup"}
                </CardTitle>
                <CardDescription className="text-xs">
                  {locale === "bn"
                    ? "রসিদ ও নথিপত্র প্রিন্টিং সরাসরি এই মেশিনের সাথে সংযুক্ত প্রিন্টারে হবে। হোস্টের প্রিন্টার ব্যবহৃত হবে না।"
                    : "Receipts and documents are routed strictly to printers physically connected to this PC."}
                </CardDescription>
              </div>
            </div>
          </CardHeader>

          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
              <div className="space-y-1.5">
                <label className="font-semibold">
                  {locale === "bn"
                    ? "ডিফল্ট রসিদ প্রিন্টার (থার্মাল / স্লিপ)"
                    : "Default Receipt Printer (POS Thermal)"}
                </label>
                <select
                  value={printerSettings.printer_name || ""}
                  onChange={(e) =>
                    setPrinterSettings({
                      ...printerSettings,
                      printer_name: e.target.value || null,
                    })
                  }
                  className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs"
                >
                  <option value="">
                    {locale === "bn"
                      ? "-- কোনো প্রিন্টার নির্বাচিত নেই --"
                      : "-- No Printer Selected --"}
                  </option>
                  {printers.map((p, i) => (
                    <option key={i} value={p.name}>
                      {p.name} {p.is_default ? "(Default)" : ""}
                    </option>
                  ))}
                </select>
              </div>

              <div className="space-y-1.5">
                <label className="font-semibold">
                  {locale === "bn" ? "রসিদের কাগজের আকার" : "Receipt Paper Format"}
                </label>
                <select
                  value={printerSettings.page_size}
                  onChange={(e) =>
                    setPrinterSettings({
                      ...printerSettings,
                      page_size: e.target.value as
                        "thermal_80mm" | "a4_dual" | "a5_single",
                    })
                  }
                  className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs"
                >
                  <option value="thermal_80mm">POS Thermal 80mm</option>
                  <option value="a4_dual">A4 Dual Copy (Office + Student)</option>
                  <option value="a5_single">A5 Single Copy</option>
                </select>
              </div>
            </div>

            <div className="space-y-2 pt-1 text-xs">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={printerSettings.auto_print_on_payment}
                  onChange={(e) =>
                    setPrinterSettings({
                      ...printerSettings,
                      auto_print_on_payment: e.target.checked,
                    })
                  }
                  className="rounded border-input text-primary h-4 w-4"
                />
                <span>
                  {locale === "bn"
                    ? "পেমেন্ট সম্পন্ন হওয়ার সাথে সাথে স্বয়ংক্রিয়ভাবে রসিদ প্রিন্ট করুন"
                    : "Automatically print physical receipt upon payment collection"}
                </span>
              </label>

              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={printerSettings.open_cash_drawer}
                  onChange={(e) =>
                    setPrinterSettings({
                      ...printerSettings,
                      open_cash_drawer: e.target.checked,
                    })
                  }
                  className="rounded border-input text-primary h-4 w-4"
                />
                <span>
                  {locale === "bn"
                    ? "রসিদ প্রিন্ট করার সময় ক্যাশ ড্রয়ার খুলুন (RJ11/POS-80 ড্রয়ার পালস)"
                    : "Pulse kick drawer on receipt print (RJ11 cash drawer trigger)"}
                </span>
              </label>
            </div>

            {saveSuccess && (
              <div className="p-2.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-300 text-xs flex items-center gap-2 font-medium">
                <CheckCircle2 className="h-4 w-4 text-emerald-600" />
                <span>
                  {locale === "bn"
                    ? "স্থানীয় প্রিন্টার সেটিংস সংরক্ষিত হয়েছে!"
                    : "Local workstation printer settings saved!"}
                </span>
              </div>
            )}

            <div className="pt-2">
              <Button
                size="sm"
                onClick={handleSavePrinterSettings}
                disabled={savingPrinters}
                className="text-xs font-semibold"
              >
                {savingPrinters && (
                  <RefreshCw className="h-3.5 w-3.5 animate-spin mr-1.5" />
                )}
                {locale === "bn"
                  ? "প্রিন্টার সেটিংস সংরক্ষণ করুন"
                  : "Save Printer Settings"}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
