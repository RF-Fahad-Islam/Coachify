import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { CopilotDrawer } from "./copilot-drawer";
import { AiSettingsCard } from "./ai-settings-card";

describe("AI Foundation & Copilot UI Components (Slice 10)", () => {
  describe("CopilotDrawer", () => {
    it("renders nothing when open is false", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CopilotDrawer open={false} onClose={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toBe("");
    });

    it("renders drawer header, suggested questions, and protection banner when open is true", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CopilotDrawer open={true} onClose={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Coachify Copilot");
      expect(html).toContain("Local Data · Controlled Tools · Zero Retention");
      expect(html).toContain("Shield Active:");
      expect(html).toContain("Ask Coachify Institutional Copilot");
      expect(html).toContain("Which students have unpaid dues above ৳5,000?");
      expect(html).toContain("Give me today&#x27;s operational summary.");
      expect(html).toContain("Show attendance performance across active batches.");
      expect(html).toContain("Find student Nafis Fuad and check status.");
    });
  });

  describe("AiSettingsCard", () => {
    it("renders AI settings tabs: Privacy Shield, Credit Wallet, OpenRouter Gateway, and Audit Log", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AiSettingsCard />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("AI Foundation &amp; Institutional Intelligence");
      expect(html).toContain("Privacy Shield");
      expect(html).toContain("Credit Wallet");
      expect(html).toContain("OpenRouter Gateway");
      expect(html).toContain("AI Audit Log");
      expect(html).toContain("Zero-Leaked-Data Privacy Engine (Feature 55)");
      expect(html).toContain("Student Full Names");
      expect(html).toContain("Bangladeshi Mobile Numbers");
      expect(html).toContain("Guardian Names &amp; Contacts");
      expect(html).toContain("Financial Figures &amp; Dues Amounts");
      expect(html).toContain("Exam Marks &amp; Scores");
      expect(html).toContain("Teacher &amp; Staff Names");
    });
  });
});
