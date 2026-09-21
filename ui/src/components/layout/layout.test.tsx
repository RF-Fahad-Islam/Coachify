import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { Sidebar, NAV_ITEMS } from "./sidebar";
import { TopBar } from "./top-bar";
import { QuickSearchDialog } from "./quick-search-dialog";
import { AppShell } from "./app-shell";
import { DashboardView } from "@/components/dashboard/dashboard-view";
import { ThemeCustomizerDialog } from "@/components/theme/theme-customizer-dialog";
import { ThemeProvider } from "@/theme/theme-provider";

describe("Layout and application shell components", () => {
  describe("Sidebar", () => {
    it("renders institute branding and Coachify title", () => {
      const html = renderToString(
        <Sidebar
          activeTab="dashboard"
          onTabChange={() => {}}
          onOpenThemeCustomizer={() => {}}
        />
      );
      expect(html).toContain("Coachify");
      expect(html).toContain("Coaching Management System");
      expect(html).toContain("Empowering Futures");
      expect(html).toContain("Customize Theme");
    });

    it("renders all navigation items with active highlighting and badges", () => {
      const html = renderToString(
        <Sidebar
          activeTab="students"
          onTabChange={() => {}}
          onOpenThemeCustomizer={() => {}}
        />
      );

      for (const item of NAV_ITEMS) {
        expect(html).toContain(item.label);
      }

      // Badge check
      expect(html).toContain("1.2k");
      expect(html).toContain("Due");
    });
  });

  describe("TopBar", () => {
    it("renders global search input and shortcut indicator", () => {
      const html = renderToString(
        <TopBar onOpenThemeCustomizer={() => {}} onSearchClick={() => {}} />
      );
      expect(html).toContain("Ctrl K");
      expect(html).toContain("Search students, payments, classes...");
    });

    it("renders date range filter, notifications badge, and user profile", () => {
      const html = renderToString(
        <TopBar onOpenThemeCustomizer={() => {}} onSearchClick={() => {}} />
      );
      expect(html).toContain("Apr 21, 2026");
      expect(html).toContain("Rohan Mehta");
      expect(html).toContain("Admin");
      expect(html).toContain("RM");
      expect(html).toContain("3");
    });
  });

  describe("QuickSearchDialog and ThemeCustomizerDialog", () => {
    it("renders QuickSearchDialog without crashing", () => {
      const html = renderToString(
        <QuickSearchDialog
          open={false}
          onOpenChange={() => {}}
          onSelectTab={() => {}}
          onOpenThemeCustomizer={() => {}}
        />
      );
      expect(typeof html).toBe("string");
    });

    it("renders ThemeCustomizerDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <ThemeCustomizerDialog open={false} onOpenChange={() => {}} />
        </ThemeProvider>
      );
      expect(typeof html).toBe("string");
    });
  });

  describe("AppShell", () => {
    it("renders sidebar, top bar, and main children container", () => {
      const html = renderToString(
        <ThemeProvider>
          <AppShell activeTab="dashboard">
            <div data-testid="test-content">Dashboard Main Content</div>
          </AppShell>
        </ThemeProvider>
      );
      expect(html).toContain("Dashboard Main Content");
      expect(html).toContain("Coachify");
      expect(html).toContain("Search students, payments, classes...");
    });
  });

  describe("DashboardView", () => {
    it("renders KPI metric cards, charts, actions, activity, and tables", () => {
      const html = renderToString(
        <ThemeProvider>
          <DashboardView
            rpcResponse={{
              status: "ok",
              database_status: "ok",
              database_path: "memory",
              roundtrip_timestamp: "2026-09-20T15:00:00Z",
            }}
            rpcLoading={false}
            rpcError={null}
            onRefreshRpc={() => {}}
          />
        </ThemeProvider>
      );

      // KPI Metric Cards
      expect(html).toContain("Total Students");
      expect(html).toContain("482");
      expect(html).toContain("Today&#x27;s Attendance");
      expect(html).toContain("412 / 482");
      expect(html).toContain("Pending Payments");
      expect(html).toContain("Upcoming Exams");

      // Attendance Trends SVG chart
      expect(html).toContain("Attendance Trends");
      expect(html).toContain("attendanceGradient");

      // Quick Actions
      expect(html).toContain("Quick Actions");
      expect(html).toContain("Add Student");
      expect(html).toContain("Record Payment");

      // Recent Activity
      expect(html).toContain("Recent Activity");
      expect(html).toContain("Aarav Sharma");

      // Recent Students Table
      expect(html).toContain("Recent Students");
      expect(html).toContain("Class 10");
      expect(html).toContain("Priya Singh");
      expect(html).toContain("Paid");
      expect(html).toContain("Pending");

      // Fee Collection
      expect(html).toContain("Fee Collection");
      expect(html).toContain("3,28,000");
      expect(html).toContain("Target Progress");
      expect(html).toContain("78%");

      // Core Architecture Connection
      expect(html).toContain("Core Architecture Connection");
      expect(html).toContain("Core Connected");
      expect(html).toContain("Verify RPC");
    });
  });
});
