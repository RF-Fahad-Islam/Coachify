import { useEffect, useState } from "react";
import {
  Users,
  CalendarCheck,
  CreditCard,
  Award,
  ChevronLeft,
  ChevronRight,
  Sparkles,
  Printer,
  Check,
  TrendingUp,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { loadCarouselSlideIndex, saveCarouselSlideIndex } from "@/lib/onboarding-storage";

interface OnboardingSlide {
  id: string;
  badge: string;
  badgeColor: string;
  title: string;
  tagline: string;
  description: string;
  icon: typeof Users;
  iconBg: string;
  visual: () => React.JSX.Element;
  features: string[];
}

// 1. Fatty Visual: Student Admission & Directory Card
function StudentCardVisual() {
  return (
    <div className="relative group transition-all duration-300 transform hover:scale-[1.01]">
      {/* Ambient Glow */}
      <div className="absolute -inset-1 rounded-3xl bg-black/20 blur-xl opacity-60 group-hover:opacity-80 transition duration-500" />

      {/* Card Content */}
      <div className="relative rounded-2xl bg-card text-card-foreground backdrop-blur-md border border-white/30 p-4 sm:p-5 shadow-2xl shadow-black/20 space-y-3 sm:space-y-4 max-w-md mx-auto">
        {/* Header with Avatar & Status */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="h-12 w-12 rounded-2xl bg-gradient-to-tr from-blue-600 to-indigo-500 flex items-center justify-center text-white font-bold text-base shadow-md shadow-blue-500/20">
              TA
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h4 className="font-bold text-sm text-foreground">Tanvir Ahmed</h4>
                <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30">
                  Enrolled
                </span>
              </div>
              <p className="text-xs text-muted-foreground">
                Roll #01 · Class 10 (Science)
              </p>
            </div>
          </div>
          <span className="text-xs font-mono font-bold text-blue-600 dark:text-blue-400 bg-blue-500/10 px-2.5 py-1 rounded-lg">
            Batch Delta
          </span>
        </div>

        {/* Micro Details Grid */}
        <div className="grid grid-cols-2 gap-2 text-xs">
          <div className="p-2.5 rounded-xl bg-muted/60 border border-border/50">
            <span className="text-[10px] text-muted-foreground block">
              Guardian Phone
            </span>
            <span className="font-semibold text-foreground font-mono">01711-432109</span>
          </div>
          <div className="p-2.5 rounded-xl bg-muted/60 border border-border/50">
            <span className="text-[10px] text-muted-foreground block">
              Admission Date
            </span>
            <span className="font-semibold text-foreground">15 Jan 2026</span>
          </div>
        </div>

        {/* Enrolled Subjects Pill List */}
        <div className="flex items-center gap-1.5 pt-1">
          <span className="text-[11px] text-muted-foreground">Subjects:</span>
          {["Higher Math", "Physics", "Chemistry"].map((sub) => (
            <span
              key={sub}
              className="text-[10px] px-2 py-0.5 rounded-md bg-secondary text-secondary-foreground font-medium"
            >
              {sub}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}

// 2. Fatty Visual: Rapid Attendance Grid & Counter
function AttendanceVisual() {
  return (
    <div className="relative group transition-all duration-300 transform hover:scale-[1.01]">
      {/* Ambient Glow */}
      <div className="absolute -inset-1 rounded-3xl bg-black/20 blur-xl opacity-60 group-hover:opacity-80 transition duration-500" />

      {/* Card Content */}
      <div className="relative rounded-2xl bg-card text-card-foreground backdrop-blur-md border border-white/30 p-4 sm:p-5 shadow-2xl shadow-black/20 space-y-3 sm:space-y-4 max-w-md mx-auto">
        {/* Top Header */}
        <div className="flex items-center justify-between">
          <div>
            <span className="text-[10px] font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wider">
              Morning Batch · Today
            </span>
            <h4 className="font-bold text-sm sm:text-base text-foreground">
              Class 10 Attendance
            </h4>
          </div>
          <div className="text-right">
            <span className="text-base sm:text-lg font-extrabold text-foreground">
              96%
            </span>
            <span className="text-[10px] text-muted-foreground block">
              Attendance Rate
            </span>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="w-full bg-muted rounded-full h-2.5 overflow-hidden">
          <div
            className="bg-gradient-to-r from-emerald-500 to-teal-400 h-2.5 rounded-full transition-all duration-500"
            style={{ width: "96%" }}
          />
        </div>

        {/* Big Counter Badges */}
        <div className="grid grid-cols-2 gap-2.5 sm:gap-3">
          <div className="p-2.5 sm:p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-between">
            <div>
              <span className="text-[11px] sm:text-xs text-emerald-700 dark:text-emerald-300 font-medium">
                Present
              </span>
              <div className="text-lg sm:text-xl font-extrabold text-emerald-600 dark:text-emerald-400">
                48 Students
              </div>
            </div>
            <div className="h-7 w-7 sm:h-8 sm:w-8 rounded-full bg-emerald-500 text-white flex items-center justify-center font-bold text-xs shadow-sm">
              ✓
            </div>
          </div>
          <div className="p-2.5 sm:p-3 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-center justify-between">
            <div>
              <span className="text-[11px] sm:text-xs text-rose-700 dark:text-rose-300 font-medium">
                Absent
              </span>
              <div className="text-lg sm:text-xl font-extrabold text-rose-600 dark:text-rose-400">
                2 Students
              </div>
            </div>
            <div className="h-7 w-7 sm:h-8 sm:w-8 rounded-full bg-rose-500 text-white flex items-center justify-center font-bold text-xs shadow-sm">
              ✕
            </div>
          </div>
        </div>

        {/* Action Status Pill */}
        <div className="flex items-center gap-2 px-3 py-1.5 sm:py-2 rounded-xl bg-muted/70 text-muted-foreground text-[11px] sm:text-xs">
          <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse shrink-0" />
          <span className="truncate">
            Automatic SMS parent notifications ready for 2 absent students
          </span>
        </div>
      </div>
    </div>
  );
}

// 3. Fatty Visual: Tuition Fee Collection & Money Receipt
function ReceiptVisual() {
  return (
    <div className="relative group transition-all duration-300 transform hover:scale-[1.01]">
      {/* Ambient Glow */}
      <div className="absolute -inset-1 rounded-3xl bg-black/20 blur-xl opacity-60 group-hover:opacity-80 transition duration-500" />

      {/* Card Content */}
      <div className="relative rounded-2xl bg-card text-card-foreground backdrop-blur-md border border-white/30 p-4 sm:p-5 shadow-2xl shadow-black/20 space-y-3 sm:space-y-4 max-w-md mx-auto">
        {/* Receipt Header */}
        <div className="flex items-center justify-between pb-2.5 sm:pb-3 border-b border-border/60">
          <div>
            <span className="text-[10px] font-mono text-muted-foreground uppercase">
              VOUCHER #RCP-2026-084
            </span>
            <h4 className="font-bold text-xs sm:text-sm text-foreground">
              Official Money Receipt
            </h4>
          </div>
          <span className="px-2.5 py-1 rounded-lg text-[10px] sm:text-xs font-extrabold bg-emerald-500 text-white shadow-sm">
            PAID IN CASH
          </span>
        </div>

        {/* Receipt Body */}
        <div className="space-y-1.5 sm:space-y-2">
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">Student Name:</span>
            <span className="font-semibold text-foreground">Anika Rahman (Roll 04)</span>
          </div>
          <div className="flex justify-between text-xs">
            <span className="text-muted-foreground">Month / Purpose:</span>
            <span className="font-medium text-foreground">September Monthly Tuition</span>
          </div>
          <div className="flex justify-between items-baseline pt-2 border-t border-dashed border-border/70">
            <span className="text-xs font-semibold text-foreground">
              Total Collected:
            </span>
            <span className="text-lg sm:text-xl font-black text-amber-600 dark:text-amber-400 font-mono">
              ৳ 3,500.00
            </span>
          </div>
        </div>

        {/* Due Status & Print Button */}
        <div className="flex items-center justify-between pt-1">
          <div className="flex items-center gap-1.5 text-[11px] sm:text-xs text-emerald-600 dark:text-emerald-400 font-medium">
            <Check className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
            <span>Due Balance: ৳ 0 (Cleared)</span>
          </div>
          <div className="inline-flex items-center gap-1.5 px-2.5 sm:px-3 py-1 sm:py-1.5 rounded-lg bg-primary text-primary-foreground text-xs font-semibold shadow-sm">
            <Printer className="h-3.5 w-3.5" />
            <span>Print Receipt</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// 4. Fatty Visual: Exam Marksheet & Report Card
function ReportCardVisual() {
  return (
    <div className="relative group transition-all duration-300 transform hover:scale-[1.01]">
      {/* Ambient Glow */}
      <div className="absolute -inset-1 rounded-3xl bg-black/20 blur-xl opacity-60 group-hover:opacity-80 transition duration-500" />

      {/* Card Content */}
      <div className="relative rounded-2xl bg-card text-card-foreground backdrop-blur-md border border-white/30 p-4 sm:p-5 shadow-2xl shadow-black/20 space-y-3 sm:space-y-4 max-w-md mx-auto">
        {/* Exam Title */}
        <div className="flex items-center justify-between">
          <div>
            <span className="text-[10px] font-semibold text-purple-600 dark:text-purple-400 uppercase tracking-wider">
              Term Final Exam 2026
            </span>
            <h4 className="font-bold text-sm sm:text-base text-foreground">
              Academic Progress Report
            </h4>
          </div>
          <div className="h-9 w-9 sm:h-10 sm:w-10 rounded-xl bg-purple-500/10 border border-purple-500/30 text-purple-600 dark:text-purple-400 flex items-center justify-center font-bold text-xs sm:text-sm">
            A+
          </div>
        </div>

        {/* Subject Score Pills */}
        <div className="space-y-1.5">
          <div className="flex items-center justify-between p-2 rounded-lg bg-muted/60 text-xs">
            <span className="font-medium text-foreground">Mathematics</span>
            <span className="font-mono font-bold text-purple-600 dark:text-purple-400">
              96 / 100 (A+)
            </span>
          </div>
          <div className="flex items-center justify-between p-2 rounded-lg bg-muted/60 text-xs">
            <span className="font-medium text-foreground">Physics</span>
            <span className="font-mono font-bold text-purple-600 dark:text-purple-400">
              92 / 100 (A+)
            </span>
          </div>
          <div className="flex items-center justify-between p-2 rounded-lg bg-muted/60 text-xs">
            <span className="font-medium text-foreground">Chemistry</span>
            <span className="font-mono font-bold text-purple-600 dark:text-purple-400">
              89 / 100 (A+)
            </span>
          </div>
        </div>

        {/* Ranking & GPA Banner */}
        <div className="flex items-center justify-between pt-1 text-xs">
          <div className="flex items-center gap-1.5 font-semibold text-emerald-600 dark:text-emerald-400">
            <TrendingUp className="h-4 w-4" />
            <span>Class Rank: #1 of 52</span>
          </div>
          <span className="px-2.5 py-1 rounded-md bg-purple-500/10 text-purple-700 dark:text-purple-300 font-bold text-[11px]">
            Overall GPA: 5.00
          </span>
        </div>
      </div>
    </div>
  );
}

const ONBOARDING_SLIDES: OnboardingSlide[] = [
  {
    id: "student-directory",
    badge: "Student Hub",
    badgeColor: "bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/30",
    title: "Admissions & Batch Management",
    tagline: "Organize students, guardian contacts, and batches effortlessly.",
    description:
      "Enroll new students in seconds with custom roll numbers and guardian info. Assign batches, manage subjects, and search any student profile instantly.",
    icon: Users,
    iconBg: "from-blue-600 to-indigo-600 shadow-blue-500/25",
    visual: StudentCardVisual,
    features: ["Quick Admissions", "Guardian Contacts", "Batch Timetables"],
  },
  {
    id: "fast-attendance",
    badge: "Daily Roll Call",
    badgeColor:
      "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30",
    title: "One-Tap Daily Attendance",
    tagline: "Mark attendance in seconds with keyboard shortcuts.",
    description:
      "Take daily roll call for full batches with rapid present/absent toggles. Spot missing students instantly and review monthly percentage trends.",
    icon: CalendarCheck,
    iconBg: "from-emerald-600 to-teal-600 shadow-emerald-500/25",
    visual: AttendanceVisual,
    features: ["One-Tap Roll Call", "Monthly Trends", "Absence Tracking"],
  },
  {
    id: "fees-receipts",
    badge: "Counter Cashier",
    badgeColor: "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30",
    title: "Instant Fee Collection & Receipts",
    tagline: "Collect fees with exact poisha precision and print slips.",
    description:
      "Collect monthly tuition, exam fees, and admission charges. Keep real-time due balance tracking and generate print-ready vouchers for parents with one click.",
    icon: CreditCard,
    iconBg: "from-amber-500 to-orange-600 shadow-orange-500/25",
    visual: ReceiptVisual,
    features: ["Instant Receipt Print", "Due Balance Alerts", "Tuition Tracking"],
  },
  {
    id: "exams-reports",
    badge: "Academic Results",
    badgeColor:
      "bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/30",
    title: "Marksheets & Report Cards",
    tagline: "Automatic GPA calculation, batch ranks, and printed report cards.",
    description:
      "Enter subject marks smoothly with instant GPA calculation and missing score warnings. Generate elegant printable progress cards for parent-teacher meetings.",
    icon: Award,
    iconBg: "from-purple-600 to-pink-600 shadow-purple-500/25",
    visual: ReportCardVisual,
    features: ["Auto GPA & Grading", "Batch Rankings", "Printable Progress Cards"],
  },
];

export function OnboardingCarousel() {
  const [currentSlide, setCurrentSlide] = useState(() => loadCarouselSlideIndex());
  const [isPaused, setIsPaused] = useState(false);

  useEffect(() => {
    saveCarouselSlideIndex(currentSlide);
  }, [currentSlide]);

  useEffect(() => {
    if (isPaused) return;

    const timer = setInterval(() => {
      setCurrentSlide((prev) => (prev + 1) % ONBOARDING_SLIDES.length);
    }, 7000);

    return () => clearInterval(timer);
  }, [isPaused]);

  const slide = ONBOARDING_SLIDES[currentSlide];
  const IconComponent = slide.icon;
  const VisualComponent = slide.visual;

  const handlePrev = () => {
    setCurrentSlide(
      (prev) => (prev - 1 + ONBOARDING_SLIDES.length) % ONBOARDING_SLIDES.length
    );
  };

  const handleNext = () => {
    setCurrentSlide((prev) => (prev + 1) % ONBOARDING_SLIDES.length);
  };

  return (
    <div
      className="relative flex flex-col justify-between h-full p-4 sm:p-6 lg:p-10 select-none overflow-hidden"
      onMouseEnter={() => setIsPaused(true)}
      onMouseLeave={() => setIsPaused(false)}
    >
      {/* Decorative Ambient Background Gradients */}
      <div className="absolute top-0 left-0 w-96 h-96 bg-white/10 rounded-full blur-3xl pointer-events-none -translate-x-1/2 -translate-y-1/2" />
      <div className="absolute bottom-0 right-0 w-96 h-96 bg-black/15 rounded-full blur-3xl pointer-events-none translate-x-1/3 translate-y-1/3" />

      {/* Top Brand Header */}
      <div className="relative z-10 flex items-center justify-between max-w-xl w-full mx-auto">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-white text-primary font-black text-sm shadow-md ring-2 ring-white/30">
            C
          </div>
          <div>
            <span className="text-base font-extrabold tracking-tight text-white block leading-tight">
              Coachify
            </span>
            <span className="text-[11px] text-white/80 font-medium">
              Coaching Institute OS
            </span>
          </div>
        </div>
        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-white/15 border border-white/20 text-white shadow-sm backdrop-blur-sm">
          <Sparkles className="h-3.5 w-3.5 text-white" />
          <span>Made for Coaching Centers</span>
        </div>
      </div>

      {/* Middle Interactive Presentation Body */}
      <div className="relative z-10 my-auto py-2 sm:py-4 md:py-6 space-y-4 sm:space-y-6 max-w-xl w-full mx-auto">
        {/* Fatty Interactive Visual Preview Widget */}
        <div className="w-full">
          <VisualComponent />
        </div>

        {/* Feature Information Details */}
        <div className="space-y-2.5 sm:space-y-3">
          <div className="flex items-center gap-2.5">
            <div className="flex h-9 w-9 sm:h-10 sm:w-10 items-center justify-center rounded-xl bg-white/20 text-white border border-white/30 backdrop-blur-sm shadow-md shrink-0">
              <IconComponent className="h-4 w-4 sm:h-5 sm:w-5" />
            </div>
            <div>
              <span className="inline-block px-2.5 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-white/20 text-white border border-white/25 backdrop-blur-sm">
                {slide.badge}
              </span>
              <h2 className="text-lg sm:text-xl md:text-2xl font-black tracking-tight text-white">
                {slide.title}
              </h2>
            </div>
          </div>

          <p className="text-xs sm:text-sm text-white/90 leading-relaxed font-medium">
            {slide.description}
          </p>

          {/* Fatty Feature Highlight Chips */}
          <div className="flex flex-wrap gap-1.5 sm:gap-2 pt-1">
            {slide.features.map((feat) => (
              <span
                key={feat}
                className="inline-flex items-center gap-1.5 px-2.5 sm:px-3 py-1 rounded-xl text-[11px] sm:text-xs font-semibold bg-white/15 border border-white/20 text-white shadow-sm backdrop-blur-sm hover:scale-[1.02] hover:bg-white/25 transition-all"
              >
                <span className="h-1.5 w-1.5 rounded-full bg-white" />
                {feat}
              </span>
            ))}
          </div>
        </div>
      </div>

      {/* Bottom Fatty Tabs & Navigation Controls */}
      <div className="relative z-10 flex flex-col sm:flex-row items-center justify-between gap-3 sm:gap-4 pt-3 sm:pt-4 border-t border-white/20 max-w-xl w-full mx-auto">
        {/* Quick Clickable Feature Thumbnails */}
        <div className="flex items-center gap-2">
          {ONBOARDING_SLIDES.map((s, idx) => (
            <button
              key={s.id}
              type="button"
              onClick={() => setCurrentSlide(idx)}
              className={`h-2.5 rounded-full transition-all duration-300 ${
                idx === currentSlide
                  ? "w-10 bg-white shadow-sm shadow-black/10"
                  : "w-2.5 bg-white/40 hover:bg-white/70"
              }`}
              aria-label={`Go to ${s.title}`}
            />
          ))}
        </div>

        {/* Carousel Navigation Buttons */}
        <div className="flex items-center gap-2">
          <span className="text-[11px] text-white/80 font-semibold">
            {currentSlide + 1} of {ONBOARDING_SLIDES.length}
          </span>
          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="icon"
              onClick={handlePrev}
              className="h-8 w-8 rounded-xl bg-white/15 border-white/25 text-white hover:bg-white/25 hover:text-white shadow-sm hover:scale-105 transition-transform"
              aria-label="Previous slide"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={handleNext}
              className="h-8 w-8 rounded-xl bg-white/15 border-white/25 text-white hover:bg-white/25 hover:text-white shadow-sm hover:scale-105 transition-transform"
              aria-label="Next slide"
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
