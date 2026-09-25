import { useLayoutEffect, useRef } from "react";
import { gsap, revealUp } from "../lib/anim";
import { SectionTag, Counter } from "./shared";
import { Av } from "./ProductUI";
import {
  ShieldIcon,
  MonitorIcon,
  ClockIcon,
  UsersIcon,
  StarIcon,
} from "./icons";
import { INSTITUTION } from "../data/appData";

const STATS = [
  { to: 600, suffix: "+", label: "coaching centers running Coachify" },
  { to: 30000, suffix: "+", label: "student records managed daily" },
  { to: 5, suffix: " min", label: "average install-to-first-student time" },
  { to: 0, suffix: "", label: "cloud dependency. It is a desktop app" },
];

const TESTIMONIALS = [
  {
    quote:
      "The 7 PM queue is gone. Fee plus receipt takes under ten seconds now, and the parent gets a confirmation SMS before they even leave the counter.",
    name: "R. Sharma",
    org: `${INSTITUTION.name} · Patna`,
  },
  {
    quote:
      "Our internet drops at least two days a week. Coachify simply keeps working — fees, attendance, receipts, everything. That alone paid for it.",
    name: "Kavita Desai",
    org: "Deshi Science Academy · Indore",
  },
  {
    quote:
      "Result day used to be a three-week war. Now I enter marks once and report cards print in an afternoon. Even the office staff enjoy using it.",
    name: "Mohammed Ansari",
    org: "Ansari Coaching Centre · Hyderabad",
  },
];

const BADGES = [
  {
    icon: ShieldIcon,
    title: "Built for coaching centers",
    sub: "Not a generic business suite bent into shape",
  },
  {
    icon: MonitorIcon,
    title: "No complicated setup",
    sub: "Install on the PC you already have",
  },
  {
    icon: ClockIcon,
    title: "Ready for daily use",
    sub: "Built for the 7 PM rush, not the demo",
  },
  {
    icon: UsersIcon,
    title: "Practical for real staff",
    sub: "Reception can learn it in an afternoon",
  },
];

export function Trust() {
  const ref = useRef<HTMLElement>(null);

  useLayoutEffect(() => {
    if (!ref.current) return;
    const ctx = gsap.context(() => {
      revealUp(ref.current, "[data-reveal]", { stagger: 0.1, y: 32 });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section ref={ref} className="relative py-24 sm:py-32">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="max-w-3xl">
          <span data-reveal>
            <SectionTag>Trusted at the counter</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl"
          >
            Made for the people who <span className="text-orange-500">run</span>{" "}
            your center.
          </h2>
        </div>

        {/* stat band */}
        <div
          data-reveal
          className="mt-12 grid grid-cols-2 gap-px overflow-hidden rounded-3xl border border-stone-200/80 bg-stone-200/60 lg:grid-cols-4"
        >
          {STATS.map((s) => (
            <div key={s.label} className="bg-white p-6 sm:p-8">
              <p className="font-display text-4xl font-extrabold tracking-tight text-ink sm:text-5xl">
                <Counter to={s.to} suffix={s.suffix} />
              </p>
              <p className="mt-2 text-[13px] font-medium leading-snug text-stone-500">
                {s.label}
              </p>
            </div>
          ))}
        </div>

        {/* testimonials */}
        <div className="mt-14 grid gap-4 lg:grid-cols-3">
          {TESTIMONIALS.map((t, i) => (
            <figure
              key={t.name}
              data-reveal
              className="group flex flex-col rounded-3xl border border-stone-200/80 bg-white p-7 shadow-card transition-all duration-500 hover:-translate-y-1.5 hover:shadow-lift"
            >
              <div className="flex gap-1 text-orange-400">
                {Array.from({ length: 5 }).map((_, j) => (
                  <StarIcon key={j} className="h-3.5 w-3.5" />
                ))}
              </div>
              <blockquote className="mt-4 flex-1 text-[15px] leading-relaxed text-stone-600">
                &ldquo;{t.quote}&rdquo;
              </blockquote>
              <figcaption className="mt-6 flex items-center gap-3 border-t border-stone-100 pt-5">
                <Av name={t.name} i={i} size="h-10 w-10 text-xs" />
                <div>
                  <p className="text-sm font-bold text-ink">{t.name}</p>
                  <p className="text-xs font-medium text-stone-400">{t.org}</p>
                </div>
              </figcaption>
            </figure>
          ))}
        </div>

        {/* badges */}
        <div className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {BADGES.map((b) => (
            <div
              key={b.title}
              data-reveal
              className="flex items-start gap-3.5 rounded-2xl border border-stone-200/80 bg-white/70 p-5 transition-colors duration-300 hover:border-orange-300"
            >
              <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-orange-50 text-orange-600">
                <b.icon className="h-5 w-5" />
              </span>
              <div>
                <p className="text-sm font-bold text-ink">{b.title}</p>
                <p className="mt-1 text-xs leading-snug text-stone-400">
                  {b.sub}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
