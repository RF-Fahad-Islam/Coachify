import { useLayoutEffect, useRef, type ReactNode } from "react";
import { gsap, revealUp } from "../lib/anim";
import { SectionTag, useCta } from "./shared";
import {
  WifiOffIcon,
  MonitorIcon,
  LanIcon,
  BanknoteIcon,
  AttendanceIcon,
  ExamIcon,
  BellIcon,
  ShieldIcon,
  ArrowRightIcon,
} from "./icons";

type Benefit = {
  icon: ReactNode;
  title: string;
  copy: string;
  foot: string;
  wide?: boolean;
};

const BENEFITS: Benefit[] = [
  {
    icon: <WifiOffIcon className="h-6 w-6" />,
    title: "Works Offline",
    copy: "Close the Wi-Fi. Keep the center running. Coachify needs no internet, no server and no cloud — the admission queue at 7 PM doesn’t stop because a tower is down.",
    foot: "0 internet required, ever",
    wide: true,
  },
  {
    icon: <MonitorIcon className="h-6 w-6" />,
    title: "Runs on Your PC",
    copy: "Install once on a Windows machine and you own everything. No accounts, no sync, no third party holding a copy of your data.",
    foot: "100% local database",
  },
  {
    icon: <LanIcon className="h-6 w-6" />,
    title: "LAN Ready",
    copy: "Reception, accounts and admin work together in real time over your office network — one database, several desks.",
    foot: "Up to 5 PCs on one network",
  },
  {
    icon: <BanknoteIcon className="h-6 w-6" />,
    title: "Fast Fee Collection",
    copy: "Pick a student, enter the amount, print the receipt. Under ten seconds at the counter — every single time.",
    foot: "<10s per receipt",
  },
  {
    icon: <AttendanceIcon className="h-6 w-6" />,
    title: "Attendance Made Easy",
    copy: "Open the batch, mark present or absent, done. Daily, monthly and yearly summaries compute themselves.",
    foot: "Roll call in under a minute",
  },
  {
    icon: <ExamIcon className="h-6 w-6" />,
    title: "Exams & Results",
    copy: "Schedule papers, enter marks once, apply your grading scheme, publish report cards batch-wide in one go.",
    foot: "Marks entered exactly once",
  },
  {
    icon: <BellIcon className="h-6 w-6" />,
    title: "SMS & Email Alerts",
    copy: "Payment confirmations, due reminders and absence alerts — sent from the desk, read by parents on their phones.",
    foot: "Confirmations, dues, absences",
  },
  {
    icon: <ShieldIcon className="h-6 w-6" />,
    title: "Safe Backup & Restore",
    copy: "One-click encrypted backups to a folder, disk or pen drive. Restore a full year of records in minutes, not days.",
    foot: "One-click encrypted backups",
  },
];

