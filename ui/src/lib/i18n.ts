import { useSyncExternalStore } from "react";

export type SupportedLocale = "en" | "bn";

const LOCALE_STORAGE_KEY = "coachify.locale";

function readInitialLocale(): SupportedLocale {
  if (typeof window !== "undefined" && window.localStorage) {
    const saved = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    if (saved === "bn" || saved === "en") {
      return saved;
    }
  }
  return "en";
}

let currentLocale: SupportedLocale = readInitialLocale();
const listeners: Array<(locale: SupportedLocale) => void> = [];

const BENGALI_DIGITS = ["০", "১", "২", "৩", "৪", "৫", "৬", "৭", "৮", "৯"] as const;

const EN_MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
] as const;

const BN_MONTHS = [
  "জানুয়ারি",
  "ফেব্রুয়ারি",
  "মার্চ",
  "এপ্রিল",
  "মে",
  "জুন",
  "জুলাই",
  "আগস্ট",
  "সেপ্টেম্বর",
  "অক্টোবর",
  "নভেম্বর",
  "ডিসেম্বর",
] as const;

export const translations: Record<SupportedLocale, Record<string, string>> = {
  en: {
    // Navigation & Sidebar
    "nav.dashboard": "Dashboard",
    "nav.intelligence": "Owner Intelligence",
    "nav.students": "Students",
    "nav.academic": "Academic Setup",
    "nav.staff": "Staff & Teachers",
    "nav.attendance": "Attendance",
    "nav.payments": "Finance & Fees",
    "nav.cashier": "Cashier Desk",
    "nav.dues": "Dues Center",
    "nav.cashClose": "Expenses & Close",
    "nav.feeSetup": "Fee Setup",
    "nav.communication": "Communication",
    "nav.exams": "Exams & Marks",
    "nav.reports": "Reports",
    "nav.integrity": "Integrity & Safety",
    "nav.backup": "Backup & Safety",
    "nav.diagnostics": "Diagnostics",
    "nav.settings": "Settings",
    "nav.group.operations": "Daily Operations",
    "nav.group.academics": "People & Academics",
    "nav.group.finance": "Finance & Ledger",
    "nav.group.system": "System & Safety",

    // Top Bar & Shell
    "topbar.searchPlaceholder": "Search students, payments, receipts... (Ctrl+K)",
    "topbar.commandPalette": "Command Palette (Ctrl+K)",
    "topbar.languageToggle": "বাংলা",
    "topbar.languageCurrent": "English",
    "topbar.viewAsRole": "View As Role",
    "topbar.lockScreen": "Lock Screen",
    "topbar.logout": "Sign Out",
    "topbar.aiCopilot": "AI Copilot",
    "topbar.readOnlyBanner":
      "Read-Only Mode — License expired or moved. View, export, and backup remain active.",

    // Common Actions
    "action.search": "Search students, payments, classes...",
    "action.addStudent": "Add Student",
    "action.importExcel": "Import Excel",
    "action.exportCsv": "Export CSV / Excel",
    "action.exporting": "Exporting...",
    "action.print": "Print",
    "action.printIdCards": "Print ID Cards",
    "action.printAdmitCards": "Print Admit Cards",
    "action.printDueSlips": "Print Due Slips",
    "action.printReportCards": "Print Report Cards",
    "action.receivePayment": "Receive Payment",
    "action.markAllPresent": "Mark All Present",
    "action.createExam": "Create Exam",
    "action.save": "Save Changes",
    "action.cancel": "Cancel",
    "action.close": "Close",
    "action.refresh": "Refresh",
    "action.filter": "Filter",
    "action.clearFilters": "Clear Filters",

    // Metrics & Stats
    "stat.totalStudents": "Total Students",
    "stat.activeBatches": "Active Batches",
    "stat.feeCollection": "Fee Collection",
    "stat.attendanceRate": "Attendance Rate",
    "stat.todayCollection": "Today's Collection",
    "stat.totalDues": "Total Outstanding Dues",
    "stat.cashInDrawer": "Cash in Drawer",

    // Statuses
    "status.paid": "Paid",
    "status.pending": "Pending",
    "status.overdue": "Overdue",
    "status.active": "Active",
    "status.inactive": "Inactive",
    "status.dropped": "Dropped",
    "status.graduated": "Graduated",
    "status.draft": "Draft",
    "status.published": "Published",
    "status.healthy": "Healthy",

    // Views & Headings
    "students.title": "Student Directory",
    "students.subtitle": "Manage admissions, guardians, enrollments, and ID cards",
    "dues.title": "Dues & Statements Center",
    "dues.subtitle": "Track unpaid balances, print due statements, and send reminders",
    "cashier.title": "Fast Cashier Desk",
    "cashier.subtitle": "Collect fees, allocate charges, and print official receipts",
    "exams.title": "Exams & Results Hub",
    "exams.subtitle":
      "Manage exams, enter marks, publish results, and print report cards",
    "reports.title": "Reports & Exports Center",
    "reports.subtitle": "Financial, attendance, and academic analytics with Excel export",
    "theme.customizer": "Customize Theme & Appearance",

    // Domain Errors
    "error.unauthenticated": "Please sign in to continue.",
    "error.forbidden": "You do not have permission to perform this action.",
    "error.validation_error": "Please check the highlighted fields and try again.",
    "error.not_found": "The requested record could not be found.",
    "error.duplicate_payment_detected":
      "A duplicate payment was detected within the safety window.",
    "error.license_read_only":
      "This action is disabled while Coachify is in read-only license mode.",
    "error.provider_secret_foreign_machine":
      "Provider credentials were encrypted on another PC. Please re-enter the API key in Settings.",
    "error.database_error": "A local database error occurred. Your data remains safe.",
    "error.FIN_OVERPAYMENT": "Payment exceeds total outstanding due by {amount}.",
  },
  bn: {
    // Navigation & Sidebar
    "nav.dashboard": "ড্যাশবোর্ড",
    "nav.intelligence": "মালিকানা বুদ্ধিমত্তা",
    "nav.students": "শিক্ষার্থীবৃন্দ",
    "nav.academic": "একাডেমিক ব্যবস্থাপনা",
    "nav.staff": "শিক্ষক ও স্টাফ",
    "nav.attendance": "উপস্থিতি",
    "nav.payments": "অর্থ ও ফি হিসাব",
    "nav.cashier": "ক্যাশিয়ার ডেস্ক",
    "nav.dues": "বকেয়া কেন্দ্র",
    "nav.cashClose": "খরচ ও ক্যাশ ক্লোজ",
    "nav.feeSetup": "ফি নির্ধারণ",
    "nav.communication": "যোগাযোগ ও এসএমএস",
    "nav.exams": "পরীক্ষা ও ফলাফল",
    "nav.reports": "প্রতিবেদন",
    "nav.integrity": "নিরাপত্তা ও অখণ্ডতা",
    "nav.backup": "ব্যাকআপ ও সুরক্ষা",
    "nav.diagnostics": "ডায়াগনস্টিকস",
    "nav.settings": "সেটিংস",
    "nav.group.operations": "দৈনিক কার্যক্রম",
    "nav.group.academics": "শিক্ষার্থী ও একাডেমিক",
    "nav.group.finance": "অর্থ ও লেজার",
    "nav.group.system": "সিস্টেম ও সুরক্ষা",

    // Top Bar & Shell
    "topbar.searchPlaceholder": "শিক্ষার্থী, পেমেন্ট, রসিদ খুঁজুন... (Ctrl+K)",
    "topbar.commandPalette": "কমান্ড প্যালেট (Ctrl+K)",
    "topbar.languageToggle": "EN",
    "topbar.languageCurrent": "বাংলা",
    "topbar.viewAsRole": "ভূমিকা প্রিভিউ",
    "topbar.lockScreen": "স্ক্রিন লক করুন",
    "topbar.logout": "লগ আউট",
    "topbar.aiCopilot": "এআই কোপাইলট",
    "topbar.readOnlyBanner":
      "রিড-অনলি মোড — লাইসেন্সের মেয়াদ শেষ বা স্থানান্তরিত হয়েছে। তথ্য দেখা, এক্সপোর্ট ও ব্যাকআপ সচল আছে।",

    // Common Actions
    "action.search": "শিক্ষার্থী, পেমেন্ট, ব্যাচ খুঁজুন...",
    "action.addStudent": "শিক্ষার্থী যোগ করুন",
    "action.importExcel": "এক্সেল ইমপোর্ট",
    "action.exportCsv": "এক্সেল / CSV এক্সপোর্ট",
    "action.exporting": "এক্সপোর্ট হচ্ছে...",
    "action.print": "প্রিন্ট করুন",
    "action.printIdCards": "আইডি কার্ড প্রিন্ট",
    "action.printAdmitCards": "প্রবেশপত্র প্রিন্ট",
    "action.printDueSlips": "বকেয়া স্লিপ প্রিন্ট",
    "action.printReportCards": "রিপোর্ট কার্ড প্রিন্ট",
    "action.receivePayment": "ফি গ্রহণ",
    "action.markAllPresent": "সবাই উপস্থিত দিন",
    "action.createExam": "পরীক্ষা তৈরি করুন",
    "action.save": "সংরক্ষণ করুন",
    "action.cancel": "বাতিল",
    "action.close": "বন্ধ করুন",
    "action.refresh": "রিফ্রেশ",
    "action.filter": "ফিল্টার",
    "action.clearFilters": "ফিল্টার মুছুন",

    // Metrics & Stats
    "stat.totalStudents": "মোট শিক্ষার্থী",
    "stat.activeBatches": "চলমান ব্যাচ",
    "stat.feeCollection": "ফি আদায়",
    "stat.attendanceRate": "উপস্থিতির হার",
    "stat.todayCollection": "আজকের আদায়",
    "stat.totalDues": "মোট বকেয়া",
    "stat.cashInDrawer": "ক্যাশ ড্রয়ারে জমা",

    // Statuses
    "status.paid": "পরিশোধিত",
    "status.pending": "অপেক্ষমাণ",
    "status.overdue": "বকেয়া",
    "status.active": "সক্রিয়",
    "status.inactive": "নিষ্ক্রিয়",
    "status.dropped": "বাদ পড়েছে",
    "status.graduated": "উত্তীর্ণ",
    "status.draft": "খসড়া",
    "status.published": "প্রকাশিত",
    "status.healthy": "সুরক্ষিত",

    // Views & Headings
    "students.title": "শিক্ষার্থী তালিকা",
    "students.subtitle": "ভর্তি, অভিভাবক, ব্যাচ তালিকাভুক্তি এবং আইডি কার্ড পরিচালনা",
    "dues.title": "বকেয়া ও বিবরণী কেন্দ্র",
    "dues.subtitle": "বকেয়া হিসাব, বকেয়া বিবরণী প্রিন্ট এবং তাগাদা বার্তা প্রেরণ",
    "cashier.title": "দ্রুত ক্যাশিয়ার ডেস্ক",
    "cashier.subtitle": "ফি আদায়, খাত সমন্বয় এবং অফিসিয়াল রসিদ প্রিন্ট",
    "exams.title": "পরীক্ষা ও ফলাফল কেন্দ্র",
    "exams.subtitle":
      "পরীক্ষা পরিচালনা, নম্বর এন্ট্রি, মেধা তালিকা ও রিপোর্ট কার্ড প্রিন্ট",
    "reports.title": "প্রতিবেদন ও এক্সপোর্ট কেন্দ্র",
    "reports.subtitle": "আর্থিক, উপস্থিতি ও একাডেমিক বিশ্লেষণ এবং এক্সেল এক্সপোর্ট",
    "theme.customizer": "থিম ও রূপরেখা পরিবর্তন",

    // Domain Errors
    "error.unauthenticated": "অনুগ্রহ করে পুনরায় লগইন করুন।",
    "error.forbidden": "এই কাজটি করার জন্য আপনার অনুমতি নেই।",
    "error.validation_error": "অনুগ্রহ করে চিহ্নিত ঘরগুলো সঠিক তথ্যে পূরণ করুন।",
    "error.not_found": "অনুরোধকৃত তথ্যটি খুঁজে পাওয়া যায়নি।",
    "error.duplicate_payment_detected":
      "অল্প সময়ের মধ্যে একই পেমেন্ট পুনরায় শনাক্ত হয়েছে।",
    "error.license_read_only": "রিড-অনলি লাইসেন্স মোডে নতুন তথ্য পরিবর্তন করা যাবে না।",
    "error.provider_secret_foreign_machine":
      "গেটওয়ে পাসওয়ার্ডটি অন্য কম্পিউটারে এনক্রিপ্ট করা হয়েছিল। সেটিংস থেকে পুনরায় প্রদান করুন।",
    "error.database_error": "ডাটাবেস ত্রুটি ঘটেছে। আপনার সকল তথ্য নিরাপদ আছে।",
    "error.FIN_OVERPAYMENT": "প্রদত্ত অর্থ মোট বকেয়ার চেয়ে {amount} বেশি।",
  },
};

