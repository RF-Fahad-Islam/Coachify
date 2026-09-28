import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { GeneralSettingsCard } from "./general-settings-card";
import { ThemeProvider } from "@/theme/theme-provider";

describe("GeneralSettingsCard component (Module 13)", () => {
  it("renders institute profile sections and default branding", () => {
    const html = renderToString(
      <ThemeProvider>
        <GeneralSettingsCard />
      </ThemeProvider>
    );

    expect(html).toContain("Institute Profile &amp; Operational Details");
    expect(html).toContain("Apex Academy Coaching Center");
    expect(html).toContain("AAC-DHK");
    expect(html).toContain("contact@apexacademy.edu.bd");
    expect(html).toContain("Primary Currency Symbol");
    expect(html).toContain("Current Academic Session");
  });

  it("renders theme, palette, density, and typography controls", () => {
    const html = renderToString(
      <ThemeProvider>
        <GeneralSettingsCard />
      </ThemeProvider>
    );

    expect(html).toContain("Appearance, Density &amp; Typography Engine");
    expect(html).toContain("Sunset Orange");
    expect(html).toContain("Ocean Blue");
    expect(html).toContain("Emerald Growth");
    expect(html).toContain("Appearance Mode");
    expect(html).toContain("Layout Density");
    expect(html).toContain("Corner Radius Style (Fatty / Plush)");
    expect(html).toContain("Bilingual Font Family");
  });

  it("renders save and reset action buttons", () => {
    const html = renderToString(
      <ThemeProvider>
        <GeneralSettingsCard />
      </ThemeProvider>
    );

    expect(html).toContain("Save Configuration");
    expect(html).toContain("Reset Defaults");
  });
});
