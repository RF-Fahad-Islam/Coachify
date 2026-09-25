import { useLayoutEffect, useRef } from "react";
import { gsap, prefersReduced, revealUp } from "../lib/anim";
import { SectionTag } from "./shared";
import { ArrowRightIcon } from "./icons";

const PAINS = [
  {
    n: "01",
    title: "Records are scattered everywhere",
    copy: "Admission forms in one register, fees in another, phone numbers on a sticky note by the desk. When a parent asks for “this month’s balance,” somebody digs for fifteen minutes.",
    cost: "15 minutes per question",
    notes: ["Rohan’s balance??", "sheet 3 missing", "call the 4th one again"],
    rot: "-0.5deg",
  },
  {
    n: "02",
    title: "Attendance is still pen and paper",
    copy: "Roll call eats the first ten minutes of every class. Sheets get lost, days get forgotten, and the monthly summary is rebuilt from memory at month-end.",
    cost: "10 minutes per class, daily",
    notes: ["Mon: 29? 31?", "who was absent on the 12th?", "redraw the sheet"],
    rot: "0.6deg",
  },
  {
    n: "03",
    title: "Fee chaos at the counter",
    copy: "The queue forms before 7 PM. Handwritten receipts, cash that never quite tallies, and due dates that slip past because only a few parents ever get chased.",
    cost: "Long queue = lost trust",
    notes: ["4500 — cash", "receipt book 2: EMPTY", "remind who? …all of them"],
    rot: "-0.4deg",
  },
  {
    n: "04",
    title: "Results take weeks, not days",
    copy: "Marks live in registers, then get re-typed into spreadsheets, then re-checked against the notebook. Parents keep asking — and the report card still sits unfinished.",
    cost: "2–3 weeks per result cycle",
    notes: [
      "re-enter marks??",
      "checking for the 3rd time",
      "parents calling daily",
    ],
    rot: "0.5deg",
  },
  {
    n: "05",
    title: "Your data lives in fragile files",
    copy: "One crashed laptop, one missing pen drive, one week without internet — and a year of records is at risk. There is no backup, no restore, no plan B.",
    cost: "One bad day can cost a year",
    notes: ["where’s the USB?", "no backup since June", "internet down again…"],
    rot: "-0.6deg",
  },
];

export function Problem() {
  const ref = useRef<HTMLElement>(null);
  const stickyRefs = useRef<(HTMLDivElement | null)[]>([]);
  const innerRefs = useRef<(HTMLDivElement | null)[]>([]);

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root) return;
    const ctx = gsap.context(() => {
      revealUp(root, "[data-reveal]", { stagger: 0.1 });
      if (prefersReduced()) return;

      innerRefs.current.forEach((inner, i) => {
        if (!inner || i === innerRefs.current.length - 1) return;
        const nextSticky = stickyRefs.current[i + 1];
        if (!nextSticky) return;
        gsap.fromTo(
          inner,
          { scale: 1, y: 0 },
          {
            scale: 0.96,
            y: -12,
            ease: "none",
            force3D: true,
            transformOrigin: "50% 0%",
            scrollTrigger: {
              trigger: nextSticky,
              start: "top 85%",
              end: "top 22%",
              scrub: 0.15,
              invalidateOnRefresh: true,
            },
          },
        );
      });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section ref={ref} className="relative py-24 sm:py-32">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="max-w-3xl" data-reveal-group>
          <span data-reveal>
            <SectionTag>The daily reality</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl lg:text-[56px]"
          >
            Running a coaching center shouldn&rsquo;t feel like a{" "}
            <span className="relative whitespace-nowrap">
              rescue operation.
              <svg
                className="absolute -bottom-2 left-0 w-full"
                viewBox="0 0 300 12"
                fill="none"
                preserveAspectRatio="none"
              >
                <path
                  d="M2 9c48-6 158-8 296-3"
                  stroke="#F97316"
                  strokeWidth="4"
                  strokeLinecap="round"
                  opacity="0.5"
                />
              </svg>
            </span>
          </h2>
          <p
            data-reveal
            className="mt-5 max-w-2xl text-lg leading-relaxed text-stone-500"
          >
            The academics are yours to handle. But every single day, the same
            admin fights consume the people who should be teaching.
          </p>
        </div>
      </div>

      <div className="mx-auto mt-14 max-w-5xl px-4 sm:px-6 lg:px-8">
        <div>
          {PAINS.map((p, i) => (
            <div
              key={p.n}
              className="sticky mb-6"
              style={{ top: `${96 + i * 16}px`, zIndex: i + 1 }}
              ref={(el) => {
                stickyRefs.current[i] = el;
              }}
            >
              <div
                ref={(el) => {
                  innerRefs.current[i] = el;
                }}
                className="relative overflow-hidden rounded-3xl border border-stone-200/90 bg-white p-6 shadow-card will-change-transform sm:p-9"
              >
                <span className="pointer-events-none absolute -top-4 right-4 font-display text-[120px] font-extrabold leading-none text-stone-100 select-none sm:text-[150px]">
                  {p.n}
                </span>

                <div className="relative grid gap-8 md:grid-cols-[1.15fr_1fr] md:items-center">
                  <div>
                    <span className="inline-flex items-center gap-2 rounded-full bg-rose-50 px-3 py-1 text-[11px] font-bold uppercase tracking-[0.14em] text-rose-500">
                      <span className="h-1.5 w-1.5 rounded-full bg-rose-400" />{" "}
                      Pain point {p.n}
                    </span>
                    <h3 className="mt-4 font-display text-2xl font-bold tracking-tight text-ink sm:text-[28px]">
                      {p.title}
                    </h3>
                    <p className="mt-3 max-w-md text-[15px] leading-relaxed text-stone-500">
                      {p.copy}
                    </p>
                    <span className="mt-5 inline-flex items-center gap-2 rounded-lg bg-stone-100 px-3 py-1.5 text-xs font-semibold text-stone-600">
                      Cost: <span className="text-stone-800">{p.cost}</span>
                    </span>
                  </div>

                  {/* scattered desk notes */}
                  <div
                    className="relative h-44 select-none sm:h-52"
                    aria-hidden="true"
                  >
                    <div className="absolute inset-0 rounded-2xl bg-dots-warm" />
                    {p.notes.map((note, j) => (
                      <div
                        key={note}
                        className="absolute w-40 max-w-[70%] rounded-md border border-stone-200 bg-[#fffdf6] px-3 py-2.5 shadow-md"
                        style={{
                          left: `${12 + j * 30}%`,
                          top: `${8 + j * 26}%`,
                          transform: `rotate(${[-5, 4, -3][j] ?? 0}deg)`,
                          zIndex: j,
                        }}
                      >
                        <span className="mb-1.5 block h-1 w-8 rounded-full bg-orange-300/70" />
                        <p className="text-[12px] font-semibold leading-snug text-stone-600 italic">
                          {note}
                        </p>
                      </div>
                    ))}
                    <span
                      className="absolute bottom-3 right-4 font-mono text-[11px] font-bold text-rose-300"
                      style={{ transform: `rotate(${p.rot})` }}
                    >
                      ⚠ unmanaged
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>

        <div
          data-reveal-group
          className="mt-10 flex items-center justify-center gap-3 text-center"
        >
          <p className="text-lg font-semibold text-stone-600" data-reveal>
            Sound familiar? There&rsquo;s a calmer way.
          </p>
          <ArrowRightIcon className="h-5 w-5 text-orange-500" />
        </div>
      </div>
    </section>
  );
}
