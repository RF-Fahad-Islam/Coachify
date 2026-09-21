import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { AuthProvider } from "@/contexts/auth-context";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuditLogView } from "./audit-log-view";

describe("AuditLogView Component", () => {
  it("renders restricted access card when not logged in or missing audit.view permission", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <AuditLogView />
        </AuthProvider>
      </ThemeProvider>
    );
    expect(html).toContain("Audit Log Access Restricted");
    expect(html).toContain("audit.view");
  });
});
