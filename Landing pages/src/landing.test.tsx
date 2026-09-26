import { describe, expect, it, beforeEach, vi } from "vitest";
import { createRoot } from "react-dom/client";
import { act } from "react";
import { MemoryRouter } from "react-router-dom";
import {
  CURRENCY,
  INSTITUTION,
  PAYMENTS,
  PRICING_PLANS,
  RELEASE_INFO,
  STUDENTS,
  bdt,
  bdtShort,
} from "./data/appData";
import {
  LANDING_LOCALE_STORAGE_KEY,
  LANDING_TRANSLATIONS,
  LandingLocaleProvider,
  toBengaliDigits,
} from "./lib/i18n";
import {
  LEADS_STORAGE_KEY,
  buildMailtoInquiryUrl,
  buildWhatsAppInquiryUrl,
  readStoredLeads,
  submitLeadRequest,
  validateContactString,
} from "./lib/leads";
import { CtaProvider } from "./components/shared";
import { App, SiteRoutes } from "./App";

(
  globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }
).IS_REACT_ACT_ENVIRONMENT = true;

if (typeof window !== "undefined") {
  (window as unknown as { __VITEST__?: boolean }).__VITEST__ = true;
  if (!window.requestAnimationFrame) {
    window.requestAnimationFrame = (cb) =>
      setTimeout(cb, 0) as unknown as number;
    window.cancelAnimationFrame = (id) => clearTimeout(id);
  }
}

function setVal(el: HTMLInputElement, val: string) {
  const setter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    "value",
  )?.set;
  setter?.call(el, val);
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

vi.mock("react-chartjs-2", () => ({
  Bar: () => <div data-testid="mock-bar-chart" />,
  Line: () => <div data-testid="mock-line-chart" />,
  Doughnut: () => <div data-testid="mock-doughnut-chart" />,
}));

