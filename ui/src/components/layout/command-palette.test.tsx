import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { CommandPalette } from "./command-palette";

describe("CommandPalette Component", () => {
  it("renders closed state without crashing", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <CommandPalette
            open={false}
            onOpenChange={() => {}}
            onSelectTab={() => {}}
            onOpenThemeCustomizer={() => {}}
          />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(typeof html).toBe("string");
  });

  it("renders open state without crashing", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <CommandPalette
            open={true}
            onOpenChange={() => {}}
            onSelectTab={() => {}}
            onOpenThemeCustomizer={() => {}}
          />
        </AuthProvider>
      </ThemeProvider>
    );

    expect(typeof html).toBe("string");
  });
});
