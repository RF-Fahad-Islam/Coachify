import { useEffect, useRef, useState } from "react";
import { Link, NavLink, useLocation } from "react-router-dom";
import { cn } from "../utils/cn";
import { gsap, prefersReduced } from "../lib/anim";
import { CtaButton, useCta } from "./shared";
import { BrandLockup, MenuIcon, XIcon } from "./icons";
import { useLandingLocale, type LandingTranslationKey } from "../lib/i18n";

const LINKS: ReadonlyArray<readonly [LandingTranslationKey, string]> = [
  ["nav.features", "/features"],
  ["nav.howItWorks", "/how-it-works"],
  ["nav.screens", "/screens"],
  ["nav.pricing", "/pricing"],
  ["nav.download", "/download"],
  ["nav.help", "/help"],
];

const EXTRA_MOBILE_LINKS: ReadonlyArray<
  readonly [LandingTranslationKey, string]
> = [
  ["nav.offlineLan", "/offline-lan"],
  ["nav.whyCoachify", "/why-coachify"],
  ["nav.faq", "/faq"],
  ["nav.contact", "/contact"],
];

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);
  const cta = useCta();
  const { locale, toggleLocale, t } = useLandingLocale();
  const navRef = useRef<HTMLElement>(null);
  const scrolledRef = useRef(false);
  const location = useLocation();

  useEffect(() => setOpen(false), [location.pathname]);
  useEffect(() => {
    if (navRef.current && !prefersReduced()) {
      gsap.fromTo(
        navRef.current,
        { y: -48, autoAlpha: 0 },
        {
          y: 0,
          autoAlpha: 1,
          duration: 0.7,
          ease: "expo.out",
          clearProps: "transform",
        },
      );
    }
    const onScroll = () => {
      const next = window.scrollY > 24;
      if (next !== scrolledRef.current) {
        scrolledRef.current = next;
        setScrolled(next);
      }
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header ref={navRef} className="fixed inset-x-0 top-0 z-50">
      <div
        className={cn(
          "mx-auto flex max-w-7xl items-center gap-3 px-4 transition-[padding] duration-300 sm:px-6 lg:px-8",
          scrolled ? "py-2.5" : "py-4",
        )}
      >
        <div
          className={cn(
            "pointer-events-none absolute inset-0 -z-10 backdrop-blur-md transition-[background-color,border-color] duration-300",
            scrolled
              ? "border-b border-stone-200/70 bg-cream/90"
              : "border-b border-transparent bg-cream/45",
          )}
        />
        <div className="flex items-center gap-2.5">
          <Link
            to="/"
            className="flex items-center gap-2.5"
            aria-label="Coachify home"
          >
            <BrandLockup markClassName="h-8 w-8" textClassName="text-[19px]" />
          </Link>
          <button
            type="button"
            onClick={toggleLocale}
            aria-label={t("nav.langToggleLabel")}
            data-testid="landing-locale-toggle"
            className="hidden cursor-pointer rounded-full border border-stone-200 bg-white/70 px-2 py-0.5 text-[10px] font-semibold text-stone-500 transition hover:border-orange-300 hover:text-orange-600 xl:inline"
          >
            Desktop · {locale === "en" ? "EN / বাংলা" : "বাংলা / EN"}
          </button>
        </div>
        <nav className="ml-auto hidden items-center lg:flex">
          {LINKS.map(([labelKey, to]) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  "rounded-full px-3 py-2 text-[13px] font-medium transition",
                  isActive
                    ? "bg-orange-50 text-orange-700"
                    : "text-stone-600 hover:bg-stone-900/5 hover:text-ink",
                )
              }
            >
              {t(labelKey)}
            </NavLink>
          ))}
        </nav>
        <div className="ml-auto flex items-center gap-2 lg:ml-2">
          <CtaButton
            size="sm"
            variant="ghost"
            className="hidden sm:inline-flex"
            onClick={() => cta("request")}
          >
            {t("nav.getCoachify")}
          </CtaButton>
          <Link
            to="/download"
            className="inline-flex items-center rounded-full bg-orange-500 px-4 py-2 text-[13px] font-semibold text-white shadow-lg shadow-orange-500/25 transition hover:bg-orange-600"
          >
            {t("nav.downloadDemo")}
          </Link>
          <button
            type="button"
            onClick={() => setOpen(!open)}
            className="cursor-pointer rounded-full border border-stone-200 bg-white p-2 lg:hidden"
            aria-label="Toggle menu"
          >
            {open ? (
              <XIcon className="h-4 w-4" />
            ) : (
              <MenuIcon className="h-4 w-4" />
            )}
          </button>
        </div>
      </div>
      {open && (
        <div className="absolute inset-x-3 top-full mt-2 rounded-2xl border border-stone-200 bg-white p-3 shadow-lift lg:hidden">
          <nav className="grid grid-cols-2 gap-1">
            {[...LINKS, ...EXTRA_MOBILE_LINKS].map(([labelKey, to]) => (
              <NavLink
                key={to}
                to={to}
                className="rounded-xl px-3 py-2.5 text-sm font-medium text-stone-700 hover:bg-orange-50"
              >
                {t(labelKey)}
              </NavLink>
            ))}
          </nav>
        </div>
      )}
    </header>
  );
}
