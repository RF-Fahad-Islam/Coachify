import { useEffect, useState, useCallback } from "react";
import type {
  WorkstationPrinterSettings,
  SystemPrinterInfo,
  PrintableReceiptData,
} from "../../gen/rpc";
import {
  getPrinterSettings,
  savePrinterSettings,
  listSystemPrinters,
  executeHtmlPrint,
} from "../../lib/printer";
import { generateThermalReceiptHtml } from "../print/thermal-receipt";
import { generateA4DualReceiptHtml } from "../print/a4-dual-receipt";
import { generateA5ReceiptHtml } from "../print/a5-receipt";
import { Card, CardHeader, CardTitle, CardContent } from "../ui/card";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Switch } from "../ui/switch";
import { Printer, RefreshCw, Check, AlertCircle, Sparkles } from "lucide-react";

export function PrinterSettingsCard() {
  const [settings, setSettings] = useState<WorkstationPrinterSettings | null>(null);
  const [printers, setPrinters] = useState<SystemPrinterInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);
  const [testPrinting, setTestPrinting] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [curSettings, systemPrinters] = await Promise.all([
        getPrinterSettings(),
        listSystemPrinters(),
      ]);
      setSettings(curSettings);
      setPrinters(systemPrinters);
    } catch (err: unknown) {
      console.error("Failed to load printer settings:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const handleSave = async () => {
    if (!settings) return;
    setSaving(true);
    setStatusMessage(null);
    try {
      const updated = await savePrinterSettings(settings);
      setSettings(updated);
      setStatusMessage({ type: "success", text: "Printer settings saved successfully." });
      setTimeout(() => setStatusMessage(null), 4000);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setStatusMessage({ type: "error", text: message });
    } finally {
      setSaving(false);
    }
  };

  const handleTestPrint = async () => {
    if (!settings) return;
    setTestPrinting(true);

    const mockReceipt: PrintableReceiptData = {
      receipt_no: "REC-TEST-0001",
      business_date: new Date().toISOString().slice(0, 10),
      received_at: new Date().toISOString(),
      student_id: "test-student-id",
      student_code: "STU-TEST-001",
      student_name: "Test Student (Sample)",
      student_name_bn: "নমুনা শিক্ষার্থী",
      guardian_name: "Test Guardian",
      guardian_phone: "01700000000",
      batch_name: "Physics Model Batch",
      program_name: "HSC 2026",
      roll_no: "01",
      amount_poisha: 250000,
      amount_in_words_en: "Two Thousand Five Hundred Taka Only",
      amount_in_words_bn: "দুই হাজার পাঁচ শত টাকা মাত্র",
      method: "cash",
      reference: null,
      collector_name: "Counter Cashier",
      institution_name: "Apex Coaching Centre",
      institution_name_bn: "এপেক্স কোচিং সেন্টার",
      institution_address: "Farmgate, Dhaka-1215",
      institution_phone: "01712345678",
      institution_email: "info@apexcoaching.edu.bd",
      institution_website: "www.apexcoaching.edu.bd",
      balance_after_poisha: 0,
      items: [
        {
          fee_head_name: "Monthly Tuition Fee",
          fee_head_name_bn: "মাসিক বেতন",
          period: "2026-04",
          amount_poisha: 200000,
        },
        {
          fee_head_name: "Exam & Model Test Fee",
          fee_head_name_bn: "মডেল টেস্ট ফি",
          period: null,
          amount_poisha: 50000,
        },
      ],
      is_reprint: false,
    };

    let html = "";
    if (settings.page_size === "thermal_80mm") {
      html = generateThermalReceiptHtml(mockReceipt, settings.footer_note);
    } else if (settings.page_size === "a4_dual") {
      html = generateA4DualReceiptHtml(mockReceipt, settings.footer_note);
    } else {
      html = generateA5ReceiptHtml(mockReceipt, settings.footer_note);
    }

    const res = await executeHtmlPrint(html);
    setTestPrinting(false);
    if (res.success) {
      setStatusMessage({ type: "success", text: "Test receipt sent to print queue." });
      setTimeout(() => setStatusMessage(null), 4000);
    } else {
      setStatusMessage({ type: "error", text: res.error || "Test print failed." });
    }
  };

  if (loading || !settings) {
    return (
      <Card>
        <CardContent className="p-8 flex items-center justify-center">
          <div className="flex flex-col items-center gap-2">
            <RefreshCw className="h-6 w-6 animate-spin text-primary" />
            <span className="text-xs text-muted-foreground">
              Detecting system printers...
            </span>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="border-b border-border pb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Printer className="h-5 w-5 text-primary" />
            <div>
              <CardTitle className="text-base font-bold text-foreground">
                Workstation Printer & Receipt Configuration
              </CardTitle>
              <p className="text-xs text-muted-foreground mt-0.5">
                Saved locally on this PC (%ProgramData%\Coachify). Each counter computer
                can have its own dedicated receipt printer.
              </p>
            </div>
          </div>

          <Button
            variant="outline"
            size="sm"
            onClick={loadData}
            disabled={loading}
            className="gap-1.5 text-xs"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh Printers
          </Button>
        </div>
      </CardHeader>

      <CardContent className="space-y-6 pt-6 text-sm">
        {/* Printer Selection */}
        <div className="grid md:grid-cols-2 gap-6">
          <div className="space-y-2">
            <label className="text-xs font-semibold text-foreground">
              Target Printer
            </label>
            <select
              value={settings.printer_name || ""}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  printer_name: e.target.value ? e.target.value : null,
                })
              }
              className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <option value="">(Windows System Default Printer)</option>
              {printers.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name} {p.is_default ? "★ (Default)" : ""}
                </option>
              ))}
            </select>
            <p className="text-[11px] text-muted-foreground">
              {printers.length > 0
                ? `Detected ${printers.length} installed printers on this workstation.`
                : "No physical printers detected; printing will use default system print spooler."}
            </p>
          </div>

          <div className="space-y-2">
            <label className="text-xs font-semibold text-foreground">
              Receipt Paper Layout
            </label>
            <div className="grid grid-cols-3 gap-2">
              <button
                type="button"
                onClick={() => setSettings({ ...settings, page_size: "thermal_80mm" })}
                className={`p-2.5 rounded-md border text-left flex flex-col justify-between transition-colors ${
                  settings.page_size === "thermal_80mm"
                    ? "border-primary bg-primary/10 text-foreground font-semibold"
                    : "border-border hover:bg-muted/50 text-muted-foreground"
                }`}
              >
                <div className="text-xs">80mm Thermal</div>
                <div className="text-[10px] text-muted-foreground mt-1">
                  Continuous POS Roll
                </div>
              </button>

              <button
                type="button"
                onClick={() => setSettings({ ...settings, page_size: "a4_dual" })}
                className={`p-2.5 rounded-md border text-left flex flex-col justify-between transition-colors ${
                  settings.page_size === "a4_dual"
                    ? "border-primary bg-primary/10 text-foreground font-semibold"
                    : "border-border hover:bg-muted/50 text-muted-foreground"
                }`}
              >
                <div className="text-xs">A4 Dual Copy</div>
                <div className="text-[10px] text-muted-foreground mt-1">
                  Student & Office Copies
                </div>
              </button>

              <button
                type="button"
                onClick={() => setSettings({ ...settings, page_size: "a5_single" })}
                className={`p-2.5 rounded-md border text-left flex flex-col justify-between transition-colors ${
                  settings.page_size === "a5_single"
                    ? "border-primary bg-primary/10 text-foreground font-semibold"
                    : "border-border hover:bg-muted/50 text-muted-foreground"
                }`}
              >
                <div className="text-xs">A5 Single Copy</div>
                <div className="text-[10px] text-muted-foreground mt-1">
                  Half Sheet Laser Voucher
                </div>
              </button>
            </div>
          </div>
        </div>

        {/* Toggles */}
        <div className="grid md:grid-cols-2 gap-6 pt-2 border-t border-border">
          <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
            <div>
              <div className="font-semibold text-xs text-foreground">
                Auto Print on Payment
              </div>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Automatically triggers printing immediately when Receive Payment is
                clicked.
              </p>
            </div>
            <Switch
              checked={settings.auto_print_on_payment}
              onCheckedChange={(checked) =>
                setSettings({ ...settings, auto_print_on_payment: checked })
              }
            />
          </div>

          <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
            <div>
              <div className="font-semibold text-xs text-foreground">
                Open Cash Drawer Kick
              </div>
              <p className="text-[11px] text-muted-foreground mt-0.5">
                Sends pulse to thermal POS printer drawer port upon payment confirmation.
              </p>
            </div>
            <Switch
              checked={settings.open_cash_drawer}
              onCheckedChange={(checked) =>
                setSettings({ ...settings, open_cash_drawer: checked })
              }
            />
          </div>
        </div>

        {/* Receipt Header & Footer Text */}
        <div className="grid md:grid-cols-2 gap-6 pt-2 border-t border-border">
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Custom Header Title (Optional)
            </label>
            <Input
              value={settings.header_title || ""}
              onChange={(e) =>
                setSettings({ ...settings, header_title: e.target.value || null })
              }
              placeholder="Defaults to institution name"
              className="text-xs h-9"
            />
            <p className="text-[11px] text-muted-foreground">
              Overrides default institution name printed at top of receipt if set.
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-foreground">
              Receipt Footer Note
            </label>
            <Input
              value={settings.footer_note || ""}
              onChange={(e) =>
                setSettings({ ...settings, footer_note: e.target.value || null })
              }
              placeholder="e.g. Fees once paid are non-refundable. Keep receipt safe."
              className="text-xs h-9"
            />
            <p className="text-[11px] text-muted-foreground">
              Printed at the bottom of customer receipts.
            </p>
          </div>
        </div>

        {/* Action Bar */}
        <div className="pt-4 border-t border-border flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleTestPrint}
              disabled={testPrinting}
              className="gap-1.5 text-xs font-medium"
            >
              <Sparkles className="h-3.5 w-3.5 text-amber-500" />
              {testPrinting ? "Sending Test..." : "Print Test Voucher"}
            </Button>

            {statusMessage && (
              <div
                className={`flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded ${
                  statusMessage.type === "success"
                    ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                    : "bg-red-500/10 text-red-600 dark:text-red-400"
                }`}
              >
                {statusMessage.type === "success" ? (
                  <Check className="h-3.5 w-3.5" />
                ) : (
                  <AlertCircle className="h-3.5 w-3.5" />
                )}
                <span>{statusMessage.text}</span>
              </div>
            )}
          </div>

          <Button
            type="button"
            size="sm"
            onClick={handleSave}
            disabled={saving}
            className="gap-1.5 font-semibold text-xs"
          >
            <Check className="h-3.5 w-3.5" />
            {saving ? "Saving..." : "Save Workstation Settings"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
