import { useState, useEffect, useRef, useCallback } from "react";
import {
  rpc,
  type FileInspectionResult,
  type ColumnMappingInput,
  type ValidationPreviewResult,
  type ImportSummaryResult,
  type BatchSummary,
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
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  FileSpreadsheet,
  Upload,
  ArrowRight,
  ArrowLeft,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  ShieldCheck,
  Copy,
  Check,
  Users,
  Loader2,
  FileText,
  AlertCircle,
  Database,
  Layers,
} from "lucide-react";

export interface ImportStudentsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImportSuccess?: () => void;
}

const TARGET_FIELDS: { id: string; label: string; required: boolean; hint: string }[] = [
  {
    id: "student_name",
    label: "Student Name",
    required: true,
    hint: "Primary student name",
  },
  { id: "gender", label: "Gender", required: true, hint: "male, female, other" },
  { id: "dob", label: "Date of Birth", required: true, hint: "YYYY-MM-DD or DD/MM/YYYY" },
  {
    id: "student_phone",
    label: "Student Phone",
    required: true,
    hint: "11 or 13-digit Bangladeshi phone",
  },
  {
    id: "guardian_name",
    label: "Guardian Name",
    required: true,
    hint: "Parent or guardian name",
  },
  {
    id: "guardian_phone",
    label: "Guardian Phone",
    required: true,
    hint: "Primary family contact",
  },
  {
    id: "present_address",
    label: "Present Address",
    required: false,
    hint: "Current living address",
  },
  {
    id: "student_name_bn",
    label: "Student Name (Bangla)",
    required: false,
    hint: "বাংলায় নাম",
  },
  {
    id: "student_code",
    label: "Student Code / ID",
    required: false,
    hint: "Custom enrollment code",
  },
  {
    id: "guardian_relationship",
    label: "Guardian Relationship",
    required: false,
    hint: "father, mother, legal_guardian",
  },
  {
    id: "emergency_phone",
    label: "Emergency Phone",
    required: false,
    hint: "Secondary contact phone",
  },
  {
    id: "blood_group",
    label: "Blood Group",
    required: false,
    hint: "A+, B+, O+, AB+, etc.",
  },
  { id: "religion", label: "Religion", required: false, hint: "Islam, Hinduism, etc." },
  { id: "email", label: "Email Address", required: false, hint: "Contact email" },
  {
    id: "class_level",
    label: "Class / Grade Level",
    required: false,
    hint: "Academic grade",
  },
  {
    id: "current_school_roll",
    label: "School Roll No",
    required: false,
    hint: "Institutional roll",
  },
  {
    id: "permanent_address",
    label: "Permanent Address",
    required: false,
    hint: "Permanent home address",
  },
  {
    id: "previous_school",
    label: "Previous School",
    required: false,
    hint: "Previous institution",
  },
];

