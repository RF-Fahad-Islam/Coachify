/* ================================================================
   COACHIFY — SINGLE SOURCE OF TRUTH FOR ALL ON-SCREEN DATA
   ----------------------------------------------------------------
   Every product mockup on the site reads from this file.
   Localized for Bangladesh coaching centers (৳ BDT, HSC/SSC/Admission,
   Atomic Coaching Center, bKash/Nagad/Cash).
   ================================================================ */

export const CURRENCY = "৳";

export const RELEASE_INFO = {
  version: "v1.0.0",
  fileName: "CoachifyDemoSetup-v1.0.0-x64.exe",
  sizeLabel: "68.4 MB",
  osRequirements: "Windows 10 / 11 (64-bit)",
  sha256: "8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4",
  downloadUrl:
    (typeof import.meta !== "undefined" &&
      (import.meta as { env?: Record<string, string | undefined> }).env
        ?.VITE_INSTALLER_URL) ||
    "/downloads/CoachifyDemoSetup-v1.0.0-x64.exe",
} as const;

/* ----------------------------- identity ----------------------------- */

export const INSTITUTION = {
  name: "Atomic Coaching Center",
  banglaName: "অ্যাটমিক কোচিং সেন্টার",
  shortName: "Atomic Coaching",
  receiptPrefix: "R",
  currency: "৳",
};

/* ----------------------------- dashboard ----------------------------- */

export const DASHBOARD = {
  greeting: "Good morning",
  dateLine: "Friday, 25 September 2026",
  classesToday: 34,
  stats: [
    { label: "Students", value: "1,284", sub: "+38 this month" },
    { label: "Fees collected", value: "৳8.42L", sub: "September to date" },
    { label: "Pending dues", value: "৳62.5K", sub: "42 students" },
    { label: "Attendance", value: "94.2%", sub: "today" },
  ],
  /** Monthly fee collection in ৳ lakhs — drives the dashboard bar chart */
  monthlyCollections: {
    labels: [
      "Oct",
      "Nov",
      "Dec",
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
      "Jul",
      "Aug",
      "Sep",
    ],
    values: [4.6, 5.8, 5.1, 6.9, 6.2, 7.8, 7.1, 8.6, 7.4, 9.2, 9.8, 10.6],
  },
};

/* ----------------------------- students ----------------------------- */

export type StudentStatus = "paid" | "due" | "overdue";

export interface Student {
  name: string;
  batch: string;
  guardian: string;
  status: StudentStatus;
  dueLabel?: string;
  lastPay: string;
}

export const STUDENTS: Student[] = [
  {
    name: "Tanvir Ahmed",
    batch: "HSC 2026–A",
    guardian: "M. Ahmed",
    status: "paid",
    lastPay: "22 Sep",
  },
  {
    name: "Nusrat Jahan",
    batch: "Medical 2026–A",
    guardian: "R. Jahan",
    status: "due",
    dueLabel: "Due ৳4,500",
    lastPay: "10 Sep",
  },
  {
    name: "Farhan Sadik",
    batch: "BUET 2026",
    guardian: "K. Sadik",
    status: "paid",
    lastPay: "21 Sep",
  },
  {
    name: "Tasnim Zara",
    batch: "SSC 2027 Found.",
    guardian: "S. Islam",
    status: "due",
    dueLabel: "Due ৳2,500",
    lastPay: "08 Sep",
  },
  {
    name: "Mahmudul Hasan",
    batch: "HSC 2026–A",
    guardian: "A. Hasan",
    status: "paid",
    lastPay: "19 Sep",
  },
  {
    name: "Sadia Afrin",
    batch: "Medical 2027",
    guardian: "H. Rahman",
    status: "overdue",
    dueLabel: "Overdue",
    lastPay: "03 Aug",
  },
];

export const STUDENT_FILTERS = ["All", "HSC", "Medical", "BUET", "SSC"];
export const STUDENT_COUNTS = { students: 1284, batches: 43, guardians: 1291 };

/* ----------------------------- attendance ----------------------------- */

