import { useLayoutEffect, useRef, useState } from "react";
import { gsap, prefersReduced, revealUp } from "../lib/anim";
import { cn } from "../utils/cn";
import { CtaButton, SectionTag, useCta } from "../components/shared";
import { CheckList, PageCta } from "../components/PagePrimitives";
import {
  WindowFrame,
  DashboardScreen,
  StudentsScreen,
  AttendanceScreen,
  PaymentsScreen,
  ResultsScreen,
} from "../components/ProductUI";
import {
  CheckIcon,
  DownloadIcon,
  MonitorIcon,
  ShieldIcon,
  WifiOffIcon,
  DatabaseIcon,
} from "../components/icons";

const PREVIEW = [
  ["Dashboard", <DashboardScreen />],
  ["Student", <StudentsScreen />],
  ["Attendance", <AttendanceScreen />],
  ["Payment", <PaymentsScreen />],
  ["Result", <ResultsScreen />],
] as const;

export function DownloadPage() {
  const ref = useRef<HTMLDivElement>(null);
  const [preview, setPreview] = useState(0);
  const cta = useCta();

  useLayoutEffect(() => {
    if (!ref.current) return;
    const ctx = gsap.context(() => {
      revealUp(ref.current, "[data-download-reveal]", {
        stagger: 0.08,
        y: 28,
        start: "top 92%",
      });
      if (prefersReduced()) return;
      gsap.fromTo(
        "[data-download-mock]",
        { y: 56, rotateX: 6, autoAlpha: 0, transformPerspective: 1200 },
        {
          y: 0,
          rotateX: 0,
          autoAlpha: 1,
          transformPerspective: 1200,
          duration: 1.0,
          ease: "power3.out",
          delay: 0.25,
        },
      );
      gsap.to("[data-download-mock]", {
        y: -24,
        scrollTrigger: {
          trigger: "[data-download-hero]",
          start: "top top",
          end: "bottom top",
          scrub: 1,
        },
      });
      const install = gsap.timeline({
        scrollTrigger: {
          trigger: "[data-install]",
          start: "top 72%",
          once: true,
        },
      });
      install
        .fromTo(
          "[data-package]",
          { scale: 0.85, y: 30, autoAlpha: 0 },
          {
            scale: 1,
            y: 0,
            autoAlpha: 1,
            duration: 0.65,
            ease: "back.out(1.5)",
          },
        )
        .fromTo(
          "[data-install-line]",
          { scaleX: 0 },
          { scaleX: 1, duration: 0.65, stagger: 0.12, ease: "power2.inOut" },
          0.2,
        );
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <div ref={ref}>
      <section
        data-download-hero
        className="relative overflow-hidden pb-20 pt-36 sm:pt-44"
      >
        <div className="absolute inset-0 bg-grid-warm opacity-70" />
        <div className="absolute left-1/2 top-0 h-[480px] w-[800px] -translate-x-1/2 rounded-full bg-orange-300/25 blur-[130px]" />
        <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="mx-auto max-w-4xl text-center">
            <span data-download-reveal>
              <SectionTag>Windows desktop demo</SectionTag>
            </span>
            <h1
              data-download-reveal
              className="mt-6 font-display text-5xl font-extrabold leading-[1.02] tracking-tight text-ink sm:text-6xl lg:text-[72px]"
            >
              Meet Coachify on{" "}
              <span className="text-orange-500">your own PC.</span>
            </h1>
            <p
              data-download-reveal
              className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-stone-500 sm:text-xl"
            >
              Download the demo and experience a faster way to manage students,
              attendance, payments, exams and your entire coaching center.
            </p>
            <div data-download-reveal className="mt-9">
              <CtaButton
                size="lg"
                className="min-w-64"
                onClick={() => cta("download")}
                icon={<DownloadIcon className="h-5 w-5" />}
              >
                Download Coachify Demo
              </CtaButton>
              <p className="mt-4 text-sm font-semibold text-stone-400">
                Windows 10 / 11 · Quick installation · Demo institution included
              </p>
            </div>
          </div>
          <div
            data-download-mock
            className="mx-auto mt-14 max-w-5xl perspective-[1200px]"
          >
            <WindowFrame
              title="Coachify — Demo Institution"
              bodyClassName="min-h-[430px]"
            >
              <DashboardScreen />
            </WindowFrame>
          </div>
          <div className="mx-auto mt-8 grid max-w-4xl gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {[
              "Works offline",
              "No credit card",
              "Demo institution included",
              "PC data stays local",
            ].map((x) => (
              <div
                key={x}
                data-download-reveal
                className="flex items-center gap-2 rounded-xl border border-stone-200 bg-white/80 px-4 py-3 text-sm font-semibold text-stone-700"
              >
                <CheckIcon className="h-4 w-4 text-orange-500" />
                {x}
              </div>
            ))}
          </div>
        </div>
      </section>

      <section data-install className="bg-white py-24 sm:py-32">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="max-w-3xl">
            <SectionTag>From website to working software</SectionTag>
            <h2 className="mt-5 font-display text-4xl font-extrabold text-ink sm:text-5xl">
              On your PC in four simple steps.
            </h2>
            <p className="mt-4 text-lg text-stone-500">
              You will not land in an empty shell. The demo opens with a
              complete institute ready to explore.
            </p>
          </div>
          <div className="mt-14 grid gap-4 lg:grid-cols-4">
            {[
              [
                "01",
                "Download",
                "A Coachify installer package arrives in your Downloads folder.",
                DownloadIcon,
              ],
              [
                "02",
                "Install",
                "A short guided setup places Coachify safely on your Windows PC.",
                MonitorIcon,
              ],
              [
                "03",
                "Open Coachify",
                "Launch the desktop app. No browser account or cloud workspace required.",
                WifiOffIcon,
              ],
              [
                "04",
                "Explore the demo",
                "Students, payments, attendance, exams and reports are already populated.",
                DatabaseIcon,
              ],
            ].map(([n, t, d, Icon], i) => {
              const I = Icon as typeof DownloadIcon;
              return (
                <div
                  key={t as string}
                  data-package={i === 0 || undefined}
                  className="relative rounded-3xl border border-stone-200 bg-cream p-6"
                >
                  <div
                    data-install-line
                    className="absolute left-0 right-0 top-0 h-1 origin-left rounded-t-3xl bg-orange-500"
                  />
                  <span className="tnum text-xs font-bold text-orange-600">
                    {n as string}
                  </span>
                  <I className="mt-8 h-7 w-7 text-orange-500" />
                  <h3 className="mt-4 font-display text-xl font-bold text-ink">
                    {t as string}
                  </h3>
                  <p className="mt-2 text-sm leading-relaxed text-stone-500">
                    {d as string}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      </section>

      <section className="py-24 sm:py-32">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="mx-auto max-w-3xl text-center">
            <SectionTag>Interactive preview</SectionTag>
            <h2 className="mt-5 font-display text-4xl font-extrabold text-ink sm:text-5xl">
              This is what you'll have on your PC in minutes.
            </h2>
          </div>
          <div className="no-scrollbar mt-10 flex justify-center gap-2 overflow-x-auto">
            {PREVIEW.map((x, i) => (
              <button
                key={x[0]}
                onClick={() => setPreview(i)}
                className={cn(
                  "shrink-0 cursor-pointer rounded-full px-4 py-2 text-sm font-semibold",
                  i === preview
                    ? "bg-orange-500 text-white"
                    : "border border-stone-200 bg-white text-stone-500",
                )}
              >
                {x[0]}
              </button>
            ))}
          </div>
          <div key={preview} className="mx-auto mt-6 max-w-5xl">
            <WindowFrame
              title={`Coachify Demo — ${PREVIEW[preview][0]}`}
              bodyClassName="min-h-[430px]"
            >
              {PREVIEW[preview][1]}
            </WindowFrame>
          </div>
          <div className="mt-8 text-center">
            <CtaButton size="lg" onClick={() => cta("download")}>
              Download Demo
            </CtaButton>
          </div>
        </div>
      </section>

      <section className="bg-white py-24">
        <div className="mx-auto grid max-w-7xl gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:items-center lg:px-8">
          <div>
            <SectionTag>Complete enough to judge properly</SectionTag>
            <h2 className="mt-5 font-display text-4xl font-extrabold text-ink">
              Don't explore empty screens.
            </h2>
            <p className="mt-4 leading-relaxed text-stone-500">
              Coachify includes a realistic demo institution so you can follow
              connected records from a student profile to attendance, a fee
              receipt and a result. Commercial services that require your own
              credentials, such as an SMS gateway, may need configuration.
            </p>
          </div>
          <CheckList
            items={[
              "Admissions",
              "Students",
              "Attendance",
              "Fees & receipts",
              "Exams",
              "Results",
              "Reports",
              "Backup & restore",
            ]}
          />
        </div>
      </section>

      <section className="relative overflow-hidden bg-ink py-24 text-white">
        <div className="absolute inset-0 bg-grid-dark opacity-50" />
        <div className="relative mx-auto grid max-w-7xl gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:items-center lg:px-8">
          <div>
            <WifiOffIcon className="h-10 w-10 text-orange-400" />
            <h2 className="mt-5 font-display text-4xl font-extrabold">
              Wi-Fi disappears. Coachify keeps working.
            </h2>
            <p className="mt-4 leading-relaxed text-stone-400">
              Disconnect during the demo. Student search, payments, attendance,
              receipts and reports remain available because the core system is
              local.
            </p>
          </div>
          <CheckList
            dark
            items={[
              "Payment collection",
              "Attendance",
              "Student search",
              "Receipt printing",
              "Reports",
              "Local backup",
            ]}
          />
        </div>
      </section>

      <section className="py-24">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid gap-12 lg:grid-cols-2">
            <div>
              <SectionTag>Safe to try</SectionTag>
              <h2 className="mt-5 font-display text-4xl font-extrabold text-ink">
                Your computer remains yours.
              </h2>
              <div className="mt-8 space-y-5">
                {[
                  [
                    "Will it affect existing files?",
                    "Coachify installs as its own application and does not modify your documents.",
                  ],
                  [
                    "Does it upload student data?",
                    "There is no automatic cloud upload. Core records remain local.",
                  ],
                  [
                    "Can I uninstall it?",
                    "Yes. Use the standard Windows uninstall flow.",
                  ],
                  [
                    "Can I keep or restore data?",
                    "Coachify includes local backup and restore tools you can test in the demo.",
                  ],
                ].map((x) => (
                  <div key={x[0]} className="border-b border-stone-200 pb-5">
                    <h3 className="font-bold text-ink">{x[0]}</h3>
                    <p className="mt-1 text-sm leading-relaxed text-stone-500">
                      {x[1]}
                    </p>
                  </div>
                ))}
              </div>
            </div>
            <div className="rounded-3xl bg-cream p-8">
              <ShieldIcon className="h-8 w-8 text-orange-500" />
              <h3 className="mt-4 font-display text-2xl font-bold text-ink">
                Preliminary system guidance
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-stone-500">
                Final tested requirements will be published with the production
                installer. For demo evaluation, use a normal modern Windows
                office PC.
              </p>
              <div className="mt-6">
                <CheckList
                  items={[
                    "Windows 10 or Windows 11",
                    "Modern dual-core processor or better",
                    "4–8 GB RAM recommended",
                    "Local disk space for records and documents",
                  ]}
                />
              </div>
            </div>
          </div>
        </div>
      </section>
      <PageCta
        title="Ready to see how much easier managing your center can feel?"
        copy="Install Coachify Demo and explore the complete workflow yourself. Windows, demo data included, no credit card."
      />
    </div>
  );
}
