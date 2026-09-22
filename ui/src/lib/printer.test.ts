import { describe, it, expect } from "vitest";
import { DEFAULT_PRINTER_SETTINGS, getPrinterSettings } from "./printer";

describe("Printer Library & Settings", () => {
  it("provides default printer settings adhering to standard thermal 80mm roll", () => {
    expect(DEFAULT_PRINTER_SETTINGS.page_size).toBe("thermal_80mm");
    expect(DEFAULT_PRINTER_SETTINGS.auto_print_on_payment).toBe(true);
    expect(DEFAULT_PRINTER_SETTINGS.copies).toBe(1);
    expect(DEFAULT_PRINTER_SETTINGS.open_cash_drawer).toBe(false);
  });

  it("falls back gracefully to default settings when tauri rpc bridge is absent in test", async () => {
    const settings = await getPrinterSettings();
    expect(settings.page_size).toBe("thermal_80mm");
    expect(settings.auto_print_on_payment).toBe(true);
  });
});
