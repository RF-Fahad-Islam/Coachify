import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { ImportStudentsDialog } from "./import-students-dialog";
import { StudentDirectoryView } from "./student-directory-view";

let hasManagePermission = true;

vi.mock("@/contexts/auth-context", () => ({
  useAuth: () => ({
    currentUser: { display_name: "Admin Owner", roles: ["Admin"] },
    hasPermission: (code: string) =>
      code === "students.manage" ? hasManagePermission : false,
    isOwner: () => true,
    isIdleLocked: false,
    needsSetup: false,
    loading: false,
    sessionToken: "mock-session-token",
  }),
  AuthProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

describe("ImportStudentsDialog & Bulk Excel Import UI", () => {
  beforeEach(() => {
    hasManagePermission = true;
  });

  it("renders ImportStudentsDialog without crashing when open", () => {
    const html = renderToString(
      <ThemeProvider>
        <ImportStudentsDialog open={true} onOpenChange={() => {}} />
      </ThemeProvider>
    );

    expect(html).toBeDefined();
  });

  it("renders ImportStudentsDialog without crashing when closed", () => {
    const html = renderToString(
      <ThemeProvider>
        <ImportStudentsDialog open={false} onOpenChange={() => {}} />
      </ThemeProvider>
    );

    expect(html).toBeDefined();
  });

  it("renders 'Import from Excel' action button alongside 'Admit Student' in StudentDirectoryView when user has students.manage permission", () => {
    hasManagePermission = true;
    const html = renderToString(
      <ThemeProvider>
        <StudentDirectoryView />
      </ThemeProvider>
    );

    expect(html).toContain("Import from Excel");
    expect(html).toContain("Admit Student");
    expect(html).toContain("Students Directory");
  });

  it("hides 'Import from Excel' action button in StudentDirectoryView when user lacks students.manage permission", () => {
    hasManagePermission = false;
    const html = renderToString(
      <ThemeProvider>
        <StudentDirectoryView />
      </ThemeProvider>
    );

    expect(html).not.toContain("Import from Excel");
    expect(html).not.toContain("Admit Student");
    expect(html).toContain("Students Directory");
  });
});