export function getLocale(): SupportedLocale {
  return currentLocale;
}

export function setLocale(locale: SupportedLocale): void {
  currentLocale = locale;
  if (typeof window !== "undefined") {
    try {
      window.localStorage?.setItem(LOCALE_STORAGE_KEY, locale);
    } catch {
      // Ignore storage quota / sandbox errors
    }
  }
  if (typeof document !== "undefined" && document.documentElement) {
    document.documentElement.lang = locale;
  }
  for (const listener of listeners) {
    listener(currentLocale);
  }
}

export function toggleLocale(): SupportedLocale {
  const next: SupportedLocale = currentLocale === "en" ? "bn" : "en";
  setLocale(next);
  return next;
}

export function subscribeLocale(listener: (locale: SupportedLocale) => void): () => void {
  listeners.push(listener);
  return () => {
    const idx = listeners.indexOf(listener);
    if (idx !== -1) {
      listeners.splice(idx, 1);
    }
  };
}

export function t(
  key: string,
  fallback?: string,
  params?: Record<string, string | number>
): string {
  const dictionary = translations[currentLocale] || translations.en;
  let text = dictionary[key] ?? translations.en[key] ?? fallback ?? key;

  if (params) {
    for (const [paramKey, paramVal] of Object.entries(params)) {
      text = text.replace(new RegExp(`\\{${paramKey}\\}`, "g"), String(paramVal));
    }
  }

  return text;
}

