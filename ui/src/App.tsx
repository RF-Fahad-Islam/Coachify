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
import { CashierDeskView } from "./components/cashier/cashier-desk-view";
import { DuesCenterView } from "./components/cashier/dues-center-view";
import { CashCloseView } from "./components/cashier/cash-close-view";
import { PrinterSettingsCard } from "./components/settings/printer-settings-card";
import { BackupDashboard } from "./components/backup/backup-dashboard";
import { DiagnosticsView } from "./components/diagnostics/diagnostics-view";
import { UpdateSettingsCard } from "./components/settings/update-settings";
import { CorruptionBanner } from "./components/restore/corruption-banner";
import { RestoreWizard } from "./components/restore/restore-wizard";
import { AttendanceView } from "./components/attendance/attendance-view";
import { CommunicationView } from "./components/communication/communication-view";
import { CommunicationSettings } from "./components/settings/communication-settings";
import { ExamsHubView } from "./components/exams/exams-hub-view";
import {
  FileText,
  BarChart3,
  Settings as SettingsIcon,
  History,
  KeyRound,
  Printer,
  Database,
  CreditCard,
  Calendar,
  HardDrive,
  Stethoscope,
  ArrowUpCircle,
  MessageSquare,
} from "lucide-react";

function AppContent() {
  const { currentUser, loading: authLoading } = useAuth();
  const [activeTab, setActiveTab] = useState("dashboard");
  const [settingsSubTab, setSettingsSubTab] = useState("audit");
  const [restoreWizardOpen, setRestoreWizardOpen] = useState(false);
  const [paymentsSubTab, setPaymentsSubTab] = useState<"counter" | "dues" | "cash_close">(
    "counter"
  );
  const [cashierStudentId, setCashierStudentId] = useState<string | null>(null);
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
        <CorruptionBanner
          onOpenRestore={() => setRestoreWizardOpen(true)}
          onOpenDiagnostics={() => {
            setActiveTab("settings");
            setSettingsSubTab("diagnostics");
          }}
        />

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

        {activeTab === "attendance" && <AttendanceView />}

        {activeTab === "communication" && <CommunicationView />}

        {activeTab === "payments" && (
          <div className="space-y-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Financial Center & Counter Operations
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Counter collections, dues directory with aging breakdown, petty
                  expenses, and daily cash drawer closing.
                </p>
              </div>

              <div className="flex items-center gap-1.5 p-1 rounded-lg bg-muted border border-border">
                <Button
                  variant={paymentsSubTab === "counter" ? "primary" : "ghost"}
                  size="sm"
                  onClick={() => setPaymentsSubTab("counter")}
                  className="h-8 text-xs font-semibold gap-1.5"
                >
                  <CreditCard className="h-3.5 w-3.5" />
                  Counter Cashier
                </Button>
                <Button
                  variant={paymentsSubTab === "dues" ? "primary" : "ghost"}
                  size="sm"
                  onClick={() => setPaymentsSubTab("dues")}
                  className="h-8 text-xs font-semibold gap-1.5"
                >
                  <FileText className="h-3.5 w-3.5" />
                  Dues Directory
                </Button>
                <Button
                  variant={paymentsSubTab === "cash_close" ? "primary" : "ghost"}
                  size="sm"
                  onClick={() => setPaymentsSubTab("cash_close")}
                  className="h-8 text-xs font-semibold gap-1.5"
                >
                  <Calendar className="h-3.5 w-3.5" />
                  Daily Cash Close
                </Button>
              </div>
            </div>

            {paymentsSubTab === "counter" && (
              <CashierDeskView
                initialStudentId={cashierStudentId}
                onSelectStudent={setCashierStudentId}
              />
            )}

            {paymentsSubTab === "dues" && (
              <DuesCenterView
                onCollectPayment={(studentId) => {
                  setCashierStudentId(studentId);
                  setPaymentsSubTab("counter");
                }}
              />
            )}

            {paymentsSubTab === "cash_close" && <CashCloseView />}
          </div>
        )}

        {activeTab === "exams" && <ExamsHubView />}

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
          <Tabs
            value={settingsSubTab}
            onValueChange={setSettingsSubTab}
            className="space-y-6"
          >
            <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-4 border-b border-border pb-4">
              <div>
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  Institute Settings & Security
                </h1>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Manage institution profile, user accounts, backups, diagnostics, and
                  audit trail.
                </p>
              </div>

              <TabsList className="flex flex-wrap h-auto gap-1">
                <TabsTrigger value="general" className="gap-1.5 text-xs">
                  <SettingsIcon className="h-3.5 w-3.5" />
                  General
                </TabsTrigger>
                <TabsTrigger value="backups" className="gap-1.5 text-xs">
                  <HardDrive className="h-3.5 w-3.5" />
                  Backups & Restore
                </TabsTrigger>
                <TabsTrigger value="diagnostics" className="gap-1.5 text-xs">
                  <Stethoscope className="h-3.5 w-3.5" />
                  Diagnostics
                </TabsTrigger>
                <TabsTrigger value="security" className="gap-1.5 text-xs">
                  <KeyRound className="h-3.5 w-3.5" />
                  Key Vault
                </TabsTrigger>
                <TabsTrigger value="audit" className="gap-1.5 text-xs">
                  <History className="h-3.5 w-3.5" />
                  Audit Trail
                </TabsTrigger>
                <TabsTrigger value="printers" className="gap-1.5 text-xs">
                  <Printer className="h-3.5 w-3.5" />
                  Printers
                </TabsTrigger>
                <TabsTrigger value="messaging" className="gap-1.5 text-xs">
                  <MessageSquare className="h-3.5 w-3.5" />
                  Messaging
                </TabsTrigger>
                <TabsTrigger value="updates" className="gap-1.5 text-xs">
                  <ArrowUpCircle className="h-3.5 w-3.5" />
                  Updates
                </TabsTrigger>
                <TabsTrigger value="demo" className="gap-1.5 text-xs">
                  <Database className="h-3.5 w-3.5" />
                  Developer Demo
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

            <TabsContent value="backups">
              <BackupDashboard />
            </TabsContent>

            <TabsContent value="diagnostics">
              <DiagnosticsView />
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

            <TabsContent value="printers">
              <PrinterSettingsCard />
            </TabsContent>

            <TabsContent value="updates">
              <UpdateSettingsCard />
            </TabsContent>

            <TabsContent value="messaging">
              <CommunicationSettings />
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

      {/* Restore Wizard Modal */}
      <RestoreWizard open={restoreWizardOpen} onOpenChange={setRestoreWizardOpen} />
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
