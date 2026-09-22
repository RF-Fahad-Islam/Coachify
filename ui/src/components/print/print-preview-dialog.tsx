import { useState } from "react";
import type { PrintableReceiptData } from "../../gen/rpc";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { Button } from "../ui/button";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import { ThermalReceipt, generateThermalReceiptHtml } from "./thermal-receipt";
import { A4DualReceipt, generateA4DualReceiptHtml } from "./a4-dual-receipt";
import { A5Receipt, generateA5ReceiptHtml } from "./a5-receipt";
import { executeHtmlPrint } from "../../lib/printer";
import { Printer, ZoomIn, ZoomOut, Check, AlertCircle } from "lucide-react";

interface PrintPreviewDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  receipt: PrintableReceiptData | null;
  defaultLayout?: "thermal_80mm" | "a4_dual" | "a5_single";
  footerNote?: string | null;
}

export function PrintPreviewDialog({
  open,
  onOpenChange,
  receipt,
  defaultLayout = "thermal_80mm",
  footerNote,
}: PrintPreviewDialogProps) {
  const [layout, setLayout] = useState<"thermal_80mm" | "a4_dual" | "a5_single">(
    defaultLayout
  );
  const [zoom, setZoom] = useState<number>(100);
  const [printing, setPrinting] = useState<boolean>(false);
  const [printStatus, setPrintStatus] = useState<"idle" | "success" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");

  if (!receipt) return null;

  const handlePrint = async () => {
    setPrinting(true);
    setPrintStatus("idle");
    setErrorMessage("");

    let html = "";
    if (layout === "thermal_80mm") {
      html = generateThermalReceiptHtml(receipt, footerNote);
    } else if (layout === "a4_dual") {
      html = generateA4DualReceiptHtml(receipt, footerNote);
    } else {
      html = generateA5ReceiptHtml(receipt, footerNote);
    }

    const res = await executeHtmlPrint(html);
    setPrinting(false);
    if (res.success) {
      setPrintStatus("success");
      setTimeout(() => setPrintStatus("idle"), 3000);
    } else {
      setPrintStatus("error");
      setErrorMessage(res.error || "Print failed");
    }
  };

  const handleZoomIn = () => setZoom((prev) => Math.min(prev + 15, 140));
  const handleZoomOut = () => setZoom((prev) => Math.max(prev - 15, 60));

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[90vh] flex flex-col p-0 overflow-hidden bg-background text-foreground">
        <DialogHeader className="p-4 border-b border-border flex-shrink-0 flex flex-row items-center justify-between">
          <div>
            <DialogTitle className="text-base font-bold flex items-center gap-2">
              <Printer className="h-4 w-4 text-primary" />
              Receipt Print Preview
              <span className="font-mono text-xs text-muted-foreground ml-1">
                ({receipt.receipt_no})
              </span>
            </DialogTitle>
            <p className="text-xs text-muted-foreground mt-0.5">
              {receipt.student_name} • {receipt.student_code} • {receipt.business_date}
            </p>
          </div>

          <div className="flex items-center gap-3">
            <Tabs
              value={layout}
              onValueChange={(val) =>
                setLayout(val as "thermal_80mm" | "a4_dual" | "a5_single")
              }
            >
              <TabsList className="h-8 text-xs">
                <TabsTrigger value="thermal_80mm" className="text-xs px-2.5">
                  80mm Thermal
                </TabsTrigger>
                <TabsTrigger value="a4_dual" className="text-xs px-2.5">
                  A4 Dual Copy
                </TabsTrigger>
                <TabsTrigger value="a5_single" className="text-xs px-2.5">
                  A5 Single Copy
                </TabsTrigger>
              </TabsList>
            </Tabs>

            <div className="flex items-center border border-border rounded bg-muted/40 px-1 py-0.5 text-xs">
              <button
                type="button"
                onClick={handleZoomOut}
                className="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                title="Zoom Out"
              >
                <ZoomOut className="h-3.5 w-3.5" />
              </button>
              <span className="px-1.5 font-mono text-[11px] min-w-[38px] text-center">
                {zoom}%
              </span>
              <button
                type="button"
                onClick={handleZoomIn}
                className="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                title="Zoom In"
              >
                <ZoomIn className="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
        </DialogHeader>

        {/* Preview Scroll Canvas */}
        <div className="flex-1 overflow-auto p-6 bg-neutral-900/10 dark:bg-neutral-950/40 flex justify-center items-start">
          <div
            style={{
              transform: `scale(${zoom / 100})`,
              transformOrigin: "top center",
              transition: "transform 0.15s ease-out",
            }}
            className="shadow-2xl rounded-sm overflow-hidden"
          >
            {layout === "thermal_80mm" && (
              <ThermalReceipt receipt={receipt} footerNote={footerNote} />
            )}
            {layout === "a4_dual" && (
              <A4DualReceipt receipt={receipt} footerNote={footerNote} />
            )}
            {layout === "a5_single" && (
              <A5Receipt receipt={receipt} footerNote={footerNote} />
            )}
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-3 border-t border-border bg-card flex-shrink-0 flex items-center justify-between">
          <div className="flex items-center gap-2 text-xs">
            {printStatus === "success" && (
              <span className="flex items-center gap-1 text-emerald-600 font-medium">
                <Check className="h-3.5 w-3.5" />
                Print command dispatched to spooler
              </span>
            )}
            {printStatus === "error" && (
              <span className="flex items-center gap-1 text-red-600 font-medium">
                <AlertCircle className="h-3.5 w-3.5" />
                {errorMessage}
              </span>
            )}
            {printStatus === "idle" && (
              <span className="text-muted-foreground">
                Decoupled printing: Hardware issues do not alter financial records.
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => onOpenChange(false)}>
              Close
            </Button>
            <Button
              size="sm"
              className="gap-1.5 font-semibold"
              disabled={printing}
              onClick={handlePrint}
            >
              <Printer className="h-4 w-4" />
              {printing ? "Printing..." : "Print Receipt"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