export const ATTENDANCE = {
  batch: "HSC 2026–A",
  totalStudents: 34,
  date: "Fri, 25 Sep",
  roster: [
    { name: "Tanvir Ahmed", present: true },
    { name: "Sadia Afrin", present: true },
    { name: "Farhan Sadik", present: true },
    { name: "Mahmudul Hasan", present: true },
    { name: "Nusrat Jahan", present: false },
    { name: "Tasnim Zara", present: true },
    { name: "Rafiqul Islam", present: false },
    { name: "Meherab Hossain", present: true },
  ],
  /** Month calendar: total days, days filled so far, weekend rule */
  calendar: { totalDays: 30, filled: 25 },
  absentSms: "SMS queued to 2 absent guardians",
};

/* ----------------------------- payments ----------------------------- */

export const PAYMENTS = {
  cashier: {
    student: "Tanvir Ahmed",
    batch: "HSC 2026–A",
    outstanding: 12500,
    amount: 5000,
    installment: "Installment 2 of 4",
    term: "Session Jul–Sep",
    note: "Merit scholarship applied",
    modes: ["Cash", "bKash", "Nagad", "Bank"],
    activeMode: "bKash",
  },
  recent: [
    { name: "Tanvir Ahmed", amount: 5000 },
    { name: "Nusrat Jahan", amount: 3500 },
    { name: "Farhan Sadik", amount: 6000 },
  ],
  receipt: {
    number: "R-10247",
    student: "Tanvir Ahmed",
    batch: "HSC 2026–A",
    date: "25 Sep 2026",
    amount: 5000,
    note: "Thermal 80mm or A4 · Bangla & English copy",
  },
};

export const HERO_RECEIPT = { number: "R-10247" };

/* ----------------------------- dues ----------------------------- */

export const DUES = {
  summary: [
    { label: "Outstanding", value: "৳18.4L" },
    { label: "Overdue", value: "42", tone: "red" as const },
    { label: "Due this month", value: "৳3.2L" },
  ],
  rows: [
    {
      name: "Nusrat Jahan",
      batch: "Medical 2026–A",
      due: "৳4,500",
      by: "28 Sep",
      status: "Due soon",
      tone: "amber" as const,
    },
    {
      name: "Tasnim Zara",
      batch: "SSC 2027 Found.",
      due: "৳2,500",
      by: "27 Sep",
      status: "Due soon",
      tone: "amber" as const,
    },
    {
      name: "Sadia Afrin",
      batch: "Medical 2027",
      due: "৳9,000",
      by: "03 Aug",
      status: "Overdue",
      tone: "red" as const,
    },
    {
      name: "Nafis Iqbal",
      batch: "BUET 2026",
      due: "৳4,500",
      by: "30 Sep",
      status: "Scheduled",
      tone: "stone" as const,
    },
    {
      name: "Maliha Chowdhury",
      batch: "HSC 2026–A",
      due: "৳6,500",
      by: "29 Sep",
      status: "Due soon",
      tone: "amber" as const,
    },
  ],
  showing: "Showing 42 of 216 with dues",
  action: "Send reminders (SMS)",
};

/* ----------------------------- exams ----------------------------- */

export const EXAMS = {
  list: [
    {
      name: "HSC Model Test II",
      meta: "Physics 1st Paper · 90 min · 100 marks",
      date: "28 Sep",
      status: "Marks entered · 28/34",
      tone: "orange" as const,
    },
    {
      name: "Medical Admission Full Mock",
      meta: "Biology + Chem · 60 min · 100 marks",
      date: "20 Sep",
      status: "To conduct",
      tone: "stone" as const,
    },
    {
      name: "SSC Term Assessment",
      meta: "Higher Math · 3 hrs",
      date: "15 Sep",
      status: "Results ready",
      tone: "green" as const,
    },
    {
      name: "BUET Preliminary Mock I",
      meta: "PCM · 3 hrs",
      date: "02 Oct",
      status: "Scheduled",
      tone: "stone" as const,
    },
  ],
  markEntry: {
    title: "Mark entry — HSC Model Test II · Physics",
    note: "GPA 5.00 scale auto-graded on save",
    marks: [
      "88",
      "76",
      "92",
      "68",
      "95",
      "81",
      "72",
      "89",
      "74",
      "",
      "84",
      "",
      "91",
      "64",
      "",
      "94",
    ],
  },
};

/* ----------------------------- results ----------------------------- */

