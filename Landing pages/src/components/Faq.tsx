import { useState } from "react";
import { cn } from "../utils/cn";
import { gsap, revealUp } from "../lib/anim";
import { useLayoutEffect, useRef } from "react";
import { SectionTag, CtaButton, useCta } from "./shared";
import { MailIcon, PhoneIcon } from "./icons";

const FAQS = [
  {
    q: "Does Coachify work without internet?",
    a: "Yes — completely. The entire application runs on your Windows PC: admissions, attendance, fees, receipts, exams, results, reports and backups all work with zero connectivity. The only thing that uses the internet is the optional SMS / email feature, and even those alerts are queued and sent whenever your connection is back.",
  },
  {
    q: "Can I use it on multiple computers?",
    a: "Yes. In LAN mode, up to 5 PCs in your office share one live local database. Reception admits a student, accounts collects the fee and the principal reads reports — at the same time, with no conflicts and no cloud in between.",
  },
  {
    q: "Is my data stored locally?",
    a: "Always. The database lives on your machine and your institution owns it outright. No copies go to any server, and the only data that ever leaves the PC is the encrypted backup you choose to make.",
  },
  {
    q: "Can I take backups?",
    a: "One click, encrypted, to a folder, external disk or pen drive. You can schedule daily automatic backups, and restoring a full year of records takes minutes — not a data-recovery agency.",
  },
  {
    q: "Does it support receipts and reports?",
    a: "Both. Print thermal (80mm) or A4 fee receipts at the counter, with a duplicate copy for your books — and reprint any receipt years later. 40+ print-ready reports cover fees, dues, attendance, student statements and exam performance.",
  },
  {
    q: "Can it send SMS or email?",
    a: "Yes, as an optional module. Connect your own SMS / email gateway and Coachify will send fee confirmations, due reminders and absence alerts from your desk. When you're offline, alerts queue and send automatically once you're connected.",
  },
  {
    q: "Is there a demo version?",
    a: "Yes — a free, full-featured 30-day demo for Windows. Every module is enabled for up to 100 students, there's no card required, and there are no nag screens. It's the best way to judge it with your own data.",
  },
  {
    q: "How do I get the full version?",
    a: "Hit “Request Full Version” and we'll share pricing for the Full or Pro Multi-PC license, send you the installer with your license key, and run a setup call with you — including migration from your existing registers or spreadsheets.",
  },
];

export function Faq() {
  const ref = useRef<HTMLElement>(null);
  const [open, setOpen] = useState(0);
  const openCta = useCta();

  useLayoutEffect(() => {
    if (!ref.current) return;
    const ctx = gsap.context(() => {
      revealUp(ref.current, "[data-reveal]", { stagger: 0.1 });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section id="faq" ref={ref} className="scroll-mt-24 py-24 sm:py-32">
      <div className="mx-auto grid max-w-7xl gap-14 px-4 sm:px-6 lg:grid-cols-[0.8fr_1.2fr] lg:gap-20 lg:px-8">
        <div className="lg:sticky lg:top-28 lg:self-start">
          <span data-reveal>
            <SectionTag>Questions, answered</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl"
          >
            The questions centers{" "}
            <span className="text-orange-500">actually</span> ask.
          </h2>
          <p
            data-reveal
            className="mt-5 max-w-md text-[16px] leading-relaxed text-stone-500"
          >
            Short answers first, full answers when you open them. Anything else
            — just ask a human.
          </p>
          <div
            data-reveal
            className="mt-8 rounded-3xl border border-stone-200/80 bg-white p-6 shadow-card"
          >
            <p className="font-display text-lg font-bold text-ink">
              Still unsure about something?
            </p>
            <p className="mt-1.5 text-sm text-stone-500">
              A 20-minute call with our team beats a page of specs.
            </p>
            <div className="mt-4">
              <CtaButton
                variant="dark"
                onClick={() => openCta("request", "Book a live demo")}
              >
                Book a Demo
              </CtaButton>
            </div>
            <div className="mt-4 space-y-1.5 border-t border-stone-100 pt-4 text-sm font-medium text-stone-500">
              <p className="flex items-center gap-2">
                <MailIcon className="h-4 w-4 text-orange-500" />{" "}
                sales@coachify.app
              </p>
              <p className="flex items-center gap-2">
                <PhoneIcon className="h-4 w-4 text-orange-500" /> +91 98765
                04321 · Mon–Sat, 10–7
              </p>
            </div>
          </div>
        </div>

        <div data-reveal>
          {FAQS.map((f, i) => (
            <div
              key={f.q}
              className={cn("border-b border-stone-200", i === 0 && "border-t")}
            >
              <button
                onClick={() => setOpen(open === i ? -1 : i)}
                aria-expanded={open === i}
                className="flex w-full cursor-pointer items-center gap-4 py-5 text-left"
              >
                <span
                  className={cn(
                    "tnum text-sm font-bold transition-colors duration-300",
                    open === i ? "text-orange-500" : "text-stone-300",
                  )}
                >
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span
                  className={cn(
                    "flex-1 font-display text-[17px] font-semibold transition-colors duration-300",
                    open === i ? "text-ink" : "text-stone-600",
                  )}
                >
                  {f.q}
                </span>
                <span
                  className={cn(
                    "flex h-8 w-8 shrink-0 items-center justify-center rounded-full border transition-all duration-400",
                    open === i
                      ? "rotate-45 border-orange-500 bg-orange-500 text-white"
                      : "border-stone-300 text-stone-500",
                  )}
                >
                  <svg
                    viewBox="0 0 16 16"
                    className="h-3.5 w-3.5"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                  >
                    <path d="M8 2v12M2 8h12" />
                  </svg>
                </span>
              </button>
              <div
                className="grid transition-[grid-template-rows] duration-500 ease-[cubic-bezier(0.22,1,0.36,1)]"
                style={{ gridTemplateRows: open === i ? "1fr" : "0fr" }}
              >
                <div className="overflow-hidden">
                  <p className="pb-6 pl-10 pr-4 text-[15px] leading-relaxed text-stone-500">
                    {f.a}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
