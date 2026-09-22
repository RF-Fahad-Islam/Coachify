import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { AttendanceView } from "./attendance-view";
import { AttendanceSheet } from "./attendance-sheet";
import { AttendanceMatrixView } from "./attendance-matrix-view";

describe("Attendance UI Components", () => {
  describe("AttendanceView", () => {
    it("renders page header and navigation tabs for daily sheet and monthly matrix", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AttendanceView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Attendance &amp; Class Register");
      expect(html).toContain("Daily Sheet");
      expect(html).toContain("Monthly Matrix");
    });
  });

  describe("AttendanceSheet", () => {
    it("renders batch selection, date picker, quick actions, and keyboard shortcuts guide", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AttendanceSheet />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Batch:");
      expect(html).toContain("Date:");
      expect(html).toContain("Save Attendance");
      expect(html).toContain("Keyboard Controls:");
      expect(html).toContain("Present");
      expect(html).toContain("Absent");
      expect(html).toContain("Late");
      expect(html).toContain("Excused");
      expect(html).toContain("Class Student Roster");
      expect(html).toContain("Class Notes &amp; Covered Curriculum:");
    });
  });

  describe("AttendanceMatrixView", () => {
    it("renders month selector, batch selector, and monthly register legend", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AttendanceMatrixView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Batch:");
      expect(html).toContain("Month:");
      expect(html).toContain("Monthly Attendance Register");
      expect(html).toContain("Status Legend:");
      expect(html).toContain("Present");
      expect(html).toContain("Absent");
      expect(html).toContain("Late");
      expect(html).toContain("Excused");
      expect(html).toContain("No Session Held");
    });
  });
});