describe("Coachify Slice 12 — Marketing & Commercial Landing Pages", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.scrollTo = vi.fn();
    (window as unknown as { __VITEST__?: boolean }).__VITEST__ = true;
  });

  it("enforces 100% key parity between English and Bangla dictionaries", () => {
    const enKeys = Object.keys(LANDING_TRANSLATIONS.en).sort();
    const bnKeys = Object.keys(LANDING_TRANSLATIONS.bn).sort();
    expect(bnKeys).toEqual(enKeys);
    for (const key of enKeys) {
      const k = key as keyof typeof LANDING_TRANSLATIONS.en;
      expect(LANDING_TRANSLATIONS.en[k].length).toBeGreaterThan(0);
      expect(LANDING_TRANSLATIONS.bn[k].length).toBeGreaterThan(0);
    }
    expect(toBengaliDigits("14900")).toBe("১৪৯০০");
  });

  it("localizes product mockup data and commercial pricing to Bangladesh coaching centers (৳ BDT)", () => {
    expect(CURRENCY).toBe("৳");
    expect(INSTITUTION.name).toBe("Atomic Coaching Center");
    expect(INSTITUTION.banglaName).toBe("অ্যাটমিক কোচিং সেন্টার");
    expect(PAYMENTS.cashier.modes).toEqual(["Cash", "bKash", "Nagad", "Bank"]);
    expect(STUDENTS.some((s) => s.batch.startsWith("HSC"))).toBe(true);
    expect(bdt(5000)).toBe("৳5,000.00");
    expect(bdtShort(14900)).toBe("৳14,900");

    expect(PRICING_PLANS).toHaveLength(4);
    expect(PRICING_PLANS.map((p) => p.price)).toEqual([
      "৳4,500",
      "৳7,200",
      "৳12,000",
      "৳50,000",
    ]);

    expect(RELEASE_INFO.version).toBe("v1.0.0");
    expect(RELEASE_INFO.sha256).toMatch(/^[a-f0-9]{64}$/);
  });

  it("validates Bangladesh mobile numbers and emails and persists lead requests", async () => {
    expect(validateContactString("")).toBeTruthy();
    expect(validateContactString("abc")).toBeTruthy();
    expect(validateContactString("01711-234567")).toBeNull();
    expect(validateContactString("+8801711234567")).toBeNull();
    expect(validateContactString("director@atomic.edu.bd")).toBeNull();

    const saved = await submitLeadRequest({
      name: "Tanvir Rahman",
      institution: "Atomic Coaching Center",
      contact: "01711-234567",
      interest: "12 Months Annual (৳12,000)",
      source: "modal",
    });

    expect(saved.id).toMatch(/^lead-/);
    const stored = readStoredLeads();
    expect(stored).toHaveLength(1);
    expect(stored[0]?.institution).toBe("Atomic Coaching Center");
    expect(window.localStorage.getItem(LEADS_STORAGE_KEY)).toContain(
      "Atomic Coaching Center",
    );

    expect(buildWhatsAppInquiryUrl(saved)).toContain(
      "https://wa.me/8801711234567?text=",
    );
    expect(buildMailtoInquiryUrl(saved)).toContain("mailto:sales@coachify.app");
  });

  it("renders the landing page application and toggles EN / বাংলা locale", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    await act(async () => {
      root.render(<App />);
    });

    expect(container.textContent).toContain("Your coaching");
    expect(container.textContent).toContain("৳4,500");
    expect(container.textContent).toContain("৳12,000");
    expect(container.textContent).toContain("৳50,000");

    const toggleBtn = container.querySelector(
      '[data-testid="landing-locale-toggle"]',
    ) as HTMLButtonElement | null;
    expect(toggleBtn).not.toBeNull();

    await act(async () => {
      toggleBtn?.click();
    });

    expect(window.localStorage.getItem(LANDING_LOCALE_STORAGE_KEY)).toBe("bn");
    expect(document.documentElement.lang).toBe("bn");
    expect(container.textContent).toContain("আপনার কোচিং");

    await act(async () => {
      root.unmount();
    });
    document.body.removeChild(container);
  });

  it("renders all marketing, portal, and downloads routes cleanly without errors", async () => {
    const routes = [
      "/",
      "/features",
      "/how-it-works",
      "/screens",
      "/offline-lan",
      "/pricing",
      "/download",
      "/downloads",
      "/portal",
      "/privacy",
      "/help",
      "/why-coachify",
      "/contact",
      "/faq",
    ];

    for (const path of routes) {
      const container = document.createElement("div");
      document.body.appendChild(container);
      const root = createRoot(container);

      await act(async () => {
        root.render(
          <MemoryRouter initialEntries={[path]}>
            <LandingLocaleProvider>
              <CtaProvider>
                <SiteRoutes />
              </CtaProvider>
            </LandingLocaleProvider>
          </MemoryRouter>,
        );
      });

      expect(container.textContent?.length).toBeGreaterThan(100);

      await act(async () => {
        root.unmount();
      });
      document.body.removeChild(container);
    }
  });

  it("discloses exact 5 activation fields, 2 update fields, and zero student PII on /privacy", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    await act(async () => {
      root.render(
        <MemoryRouter initialEntries={["/privacy"]}>
          <LandingLocaleProvider>
            <CtaProvider>
              <SiteRoutes />
            </CtaProvider>
          </LandingLocaleProvider>
        </MemoryRouter>,
      );
    });

    // 5 Activation/Refresh fields
    expect(container.textContent).toContain("installation_id");
    expect(container.textContent).toContain("license_id");
    expect(container.textContent).toContain("current_serial");
    expect(container.textContent).toContain("app_version");
    expect(container.textContent).toContain("active_students");

    // 2 Update fields
    expect(container.textContent).toContain("channel");
    expect(container.textContent).toContain("current_version");

    // Zero student PII guarantee
    expect(container.textContent).toContain("Zero Student PII Transmitted");

    await act(async () => {
      root.unmount();
    });
    document.body.removeChild(container);
  });

  it("renders customer portal tabs with verified installers and BDT invoices", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    await act(async () => {
      root.render(
        <MemoryRouter initialEntries={["/portal"]}>
          <LandingLocaleProvider>
            <CtaProvider>
              <SiteRoutes />
            </CtaProvider>
          </LandingLocaleProvider>
        </MemoryRouter>,
      );
    });

    expect(container.textContent).toContain("Atomic Coaching Center");
    expect(container.textContent).toContain("COACH-PRO-2026-ATOMIC");
    expect(container.textContent).toContain(".coachifylicense");

    // Check tabs
    expect(container.textContent).toContain("Verified Installers");
    expect(container.textContent).toContain("Invoices & Receipts");
    expect(container.textContent).toContain("Linked PCs");

    await act(async () => {
      root.unmount();
    });
    document.body.removeChild(container);
  });

  it("renders interactive online Checkout Modal with bKash/Nagad and Bank Transfer flow (Spec 0043a AC-1, AC-10)", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    await act(async () => {
      root.render(
        <MemoryRouter initialEntries={["/"]}>
          <LandingLocaleProvider>
            <CtaProvider>
              <SiteRoutes />
            </CtaProvider>
          </LandingLocaleProvider>
        </MemoryRouter>,
      );
    });

    // Find and click one of the paid tier checkout buttons
    const buttons = Array.from(container.querySelectorAll("button"));
    const orderBtn = buttons.find(
      (b) =>
        b.textContent?.includes("Order 12 Months") ||
        b.textContent?.includes("Order 3 Months"),
    );
    expect(orderBtn).toBeDefined();

    await act(async () => {
      orderBtn?.click();
    });

    // Checkout modal is visible
    expect(container.textContent).toContain("Online Plan Purchase & Renewal");
    expect(container.textContent).toContain("Payment Method");
    expect(container.textContent).toContain("bKash Direct");
    expect(container.textContent).toContain("Bank Transfer");
    expect(container.textContent).toContain("Total Payable");

    // Fill in required checkout fields
    const inputs = Array.from(container.querySelectorAll("input"));
    const orgInput = inputs.find((i) =>
      i.placeholder.includes("Dhanmondi Science Academy"),
    );
    const ownerInput = inputs.find((i) =>
      i.placeholder.includes("Rafiqul Islam"),
    );
    const phoneInput = inputs.find((i) =>
      i.placeholder.includes("01711-000000"),
    );

    expect(orgInput).toBeDefined();
    expect(ownerInput).toBeDefined();
    expect(phoneInput).toBeDefined();

    await act(async () => {
      if (orgInput) setVal(orgInput, "Mirpur Academic Care");
      if (ownerInput) setVal(ownerInput, "Tariqul Islam");
      if (phoneInput) setVal(phoneInput, "01711-001122");
    });

    // Submit checkout form
    const submitBtn = Array.from(container.querySelectorAll("button")).find(
      (b) => b.type === "submit" && b.textContent?.includes("Pay ৳"),
    );
    expect(submitBtn).toBeDefined();

    await act(async () => {
      submitBtn?.click();
    });

    // Verify successful fulfillment screen with pairing code and invoice
    expect(container.textContent).toContain(
      "Payment Completed & License Issued",
    );
    expect(container.textContent).toContain(
      "Your 6-Digit Desktop Pairing Code",
    );
    expect(container.textContent).toContain("Download License");

    await act(async () => {
      root.unmount();
    });
    document.body.removeChild(container);
  });

  it("manages linked PC devices, OTP-guarded deactivation, replacement, and 3-transfer cap in portal (Spec 0043b AC-6, AC-7, AC-8, AC-10)", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    await act(async () => {
      root.render(
        <MemoryRouter initialEntries={["/portal"]}>
          <LandingLocaleProvider>
            <CtaProvider>
              <SiteRoutes />
            </CtaProvider>
          </LandingLocaleProvider>
        </MemoryRouter>,
      );
    });

    // Switch to Linked PCs tab
    const linkedPcsTab = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent?.includes("Linked PCs"),
    );
    expect(linkedPcsTab).toBeDefined();

    await act(async () => {
      linkedPcsTab?.click();
    });

    // Check device management studio renders
    expect(container.textContent).toContain("PC Device Management Studio");
    expect(container.textContent).toContain("Transfers Used:");
    expect(container.textContent).toContain("1 of 3 / 12m");
    expect(container.textContent).toContain(
      "DESKTOP-FRONTDESK (Main Reception PC)",
    );
    expect(container.textContent).toContain("DESKTOP-ACAD-01");
    expect(container.textContent).toContain("Windows 11 Pro 64-bit");

    // Click Deactivate button
    const deactBtn = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent === "Deactivate",
    );
    expect(deactBtn).toBeDefined();

    await act(async () => {
      deactBtn?.click();
    });

    // Verify Step-up OTP prompt and immediate read-only warning
    expect(container.textContent).toContain("Deactivate PC Installation");
    expect(container.textContent).toContain("Read-Only mode");
    expect(container.textContent).toContain("6-Digit Step-Up OTP Code");

    // Enter valid demo OTP (123456)
    const otpInput = container.querySelector(
      'input[placeholder*="Enter 6-digit OTP"]',
    ) as HTMLInputElement | null;
    expect(otpInput).not.toBeNull();

    await act(async () => {
      if (otpInput) {
        setVal(otpInput, "123456");
      }
    });

    const confirmDeactBtn = Array.from(
      container.querySelectorAll("button"),
    ).find((b) => b.textContent?.includes("Confirm Deactivation"));
    expect(confirmDeactBtn).toBeDefined();

    await act(async () => {
      confirmDeactBtn?.click();
    });

    // Device should now show deactivated and transfer counter incremented to 2 of 3
    expect(container.textContent).toContain("Deactivated (Read-Only)");
    expect(container.textContent).toContain("2 of 3 / 12m");
    expect(container.textContent).toContain(
      "Fresh Replacement Pairing Code Generated",
    );

    await act(async () => {
      root.unmount();
    });
    document.body.removeChild(container);
  });
});