export const RESULTS = {
  title: "Results — HSC 2026–A · Model Test II",
  gradeBands: ["A+ ≥ 80 (GPA 5.0)", "A ≥ 70", "A- ≥ 60", "B ≥ 50"],
  columns: ["Student", "Phy", "Chem", "Math", "Total", "Grade"],
  rows: [
    {
      name: "Sadia Afrin",
      phy: 92,
      chem: 94,
      math: 90,
      total: 276,
      grade: "A+",
      tone: "green" as const,
    },
    {
      name: "Tanvir Ahmed",
      phy: 85,
      chem: 88,
      math: 84,
      total: 257,
      grade: "A+",
      tone: "green" as const,
    },
    {
      name: "Farhan Sadik",
      phy: 74,
      chem: 78,
      math: 72,
      total: 224,
      grade: "A",
      tone: "sky" as const,
    },
    {
      name: "Nusrat Jahan",
      phy: 62,
      chem: 68,
      math: 65,
      total: 195,
      grade: "A-",
      tone: "amber" as const,
    },
    {
      name: "Mahmudul Hasan",
      phy: 76,
      chem: 71,
      math: 75,
      total: 222,
      grade: "A",
      tone: "sky" as const,
    },
  ],
  rankNote: "Merit 1: Sadia Afrin · 276/300 (GPA 5.00)",
};

/* ----------------------------- reports ----------------------------- */

export const REPORTS = {
  monthly: {
    title: "Fee collection — 8 months (৳ Lakh)",
    labels: ["Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep"],
    values: [6.8, 7.9, 6.2, 8.4, 9.3, 7.6, 9.9, 10.6],
    footnote: "Avg ৳9.1L/month · best: Sep",
  },
  attendanceByBatch: [
    { batch: "HSC 2026–A", pct: 94 },
    { batch: "HSC 2026–B", pct: 91 },
    { batch: "Medical 2026–A", pct: 96 },
    { batch: "SSC 2027 Found.", pct: 89 },
  ],
  templates: [
    "Monthly fee summary",
    "Student due statement",
    "Batch dues",
    "Exam merit list",
    "Daily cash close",
  ],
};

/* ----------------------------- notifications ----------------------------- */

export const NOTIFICATIONS = {
  phoneTime: "Guardian phone · 6:12 PM",
  messages: [
    {
      text: `Fee of ৳5,000 received for Tanvir (HSC 2026–A) via bKash. Balance: ৳7,500. — ${INSTITUTION.shortName}`,
      tone: "green" as const,
    },
    {
      text: `Reminder: ৳4,500 monthly tuition for Nusrat is due by 28 Sep. — ${INSTITUTION.shortName}`,
      tone: "orange" as const,
    },
    {
      text: `Sadia was marked absent in Physics 1st Paper today. — ${INSTITUTION.shortName}`,
      tone: "red" as const,
    },
  ],
  weeklyCount: 142,
};

/* ----------------------------- backups ----------------------------- */

export const BACKUPS = {
  label: "Backup — encrypted SQLite archive",
  files: [
    { name: "atomic-backup-2026-09-25.cfb", size: "212 MB" },
    { name: "atomic-backup-2026-09-23.cfb", size: "211 MB" },
    { name: "atomic-backup-2026-09-20.cfb", size: "209 MB" },
  ],
  caption:
    "Schedule daily · copy to USB pen drive or second PC · restore in one click",
};

/* ----------------------------- setup / institution wizard ----------------------------- */

export const SETUP = {
  title: "Institution setup",
  step: "Step 3 of 4",
  fields: [
    { label: "Currency", value: "৳ Bangladeshi Taka (BDT)" },
    { label: "Academic session", value: "HSC 2026 – 2027" },
    { label: "Language", value: "Bangla + English" },
    { label: "Receipt series", value: "R-00001" },
  ],
  progress: 80,
  footnote: "Done in about 15 minutes — once",
};

/* ----------------------------- admission ----------------------------- */

export const ADMISSION = {
  fields: [
    { label: "Student", value: "Sadia Afrin (সাদিয়া আফরিন)" },
    { label: "Guardian", value: "H. Rahman · 01711-234567" },
    { label: "Batch", value: "Medical 2027 · Morning A" },
    { label: "Fee plan", value: "৳4,500 × 4 installments" },
  ],
  idCard: {
    student: "Sadia Afrin",
    initials: "SA",
    id: "2026-0142",
    batch: "Medical 2027 · A",
  },
};

