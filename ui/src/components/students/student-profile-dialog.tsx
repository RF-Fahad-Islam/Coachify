import { useEffect, useState, useRef, useCallback } from "react";
import {
  rpc,
  type GetStudentResponse,
  type StudentEnrollmentHistoryItem,
  type CashierStudentOverview,
  type PaymentReceipt,
} from "@/gen/rpc";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { EditSharedGuardianDialog } from "./edit-shared-guardian-dialog";
import { VoidPaymentDialog } from "../cashier/void-payment-dialog";
import { RefundDialog } from "../cashier/refund-dialog";
import { AdjustmentDialog } from "../cashier/adjustment-dialog";
import { useAuth } from "@/contexts/auth-context";
import { formatPhone, formatBusinessDate, formatTaka } from "@/lib/utils";
import { executeHtmlPrint, getPrintableReceipt, getPrinterSettings } from "@/lib/printer";
import { generateDueStatementHtml } from "../print/due-statement-template";
import { generateThermalReceiptHtml } from "../print/thermal-receipt";
import { generateA4DualReceiptHtml } from "../print/a4-dual-receipt";
import { generateA5ReceiptHtml } from "../print/a5-receipt";
import {
  User,
  Phone,
  MapPin,
  FileText,
  Users,
  Archive,
  Loader2,
  Upload,
  Download,
  AlertTriangle,
  Edit2,
  School,
  IdCard,
  Layers,
  CreditCard,
  Printer,
  Sliders,
  RotateCcw,
  ArrowDownLeft,
  CheckCircle2,
  History as HistoryIcon,
} from "lucide-react";

export interface StudentProfileDialogProps {
  studentId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onStudentUpdated?: () => void;
  onStudentArchived?: () => void;
  onSelectStudent?: (id: string) => void;
}

