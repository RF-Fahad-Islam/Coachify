import { useState, useEffect, useCallback } from "react";
import { rpc, type ProgramSummary, type BatchSummary } from "@/gen/rpc";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CreateProgramDialog } from "./create-program-dialog";
import { CreateBatchDialog } from "./create-batch-dialog";
import { BatchDetailView } from "./batch-detail-view";
import { useAuth } from "@/contexts/auth-context";
import {
  Layers,
  Plus,
  Search,
  BookOpen,
  GraduationCap,
  MapPin,
  Users,
  ChevronRight,
  Loader2,
} from "lucide-react";

export interface AcademicClassesViewProps {
  onSelectStudent?: (studentId: string) => void;
}

export function AcademicClassesView({ onSelectStudent }: AcademicClassesViewProps = {}) {
  const { hasPermission } = useAuth();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [programs, setPrograms] = useState<ProgramSummary[]>([]);
  const [selectedProgramId, setSelectedProgramId] = useState<string>("all");

  const [batches, setBatches] = useState<BatchSummary[]>([]);

  // Filtering
  const [searchQuery, setSearchQuery] = useState("");
  const [shiftFilter, setShiftFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("active");

  // Active view: grid vs batch detail
  const [activeBatchId, setActiveBatchId] = useState<string | null>(null);

  // Dialogs
  const [createProgramOpen, setCreateProgramOpen] = useState(false);
  const [createBatchOpen, setCreateBatchOpen] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [progRes, batchRes] = await Promise.all([
        rpc("programs.list", {}),
        rpc("batches.list", {
          program_id: selectedProgramId !== "all" ? selectedProgramId : null,
          status: statusFilter !== "all" ? statusFilter : null,
        }),
      ]);

      if (progRes.ok && progRes.data) {
        setPrograms(progRes.data);
      }
      if (batchRes.ok && batchRes.data) {
        setBatches(batchRes.data);
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Failed to load academic data");
    } finally {
      setLoading(false);
    }
  }, [selectedProgramId, statusFilter]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  // Filter batches
  const filteredBatches = batches.filter((b) => {
    if (selectedProgramId !== "all" && b.program_id !== selectedProgramId) {
      return false;
    }
    if (shiftFilter !== "all" && b.shift.toLowerCase() !== shiftFilter.toLowerCase()) {
      return false;
    }
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      b.name.toLowerCase().includes(q) ||
      b.code.toLowerCase().includes(q) ||
      b.program_name.toLowerCase().includes(q) ||
      (b.teacher_name && b.teacher_name.toLowerCase().includes(q)) ||
      (b.room_number && b.room_number.toLowerCase().includes(q))
    );
  });

  // If a batch is selected, render the batch detail view
  if (activeBatchId) {
    return (
      <BatchDetailView
        batchId={activeBatchId}
        onBack={() => {
          setActiveBatchId(null);
          void loadData();
        }}
        onSelectStudent={onSelectStudent}
      />
    );
  }

  const selectedProgram = programs.find((p) => p.id === selectedProgramId);

  return (
    <div className="space-y-6">
      {/* Header & Primary Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Academic Programs & Batches
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Manage course streams, classroom sections, weekly schedules, and rosters.
          </p>
        </div>

        {hasPermission("academic.manage") && (
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setCreateProgramOpen(true)}
              className="gap-1.5 text-xs"
            >
              <BookOpen className="h-3.5 w-3.5" />
              New Program
            </Button>
            <Button
              size="sm"
              onClick={() => setCreateBatchOpen(true)}
              className="gap-1.5 text-xs"
            >
              <Plus className="h-3.5 w-3.5" />
              New Batch / Section
            </Button>
          </div>
        )}
      </div>

      {error && (
        <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
          {error}
        </div>
      )}

      {/* Program Selector Bar */}
      <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card shadow-xs space-y-3">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <span className="text-xs font-semibold text-muted-foreground whitespace-nowrap">
              Program Stream:
            </span>
            <select
              value={selectedProgramId}
              onChange={(e) => setSelectedProgramId(e.target.value)}
              aria-label="Filter by program stream"
              className="h-8 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs font-medium focus:outline-none focus:ring-2 focus:ring-primary"
            >
              <option value="all">All Programs ({programs.length})</option>
              {programs.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} ({p.code}) — {p.batch_count} batches
                </option>
              ))}
            </select>
          </div>

          {selectedProgram && (
            <div className="flex items-center gap-3 text-xs text-muted-foreground flex-wrap">
              {selectedProgram.class_level && (
                <Badge variant="outline" className="text-[11px]">
                  {selectedProgram.class_level}
                </Badge>
              )}
              {selectedProgram.group_major && (
                <Badge variant="secondary" className="text-[11px]">
                  {selectedProgram.group_major}
                </Badge>
              )}
              <span>•</span>
              <span>
                <strong>{selectedProgram.batch_count}</strong> Batches
              </span>
              <span>•</span>
              <span>
                <strong>{selectedProgram.active_student_count}</strong> Active Students
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Filter Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
        <div className="relative w-full sm:w-80">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search batches by name, code, teacher..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 text-xs"
          />
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          {/* Shift selector */}
          <select
            value={shiftFilter}
            onChange={(e) => setShiftFilter(e.target.value)}
            aria-label="Filter by shift"
            className="h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
          >
            <option value="all">All Shifts</option>
            <option value="Morning">Morning</option>
            <option value="Day">Day</option>
            <option value="Evening">Evening</option>
            <option value="Weekend">Weekend</option>
          </select>

          {/* Status filter */}
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            aria-label="Filter by status"
            className="h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
          >
            <option value="all">All Statuses</option>
            <option value="active">Active Only</option>
            <option value="completed">Completed</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </div>
      </div>

      {/* Batch Cards Grid */}
      {loading ? (
        <div className="py-24 flex flex-col items-center justify-center gap-3">
          <Loader2 className="h-6 w-6 animate-spin text-primary" />
          <span className="text-xs text-muted-foreground font-medium">
            Loading batches & classes...
          </span>
        </div>
      ) : filteredBatches.length === 0 ? (
        <div className="py-20 text-center rounded-[var(--radius-lg)] border border-dashed border-border p-8 bg-card/40 space-y-3">
          <Layers className="h-10 w-10 text-muted-foreground/40 mx-auto" />
          <div className="font-semibold text-foreground text-sm">No Batches Found</div>
          <p className="text-xs text-muted-foreground max-w-sm mx-auto">
            {programs.length === 0
              ? "Start by creating your first academic program, then create class sections."
              : "No batches match the selected program or search filter."}
          </p>
          {hasPermission("academic.manage") && (
            <div className="pt-2 flex justify-center gap-2">
              {programs.length === 0 ? (
                <Button
                  size="sm"
                  onClick={() => setCreateProgramOpen(true)}
                  className="gap-1.5 text-xs"
                >
                  <BookOpen className="h-3.5 w-3.5" />
                  Create First Program
                </Button>
              ) : (
                <Button
                  size="sm"
                  onClick={() => setCreateBatchOpen(true)}
                  className="gap-1.5 text-xs"
                >
                  <Plus className="h-3.5 w-3.5" />
                  Add Section / Batch
                </Button>
              )}
            </div>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredBatches.map((b) => {
            const fillPct = Math.min(
              100,
              Math.round((b.enrolled_count / b.capacity) * 100)
            );
            return (
              <Card
                key={b.id}
                className="group hover:border-primary/40 hover:shadow-md transition-all duration-200 cursor-pointer flex flex-col justify-between"
                onClick={() => setActiveBatchId(b.id)}
              >
                <CardHeader className="pb-3 space-y-2">
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <span className="text-[11px] font-medium text-muted-foreground block">
                        {b.program_name}
                      </span>
                      <CardTitle className="text-base font-bold text-foreground group-hover:text-primary transition-colors">
                        {b.name}
                      </CardTitle>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground font-semibold">
                        {b.code}
                      </span>
                      <Badge variant="outline" className="text-[10px] capitalize">
                        {b.shift}
                      </Badge>
                    </div>
                  </div>

                  {/* Teacher & Room */}
                  <div className="space-y-1 text-xs text-muted-foreground">
                    <div className="flex items-center gap-2">
                      <GraduationCap className="h-3.5 w-3.5 shrink-0 text-primary" />
                      <span className="truncate">
                        {b.teacher_name || "Unassigned Lead Teacher"}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <MapPin className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      <span>{b.room_number || "Room TBD"}</span>
                    </div>
                  </div>
                </CardHeader>

                <CardContent className="space-y-3 pt-0">
                  {/* Schedule Pills */}
                  <div className="flex flex-wrap gap-1">
                    {b.schedule && b.schedule.length > 0 ? (
                      b.schedule.slice(0, 3).map((s, idx) => (
                        <span
                          key={idx}
                          className="px-1.5 py-0.5 rounded bg-muted/70 text-foreground text-[10px] font-mono"
                        >
                          {s.day_of_week.slice(0, 3)} {s.start_time}
                        </span>
                      ))
                    ) : (
                      <span className="text-[11px] text-muted-foreground italic">
                        No schedule set
                      </span>
                    )}
                    {b.schedule && b.schedule.length > 3 && (
                      <span className="text-[10px] text-muted-foreground">
                        +{b.schedule.length - 3} more
                      </span>
                    )}
                  </div>

                  {/* Capacity Bar */}
                  <div className="space-y-1">
                    <div className="flex justify-between text-[11px]">
                      <span className="text-muted-foreground">Enrollment:</span>
                      <span className="font-mono font-medium text-foreground">
                        {b.enrolled_count} / {b.capacity} seats ({b.available_seats} left)
                      </span>
                    </div>
                    <div className="w-full bg-muted rounded-full h-1.5 overflow-hidden">
                      <div
                        className={`h-full ${
                          fillPct >= 100
                            ? "bg-destructive"
                            : fillPct >= 80
                              ? "bg-amber-500"
                              : "bg-primary"
                        }`}
                        style={{ width: `${fillPct}%` }}
                      />
                    </div>
                  </div>

                  {/* Footer button */}
                  <div className="pt-2 border-t border-border/50 flex items-center justify-between">
                    <span className="text-[11px] text-muted-foreground flex items-center gap-1">
                      <Users className="h-3 w-3" />
                      {b.enrolled_count} students
                    </span>
                    <span className="text-xs font-medium text-primary flex items-center gap-0.5 group-hover:translate-x-0.5 transition-transform">
                      View Roster
                      <ChevronRight className="h-3.5 w-3.5" />
                    </span>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Dialogs */}
      <CreateProgramDialog
        open={createProgramOpen}
        onOpenChange={setCreateProgramOpen}
        onCreated={() => void loadData()}
      />

      <CreateBatchDialog
        open={createBatchOpen}
        onOpenChange={setCreateBatchOpen}
        programs={programs}
        defaultProgramId={selectedProgramId !== "all" ? selectedProgramId : undefined}
        onCreated={() => void loadData()}
      />
    </div>
  );
}
