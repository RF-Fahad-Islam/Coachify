import { useState, useEffect, useCallback } from "react";
import { rpc, type StudentSummary, type StudentSearchHit } from "@/gen/rpc";
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
import { useAuth } from "@/contexts/auth-context";
import { AddStudentDialog } from "./add-student-dialog";
import { StudentProfileDialog } from "./student-profile-dialog";
import { ImportStudentsDialog } from "./import-students-dialog";
import { formatPhone } from "@/lib/utils";
import {
  Search,
  UserPlus,
  Users,
  FileSpreadsheet,
  ChevronLeft,
  ChevronRight,
  Loader2,
  X,
  Copy,
  Check,
  Eye,
  CreditCard,
} from "lucide-react";
import { generateIdCardsHtml } from "../print/id-card-template";
import { DocumentPrintPreviewModal } from "../print/document-print-preview-modal";

type FilterStatus =
  "all" | "active" | "inactive" | "completed" | "transferred" | "archived";

export interface StudentDirectoryViewProps {
  initialStudentId?: string | null;
  onClearInitialStudentId?: () => void;
}

export function StudentDirectoryView({
  initialStudentId,
  onClearInitialStudentId,
}: StudentDirectoryViewProps = {}) {
  const { hasPermission } = useAuth();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedStatus, setSelectedStatus] = useState<FilterStatus>("all");
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(25);

  const [students, setStudents] = useState<StudentSummary[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Row selection & keyboard navigation
  const [selectedRowIndex, setSelectedRowIndex] = useState<number>(-1);

  // Dialogs
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [profileStudentId, setProfileStudentId] = useState<string | null>(null);
  const [profileDialogOpen, setProfileDialogOpen] = useState(false);
  const [idCardPreviewOpen, setIdCardPreviewOpen] = useState(false);
  const [idCardHtml, setIdCardHtml] = useState("");
  const [idCardsCount, setIdCardsCount] = useState(1);
  const [idCardPaperSize, setIdCardPaperSize] = useState<"a4" | "id_card_pvc">("a4");
  const [idCardsLoading, setIdCardsLoading] = useState(false);

  const handlePrintIdCards = async (
    studentIds: string[],
    mode: "a4_sheet" | "single_pvc" = "a4_sheet"
  ) => {
    if (studentIds.length === 0) return;
    setIdCardsLoading(true);
    try {
      const res = await rpc("students.get_id_card_data", { student_ids: studentIds });
      if (res.ok && res.data) {
        const html = generateIdCardsHtml(res.data, { layoutMode: mode });
        setIdCardHtml(html);
        setIdCardsCount(res.data.length);
        setIdCardPaperSize(mode === "single_pvc" ? "id_card_pvc" : "a4");
        setIdCardPreviewOpen(true);
      } else {
        alert(res.error?.message || "Failed to load ID card data");
      }
    } catch (err: unknown) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      setIdCardsLoading(false);
    }
  };

  useEffect(() => {
    if (initialStudentId) {
      setProfileStudentId(initialStudentId);
      setProfileDialogOpen(true);
      onClearInitialStudentId?.();
    }
  }, [initialStudentId, onClearInitialStudentId]);

  // Copied code feedback
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const fetchStudents = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const q = searchQuery.trim();
      if (q.length >= 2) {
        // Use FTS5 Substring Search
        const res = await rpc("students.search", { query: q, limit: 100 });
        if (res.ok && res.data) {
          // Map SearchHits to StudentSummary
          const hits: StudentSummary[] = (res.data.matches || []).map(
            (h: StudentSearchHit) => ({
              id: h.id,
              student_code: h.student_code,
              name: h.name,
              name_bn: h.name_bn,
              gender: h.gender,
              class_level: h.class_level,
              current_school_roll: h.current_school_roll,
              phone_norm: h.phone_norm,
              emergency_phone_norm: null,
              status: h.status,
              photo_vault_id: h.photo_vault_id,
              admission_date: "",
              row_version: 1,
              guardian_names: h.guardian_names,
              guardian_phones: h.guardian_phones,
            })
          );

          // Filter by status if set
          const filtered =
            selectedStatus === "all"
              ? hits
              : hits.filter(
                  (s) => s.status.toLowerCase() === selectedStatus.toLowerCase()
                );

          setStudents(filtered);
          setTotalCount(filtered.length);
        } else {
          setError(res.error?.message || "Search failed");
        }
      } else {
        // Regular paginated list
        const statusFilter = selectedStatus === "all" ? null : selectedStatus;
        const res = await rpc("students.list", {
          status: statusFilter,
          limit: pageSize,
          offset: page * pageSize,
        });

        if (res.ok && res.data) {
          setStudents(res.data.items || []);
          setTotalCount(res.data.total || 0);
        } else {
          setError(res.error?.message || "Failed to list students");
        }
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [searchQuery, selectedStatus, page, pageSize]);

  // Debounced search trigger
  useEffect(() => {
    const timer = setTimeout(() => {
      void fetchStudents();
    }, 150);
    return () => clearTimeout(timer);
  }, [fetchStudents]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Avoid capturing when typing inside inputs or dialogs
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement ||
        addDialogOpen ||
        profileDialogOpen
      ) {
        return;
      }

      if (e.key === "n" || e.key === "N") {
        if (hasPermission("students.manage")) {
          e.preventDefault();
          setAddDialogOpen(true);
        }
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedRowIndex((prev) => (prev < students.length - 1 ? prev + 1 : prev));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedRowIndex((prev) => (prev > 0 ? prev - 1 : 0));
      } else if (
        e.key === "Enter" &&
        selectedRowIndex >= 0 &&
        selectedRowIndex < students.length
      ) {
        e.preventDefault();
        const s = students[selectedRowIndex];
        if (s) {
          setProfileStudentId(s.id);
          setProfileDialogOpen(true);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [students, selectedRowIndex, addDialogOpen, profileDialogOpen, hasPermission]);

  const handleCopyCode = (code: string, e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(code);
    setCopiedCode(code);
    setTimeout(() => setCopiedCode(null), 1500);
  };

  const handleOpenProfile = (studentId: string) => {
    setProfileStudentId(studentId);
    setProfileDialogOpen(true);
  };

  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case "active":
        return <Badge variant="paid">Active</Badge>;
      case "inactive":
        return <Badge variant="secondary">Inactive</Badge>;
      case "archived":
        return <Badge variant="overdue">Archived</Badge>;
      case "completed":
        return <Badge variant="active">Completed</Badge>;
      case "transferred":
        return <Badge variant="pending">Transferred</Badge>;
      default:
        return <Badge variant="outline">{status}</Badge>;
    }
  };

  const totalPages = Math.ceil(totalCount / pageSize);

  return (
    <div className="space-y-4">
      {/* Top Header & Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground flex items-center gap-2.5">
            <Users className="h-6 w-6 text-primary" />
            Students Directory
          </h1>
          <p className="text-xs text-muted-foreground mt-0.5">
            Manage enrolled students, search records, and configure guardian contacts.
          </p>
        </div>

        {hasPermission("students.manage") && (
          <div className="flex items-center gap-2 self-start sm:self-auto">
            <Button
              onClick={() =>
                void handlePrintIdCards(
                  students.map((s) => s.id),
                  "a4_sheet"
                )
              }
              variant="outline"
              size="sm"
              className="gap-1.5 cursor-pointer"
              disabled={idCardsLoading || students.length === 0}
            >
              {idCardsLoading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <CreditCard className="h-4 w-4 text-primary" />
              )}
              Print ID Cards
            </Button>
            <Button
              onClick={() => setImportDialogOpen(true)}
              variant="outline"
              size="sm"
              className="gap-1.5 cursor-pointer"
            >
              <FileSpreadsheet className="h-4 w-4 text-primary" />
              Import from Excel
            </Button>
            <Button
              onClick={() => setAddDialogOpen(true)}
              size="sm"
              className="gap-1.5 cursor-pointer"
            >
              <UserPlus className="h-4 w-4" />
              Admit Student
              <span className="ml-1 px-1.5 py-0.2 rounded text-[10px] bg-primary-foreground/20 font-mono">
                N
              </span>
            </Button>
          </div>
        )}
      </div>

      {/* Filter Tabs & Search Toolbar */}
      <div className="flex flex-col md:flex-row items-stretch md:items-center justify-between gap-3 bg-card p-3 rounded-[var(--radius-lg)] border border-border shadow-xs">
        {/* Search Bar */}
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-9 pr-8 h-9 text-xs"
            placeholder="Search by student code, name, phone, or guardian..."
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              setPage(0);
            }}
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-2.5 top-2.5 text-muted-foreground hover:text-foreground cursor-pointer"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>

        {/* Status Filter Chips */}
        <div className="flex items-center gap-1 overflow-x-auto pb-1 md:pb-0">
          {(
            [
              { id: "all", label: "All" },
              { id: "active", label: "Active" },
              { id: "inactive", label: "Inactive" },
              { id: "completed", label: "Completed" },
              { id: "transferred", label: "Transferred" },
              { id: "archived", label: "Archived" },
            ] as const
          ).map((tab) => (
            <button
              key={tab.id}
              onClick={() => {
                setSelectedStatus(tab.id);
                setPage(0);
              }}
              className={`px-3 py-1.5 text-xs font-semibold rounded-[var(--radius-md)] transition-colors select-none cursor-pointer shrink-0 ${
                selectedStatus === tab.id
                  ? "bg-primary text-primary-foreground shadow-xs"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {error && (
        <div className="p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
          {error}
        </div>
      )}

      {/* Directory Table */}
      <div className="rounded-[var(--radius-lg)] border border-border bg-card overflow-hidden shadow-xs">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[120px]">Code</TableHead>
              <TableHead>Student Name</TableHead>
              <TableHead>Class / Roll</TableHead>
              <TableHead>Phone Number</TableHead>
              <TableHead>Primary Guardian</TableHead>
              <TableHead className="w-[100px]">Status</TableHead>
              <TableHead className="w-[110px] text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading ? (
              <TableRow>
                <TableCell colSpan={7} className="h-36 text-center">
                  <div className="flex flex-col items-center justify-center gap-2">
                    <Loader2 className="h-6 w-6 animate-spin text-primary" />
                    <span className="text-xs text-muted-foreground">
                      Loading directory...
                    </span>
                  </div>
                </TableCell>
              </TableRow>
            ) : students.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="h-36 text-center">
                  <div className="flex flex-col items-center justify-center gap-2 text-muted-foreground">
                    <Users className="h-8 w-8 text-muted-foreground/40" />
                    <span className="text-sm font-medium">
                      No students match your filter
                    </span>
                    <span className="text-xs text-muted-foreground/80">
                      Try searching with different keywords or clearing the status filter.
                    </span>
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              students.map((student, idx) => {
                const isSelected = selectedRowIndex === idx;
                return (
                  <TableRow
                    key={student.id}
                    onClick={() => handleOpenProfile(student.id)}
                    className={`cursor-pointer transition-colors ${
                      isSelected
                        ? "bg-muted/80 ring-1 ring-primary/40"
                        : "hover:bg-muted/40"
                    }`}
                  >
                    {/* Student Code */}
                    <TableCell className="font-mono text-xs font-semibold text-primary">
                      <div className="flex items-center gap-1.5">
                        <span>{student.student_code}</span>
                        <button
                          type="button"
                          onClick={(e) => handleCopyCode(student.student_code, e)}
                          title="Copy Student Code"
                          className="opacity-0 group-hover:opacity-100 hover:text-foreground text-muted-foreground transition-opacity"
                        >
                          {copiedCode === student.student_code ? (
                            <Check className="h-3 w-3 text-emerald-500" />
                          ) : (
                            <Copy className="h-3 w-3" />
                          )}
                        </button>
                      </div>
                    </TableCell>

                    {/* Name */}
                    <TableCell>
                      <div>
                        <span className="font-bold text-sm text-foreground block">
                          {student.name}
                        </span>
                        {student.name_bn && (
                          <span className="text-xs text-muted-foreground font-bengali block">
                            {student.name_bn}
                          </span>
                        )}
                      </div>
                    </TableCell>

                    {/* Class / Roll */}
                    <TableCell className="text-xs">
                      {student.class_level ? (
                        <div>
                          <span className="font-medium text-foreground">
                            {student.class_level}
                          </span>
                          {student.current_school_roll && (
                            <span className="text-muted-foreground ml-1">
                              (Roll {student.current_school_roll})
                            </span>
                          )}
                        </div>
                      ) : (
                        <span className="text-muted-foreground italic">Unassigned</span>
                      )}
                    </TableCell>

                    {/* Student Phone */}
                    <TableCell className="font-mono text-xs text-foreground">
                      {formatPhone(student.phone_norm)}
                    </TableCell>

                    {/* Guardian Info */}
                    <TableCell className="text-xs">
                      {student.guardian_names ? (
                        <div>
                          <span className="font-medium text-foreground block">
                            {student.guardian_names}
                          </span>
                          {student.guardian_phones && (
                            <span className="font-mono text-muted-foreground text-[11px]">
                              {formatPhone(student.guardian_phones.split(" ")[0] || "")}
                            </span>
                          )}
                        </div>
                      ) : (
                        <span className="text-muted-foreground italic">
                          None recorded
                        </span>
                      )}
                    </TableCell>

                    {/* Status */}
                    <TableCell>{getStatusBadge(student.status)}</TableCell>

                    {/* Actions */}
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-8 px-2 text-xs gap-1 text-muted-foreground hover:text-foreground"
                          title="Print ID Card (Single PVC)"
                          onClick={(e) => {
                            e.stopPropagation();
                            void handlePrintIdCards([student.id], "single_pvc");
                          }}
                        >
                          <CreditCard className="h-3.5 w-3.5" />
                          <span className="sr-only sm:not-sr-only sm:inline">
                            ID Card
                          </span>
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-8 px-2 text-xs gap-1 text-primary hover:text-primary hover:bg-primary/10"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleOpenProfile(student.id);
                          }}
                        >
                          <Eye className="h-3.5 w-3.5" />
                          Profile
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>

        {/* Pagination Footer */}
        <div className="flex items-center justify-between p-3 border-t border-border text-xs text-muted-foreground bg-muted/20">
          <div>
            Showing{" "}
            <span className="font-semibold text-foreground">
              {totalCount === 0 ? 0 : page * pageSize + 1}
            </span>{" "}
            to{" "}
            <span className="font-semibold text-foreground">
              {Math.min((page + 1) * pageSize, totalCount)}
            </span>{" "}
            of <span className="font-semibold text-foreground">{totalCount}</span>{" "}
            students
          </div>

          <div className="flex items-center gap-2">
            <select
              className="h-8 rounded-[var(--radius-md)] border border-input bg-transparent px-2 text-xs"
              value={pageSize}
              onChange={(e) => {
                setPageSize(Number(e.target.value));
                setPage(0);
              }}
            >
              <option value={10}>10 / page</option>
              <option value={25}>25 / page</option>
              <option value={50}>50 / page</option>
              <option value={100}>100 / page</option>
            </select>

            <div className="flex items-center gap-1">
              <Button
                variant="outline"
                size="sm"
                className="h-8 w-8 p-0"
                onClick={() => setPage((p) => Math.max(0, p - 1))}
                disabled={page === 0 || loading}
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
              <span className="px-2 text-xs font-medium">
                {totalPages === 0 ? 1 : page + 1} / {Math.max(1, totalPages)}
              </span>
              <Button
                variant="outline"
                size="sm"
                className="h-8 w-8 p-0"
                onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
                disabled={page >= totalPages - 1 || loading}
              >
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Keyboard Shortcuts Hint Bar */}
      <div className="flex flex-col sm:flex-row items-center justify-between px-3 py-2 bg-muted/40 rounded-[var(--radius-md)] text-[11px] text-muted-foreground border border-border/50 gap-2">
        <div className="flex flex-wrap items-center gap-4">
          <span>
            <kbd className="font-mono bg-muted px-1.5 py-0.5 rounded border border-border text-[10px]">
              ↑
            </kbd>{" "}
            <kbd className="font-mono bg-muted px-1.5 py-0.5 rounded border border-border text-[10px]">
              ↓
            </kbd>{" "}
            to navigate rows
          </span>
          <span>
            <kbd className="font-mono bg-muted px-1.5 py-0.5 rounded border border-border text-[10px]">
              Enter
            </kbd>{" "}
            to inspect student
          </span>
          <span>
            <kbd className="font-mono bg-muted px-1.5 py-0.5 rounded border border-border text-[10px]">
              N
            </kbd>{" "}
            to add student
          </span>
        </div>
        <span className="font-bengali text-muted-foreground/70 hidden md:inline">
          কীবোর্ড শর্টকাট ব্যবহার করে দ্রুত কাজ করুন
        </span>
      </div>

      {/* Add Student Dialog */}
      <AddStudentDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        onSuccess={() => {
          void fetchStudents();
        }}
      />

      {/* Import Students from Excel/CSV Dialog */}
      <ImportStudentsDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        onImportSuccess={() => {
          void fetchStudents();
        }}
      />

      {/* Student Profile Dialog */}
      <StudentProfileDialog
        studentId={profileStudentId}
        open={profileDialogOpen}
        onOpenChange={setProfileDialogOpen}
        onStudentUpdated={fetchStudents}
        onStudentArchived={fetchStudents}
        onSelectStudent={(newId) => {
          setProfileStudentId(newId);
        }}
      />

      {/* ID Card Document Print Preview Modal */}
      <DocumentPrintPreviewModal
        open={idCardPreviewOpen}
        onOpenChange={setIdCardPreviewOpen}
        title="Student ID Cards Print Preview"
        documentType="id_card"
        htmlContent={idCardHtml}
        initialPaperSize={idCardPaperSize}
        totalItems={idCardsCount}
      />
    </div>
  );
}
