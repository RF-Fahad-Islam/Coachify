import { useLayoutEffect, useRef } from "react";
import { gsap, prefersReduced, revealUp } from "../lib/anim";
import { CtaButton, useCta } from "./shared";
import { WifiOffIcon, MonitorIcon, ClockIcon } from "./icons";

export function FinalCta() {
  const ref = useRef<HTMLElement>(null);
  const openCta = useCta();

  useLayoutEffect(() => {
    const root = ref.current;
    if (!root) return;
    const ctx = gsap.context(() => {
      revealUp(root, "[data-reveal]", { stagger: 0.1, y: 30 });
      if (prefersReduced()) return;
      gsap.to(root.querySelector("[data-cta-ring]"), {
        rotate: 180,
        ease: "none",
        force3D: true,
        scrollTrigger: {
          trigger: root,
          start: "top bottom",
          end: "bottom top",
          scrub: 1.2,
        },
      });
    }, ref);
    return () => ctx.revert();
  }, []);

  return (
    <section
      id="contact"
      ref={ref}
      className="scroll-mt-24 px-4 pb-24 pt-4 sm:px-6 sm:pb-32 lg:px-8"
    >
      <div className="relative mx-auto max-w-7xl overflow-hidden rounded-[36px] bg-gradient-to-br from-orange-600 via-orange-500 to-amber-500 px-6 py-20 text-center shadow-lift sm:px-12 sm:py-28">
        {/* layered decor */}
        <div className="absolute inset-0 bg-grid-dark opacity-30" />
        <div
          data-cta-glow
          className="absolute -top-24 left-1/2 h-72 w-[560px] -translate-x-1/2 rounded-full bg-white/15 blur-3xl"
        />
        <div
          data-cta-ring
          className="pointer-events-none absolute -right-40 -top-40 h-[420px] w-[420px] rounded-full border-2 border-dashed border-white/20"
        />
        <div className="pointer-events-none absolute -bottom-24 -left-24 h-72 w-72 rounded-full border border-white/15" />
        <div className="pointer-events-none absolute bottom-10 right-14 h-4 w-4 rounded-full bg-white/30" />
        <div className="pointer-events-none absolute left-16 top-14 h-3 w-3 rounded-full bg-white/25" />

        <div className="relative">
          <span
            data-reveal
            className="inline-flex items-center gap-2 rounded-full border border-white/30 bg-white/10 px-4 py-2 text-[11px] font-bold uppercase tracking-[0.2em] text-white"
          >
            <svg viewBox="0 0 16 16" className="h-3 w-3 fill-white/80">
              <path d="M8 0c.6 4.2 3.8 7.4 8 8-4.2.6-7.4 3.8-8 8-.6-4.2-3.8-7.4-8-8 4.2-.6 7.4-3.8 8-8Z" />
            </svg>
            Desktop · Offline-first · Windows 10 / 11
          </span>

          <h2
            data-reveal
            className="mx-auto mt-7 max-w-3xl font-display text-4xl font-extrabold leading-[1.05] tracking-tight text-white sm:text-5xl lg:text-[58px]"
          >
            Ready to run your center with more clarity and less chaos?
          </h2>

          <p
            data-reveal
            className="mx-auto mt-6 max-w-xl text-lg leading-relaxed text-orange-50/90"
          >
            Download the free demo today. Your first students can be inside
            before the next batch even opens.
          </p>

          <div
            data-reveal
            className="mt-10 flex flex-wrap items-center justify-center gap-3.5"
          >
            <CtaButton
              size="lg"
              variant="light"
              onClick={() => openCta("download")}
            >
              Download Demo
            </CtaButton>
            <CtaButton
              size="lg"
              variant="dark"
              onClick={() => openCta("request")}
            >
              Request Full Version
            </CtaButton>
            <CtaButton
              size="lg"
              variant="lightGhost"
              onClick={() => openCta("request", "Book a live demo")}
            >
              Book a Demo
            </CtaButton>
          </div>

          <div
            data-reveal
            className="mt-10 flex flex-wrap items-center justify-center gap-x-8 gap-y-3 text-[13px] font-semibold text-orange-100/90"
          >
            <span className="inline-flex items-center gap-2">
              <WifiOffIcon className="h-4 w-4" /> 100% offline capable
            </span>
            <span className="inline-flex items-center gap-2">
              <MonitorIcon className="h-4 w-4" /> Data stays on your PC
            </span>
            <span className="inline-flex items-center gap-2">
              <ClockIcon className="h-4 w-4" /> Live in 5 minutes
            </span>
          </div>
        </div>
      </div>
    </section>
  );
}
