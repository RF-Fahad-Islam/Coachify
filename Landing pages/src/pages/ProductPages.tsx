import { useLayoutEffect, useRef, useState } from "react";
import { gsap, prefersReduced } from "../lib/anim";
import { cn } from "../utils/cn";
import { PageCta, PageHero, CheckList } from "../components/PagePrimitives";
import { SectionTag } from "../components/shared";
import {
  WindowFrame,
  AdmitScene,
  StudentsScreen,
  AttendanceScreen,
  PaymentsScreen,
  ExamsScreen,
  ResultsScreen,
  NotifyScene,
  BackupScene,
  DashboardScreen,
  DuesScreen,
  ReportsScreen,
  SetupScene,
} from "../components/ProductUI";
import { Offline } from "../components/Offline";
import { Pricing } from "../components/Pricing";
import { Faq } from "../components/Faq";
import { ArrowRightIcon } from "../components/icons";

const FEATURE_STORIES = [
  {
    k: "Admissions",
    title: "Turn a new inquiry into an enrolled student in minutes.",
    copy: "Capture the inquiry, guardian and course once. Assign a batch, set the fee plan and collect the initial payment without moving between registers.",
    flow: ["Inquiry", "Admission", "Profile", "Batch", "Payment"],
    ui: <AdmitScene />,
  },
  {
    k: "Student management",
    title: "Every student. One complete record.",
    copy: "Guardian details, attendance, fees, payments, exams, results and documents remain attached to one durable student profile.",
    flow: ["Guardian", "Attendance", "Fees", "Exams", "Documents"],
    ui: <StudentsScreen />,
  },
  {
    k: "Attendance",
    title: "Attendance that takes seconds, not paperwork.",
    copy: "Mark present, absent, late or excused from the batch list. Coachify turns each day into useful monthly and yearly analytics.",
    flow: ["Batch", "Mark", "Save", "Analyze"],
    ui: <AttendanceScreen />,
  },
  {
    k: "Finance",
    title: "Find a student. Collect payment. Print the receipt. Done.",
    copy: "A fast cashier flow built for the evening queue. Search by student, phone or receipt number and finish the collection in seconds.",
    flow: ["Search", "Due", "Payment", "Receipt"],
    ui: <PaymentsScreen />,
  },
  {
    k: "Exams & results",
    title: "From marks to report cards without spreadsheets.",
    copy: "Create the exam, enter marks once and let Coachify calculate totals, grades, ranks and print-ready report cards.",
    flow: ["Exam", "Marks", "Calculate", "Result", "Report card"],
    ui: <ResultsScreen />,
  },
  {
    k: "Communication",
    title: "Keep guardians informed automatically.",
    copy: "Payment confirmations, absence alerts and fee reminders are generated from real actions, not a separate list someone has to remember.",
    flow: ["Event", "Message", "Guardian", "Delivered"],
    ui: <NotifyScene />,
  },
  {
    k: "Backup",
    title: "Years of institutional data deserve more than hope.",
    copy: "Create verified backups automatically, copy them to an external drive and restore the institution when you need it.",
    flow: ["Database", "Backup", "Verify", "Restore"],
    ui: <BackupScene />,
  },
];

