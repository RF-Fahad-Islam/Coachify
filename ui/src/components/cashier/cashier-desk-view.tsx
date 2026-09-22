import { useState, useEffect, useRef, useCallback } from "react";
import {
  rpc,
  type StudentSearchHit,
  type CashierStudentOverview,
  type PaymentReceipt,
  type PrintableReceiptData,
  type WorkstationPrinterSettings,
} from "../../gen/rpc";
import { formatTaka, formatPhone } from "../../lib/utils";
import {
  getPrinterSettings,
  executeHtmlPrint,
  getPrintableReceipt,
} from "../../lib/printer";
import { generateThermalReceiptHtml } from "../print/thermal-receipt";
import { generateA4DualReceiptHtml } from "../print/a4-dual-receipt";
import { generateA5ReceiptHtml } from "../print/a5-receipt";
import { PrintPreviewDialog } from "../print/print-preview-dialog";
import { Card, CardHeader, CardTitle, CardContent } from "../ui/card";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Badge } from "../ui/badge";
import {
  Search,
  CreditCard,
  CheckCircle2,
  AlertCircle,
  Printer,
  History,
  User,
  RefreshCw,
  Eye,
  FileText,
  Calendar,
} from "lucide-react";

export function CashierDeskView() {
  // Search state
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<StudentSearchHit[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchDropdownOpen, setSearchDropdownOpen] = useState(false);
  const [selectedSearchIdx, setSelectedSearchIdx] = useState(0);

  // Active student state
  const [selectedStudentId, setSelectedStudentId] = useState<string | null>(null);
  const [overview, setOverview] = useState<CashierStudentOverview | null>(null);
  const [overviewLoading, setOverviewLoading] = useState(false);
  const [overviewError, setOverviewError] = useState<string | null>(null);

  // Concurrency guard alert
  const [concurrencyAlert, setConcurrencyAlert] = useState<{
    message: string;
  } | null>(null);

  // Payment form state
  const [amountTaka, setAmountTaka] = useState<string>("");
  const [paymentMethod, setPaymentMethod] = useState<string>("cash");
  const [reference, setReference] = useState<string>("");
  const [businessDate, setBusinessDate] = useState<string>(
    new Date().toISOString().slice(0, 10)
  );
  const [selectedChargeIds, setSelectedChargeIds] = useState<string[]>([]);
  const [manualAllocation, setManualAllocation] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [idempotencyKey, setIdempotencyKey] = useState<string>("");

  // Post payment receipt state
  const [lastPayment, setLastPayment] = useState<PaymentReceipt | null>(null);
  const [printableReceipt, setPrintableReceipt] = useState<PrintableReceiptData | null>(
    null
  );
  const [printSuccessNotice, setPrintSuccessNotice] = useState<string | null>(null);
  const [previewDialogOpen, setPreviewDialogOpen] = useState(false);
  const [printerSettings, setPrinterSettings] =
    useState<WorkstationPrinterSettings | null>(null);

  // Refs for keyboard navigation
  const searchInputRef = useRef<HTMLInputElement>(null);
  const amountInputRef = useRef<HTMLInputElement>(null);

  // Initialize idempotency key and printer settings
  const resetIdempotencyKey = () => {
    setIdempotencyKey(`pay-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`);
  };

  useEffect(() => {
    resetIdempotencyKey();
    void getPrinterSettings().then(setPrinterSettings);
  }, []);

  // Search input debounced query
  useEffect(() => {
    const q = searchQuery.trim();
    if (q.length < 2) {
      setSearchResults([]);
      setSearchDropdownOpen(false);
      return;
    }

    setSearchLoading(true);
    const timer = setTimeout(async () => {
      try {
        const res = await rpc("students.search", { query: q, limit: 8 });
        if (res.ok && res.data) {
          setSearchResults(res.data.matches || []);
          setSearchDropdownOpen((res.data.matches || []).length > 0);
          setSelectedSearchIdx(0);
        }
      } catch (err) {
        console.error("Student search failed:", err);
      } finally {
        setSearchLoading(false);
      }
    }, 150);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  // Load student overview
  const loadStudentOverview = useCallback(async (studentId: string) => {
    setOverviewLoading(true);
    setOverviewError(null);
    setConcurrencyAlert(null);
    setSubmitError(null);
    resetIdempotencyKey();

    try {
      const res = await rpc("ledger.get_cashier_student_overview", {
        student_id: studentId,
      });

      if (res.ok && res.data) {
        setOverview(res.data);
        setSelectedStudentId(studentId);
        // Default amount to full due if positive
        if (res.data.total_due_poisha > 0) {
          setAmountTaka(String(Math.floor(res.data.total_due_poisha / 100)));
        } else {
          setAmountTaka("");
        }
        setSelectedChargeIds([]);
        setManualAllocation(false);
      } else {
        setOverviewError(res.error?.message || "Failed to load student overview");
      }
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setOverviewError(message);
    } finally {
      setOverviewLoading(false);
    }
  }, []);

  const handleSelectStudent = (student: StudentSearchHit) => {
    setSearchQuery(`${student.student_code} - ${student.name}`);
    setSearchDropdownOpen(false);
    void loadStudentOverview(student.id);
    setTimeout(() => {
      amountInputRef.current?.focus();
      amountInputRef.current?.select();
    }, 100);
  };

  // Quick fill buttons
  const handleFillFullDue = useCallback(() => {
    if (!overview) return;
    const dueTaka = Math.max(0, Math.floor(overview.total_due_poisha / 100));
    setAmountTaka(String(dueTaka));
  }, [overview]);

  const handleFillCurrentMonth = useCallback(() => {
    if (!overview) return;
    const currentMonthPrefix = new Date().toISOString().slice(0, 7);
    const monthCharges = overview.open_charges.filter(
      (c) => c.period && c.period.startsWith(currentMonthPrefix)
    );
    const sumPoisha = monthCharges.reduce((acc, c) => acc + c.remaining_due_poisha, 0);
    if (sumPoisha > 0) {
      setAmountTaka(String(Math.floor(sumPoisha / 100)));
    } else {
      handleFillFullDue();
    }
  }, [overview, handleFillFullDue]);

  const handleFillAmount = (taka: number) => {
    setAmountTaka(String(taka));
  };

  // Submit payment
  const handleSubmitPayment = useCallback(
    async (e?: React.FormEvent) => {
      if (e) e.preventDefault();
      if (!overview || !selectedStudentId) return;

      const parsedTaka = parseInt(amountTaka, 10);
      if (isNaN(parsedTaka) || parsedTaka <= 0) {
        setSubmitError("Please enter a valid positive payment amount in Taka.");
        return;
      }

      if (
        (paymentMethod === "bkash" ||
          paymentMethod === "nagad" ||
          paymentMethod === "rocket" ||
          paymentMethod === "bank" ||
          paymentMethod === "cheque") &&
        !reference.trim()
      ) {
        setSubmitError(
          `Transaction reference or TrxID is required for ${paymentMethod.toUpperCase()} payments.`
        );
        return;
      }

      setSubmitting(true);
      setSubmitError(null);
      setConcurrencyAlert(null);

      const amountPoisha = parsedTaka * 100;
      const targetCharges =
        manualAllocation && selectedChargeIds.length > 0 ? selectedChargeIds : null;

      try {
        const res = await rpc("payments.create", {
          payment: {
            student_id: selectedStudentId,
            amount_poisha: amountPoisha,
            method: paymentMethod,
            reference: reference.trim() || null,
            business_date: businessDate || null,
            idempotency_key: idempotencyKey,
            target_charge_ids: targetCharges,
            expected_balance_poisha: overview.net_balance_poisha,
          },
        });

        if (res.ok && res.data) {
          const receiptData = res.data;
          setLastPayment(receiptData);

          // Fetch printable receipt data
          try {
            const printable = await getPrintableReceipt(receiptData.id, false);
            setPrintableReceipt(printable);

            // Silent auto print if configured on this workstation
            if (printerSettings?.auto_print_on_payment) {
              let html = "";
              if (printerSettings.page_size === "thermal_80mm") {
                html = generateThermalReceiptHtml(printable, printerSettings.footer_note);
              } else if (printerSettings.page_size === "a4_dual") {
                html = generateA4DualReceiptHtml(printable, printerSettings.footer_note);
              } else {
                html = generateA5ReceiptHtml(printable, printerSettings.footer_note);
              }

              const printRes = await executeHtmlPrint(html);
              if (printRes.success) {
                setPrintSuccessNotice("Receipt printed automatically.");
              } else {
                setPrintSuccessNotice(
                  `Payment saved safely! Printer warning: ${printRes.error || "Offline"}`
                );
              }
            }
          } catch (printFetchErr) {
            console.error("Printable fetch error:", printFetchErr);
          }

          // Refresh student ledger overview
          void loadStudentOverview(selectedStudentId);
        } else if (res.error) {
          if (res.error.code === "due_changed") {
            // Optimistic concurrency mismatch detected
            setConcurrencyAlert({
              message: res.error.message,
            });
          } else {
            setSubmitError(res.error.message || "Payment submission failed");
          }
        }
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err);
        setSubmitError(message);
      } finally {
        setSubmitting(false);
      }
    },
    [
      overview,
      selectedStudentId,
      amountTaka,
      paymentMethod,
      reference,
      manualAllocation,
      selectedChargeIds,
      businessDate,
      idempotencyKey,
      printerSettings,
      loadStudentOverview,
    ]
  );

  // Keyboard shortcut listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is currently inside an input other than amount/search
      const activeEl = document.activeElement as HTMLElement | null;
      const isInput =
        activeEl?.tagName === "INPUT" ||
        activeEl?.tagName === "TEXTAREA" ||
        activeEl?.tagName === "SELECT";

      // Alt+F or '/' to focus search
      if ((e.altKey && e.key.toLowerCase() === "f") || (e.key === "/" && !isInput)) {
        e.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
        return;
      }

      // Alt+M to focus amount input
      if (e.altKey && e.key.toLowerCase() === "m") {
        e.preventDefault();
        amountInputRef.current?.focus();
        amountInputRef.current?.select();
        return;
      }

      // Quick fills: Alt+1 (Full Due), Alt+2 (Current Month)
      if (e.altKey && e.key === "1") {
        e.preventDefault();
        handleFillFullDue();
        return;
      }
      if (e.altKey && e.key === "2") {
        e.preventDefault();
        handleFillCurrentMonth();
        return;
      }

      // Ctrl+Enter or Alt+S to submit payment
      if ((e.ctrlKey && e.key === "Enter") || (e.altKey && e.key.toLowerCase() === "s")) {
        e.preventDefault();
        void handleSubmitPayment();
        return;
      }

      // Ctrl+P to print preview or reprint latest receipt
      if (e.ctrlKey && e.key.toLowerCase() === "p") {
        if (printableReceipt) {
          e.preventDefault();
          setPreviewDialogOpen(true);
        }
        return;
      }

      // Esc to clear search or dismiss banner
      if (e.key === "Escape") {
        if (searchDropdownOpen) {
          setSearchDropdownOpen(false);
        } else if (lastPayment) {
          setLastPayment(null);
          setPrintableReceipt(null);
          setPrintSuccessNotice(null);
          searchInputRef.current?.focus();
        }
        return;
      }

      // Numeric keys 1-6 for payment method when not in a text input
      if (!isInput && overview) {
        if (e.key === "1") setPaymentMethod("cash");
        if (e.key === "2") setPaymentMethod("bkash");
        if (e.key === "3") setPaymentMethod("nagad");
        if (e.key === "4") setPaymentMethod("rocket");
        if (e.key === "5") setPaymentMethod("bank");
        if (e.key === "6") setPaymentMethod("cheque");
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    searchDropdownOpen,
    lastPayment,
    printableReceipt,
    overview,
    handleFillFullDue,
    handleFillCurrentMonth,
    handleSubmitPayment,
  ]);

  const handleReprint = async (paymentId: string) => {
    try {
      const printable = await getPrintableReceipt(paymentId, true);
      setPrintableReceipt(printable);
      setPreviewDialogOpen(true);
    } catch (err: unknown) {
      console.error("Failed to load receipt for reprint:", err);
    }
  };

  return (
    <div className="space-y-4">
      {/* Top Search & Action Bar */}
      <div className="relative">
        <div className="flex items-center gap-3">
          <div className="relative flex-1">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted-foreground">
              <Search className="h-4 w-4" />
            </div>
            <Input
              ref={searchInputRef}
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Instant Student Search: Student ID, Roll, Phone, Guardian Phone, or Name (Shortcut: Alt+F or /)"
              className="pl-9 pr-24 h-11 text-sm bg-card border-border shadow-sm"
              onFocus={() => {
                if (searchResults.length > 0) setSearchDropdownOpen(true);
              }}
              onKeyDown={(e) => {
                if (searchDropdownOpen && searchResults.length > 0) {
                  if (e.key === "ArrowDown") {
                    e.preventDefault();
                    setSelectedSearchIdx((prev) => (prev + 1) % searchResults.length);
                  } else if (e.key === "ArrowUp") {
                    e.preventDefault();
                    setSelectedSearchIdx(
                      (prev) => (prev - 1 + searchResults.length) % searchResults.length
                    );
                  } else if (e.key === "Enter") {
                    e.preventDefault();
                    if (searchResults[selectedSearchIdx]) {
                      handleSelectStudent(searchResults[selectedSearchIdx]);
                    }
                  }
                }
              }}
            />
            <div className="absolute inset-y-0 right-0 pr-3 flex items-center gap-1 pointer-events-none">
              {searchLoading ? (
                <RefreshCw className="h-4 w-4 animate-spin text-muted-foreground" />
              ) : (
                <kbd className="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground bg-muted border border-border rounded">
                  Alt+F
                </kbd>
              )}
            </div>
          </div>

          {overview && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                setSelectedStudentId(null);
                setOverview(null);
                setSearchQuery("");
                searchInputRef.current?.focus();
              }}
              className="h-11 text-xs"
            >
              Clear
            </Button>
          )}
        </div>

        {/* Search Results Dropdown */}
        {searchDropdownOpen && searchResults.length > 0 && (
          <div className="absolute z-30 left-0 right-0 mt-1 bg-popover border border-border rounded-lg shadow-xl overflow-hidden max-h-80 overflow-y-auto">
            {searchResults.map((hit, idx) => (
              <div
                key={hit.id}
                onClick={() => handleSelectStudent(hit)}
                onMouseEnter={() => setSelectedSearchIdx(idx)}
                className={`p-3 cursor-pointer border-b border-border/50 flex items-center justify-between transition-colors ${
                  idx === selectedSearchIdx
                    ? "bg-primary/10 text-foreground"
                    : "hover:bg-muted/50 text-foreground"
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className="h-9 w-9 rounded-full bg-primary/20 text-primary flex items-center justify-center font-bold text-xs">
                    {hit.student_code.slice(-2)}
                  </div>
                  <div>
                    <div className="font-semibold text-xs flex items-center gap-2">
                      <span>{hit.name}</span>
                      {hit.name_bn && (
                        <span className="text-muted-foreground text-[11px]">
                          ({hit.name_bn})
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-muted-foreground flex items-center gap-2 mt-0.5">
                      <span className="font-mono font-medium text-foreground">
                        {hit.student_code}
                      </span>
                      {hit.current_school_roll && (
                        <span>• Roll: {hit.current_school_roll}</span>
                      )}
                      <span>• Phone: {formatPhone(hit.phone_norm)}</span>
                    </div>
                  </div>
                </div>

                <div className="text-right">
                  <Badge variant="outline" className="text-[10px] capitalize">
                    {hit.status}
                  </Badge>
                  {hit.guardian_names && (
                    <div className="text-[10px] text-muted-foreground mt-0.5">
                      Guardian: {hit.guardian_names}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Post Payment Success Notification Banner */}
      {lastPayment && (
        <div className="p-4 rounded-lg bg-emerald-500/10 border-2 border-emerald-500/30 text-foreground flex items-center justify-between">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="h-6 w-6 text-emerald-600 flex-shrink-0" />
            <div>
              <div className="font-bold text-sm text-emerald-800 dark:text-emerald-300 flex items-center gap-2">
                <span>Payment Collected Successfully!</span>
                <span className="font-mono text-xs px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-900 dark:text-emerald-200">
                  {lastPayment.receipt_no}
                </span>
                <span className="text-xs font-normal text-muted-foreground">
                  ({formatTaka(lastPayment.amount_poisha)} via{" "}
                  {lastPayment.method.toUpperCase()})
                </span>
              </div>
              <p className="text-xs text-muted-foreground mt-0.5">
                {printSuccessNotice || "Payment recorded in immutable ledger."}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {printableReceipt && (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setPreviewDialogOpen(true)}
                  className="gap-1 text-xs"
                >
                  <Eye className="h-3.5 w-3.5" />
                  View Receipt
                </Button>
                <Button
                  size="sm"
                  onClick={async () => {
                    if (!printerSettings) return;
                    let html = "";
                    if (printerSettings.page_size === "thermal_80mm") {
                      html = generateThermalReceiptHtml(
                        printableReceipt,
                        printerSettings.footer_note
                      );
                    } else if (printerSettings.page_size === "a4_dual") {
                      html = generateA4DualReceiptHtml(
                        printableReceipt,
                        printerSettings.footer_note
                      );
                    } else {
                      html = generateA5ReceiptHtml(
                        printableReceipt,
                        printerSettings.footer_note
                      );
                    }
                    const res = await executeHtmlPrint(html);
                    if (res.success) {
                      setPrintSuccessNotice("Receipt printed successfully.");
                    }
                  }}
                  className="gap-1 text-xs font-semibold"
                >
                  <Printer className="h-3.5 w-3.5" />
                  Print (Ctrl+P)
                </Button>
              </>
            )}
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setLastPayment(null);
                setPrintableReceipt(null);
                setPrintSuccessNotice(null);
                searchInputRef.current?.focus();
              }}
              className="text-xs text-muted-foreground"
            >
              Done (Esc)
            </Button>
          </div>
        </div>
      )}

      {/* Optimistic Concurrency Alert */}
      {concurrencyAlert && (
        <div className="p-4 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-900 dark:text-amber-200 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <AlertCircle className="h-5 w-5 text-amber-600 flex-shrink-0" />
            <div>
              <div className="font-bold text-xs">
                Student balance changed on another workstation!
              </div>
              <p className="text-xs mt-0.5">{concurrencyAlert.message}</p>
            </div>
          </div>
          {selectedStudentId && (
            <Button
              size="sm"
              variant="outline"
              onClick={() => loadStudentOverview(selectedStudentId)}
              className="gap-1 text-xs font-semibold"
            >
              <RefreshCw className="h-3.5 w-3.5" />
              Refresh Student Ledger
            </Button>
          )}
        </div>
      )}

      {/* Overview Load Error */}
      {overviewError && (
        <div className="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-600 text-xs flex items-center gap-2">
          <AlertCircle className="h-4 w-4 flex-shrink-0" />
          <span>{overviewError}</span>
        </div>
      )}

      {/* Main Cashier Grid */}
      {overviewLoading ? (
        <Card>
          <CardContent className="p-12 flex flex-col items-center justify-center gap-3">
            <RefreshCw className="h-8 w-8 animate-spin text-primary" />
            <span className="text-xs font-medium text-muted-foreground">
              Loading student ledger overview...
            </span>
          </CardContent>
        </Card>
      ) : overview ? (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Left Column: Student Details, Open Dues & History (7 cols) */}
          <div className="lg:col-span-7 space-y-4">
            {/* Student Snapshot Card */}
            <Card>
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3.5">
                    <div className="h-12 w-12 rounded-full bg-primary/20 text-primary flex items-center justify-center font-bold text-sm">
                      {overview.photo_vault_id ? (
                        <User className="h-6 w-6" />
                      ) : (
                        overview.student_code.slice(-2)
                      )}
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h2 className="text-base font-bold text-foreground">
                          {overview.name}
                        </h2>
                        {overview.name_bn && (
                          <span className="text-xs text-muted-foreground">
                            ({overview.name_bn})
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-muted-foreground flex items-center gap-2 mt-0.5">
                        <span className="font-mono font-bold text-foreground">
                          {overview.student_code}
                        </span>
                        {overview.roll_no && (
                          <span>
                            • Roll: <strong>{overview.roll_no}</strong>
                          </span>
                        )}
                        {overview.program_name && <span>• {overview.program_name}</span>}
                        {overview.batch_name && (
                          <span>• Batch: {overview.batch_name}</span>
                        )}
                      </div>
                      {overview.guardian_name && (
                        <div className="text-[11px] text-muted-foreground mt-0.5">
                          Guardian: {overview.guardian_name}{" "}
                          {overview.guardian_phone &&
                            `(${formatPhone(overview.guardian_phone)})`}
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="text-right">
                    <div className="text-xs text-muted-foreground">
                      Net Ledger Balance
                    </div>
                    <div
                      className={`text-lg font-extrabold tabular-nums ${
                        overview.net_balance_poisha > 0
                          ? "text-red-600 dark:text-red-400"
                          : overview.net_balance_poisha < 0
                            ? "text-emerald-600 dark:text-emerald-400"
                            : "text-foreground"
                      }`}
                    >
                      {overview.net_balance_poisha > 0
                        ? `${formatTaka(overview.net_balance_poisha)} Due`
                        : overview.net_balance_poisha < 0
                          ? `${formatTaka(-overview.net_balance_poisha)} Advance`
                          : "৳ 0 Cleared"}
                    </div>
                  </div>
                </div>

                {/* Balance Metrics Row */}
                <div className="grid grid-cols-3 gap-2 mt-3 pt-3 border-t border-border text-xs">
                  <div className="p-2 rounded bg-muted/30 border border-border">
                    <div className="text-muted-foreground text-[11px]">
                      Total Unpaid Due
                    </div>
                    <div className="font-bold text-red-600 dark:text-red-400 tabular-nums mt-0.5">
                      {formatTaka(overview.total_due_poisha)}
                    </div>
                  </div>
                  <div className="p-2 rounded bg-muted/30 border border-border">
                    <div className="text-muted-foreground text-[11px]">
                      Advance Credit Deposit
                    </div>
                    <div className="font-bold text-emerald-600 dark:text-emerald-400 tabular-nums mt-0.5">
                      {formatTaka(overview.advance_credit_poisha)}
                    </div>
                  </div>
                  <div className="p-2 rounded bg-muted/30 border border-border">
                    <div className="text-muted-foreground text-[11px]">
                      Open Charge Count
                    </div>
                    <div className="font-bold text-foreground tabular-nums mt-0.5">
                      {overview.open_charges.length} Invoices
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Itemized Open Charges Table */}
            <Card>
              <CardHeader className="py-3 px-4 border-b border-border flex flex-row items-center justify-between">
                <div>
                  <CardTitle className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                    <FileText className="h-3.5 w-3.5 text-primary" />
                    Open Dues & Invoices
                  </CardTitle>
                </div>
                <div className="flex items-center gap-2">
                  <label className="text-[11px] text-muted-foreground flex items-center gap-1.5 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={manualAllocation}
                      onChange={(e) => {
                        setManualAllocation(e.target.checked);
                        if (!e.target.checked) setSelectedChargeIds([]);
                      }}
                      className="rounded border-border"
                    />
                    <span>Manual charge selection</span>
                  </label>
                </div>
              </CardHeader>
              <CardContent className="p-0">
                {overview.open_charges.length === 0 ? (
                  <div className="p-6 text-center text-xs text-muted-foreground">
                    No outstanding unpaid charges for this student. Any collection will be
                    credited to advance deposit.
                  </div>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full text-xs">
                      <thead>
                        <tr className="border-b border-border bg-muted/20 text-muted-foreground text-[11px]">
                          {manualAllocation && (
                            <th className="p-2 text-center w-8">Pick</th>
                          )}
                          <th className="p-2 text-left">Fee Head</th>
                          <th className="p-2 text-left">Period</th>
                          <th className="p-2 text-left">Due Date</th>
                          <th className="p-2 text-right">Net Charge</th>
                          <th className="p-2 text-right font-semibold text-foreground">
                            Remaining Due
                          </th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-border">
                        {overview.open_charges.map((charge) => {
                          const isOverdue =
                            charge.due_date < new Date().toISOString().slice(0, 10);
                          const isSelected = selectedChargeIds.includes(charge.id);

                          return (
                            <tr
                              key={charge.id}
                              className={`hover:bg-muted/40 transition-colors ${
                                isSelected ? "bg-primary/10" : ""
                              }`}
                            >
                              {manualAllocation && (
                                <td className="p-2 text-center">
                                  <input
                                    type="checkbox"
                                    checked={isSelected}
                                    onChange={(e) => {
                                      if (e.target.checked) {
                                        setSelectedChargeIds([
                                          ...selectedChargeIds,
                                          charge.id,
                                        ]);
                                      } else {
                                        setSelectedChargeIds(
                                          selectedChargeIds.filter(
                                            (id) => id !== charge.id
                                          )
                                        );
                                      }
                                    }}
                                  />
                                </td>
                              )}
                              <td className="p-2">
                                <div className="font-semibold text-foreground">
                                  {charge.fee_head_name}
                                </div>
                                <div className="text-[10px] text-muted-foreground capitalize">
                                  {charge.fee_head_kind}
                                </div>
                              </td>
                              <td className="p-2 text-muted-foreground">
                                {charge.period || "—"}
                              </td>
                              <td className="p-2">
                                <span>{charge.due_date}</span>
                                {isOverdue && (
                                  <span className="ml-1.5 px-1.5 py-0.2 rounded text-[9px] font-bold bg-red-500/10 text-red-600">
                                    OVERDUE
                                  </span>
                                )}
                              </td>
                              <td className="p-2 text-right text-muted-foreground tabular-nums">
                                {formatTaka(charge.net_amount_poisha)}
                              </td>
                              <td className="p-2 text-right font-bold text-red-600 dark:text-red-400 tabular-nums">
                                {formatTaka(charge.remaining_due_poisha)}
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Recent Payments History */}
            <Card>
              <CardHeader className="py-3 px-4 border-b border-border">
                <CardTitle className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                  <History className="h-3.5 w-3.5 text-primary" />
                  Recent Payments History (Last 5)
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0">
                {overview.recent_payments.length === 0 ? (
                  <div className="p-4 text-center text-xs text-muted-foreground">
                    No payment history on record.
                  </div>
                ) : (
                  <div className="divide-y divide-border">
                    {overview.recent_payments.map((p) => (
                      <div
                        key={p.id}
                        className="p-3 flex items-center justify-between text-xs hover:bg-muted/30 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <div className="h-7 w-7 rounded bg-muted flex items-center justify-center font-mono text-[11px] font-bold text-foreground">
                            #{p.receipt_no.slice(-4)}
                          </div>
                          <div>
                            <div className="font-semibold text-foreground flex items-center gap-1.5">
                              <span>{p.receipt_no}</span>
                              <span className="text-muted-foreground font-normal text-[11px]">
                                • {p.business_date}
                              </span>
                            </div>
                            <div className="text-[11px] text-muted-foreground">
                              Method: <strong className="uppercase">{p.method}</strong>
                              {p.reference && ` (${p.reference})`}
                            </div>
                          </div>
                        </div>

                        <div className="flex items-center gap-3">
                          <div className="text-right">
                            <div className="font-bold text-emerald-600 dark:text-emerald-400 tabular-nums">
                              {formatTaka(p.amount_poisha)}
                            </div>
                            <div className="text-[10px] text-muted-foreground">
                              {p.allocated_amount_poisha > 0 && "Settled Due"}
                              {p.unallocated_amount_poisha > 0 && " + Advance"}
                            </div>
                          </div>

                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleReprint(p.id)}
                            className="h-7 text-xs gap-1 text-muted-foreground hover:text-foreground"
                          >
                            <Printer className="h-3 w-3" />
                            Reprint
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Right Column: Quick Payment Form (5 cols) */}
          <div className="lg:col-span-5 space-y-4">
            <Card className="border-primary/40 shadow-lg">
              <CardHeader className="border-b border-border bg-primary/5 pb-3">
                <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
                  <CreditCard className="h-4 w-4 text-primary" />
                  Quick Payment Counter
                </CardTitle>
                <p className="text-[11px] text-muted-foreground">
                  Shortcuts: Alt+M (Amount), Alt+1 (Full Due), 1-6 (Method), Ctrl+Enter
                  (Collect)
                </p>
              </CardHeader>

              <CardContent className="p-4 space-y-4">
                {/* Amount to Collect */}
                <div className="space-y-1.5">
                  <div className="flex justify-between items-center">
                    <label className="text-xs font-bold text-foreground">
                      Amount to Collect (BDT Taka):
                    </label>
                    <span className="text-[10px] font-mono text-muted-foreground">
                      Alt+M
                    </span>
                  </div>
                  <div className="relative">
                    <span className="absolute inset-y-0 left-0 pl-3 flex items-center text-muted-foreground font-bold text-base">
                      ৳
                    </span>
                    <Input
                      ref={amountInputRef}
                      type="number"
                      min="1"
                      value={amountTaka}
                      onChange={(e) => setAmountTaka(e.target.value)}
                      placeholder="0"
                      className="pl-8 text-lg font-bold tabular-nums h-12 bg-background border-border"
                    />
                  </div>

                  {/* Quick Fill Buttons */}
                  <div className="flex flex-wrap gap-1.5 pt-1">
                    <button
                      type="button"
                      onClick={handleFillFullDue}
                      className="px-2 py-1 text-[11px] font-semibold rounded bg-muted hover:bg-muted/80 text-foreground border border-border"
                    >
                      Full Due (Alt+1)
                    </button>
                    <button
                      type="button"
                      onClick={handleFillCurrentMonth}
                      className="px-2 py-1 text-[11px] font-medium rounded bg-muted hover:bg-muted/80 text-foreground border border-border"
                    >
                      This Month (Alt+2)
                    </button>
                    <button
                      type="button"
                      onClick={() => handleFillAmount(1000)}
                      className="px-2 py-1 text-[11px] rounded bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground"
                    >
                      1,000
                    </button>
                    <button
                      type="button"
                      onClick={() => handleFillAmount(2000)}
                      className="px-2 py-1 text-[11px] rounded bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground"
                    >
                      2,000
                    </button>
                    <button
                      type="button"
                      onClick={() => handleFillAmount(5000)}
                      className="px-2 py-1 text-[11px] rounded bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground"
                    >
                      5,000
                    </button>
                  </div>
                </div>

                {/* Payment Method Selector */}
                <div className="space-y-1.5 pt-2 border-t border-border">
                  <label className="text-xs font-bold text-foreground">
                    Payment Method:
                  </label>
                  <div className="grid grid-cols-3 gap-2">
                    {[
                      { id: "cash", label: "Cash", key: "1" },
                      { id: "bkash", label: "bKash", key: "2" },
                      { id: "nagad", label: "Nagad", key: "3" },
                      { id: "rocket", label: "Rocket", key: "4" },
                      { id: "bank", label: "Bank", key: "5" },
                      { id: "cheque", label: "Cheque", key: "6" },
                    ].map((m) => (
                      <button
                        key={m.id}
                        type="button"
                        onClick={() => setPaymentMethod(m.id)}
                        className={`p-2 rounded border text-left flex items-center justify-between transition-colors ${
                          paymentMethod === m.id
                            ? "border-primary bg-primary/10 text-foreground font-bold shadow-sm"
                            : "border-border hover:bg-muted/40 text-muted-foreground"
                        }`}
                      >
                        <span className="text-xs">{m.label}</span>
                        <kbd className="px-1 py-0.2 text-[9px] font-mono text-muted-foreground bg-muted border border-border rounded">
                          {m.key}
                        </kbd>
                      </button>
                    ))}
                  </div>
                </div>

                {/* Reference & TrxID */}
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground flex items-center justify-between">
                    <span>
                      Transaction Reference / TrxID
                      {paymentMethod !== "cash" && (
                        <span className="text-red-500 ml-0.5">*</span>
                      )}
                    </span>
                    <span className="text-[10px] text-muted-foreground">
                      {paymentMethod === "cash" ? "Optional" : "Required for digital"}
                    </span>
                  </label>
                  <Input
                    type="text"
                    value={reference}
                    onChange={(e) => setReference(e.target.value)}
                    placeholder={
                      paymentMethod === "cash"
                        ? "e.g. Counter note (optional)"
                        : `Enter ${paymentMethod.toUpperCase()} TrxID / Reference`
                    }
                    className="text-xs h-9"
                  />
                </div>

                {/* Business Date */}
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                    <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
                    Collection Business Date:
                  </label>
                  <Input
                    type="date"
                    value={businessDate}
                    onChange={(e) => setBusinessDate(e.target.value)}
                    className="text-xs h-9"
                  />
                </div>

                {/* Error Banner */}
                {submitError && (
                  <div className="p-3 rounded bg-red-500/10 border border-red-500/20 text-red-600 dark:text-red-400 text-xs flex items-center gap-2">
                    <AlertCircle className="h-4 w-4 flex-shrink-0" />
                    <span>{submitError}</span>
                  </div>
                )}

                {/* Submit Action Button */}
                <div className="pt-2">
                  <Button
                    type="button"
                    onClick={() => handleSubmitPayment()}
                    disabled={submitting}
                    className="w-full h-12 text-sm font-bold gap-2 shadow-md"
                  >
                    {submitting ? (
                      <>
                        <RefreshCw className="h-4 w-4 animate-spin" />
                        <span>Processing Payment & Allocation...</span>
                      </>
                    ) : (
                      <>
                        <CheckCircle2 className="h-4 w-4" />
                        <span>Receive Payment (Ctrl+Enter)</span>
                      </>
                    )}
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      ) : (
        /* Empty State: Prompt search */
        <Card className="border-dashed">
          <CardContent className="p-16 flex flex-col items-center justify-center text-center">
            <div className="h-14 w-14 rounded-full bg-primary/10 text-primary flex items-center justify-center mb-4">
              <CreditCard className="h-7 w-7" />
            </div>
            <h3 className="text-base font-bold text-foreground">
              Fast Cashier Counter Ready
            </h3>
            <p className="text-xs text-muted-foreground max-w-md mt-1 mb-4 leading-relaxed">
              Find a student by typing their ID, Roll number, Name, or Phone in the search
              box above (or press <strong>Alt+F</strong>). View current dues and print
              receipts instantly.
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={() => searchInputRef.current?.focus()}
              className="gap-1.5 text-xs font-semibold"
            >
              <Search className="h-3.5 w-3.5" />
              Focus Search Box (Alt+F)
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Print Preview Dialog */}
      <PrintPreviewDialog
        open={previewDialogOpen}
        onOpenChange={setPreviewDialogOpen}
        receipt={printableReceipt}
        defaultLayout={printerSettings?.page_size || "thermal_80mm"}
        footerNote={printerSettings?.footer_note}
      />
    </div>
  );
}
