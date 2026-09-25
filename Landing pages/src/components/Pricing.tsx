import { useLayoutEffect, useRef } from "react";
import { gsap, revealUp } from "../lib/anim";
import { cn } from "../utils/cn";
import { SectionTag, CtaButton, useCta } from "./shared";
import { CheckIcon, SparkIcon } from "./icons";
import { PRICING_PLANS } from "../data/appData";
import { useLandingLocale } from "../lib/i18n";

export function Pricing() {
  const ref = useRef<HTMLElement>(null);
  const openCta = useCta();
  const { t } = useLandingLocale();

  useLayoutEffect(() => {
    if (!ref.current) return;
    const ctx = gsap.context(() => {
      revealUp(ref.current, "[data-reveal]", { stagger: 0.1, y: 32 });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section
      id="pricing"
      ref={ref}
      className="relative scroll-mt-24 bg-white py-24 sm:py-32"
    >
      <div className="absolute inset-0 -z-10 bg-grid-warm [mask-image:radial-gradient(ellipse_60%_60%_at_50%_0%,black,transparent)] opacity-50" />
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-3xl text-center">
          <span data-reveal>
            <SectionTag>{t("pricing.tag")}</SectionTag>
          </span>
          <h2
            data-reveal
            className="mt-5 font-display text-4xl font-extrabold leading-[1.06] tracking-tight text-ink sm:text-5xl lg:text-[54px]"
          >
            {t("pricing.titlePrefix")}{" "}
            <span className="text-orange-500">
              {t("pricing.titleHighlight")}
            </span>
          </h2>
          <p data-reveal className="mt-5 text-lg text-stone-500">
            {t("pricing.subtitle")}
          </p>
        </div>

        <div className="mx-auto mt-16 grid max-w-6xl items-stretch gap-5 lg:grid-cols-3">
          {PRICING_PLANS.map((plan) => (
            <div
              key={plan.name}
              data-reveal
              className={cn(
                "relative flex flex-col rounded-3xl p-8 transition-all duration-500 hover:-translate-y-2",
                plan.featured
                  ? "bg-ink text-white shadow-lift ring-2 ring-orange-500 lg:-translate-y-3 lg:hover:-translate-y-4"
                  : "border border-stone-200/80 bg-cream/60 shadow-card hover:shadow-lift",
              )}
            >
              {plan.featured && (
                <>
                  <div className="absolute inset-x-0 top-0 h-1.5 rounded-t-3xl bg-gradient-to-r from-orange-400 via-orange-500 to-amber-400" />
                  <span className="absolute -top-4 left-1/2 flex -translate-x-1/2 items-center gap-1.5 whitespace-nowrap rounded-full bg-orange-500 px-4 py-1.5 text-[11px] font-bold uppercase tracking-[0.14em] text-white shadow-lg shadow-orange-500/40">
                    <SparkIcon className="h-3 w-3" /> {t("pricing.mostPopular")}
                  </span>
                </>
              )}
              <h3
                className={cn(
                  "font-display text-lg font-bold",
                  plan.featured ? "text-white" : "text-ink",
                )}
              >
                {plan.name}
              </h3>
              <div className="mt-4 flex items-baseline gap-2">
                <span
                  className={cn(
                    "font-display text-5xl font-extrabold tracking-tight",
                    plan.featured ? "text-white" : "text-ink",
                  )}
                >
                  {plan.price}
                </span>
              </div>
              <p
                className={cn(
                  "mt-1 text-[13px] font-semibold",
                  plan.featured ? "text-orange-300" : "text-orange-600",
                )}
              >
                {plan.period}
              </p>
              <p
                className={cn(
                  "mt-4 text-sm leading-relaxed",
                  plan.featured ? "text-stone-400" : "text-stone-500",
                )}
              >
                {plan.desc}
              </p>
              <ul className="mt-6 flex-1 space-y-3">
                {plan.features.map((feature) => (
                  <li
                    key={feature}
                    className={cn(
                      "flex items-start gap-2.5 text-sm",
                      plan.featured ? "text-stone-300" : "text-stone-600",
                    )}
                  >
                    <span
                      className={cn(
                        "mt-0.5 flex h-4.5 w-4.5 shrink-0 items-center justify-center rounded-full",
                        plan.featured
                          ? "bg-orange-500/20 text-orange-400"
                          : "bg-orange-100 text-orange-600",
                      )}
                    >
                      <CheckIcon className="h-2.5 w-2.5" />
                    </span>
                    {feature}
                  </li>
                ))}
              </ul>
              <div className="mt-8">
                <CtaButton
                  variant={
                    plan.featured
                      ? "primary"
                      : plan.name === "Demo"
                        ? "dark"
                        : "ghost"
                  }
                  className="w-full"
                  onClick={() => openCta(plan.kind, plan.interest)}
                >
                  {plan.cta}
                </CtaButton>
              </div>
            </div>
          ))}
        </div>

        <p
          data-reveal
          className="mt-10 text-center text-sm font-medium text-stone-400"
        >
          {t("pricing.footerNote")}
        </p>
      </div>
    </section>
  );
}
