import { describe, it, expect, beforeEach } from "vitest";
import { t, getLocale, setLocale, subscribeLocale } from "./i18n";

describe("i18n translation layer", () => {
  beforeEach(() => {
    setLocale("en");
  });

  it("translates navigation labels in English", () => {
    expect(t("nav.dashboard")).toBe("Dashboard");
    expect(t("nav.students")).toBe("Students");
    expect(t("nav.payments")).toBe("Finance & Fees");
  });

  it("translates navigation labels in Bangla when locale is set to bn", () => {
    setLocale("bn");
    expect(getLocale()).toBe("bn");
    expect(t("nav.dashboard")).toBe("ড্যাশবোর্ড");
    expect(t("nav.students")).toBe("শিক্ষার্থীবৃন্দ");
    expect(t("nav.payments")).toBe("অর্থ ও ফি হিসাব");
  });

  it("falls back to provided fallback string or key if key is missing", () => {
    expect(t("unknown.key", "Fallback Value")).toBe("Fallback Value");
    expect(t("another.missing.key")).toBe("another.missing.key");
  });

  it("interpolates parameters into translated strings", () => {
    expect(
      t("greeting", "Hello {name}, you have {count} items", { name: "Rahim", count: 5 })
    ).toBe("Hello Rahim, you have 5 items");
  });

  it("notifies subscribers when locale changes", () => {
    let notifiedLocale = "";
    const unsubscribe = subscribeLocale((newLocale) => {
      notifiedLocale = newLocale;
    });

    setLocale("bn");
    expect(notifiedLocale).toBe("bn");

    unsubscribe();
    setLocale("en");
    expect(notifiedLocale).toBe("bn"); // Did not change after unsubscribe
  });
});
