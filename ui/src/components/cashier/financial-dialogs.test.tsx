import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { VoidPaymentDialog } from "./void-payment-dialog";
import { RefundDialog } from "./refund-dialog";
import { AdjustmentDialog } from "./adjustment-dialog";
import { ExpenseEntryDialog } from "./expense-entry-dialog";
import type { PaymentReceipt } from "@/gen/rpc";

const mockReceipt: PaymentReceipt = {
  id: "p1",
  receipt_no: "RCT-2026-0001",
  student_id: "s1",
  student_name: "Rafiq Islam",
  amount_poisha: 50000,
  method: "cash",
  reference: null,
  business_date: "2026-09-22",
  received_at: "2026-09-22T10:00:00Z",
  collector_user_id: "u1",
  device_id: "d1",
  idempotency_key: "idem_1",
  status: "confirmed",
  void_reason: null,
  allocated_amount_poisha: 50000,
  unallocated_amount_poisha: 0,
  created_at: "2026-09-22T10:00:00Z",
};

describe("Financial Dialogs (Slice 2)", () => {
  it("renders VoidPaymentDialog without crashing", () => {
    const html = renderToString(
      <VoidPaymentDialog
        open={true}
        onOpenChange={() => {}}
        payment={mockReceipt}
        onSuccess={() => {}}
      />
    );
    expect(html).toBeDefined();
  });

  it("renders RefundDialog without crashing", () => {
    const html = renderToString(
      <RefundDialog
        open={true}
        onOpenChange={() => {}}
        payment={mockReceipt}
        onSuccess={() => {}}
      />
    );
    expect(html).toBeDefined();
  });

  it("renders AdjustmentDialog without crashing", () => {
    const html = renderToString(
      <AdjustmentDialog
        open={true}
        onOpenChange={() => {}}
        studentId="s1"
        studentName="Rafiq Islam"
        onSuccess={() => {}}
      />
    );
    expect(html).toBeDefined();
  });

  it("renders ExpenseEntryDialog without crashing", () => {
    const html = renderToString(
      <ExpenseEntryDialog open={true} onOpenChange={() => {}} onSuccess={() => {}} />
    );
    expect(html).toBeDefined();
  });
});
