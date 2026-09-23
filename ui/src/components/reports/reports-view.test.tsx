import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { ReportsView } from "./reports-view";

describe("ReportsView Component", () => {
  it("renders reports center header, all 6 report tabs, and export triggers", () => {
    const html = renderToString(
      <ThemeProvider>
        <AuthProvider>
          <ReportsView />
        </AuthProvider>
      </ThemeProvider>
    );

    // Title & subtitle
    expect(html).toContain("Analytical Reports Center");
    expect(html).toContain(
      "Real-time financial audits, dues aging recovery, attendance registers, and"
    );

    // All 6 Spec 0028 Report Tabs
    expect(html).toContain("Fee Collections");
    expect(html).toContain("Cashier Performance");
    expect(html).toContain("Dues Aging &amp; Recovery");
    expect(html).toContain("Expenses &amp; Cash Drawer");
    expect(html).toContain("Attendance &amp; Chronic Absenteeism");
    expect(html).toContain("Exam Merit Summaries");

    // Filter controls and buttons
    expect(html).toContain("Filters:");
    expect(html).toContain("From:");
    expect(html).toContain("To:");
    expect(html).toContain("Export Excel (CSV)");
    expect(html).toContain("Print View");
    expect(html).toContain("Apply Filters");
  });
});
