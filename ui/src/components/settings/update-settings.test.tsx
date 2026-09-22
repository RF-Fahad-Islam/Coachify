import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { UpdateSettingsCard } from "./update-settings";

describe("UpdateSettingsCard Component", () => {
  it("renders version and pre-update policy details", () => {
    const html = renderToString(<UpdateSettingsCard />);
    expect(html).toContain("Coachify v0.1.0-alpha");
    expect(html).toContain("Automatic Pre-Update Snapshot");
    expect(html).toContain("Data Preserved on Uninstall");
  });
});
