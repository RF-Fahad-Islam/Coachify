import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { PrinterSettingsCard } from "./printer-settings-card";

describe("PrinterSettingsCard Component", () => {
  it("renders loading skeleton initially during system printer detection", () => {
    const html = renderToString(<PrinterSettingsCard />);
    expect(html).toContain("Detecting system printers...");
  });
});
