import { memo, useLayoutEffect, useRef, useState } from "react";
import { gsap, prefersReduced, ScrollTrigger, revealUp } from "../lib/anim";
import { cn } from "../utils/cn";
import { SectionTag } from "./shared";
import {
  WindowFrame,
  DashboardScreen,
  StudentsScreen,
  AttendanceScreen,
  PaymentsScreen,
  DuesScreen,
  ExamsScreen,
  ResultsScreen,
  ReportsScreen,
} from "./ProductUI";

const SCREENS = [
  {
    name: "Dashboard",
    caption:
      "The morning pulse of your center — today’s classes, fees in, dues waiting.",
    node: <DashboardScreen />,
  },
  {
    name: "Students",
    caption:
      "Every student, guardian and payment history — searchable in seconds.",
    node: <StudentsScreen />,
  },
  {
    name: "Attendance",
    caption: "Roll call in under a minute, summaries that write themselves.",
    node: <AttendanceScreen />,
  },
  {
    name: "Payments",
    caption: "Counter-speed collection with instant thermal or A4 receipts.",
    node: <PaymentsScreen />,
  },
  {
    name: "Dues",
    caption: "Know exactly who owes what, down to the last taka.",
    node: <DuesScreen />,
  },
  {
    name: "Exams",
    caption: "Papers, schedules and one-pass mark entry in a single place.",
    node: <ExamsScreen />,
  },
  {
    name: "Results",
    caption: "Grading, ranks and report cards — published for the whole batch.",
    node: <ResultsScreen />,
  },
  {
    name: "Reports",
    caption: "Print-ready reports for fees, attendance, students and exams.",
    node: <ReportsScreen />,
  },
];

const ScreenPanels = memo(function ScreenPanels() {
  return (
    <>
      {SCREENS.map((sc, idx) => (
        <div
          key={sc.name}
          data-screen-panel
          style={{ zIndex: idx + 1 }}
          className="absolute inset-0 h-full overflow-hidden bg-white"
        >
          {sc.node}
        </div>
      ))}
    </>
  );
});

