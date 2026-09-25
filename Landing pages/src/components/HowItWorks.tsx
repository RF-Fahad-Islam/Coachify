import { memo, useLayoutEffect, useRef, useState } from "react";
import { gsap, prefersReduced, ScrollTrigger, revealUp } from "../lib/anim";
import { cn } from "../utils/cn";
import { SectionTag } from "./shared";
import {
  WindowFrame,
  SetupScene,
  AdmitScene,
  BatchesScene,
  AttendanceScreen,
  PaymentsScreen,
  ReceiptScene,
  ExamsScreen,
  ResultsScreen,
  NotifyScene,
  BackupScene,
} from "./ProductUI";

type Step = {
  label: string;
  title: string;
  desc: string;
  scene: React.ReactNode;
  frameTitle: string;
};

const STEPS: Step[] = [
  {
    label: "Set up",
    title: "Set up your institution",
    desc: "Name, logo, fee terms, branches and grading schemes. Fifteen minutes — done once, never repeated.",
    scene: <SetupScene />,
    frameTitle: "Coachify — Setup",
  },
  {
    label: "Admit",
    title: "Admit students",
    desc: "Guardian details, photo, batch and fee plan in one form. ID card and the first receipt print together.",
    scene: <AdmitScene />,
    frameTitle: "Coachify — Admission",
  },
  {
    label: "Batch",
    title: "Organize batches",
    desc: "Assign students to batches with subjects, teachers, timings and fee structures attached.",
    scene: <BatchesScene />,
    frameTitle: "Coachify — Batches",
  },
  {
    label: "Attend",
    title: "Take attendance",
    desc: "One tap per student, per batch. Daily, monthly and yearly summaries write themselves.",
    scene: <AttendanceScreen />,
    frameTitle: "Coachify — Attendance",
  },
  {
    label: "Collect",
    title: "Collect fees",
    desc: "Search by name or receipt number. Record installments, discounts and late charges without a second sheet.",
    scene: <PaymentsScreen />,
    frameTitle: "Coachify — Payments",
  },
  {
    label: "Receipt",
    title: "Print receipts",
    desc: "Thermal or A4, duplicate copy for your records, and any receipt can be reprinted years later.",
    scene: <ReceiptScene />,
    frameTitle: "Coachify — Receipts",
  },
  {
    label: "Exam",
    title: "Conduct exams",
    desc: "Schedule papers, enter marks once, and let the grading scheme do the arithmetic for you.",
    scene: <ExamsScreen />,
    frameTitle: "Coachify — Exams",
  },
  {
    label: "Results",
    title: "Publish results",
    desc: "Ranks, grades and full report cards — generated for the whole batch in a single click.",
    scene: <ResultsScreen />,
    frameTitle: "Coachify — Results",
  },
  {
    label: "Notify",
    title: "Send reminders",
    desc: "Fee confirmations, due reminders and absence alerts land on parents’ phones from your desk.",
    scene: <NotifyScene />,
    frameTitle: "Coachify — Notifications",
  },
  {
    label: "Backup",
    title: "Back up safely",
    desc: "One-click encrypted backups to a disk or pen drive. Restore a full year in minutes.",
    scene: <BackupScene />,
    frameTitle: "Coachify — Backup",
  },
];

const HowStoryPanels = memo(function HowStoryPanels() {
  return (
    <div className="relative h-[44vh] min-h-[340px] overflow-hidden sm:h-[48vh] sm:min-h-[420px]">
      {STEPS.map((st, idx) => (
        <div
          key={st.label}
          data-hw-panel
          style={{ zIndex: idx + 1 }}
          className="absolute inset-0 flex items-center justify-center overflow-hidden bg-white"
        >
          <div className="w-full max-w-[540px] overflow-hidden px-1">
            <WindowFrame
              title={st.frameTitle}
              className="max-h-full w-full max-sm:scale-[0.86] max-md:scale-[0.92]"
              bodyClassName="max-h-[36vh] overflow-hidden sm:max-h-[40vh]"
            >
              {st.scene}
            </WindowFrame>
          </div>
        </div>
      ))}
    </div>
  );
});

