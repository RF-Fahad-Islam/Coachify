import { useLayoutEffect, useRef, useState } from "react";
import { gsap, prefersReduced, revealUp } from "../lib/anim";
import { cn } from "../utils/cn";
import { SectionTag } from "./shared";
import { CheckIcon, XIcon } from "./icons";

const COMPARE: [string, string, string][] = [
  ["Needs internet", "All the time", "Never"],
  ["Where data lives", "Vendor’s cloud", "Your machine"],
  ["Cost model", "Monthly, per user", "One-time license"],
  ["When the net drops", "Center stops", "Nothing changes"],
  ["Setup", "Signups + data uploads", "Install & go"],
];

function Monitor({
  x,
  y,
  label,
  active = false,
}: {
  x: number;
  y: number;
  label: string;
  active?: boolean;
}) {
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={64}
        height={42}
        rx={6}
        fill="#241b13"
        stroke={active ? "#f97316" : "#4a3a2c"}
        strokeWidth="1.5"
      />
      <rect
        x={x + 8}
        y={y + 8}
        width={28}
        height={4}
        rx={2}
        fill={active ? "#f97316" : "#57432f"}
      />
      <rect
        x={x + 8}
        y={y + 16}
        width={40}
        height={3}
        rx={1.5}
        fill="#3d3022"
      />
      <rect
        x={x + 8}
        y={y + 23}
        width={34}
        height={3}
        rx={1.5}
        fill="#3d3022"
      />
      <rect x={x + 26} y={y + 42} width={12} height={6} fill="#3d3022" />
      <rect
        x={x + 18}
        y={y + 48}
        width={28}
        height={3}
        rx={1.5}
        fill="#4a3a2c"
      />
      <text
        x={x + 32}
        y={y + 66}
        textAnchor="middle"
        fill="#a8a29e"
        fontSize="10"
        fontWeight="600"
      >
        {label}
      </text>
    </g>
  );
}

function Db({ x, y }: { x: number; y: number }) {
  return (
    <g>
      <ellipse
        cx={x + 24}
        cy={y}
        rx={24}
        ry={8}
        fill="#241b13"
        stroke="#f97316"
        strokeWidth="1.5"
      />
      <path
        d={`M ${x} ${y} v 26 c 0 4.4 10.7 8 24 8 s 24 -3.6 24 -8 v -26`}
        fill="#241b13"
        stroke="#f97316"
        strokeWidth="1.5"
      />
      <path
        d={`M ${x} ${y + 13} c 0 4.4 10.7 8 24 8 s 24 -3.6 24 -8`}
        fill="none"
        stroke="#f97316"
        strokeWidth="1.2"
        opacity="0.5"
      />
      <text
        x={x + 24}
        y={y + 48}
        textAnchor="middle"
        fill="#fdba74"
        fontSize="9.5"
        fontWeight="700"
      >
        LOCAL DATABASE
      </text>
    </g>
  );
}

const Line = (props: { d: string; delay?: number }) => (
  <path
    data-draw
    d={props.d}
    fill="none"
    stroke="#f97316"
    strokeWidth="1.6"
    strokeLinecap="round"
    pathLength={1}
    strokeDasharray="1"
    strokeDashoffset="1"
    opacity="0.9"
  />
);

