import {
  Users,
  CalendarCheck,
  CreditCard,
  FileText,
  UserPlus,
  CheckCircle2,
  Receipt,
  FilePlus2,
  Download,
  MoreHorizontal,
  ChevronRight,
  TrendingUp,
  Clock,
  ShieldCheck,
  RefreshCw,
  Search,
  DollarSign,
  AlertTriangle,
  BookOpen,
  Eye,
  Layers,
  MessageSquare,
  Printer,
} from "lucide-react";
import { MetricCard } from "@/components/ui/metric-card";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { useEffect, useState } from "react";
import { useAuth } from "@/contexts/auth-context";
import {
  rpc,
  type PingResponse,
  type RpcError,
  type TodayCockpitResponse,
} from "@/gen/rpc";

interface DashboardViewProps {
  rpcResponse?: PingResponse | null;
  rpcLoading?: boolean;
  rpcError?: RpcError | null;
  onRefreshRpc?: () => void;
  onSelectTab?: (tabId: string) => void;
  onOpenReprint?: () => void;
  dateRange?: string;
}

export function DashboardView({
  rpcResponse,
  rpcLoading = false,
  rpcError = null,
  onRefreshRpc,
  onSelectTab,
  onOpenReprint,
  dateRange,
}: DashboardViewProps) {
  const { roles, previewRole, setPreviewRole } = useAuth();
  const [cockpitData, setCockpitData] = useState<TodayCockpitResponse | null>(null);

  useEffect(() => {
    let mounted = true;
    rpc("intelligence.get_today_cockpit", {})
      .then((res) => {
        if (mounted && res.ok && res.data) {
          setCockpitData(res.data);
        }
      })
      .catch(() => {});
    return () => {
      mounted = false;
    };
  }, [dateRange]);

  // Determine effective operational role
  const activeRole =
    previewRole || (roles.includes("Owner") ? "Owner" : roles[0] || "Admin");

  // Sample student data matching reference
  const recentStudents = [
    {
      id: 1,
      name: "Aarav Sharma",
      class: "Class 10",
      phone: "01711 432109",
      status: "paid" as const,
    },
    {
      id: 2,
      name: "Priya Singh",
      class: "Class 11",
      phone: "01712 321098",
      status: "pending" as const,
    },
    {
      id: 3,
      name: "Aditya Verma",
      class: "Class 10",
      phone: "01713 210987",
      status: "paid" as const,
    },
    {
      id: 4,
      name: "Sneha Patel",
      class: "Class 12",
      phone: "01714 678901",
      status: "pending" as const,
    },
    {
      id: 5,
      name: "Karan Mehta",
      class: "Class 11",
      phone: "01715 567890",
      status: "paid" as const,
    },
  ];

  // Cashier recent transactions
  const recentReceipts = [
    {
      receiptNo: "RCP-2026-00084",
      student: "Aarav Sharma",
      amount: "৳ 5,000",
      method: "Cash",
      time: "10m ago",
    },
    {
      receiptNo: "RCP-2026-00083",
      student: "Tanvir Ahmed",
      amount: "৳ 3,500",
      method: "bKash",
      time: "25m ago",
    },
    {
      receiptNo: "RCP-2026-00082",
      student: "Nusrat Jahan",
      amount: "৳ 6,000",
      method: "Cash",
      time: "1h ago",
    },
    {
      receiptNo: "RCP-2026-00081",
      student: "Rahim Uddin",
      amount: "৳ 4,000",
      method: "Nagad",
      time: "2h ago",
    },
    {
      receiptNo: "RCP-2026-00080",
      student: "Priya Singh",
      amount: "৳ 5,000",
      method: "Cash",
      time: "3h ago",
    },
  ];

  // Teacher assigned batches
  const teacherBatches = [
    {
      id: "b1",
      name: "HSC Physics Champions",
      time: "10:00 AM - 11:30 AM",
      room: "Room 201",
      status: "completed",
      attendance: "38/40",
    },
    {
      id: "b2",
      name: "SSC Physics Foundation",
      time: "02:00 PM - 03:30 PM",
      room: "Room 104",
      status: "pending",
      attendance: "Pending",
    },
    {
      id: "b3",
      name: "Medical Physics Intake",
      time: "04:30 PM - 06:00 PM",
      room: "Auditorium B",
      status: "pending",
      attendance: "Pending",
    },
  ];

  // Teacher pending exams
  const pendingExams = [
    {
      id: "e1",
      examName: "Weekly Term Test #4",
      subject: "Physics 1st Paper",
      batch: "HSC Physics Champions",
      papersCount: 40,
      deadline: "Tomorrow",
    },
    {
      id: "e2",
      examName: "SSC Monthly Model Test",
      subject: "General Science",
      batch: "SSC Physics Foundation",
      papersCount: 35,
      deadline: "Sep 26",
    },
  ];

  // Role Preview Switcher (Available in zero training mode)
  const renderRoleSwitcher = () => (
    <div className="flex items-center gap-2.5 p-1 rounded-2xl bg-muted/50 border border-border/70 shadow-2xs">
      <div className="flex items-center gap-1.5 pl-2 text-xs text-muted-foreground font-semibold">
        <Eye className="h-3.5 w-3.5 text-primary" />
        <span className="hidden sm:inline">Role:</span>
      </div>
      <select
        value={previewRole || "Owner"}
        onChange={(e) => {
          const val = e.target.value;
          setPreviewRole(val === "Owner" ? null : val);
        }}
        className="text-xs bg-card border border-border/80 rounded-xl px-3 py-1.5 text-foreground font-bold shadow-xs cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/20"
      >
        <option value="Owner">Owner (Executive Cockpit)</option>
        <option value="Receptionist">Receptionist (Front Desk)</option>
        <option value="Cashier">Cashier (Counter Desk)</option>
        <option value="Teacher">Teacher (Academic Desk)</option>
        <option value="Admin">Admin (Operations Pulse)</option>
      </select>
    </div>
  );

  // ---------------------------------------------------------------------------
  // 1. RECEPTIONIST DASHBOARD VIEW (Spec 0029 AC-2)
  // ---------------------------------------------------------------------------
  if (activeRole === "Receptionist") {
    return (
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                Front Desk Cockpit
              </h1>
              <Badge variant="active" size="sm" className="font-semibold">
                Receptionist Mode
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Zero Training Mode: High-frequency intake, visitor search, and class
              attendance
            </p>
          </div>
          {renderRoleSwitcher()}
        </div>

        {/* AC-2: 3 Large Primary Action Cards */}
        <div className="grid gap-4 sm:grid-cols-3">
          <Card
            className="p-5 border-2 border-primary/30 hover:border-primary cursor-pointer transition-all hover:shadow-md bg-gradient-to-br from-primary-subtle/30 to-background flex flex-col justify-between group"
            onClick={() => onSelectTab?.("students")}
          >
            <div>
              <div className="flex items-center justify-between mb-3">
                <div className="h-10 w-10 rounded-lg bg-primary text-primary-foreground flex items-center justify-center shadow-xs">
                  <UserPlus className="h-5 w-5" />
                </div>
                <Badge
                  variant="default"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  N S
                </Badge>
              </div>
              <h3 className="text-base font-bold text-foreground group-hover:text-primary transition-colors">
                Admit New Student
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Collect biodata, guardian contacts, and enroll into an active batch.
              </p>
            </div>
            <div className="mt-4 flex items-center gap-1.5 text-xs font-semibold text-primary">
              <span>Start Intake Form</span>
              <ChevronRight className="h-3.5 w-3.5" />
            </div>
          </Card>

          <Card
            className="p-5 border-2 border-border hover:border-primary/60 cursor-pointer transition-all hover:shadow-md flex flex-col justify-between group"
            onClick={() => {
              window.dispatchEvent(
                new KeyboardEvent("keydown", { key: "k", ctrlKey: true })
              );
            }}
          >
            <div>
              <div className="flex items-center justify-between mb-3">
                <div className="h-10 w-10 rounded-lg bg-info-subtle text-info-text flex items-center justify-center shadow-xs">
                  <Search className="h-5 w-5" />
                </div>
                <Badge
                  variant="outline"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  Ctrl+K
                </Badge>
              </div>
              <h3 className="text-base font-bold text-foreground group-hover:text-primary transition-colors">
                Search Directory
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Search student name (English/বাংলা), guardian phone, or student code.
              </p>
            </div>
            <div className="mt-4 flex items-center gap-1.5 text-xs font-semibold text-primary">
              <span>Open Fast Search</span>
              <ChevronRight className="h-3.5 w-3.5" />
            </div>
          </Card>

          <Card
            className="p-5 border-2 border-border hover:border-primary/60 cursor-pointer transition-all hover:shadow-md flex flex-col justify-between group"
            onClick={() => onSelectTab?.("attendance")}
          >
            <div>
              <div className="flex items-center justify-between mb-3">
                <div className="h-10 w-10 rounded-lg bg-success-subtle text-success-text flex items-center justify-center shadow-xs">
                  <CalendarCheck className="h-5 w-5" />
                </div>
                <Badge
                  variant="outline"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  G A
                </Badge>
              </div>
              <h3 className="text-base font-bold text-foreground group-hover:text-primary transition-colors">
                Batch Attendance
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Mark present/absent roster with instant automated SMS notifications.
              </p>
            </div>
            <div className="mt-4 flex items-center gap-1.5 text-xs font-semibold text-primary">
              <span>Open Attendance Desk</span>
              <ChevronRight className="h-3.5 w-3.5" />
            </div>
          </Card>
        </div>

        {/* AC-2: Today's Metrics */}
        <div className="grid gap-4 sm:grid-cols-3">
          <MetricCard
            title="New Admissions Today"
            value="14"
            icon={<UserPlus className="h-5 w-5" />}
            trend={{ value: "+12% vs yesterday", isPositive: true }}
          />
          <MetricCard
            title="Total Active Students"
            value="1,248"
            icon={<Users className="h-5 w-5" />}
            subtitle="Enrolled across 16 active batches"
          />
          <MetricCard
            title="Batch Attendance Pulse"
            value="8 / 12"
            icon={<CalendarCheck className="h-5 w-5" />}
            subtitle="Batches completed today (67%)"
          />
        </div>

        {/* Quick Students Roster */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="text-sm">Recent Student Inquiries</CardTitle>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-7"
              onClick={() => onSelectTab?.("students")}
            >
              View All Students
            </Button>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-xs">Student</TableHead>
                  <TableHead className="text-xs">Class / Program</TableHead>
                  <TableHead className="text-xs">Guardian Phone</TableHead>
                  <TableHead className="text-xs">Status</TableHead>
                  <TableHead className="text-xs text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentStudents.slice(0, 4).map((student) => (
                  <TableRow key={student.id}>
                    <TableCell className="font-medium text-xs text-foreground">
                      {student.name}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {student.class}
                    </TableCell>
                    <TableCell className="text-xs font-mono text-muted-foreground">
                      {student.phone}
                    </TableCell>
                    <TableCell>
                      <Badge variant={student.status} size="sm">
                        {student.status === "paid" ? "Active" : "Pending"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-xs h-6 text-primary"
                        onClick={() => onSelectTab?.("students")}
                      >
                        Profile
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    );
  }

  // ---------------------------------------------------------------------------
  // 2. CASHIER / ACCOUNTS DASHBOARD VIEW (Spec 0029 AC-3)
  // ---------------------------------------------------------------------------
  if (activeRole === "Cashier") {
    return (
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                Cash Counter & Ledger Cockpit
              </h1>
              <Badge variant="pending" size="sm" className="font-semibold">
                Cashier Mode
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Zero Training Mode: Rapid fee collection, drawer balancing, and dues
              recovery
            </p>
          </div>
          {renderRoleSwitcher()}
        </div>

        {/* AC-3: 4 Primary Action Cards */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <Card
            className="p-4 border-2 border-primary/40 hover:border-primary cursor-pointer transition-all hover:shadow-md bg-gradient-to-br from-primary-subtle/30 to-background flex flex-col justify-between group"
            onClick={() => onSelectTab?.("payments")}
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="h-9 w-9 rounded-lg bg-primary text-primary-foreground flex items-center justify-center shadow-xs">
                  <CreditCard className="h-4 w-4" />
                </div>
                <Badge
                  variant="default"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  N P
                </Badge>
              </div>
              <h3 className="text-sm font-bold text-foreground group-hover:text-primary transition-colors">
                Receive Payment
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Collect cash or digital fees with instant auto-print receipt.
              </p>
            </div>
            <div className="mt-3 flex items-center gap-1 text-xs font-semibold text-primary">
              <span>Open Cashier Desk</span>
              <ChevronRight className="h-3 w-3" />
            </div>
          </Card>

          <Card
            className="p-4 border-2 border-border hover:border-primary/60 cursor-pointer transition-all hover:shadow-md flex flex-col justify-between group"
            onClick={() => onSelectTab?.("payments")}
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="h-9 w-9 rounded-lg bg-warning-subtle text-warning-text flex items-center justify-center shadow-xs">
                  <Users className="h-4 w-4" />
                </div>
                <Badge
                  variant="outline"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  G F
                </Badge>
              </div>
              <h3 className="text-sm font-bold text-foreground group-hover:text-primary transition-colors">
                Dues Directory
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Aging dues report & 1-click payment intake for debtors.
              </p>
            </div>
            <div className="mt-3 flex items-center gap-1 text-xs font-semibold text-primary">
              <span>View Outstanding Dues</span>
              <ChevronRight className="h-3 w-3" />
            </div>
          </Card>

          <Card
            className="p-4 border-2 border-border hover:border-primary/60 cursor-pointer transition-all hover:shadow-md flex flex-col justify-between group"
            onClick={() => onSelectTab?.("reports")}
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="h-9 w-9 rounded-lg bg-info-subtle text-info-text flex items-center justify-center shadow-xs">
                  <DollarSign className="h-4 w-4" />
                </div>
                <Badge
                  variant="outline"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  Alt+E
                </Badge>
              </div>
              <h3 className="text-sm font-bold text-foreground group-hover:text-primary transition-colors">
                Record Expense
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Post petty cash payment for utility, rent, or stationery.
              </p>
            </div>
            <div className="mt-3 flex items-center gap-1 text-xs font-semibold text-primary">
              <span>Add Expense</span>
              <ChevronRight className="h-3 w-3" />
            </div>
          </Card>

          <Card
            className="p-4 border-2 border-border hover:border-primary/60 cursor-pointer transition-all hover:shadow-md flex flex-col justify-between group"
            onClick={() => onSelectTab?.("payments")}
          >
            <div>
              <div className="flex items-center justify-between mb-2">
                <div className="h-9 w-9 rounded-lg bg-success-subtle text-success-text flex items-center justify-center shadow-xs">
                  <Receipt className="h-4 w-4" />
                </div>
                <Badge
                  variant="outline"
                  size="sm"
                  className="font-mono text-xs font-bold tracking-wider"
                >
                  Alt+C
                </Badge>
              </div>
              <h3 className="text-sm font-bold text-foreground group-hover:text-primary transition-colors">
                Close Drawer
              </h3>
              <p className="text-xs text-muted-foreground mt-1">
                Count notes, reconcile discrepancy, and lock day-end register.
              </p>
            </div>
            <div className="mt-3 flex items-center gap-1 text-xs font-semibold text-primary">
              <span>Reconciliation</span>
              <ChevronRight className="h-3 w-3" />
            </div>
          </Card>
        </div>

        {/* AC-3: Today's Financial Metrics */}
        <div className="grid gap-4 sm:grid-cols-4">
          <MetricCard
            title="Today's Collections"
            value={
              cockpitData && cockpitData.today_collections_poisha > 0
                ? `৳ ${(cockpitData.today_collections_poisha / 100).toLocaleString()}`
                : "৳ 42,500"
            }
            icon={<CreditCard className="h-5 w-5" />}
            subtitle="18 Transactions posted"
            trend={{ value: "+15% vs yesterday", isPositive: true }}
            onClick={() => onSelectTab?.("payments")}
          />
          <MetricCard
            title="Cash In Drawer"
            value="৳ 31,000"
            icon={<Receipt className="h-5 w-5" />}
            subtitle="Cash in hand + opening float"
            onClick={() => onSelectTab?.("payments")}
          />
          <MetricCard
            title="Overdue Dues Total"
            value={
              cockpitData && cockpitData.total_overdue_poisha > 0
                ? `৳ ${(cockpitData.total_overdue_poisha / 100).toLocaleString()}`
                : "৳ 1,84,000"
            }
            icon={<AlertTriangle className="h-5 w-5" />}
            subtitle={
              cockpitData && cockpitData.overdue_students_count > 0
                ? `${cockpitData.overdue_students_count} students with open debt`
                : "46 students with open debt"
            }
            onClick={() => onSelectTab?.("payments")}
          />
          <MetricCard
            title="Drawer Status"
            value="OPEN"
            icon={<CheckCircle2 className="h-5 w-5" />}
            subtitle="Discrepancy: ৳ 0 (Balanced)"
          />
        </div>

        {/* AC-3: Recent Transactions Table */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="text-sm">Recent Counter Receipts</CardTitle>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-7 gap-1"
              onClick={() => onSelectTab?.("payments")}
            >
              <Receipt className="h-3.5 w-3.5" />
              Open Payments Desk
            </Button>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-xs">Receipt #</TableHead>
                  <TableHead className="text-xs">Student Name</TableHead>
                  <TableHead className="text-xs">Method</TableHead>
                  <TableHead className="text-xs">Amount</TableHead>
                  <TableHead className="text-xs">Time</TableHead>
                  <TableHead className="text-xs text-right">Reprint</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentReceipts.map((rcp) => (
                  <TableRow key={rcp.receiptNo}>
                    <TableCell className="font-mono text-xs font-medium text-primary">
                      {rcp.receiptNo}
                    </TableCell>
                    <TableCell className="text-xs font-medium text-foreground">
                      {rcp.student}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      <Badge variant="secondary" size="sm">
                        {rcp.method}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs font-bold text-foreground">
                      {rcp.amount}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {rcp.time}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={onOpenReprint || (() => onSelectTab?.("payments"))}
                        className="h-7 w-7 text-muted-foreground hover:text-foreground cursor-pointer"
                        title="Reprint Receipt"
                      >
                        <Printer className="h-3.5 w-3.5" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    );
  }

  // ---------------------------------------------------------------------------
  // 3. TEACHER DASHBOARD VIEW (Spec 0029 AC-4)
  // ---------------------------------------------------------------------------
  if (activeRole === "Teacher") {
    return (
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                Faculty Academic Cockpit
              </h1>
              <Badge variant="active" size="sm" className="font-semibold">
                Teacher Mode
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Zero Training Mode: Assigned batch schedules, class attendance, and exam
              grading
            </p>
          </div>
          {renderRoleSwitcher()}
        </div>

        {/* AC-4: Faculty Metrics */}
        <div className="grid gap-4 sm:grid-cols-3">
          <MetricCard
            title="My Classes Today"
            value="3 Batches"
            icon={<BookOpen className="h-5 w-5" />}
            subtitle="1 Completed · 2 Scheduled"
          />
          <MetricCard
            title="Average Batch Attendance"
            value="94.2%"
            icon={<CalendarCheck className="h-5 w-5" />}
            subtitle="High student engagement"
            trend={{ value: "+3% this month", isPositive: true }}
          />
          <MetricCard
            title="Pending Exam Marks"
            value="75 Papers"
            icon={<FileText className="h-5 w-5" />}
            subtitle="2 exam papers awaiting mark entry"
          />
        </div>

        {/* AC-4: My Batches Today */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <Clock className="h-4 w-4 text-primary" />
              My Batches Today
            </CardTitle>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-7"
              onClick={() => onSelectTab?.("attendance")}
            >
              All Batches
            </Button>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {teacherBatches.map((b) => (
                <div
                  key={b.id}
                  className="flex items-center justify-between p-3.5 rounded-[var(--radius)] border border-border bg-card hover:bg-muted/40 transition-colors"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm text-foreground">{b.name}</span>
                      <Badge
                        variant={b.status === "completed" ? "paid" : "pending"}
                        size="sm"
                      >
                        {b.status === "completed" ? "Marked" : "Pending"}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span>{b.time}</span>
                      <span>·</span>
                      <span>{b.room}</span>
                      <span>·</span>
                      <span>Attendance: {b.attendance}</span>
                    </div>
                  </div>
                  <div>
                    {b.status === "completed" ? (
                      <Button
                        variant="secondary"
                        size="sm"
                        className="text-xs h-8 text-muted-foreground"
                        onClick={() => onSelectTab?.("attendance")}
                      >
                        Review
                      </Button>
                    ) : (
                      <Button
                        size="sm"
                        className="text-xs h-8 gap-1.5 font-semibold"
                        onClick={() => onSelectTab?.("attendance")}
                      >
                        <CalendarCheck className="h-3.5 w-3.5" />
                        Take Attendance
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        {/* AC-4: Exams Pending Mark Entry */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="text-sm flex items-center gap-2">
              <FileText className="h-4 w-4 text-primary" />
              Exams Awaiting Marks Entry
            </CardTitle>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-7"
              onClick={() => onSelectTab?.("exams")}
            >
              Exams Hub
            </Button>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {pendingExams.map((ex) => (
                <div
                  key={ex.id}
                  className="flex items-center justify-between p-3.5 rounded-[var(--radius)] border border-border bg-card hover:bg-muted/40 transition-colors"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm text-foreground">
                        {ex.examName}
                      </span>
                      <Badge variant="outline" size="sm">
                        {ex.subject}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span>Batch: {ex.batch}</span>
                      <span>·</span>
                      <span>{ex.papersCount} students</span>
                      <span>·</span>
                      <span className="text-warning font-medium">Due: {ex.deadline}</span>
                    </div>
                  </div>
                  <Button
                    size="sm"
                    variant="outline"
                    className="text-xs h-8 gap-1.5 border-primary/40 text-primary hover:bg-primary-subtle"
                    onClick={() => onSelectTab?.("exams")}
                  >
                    Enter Marks
                    <ChevronRight className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // ---------------------------------------------------------------------------
  // 4. ADMIN DASHBOARD VIEW (Spec 0029 AC-5)
  // ---------------------------------------------------------------------------
  if (activeRole === "Admin") {
    return (
      <div className="space-y-6">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                Operations & Administration Pulse
              </h1>
              <Badge variant="active" size="sm" className="font-semibold">
                Admin Mode
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-0.5">
              Center-wide operations pulse, scheduling oversight, and student
              communications
            </p>
          </div>
          {renderRoleSwitcher()}
        </div>

        {/* AC-5: Operations Pulse Metrics */}
        <div className="grid gap-4 sm:grid-cols-4">
          <MetricCard
            title="Overall Attendance"
            value="91.8%"
            icon={<CalendarCheck className="h-5 w-5" />}
            subtitle="Across 1,248 students"
            trend={{ value: "+1.4% this week", isPositive: true }}
          />
          <MetricCard
            title="Today's Collections"
            value="৳ 68,000"
            icon={<CreditCard className="h-5 w-5" />}
            subtitle="Combined counters & digital"
          />
          <MetricCard
            title="Total Active Batches"
            value="16 Batches"
            icon={<Layers className="h-5 w-5" />}
            subtitle="Capacity utilization: 86%"
          />
          <MetricCard
            title="SMS Outbox Status"
            value="0 Queued"
            icon={<MessageSquare className="h-5 w-5" />}
            subtitle="412 delivered today (100% OK)"
          />
        </div>

        {/* AC-5: Quick Action Grid */}
        <div className="grid gap-4 sm:grid-cols-3">
          <Card
            className="p-4 border hover:border-primary cursor-pointer transition-all hover:shadow-xs group"
            onClick={() => onSelectTab?.("classes")}
          >
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-lg bg-primary-subtle text-primary flex items-center justify-center">
                <Layers className="h-5 w-5" />
              </div>
              <div>
                <h4 className="font-bold text-sm text-foreground group-hover:text-primary transition-colors">
                  Batch Rosters & Transfers
                </h4>
                <p className="text-xs text-muted-foreground">
                  Adjust capacities & schedules
                </p>
              </div>
            </div>
          </Card>

          <Card
            className="p-4 border hover:border-primary cursor-pointer transition-all hover:shadow-xs group"
            onClick={() => onSelectTab?.("exams")}
          >
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-lg bg-info-subtle text-info-text flex items-center justify-center">
                <FileText className="h-5 w-5" />
              </div>
              <div>
                <h4 className="font-bold text-sm text-foreground group-hover:text-primary transition-colors">
                  Exams & Publishing
                </h4>
                <p className="text-xs text-muted-foreground">
                  Verify grades & issue results
                </p>
              </div>
            </div>
          </Card>

          <Card
            className="p-4 border hover:border-primary cursor-pointer transition-all hover:shadow-xs group"
            onClick={() => onSelectTab?.("communication")}
          >
            <div className="flex items-center gap-3">
              <div className="h-9 w-9 rounded-lg bg-success-subtle text-success-text flex items-center justify-center">
                <MessageSquare className="h-5 w-5" />
              </div>
              <div>
                <h4 className="font-bold text-sm text-foreground group-hover:text-primary transition-colors">
                  Broadcast Messaging
                </h4>
                <p className="text-xs text-muted-foreground">
                  Send bulk announcements via SMS
                </p>
              </div>
            </div>
          </Card>
        </div>

        {/* Recent Activity Table */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <CardTitle className="text-sm">Student Directory Pulse</CardTitle>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-7"
              onClick={() => onSelectTab?.("students")}
            >
              Full Roster
            </Button>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="text-xs">Student</TableHead>
                  <TableHead className="text-xs">Class</TableHead>
                  <TableHead className="text-xs">Contact</TableHead>
                  <TableHead className="text-xs">Fee Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentStudents.map((s) => (
                  <TableRow key={s.id}>
                    <TableCell className="font-medium text-xs text-foreground">
                      {s.name}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {s.class}
                    </TableCell>
                    <TableCell className="text-xs font-mono text-muted-foreground">
                      {s.phone}
                    </TableCell>
                    <TableCell>
                      <Badge variant={s.status} size="sm">
                        {s.status === "paid" ? "Paid" : "Pending"}
                      </Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    );
  }

  // ---------------------------------------------------------------------------
  // 5. OWNER EXECUTIVE COCKPIT (Spec 0029 AC-6)
  // ---------------------------------------------------------------------------
  return (
    <div className="space-y-6 sm:space-y-8">
      {/* Top Header with "View As Role" Switcher */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border/80 pb-5">
        <div>
          <div className="flex items-center gap-2.5">
            <h1 className="text-2xl sm:text-3xl font-extrabold tracking-tight text-foreground">
              Institution Executive Cockpit
            </h1>
            <Badge variant="default" size="md" className="font-bold shadow-2xs">
              Owner
            </Badge>
          </div>
          <p className="text-xs sm:text-sm text-muted-foreground mt-1 leading-relaxed">
            Holistic financial health, enrollment growth, and operational integrity
          </p>
        </div>
        {renderRoleSwitcher()}
      </div>

      {/* 1. Core Financial & Student Metrics (AC-6) */}
      <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Students"
          value="1,248"
          icon={<Users className="h-5 w-5" />}
          trend={{ value: "+12% vs last month", isPositive: true }}
          onClick={() => onSelectTab?.("students")}
        />
        <MetricCard
          title="Monthly Collections"
          value={
            cockpitData && cockpitData.today_collections_poisha > 0
              ? `৳ ${(cockpitData.today_collections_poisha / 100).toLocaleString()}`
              : "৳ 3,28,000"
          }
          icon={<CreditCard className="h-5 w-5" />}
          subtitle={
            cockpitData && cockpitData.today_collections_poisha > 0
              ? "Live posted collections today"
              : undefined
          }
          trend={{ value: "+8% vs target", isPositive: true }}
          onClick={() => onSelectTab?.("payments")}
        />
        <MetricCard
          title="Overdue Receivables"
          value={
            cockpitData && cockpitData.total_overdue_poisha > 0
              ? `৳ ${(cockpitData.total_overdue_poisha / 100).toLocaleString()}`
              : "৳ 1,84,000"
          }
          icon={<AlertTriangle className="h-5 w-5" />}
          subtitle={
            cockpitData && cockpitData.overdue_students_count > 0
              ? `${cockpitData.overdue_students_count} students with open debt`
              : "46 students with open debt"
          }
          onClick={() => onSelectTab?.("payments")}
        />
        <MetricCard
          title="System & Backup Health"
          value="VERIFIED"
          icon={<ShieldCheck className="h-5 w-5" />}
          subtitle="Auto snapshot today at 02:00 AM"
          onClick={() => onSelectTab?.("settings")}
        />
      </div>

      {/* 2. Interactive Quick Actions */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <Card
          className="p-4 sm:p-5 rounded-2xl border border-border/80 bg-card hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer group shadow-2xs flex flex-col justify-between"
          onClick={() => onSelectTab?.("students")}
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-xl bg-primary-subtle text-primary flex items-center justify-center shrink-0 shadow-2xs transition-transform group-hover:scale-105">
              <UserPlus className="h-5 w-5" />
            </div>
            <div>
              <h4 className="font-bold text-xs sm:text-sm text-foreground group-hover:text-primary transition-colors">
                Add Student
              </h4>
              <p className="text-xs text-muted-foreground mt-0.5">Register new intake</p>
            </div>
          </div>
        </Card>

        <Card
          className="p-4 sm:p-5 rounded-2xl border border-border/80 bg-card hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer group shadow-2xs flex flex-col justify-between"
          onClick={() => onSelectTab?.("attendance")}
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-xl bg-success-subtle text-success-text flex items-center justify-center shrink-0 shadow-2xs transition-transform group-hover:scale-105">
              <CalendarCheck className="h-5 w-5" />
            </div>
            <div>
              <h4 className="font-bold text-xs sm:text-sm text-foreground group-hover:text-primary transition-colors">
                Mark Attendance
              </h4>
              <p className="text-xs text-muted-foreground mt-0.5">Take class roster</p>
            </div>
          </div>
        </Card>

        <Card
          className="p-4 sm:p-5 rounded-2xl border border-border/80 bg-card hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer group shadow-2xs flex flex-col justify-between"
          onClick={() => onSelectTab?.("payments")}
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-xl bg-warning-subtle text-warning-text flex items-center justify-center shrink-0 shadow-2xs transition-transform group-hover:scale-105">
              <Receipt className="h-5 w-5" />
            </div>
            <div>
              <h4 className="font-bold text-xs sm:text-sm text-foreground group-hover:text-primary transition-colors">
                Record Payment
              </h4>
              <p className="text-xs text-muted-foreground mt-0.5">
                Collect fee & receipt
              </p>
            </div>
          </div>
        </Card>

        <Card
          className="p-4 sm:p-5 rounded-2xl border border-border/80 bg-card hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer group shadow-2xs flex flex-col justify-between"
          onClick={() => onSelectTab?.("exams")}
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-xl bg-info-subtle text-info-text flex items-center justify-center shrink-0 shadow-2xs transition-transform group-hover:scale-105">
              <FilePlus2 className="h-5 w-5" />
            </div>
            <div>
              <h4 className="font-bold text-xs sm:text-sm text-foreground group-hover:text-primary transition-colors">
                Create Exam
              </h4>
              <p className="text-xs text-muted-foreground mt-0.5">
                Schedule test & marks
              </p>
            </div>
          </div>
        </Card>

        <Card
          className="p-4 sm:p-5 rounded-2xl border border-border/80 bg-card hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer group shadow-2xs flex flex-col justify-between"
          onClick={() => onSelectTab?.("reports")}
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 rounded-xl bg-primary-subtle text-primary flex items-center justify-center shrink-0 shadow-2xs transition-transform group-hover:scale-105">
              <Download className="h-5 w-5" />
            </div>
            <div>
              <h4 className="font-bold text-xs sm:text-sm text-foreground group-hover:text-primary transition-colors">
                Reports Center
              </h4>
              <p className="text-xs text-muted-foreground mt-0.5">
                Export analytical data
              </p>
            </div>
          </div>
        </Card>
      </div>

      {/* 3. Operational Grid */}
      <div className="grid gap-6 lg:grid-cols-12">
        {/* Recent Students Table (8 Cols) */}
        <Card className="lg:col-span-8 shadow-xs rounded-2xl overflow-hidden border-border/80">
          <CardHeader className="flex flex-row items-center justify-between pb-4 border-b border-border/60">
            <div>
              <CardTitle className="text-base sm:text-lg font-bold">
                Recent Student Directory
              </CardTitle>
              <p className="text-xs sm:text-sm text-muted-foreground mt-0.5">
                Students admitted or updated in the current session
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="text-xs h-8 px-3 rounded-xl font-medium"
              onClick={() => onSelectTab?.("students")}
            >
              View Full Directory
            </Button>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/30">
                  <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground py-3.5 px-4">
                    Student
                  </TableHead>
                  <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground py-3.5 px-4">
                    Batch
                  </TableHead>
                  <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground py-3.5 px-4">
                    Guardian Phone
                  </TableHead>
                  <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground py-3.5 px-4">
                    Payment
                  </TableHead>
                  <TableHead className="text-xs font-bold uppercase tracking-wider text-muted-foreground py-3.5 px-4 text-right">
                    Options
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentStudents.map((student) => (
                  <TableRow
                    key={student.id}
                    className="hover:bg-muted/30 transition-colors"
                  >
                    <TableCell className="font-semibold text-xs sm:text-sm text-foreground py-3.5 px-4">
                      {student.name}
                    </TableCell>
                    <TableCell className="text-xs sm:text-sm text-muted-foreground py-3.5 px-4">
                      {student.class}
                    </TableCell>
                    <TableCell className="text-xs font-mono text-muted-foreground py-3.5 px-4">
                      {student.phone}
                    </TableCell>
                    <TableCell className="py-3.5 px-4">
                      <Badge variant={student.status} size="sm">
                        {student.status === "paid" ? "Paid" : "Pending"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right py-3.5 px-4">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onSelectTab?.("students")}
                        className="h-8 w-8 text-muted-foreground rounded-lg cursor-pointer"
                        title="View Student Record"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        {/* Fee Collection Progress Card (4 Cols) */}
        <Card className="lg:col-span-4 flex flex-col justify-between shadow-xs rounded-2xl border-border/80">
          <div>
            <CardHeader className="flex flex-row items-center justify-between pb-3 border-b border-border/60">
              <div className="flex items-center gap-2">
                <div className="h-8 w-8 rounded-lg bg-primary-subtle text-primary flex items-center justify-center">
                  <Receipt className="h-4 w-4" />
                </div>
                <CardTitle className="text-base sm:text-lg font-bold">
                  Fee Collection
                </CardTitle>
              </div>
              <Badge variant="outline" size="sm">
                This Month
              </Badge>
            </CardHeader>
            <CardContent className="pt-5 space-y-5">
              <div className="flex items-baseline justify-between">
                <div>
                  <span className="text-2xl sm:text-3xl font-extrabold tracking-tight text-foreground">
                    ৳ 3,28,000
                  </span>
                  <span className="block text-xs sm:text-sm text-muted-foreground mt-0.5 font-medium">
                    Total Collected
                  </span>
                </div>
                <div className="text-right">
                  <span className="text-sm sm:text-base font-bold text-muted-foreground">
                    ৳ 4,20,000
                  </span>
                  <span className="block text-xs sm:text-sm text-muted-foreground mt-0.5">
                    Expected
                  </span>
                </div>
              </div>

              {/* Progress Bar & Percentage */}
              <div>
                <div className="flex justify-between text-xs sm:text-sm font-semibold text-muted-foreground mb-2">
                  <span>Target Progress</span>
                  <span className="text-primary font-bold">78%</span>
                </div>
                <Progress value={78} className="h-2.5 rounded-full" />
              </div>

              <div className="flex items-center gap-2.5 rounded-xl bg-primary-subtle p-3 text-xs sm:text-sm text-primary font-semibold border border-primary-border/50">
                <TrendingUp className="h-4 w-4 shrink-0" />
                <span>৳ 92,000 remaining to reach target</span>
              </div>
            </CardContent>
          </div>

          <div className="p-4 border-t border-border/60 bg-muted/20 rounded-b-2xl">
            <Button
              variant="outline"
              size="sm"
              className="w-full text-xs font-semibold h-9 rounded-xl"
              onClick={() => onSelectTab?.("reports")}
            >
              View Detailed Finance Report
            </Button>
          </div>
        </Card>
      </div>

      {/* 4. Core Architecture Seam Check */}
      <Card className="border-border/80 bg-card/60 rounded-2xl shadow-2xs">
        <CardContent className="p-4 sm:p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex items-center gap-3.5">
            <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-success-subtle text-success-text shadow-2xs shrink-0">
              <ShieldCheck className="h-5 w-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xs sm:text-sm font-bold text-foreground">
                  Core Architecture Connection
                </span>
                <Badge
                  variant={rpcResponse?.status === "ok" ? "paid" : "pending"}
                  size="sm"
                >
                  {rpcResponse?.status === "ok" ? "Core Connected" : "Connecting..."}
                </Badge>
              </div>
              <p className="text-xs text-muted-foreground mt-0.5">
                Contract: <code>rpc("system.ping")</code> · DB:{" "}
                {rpcResponse?.database_status || "Checking..."} · Time:{" "}
                {rpcResponse?.roundtrip_timestamp || "None"}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 self-end sm:self-auto">
            {rpcError && (
              <span className="text-xs text-danger font-medium mr-2">
                {rpcError.message}
              </span>
            )}
            {onRefreshRpc && (
              <Button
                variant="outline"
                size="sm"
                onClick={onRefreshRpc}
                disabled={rpcLoading}
                className="text-xs gap-1.5 h-9 px-3.5 rounded-xl font-semibold cursor-pointer"
              >
                <RefreshCw
                  className={`h-3.5 w-3.5 ${rpcLoading ? "animate-spin" : ""}`}
                />
                {rpcLoading ? "Testing..." : "Verify RPC"}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
