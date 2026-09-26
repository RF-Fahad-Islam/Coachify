import { useState } from "react";
import { DollarSign, Printer, Share2, CheckCircle2, User, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useLocale } from "@/lib/i18n";

export interface StudentFeeSummary {
  studentId: string;
  studentName: string;
  rollNumber: string;
  guardianPhone: string;
  dueAmountPoisha: number;
}

export interface MobileCashierViewProps {
  onDispatchHostReceipt?: (
    receiptId: string,
    printerName?: string
  ) => Promise<{
    success: boolean;
    jobId: string;
    printer: string;
  }>;
  onRecordPayment?: (
    studentId: string,
    amountPoisha: number,
    paymentMethod: string
  ) => Promise<{
    receiptId: string;
    receiptNumber: string;
  }>;
  designatedPrinter?: string | null;
}

const mockStudents: StudentFeeSummary[] = [
  {
    studentId: "stu_001",
    studentName: "Tahmid Rahman",
    rollNumber: "R-101",
    guardianPhone: "01711223344",
    dueAmountPoisha: 250000, // 2,500 BDT
  },
  {
    studentId: "stu_002",
    studentName: "Nusrat Jahan",
    rollNumber: "R-102",
    guardianPhone: "01811998877",
    dueAmountPoisha: 180000, // 1,800 BDT
  },
  {
    studentId: "stu_003",
    studentName: "Abrar Hasan",
    rollNumber: "R-103",
    guardianPhone: "01911445566",
    dueAmountPoisha: 320000, // 3,200 BDT
  },
];

