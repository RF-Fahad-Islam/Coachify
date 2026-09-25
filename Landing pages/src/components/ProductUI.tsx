import type { ReactNode } from "react";
import { cn } from "../utils/cn";
import { LogoMark, PrinterIcon, DatabaseIcon } from "./icons";
import { MockBars } from "./charts";
import {
  INSTITUTION,
  DASHBOARD,
  STUDENTS,
  STUDENT_FILTERS,
  STUDENT_COUNTS,
  ATTENDANCE,
  PAYMENTS,
  DUES,
  EXAMS,
  RESULTS,
  REPORTS,
  NOTIFICATIONS,
  BACKUPS,
  SETUP,
  ADMISSION,
  BATCHES,
  PRINT_RECEIPT,
  TODAY_BATCHES,
  inr,
  inrShort,
  type StudentStatus,
} from "../data/appData";

/* ------------------------------------------------------------------ */
/* Shared mini-parts                                                   */
/* ------------------------------------------------------------------ */

export function WindowFrame({
  title,
  children,
  className,
  bodyClassName,
}: {
  title: string;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
}) {
  return (
    <div
      className={cn(
        "overflow-hidden rounded-2xl border border-stone-200/80 bg-white shadow-lift",
        className,
      )}
    >
      <div className="flex items-center gap-3 border-b border-stone-100 bg-stone-50/80 px-4 py-2.5">
        <div className="flex gap-1.5">
          <span className="h-2.5 w-2.5 rounded-full bg-stone-200" />
          <span className="h-2.5 w-2.5 rounded-full bg-stone-200" />
          <span className="h-2.5 w-2.5 rounded-full bg-stone-200" />
        </div>
        <span className="flex-1 truncate text-center text-[11px] font-medium text-stone-400">
          {title}
        </span>
        <span className="flex items-center gap-1.5 rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-[9px] font-semibold text-emerald-700">
          <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
          LOCAL · OFFLINE
        </span>
      </div>
      <div className={cn("overflow-hidden p-3 sm:p-4", bodyClassName)}>
        {children}
      </div>
    </div>
  );
}

const AV_COLORS = [
  "bg-orange-100 text-orange-700",
  "bg-sky-100 text-sky-700",
  "bg-violet-100 text-violet-700",
  "bg-emerald-100 text-emerald-700",
  "bg-rose-100 text-rose-700",
  "bg-amber-100 text-amber-700",
];

export function Av({
  name,
  i = 0,
  size = "h-7 w-7 text-[10px]",
}: {
  name: string;
  i?: number;
  size?: string;
}) {
  const initials = name
    .split(" ")
    .map((w) => w[0])
    .slice(0, 2)
    .join("");
  return (
    <span
      className={cn(
        "flex shrink-0 items-center justify-center rounded-full font-bold",
        AV_COLORS[i % AV_COLORS.length],
        size,
      )}
    >
      {initials}
    </span>
  );
}

