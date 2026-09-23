import {
  rpc,
  type WorkstationPrinterSettings,
  type SystemPrinterInfo,
  type PrintableReceiptData,
} from "../gen/rpc";

export const DEFAULT_PRINTER_SETTINGS: WorkstationPrinterSettings = {
  printer_name: null,
  page_size: "thermal_80mm",
  paper_source: null,
  copies: 1,
  auto_print_on_payment: true,
  open_cash_drawer: false,
  header_title: null,
  footer_note:
    "Thank you for being with us. Please keep this receipt for future reference.",
  updated_at: new Date().toISOString(),
};

/**
 * Loads the workstation printer configuration from local machine storage.
 */
export async function getPrinterSettings(): Promise<WorkstationPrinterSettings> {
  try {
    const res = await rpc("printers.get_settings", {});
    if (res.ok && res.data) {
      return res.data;
    }
  } catch (err: unknown) {
    console.error("Failed to load printer settings from core:", err);
  }
  return DEFAULT_PRINTER_SETTINGS;
}

/**
 * Saves the workstation printer configuration into local machine storage.
 */
export async function savePrinterSettings(
  settings: WorkstationPrinterSettings
): Promise<WorkstationPrinterSettings> {
  const res = await rpc("printers.save_settings", { settings });
  if (res.ok && res.data) {
    return res.data;
  }
  throw new Error(res.error?.message || "Failed to save printer settings");
}

/**
 * Enumerates installed operating system printers and reports the system default.
 */
export async function listSystemPrinters(): Promise<SystemPrinterInfo[]> {
  try {
    const res = await rpc("printers.list", {});
    if (res.ok && res.data) {
      return res.data;
    }
  } catch (err: unknown) {
    console.error("Failed to list system printers:", err);
  }
  return [];
}

/**
 * Retrieves printable receipt data with English and Bengali amount in words.
 */
export async function getPrintableReceipt(
  paymentId: string,
  isReprint = false
): Promise<PrintableReceiptData> {
  const res = await rpc("receipts.get_printable", {
    payment_id: paymentId,
    is_reprint: isReprint,
  });
  if (res.ok && res.data) {
    return res.data;
  }
  throw new Error(res.error?.message || "Failed to fetch printable receipt data");
}

export interface PrintResult {
  success: boolean;
  error?: string;
}

/**
 * Executes a fault tolerant print operation via a hidden iframe.
 * Any hardware or driver failure is caught and reported without affecting database operations.
 */
export function executeHtmlPrint(htmlContent: string): Promise<PrintResult> {
  return new Promise((resolve) => {
    try {
      const iframe = document.createElement("iframe");
      iframe.style.position = "fixed";
      iframe.style.right = "0";
      iframe.style.bottom = "0";
      iframe.style.width = "0";
      iframe.style.height = "0";
      iframe.style.border = "0";
      iframe.setAttribute("aria-hidden", "true");
      iframe.setAttribute("tabindex", "-1");

      document.body.appendChild(iframe);

      const doc = iframe.contentWindow?.document;
      if (!doc) {
        document.body.removeChild(iframe);
        resolve({ success: false, error: "Unable to access print frame document" });
        return;
      }

      doc.open();
      doc.write(htmlContent);
      doc.close();

      const handleAfterPrint = () => {
        setTimeout(() => {
          try {
            if (document.body.contains(iframe)) {
              document.body.removeChild(iframe);
            }
          } catch {
            // Frame might already be removed
          }
          resolve({ success: true });
        }, 300);
      };

      iframe.contentWindow?.addEventListener("afterprint", handleAfterPrint, {
        once: true,
      });

      setTimeout(() => {
        try {
          iframe.contentWindow?.focus();
          iframe.contentWindow?.print();
          // Fallback resolve in case afterprint does not fire in some WebViews
          setTimeout(() => {
            if (document.body.contains(iframe)) {
              try {
                document.body.removeChild(iframe);
              } catch {
                // Ignore cleanup errors
              }
            }
            resolve({ success: true });
          }, 2000);
        } catch (printErr: unknown) {
          if (document.body.contains(iframe)) {
            document.body.removeChild(iframe);
          }
          const message = printErr instanceof Error ? printErr.message : String(printErr);
          resolve({ success: false, error: message });
        }
      }, 250);
    } catch (topErr: unknown) {
      const message = topErr instanceof Error ? topErr.message : String(topErr);
      resolve({ success: false, error: message });
    }
  });
}