/**
 * Converts ASCII digits (`0-9`) in a string or number to Bengali numerals (`০-৯`).
 */
export function toBengaliDigits(input: string | number): string {
  return String(input).replace(/[0-9]/g, (d) => BENGALI_DIGITS[Number(d)] ?? d);
}

/**
 * Formats an integer or decimal string using South Asian (`#,##,###`) comma grouping.
 */
function formatSouthAsianGrouping(numStr: string): string {
  const [intPartRaw, decPart] = numStr.split(".");
  const isNegative = intPartRaw.startsWith("-");
  const digits = isNegative ? intPartRaw.slice(1) : intPartRaw;

  let grouped = digits;
  if (digits.length > 3) {
    const lastThree = digits.slice(-3);
    const otherNumbers = digits.slice(0, -3);
    const formattedOthers = otherNumbers.replace(/\B(?=(\d{2})+(?!\d))/g, ",");
    grouped = `${formattedOthers},${lastThree}`;
  }

  const signed = isNegative ? `-${grouped}` : grouped;
  return decPart !== undefined ? `${signed}.${decPart}` : signed;
}

/**
 * Formats a number according to Bangladeshi conventions (`1,25,000` in `en`, `১,২৫,০০০` in `bn`).
 */
export function formatNumberLocale(
  value: number,
  locale: SupportedLocale = currentLocale
): string {
  const grouped = formatSouthAsianGrouping(String(value));
  return locale === "bn" ? toBengaliDigits(grouped) : grouped;
}

