import { useState, useMemo } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { Button } from "../ui/button";
import { executeHtmlPrintWithAssets, recordPrintJobRun } from "@/lib/printer";
import {
  Printer,
  ZoomIn,
  ZoomOut,
  RotateCw,
  FileText,
  ChevronLeft,
  ChevronRight,
  Check,
  AlertCircle,
  Loader2,
} from "lucide-react";

export interface DocumentPrintPreviewModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  documentType:
    | "id_card"
    | "admit_card"
    | "receipt"
    | "due_statement"
    | "due_slip_thermal"
    | "cash_close_thermal"
    | "attendance_sheet"
    | "financial_ledger"
    | "report_card"
    | "tabulation_sheet"
    | "admission_form";
  initialPaperSize?: "a4" | "a5" | "thermal_80mm" | "id_card_pvc";
  initialOrientation?: "portrait" | "landscape";
  totalItems?: number;
  chunkSize?: number;
  htmlContent?: string;
  generateHtmlForChunk?: (chunkIndex: number, options: PreviewRenderOptions) => string;
  onPrintConfirmed?: () => Promise<void>;
}

export interface PreviewRenderOptions {
  paperSize: "a4" | "a5" | "thermal_80mm" | "id_card_pvc";
  orientation: "portrait" | "landscape";
  showWatermark: boolean;
  showHeader: boolean;
}