/**
 * Executes an HTML print operation with mandatory asset preloading:
 * - Awaits document.fonts.ready for Bengali ligatures (Kalpurush, Noto Sans Bengali)
 * - Synchronizes and decodes all embedded images prior to triggering window.print()
 */
export function executeHtmlPrintWithAssets(htmlContent: string): Promise<PrintResult> {
  return new Promise((resolve) => {
    try {
      const iframe = document.createElement("iframe");
      iframe.style.position = "fixed";
      iframe.style.right = "0";
      iframe.style.bottom = "0";
      iframe.style.width = "0";
      iframe.style.height = "0";
      iframe.style.border = "0";
      iframe.setAttribute("aria-hidden", "true");
      iframe.setAttribute("tabindex", "-1");

      document.body.appendChild(iframe);

      const doc = iframe.contentWindow?.document;
      if (!doc) {
        document.body.removeChild(iframe);
        resolve({ success: false, error: "Unable to access print frame document" });
        return;
      }

      doc.open();
      doc.write(htmlContent);
      doc.close();

      const handleAfterPrint = () => {
        setTimeout(() => {
          try {
            if (document.body.contains(iframe)) {
              document.body.removeChild(iframe);
            }
          } catch {
            // Frame might already be removed
          }
          resolve({ success: true });
        }, 300);
      };

      iframe.contentWindow?.addEventListener("afterprint", handleAfterPrint, {
        once: true,
      });

      // Synchronize fonts and images before calling window.print()
      const triggerPrintWhenReady = async () => {
        try {
          // Wait for fonts (Kalpurush, Noto Sans Bengali) to be fully ready
          if (doc.fonts) {
            await doc.fonts.ready;
          }

          // Wait for all images to decode
          const images = Array.from(doc.images);
          await Promise.all(
            images.map((img) => {
              if (img.complete) return Promise.resolve();
              return img.decode().catch(() => {});
            })
          );

          iframe.contentWindow?.focus();
          iframe.contentWindow?.print();

          // Fallback resolve in case afterprint does not fire
          setTimeout(() => {
            if (document.body.contains(iframe)) {
              try {
                document.body.removeChild(iframe);
              } catch {
                // Ignore cleanup error
              }
            }
            resolve({ success: true });
          }, 3000);
        } catch (err: unknown) {
          if (document.body.contains(iframe)) {
            document.body.removeChild(iframe);
          }
          const message = err instanceof Error ? err.message : String(err);
          resolve({ success: false, error: message });
        }
      };

      // Slight delay for iframe DOM attachment
      setTimeout(triggerPrintWhenReady, 150);
    } catch (topErr: unknown) {
      const message = topErr instanceof Error ? topErr.message : String(topErr);
      resolve({ success: false, error: message });
    }
  });
}

/**
 * Records a completed or attempted print job run in the audit log.
 */
export async function recordPrintJobRun(input: {
  document_type: string;
  batch_mode: boolean;
  item_count: number;
  printer_name?: string | null;
  paper_size: string;
  status: "completed" | "failed" | "cancelled";
  metadata_json?: string | null;
}) {
  try {
    const res = await rpc("printers.record_print_job", { job: input });
    if (res.ok && res.data) {
      return res.data;
    }
  } catch (err: unknown) {
    console.error("Failed to record print job run audit:", err);
  }
  return null;
}

/**
 * Fetches all document print template configurations.
 */
export async function getPrintTemplateConfigs() {
  const res = await rpc("printers.get_template_configs", {});
  if (res.ok && res.data) {
    return res.data;
  }
  throw new Error(res.error?.message || "Failed to load template configs");
}

/**
 * Updates a document print template configuration.
 */
export async function updatePrintTemplateConfig(config: {
  document_type: string;
  paper_size: string;
  orientation: string;
  show_header: boolean;
  show_logo: boolean;
  show_watermark: boolean;
  custom_header_text?: string | null;
  custom_footer_text?: string | null;
  signature_title?: string | null;
  options_json: string;
}) {
  const res = await rpc("printers.update_template_config", { config });
  if (res.ok && res.data) {
    return res.data;
  }
  throw new Error(res.error?.message || "Failed to update template config");
}

/**
 * Lists past print job audit records.
 */
export async function listPrintJobRuns(params?: {
  limit?: number;
  offset?: number;
  document_type?: string;
}) {
  const res = await rpc("printers.list_print_jobs", {
    limit: params?.limit ?? 50,
    offset: params?.offset ?? 0,
    document_type: params?.document_type ?? null,
  });
  if (res.ok && res.data) {
    return res.data;
  }
  throw new Error(res.error?.message || "Failed to list print jobs");
}
