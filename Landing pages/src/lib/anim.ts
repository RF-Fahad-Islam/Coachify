import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

if (typeof window !== "undefined" && typeof window.matchMedia !== "function") {
  window.matchMedia = (query: string) =>
    ({
      matches: query.includes("reduce"),
      media: query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }) as MediaQueryList;
}

gsap.registerPlugin(ScrollTrigger);

// ---- global performance tuning ----
gsap.config({ force3D: true, nullTargetWarn: false });
gsap.ticker.lagSmoothing(500, 33);

ScrollTrigger.config({ ignoreMobileResize: true });

export { gsap, ScrollTrigger };

export const prefersReduced = () =>
  typeof window !== "undefined" &&
  typeof window.matchMedia === "function" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

let smoothCurrentY = 0;
let smoothTargetY = 0;
let smoothActive = false;
let internalScrollWrite = false;
let wheelInitialized = false;

/** Synchronizes the smooth wheel state with the current window.scrollY immediately. */
export function syncSmoothScroll() {
  if (typeof window === "undefined") return;
  smoothCurrentY = window.scrollY;
  smoothTargetY = window.scrollY;
  smoothActive = false;
}

/**
 * Initializes a lightweight, frame-rate-independent smooth wheel scroller
 * driven by `gsap.ticker` and locked to `ScrollTrigger.update`.
 * Eliminates the 100px discrete mouse-wheel step jitter on Windows while
 * yielding immediately to programmatic `window.scrollTo`, touchpads, or scrollbar drags.
 */
export function initSmoothWheel() {
  if (typeof window === "undefined" || wheelInitialized) return;
  wheelInitialized = true;
  syncSmoothScroll();

  const getMaxScroll = () =>
    Math.max(0, document.documentElement.scrollHeight - window.innerHeight);

  const onTick = (_time: number, deltaTime: number) => {
    if (!smoothActive) return;
    const dt = Math.min(Math.max(deltaTime || 16.67, 1), 64) / 1000;
    const factor = 1 - Math.exp(-15 * dt);
    const diff = smoothTargetY - smoothCurrentY;

    if (Math.abs(diff) < 0.35) {
      smoothCurrentY = smoothTargetY;
      smoothActive = false;
    } else {
      smoothCurrentY += diff * factor;
    }

    internalScrollWrite = true;
    window.scrollTo(0, smoothCurrentY);
    ScrollTrigger.update();
    internalScrollWrite = false;
  };

  gsap.ticker.add(onTick);

  window.addEventListener(
    "scroll",
    () => {
      if (!internalScrollWrite) {
        smoothCurrentY = window.scrollY;
        if (!smoothActive) {
          smoothTargetY = window.scrollY;
        }
      }
    },
    { passive: true },
  );

  window.addEventListener(
    "wheel",
    (e: WheelEvent) => {
      if (prefersReduced() || e.ctrlKey || e.metaKey || e.defaultPrevented)
        return;
      // Check if scrolling inside a scrollable modal or inner overflow container
      let node = e.target as HTMLElement | null;
      while (
        node &&
        node !== document.body &&
        node !== document.documentElement
      ) {
        const style = window.getComputedStyle(node);
        const overflowY = style.overflowY;
        if (
          (overflowY === "auto" || overflowY === "scroll") &&
          node.scrollHeight > node.clientHeight
        ) {
          return;
        }
        node = node.parentElement;
      }

      // Only smooth discrete mouse wheels or large wheel steps; let native high-frequency
      // precision trackpads (small pixel deltas < 12px) glide without double-damping if desired,
      // or unify all wheel events through the 120fps ticker.
      const multiplier =
        e.deltaMode === 1
          ? 36
          : e.deltaMode === 2
            ? window.innerHeight * 0.85
            : 1;
      const delta = e.deltaY * multiplier;
      if (Math.abs(delta) < 0.5) return;

      e.preventDefault();
      if (!smoothActive) {
        smoothCurrentY = window.scrollY;
        smoothTargetY = window.scrollY;
      }
      smoothTargetY = Math.max(
        0,
        Math.min(getMaxScroll(), smoothTargetY + delta),
      );
      smoothActive = true;
    },
    { passive: false },
  );

  window.addEventListener(
    "keydown",
    () => {
      smoothActive = false;
    },
    { passive: true },
  );
  window.addEventListener(
    "mousedown",
    () => {
      smoothActive = false;
    },
    { passive: true },
  );
}

