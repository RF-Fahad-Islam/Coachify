import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type LandingLocale = "en" | "bn";

export const LANDING_LOCALE_STORAGE_KEY = "coachify.landing.locale";

const BENGALI_DIGITS = [
  "০",
  "১",
  "২",
  "৩",
  "৪",
  "৫",
  "৬",
  "৭",
  "৮",
  "৯",
] as const;

export function toBengaliDigits(input: string | number): string {
  return String(input).replace(/[0-9]/g, (d) => BENGALI_DIGITS[Number(d)] ?? d);
}

export const LANDING_TRANSLATIONS = {
  en: {
    "nav.desktopBadge": "Desktop · Bangladesh",
    "nav.features": "Features",
    "nav.howItWorks": "How It Works",
    "nav.screens": "Screens",
    "nav.pricing": "Pricing",
    "nav.download": "Download",
    "nav.help": "Help",
    "nav.offlineLan": "Offline & LAN",
    "nav.whyCoachify": "Why Coachify",
    "nav.faq": "FAQ",
    "nav.contact": "Contact",
    "nav.portal": "Customer Portal",
    "nav.privacy": "Privacy",
    "nav.getCoachify": "Get Coachify",
    "nav.downloadDemo": "Download Demo",
    "nav.langToggleLabel": "Switch language to Bangla",

    "hero.eyebrow": "OFFLINE-FIRST DESKTOP SOFTWARE FOR COACHING CENTERS",
    "hero.line1": "Your coaching",
    "hero.line2Prefix": "center,",
    "hero.line2Highlight": "finally",
    "hero.line3": "organized.",
    "hero.subtitle":
      "Admissions, students, attendance, BDT fees, bKash/Nagad receipts, exams and results — in one fast desktop app built for Bangladeshi institutes. Runs on your own PC, even when the internet is down.",
    "hero.downloadDemo": "Download Demo",
    "hero.requestFull": "Request Full Version",
    "hero.trialNote":
      "Free 30-day demo · Atomic Coaching dataset included · Installs in 5 minutes",
    "hero.badgeOffline": "100% offline capable",
    "hero.badgeWindows": "Runs on Windows 10 / 11",
    "hero.badgeInstall": "5-minute install",

    "pricing.tag": "Simple, one-time BDT pricing",
    "pricing.titlePrefix": "No per-student fees.",
    "pricing.titleHighlight": "No monthly surprises.",
    "pricing.subtitle":
      "Pay once in BDT (৳), own it forever. Your coaching center's records stay on your PC — never locked behind a monthly cloud subscription.",
    "pricing.mostPopular": "Most popular",
    "pricing.footerNote":
      "All editions run 100% offline with Bangla & English support. bKash, Nagad, and bank transfer accepted for license activation.",

    "modal.download.title": "Windows Desktop Demo Installer",
    "modal.download.subtitle":
      "Windows 10 / 11 (64-bit) · Atomic Coaching demo preloaded",
    "modal.download.cta": "Download CoachifyDemoSetup.exe",
    "modal.download.downloading": "Preparing installer package…",
    "modal.download.checksumLabel": "SHA-256 Checksum",
    "modal.download.copyChecksum": "Copy SHA-256",
    "modal.download.copied": "Copied!",
    "modal.download.directLink": "Download again",

    "modal.request.title": "Set up Coachify at your center",
    "modal.request.subtitle":
      "Tell us about your coaching center — our Dhaka onboarding team will send your license quote or schedule a live setup.",
    "modal.request.submit": "Submit license request",
    "modal.request.receivedTitle": "Request received",
    "modal.request.receivedSubtitle":
      "We have saved your request and our team will contact you via WhatsApp or phone within one business day.",

    "modal.checkout.title": "Online Checkout & Instant Activation",
    "modal.checkout.subtitle":
      "Pick your payment method (bKash, Nagad, or Bank Transfer) to receive your sequential invoice and license instantly.",
    "modal.checkout.payNow": "Complete Payment & Issue License",
    "modal.checkout.bankSubmit": "Submit Bank Slip for Verification",
  },
  bn: {
    "nav.desktopBadge": "ডেস্কটপ · বাংলাদেশ",
    "nav.features": "ফিচারসমূহ",
    "nav.howItWorks": "কীভাবে কাজ করে",
    "nav.screens": "স্ক্রিনসমূহ",
    "nav.pricing": "মূল্য তালিকা",
    "nav.download": "ডাউনলোড",
    "nav.help": "সহায়তা",
    "nav.offlineLan": "অফলাইন ও ল্যান",
    "nav.whyCoachify": "কেন কোচিফাই",
    "nav.faq": "প্রশ্নোত্তর",
    "nav.contact": "যোগাযোগ",
    "nav.portal": "কাস্টমার পোর্টাল",
    "nav.privacy": "গোপনীয়তা",
    "nav.getCoachify": "লাইসেন্স নিন",
    "nav.downloadDemo": "ডেমো ডাউনলোড",
    "nav.langToggleLabel": "Switch language to English",

    "hero.eyebrow": "কোচিং সেন্টারের জন্য অফলাইন-ফার্স্ট ডেস্কটপ সফটওয়্যার",
    "hero.line1": "আপনার কোচিং",
    "hero.line2Prefix": "সেন্টার,",
    "hero.line2Highlight": "এখন সম্পূর্ণ",
    "hero.line3": "সুসংগঠিত।",
    "hero.subtitle":
      "ভর্তি, শিক্ষার্থী, হাজিরা, মাসিক ফি (৳), বিকাশ/নগদ রসিদ, পরীক্ষা ও ফলাফল — বাংলাদেশের কোচিং সেন্টারের জন্য তৈরি দ্রুতগতির ডেস্কটপ অ্যাপ। ইন্টারনেট না থাকলেও আপনার পিসিতে নির্বিঘ্নে চলে।",
    "hero.downloadDemo": "ডেমো ডাউনলোড করুন",
    "hero.requestFull": "ফুল ভার্সন অর্ডার করুন",
    "hero.trialNote":
      "৩০ দিনের ফ্রি ডেমো · অ্যাটমিক কোচিং ডেমো ডেটা সহ · ৫ মিনিটে ইনস্টল",
    "hero.badgeOffline": "১০০% অফলাইনে চলে",
    "hero.badgeWindows": "Windows 10 / 11 সাপোর্ট",
    "hero.badgeInstall": "৫ মিনিটে ইনস্টল",

    "pricing.tag": "সহজ এককালীন মূল্য (৳ BDT)",
    "pricing.titlePrefix": "শিক্ষার্থী প্রতি কোনো চার্জ নেই।",
    "pricing.titleHighlight": "নেই মাসিক সাবস্ক্রিপশন।",
    "pricing.subtitle":
      "একবার পরিশোধ করুন, আজীবন ব্যবহার করুন। আপনার প্রতিষ্ঠানের সব তথ্য আপনার কম্পিউটারেই নিরাপদ থাকে।",
    "pricing.mostPopular": "সবচেয়ে জনপ্রিয়",
    "pricing.footerNote":
      "সব ভার্সন ১০০% অফলাইনে চলে এবং বাংলা ও ইংরেজি সাপোর্ট করে। বিকাশ, নগদ ও ব্যাংকের মাধ্যমে লাইসেন্স পেমেন্ট করা যায়।",

    "modal.download.title": "উইন্ডোজ ডেস্কটপ ডেমো ইনস্টলার",
    "modal.download.subtitle":
      "Windows 10 / 11 (64-bit) · অ্যাটমিক কোচিং ডেমো ডেটা সংযুক্ত",
    "modal.download.cta": "CoachifyDemoSetup.exe ডাউনলোড করুন",
    "modal.download.downloading": "ইনস্টলার প্রস্তুত হচ্ছে…",
    "modal.download.checksumLabel": "SHA-256 চেকসাম",
    "modal.download.copyChecksum": "কপি করুন",
    "modal.download.copied": "কপি হয়েছে!",
    "modal.download.directLink": "পুনরায় ডাউনলোড করুন",

    "modal.request.title": "আপনার কোচিং সেন্টার সেটআপ করুন",
    "modal.request.subtitle":
      "আপনার প্রতিষ্ঠানের তথ্য দিন — আমাদের টিম দ্রুত লাইসেন্স বা লাইভ ডেমোর জন্য যোগাযোগ করবে।",
    "modal.request.submit": "রিকোয়েস্ট পাঠান",
    "modal.request.receivedTitle": "রিকোয়েস্ট গৃহীত হয়েছে",
    "modal.request.receivedSubtitle":
      "আপনার তথ্য সংরক্ষিত হয়েছে। ১ কর্মদিবসের মধ্যে আমাদের প্রতিনিধি আপনার হোয়াটসঅ্যাপ বা ফোনে যোগাযোগ করবেন।",

    "modal.checkout.title": "অনলাইন পেমেন্ট ও তাত্ক্ষণিক লাইসেন্স অ্যাক্টিভেশন",
    "modal.checkout.subtitle":
      "বিকাশ, নগদ বা ব্যাংক ট্রান্সফারের মাধ্যমে সহজেই পেমেন্ট করে চালান ও লাইসেন্স সংগ্রহ করুন।",
    "modal.checkout.payNow": "পেমেন্ট সম্পন্ন করুন এবং লাইসেন্স নিন",
    "modal.checkout.bankSubmit": "ব্যাংক স্লিপ ভেরিফিকেশনের জন্য জমা দিন",
  },
} as const;