export function DocumentPrintPreviewModal({
  open,
  onOpenChange,
  title,
  documentType,
  initialPaperSize = "a4",
  initialOrientation = "portrait",
  totalItems = 1,
  chunkSize = 50,
  htmlContent,
  generateHtmlForChunk,
  onPrintConfirmed,
}: DocumentPrintPreviewModalProps) {
  const [paperSize, setPaperSize] = useState<
    "a4" | "a5" | "thermal_80mm" | "id_card_pvc"
  >(initialPaperSize);
  const [orientation, setOrientation] = useState<"portrait" | "landscape">(
    initialOrientation
  );
  const [showWatermark, setShowWatermark] = useState<boolean>(false);
  const [showHeader, setShowHeader] = useState<boolean>(true);
  const [zoom, setZoom] = useState<number>(100);
  const [currentChunk, setCurrentChunk] = useState<number>(0);
  const [printing, setPrinting] = useState<boolean>(false);
  const [printStatus, setPrintStatus] = useState<"idle" | "success" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");

  const totalChunks = Math.max(1, Math.ceil(totalItems / chunkSize));

  // Determine current active HTML content
  const activeHtml = useMemo(() => {
    if (generateHtmlForChunk) {
      return generateHtmlForChunk(currentChunk, {
        paperSize,
        orientation,
        showWatermark,
        showHeader,
      });
    }
    return htmlContent || "";
  }, [
    generateHtmlForChunk,
    currentChunk,
    paperSize,
    orientation,
    showWatermark,
    showHeader,
    htmlContent,
  ]);

  const handleZoomIn = () => setZoom((prev) => Math.min(prev + 15, 140));
  const handleZoomOut = () => setZoom((prev) => Math.max(prev - 15, 60));
  const handleResetZoom = () => setZoom(100);

  const toggleOrientation = () => {
    setOrientation((prev) => (prev === "portrait" ? "landscape" : "portrait"));
  };

  const handlePrint = async () => {
    setPrinting(true);
    setPrintStatus("idle");
    setErrorMessage("");

    try {
      if (onPrintConfirmed) {
        await onPrintConfirmed();
      } else {
        // If batch with multiple chunks and generator provided, compile full HTML across chunks
        let printHtml = activeHtml;
        if (generateHtmlForChunk && totalChunks > 1) {
          const allChunksHtml: string[] = [];
          for (let i = 0; i < totalChunks; i++) {
            allChunksHtml.push(
              generateHtmlForChunk(i, {
                paperSize,
                orientation,
                showWatermark,
                showHeader,
              })
            );
          }
          printHtml = allChunksHtml.join("\n<!-- PAGE_BREAK -->\n");
        }

        const res = await executeHtmlPrintWithAssets(printHtml);
        if (res.success) {
          setPrintStatus("success");
          // Record print job run audit
          await recordPrintJobRun({
            document_type: documentType,
            batch_mode: totalItems > 1,
            item_count: totalItems,
            paper_size: paperSize,
            status: "completed",
            metadata_json: JSON.stringify({
              orientation,
              total_chunks: totalChunks,
            }),
          });
          setTimeout(() => setPrintStatus("idle"), 3500);
        } else {
          setPrintStatus("error");
          setErrorMessage(res.error || "Print failed on workstation hardware");
          await recordPrintJobRun({
            document_type: documentType,
            batch_mode: totalItems > 1,
            item_count: totalItems,
            paper_size: paperSize,
            status: "failed",
            metadata_json: JSON.stringify({ error: res.error }),
          });
        }
      }
    } catch (err: unknown) {
      setPrintStatus("error");
      setErrorMessage(err instanceof Error ? err.message : String(err));
    } finally {
      setPrinting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-5xl max-h-[92vh] flex flex-col p-0 overflow-hidden bg-background text-foreground shadow-2xl">
        {/* Header Toolbar */}
        <DialogHeader className="p-3.5 border-b border-border flex-shrink-0 flex flex-row items-center justify-between">
          <div className="flex items-center gap-2">
            <Printer className="h-5 w-5 text-primary" />
            <div>
              <DialogTitle className="text-base font-bold leading-tight">
                {title}
              </DialogTitle>
              <div className="text-xs text-muted-foreground">
                Document Preview &bull; {totalItems} {totalItems === 1 ? "item" : "items"}
              </div>
            </div>
          </div>

          {/* Action and Print Status Controls */}
          <div className="flex items-center gap-2">
            {printStatus === "success" && (
              <span className="flex items-center gap-1 text-xs text-emerald-600 bg-emerald-50 dark:bg-emerald-950/40 dark:text-emerald-400 px-2.5 py-1 rounded-md font-medium border border-emerald-200 dark:border-emerald-800">
                <Check className="h-3.5 w-3.5" /> Spooled successfully
              </span>
            )}
            {printStatus === "error" && (
              <span className="flex items-center gap-1 text-xs text-destructive bg-destructive/10 px-2.5 py-1 rounded-md font-medium border border-destructive/20 max-w-[240px] truncate">
                <AlertCircle className="h-3.5 w-3.5 flex-shrink-0" />
                {errorMessage}
              </span>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => onOpenChange(false)}
              disabled={printing}
            >
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={handlePrint}
              disabled={printing}
              className="gap-1.5 font-semibold"
            >
              {printing ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Spooling...
                </>
              ) : (
                <>
                  <Printer className="h-4 w-4" />
                  Print ({totalItems})
                </>
              )}
            </Button>
          </div>
        </DialogHeader>

        {/* Options Toolbar: Paper, Orientation, Zoom, Chunk Navigation */}
        <div className="px-4 py-2 bg-muted/30 border-b border-border flex flex-wrap items-center justify-between gap-3 text-xs">
          <div className="flex items-center gap-3">
            {/* Paper Size selector */}
            <div className="flex items-center gap-1.5">
              <span className="text-muted-foreground font-medium">Paper:</span>
              <select
                value={paperSize}
                onChange={(e) =>
                  setPaperSize(
                    e.target.value as "a4" | "a5" | "thermal_80mm" | "id_card_pvc"
                  )
                }
                className="h-8 rounded-md border border-input bg-background px-2 text-xs font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              >
                <option value="a4">A4 (210 &times; 297 mm)</option>
                <option value="a5">A5 (148 &times; 210 mm)</option>
                <option value="thermal_80mm">80mm Thermal Roll</option>
                <option value="id_card_pvc">CR80 Plastic Card</option>
              </select>
            </div>

            {/* Orientation Toggle */}
            <Button
              variant="outline"
              size="sm"
              onClick={toggleOrientation}
              className="h-8 gap-1 text-xs"
              title="Toggle Orientation"
            >
              <RotateCw className="h-3.5 w-3.5" />
              <span className="capitalize">{orientation}</span>
            </Button>

            {/* Watermark Toggle */}
            <Button
              variant={showWatermark ? "secondary" : "outline"}
              size="sm"
              onClick={() => setShowWatermark((prev) => !prev)}
              className="h-8 text-xs"
            >
              Watermark: {showWatermark ? "ON" : "OFF"}
            </Button>

            {/* Header Toggle */}
            <Button
              variant={showHeader ? "outline" : "ghost"}
              size="sm"
              onClick={() => setShowHeader((prev) => !prev)}
              className="h-8 text-xs"
            >
              Header: {showHeader ? "Visible" : "Hidden"}
            </Button>
          </div>

          {/* Right Toolbar: Batch Chunking & Zoom Controls */}
          <div className="flex items-center gap-3">
            {/* Batch Chunker (AC-3: 50 items per chunk) */}
            {totalChunks > 1 && (
              <div className="flex items-center gap-1.5 bg-background border border-border rounded-md px-2 py-0.5">
                <span className="text-muted-foreground">Chunk:</span>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setCurrentChunk((prev) => Math.max(0, prev - 1))}
                  disabled={currentChunk === 0}
                >
                  <ChevronLeft className="h-3.5 w-3.5" />
                </Button>
                <span className="font-mono font-semibold">
                  {currentChunk + 1} / {totalChunks}
                </span>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() =>
                    setCurrentChunk((prev) => Math.min(totalChunks - 1, prev + 1))
                  }
                  disabled={currentChunk >= totalChunks - 1}
                >
                  <ChevronRight className="h-3.5 w-3.5" />
                </Button>
              </div>
            )}

            {/* Zoom Controls */}
            <div className="flex items-center gap-1 bg-background border border-border rounded-md px-1.5 py-0.5">
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={handleZoomOut}
                disabled={zoom <= 60}
                title="Zoom Out"
              >
                <ZoomOut className="h-3.5 w-3.5" />
              </Button>
              <button
                type="button"
                onClick={handleResetZoom}
                className="font-mono text-xs px-1 hover:text-primary transition-colors cursor-pointer"
                title="Reset Zoom to 100%"
              >
                {zoom}%
              </button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={handleZoomIn}
                disabled={zoom >= 140}
                title="Zoom In"
              >
                <ZoomIn className="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>
        </div>

        {/* Live Preview Canvas Container */}
        <div className="flex-grow overflow-auto p-6 bg-muted/40 flex items-start justify-center">
          <div
            style={{
              transform: `scale(${zoom / 100})`,
              transformOrigin: "top center",
              transition: "transform 0.15s ease",
            }}
            className="flex flex-col items-center justify-center max-w-full"
          >
            {activeHtml ? (
              <div
                className="bg-white text-slate-900 shadow-lg border border-border/80 rounded-sm overflow-hidden"
                dangerouslySetInnerHTML={{ __html: activeHtml }}
              />
            ) : (
              <div className="p-12 text-center text-muted-foreground flex flex-col items-center gap-2">
                <FileText className="h-10 w-10 text-muted-foreground/50" />
                <p>No document content rendered</p>
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
