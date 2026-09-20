import { useCallback, useEffect, useState } from "react";
import { rpc, type PingResponse, type RpcError } from "./gen/rpc";
import { AuthProvider } from "./contexts/auth-context";
import { IdleLockDialog } from "./components/auth/idle-lock-dialog";
import { BootstrapOwnerDialog } from "./components/auth/bootstrap-owner-dialog";
import { LoginDialog } from "./components/auth/login-dialog";
import { AppShell } from "./components/layout/app-shell";
import { DashboardView } from "./components/dashboard/dashboard-view";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import { Button } from "./components/ui/button";
import {
  Plus,
  Users,
  CalendarCheck,
  CreditCard,
  FileText,
  BarChart3,
  Settings as SettingsIcon,
} from "lucide-react";

function AppContent() {
  const [activeTab, setActiveTab] = useState("dashboard");
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<PingResponse | null>(null);
  const [error, setError] = useState<RpcError | null>(null);

  const runPing = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("system.ping", {
        message: "Ping from UI",
      });

      if (res.ok && res.data) {
        setResponse(res.data);
      } else if (res.error) {
        setError(res.error);
      }
    } catch (e: unknown) {
      setError({
        code: "client_exception",
        message: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void runPing();
  }, [runPing]);

  return (
    <>
      <AppShell activeTab={activeTab} onTabChange={setActiveTab}>
        {activeTab === "dashboard" && (
          <DashboardView
            rpcResponse={response}
            rpcLoading={loading}
            rpcError={error}
            onRefreshRpc={runPing}
          />
        )}

        {activeTab === "students" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Students Directory
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Manage enrolled students, profiles, and batches.
                </p>
              </div>
              <Button size="sm" className="gap-1.5">
                <Plus className="h-4 w-4" />
                Add Student
              </Button>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Users className="h-4 w-4 text-primary" />
                  Student Directory View (Module 2)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  This view inherits all design system tokens: current palette, font
                  family, density, and scale.
                </p>
              </CardContent>
            </Card>
          </div>
        )}

        {activeTab === "attendance" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Batch Attendance
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Take daily attendance and review monthly trends.
                </p>
              </div>
              <Button size="sm" className="gap-1.5">
                <CalendarCheck className="h-4 w-4" />
                Mark All Present
              </Button>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <CalendarCheck className="h-4 w-4 text-primary" />
                  Attendance Grid (Module 5)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  High-density matrix with Present, Absent, Late, and Excused radio
                  buttons per student.
                </p>
              </CardContent>
            </Card>
          </div>
        )}

        {activeTab === "payments" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Finance & Fee Ledger
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Collect fees, print receipts, and track due balances.
                </p>
              </div>
              <Button size="sm" className="gap-1.5">
                <CreditCard className="h-4 w-4" />
                Receive Payment
              </Button>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <CreditCard className="h-4 w-4 text-primary" />
                  Cashier Payment Desk (Module 6 & 7)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  Supports rapid fee collection in integer poisha with offline receipt
                  printing.
                </p>
              </CardContent>
            </Card>
          </div>
        )}

        {activeTab === "exams" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Exams & Mark Entry
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Create exam schedules, enter subject marks, and publish report cards.
                </p>
              </div>
              <Button size="sm" className="gap-1.5">
                <Plus className="h-4 w-4" />
                Create Exam
              </Button>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <FileText className="h-4 w-4 text-primary" />
                  Exam Management (Module 8 & 9)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  Subject-wise mark entry with instant calculation and missing mark
                  warnings.
                </p>
              </CardContent>
            </Card>
          </div>
        )}

        {activeTab === "reports" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Analytical Reports
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Export collection, attendance, and exam reports to PDF or Excel.
                </p>
              </div>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <BarChart3 className="h-4 w-4 text-primary" />
                  Reporting Center (Module 12)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  Generates high-fidelity PDF reports with bundled Bangla and Latin
                  typography.
                </p>
              </CardContent>
            </Card>
          </div>
        )}

        {activeTab === "settings" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Institute Settings
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Manage institution profile, user roles, notifications, and styling.
                </p>
              </div>
            </div>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <SettingsIcon className="h-4 w-4 text-primary" />
                  Configuration & Appearance (Module 13)
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-xs text-muted-foreground">
                  Theme and typography customizer is accessible globally via the top bar
                  palette icon and left sidebar banner.
                </p>
              </CardContent>
            </Card>
          </div>
        )}
      </AppShell>

      {/* Auth & Setup Modals */}
      <BootstrapOwnerDialog />
      <LoginDialog />
      <IdleLockDialog />
    </>
  );
}

export function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}
