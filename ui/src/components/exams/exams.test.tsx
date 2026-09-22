import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { ExamsHubView } from "./exams-hub-view";
import { CreateExamDialog } from "./create-exam-dialog";
import { MarkEntryGrid } from "./mark-entry-grid";
import { ResultPublishView } from "./result-publish-view";
import { generateReportCardHtml } from "../print/report-card-template";
import { generateTabulationSheetHtml } from "../print/tabulation-sheet-template";
import type { StudentReportCardData, BatchTabulationSheetData } from "@/gen/rpc";

describe("Exams and Results UI Components", () => {
  describe("ExamsHubView", () => {
    it("renders page title, create exam button, search bar, and status tabs", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <ExamsHubView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Exams &amp; Results Center");
      expect(html).toContain("Create Examination");
      expect(html).toContain("All");
      expect(html).toContain("In Progress");
      expect(html).toContain("Verified");
      expect(html).toContain("Published");
      expect(html).toContain("Search exams by name...");
    });
  });

  describe("CreateExamDialog", () => {
    it("renders CreateExamDialog without crashing", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CreateExamDialog
              open={true}
              onOpenChange={() => {}}
              onExamCreated={() => {}}
            />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toBeDefined();
    });
  });

  describe("MarkEntryGrid", () => {
    it("renders subject selector, bounds information, keyboard navigation hints, and table headers", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <MarkEntryGrid examId="exam-123" onBack={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Full Marks:");
      expect(html).toContain("Pass Marks:");
      expect(html).toContain("Entry Progress:");
      expect(html).toContain("Roll");
      expect(html).toContain("Student ID");
      expect(html).toContain("Student Name");
      expect(html).toContain("Absent");
      expect(html).toContain("4th Subject");
    });
  });

  describe("ResultPublishView", () => {
    it("renders metric cards, search input, tabulation button, and candidate preview table", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <ResultPublishView examId="exam-123" onBack={() => {}} />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Total Candidates");
      expect(html).toContain("Average GPA");
      expect(html).toContain("Tabulation Sheet");
      expect(html).toContain("Publish Results");
      expect(html).toContain("Rank");
      expect(html).toContain("Total Marks");
      expect(html).toContain("GPA");
      expect(html).toContain("Grade");
    });
  });

  describe("Print Templates", () => {
    it("generates student report card HTML with bilingual headers, subject breakdown, and signatures", () => {
      const sampleReportCard: StudentReportCardData = {
        institution_name: "Apex Model Academy",
        institution_name_bn: "অ্যাপেক্স মডেল একাডেমি",
        institution_address: "Dhanmondi, Dhaka-1209",
        institution_phones: "01711000000",
        student_id: "st-1",
        student_code: "STD-2026-0001",
        student_name: "Rahim Chowdhury",
        student_name_bn: "রহিম চৌধুরী",
        roll: 1,
        batch_name: "Batch Alpha",
        program_name: "Class 10 SSC",
        exam_name: "First Terminal Examination 2026",
        session_name: "Session 2026",
        total_marks: 32000,
        total_max_marks: 40000,
        percentage: 8000,
        gpa: 483,
        grade: "A",
        passed: true,
        rank: 1,
        total_students: 45,
        subjects: [
          {
            exam_subject_id: "es-1",
            subject_name: "Bangla",
            subject_code: "101",
            marks: 8500,
            full_marks: 10000,
            pass_marks: 3300,
            is_absent: false,
            is_optional: false,
            grade: "A+",
            grade_point: 500,
            passed: true,
          },
          {
            exam_subject_id: "es-2",
            subject_name: "Higher Math",
            subject_code: "104",
            marks: 7500,
            full_marks: 10000,
            pass_marks: 3300,
            is_absent: false,
            is_optional: true,
            grade: "A",
            grade_point: 400,
            passed: true,
          },
        ],
        attendance_percentage: 95.5,
      };

      const html = generateReportCardHtml(sampleReportCard);
      expect(html).toContain("Apex Model Academy");
      expect(html).toContain("অ্যাপেক্স মডেল একাডেমি");
      expect(html).toContain("Progress Report Card");
      expect(html).toContain("STD-2026-0001");
      expect(html).toContain("Rahim Chowdhury");
      expect(html).toContain("Bangla");
      expect(html).toContain("(4th Subject)");
      expect(html).toContain("4.83");
      expect(html).toContain("PASSED");
      expect(html).toContain("Class Teacher");
      expect(html).toContain("Academic In-Charge");
      expect(html).toContain("Principal / Headmaster");
    });

    it("generates landscape tabulation sheet HTML with fixed layout, subject columns, and ranks", () => {
      const sampleTabulation: BatchTabulationSheetData = {
        exam_id: "ex-1",
        exam_name: "First Terminal Examination 2026",
        batch_id: "b-1",
        batch_name: "Batch Alpha",
        session_name: "Session 2026",
        subjects: [
          {
            exam_subject_id: "es-1",
            subject_name: "Bangla",
            subject_code: "101",
            full_marks: 10000,
            pass_marks: 3300,
          },
          {
            exam_subject_id: "es-2",
            subject_name: "English",
            subject_code: "102",
            full_marks: 10000,
            pass_marks: 3300,
          },
        ],
        rows: [
          {
            student_id: "st-1",
            roll: 1,
            student_code: "STD-001",
            student_name: "Rahim Chowdhury",
            marks_by_subject: {
              "es-1": {
                marks: 8500,
                grade: "A+",
                grade_point: 500,
                is_absent: false,
                is_optional: false,
                passed: true,
              },
              "es-2": {
                marks: 7500,
                grade: "A",
                grade_point: 400,
                is_absent: false,
                is_optional: false,
                passed: true,
              },
            },
            total_marks: 16000,
            gpa: 450,
            grade: "A",
            passed: true,
            rank: 1,
          },
        ],
      };

      const html = generateTabulationSheetHtml(sampleTabulation);
      expect(html).toContain("size: A4 landscape;");
      expect(html).toContain("Academic Tabulation Register");
      expect(html).toContain("Batch Alpha");
      expect(html).toContain("Rahim Chowdhury");
      expect(html).toContain("#1");
      expect(html).toContain("Tabulated by");
      expect(html).toContain("Scrutinized by");
      expect(html).toContain("Approved by");
    });
  });
});