export function FeaturesPage() {
  const ref = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root) return;
    if (prefersReduced()) return;

    const ctx = gsap.context(() => {
      const rows = gsap.utils.toArray<HTMLElement>("[data-feature-row]", root);
      rows.forEach((row) => {
        const textEls = row.querySelectorAll<HTMLElement>("[data-feature]");
        const uiEl = row.querySelector<HTMLElement>("[data-feature-ui]");
        if (textEls.length) {
          gsap.set(textEls, { y: 24, autoAlpha: 0, force3D: true });
        }
        if (uiEl) {
          gsap.set(uiEl, { y: 32, scale: 0.975, autoAlpha: 0, force3D: true });
        }
        const tl = gsap.timeline({
          scrollTrigger: {
            trigger: row,
            start: "top 86%",
            once: true,
          },
        });
        if (textEls.length) {
          tl.to(
            textEls,
            {
              y: 0,
              autoAlpha: 1,
              duration: 1.05,
              stagger: 0.11,
              ease: "power2.out",
              force3D: true,
              clearProps: "transform",
            },
            0,
          );
        }
        if (uiEl) {
          tl.to(
            uiEl,
            {
              y: 0,
              scale: 1,
              autoAlpha: 1,
              duration: 1.15,
              ease: "power2.out",
              force3D: true,
              clearProps: "transform",
            },
            0.08,
          );
        }
      });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <div ref={ref}>
      <PageHero
        tag="Complete management system"
        title="Everything your coaching center needs."
        accent="Nothing it doesn't."
        copy="One focused desktop application for the real work behind admissions, students, classes, money, exams and communication."
      />
      <section className="py-24 sm:py-32">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          {FEATURE_STORIES.map((f, i) => (
            <article
              key={f.k}
              data-feature-row
              className="grid gap-10 border-t border-stone-200 py-16 lg:grid-cols-2 lg:items-center lg:gap-16"
            >
              <div className={cn(i % 2 && "lg:order-2")}>
                <p
                  data-feature
                  className="text-xs font-bold uppercase tracking-[.2em] text-orange-600"
                >
                  {String(i + 1).padStart(2, "0")} · {f.k}
                </p>
                <h2
                  data-feature
                  className="mt-4 font-display text-3xl font-extrabold leading-tight text-ink sm:text-4xl"
                >
                  {f.title}
                </h2>
                <p
                  data-feature
                  className="mt-4 max-w-lg text-base leading-relaxed text-stone-500"
                >
                  {f.copy}
                </p>
                <div
                  data-feature
                  className="mt-6 flex flex-wrap items-center gap-2"
                >
                  {f.flow.map((s, j) => (
                    <span
                      key={s}
                      className="flex items-center gap-2 text-xs font-bold text-stone-500"
                    >
                      <span className="rounded-full border border-stone-200 bg-white px-3 py-1.5">
                        {s}
                      </span>
                      {j < f.flow.length - 1 && (
                        <ArrowRightIcon className="h-3 w-3 text-orange-400" />
                      )}
                    </span>
                  ))}
                </div>
              </div>
              <div
                data-feature-ui
                className={cn("perspective-[1200px]", i % 2 && "lg:order-1")}
              >
                <WindowFrame title={`Coachify — ${f.k}`}>{f.ui}</WindowFrame>
              </div>
            </article>
          ))}
        </div>
      </section>
      <PageCta
        title="See what Coachify can do for your institute."
        copy="Install the demo with a complete sample institution and experience each workflow yourself."
      />
    </div>
  );
}

const DAY = [
  [
    "8:00 AM",
    "Reception opens Coachify",
    "Today's classes, collections, attendance and pending tasks settle into one calm dashboard.",
    <DashboardScreen />,
  ],
  [
    "9:15 AM",
    "A student arrives for admission",
    "Guardian, course, batch, fee plan and student ID are completed in one guided flow.",
    <AdmitScene />,
  ],
  [
    "11:30 AM",
    "A teacher takes attendance",
    "The batch list is ready. Two absences are marked and guardian alerts queue automatically.",
    <AttendanceScreen />,
  ],
  [
    "1:20 PM",
    "A guardian pays fees",
    "Search by phone or student ID, see the exact due, collect and print the receipt.",
    <PaymentsScreen />,
  ],
  [
    "3:00 PM",
    "Exam marks go in",
    "Marks populate once. Totals, grades and rankings calculate without a spreadsheet.",
    <ExamsScreen />,
  ],
  [
    "5:00 PM",
    "The administrator checks reports",
    "Collection, attendance, outstanding dues and performance are already current.",
    <ReportsScreen />,
  ],
  [
    "6:30 PM",
    "Coachify closes the day safely",
    "A verified local backup protects the day before the office closes.",
    <BackupScene />,
  ],
] as const;

