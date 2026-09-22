import { useState, useEffect, useCallback } from "react";
import {
  BarChart3,
  Calendar,
  CreditCard,
  Download,
  DollarSign,
  AlertTriangle,
  Users,
  Award,
  Filter,
  Printer,
  TrendingUp,
  Clock,
  CheckCircle2,
  XCircle,
  Phone,
} from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { MetricCard } from "@/components/ui/metric-card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  rpc,
  type CollectionSummaryResponse,
  type CollectorSummaryResponse,
  type DuesAgingResponse,
  type ExpensesSummaryResponse,
  type AttendanceSummaryResponse,
  type ExamPerformanceResponse,
} from "@/gen/rpc";

export type ReportTab =
  "collections" | "collectors" | "dues" | "expenses" | "attendance" | "exams";

export function ReportsView() {
  const [activeTab, setActiveTab] = useState<ReportTab>("collections");

  // Date filters
  const today = new Date().toISOString().substring(0, 10);
  const firstDayOfMonth = today.substring(0, 8) + "01";
  const currentMonth = today.substring(0, 7);

  const [startDate, setStartDate] = useState(firstDayOfMonth);
  const [endDate, setEndDate] = useState(today);
  const [attendanceMonth, setAttendanceMonth] = useState(currentMonth);
  const [examId, setExamId] = useState("");

  // Report state
  const [loading, setLoading] = useState(false);
  const [collectionData, setCollectionData] = useState<CollectionSummaryResponse | null>(
    null
  );
  const [collectorData, setCollectorData] = useState<CollectorSummaryResponse | null>(
    null
  );
  const [duesData, setDuesData] = useState<DuesAgingResponse | null>(null);
  const [expensesData, setExpensesData] = useState<ExpensesSummaryResponse | null>(null);
  const [attendanceData, setAttendanceData] = useState<AttendanceSummaryResponse | null>(
    null
  );
  const [examData, setExamData] = useState<ExamPerformanceResponse | null>(null);

  // Fetch report based on active tab
  const fetchReport = useCallback(async () => {
    setLoading(true);
    try {
      if (activeTab === "collections") {
        const res = await rpc("reports.get_collection_summary", {
          start_date: startDate,
          end_date: endDate,
        });
        if (res.ok && res.data) setCollectionData(res.data);
      } else if (activeTab === "collectors") {
        const res = await rpc("reports.get_collector_summary", {
          start_date: startDate,
          end_date: endDate,
        });
        if (res.ok && res.data) setCollectorData(res.data);
      } else if (activeTab === "dues") {
        const res = await rpc("reports.get_dues_aging_report", {
          as_of_date: endDate,
        });
        if (res.ok && res.data) setDuesData(res.data);
      } else if (activeTab === "expenses") {
        const res = await rpc("reports.get_expenses_summary", {
          start_date: startDate,
          end_date: endDate,
        });
        if (res.ok && res.data) setExpensesData(res.data);
      } else if (activeTab === "attendance") {
        const res = await rpc("reports.get_attendance_summary", {
          month: attendanceMonth,
        });
        if (res.ok && res.data) setAttendanceData(res.data);
      } else if (activeTab === "exams" && examId) {
        const res = await rpc("reports.get_exam_performance", {
          exam_id: examId,
        });
        if (res.ok && res.data) setExamData(res.data);
      }
    } catch (e) {
      console.error("Failed to load report", e);
    } finally {
      setLoading(false);
    }
  }, [activeTab, startDate, endDate, attendanceMonth, examId]);

  useEffect(() => {
    fetchReport();
  }, [fetchReport]);

  // Export to CSV with UTF-8 BOM
  const handleExportCsv = async (reportType: string, params: unknown) => {
    try {
      const res = await rpc("reports.export_csv", {
        report_type: reportType,
        params,
      });
      if (res.ok && res.data) {
        const blob = new Blob([res.data.csv_content], {
          type: "text/csv;charset=utf-8;",
        });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = res.data.filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      }
    } catch (e) {
      console.error("CSV export failed", e);
    }
  };

  return (
    <div className="space-y-6 print:space-y-4 print:p-0">
      {/* Top Header & Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-border pb-4 print:hidden">
        <div>
          <div className="flex items-center gap-2">
            <BarChart3 className="h-6 w-6 text-primary" />
            <h1 className="text-xl font-bold tracking-tight text-foreground">
              Analytical Reports Center
            </h1>
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Real-time financial audits, dues aging recovery, attendance registers, and
            academic summaries.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => window.print()}
            className="h-8 gap-1.5 text-xs"
          >
            <Printer className="h-3.5 w-3.5" />
            Print View
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={() => {
              const params =
                activeTab === "collections" || activeTab === "expenses"
                  ? { start_date: startDate, end_date: endDate }
                  : activeTab === "collectors"
                    ? { start_date: startDate, end_date: endDate }
                    : activeTab === "dues"
                      ? { as_of_date: endDate }
                      : activeTab === "attendance"
                        ? { month: attendanceMonth }
                        : { exam_id: examId };
              handleExportCsv(activeTab, params);
            }}
            className="h-8 gap-1.5 text-xs bg-emerald-600 hover:bg-emerald-700 text-white"
          >
            <Download className="h-3.5 w-3.5" />
            Export Excel (CSV)
          </Button>
        </div>
      </div>

      {/* Sub-report Navigation Tabs */}
      <div className="flex flex-wrap items-center gap-2 border-b border-border pb-2 print:hidden">
        {[
          { id: "collections", label: "Fee Collections", icon: DollarSign },
          { id: "collectors", label: "Cashier Performance", icon: Users },
          { id: "dues", label: "Dues Aging & Recovery", icon: CreditCard },
          { id: "expenses", label: "Expenses & Cash Drawer", icon: TrendingUp },
          { id: "attendance", label: "Attendance & Chronic Absenteeism", icon: Calendar },
          { id: "exams", label: "Exam Merit Summaries", icon: Award },
        ].map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as ReportTab)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
                isActive
                  ? "bg-primary text-primary-foreground shadow-xs"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
            >
              <Icon className="h-3.5 w-3.5" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Filter Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-muted/40 p-3 rounded-lg border border-border print:hidden">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
            <Filter className="h-3.5 w-3.5 text-muted-foreground" />
            <span>Filters:</span>
          </div>

          {activeTab !== "attendance" && activeTab !== "exams" && (
            <div className="flex items-center gap-2 text-xs">
              <span className="text-muted-foreground">From:</span>
              <input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                className="h-8 px-2 rounded-md border border-border bg-background text-xs"
              />
              <span className="text-muted-foreground">To:</span>
              <input
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
                className="h-8 px-2 rounded-md border border-border bg-background text-xs"
              />
            </div>
          )}

          {activeTab === "attendance" && (
            <div className="flex items-center gap-2 text-xs">
              <span className="text-muted-foreground">Month:</span>
              <input
                type="month"
                value={attendanceMonth}
                onChange={(e) => setAttendanceMonth(e.target.value)}
                className="h-8 px-2 rounded-md border border-border bg-background text-xs"
              />
            </div>
          )}

          {activeTab === "exams" && (
            <div className="flex items-center gap-2 text-xs">
              <span className="text-muted-foreground">Exam ID:</span>
              <input
                type="text"
                placeholder="Enter exam ID..."
                value={examId}
                onChange={(e) => setExamId(e.target.value)}
                className="h-8 px-2.5 w-64 rounded-md border border-border bg-background text-xs"
              />
            </div>
          )}
        </div>

        <Button
          variant="outline"
          size="sm"
          onClick={fetchReport}
          disabled={loading}
          className="h-8 text-xs"
        >
          {loading ? "Refreshing..." : "Apply Filters"}
        </Button>
      </div>

      {/* Tab 1: Collection Summary */}
      {activeTab === "collections" && collectionData && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <MetricCard
              title="Gross Collections"
              value={`৳${collectionData.total_collected_formatted}`}
              icon={<DollarSign className="h-4 w-4" />}
              subtitle={`${collectionData.payment_count} successful payments`}
            />
            <MetricCard
              title="Net Collections"
              value={`৳${collectionData.net_collected_formatted}`}
              icon={<TrendingUp className="h-4 w-4" />}
              subtitle={`Gross minus refunds & voids`}
            />
            <MetricCard
              title="Payment Count"
              value={collectionData.payment_count.toString()}
              icon={<CheckCircle2 className="h-4 w-4" />}
              subtitle="Valid transactions"
            />
            <MetricCard
              title="Voided Receipts"
              value={collectionData.void_count.toString()}
              icon={<XCircle className="h-4 w-4" />}
              subtitle="Cancelled at counter"
            />
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Method Breakdown */}
            <Card>
              <CardHeader className="py-3 px-4 border-b border-border">
                <CardTitle className="text-sm font-semibold">
                  Payment Method Distribution
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Method</TableHead>
                      <TableHead className="text-center">Count</TableHead>
                      <TableHead className="text-right">Total (৳)</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {collectionData.method_distribution.map((m) => (
                      <TableRow key={m.method}>
                        <TableCell className="font-medium uppercase text-xs">
                          {m.method}
                        </TableCell>
                        <TableCell className="text-center text-xs">{m.count}</TableCell>
                        <TableCell className="text-right font-mono text-xs font-semibold">
                          ৳{m.amount_formatted}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            {/* Daily Timeline */}
            <Card>
              <CardHeader className="py-3 px-4 border-b border-border">
                <CardTitle className="text-sm font-semibold">
                  Daily Collection Timeline
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0 max-h-80 overflow-y-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Date</TableHead>
                      <TableHead className="text-center">Transactions</TableHead>
                      <TableHead className="text-right">Daily Total (৳)</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {collectionData.daily_breakdown.map((d) => (
                      <TableRow key={d.date}>
                        <TableCell className="text-xs font-mono">{d.date}</TableCell>
                        <TableCell className="text-center text-xs">
                          {d.payment_count}
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs font-semibold">
                          ৳{d.amount_formatted}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {/* Tab 2: Cashier Performance */}
      {activeTab === "collectors" && collectorData && (
        <Card>
          <CardHeader className="py-3 px-4 border-b border-border">
            <CardTitle className="text-sm font-semibold">
              Collector & Counter Staff Performance Summary
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Staff Username</TableHead>
                  <TableHead>Display Name</TableHead>
                  <TableHead className="text-center">Receipts</TableHead>
                  <TableHead className="text-right">Cash (৳)</TableHead>
                  <TableHead className="text-right">Digital (৳)</TableHead>
                  <TableHead className="text-right">Total Collected (৳)</TableHead>
                  <TableHead className="text-center">Voids</TableHead>
                  <TableHead className="text-right">Void Amount (৳)</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {collectorData.collectors.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={8}
                      className="text-center py-6 text-muted-foreground text-xs"
                    >
                      No collector activity found for the selected date range.
                    </TableCell>
                  </TableRow>
                ) : (
                  collectorData.collectors.map((c) => (
                    <TableRow key={c.user_id}>
                      <TableCell className="font-medium text-xs">{c.username}</TableCell>
                      <TableCell className="text-xs">{c.display_name}</TableCell>
                      <TableCell className="text-center text-xs">
                        {c.total_payments}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs">
                        ৳{c.cash_collected_formatted}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs">
                        ৳{c.digital_collected_formatted}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs font-semibold text-emerald-600">
                        ৳{c.total_collected_formatted}
                      </TableCell>
                      <TableCell className="text-center text-xs">
                        {c.void_count > 0 ? (
                          <Badge variant="overdue" className="text-3xs">
                            {c.void_count}
                          </Badge>
                        ) : (
                          "0"
                        )}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs text-muted-foreground">
                        ৳{c.void_amount_formatted}
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      {/* Tab 3: Dues Aging */}
      {activeTab === "dues" && duesData && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
            <MetricCard
              title="Current (0–30 Days)"
              value={`৳${duesData.summary.total_current_formatted}`}
              icon={<CreditCard className="h-4 w-4" />}
              subtitle="Fresh due notices"
            />
            <MetricCard
              title="31–60 Days Overdue"
              value={`৳${duesData.summary.total_days_31_60_formatted}`}
              icon={<Clock className="h-4 w-4" />}
              subtitle="First follow-up cycle"
            />
            <MetricCard
              title="61–90 Days Overdue"
              value={`৳${duesData.summary.total_days_61_90_formatted}`}
              icon={<AlertTriangle className="h-4 w-4" />}
              subtitle="Urgent guardian calls"
            />
            <MetricCard
              title="90+ Days Chronic"
              value={`৳${duesData.summary.total_days_90_plus_formatted}`}
              icon={<XCircle className="h-4 w-4" />}
              subtitle="Severe arrears"
            />
            <MetricCard
              title="Total Overdue"
              value={`৳${duesData.summary.grand_total_due_formatted}`}
              icon={<DollarSign className="h-4 w-4" />}
              subtitle={`${duesData.summary.total_students_with_dues} students with dues`}
            />
          </div>

          <Card>
            <CardHeader className="py-3 px-4 border-b border-border">
              <CardTitle className="text-sm font-semibold">
                Student Dues Aging Directory
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Student</TableHead>
                    <TableHead>Batch</TableHead>
                    <TableHead>Guardian Contact</TableHead>
                    <TableHead className="text-right">0–30d (৳)</TableHead>
                    <TableHead className="text-right">31–60d (৳)</TableHead>
                    <TableHead className="text-right">61–90d (৳)</TableHead>
                    <TableHead className="text-right">90+d (৳)</TableHead>
                    <TableHead className="text-right font-bold">Total Due (৳)</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {duesData.items.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={8}
                        className="text-center py-6 text-muted-foreground text-xs"
                      >
                        All student dues are cleared!
                      </TableCell>
                    </TableRow>
                  ) : (
                    duesData.items.map((item) => (
                      <TableRow key={item.student_id}>
                        <TableCell>
                          <div className="flex flex-col">
                            <span className="font-medium text-xs">{item.name}</span>
                            <span className="text-3xs text-muted-foreground font-mono">
                              {item.student_code}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell className="text-xs text-muted-foreground">
                          {item.batch_name}
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-col text-xs">
                            <span>{item.primary_guardian_name || "—"}</span>
                            {item.primary_guardian_phone && (
                              <span className="text-3xs text-primary font-mono flex items-center gap-1">
                                <Phone className="h-2.5 w-2.5" />
                                {item.primary_guardian_phone}
                              </span>
                            )}
                          </div>
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs">
                          {item.current_bucket_poisha > 0
                            ? `৳${item.current_bucket_formatted}`
                            : "—"}
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs text-amber-600">
                          {item.days_31_60_poisha > 0
                            ? `৳${item.days_31_60_formatted}`
                            : "—"}
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs text-orange-600 font-semibold">
                          {item.days_61_90_poisha > 0
                            ? `৳${item.days_61_90_formatted}`
                            : "—"}
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs text-rose-600 font-bold">
                          {item.days_90_plus_poisha > 0
                            ? `৳${item.days_90_plus_formatted}`
                            : "—"}
                        </TableCell>
                        <TableCell className="text-right font-mono text-xs font-bold text-foreground">
                          ৳{item.total_due_formatted}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Tab 4: Expenses & Drawer */}
      {activeTab === "expenses" && expensesData && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Categories */}
          <Card>
            <CardHeader className="py-3 px-4 border-b border-border">
              <CardTitle className="text-sm font-semibold">
                Petty & Operational Expenses
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Category</TableHead>
                    <TableHead className="text-center">Count</TableHead>
                    <TableHead className="text-right">Total (৳)</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {expensesData.categories.map((cat) => (
                    <TableRow key={cat.category_name}>
                      <TableCell className="font-medium text-xs">
                        {cat.category_name}
                      </TableCell>
                      <TableCell className="text-center text-xs">{cat.count}</TableCell>
                      <TableCell className="text-right font-mono text-xs font-semibold">
                        ৳{cat.amount_formatted}
                      </TableCell>
                    </TableRow>
                  ))}
                  <TableRow className="bg-muted/50 font-bold">
                    <TableCell className="text-xs">TOTAL EXPENSES</TableCell>
                    <TableCell className="text-center text-xs">
                      {expensesData.expense_count}
                    </TableCell>
                    <TableCell className="text-right font-mono text-xs text-rose-600">
                      ৳{expensesData.total_expense_formatted}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          {/* Drawer Reconciliation */}
          <Card>
            <CardHeader className="py-3 px-4 border-b border-border">
              <CardTitle className="text-sm font-semibold">
                Cash Drawer Reconciliation
              </CardTitle>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="flex justify-between items-center text-xs pb-2 border-b border-border">
                <span className="text-muted-foreground">Starting Cash Balance:</span>
                <span className="font-mono font-semibold">
                  ৳{expensesData.drawer_reconciliation.starting_cash_formatted}
                </span>
              </div>
              <div className="flex justify-between items-center text-xs pb-2 border-b border-border">
                <span className="text-muted-foreground">+ Cash In (Collections):</span>
                <span className="font-mono text-emerald-600 font-semibold">
                  ৳{expensesData.drawer_reconciliation.cash_collections_formatted}
                </span>
              </div>
              <div className="flex justify-between items-center text-xs pb-2 border-b border-border">
                <span className="text-muted-foreground">- Cash Out (Expenses):</span>
                <span className="font-mono text-rose-600 font-semibold">
                  ৳{expensesData.drawer_reconciliation.cash_expenses_formatted}
                </span>
              </div>
              <div className="flex justify-between items-center text-xs pb-2 border-b border-border">
                <span className="font-semibold text-foreground">
                  Expected Drawer Balance:
                </span>
                <span className="font-mono font-bold text-foreground">
                  ৳{expensesData.drawer_reconciliation.expected_closing_formatted}
                </span>
              </div>
              <div className="flex justify-between items-center text-xs pb-2 border-b border-border">
                <span className="text-muted-foreground">Actual Counted Cash:</span>
                <span className="font-mono font-bold">
                  ৳{expensesData.drawer_reconciliation.counted_closing_formatted}
                </span>
              </div>
              <div className="flex justify-between items-center text-xs pt-1">
                <span className="font-semibold">Discrepancy (Over / Short):</span>
                <Badge
                  variant={
                    expensesData.drawer_reconciliation.discrepancy_poisha === 0
                      ? "outline"
                      : "overdue"
                  }
                  className="font-mono text-2xs"
                >
                  ৳{expensesData.drawer_reconciliation.discrepancy_formatted}
                </Badge>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Tab 5: Attendance */}
      {activeTab === "attendance" && attendanceData && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <MetricCard
              title="Batch Average Attendance"
              value={`${attendanceData.batch_average_percentage}%`}
              icon={<Calendar className="h-4 w-4" />}
              subtitle={`Overall monthly presence`}
            />
            <MetricCard
              title="Total Enrolled Students"
              value={attendanceData.total_students.toString()}
              icon={<Users className="h-4 w-4" />}
              subtitle="Active in sessions"
            />
            <MetricCard
              title="Chronic Absentees (< 75%)"
              value={attendanceData.chronic_absentee_count.toString()}
              icon={<AlertTriangle className="h-4 w-4" />}
              subtitle="Barred from exams without excuse"
            />
          </div>

          <Card>
            <CardHeader className="py-3 px-4 border-b border-border">
              <CardTitle className="text-sm font-semibold">
                Monthly Batch Attendance Register
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Student ID</TableHead>
                    <TableHead>Student Name</TableHead>
                    <TableHead>Batch</TableHead>
                    <TableHead className="text-center">Sessions</TableHead>
                    <TableHead className="text-center">Present</TableHead>
                    <TableHead className="text-center">Late</TableHead>
                    <TableHead className="text-center">Absent</TableHead>
                    <TableHead className="text-center">Excused</TableHead>
                    <TableHead className="text-right">Attendance %</TableHead>
                    <TableHead className="text-center">Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {attendanceData.items.map((s) => (
                    <TableRow key={s.student_id}>
                      <TableCell className="font-mono text-xs">
                        {s.student_code}
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-col">
                          <span className="font-medium text-xs">{s.name}</span>
                          {s.name_bn && (
                            <span className="text-3xs text-muted-foreground font-bangla">
                              {s.name_bn}
                            </span>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        {s.batch_name}
                      </TableCell>
                      <TableCell className="text-center text-xs">
                        {s.total_sessions}
                      </TableCell>
                      <TableCell className="text-center text-xs text-emerald-600 font-semibold">
                        {s.present_count}
                      </TableCell>
                      <TableCell className="text-center text-xs text-amber-600">
                        {s.late_count}
                      </TableCell>
                      <TableCell className="text-center text-xs text-rose-600 font-semibold">
                        {s.absent_count}
                      </TableCell>
                      <TableCell className="text-center text-xs text-muted-foreground">
                        {s.excused_count}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs font-bold">
                        {s.attendance_percentage}%
                      </TableCell>
                      <TableCell className="text-center">
                        {s.is_chronic_absentee ? (
                          <Badge variant="overdue" className="text-3xs px-1.5 py-0 h-4">
                            CHRONIC ABSENTEE
                          </Badge>
                        ) : (
                          <Badge
                            variant="outline"
                            className="text-3xs px-1.5 py-0 h-4 text-emerald-600 border-emerald-300"
                          >
                            REGULAR
                          </Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Tab 6: Exams */}
      {activeTab === "exams" && (
        <div className="space-y-6">
          {!examData ? (
            <div className="text-center py-12 border border-dashed border-border rounded-lg">
              <Award className="h-10 w-10 text-muted-foreground mx-auto mb-2 opacity-50" />
              <h3 className="font-semibold text-sm">No Exam Selected</h3>
              <p className="text-xs text-muted-foreground mt-1 max-w-sm mx-auto">
                Enter a published Exam ID in the filter bar above to inspect grade
                distributions and merit rankers.
              </p>
            </div>
          ) : (
            <div className="space-y-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
                <MetricCard
                  title="Total Candidates"
                  value={examData.total_candidates.toString()}
                  icon={<Users className="h-4 w-4" />}
                  subtitle={examData.exam_name}
                />
                <MetricCard
                  title="Passed Candidates"
                  value={examData.passed_count.toString()}
                  icon={<CheckCircle2 className="h-4 w-4" />}
                  subtitle={`${examData.pass_percentage}% Pass Rate`}
                />
                <MetricCard
                  title="Failed Candidates"
                  value={examData.failed_count.toString()}
                  icon={<XCircle className="h-4 w-4" />}
                  subtitle="Required remediation"
                />
                <MetricCard
                  title="Pass Rate"
                  value={`${examData.pass_percentage}%`}
                  icon={<TrendingUp className="h-4 w-4" />}
                  subtitle="Batch success rate"
                />
                <MetricCard
                  title="Average GPA"
                  value={examData.average_gpa.toFixed(2)}
                  icon={<Award className="h-4 w-4" />}
                  subtitle="Across all examinees"
                />
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Grade Distribution */}
                <Card>
                  <CardHeader className="py-3 px-4 border-b border-border">
                    <CardTitle className="text-sm font-semibold">
                      Grade Distribution
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="p-0">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>Letter Grade</TableHead>
                          <TableHead className="text-center">Student Count</TableHead>
                          <TableHead className="text-right">Percentage</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {examData.grade_distribution.map((g) => (
                          <TableRow key={g.grade}>
                            <TableCell className="font-bold text-xs">{g.grade}</TableCell>
                            <TableCell className="text-center text-xs">
                              {g.count}
                            </TableCell>
                            <TableCell className="text-right font-mono text-xs">
                              {g.percentage}%
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </CardContent>
                </Card>

                {/* Top 5 Merit Rankers */}
                <Card>
                  <CardHeader className="py-3 px-4 border-b border-border">
                    <CardTitle className="text-sm font-semibold">
                      Top 5 Merit Rankers
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="p-0">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-12 text-center">Rank</TableHead>
                          <TableHead>Student</TableHead>
                          <TableHead className="text-center">Grade</TableHead>
                          <TableHead className="text-right">GPA</TableHead>
                          <TableHead className="text-right">Marks</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {examData.top_rankers.map((r) => (
                          <TableRow key={r.student_id}>
                            <TableCell className="text-center">
                              <Badge variant="outline" className="font-bold text-2xs">
                                #{r.rank}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              <div className="flex flex-col">
                                <span className="font-semibold text-xs">{r.name}</span>
                                <span className="text-3xs font-mono text-muted-foreground">
                                  {r.student_code}
                                </span>
                              </div>
                            </TableCell>
                            <TableCell className="text-center font-bold text-xs">
                              {r.grade}
                            </TableCell>
                            <TableCell className="text-right font-mono font-bold text-xs text-primary">
                              {r.gpa.toFixed(2)}
                            </TableCell>
                            <TableCell className="text-right font-mono text-xs">
                              {r.total_marks}
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </CardContent>
                </Card>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
