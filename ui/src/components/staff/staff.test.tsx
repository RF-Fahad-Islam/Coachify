import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { StaffDirectoryView } from "./staff-directory-view";
import { AddStaffDialog } from "./add-staff-dialog";
import { StaffDetailDialog } from "./staff-detail-dialog";

describe("Staff & Faculty Directory UI Components", () => {
  describe("StaffDirectoryView", () => {
    it("renders page header, FTS5 search input, and directory table headers", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <StaffDirectoryView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Staff &amp; Faculty Directory");
      expect(html).toContain(
        "Manage teachers, instructors, administrative personnel, and class allocations."
      );
      expect(html).toContain("Search by name, phone, designation (FTS5)...");
      expect(html).toContain("All Departments");
      expect(html).toContain("All Statuses");
    });
  });

  describe("AddStaffDialog", () => {
    it("renders AddStaffDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AddStaffDialog open={true} onOpenChange={() => {}} onCreated={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("StaffDetailDialog", () => {
    it("renders StaffDetailDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <StaffDetailDialog
              staffId="staff-001"
              open={true}
              onOpenChange={() => {}}
              onUpdated={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });
});