export function MobileCashierView({
  onDispatchHostReceipt,
  onRecordPayment,
  designatedPrinter,
}: MobileCashierViewProps) {
  const { formatCurrency } = useLocale();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedStudent, setSelectedStudent] = useState<StudentFeeSummary | null>(null);
  const [amountInput, setAmountInput] = useState("");
  const [paymentMethod, setPaymentMethod] = useState<"cash" | "bkash" | "nagad" | "bank">(
    "cash"
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isPrinting, setIsPrinting] = useState(false);
  const [printSuccessMsg, setPrintSuccessMsg] = useState<string | null>(null);
  const [receiptResult, setReceiptResult] = useState<{
    receiptId: string;
    receiptNumber: string;
    amount: number;
    student: StudentFeeSummary;
  } | null>(null);

  const filteredStudents = searchQuery.trim()
    ? mockStudents.filter(
        (s) =>
          s.studentName.toLowerCase().includes(searchQuery.toLowerCase()) ||
          s.rollNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
          s.guardianPhone.includes(searchQuery)
      )
    : [];

  const handleSelectStudent = (student: StudentFeeSummary) => {
    setSelectedStudent(student);
    setAmountInput((student.dueAmountPoisha / 100).toString());
  };

  const handleCollectPayment = async () => {
    if (!selectedStudent) return;
    const amountVal = parseFloat(amountInput);
    if (isNaN(amountVal) || amountVal <= 0) return;

    setIsSubmitting(true);
    setPrintSuccessMsg(null);

    try {
      const amountPoisha = Math.round(amountVal * 100);
      let receiptInfo = {
        receiptId: `rcpt_${Date.now()}`,
        receiptNumber: `REC-2026-${Math.floor(1000 + Math.random() * 9000)}`,
      };

      if (onRecordPayment) {
        receiptInfo = await onRecordPayment(
          selectedStudent.studentId,
          amountPoisha,
          paymentMethod
        );
      }

      setReceiptResult({
        receiptId: receiptInfo.receiptId,
        receiptNumber: receiptInfo.receiptNumber,
        amount: amountPoisha,
        student: selectedStudent,
      });
      setSelectedStudent(null);
      setSearchQuery("");
      setAmountInput("");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleTriggerHostPrint = async () => {
    if (!receiptResult) return;
    setIsPrinting(true);
    setPrintSuccessMsg(null);

    try {
      if (onDispatchHostReceipt) {
        const res = await onDispatchHostReceipt(
          receiptResult.receiptId,
          designatedPrinter || undefined
        );
        setPrintSuccessMsg(`Receipt dispatched to counter printer "${res.printer}"`);
      } else {
        setPrintSuccessMsg(
          `Receipt sent to counter printer "${designatedPrinter || "Epson TM-T88VI"}"`
        );
      }
    } finally {
      setIsPrinting(false);
    }
  };

  const handleWhatsAppShare = () => {
    if (!receiptResult) return;
    const msg = `Dear Guardian, Payment of ${formatCurrency(
      receiptResult.amount
    )} for ${receiptResult.student.studentName} (${receiptResult.student.rollNumber}) has been received. Receipt #${receiptResult.receiptNumber}. Thank you, Coachify.`;
    const cleanPhone = receiptResult.student.guardianPhone.replace(/\D/g, "");
    const waUrl = `https://wa.me/88${cleanPhone}?text=${encodeURIComponent(msg)}`;
    window.open(waUrl, "_blank");
  };

  return (
    <div className="flex flex-col gap-4 max-w-md mx-auto w-full p-3">
      {/* Search Header */}
      {!receiptResult && (
        <Card className="border-border shadow-sm">
          <CardHeader className="p-4 pb-2">
            <CardTitle className="text-base flex items-center gap-2">
              <DollarSign className="w-5 h-5 text-primary" />
              Mobile Fee Collection
            </CardTitle>
            <CardDescription className="text-xs">
              Search student by name, roll, or mobile number
            </CardDescription>
          </CardHeader>
          <CardContent className="p-4 pt-2 space-y-3">
            <Input
              placeholder="Search Tahmid, R-101, 01711..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="text-xs"
              data-testid="student-search-input"
            />

            {/* Student Search Dropdown list */}
            {filteredStudents.length > 0 && !selectedStudent && (
              <div className="rounded-lg border border-border divide-y divide-border bg-card max-h-48 overflow-y-auto shadow-sm">
                {filteredStudents.map((s) => (
                  <button
                    key={s.studentId}
                    type="button"
                    onClick={() => handleSelectStudent(s)}
                    className="w-full text-left p-2.5 hover:bg-muted/50 transition-colors flex items-center justify-between text-xs"
                    data-testid={`student-option-${s.studentId}`}
                  >
                    <div>
                      <div className="font-semibold text-foreground">{s.studentName}</div>
                      <div className="text-[11px] text-muted-foreground font-mono">
                        {s.rollNumber} • {s.guardianPhone}
                      </div>
                    </div>
                    <Badge
                      variant="outline"
                      className="text-primary font-mono text-[11px]"
                    >
                      Due: {formatCurrency(s.dueAmountPoisha)}
                    </Badge>
                  </button>
                ))}
              </div>
            )}

            {/* Selected Student Card */}
            {selectedStudent && (
              <div className="rounded-xl border border-primary/30 bg-primary/5 p-3 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <User className="w-4 h-4 text-primary" />
                    <span className="font-bold text-xs">
                      {selectedStudent.studentName}
                    </span>
                    <Badge variant="secondary" className="text-[10px] font-mono">
                      {selectedStudent.rollNumber}
                    </Badge>
                  </div>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => setSelectedStudent(null)}
                    className="h-6 text-[11px] text-muted-foreground"
                  >
                    Change
                  </Button>
                </div>

                {/* Amount Entry */}
                <div className="space-y-1">
                  <label className="text-[11px] text-muted-foreground font-medium">
                    Payment Amount (BDT)
                  </label>
                  <Input
                    type="number"
                    value={amountInput}
                    onChange={(e) => setAmountInput(e.target.value)}
                    className="text-base font-bold font-mono h-10"
                    placeholder="Enter amount"
                    data-testid="payment-amount-input"
                  />
                </div>

                {/* Payment Method Badges */}
                <div className="space-y-1">
                  <label className="text-[11px] text-muted-foreground font-medium">
                    Payment Channel
                  </label>
                  <div className="grid grid-cols-4 gap-1.5">
                    {(["cash", "bkash", "nagad", "bank"] as const).map((method) => (
                      <Button
                        key={method}
                        type="button"
                        size="sm"
                        variant={paymentMethod === method ? "primary" : "outline"}
                        onClick={() => setPaymentMethod(method)}
                        className="text-xs uppercase font-semibold h-8"
                        data-testid={`method-${method}`}
                      >
                        {method}
                      </Button>
                    ))}
                  </div>
                </div>

                <Button
                  className="w-full gap-2 font-semibold"
                  onClick={handleCollectPayment}
                  disabled={isSubmitting || !amountInput}
                  data-testid="collect-payment-btn"
                >
                  {isSubmitting && <RefreshCw className="w-4 h-4 animate-spin" />}
                  Confirm &amp; Issue Receipt
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Payment Success & Receipt Actions Screen */}
      {receiptResult && (
        <Card className="border-emerald-500/30 bg-emerald-500/5 shadow-md">
          <CardContent className="p-4 space-y-4 text-center">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 text-emerald-600 flex items-center justify-center mx-auto">
              <CheckCircle2 className="w-7 h-7" />
            </div>

            <div>
              <h3 className="text-base font-bold text-foreground">
                Fee Collected Successfully
              </h3>
              <p className="text-xs text-muted-foreground mt-0.5">
                Receipt #{receiptResult.receiptNumber}
              </p>
            </div>

            <div className="rounded-lg bg-card border border-border p-3 space-y-1.5 text-xs text-left">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Student:</span>
                <span className="font-semibold">{receiptResult.student.studentName}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Amount:</span>
                <span className="font-bold text-emerald-600 font-mono">
                  {formatCurrency(receiptResult.amount)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Channel:</span>
                <span className="font-mono uppercase">{paymentMethod}</span>
              </div>
            </div>

            {printSuccessMsg && (
              <div
                data-testid="print-success-alert"
                className="p-2.5 rounded bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 text-xs font-medium border border-emerald-500/30 flex items-center gap-2 justify-center"
              >
                <CheckCircle2 className="w-4 h-4 shrink-0" />
                <span>{printSuccessMsg}</span>
              </div>
            )}

            {/* Spec 0046: AC-7 Counter Thermal Print & WhatsApp Share Buttons */}
            <div className="grid grid-cols-2 gap-2 pt-1">
              <Button
                variant="outline"
                className="gap-2 border-primary/40 text-primary hover:bg-primary/10"
                onClick={handleTriggerHostPrint}
                disabled={isPrinting}
                data-testid="host-print-btn"
              >
                {isPrinting ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Printer className="w-4 h-4" />
                )}
                Print at Host
              </Button>

              <Button
                variant="outline"
                className="gap-2 border-emerald-500/40 text-emerald-600 hover:bg-emerald-500/10"
                onClick={handleWhatsAppShare}
                data-testid="whatsapp-share-btn"
              >
                <Share2 className="w-4 h-4" />
                WhatsApp
              </Button>
            </div>

            <Button
              variant="ghost"
              size="sm"
              className="text-xs text-muted-foreground w-full"
              onClick={() => {
                setReceiptResult(null);
                setPrintSuccessMsg(null);
              }}
              data-testid="new-collection-btn"
            >
              Collect Another Fee
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
