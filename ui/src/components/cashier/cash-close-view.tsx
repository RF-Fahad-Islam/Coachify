import { useState, useEffect, useCallback } from "react";
import { rpc, type CashDaySummary, type ExpenseItem } from "@/gen/rpc";
import { formatTaka } from "@/lib/utils";
import { executeHtmlPrint } from "@/lib/printer";
import { generateCashCloseSheetHtml } from "../print/cash-close-sheet-template";
import { PrintPreviewDialog } from "../print/print-preview-dialog";
import { ExpenseEntryDialog } from "./expense-entry-dialog";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Lock,
  Unlock,
  Printer,
  Plus,
  RefreshCw,
  AlertTriangle,
  Receipt,
  Loader2,
  Trash2,
  Calendar,
} from "lucide-react";

export function CashCloseView() {
  const [businessDate, setBusinessDate] = useState(() =>
    new Date().toISOString().slice(0, 10)
  );

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<CashDaySummary | null>(null);
  const [expenses, setExpenses] = useState<ExpenseItem[]>([]);

  // Counted closing input state
  const [countedTaka, setCountedTaka] = useState("");
  const [closingNote, setClosingNote] = useState("");
  const [closingDay, setClosingDay] = useState(false);

  // Dialogs
  const [expenseDialogOpen, setExpenseDialogOpen] = useState(false);
  const [previewOpen, setPreviewOpen] = useState(false);
  const [previewHtml, setPreviewHtml] = useState("");

  // Voiding expense
  const [voidingExpenseId, setVoidingExpenseId] = useState<string | null>(null);

  const loadDayData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // First ensure today's cash day is open or retrieve summary
      const [sumRes, expRes] = await Promise.all([
        rpc("cash_days.get_summary", { business_date: businessDate }),
        rpc("expenses.list", {
          filter: {
            business_date: businessDate,
            category_id: null,
            from_date: null,
            to_date: null,
            method: null,
            status: null,
            limit: 50,
            offset: 0,
          },
        }),
      ]);

      if (sumRes.ok && sumRes.data) {
        setSummary(sumRes.data);
        if (
          sumRes.data.counted_closing_poisha !== null &&
          sumRes.data.counted_closing_poisha !== undefined
        ) {
          setCountedTaka((sumRes.data.counted_closing_poisha / 100).toFixed(2));
        } else {
          setCountedTaka("");
        }
        if (sumRes.data.note) {
          setClosingNote(sumRes.data.note);
        }
      } else {
        setError(sumRes.error?.message || "Failed to load cash day summary");
      }

      if (expRes.ok && expRes.data) {
        setExpenses(expRes.data);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [businessDate]);

  useEffect(() => {
    void loadDayData();
  }, [loadDayData]);

  const handleCloseCashDay = async () => {
    if (!summary) return;
    const numTaka = parseFloat(countedTaka);
    if (isNaN(numTaka) || numTaka < 0) {
      setError("Please enter the physical cash count in the drawer");
      return;
    }

    const countedPoisha = Math.round(numTaka * 100);
    const diff = countedPoisha - summary.expected_closing_poisha;

    if (diff !== 0 && !closingNote.trim()) {
      setError(
        "There is a cash difference! Please provide an explanation note before closing."
      );
      return;
    }

    setClosingDay(true);
    setError(null);

    try {
      const res = await rpc("cash_days.close", {
        input: {
          business_date: businessDate,
          counted_closing_poisha: countedPoisha,
          note: closingNote.trim() || null,
        },
      });

      if (res.ok && res.data) {
        setSummary(res.data);
        // Automatically open print sheet
        const html = generateCashCloseSheetHtml(res.data);
        setPreviewHtml(html);
        setPreviewOpen(true);
      } else {
        setError(res.error?.message || "Failed to close cash day");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setClosingDay(false);
    }
  };

  const handleVoidExpense = async (expenseId: string) => {
    const reason = prompt("Please enter the reason for voiding this expense voucher:");
    if (!reason || !reason.trim()) return;

    setVoidingExpenseId(expenseId);
    try {
      const res = await rpc("expenses.void", {
        expense_id: expenseId,
        reason: reason.trim(),
      });
      if (res.ok) {
        void loadDayData();
      } else {
        alert(res.error?.message || "Failed to void expense");
      }
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      setVoidingExpenseId(null);
    }
  };

  const handlePrintSheet = () => {
    if (!summary) return;
    const html = generateCashCloseSheetHtml(summary);
    setPreviewHtml(html);
    setPreviewOpen(true);
  };

  const numCounted = parseFloat(countedTaka);
  const countedPoisha = !isNaN(numCounted) ? Math.round(numCounted * 100) : null;
  const discrepancyPoisha =
    summary && countedPoisha !== null
      ? countedPoisha - summary.expected_closing_poisha
      : null;

  return (
    <div className="space-y-6">
      {/* Top Controls: Date Selector, Refresh, Print, Record Expense */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3 p-4 bg-card rounded-[var(--radius-lg)] border border-border">
        <div className="flex items-center gap-3 w-full sm:w-auto">
          <div className="flex items-center gap-2">
            <Calendar className="h-4 w-4 text-muted-foreground" />
            <span className="text-xs font-semibold text-foreground">Business Date:</span>
          </div>
          <Input
            type="date"
            value={businessDate}
            onChange={(e) => setBusinessDate(e.target.value)}
            className="w-40 text-xs h-9"
          />
          {summary && (
            <Badge
              variant={summary.is_closed ? "default" : "outline"}
              className={`text-xs px-2.5 py-1 gap-1.5 ${
                summary.is_closed
                  ? "bg-muted text-muted-foreground border-border"
                  : "bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-300"
              }`}
            >
              {summary.is_closed ? (
                <Lock className="h-3 w-3" />
              ) : (
                <Unlock className="h-3 w-3" />
              )}
              {summary.is_closed ? "Closed & Locked" : "Day Open"}
            </Badge>
          )}
        </div>

        <div className="flex items-center gap-2 self-end sm:self-auto">
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadDayData()}
            disabled={loading}
            className="h-9 text-xs gap-1.5"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>

          <Button
            variant="outline"
            size="sm"
            onClick={handlePrintSheet}
            disabled={!summary}
            className="h-9 text-xs gap-1.5"
          >
            <Printer className="h-3.5 w-3.5" />
            Print Sheet
          </Button>

          {summary && !summary.is_closed && (
            <Button
              size="sm"
              onClick={() => setExpenseDialogOpen(true)}
              className="h-9 text-xs gap-1.5"
            >
              <Plus className="h-3.5 w-3.5" />
              Record Expense
            </Button>
          )}
        </div>
      </div>

      {error && (
        <div className="p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20 flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Main Drawer Balancing Card & Reconciliation */}
      {summary && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left 2 Cols: Cash Drawer Balancing */}
          <div className="lg:col-span-2 space-y-6">
            <Card className="border-t-4 border-t-primary shadow-xs">
              <CardHeader className="pb-3 border-b border-border">
                <CardTitle className="text-base font-bold flex items-center justify-between">
                  <span>Physical Cash Drawer Balancing</span>
                  <span className="text-xs font-normal text-muted-foreground">
                    Cash only • Excludes bKash/Bank
                  </span>
                </CardTitle>
              </CardHeader>
              <CardContent className="p-6 space-y-4">
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                  <div className="p-3.5 rounded-[var(--radius-md)] bg-muted/40 border border-border">
                    <span className="text-[11px] font-semibold text-muted-foreground uppercase">
                      Opening Cash
                    </span>
                    <div className="text-xl font-mono font-bold text-foreground mt-1">
                      {formatTaka(summary.opening_cash_poisha)}
                    </div>
                    <span className="text-[10px] text-muted-foreground">
                      From previous day
                    </span>
                  </div>

                  <div className="p-3.5 rounded-[var(--radius-md)] bg-success/10 border border-success/20">
                    <span className="text-[11px] font-semibold text-success uppercase">
                      + Cash In (Tuition)
                    </span>
                    <div className="text-xl font-mono font-bold text-success mt-1">
                      +{formatTaka(summary.cash_in_poisha)}
                    </div>
                    <span className="text-[10px] text-muted-foreground">
                      Cash receipts
                    </span>
                  </div>

                  <div className="p-3.5 rounded-[var(--radius-md)] bg-destructive/10 border border-destructive/20">
                    <span className="text-[11px] font-semibold text-destructive uppercase">
                      - Cash Out (Expenses)
                    </span>
                    <div className="text-xl font-mono font-bold text-destructive mt-1">
                      -{formatTaka(summary.cash_out_poisha)}
                    </div>
                    <span className="text-[10px] text-muted-foreground">
                      Petty cash & refunds
                    </span>
                  </div>
                </div>

                <div className="p-4 rounded-[var(--radius-lg)] bg-primary-subtle border border-primary-border flex items-center justify-between">
                  <div>
                    <span className="text-xs font-bold text-primary uppercase tracking-wide">
                      Expected Drawer Cash
                    </span>
                    <p className="text-[11px] text-muted-foreground">
                      Opening + Cash In - Cash Out
                    </p>
                  </div>
                  <div className="text-2xl font-mono font-extrabold text-foreground">
                    {formatTaka(summary.expected_closing_poisha)}
                  </div>
                </div>

                {/* Closing Drawer Section */}
                <div className="pt-2 border-t border-border space-y-3">
                  <h4 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Physical Drawer Count & Verification
                  </h4>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 items-center">
                    <div className="space-y-1.5">
                      <label
                        htmlFor="counted-cash"
                        className="text-xs font-medium text-foreground"
                      >
                        Counted Cash in Drawer (৳)
                      </label>
                      <Input
                        id="counted-cash"
                        type="number"
                        step="any"
                        min="0"
                        placeholder="e.g. 15400"
                        value={countedTaka}
                        onChange={(e) => setCountedTaka(e.target.value)}
                        disabled={summary.is_closed}
                        className="font-mono font-bold text-base h-10"
                      />
                    </div>

                    <div className="p-3 rounded-[var(--radius-md)] border bg-card flex flex-col justify-center">
                      <span className="text-[11px] font-semibold text-muted-foreground">
                        Calculated Discrepancy
                      </span>
                      {discrepancyPoisha !== null ? (
                        <div
                          className={`text-lg font-mono font-extrabold mt-0.5 ${
                            discrepancyPoisha === 0
                              ? "text-success"
                              : discrepancyPoisha > 0
                                ? "text-primary"
                                : "text-destructive"
                          }`}
                        >
                          {discrepancyPoisha === 0
                            ? "Balanced (৳ 0)"
                            : discrepancyPoisha > 0
                              ? `+${formatTaka(discrepancyPoisha)} Surplus`
                              : `-${formatTaka(-discrepancyPoisha)} Shortage`}
                        </div>
                      ) : (
                        <span className="text-xs text-muted-foreground italic mt-0.5">
                          Enter counted cash
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="space-y-1.5">
                    <label
                      htmlFor="closing-note"
                      className="text-xs font-medium text-foreground"
                    >
                      Closing Note / Discrepancy Explanation
                    </label>
                    <Input
                      id="closing-note"
                      placeholder="e.g. All counted and reconciled. Shortage of ৳20 due to rounding."
                      value={closingNote}
                      onChange={(e) => setClosingNote(e.target.value)}
                      disabled={summary.is_closed}
                      className="text-xs"
                    />
                  </div>

                  {!summary.is_closed && (
                    <div className="pt-2 flex justify-end">
                      <Button
                        onClick={() => void handleCloseCashDay()}
                        disabled={closingDay || !countedTaka}
                        className="gap-2 bg-destructive hover:bg-destructive/90 text-destructive-foreground font-semibold"
                      >
                        {closingDay ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <Lock className="h-4 w-4" />
                        )}
                        Finalize & Close Cash Day
                      </Button>
                    </div>
                  )}

                  {summary.is_closed && (
                    <div className="p-3 bg-muted/60 rounded-[var(--radius-md)] border border-border text-xs text-muted-foreground flex items-center gap-2">
                      <Lock className="h-4 w-4 text-primary shrink-0" />
                      <span>
                        This business day is closed and permanently locked. No back-dated
                        modifications or voids are permitted.
                      </span>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            {/* Expenses List */}
            <Card>
              <CardHeader className="pb-3 border-b border-border flex flex-row items-center justify-between">
                <CardTitle className="text-sm font-bold flex items-center gap-2">
                  <Receipt className="h-4 w-4 text-primary" />
                  Today&apos;s Petty Cash Expenses & Disbursed Refunds ({expenses.length})
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0">
                <div className="overflow-x-auto">
                  <table className="w-full text-left text-xs border-collapse">
                    <thead>
                      <tr className="border-b border-border bg-muted/30 text-muted-foreground font-semibold uppercase text-[10px]">
                        <th className="py-2.5 px-4">Payee</th>
                        <th className="py-2.5 px-4">Category</th>
                        <th className="py-2.5 px-4">Method / Ref</th>
                        <th className="py-2.5 px-4 text-right">Amount</th>
                        <th className="py-2.5 px-4 text-center">Status</th>
                        {!summary.is_closed && (
                          <th className="py-2.5 px-4 text-center">Action</th>
                        )}
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border">
                      {expenses.length === 0 ? (
                        <tr>
                          <td
                            colSpan={6}
                            className="py-6 text-center text-muted-foreground"
                          >
                            No expense vouchers posted for this business date.
                          </td>
                        </tr>
                      ) : (
                        expenses.map((e) => (
                          <tr key={e.id} className="hover:bg-muted/30">
                            <td className="py-2.5 px-4 font-semibold text-foreground">
                              {e.payee}
                            </td>
                            <td className="py-2.5 px-4 text-muted-foreground">
                              {e.category_name}
                            </td>
                            <td className="py-2.5 px-4">
                              <span className="capitalize font-medium text-foreground">
                                {e.method}
                              </span>
                              {e.reference && (
                                <span className="text-[11px] text-muted-foreground ml-1.5 font-mono">
                                  ({e.reference})
                                </span>
                              )}
                            </td>
                            <td className="py-2.5 px-4 text-right font-mono font-bold text-destructive">
                              -{formatTaka(e.amount_poisha)}
                            </td>
                            <td className="py-2.5 px-4 text-center">
                              {e.status === "voided" ? (
                                <Badge
                                  variant="outline"
                                  className="text-[9px] text-muted-foreground"
                                >
                                  Voided
                                </Badge>
                              ) : (
                                <Badge
                                  variant="outline"
                                  className="text-[9px] text-success border-success/30"
                                >
                                  Posted
                                </Badge>
                              )}
                            </td>
                            {!summary.is_closed && (
                              <td className="py-2.5 px-4 text-center">
                                {e.status === "posted" && (
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => void handleVoidExpense(e.id)}
                                    disabled={voidingExpenseId === e.id}
                                    className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                                  >
                                    {voidingExpenseId === e.id ? (
                                      <Loader2 className="h-3 w-3 animate-spin" />
                                    ) : (
                                      <Trash2 className="h-3 w-3" />
                                    )}
                                  </Button>
                                )}
                              </td>
                            )}
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Right 1 Col: Digital Collections & Cashier Breakdowns */}
          <div className="space-y-6">
            {/* Digital Collections */}
            <Card>
              <CardHeader className="pb-3 border-b border-border">
                <CardTitle className="text-sm font-bold">
                  Digital & Bank Collections
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-3">
                {summary.digital_collections.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No digital receipts today.
                  </p>
                ) : (
                  summary.digital_collections.map((d) => (
                    <div
                      key={d.method}
                      className="flex items-center justify-between p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border"
                    >
                      <div>
                        <span className="text-xs font-bold uppercase text-foreground">
                          {d.method}
                        </span>
                        <p className="text-[11px] text-muted-foreground">
                          {d.count} transactions
                        </p>
                      </div>
                      <span className="text-sm font-mono font-bold text-primary">
                        {formatTaka(d.total_amount_poisha)}
                      </span>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>

            {/* Collector Breakdown */}
            <Card>
              <CardHeader className="pb-3 border-b border-border">
                <CardTitle className="text-sm font-bold">
                  Counter Cashier Collections
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-3">
                {summary.collector_breakdown.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No counter collections recorded.
                  </p>
                ) : (
                  summary.collector_breakdown.map((c, i) => (
                    <div
                      key={c.collector_user_id || i}
                      className="flex items-center justify-between p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border"
                    >
                      <div>
                        <span className="text-xs font-semibold text-foreground">
                          {c.collector_name}
                        </span>
                        <p className="text-[11px] text-muted-foreground">
                          {c.receipt_count} receipts issued
                        </p>
                      </div>
                      <span className="text-sm font-mono font-bold text-foreground">
                        {formatTaka(c.total_amount_poisha)}
                      </span>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>

            {/* Category Expenses Breakdown */}
            <Card>
              <CardHeader className="pb-3 border-b border-border">
                <CardTitle className="text-sm font-bold">Expenses by Category</CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-2.5">
                {summary.expenses_by_category.length === 0 ? (
                  <p className="text-xs text-muted-foreground">
                    No expenses recorded today.
                  </p>
                ) : (
                  summary.expenses_by_category.map((cat) => (
                    <div
                      key={cat.category_id}
                      className="flex items-center justify-between text-xs py-1 border-b border-border/50"
                    >
                      <span className="text-muted-foreground">{cat.category_name}</span>
                      <span className="font-mono font-semibold text-destructive">
                        -{formatTaka(cat.total_amount_poisha)}
                      </span>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {/* Expense Entry Dialog */}
      <ExpenseEntryDialog
        open={expenseDialogOpen}
        onOpenChange={setExpenseDialogOpen}
        onSuccess={() => void loadDayData()}
      />

      {/* Print Close Sheet Preview */}
      <PrintPreviewDialog
        open={previewOpen}
        onOpenChange={setPreviewOpen}
        title="Daily Cash Close Sheet"
        htmlContent={previewHtml}
        pageSize="a4"
        onPrintConfirmed={async () => {
          await executeHtmlPrint(previewHtml);
          setPreviewOpen(false);
        }}
      />
    </div>
  );
}
