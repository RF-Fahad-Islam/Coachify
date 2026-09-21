import { useCallback, useEffect, useState } from "react";
import {
  rpc,
  type PingResponse,
  type RpcError,
  type RecoveryInfoResponse,
} from "./gen/rpc";
import { AuthProvider, useAuth } from "./contexts/auth-context";
import { IdleLockDialog } from "./components/auth/idle-lock-dialog";
import { StartupAuthPage } from "./components/auth/startup-auth-page";
import { MigrationRecoveryScreen } from "./components/system/migration-recovery-screen";
import { AppShell } from "./components/layout/app-shell";
import { DashboardView } from "./components/dashboard/dashboard-view";
import { Card, CardContent, CardHeader, CardTitle } from "./components/ui/card";
import { Button } from "./components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./components/ui/tabs";
import { AuditLogView } from "./components/settings/audit-log-view";
import { ReprintRecoveryDialog } from "./components/settings/reprint-recovery-dialog";
import { DeveloperDemoCard } from "./components/settings/demo-generator-dialog";
import { StudentDirectoryView } from "./components/students/student-directory-view";
import { AcademicClassesView } from "./components/academic/academic-classes-view";
import { StaffDirectoryView } from "./components/staff/staff-directory-view";
import {
  Plus,
  CalendarCheck,
  CreditCard,
  FileText,
  BarChart3,
  Settings as SettingsIcon,
  History,
  KeyRound,
  Printer,
  Database,
} from "lucide-react";

