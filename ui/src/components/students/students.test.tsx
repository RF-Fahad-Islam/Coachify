import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { StudentDirectoryView } from "./student-directory-view";
import { AddStudentDialog } from "./add-student-dialog";
import { StudentProfileDialog } from "./student-profile-dialog";
import { EditSharedGuardianDialog } from "./edit-shared-guardian-dialog";
import { formatPhone } from "@/lib/utils";

describe("Students & Guardians UI Components", () => {
  describe("formatPhone helper", () => {
    it("formats 11-digit Bangladeshi mobile numbers into 01XXX-XXXXXX format", () => {
      expect(formatPhone("01712345678")).toBe("01712-345678");
      expect(formatPhone("01812345678")).toBe("01812-345678");
      expect(formatPhone("01999999999")).toBe("01999-999999");
    });

    it("formats 13-digit Bangladeshi numbers with 880 prefix", () => {
      expect(formatPhone("8801712345678")).toBe("+880 1712-345678");
      expect(formatPhone("+8801712345678")).toBe("+880 1712-345678");
    });

    it("preserves non-Bangladeshi numbers", () => {
      expect(formatPhone("+14155552671")).toBe("+14155552671");
      expect(formatPhone("12345")).toBe("12345");
    });
  });

  describe("StudentDirectoryView", () => {
    it("renders search input, status filter tabs, and action buttons", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <StudentDirectoryView />
          </AuthProvider>
        </ThemeProvider>
      );

      // Main header and title
      expect(html).toContain("Students Directory");
      expect(html).toContain(
        "Manage enrolled students, search records, and configure guardian contacts."
      );

      // Filter tabs
      expect(html).toContain("All");
      expect(html).toContain("Active");
      expect(html).toContain("Inactive");
      expect(html).toContain("Completed");
      expect(html).toContain("Transferred");
      expect(html).toContain("Archived");

      // Search input placeholder
      expect(html).toContain("Search by student code, name, phone, or guardian...");

      // Table headers
      expect(html).toContain("Code");
      expect(html).toContain("Student Name");
      expect(html).toContain("Class / Roll");
      expect(html).toContain("Phone Number");
      expect(html).toContain("Primary Guardian");
      expect(html).toContain("Status");
      expect(html).toContain("Actions");

      // Keyboard helper
      expect(html).toContain("navigate rows");
      expect(html).toContain("inspect student");
      expect(html).toContain("add student");
    });
  });

  describe("AddStudentDialog", () => {
    it("renders AddStudentDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AddStudentDialog open={true} onOpenChange={() => {}} onSuccess={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("StudentProfileDialog", () => {
    it("renders StudentProfileDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <StudentProfileDialog
              studentId="stud-001"
              open={true}
              onOpenChange={() => {}}
              onStudentUpdated={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("EditSharedGuardianDialog", () => {
    it("renders EditSharedGuardianDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <EditSharedGuardianDialog
            open={true}
            onOpenChange={() => {}}
            guardianId="guard-001"
            currentStudentId="stud-001"
            currentStudentName="Rahim Chowdhury"
            onSuccess={() => {}}
          />
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });
});
