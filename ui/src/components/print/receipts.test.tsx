import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import type { PrintableReceiptData } from "../../gen/rpc";
import { ThermalReceipt, generateThermalReceiptHtml } from "./thermal-receipt";
import { A4DualReceipt, generateA4DualReceiptHtml } from "./a4-dual-receipt";
import { A5Receipt, generateA5ReceiptHtml } from "./a5-receipt";
import { PrintPreviewDialog } from "./print-preview-dialog";

const mockReceipt: PrintableReceiptData = {
  receipt_no: "REC-20260424-0042",
  business_date: "2026-04-24",
  received_at: "2026-04-24T10:30:00Z",
  student_id: "stu-123",
  student_code: "STU-2026-0042",
  student_name: "Tanvir Ahmed",
  student_name_bn: "তানভীর আহমেদ",
  guardian_name: "Rafiqul Islam",
  guardian_phone: "01711234567",
  batch_name: "Physics Model Batch A",
  program_name: "HSC 2026 Academic",
  roll_no: "12",
  amount_poisha: 250000,
  amount_in_words_en: "Two Thousand Five Hundred Taka Only",
  amount_in_words_bn: "দুই হাজার পাঁচ শত টাকা মাত্র",
  method: "bkash",
  reference: "TRX998877",
  collector_name: "Shahadat Hossain",
  institution_name: "Apex Academic Coaching Centre",
  institution_name_bn: "এপেক্স একাডেমিক কোচিং সেন্টার",
  institution_address: "Farmgate, Dhaka-1215",
  institution_phone: "01700000000",
  institution_email: "info@apexcoaching.edu.bd",
  institution_website: "www.apexcoaching.edu.bd",
  balance_after_poisha: 50000,
  items: [
    {
      fee_head_name: "Monthly Tuition Fee",
      fee_head_name_bn: "মাসিক বেতন",
      period: "2026-04",
      amount_poisha: 200000,
    },
    {
      fee_head_name: "Model Test & Notes Fee",
      fee_head_name_bn: "নোট ও মডেল টেস্ট",
      period: null,
      amount_poisha: 50000,
    },
  ],
  is_reprint: false,
};

describe("Receipt Layout Components", () => {
  it("renders 80mm continuous thermal receipt with English and Bengali details", () => {
    const html = renderToString(
      <ThermalReceipt receipt={mockReceipt} footerNote="Keep this slip safe." />
    );
    expect(html).toContain("REC-20260424-0042");
    expect(html).toContain("STU-2026-0042");
    expect(html).toContain("Tanvir Ahmed");
    expect(html).toContain("তানভীর আহমেদ");
    expect(html).toContain("Monthly Tuition Fee");
    expect(html).toContain("Two Thousand Five Hundred Taka Only");
    expect(html).toContain("দুই হাজার পাঁচ শত টাকা মাত্র");
    expect(html).toContain("bkash");
    expect(html).toContain("TRX998877");
    expect(html).toContain("Helpline: 01700000000");
    expect(html).toContain("www.apexcoaching.edu.bd");
  });

  it("generates self contained HTML for silent 80mm thermal printing", () => {
    const html = generateThermalReceiptHtml(mockReceipt, "Keep this slip safe.");
    expect(html).toContain("<!DOCTYPE html>");
    expect(html).toContain("@page {");
    expect(html).toContain("80mm");
    expect(html).toContain("REC-20260424-0042");
    expect(html).toContain("Tanvir Ahmed");
  });

  it("renders A4 dual copy receipt with perforation divider and student and office copies", () => {
    const html = renderToString(
      <A4DualReceipt receipt={mockReceipt} footerNote="Thank you." />
    );
    expect(html).toContain("STUDENT COPY");
    expect(html).toContain("শিক্ষার্থী কপি");
    expect(html).toContain("OFFICE COPY");
    expect(html).toContain("অফিস কপি");
    expect(html).toContain("CUT HERE / ছিঁড়ে নিন");
    expect(html).toContain("✂");
    expect(html).toContain("OFFICIAL SEAL");
    expect(html).toContain("অফিসিয়াল সিল");
    expect(html).toContain("Authorized Officer");
  });

  it("generates self contained HTML for A4 dual copy printing", () => {
    const html = generateA4DualReceiptHtml(mockReceipt, "Thank you.");
    expect(html).toContain("STUDENT COPY");
    expect(html).toContain("OFFICE COPY");
    expect(html).toContain("divider-badge");
    expect(html).toContain("OFFICIAL SEAL");
    expect(html).toContain("info@apexcoaching.edu.bd");
  });

  it("renders A5 single voucher receipt with proper signature blocks and emblem logo", () => {
    const html = renderToString(
      <A5Receipt receipt={mockReceipt} footerNote="Authorized voucher." />
    );
    expect(html).toContain("MONEY RECEIPT");
    expect(html).toContain("Student / Guardian");
    expect(html).toContain("Authorized Officer");
    expect(html).toContain("OFFICIAL SEAL");
    expect(html).toContain("REC-20260424-0042");
    expect(html).toContain("www.apexcoaching.edu.bd");
  });

  it("generates self contained HTML for A5 single voucher printing", () => {
    const html = generateA5ReceiptHtml(mockReceipt, "Authorized voucher.");
    expect(html).toContain("A5 portrait");
    expect(html).toContain("REC-20260424-0042");
    expect(html).toContain("OFFICIAL SEAL");
    expect(html).toContain("info@apexcoaching.edu.bd");
  });

  it("safely handles null receipt in print preview dialog (covers AC-5)", () => {
    const html = renderToString(
      <PrintPreviewDialog
        open={false}
        onOpenChange={() => {}}
        receipt={null}
        defaultLayout="thermal_80mm"
      />
    );
    expect(html).toBe("");
  });
});
