export type SetupStepId =
  "profile" | "storage" | "owner" | "recovery" | "backup" | "complete";

export interface SetupWizardDraftState {
  currentStep: SetupStepId;
  // Step 1: Center Profile
  institutionName: string;
  institutionNameBn: string;
  address: string;
  phone: string;
  email: string;
  academicSession: string;
  studentIdPrefix: string;
  receiptPrefix: string;
  // Step 2: Storage
  selectedDataDir: string;
  customDataDir: string;
  // Step 3: Owner Account
  ownerDisplayName: string;
  ownerUsername: string;
  ownerPassword?: string;
  ownerPasswordConfirm?: string;
  // Step 4: Recovery Sheet
  hasPrinted: boolean;
  confirmationInput: string;
  // Step 5: Backup Target
  backupDestination: string;
  // Metadata
  lastSavedAt: string;
}

export const SETUP_WIZARD_DRAFT_KEY = "coachify:setup_wizard_draft_v1";
export const ONBOARDING_CAROUSEL_SLIDE_KEY = "coachify:onboarding_slide_index";

const VALID_STEPS: ReadonlySet<SetupStepId> = new Set([
  "profile",
  "storage",
  "owner",
  "recovery",
  "backup",
  "complete",
]);

/**
 * Loads the saved setup wizard draft from local client storage.
 * Returns null if no draft exists, if storage is unavailable, or if the stored JSON is malformed.
 */
export function loadOnboardingDraft(): SetupWizardDraftState | null {
  if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
    return null;
  }

  try {
    const raw = window.localStorage.getItem(SETUP_WIZARD_DRAFT_KEY);
    if (!raw) return null;

    const parsed = JSON.parse(raw) as Partial<SetupWizardDraftState>;
    if (!parsed || typeof parsed !== "object") return null;

    // Validate currentStep
    const currentStep: SetupStepId =
      parsed.currentStep && VALID_STEPS.has(parsed.currentStep)
        ? parsed.currentStep
        : "profile";

    return {
      currentStep,
      institutionName:
        typeof parsed.institutionName === "string" ? parsed.institutionName : "",
      institutionNameBn:
        typeof parsed.institutionNameBn === "string" ? parsed.institutionNameBn : "",
      address: typeof parsed.address === "string" ? parsed.address : "",
      phone: typeof parsed.phone === "string" ? parsed.phone : "",
      email: typeof parsed.email === "string" ? parsed.email : "",
      academicSession:
        typeof parsed.academicSession === "string" &&
        parsed.academicSession.trim().length > 0
          ? parsed.academicSession
          : new Date().getFullYear().toString(),
      studentIdPrefix:
        typeof parsed.studentIdPrefix === "string" ? parsed.studentIdPrefix : "S",
      receiptPrefix:
        typeof parsed.receiptPrefix === "string" ? parsed.receiptPrefix : "REC-",
      selectedDataDir:
        typeof parsed.selectedDataDir === "string" ? parsed.selectedDataDir : "",
      customDataDir: typeof parsed.customDataDir === "string" ? parsed.customDataDir : "",
      ownerDisplayName:
        typeof parsed.ownerDisplayName === "string" ? parsed.ownerDisplayName : "",
      ownerUsername:
        typeof parsed.ownerUsername === "string" && parsed.ownerUsername.trim().length > 0
          ? parsed.ownerUsername
          : "owner",
      ownerPassword: typeof parsed.ownerPassword === "string" ? parsed.ownerPassword : "",
      ownerPasswordConfirm:
        typeof parsed.ownerPasswordConfirm === "string"
          ? parsed.ownerPasswordConfirm
          : "",
      hasPrinted: Boolean(parsed.hasPrinted),
      confirmationInput:
        typeof parsed.confirmationInput === "string" ? parsed.confirmationInput : "",
      backupDestination:
        typeof parsed.backupDestination === "string" ? parsed.backupDestination : "",
      lastSavedAt:
        typeof parsed.lastSavedAt === "string"
          ? parsed.lastSavedAt
          : new Date().toISOString(),
    };
  } catch {
    return null;
  }
}

/**
 * Persists changes to the setup wizard draft.
 * Automatically updates `lastSavedAt` with the current ISO timestamp.
 */
export function saveOnboardingDraft(updates: Partial<SetupWizardDraftState>): void {
  if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
    return;
  }

  try {
    const existing = loadOnboardingDraft();
    const merged: SetupWizardDraftState = {
      currentStep: updates.currentStep ?? existing?.currentStep ?? "profile",
      institutionName: updates.institutionName ?? existing?.institutionName ?? "",
      institutionNameBn: updates.institutionNameBn ?? existing?.institutionNameBn ?? "",
      address: updates.address ?? existing?.address ?? "",
      phone: updates.phone ?? existing?.phone ?? "",
      email: updates.email ?? existing?.email ?? "",
      academicSession:
        updates.academicSession ??
        existing?.academicSession ??
        new Date().getFullYear().toString(),
      studentIdPrefix: updates.studentIdPrefix ?? existing?.studentIdPrefix ?? "S",
      receiptPrefix: updates.receiptPrefix ?? existing?.receiptPrefix ?? "REC-",
      selectedDataDir: updates.selectedDataDir ?? existing?.selectedDataDir ?? "",
      customDataDir: updates.customDataDir ?? existing?.customDataDir ?? "",
      ownerDisplayName: updates.ownerDisplayName ?? existing?.ownerDisplayName ?? "",
      ownerUsername: updates.ownerUsername ?? existing?.ownerUsername ?? "owner",
      ownerPassword: updates.ownerPassword ?? existing?.ownerPassword ?? "",
      ownerPasswordConfirm:
        updates.ownerPasswordConfirm ?? existing?.ownerPasswordConfirm ?? "",
      hasPrinted: updates.hasPrinted ?? existing?.hasPrinted ?? false,
      confirmationInput: updates.confirmationInput ?? existing?.confirmationInput ?? "",
      backupDestination: updates.backupDestination ?? existing?.backupDestination ?? "",
      lastSavedAt: new Date().toISOString(),
    };

    window.localStorage.setItem(SETUP_WIZARD_DRAFT_KEY, JSON.stringify(merged));
  } catch {
    // Ignore storage quota or security errors gracefully
  }
}

/**
 * Permanently removes the setup wizard draft from storage.
 * Should be called when initialization completes successfully or when the user chooses "Start Fresh".
 */
export function clearOnboardingDraft(): void {
  if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
    return;
  }

  try {
    window.localStorage.removeItem(SETUP_WIZARD_DRAFT_KEY);
  } catch {
    // Ignore
  }
}

/**
 * Loads the last viewed carousel slide index.
 */
export function loadCarouselSlideIndex(): number {
  if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
    return 0;
  }

  try {
    const raw = window.localStorage.getItem(ONBOARDING_CAROUSEL_SLIDE_KEY);
    if (!raw) return 0;
    const parsed = parseInt(raw, 10);
    return Number.isFinite(parsed) && parsed >= 0 ? parsed : 0;
  } catch {
    return 0;
  }
}

/**
 * Persists the current carousel slide index.
 */
export function saveCarouselSlideIndex(index: number): void {
  if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(ONBOARDING_CAROUSEL_SLIDE_KEY, String(index));
  } catch {
    // Ignore
  }
}
