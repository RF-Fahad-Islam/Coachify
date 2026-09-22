import { useState, useEffect, useCallback, type FormEvent } from "react";
import {
  rpc,
  type DriveInfo,
  type GenerateRecoveryPreviewResponse,
  type InitializeInstitutionPayload,
  type UserSummary,
} from "@/gen/rpc";
import {
  loadOnboardingDraft,
  saveOnboardingDraft,
  clearOnboardingDraft,
  type SetupStepId,
} from "@/lib/onboarding-storage";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { RecoverySheetPrintable } from "./recovery-sheet-printable";
import {
  Building2,
  HardDrive,
  ShieldCheck,
  Printer,
  Sparkles,
  ArrowRight,
  ArrowLeft,
  CheckCircle2,
  AlertTriangle,
  User,
  Calendar,
  Layers,
  Eye,
  EyeOff,
  RotateCcw,
} from "lucide-react";

export interface SetupWizardProps {
  onComplete: (user: UserSummary, sessionToken: string) => void;
}

export function SetupWizard({ onComplete }: SetupWizardProps) {
  // Load initial draft from persistent storage if available
  const [initialDraft] = useState(() => loadOnboardingDraft());
  const [hasResumedDraft, setHasResumedDraft] = useState(() => Boolean(initialDraft));
  const [draftSavedAt, setDraftSavedAt] = useState<string | null>(
    () => initialDraft?.lastSavedAt ?? null
  );

  const [currentStep, setCurrentStep] = useState<SetupStepId>(() =>
    initialDraft?.currentStep && initialDraft.currentStep !== "complete"
      ? initialDraft.currentStep
      : "profile"
  );

  // Form states
  // Step 1: Center Profile
  const [institutionName, setInstitutionName] = useState(
    () => initialDraft?.institutionName || ""
  );
  const [institutionNameBn, setInstitutionNameBn] = useState(
    () => initialDraft?.institutionNameBn || ""
  );
  const [address, setAddress] = useState(() => initialDraft?.address || "");
  const [phone, setPhone] = useState(() => initialDraft?.phone || "");
  const [email, setEmail] = useState(() => initialDraft?.email || "");
  const [academicSession, setAcademicSession] = useState(
    () => initialDraft?.academicSession || new Date().getFullYear().toString()
  );
  const [studentIdPrefix, setStudentIdPrefix] = useState(
    () => initialDraft?.studentIdPrefix || "S"
  );
  const [receiptPrefix, setReceiptPrefix] = useState(
    () => initialDraft?.receiptPrefix || "REC-"
  );

  // Step 2: Storage
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [selectedDataDir, setSelectedDataDir] = useState(
    () => initialDraft?.selectedDataDir || ""
  );
  const [customDataDir, setCustomDataDir] = useState(
    () => initialDraft?.customDataDir || ""
  );
  const [isNonSystemRecommended, setIsNonSystemRecommended] = useState(false);
  const [storageValid, setStorageValid] = useState(true);
  const [storageError, setStorageError] = useState<string | null>(null);

  // Step 3: Owner Account
  const [ownerDisplayName, setOwnerDisplayName] = useState(
    () => initialDraft?.ownerDisplayName || ""
  );
  const [ownerUsername, setOwnerUsername] = useState(
    () => initialDraft?.ownerUsername || "owner"
  );
  const [ownerPassword, setOwnerPassword] = useState(
    () => initialDraft?.ownerPassword || ""
  );
  const [ownerPasswordConfirm, setOwnerPasswordConfirm] = useState(
    () => initialDraft?.ownerPasswordConfirm || ""
  );
  const [showPassword, setShowPassword] = useState(false);

  // Step 4: Recovery Sheet
  const [recoveryPreview, setRecoveryPreview] =
    useState<GenerateRecoveryPreviewResponse | null>(null);
  const [confirmationInput, setConfirmationInput] = useState(
    () => initialDraft?.confirmationInput || ""
  );
  const [hasPrinted, setHasPrinted] = useState(() => initialDraft?.hasPrinted || false);

  // Step 5: Backup Target
  const [backupDestination, setBackupDestination] = useState(
    () => initialDraft?.backupDestination || ""
  );
  const [backupValid, setBackupValid] = useState(true);
  const [backupError, setBackupError] = useState<string | null>(null);

  // General state
  const [submitting, setSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [initSuccessData, setInitSuccessData] = useState<{
    user: UserSummary;
    token: string;
    institutionId: string;
  } | null>(null);

  // Auto-save draft on every change
  useEffect(() => {
    if (currentStep === "complete") return;
    saveOnboardingDraft({
      currentStep,
      institutionName,
      institutionNameBn,
      address,
      phone,
      email,
      academicSession,
      studentIdPrefix,
      receiptPrefix,
      selectedDataDir,
      customDataDir,
      ownerDisplayName,
      ownerUsername,
      ownerPassword,
      ownerPasswordConfirm,
      hasPrinted,
      confirmationInput,
      backupDestination,
    });
    setDraftSavedAt(new Date().toISOString());
  }, [
    currentStep,
    institutionName,
    institutionNameBn,
    address,
    phone,
    email,
    academicSession,
    studentIdPrefix,
    receiptPrefix,
    selectedDataDir,
    customDataDir,
    ownerDisplayName,
    ownerUsername,
    ownerPassword,
    ownerPasswordConfirm,
    hasPrinted,
    confirmationInput,
    backupDestination,
  ]);

  // Reset draft handler
  const handleResetDraft = () => {
    clearOnboardingDraft();
    setHasResumedDraft(false);
    setDraftSavedAt(null);
    setCurrentStep("profile");
    setInstitutionName("");
    setInstitutionNameBn("");
    setAddress("");
    setPhone("");
    setEmail("");
    setAcademicSession(new Date().getFullYear().toString());
    setStudentIdPrefix("S");
    setReceiptPrefix("REC-");
    setSelectedDataDir("");
    setCustomDataDir("");
    setOwnerDisplayName("");
    setOwnerUsername("owner");
    setOwnerPassword("");
    setOwnerPasswordConfirm("");
    setHasPrinted(false);
    setConfirmationInput("");
    setBackupDestination("");
  };

  // 1. Fetch drives & default path on mount
  useEffect(() => {
    async function loadDrivesAndPreview() {
      try {
        const driveRes = await rpc("setup.detect_drives", {});
        if (driveRes.ok && driveRes.data) {
          const data = driveRes.data;
          setDrives(data.drives);
          setSelectedDataDir((prev) => prev || data.recommended_path);
          setCustomDataDir((prev) => prev || data.recommended_path);
          setIsNonSystemRecommended(data.is_non_system_recommended);
        }

        const previewRes = await rpc("setup.generate_recovery_preview", {});
        if (previewRes.ok && previewRes.data) {
          setRecoveryPreview(previewRes.data);
        }
      } catch (err) {
        console.error("Failed to load initial setup data:", err);
      }
    }
    void loadDrivesAndPreview();
  }, []);

  // Validate data path when changed
  const validateDataPath = useCallback(async (path: string) => {
    if (!path.trim()) {
      setStorageValid(false);
      setStorageError("Storage path cannot be empty.");
      return;
    }
    try {
      const res = await rpc("setup.validate_path", {
        path: path.trim(),
        kind: "data",
      });
      if (res.ok && res.data) {
        setStorageValid(res.data.valid);
        setStorageError(res.data.error ?? null);
      }
    } catch {
      setStorageValid(true);
      setStorageError(null);
    }
  }, []);

  // Validate backup path when changed
  const validateBackupPath = useCallback(async (path: string) => {
    if (!path.trim()) {
      setBackupValid(true);
      setBackupError(null);
      return;
    }
    try {
      const res = await rpc("setup.validate_path", {
        path: path.trim(),
        kind: "backup",
      });
      if (res.ok && res.data) {
        setBackupValid(res.data.valid);
        setBackupError(res.data.error ?? null);
      }
    } catch {
      setBackupValid(true);
      setBackupError(null);
    }
  }, []);

  // Step 1 Validation
  const canProceedFromProfile =
    institutionName.trim().length > 0 && academicSession.trim().length > 0;

  // Step 2 Validation
  const canProceedFromStorage = storageValid && selectedDataDir.trim().length > 0;

  // Step 3 Validation
  const canProceedFromOwner =
    ownerDisplayName.trim().length > 0 &&
    ownerUsername.trim().length > 0 &&
    ownerPassword.length >= 8 &&
    ownerPassword === ownerPasswordConfirm;

  // Step 4 Validation (Gate: last 4 chars must match checksum exactly)
  const isConfirmationGatePassed =
    recoveryPreview !== null &&
    confirmationInput.trim().toUpperCase() ===
      recoveryPreview.checksum.trim().toUpperCase();

  // Final submission handler
  const handleFinalInitialize = async (e?: FormEvent) => {
    if (e) e.preventDefault();
    if (!recoveryPreview) return;

    setErrorMsg(null);
    setSubmitting(true);

    const payload: InitializeInstitutionPayload = {
      institution_name: institutionName.trim(),
      institution_name_bn: institutionNameBn.trim() || null,
      address: address.trim() || null,
      phone: phone.trim() || null,
      email: email.trim() || null,
      academic_session_name: academicSession.trim(),
      student_id_prefix: studentIdPrefix.trim() || null,
      receipt_prefix: receiptPrefix.trim() || null,
      data_dir: selectedDataDir.trim(),
      backup_destination: backupDestination.trim() || null,
      owner_display_name: ownerDisplayName.trim(),
      owner_username: ownerUsername.trim(),
      owner_password: ownerPassword,
      recovery_key: recoveryPreview.recovery_key,
      recovery_key_confirmation: confirmationInput.trim().toUpperCase(),
    };

    try {
      const res = await rpc("setup.initialize_institution", payload);
      if (res.ok && res.data) {
        clearOnboardingDraft();
        setInitSuccessData({
          user: res.data.user,
          token: res.data.session_token,
          institutionId: res.data.institution_id,
        });
        setCurrentStep("complete");
      } else {
        setErrorMsg(res.error?.message ?? "Institutional initialization failed.");
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const stepsList: { id: SetupStepId; label: string; number: number }[] = [
    { id: "profile", label: "Profile", number: 1 },
    { id: "storage", label: "Storage", number: 2 },
    { id: "owner", label: "Owner", number: 3 },
    { id: "recovery", label: "Recovery", number: 4 },
    { id: "backup", label: "Backup", number: 5 },
  ];

  const currentStepIndex = stepsList.findIndex((s) => s.id === currentStep);
  const currentStepNumber = currentStepIndex >= 0 ? currentStepIndex + 1 : 1;
  const currentStepObj = stepsList[currentStepIndex];

  return (
    <div className="min-h-screen w-full flex flex-col bg-background text-foreground select-none overflow-y-auto">
      {/* Top Banner Header */}
      <header className="border-b border-border/60 bg-card/80 backdrop-blur-md px-4 sm:px-6 py-3 sm:py-4 flex items-center justify-between sticky top-0 z-20 shadow-xs">
        <div className="flex items-center gap-3">
          <div className="h-9 w-9 rounded-xl bg-primary flex items-center justify-center text-primary-foreground shadow-sm">
            <Building2 className="h-5 w-5" />
          </div>
          <div>
            <h1 className="text-sm sm:text-base font-bold tracking-tight text-foreground">
              Coachify Setup Wizard
            </h1>
            <p className="text-[11px] sm:text-xs text-muted-foreground hidden xs:block">
              Offline Institutional Bootstrap &amp; Master Key Envelope
            </p>
          </div>
        </div>

        {/* Step Progression Indicators */}
        {currentStep !== "complete" && (
          <>
            {/* Desktop step pill chain */}
            <div className="hidden lg:flex items-center gap-2">
              {stepsList.map((st, idx) => {
                const isCurrent = currentStep === st.id;
                const isCompleted =
                  stepsList.findIndex((s) => s.id === currentStep) > idx;
                return (
                  <div key={st.id} className="flex items-center gap-2">
                    <div
                      className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold transition-all ${
                        isCurrent
                          ? "bg-primary text-primary-foreground shadow-sm"
                          : isCompleted
                            ? "bg-primary/20 text-primary"
                            : "bg-muted text-muted-foreground"
                      }`}
                    >
                      <span>{st.number}.</span>
                      <span>{st.label}</span>
                    </div>
                    {idx < stepsList.length - 1 && (
                      <div className="h-0.5 w-3 bg-border" />
                    )}
                  </div>
                );
              })}
            </div>

            {/* Mobile / Tablet compact step badge */}
            <div className="flex lg:hidden items-center gap-2">
              <span className="px-2.5 py-1 rounded-full bg-primary/10 text-primary border border-primary/20 text-xs font-bold">
                Step {currentStepNumber} of 5
              </span>
              <span className="text-xs font-semibold text-foreground hidden sm:inline">
                {currentStepObj?.label}
              </span>
            </div>
          </>
        )}

        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
          <span className="text-xs font-semibold text-muted-foreground hidden sm:inline">
            Offline First
          </span>
        </div>
      </header>

      {/* Mobile/Compact Step Progress Line */}
      {currentStep !== "complete" && (
        <div className="w-full bg-border/40 h-1 lg:hidden shrink-0">
          <div
            className="bg-primary h-1 transition-all duration-300"
            style={{ width: `${(currentStepNumber / 5) * 100}%` }}
          />
        </div>
      )}

      {/* Main Container - Scrollable without top vertical clipping */}
      <main className="flex-1 max-w-4xl w-full mx-auto p-4 sm:p-6 md:p-8 flex flex-col justify-start my-auto">
        {/* Resumed Draft Context Banner */}
        {hasResumedDraft && currentStep !== "complete" && (
          <div className="mb-4 sm:mb-6 p-3 sm:p-4 rounded-xl bg-primary/10 border border-primary/25 text-foreground flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 shadow-sm">
            <div className="flex items-center gap-2.5 text-xs sm:text-sm">
              <Sparkles className="h-4 w-4 text-primary shrink-0" />
              <span>
                <strong>Setup draft restored:</strong> Resumed from previous session
                {draftSavedAt ? (
                  <span className="text-muted-foreground">
                    {" "}
                    (saved{" "}
                    {new Date(draftSavedAt).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                    )
                  </span>
                ) : null}
                .
              </span>
            </div>
            <div className="flex items-center gap-2 self-end sm:self-auto">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleResetDraft}
                className="h-8 px-2.5 text-xs text-muted-foreground hover:text-destructive hover:bg-destructive/10 border-border/80"
              >
                <RotateCcw className="h-3.5 w-3.5 mr-1" />
                Start Fresh
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setHasResumedDraft(false)}
                className="h-8 px-2 text-xs text-muted-foreground hover:text-foreground"
                aria-label="Dismiss banner"
              >
                ✕
              </Button>
            </div>
          </div>
        )}

        {errorMsg && (
          <div className="mb-6 p-4 rounded-xl bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-3">
            <AlertTriangle className="h-5 w-5 shrink-0" />
            <span>{errorMsg}</span>
          </div>
        )}

        {/* STEP 1: Center Profile */}
        {currentStep === "profile" && (
          <Card className="border-border/80 shadow-lg shadow-black/5 rounded-2xl sm:rounded-3xl overflow-hidden">
            <CardHeader className="border-b border-border/40 pb-4">
              <div className="flex items-center gap-2 text-primary text-xs sm:text-sm font-bold">
                <Building2 className="h-4 w-4" />
                <span>Step 1 of 5</span>
              </div>
              <CardTitle className="text-xl sm:text-2xl font-black tracking-tight">
                Institution Profile
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                Enter your coaching academy or center details. These will appear on
                printed fee vouchers, reports, and student receipts.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 sm:space-y-5 pt-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Institution Name (English) *
                  </label>
                  <Input
                    placeholder="e.g. Dhaka Science Academy"
                    value={institutionName}
                    onChange={(e) => setInstitutionName(e.target.value)}
                    required
                    autoFocus
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground font-bangla">
                    নাম (বাংলায়)
                  </label>
                  <Input
                    placeholder="যেমন: ঢাকা সায়েন্স একাডেমি"
                    value={institutionNameBn}
                    onChange={(e) => setInstitutionNameBn(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm font-bangla leading-relaxed"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Address / Campus
                  </label>
                  <Input
                    placeholder="e.g. Sector 4, Uttara, Dhaka"
                    value={address}
                    onChange={(e) => setAddress(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Contact Phone
                  </label>
                  <Input
                    placeholder="e.g. 01711-000000"
                    value={phone}
                    onChange={(e) => setPhone(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Email Address
                  </label>
                  <Input
                    type="email"
                    placeholder="e.g. info@coaching.edu"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                    <Calendar className="h-3.5 w-3.5 text-muted-foreground" />
                    Academic Session *
                  </label>
                  <Input
                    placeholder="e.g. 2026"
                    value={academicSession}
                    onChange={(e) => setAcademicSession(e.target.value)}
                    required
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Student ID Prefix
                  </label>
                  <Input
                    placeholder="e.g. S or DSA-"
                    value={studentIdPrefix}
                    onChange={(e) => setStudentIdPrefix(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Receipt Prefix
                  </label>
                  <Input
                    placeholder="e.g. REC-"
                    value={receiptPrefix}
                    onChange={(e) => setReceiptPrefix(e.target.value)}
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
              </div>

              <div className="pt-4 flex justify-end">
                <Button
                  type="button"
                  disabled={!canProceedFromProfile}
                  onClick={() => setCurrentStep("storage")}
                  className="gap-2 h-11 px-6 rounded-xl font-bold shadow-md shadow-primary/20"
                >
                  Continue to Storage
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {/* STEP 2: Storage Location & Non-System Drive Recommendation */}
        {currentStep === "storage" && (
          <Card className="border-border/80 shadow-lg shadow-black/5 rounded-2xl sm:rounded-3xl overflow-hidden">
            <CardHeader className="border-b border-border/40 pb-4">
              <div className="flex items-center gap-2 text-primary text-xs sm:text-sm font-bold">
                <HardDrive className="h-4 w-4" />
                <span>Step 2 of 5</span>
              </div>
              <CardTitle className="text-xl sm:text-2xl font-black tracking-tight">
                Storage Drive &amp; Data Location
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                Choose where your Coachify database, encrypted documents, and archives are
                saved.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-5 pt-6">
              {/* Defense against Windows C: re-imaging notice */}
              <div className="p-3.5 sm:p-4 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-900 dark:text-amber-200 flex items-start gap-3">
                <ShieldCheck className="h-5 w-5 text-amber-600 shrink-0 mt-0.5" />
                <div className="text-xs sm:text-sm leading-relaxed">
                  <p className="font-bold">Protect against Windows OS Re-Imaging:</p>
                  <p className="text-muted-foreground mt-0.5">
                    Windows re-installations frequently format the{" "}
                    <code className="font-mono font-semibold">C:\</code> system drive. We
                    strongly recommend storing your institution data on a secondary
                    partition (e.g. <code className="font-mono font-semibold">D:\</code>{" "}
                    or <code className="font-mono font-semibold">E:\</code>) so student
                    and financial records remain safe if Windows crashes.
                  </p>
                </div>
              </div>

              {/* Detected Drives Grid */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-semibold text-foreground">
                    Detected Storage Partitions
                  </label>
                  {isNonSystemRecommended ? (
                    <span className="text-[11px] sm:text-xs text-emerald-600 dark:text-emerald-400 font-medium flex items-center gap-1">
                      <CheckCircle2 className="h-3.5 w-3.5" />
                      Secondary data drive pre-selected
                    </span>
                  ) : (
                    <span className="text-[11px] sm:text-xs text-muted-foreground">
                      Only system drive detected
                    </span>
                  )}
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {drives.map((d) => {
                    const isSelected = selectedDataDir
                      .toUpperCase()
                      .startsWith(d.letter.toUpperCase());
                    const freeGb = (d.free_bytes / (1024 * 1024 * 1024)).toFixed(1);
                    const totalGb = (d.total_bytes / (1024 * 1024 * 1024)).toFixed(1);

                    return (
                      <div
                        key={d.letter}
                        onClick={() => {
                          const newPath = `${d.letter}\\Coachify Data`;
                          setSelectedDataDir(newPath);
                          setCustomDataDir(newPath);
                          void validateDataPath(newPath);
                        }}
                        className={`p-3.5 sm:p-4 rounded-xl border cursor-pointer transition-all flex flex-col justify-between ${
                          isSelected
                            ? "border-primary bg-primary/5 ring-2 ring-primary/20 shadow-sm"
                            : "border-border hover:border-border/80 bg-card/40 hover:bg-card/70"
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <HardDrive
                              className={`h-4 w-4 ${isSelected ? "text-primary" : "text-muted-foreground"}`}
                            />
                            <span className="font-mono font-bold text-sm text-foreground">
                              {d.letter} ({d.label || "Local Disk"})
                            </span>
                          </div>
                          {d.is_recommended && (
                            <Badge
                              variant="default"
                              className="text-[10px] bg-emerald-600 hover:bg-emerald-600"
                            >
                              Recommended
                            </Badge>
                          )}
                          {d.is_system && (
                            <Badge
                              variant="outline"
                              className="text-[10px] text-muted-foreground"
                            >
                              System (C:)
                            </Badge>
                          )}
                        </div>
                        <div className="mt-2 text-xs text-muted-foreground flex justify-between">
                          <span>Free: {freeGb} GB</span>
                          <span>Total: {totalGb} GB</span>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Custom Directory Input */}
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  Exact Data Directory Path
                </label>
                <div className="flex gap-2">
                  <Input
                    value={customDataDir}
                    onChange={(e) => {
                      setCustomDataDir(e.target.value);
                      setSelectedDataDir(e.target.value);
                      void validateDataPath(e.target.value);
                    }}
                    placeholder="D:\Coachify Data"
                    className={`h-10 sm:h-11 rounded-xl text-xs sm:text-sm ${!storageValid ? "border-destructive" : ""}`}
                  />
                </div>
                {storageError ? (
                  <p className="text-xs text-destructive flex items-center gap-1 mt-1">
                    <AlertTriangle className="h-3.5 w-3.5" />
                    {storageError}
                  </p>
                ) : (
                  <p className="text-[11px] text-muted-foreground">
                    Institutional database, media vault, and daily backups will be
                    scaffolded here.
                  </p>
                )}
              </div>

              <div className="pt-4 flex justify-between">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setCurrentStep("profile")}
                  className="gap-2 h-11 px-5 rounded-xl font-semibold border-border/80"
                >
                  <ArrowLeft className="h-4 w-4" />
                  Back
                </Button>
                <Button
                  type="button"
                  disabled={!canProceedFromStorage}
                  onClick={() => setCurrentStep("owner")}
                  className="gap-2 h-11 px-6 rounded-xl font-bold shadow-md shadow-primary/20"
                >
                  Continue to Owner Account
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {/* STEP 3: Owner Account Credentials */}
        {currentStep === "owner" && (
          <Card className="border-border/80 shadow-lg shadow-black/5 rounded-2xl sm:rounded-3xl overflow-hidden">
            <CardHeader className="border-b border-border/40 pb-4">
              <div className="flex items-center gap-2 text-primary text-xs sm:text-sm font-bold">
                <User className="h-4 w-4" />
                <span>Step 3 of 5</span>
              </div>
              <CardTitle className="text-xl sm:text-2xl font-black tracking-tight">
                Principal / Owner Account
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                Set up the primary Administrator account. The Owner holds ultimate
                permissions for tuition fees, audit logs, and institutional recovery.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 sm:space-y-5 pt-6">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Full Display Name *
                  </label>
                  <Input
                    placeholder="e.g. Dr. Masud Rana"
                    value={ownerDisplayName}
                    onChange={(e) => setOwnerDisplayName(e.target.value)}
                    required
                    autoFocus
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Login Username *
                  </label>
                  <Input
                    placeholder="e.g. owner or principal"
                    value={ownerUsername}
                    onChange={(e) => setOwnerUsername(e.target.value)}
                    required
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-semibold text-foreground">
                      Master Password (Min 8 Characters) *
                    </label>
                    <button
                      type="button"
                      onClick={() => setShowPassword((prev) => !prev)}
                      className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1 cursor-pointer transition-colors"
                      tabIndex={-1}
                    >
                      {showPassword ? (
                        <EyeOff className="h-3.5 w-3.5" />
                      ) : (
                        <Eye className="h-3.5 w-3.5" />
                      )}
                      <span>{showPassword ? "Hide" : "Show"}</span>
                    </button>
                  </div>
                  <Input
                    type={showPassword ? "text" : "password"}
                    placeholder="••••••••••••"
                    value={ownerPassword}
                    onChange={(e) => setOwnerPassword(e.target.value)}
                    required
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground">
                    Confirm Password *
                  </label>
                  <Input
                    type={showPassword ? "text" : "password"}
                    placeholder="••••••••••••"
                    value={ownerPasswordConfirm}
                    onChange={(e) => setOwnerPasswordConfirm(e.target.value)}
                    required
                    className="h-10 sm:h-11 rounded-xl text-xs sm:text-sm"
                  />
                </div>
              </div>

              {ownerPassword && ownerPassword.length < 8 && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <AlertTriangle className="h-3.5 w-3.5" />
                  Password must be at least 8 characters long.
                </p>
              )}

              {ownerPasswordConfirm && ownerPassword !== ownerPasswordConfirm && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <AlertTriangle className="h-3.5 w-3.5" />
                  Passwords do not match.
                </p>
              )}

              <div className="pt-4 flex justify-between">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setCurrentStep("storage")}
                  className="gap-2 h-11 px-5 rounded-xl font-semibold border-border/80"
                >
                  <ArrowLeft className="h-4 w-4" />
                  Back
                </Button>
                <Button
                  type="button"
                  disabled={!canProceedFromOwner}
                  onClick={() => setCurrentStep("recovery")}
                  className="gap-2 h-11 px-6 rounded-xl font-bold shadow-md shadow-primary/20"
                >
                  Continue to Recovery Sheet
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {/* STEP 4: Printed Recovery Sheet & Mandatory Confirmation Gate */}
        {currentStep === "recovery" && (
          <Card className="border-border/80 shadow-lg shadow-black/5 rounded-2xl sm:rounded-3xl overflow-hidden">
            <CardHeader className="border-b border-border/40 pb-4">
              <div className="flex items-center gap-2 text-primary text-xs sm:text-sm font-bold">
                <Printer className="h-4 w-4" />
                <span>Step 4 of 5 (Mandatory Gate)</span>
              </div>
              <CardTitle className="text-xl sm:text-2xl font-black tracking-tight">
                Print Owner Recovery Sheet
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                To guarantee zero data loss, print this sheet now and store it in your
                physical safe. You must confirm the last 4 characters below to proceed.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6 pt-6">
              {recoveryPreview ? (
                <RecoverySheetPrintable
                  institutionName={institutionName}
                  ownerUsername={ownerUsername}
                  academicSession={academicSession}
                  createdAt={new Date().toISOString()}
                  recoveryKey={recoveryPreview.recovery_key}
                  checksum={recoveryPreview.checksum}
                  onPrintComplete={() => setHasPrinted(true)}
                />
              ) : (
                <div className="p-8 text-center text-xs text-muted-foreground animate-pulse">
                  Generating cryptographic recovery key...
                </div>
              )}

              {/* Mandatory Confirmation Gate Input */}
              <div className="p-4 sm:p-5 rounded-2xl bg-card border border-border/80 shadow-xs space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 text-xs sm:text-sm font-bold text-foreground">
                    <ShieldCheck className="h-4 w-4 text-primary" />
                    <span>Mandatory Security Confirmation Gate</span>
                  </div>
                  {hasPrinted && (
                    <Badge
                      variant="outline"
                      className="text-[10px] sm:text-xs text-emerald-600 dark:text-emerald-400 border-emerald-500/30 gap-1 font-semibold"
                    >
                      <CheckCircle2 className="h-3.5 w-3.5" />
                      Print Triggered
                    </Badge>
                  )}
                </div>
                <p className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                  Verify that you have written down or printed the sheet by typing the{" "}
                  <strong>last 4 characters</strong> of your Recovery Key (Block 6
                  Checksum):
                </p>

                <div className="flex flex-col sm:flex-row sm:items-center gap-3">
                  <Input
                    placeholder="e.g. 7M9K"
                    maxLength={4}
                    value={confirmationInput}
                    onChange={(e) => setConfirmationInput(e.target.value.toUpperCase())}
                    className="max-w-[150px] font-mono font-bold text-center tracking-widest text-base sm:text-lg h-11 rounded-xl"
                  />
                  {isConfirmationGatePassed ? (
                    <span className="text-xs sm:text-sm font-semibold text-emerald-600 dark:text-emerald-400 flex items-center gap-1.5 p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                      <CheckCircle2 className="h-4 w-4 shrink-0" />
                      Checksum Verified! Ready to proceed.
                    </span>
                  ) : confirmationInput.length === 4 ? (
                    <span className="text-xs sm:text-sm font-semibold text-destructive flex items-center gap-1.5 p-2 rounded-xl bg-destructive/10 border border-destructive/20">
                      <AlertTriangle className="h-4 w-4 shrink-0" />
                      Checksum mismatch. Look at the last block on your printed sheet.
                    </span>
                  ) : null}
                </div>
              </div>

              <div className="pt-2 flex justify-between">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setCurrentStep("owner")}
                  className="gap-2 h-11 px-5 rounded-xl font-semibold border-border/80"
                >
                  <ArrowLeft className="h-4 w-4" />
                  Back
                </Button>
                <Button
                  type="button"
                  disabled={!isConfirmationGatePassed}
                  onClick={() => setCurrentStep("backup")}
                  className="gap-2 h-11 px-6 rounded-xl font-bold shadow-md shadow-primary/20"
                >
                  Continue to Backup Target
                  <ArrowRight className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {/* STEP 5: Backup Target (Optional) */}
        {currentStep === "backup" && (
          <Card className="border-border/80 shadow-lg shadow-black/5 rounded-2xl sm:rounded-3xl overflow-hidden">
            <CardHeader className="border-b border-border/40 pb-4">
              <div className="flex items-center gap-2 text-primary text-xs sm:text-sm font-bold">
                <Layers className="h-4 w-4" />
                <span>Step 5 of 5</span>
              </div>
              <CardTitle className="text-xl sm:text-2xl font-black tracking-tight">
                External Backup Target (Optional)
              </CardTitle>
              <CardDescription className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
                Specify a secondary USB thumb drive or external folder for automatic daily
                snapshots.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-5 pt-6">
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  Backup Destination Path (Optional)
                </label>
                <Input
                  placeholder="e.g. E:\Coachify Backups"
                  value={backupDestination}
                  onChange={(e) => {
                    setBackupDestination(e.target.value);
                    void validateBackupPath(e.target.value);
                  }}
                  className={`h-10 sm:h-11 rounded-xl text-xs sm:text-sm ${!backupValid ? "border-destructive" : ""}`}
                />
                {backupError ? (
                  <p className="text-xs text-destructive flex items-center gap-1 mt-1">
                    <AlertTriangle className="h-3.5 w-3.5" />
                    {backupError}
                  </p>
                ) : (
                  <p className="text-[11px] text-muted-foreground">
                    Coachify will place a verified marker file (.coachify-backup-target)
                    to guard against accidental drive swaps.
                  </p>
                )}
              </div>

              {/* Summary Checklist before initialization */}
              <div className="p-4 sm:p-5 rounded-2xl bg-muted/40 border border-border/60 space-y-3 text-xs sm:text-sm">
                <span className="font-bold text-foreground block">
                  Configuration Summary
                </span>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-muted-foreground">
                  <div>
                    Academy:{" "}
                    <strong className="text-foreground">{institutionName}</strong>
                  </div>
                  <div>
                    Academic Session:{" "}
                    <strong className="text-foreground">{academicSession}</strong>
                  </div>
                  <div>
                    Data Directory:{" "}
                    <strong className="text-foreground font-mono">
                      {selectedDataDir}
                    </strong>
                  </div>
                  <div>
                    Owner:{" "}
                    <strong className="text-foreground">
                      {ownerDisplayName} ({ownerUsername})
                    </strong>
                  </div>
                  <div>
                    License:{" "}
                    <strong className="text-emerald-600 font-semibold">
                      30-Day Offline Trial
                    </strong>
                  </div>
                  <div>
                    Key Envelope:{" "}
                    <strong className="text-foreground font-mono">
                      DPAPI + Argon2id
                    </strong>
                  </div>
                </div>
              </div>

              <div className="pt-4 flex justify-between">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setCurrentStep("recovery")}
                  className="gap-2 h-11 px-5 rounded-xl font-semibold border-border/80"
                >
                  <ArrowLeft className="h-4 w-4" />
                  Back
                </Button>
                <Button
                  type="button"
                  disabled={submitting || !backupValid}
                  onClick={() => void handleFinalInitialize()}
                  className="gap-2 bg-primary text-primary-foreground hover:bg-primary-hover shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/30 font-bold px-7 h-12 rounded-xl text-sm"
                >
                  {submitting ? (
                    <>
                      <div className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                      Bootstrapping Institution...
                    </>
                  ) : (
                    <>
                      <Sparkles className="h-4 w-4" />
                      Initialize &amp; Start Trial
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {/* STEP 6: Completion / Celebration */}
        {currentStep === "complete" && initSuccessData && (
          <Card className="border-border/80 shadow-2xl shadow-primary/5 text-center p-6 sm:p-10 rounded-3xl max-w-xl mx-auto">
            <CardContent className="space-y-6 pt-4">
              <div className="h-16 w-16 rounded-2xl bg-emerald-500/10 text-emerald-600 mx-auto flex items-center justify-center border-2 border-emerald-500/20 shadow-sm">
                <CheckCircle2 className="h-8 w-8" />
              </div>

              <div className="space-y-2">
                <h2 className="text-2xl sm:text-3xl font-black tracking-tight text-foreground">
                  Welcome to Coachify!
                </h2>
                <p className="text-xs sm:text-sm text-muted-foreground max-w-md mx-auto leading-relaxed">
                  <strong>{institutionName}</strong> has been initialized successfully.
                  Your 30-day offline trial is now active, and your master key is sealed.
                </p>
              </div>

              <div className="p-4 sm:p-5 rounded-2xl bg-card border border-border/70 max-w-md mx-auto text-left text-xs sm:text-sm space-y-2.5 shadow-xs">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Institution ID:</span>
                  <span className="font-mono font-semibold">
                    {initSuccessData.institutionId}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Logged in as:</span>
                  <span className="font-semibold">
                    {initSuccessData.user.display_name}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Role:</span>
                  <Badge variant="default" className="text-xs">
                    Institute Owner
                  </Badge>
                </div>
              </div>

              <Button
                type="button"
                size="lg"
                onClick={() => onComplete(initSuccessData.user, initSuccessData.token)}
                className="gap-2 font-bold px-8 h-12 rounded-xl shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/30 text-sm"
              >
                Enter Coachify Dashboard
                <ArrowRight className="h-4 w-4" />
              </Button>
            </CardContent>
          </Card>
        )}
      </main>
    </div>
  );
}
