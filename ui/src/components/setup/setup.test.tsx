import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { RecoverySheetPrintable } from "./recovery-sheet-printable";
import { SetupWizard } from "./setup-wizard";
import { EmergencyRecoveryDialog } from "./emergency-recovery-dialog";
import { ReprintRecoveryDialog } from "../settings/reprint-recovery-dialog";

describe("Setup Wizard & Key Management Components", () => {
  it("renders RecoverySheetPrintable with key chunks, checksum, and bilingual instructions", () => {
    const recoveryKey = "CKREC-7M9K-2P4X-8R1N-5T6Y-9W3Z-ABCD";
    const checksum = "ABCD";

    const html = renderToString(
      <RecoverySheetPrintable
        institutionName="Uttara Model Academy"
        ownerUsername="principal_rahman"
        academicSession="2026-2027"
        createdAt="2026-09-20T12:00:00Z"
        recoveryKey={recoveryKey}
        checksum={checksum}
        showPrintButton={true}
      />
    );

    // Verify header and institution details
    expect(html).toContain("Uttara Model Academy");
    expect(html).toContain("principal_rahman");
    expect(html).toContain("2026-2027");
    expect(html).toContain("Physical Recovery Sheet");

    // Verify key chunks and checksum are rendered
    expect(html).toContain("CKREC");
    expect(html).toContain("7M9K");
    expect(html).toContain("ABCD");
    expect(html).toContain("Confirmation Checksum (Last 4)");

    // Verify security warnings in English and Bangla
    expect(html).toContain("IMPORTANT: NO CLOUD VENDOR ESCROW");
    expect(html).toContain("এই রিকভারি শীটটি সুরক্ষিত স্থানে রাখুন");
    expect(html).toContain("Print Recovery Sheet");
  });

  it("renders SetupWizard starting at Step 1 (Institution Profile)", () => {
    const html = renderToString(
      <ThemeProvider>
        <SetupWizard onComplete={() => {}} />
      </ThemeProvider>
    );

    expect(html).toContain("Coachify Setup Wizard");
    expect(html).toContain("Institution Profile");
    expect(html).toContain("Institution Name (English) *");
    expect(html).toContain("নাম (বাংলায়)");
    expect(html).toContain("Academic Session *");
    expect(html).toContain("Continue to Storage");
    expect(html).toContain("Offline First");
  });

  it("renders EmergencyRecoveryDialog without crashing", () => {
    const html = renderToString(
      <EmergencyRecoveryDialog
        open={true}
        onOpenChange={() => {}}
        onRecoverySuccess={() => {}}
      />
    );

    expect(html).toBeDefined();
  });

  it("renders ReprintRecoveryDialog without crashing", () => {
    const html = renderToString(
      <ReprintRecoveryDialog open={true} onOpenChange={() => {}} />
    );

    expect(html).toBeDefined();
  });

  it("renders SetupWizard with resumed draft notification when draft exists in storage", () => {
    class MockStorage implements Storage {
      private store: Record<string, string> = {};
      get length() {
        return Object.keys(this.store).length;
      }
      clear() {
        this.store = {};
      }
      getItem(key: string) {
        return this.store[key] ?? null;
      }
      key(index: number) {
        return Object.keys(this.store)[index] ?? null;
      }
      removeItem(key: string) {
        delete this.store[key];
      }
      setItem(key: string, value: string) {
        this.store[key] = value;
      }
    }

    const mockStorage = new MockStorage();
    (globalThis as unknown as { window?: { localStorage: Storage } }).window = {
      localStorage: mockStorage,
    };

    mockStorage.setItem(
      "coachify:setup_wizard_draft_v1",
      JSON.stringify({
        currentStep: "profile",
        institutionName: "Dhaka Science Academy",
        institutionNameBn: "ঢাকা সায়েন্স একাডেমি",
        academicSession: "2026",
        phone: "01711223344",
        email: "info@dhakascience.edu",
        address: "Dhanmondi, Dhaka",
        studentIdPrefix: "S",
        receiptPrefix: "REC-",
        selectedDataDir: "D:\\CoachifyData",
        customDataDir: "",
        ownerDisplayName: "Principal Ahmed",
        ownerUsername: "admin",
        ownerPassword: "SecretPassword123!",
        ownerPasswordConfirm: "SecretPassword123!",
        hasPrinted: false,
        confirmationInput: "",
        backupDestination: "",
        lastSavedAt: new Date().toISOString(),
      })
    );

    try {
      const html = renderToString(
        <ThemeProvider>
          <SetupWizard onComplete={() => {}} />
        </ThemeProvider>
      );

      expect(html).toContain("Setup draft restored");
      expect(html).toContain("Start Fresh");
      expect(html).toContain("Dhaka Science Academy");
    } finally {
      delete (globalThis as unknown as { window?: { localStorage: Storage } }).window;
    }
  });
});
