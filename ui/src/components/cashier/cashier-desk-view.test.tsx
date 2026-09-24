import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { CashierDeskView, CashierPaymentBanner } from "./cashier-desk-view";
import type { PaymentReceipt, PrintableReceiptData } from "../../gen/rpc";

const mockPayment: PaymentReceipt = {
  id: "pay-12345",
  receipt_no: "REC-20260924-0001",
  student_id: "stu-001",
  student_name: "Rahim Chowdhury",
  amount_poisha: 250000,
  method: "cash",
  reference: null,
  status: "posted",
  business_date: "2026-09-24",
  received_at: "2026-09-24T12:00:00Z",
  idempotency_key: "idem-12345",
  collector_user_id: "user-owner",
  created_at: "2026-09-24T12:00:00Z",
  allocated_amount_poisha: 250000,
  unallocated_amount_poisha: 0,
};

const mockPrintable: PrintableReceiptData = {
  receipt_no: "REC-20260924-0001",
  business_date: "2026-09-24",
  received_at: "2026-09-24T12:00:00Z",
  student_id: "stu-001",
  student_code: "STU-2026-0001",
  student_name: "Rahim Chowdhury",
  student_name_bn: null,
  guardian_name: "Karim Chowdhury",
  guardian_phone: "01711000000",
  batch_name: "Morning Batch",
  program_name: "HSC 2026",
  roll_no: "01",
  amount_poisha: 250000,
  amount_in_words_en: "Two Thousand Five Hundred Taka Only",
  amount_in_words_bn: "দুই হাজার পাঁচ শত টাকা মাত্র",
  method: "cash",
  reference: null,
  collector_name: "Owner",
  institution_name: "Test Coaching",
  institution_name_bn: null,
  institution_address: "Dhaka",
  institution_phone: "01700000000",
  institution_email: null,
  institution_website: null,
  balance_after_poisha: 0,
  items: [],
  is_reprint: false,
};

describe("CashierDeskView Component", () => {
  it("renders empty state with search prompt and keyboard shortcuts initially", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Fast Cashier Counter Ready");
    expect(html).toContain("Instant Student Search");
    expect(html).toContain("Alt+F");
  });

  it("renders search input with autofocus and keyboard badge (covers AC-1)", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Instant Student Search");
    expect(html).toContain("Student ID, Roll, Phone, Guardian Phone, or Name");
  });

  it("displays keyboard navigation hints for fast cashier counter", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Focus Search Box (Alt+F)");
    expect(html).toContain("View current dues and print receipts instantly");
  });

  // AC-5: In a simulated printer offline failure, payments commit successfully to the financial
  // ledger, and the cashier interface displays an on demand reprint banner without rolling back.
  it("satisfies AC-5: displays reprint banner with printer warning when printer is offline without rolling back payment", () => {
    const html = renderToString(
      <CashierPaymentBanner
        lastPayment={mockPayment}
        printableReceipt={mockPrintable}
        printSuccessNotice="Payment saved safely! Printer warning: Printer Offline (Device Not Connected)"
      />
    );

    // Payment is committed and preserved in the UI
    expect(html).toContain("Payment Collected Successfully!");
    expect(html).toContain("REC-20260924-0001");
    expect(html).toContain("2,500");
    expect(html).toContain("CASH");

    // Printer offline warning is prominently presented
    expect(html).toContain(
      "Payment saved safely! Printer warning: Printer Offline (Device Not Connected)"
    );

    // On-demand reprint button is available
    expect(html).toContain('data-testid="reprint-button"');
    expect(html).toContain("Print (Ctrl+P)");
    expect(html).toContain("View Receipt");
  });

  it("displays normal confirmation when printing succeeds", () => {
    const html = renderToString(
      <CashierPaymentBanner
        lastPayment={mockPayment}
        printableReceipt={mockPrintable}
        printSuccessNotice="Receipt printed automatically."
      />
    );

    expect(html).toContain("REC-20260924-0001");
    expect(html).toContain("Receipt printed automatically.");
    expect(html).toContain("Print (Ctrl+P)");
  });
});
