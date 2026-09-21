import { useState } from "react";
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
} from "lucide-react";
import { MetricCard } from "@/components/ui/metric-card";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import type { PingResponse, RpcError } from "@/gen/rpc";

interface DashboardViewProps {
  rpcResponse: PingResponse | null;
  rpcLoading: boolean;
  rpcError: RpcError | null;
  onRefreshRpc: () => void;
}

export function DashboardView({
  rpcResponse,
  rpcLoading,
  rpcError,
  onRefreshRpc,
}: DashboardViewProps) {
  const [trendPeriod, setTrendPeriod] = useState("Last 7 Days");

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

  // Sample quick action items
  const quickActions = [
    {
      title: "Add Student",
      desc: "Register a new student",
      icon: UserPlus,
      color: "text-primary",
      bg: "bg-primary-subtle",
    },
    {
      title: "Mark Attendance",
      desc: "Take attendance for a class",
      icon: CalendarCheck,
      color: "text-success",
      bg: "bg-success-subtle",
    },
    {
      title: "Record Payment",
      desc: "Add or update fee payment",
      icon: Receipt,
      color: "text-warning",
      bg: "bg-warning-subtle",
    },
    {
      title: "Create Exam",
      desc: "Schedule a new exam",
      icon: FilePlus2,
      color: "text-info",
      bg: "bg-info-subtle",
    },
    {
      title: "Generate Report",
      desc: "View and export reports",
      icon: Download,
      color: "text-primary",
      bg: "bg-primary-subtle",
    },
  ];

  // Sample recent activity feed
  const activities = [
    {
      title: "New student added",
      detail: "Aarav Sharma",
      time: "10 minutes ago",
      icon: UserPlus,
      color: "text-primary",
      bg: "bg-primary-subtle",
    },
    {
      title: "Payment received",
      detail: "৳ 12,000 from Priya Singh",
      time: "28 minutes ago",
      icon: Receipt,
      color: "text-success",
      bg: "bg-success-subtle",
    },
    {
      title: "Attendance marked",
      detail: "Class 10A · Mathematics",
      time: "1 hour ago",
      icon: CheckCircle2,
      color: "text-info",
      bg: "bg-info-subtle",
    },
    {
      title: "Exam created",
      detail: "Unit Test · Physics",
      time: "3 hours ago",
      icon: FileText,
      color: "text-primary",
      bg: "bg-primary-subtle",
    },
    {
      title: "New teacher added",
      detail: "Neha Gupta",
      time: "5 hours ago",
      icon: Users,
      color: "text-warning",
      bg: "bg-warning-subtle",
    },
  ];

  return (
    <div className="space-y-6">
      {/* 1. Header Welcome Bar & Inspirational Quote */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Good morning, Rohan! 👋
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Here's what's happening at your institute today.
          </p>
        </div>
        <div className="hidden lg:block max-w-md text-right">
          <p className="text-xs italic text-muted-foreground/80 leading-relaxed">
            "Education is the most powerful weapon which you can use to change the world."
          </p>
          <span className="text-[11px] font-medium text-muted-foreground">
            — Nelson Mandela
          </span>
        </div>
      </div>

      {/* 2. Top 4 Metric KPI Stat Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          title="Total Students"
          value="482"
          subtitle="+52 from last month"
          icon={<Users className="h-5 w-5" />}
          iconBg="bg-primary-subtle"
          iconColor="text-primary"
          trend={{ value: "12%", isPositive: true }}
          onClick={() => {}}
        />
        <MetricCard
          title="Today's Attendance"
          value="412 / 482"
          subtitle="85% attendance rate"
          icon={<CalendarCheck className="h-5 w-5" />}
          iconBg="bg-success-subtle"
          iconColor="text-success-text"
          trend={{ value: "5%", isPositive: true }}
          onClick={() => {}}
        />
        <MetricCard
          title="Pending Payments"
          value="৳ 1,24,500"
          subtitle="37 students pending"
          icon={<CreditCard className="h-5 w-5" />}
          iconBg="bg-warning-subtle"
          iconColor="text-warning-text"
          trend={{ value: "8%", isNeutral: true }}
          onClick={() => {}}
        />
        <MetricCard
          title="Upcoming Exams"
          value="5"
          subtitle="in next 7 days"
          icon={<FileText className="h-5 w-5" />}
          iconBg="bg-info-subtle"
          iconColor="text-info-text"
          onClick={() => {}}
        />
      </div>

      {/* 3. Middle Section: Attendance Trends, Quick Actions, and Recent Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Attendance Trends Chart Card (6 Cols) */}
        <Card className="lg:col-span-6">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <div className="flex items-center gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-[var(--radius-sm)] bg-primary-subtle text-primary">
                <TrendingUp className="h-4 w-4" />
              </div>
              <CardTitle className="text-sm">Attendance Trends</CardTitle>
            </div>
            <select
              value={trendPeriod}
              onChange={(e) => setTrendPeriod(e.target.value)}
              className="text-xs bg-muted/50 border border-border rounded-[var(--radius-sm)] px-2.5 py-1 text-foreground focus:outline-none focus:ring-1 focus:ring-primary cursor-pointer"
            >
              <option>Last 7 Days</option>
              <option>Last 30 Days</option>
              <option>This Month</option>
            </select>
          </CardHeader>
          <CardContent className="pt-4">
            {/* SVG Attendance Line Chart */}
            <div className="relative h-52 w-full pt-4">
              <svg className="w-full h-40 overflow-visible" viewBox="0 0 500 120">
                <defs>
                  <linearGradient id="attendanceGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--primary)" stopOpacity="0.25" />
                    <stop offset="100%" stopColor="var(--primary)" stopOpacity="0.0" />
                  </linearGradient>
                </defs>

                {/* Background Grid Lines */}
                <line
                  x1="0"
                  y1="0"
                  x2="500"
                  y2="0"
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  opacity="0.6"
                />
                <line
                  x1="0"
                  y1="30"
                  x2="500"
                  y2="30"
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  opacity="0.6"
                />
                <line
                  x1="0"
                  y1="60"
                  x2="500"
                  y2="60"
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  opacity="0.6"
                />
                <line
                  x1="0"
                  y1="90"
                  x2="500"
                  y2="90"
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  opacity="0.6"
                />
                <line
                  x1="0"
                  y1="120"
                  x2="500"
                  y2="120"
                  stroke="var(--border)"
                  opacity="0.8"
                />

                {/* Area under curve */}
                <path
                  d="M 20 80 Q 95 65 170 55 T 320 60 T 400 45 T 480 30 L 480 120 L 20 120 Z"
                  fill="url(#attendanceGradient)"
                />

                {/* Smooth Curve */}
                <path
                  d="M 20 80 Q 95 65 170 55 T 320 60 T 400 45 T 480 30"
                  fill="none"
                  stroke="var(--primary)"
                  strokeWidth="3"
                  strokeLinecap="round"
                />

                {/* Data Points */}
                <circle
                  cx="20"
                  cy="80"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
                <circle
                  cx="95"
                  cy="65"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
                <circle
                  cx="170"
                  cy="55"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
                <circle
                  cx="245"
                  cy="45"
                  r="5"
                  fill="var(--primary)"
                  stroke="var(--card)"
                  strokeWidth="2"
                />
                <circle
                  cx="320"
                  cy="60"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
                <circle
                  cx="400"
                  cy="45"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
                <circle
                  cx="480"
                  cy="30"
                  r="4"
                  fill="var(--card)"
                  stroke="var(--primary)"
                  strokeWidth="2.5"
                />
              </svg>

              {/* Tooltip on active point (Apr 24) */}
              <div className="absolute top-2 left-[44%] -translate-x-1/2 rounded-[var(--radius-sm)] bg-card border border-border px-2 py-1 shadow-sm text-center">
                <span className="block text-[10px] text-muted-foreground font-medium">
                  Apr 24
                </span>
                <span className="block text-xs font-bold text-primary">
                  88% attendance
                </span>
              </div>

              {/* X Axis Labels */}
              <div className="flex justify-between text-[11px] text-muted-foreground/80 mt-2 px-1">
                <span>Apr 21</span>
                <span>Apr 22</span>
                <span>Apr 23</span>
                <span className="font-semibold text-primary">Apr 24</span>
                <span>Apr 25</span>
                <span>Apr 26</span>
                <span>Apr 27</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Quick Actions (3 Cols) */}
        <Card className="lg:col-span-3">
          <CardHeader className="pb-2">
            <div className="flex items-center gap-2">
              <span className="text-primary text-sm">⚡</span>
              <CardTitle className="text-sm">Quick Actions</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="space-y-1.5 pt-2">
            {quickActions.map((action, idx) => {
              const Icon = action.icon;
              return (
                <button
                  key={idx}
                  type="button"
                  className="w-full flex items-center justify-between p-2.5 rounded-[var(--radius)] hover:bg-muted/70 hover:shadow-xs border border-transparent hover:border-border/60 transition-all duration-150 text-left group cursor-pointer hover:translate-x-0.5"
                >
                  <div className="flex items-center gap-2.5">
                    <div
                      className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-[var(--radius-sm)] ${action.bg} ${action.color}`}
                    >
                      <Icon className="h-4 w-4" />
                    </div>
                    <div>
                      <p className="text-xs font-semibold text-foreground leading-tight">
                        {action.title}
                      </p>
                      <p className="text-[10px] text-muted-foreground leading-tight mt-0.5">
                        {action.desc}
                      </p>
                    </div>
                  </div>
                  <ChevronRight className="h-3.5 w-3.5 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
                </button>
              );
            })}
          </CardContent>
        </Card>

        {/* Recent Activity Feed (3 Cols) */}
        <Card className="lg:col-span-3">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-sm">Recent Activity</CardTitle>
            </div>
            <button
              type="button"
              className="text-xs font-semibold text-primary hover:underline cursor-pointer"
            >
              View All
            </button>
          </CardHeader>
          <CardContent className="space-y-3 pt-2">
            {activities.map((act, idx) => {
              const Icon = act.icon;
              return (
                <div key={idx} className="flex items-start gap-2.5 text-left">
                  <div
                    className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full ${act.bg} ${act.color} mt-0.5`}
                  >
                    <Icon className="h-3.5 w-3.5" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium text-foreground leading-snug">
                      {act.title}
                    </p>
                    <p className="text-[11px] text-muted-foreground leading-snug truncate">
                      {act.detail}
                    </p>
                    <span className="text-[10px] text-muted-foreground/70">
                      {act.time}
                    </span>
                  </div>
                </div>
              );
            })}
          </CardContent>
        </Card>
      </div>

      {/* 4. Bottom Section: Recent Students Table & Fee Collection Progress */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Recent Students Table (8 Cols) */}
        <Card className="lg:col-span-8">
          <CardHeader className="flex flex-row items-center justify-between pb-3">
            <div className="flex items-center gap-2">
              <Users className="h-4 w-4 text-primary" />
              <CardTitle className="text-sm">Recent Students</CardTitle>
            </div>
            <button
              type="button"
              className="text-xs font-semibold text-primary hover:underline cursor-pointer"
            >
              View All
            </button>
          </CardHeader>
          <CardContent className="p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12 text-center">#</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead>Class</TableHead>
                  <TableHead>Contact</TableHead>
                  <TableHead>Fee Status</TableHead>
                  <TableHead className="w-16 text-right">Action</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentStudents.map((student) => (
                  <TableRow key={student.id}>
                    <TableCell className="text-center font-medium text-muted-foreground text-xs">
                      {student.id}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2.5">
                        <Avatar className="h-7 w-7">
                          <AvatarFallback className="text-[11px]">
                            {student.name
                              .split(" ")
                              .map((n) => n[0])
                              .join("")}
                          </AvatarFallback>
                        </Avatar>
                        <span className="font-semibold text-xs text-foreground">
                          {student.name}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {student.class}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground font-mono">
                      {student.phone}
                    </TableCell>
                    <TableCell>
                      <Badge variant={student.status} size="sm">
                        {student.status === "paid" ? "Paid" : "Pending"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground"
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
        <Card className="lg:col-span-4 flex flex-col justify-between">
          <div>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <div className="flex items-center gap-2">
                <Receipt className="h-4 w-4 text-primary" />
                <CardTitle className="text-sm">Fee Collection</CardTitle>
              </div>
              <select className="text-xs bg-muted/50 border border-border rounded-[var(--radius-sm)] px-2 py-0.5 text-foreground cursor-pointer">
                <option>This Month</option>
                <option>Last Month</option>
              </select>
            </CardHeader>
            <CardContent className="pt-2">
              <div className="flex items-baseline justify-between">
                <div>
                  <span className="text-2xl font-bold tracking-tight text-foreground">
                    ৳ 3,28,000
                  </span>
                  <span className="block text-[11px] text-muted-foreground mt-0.5">
                    Total Collected
                  </span>
                </div>
                <div className="text-right">
                  <span className="text-sm font-semibold text-muted-foreground">
                    ৳ 4,20,000
                  </span>
                  <span className="block text-[11px] text-muted-foreground/70">
                    Expected
                  </span>
                </div>
              </div>

              {/* Progress Bar & Percentage */}
              <div className="mt-4">
                <div className="flex justify-between text-xs font-semibold text-muted-foreground mb-1.5">
                  <span>Target Progress</span>
                  <span className="text-primary font-bold">78%</span>
                </div>
                <Progress value={78} />
              </div>

              <div className="mt-4 flex items-center gap-2 rounded-[var(--radius-sm)] bg-primary-subtle/50 p-2.5 text-xs text-primary font-medium border border-primary-border/40">
                <TrendingUp className="h-4 w-4 shrink-0" />
                <span>৳ 92,000 remaining to reach target</span>
              </div>
            </CardContent>
          </div>

          <div className="p-4 border-t border-border/60 bg-muted/20 rounded-b-[var(--radius-lg)]">
            <Button variant="subtle" size="sm" className="w-full text-xs">
              View Detailed Finance Report
            </Button>
          </div>
        </Card>
      </div>

      {/* 5. Core Architectural Seam Check: RPC Ping Verification Bar */}
      <Card className="border-border/80 bg-muted/30">
        <CardContent className="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-full bg-success-subtle text-success-text">
              <ShieldCheck className="h-5 w-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xs font-bold text-foreground">
                  Core Architecture Connection
                </span>
                <Badge
                  variant={rpcResponse?.status === "ok" ? "paid" : "pending"}
                  size="sm"
                >
                  {rpcResponse?.status === "ok" ? "Core Connected" : "Connecting..."}
                </Badge>
              </div>
              <p className="text-[11px] text-muted-foreground mt-0.5">
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
            <Button
              variant="outline"
              size="sm"
              onClick={onRefreshRpc}
              disabled={rpcLoading}
              className="text-xs gap-1.5 h-8"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${rpcLoading ? "animate-spin" : ""}`} />
              {rpcLoading ? "Testing..." : "Verify RPC"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