export function Offline() {
  const ref = useRef<HTMLElement>(null);
  const [mode, setMode] = useState<"single" | "lan">("single");
  const modeRef = useRef(mode);
  modeRef.current = mode;

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root) return;
    const ctx = gsap.context(() => {
      revealUp(root, "[data-reveal]", { stagger: 0.12 });
      if (prefersReduced()) {
        gsap.set(root.querySelectorAll("[data-draw]"), { strokeDashoffset: 0 });
        return;
      }
      const drawPaths = root.querySelectorAll<SVGPathElement>("[data-draw]");
      gsap.set('[data-diag="lan"]', {
        autoAlpha: 0,
        scale: 0.94,
        transformOrigin: "50% 50%",
      });
      gsap.set(drawPaths, { strokeDashoffset: 1 });
      gsap.to(drawPaths, {
        strokeDashoffset: 0,
        duration: 1.1,
        stagger: 0.18,
        ease: "power2.inOut",
        scrollTrigger: {
          trigger: root.querySelector(".offline-diagram"),
          start: "top 78%",
          once: true,
        },
      });
    }, ref);
    return () => ctx.revert();
  }, []);

  const switchMode = (m: "single" | "lan") => {
    if (m === mode) return;
    setMode(m);
    if (prefersReduced()) return;
    const show = m === "single" ? '[data-diag="single"]' : '[data-diag="lan"]';
    const hide = m === "single" ? '[data-diag="lan"]' : '[data-diag="single"]';
    gsap.to(hide, {
      autoAlpha: 0,
      scale: 0.94,
      duration: 0.35,
      ease: "power2.in",
    });
    gsap.to(show, {
      autoAlpha: 1,
      scale: 1,
      duration: 0.5,
      delay: 0.12,
      ease: "power3.out",
    });
  };

  return (
    <section
      ref={ref}
      className="relative overflow-hidden bg-ink py-24 text-white sm:py-32"
    >
      <div className="absolute inset-0 bg-grid-dark opacity-50 [mask-image:radial-gradient(ellipse_80%_70%_at_50%_20%,black,transparent)]" />
      <div className="absolute -top-40 left-1/2 h-[420px] w-[720px] -translate-x-1/2 rounded-full bg-orange-500/15 blur-[120px]" />

      <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="grid gap-14 lg:grid-cols-[1fr_1.1fr] lg:items-center lg:gap-16">
          <div>
            <span data-reveal>
              <SectionTag dark>The real difference</SectionTag>
            </span>
            <h2
              data-reveal
              className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight sm:text-5xl lg:text-[52px]"
            >
              Your software should work when your{" "}
              <span className="text-orange-400">internet doesn&rsquo;t.</span>
            </h2>
            <p
              data-reveal
              className="mt-6 max-w-lg text-[16px] leading-relaxed text-stone-400"
            >
              Cloud tools hand your operations to a network you don&rsquo;t
              control. Coachify lives on your own machines — a single PC today,
              the whole office over your LAN tomorrow. No outages, no
              subscriptions, no lock-in.
            </p>

            <div data-reveal className="mt-8 flex flex-wrap gap-2">
              {[
                "No subscription lock-in",
                "You own the data",
                "Windows 10 / 11",
                "Updates when you choose",
              ].map((c) => (
                <span
                  key={c}
                  className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3.5 py-2 text-[13px] font-semibold text-stone-300"
                >
                  <CheckIcon className="h-3.5 w-3.5 text-orange-400" />
                  {c}
                </span>
              ))}
            </div>
          </div>

          {/* diagram + toggle */}
          <div
            data-reveal
            className="offline-diagram rounded-3xl border border-white/10 bg-white/[0.03] p-5 sm:p-7"
          >
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex rounded-full border border-white/10 bg-white/5 p-1">
                {(["single", "lan"] as const).map((m) => (
                  <button
                    key={m}
                    onClick={() => switchMode(m)}
                    className={cn(
                      "cursor-pointer rounded-full px-4 py-2 text-[13px] font-semibold transition-all duration-300",
                      mode === m
                        ? "bg-orange-500 text-white shadow-lg shadow-orange-500/30"
                        : "text-stone-400 hover:text-white",
                    )}
                  >
                    {m === "single" ? "Single PC" : "LAN office"}
                  </button>
                ))}
              </div>
              <span className="flex items-center gap-2 text-[11px] font-bold uppercase tracking-[0.16em] text-emerald-400">
                <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 pulse-dot" />
                Running locally
              </span>
            </div>

            <div className="relative mt-4 h-[280px] sm:h-[300px]">
              {/* single PC diagram */}
              <svg
                data-diag="single"
                viewBox="0 0 420 280"
                className={cn(
                  "absolute inset-0 h-full w-full",
                  prefersReduced() && mode !== "single" && "opacity-0",
                )}
              >
                <Monitor x={178} y={30} label="Front desk PC" active />
                <Line d="M210 100 L210 150" />
                <Db x={186} y={150} />
                <g>
                  <circle
                    cx={350}
                    cy={40}
                    r={17}
                    fill="none"
                    stroke="#57534e"
                    strokeWidth="1.5"
                  />
                  <path
                    d="M338 52 L362 28"
                    stroke="#f97316"
                    strokeWidth="2"
                    strokeLinecap="round"
                  />
                  <path
                    d="M342 40 a 14 14 0 0 1 8 -5 M358 32 a 14 14 0 0 1 5 4 M345 46 a 8 8 0 0 1 5 -3"
                    stroke="#a8a29e"
                    strokeWidth="1.5"
                    fill="none"
                    strokeLinecap="round"
                  />
                  <text
                    x={350}
                    y={78}
                    textAnchor="middle"
                    fill="#a8a29e"
                    fontSize="10"
                    fontWeight="600"
                  >
                    Internet off
                  </text>
                </g>
                <text x={60} y={210} fill="#78716c" fontSize="10.5">
                  Reception, fees and reports —
                </text>
                <text x={60} y={226} fill="#78716c" fontSize="10.5">
                  one machine, everything runs.
                </text>
              </svg>

              {/* LAN diagram */}
              <svg
                data-diag="lan"
                viewBox="0 0 420 280"
                className={cn(
                  "absolute inset-0 h-full w-full",
                  prefersReduced() && mode !== "lan" && "opacity-0",
                )}
              >
                {/* router */}
                <rect
                  x={178}
                  y={14}
                  width={64}
                  height={26}
                  rx={6}
                  fill="#241b13"
                  stroke="#f97316"
                  strokeWidth="1.5"
                />
                <circle cx={194} cy={27} r={2.5} fill="#34d399" />
                <circle cx={204} cy={27} r={2.5} fill="#f97316" />
                <circle cx={214} cy={27} r={2.5} fill="#f97316" />
                <text
                  x={210}
                  y={58}
                  textAnchor="middle"
                  fill="#a8a29e"
                  fontSize="10"
                  fontWeight="600"
                >
                  Office router
                </text>
                <Line d="M210 40 L210 64" delay={1} />
                <Db x={186} y={64} />
                <Line d="M210 118 L70 168" />
                <Line d="M210 118 L210 168" />
                <Line d="M210 118 L350 168" />
                <Monitor x={38} y={168} label="Reception" />
                <Monitor x={178} y={168} label="Accounts" active />
                <Monitor x={318} y={168} label="Principal" />
                <text x={60} y={262} fill="#78716c" fontSize="10.5">
                  One database · several desks · real time.
                </text>
              </svg>
            </div>

            <p className="mt-2 text-center text-xs font-medium text-stone-500">
              {mode === "single"
                ? "Start on the PC you already have. Move to LAN whenever you need more hands."
                : "Reception admits, accounts collects fees, the principal reads reports — all at once."}
            </p>
          </div>
        </div>

        {/* comparison */}
        <div
          data-reveal
          className="mt-20 overflow-hidden rounded-3xl border border-white/10"
        >
          <div className="grid grid-cols-[1.2fr_1fr_1fr] gap-x-4 bg-white/[0.04] px-5 py-4 text-[12px] font-bold uppercase tracking-[0.14em] sm:px-7 sm:text-[13px]">
            <span className="text-stone-400">What actually matters</span>
            <span className="text-stone-500">Typical cloud app</span>
            <span className="text-orange-400">Coachify</span>
          </div>
          {COMPARE.map(([label, cloud, coach], i) => (
            <div
              key={label}
              className={cn(
                "grid grid-cols-[1.2fr_1fr_1fr] items-center gap-x-4 px-5 py-4 sm:px-7",
                i !== COMPARE.length - 1 && "border-b border-white/5",
                "border-white/5",
              )}
            >
              <span className="text-sm font-semibold text-stone-300">
                {label}
              </span>
              <span className="flex items-center gap-2 text-[13px] text-stone-500">
                <XIcon className="h-3.5 w-3.5 shrink-0 text-rose-500/70" />
                {cloud}
              </span>
              <span className="flex items-center gap-2 rounded-xl bg-orange-500/10 px-3 py-1.5 text-[13px] font-semibold text-orange-200">
                <CheckIcon className="h-3.5 w-3.5 shrink-0 text-orange-400" />
                {coach}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