/**
 * Scroll-triggered staggered reveal for a set of elements inside a scope.
 * Safely cleans up previous tweens on re-mounts (<StrictMode>), disables
 * competing CSS transitions via `[data-gsap-active="true"]`, and restores
 * CSS hover transitions on the frame after `transform` is cleared so there
 * is zero end-of-tween snap or subpixel jitter.
 */
export function revealUp(
  scope: HTMLElement | null,
  targets: string,
  opts: {
    y?: number;
    x?: number;
    dur?: number;
    stagger?: number;
    start?: string;
    ease?: string;
    delay?: number;
  } = {},
) {
  if (!scope) return;
  const els = Array.from(scope.querySelectorAll<HTMLElement>(targets));
  if (!els.length) return;

  gsap.killTweensOf(els);

  if (prefersReduced()) {
    els.forEach((el) => delete el.dataset.gsapActive);
    gsap.set(els, {
      autoAlpha: 1,
      y: 0,
      x: 0,
      clearProps: "transform,transition",
    });
    return;
  }

  const {
    y = 28,
    x = 0,
    dur = 1.18,
    stagger = 0.13,
    start = "top 88%",
    ease = "power2.out",
    delay = 0,
  } = opts;

  const groups = new Map<HTMLElement, HTMLElement[]>();
  for (const el of els) {
    const groupTrigger =
      el.closest<HTMLElement>(
        "[data-reveal-group], article, footer, .sol-checklist",
      ) ??
      el.parentElement ??
      scope;
    const list = groups.get(groupTrigger) ?? [];
    list.push(el);
    groups.set(groupTrigger, list);
  }

  groups.forEach((groupEls, triggerEl) => {
    groupEls.forEach((el) => {
      el.dataset.gsapActive = "true";
    });
    // Pre-hide every element in the group synchronously before paint so staggered
    // items (index 1..N) are never visible at opacity 1 before their stagger delay.
    gsap.set(groupEls, { y, x, autoAlpha: 0, force3D: true });

    gsap.to(groupEls, {
      y: 0,
      x: 0,
      autoAlpha: 1,
      duration: dur,
      stagger,
      ease,
      delay,
      overwrite: "auto",
      force3D: true,
      onComplete: () => {
        gsap.set(groupEls, { clearProps: "transform" });
        if (
          typeof window !== "undefined" &&
          typeof window.requestAnimationFrame === "function"
        ) {
          window.requestAnimationFrame(() => {
            groupEls.forEach((el) => {
              delete el.dataset.gsapActive;
            });
          });
        } else {
          groupEls.forEach((el) => {
            delete el.dataset.gsapActive;
          });
        }
      },
      scrollTrigger: {
        trigger: triggerEl,
        start,
        once: true,
      },
    });
  });
}

/** Animate a counter element from 0 to `to` when it enters the viewport. */
export function countUp(
  el: HTMLElement,
  to: number,
  opts: { dur?: number; decimals?: number; suffix?: string } = {},
) {
  const { dur = 1.4, decimals = 0, suffix = "" } = opts;
  const format = (v: number) =>
    v.toLocaleString("en-US", {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    }) + suffix;
  if (prefersReduced()) {
    el.textContent = format(to);
    return;
  }
  const state = { v: 0 };
  gsap.to(state, {
    v: to,
    duration: dur,
    ease: "power2.out",
    scrollTrigger: {
      trigger: el,
      start: "top 90%",
      once: true,
      invalidateOnRefresh: true,
    },
    onUpdate: () => {
      el.textContent = format(state.v);
    },
  });
}
