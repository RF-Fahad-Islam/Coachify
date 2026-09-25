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

vi.mock("react-chartjs-2", () => ({
  Bar: () => <div data-testid="mock-bar-chart" />,
  Line: () => <div data-testid="mock-line-chart" />,
  Doughnut: () => <div data-testid="mock-doughnut-chart" />,
}));

describe("Coachify Slice 12 — Marketing & Commercial Landing Pages", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.scrollTo = vi.fn();
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

    expect(PRICING_PLANS).toHaveLength(3);
    expect(PRICING_PLANS.map((p) => p.price)).toEqual([
      "Free",
      "৳14,900",
      "৳29,900",
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
      interest: "Full Version (৳14,900)",
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
    expect(container.textContent).toContain("৳14,900");
    expect(container.textContent).toContain("৳29,900");

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

  it("renders all 11 marketing routes cleanly without errors", async () => {
    const routes = [
      "/",
      "/features",
      "/how-it-works",
      "/screens",
      "/offline-lan",
      "/pricing",
      "/download",
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
});
