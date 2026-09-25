import { useLayoutEffect, useRef, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { revealUp } from "../lib/anim";
import { CtaButton, SectionTag, useCta } from "./shared";
import { ArrowRightIcon, CheckIcon, BrandLockup } from "./icons";

export function PageHero({
  tag,
  title,
  accent,
  copy,
  children,
  dark = false,
  primary = "Download Coachify Demo",
}: {
  tag: string;
  title: string;
  accent?: string;
  copy: string;
  children?: ReactNode;
  dark?: boolean;
  primary?: string;
}) {
  const ref = useRef<HTMLElement>(null);
  const cta = useCta();
  useLayoutEffect(
    () =>
      revealUp(ref.current!, "[data-hero]", {
        stagger: 0.11,
        y: 34,
        start: "top 95%",
      }),
    [],
  );
  return (
    <section
      ref={ref}
      className={`relative overflow-hidden pb-20 pt-36 sm:pb-28 sm:pt-44 ${dark ? "bg-ink text-white" : ""}`}
    >
      <div
        className={`absolute inset-0 -z-0 ${dark ? "bg-grid-dark opacity-50" : "bg-grid-warm opacity-65"}`}
      />
      <div className="absolute left-1/2 top-8 h-80 w-[680px] -translate-x-1/2 rounded-full bg-orange-400/15 blur-[110px]" />
      <div className="relative mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="max-w-4xl">
          <span data-hero>
            <SectionTag dark={dark}>{tag}</SectionTag>
          </span>
          <h1
            data-hero
            className={`mt-6 font-display text-5xl font-extrabold leading-[1.02] tracking-tight sm:text-6xl lg:text-[72px] ${dark ? "text-white" : "text-ink"}`}
          >
            {title}{" "}
            {accent && <span className="text-orange-500">{accent}</span>}
          </h1>
          <p
            data-hero
            className={`mt-6 max-w-2xl text-lg leading-relaxed sm:text-xl ${dark ? "text-stone-400" : "text-stone-500"}`}
          >
            {copy}
          </p>
          <div data-hero className="mt-9 flex flex-wrap gap-3">
            <CtaButton size="lg" onClick={() => cta("download")}>
              {primary}
            </CtaButton>
            <CtaButton
              size="lg"
              variant={dark ? "lightGhost" : "ghost"}
              onClick={() => cta("request", "Book a live demo")}
            >
              Book a Demo
            </CtaButton>
          </div>
        </div>
        {children && (
          <div data-hero className="mt-14">
            {children}
          </div>
        )}
      </div>
    </section>
  );
}

export function PageCta({ title, copy }: { title: string; copy: string }) {
  const cta = useCta();
  return (
    <section className="px-4 py-20 sm:px-6 sm:py-24 lg:px-8">
      <div className="relative mx-auto max-w-7xl overflow-hidden rounded-[34px] bg-orange-500 px-6 py-16 text-center text-white shadow-lift sm:px-12 sm:py-20">
        <div className="absolute inset-0 bg-grid-dark opacity-25" />
        <div className="absolute -top-24 left-1/2 h-56 w-[520px] -translate-x-1/2 rounded-full bg-white/20 blur-3xl" />
        <div className="relative">
          <BrandLockup
            className="mx-auto justify-center"
            markClassName="h-11 w-11 shadow-lg"
            textClassName="text-2xl"
            dark
          />
          <h2 className="mx-auto mt-5 max-w-3xl font-display text-3xl font-extrabold leading-tight sm:text-5xl">
            {title}
          </h2>
          <p className="mx-auto mt-4 max-w-xl text-base leading-relaxed text-orange-50/90">
            {copy}
          </p>
          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <CtaButton
              size="lg"
              variant="light"
              onClick={() => cta("download")}
            >
              Download Coachify Demo
            </CtaButton>
            <CtaButton size="lg" variant="dark" onClick={() => cta("request")}>
              Get Coachify
            </CtaButton>
          </div>
        </div>
      </div>
    </section>
  );
}

export function CheckList({
  items,
  dark = false,
}: {
  items: string[];
  dark?: boolean;
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {items.map((item) => (
        <div
          key={item}
          className={`flex items-center gap-3 rounded-xl px-4 py-3 text-sm font-semibold ${dark ? "bg-white/5 text-stone-300" : "bg-white text-stone-700 shadow-card"}`}
        >
          <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-orange-100 text-orange-600">
            <CheckIcon className="h-3 w-3" />
          </span>
          {item}
        </div>
      ))}
    </div>
  );
}

export function TextLink({
  to,
  children,
}: {
  to: string;
  children: ReactNode;
}) {
  return (
    <Link
      to={to}
      className="group inline-flex items-center gap-2 text-sm font-bold text-orange-600"
    >
      {children}
      <ArrowRightIcon className="h-4 w-4 transition-transform group-hover:translate-x-1" />
    </Link>
  );
}