export function Screens() {
  const ref = useRef<HTMLElement>(null);
  const pinRef = useRef<HTMLDivElement>(null);
  const stRef = useRef<ScrollTrigger | null>(null);
  const idxRef = useRef(0);
  const [active, setActive] = useState(0);
  const [reduced] = useState(() => prefersReduced());

  useLayoutEffect(() => {
    const section = ref.current;
    if (!section) return;

    const ctx = gsap.context(() => {
      revealUp(section, "[data-reveal]", { stagger: 0.1 });
      if (reduced) return;

      const n = SCREENS.length;
      const panels = gsap.utils.toArray<HTMLElement>(
        "[data-screen-panel]",
        section,
      );
      gsap.set(panels, {
        autoAlpha: 0,
        yPercent: 2.2,
        scale: 0.988,
        transformOrigin: "50% 50%",
        force3D: true,
        willChange: "transform, opacity",
      });
      gsap.set(panels[0], { autoAlpha: 1, yPercent: 0, scale: 1 });

      const tl = gsap.timeline({
        defaults: { ease: "sine.inOut", force3D: true },
        scrollTrigger: {
          trigger: pinRef.current,
          start: "top top",
          end: () => `+=${(n - 1) * (window.innerWidth < 768 ? 62 : 72)}%`,
          scrub: 0.16,
          pin: true,
          pinSpacing: true,
          anticipatePin: 1,
          invalidateOnRefresh: true,
          onUpdate: (self) => {
            const idx = Math.max(
              0,
              Math.min(n - 1, Math.round(self.progress * (n - 1))),
            );
            if (idx !== idxRef.current) {
              idxRef.current = idx;
              setActive(idx);
            }
          },
        },
      });

      for (let i = 1; i < n; i++) {
        const mid = i - 0.5;
        tl.fromTo(
          panels[i],
          { autoAlpha: 0, yPercent: 2.2, scale: 0.988 },
          {
            autoAlpha: 1,
            yPercent: 0,
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
            yPercent: -1.8,
            scale: 0.988,
            duration: 0.56,
            ease: "sine.inOut",
          },
          mid - 0.22,
        );
      }
      tl.set({}, {}, n - 1);

      stRef.current = tl.scrollTrigger ?? null;
    }, ref);
    return () => ctx.revert();
  }, [reduced]);

  const goTo = (i: number) => {
    if (reduced) {
      setActive(i);
      return;
    }
    const st = stRef.current;
    if (!st) return;
    const target = st.start + (i / (SCREENS.length - 1)) * (st.end - st.start);
    window.scrollTo({ top: target, behavior: "smooth" });
  };

  const s = SCREENS[active];

  return (
    <section
      id="screens"
      ref={ref}
      className="relative scroll-mt-24 overflow-hidden py-24 sm:py-32"
    >
      <div className="absolute inset-0 -z-10 bg-dots-warm [mask-image:radial-gradient(ellipse_75%_60%_at_50%_40%,black,transparent)] opacity-70" />
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8" data-reveal-group>
        <div className="mx-auto max-w-3xl text-center">
          <span data-reveal>
            <SectionTag>Inside the product</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl lg:text-[54px]"
          >
            Eight screens. <span className="text-orange-500">Every</span>{" "}
            workflow covered.
          </h2>
          <p
            data-reveal
            className="mt-5 text-lg leading-relaxed text-stone-500"
          >
            Scroll to move through the app — or tap a screen to jump straight to
            it.
          </p>
        </div>
      </div>

      <div
        ref={pinRef}
        className="relative z-10 h-screen overflow-hidden bg-cream"
      >
        <div className="mx-auto flex h-full max-w-5xl flex-col justify-center px-4 sm:px-6">
          {/* tabs */}
          <div className="no-scrollbar -mx-4 mb-5 flex gap-2 overflow-x-auto px-4 sm:mx-0 sm:justify-center sm:px-0">
            {SCREENS.map((sc, i) => (
              <button
                key={sc.name}
                onClick={() => goTo(i)}
                className={cn(
                  "shrink-0 cursor-pointer rounded-full border px-4 py-2 text-[13px] font-semibold transition-colors duration-200",
                  i === active
                    ? "border-orange-500 bg-orange-500 text-white shadow-lg shadow-orange-500/25"
                    : "border-stone-200 bg-white text-stone-500 hover:border-stone-300 hover:text-stone-700",
                )}
              >
                {sc.name}
              </button>
            ))}
          </div>

          <div className="relative mx-auto w-full max-w-[880px]">
            <div className="absolute -inset-4 -z-10 rounded-[32px] bg-gradient-to-tr from-orange-100/70 via-transparent to-amber-100/50" />
            <WindowFrame
              title={`Coachify — ${s.name}`}
              className="w-full"
              bodyClassName="relative h-[380px] overflow-hidden sm:h-[420px]"
            >
              {reduced ? (
                <div className="absolute inset-0 h-full overflow-hidden">
                  {s.node}
                </div>
              ) : (
                <ScreenPanels />
              )}
            </WindowFrame>
          </div>

          {/* caption */}
          <div className="mt-5 flex items-center justify-between gap-4">
            <p className="tnum font-display text-lg font-bold text-ink">
              <span className="text-orange-500">
                {String(active + 1).padStart(2, "0")}
              </span>
              <span className="text-stone-300"> / {SCREENS.length}</span>
              <span className="ml-3 text-base font-semibold text-stone-600">
                {s.name}
              </span>
            </p>
            <p className="hidden max-w-md text-right text-sm text-stone-400 sm:block">
              {s.caption}
            </p>
          </div>
          <p className="mt-1 text-sm text-stone-400 sm:hidden">{s.caption}</p>
        </div>
      </div>
    </section>
  );
}
