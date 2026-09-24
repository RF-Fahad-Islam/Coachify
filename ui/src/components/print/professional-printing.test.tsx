import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import type {
  StudentIdCardData,
  AdmitCardBatchData,
  DueStatementData,
  CashDaySummary,
} from "@/gen/rpc";
import {
  generateIdCardsHtml,
  generateSingleCardInnerHtml,
  IdCardSheet,
} from "./id-card-template";
import { generateAdmitCardsHtml, AdmitCardSheet } from "./admit-card-template";
import { generateThermalDueSlipHtml, ThermalDueSlip } from "./thermal-due-slip-template";
import {
  generateThermalCashCloseHtml,
  ThermalCashCloseSlip,
} from "./thermal-cash-close-template";
import {
  generateAttendanceSheetHtml,
  AttendanceSheetPrintable,
} from "./attendance-sheet-printable";
import {
  generateFinancialLedgerHtml,
  FinancialLedgerPrintable,
} from "./financial-ledger-template";
import { DocumentPrintPreviewModal } from "./document-print-preview-modal";

const mockIdCardData: StudentIdCardData[] = [
  {
    student_id: "stu-1",
    student_code: "STU-2026-001",
    full_name_en: "Tanvir Rahman",
    full_name_bn: "তানভীর রহমান",
    gender: "male",
    blood_group: "B+",
    emergency_phone: "01712345678",
    primary_batch_name: "Batch Alpha HSC",
    academic_session: "Session 2026",
    photo_data_uri: null, // missing photo -> tests AC-8 silhouette fallback
    barcode_payload: "STU-2026-001",
  },
  {
    student_id: "stu-2",
    student_code: "STU-2026-002",
    full_name_en: "Nusrat Jahan",
    full_name_bn: "নুসরাত জাহান",
    gender: "female",
    blood_group: "O+",
    emergency_phone: "01812345679",
    primary_batch_name: "Batch Alpha HSC",
    academic_session: "Session 2026",
    photo_data_uri:
      "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
    barcode_payload: "STU-2026-002",
  },
];

const mockAdmitCardBatch: AdmitCardBatchData = {
  exam_id: "exam-99",
  exam_title: "HSC Term 1 Final Exam 2026",
  academic_session: "Session 2026",
  center_instructions: [
    "Candidates must arrive 15 minutes before exam.",
    "Smartphones are strictly prohibited.",
    "পরীক্ষার হলে কোনো প্রকার অসদুপায় অবলম্বন নিষিদ্ধ।",
  ],
  students: [
    {
      student_id: "stu-1",
      student_code: "STU-2026-001",
      student_name: "Tanvir Rahman",
      batch_name: "Batch Alpha HSC",
      roll_number: "12",
      photo_data_uri: null,
      qr_verification_payload: "ADM|exam-99|STU-2026-001|VERIFIED",
      subjects: [
        {
          subject_id: "sub-1",
          subject_name: "Physics Paper 1",
          exam_date: "2026-10-15",
          start_time: "10:00 AM",
          room_number: "Room 301",
        },
        {
          subject_id: "sub-2",
          subject_name: "Chemistry Paper 1",
          exam_date: "2026-10-17",
          start_time: "10:00 AM",
          room_number: "Room 302",
        },
      ],
    },
  ],
};

const mockDueStatement: DueStatementData = {
  institution_name: "Coachify Academy",
  institution_name_bn: "কোচিফাই একাডেমি",
  institution_address: "Farmgate, Dhaka",
  institution_phone: "01712345678",
  student_id: "stu-1",
  student_code: "STU-2026-001",
  student_name: "Tanvir Rahman",
  student_name_bn: "তানভীর রহমান",
  guardian_name: "Rafiqul Islam",
  guardian_phone: "01712345678",
  batch_name: "Batch Alpha HSC",
  roll_no: "12",
  statement_date: "2026-10-10",
  total_charged_poisha: 500000,
  total_concession_poisha: 0,
  total_paid_poisha: 200000,
  total_adjusted_poisha: 0,
  net_due_poisha: 300000,
  amount_in_words_en: "Three Thousand Taka Only",
  amount_in_words_bn: "তিন হাজার টাকা মাত্র",
  payment_instructions: "Pay at counter",
  itemized_charges: [
    {
      id: "ch-1",
      fee_head_name: "Tuition Fee October",
      batch_name: "Batch Alpha HSC",
      billing_month: "2026-10",
      due_date: "2026-10-10",
      amount_poisha: 300000,
      paid_poisha: 0,
      balance_poisha: 300000,
      status: "unpaid",
    },
  ],
  recent_payments: [],
};

