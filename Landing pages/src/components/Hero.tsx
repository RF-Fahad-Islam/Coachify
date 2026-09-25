import { useLayoutEffect, useRef } from "react";
import { gsap, prefersReduced } from "../lib/anim";
import { CtaButton, useCta } from "./shared";
import { WindowFrame, DashboardScreen, Chip } from "./ProductUI";
import {
  BrandLockup,
  WifiOffIcon,
  MonitorIcon,
  ClockIcon,
  AttendanceIcon,
  MessageIcon,
  BanknoteIcon,
} from "./icons";
import { PAYMENTS, ATTENDANCE, NOTIFICATIONS, bdtShort } from "../data/appData";
import { useLandingLocale } from "../lib/i18n";

const TICKER = [
  "Admissions",
  "Fee Collection (৳)",
  "Receipts",
  "Attendance",
  "Batches",
  "Exams",
  "Results",
  "Dues",
  "Reports",
  "SMS & Email",
  "Backups",
  "Guardians",
];

function scrambleTo(
  el: HTMLElement,
  finalText: string,
  duration: number,
  delay = 0,
  glyphs = "AC01#%&*<>/",
) {
  const renderFrame = (progress: number) => {
    const revealed = Math.floor(progress * finalText.length);
    el.textContent =
      finalText.slice(0, revealed) +
      finalText
        .slice(revealed)
        .split("")
        .map((c) =>
          c === " " || c === "—" || c === "."
            ? c
            : glyphs[(Math.random() * glyphs.length) | 0],
        )
        .join("");
  };

  // Pre-seed frame 0 synchronously so it never flashes plain text before typing
  renderFrame(0);

  const obj = { p: 0 };
  gsap.to(obj, {
    p: 1,
    duration,
    delay,
    ease: "power2.out",
    onUpdate: () => renderFrame(obj.p),
    onComplete: () => {
      el.textContent = finalText;
    },
  });
}