/* ----------------------------- batches ----------------------------- */

export const BATCHES = [
  {
    name: "HSC 2026–A",
    students: "34 students",
    subjects: "4 subjects",
    timing: "Sat–Thu · 4–6 PM",
    teachers: ["TR", "MK", "SH", "AR"],
    teacherCount: 4,
  },
  {
    name: "HSC 2026–B",
    students: "28 students",
    subjects: "4 subjects",
    timing: "Sat–Thu · 6–8 PM",
    teachers: ["TR", "MK", "SH", "AR"],
    teacherCount: 4,
  },
  {
    name: "Medical 2026–A",
    students: "41 students",
    subjects: "3 subjects",
    timing: "Sun–Thu · 3–5 PM",
    teachers: ["TR", "MK", "SH", "AR"],
    teacherCount: 4,
  },
];

/* ----------------------------- receipt (print scene) ----------------------------- */

export const PRINT_RECEIPT = {
  number: "R-10248",
  student: "Nusrat Jahan",
  batch: "Medical 2026–A",
  installment: "3 of 4",
  mode: "bKash",
  amount: 4500,
  chips: ["Printed in 4s", "80mm & A4 ready"],
};

/* ----------------------------- today's schedule ----------------------------- */

export const TODAY_BATCHES = [
  { time: "4:00 PM", batch: "HSC 2026–A · Room 201" },
  { time: "4:00 PM", batch: "Medical 2026–A · Room 105" },
  { time: "6:00 PM", batch: "SSC 2027 Found. · Room 110" },
];

/* ----------------------------- commercial pricing ----------------------------- */

export interface CommercialPlan {
  name: string;
  price: string;
  period: string;
  desc: string;
  features: string[];
  cta: string;
  kind: "download" | "request";
  interest?: string;
  featured: boolean;
}

export const PRICING_PLANS: CommercialPlan[] = [
  {
    name: "Demo",
    price: "Free",
    period: "৳0 · 30 full-featured days",
    desc: "Try everything risk-free with the preloaded 500+ student Atomic Coaching dataset or your own center's records.",
    features: [
      "Every module enabled for 30 days",
      "Up to 100 students · 10 batches",
      "Single PC · runs 100% offline",
      "Full SQLite backup & restore tools",
      "No credit card or bKash auto-debit",
    ],
    cta: "Download Demo",
    kind: "download",
    featured: false,
  },
  {
    name: "Full Version",
    price: "৳14,900",
    period: "One-time per-PC license · lifetime use",
    desc: "Own the software outright for your main office PC. Unlimited students, unlimited academic sessions, zero monthly rent.",
    features: [
      "Unlimited students, guardians & batches",
      "All modules + Bangla/English SMS & receipts",
      "Encrypted backups & one-click restore",
      "Perpetual license for 1 Windows PC",
      "Priority WhatsApp/phone support · 1yr updates",
    ],
    cta: "Request Full Version",
    kind: "request",
    interest: "Full Version (৳14,900)",
    featured: true,
  },
  {
    name: "Pro Multi-PC",
    price: "৳29,900",
    period: "One-time LAN license · up to 5 PCs",
    desc: "Run reception, cashier, exam cell, and director desk together over your local Wi-Fi/LAN with zero cloud lag.",
    features: [
      "Everything in Full Version",
      "Up to 5 PCs on your office LAN",
      "Live shared local database, zero conflicts",
      "Free Excel student & balance migration",
      "Dedicated onboarding + phone support",
    ],
    cta: "Talk to Sales",
    kind: "request",
    interest: "Pro Multi-PC LAN (৳29,900)",
    featured: false,
  },
];

/* ----------------------------- helpers ----------------------------- */

export const bdt = (n: number) => `${CURRENCY}${n.toLocaleString("en-IN")}.00`;
export const bdtShort = (n: number) =>
  `${CURRENCY}${n.toLocaleString("en-IN")}`;

/** Backward-compatible aliases used by screen components */
export const inr = bdt;
export const inrShort = bdtShort;
