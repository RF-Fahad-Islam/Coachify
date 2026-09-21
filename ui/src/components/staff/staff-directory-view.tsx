import { useState, useEffect, useCallback } from "react";
import { rpc, type StaffSummary, type StaffSearchHit } from "@/gen/rpc";
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
import { AddStaffDialog } from "./add-staff-dialog";
import { StaffDetailDialog } from "./staff-detail-dialog";
import { useAuth } from "@/contexts/auth-context";
import { formatPhone } from "@/lib/utils";
import { Search, UserPlus, GraduationCap, Loader2, Eye } from "lucide-react";

export function StaffDirectoryView() {
  const { hasPermission } = useAuth();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [staffList, setStaffList] = useState<StaffSummary[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [departmentFilter, setDepartmentFilter] = useState("all");

  // Dialogs
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [selectedStaffId, setSelectedStaffId] = useState<string | null>(null);
  const [detailDialogOpen, setDetailDialogOpen] = useState(false);

  const fetchStaff = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const q = searchQuery.trim();
      if (q.length >= 2) {
        // Use FTS5 staff search
        const res = await rpc("staff.search", { query: q, limit: 50 });
        if (res.ok && res.data) {
          const hits = res.data;
          // Map SearchHits to StaffSummary shape
          const summaries: StaffSummary[] = hits.map((h: StaffSearchHit) => ({
            id: h.id,
            name: h.name,
            name_bn: h.name_bn,
            phone_norm: h.phone_norm,
            email: null,
            designation: h.designation,
            department: h.department,
            status: h.status,
            salary_type: "fixed_monthly",
            joining_date: "",
            active_batch_count: 0,
          }));
          setStaffList(summaries);
        }
      } else {
        const res = await rpc("staff.list", {
          status: statusFilter !== "all" ? statusFilter : null,
          department: departmentFilter !== "all" ? departmentFilter : null,
          limit: 100,
        });
        if (res.ok && res.data) {
          setStaffList(res.data);
        } else {
          setError(res.error?.message || "Failed to load staff list");
        }
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Network error");
    } finally {
      setLoading(false);
    }
  }, [searchQuery, statusFilter, departmentFilter]);

  useEffect(() => {
    const timer = setTimeout(() => {
      void fetchStaff();
    }, 200);
    return () => clearTimeout(timer);
  }, [fetchStaff]);

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

  const handleRowClick = (id: string) => {
    setSelectedStaffId(id);
    setDetailDialogOpen(true);
  };

  return (
    <div className="space-y-6">
      {/* Top Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Staff & Faculty Directory
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Manage teachers, instructors, administrative personnel, and class allocations.
          </p>
        </div>

        {hasPermission("staff.manage") && (
          <Button
            size="sm"
            onClick={() => setAddDialogOpen(true)}
            className="gap-1.5 text-xs"
          >
            <UserPlus className="h-3.5 w-3.5" />
            Add Staff Member
          </Button>
        )}
      </div>

      {error && (
        <div className="p-3 text-xs rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
          {error}
        </div>
      )}

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
        <div className="relative w-full sm:w-80">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search by name, phone, designation (FTS5)..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9 text-xs"
          />
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto">
          <select
            value={departmentFilter}
            onChange={(e) => setDepartmentFilter(e.target.value)}
            disabled={searchQuery.length >= 2}
            aria-label="Filter by department"
            className="h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
          >
            <option value="all">All Departments</option>
            <option value="Science (বিজ্ঞান)">Science (বিজ্ঞান)</option>
            <option value="Commerce / Business (ব্যবসায় শিক্ষা)">
              Commerce (ব্যবসায় শিক্ষা)
            </option>
            <option value="Humanities (মানবিক)">Humanities (মানবিক)</option>
            <option value="Mathematics (গণিত)">Mathematics (গণিত)</option>
            <option value="English (ইংরেজি)">English (ইংরেজি)</option>
            <option value="Bangla (বাংলা)">Bangla (বাংলা)</option>
            <option value="Administration & Accounts (প্রশাসন ও হিসাব)">
              Administration
            </option>
          </select>

          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            disabled={searchQuery.length >= 2}
            aria-label="Filter by status"
            className="h-9 px-3 rounded-[var(--radius)] border border-input bg-background text-foreground text-xs"
          >
            <option value="all">All Statuses</option>
            <option value="active">Active</option>
            <option value="on_leave">On Leave</option>
            <option value="resigned">Resigned</option>
            <option value="terminated">Terminated</option>
          </select>
        </div>
      </div>

      {/* Staff Directory Table */}
      <div className="rounded-[var(--radius-lg)] border border-border bg-card shadow-xs overflow-hidden">
        {loading ? (
          <div className="py-20 flex flex-col items-center justify-center gap-3">
            <Loader2 className="h-6 w-6 animate-spin text-primary" />
            <span className="text-xs text-muted-foreground">
              Loading faculty directory...
            </span>
          </div>
        ) : staffList.length === 0 ? (
          <div className="py-20 text-center space-y-2">
            <GraduationCap className="h-10 w-10 text-muted-foreground/40 mx-auto" />
            <div className="font-semibold text-foreground text-sm">No Staff Found</div>
            <p className="text-xs text-muted-foreground max-w-sm mx-auto">
              {searchQuery
                ? `No staff records matched "${searchQuery}".`
                : "No staff members have been added yet."}
            </p>
            {hasPermission("staff.manage") && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => setAddDialogOpen(true)}
                className="mt-2 text-xs"
              >
                Add First Staff
              </Button>
            )}
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/40 hover:bg-muted/40">
                <TableHead className="w-12"></TableHead>
                <TableHead className="font-bold">Faculty Name</TableHead>
                <TableHead className="font-bold">Designation</TableHead>
                <TableHead className="font-bold">Department</TableHead>
                <TableHead className="w-36 font-bold">Phone</TableHead>
                <TableHead className="w-24 text-center font-bold">Batches</TableHead>
                <TableHead className="w-24 text-center font-bold">Status</TableHead>
                <TableHead className="w-20 text-right font-bold">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {staffList.map((s) => (
                <TableRow
                  key={s.id}
                  className="group hover:bg-muted/40 transition-colors cursor-pointer"
                  onClick={() => handleRowClick(s.id)}
                >
                  <TableCell className="w-12 pr-0">
                    <div className="h-8 w-8 rounded-full bg-primary/10 text-primary font-bold flex items-center justify-center text-xs">
                      {s.name.slice(0, 2).toUpperCase()}
                    </div>
                  </TableCell>

                  <TableCell>
                    <div>
                      <span className="font-medium text-foreground group-hover:text-primary transition-colors">
                        {s.name}
                      </span>
                      {s.name_bn && (
                        <span className="block text-[11px] text-muted-foreground font-bengali">
                          {s.name_bn}
                        </span>
                      )}
                    </div>
                  </TableCell>

                  <TableCell className="text-xs text-foreground font-medium">
                    {s.designation}
                  </TableCell>

                  <TableCell className="text-xs text-muted-foreground">
                    {s.department || "General"}
                  </TableCell>

                  <TableCell className="font-mono text-xs text-muted-foreground">
                    {formatPhone(s.phone_norm)}
                  </TableCell>

                  <TableCell className="text-center font-mono text-xs font-semibold">
                    {s.active_batch_count > 0 ? (
                      <span className="px-2 py-0.5 rounded bg-primary/10 text-primary">
                        {s.active_batch_count}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">0</span>
                    )}
                  </TableCell>

                  <TableCell className="text-center">
                    {getStatusBadge(s.status)}
                  </TableCell>

                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRowClick(s.id);
                      }}
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-primary"
                    >
                      <Eye className="h-3.5 w-3.5" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>

      {/* Dialogs */}
      <AddStaffDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        onCreated={() => void fetchStaff()}
      />

      <StaffDetailDialog
        staffId={selectedStaffId}
        open={detailDialogOpen}
        onOpenChange={setDetailDialogOpen}
        onUpdated={() => void fetchStaff()}
      />
    </div>
  );
}