export function HowPage() {
  const ref = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const root = ref.current;
    if (!root || prefersReduced()) return;
    const ctx = gsap.context(() => {
      gsap.utils.toArray<HTMLElement>("[data-day]", root).forEach((dayEl) => {
        gsap.set(dayEl, { y: 28, autoAlpha: 0, force3D: true });
        gsap.to(dayEl, {
          y: 0,
          autoAlpha: 1,
          duration: 1.05,
          ease: "power2.out",
          force3D: true,
          clearProps: "transform",
          scrollTrigger: {
            trigger: dayEl,
            start: "top 86%",
            once: true,
          },
        });
      });
    }, ref);
    return () => ctx.revert();
  }, []);
  return (
    <div ref={ref}>
      <PageHero
        tag="A day with Coachify"
        title="One system quietly running"
        accent="the whole day."
        copy="Follow a coaching center from the first login to the final backup. The software stays out of the way while every operation stays connected."
      />
      <section className="bg-white py-24 sm:py-32">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="relative">
            <div className="absolute bottom-0 left-[39px] top-0 hidden w-px bg-stone-200 sm:block" />
            {DAY.map((d, i) => (
              <article
                key={d[0]}
                data-day
                className="relative grid gap-8 py-12 sm:grid-cols-[80px_0.8fr_1.2fr] sm:gap-8"
              >
                <div className="relative z-10">
                  <span className="tnum inline-flex rounded-full bg-orange-500 px-3 py-1.5 text-xs font-bold text-white">
                    {d[0]}
                  </span>
                </div>
                <div>
                  <p className="text-xs font-bold uppercase tracking-[.18em] text-stone-400">
                    Moment {i + 1}
                  </p>
                  <h2 className="mt-3 font-display text-2xl font-bold text-ink sm:text-3xl">
                    {d[1]}
                  </h2>
                  <p className="mt-3 text-[15px] leading-relaxed text-stone-500">
                    {d[2]}
                  </p>
                </div>
                <WindowFrame title={`Coachify · ${d[0]}`}>{d[3]}</WindowFrame>
              </article>
            ))}
          </div>
        </div>
      </section>
      <PageCta
        title="One system quietly running the whole day."
        copy="Try the same connected workflow on your own Windows PC."
      />
    </div>
  );
}

const GALLERY = [
  [
    "Dashboard",
    "A clear opening view of classes, collections, dues and attendance.",
    <DashboardScreen />,
  ],
  [
    "Students",
    "Every learner, guardian and history in one searchable record.",
    <StudentsScreen />,
  ],
  [
    "Admissions",
    "A guided path from inquiry to batch and initial payment.",
    <AdmitScene />,
  ],
  [
    "Attendance",
    "Fast batch roll call with useful analytics built in.",
    <AttendanceScreen />,
  ],
  [
    "Finance",
    "Collections and outstanding dues without duplicate entry.",
    <DuesScreen />,
  ],
  [
    "Cashier",
    "Search, collect, receipt and confirm in one short flow.",
    <PaymentsScreen />,
  ],
  [
    "Exams",
    "Schedule exams and enter marks without external sheets.",
    <ExamsScreen />,
  ],
  ["Results", "Grades, ranks and printable report cards.", <ResultsScreen />],
  [
    "Reports",
    "Operational answers ready to print or export.",
    <ReportsScreen />,
  ],
  [
    "Communication",
    "Action-based alerts that keep guardians informed.",
    <NotifyScene />,
  ],
  [
    "Settings",
    "Institution, grading, receipts and backup preferences.",
    <SetupScene />,
  ],
] as const;