export function ImportStudentsDialog({
  open,
  onOpenChange,
  onImportSuccess,
}: ImportStudentsDialogProps) {
  const [step, setStep] = useState<1 | 2 | 3 | 4>(1);
  const [filePath, setFilePath] = useState("");
  const [selectedSheet, setSelectedSheet] = useState<string>("");
  const [inspection, setInspection] = useState<FileInspectionResult | null>(null);
  const [inspecting, setInspecting] = useState(false);

  // Step 2: Mappings
  const [mappings, setMappings] = useState<Record<string, string>>({});

  // Step 3: Validation Preview & Options
  const [validating, setValidating] = useState(false);
  const [preview, setPreview] = useState<ValidationPreviewResult | null>(null);
  const [activeTab, setActiveTab] = useState<"all" | "valid" | "warning" | "error">(
    "all"
  );
  const [skipErrorRows, setSkipErrorRows] = useState(true);
  const [phoneConflictMode, setPhoneConflictMode] = useState<string>("link_guardian");
  const [targetBatchId, setTargetBatchId] = useState<string>("");
  const [defaultAddress, setDefaultAddress] = useState<string>("Dhaka, Bangladesh");

  // Step 4: Execution
  const [executing, setExecuting] = useState(false);
  const [importSummary, setImportSummary] = useState<ImportSummaryResult | null>(null);
  const [copiedBackup, setCopiedBackup] = useState(false);
  const [copiedRejected, setCopiedRejected] = useState(false);

  // Available batches for enrollment
  const [batches, setBatches] = useState<BatchSummary[]>([]);

  // General error banner
  const [error, setError] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Reset dialog state when opening/closing
  useEffect(() => {
    if (open) {
      setStep(1);
      setFilePath("");
      setSelectedSheet("");
      setInspection(null);
      setMappings({});
      setPreview(null);
      setActiveTab("all");
      setSkipErrorRows(true);
      setPhoneConflictMode("link_guardian");
      setTargetBatchId("");
      setDefaultAddress("Dhaka, Bangladesh");
      setImportSummary(null);
      setError(null);

      // Preload active batches for Step 3
      void rpc("batches.list", { status: "active" }).then((res) => {
        if (res.ok && res.data) {
          setBatches(res.data);
        }
      });
    }
  }, [open]);

  // Handle local file selection from file input
  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Check if browser/webview provides path property
    const candidatePath = (file as File & { path?: string }).path || file.name;
    setFilePath(candidatePath);
    void inspectFilePath(candidatePath);
  };

  const inspectFilePath = async (path: string) => {
    if (!path.trim()) {
      setError("Please specify a valid spreadsheet file path");
      return;
    }

    setInspecting(true);
    setError(null);

    try {
      const res = await rpc("import.inspect_file", { file_path: path.trim() });
      if (res.ok && res.data) {
        setInspection(res.data);
        const firstSheet = res.data.sheets[0]?.name || "";
        setSelectedSheet(firstSheet);

        // Pre-populate mappings from suggested_mappings
        const initialMap: Record<string, string> = {};
        for (const sug of res.data.suggested_mappings) {
          if (sug.target_field) {
            initialMap[sug.source_column] = sug.target_field;
          }
        }
        setMappings(initialMap);
      } else {
        setError(res.error?.message || "Failed to inspect file");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setInspecting(false);
    }
  };

  // Convert mapping state to array of ColumnMappingInput
  const getMappingArray = useCallback((): ColumnMappingInput[] => {
    const result: ColumnMappingInput[] = [];
    for (const [src, tgt] of Object.entries(mappings)) {
      if (tgt && tgt.trim().length > 0) {
        result.push({ source_column: src, target_field: tgt.trim() });
      }
    }
    return result;
  }, [mappings]);

  // Run validation preview
  const runPreviewValidation = async () => {
    if (!inspection) return;

    setValidating(true);
    setError(null);

    try {
      const mappingArray = getMappingArray();
      const res = await rpc("import.preview_validation", {
        file_path: filePath.trim(),
        sheet_name: selectedSheet ? selectedSheet : null,
        mappings: mappingArray,
        phone_conflict_mode: phoneConflictMode,
        default_present_address: defaultAddress.trim() || null,
      });

      if (res.ok && res.data) {
        setPreview(res.data);
        setStep(3);
      } else {
        setError(res.error?.message || "Validation preview failed");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setValidating(false);
    }
  };

  // Execute import
  const handleExecuteImport = async () => {
    if (!inspection) return;

    setExecuting(true);
    setError(null);

    try {
      const mappingArray = getMappingArray();
      const res = await rpc("import.execute", {
        file_path: filePath.trim(),
        sheet_name: selectedSheet ? selectedSheet : null,
        mappings: mappingArray,
        skip_error_rows: skipErrorRows,
        phone_conflict_mode: phoneConflictMode,
        target_batch_id: targetBatchId ? targetBatchId : null,
        default_present_address: defaultAddress.trim() || null,
      });

      if (res.ok && res.data) {
        setImportSummary(res.data);
        setStep(4);
        onImportSuccess?.();
      } else {
        setError(res.error?.message || "Import execution failed");
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setExecuting(false);
    }
  };

  // Check which required fields are mapped
  const mappedTargets = new Set(Object.values(mappings).filter(Boolean));
  const missingRequiredFields = TARGET_FIELDS.filter(
    (f) => f.required && !mappedTargets.has(f.id)
  );
  const canProceedToPreview = missingRequiredFields.length === 0;

  // Filter preview rows
  const filteredRows = (preview?.rows_sample || []).filter((r) => {
    if (activeTab === "valid") return r.is_valid && !r.has_warnings;
    if (activeTab === "warning") return r.has_warnings;
    if (activeTab === "error") return r.has_errors;
    return true;
  });

  const copyToClipboard = (text: string, type: "backup" | "rejected") => {
    navigator.clipboard.writeText(text);
    if (type === "backup") {
      setCopiedBackup(true);
      setTimeout(() => setCopiedBackup(false), 2000);
    } else {
      setCopiedRejected(true);
      setTimeout(() => setCopiedRejected(false), 2000);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[92vh] flex flex-col p-0 gap-0 overflow-hidden">
        {/* Header */}
        <DialogHeader className="px-6 py-4 border-b border-border bg-muted/20 shrink-0">
          <div className="flex items-center justify-between">
            <DialogTitle className="text-xl font-bold flex items-center gap-2 text-foreground">
              <FileSpreadsheet className="h-5 w-5 text-primary" />
              Bulk Student Import
            </DialogTitle>
            {/* Step Pills */}
            <div className="flex items-center gap-1.5 text-xs">
              {[
                { s: 1, label: "File" },
                { s: 2, label: "Mapping" },
                { s: 3, label: "Validation" },
                { s: 4, label: "Summary" },
              ].map(({ s, label }) => (
                <div
                  key={s}
                  className={`flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
                    step === s
                      ? "bg-primary text-primary-foreground font-semibold shadow-xs"
                      : step > s
                        ? "bg-primary-subtle text-primary"
                        : "bg-muted text-muted-foreground"
                  }`}
                >
                  <span>{s}.</span>
                  <span>{label}</span>
                </div>
              ))}
            </div>
          </div>
        </DialogHeader>

        {/* Global Error Banner */}
        {error && (
          <div className="mx-6 mt-4 p-3 rounded-[var(--radius-md)] bg-danger-subtle border border-danger-border flex items-start gap-2.5 text-danger-text text-xs shrink-0">
            <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
            <div className="flex-1 font-medium">{error}</div>
            <button
              onClick={() => setError(null)}
              className="text-muted-foreground hover:text-foreground cursor-pointer"
            >
              <XCircle className="h-4 w-4" />
            </button>
          </div>
        )}

        {/* Wizard Step Body */}
        <div className="p-6 overflow-y-auto flex-1 space-y-6">
          {/* STEP 1: FILE SELECTION & INSPECTION */}
          {step === 1 && (
            <div className="space-y-6">
              <div>
                <h3 className="text-base font-semibold text-foreground">
                  Select Spreadsheet File
                </h3>
                <p className="text-xs text-muted-foreground mt-1">
                  Upload or specify an Excel (.xlsx, .xls) or CSV (.csv) file containing
                  student and guardian admissions.
                </p>
              </div>

              {/* Dropzone & Picker */}
              <div
                onClick={() => fileInputRef.current?.click()}
                className="border-2 border-dashed border-border hover:border-primary rounded-[var(--radius-lg)] p-8 text-center bg-card hover:bg-muted/10 transition-colors cursor-pointer space-y-3"
              >
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".xlsx,.xls,.csv,.ods"
                  className="hidden"
                  onChange={handleFileChange}
                />
                <div className="h-12 w-12 rounded-full bg-primary-subtle flex items-center justify-center mx-auto text-primary">
                  <Upload className="h-6 w-6" />
                </div>
                <div>
                  <div className="text-sm font-semibold text-foreground">
                    Click to select spreadsheet or drop file here
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    Supports Microsoft Excel (.xlsx, .xls) and standard Comma-Separated
                    Values (.csv)
                  </p>
                </div>
                <div className="flex items-center justify-center gap-2 pt-2">
                  <Badge variant="outline" className="text-[11px] font-mono">
                    .XLSX
                  </Badge>
                  <Badge variant="outline" className="text-[11px] font-mono">
                    .XLS
                  </Badge>
                  <Badge variant="outline" className="text-[11px] font-mono">
                    .CSV
                  </Badge>
                </div>
              </div>

              {/* Direct Path Input */}
              <div className="space-y-2">
                <label className="text-xs font-semibold text-foreground">
                  Or enter full system file path:
                </label>
                <div className="flex items-center gap-2">
                  <Input
                    placeholder="e.g. C:\Users\User\Documents\students_batch_2026.xlsx"
                    value={filePath}
                    onChange={(e) => setFilePath(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        void inspectFilePath(filePath);
                      }
                    }}
                    className="font-mono text-xs"
                  />
                  <Button
                    onClick={() => inspectFilePath(filePath)}
                    disabled={inspecting || !filePath.trim()}
                    size="sm"
                    className="gap-1.5 shrink-0 cursor-pointer"
                  >
                    {inspecting ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <ArrowRight className="h-4 w-4" />
                    )}
                    Inspect
                  </Button>
                </div>
              </div>

              {/* Inspection Details */}
              {inspection && (
                <div className="rounded-[var(--radius-lg)] border border-border bg-card p-4 space-y-4 shadow-xs">
                  <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-border">
                    <div className="flex items-center gap-3">
                      <div className="p-2 rounded-[var(--radius-md)] bg-primary-subtle text-primary">
                        <FileSpreadsheet className="h-5 w-5" />
                      </div>
                      <div>
                        <div className="text-sm font-semibold text-foreground flex items-center gap-2">
                          {inspection.file_name}
                          <Badge variant="paid" className="uppercase text-[10px]">
                            {inspection.format}
                          </Badge>
                        </div>
                        <div className="text-xs text-muted-foreground mt-0.5">
                          Size: {(inspection.file_size_bytes / 1024).toFixed(1)} KB •
                          Sheets: {inspection.sheets.length}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Multi-sheet selector */}
                  {inspection.sheets.length > 1 && (
                    <div className="space-y-1.5">
                      <label className="text-xs font-medium text-foreground flex items-center gap-1.5">
                        <Layers className="h-3.5 w-3.5 text-primary" />
                        Select Sheet to Import:
                      </label>
                      <div className="flex flex-wrap gap-2">
                        {inspection.sheets.map((s) => (
                          <button
                            key={s.name}
                            type="button"
                            onClick={() => setSelectedSheet(s.name)}
                            className={`px-3 py-1.5 rounded-[var(--radius-md)] text-xs font-medium border transition-colors cursor-pointer ${
                              selectedSheet === s.name
                                ? "bg-primary text-primary-foreground border-primary"
                                : "bg-card text-foreground border-border hover:bg-muted"
                            }`}
                          >
                            {s.name} ({s.row_count} rows)
                          </button>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Inspection summary stats */}
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 pt-1">
                    <div className="p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border">
                      <div className="text-[11px] text-muted-foreground">
                        Estimated Rows
                      </div>
                      <div className="text-base font-bold text-foreground">
                        {inspection.sheets.find((s) => s.name === selectedSheet)
                          ?.row_count ??
                          inspection.sheets[0]?.row_count ??
                          0}
                      </div>
                    </div>
                    <div className="p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border">
                      <div className="text-[11px] text-muted-foreground">
                        Columns Detected
                      </div>
                      <div className="text-base font-bold text-foreground">
                        {inspection.suggested_mappings.length}
                      </div>
                    </div>
                    <div className="p-2.5 rounded-[var(--radius-md)] bg-muted/30 border border-border col-span-2 sm:col-span-1">
                      <div className="text-[11px] text-muted-foreground">
                        Auto-Mapped Fields
                      </div>
                      <div className="text-base font-bold text-success-text">
                        {
                          inspection.suggested_mappings.filter(
                            (m) => m.target_field && m.confidence >= 0.65
                          ).length
                        }
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* STEP 2: COLUMN MAPPING */}
          {step === 2 && inspection && (
            <div className="space-y-4">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                <div>
                  <h3 className="text-base font-semibold text-foreground">
                    Map Spreadsheet Columns
                  </h3>
                  <p className="text-xs text-muted-foreground mt-0.5">
                    Match each column from your file to the corresponding student profile
                    field.
                  </p>
                </div>
                {!canProceedToPreview && (
                  <Badge variant="overdue" className="text-xs self-start sm:self-auto">
                    Missing required:{" "}
                    {missingRequiredFields.map((f) => f.label).join(", ")}
                  </Badge>
                )}
              </div>

              {/* Mapping Table */}
              <div className="border border-border rounded-[var(--radius-lg)] overflow-hidden bg-card shadow-xs">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[28%] text-xs font-semibold">
                        Source Column (File)
                      </TableHead>
                      <TableHead className="w-[30%] text-xs font-semibold">
                        Sample Data (Preview)
                      </TableHead>
                      <TableHead className="w-[28%] text-xs font-semibold">
                        Target Field
                      </TableHead>
                      <TableHead className="w-[14%] text-xs font-semibold text-right">
                        Match Quality
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {inspection.suggested_mappings.map((sug) => {
                      const currentVal = mappings[sug.source_column] || "";
                      const confidence = sug.confidence;

                      return (
                        <TableRow key={sug.source_column} className="hover:bg-muted/10">
                          <TableCell className="font-medium text-xs text-foreground">
                            {sug.source_column}
                          </TableCell>
                          <TableCell className="text-xs text-muted-foreground">
                            <div className="flex flex-wrap gap-1">
                              {sug.sample_values.length > 0 ? (
                                sug.sample_values.map((v, i) => (
                                  <span
                                    key={i}
                                    className="px-1.5 py-0.5 rounded bg-muted text-[11px] font-mono truncate max-w-[120px]"
                                    title={v}
                                  >
                                    {v}
                                  </span>
                                ))
                              ) : (
                                <span className="italic text-muted-foreground text-[11px]">
                                  (empty)
                                </span>
                              )}
                            </div>
                          </TableCell>
                          <TableCell>
                            <select
                              value={currentVal}
                              onChange={(e) => {
                                const val = e.target.value;
                                setMappings((prev) => ({
                                  ...prev,
                                  [sug.source_column]: val,
                                }));
                              }}
                              className="w-full text-xs rounded-[var(--radius-md)] border border-border bg-background px-2.5 py-1.5 text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                            >
                              <option value="">-- Ignore / Unmapped --</option>
                              <optgroup label="Required Fields">
                                {TARGET_FIELDS.filter((f) => f.required).map((f) => (
                                  <option key={f.id} value={f.id}>
                                    {f.label} *
                                  </option>
                                ))}
                              </optgroup>
                              <optgroup label="Optional Fields">
                                {TARGET_FIELDS.filter((f) => !f.required).map((f) => (
                                  <option key={f.id} value={f.id}>
                                    {f.label}
                                  </option>
                                ))}
                              </optgroup>
                            </select>
                          </TableCell>
                          <TableCell className="text-right">
                            {confidence >= 0.85 ? (
                              <Badge variant="paid" className="text-[10px]">
                                High Match
                              </Badge>
                            ) : confidence >= 0.65 ? (
                              <Badge variant="pending" className="text-[10px]">
                                Suggested
                              </Badge>
                            ) : (
                              <Badge
                                variant="outline"
                                className="text-[10px] text-muted-foreground"
                              >
                                Unmapped
                              </Badge>
                            )}
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}

          {/* STEP 3: VALIDATION MATRIX & SETTINGS */}
          {step === 3 && preview && (
            <div className="space-y-6">
              {/* Metric Summary Cards */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div className="p-3.5 rounded-[var(--radius-md)] border border-border bg-card shadow-xs">
                  <div className="text-xs text-muted-foreground font-medium">
                    Total Rows
                  </div>
                  <div className="text-2xl font-bold text-foreground mt-0.5">
                    {preview.total_rows}
                  </div>
                </div>
                <div className="p-3.5 rounded-[var(--radius-md)] border border-success-border bg-success-subtle shadow-xs">
                  <div className="text-xs text-success-text font-medium flex items-center gap-1">
                    <CheckCircle2 className="h-3.5 w-3.5" />
                    Valid Records
                  </div>
                  <div className="text-2xl font-bold text-success-text mt-0.5">
                    {preview.valid_count}
                  </div>
                </div>
                <div className="p-3.5 rounded-[var(--radius-md)] border border-warning-border bg-warning-subtle shadow-xs">
                  <div className="text-xs text-warning-text font-medium flex items-center gap-1">
                    <AlertTriangle className="h-3.5 w-3.5" />
                    Warnings
                  </div>
                  <div className="text-2xl font-bold text-warning-text mt-0.5">
                    {preview.warning_count}
                  </div>
                </div>
                <div className="p-3.5 rounded-[var(--radius-md)] border border-danger-border bg-danger-subtle shadow-xs">
                  <div className="text-xs text-danger-text font-medium flex items-center gap-1">
                    <XCircle className="h-3.5 w-3.5" />
                    Errors
                  </div>
                  <div className="text-2xl font-bold text-danger-text mt-0.5">
                    {preview.error_count}
                  </div>
                </div>
              </div>

              {/* Import Configuration Card */}
              <div className="rounded-[var(--radius-lg)] border border-border bg-card p-4 space-y-4 shadow-xs">
                <div className="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                  <ShieldCheck className="h-4 w-4 text-primary" />
                  Import Strategies & Options
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {/* Skip error rows */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={skipErrorRows}
                        onChange={(e) => setSkipErrorRows(e.target.checked)}
                        className="rounded text-primary focus:ring-primary h-4 w-4 cursor-pointer"
                      />
                      Skip Error Rows & Export Rejections (Selective Mode)
                    </label>
                    <p className="text-[11px] text-muted-foreground ml-6">
                      Valid rows will be imported safely. Rejected rows will be exported
                      to a CSV with error notes.
                    </p>
                  </div>

                  {/* Phone conflict resolution */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">
                      Guardian Phone Duplicate Strategy:
                    </label>
                    <select
                      value={phoneConflictMode}
                      onChange={(e) => setPhoneConflictMode(e.target.value)}
                      className="w-full text-xs rounded-[var(--radius-md)] border border-border bg-background px-2.5 py-1.5 text-foreground"
                    >
                      <option value="link_guardian">
                        Link to Existing Guardian / Family (Recommended)
                      </option>
                      <option value="allow_duplicate">
                        Allow Duplicate Phone Numbers
                      </option>
                      <option value="skip_row">
                        Skip Rows with Existing Phone Conflicts
                      </option>
                    </select>
                  </div>

                  {/* Target Academic Batch */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground flex items-center gap-1">
                      <Users className="h-3.5 w-3.5 text-primary" />
                      Enroll in Academic Batch (Optional):
                    </label>
                    <select
                      value={targetBatchId}
                      onChange={(e) => setTargetBatchId(e.target.value)}
                      className="w-full text-xs rounded-[var(--radius-md)] border border-border bg-background px-2.5 py-1.5 text-foreground"
                    >
                      <option value="">-- Do not enroll into a batch --</option>
                      {batches.map((b) => (
                        <option key={b.id} value={b.id}>
                          {b.program_name} - {b.name} ({b.shift}) • Enrolled:{" "}
                          {b.enrolled_count}/{b.capacity}
                        </option>
                      ))}
                    </select>
                  </div>

                  {/* Fallback Present Address */}
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">
                      Default Present Address Fallback:
                    </label>
                    <Input
                      value={defaultAddress}
                      onChange={(e) => setDefaultAddress(e.target.value)}
                      placeholder="e.g. Dhaka, Bangladesh"
                      className="text-xs"
                    />
                  </div>
                </div>
              </div>

              {/* Rows Sample & Issue Explorer */}
              <div className="space-y-3">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                  <div className="flex items-center gap-1 bg-muted p-1 rounded-[var(--radius-md)]">
                    {(
                      [
                        { id: "all", label: `All (${preview.total_rows})` },
                        { id: "valid", label: `Valid (${preview.valid_count})` },
                        { id: "warning", label: `Warnings (${preview.warning_count})` },
                        { id: "error", label: `Errors (${preview.error_count})` },
                      ] as const
                    ).map((t) => (
                      <button
                        key={t.id}
                        type="button"
                        onClick={() => setActiveTab(t.id)}
                        className={`px-3 py-1 rounded-[var(--radius-sm)] text-xs font-medium transition-colors cursor-pointer ${
                          activeTab === t.id
                            ? "bg-card text-foreground shadow-xs"
                            : "text-muted-foreground hover:text-foreground"
                        }`}
                      >
                        {t.label}
                      </button>
                    ))}
                  </div>

                  <div className="text-xs text-muted-foreground">
                    Showing {filteredRows.length} of {preview.total_rows} records
                  </div>
                </div>

                <div className="border border-border rounded-[var(--radius-lg)] overflow-hidden bg-card shadow-xs max-h-72 overflow-y-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-[8%] text-xs font-semibold">
                          Row
                        </TableHead>
                        <TableHead className="w-[24%] text-xs font-semibold">
                          Student
                        </TableHead>
                        <TableHead className="w-[18%] text-xs font-semibold">
                          Phone
                        </TableHead>
                        <TableHead className="w-[24%] text-xs font-semibold">
                          Guardian
                        </TableHead>
                        <TableHead className="w-[26%] text-xs font-semibold">
                          Issues / Notes
                        </TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {filteredRows.length === 0 ? (
                        <TableRow>
                          <TableCell
                            colSpan={5}
                            className="py-8 text-center text-xs text-muted-foreground"
                          >
                            No rows matching the selected filter.
                          </TableCell>
                        </TableRow>
                      ) : (
                        filteredRows.map((r) => (
                          <TableRow
                            key={r.row_index}
                            className={
                              r.has_errors
                                ? "bg-danger-subtle/30"
                                : r.has_warnings
                                  ? "bg-warning-subtle/20"
                                  : ""
                            }
                          >
                            <TableCell className="font-mono text-xs text-muted-foreground">
                              #{r.row_index + 1}
                            </TableCell>
                            <TableCell className="text-xs">
                              <div className="font-medium text-foreground">
                                {r.parsed_name || (
                                  <span className="text-danger-text font-bold">
                                    (Missing Name)
                                  </span>
                                )}
                              </div>
                            </TableCell>
                            <TableCell className="text-xs font-mono text-muted-foreground">
                              {r.parsed_phone || "—"}
                            </TableCell>
                            <TableCell className="text-xs">
                              <div className="font-medium text-foreground">
                                {r.parsed_guardian_name || (
                                  <span className="text-danger-text">
                                    (Missing Guardian)
                                  </span>
                                )}
                              </div>
                              <div className="text-[11px] font-mono text-muted-foreground">
                                {r.parsed_guardian_phone || "—"}
                              </div>
                            </TableCell>
                            <TableCell className="text-xs">
                              {r.issues.length === 0 ? (
                                <span className="inline-flex items-center gap-1 text-[11px] text-success-text">
                                  <CheckCircle2 className="h-3 w-3" />
                                  Ready to import
                                </span>
                              ) : (
                                <div className="space-y-1">
                                  {r.issues.map((issue, idx) => (
                                    <div
                                      key={idx}
                                      className={`text-[11px] flex items-center gap-1 ${
                                        issue.severity === "error"
                                          ? "text-danger-text font-medium"
                                          : "text-warning-text"
                                      }`}
                                    >
                                      {issue.severity === "error" ? (
                                        <XCircle className="h-3 w-3 shrink-0" />
                                      ) : (
                                        <AlertTriangle className="h-3 w-3 shrink-0" />
                                      )}
                                      <span>{issue.message}</span>
                                    </div>
                                  ))}
                                </div>
                              )}
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                </div>
              </div>
            </div>
          )}

          {/* STEP 4: SUMMARY & REPORT */}
          {step === 4 && importSummary && (
            <div className="space-y-6 text-center py-4">
              <div className="h-16 w-16 rounded-full bg-success-subtle text-success-text flex items-center justify-center mx-auto shadow-xs">
                <CheckCircle2 className="h-10 w-10" />
              </div>

              <div>
                <h3 className="text-xl font-bold text-foreground">
                  Import Process Completed!
                </h3>
                <p className="text-xs text-muted-foreground mt-1 max-w-md mx-auto">
                  Student and guardian records have been committed to the database
                  atomically with full audit history.
                </p>
              </div>

              {/* Stat cards */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 max-w-2xl mx-auto text-left">
                <div className="p-3 rounded-[var(--radius-md)] border border-border bg-card">
                  <div className="text-[11px] text-muted-foreground">Total Rows</div>
                  <div className="text-xl font-bold text-foreground mt-0.5">
                    {importSummary.total_rows}
                  </div>
                </div>
                <div className="p-3 rounded-[var(--radius-md)] border border-success-border bg-success-subtle">
                  <div className="text-[11px] text-success-text font-medium">
                    Students Enrolled
                  </div>
                  <div className="text-xl font-bold text-success-text mt-0.5">
                    {importSummary.imported_students_count}
                  </div>
                </div>
                <div className="p-3 rounded-[var(--radius-md)] border border-border bg-card">
                  <div className="text-[11px] text-muted-foreground">
                    Guardians Linked
                  </div>
                  <div className="text-xl font-bold text-foreground mt-0.5">
                    {importSummary.imported_guardians_count}
                  </div>
                </div>
                <div className="p-3 rounded-[var(--radius-md)] border border-warning-border bg-warning-subtle">
                  <div className="text-[11px] text-warning-text font-medium">
                    Skipped Rows
                  </div>
                  <div className="text-xl font-bold text-warning-text mt-0.5">
                    {importSummary.skipped_errors_count}
                  </div>
                </div>
              </div>

              {/* Safety Snapshot Info */}
              <div className="rounded-[var(--radius-lg)] border border-border bg-muted/20 p-4 max-w-2xl mx-auto text-left space-y-2">
                <div className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                  <Database className="h-4 w-4 text-primary" />
                  Pre-Import Safety Backup Snapshot
                </div>
                <div className="flex items-center gap-2">
                  <span className="font-mono text-xs text-muted-foreground bg-card px-2.5 py-1.5 rounded border border-border flex-1 truncate">
                    {importSummary.backup_path}
                  </span>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => copyToClipboard(importSummary.backup_path, "backup")}
                    className="gap-1.5 shrink-0 cursor-pointer text-xs"
                  >
                    {copiedBackup ? (
                      <Check className="h-3.5 w-3.5 text-success-text" />
                    ) : (
                      <Copy className="h-3.5 w-3.5" />
                    )}
                    {copiedBackup ? "Copied" : "Copy"}
                  </Button>
                </div>
                <p className="text-[11px] text-muted-foreground">
                  An online SQLite database snapshot was captured prior to executing any
                  writes.
                </p>
              </div>

              {/* Rejected rows export */}
              {importSummary.rejected_rows_csv_path && (
                <div className="rounded-[var(--radius-lg)] border border-danger-border bg-danger-subtle/30 p-4 max-w-2xl mx-auto text-left space-y-2">
                  <div className="text-xs font-semibold text-danger-text flex items-center gap-1.5">
                    <FileText className="h-4 w-4" />
                    Rejected Rows CSV Available
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    {importSummary.skipped_errors_count} invalid row(s) were rejected. The
                    rejected rows file includes a detailed{" "}
                    <code className="text-foreground">_import_error</code> column
                    explaining each problem.
                  </p>
                  <div className="flex items-center gap-2 pt-1">
                    <span className="font-mono text-xs text-muted-foreground bg-card px-2.5 py-1.5 rounded border border-border flex-1 truncate">
                      {importSummary.rejected_rows_csv_path}
                    </span>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() =>
                        copyToClipboard(importSummary.rejected_rows_csv_path!, "rejected")
                      }
                      className="gap-1.5 shrink-0 cursor-pointer text-xs"
                    >
                      {copiedRejected ? (
                        <Check className="h-3.5 w-3.5 text-success-text" />
                      ) : (
                        <Copy className="h-3.5 w-3.5" />
                      )}
                      {copiedRejected ? "Copied" : "Copy Path"}
                    </Button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Wizard Navigation Footer */}
        <DialogFooter className="px-6 py-4 border-t border-border bg-muted/20 shrink-0 flex items-center justify-between gap-3">
          {step === 1 && (
            <div className="flex items-center justify-between w-full">
              <Button
                variant="outline"
                size="sm"
                onClick={() => onOpenChange(false)}
                className="cursor-pointer"
              >
                Cancel
              </Button>
              <Button
                disabled={!inspection || inspecting}
                size="sm"
                onClick={() => setStep(2)}
                className="gap-1.5 cursor-pointer"
              >
                Continue to Mapping
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          )}

          {step === 2 && (
            <div className="flex items-center justify-between w-full">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setStep(1)}
                className="gap-1.5 cursor-pointer"
              >
                <ArrowLeft className="h-4 w-4" />
                Back to File
              </Button>
              <Button
                disabled={!canProceedToPreview || validating}
                size="sm"
                onClick={runPreviewValidation}
                className="gap-1.5 cursor-pointer"
              >
                {validating && <Loader2 className="h-4 w-4 animate-spin" />}
                Preview Validation
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>
          )}

          {step === 3 && (
            <div className="flex items-center justify-between w-full">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setStep(2)}
                disabled={executing}
                className="gap-1.5 cursor-pointer"
              >
                <ArrowLeft className="h-4 w-4" />
                Back to Mapping
              </Button>
              <div className="flex items-center gap-2">
                <Button
                  disabled={
                    executing ||
                    (!skipErrorRows && (preview?.error_count ?? 0) > 0) ||
                    (preview?.valid_count ?? 0) === 0
                  }
                  size="sm"
                  onClick={handleExecuteImport}
                  className="gap-1.5 cursor-pointer"
                >
                  {executing ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <ShieldCheck className="h-4 w-4" />
                  )}
                  Execute Import (Atomic)
                </Button>
              </div>
            </div>
          )}

          {step === 4 && (
            <div className="flex items-center justify-end w-full">
              <Button
                size="sm"
                onClick={() => onOpenChange(false)}
                className="cursor-pointer"
              >
                Done
              </Button>
            </div>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
