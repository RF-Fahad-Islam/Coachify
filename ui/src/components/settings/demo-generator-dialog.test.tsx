import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { DemoGeneratorDialog, DeveloperDemoCard } from "./demo-generator-dialog";

describe("Demo Generator Components", () => {
  it("renders DeveloperDemoCard with developer and demo datasets description", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <DeveloperDemoCard />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(html).toContain("Developer &amp; Demo Datasets");
    expect(html).toContain("Generate synthetic datasets");
    expect(html).toContain("Open Demo Generator");
  });

  it("renders DemoGeneratorDialog without crashing when open", () => {
    const html = renderToString(
      <ThemeProvider>
        <DemoGeneratorDialog open={true} onOpenChange={() => {}} />
      </ThemeProvider>
    );

    expect(html).toBeDefined();
  });

  it("renders DemoGeneratorDialog without crashing when closed", () => {
    const html = renderToString(
      <ThemeProvider>
        <DemoGeneratorDialog open={false} onOpenChange={() => {}} />
      </ThemeProvider>
    );

    expect(html).toBeDefined();
  });
});