export function ScreensPage() {
  const [active, setActive] = useState(0);
  const [zoom, setZoom] = useState(false);
  const selected = GALLERY[active];
  return (
    <div>
      <PageHero
        tag="Interactive product gallery"
        title="See Coachify before"
        accent="you install it."
        copy="Explore the real workflows that reception, accounts, teachers and owners use every day."
      />
      <section className="pb-28">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="no-scrollbar flex gap-2 overflow-x-auto border-b border-stone-200 pb-5">
            {GALLERY.map((g, i) => (
              <button
                key={g[0]}
                onClick={() => setActive(i)}
                className={cn(
                  "shrink-0 cursor-pointer rounded-full px-4 py-2 text-sm font-semibold transition",
                  i === active
                    ? "bg-orange-500 text-white"
                    : "bg-white text-stone-500 hover:text-ink",
                )}
              >
                {g[0]}
              </button>
            ))}
          </div>
          <div className="mt-10 grid gap-10 lg:grid-cols-[0.3fr_0.7fr] lg:items-center">
            <div>
              <p className="text-xs font-bold uppercase tracking-[.2em] text-orange-600">
                Screen {String(active + 1).padStart(2, "0")}
              </p>
              <h2 className="mt-3 font-display text-4xl font-extrabold text-ink">
                {selected[0]}
              </h2>
              <p className="mt-4 text-base leading-relaxed text-stone-500">
                {selected[1]}
              </p>
              <button
                onClick={() => setZoom(true)}
                className="mt-6 cursor-pointer text-sm font-bold text-orange-600"
              >
                Enlarge screen ↗
              </button>
            </div>
            <div key={selected[0]} className="animate-[fadeIn_.35s_ease-out]">
              <WindowFrame
                title={`Coachify — ${selected[0]}`}
                bodyClassName="min-h-[420px]"
              >
                {selected[2]}
              </WindowFrame>
            </div>
          </div>
        </div>
      </section>
      {zoom && (
        <div
          className="fixed inset-0 z-[90] flex items-center justify-center bg-ink/80 p-4 backdrop-blur"
          onClick={() => setZoom(false)}
        >
          <div
            className="w-full max-w-6xl"
            onClick={(e) => e.stopPropagation()}
          >
            <WindowFrame
              title={`Coachify — ${selected[0]}`}
              bodyClassName="min-h-[520px]"
            >
              {selected[2]}
            </WindowFrame>
            <button
              onClick={() => setZoom(false)}
              className="mx-auto mt-4 block cursor-pointer rounded-full bg-white px-5 py-2 text-sm font-bold text-ink"
            >
              Close
            </button>
          </div>
        </div>
      )}
      <PageCta
        title="Looks simple because it was designed to be simple."
        copy="The downloadable demo includes realistic data, so every screen is ready to explore."
      />
    </div>
  );
}

export function OfflinePage() {
  return (
    <div>
      <PageHero
        dark
        tag="Offline-first by design"
        title="Your software shouldn't stop because"
        accent="your internet did."
        copy="Coachify's core workflows run locally. Admissions, attendance, payments, receipts, exams and reports remain available when Wi-Fi disappears."
      >
        <CheckList
          dark
          items={[
            "Admissions keep running",
            "Attendance stays available",
            "Payments and receipts work",
            "Exams and reports remain open",
          ]}
        />
      </PageHero>
      <Offline />
      <section className="bg-white py-24">
        <div className="mx-auto grid max-w-7xl gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:px-8">
          <div>
            <SectionTag>Optional connectivity</SectionTag>
            <h2 className="mt-5 font-display text-4xl font-extrabold text-ink">
              Offline doesn't mean outdated. It means dependable.
            </h2>
            <p className="mt-4 leading-relaxed text-stone-500">
              SMS, email, software updates and license services may use internet
              connectivity. If the connection disappears, core operations
              continue and queued messages can wait safely until it returns.
            </p>
          </div>
          <div className="rounded-3xl bg-cream p-8">
            <CheckList
              items={[
                "Core records stay local",
                "Messages can queue",
                "Updates happen when you choose",
                "Backups remain under your control",
              ]}
            />
          </div>
        </div>
      </section>
      <PageCta
        title="Put dependable software on your own PC."
        copy="Download Coachify Demo and test every core workflow with Wi-Fi switched off."
      />
    </div>
  );
}

