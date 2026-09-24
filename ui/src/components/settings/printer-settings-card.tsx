import { useEffect, useState, useCallback } from "react";
import type {
  WorkstationPrinterSettings,
  SystemPrinterInfo,
  PrintableReceiptData,
  PrintTemplateConfig,
  PrintJobRun,
} from "../../gen/rpc";
import {
  getPrinterSettings,
  savePrinterSettings,
  listSystemPrinters,
  executeHtmlPrint,
  getPrintTemplateConfigs,
  updatePrintTemplateConfig,
  listPrintJobRuns,
} from "../../lib/printer";
import { generateThermalReceiptHtml } from "../print/thermal-receipt";
import { generateA4DualReceiptHtml } from "../print/a4-dual-receipt";
import { generateA5ReceiptHtml } from "../print/a5-receipt";
import { Card, CardHeader, CardTitle, CardContent } from "../ui/card";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Switch } from "../ui/switch";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import {
  Printer,
  RefreshCw,
  Check,
  AlertCircle,
  Sparkles,
  Sliders,
  History,
} from "lucide-react";
import { formatBusinessDate } from "@/lib/utils";

export function PrinterSettingsCard() {
  const [activeTab, setActiveTab] = useState<"workstation" | "templates" | "history">(
    "workstation"
  );
  const [settings, setSettings] = useState<WorkstationPrinterSettings | null>(null);
  const [printers, setPrinters] = useState<SystemPrinterInfo[]>([]);
  const [templateConfigs, setTemplateConfigs] = useState<PrintTemplateConfig[]>([]);
  const [selectedDocType, setSelectedDocType] = useState<string>("id_card");
  const [printJobs, setPrintJobs] = useState<PrintJobRun[]>([]);
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
      const [curSettings, systemPrinters, configs, jobsResult] = await Promise.all([
        getPrinterSettings(),
        listSystemPrinters(),
        getPrintTemplateConfigs().catch(() => []),
        listPrintJobRuns({ limit: 20 }).catch(() => ({ items: [], total_count: 0 })),
      ]);
      setSettings(curSettings);
      setPrinters(systemPrinters);
      setTemplateConfigs(configs);
      setPrintJobs(jobsResult.items);
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

  const handleSaveTemplateConfig = async (cfg: PrintTemplateConfig) => {
    setSaving(true);
    setStatusMessage(null);
    try {
      const updated = await updatePrintTemplateConfig({
        document_type: cfg.document_type,
        paper_size: cfg.paper_size,
        orientation: cfg.orientation,
        show_header: cfg.show_header,
        show_logo: cfg.show_logo,
        show_watermark: cfg.show_watermark,
        custom_header_text: cfg.custom_header_text,
        custom_footer_text: cfg.custom_footer_text,
        signature_title: cfg.signature_title,
        options_json: cfg.options_json,
      });
      setTemplateConfigs((prev) =>
        prev.map((c) => (c.document_type === updated.document_type ? updated : c))
      );
      setStatusMessage({
        type: "success",
        text: `Saved template for ${cfg.document_type}.`,
      });
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
          period: "2026-05",
          amount_poisha: 250000,
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

    try {
      const res = await executeHtmlPrint(html);
      if (res.success) {
        setStatusMessage({ type: "success", text: "Test voucher sent to printer." });
      } else {
        setStatusMessage({
          type: "error",
          text: res.error || "Failed to trigger print spooler",
        });
      }
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setStatusMessage({ type: "error", text: message });
    } finally {
      setTestPrinting(false);
      setTimeout(() => setStatusMessage(null), 5000);
    }
  };

  if (loading) {
    return (
      <Card className="shadow-sm border-border bg-card">
        <CardHeader className="pb-3 border-b border-border">
          <CardTitle className="text-base font-bold flex items-center gap-2">
            <Printer className="h-5 w-5 text-primary" />
            Printing & Hardware Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-6 pb-6 text-center text-sm text-muted-foreground">
          <RefreshCw className="h-5 w-5 animate-spin mx-auto mb-2 text-primary" />
          Detecting system printers...
        </CardContent>
      </Card>
    );
  }

  if (!settings) {
    return (
      <Card className="shadow-sm border-border bg-card">
        <CardContent className="pt-6 pb-6 text-center text-sm text-destructive">
          Failed to load workstation printer settings.
        </CardContent>
      </Card>
    );
  }

  const activeTemplate = templateConfigs.find((c) => c.document_type === selectedDocType);

  return (
    <Card className="shadow-sm border-border bg-card">
      <CardHeader className="pb-3 border-b border-border flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <Printer className="h-5 w-5 text-primary" />
          <div>
            <CardTitle className="text-base font-bold">
              Printing & Template Management
            </CardTitle>
            <p className="text-xs text-muted-foreground">
              Configure workstation printers, document templates, and view print run audit
              history.
            </p>
          </div>
        </div>

        <Tabs
          value={activeTab}
          onValueChange={(v) => setActiveTab(v as typeof activeTab)}
        >
          <TabsList className="h-8">
            <TabsTrigger value="workstation" className="text-xs gap-1.5 h-7">
              <Printer className="h-3.5 w-3.5" />
              Workstation
            </TabsTrigger>
            <TabsTrigger value="templates" className="text-xs gap-1.5 h-7">
              <Sliders className="h-3.5 w-3.5" />
              Templates ({templateConfigs.length})
            </TabsTrigger>
            <TabsTrigger value="history" className="text-xs gap-1.5 h-7">
              <History className="h-3.5 w-3.5" />
              Job Runs
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </CardHeader>

      <CardContent className="pt-4 space-y-6">
        {/* Workstation Hardware Settings Tab */}
        {activeTab === "workstation" && (
          <div className="space-y-6">
            <div className="grid md:grid-cols-2 gap-6">
              {/* Selected Printer */}
              <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-semibold text-foreground">
                    Target Receipt Printer
                  </label>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="h-6 px-1.5 text-xs text-muted-foreground hover:text-foreground"
                    onClick={() => void loadData()}
                  >
                    <RefreshCw className="h-3 w-3 mr-1" />
                    Rescan
                  </Button>
                </div>
                <select
                  value={settings.printer_name || ""}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      printer_name: e.target.value === "" ? null : e.target.value,
                    })
                  }
                  className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  <option value="">(System Default Printer)</option>
                  {printers.map((p) => (
                    <option key={p.name} value={p.name}>
                      {p.name} {p.is_default ? "★ Default" : ""}
                    </option>
                  ))}
                </select>
                <p className="text-[11px] text-muted-foreground">
                  Select a dedicated thermal or document printer for this workstation.
                </p>
              </div>

              {/* Default Receipt Paper Size */}
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  Receipt Paper Format
                </label>
                <select
                  value={settings.page_size}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      page_size: e.target.value as
                        "thermal_80mm" | "a4_dual" | "a5_single",
                    })
                  }
                  className="w-full h-9 rounded-md border border-input bg-background px-3 py-1 text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  <option value="thermal_80mm">
                    80mm Continuous Thermal Roll (Default)
                  </option>
                  <option value="a4_dual">
                    A4 Dual Copy (Student Copy + Office Copy)
                  </option>
                  <option value="a5_single">A5 Single Copy (Half Page Voucher)</option>
                </select>
                <p className="text-[11px] text-muted-foreground">
                  Format applied when printing fee payment receipts.
                </p>
              </div>
            </div>

            {/* Automation Switches */}
            <div className="grid md:grid-cols-2 gap-6 pt-2 border-t border-border">
              <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
                <div className="space-y-0.5 pr-2">
                  <div className="text-xs font-semibold text-foreground">
                    Auto-Print on Payment
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    Instantly trigger silent background printing when collecting fees.
                  </div>
                </div>
                <Switch
                  checked={settings.auto_print_on_payment}
                  onCheckedChange={(checked) =>
                    setSettings({ ...settings, auto_print_on_payment: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg border border-border bg-muted/20">
                <div className="space-y-0.5 pr-2">
                  <div className="text-xs font-semibold text-foreground">
                    Kick Cash Drawer (ESC/POS)
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    Send pulse signal to RJ11/RJ12 drawer port on thermal receipt print.
                  </div>
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
          </div>
        )}

        {/* Document Templates Management Tab (AC-5, AC-6) */}
        {activeTab === "templates" && (
          <div className="space-y-6">
            <div className="flex items-center gap-3">
              <label className="text-xs font-semibold text-foreground whitespace-nowrap">
                Select Document:
              </label>
              <select
                value={selectedDocType}
                onChange={(e) => setSelectedDocType(e.target.value)}
                className="h-9 rounded-md border border-input bg-background px-3 py-1 text-xs shadow-sm font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                {templateConfigs.map((cfg) => (
                  <option key={cfg.document_type} value={cfg.document_type}>
                    {cfg.document_type.replace(/_/g, " ").toUpperCase()} (
                    {cfg.paper_size.toUpperCase()})
                  </option>
                ))}
              </select>
            </div>

            {activeTemplate && (
              <div className="border border-border rounded-lg p-4 bg-muted/10 space-y-4">
                <div className="flex justify-between items-center border-b border-border pb-3">
                  <div>
                    <h4 className="text-sm font-bold capitalize">
                      {activeTemplate.document_type.replace(/_/g, " ")} Template
                    </h4>
                    <p className="text-xs text-muted-foreground">
                      Last updated: {formatBusinessDate(activeTemplate.updated_at)}
                    </p>
                  </div>
                  <Button
                    size="sm"
                    onClick={() => handleSaveTemplateConfig(activeTemplate)}
                    disabled={saving}
                    className="h-8 gap-1.5 text-xs font-semibold"
                  >
                    <Check className="h-3.5 w-3.5" />
                    Save Template
                  </Button>
                </div>

                <div className="grid md:grid-cols-2 gap-4">
                  {/* Paper Size */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold">Paper Size</label>
                    <select
                      value={activeTemplate.paper_size}
                      onChange={(e) => {
                        const val = e.target.value;
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, paper_size: val }
                              : c
                          )
                        );
                      }}
                      className="w-full h-8 rounded-md border border-input bg-background px-2.5 text-xs"
                    >
                      <option value="a4">A4 Standard Sheet</option>
                      <option value="a5">A5 Half Page</option>
                      <option value="thermal_80mm">80mm Continuous Thermal Roll</option>
                      <option value="id_card_pvc">CR80 Plastic ID Card</option>
                    </select>
                  </div>

                  {/* Orientation */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold">Orientation</label>
                    <select
                      value={activeTemplate.orientation}
                      onChange={(e) => {
                        const val = e.target.value;
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, orientation: val }
                              : c
                          )
                        );
                      }}
                      className="w-full h-8 rounded-md border border-input bg-background px-2.5 text-xs"
                    >
                      <option value="portrait">Portrait</option>
                      <option value="landscape">Landscape</option>
                    </select>
                  </div>
                </div>

                {/* Toggles */}
                <div className="grid grid-cols-3 gap-4 pt-2 border-t border-border">
                  <div className="flex items-center justify-between p-2.5 rounded border border-border bg-background">
                    <span className="text-xs font-medium">Show Header</span>
                    <Switch
                      checked={activeTemplate.show_header}
                      onCheckedChange={(checked) =>
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, show_header: checked }
                              : c
                          )
                        )
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between p-2.5 rounded border border-border bg-background">
                    <span className="text-xs font-medium">Show Logo</span>
                    <Switch
                      checked={activeTemplate.show_logo}
                      onCheckedChange={(checked) =>
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, show_logo: checked }
                              : c
                          )
                        )
                      }
                    />
                  </div>
                  <div className="flex items-center justify-between p-2.5 rounded border border-border bg-background">
                    <span className="text-xs font-medium">Watermark</span>
                    <Switch
                      checked={activeTemplate.show_watermark}
                      onCheckedChange={(checked) =>
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, show_watermark: checked }
                              : c
                          )
                        )
                      }
                    />
                  </div>
                </div>

                {/* Signature Title & Notes */}
                <div className="grid md:grid-cols-2 gap-4 pt-2 border-t border-border">
                  <div className="space-y-1">
                    <label className="text-xs font-semibold">Signature Title</label>
                    <Input
                      value={activeTemplate.signature_title || ""}
                      onChange={(e) => {
                        const val = e.target.value;
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, signature_title: val || null }
                              : c
                          )
                        );
                      }}
                      placeholder="e.g. Authorized Signatory, Headmaster"
                      className="h-8 text-xs"
                    />
                  </div>
                  <div className="space-y-1">
                    <label className="text-xs font-semibold">Custom Footer Notice</label>
                    <Input
                      value={activeTemplate.custom_footer_text || ""}
                      onChange={(e) => {
                        const val = e.target.value;
                        setTemplateConfigs((prev) =>
                          prev.map((c) =>
                            c.document_type === activeTemplate.document_type
                              ? { ...c, custom_footer_text: val || null }
                              : c
                          )
                        );
                      }}
                      placeholder="Terms, property notices, or helpline"
                      className="h-8 text-xs"
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Print Job Runs Audit History Tab (AC-7) */}
        {activeTab === "history" && (
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-xs text-muted-foreground">
                Showing past print execution audit logs.
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => void loadData()}
                className="h-7 text-xs gap-1"
              >
                <RefreshCw className="h-3 w-3" /> Refresh Logs
              </Button>
            </div>

            <div className="border border-border rounded-lg overflow-hidden">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="bg-muted/50 border-b border-border font-semibold text-muted-foreground">
                    <th className="p-2.5">Date & Time</th>
                    <th className="p-2.5">Document Type</th>
                    <th className="p-2.5 text-center">Batch Mode</th>
                    <th className="p-2.5 text-right">Items</th>
                    <th className="p-2.5">Paper</th>
                    <th className="p-2.5">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {printJobs.length > 0 ? (
                    printJobs.map((job) => (
                      <tr key={job.id} className="hover:bg-muted/30">
                        <td className="p-2.5 font-mono text-[11px] text-muted-foreground">
                          {formatBusinessDate(job.created_at)}
                        </td>
                        <td className="p-2.5 font-semibold capitalize">
                          {job.document_type.replace(/_/g, " ")}
                        </td>
                        <td className="p-2.5 text-center">
                          {job.batch_mode ? (
                            <span className="px-1.5 py-0.5 bg-sky-500/10 text-sky-600 dark:text-sky-400 rounded text-[10px] font-bold">
                              BATCH
                            </span>
                          ) : (
                            <span className="text-muted-foreground text-[10px]">
                              SINGLE
                            </span>
                          )}
                        </td>
                        <td className="p-2.5 text-right font-mono font-bold">
                          {job.item_count}
                        </td>
                        <td className="p-2.5 text-muted-foreground uppercase text-[11px]">
                          {job.paper_size}
                        </td>
                        <td className="p-2.5">
                          <span
                            className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                              job.status === "completed"
                                ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
                                : "bg-red-500/10 text-red-600 dark:text-red-400"
                            }`}
                          >
                            {job.status}
                          </span>
                        </td>
                      </tr>
                    ))
                  ) : (
                    <tr>
                      <td colSpan={6} className="p-6 text-center text-muted-foreground">
                        No print job runs recorded yet.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