export type LandingTranslationKey = keyof typeof LANDING_TRANSLATIONS.en;

interface LandingLocaleContextValue {
  locale: LandingLocale;
  setLocale: (next: LandingLocale) => void;
  toggleLocale: () => void;
  t: (key: LandingTranslationKey) => string;
}

function readInitialLocale(): LandingLocale {
  if (typeof window === "undefined") return "en";
  try {
    const saved = window.localStorage.getItem(LANDING_LOCALE_STORAGE_KEY);
    if (saved === "bn" || saved === "en") return saved;
  } catch {
    // Ignore storage access errors in restricted environments
  }
  return "en";
}

const LandingLocaleContext = createContext<LandingLocaleContextValue>({
  locale: "en",
  setLocale: () => {},
  toggleLocale: () => {},
  t: (key) => LANDING_TRANSLATIONS.en[key],
});

export function LandingLocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<LandingLocale>(readInitialLocale);

  const setLocale = useCallback((next: LandingLocale) => {
    setLocaleState(next);
    try {
      window.localStorage.setItem(LANDING_LOCALE_STORAGE_KEY, next);
    } catch {
      // Ignore storage access errors
    }
    if (typeof document !== "undefined") {
      document.documentElement.lang = next;
    }
  }, []);

  const toggleLocale = useCallback(() => {
    setLocale(locale === "en" ? "bn" : "en");
  }, [locale, setLocale]);

  useEffect(() => {
    if (typeof document !== "undefined") {
      document.documentElement.lang = locale;
    }
  }, [locale]);

  const t = useCallback(
    (key: LandingTranslationKey): string => {
      return LANDING_TRANSLATIONS[locale][key] ?? LANDING_TRANSLATIONS.en[key];
    },
    [locale],
  );

  const value = useMemo(
    () => ({ locale, setLocale, toggleLocale, t }),
    [locale, setLocale, toggleLocale, t],
  );

  return (
    <LandingLocaleContext.Provider value={value}>
      {children}
    </LandingLocaleContext.Provider>
  );
}

export function useLandingLocale(): LandingLocaleContextValue {
  return useContext(LandingLocaleContext);
}