export function StudentProfileDialog({
  studentId,
  open,
  onOpenChange,
  onStudentUpdated,
  onStudentArchived,
  onSelectStudent,
}: StudentProfileDialogProps) {
  const { hasPermission } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [studentData, setStudentData] = useState<GetStudentResponse | null>(null);
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [enrollments, setEnrollments] = useState<StudentEnrollmentHistoryItem[]>([]);

  // Active tab
  const [currentTab, setCurrentTab] = useState("overview");

  // Edit guardian modal
  const [editingGuardianId, setEditingGuardianId] = useState<string | null>(null);

  // Archive modal
  const [archiveModalOpen, setArchiveModalOpen] = useState(false);
  const [archiveReason, setArchiveReason] = useState("");
  const [archiving, setArchiving] = useState(false);

  // Upload document
  const [uploadingDoc, setUploadingDoc] = useState(false);
  const docInputRef = useRef<HTMLInputElement>(null);

  // Finance overview and dialog states
  const [financeOverview, setFinanceOverview] = useState<CashierStudentOverview | null>(
    null
  );
  const [financeLoading, setFinanceLoading] = useState(false);
  const [voidPaymentTarget, setVoidPaymentTarget] = useState<PaymentReceipt | null>(null);
  const [voidDialogOpen, setVoidDialogOpen] = useState(false);
  const [refundTarget, setRefundTarget] = useState<PaymentReceipt | null>(null);
  const [refundDialogOpen, setRefundDialogOpen] = useState(false);
  const [adjustmentDialogOpen, setAdjustmentDialogOpen] = useState(false);
  const [statementPrinting, setStatementPrinting] = useState(false);
  const [financeNotice, setFinanceNotice] = useState<string | null>(null);

  const loadFinanceOverview = useCallback(async () => {
    if (!studentId) return;
    setFinanceLoading(true);
    try {
      const res = await rpc("ledger.get_cashier_student_overview", {
        student_id: studentId,
      });
      if (res.ok && res.data) {
        setFinanceOverview(res.data);
      }
    } catch (e) {
      console.error("Failed to load finance overview", e);
    } finally {
      setFinanceLoading(false);
    }
  }, [studentId]);

  useEffect(() => {
    if (open && studentId && currentTab === "finance") {
      void loadFinanceOverview();
    }
  }, [open, studentId, currentTab, loadFinanceOverview]);

  const handlePrintStatement = async () => {
    if (!studentId) return;
    setStatementPrinting(true);
    setFinanceNotice(null);
    try {
      const res = await rpc("ledger.get_due_statement", {
        student_id: studentId,
      });
      if (res.ok && res.data) {
        const html = generateDueStatementHtml(res.data);
        const printRes = await executeHtmlPrint(html);
        if (printRes.success) {
          setFinanceNotice("Account statement sent to printer.");
        } else {
          setFinanceNotice(`Printer warning: ${printRes.error || "Device offline"}`);
        }
      } else {
        setFinanceNotice(res.error?.message || "Failed to load statement");
      }
    } catch (e) {
      console.error("Failed to print statement:", e);
      setFinanceNotice("Failed to generate statement for printing.");
    } finally {
      setStatementPrinting(false);
    }
  };

  const handleReprintReceipt = async (receiptId: string) => {
    try {
      const printable = await getPrintableReceipt(receiptId, true);
      const settings = await getPrinterSettings();
      let html = "";
      if (settings?.page_size === "thermal_80mm") {
        html = generateThermalReceiptHtml(printable, settings?.footer_note);
      } else if (settings?.page_size === "a4_dual") {
        html = generateA4DualReceiptHtml(printable, settings?.footer_note);
      } else {
        html = generateA5ReceiptHtml(printable, settings?.footer_note);
      }
      const printRes = await executeHtmlPrint(html);
      if (printRes.success) {
        setFinanceNotice("Receipt reprint sent to printer.");
      }
    } catch (e) {
      console.error("Failed to reprint receipt:", e);
    }
  };

  const loadStudent = useCallback(async () => {
    if (!studentId) return;
    setLoading(true);
    setError(null);
    setPhotoUrl(null);

    try {
      const res = await rpc("students.get", { id: studentId });
      if (res.ok && res.data) {
        setStudentData(res.data);

        // Load photo if present
        if (res.data.student.photo_vault_id) {
          try {
            const photoRes = await rpc("vault.get_file", {
              id: res.data.student.photo_vault_id,
            });
            if (photoRes.ok && photoRes.data) {
              setPhotoUrl(
                `data:${photoRes.data.mime_type};base64,${photoRes.data.data_base64}`
              );
            }
          } catch (e) {
            console.error("Failed to load student photo", e);
          }
        }

        // Load enrollments
        try {
          const enrRes = await rpc("enrollments.list_by_student", {
            student_id: studentId,
          });
          if (enrRes.ok && enrRes.data) {
            setEnrollments(enrRes.data);
          }
        } catch (e) {
          console.error("Failed to load student enrollments", e);
        }
      } else {
        setError(res.error?.message || "Failed to load student profile");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [studentId]);

  useEffect(() => {
    if (open && studentId) {
      void loadStudent();
    }
  }, [open, studentId, loadStudent]);

  const handleArchive = async () => {
    if (!studentData) return;
    if (!archiveReason.trim()) {
      setError("Please provide a reason for archiving");
      return;
    }

    setArchiving(true);
    try {
      const res = await rpc("students.archive", {
        id: studentData.student.id,
        reason: archiveReason.trim(),
        expected_version: studentData.student.row_version,
      });

      if (res.ok) {
        setArchiveModalOpen(false);
        setArchiveReason("");
        onStudentArchived?.();
        void loadStudent();
      } else {
        setError(res.error?.message || "Failed to archive student");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setArchiving(false);
    }
  };

  const handleDocumentUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !studentData) return;

    if (file.size > 20 * 1024 * 1024) {
      setError("Document size exceeds maximum limit of 20MB");
      return;
    }

    const reader = new FileReader();
    reader.onload = async () => {
      const b64 = reader.result as string;
      setUploadingDoc(true);
      setError(null);

      try {
        const res = await rpc("vault.upload_file", {
          owner_type: "student",
          owner_id: studentData.student.id,
          kind: "document",
          file_name: file.name,
          mime_type: file.type || "application/pdf",
          data_base64: b64,
        });

        if (res.ok) {
          void loadStudent();
        } else {
          setError(res.error?.message || "Failed to upload document");
        }
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setUploadingDoc(false);
      }
    };
    reader.readAsDataURL(file);
  };

  const downloadDocument = async (docId: string, originalName: string, mime: string) => {
    try {
      const res = await rpc("vault.get_file", { id: docId });
      if (res.ok && res.data) {
        const link = document.createElement("a");
        link.href = `data:${mime};base64,${res.data.data_base64}`;
        link.download = originalName;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
      }
    } catch (e) {
      console.error("Failed to download document", e);
    }
  };

  const student = studentData?.student;
  const guardians = studentData?.student.guardians || [];
  const siblings = studentData?.siblings || [];
  const documents = studentData?.documents || [];

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

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-4xl max-h-[92vh] overflow-y-auto p-0 gap-0">
          {loading ? (
            <div className="py-20 flex flex-col items-center justify-center gap-3">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
              <span className="text-sm text-muted-foreground">
                Loading student profile...
              </span>
            </div>
          ) : student ? (
            <div>
              {/* Profile Header */}
              <div className="p-6 bg-muted/30 border-b border-border">
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                  <div className="flex items-center gap-4">
                    {/* Photo or Initials */}
                    <div className="h-20 w-20 rounded-[var(--radius-lg)] border-2 border-border bg-card overflow-hidden flex items-center justify-center shrink-0 shadow-xs">
                      {photoUrl ? (
                        <img
                          src={photoUrl}
                          alt={student.name}
                          className="h-full w-full object-cover"
                        />
                      ) : (
                        <div className="text-xl font-bold text-muted-foreground uppercase">
                          {student.name.slice(0, 2)}
                        </div>
                      )}
                    </div>

                    <div>
                      <div className="flex items-center gap-2.5 flex-wrap">
                        <h2 className="text-xl font-bold text-foreground tracking-tight">
                          {student.name}
                        </h2>
                        {student.name_bn && (
                          <span className="text-sm text-muted-foreground font-bengali">
                            ({student.name_bn})
                          </span>
                        )}
                        {getStatusBadge(student.status)}
                      </div>

                      <div className="flex items-center gap-3 text-xs text-muted-foreground mt-1.5 flex-wrap">
                        <span className="font-mono px-2 py-0.5 rounded bg-primary/10 text-primary font-semibold">
                          {student.student_code}
                        </span>
                        <span>•</span>
                        <span className="capitalize">{student.gender}</span>
                        {student.class_level && (
                          <>
                            <span>•</span>
                            <span className="font-medium text-foreground">
                              {student.class_level}
                              {student.current_school_roll
                                ? ` (Roll ${student.current_school_roll})`
                                : ""}
                            </span>
                          </>
                        )}
                        <span>•</span>
                        <span>Enrolled {formatBusinessDate(student.admission_date)}</span>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 self-end sm:self-center">
                    {student.status === "active" && hasPermission("students.archive") && (
                      <Button
                        variant="outline"
                        size="sm"
                        className="text-destructive hover:bg-destructive/10 hover:text-destructive border-destructive/30 gap-1.5"
                        onClick={() => setArchiveModalOpen(true)}
                      >
                        <Archive className="h-4 w-4" />
                        Archive
                      </Button>
                    )}
                  </div>
                </div>
              </div>

              {error && (
                <div className="m-6 p-3 text-sm rounded-[var(--radius-md)] bg-destructive/15 text-destructive border border-destructive/20">
                  {error}
                </div>
              )}

              {/* Tabs Content */}
              <div className="p-6">
                <Tabs value={currentTab} onValueChange={setCurrentTab}>
                  <TabsList className="grid grid-cols-5 w-full mb-6">
                    <TabsTrigger value="overview" className="gap-2">
                      <User className="h-4 w-4" />
                      Overview
                    </TabsTrigger>
                    <TabsTrigger value="guardians" className="gap-2">
                      <Users className="h-4 w-4" />
                      Guardians & Siblings ({guardians.length})
                    </TabsTrigger>
                    <TabsTrigger value="documents" className="gap-2">
                      <FileText className="h-4 w-4" />
                      Documents ({documents.length})
                    </TabsTrigger>
                    <TabsTrigger value="academic" className="gap-2">
                      <School className="h-4 w-4" />
                      Academic & History
                    </TabsTrigger>
                    <TabsTrigger value="finance" className="gap-2">
                      <CreditCard className="h-4 w-4" />
                      Finance & Ledger
                    </TabsTrigger>
                  </TabsList>

                  {/* 1. Overview Tab */}
                  <TabsContent value="overview" className="space-y-6">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      {/* Contact Info Card */}
                      <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs">
                        <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                          <Phone className="h-3.5 w-3.5" />
                          Contact Information
                        </h3>
                        <div className="space-y-2 text-sm divide-y divide-border/50">
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              Primary Phone
                            </span>
                            <span className="font-mono font-medium text-foreground">
                              {formatPhone(student.phone_norm)}
                            </span>
                          </div>
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              Emergency Phone
                            </span>
                            <span className="font-mono text-foreground">
                              {student.emergency_phone_norm
                                ? formatPhone(student.emergency_phone_norm)
                                : "None recorded"}
                            </span>
                          </div>
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">Email</span>
                            <span className="text-foreground">
                              {student.email || "None recorded"}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Personal Details Card */}
                      <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs">
                        <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                          <IdCard className="h-3.5 w-3.5" />
                          Personal Identification
                        </h3>
                        <div className="space-y-2 text-sm divide-y divide-border/50">
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              Date of Birth
                            </span>
                            <span className="font-medium text-foreground">
                              {student.dob}
                            </span>
                          </div>
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              Blood Group
                            </span>
                            <span className="font-bold text-destructive">
                              {student.blood_group || "Unknown"}
                            </span>
                          </div>
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              Religion
                            </span>
                            <span className="text-foreground">
                              {student.religion || "None recorded"}
                            </span>
                          </div>
                          <div className="pt-2 flex justify-between">
                            <span className="text-muted-foreground text-xs">
                              NID / Birth Reg.
                            </span>
                            <span className="font-mono text-foreground">
                              {student.nid_or_birth_cert || "None recorded"}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Address Details Card */}
                      <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs md:col-span-2">
                        <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                          <MapPin className="h-3.5 w-3.5" />
                          Address & Location
                        </h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm pt-1">
                          <div>
                            <span className="text-xs font-semibold text-muted-foreground block">
                              Present Address
                            </span>
                            <p className="mt-0.5 text-foreground leading-relaxed">
                              {student.present_address}
                            </p>
                            {(student.thana || student.district) && (
                              <p className="text-xs text-muted-foreground mt-0.5">
                                {[student.thana, student.district]
                                  .filter(Boolean)
                                  .join(", ")}
                              </p>
                            )}
                          </div>
                          <div>
                            <span className="text-xs font-semibold text-muted-foreground block">
                              Permanent Address
                            </span>
                            <p className="mt-0.5 text-foreground leading-relaxed">
                              {student.permanent_address || "Same as present address"}
                            </p>
                          </div>
                        </div>
                      </div>

                      {/* Internal Remarks */}
                      {student.notes && (
                        <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-2 md:col-span-2">
                          <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                            Profile Notes & History
                          </h3>
                          <pre className="text-xs text-muted-foreground whitespace-pre-wrap font-sans bg-muted/40 p-3 rounded-[var(--radius-md)] border border-border">
                            {student.notes}
                          </pre>
                        </div>
                      )}
                    </div>
                  </TabsContent>

                  {/* 2. Guardians & Siblings Tab */}
                  <TabsContent value="guardians" className="space-y-6">
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">
                          Authorized Guardians ({guardians.length})
                        </h3>
                      </div>

                      <div className="grid grid-cols-1 gap-4">
                        {guardians.map((g) => (
                          <div
                            key={g.id}
                            className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs"
                          >
                            <div className="flex items-start justify-between">
                              <div>
                                <div className="flex items-center gap-2">
                                  <span className="font-bold text-base text-foreground">
                                    {g.name}
                                  </span>
                                  {g.name_bn && (
                                    <span className="text-xs text-muted-foreground font-bengali">
                                      ({g.name_bn})
                                    </span>
                                  )}
                                  <Badge variant="outline" className="capitalize">
                                    {g.relationship}
                                  </Badge>
                                  {g.is_primary && (
                                    <Badge variant="paid">Primary Guardian</Badge>
                                  )}
                                  {g.receives_sms && (
                                    <Badge variant="active">Receives SMS</Badge>
                                  )}
                                </div>
                                <div className="flex items-center gap-3 text-xs text-muted-foreground mt-1">
                                  <span className="font-mono font-medium text-foreground">
                                    {formatPhone(g.phone_norm)}
                                  </span>
                                  {g.phone2_norm && (
                                    <>
                                      <span>•</span>
                                      <span className="font-mono">
                                        Alt: {formatPhone(g.phone2_norm)}
                                      </span>
                                    </>
                                  )}
                                  {g.occupation && (
                                    <>
                                      <span>•</span>
                                      <span>{g.occupation}</span>
                                    </>
                                  )}
                                </div>
                              </div>

                              {hasPermission("students.manage") && (
                                <Button
                                  variant="outline"
                                  size="sm"
                                  className="gap-1.5"
                                  onClick={() => setEditingGuardianId(g.id)}
                                >
                                  <Edit2 className="h-3.5 w-3.5" />
                                  Edit Guardian
                                </Button>
                              )}
                            </div>

                            {g.present_address && (
                              <div className="text-xs text-muted-foreground border-t border-border/50 pt-2 flex items-center gap-1.5">
                                <MapPin className="h-3 w-3 shrink-0" />
                                <span>{g.present_address}</span>
                              </div>
                            )}
                          </div>
                        ))}
                      </div>

                      {/* Siblings Section */}
                      <div className="pt-4 space-y-3 border-t border-border">
                        <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-2">
                          <Users className="h-4 w-4 text-primary" />
                          Enrolled Siblings ({siblings.length})
                        </h3>

                        {siblings.length === 0 ? (
                          <div className="p-4 rounded-[var(--radius-md)] border border-dashed border-border text-center text-xs text-muted-foreground">
                            No other students currently share the guardians listed above.
                          </div>
                        ) : (
                          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            {siblings.map((sib) => (
                              <div
                                key={sib.id}
                                onClick={() => onSelectStudent?.(sib.id)}
                                className="p-3 rounded-[var(--radius-md)] border border-border bg-card hover:bg-muted/40 cursor-pointer flex items-center justify-between transition-colors shadow-xs"
                              >
                                <div>
                                  <div className="text-sm font-bold text-foreground">
                                    {sib.name}
                                  </div>
                                  <div className="text-xs text-muted-foreground">
                                    <span className="font-mono text-primary font-semibold">
                                      {sib.student_code}
                                    </span>
                                    {sib.class_level ? ` • ${sib.class_level}` : ""}
                                  </div>
                                </div>
                                <Button variant="ghost" size="sm" className="text-xs">
                                  Switch Profile
                                </Button>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  </TabsContent>

                  {/* 3. Documents Tab */}
                  <TabsContent value="documents" className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h3 className="text-sm font-bold uppercase tracking-wider text-muted-foreground">
                          Student Vault Documents
                        </h3>
                        <p className="text-xs text-muted-foreground">
                          PDF certificates, birth records, and mark sheets stored in the
                          content-addressed vault.
                        </p>
                      </div>

                      {hasPermission("students.manage") && (
                        <div>
                          <Button
                            size="sm"
                            className="gap-1.5"
                            onClick={() => docInputRef.current?.click()}
                            disabled={uploadingDoc}
                          >
                            {uploadingDoc ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <Upload className="h-4 w-4" />
                            )}
                            Upload Document
                          </Button>
                          <input
                            ref={docInputRef}
                            type="file"
                            accept="application/pdf,image/jpeg,image/png"
                            className="hidden"
                            onChange={handleDocumentUpload}
                          />
                        </div>
                      )}
                    </div>

                    {documents.length === 0 ? (
                      <div className="py-12 border-2 border-dashed border-border rounded-[var(--radius-lg)] text-center text-muted-foreground space-y-2">
                        <FileText className="h-8 w-8 mx-auto text-muted-foreground/50" />
                        <div className="text-sm font-medium">
                          No documents uploaded yet
                        </div>
                        <p className="text-xs max-w-sm mx-auto">
                          Upload PDF transcripts, birth registrations, and certificates
                          for long-term encrypted local storage.
                        </p>
                      </div>
                    ) : (
                      <div className="border border-border rounded-[var(--radius-lg)] divide-y divide-border bg-card overflow-hidden">
                        {documents.map((doc) => (
                          <div
                            key={doc.id}
                            className="p-3.5 flex items-center justify-between hover:bg-muted/30 transition-colors"
                          >
                            <div className="flex items-center gap-3">
                              <div className="h-9 w-9 rounded bg-primary/10 text-primary flex items-center justify-center shrink-0">
                                <FileText className="h-5 w-5" />
                              </div>
                              <div>
                                <span className="text-sm font-bold text-foreground block">
                                  {doc.original_name}
                                </span>
                                <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
                                  <span>{(doc.size / 1024).toFixed(1)} KB</span>
                                  <span>•</span>
                                  <span className="uppercase">
                                    {doc.mime.split("/")[1] || doc.mime}
                                  </span>
                                  <span>•</span>
                                  <span>
                                    Uploaded{" "}
                                    {formatBusinessDate(doc.created_at.slice(0, 10))}
                                  </span>
                                </div>
                              </div>
                            </div>

                            <Button
                              variant="ghost"
                              size="sm"
                              className="gap-1.5 text-xs text-primary"
                              onClick={() =>
                                downloadDocument(doc.id, doc.original_name, doc.mime)
                              }
                            >
                              <Download className="h-4 w-4" />
                              Download
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}
                  </TabsContent>

                  {/* 4. Academic Tab */}
                  <TabsContent value="academic" className="space-y-4">
                    <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs">
                      <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                        Academic Structure & Batches
                      </h3>
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
                        <div>
                          <span className="text-xs text-muted-foreground">
                            Class / Level
                          </span>
                          <p className="font-medium text-foreground">
                            {student.class_level || "Not assigned"}
                          </p>
                        </div>
                        <div>
                          <span className="text-xs text-muted-foreground">
                            School Roll
                          </span>
                          <p className="font-medium text-foreground">
                            {student.current_school_roll || "Not assigned"}
                          </p>
                        </div>
                        <div>
                          <span className="text-xs text-muted-foreground">
                            Academic Group
                          </span>
                          <p className="font-medium text-foreground">
                            {student.group_major || "General"}
                          </p>
                        </div>
                        <div>
                          <span className="text-xs text-muted-foreground">
                            Previous Institution
                          </span>
                          <p className="font-medium text-foreground">
                            {student.previous_school || "None recorded"}
                          </p>
                        </div>
                      </div>
                    </div>

                    {/* Active & Historical Enrollments */}
                    <div className="p-4 rounded-[var(--radius-lg)] border border-border bg-card space-y-3 shadow-xs">
                      <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                        <Layers className="h-3.5 w-3.5 text-primary" />
                        Batch & Section Enrollments ({enrollments.length})
                      </h3>

                      {enrollments.length === 0 ? (
                        <div className="p-4 text-center text-xs text-muted-foreground border border-dashed rounded-[var(--radius)]">
                          Student is not enrolled in any academic batches yet.
                        </div>
                      ) : (
                        <div className="space-y-2">
                          {enrollments.map((e) => (
                            <div
                              key={e.enrollment_id}
                              className="p-3 rounded-[var(--radius-md)] border border-border/80 bg-muted/20 flex items-center justify-between text-xs"
                            >
                              <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                  <span className="font-semibold text-foreground">
                                    {e.batch_name}
                                  </span>
                                  <span className="font-mono text-[10px] text-muted-foreground">
                                    ({e.batch_code})
                                  </span>
                                  <Badge
                                    variant={
                                      e.status === "active" ? "default" : "secondary"
                                    }
                                    className="text-[10px] capitalize"
                                  >
                                    {e.status}
                                  </Badge>
                                </div>
                                <div className="text-[11px] text-muted-foreground flex items-center gap-2">
                                  <span>{e.program_name}</span>
                                  <span>•</span>
                                  <span>{e.shift} Shift</span>
                                  {e.room_number && (
                                    <>
                                      <span>•</span>
                                      <span>{e.room_number}</span>
                                    </>
                                  )}
                                  <span>•</span>
                                  <span>
                                    From {formatBusinessDate(e.start_date)}
                                    {e.end_date
                                      ? ` to ${formatBusinessDate(e.end_date)}`
                                      : " (Present)"}
                                  </span>
                                </div>
                              </div>

                              <div className="text-right font-mono">
                                <span className="px-2 py-0.5 rounded bg-primary/10 text-primary font-bold text-xs">
                                  Roll #{e.roll || "—"}
                                </span>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </TabsContent>

                  {/* 5. Finance and Ledger Tab */}
                  <TabsContent value="finance" className="space-y-6">
                    {financeLoading ? (
                      <div className="p-12 flex flex-col items-center justify-center gap-3">
                        <Loader2 className="h-8 w-8 animate-spin text-primary" />
                        <span className="text-xs font-medium text-muted-foreground">
                          Loading student finance and ledger overview...
                        </span>
                      </div>
                    ) : financeOverview ? (
                      <div className="space-y-6">
                        {/* Top Action and Metrics Bar */}
                        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-[var(--radius-lg)] border border-border bg-card">
                          <div>
                            <div className="text-xs text-muted-foreground uppercase font-semibold">
                              Net Outstanding Balance
                            </div>
                            <div
                              className={`text-2xl font-extrabold tabular-nums mt-0.5 ${
                                financeOverview.net_balance_poisha > 0
                                  ? "text-red-600 dark:text-red-400"
                                  : financeOverview.net_balance_poisha < 0
                                    ? "text-emerald-600 dark:text-emerald-400"
                                    : "text-foreground"
                              }`}
                            >
                              {financeOverview.net_balance_poisha > 0
                                ? `${formatTaka(financeOverview.net_balance_poisha)} Due`
                                : financeOverview.net_balance_poisha < 0
                                  ? `${formatTaka(-financeOverview.net_balance_poisha)} Advance Credit`
                                  : "৳ 0 (Fully Cleared)"}
                            </div>
                          </div>

                          <div className="flex items-center gap-2 flex-wrap">
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => void handlePrintStatement()}
                              disabled={statementPrinting}
                              className="h-8 text-xs font-semibold gap-1.5"
                            >
                              <Printer className="h-3.5 w-3.5" />
                              {statementPrinting ? "Generating..." : "Print Statement"}
                            </Button>

                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setAdjustmentDialogOpen(true)}
                              className="h-8 text-xs font-semibold gap-1.5"
                            >
                              <Sliders className="h-3.5 w-3.5 text-primary" />
                              Waiver / Adjustment
                            </Button>

                            <Button
                              size="sm"
                              onClick={() => {
                                onSelectStudent?.(student.id);
                                onOpenChange(false);
                              }}
                              className="h-8 text-xs font-semibold gap-1.5"
                            >
                              <CreditCard className="h-3.5 w-3.5" />
                              Collect at Counter
                            </Button>
                          </div>
                        </div>

                        {/* Balance Cards Grid */}
                        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                          <div className="p-3.5 rounded-[var(--radius-md)] bg-muted/30 border border-border">
                            <span className="text-[11px] font-semibold text-muted-foreground uppercase">
                              Total Unpaid Due
                            </span>
                            <div className="text-lg font-bold text-red-600 dark:text-red-400 tabular-nums mt-0.5">
                              {formatTaka(financeOverview.total_due_poisha)}
                            </div>
                            <span className="text-[10px] text-muted-foreground">
                              {financeOverview.open_charges.length} unpaid charge invoices
                            </span>
                          </div>

                          <div className="p-3.5 rounded-[var(--radius-md)] bg-muted/30 border border-border">
                            <span className="text-[11px] font-semibold text-muted-foreground uppercase">
                              Advance Credit Deposit
                            </span>
                            <div className="text-lg font-bold text-emerald-600 dark:text-emerald-400 tabular-nums mt-0.5">
                              {formatTaka(financeOverview.advance_credit_poisha)}
                            </div>
                            <span className="text-[10px] text-muted-foreground">
                              Available for future bill settlement
                            </span>
                          </div>

                          <div className="p-3.5 rounded-[var(--radius-md)] bg-muted/30 border border-border">
                            <span className="text-[11px] font-semibold text-muted-foreground uppercase">
                              Recent Payments Count
                            </span>
                            <div className="text-lg font-bold text-foreground tabular-nums mt-0.5">
                              {financeOverview.recent_payments.length} receipts
                            </div>
                            <span className="text-[10px] text-muted-foreground">
                              Recorded in immutable ledger
                            </span>
                          </div>
                        </div>

                        {financeNotice && (
                          <div className="p-3 text-xs rounded-[var(--radius-md)] bg-primary/10 border border-primary/20 text-primary font-medium flex items-center gap-2">
                            <CheckCircle2 className="h-4 w-4 shrink-0" />
                            <span>{financeNotice}</span>
                          </div>
                        )}

                        {/* Itemized Open Charges Table */}
                        <div className="rounded-[var(--radius-lg)] border border-border bg-card overflow-hidden shadow-xs">
                          <div className="p-3.5 border-b border-border bg-muted/20 flex items-center justify-between">
                            <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                              <FileText className="h-3.5 w-3.5 text-primary" />
                              Open Invoices and Charges
                            </h3>
                            <span className="text-[11px] text-muted-foreground">
                              {financeOverview.open_charges.length} active
                            </span>
                          </div>

                          {financeOverview.open_charges.length === 0 ? (
                            <div className="p-6 text-center text-xs text-muted-foreground">
                              No outstanding dues or unpaid invoices for this student.
                            </div>
                          ) : (
                            <div className="overflow-x-auto">
                              <table className="w-full text-xs">
                                <thead>
                                  <tr className="border-b border-border bg-muted/10 text-muted-foreground text-[11px]">
                                    <th className="p-2.5 text-left">Fee Head</th>
                                    <th className="p-2.5 text-left">Period</th>
                                    <th className="p-2.5 text-left">Due Date</th>
                                    <th className="p-2.5 text-right">Net Amount</th>
                                    <th className="p-2.5 text-right font-semibold text-foreground">
                                      Remaining Due
                                    </th>
                                  </tr>
                                </thead>
                                <tbody className="divide-y divide-border">
                                  {financeOverview.open_charges.map((c) => {
                                    const isOverdue =
                                      c.due_date < new Date().toISOString().slice(0, 10);
                                    return (
                                      <tr
                                        key={c.id}
                                        className="hover:bg-muted/30 transition-colors"
                                      >
                                        <td className="p-2.5 font-medium text-foreground">
                                          {c.fee_head_name}
                                          <span className="ml-1.5 text-[10px] text-muted-foreground capitalize">
                                            ({c.fee_head_kind})
                                          </span>
                                        </td>
                                        <td className="p-2.5 text-muted-foreground">
                                          {c.period || "—"}
                                        </td>
                                        <td className="p-2.5">
                                          <span>{c.due_date}</span>
                                          {isOverdue && (
                                            <span className="ml-1.5 px-1.5 py-0.2 rounded text-[9px] font-bold bg-red-500/10 text-red-600">
                                              OVERDUE
                                            </span>
                                          )}
                                        </td>
                                        <td className="p-2.5 text-right tabular-nums text-muted-foreground">
                                          {formatTaka(c.net_amount_poisha)}
                                        </td>
                                        <td className="p-2.5 text-right font-bold text-red-600 dark:text-red-400 tabular-nums">
                                          {formatTaka(c.remaining_due_poisha)}
                                        </td>
                                      </tr>
                                    );
                                  })}
                                </tbody>
                              </table>
                            </div>
                          )}
                        </div>

                        {/* Recent Payments History Table */}
                        <div className="rounded-[var(--radius-lg)] border border-border bg-card overflow-hidden shadow-xs">
                          <div className="p-3.5 border-b border-border bg-muted/20 flex items-center justify-between">
                            <h3 className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                              <HistoryIcon className="h-3.5 w-3.5 text-primary" />
                              Recent Payment Receipts (Last 5)
                            </h3>
                          </div>

                          {financeOverview.recent_payments.length === 0 ? (
                            <div className="p-6 text-center text-xs text-muted-foreground">
                              No payment receipts recorded for this student yet.
                            </div>
                          ) : (
                            <div className="divide-y divide-border">
                              {financeOverview.recent_payments.map((p) => (
                                <div
                                  key={p.id}
                                  className="p-3 flex items-center justify-between text-xs hover:bg-muted/30 transition-colors"
                                >
                                  <div className="flex items-center gap-3">
                                    <div className="h-7 w-7 rounded bg-muted flex items-center justify-center font-mono text-[11px] font-bold text-foreground">
                                      #{p.receipt_no.slice(-4)}
                                    </div>
                                    <div>
                                      <div className="font-semibold text-foreground flex items-center gap-1.5">
                                        <span>{p.receipt_no}</span>
                                        <span className="text-muted-foreground font-normal text-[11px]">
                                          • {p.business_date}
                                        </span>
                                      </div>
                                      <div className="text-[11px] text-muted-foreground">
                                        Method:{" "}
                                        <strong className="uppercase">{p.method}</strong>
                                        {p.reference && ` (${p.reference})`}
                                      </div>
                                    </div>
                                  </div>

                                  <div className="flex items-center gap-2">
                                    <div className="text-right">
                                      <div className="font-bold text-emerald-600 dark:text-emerald-400 tabular-nums">
                                        {formatTaka(p.amount_poisha)}
                                      </div>
                                      <div className="text-[10px] text-muted-foreground">
                                        {p.allocated_amount_poisha > 0 && "Settled Due"}
                                        {p.unallocated_amount_poisha > 0 && " + Advance"}
                                      </div>
                                    </div>

                                    <Button
                                      variant="ghost"
                                      size="sm"
                                      onClick={() => void handleReprintReceipt(p.id)}
                                      className="h-7 text-xs gap-1 text-muted-foreground hover:text-foreground"
                                    >
                                      <Printer className="h-3 w-3" />
                                      Reprint
                                    </Button>

                                    {p.unallocated_amount_poisha > 0 && (
                                      <Button
                                        variant="outline"
                                        size="sm"
                                        onClick={() => {
                                          setRefundTarget(p);
                                          setRefundDialogOpen(true);
                                        }}
                                        className="h-7 text-xs gap-1 text-primary hover:text-primary"
                                      >
                                        <ArrowDownLeft className="h-3 w-3" />
                                        Refund
                                      </Button>
                                    )}

                                    <Button
                                      variant="ghost"
                                      size="sm"
                                      onClick={() => {
                                        setVoidPaymentTarget(p);
                                        setVoidDialogOpen(true);
                                      }}
                                      className="h-7 text-xs gap-1 text-destructive hover:bg-destructive/10 hover:text-destructive"
                                    >
                                      <RotateCcw className="h-3 w-3" />
                                      Void
                                    </Button>
                                  </div>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      </div>
                    ) : (
                      <div className="p-8 text-center text-xs text-muted-foreground">
                        Unable to load student financial overview.
                      </div>
                    )}
                  </TabsContent>
                </Tabs>
              </div>

              <DialogFooter className="p-4 border-t border-border bg-muted/20">
                <Button variant="outline" onClick={() => onOpenChange(false)}>
                  Close
                </Button>
              </DialogFooter>
            </div>
          ) : (
            <div className="py-16 text-center text-muted-foreground">
              Student record not found.
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Edit Shared Guardian Dialog */}
      {student && (
        <EditSharedGuardianDialog
          open={Boolean(editingGuardianId)}
          onOpenChange={(op) => !op && setEditingGuardianId(null)}
          guardianId={editingGuardianId}
          currentStudentId={student.id}
          currentStudentName={student.name}
          onSuccess={() => {
            void loadStudent();
            onStudentUpdated?.();
          }}
        />
      )}

      {/* Archive Reason Confirmation Modal */}
      {student && (
        <Dialog open={archiveModalOpen} onOpenChange={setArchiveModalOpen}>
          <DialogContent className="max-w-md">
            <DialogHeader>
              <DialogTitle className="text-lg font-bold flex items-center gap-2 text-destructive">
                <AlertTriangle className="h-5 w-5" />
                Archive Student Profile
              </DialogTitle>
            </DialogHeader>

            <div className="space-y-3 pt-2">
              <p className="text-xs text-muted-foreground">
                Archiving student{" "}
                <span className="font-bold text-foreground">{student.name}</span> (
                {student.student_code}) will deactivate active batch enrollments and mark
                their status as archived.
              </p>

              <div>
                <label className="text-xs font-semibold text-foreground">
                  Reason for Archiving <span className="text-destructive">*</span>
                </label>
                <textarea
                  className="mt-1 flex w-full rounded-[var(--radius-md)] border border-input bg-transparent px-3 py-2 text-sm shadow-xs focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  rows={3}
                  value={archiveReason}
                  onChange={(e) => setArchiveReason(e.target.value)}
                  placeholder="e.g. Student relocated, completed coaching, dropped out..."
                  required
                />
              </div>
            </div>

            <DialogFooter className="pt-4 gap-2">
              <Button
                variant="outline"
                onClick={() => setArchiveModalOpen(false)}
                disabled={archiving}
              >
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => void handleArchive()}
                disabled={archiving || !archiveReason.trim()}
              >
                {archiving ? "Archiving..." : "Confirm Archive"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )}

      {/* Void Payment Dialog */}
      <VoidPaymentDialog
        payment={voidPaymentTarget}
        open={voidDialogOpen}
        onOpenChange={setVoidDialogOpen}
        onSuccess={() => {
          void loadFinanceOverview();
          onStudentUpdated?.();
        }}
      />

      {/* Refund Dialog */}
      <RefundDialog
        payment={refundTarget}
        open={refundDialogOpen}
        onOpenChange={setRefundDialogOpen}
        onSuccess={() => {
          void loadFinanceOverview();
          onStudentUpdated?.();
        }}
      />

      {/* Adjustment Dialog */}
      {student && (
        <AdjustmentDialog
          studentId={student.id}
          studentName={student.name}
          charges={financeOverview?.open_charges || []}
          open={adjustmentDialogOpen}
          onOpenChange={setAdjustmentDialogOpen}
          onSuccess={() => {
            void loadFinanceOverview();
            onStudentUpdated?.();
          }}
        />
      )}
    </>
  );
}
