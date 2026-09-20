import { useEffect, useState } from "react";
import {
  Zap,
  Database,
  Receipt,
  ShieldCheck,
  ChevronLeft,
  ChevronRight,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

interface OnboardingSlide {
  id: string;
  badge: string;
  title: string;
  subtitle: string;
  description: string;
  icon: typeof Zap;
  features: string[];
}

const ONBOARDING_SLIDES: OnboardingSlide[] = [
  {
    id: "offline-speed",
    badge: "Local-First Architecture",
    title: "Instant Counter Speed",
    subtitle: "Zero cloud latency · 100% offline ready",
    description:
      "Powered by an embedded SQLite engine with WAL mode. Perform admissions, record marks, and search students with sub-millisecond local queries—even during internet outages.",
    icon: Database,
    features: ["Local SQLite Engine", "Zero Network Lag", "Works 100% Offline"],
  },
  {
    id: "attendance-ledger",
    badge: "Academics & Cashier",
    title: "Attendance & Poisha Ledger",
    subtitle: "High-density matrix · Integer poisha receipts",
    description:
      "Take daily batch attendance in seconds with keyboard shortcuts. Collect tuition fees with exact integer poisha precision and generate print-ready vouchers with one click.",
    icon: Receipt,
    features: ["Rapid Batch Attendance", "Poisha Exact Balance", "Offline Receipt Print"],
  },
  {
    id: "security-audit",
    badge: "Enterprise Protection",
    title: "Tamper-Proof Security",
    subtitle: "Argon2id hashing · Hardware clock guards",
    description:
      "Built with banking-grade protections: Argon2id password hashing, automatic session idle-lock, dual UTC & Dhaka business timestamps, and SQLite immutability triggers.",
    icon: ShieldCheck,
    features: [
      "Argon2id Cryptography",
      "Dual Clock Timestamps",
      "Trigger-Protected Audit",
    ],
  },
];

export function OnboardingCarousel() {
  const [currentSlide, setCurrentSlide] = useState(0);
  const [isPaused, setIsPaused] = useState(false);

  useEffect(() => {
    if (isPaused) return;

    const timer = setInterval(() => {
      setCurrentSlide((prev) => (prev + 1) % ONBOARDING_SLIDES.length);
    }, 6000);

    return () => clearInterval(timer);
  }, [isPaused]);

  const slide = ONBOARDING_SLIDES[currentSlide];
  const IconComponent = slide.icon;

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
      className="relative flex flex-col justify-between h-full p-8 select-none"
      onMouseEnter={() => setIsPaused(true)}
      onMouseLeave={() => setIsPaused(false)}
    >
      {/* Top Banner Tag */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-primary-foreground font-bold text-xs shadow-sm">
            C
          </div>
          <span className="text-sm font-bold tracking-tight text-foreground">
            Coachify Desktop
          </span>
        </div>
        <Badge variant="outline" className="text-[11px] gap-1 px-2 py-0.5 font-medium">
          <Sparkles className="h-3 w-3 text-primary" />
          <span>v0.1.0 Offline Core</span>
        </Badge>
      </div>

      {/* Main Slide Content */}
      <div className="my-auto py-8 space-y-6 max-w-lg">
        {/* Slide Visual Icon Badge */}
        <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10 border border-primary/20 text-primary shadow-inner">
          <IconComponent className="h-8 w-8" />
        </div>

        {/* Badge & Headings */}
        <div className="space-y-2">
          <span className="inline-block text-xs font-semibold uppercase tracking-wider text-primary">
            {slide.badge}
          </span>
          <h2 className="text-2xl sm:text-3xl font-bold tracking-tight text-foreground">
            {slide.title}
          </h2>
          <p className="text-xs font-medium text-muted-foreground">{slide.subtitle}</p>
        </div>

        {/* Description */}
        <p className="text-xs sm:text-sm text-muted-foreground leading-relaxed">
          {slide.description}
        </p>

        {/* Feature Pills */}
        <div className="flex flex-wrap gap-2 pt-2">
          {slide.features.map((feature) => (
            <span
              key={feature}
              className="inline-flex items-center px-2.5 py-1 rounded-md text-[11px] font-medium bg-muted/70 text-foreground border border-border/60"
            >
              ✓ {feature}
            </span>
          ))}
        </div>
      </div>

      {/* Slide Navigation & Indicators */}
      <div className="flex items-center justify-between pt-4 border-t border-border/40">
        {/* Step Indicators */}
        <div className="flex items-center gap-2">
          {ONBOARDING_SLIDES.map((s, idx) => (
            <button
              key={s.id}
              type="button"
              onClick={() => setCurrentSlide(idx)}
              className={`h-2 rounded-full transition-all duration-300 ${
                idx === currentSlide
                  ? "w-8 bg-primary"
                  : "w-2 bg-muted-foreground/30 hover:bg-muted-foreground/50"
              }`}
              aria-label={`Go to slide ${idx + 1}`}
            />
          ))}
        </div>

        {/* Prev / Next Buttons */}
        <div className="flex items-center gap-1.5">
          <Button
            variant="ghost"
            size="icon"
            onClick={handlePrev}
            className="h-8 w-8 rounded-full text-muted-foreground hover:text-foreground"
            aria-label="Previous slide"
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={handleNext}
            className="h-8 w-8 rounded-full text-muted-foreground hover:text-foreground"
            aria-label="Next slide"
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
