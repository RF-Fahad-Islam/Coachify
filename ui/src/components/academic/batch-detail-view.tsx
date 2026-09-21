import { useState, useEffect, useCallback } from "react";
import { rpc, type BatchDetail, type BatchRosterItem } from "@/gen/rpc";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { EnrollStudentDialog } from "./enroll-student-dialog";
import { TransferStudentDialog } from "./transfer-student-dialog";
import { EndEnrollmentDialog } from "./end-enrollment-dialog";
import { BulkEnrollDialog } from "./bulk-enroll-dialog";
import { useAuth } from "@/contexts/auth-context";
import { formatPhone, formatBusinessDate } from "@/lib/utils";
import {
  ArrowLeft,
  UserPlus,
  Users,
  Search,
  Clock,
  MapPin,
  GraduationCap,
  ArrowRightLeft,
  UserMinus,
  Loader2,
} from "lucide-react";

export interface BatchDetailViewProps {
  batchId: string;
  onBack: () => void;
  onSelectStudent?: (studentId: string) => void;
}

export function BatchDetailView({
  batchId,
  onBack,
  onSelectStudent,
}: BatchDetailViewProps) {
  const { hasPermission } = useAuth();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [batch, setBatch] = useState<BatchDetail | null>(null);
  const [roster, setRoster] = useState<BatchRosterItem[]>([]);

  // Filters
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | "active" | "ended">("active");

  // Dialogs
  const [enrollDialogOpen, setEnrollDialogOpen] = useState(false);
  const [bulkDialogOpen, setBulkDialogOpen] = useState(false);
  const [transferTarget, setTransferTarget] = useState<BatchRosterItem | null>(null);
  const [endTarget, setEndTarget] = useState<BatchRosterItem | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [batchRes, rosterRes] = await Promise.all([
        rpc("batches.get", { id: batchId }),
        rpc("enrollments.list_by_batch", { batch_id: batchId }),
      ]);

      if (batchRes.ok && batchRes.data) {
        setBatch(batchRes.data);
      } else {
        setError(batchRes.error?.message || "Failed to load batch details");
      }

      if (rosterRes.ok && rosterRes.data) {
        setRoster(rosterRes.data);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load data");
    } finally {
      setLoading(false);
    }
  }, [batchId]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  // Filter roster
  const filteredRoster = roster.filter((item) => {
    if (statusFilter !== "all" && item.status !== statusFilter) {
      return false;
    }
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      item.student_name.toLowerCase().includes(q) ||
      (item.student_name_bn && item.student_name_bn.toLowerCase().includes(q)) ||
      item.student_code.toLowerCase().includes(q) ||
      (item.roll !== null &&
        item.roll !== undefined &&
        item.roll.toString().includes(q)) ||
      item.phone_norm.includes(q)
    );
  });

  const activeCount = roster.filter((r) => r.status === "active").length;
  const capacity = batch?.capacity || 40;
  const fillPercentage = Math.min(100, Math.round((activeCount / capacity) * 100));

  return (
    <div className="space-y-6">
      {/* Breadcrumb & Navigation */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="sm"
            onClick={onBack}
            className="h-8 gap-1.5 text-xs text-muted-foreground hover:text-foreground"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to Classes
          </Button>
          <span className="text-muted-foreground text-xs">•</span>
          <span className="text-xs text-muted-foreground">
            {batch?.program_name || "Program"}
          </span>
          <span className="text-muted-foreground text-xs">/</span>
          <span className="text-xs font-semibold text-foreground">
            {batch?.name || "Batch Roster"}
          </span>
        </div>

        {hasPermission("academic.manage") && (
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setBulkDialogOpen(true)}
              className="gap-1.5 text-xs"
            >
              <Users className="h-3.5 w-3.5" />
              Bulk Enroll
            </Button>
            <Button
              size="sm"
              onClick={() => setEnrollDialogOpen(true)}
              className="gap-1.5 text-xs"
            >
              <UserPlus className="h-3.5 w-3.5" />
              Enroll Student
            </Button>
          </div>
        )}
      </div>

      {error && (
        <div className="p-4 rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20 text-sm">
          {error}
        </div>
      )}

      {/* Batch Header Summary Card */}
      {batch && (
        <div className="p-5 rounded-[var(--radius-lg)] border border-border bg-card shadow-xs space-y-4">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div>
              <div className="flex items-center gap-2.5 flex-wrap">
                <h1 className="text-2xl font-bold tracking-tight text-foreground">
                  {batch.name}
                </h1>
                <span className="font-mono text-xs px-2 py-0.5 rounded bg-primary/10 text-primary font-bold">
                  {batch.code}
                </span>
                <Badge variant={batch.status === "active" ? "default" : "secondary"}>
                  {batch.status}
                </Badge>
                <Badge variant="outline" className="text-xs">
                  {batch.shift} Shift
                </Badge>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Program:{" "}
                <span className="font-medium text-foreground">{batch.program_name}</span>{" "}
                • Started {formatBusinessDate(batch.start_date)}
              </p>
            </div>

            {/* Occupancy Progress Meter */}
            <div className="min-w-[220px] space-y-1.5 p-3 rounded-[var(--radius-md)] bg-muted/30 border border-border/60">
              <div className="flex justify-between text-xs font-semibold">
                <span className="text-muted-foreground">Enrolled Students:</span>
                <span className="font-mono text-foreground">
                  {activeCount} / {capacity}
                </span>
              </div>
              <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
                <div
                  className={`h-full transition-all duration-300 ${
                    fillPercentage >= 100
                      ? "bg-destructive"
                      : fillPercentage >= 80
                        ? "bg-amber-500"
                        : "bg-primary"
                  }`}
                  style={{ width: `${fillPercentage}%` }}
                />
              </div>
              <div className="flex justify-between text-[11px] text-muted-foreground">
                <span>{capacity - activeCount} seats left</span>
                <span>{fillPercentage}% full</span>
              </div>
            </div>
          </div>

          {/* Teacher, Room, and Schedule */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-3 border-t border-border/60 text-xs">
            <div className="flex items-center gap-2.5">
              <div className="h-8 w-8 rounded-full bg-primary/10 text-primary flex items-center justify-center font-bold">
                <GraduationCap className="h-4 w-4" />
              </div>
              <div>
                <span className="text-muted-foreground block text-[11px]">
                  Lead Teacher
                </span>
                <span className="font-semibold text-foreground">
                  {batch.teacher_name || "Unassigned"}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-2.5">
              <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-muted-foreground">
                <MapPin className="h-4 w-4" />
              </div>
              <div>
                <span className="text-muted-foreground block text-[11px]">Classroom</span>
                <span className="font-semibold text-foreground">
                  {batch.room_number || "Room TBD"}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-2.5">
              <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-muted-foreground">
                <Clock className="h-4 w-4" />
              </div>
              <div>
                <span className="text-muted-foreground block text-[11px]">Timetable</span>
                <div className="flex flex-wrap gap-1 mt-0.5">
                  {batch.schedule && batch.schedule.length > 0 ? (
                    batch.schedule.map((s, idx) => (
                      <span
                        key={idx}
                        className="px-1.5 py-0.5 rounded bg-muted text-foreground text-[10px] font-mono"
                      >
                        {s.day_of_week.slice(0, 3)} {s.start_time}-{s.end_time}
                      </span>
                    ))
                  ) : (
                    <span className="text-muted-foreground">Flexible</span>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Roster Controls */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
        <div className="relative w-full sm:w-72">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search by name, roll, code..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 text-xs"
          />
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <div className="flex items-center rounded-[var(--radius)] border border-input p-0.5 bg-muted/20 text-xs">
            <button
              type="button"
              onClick={() => setStatusFilter("active")}
              className={`px-3 py-1 rounded-[var(--radius-sm)] font-medium transition-colors ${
                statusFilter === "active"
                  ? "bg-background text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Active ({roster.filter((r) => r.status === "active").length})
            </button>
            <button
              type="button"
              onClick={() => setStatusFilter("ended")}
              className={`px-3 py-1 rounded-[var(--radius-sm)] font-medium transition-colors ${
                statusFilter === "ended"
                  ? "bg-background text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Ended ({roster.filter((r) => r.status === "ended").length})
            </button>
            <button
              type="button"
              onClick={() => setStatusFilter("all")}
              className={`px-3 py-1 rounded-[var(--radius-sm)] font-medium transition-colors ${
                statusFilter === "all"
                  ? "bg-background text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              All ({roster.length})
            </button>
          </div>
        </div>
      </div>

      {/* High-Density Roster Table */}
      <div className="rounded-[var(--radius-lg)] border border-border bg-card shadow-xs overflow-hidden">
        {loading ? (
          <div className="py-16 flex flex-col items-center justify-center gap-3">
            <Loader2 className="h-6 w-6 animate-spin text-primary" />
            <span className="text-xs text-muted-foreground">Loading batch roster...</span>
          </div>
        ) : filteredRoster.length === 0 ? (
          <div className="py-16 text-center space-y-2">
            <Users className="h-8 w-8 text-muted-foreground/40 mx-auto" />
            <div className="font-semibold text-foreground text-sm">
              No students enrolled
            </div>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              There are no {statusFilter !== "all" ? statusFilter : ""} students in this
              batch matching your search.
            </p>
            {hasPermission("academic.manage") && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => setEnrollDialogOpen(true)}
                className="mt-2 text-xs"
              >
                Enroll First Student
              </Button>
            )}
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/40 hover:bg-muted/40">
                <TableHead className="w-16 text-center font-bold">Roll</TableHead>
                <TableHead className="w-28 font-bold">Code</TableHead>
                <TableHead className="font-bold">Student Name</TableHead>
                <TableHead className="w-36 font-bold">Student Phone</TableHead>
                <TableHead className="w-40 font-bold">Guardian</TableHead>
                <TableHead className="w-28 font-bold">Enrolled</TableHead>
                <TableHead className="w-24 text-center font-bold">Status</TableHead>
                {hasPermission("academic.manage") && (
                  <TableHead className="w-28 text-right font-bold">Actions</TableHead>
                )}
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredRoster.map((item) => (
                <TableRow
                  key={item.enrollment_id}
                  className="group hover:bg-muted/40 transition-colors"
                >
                  <TableCell className="text-center font-mono font-bold text-foreground">
                    {item.roll ? `#${item.roll}` : "—"}
                  </TableCell>

                  <TableCell className="font-mono text-xs text-primary font-semibold">
                    <button
                      type="button"
                      onClick={() => onSelectStudent?.(item.student_id)}
                      className="hover:underline cursor-pointer"
                    >
                      {item.student_code}
                    </button>
                  </TableCell>

                  <TableCell>
                    <div>
                      <button
                        type="button"
                        onClick={() => onSelectStudent?.(item.student_id)}
                        className="font-medium text-foreground hover:text-primary transition-colors cursor-pointer text-left"
                      >
                        {item.student_name}
                      </button>
                      {item.student_name_bn && (
                        <span className="block text-[11px] text-muted-foreground font-bengali">
                          {item.student_name_bn}
                        </span>
                      )}
                    </div>
                  </TableCell>

                  <TableCell className="font-mono text-xs text-muted-foreground">
                    {formatPhone(item.phone_norm)}
                  </TableCell>

                  <TableCell className="text-xs text-muted-foreground">
                    <div>
                      <span className="text-foreground">{item.guardian_name || "—"}</span>
                      {item.guardian_phone && (
                        <span className="block text-[11px] font-mono">
                          {formatPhone(item.guardian_phone)}
                        </span>
                      )}
                    </div>
                  </TableCell>

                  <TableCell className="text-xs text-muted-foreground whitespace-nowrap">
                    {formatBusinessDate(item.start_date)}
                  </TableCell>

                  <TableCell className="text-center">
                    <Badge
                      variant={item.status === "active" ? "default" : "secondary"}
                      className="capitalize text-[10px]"
                    >
                      {item.status}
                    </Badge>
                  </TableCell>

                  {hasPermission("academic.manage") && (
                    <TableCell className="text-right">
                      {item.status === "active" ? (
                        <div className="flex items-center justify-end gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            title="Transfer Section"
                            onClick={() => setTransferTarget(item)}
                            className="h-7 w-7 p-0 text-muted-foreground hover:text-primary"
                          >
                            <ArrowRightLeft className="h-3.5 w-3.5" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            title="End Enrollment"
                            onClick={() => setEndTarget(item)}
                            className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                          >
                            <UserMinus className="h-3.5 w-3.5" />
                          </Button>
                        </div>
                      ) : (
                        <span className="text-[11px] text-muted-foreground">Ended</span>
                      )}
                    </TableCell>
                  )}
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>

      {/* Modals */}
      <EnrollStudentDialog
        open={enrollDialogOpen}
        onOpenChange={setEnrollDialogOpen}
        batch={batch}
        roster={roster}
        onEnrolled={() => void loadData()}
      />

      <BulkEnrollDialog
        open={bulkDialogOpen}
        onOpenChange={setBulkDialogOpen}
        batch={batch}
        roster={roster}
        onBulkEnrolled={() => void loadData()}
      />

      <TransferStudentDialog
        open={Boolean(transferTarget)}
        onOpenChange={(op) => !op && setTransferTarget(null)}
        currentBatchId={batchId}
        currentBatchName={batch?.name || ""}
        rosterItem={transferTarget}
        onTransferred={() => void loadData()}
      />

      <EndEnrollmentDialog
        open={Boolean(endTarget)}
        onOpenChange={(op) => !op && setEndTarget(null)}
        rosterItem={endTarget}
        onEnded={() => void loadData()}
      />
    </div>
  );
}
