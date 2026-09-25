import { useLayoutEffect, useRef } from "react";
import { gsap, prefersReduced, revealUp } from "../lib/anim";
import { SectionTag } from "./shared";
import { WindowFrame, DashboardScreen } from "./ProductUI";
import {
  CheckIcon,
  ArrowRightIcon,
  WifiOffIcon,
  MonitorIcon,
  BanknoteIcon,
  AttendanceIcon,
} from "./icons";

const MODULES = [
  "Admissions",
  "Student records",
  "Batches & teachers",
  "Attendance",
  "Fee collection & receipts",
  "Dues tracking",
  "Exams & results",
  "SMS & email",
  "Backup & restore",
];

export function Solution() {
  const ref = useRef<HTMLElement>(null);

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root) return;

    const ctx = gsap.context(() => {
      revealUp(root, "[data-reveal]", { stagger: 0.13, dur: 1.18 });
      if (prefersReduced()) return;

      const solMock = root.querySelectorAll<HTMLElement>(".sol-mock");
      const checkItems = Array.from(
        root.querySelectorAll<HTMLElement>(".sol-check-item"),
      );
      const solChecks = root.querySelectorAll<HTMLElement>(".sol-check");
      const solFloats = root.querySelectorAll<HTMLElement>(".sol-float");

      checkItems.forEach((el) => {
        el.dataset.gsapActive = "true";
      });

      gsap.set(solMock, { y: 44, autoAlpha: 0, scale: 0.965, force3D: true });
      gsap.set(checkItems, { y: 20, autoAlpha: 0, force3D: true });
      gsap.set(solChecks, { scale: 0, force3D: true });
      gsap.set(solFloats, { y: 22, autoAlpha: 0, force3D: true });

      gsap.to(solMock, {
        y: 0,
        autoAlpha: 1,
        scale: 1,
        duration: 1.25,
        ease: "power2.out",
        force3D: true,
        clearProps: "transform",
        scrollTrigger: {
          trigger: root.querySelector(".sol-mockwrap"),
          start: "top 82%",
          once: true,
        },
      });

      gsap.to(checkItems, {
        y: 0,
        autoAlpha: 1,
        duration: 0.95,
        stagger: 0.075,
        ease: "power2.out",
        force3D: true,
        onComplete: () => {
          gsap.set(checkItems, { clearProps: "transform" });
          if (
            typeof window !== "undefined" &&
            typeof window.requestAnimationFrame === "function"
          ) {
            window.requestAnimationFrame(() => {
              checkItems.forEach((el) => {
                delete el.dataset.gsapActive;
              });
            });
          }
        },
        scrollTrigger: {
          trigger: root.querySelector(".sol-checklist"),
          start: "top 88%",
          once: true,
        },
      });

      gsap.to(solChecks, {
        scale: 1,
        duration: 0.65,
        stagger: 0.075,
        ease: "back.out(1.8)",
        delay: 0.12,
        force3D: true,
        clearProps: "transform",
        scrollTrigger: {
          trigger: root.querySelector(".sol-checklist"),
          start: "top 88%",
          once: true,
        },
      });

      gsap.to(solFloats, {
        y: 0,
        autoAlpha: 1,
        duration: 1.05,
        stagger: 0.14,
        ease: "power2.out",
        delay: 0.3,
        force3D: true,
        clearProps: "transform",
        scrollTrigger: {
          trigger: root.querySelector(".sol-mockwrap"),
          start: "top 78%",
          once: true,
        },
      });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section ref={ref} className="relative overflow-x-clip py-24 sm:py-32">
      {/* Rich layered background */}
      <div className="absolute inset-0 -z-10 bg-gradient-to-b from-cream via-orange-50/40 to-cream" />
      <div className="absolute inset-0 -z-10 bg-grid-warm opacity-50 [mask-image:radial-gradient(ellipse_80%_70%_at_50%_40%,black,transparent)]" />
      <div className="absolute -left-24 top-20 h-72 w-72 rounded-full bg-orange-200/35 blur-[90px]" />
      <div className="absolute -right-20 bottom-10 h-80 w-80 rounded-full bg-amber-200/30 blur-[100px]" />
      <div className="absolute left-1/2 top-1/3 h-64 w-[520px] -translate-x-1/2 rounded-full bg-orange-100/40 blur-[80px]" />

      {/* Decorative rings & dots */}
      <div className="pointer-events-none absolute right-[8%] top-24 hidden h-40 w-40 rounded-full border border-dashed border-orange-300/50 lg:block" />
      <div className="pointer-events-none absolute bottom-32 left-[6%] hidden h-24 w-24 rounded-full border border-orange-200/60 lg:block" />
      <div className="pointer-events-none absolute right-[18%] top-[42%] h-3 w-3 rounded-full bg-orange-400/50" />
      <div className="pointer-events-none absolute left-[14%] top-[58%] h-2 w-2 rounded-full bg-amber-400/60" />
      <div className="pointer-events-none absolute right-[12%] bottom-[22%] h-2.5 w-2.5 rounded-full bg-orange-500/40" />

      {/* Soft diagonal stripe accent */}
      <div
        className="pointer-events-none absolute inset-y-0 right-0 w-1/3 opacity-[0.035]"
        style={{
          backgroundImage:
            "repeating-linear-gradient(-32deg, #F97316 0 2px, transparent 2px 18px)",
        }}
      />

      <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="grid gap-14 lg:grid-cols-[0.9fr_1.1fr] lg:gap-16">
          {/* sticky story column */}
          <div data-reveal-group className="lg:sticky lg:top-28 lg:self-start">
            <span data-reveal>
              <SectionTag>The turn</SectionTag>
            </span>
            <h2
              data-reveal
              className="mt-5 font-display text-4xl font-extrabold leading-[1.05] tracking-tight text-ink sm:text-5xl lg:text-[54px]"
            >
              From daily chaos{" "}
              <span className="mx-1 inline-block font-black text-orange-500">
                →
              </span>{" "}
              to operational <span className="text-orange-500">clarity.</span>
            </h2>
            <p
              data-reveal
              className="mt-6 max-w-lg text-[17px] leading-relaxed text-stone-500"
            >
              Coachify is one desktop application that holds your entire center.
              Admissions flow in, students stay organized, attendance is
              captured in seconds, fees are collected and receipted instantly,
              and exams publish without a single mark being re-typed.
            </p>
            <p
              data-reveal
              className="mt-4 max-w-lg text-[17px] leading-relaxed text-stone-500"
            >
              Every taka, every roll mark, every guardian&rsquo;s number —
              always one click away, always on your machine.
            </p>

            {/* Mini proof chips */}
            <div data-reveal className="mt-7 flex flex-wrap gap-2.5">
              {[
                [WifiOffIcon, "Works offline"],
                [MonitorIcon, "Local PC"],
                [BanknoteIcon, "Instant receipts"],
                [AttendanceIcon, "Fast attendance"],
              ].map(([Icon, label]) => {
                const I = Icon as typeof WifiOffIcon;
                return (
                  <span
                    key={label as string}
                    className="inline-flex items-center gap-2 rounded-full border border-orange-200/80 bg-white/80 px-3.5 py-2 text-[12.5px] font-semibold text-stone-600 shadow-sm backdrop-blur-sm"
                  >
                    <I className="h-3.5 w-3.5 text-orange-500" />
                    {label as string}
                  </span>
                );
              })}
            </div>

            <a
              data-reveal
              href="#how"
              className="group mt-8 inline-flex items-center gap-2 text-[15px] font-bold text-orange-600 transition hover:text-orange-700"
            >
              See a full day inside Coachify
              <span className="transition-transform duration-300 group-hover:translate-x-1.5">
                <ArrowRightIcon className="h-4.5 w-4.5" />
              </span>
            </a>
          </div>

          {/* assembling mockup with floating badges */}
          <div className="sol-mockwrap relative">
            <div className="absolute -inset-6 -z-10 rounded-[40px] bg-gradient-to-br from-orange-100/80 via-white/40 to-amber-100/60" />
            <div className="absolute -right-4 -top-4 -z-10 h-28 w-28 rounded-full border border-dashed border-orange-300/60" />
            <div className="absolute -bottom-6 -left-6 -z-10 h-20 w-20 rounded-full bg-orange-400/10" />

            <div className="sol-mock relative overflow-hidden">
              <WindowFrame
                title="Coachify — everything, one place"
                bodyClassName="overflow-hidden"
              >
                <DashboardScreen />
              </WindowFrame>
              <span className="absolute -top-3.5 left-6 rounded-full bg-ink px-3.5 py-1.5 text-[11px] font-bold text-white shadow-md">
                One system. Every workflow.
              </span>
            </div>

            {/* floating metric cards */}
            <div className="sol-float absolute -left-3 bottom-16 z-10 hidden w-40 rounded-2xl border border-stone-200/80 bg-white p-3 shadow-lift sm:block lg:-left-8">
              <p className="text-[10px] font-bold uppercase tracking-wider text-stone-400">
                Today
              </p>
              <p className="tnum mt-1 font-display text-xl font-extrabold text-ink">
                ৳48.2K
              </p>
              <p className="text-[11px] font-semibold text-emerald-600">
                fees collected
              </p>
            </div>
            <div className="sol-float absolute -right-2 top-24 z-10 hidden w-36 rounded-2xl border border-stone-200/80 bg-white p-3 shadow-lift sm:block lg:-right-6">
              <p className="text-[10px] font-bold uppercase tracking-wider text-stone-400">
                Attendance
              </p>
              <p className="tnum mt-1 font-display text-xl font-extrabold text-ink">
                94%
              </p>
              <p className="text-[11px] font-semibold text-orange-600">
                across 12 batches
              </p>
            </div>
          </div>
        </div>

        {/* module checklist */}
        <div className="sol-checklist relative mt-20">
          <div className="absolute inset-x-0 top-1/2 -z-10 h-px bg-gradient-to-r from-transparent via-orange-200 to-transparent" />
          <p className="text-center text-sm font-semibold uppercase tracking-[0.18em] text-stone-400">
            Everything a coaching center runs on
          </p>
          <div className="mx-auto mt-6 flex max-w-4xl flex-wrap justify-center gap-2.5">
            {MODULES.map((m) => (
              <span
                key={m}
                className="sol-check-item inline-flex items-center gap-2 rounded-full border border-stone-200/90 bg-white/90 px-4 py-2.5 text-sm font-semibold text-stone-700 shadow-sm backdrop-blur-sm transition-all duration-300 hover:-translate-y-0.5 hover:border-orange-300 hover:bg-orange-50 hover:text-orange-700"
              >
                <span className="sol-check flex h-4.5 w-4.5 items-center justify-center rounded-full bg-orange-500 text-white">
                  <CheckIcon className="h-2.5 w-2.5" />
                </span>
                {m}
              </span>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
