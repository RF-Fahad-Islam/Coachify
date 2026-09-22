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
