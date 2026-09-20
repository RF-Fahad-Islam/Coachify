export type SupportedLocale = "en" | "bn";

let currentLocale: SupportedLocale = "en";
const listeners: Array<(locale: SupportedLocale) => void> = [];

export const translations: Record<SupportedLocale, Record<string, string>> = {
  en: {
    "nav.dashboard": "Dashboard",
    "nav.students": "Students",
    "nav.attendance": "Attendance",
    "nav.payments": "Finance & Fees",
    "nav.exams": "Exams & Marks",
    "nav.reports": "Reports",
    "nav.settings": "Settings",
    "action.search": "Search students, payments, classes...",
    "action.addStudent": "Add Student",
    "action.receivePayment": "Receive Payment",
    "action.markAllPresent": "Mark All Present",
    "action.createExam": "Create Exam",
    "stat.totalStudents": "Total Students",
    "stat.activeBatches": "Active Batches",
    "stat.feeCollection": "Fee Collection",
    "stat.attendanceRate": "Attendance Rate",
    "status.paid": "Paid",
    "status.pending": "Pending",
    "status.overdue": "Overdue",
    "status.active": "Active",
    "status.draft": "Draft",
    "theme.customizer": "Customize Theme & Appearance",
  },
  bn: {
    "nav.dashboard": "ড্যাশবোর্ড",
    "nav.students": "শিক্ষার্থীবৃন্দ",
    "nav.attendance": "উপস্থিতি",
    "nav.payments": "অর্থ ও ফি হিসাব",
    "nav.exams": "পরীক্ষা ও ফলাফল",
    "nav.reports": "প্রতিবেদন",
    "nav.settings": "সেটিংস",
    "action.search": "শিক্ষার্থী, পেমেন্ট, ব্যাচ খুঁজুন...",
    "action.addStudent": "শিক্ষার্থী যোগ করুন",
    "action.receivePayment": "ফি গ্রহণ",
    "action.markAllPresent": "সবাই উপস্থিত দিন",
    "action.createExam": "পরীক্ষা তৈরি করুন",
    "stat.totalStudents": "মোট শিক্ষার্থী",
    "stat.activeBatches": "চলমান ব্যাচ",
    "stat.feeCollection": "ফি আদায়",
    "stat.attendanceRate": "উপস্থিতির হার",
    "status.paid": "পরিশোধিত",
    "status.pending": "অপেক্ষমাণ",
    "status.overdue": "বকেয়া",
    "status.active": "সক্রিয়",
    "status.draft": "খসড়া",
    "theme.customizer": "থিম ও রূপরেখা পরিবর্তন",
  },
};

export function getLocale(): SupportedLocale {
  return currentLocale;
}

export function setLocale(locale: SupportedLocale): void {
  currentLocale = locale;
  for (const listener of listeners) {
    listener(currentLocale);
  }
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
  let text = dictionary[key] ?? fallback ?? key;

  if (params) {
    for (const [paramKey, paramVal] of Object.entries(params)) {
      text = text.replace(new RegExp(`\\{${paramKey}\\}`, "g"), String(paramVal));
    }
  }

  return text;
}
