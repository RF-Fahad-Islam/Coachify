import { useState, useEffect, useCallback } from "react";
import {
  rpc,
  type BatchDetail,
  type ProgramSummary,
  type StaffSummary,
  type ScheduleSlot,
  type ScheduleConflictItem,
} from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Layers, Plus, Trash2, AlertTriangle, Loader2, Clock } from "lucide-react";

export interface CreateBatchDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  programs: ProgramSummary[];
  defaultProgramId?: string;
  onCreated: (batch: BatchDetail) => void;
}

const DAYS = [
  "Saturday",
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
];

export function CreateBatchDialog({
  open,
  onOpenChange,
  programs,
  defaultProgramId,
  onCreated,
}: CreateBatchDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form fields
  const [programId, setProgramId] = useState(defaultProgramId || "");
  const [name, setName] = useState("");
  const [code, setCode] = useState("");
  const [teacherStaffId, setTeacherStaffId] = useState("");
  const [roomNumber, setRoomNumber] = useState("");
  const [shift, setShift] = useState("Morning");
  const [capacity, setCapacity] = useState("40");
  const [startDate, setStartDate] = useState(
    () => new Date().toISOString().split("T")[0]
  );

  // Schedule slots
  const [schedule, setSchedule] = useState<ScheduleSlot[]>([
    {
      day_of_week: "Sunday",
      start_time: "08:00",
      end_time: "09:30",
      room: null,
    },
    {
      day_of_week: "Tuesday",
      start_time: "08:00",
      end_time: "09:30",
      room: null,
    },
    {
      day_of_week: "Thursday",
      start_time: "08:00",
      end_time: "09:30",
      room: null,
    },
  ]);

  // Teachers list
  const [teachers, setTeachers] = useState<StaffSummary[]>([]);
  const [conflicts, setConflicts] = useState<ScheduleConflictItem[]>([]);
  const [checkingConflicts, setCheckingConflicts] = useState(false);
  const [allowConflict, setAllowConflict] = useState(false);

  useEffect(() => {
    if (defaultProgramId) {
      setProgramId(defaultProgramId);
    } else if (programs.length > 0 && !programId) {
      setProgramId(programs[0].id);
    }
  }, [defaultProgramId, programs, programId]);

  useEffect(() => {
    if (open) {
      // Load teachers
      void (async () => {
        try {
          const res = await rpc("staff.list", { status: "active", limit: 100 });
          if (res.ok && res.data) {
            setTeachers(res.data);
          }
        } catch {
          // ignore
        }
      })();
    }
  }, [open]);

  // Check schedule conflicts
  const checkConflicts = useCallback(async () => {
    if (schedule.length === 0 || (!teacherStaffId && !roomNumber)) {
      setConflicts([]);
      return;
    }

    setCheckingConflicts(true);
    try {
      const res = await rpc("batches.check_conflicts", {
        teacher_staff_id: teacherStaffId || null,
        room_number: roomNumber || null,
        schedule,
      });

      if (res.ok && res.data) {
        setConflicts(res.data);
      } else {
        setConflicts([]);
      }
    } catch {
      setConflicts([]);
    } finally {
      setCheckingConflicts(false);
    }
  }, [schedule, teacherStaffId, roomNumber]);

  useEffect(() => {
    const timer = setTimeout(() => {
      void checkConflicts();
    }, 400);
    return () => clearTimeout(timer);
  }, [checkConflicts]);

  const addSlot = () => {
    setSchedule((prev) => [
      ...prev,
      {
        day_of_week: "Monday",
        start_time: "10:00",
        end_time: "11:30",
        room: roomNumber || null,
      },
    ]);
  };

  const removeSlot = (index: number) => {
    setSchedule((prev) => prev.filter((_, i) => i !== index));
  };

  const updateSlot = (index: number, field: keyof ScheduleSlot, val: string | null) => {
    setSchedule((prev) =>
      prev.map((slot, i) => (i === index ? { ...slot, [field]: val } : slot))
    );
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!programId) {
      setError("Please select a program");
      return;
    }
    if (!name.trim()) {
      setError("Batch name is required");
      return;
    }
    if (!code.trim()) {
      setError("Batch code is required");
      return;
    }

    if (conflicts.length > 0 && !allowConflict) {
      setError(
        "There are unresolved schedule collisions. Please adjust the timetable or enable conflict override."
      );
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const res = await rpc("batches.create", {
        batch: {
          program_id: programId,
          name: name.trim(),
          code: code.trim().toUpperCase(),
          teacher_staff_id: teacherStaffId || null,
          room_number: roomNumber.trim() || null,
          shift,
          capacity: capacity ? parseInt(capacity, 10) : 40,
          start_date: startDate,
          schedule: schedule.map((s) => ({
            ...s,
            room: s.room?.trim() || roomNumber.trim() || null,
          })),
        },
        allow_conflict: allowConflict,
      });

      if (res.ok && res.data) {
        onOpenChange(false);
        onCreated(res.data);
      } else {
        setError(res.error?.message || "Failed to create batch");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <Layers className="h-5 w-5 text-primary" />
            Create Academic Batch / Section
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 pt-2">
          {error && (
            <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
              {error}
            </div>
          )}

          {/* Program & Name */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Academic Program <span className="text-destructive">*</span>
              </label>
              <select
                value={programId}
                onChange={(e) => setProgramId(e.target.value)}
                required
                disabled={loading}
                aria-label="Academic Program"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="">Select Program...</option>
                {programs.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} ({p.code})
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Batch Name <span className="text-destructive">*</span>
              </label>
              <Input
                placeholder="e.g. Section A (Morning)"
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (!code) {
                    setCode(
                      e.target.value
                        .toUpperCase()
                        .replace(/[^A-Z0-9]/g, "-")
                        .slice(0, 10)
                    );
                  }
                }}
                required
                disabled={loading}
              />
            </div>
          </div>

          {/* Code, Shift, Capacity */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Batch Code <span className="text-destructive">*</span>
              </label>
              <Input
                placeholder="SEC-A"
                value={code}
                onChange={(e) => setCode(e.target.value.toUpperCase())}
                required
                disabled={loading}
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">Shift</label>
              <select
                value={shift}
                onChange={(e) => setShift(e.target.value)}
                disabled={loading}
                aria-label="Shift"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="Morning">Morning (সকাল)</option>
                <option value="Day">Day (দিন)</option>
                <option value="Evening">Evening (সন্ধ্যা)</option>
                <option value="Weekend">Weekend (সাপ্তাহিক ছুটি)</option>
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Max Capacity
              </label>
              <Input
                type="number"
                min="1"
                max="250"
                value={capacity}
                onChange={(e) => setCapacity(e.target.value)}
                disabled={loading}
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">Start Date</label>
              <Input
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
                required
                disabled={loading}
              />
            </div>
          </div>

          {/* Lead Teacher & Room */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Lead Teacher / Guide
              </label>
              <select
                value={teacherStaffId}
                onChange={(e) => setTeacherStaffId(e.target.value)}
                disabled={loading}
                aria-label="Lead Teacher"
                className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs focus:outline-none focus:ring-2 focus:ring-primary"
              >
                <option value="">Unassigned</option>
                {teachers.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name} ({t.designation})
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                Default Room / Classroom
              </label>
              <Input
                placeholder="e.g. Room 204"
                value={roomNumber}
                onChange={(e) => setRoomNumber(e.target.value)}
                disabled={loading}
              />
            </div>
          </div>

          {/* Schedule Slots Section */}
          <div className="space-y-2 pt-2 border-t border-border">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-primary" />
                <span className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                  Weekly Class Timetable ({schedule.length} slots)
                </span>
                {checkingConflicts && (
                  <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />
                )}
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={addSlot}
                disabled={loading}
                className="h-7 text-xs gap-1"
              >
                <Plus className="h-3.5 w-3.5" />
                Add Slot
              </Button>
            </div>

            <div className="space-y-2">
              {schedule.map((slot, idx) => (
                <div
                  key={idx}
                  className="grid grid-cols-12 gap-2 items-center p-2 rounded-[var(--radius-md)] bg-muted/40 border border-border/60 text-xs"
                >
                  <div className="col-span-4">
                    <select
                      value={slot.day_of_week}
                      onChange={(e) => updateSlot(idx, "day_of_week", e.target.value)}
                      aria-label="Day of week"
                      className="w-full h-8 px-2 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
                    >
                      {DAYS.map((d) => (
                        <option key={d} value={d}>
                          {d}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="col-span-3">
                    <Input
                      type="time"
                      value={slot.start_time}
                      onChange={(e) => updateSlot(idx, "start_time", e.target.value)}
                      className="h-8 text-xs font-mono"
                    />
                  </div>

                  <div className="col-span-3">
                    <Input
                      type="time"
                      value={slot.end_time}
                      onChange={(e) => updateSlot(idx, "end_time", e.target.value)}
                      className="h-8 text-xs font-mono"
                    />
                  </div>

                  <div className="col-span-2 flex justify-end">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={() => removeSlot(idx)}
                      disabled={schedule.length <= 1}
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Schedule Conflicts Warning Box */}
          {conflicts.length > 0 && (
            <div className="p-3.5 rounded-[var(--radius-md)] bg-amber-500/10 border border-amber-500/30 text-amber-600 dark:text-amber-400 space-y-2 text-xs">
              <div className="flex items-center gap-2 font-bold">
                <AlertTriangle className="h-4 w-4 shrink-0 text-amber-500" />
                <span>
                  {conflicts.length} Schedule Collision
                  {conflicts.length > 1 ? "s" : ""} Detected:
                </span>
              </div>
              <ul className="space-y-1 pl-6 list-disc">
                {conflicts.map((c, i) => (
                  <li key={i}>
                    <strong className="capitalize">{c.category} clash</strong>:{" "}
                    {c.resource_name} is already booked in &quot;
                    <span className="font-semibold">{c.conflicting_batch_name}</span>
                    &quot; on {c.day_of_week} ({c.existing_time_slot}).
                  </li>
                ))}
              </ul>
              <label className="flex items-center gap-2 pt-1 font-medium cursor-pointer">
                <input
                  type="checkbox"
                  checked={allowConflict}
                  onChange={(e) => setAllowConflict(e.target.checked)}
                  className="rounded border-amber-500 text-primary focus:ring-primary h-4 w-4"
                />
                <span>Allow schedule overlap (Admin authorized conflict override)</span>
              </label>
            </div>
          )}

          <DialogFooter className="pt-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading} className="gap-1.5">
              {loading ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Creating Batch...
                </>
              ) : (
                "Create Batch"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