function AppContent() {
  const { currentUser, loading: authLoading } = useAuth();
  const [activeTab, setActiveTab] = useState("dashboard");
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<PingResponse | null>(null);
  const [error, setError] = useState<RpcError | null>(null);
  const [recoveryInfo, setRecoveryInfo] = useState<RecoveryInfoResponse | null>(null);
  const [recoveryChecked, setRecoveryChecked] = useState(false);
  const [reprintRecoveryOpen, setReprintRecoveryOpen] = useState(false);
  const [selectedStudentId, setSelectedStudentId] = useState<string | null>(null);

  const checkRecovery = useCallback(async () => {
    try {
      const res = await rpc("system.recovery_info", {});
      setRecoveryInfo(res.ok && res.data ? res.data : null);
    } catch (e: unknown) {
      console.error("Failed to check recovery status:", e);
      setRecoveryInfo(null);
    } finally {
      setRecoveryChecked(true);
    }
  }, []);

  const retryMigration = useCallback(async () => {
    try {
      const res = await rpc("system.retry_migration", {});
      setRecoveryInfo(res.ok && res.data ? res.data : null);
    } catch (e: unknown) {
      console.error("Failed to retry migration:", e);
    }
  }, []);

  useEffect(() => {
    void checkRecovery();
  }, [checkRecovery]);

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
    if (currentUser) {
      void runPing();
    }
  }, [currentUser, runPing]);

  if (recoveryInfo && recoveryInfo.is_in_recovery) {
    return (
      <MigrationRecoveryScreen
        recoveryInfo={recoveryInfo}
        onRetry={() => void retryMigration()}
      />
    );
  }

  if (!currentUser) {
    if (authLoading || !recoveryChecked) {
      return (
        <div className="min-h-screen w-full flex items-center justify-center bg-background text-foreground">
          <div className="flex flex-col items-center gap-3">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
            <span className="text-xs text-muted-foreground font-medium">
              Initializing Coachify...
            </span>
          </div>
        </div>
      );
    }
    return <StartupAuthPage />;
  }

  return (
    <>
      <AppShell
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onSelectStudent={(studentId) => {
          setSelectedStudentId(studentId);
          setActiveTab("students");
        }}
      >
        {activeTab === "dashboard" && (
          <DashboardView
            rpcResponse={response}
            rpcLoading={loading}
            rpcError={error}
            onRefreshRpc={runPing}
          />
        )}

        {activeTab === "students" && (
          <StudentDirectoryView
            initialStudentId={selectedStudentId}
            onClearInitialStudentId={() => setSelectedStudentId(null)}
          />
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

        {activeTab === "classes" && (
          <AcademicClassesView
            onSelectStudent={(studentId) => {
              setSelectedStudentId(studentId);
              setActiveTab("students");
            }}
          />
        )}

        {activeTab === "teachers" && <StaffDirectoryView />}

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
          <Tabs defaultValue="audit" className="space-y-6">
            <div className="flex items-center justify-between border-b border-border pb-4">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Institute Settings & Security
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Manage institution profile, user accounts, appearance, and audit trail.
                </p>
              </div>

              <TabsList>
                <TabsTrigger value="general" className="gap-1.5 text-xs">
                  <SettingsIcon className="h-3.5 w-3.5" />
                  General & Appearance
                </TabsTrigger>
                <TabsTrigger value="security" className="gap-1.5 text-xs">
                  <KeyRound className="h-3.5 w-3.5" />
                  Key Vault & Recovery
                </TabsTrigger>
                <TabsTrigger value="audit" className="gap-1.5 text-xs">
                  <History className="h-3.5 w-3.5" />
                  Audit Log & Chain
                </TabsTrigger>
                <TabsTrigger value="demo" className="gap-1.5 text-xs">
                  <Database className="h-3.5 w-3.5" />
                  Developer & Demo
                </TabsTrigger>
              </TabsList>
            </div>

            <TabsContent value="general">
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
            </TabsContent>

            <TabsContent value="security">
              <div className="grid gap-6 md:grid-cols-2">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2 text-base">
                      <KeyRound className="h-4 w-4 text-primary" />
                      Two-Tier Encryption Envelope
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3 text-xs text-muted-foreground">
                    <p>
                      Coachify protects your institute database with a dual-envelope
                      architecture:
                    </p>
                    <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-1.5">
                      <div className="font-semibold text-foreground flex items-center gap-1.5">
                        <span className="h-2 w-2 rounded-full bg-emerald-500" />
                        Windows DPAPI Machine Scope
                      </div>
                      <p className="text-[11px]">
                        Enables seamless daily counter operations across any shared local
                        Windows user account without asking cashiers for master keys.
                      </p>
                    </div>
                    <div className="p-3 rounded-lg bg-muted/60 border border-border space-y-1.5">
                      <div className="font-semibold text-foreground flex items-center gap-1.5">
                        <span className="h-2 w-2 rounded-full bg-blue-500" />
                        Argon2id + AES-256-GCM Recovery Key
                      </div>
                      <p className="text-[11px]">
                        Allows emergency restoration on new hardware if Windows OS is
                        reinstalled or the drive is moved.
                      </p>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2 text-base">
                      <Printer className="h-4 w-4 text-primary" />
                      Physical Recovery Sheet
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <p className="text-xs text-muted-foreground leading-relaxed">
                      If you lost your printed paper key or need a fresh copy for the
                      institute vault or bank safety deposit box, you can reprint the
                      official Owner Recovery Sheet.
                    </p>
                    <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 text-xs">
                      Reprinting requires confirming the active Owner password and creates
                      an immutable entry in the institutional audit log.
                    </div>
                    <Button
                      onClick={() => setReprintRecoveryOpen(true)}
                      className="gap-2 font-semibold"
                    >
                      <Printer className="h-4 w-4" />
                      Reprint Recovery Sheet
                    </Button>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            <TabsContent value="audit">
              <AuditLogView />
            </TabsContent>

            <TabsContent value="demo">
              <DeveloperDemoCard />
            </TabsContent>
          </Tabs>
        )}
      </AppShell>

      {/* Idle Inactivity Lock Overlay */}
      <IdleLockDialog />

      {/* Owner Recovery Sheet Reprint Modal */}
      <ReprintRecoveryDialog
        open={reprintRecoveryOpen}
        onOpenChange={setReprintRecoveryOpen}
      />
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