const mockCashDaySummary: CashDaySummary = {
  business_date: "2026-09-23",
  is_closed: true,
  closed_at: "2026-09-23T18:00:00Z",
  closed_by: "Fahad Islam",
  opening_cash_poisha: 100000,
  cash_in_poisha: 300000,
  cash_out_poisha: 50000,
  expected_closing_poisha: 350000,
  counted_closing_poisha: 350000,
  difference_poisha: 0,
  note: "Day close balanced",
  collector_breakdown: [
    {
      collector_user_id: "user-1",
      collector_name: "Fahad Islam",
      receipt_count: 10,
      total_amount_poisha: 500000,
    },
  ],
  digital_collections: [
    {
      method: "bkash",
      count: 4,
      total_amount_poisha: 200000,
    },
  ],
  expenses_by_category: [
    {
      category_id: "cat-1",
      category_name: "Office Supplies",
      count: 2,
      total_amount_poisha: 50000,
    },
  ],
};

describe("Professional Printing System - Template Tests", () => {
  describe("Student ID Card Templates (AC-1, AC-8, AC-9)", () => {
    it("generates single PVC card HTML with CR80 dimension and barcode", () => {
      const html = generateIdCardsHtml(mockIdCardData, { layoutMode: "single_pvc" });
      expect(html).toContain("85.6mm 53.98mm");
      expect(html).toContain("STU-2026-001");
      expect(html).toContain("Tanvir Rahman");
      expect(html).toContain("তানভীর রহমান");
      expect(html).toContain("B+");
      expect(html).toContain("01712345678");
    });

    it("falls back to vector silhouette SVG when student photo is missing (AC-8)", () => {
      const cardHtml = generateSingleCardInnerHtml(mockIdCardData[0]);
      expect(cardHtml).toContain('<svg viewBox="0 0 64 64"');
      expect(cardHtml).not.toContain("<img src=");
    });

    it("renders actual photo when photo_data_uri is provided", () => {
      const cardHtml = generateSingleCardInnerHtml(mockIdCardData[1]);
      expect(cardHtml).toContain('<img src="data:image/png;base64,');
    });

    it("generates A4 8-card sheet with crop marks (AC-1)", () => {
      const html = generateIdCardsHtml(mockIdCardData, { layoutMode: "a4_sheet" });
      expect(html).toContain("size: A4 portrait");
      expect(html).toContain("grid-template-columns: repeat(2, 85.6mm)");
      expect(html).toContain("border-top: 1px dashed");
      expect(html).toContain("Crop Mark Cut Guides");
    });

    it("renders IdCardSheet preview component without error", () => {
      const html = renderToString(<IdCardSheet cards={mockIdCardData} />);
      expect(html).toContain("Previewing A4 8-card sheet layout");
    });
  });

  describe("Exam Admit Card Templates (AC-2, AC-9)", () => {
    it("generates single A5 admit card with routine table and center instructions", () => {
      const html = generateAdmitCardsHtml(mockAdmitCardBatch, {
        layoutMode: "a5_single",
      });
      expect(html).toContain("size: A5 portrait");
      expect(html).toContain("HSC Term 1 Final Exam 2026");
      expect(html).toContain("Physics Paper 1");
      expect(html).toContain("Chemistry Paper 1");
      expect(html).toContain("Room 301");
      expect(html).toContain("Candidates must arrive 15 minutes before exam.");
      expect(html).toContain("পরীক্ষার হলে কোনো প্রকার অসদুপায় অবলম্বন নিষিদ্ধ।");
    });

    it("generates A4 dual layout with cut guide divider (AC-2)", () => {
      const html = generateAdmitCardsHtml(mockAdmitCardBatch, { layoutMode: "a4_dual" });
      expect(html).toContain("size: A4 portrait");
      expect(html).toContain("✂ Cut here to separate admit cards");
    });

    it("renders AdmitCardSheet preview component without error", () => {
      const html = renderToString(<AdmitCardSheet batchData={mockAdmitCardBatch} />);
      expect(html).toContain("Previewing Admit Cards for");
      expect(html).toContain("HSC Term 1 Final Exam 2026");
    });
  });

  describe("Thermal Roll Due Slip (AC-4)", () => {
    it("generates 80mm continuous roll due slip HTML with Bangla currency", () => {
      const html = generateThermalDueSlipHtml(mockDueStatement);
      expect(html).toContain("size: 80mm auto");
      expect(html).toContain("FEE DUE REMINDER SLIP");
      expect(html).toContain("STU-2026-001");
      expect(html).toContain("Tuition Fee October");
      expect(html).toContain("৳");
      expect(html).toContain("*STU-2026-001*");
    });

    it("renders ThermalDueSlip preview component without error", () => {
      const html = renderToString(<ThermalDueSlip data={mockDueStatement} />);
      expect(html).toContain("Previewing 80mm Continuous Roll Due Slip");
    });
  });

  describe("Thermal Cash Close Summary (AC-4)", () => {
    it("generates 80mm daily cash close summary with reconciliation details", () => {
      const html = generateThermalCashCloseHtml(mockCashDaySummary);
      expect(html).toContain("DAILY CASH CLOSE");
      expect(html).toContain("Physical Cash In Drawer");
      expect(html).toContain("BALANCED");
      expect(html).toContain("bkash");
      expect(html).toContain("Fahad Islam");
    });

    it("renders ThermalCashCloseSlip preview component without error", () => {
      const html = renderToString(<ThermalCashCloseSlip summary={mockCashDaySummary} />);
      expect(html).toContain("Previewing 80mm Continuous Roll Daily Cash Close Slip");
    });
  });

  describe("Attendance Sheet Register", () => {
    it("generates A4 landscape attendance register with 31 days grid", () => {
      const html = generateAttendanceSheetHtml({
        batch_name: "Batch Alpha HSC",
        program_name: "HSC Science 2026",
        month: "October 2026",
        days_in_month: 31,
        students: [
          {
            student_id: "stu-1",
            student_code: "STU-2026-001",
            student_name: "Tanvir Rahman",
            roll: 12,
          },
        ],
      });
      expect(html).toContain("size: A4 landscape");
      expect(html).toContain("BATCH ATTENDANCE REGISTER");
      expect(html).toContain("Batch Alpha HSC");
      expect(html).toContain("October 2026");
      expect(html).toContain("STU-2026-001");
    });

    it("renders AttendanceSheetPrintable preview component without error", () => {
      const html = renderToString(
        <AttendanceSheetPrintable
          data={{
            batch_name: "Batch Alpha HSC",
            month: "October 2026",
            students: [],
          }}
        />
      );
      expect(html).toContain("Previewing A4 Landscape Attendance Register");
    });
  });

  describe("Financial Ledger Statement", () => {
    it("generates A4 financial ledger statement with debit, credit, balance columns", () => {
      const html = generateFinancialLedgerHtml({
        account_name: "Tanvir Rahman",
        account_code: "STU-2026-001",
        phone: "01712345678",
        batch_name: "Batch Alpha HSC",
        entries: [
          {
            id: "entry-1",
            entry_date: "2026-10-01",
            reference_no: "INV-001",
            description: "October 2026 Tuition Fee",
            debit_poisha: 250000,
            credit_poisha: 0,
            running_balance_poisha: 250000,
          },
          {
            id: "entry-2",
            entry_date: "2026-10-05",
            reference_no: "REC-001",
            description: "Tuition Fee Payment (bKash)",
            debit_poisha: 0,
            credit_poisha: 250000,
            running_balance_poisha: 0,
          },
        ],
        total_debit_poisha: 250000,
        total_credit_poisha: 250000,
        net_balance_poisha: 0,
      });
      expect(html).toContain("FINANCIAL LEDGER STATEMENT");
      expect(html).toContain("STU-2026-001");
      expect(html).toContain("October 2026 Tuition Fee");
      expect(html).toContain("INV-001");
      expect(html).toContain("REC-001");
    });

    it("renders FinancialLedgerPrintable preview component without error", () => {
      const html = renderToString(
        <FinancialLedgerPrintable
          data={{
            account_name: "Tanvir Rahman",
            account_code: "STU-2026-001",
            entries: [],
            total_debit_poisha: 0,
            total_credit_poisha: 0,
            net_balance_poisha: 0,
          }}
        />
      );
      expect(html).toContain("Previewing A4 Financial Ledger Statement");
    });
  });

  describe("DocumentPrintPreviewModal (AC-10)", () => {
    it("safely handles closed preview modal without throwing error in SSR", () => {
      const html = renderToString(
        <DocumentPrintPreviewModal
          open={false}
          onOpenChange={() => {}}
          title="Print ID Cards"
          documentType="id_card"
          totalItems={8}
          htmlContent="<div>Test ID Card HTML</div>"
        />
      );
      expect(html).toBe("");
    });
  });
});
