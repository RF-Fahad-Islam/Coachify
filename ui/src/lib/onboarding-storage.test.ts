import { describe, it, expect, beforeEach, afterAll } from "vitest";
import {
  loadOnboardingDraft,
  saveOnboardingDraft,
  clearOnboardingDraft,
  loadCarouselSlideIndex,
  saveCarouselSlideIndex,
  SETUP_WIZARD_DRAFT_KEY,
  ONBOARDING_CAROUSEL_SLIDE_KEY,
} from "./onboarding-storage";

class MockStorage {
  private store = new Map<string, string>();
  getItem(key: string): string | null {
    return this.store.get(key) ?? null;
  }
  setItem(key: string, value: string): void {
    this.store.set(key, String(value));
  }
  removeItem(key: string): void {
    this.store.delete(key);
  }
  clear(): void {
    this.store.clear();
  }
}

describe("onboarding-storage", () => {
  const originalWindow = globalThis.window;
  let mockStorage: MockStorage;

  beforeEach(() => {
    mockStorage = new MockStorage();
    (globalThis as unknown as { window: unknown }).window = {
      localStorage: mockStorage,
    };
  });

  afterAll(() => {
    (globalThis as unknown as { window: unknown }).window = originalWindow;
  });

  it("returns null when no draft is stored", () => {
    expect(loadOnboardingDraft()).toBeNull();
  });

  it("saves and loads complete draft state", () => {
    saveOnboardingDraft({
      currentStep: "owner",
      institutionName: "Dhaka Science Academy",
      institutionNameBn: "ঢাকা সায়েন্স একাডেমি",
      address: "Uttara, Sector 4",
      phone: "01711000000",
      email: "info@dsa.edu",
      academicSession: "2026",
      studentIdPrefix: "DSA-",
      receiptPrefix: "REC-2026-",
      selectedDataDir: "D:\\Coachify Data",
      customDataDir: "D:\\Coachify Data",
      ownerDisplayName: "Dr. Masud Rana",
      ownerUsername: "principal",
      ownerPassword: "SecretPassword123!",
      ownerPasswordConfirm: "SecretPassword123!",
      hasPrinted: true,
      confirmationInput: "7M9K",
      backupDestination: "E:\\Coachify Backups",
    });

    const loaded = loadOnboardingDraft();
    expect(loaded).not.toBeNull();
    expect(loaded?.currentStep).toBe("owner");
    expect(loaded?.institutionName).toBe("Dhaka Science Academy");
    expect(loaded?.institutionNameBn).toBe("ঢাকা সায়েন্স একাডেমি");
    expect(loaded?.address).toBe("Uttara, Sector 4");
    expect(loaded?.ownerDisplayName).toBe("Dr. Masud Rana");
    expect(loaded?.ownerUsername).toBe("principal");
    expect(loaded?.ownerPassword).toBe("SecretPassword123!");
    expect(loaded?.hasPrinted).toBe(true);
    expect(loaded?.confirmationInput).toBe("7M9K");
    expect(loaded?.backupDestination).toBe("E:\\Coachify Backups");
    expect(loaded?.lastSavedAt).toBeDefined();
  });

  it("merges partial updates cleanly into existing draft", () => {
    saveOnboardingDraft({
      institutionName: "Initial Academy",
      currentStep: "profile",
    });

    saveOnboardingDraft({
      currentStep: "storage",
      selectedDataDir: "D:\\Data",
    });

    const loaded = loadOnboardingDraft();
    expect(loaded?.institutionName).toBe("Initial Academy");
    expect(loaded?.currentStep).toBe("storage");
    expect(loaded?.selectedDataDir).toBe("D:\\Data");
  });

  it("handles corrupted JSON gracefully by returning null", () => {
    mockStorage.setItem(SETUP_WIZARD_DRAFT_KEY, "invalid-json{{}");
    expect(loadOnboardingDraft()).toBeNull();
  });

  it("clears draft completely when requested", () => {
    saveOnboardingDraft({ institutionName: "Test" });
    expect(loadOnboardingDraft()).not.toBeNull();

    clearOnboardingDraft();
    expect(loadOnboardingDraft()).toBeNull();
    expect(mockStorage.getItem(SETUP_WIZARD_DRAFT_KEY)).toBeNull();
  });

  it("saves and loads carousel slide index", () => {
    expect(loadCarouselSlideIndex()).toBe(0);

    saveCarouselSlideIndex(2);
    expect(loadCarouselSlideIndex()).toBe(2);

    mockStorage.setItem(ONBOARDING_CAROUSEL_SLIDE_KEY, "not-a-number");
    expect(loadCarouselSlideIndex()).toBe(0);
  });
});
