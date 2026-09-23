import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type TodayCockpitResponse,
  type MissingAttendanceAlert,
  type TodayScheduledClass,
} from "@/gen/rpc";
import { formatTaka, formatBusinessDate } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Clock,
  CreditCard,
  AlertTriangle,
  CheckCircle2,
  Phone,
  ArrowRight,
  ShieldCheck,
  ShieldAlert,
  GraduationCap,
  MessageSquareWarning,
  RefreshCw,
  Loader2,
  ChevronLeft,
  ChevronRight,
  Layers,
  MapPin,
  Users,
} from "lucide-react";

export interface TodayCockpitViewProps {
  onNavigateTab?: (tabId: string) => void;
  onOpenAttendance?: (batchId?: string) => void;
  onCollectPayment?: (studentId?: string) => void;
}

export function TodayCockpitView({
  onNavigateTab,
  onOpenAttendance,
}: TodayCockpitViewProps) {
  const [selectedDate, setSelectedDate] = useState<string>(() => {
    return new Date().toISOString().slice(0, 10);
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cockpit, setCockpit] = useState<TodayCockpitResponse | null>(null);

  const loadCockpit = useCallback(async (date: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await rpc("intelligence.get_today_cockpit", {
        target_date: date,
      });
      if (res.ok && res.data) {
        setCockpit(res.data);
      } else {
        setError(res.error?.message || "Failed to load today cockpit data");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadCockpit(selectedDate);
  }, [selectedDate, loadCockpit]);

  const handleDateChange = (newDate: string) => {
    setSelectedDate(newDate);
  };

  const handleShiftDate = (days: number) => {
    const current = new Date(selectedDate);
    current.setDate(current.getDate() + days);
    setSelectedDate(current.toISOString().slice(0, 10));
  };

  const isToday = selectedDate === new Date().toISOString().slice(0, 10);

  return (
    <div className="space-y-6">
      {/* Date Bar & Header Controls */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-border pb-4">
        <div>
          <div className="flex items-center gap-2">
            <h2 className="text-xl font-bold tracking-tight text-foreground">
              Daily Operations Cockpit
            </h2>
            {isToday ? (
              <Badge variant="active" size="sm">
                Live Today
              </Badge>
            ) : (
              <Badge variant="outline" size="sm">
                Historical / Projected
              </Badge>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-0.5">
            Real-time coaching center operational pulse: attendance vigilance, daily cash
            desk, schedules, and alerts.
          </p>
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <div className="flex items-center bg-card border border-border rounded-[var(--radius-md)] p-1 shadow-2xs">
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={() => handleShiftDate(-1)}
              title="Previous Day"
              aria-label="Previous day"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <input
              type="date"
              aria-label="Select Target Date"
              value={selectedDate}
              onChange={(e) => e.target.value && handleDateChange(e.target.value)}
              className="px-2 py-0.5 text-xs font-semibold bg-transparent border-0 focus:outline-hidden text-foreground cursor-pointer"
            />
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={() => handleShiftDate(1)}
              title="Next Day"
              aria-label="Next day"
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>

          {!isToday && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedDate(new Date().toISOString().slice(0, 10))}
              className="text-xs h-8 font-medium"
            >
              Jump to Today
            </Button>
          )}

          <Button
            variant="ghost"
            size="sm"
            onClick={() => void loadCockpit(selectedDate)}
            disabled={loading}
            className="h-8 w-8 p-0"
            title="Refresh"
            aria-label="Refresh today cockpit data"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          </Button>
        </div>
      </div>

      {error && (
        <div className="p-4 rounded-[var(--radius-lg)] bg-destructive/10 border border-destructive/20 text-destructive text-xs flex items-center justify-between">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => void loadCockpit(selectedDate)}
            className="h-7 text-xs border-destructive/30"
          >
            Retry
          </Button>
        </div>
      )}

      {loading && !cockpit ? (
        <div className="py-20 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Aggregating today coaching operations...
          </span>
        </div>
      ) : cockpit ? (
        <div className="space-y-6">
          {/* Top KPI Metrics Row */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Collections Card */}
            <Card className="border-border/80 bg-card shadow-2xs hover:shadow-xs transition-shadow">
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <span className="text-xs font-semibold text-muted-foreground">
                      Collections Today
                    </span>
                    <div className="text-2xl font-bold tracking-tight text-foreground">
                      {formatTaka(cockpit.today_collections_poisha)}
                    </div>
                    <p className="text-[11px] text-muted-foreground">
                      {cockpit.collections_by_method.reduce(
                        (sum, m) => sum + m.transaction_count,
                        0
                      )}{" "}
                      receipts across {cockpit.collections_by_method.length} channels
                    </p>
                  </div>
                  <div className="h-10 w-10 rounded-[var(--radius-md)] bg-primary-subtle text-primary flex items-center justify-center shrink-0">
                    <CreditCard className="h-5 w-5" />
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Missing Attendance Vigilance Card */}
            <Card
              className={`border-border/80 bg-card shadow-2xs transition-shadow ${
                cockpit.missing_attendance_alerts.length > 0
                  ? "border-amber-500/50 bg-amber-500/5 dark:bg-amber-950/10"
                  : ""
              }`}
            >
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <span className="text-xs font-semibold text-muted-foreground">
                      Missing Attendance
                    </span>
                    <div
                      className={`text-2xl font-bold tracking-tight ${
                        cockpit.missing_attendance_alerts.length > 0
                          ? "text-amber-600 dark:text-amber-400"
                          : "text-foreground"
                      }`}
                    >
                      {cockpit.missing_attendance_alerts.length}
                    </div>
                    <p className="text-[11px] text-muted-foreground">
                      {cockpit.missing_attendance_alerts.length === 0
                        ? "All scheduled classes accounted for"
                        : "Classes ended without submitted roll call"}
                    </p>
                  </div>
                  <div
                    className={`h-10 w-10 rounded-[var(--radius-md)] flex items-center justify-center shrink-0 ${
                      cockpit.missing_attendance_alerts.length > 0
                        ? "bg-amber-500/20 text-amber-600 dark:text-amber-400"
                        : "bg-emerald-500/10 text-emerald-600"
                    }`}
                  >
                    {cockpit.missing_attendance_alerts.length > 0 ? (
                      <AlertTriangle className="h-5 w-5" />
                    ) : (
                      <CheckCircle2 className="h-5 w-5" />
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Timetable Classes Card */}
            <Card className="border-border/80 bg-card shadow-2xs">
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <span className="text-xs font-semibold text-muted-foreground">
                      Scheduled Slots
                    </span>
                    <div className="text-2xl font-bold tracking-tight text-foreground">
                      {cockpit.scheduled_classes.length}
                    </div>
                    <p className="text-[11px] text-muted-foreground">
                      {
                        cockpit.scheduled_classes.filter((c) => c.attendance_submitted)
                          .length
                      }{" "}
                      completed with roll call
                    </p>
                  </div>
                  <div className="h-10 w-10 rounded-[var(--radius-md)] bg-blue-500/10 text-blue-600 flex items-center justify-center shrink-0">
                    <Clock className="h-5 w-5" />
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Overdue Dues Status */}
            <Card className="border-border/80 bg-card shadow-2xs">
              <CardContent className="p-4">
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <span className="text-xs font-semibold text-muted-foreground">
                      Institute Overdue Dues
                    </span>
                    <div className="text-2xl font-bold tracking-tight text-destructive">
                      {formatTaka(cockpit.total_overdue_poisha)}
                    </div>
                    <p className="text-[11px] text-muted-foreground">
                      {cockpit.overdue_students_count} students with overdue fees
                    </p>
                  </div>
                  <div className="h-10 w-10 rounded-[var(--radius-md)] bg-destructive/10 text-destructive flex items-center justify-center shrink-0">
                    <Users className="h-5 w-5" />
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* High Priority Alerts Section */}
          {cockpit.missing_attendance_alerts.length > 0 && (
            <div className="rounded-[var(--radius-lg)] border border-amber-500/40 bg-amber-500/10 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 text-amber-900 dark:text-amber-200">
                  <AlertTriangle className="h-5 w-5 shrink-0 text-amber-600 dark:text-amber-400" />
                  <div>
                    <h3 className="text-sm font-bold">
                      Attendance Missing Action Required (
                      {cockpit.missing_attendance_alerts.length})
                    </h3>
                    <p className="text-xs text-amber-800 dark:text-amber-300">
                      The following classes have completed or are in session today without
                      attendance submitted.
                    </p>
                  </div>
                </div>
                {onOpenAttendance && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => onOpenAttendance()}
                    className="h-8 text-xs border-amber-500/40 text-amber-900 dark:text-amber-200 hover:bg-amber-500/20 gap-1.5"
                  >
                    Open Attendance Sheet
                    <ArrowRight className="h-3.5 w-3.5" />
                  </Button>
                )}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
                {cockpit.missing_attendance_alerts.map(
                  (alert: MissingAttendanceAlert) => (
                    <div
                      key={`${alert.batch_id}-${alert.start_time}`}
                      className="p-3 rounded-[var(--radius-md)] bg-card border border-amber-500/30 flex items-center justify-between text-xs shadow-2xs"
                    >
                      <div className="space-y-1">
                        <div className="flex items-center gap-2">
                          <span className="font-bold text-foreground">
                            {alert.batch_name}
                          </span>
                          <Badge variant="outline" size="sm">
                            {alert.program_name}
                          </Badge>
                        </div>
                        <div className="flex items-center gap-3 text-muted-foreground text-[11px]">
                          <span className="flex items-center gap-1 font-mono">
                            <Clock className="h-3 w-3" />
                            {alert.start_time} - {alert.end_time}
                          </span>
                          {alert.room && (
                            <span className="flex items-center gap-1">
                              <MapPin className="h-3 w-3" />
                              {alert.room}
                            </span>
                          )}
                          <span>{alert.enrolled_count} students</span>
                        </div>
                        <div className="text-[11px] text-foreground font-medium">
                          Teacher: {alert.teacher_name || "Unassigned"}
                        </div>
                      </div>

                      <div className="flex items-center gap-1.5">
                        {alert.teacher_phone && (
                          <a
                            href={`tel:${alert.teacher_phone}`}
                            className="p-2 rounded-[var(--radius-sm)] border border-border bg-muted hover:bg-muted/80 text-foreground flex items-center gap-1 text-[11px] font-semibold"
                            title={`Call ${alert.teacher_name}`}
                          >
                            <Phone className="h-3.5 w-3.5 text-primary" />
                            <span>Call</span>
                          </a>
                        )}
                        {onOpenAttendance && (
                          <Button
                            size="sm"
                            variant="primary"
                            onClick={() => onOpenAttendance(alert.batch_id)}
                            className="h-8 text-xs font-semibold gap-1"
                          >
                            Mark
                          </Button>
                        )}
                      </div>
                    </div>
                  )
                )}
              </div>
            </div>
          )}

          {/* Failed Messages Banner */}
          {cockpit.failed_messages_count > 0 && (
            <div className="p-3 rounded-[var(--radius-md)] bg-destructive/10 border border-destructive/20 text-destructive flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <MessageSquareWarning className="h-4 w-4 shrink-0" />
                <span>
                  <strong>{cockpit.failed_messages_count} Outbox Messages Failed:</strong>{" "}
                  SMS or WhatsApp notifications require operator inspection or resend.
                </span>
              </div>
              {onNavigateTab && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onNavigateTab("communication")}
                  className="h-7 text-xs border-destructive/30 text-destructive hover:bg-destructive/10"
                >
                  View Outbox
                </Button>
              )}
            </div>
          )}

          {/* Scheduled Classes & Timetable */}
          <Card className="border-border">
            <CardHeader className="pb-3 border-b border-border/70 flex flex-row items-center justify-between">
              <div>
                <CardTitle className="text-base flex items-center gap-2">
                  <Layers className="h-4 w-4 text-primary" />
                  Today Class Timetable ({cockpit.scheduled_classes.length})
                </CardTitle>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Scheduled routines and real-time roll call status for{" "}
                  {formatBusinessDate(selectedDate)}
                </p>
              </div>
            </CardHeader>
            <CardContent className="p-0">
              {cockpit.scheduled_classes.length === 0 ? (
                <div className="py-12 text-center text-xs text-muted-foreground">
                  No classes scheduled on this day.
                </div>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full text-xs text-left border-collapse">
                    <thead>
                      <tr className="border-b border-border bg-muted/40 font-semibold text-muted-foreground">
                        <th className="py-2.5 px-4">Time Slot</th>
                        <th className="py-2.5 px-4">Batch & Program</th>
                        <th className="py-2.5 px-4">Room</th>
                        <th className="py-2.5 px-4">Assigned Teacher</th>
                        <th className="py-2.5 px-4">Roll Call Status</th>
                        <th className="py-2.5 px-4 text-right">Action</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border/60">
                      {cockpit.scheduled_classes.map(
                        (c: TodayScheduledClass, idx: number) => (
                          <tr
                            key={`${c.batch_id}-${c.start_time}-${idx}`}
                            className="hover:bg-muted/30 transition-colors"
                          >
                            <td className="py-2.5 px-4 font-mono font-medium text-foreground whitespace-nowrap">
                              {c.start_time} - {c.end_time}
                            </td>
                            <td className="py-2.5 px-4">
                              <span className="font-bold text-foreground block">
                                {c.batch_name}
                              </span>
                              <span className="text-[11px] text-muted-foreground">
                                {c.program_name}
                              </span>
                            </td>
                            <td className="py-2.5 px-4">
                              {c.room ? (
                                <span className="px-2 py-0.5 rounded bg-muted text-foreground text-[11px] font-medium">
                                  {c.room}
                                </span>
                              ) : (
                                <span className="text-muted-foreground">—</span>
                              )}
                            </td>
                            <td className="py-2.5 px-4 font-medium text-foreground">
                              {c.teacher_name || (
                                <span className="text-muted-foreground italic">
                                  Unassigned
                                </span>
                              )}
                            </td>
                            <td className="py-2.5 px-4">
                              {c.attendance_submitted ? (
                                <Badge variant="paid" size="sm">
                                  <CheckCircle2 className="h-3 w-3 mr-1" />
                                  Submitted
                                </Badge>
                              ) : c.is_completed ? (
                                <Badge variant="overdue" size="sm">
                                  <AlertTriangle className="h-3 w-3 mr-1" />
                                  Missing
                                </Badge>
                              ) : (
                                <Badge variant="pending" size="sm">
                                  <Clock className="h-3 w-3 mr-1" />
                                  Pending
                                </Badge>
                              )}
                            </td>
                            <td className="py-2.5 px-4 text-right">
                              {onOpenAttendance && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => onOpenAttendance(c.batch_id)}
                                  className="h-7 text-xs font-semibold text-primary hover:bg-primary-subtle"
                                >
                                  Roll Call
                                </Button>
                              )}
                            </td>
                          </tr>
                        )
                      )}
                    </tbody>
                  </table>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Bottom Grid: Collections by Method + Upcoming Exams + Backup Health */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {/* Collections Breakdown Card */}
            <Card className="border-border">
              <CardHeader className="pb-3 border-b border-border/70">
                <CardTitle className="text-sm flex items-center gap-2">
                  <CreditCard className="h-4 w-4 text-primary" />
                  Collections by Method
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-3">
                {cockpit.collections_by_method.length === 0 ? (
                  <p className="text-xs text-muted-foreground text-center py-4">
                    No counter receipts recorded for this date.
                  </p>
                ) : (
                  cockpit.collections_by_method.map((m) => (
                    <div
                      key={m.method}
                      className="flex items-center justify-between text-xs p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border/50"
                    >
                      <div className="space-y-0.5">
                        <span className="font-bold uppercase tracking-wider text-foreground">
                          {m.method}
                        </span>
                        <span className="text-[11px] text-muted-foreground block">
                          {m.transaction_count} transaction
                          {m.transaction_count === 1 ? "" : "s"}
                        </span>
                      </div>
                      <span className="font-bold font-mono text-sm text-foreground">
                        {formatTaka(m.total_poisha)}
                      </span>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>

            {/* Upcoming Exams (7-Day Lookahead) */}
            <Card className="border-border">
              <CardHeader className="pb-3 border-b border-border/70">
                <CardTitle className="text-sm flex items-center gap-2">
                  <GraduationCap className="h-4 w-4 text-primary" />
                  Upcoming Exams (7 Days)
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-3">
                {cockpit.upcoming_exams.length === 0 ? (
                  <p className="text-xs text-muted-foreground text-center py-4">
                    No upcoming examinations in the next 7 days.
                  </p>
                ) : (
                  cockpit.upcoming_exams.map((exam) => (
                    <div
                      key={exam.exam_id}
                      className="p-3 rounded-[var(--radius-md)] bg-muted/30 border border-border/50 text-xs space-y-1"
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-bold text-foreground">
                          {exam.exam_name}
                        </span>
                        <Badge variant="outline" size="sm">
                          {formatBusinessDate(exam.exam_date)}
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between text-[11px] text-muted-foreground">
                        <span>{exam.batch_name}</span>
                        <span>
                          {exam.subject_count} subjects • {exam.enrolled_candidates_count}{" "}
                          students
                        </span>
                      </div>
                    </div>
                  ))
                )}
              </CardContent>
            </Card>

            {/* Backup & System Health */}
            <Card className="border-border">
              <CardHeader className="pb-3 border-b border-border/70">
                <CardTitle className="text-sm flex items-center gap-2">
                  {cockpit.backup_health.is_healthy ? (
                    <ShieldCheck className="h-4 w-4 text-emerald-500" />
                  ) : (
                    <ShieldAlert className="h-4 w-4 text-destructive" />
                  )}
                  System Backup Status
                </CardTitle>
              </CardHeader>
              <CardContent className="p-4 space-y-3 text-xs">
                <div className="p-3 rounded-[var(--radius-md)] bg-muted/40 border border-border space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Status:</span>
                    {cockpit.backup_health.is_healthy ? (
                      <Badge variant="paid" size="sm">
                        Protected
                      </Badge>
                    ) : (
                      <Badge variant="overdue" size="sm">
                        Backup Overdue
                      </Badge>
                    )}
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Last Backup:</span>
                    <span className="font-mono text-foreground">
                      {cockpit.backup_health.last_backup_at
                        ? formatBusinessDate(
                            cockpit.backup_health.last_backup_at.slice(0, 10)
                          )
                        : "Never"}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground">Elapsed:</span>
                    <span className="font-semibold text-foreground">
                      {cockpit.backup_health.hours_since_last_backup !== null &&
                      cockpit.backup_health.hours_since_last_backup !== undefined
                        ? `${cockpit.backup_health.hours_since_last_backup}h ago`
                        : "N/A"}
                    </span>
                  </div>
                </div>

                {onNavigateTab && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onNavigateTab("settings")}
                    className="w-full text-xs gap-1.5"
                  >
                    Manage Backups
                  </Button>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      ) : null}
    </div>
  );
}