export function Benefits() {
  const ref = useRef<HTMLElement>(null);
  const openCta = useCta();

  useLayoutEffect(() => {
    if (!ref.current) return;
    const ctx = gsap.context(() => {
      revealUp(ref.current, "[data-reveal]", { stagger: 0.08, y: 32 });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section
      id="features"
      ref={ref}
      className="relative scroll-mt-24 py-24 sm:py-32"
    >
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="max-w-3xl">
          <span data-reveal>
            <SectionTag>Why centers switch</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl lg:text-[54px]"
          >
            Built for the way a coaching center{" "}
            <span className="text-orange-500">actually</span> works.
          </h2>
          <p
            data-reveal
            className="mt-5 max-w-2xl text-lg leading-relaxed text-stone-500"
          >
            Not a generic business suite bent into shape. Every screen, workflow
            and report is designed around admissions, fees, batches and result
            days.
          </p>
        </div>

        <div className="mt-14 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {BENEFITS.map((b) =>
            b.wide ? (
              <div
                key={b.title}
                data-reveal
                className="group relative overflow-hidden rounded-3xl bg-ink p-7 text-white shadow-lift transition-all duration-500 hover:-translate-y-1.5 md:col-span-2 lg:col-span-2"
              >
                <div className="absolute inset-0 bg-grid-dark opacity-40" />
                <div className="absolute -right-16 -top-16 h-56 w-56 rounded-full bg-orange-500/25 blur-3xl transition-opacity duration-500 group-hover:opacity-100" />
                <div className="relative flex h-full flex-col">
                  <div className="flex items-start justify-between">
                    <span className="flex h-12 w-12 items-center justify-center rounded-2xl bg-orange-500 text-white shadow-lg shadow-orange-500/40">
                      {b.icon}
                    </span>
                    <span className="rounded-full border border-white/15 bg-white/5 px-3 py-1 text-[10px] font-bold uppercase tracking-[0.16em] text-orange-300">
                      Flagship feature
                    </span>
                  </div>
                  <h3 className="mt-5 font-display text-2xl font-bold tracking-tight sm:text-[26px]">
                    {b.title}
                  </h3>
                  <p className="mt-2.5 max-w-md text-[15px] leading-relaxed text-stone-400">
                    {b.copy}
                  </p>
                  <div className="mt-auto flex items-center gap-3 pt-6">
                    <div className="flex items-end gap-1" aria-hidden="true">
                      {[14, 22, 30, 38].map((h, j) => (
                        <span
                          key={j}
                          className={`w-1.5 rounded-full ${j === 0 ? "bg-emerald-400" : "bg-white/15"}`}
                          style={{ height: h }}
                        />
                      ))}
                    </div>
                    <span className="text-xs font-bold uppercase tracking-[0.16em] text-emerald-400">
                      {b.foot}
                    </span>
                  </div>
                </div>
              </div>
            ) : (
              <div
                key={b.title}
                data-reveal
                className="group flex flex-col rounded-3xl border border-stone-200/80 bg-white p-6 shadow-card transition-all duration-500 hover:-translate-y-1.5 hover:shadow-lift hover:border-orange-200"
              >
                <span className="flex h-11 w-11 items-center justify-center rounded-xl bg-orange-50 text-orange-600 transition-all duration-500 group-hover:bg-orange-500 group-hover:text-white group-hover:shadow-lg group-hover:shadow-orange-500/30">
                  {b.icon}
                </span>
                <h3 className="mt-4 font-display text-[17px] font-bold tracking-tight text-ink">
                  {b.title}
                </h3>
                <p className="mt-2 flex-1 text-sm leading-relaxed text-stone-500">
                  {b.copy}
                </p>
                <p className="mt-4 border-t border-stone-100 pt-3 text-[11px] font-bold uppercase tracking-[0.12em] text-stone-400 group-hover:text-orange-600">
                  {b.foot}
                </p>
              </div>
            ),
          )}

          {/* CTA tile to balance the grid */}
          <div
            data-reveal
            onClick={() => openCta("download")}
            className="group flex cursor-pointer flex-col justify-between rounded-3xl border-2 border-dashed border-orange-300 bg-orange-50/60 p-6 transition-all duration-500 hover:-translate-y-1.5 hover:border-orange-400 hover:bg-orange-50 md:col-span-2"
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === "Enter" && openCta("download")}
          >
            <div>
              <h3 className="font-display text-[17px] font-bold tracking-tight text-ink">
                Don&rsquo;t take our word for it.
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-stone-500">
                The demo is the full product, free for 30 days. Put your own
                numbers in it and see how quiet the front desk gets.
              </p>
            </div>
            <span className="mt-5 inline-flex items-center gap-2 text-sm font-bold text-orange-600">
              Download the demo
              <span className="transition-transform duration-300 group-hover:translate-x-1.5">
                <ArrowRightIcon className="h-4 w-4" />
              </span>
            </span>
          </div>
        </div>
      </div>
    </section>
  );
}
