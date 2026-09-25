import { describe, it, expect, beforeEach } from "vitest";
import {
  t,
  getLocale,
  setLocale,
  subscribeLocale,
  translations,
  toBengaliDigits,
  formatNumberLocale,
  formatCurrencyLocale,
  formatDateLocale,
  translateError,
} from "./i18n";

describe("i18n translation & formatting layer (Spec 0038)", () => {
  beforeEach(() => {
    setLocale("en");
  });

  it("maintains 100% dictionary key parity between English and Bangla", () => {
    const enKeys = Object.keys(translations.en).sort();
    const bnKeys = Object.keys(translations.bn).sort();
    expect(bnKeys).toEqual(enKeys);
    expect(enKeys.length).toBeGreaterThan(80);
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

  it("converts ASCII digits to Bengali numerals via toBengaliDigits", () => {
    expect(toBengaliDigits("125,000.50")).toBe("১২৫,০০০.৫০");
  });

  it("formats numbers with Bangladeshi Lakh/Crore grouping in English and Bangla", () => {
    setLocale("en");
    expect(formatNumberLocale(125000)).toBe("1,25,000");

    setLocale("bn");
    expect(formatNumberLocale(125000)).toBe("১,২৫,০০০");
  });

  it("formats poisha into Taka currency in English and Bangla", () => {
    // 12,500,000 poisha = 125,000.00 Taka
    setLocale("en");
    expect(formatCurrencyLocale(12500000)).toBe("৳ 1,25,000.00");

    setLocale("bn");
    expect(formatCurrencyLocale(12500000)).toBe("৳ ১,২৫,০০০.০০");
  });

  it("formats ISO dates in English and Bangla", () => {
    setLocale("en");
    expect(formatDateLocale("2026-09-25")).toBe("25 Sep 2026");

    setLocale("bn");
    expect(formatDateLocale("2026-09-25")).toBe("২৫ সেপ্টেম্বর ২০২৬");
  });

  it("translates core error codes with parameter interpolation", () => {
    setLocale("en");
    expect(translateError("FIN_OVERPAYMENT", "Overpayment", { amount: "৳500" })).toBe(
      "Payment exceeds total outstanding due by ৳500."
    );

    setLocale("bn");
    expect(translateError("FIN_OVERPAYMENT", "Overpayment", { amount: "৳৫০০" })).toBe(
      "প্রদত্ত অর্থ মোট বকেয়ার চেয়ে ৳৫০০ বেশি।"
    );
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