export function Chip({
  children,
  tone = "stone",
  className,
}: {
  children: ReactNode;
  tone?: "stone" | "orange" | "green" | "red" | "amber" | "sky";
  className?: string;
}) {
  const tones = {
    stone: "bg-stone-100 text-stone-600",
    orange: "bg-orange-100 text-orange-700",
    green: "bg-emerald-100 text-emerald-700",
    red: "bg-rose-100 text-rose-700",
    amber: "bg-amber-100 text-amber-700",
    sky: "bg-sky-100 text-sky-700",
  };
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-[9px] font-semibold whitespace-nowrap",
        tones[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}

const statusTone: Record<StudentStatus, "green" | "amber" | "red"> = {
  paid: "green",
  due: "amber",
  overdue: "red",
};
const statusLabel = (s: (typeof STUDENTS)[number]) =>
  s.status === "paid"
    ? "Paid"
    : (s.dueLabel ?? (s.status === "overdue" ? "Overdue" : "Due"));

function Sidebar() {
  return (
    <div className="hidden w-10 shrink-0 flex-col items-center gap-2 rounded-xl bg-stone-100/70 py-2.5 sm:flex">
      <LogoMark className="mb-1 h-6 w-6" />
      {[
        "bg-orange-500",
        "bg-stone-300",
        "bg-stone-300",
        "bg-stone-300",
        "bg-stone-300",
        "bg-stone-300",
        "bg-stone-300",
      ].map((c, i) => (
        <span key={i} className={cn("h-5 w-5 rounded-md", c)} />
      ))}
    </div>
  );
}

function Stat({
  label,
  value,
  sub,
  dataPart,
}: {
  label: string;
  value: string;
  sub: string;
  dataPart?: boolean;
}) {
  return (
    <div
      data-part={dataPart || undefined}
      className="rounded-xl border border-stone-100 bg-white p-2.5"
    >
      <p className="text-[9px] font-semibold uppercase tracking-wider text-stone-400">
        {label}
      </p>
      <p className="tnum mt-0.5 font-display text-[17px] font-bold text-ink">
        {value}
      </p>
      <p className="text-[9px] font-medium text-stone-400">{sub}</p>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Screens                                                             */
/* ------------------------------------------------------------------ */

export function DashboardScreen() {
  return (
    <div className="flex gap-3">
      <Sidebar />
      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between">
          <div>
            <p className="font-display text-[13px] font-bold text-ink">
              {DASHBOARD.greeting}, {INSTITUTION.name}
            </p>
            <p className="text-[9.5px] text-stone-400">
              {DASHBOARD.dateLine} · {DASHBOARD.classesToday} classes today
            </p>
          </div>
          <Chip tone="green">All systems local</Chip>
        </div>
        <div className="mt-2.5 grid grid-cols-2 gap-2 sm:grid-cols-4">
          {DASHBOARD.stats.map((s) => (
            <Stat
              key={s.label}
              dataPart
              label={s.label}
              value={s.value}
              sub={s.sub}
            />
          ))}
        </div>
        <div
          data-part
          className="mt-2.5 rounded-xl border border-stone-100 p-2.5"
        >
          <div className="flex items-center justify-between">
            <p className="text-[10px] font-bold text-stone-600">
              Fee collection
            </p>
            <span className="text-[9px] font-medium text-stone-400">
              last 12 months
            </span>
          </div>
          <div className="mt-2 h-16">
            <MockBars
              labels={DASHBOARD.monthlyCollections.labels}
              values={DASHBOARD.monthlyCollections.values}
            />
          </div>
        </div>
        <div className="mt-2.5 grid grid-cols-2 gap-2">
          <div data-part className="rounded-xl border border-stone-100 p-2.5">
            <p className="mb-1.5 text-[10px] font-bold text-stone-600">
              Recent payments
            </p>
            {PAYMENTS.recent.map((s, i) => (
              <div key={s.name} className="flex items-center gap-1.5 py-1">
                <Av name={s.name} i={i} size="h-5 w-5 text-[7.5px]" />
                <span className="min-w-0 flex-1 truncate text-[9.5px] font-semibold text-stone-700">
                  {s.name}
                </span>
                <span className="tnum text-[9.5px] font-bold text-emerald-600">
                  {inrShort(s.amount)}
                </span>
              </div>
            ))}
          </div>
          <div data-part className="rounded-xl border border-stone-100 p-2.5">
            <p className="mb-1.5 text-[10px] font-bold text-stone-600">
              Today&rsquo;s batches
            </p>
            {TODAY_BATCHES.map((b, i) => (
              <div key={i} className="flex items-center justify-between py-1">
                <span className="tnum text-[9.5px] font-bold text-stone-500">
                  {b.time}
                </span>
                <span className="text-[9.5px] font-semibold text-stone-700">
                  {b.batch}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export function StudentsScreen() {
  return (
    <div>
      <div className="flex flex-wrap items-center gap-2">
        <p className="font-display text-[13px] font-bold text-ink">Students</p>
        <div className="ml-auto flex items-center gap-2 rounded-lg border border-stone-200 px-2.5 py-1.5 text-[9.5px] text-stone-400">
          <span className="h-2.5 w-2.5 rounded-full border border-stone-300" />
          Search name, receipt # or guardian…
        </div>
        <span className="rounded-lg bg-orange-500 px-2.5 py-1.5 text-[9.5px] font-bold text-white">
          + Admit
        </span>
      </div>
      <div className="mt-2 flex gap-1.5">
        {STUDENT_FILTERS.map((f, i) => (
          <Chip key={f} tone={i === 0 ? "orange" : "stone"}>
            {f}
          </Chip>
        ))}
      </div>
      <div className="mt-2.5 overflow-hidden rounded-xl border border-stone-100">
        <div className="grid grid-cols-[1.6fr_1fr_1fr_0.9fr_0.7fr] gap-2 border-b border-stone-100 bg-stone-50 px-3 py-1.5 text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
          <span>Student</span>
          <span>Batch</span>
          <span>Guardian</span>
          <span>Fee</span>
          <span>Last pay</span>
        </div>
        {STUDENTS.map((s, i) => (
          <div
            key={s.name}
            className="grid grid-cols-[1.6fr_1fr_1fr_0.9fr_0.7fr] items-center gap-2 px-3 py-1.5 text-[9.5px] odd:bg-white even:bg-stone-50/50"
          >
            <span className="flex items-center gap-1.5 font-semibold text-stone-700">
              <Av name={s.name} i={i} size="h-5 w-5 text-[7.5px]" />
              {s.name}
            </span>
            <span className="truncate text-stone-500">{s.batch}</span>
            <span className="truncate text-stone-500">{s.guardian}</span>
            <span>
              <Chip tone={statusTone[s.status]}>{statusLabel(s)}</Chip>
            </span>
            <span className="tnum text-stone-400">{s.lastPay}</span>
          </div>
        ))}
      </div>
      <p className="mt-1.5 text-[9px] font-medium text-stone-400">
        {STUDENT_COUNTS.students.toLocaleString("en-US")} students ·{" "}
        {STUDENT_COUNTS.batches} batches ·{" "}
        {STUDENT_COUNTS.guardians.toLocaleString("en-US")} guardians on file
      </p>
    </div>
  );
}

export function AttendanceScreen() {
  const present = ATTENDANCE.roster.filter((r) => r.present).length;
  const shown = ATTENDANCE.roster.length;
  const pct = Math.round((present / ATTENDANCE.totalStudents) * 100);
  return (
    <div className="flex flex-col gap-2.5 sm:flex-row">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <p className="font-display text-[13px] font-bold text-ink">
            Attendance
          </p>
          <Chip tone="orange">
            {ATTENDANCE.batch} · {ATTENDANCE.totalStudents} students
          </Chip>
          <span className="ml-auto text-[9.5px] font-medium text-stone-400">
            {ATTENDANCE.date}
          </span>
        </div>
        <div className="mt-2 space-y-1">
          {ATTENDANCE.roster.map((r, i) => (
            <div
              key={r.name}
              className="flex items-center gap-2 rounded-lg bg-stone-50 px-2.5 py-1.5"
            >
              <Av name={r.name} i={i} size="h-5 w-5 text-[7.5px]" />
              <span className="min-w-0 flex-1 truncate text-[9.5px] font-semibold text-stone-700">
                {r.name}
              </span>
              <span
                className={cn(
                  "tnum rounded-md px-1.5 py-0.5 text-[8.5px] font-bold",
                  !r.present
                    ? "bg-rose-100 text-rose-600"
                    : "bg-emerald-100 text-emerald-700",
                )}
              >
                {r.present ? "P" : "A"}
              </span>
            </div>
          ))}
        </div>
        <div className="mt-2 flex gap-1.5">
          <span className="rounded-lg bg-emerald-500 px-2.5 py-1 text-[9px] font-bold text-white">
            All present
          </span>
          <span className="rounded-lg bg-white px-2.5 py-1 text-[9px] font-bold text-stone-500 ring-1 ring-stone-200">
            Save day
          </span>
        </div>
      </div>
      <div className="w-full rounded-xl border border-stone-100 p-2.5 sm:w-44">
        <p className="text-[10px] font-bold text-stone-600">Today</p>
        <div className="tnum mt-1.5 flex items-baseline gap-1">
          <span className="font-display text-xl font-bold text-ink">
            {present}
            <span className="text-sm text-stone-400">/{shown}</span>
          </span>
          <Chip tone="green">{pct}%</Chip>
        </div>
        <div className="mt-2 grid grid-cols-7 gap-1">
          {Array.from({ length: ATTENDANCE.calendar.totalDays }).map((_, i) => {
            const { filled } = ATTENDANCE.calendar;
            const off = i % 7 === 5 || i % 7 === 6;
            return (
              <span
                key={i}
                className={cn(
                  "h-2.5 rounded-[3px]",
                  off
                    ? "bg-stone-100"
                    : i < filled
                      ? "bg-orange-200"
                      : i === filled
                        ? "bg-orange-500"
                        : "bg-stone-100",
                )}
              />
            );
          })}
        </div>
        <p className="mt-2 rounded-lg bg-orange-50 px-2 py-1.5 text-[8.5px] font-semibold text-orange-700">
          {ATTENDANCE.absentSms}
        </p>
      </div>
    </div>
  );
}

export function PaymentsScreen() {
  const c = PAYMENTS.cashier;
  const r = PAYMENTS.receipt;
  return (
    <div className="flex flex-col gap-2.5 sm:flex-row">
      <div className="flex-1 rounded-xl border border-stone-100 p-3">
        <p className="font-display text-[13px] font-bold text-ink">
          Collect payment
        </p>
        <div className="mt-2.5 space-y-2">
          <div>
            <p className="text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
              Student
            </p>
            <div className="mt-1 flex items-center gap-2 rounded-lg border border-stone-200 px-2.5 py-1.5 text-[10px] font-semibold text-stone-700">
              <Av name={c.student} i={0} size="h-5 w-5 text-[7.5px]" />{" "}
              {c.student} — {c.batch}
            </div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <p className="text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
                Outstanding
              </p>
              <p className="tnum mt-1 text-[13px] font-bold text-rose-600">
                {inrShort(c.outstanding)}
              </p>
            </div>
            <div>
              <p className="text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
                Amount
              </p>
              <p className="tnum mt-1 rounded-lg border border-orange-300 bg-orange-50 px-2.5 py-1 text-[13px] font-bold text-ink">
                {inrShort(c.amount)}
              </p>
            </div>
          </div>
          <div>
            <p className="mb-1 text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
              Mode
            </p>
            <div className="flex gap-1.5">
              {c.modes.map((m) => (
                <Chip key={m} tone={m === c.activeMode ? "orange" : "stone"}>
                  {m}
                </Chip>
              ))}
            </div>
          </div>
          <p className="text-[8.5px] font-semibold text-stone-400">
            {c.installment} · {c.term} · {c.note}
          </p>
          <div className="flex items-center gap-2 pt-1">
            <span className="flex-1 rounded-lg bg-orange-500 py-2 text-center text-[10px] font-bold text-white shadow-sm shadow-orange-500/30">
              Save &amp; print receipt
            </span>
          </div>
        </div>
      </div>
      <div className="w-full sm:w-52">
        <div className="relative overflow-hidden rounded-xl border border-dashed border-stone-300 bg-white p-3">
          <div className="flex items-center gap-1.5">
            <LogoMark className="h-5 w-5" />
            <p className="text-[9px] font-bold text-ink">{INSTITUTION.name}</p>
          </div>
          <p className="mt-1 text-[8px] text-stone-400">Fee Receipt</p>
          <div className="tnum mt-2 space-y-1 text-[8.5px] text-stone-600">
            <div className="flex justify-between">
              <span>Receipt #</span>
              <span className="font-bold">{r.number}</span>
            </div>
            <div className="flex justify-between">
              <span>Student</span>
              <span className="font-bold">{r.student}</span>
            </div>
            <div className="flex justify-between">
              <span>Batch</span>
              <span>{r.batch}</span>
            </div>
            <div className="flex justify-between">
              <span>Date</span>
              <span>{r.date}</span>
            </div>
            <div className="flex justify-between border-t border-dashed border-stone-200 pt-1 text-[10px]">
              <span className="font-bold">Amount</span>
              <span className="font-bold">{inr(r.amount)}</span>
            </div>
          </div>
          <span className="absolute right-2 top-8 rotate-[-14deg] rounded border-2 border-emerald-500 px-1.5 py-0.5 text-[9px] font-black uppercase tracking-widest text-emerald-500">
            Paid
          </span>
        </div>
        <p className="mt-1.5 text-center text-[8.5px] font-medium text-stone-400">
          {r.note}
        </p>
      </div>
    </div>
  );
}

export function DuesScreen() {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="grid shrink-0 grid-cols-3 gap-2">
        {DUES.summary.map((s) => (
          <div
            key={s.label}
            className="rounded-xl border border-stone-100 p-2.5"
          >
            <p className="text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
              {s.label}
            </p>
            <p
              className={cn(
                "tnum mt-0.5 font-display text-[16px] font-bold",
                s.tone === "red" ? "text-rose-600" : "text-ink",
              )}
            >
              {s.value}
            </p>
          </div>
        ))}
      </div>
      <div className="mt-2.5 min-h-0 flex-1 overflow-hidden rounded-xl border border-stone-100">
        <div className="grid grid-cols-[1.5fr_1fr_0.8fr_0.8fr_0.9fr] gap-2 border-b border-stone-100 bg-stone-50 px-3 py-1.5 text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
          <span>Student</span>
          <span>Batch</span>
          <span>Due</span>
          <span>By</span>
          <span>Status</span>
        </div>
        {DUES.rows.map((row) => (
          <div
            key={row.name}
            className="grid grid-cols-[1.5fr_1fr_0.8fr_0.8fr_0.9fr] items-center gap-2 px-3 py-1.5 text-[9.5px] odd:bg-white even:bg-stone-50/50"
          >
            <span className="truncate font-semibold text-stone-700">
              {row.name}
            </span>
            <span className="truncate text-stone-500">{row.batch}</span>
            <span className="tnum font-bold text-stone-700">{row.due}</span>
            <span className="tnum text-stone-400">{row.by}</span>
            <span>
              <Chip tone={row.tone}>{row.status}</Chip>
            </span>
          </div>
        ))}
      </div>
      <div className="mt-2 flex shrink-0 items-center justify-between">
        <p className="text-[9px] font-medium text-stone-400">{DUES.showing}</p>
        <span className="rounded-lg bg-ink px-2.5 py-1.5 text-[9px] font-bold text-white">
          {DUES.action}
        </span>
      </div>
    </div>
  );
}

export function ExamsScreen() {
  return (
    <div>
      <div className="flex items-center gap-2">
        <p className="font-display text-[13px] font-bold text-ink">Exams</p>
        <div className="flex gap-1.5">
          <Chip tone="orange">Schedule</Chip>
          <Chip>Mark entry</Chip>
          <Chip>Papers</Chip>
        </div>
      </div>
      <div className="mt-2.5 grid gap-2 sm:grid-cols-2">
        {EXAMS.list.map((e) => (
          <div
            key={e.name}
            className="flex items-center justify-between gap-2 rounded-xl border border-stone-100 p-2.5"
          >
            <div className="min-w-0">
              <p className="truncate text-[10px] font-bold text-stone-700">
                {e.name}
              </p>
              <p className="truncate text-[8.5px] text-stone-400">{e.meta}</p>
            </div>
            <div className="text-right">
              <p className="tnum text-[9px] font-bold text-stone-500">
                {e.date}
              </p>
              <Chip tone={e.tone}>{e.status}</Chip>
            </div>
          </div>
        ))}
      </div>
      <div className="mt-2 rounded-xl border border-stone-100 p-2.5">
        <div className="flex justify-between">
          <p className="text-[10px] font-bold text-stone-600">
            {EXAMS.markEntry.title}
          </p>
          <span className="text-[9px] text-stone-400">
            {EXAMS.markEntry.note}
          </span>
        </div>
        <div className="tnum mt-1.5 grid grid-cols-8 gap-1.5">
          {EXAMS.markEntry.marks.map((v, i) => (
            <span
              key={i}
              className={cn(
                "rounded-md border px-1 py-1 text-center text-[8.5px] font-bold",
                v
                  ? "border-stone-200 bg-white text-stone-700"
                  : "border-dashed border-stone-200 text-stone-300",
              )}
            >
              {v || "—"}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}

export function ResultsScreen() {
  return (
    <div>
      <div className="flex flex-wrap items-center gap-2">
        <p className="font-display text-[13px] font-bold text-ink">
          {RESULTS.title}
        </p>
        <div className="ml-auto flex gap-1.5">
          {RESULTS.gradeBands.map((g) => (
            <Chip key={g}>{g}</Chip>
          ))}
        </div>
      </div>
      <div className="mt-2.5 overflow-hidden rounded-xl border border-stone-100">
        <div className="grid grid-cols-[1.6fr_0.7fr_0.7fr_0.7fr_0.8fr_0.7fr] gap-2 border-b border-stone-100 bg-stone-50 px-3 py-1.5 text-[8.5px] font-bold uppercase tracking-wider text-stone-400">
          {RESULTS.columns.map((c) => (
            <span key={c}>{c}</span>
          ))}
        </div>
        {RESULTS.rows.map((row, i) => (
          <div
            key={row.name}
            className="tnum grid grid-cols-[1.6fr_0.7fr_0.7fr_0.7fr_0.8fr_0.7fr] items-center gap-2 px-3 py-1.5 text-[9.5px] odd:bg-white even:bg-stone-50/50"
          >
            <span className="flex items-center gap-1.5 font-semibold text-stone-700">
              {i + 1}. {row.name}
            </span>
            <span>{row.phy}</span>
            <span>{row.chem}</span>
            <span>{row.math}</span>
            <span className="font-bold text-ink">{row.total}</span>
            <span>
              <Chip tone={row.tone}>{row.grade}</Chip>
            </span>
          </div>
        ))}
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        <span className="rounded-lg bg-orange-500 px-2.5 py-1.5 text-[9px] font-bold text-white">
          Publish to parents
        </span>
        <span className="rounded-lg bg-white px-2.5 py-1.5 text-[9px] font-bold text-stone-500 ring-1 ring-stone-200">
          Print report cards
        </span>
        <span className="ml-auto text-[9px] font-semibold text-emerald-600">
          {RESULTS.rankNote}
        </span>
      </div>
    </div>
  );
}

export function ReportsScreen() {
  return (
    <div>
      <div className="flex items-center gap-2">
        <p className="font-display text-[13px] font-bold text-ink">Reports</p>
        <div className="ml-auto flex gap-1.5">
          <span className="rounded-lg bg-ink px-2.5 py-1.5 text-[9px] font-bold text-white">
            Export PDF
          </span>
          <span className="rounded-lg bg-white px-2.5 py-1.5 text-[9px] font-bold text-stone-500 ring-1 ring-stone-200">
            Excel
          </span>
        </div>
      </div>
      <div className="mt-2.5 grid gap-2 sm:grid-cols-2">
        <div className="rounded-xl border border-stone-100 p-2.5">
          <p className="text-[10px] font-bold text-stone-600">
            {REPORTS.monthly.title}
          </p>
          <div className="mt-2 h-16">
            <MockBars
              labels={REPORTS.monthly.labels}
              values={REPORTS.monthly.values}
            />
          </div>
          <p className="tnum mt-1 text-[8.5px] font-medium text-stone-400">
            {REPORTS.monthly.footnote}
          </p>
        </div>
        <div className="rounded-xl border border-stone-100 p-2.5">
          <p className="text-[10px] font-bold text-stone-600">
            Attendance by batch
          </p>
          <div className="mt-2 space-y-1.5">
            {REPORTS.attendanceByBatch.map((b) => (
              <div key={b.batch} className="flex items-center gap-2">
                <span className="w-20 truncate text-[8.5px] font-semibold text-stone-500">
                  {b.batch}
                </span>
                <div className="h-2 flex-1 overflow-hidden rounded-full bg-stone-100">
                  <div
                    className="h-full rounded-full bg-orange-400"
                    style={{ width: `${b.pct}%` }}
                  />
                </div>
                <span className="tnum text-[8.5px] font-bold text-stone-600">
                  {b.pct}%
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
      <div className="mt-2 grid grid-cols-2 gap-1.5 sm:grid-cols-5">
        {REPORTS.templates.map((tpl) => (
          <span
            key={tpl}
            className="flex items-center gap-1.5 rounded-lg bg-stone-50 px-2 py-1.5 text-[8.5px] font-semibold text-stone-600"
          >
            <PrinterIcon className="h-3 w-3 shrink-0 text-stone-400" />
            {tpl}
          </span>
        ))}
      </div>
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Compact scenes for the "How it works" story                         */
/* ------------------------------------------------------------------ */

export function SetupScene() {
  return (
    <div className="mx-auto max-w-sm rounded-xl border border-stone-100 p-4">
      <div className="flex items-center gap-2.5">
        <span className="flex h-9 w-9 items-center justify-center rounded-xl bg-orange-100 text-[13px] font-black text-orange-600">
          A
        </span>
        <div>
          <p className="text-[11px] font-bold text-ink">{INSTITUTION.name}</p>
          <p className="text-[9px] text-stone-400">{SETUP.title}</p>
        </div>
        <Chip tone="green" className="ml-auto">
          {SETUP.step}
        </Chip>
      </div>
      <div className="mt-3 space-y-2 text-[9.5px]">
        {SETUP.fields.map((f) => (
          <div
            key={f.label}
            className="flex items-center justify-between rounded-lg bg-stone-50 px-3 py-2"
          >
            <span className="font-semibold text-stone-500">{f.label}</span>
            <span className="tnum font-bold text-stone-700">{f.value}</span>
          </div>
        ))}
      </div>
      <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-stone-100">
        <div
          className="h-full rounded-full bg-orange-500"
          style={{ width: `${SETUP.progress}%` }}
        />
      </div>
      <p className="mt-1.5 text-center text-[9px] font-medium text-stone-400">
        {SETUP.footnote}
      </p>
    </div>
  );
}

export function AdmitScene() {
  const id = ADMISSION.idCard;
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
      <div className="flex-1 rounded-xl border border-stone-100 p-3.5">
        <p className="text-[11px] font-bold text-ink">New admission</p>
        <div className="mt-2.5 space-y-1.5 text-[9.5px]">
          {ADMISSION.fields.map((f) => (
            <div
              key={f.label}
              className="flex items-center justify-between rounded-lg bg-stone-50 px-3 py-1.5"
            >
              <span className="font-semibold text-stone-500">{f.label}</span>
              <span className="tnum font-bold text-stone-700">{f.value}</span>
            </div>
          ))}
        </div>
        <div className="mt-2.5 flex gap-1.5">
          <span className="flex-1 rounded-lg bg-orange-500 py-1.5 text-center text-[9.5px] font-bold text-white">
            Admit student
          </span>
        </div>
      </div>
      <div className="w-40 shrink-0 rounded-xl border border-stone-200 bg-gradient-to-b from-orange-50 to-white p-3">
        <div className="flex items-center gap-1.5">
          <LogoMark className="h-4 w-4" />
          <p className="text-[8px] font-bold text-stone-600">STUDENT ID</p>
        </div>
        <div className="mt-2 flex items-center gap-2">
          <span className="flex h-10 w-10 items-center justify-center rounded-lg bg-orange-200 text-[11px] font-black text-orange-700">
            {id.initials}
          </span>
          <div>
            <p className="text-[9.5px] font-bold text-ink">{id.student}</p>
            <p className="tnum text-[8px] text-stone-500">ID {id.id}</p>
          </div>
        </div>
        <p className="mt-1.5 text-[8px] text-stone-500">{id.batch}</p>
        <div className="mt-2 flex gap-px">
          {Array.from({ length: 26 }).map((_, i) => (
            <span
              key={i}
              className={cn(
                "w-0.5 bg-stone-800",
                i % 3 === 0 ? "h-3.5" : "h-2.5",
              )}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

export function BatchesScene() {
  return (
    <div className="grid gap-2.5 sm:grid-cols-3">
      {BATCHES.map((b, bi) => (
        <div key={b.name} className="rounded-xl border border-stone-100 p-3">
          <div className="flex items-center justify-between">
            <p className="text-[10.5px] font-bold text-ink">{b.name}</p>
            <Chip tone={bi === 0 ? "orange" : "stone"}>{b.students}</Chip>
          </div>
          <p className="mt-1 text-[9px] text-stone-400">
            {b.subjects} · {b.timing}
          </p>
          <div className="mt-2 flex -space-x-1.5">
            {b.teachers.map((t, i) => (
              <span
                key={t}
                className={cn(
                  "flex h-6 w-6 items-center justify-center rounded-full text-[7.5px] font-bold ring-2 ring-white",
                  AV_COLORS[i % AV_COLORS.length],
                )}
              >
                {t}
              </span>
            ))}
            <span className="ml-2 self-center text-[8.5px] font-medium text-stone-400">
              {b.teacherCount} teachers
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

export function ReceiptScene() {
  const r = PRINT_RECEIPT;
  return (
    <div className="mx-auto max-w-[240px]">
      <div className="relative rounded-xl border border-dashed border-stone-300 bg-white p-4">
        <div className="flex items-center gap-1.5">
          <LogoMark className="h-5 w-5" />
          <p className="text-[9.5px] font-bold text-ink">{INSTITUTION.name}</p>
        </div>
        <p className="mt-0.5 text-[8px] tracking-widest text-stone-400">
          FEE RECEIPT · DUPLICATE
        </p>
        <div className="tnum mt-2.5 space-y-1 text-[9px] text-stone-600">
          <div className="flex justify-between">
            <span>Receipt #</span>
            <span className="font-bold">{r.number}</span>
          </div>
          <div className="flex justify-between">
            <span>Student</span>
            <span className="font-bold">{r.student}</span>
          </div>
          <div className="flex justify-between">
            <span>Batch</span>
            <span>{r.batch}</span>
          </div>
          <div className="flex justify-between">
            <span>Installment</span>
            <span>{r.installment}</span>
          </div>
          <div className="flex justify-between">
            <span>Mode</span>
            <span>{r.mode}</span>
          </div>
          <div className="flex justify-between border-t border-dashed border-stone-200 pt-1 text-[11px]">
            <span className="font-bold">Amount</span>
            <span className="font-bold">{inr(r.amount)}</span>
          </div>
        </div>
        <span className="absolute right-3 top-10 rotate-[-14deg] rounded border-2 border-emerald-500 px-2 py-0.5 text-[10px] font-black uppercase tracking-widest text-emerald-500">
          Paid
        </span>
        <div className="mt-3 flex gap-px opacity-70">
          {Array.from({ length: 38 }).map((_, i) => (
            <span
              key={i}
              className={cn("w-px bg-stone-800", i % 2 === 0 ? "h-3" : "h-2")}
            />
          ))}
        </div>
      </div>
      <div className="mt-2 flex justify-center gap-1.5">
        <Chip tone="green">{r.chips[0]}</Chip>
        <Chip tone="stone">{r.chips[1]}</Chip>
      </div>
    </div>
  );
}

export function NotifyScene() {
  return (
    <div className="mx-auto flex h-full max-w-xs items-center justify-center overflow-hidden py-2">
      <div className="w-full rounded-2xl bg-stone-900 p-3.5">
        <p className="text-center text-[9px] font-semibold text-stone-400">
          {NOTIFICATIONS.phoneTime}
        </p>
        <div className="mt-2.5 space-y-2">
          {NOTIFICATIONS.messages.map((m) => (
            <div key={m.text} className="max-w-[92%]">
              <div
                className={cn(
                  "rounded-xl rounded-tl-sm px-3 py-2 text-[9px] leading-relaxed text-white/90",
                  m.tone === "green" && "bg-emerald-600/90",
                  m.tone === "orange" && "bg-orange-500/90",
                  m.tone === "red" && "bg-rose-600/80",
                )}
              >
                {m.text}
              </div>
              <p className="mt-0.5 text-right text-[7.5px] text-stone-500">
                Delivered ✓✓
              </p>
            </div>
          ))}
        </div>
        <p className="mt-2 text-center text-[8.5px] font-semibold text-stone-500">
          {NOTIFICATIONS.weeklyCount} reminders sent this week
        </p>
      </div>
    </div>
  );
}

export function BackupScene() {
  return (
    <div className="mx-auto max-w-sm">
      <div className="flex items-center gap-3 rounded-xl border border-stone-100 p-3.5">
        <span className="flex h-10 w-10 items-center justify-center rounded-xl bg-orange-100 text-orange-600">
          <DatabaseIcon className="h-5.5 w-5.5" />
        </span>
        <div className="flex-1">
          <div className="flex justify-between text-[9.5px] font-bold">
            <span className="text-ink">{BACKUPS.label}</span>
            <span className="tnum text-emerald-600">100%</span>
          </div>
          <div className="mt-1.5 h-1.5 overflow-hidden rounded-full bg-stone-100">
            <div className="h-full w-full rounded-full bg-emerald-500" />
          </div>
        </div>
      </div>
      <div className="mt-2 space-y-1.5">
        {BACKUPS.files.map((f) => (
          <div
            key={f.name}
            className="flex items-center justify-between rounded-lg bg-stone-50 px-3 py-2 text-[9px]"
          >
            <span className="font-mono font-semibold text-stone-600">
              {f.name}
            </span>
            <span className="tnum text-stone-400">{f.size}</span>
            <Chip tone="green">Verified</Chip>
          </div>
        ))}
      </div>
      <p className="mt-2 text-center text-[9px] font-medium text-stone-400">
        {BACKUPS.caption}
      </p>
    </div>
  );
}
