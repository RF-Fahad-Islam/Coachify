import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { AcademicClassesView } from "./academic-classes-view";
import { CreateProgramDialog } from "./create-program-dialog";
import { CreateBatchDialog } from "./create-batch-dialog";
import { BatchDetailView } from "./batch-detail-view";
import { EnrollStudentDialog } from "./enroll-student-dialog";
import { TransferStudentDialog } from "./transfer-student-dialog";
import { BulkEnrollDialog } from "./bulk-enroll-dialog";
import { EndEnrollmentDialog } from "./end-enrollment-dialog";

describe("Academic Classes, Batches & Enrollments UI Components", () => {
  describe("AcademicClassesView", () => {
    it("renders page header, program selector bar, and search filters", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <AcademicClassesView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Academic Programs &amp; Batches");
      expect(html).toContain(
        "Manage course streams, classroom sections, weekly schedules, and rosters."
      );
      expect(html).toContain("Program Stream:");
      expect(html).toContain("Search batches by name, code, teacher...");
      expect(html).toContain("All Shifts");
      expect(html).toContain("All Statuses");
    });
  });

  describe("BatchDetailView", () => {
    it("renders batch detail summary, roster table headers, and enrollment actions", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <BatchDetailView batchId="batch-1" onBack={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Back to Classes");
      expect(html).toContain("Search by name, roll, code...");
      expect(html).toContain("Active");
      expect(html).toContain("Ended");
    });
  });

  describe("CreateProgramDialog", () => {
    it("renders CreateProgramDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CreateProgramDialog
              open={true}
              onOpenChange={() => {}}
              onCreated={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("CreateBatchDialog", () => {
    it("renders CreateBatchDialog without crashing", () => {
      const mockPrograms = [
        {
          id: "prog-1",
          session_id: "sess-1",
          session_name: "2026",
          name: "HSC Science 2026",
          code: "HSC-SCI-26",
          class_level: "Class 11",
          group_major: "Science",
          description: null,
          status: "active",
          batch_count: 2,
          active_student_count: 45,
          created_at: "2026-01-01",
          updated_at: "2026-01-01",
        },
      ];

      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CreateBatchDialog
              open={true}
              onOpenChange={() => {}}
              programs={mockPrograms}
              defaultProgramId="prog-1"
              onCreated={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("EnrollStudentDialog", () => {
    it("renders EnrollStudentDialog without crashing", () => {
      const mockBatch = {
        id: "batch-1",
        program_id: "prog-1",
        program_name: "HSC Science 2026",
        name: "Section A",
        code: "SEC-A",
        teacher_staff_id: null,
        teacher_name: null,
        room_number: "Room 101",
        shift: "Morning",
        schedule: [],
        capacity: 40,
        enrolled_count: 40,
        available_seats: 0,
        status: "active",
        start_date: "2026-01-01",
        end_date: null,
        subjects: [],
        created_at: "2026-01-01",
        updated_at: "2026-01-01",
      };

      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <EnrollStudentDialog
              open={true}
              onOpenChange={() => {}}
              batch={mockBatch}
              roster={[]}
              onEnrolled={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("TransferStudentDialog", () => {
    it("renders TransferStudentDialog without crashing", () => {
      const mockRosterItem = {
        enrollment_id: "enr-1",
        student_id: "stud-1",
        student_code: "STU-2026-001",
        student_name: "Tamim Iqbal",
        student_name_bn: "তামিম ইকবাল",
        phone_norm: "8801711223344",
        guardian_phone: "8801811223344",
        guardian_name: "Father",
        roll: 5,
        start_date: "2026-01-01",
        status: "active",
        admission_date: "2026-01-01",
      };

      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <TransferStudentDialog
              open={true}
              onOpenChange={() => {}}
              currentBatchId="batch-1"
              currentBatchName="Section A (Morning)"
              rosterItem={mockRosterItem}
              onTransferred={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("BulkEnrollDialog", () => {
    it("renders BulkEnrollDialog without crashing", () => {
      const mockBatch = {
        id: "batch-1",
        program_id: "prog-1",
        program_name: "HSC Science 2026",
        name: "Section A",
        code: "SEC-A",
        teacher_staff_id: null,
        teacher_name: null,
        room_number: "Room 101",
        shift: "Morning",
        schedule: [],
        capacity: 50,
        enrolled_count: 10,
        available_seats: 40,
        status: "active",
        start_date: "2026-01-01",
        end_date: null,
        subjects: [],
        created_at: "2026-01-01",
        updated_at: "2026-01-01",
      };

      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <BulkEnrollDialog
              open={true}
              onOpenChange={() => {}}
              batch={mockBatch}
              roster={[]}
              onBulkEnrolled={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });

  describe("EndEnrollmentDialog", () => {
    it("renders EndEnrollmentDialog without crashing", () => {
      const mockRosterItem = {
        enrollment_id: "enr-1",
        student_id: "stud-1",
        student_code: "STU-2026-001",
        student_name: "Shakib Al Hasan",
        student_name_bn: null,
        phone_norm: "8801711223344",
        guardian_phone: null,
        guardian_name: null,
        roll: 1,
        start_date: "2026-01-01",
        status: "active",
        admission_date: "2026-01-01",
      };

      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <EndEnrollmentDialog
              open={true}
              onOpenChange={() => {}}
              rosterItem={mockRosterItem}
              onEnded={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );
      expect(html).toBeDefined();
    });
  });
});