export function Hero() {
  const ref = useRef<HTMLElement>(null);
  const openCta = useCta();
  const { locale, t } = useLandingLocale();

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root || prefersReduced()) return;
    const ctx = gsap.context(() => {
      const heroLines = root.querySelectorAll<HTMLElement>(".hero-line > span");
      const heroFades = root.querySelectorAll<HTMLElement>(".hero-fade");
      const heroMock = root.querySelector<HTMLElement>(".hero-mock");
      const heroFloats = root.querySelectorAll<HTMLElement>(".hero-float");

      // Synchronously pre-hide every element before first paint so nothing is
      // visible at opacity 1 before its staggered timeline position arrives.
      gsap.set(heroLines, { yPercent: 105, autoAlpha: 0, force3D: true });
      gsap.set(heroFades, { y: 24, autoAlpha: 0, force3D: true });
      if (heroMock) {
        gsap.set(heroMock, {
          y: 44,
          autoAlpha: 0,
          scale: 0.965,
          force3D: true,
        });
      }
      gsap.set(heroFloats, { y: 22, autoAlpha: 0, force3D: true });

      const tl = gsap.timeline({ defaults: { ease: "power2.out" } });
      tl.to(
        heroLines,
        {
          yPercent: 0,
          autoAlpha: 1,
          duration: 1.25,
          stagger: 0.15,
          force3D: true,
        },
        0.06,
      )
        .to(
          heroFades,
          { y: 0, autoAlpha: 1, duration: 1.15, stagger: 0.12, force3D: true },
          0.32,
        )
        .to(
          heroMock,
          { y: 0, autoAlpha: 1, scale: 1, duration: 1.3, force3D: true },
          0.38,
        )
        .to(
          heroFloats,
          { y: 0, autoAlpha: 1, duration: 1.0, stagger: 0.14, force3D: true },
          0.78,
        );

      if (locale === "en") {
        const scrEl = root.querySelector<HTMLElement>(".hero-scr");
        if (scrEl) {
          scrambleTo(scrEl, t("hero.eyebrow"), 1.5, 0.1, "AC01#%&*<>/");
        }
      }

      gsap.to(root.querySelectorAll(".float-a"), {
        y: -8,
        duration: 3.0,
        yoyo: true,
        repeat: -1,
        ease: "sine.inOut",
        force3D: true,
      });
      gsap.to(root.querySelectorAll(".float-b"), {
        y: 8,
        duration: 3.5,
        yoyo: true,
        repeat: -1,
        ease: "sine.inOut",
        force3D: true,
      });
      gsap.to(root.querySelectorAll(".float-c"), {
        y: -6,
        duration: 3.2,
        yoyo: true,
        repeat: -1,
        ease: "sine.inOut",
        delay: 0.4,
        force3D: true,
      });

      gsap.fromTo(
        root.querySelectorAll(".hero-float"),
        { yPercent: 0 },
        {
          yPercent: -24,
          ease: "none",
          force3D: true,
          immediateRender: false,
          scrollTrigger: {
            trigger: root,
            start: "top top",
            end: "bottom top",
            scrub: 0.25,
            invalidateOnRefresh: true,
          },
        },
      );
    }, ref);
    return () => ctx.revert();
  }, [locale, t]);

  return (
    <section id="top" ref={ref} className="relative overflow-hidden">
      <div className="absolute inset-0 -z-10">
        <div className="absolute inset-0 bg-grid-warm [mask-image:radial-gradient(ellipse_75%_65%_at_50%_35%,black,transparent)]" />
        <div className="orb-1 absolute -top-32 right-[-10%] h-[520px] w-[520px] rounded-full bg-orange-200/50 blur-[110px]" />
        <div className="orb-2 absolute top-[45%] left-[-14%] h-[420px] w-[420px] rounded-full bg-amber-200/40 blur-[100px]" />
        <div className="absolute bottom-0 left-1/2 h-px w-[92%] -translate-x-1/2 bg-gradient-to-r from-transparent via-stone-300/70 to-transparent" />
      </div>

      <div className="mx-auto grid max-w-7xl items-center gap-14 px-4 pb-16 pt-32 sm:px-6 lg:grid-cols-[1.02fr_1fr] lg:gap-10 lg:px-8 lg:pb-24 lg:pt-40">
        <div>
          <div className="hero-fade">
            <p className="hero-scr text-[11px] font-bold uppercase tracking-[0.22em] text-orange-600">
              {t("hero.eyebrow")}
            </p>
          </div>

          <h1 className="mt-5 font-display text-[40px] font-extrabold leading-[1.05] tracking-tight text-ink sm:text-6xl lg:text-[68px]">
            <span className="hero-line block overflow-hidden pb-1">
              <span>{t("hero.line1")}</span>
            </span>
            <span className="hero-line block overflow-hidden pb-1">
              <span>
                {t("hero.line2Prefix")}{" "}
                <span className="text-orange-500">
                  {t("hero.line2Highlight")}
                </span>
              </span>
            </span>
            <span className="hero-line block overflow-hidden pb-2">
              <span>{t("hero.line3")}</span>
            </span>
          </h1>

          <p className="hero-fade mt-5 max-w-xl text-[17px] leading-relaxed text-stone-500 sm:text-lg">
            {t("hero.subtitle")}
          </p>

          <div className="hero-fade mt-8 flex flex-wrap items-center gap-3">
            <CtaButton size="lg" onClick={() => openCta("download")}>
              {t("hero.downloadDemo")}
            </CtaButton>
            <CtaButton
              size="lg"
              variant="dark"
              onClick={() => openCta("request")}
            >
              {t("hero.requestFull")}
            </CtaButton>
          </div>
          <p className="hero-fade mt-4 text-sm font-medium text-stone-400">
            {t("hero.trialNote")}
          </p>

          <div className="hero-fade mt-8 flex flex-wrap gap-2.5">
            {[
              [WifiOffIcon, t("hero.badgeOffline")],
              [MonitorIcon, t("hero.badgeWindows")],
              [ClockIcon, t("hero.badgeInstall")],
            ].map(([Icon, label]) => {
              const I = Icon as typeof WifiOffIcon;
              return (
                <span
                  key={label as string}
                  className="inline-flex items-center gap-2 rounded-full border border-stone-200 bg-white/70 px-3.5 py-2 text-[13px] font-semibold text-stone-600"
                >
                  <I className="h-4 w-4 text-orange-500" />
                  {label as string}
                </span>
              );
            })}
          </div>
        </div>

        <div className="hero-mock relative mx-auto w-full max-w-[560px]">
          <div className="absolute -inset-6 -z-10 rounded-[32px] bg-gradient-to-br from-orange-100/80 via-transparent to-amber-100/60" />
          <div className="absolute -right-8 -top-8 -z-10 h-24 w-24 rounded-full border border-dashed border-orange-300/70" />
          <div className="absolute -bottom-10 -left-8 -z-10 h-16 w-16 rounded-full bg-orange-400/15" />

          <WindowFrame
            title="Coachify — Dashboard (Atomic Coaching)"
            className="hero-mock-frame"
          >
            <DashboardScreen />
          </WindowFrame>

          <div className="hero-float absolute -right-4 -top-7 z-10 w-44 sm:-right-8">
            <div className="float-a rounded-2xl border border-stone-200/80 bg-white p-3 shadow-lift">
              <div className="flex items-center gap-2">
                <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-emerald-100 text-emerald-600">
                  <BanknoteIcon className="h-4 w-4" />
                </span>
                <div>
                  <p className="text-[11px] font-bold text-ink">
                    {bdtShort(PAYMENTS.cashier.amount)} received
                  </p>
                  <p className="text-[9px] text-stone-400">
                    {PAYMENTS.recent[0].name.split(" ")[0]} A. ·{" "}
                    {PAYMENTS.cashier.batch}
                  </p>
                </div>
              </div>
              <div className="mt-2 flex items-center justify-between rounded-lg bg-emerald-50 px-2 py-1">
                <span className="text-[9px] font-semibold text-emerald-700">
                  Receipt {PAYMENTS.receipt.number}
                </span>
                <Chip tone="green">Printed</Chip>
              </div>
            </div>
          </div>

          <div className="hero-float absolute -bottom-7 -left-4 z-10 w-48 sm:-left-10">
            <div className="float-b rounded-2xl border border-stone-200/80 bg-white p-3 shadow-lift">
              <div className="flex items-center gap-2">
                <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-orange-100 text-orange-600">
                  <AttendanceIcon className="h-4 w-4" />
                </span>
                <div>
                  <p className="text-[11px] font-bold text-ink">
                    {ATTENDANCE.batch}
                  </p>
                  <p className="tnum text-[9px] text-stone-400">
                    {ATTENDANCE.roster.filter((r) => r.present).length} /{" "}
                    {ATTENDANCE.totalStudents} present ·{" "}
                    {Math.round(
                      (ATTENDANCE.roster.filter((r) => r.present).length /
                        ATTENDANCE.totalStudents) *
                        100,
                    )}
                    %
                  </p>
                </div>
              </div>
              <div className="mt-2 flex gap-1">
                {Array.from({ length: 14 }).map((_, i) => (
                  <span
                    key={i}
                    className={
                      i === 9
                        ? "h-3 flex-1 rounded-sm bg-rose-300"
                        : "h-3 flex-1 rounded-sm bg-emerald-300"
                    }
                  />
                ))}
              </div>
            </div>
          </div>

          <div className="hero-float absolute -right-3 bottom-24 z-10 sm:-right-10">
            <div className="float-c flex items-center gap-2 rounded-full border border-stone-200/80 bg-white py-2 pl-2.5 pr-4 shadow-lift">
              <span className="flex h-7 w-7 items-center justify-center rounded-full bg-ink text-white">
                <MessageIcon className="h-3.5 w-3.5" />
              </span>
              <div>
                <p className="text-[10px] font-bold text-ink">
                  {NOTIFICATIONS.weeklyCount} reminders sent
                </p>
                <p className="text-[9px] text-stone-400">
                  fee due · SMS to parents
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="relative border-y border-stone-200/80 bg-white/60 py-3.5 backdrop-blur-sm">
        <div className="marquee-track" aria-hidden="true">
          {[...TICKER, ...TICKER].map((item, i) => (
            <span
              key={i}
              className="flex items-center gap-6 pr-6 text-[13px] font-semibold uppercase tracking-[0.14em] text-stone-400"
            >
              {item}
              <svg
                viewBox="0 0 16 16"
                className="h-3 w-3 text-orange-400"
                fill="currentColor"
              >
                <path d="M8 0c.6 4.2 3.8 7.4 8 8-4.2.6-7.4 3.8-8 8-.6-4.2-3.8-7.4-8-8 4.2-.6 7.4-3.8 8-8Z" />
              </svg>
            </span>
          ))}
        </div>
        <div className="pointer-events-none absolute inset-y-0 left-0 w-24 bg-gradient-to-r from-cream to-transparent" />
        <div className="pointer-events-none absolute inset-y-0 right-0 w-24 bg-gradient-to-l from-cream to-transparent" />
        <span className="absolute left-4 top-1/2 hidden -translate-y-1/2 items-center gap-1.5 rounded-full bg-ink px-3 py-1.5 text-[10px] font-bold uppercase tracking-widest text-white sm:flex">
          <BrandLockup
            markClassName="h-4 w-4"
            textClassName="text-[11px] tracking-tight normal-case"
            dark
          />
        </span>
      </div>
    </section>
  );
}