export function HowItWorks() {
  const sectionRef = useRef<HTMLElement>(null);
  const pinRef = useRef<HTMLDivElement>(null);
  const stRef = useRef<ScrollTrigger | null>(null);
  const idxRef = useRef(0);
  const numRef = useRef<HTMLSpanElement>(null);
  const descRef = useRef<HTMLParagraphElement>(null);
  const [active, setActive] = useState(0);
  const [reduced] = useState(() => prefersReduced());

  useLayoutEffect(() => {
    const section = sectionRef.current;
    if (!section) return;

    const ctx = gsap.context(() => {
      revealUp(section, "[data-reveal]", { stagger: 0.1 });
      if (reduced) return;

      const n = STEPS.length;
      const panels = gsap.utils.toArray<HTMLElement>(
        "[data-hw-panel]",
        section,
      );
      const rail = section.querySelector<HTMLElement>("[data-hw-rail-fill]");
      const railX = section.querySelector<HTMLElement>("[data-hw-rail-fill-x]");
      const railSetY = rail ? gsap.quickSetter(rail, "scaleY") : null;
      const railSetX = railX ? gsap.quickSetter(railX, "scaleX") : null;

      gsap.set(panels, {
        autoAlpha: 0,
        xPercent: 2.5,
        scale: 0.985,
        force3D: true,
        willChange: "transform, opacity",
      });
      gsap.set(panels[0], { autoAlpha: 1, xPercent: 0, scale: 1 });

      const tl = gsap.timeline({
        defaults: { ease: "sine.inOut", force3D: true },
        scrollTrigger: {
          trigger: pinRef.current,
          start: "top top",
          end: () => `+=${(n - 1) * (window.innerWidth < 768 ? 65 : 75)}%`,
          scrub: 0.16,
          pin: true,
          pinSpacing: true,
          anticipatePin: 1,
          invalidateOnRefresh: true,
          onUpdate: (self) => {
            const p = self.progress;
            railSetY?.(Math.max(0.02, p));
            railSetX?.(Math.max(0.02, p));
            const idx = Math.max(0, Math.min(n - 1, Math.round(p * (n - 1))));
            if (idx !== idxRef.current) {
              idxRef.current = idx;
              setActive(idx);
              if (numRef.current) {
                gsap.fromTo(
                  numRef.current,
                  { yPercent: 18, autoAlpha: 0.25 },
                  {
                    yPercent: 0,
                    autoAlpha: 1,
                    duration: 0.55,
                    ease: "power2.out",
                    overwrite: true,
                  },
                );
              }
              if (descRef.current) {
                gsap.fromTo(
                  descRef.current,
                  { y: 10, autoAlpha: 0.25 },
                  {
                    y: 0,
                    autoAlpha: 1,
                    duration: 0.55,
                    ease: "power2.out",
                    overwrite: true,
                  },
                );
              }
            }
          },
        },
      });

      // Continuous overlapping cross-glide centered at mid = i - 0.5 so panels[i]
      // glides smoothly over panels[i-1] without any white flash or dead zone.
      for (let i = 1; i < n; i++) {
        const mid = i - 0.5;
        tl.fromTo(
          panels[i],
          { autoAlpha: 0, xPercent: 2.5, scale: 0.985 },
          {
            autoAlpha: 1,
            xPercent: 0,
            scale: 1,
            duration: 0.68,
            ease: "sine.inOut",
          },
          mid - 0.34,
        );
        tl.to(
          panels[i - 1],
          {
            autoAlpha: 0,
            xPercent: -2,
            scale: 0.985,
            duration: 0.56,
            ease: "sine.inOut",
          },
          mid - 0.22,
        );
      }
      tl.set({}, {}, n - 1);

      stRef.current = tl.scrollTrigger ?? null;
    }, section);
    return () => ctx.revert();
  }, [reduced]);

  const goTo = (i: number) => {
    const st = stRef.current;
    if (!st) return;
    const target = st.start + (i / (STEPS.length - 1)) * (st.end - st.start);
    window.scrollTo({
      top: target,
      behavior: prefersReduced() ? "auto" : "smooth",
    });
  };

  const s = STEPS[active];

  /* ---- reduced motion: simple stacked layout ---- */
  if (reduced) {
    return (
      <section
        id="how"
        ref={sectionRef}
        className="scroll-mt-24 bg-white py-24 sm:py-32"
      >
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="max-w-3xl">
            <SectionTag>The walkthrough</SectionTag>
            <h2 className="mt-5 font-display text-4xl font-extrabold tracking-tight text-ink sm:text-5xl">
              From opening day to results day.
            </h2>
          </div>
          <div className="mt-12 space-y-10">
            {STEPS.map((step, i) => (
              <div
                key={step.label}
                className="grid gap-6 lg:grid-cols-[0.8fr_1.2fr] lg:items-center"
              >
                <div>
                  <p className="tnum font-display text-5xl font-extrabold text-orange-500/30">
                    {String(i + 1).padStart(2, "0")}
                  </p>
                  <h3 className="mt-2 font-display text-2xl font-bold text-ink">
                    {step.title}
                  </h3>
                  <p className="mt-2 max-w-md text-[15px] leading-relaxed text-stone-500">
                    {step.desc}
                  </p>
                </div>
                <WindowFrame title={step.frameTitle}>{step.scene}</WindowFrame>
              </div>
            ))}
          </div>
        </div>
      </section>
    );
  }

  /* ---- scroll-pinned story ---- */
  return (
    <section id="how" ref={sectionRef} className="scroll-mt-24 bg-white">
      <div
        className="mx-auto max-w-7xl px-4 pb-16 pt-24 sm:px-6 sm:pt-32 lg:px-8"
        data-reveal-group
      >
        <div className="flex flex-wrap items-end justify-between gap-6">
          <div className="max-w-3xl">
            <span data-reveal>
              <SectionTag>The walkthrough</SectionTag>
            </span>
            <h2
              data-reveal
              className="mt-5 font-display text-4xl font-extrabold leading-[1.05] tracking-tight text-ink sm:text-5xl lg:text-[54px]"
            >
              From opening day{" "}
              <span className="text-orange-500">to results day.</span>
            </h2>
          </div>
          <p
            data-reveal
            className="max-w-xs pb-2 text-sm leading-relaxed text-stone-400"
          >
            Keep scrolling — a full year of your center, in ten moves.
          </p>
        </div>
      </div>

      <div
        ref={pinRef}
        className="relative z-10 h-screen overflow-hidden bg-white"
      >
        <div className="mx-auto flex h-full max-w-7xl flex-col justify-center gap-8 px-4 pb-10 sm:px-6 lg:flex-row lg:items-center lg:gap-14 lg:px-8">
          {/* step rail */}
          <div className="relative flex-1 lg:max-w-sm">
            <div className="mb-6 flex items-center gap-4">
              <span
                ref={numRef}
                className="tnum font-display text-6xl font-extrabold text-orange-500 sm:text-7xl"
              >
                {String(active + 1).padStart(2, "0")}
              </span>
              <div>
                <p className="text-[11px] font-bold uppercase tracking-[0.2em] text-stone-400">
                  Step of 10
                </p>
                <p className="font-display text-lg font-bold text-ink">
                  {s.label}
                </p>
              </div>
            </div>
            <div className="relative hidden h-[46vh] lg:block">
              <div className="absolute top-1 left-[15px] bottom-1 w-0.5 rounded-full bg-stone-200" />
              <div
                data-hw-rail-fill
                className="absolute top-1 left-[15px] bottom-1 w-0.5 origin-top scale-y-[0.02] rounded-full bg-gradient-to-b from-orange-400 to-orange-600"
              />
              <div className="flex h-full flex-col justify-between">
                {STEPS.map((st, i) => (
                  <button
                    key={st.label}
                    onClick={() => goTo(i)}
                    className={cn(
                      "group flex cursor-pointer items-center gap-3 rounded-lg py-0.5 text-left transition-colors",
                      i === active
                        ? "text-ink"
                        : "text-stone-400 hover:text-stone-600",
                    )}
                  >
                    <span
                      className={cn(
                        "tnum z-10 flex h-7 w-7 items-center justify-center rounded-full border-2 bg-white text-[10px] font-bold transition-all duration-300",
                        i === active
                          ? "scale-110 border-orange-500 text-orange-600"
                          : i < active
                            ? "border-orange-300 text-orange-400"
                            : "border-stone-200 text-stone-400",
                      )}
                    >
                      {String(i + 1).padStart(2, "0")}
                    </span>
                    <span
                      className={cn(
                        "text-[13px] font-semibold transition-all duration-300",
                        i === active && "translate-x-1 font-bold",
                      )}
                    >
                      {st.title}
                    </span>
                  </button>
                ))}
              </div>
            </div>
            {/* mobile progress */}
            <div className="lg:hidden">
              <div className="h-1.5 overflow-hidden rounded-full bg-stone-200">
                <div
                  className="h-full w-full origin-left scale-x-[0.02] rounded-full bg-orange-500"
                  data-hw-rail-fill-x
                />
              </div>
              <p className="mt-3 text-sm leading-relaxed text-stone-500">
                {s.desc}
              </p>
            </div>
          </div>

          {/* story panel */}
          <div className="relative flex-1">
            <div className="mb-4 lg:hidden" />
            <HowStoryPanels />
            <div className="mt-4 hidden text-center lg:block">
              <p
                ref={descRef}
                className="mx-auto max-w-xl text-[15px] leading-relaxed text-stone-500"
              >
                {s.desc}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