/**
 * Formats a Poisha integer (100 poisha = 1 Taka) into Bangladeshi Taka (`৳ 1,25,000.00` or `৳ ১,২৫,০০০.০০`).
 */
export function formatCurrencyLocale(
  poisha: number,
  locale: SupportedLocale = currentLocale
): string {
  const taka = (poisha / 100).toFixed(2);
  const grouped = formatSouthAsianGrouping(taka);
  const localizedDigits = locale === "bn" ? toBengaliDigits(grouped) : grouped;
  return `৳ ${localizedDigits}`;
}

/**
 * Formats an ISO date string (`YYYY-MM-DD` or ISO timestamp) in English (`25 Sep 2026`)
 * or Bangla (`২৫ সেপ্টেম্বর ২০২৬`).
 */
export function formatDateLocale(
  isoDate: string,
  locale: SupportedLocale = currentLocale
): string {
  if (!isoDate) return "";
  const clean = isoDate.slice(0, 10);
  const parts = clean.split("-");
  if (parts.length !== 3) return isoDate;

  const year = Number(parts[0]);
  const monthIdx = Number(parts[1]) - 1;
  const day = Number(parts[2]);

  if (Number.isNaN(year) || monthIdx < 0 || monthIdx > 11 || Number.isNaN(day)) {
    return isoDate;
  }

  if (locale === "bn") {
    return `${toBengaliDigits(day)} ${BN_MONTHS[monthIdx]} ${toBengaliDigits(year)}`;
  }

  return `${day} ${EN_MONTHS[monthIdx]} ${year}`;
}

/**
 * Translates an RPC error code (`{ code, message }`) into the active locale.
 */
export function translateError(
  code: string,
  fallbackMessage?: string,
  params?: Record<string, string | number>
): string {
  const key = `error.${code}`;
  const dict = translations[currentLocale] || translations.en;
  if (dict[key]) {
    return t(key, fallbackMessage, params);
  }
  return fallbackMessage || code;
}

/**
 * Reactive React hook for reading and switching the current UI locale (`en` | `bn`).
 */
export function useLocale() {
  const locale = useSyncExternalStore(subscribeLocale, getLocale, getLocale);
  return {
    locale,
    setLocale,
    toggleLocale,
    t,
    toBengaliDigits,
    formatNumber: (val: number) => formatNumberLocale(val, locale),
    formatCurrency: (poisha: number) => formatCurrencyLocale(poisha, locale),
    formatDate: (isoDate: string) => formatDateLocale(isoDate, locale),
    translateError,
  };
}
