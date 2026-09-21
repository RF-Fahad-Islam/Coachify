import { useState, useEffect, useCallback } from "react";
import { rpc, type StaffDetail } from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { useAuth } from "@/contexts/auth-context";
import { formatPhone, formatBusinessDate } from "@/lib/utils";
import {
  Phone,
  Mail,
  GraduationCap,
  Calendar,
  Layers,
  MapPin,
  Loader2,
  Edit2,
  Check,
} from "lucide-react";

export interface StaffDetailDialogProps {
  staffId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onUpdated: () => void;
}

export function StaffDetailDialog({
  staffId,
  open,
  onOpenChange,
  onUpdated,
}: StaffDetailDialogProps) {
  const { hasPermission } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [staff, setStaff] = useState<StaffDetail | null>(null);

  // Edit status / designation
  const [isEditing, setIsEditing] = useState(false);
  const [editDesignation, setEditDesignation] = useState("");
  const [editStatus, setEditStatus] = useState("");
  const [editPhone, setEditPhone] = useState("");
  const [editEmail, setEditEmail] = useState("");
  const [saving, setSaving] = useState(false);

  const loadStaff = useCallback(async () => {
    if (!staffId) return;
    setLoading(true);
    setError(null);

    try {
      const res = await rpc("staff.get", { id: staffId });
      if (res.ok && res.data) {
        setStaff(res.data);
        setEditDesignation(res.data.designation);
        setEditStatus(res.data.status);
        setEditPhone(res.data.phone_norm);
        setEditEmail(res.data.email || "");
      } else {
        setError(res.error?.message || "Failed to load staff details");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  }, [staffId]);

  useEffect(() => {
    if (open) {
      setIsEditing(false);
      void loadStaff();
    }
  }, [open, loadStaff]);

  const handleSaveUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!staff) return;

    setSaving(true);
    setError(null);

    try {
      const res = await rpc("staff.update", {
        id: staff.id,
        staff: {
          name: staff.name,
          name_bn: staff.name_bn,
          phone: editPhone,
          email: editEmail.trim() || null,
          designation: editDesignation,
          department: staff.department,
          educational_qualifications: staff.educational_qualifications,
          nid: staff.nid,
          blood_group: staff.blood_group,
          gender: staff.gender,
          dob: staff.dob,
          present_address: staff.present_address,
          joining_date: staff.joining_date,
          resignation_date: staff.resignation_date,
          salary_type: staff.salary_type,
          status: editStatus,
          notes: staff.notes,
        },
      });

      if (res.ok && res.data) {
        setIsEditing(false);
        setStaff(res.data);
        onUpdated();
      } else {
        setError(res.error?.message || "Failed to update staff record");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setSaving(false);
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "active":
        return <Badge variant="default">Active</Badge>;
      case "on_leave":
        return <Badge variant="secondary">On Leave</Badge>;
      case "resigned":
        return <Badge variant="outline">Resigned</Badge>;
      case "terminated":
        return <Badge variant="overdue">Terminated</Badge>;
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg font-bold text-foreground">
            <GraduationCap className="h-5 w-5 text-primary" />
            Staff & Teacher Profile
          </DialogTitle>
        </DialogHeader>

        {loading ? (
          <div className="py-16 flex flex-col items-center justify-center gap-3">
            <Loader2 className="h-6 w-6 animate-spin text-primary" />
            <span className="text-xs text-muted-foreground">Loading staff record...</span>
          </div>
        ) : staff ? (
          <div className="space-y-6 pt-2">
            {error && (
              <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
                {error}
              </div>
            )}

            {/* Profile Banner */}
            <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card shadow-xs flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
              <div className="flex items-center gap-3.5">
                <div className="h-12 w-12 rounded-full bg-primary/15 text-primary font-bold flex items-center justify-center text-lg">
                  {staff.name.slice(0, 2).toUpperCase()}
                </div>
                <div>
                  <div className="flex items-center gap-2 flex-wrap">
                    <h2 className="text-lg font-bold text-foreground">{staff.name}</h2>
                    {staff.name_bn && (
                      <span className="text-sm text-muted-foreground font-bengali">
                        ({staff.name_bn})
                      </span>
                    )}
                    {getStatusBadge(staff.status)}
                  </div>
                  <div className="text-xs text-muted-foreground mt-0.5 flex items-center gap-2">
                    <span className="font-medium text-foreground">
                      {staff.designation}
                    </span>
                    {staff.department && (
                      <>
                        <span>•</span>
                        <span>{staff.department}</span>
                      </>
                    )}
                  </div>
                </div>
              </div>

              {hasPermission("staff.manage") && !isEditing && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setIsEditing(true)}
                  className="gap-1.5 text-xs self-end sm:self-center"
                >
                  <Edit2 className="h-3.5 w-3.5" />
                  Edit Status
                </Button>
              )}
            </div>

            {/* Edit Mode vs Read-only Info */}
            {isEditing ? (
              <form
                onSubmit={handleSaveUpdate}
                className="p-4 rounded-[var(--radius-lg)] border border-primary/30 bg-primary-subtle/30 space-y-4 text-xs"
              >
                <div className="font-semibold text-foreground text-sm flex items-center gap-1.5">
                  <Edit2 className="h-4 w-4 text-primary" />
                  Update Staff Information
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <label className="font-medium text-foreground">Designation</label>
                    <Input
                      value={editDesignation}
                      onChange={(e) => setEditDesignation(e.target.value)}
                      required
                    />
                  </div>

                  <div className="space-y-1">
                    <label className="font-medium text-foreground">
                      Employment Status
                    </label>
                    <select
                      value={editStatus}
                      onChange={(e) => setEditStatus(e.target.value)}
                      aria-label="Employment Status"
                      className="w-full h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
                    >
                      <option value="active">Active (সক্রিয়)</option>
                      <option value="on_leave">On Leave (ছুটিতে)</option>
                      <option value="resigned">Resigned (পদত্যাগ)</option>
                      <option value="terminated">Terminated (বরখাস্ত)</option>
                    </select>
                  </div>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div className="space-y-1">
                    <label className="font-medium text-foreground">Phone</label>
                    <Input
                      value={editPhone}
                      onChange={(e) => setEditPhone(e.target.value)}
                      required
                    />
                  </div>
                  <div className="space-y-1">
                    <label className="font-medium text-foreground">Email</label>
                    <Input
                      type="email"
                      value={editEmail}
                      onChange={(e) => setEditEmail(e.target.value)}
                    />
                  </div>
                </div>

                <div className="flex justify-end gap-2 pt-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setIsEditing(false)}
                    disabled={saving}
                  >
                    Cancel
                  </Button>
                  <Button type="submit" size="sm" disabled={saving} className="gap-1.5">
                    {saving ? (
                      <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <Check className="h-3.5 w-3.5" />
                    )}
                    Save Changes
                  </Button>
                </div>
              </form>
            ) : (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
                <div className="p-3.5 rounded-[var(--radius-md)] border border-border bg-card space-y-2.5">
                  <div className="font-semibold text-muted-foreground uppercase text-[11px] tracking-wider">
                    Contact & Personal
                  </div>
                  <div className="flex items-center gap-2">
                    <Phone className="h-4 w-4 text-primary shrink-0" />
                    <span className="font-mono text-foreground font-medium">
                      {formatPhone(staff.phone_norm)}
                    </span>
                  </div>
                  {staff.email && (
                    <div className="flex items-center gap-2">
                      <Mail className="h-4 w-4 text-muted-foreground shrink-0" />
                      <span className="text-foreground">{staff.email}</span>
                    </div>
                  )}
                  {staff.present_address && (
                    <div className="flex items-center gap-2">
                      <MapPin className="h-4 w-4 text-muted-foreground shrink-0" />
                      <span className="text-foreground">{staff.present_address}</span>
                    </div>
                  )}
                </div>

                <div className="p-3.5 rounded-[var(--radius-md)] border border-border bg-card space-y-2.5">
                  <div className="font-semibold text-muted-foreground uppercase text-[11px] tracking-wider">
                    Employment Details
                  </div>
                  <div className="flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-primary shrink-0" />
                    <span>
                      Joined on{" "}
                      <strong className="text-foreground">
                        {formatBusinessDate(staff.joining_date)}
                      </strong>
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-muted-foreground">Salary Structure:</span>
                    <Badge variant="outline" className="capitalize text-[10px]">
                      {staff.salary_type.replace(/_/g, " ")}
                    </Badge>
                  </div>
                  {staff.notes && (
                    <div className="pt-1 text-[11px] text-muted-foreground italic">
                      &quot;{staff.notes}&quot;
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Assigned Batches Section */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                  <Layers className="h-4 w-4 text-primary" />
                  Assigned Teaching Batches ({staff.assigned_batches.length})
                </h3>
              </div>

              {staff.assigned_batches.length === 0 ? (
                <div className="p-6 border border-dashed rounded-[var(--radius-md)] text-center text-xs text-muted-foreground">
                  No batches currently assigned to this teacher.
                </div>
              ) : (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {staff.assigned_batches.map((b) => (
                    <div
                      key={b.id}
                      className="p-3 rounded-[var(--radius-md)] border border-border bg-card space-y-2 text-xs"
                    >
                      <div className="flex items-start justify-between">
                        <div>
                          <span className="font-semibold text-foreground block">
                            {b.name}
                          </span>
                          <span className="text-[11px] text-muted-foreground">
                            {b.program_name}
                          </span>
                        </div>
                        <Badge variant="outline" className="text-[10px] capitalize">
                          {b.shift}
                        </Badge>
                      </div>

                      <div className="flex items-center justify-between text-[11px] text-muted-foreground pt-1 border-t border-border/50">
                        <span className="flex items-center gap-1">
                          <MapPin className="h-3 w-3" />
                          {b.room_number || "Room TBD"}
                        </span>
                        <span className="font-mono">
                          {b.enrolled_count} / {b.capacity} Students
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <DialogFooter className="pt-2 border-t border-border">
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Close
              </Button>
            </DialogFooter>
          </div>
        ) : (
          <div className="py-16 text-center text-muted-foreground text-xs">
            Staff record not found.
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