export function PricingPage() {
  return (
    <div>
      <PageHero
        tag="Straightforward licensing"
        title="Choose the setup that fits"
        accent="your office."
        copy="Start with one primary PC or connect the team over your local network. Contact us for current pricing; we do not invent placeholder prices."
      />
      <Pricing />
      <PageCta
        title="Not sure which setup you need?"
        copy="Tell us your student count and number of office PCs. We will recommend the smallest suitable option."
      />
    </div>
  );
}

export function WhyPage() {
  const old = [
    "Registers",
    "Excel sheets",
    "Paper receipts",
    "WhatsApp notes",
    "Scattered files",
  ];
  return (
    <div>
      <PageHero
        tag="Why Coachify"
        title="Management software should help your staff,"
        accent="not become more work."
        copy="Coachify is simple enough for reception, fast enough for accounts, useful for teachers and clear enough for owners."
      />
      <section className="py-24">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid gap-5 lg:grid-cols-[1fr_auto_1fr] lg:items-center">
            <div className="rounded-3xl border border-stone-200 bg-white p-8">
              <p className="text-xs font-bold uppercase tracking-[.2em] text-stone-400">
                Before
              </p>
              <div className="mt-6 space-y-3">
                {old.map((x) => (
                  <div
                    key={x}
                    className="rounded-xl bg-stone-100 px-4 py-3 font-semibold text-stone-500"
                  >
                    {x}
                  </div>
                ))}
              </div>
            </div>
            <ArrowRightIcon className="mx-auto h-8 w-8 rotate-90 text-orange-500 lg:rotate-0" />
            <div className="relative overflow-hidden rounded-3xl bg-ink p-8 text-white">
              <div className="absolute inset-0 bg-grid-dark opacity-40" />
              <div className="relative">
                <p className="text-xs font-bold uppercase tracking-[.2em] text-orange-400">
                  After
                </p>
                <h2 className="mt-5 font-display text-4xl font-extrabold">
                  One clear system.
                </h2>
                <p className="mt-4 leading-relaxed text-stone-400">
                  Every student, payment, attendance mark, exam and report stays
                  connected and understandable.
                </p>
                <div className="mt-7">
                  <CheckList
                    dark
                    items={[
                      "One record per student",
                      "One source of truth",
                      "One fast counter flow",
                      "One safe backup routine",
                    ]}
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
      <section className="bg-white py-24">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
            {[
              ["Reception", "Clear guided forms"],
              ["Accounts", "Fast collections"],
              ["Teachers", "Simple attendance & marks"],
              ["Owners", "Reports that answer questions"],
            ].map((x) => (
              <div key={x[0]} className="border-t-2 border-orange-500 pt-5">
                <h3 className="font-display text-xl font-bold text-ink">
                  {x[0]}
                </h3>
                <p className="mt-2 text-sm text-stone-500">{x[1]}</p>
              </div>
            ))}
          </div>
        </div>
      </section>
      <PageCta
        title="Give your staff one system they can trust."
        copy="Explore Coachify with a complete demo institution already included."
      />
    </div>
  );
}

export function FaqPage() {
  return (
    <div>
      <PageHero
        tag="Practical answers"
        title="Questions before you"
        accent="bring Coachify in."
        copy="Understand offline use, local data, LAN setup, backups, receipts, communication and licensing."
        primary="Download Coachify Demo"
      />
      <Faq />
      <PageCta
        title="The fastest answer is often a short conversation."
        copy="Book a demo and walk us through the way your institute works today."
      />
    </div>
  );
}
