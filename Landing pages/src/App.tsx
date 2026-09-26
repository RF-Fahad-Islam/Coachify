import { useEffect, useLayoutEffect, useRef } from "react";
import {
  BrowserRouter,
  Navigate,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import {
  gsap,
  initSmoothWheel,
  prefersReduced,
  ScrollTrigger,
  syncSmoothScroll,
} from "./lib/anim";
import { CtaProvider } from "./components/shared";
import { LandingLocaleProvider } from "./lib/i18n";
import { Nav } from "./components/Nav";
import { Footer } from "./components/Footer";
import { Hero } from "./components/Hero";
import { Problem } from "./components/Problem";
import { Solution } from "./components/Solution";
import { Benefits } from "./components/Benefits";
import { HowItWorks } from "./components/HowItWorks";
import { Screens } from "./components/Screens";
import { Offline } from "./components/Offline";
import { Trust } from "./components/Trust";
import { Pricing } from "./components/Pricing";
import { Faq } from "./components/Faq";
import { FinalCta } from "./components/FinalCta";
import {
  FeaturesPage,
  HowPage,
  ScreensPage,
  OfflinePage,
  PricingPage,
  WhyPage,
  FaqPage,
} from "./pages/ProductPages";
import { DownloadPage } from "./pages/DownloadPage";
import { ContactPage, HelpPage } from "./pages/SupportPages";
import { PortalPage, PrivacyPage } from "./pages/PortalAndPrivacyPages";

function HomePage() {
  return (
    <>
      <Hero />
      <Problem />
      <Solution />
      <Benefits />
      <HowItWorks />
      <Screens />
      <Offline />
      <Trust />
      <Pricing />
      <Faq />
      <FinalCta />
    </>
  );
}

export function SiteRoutes() {
  const location = useLocation();
  const pageRef = useRef<HTMLDivElement>(null);
  const maskRef = useRef<HTMLDivElement>(null);
  const prevPathRef = useRef(location.pathname);

  // Synchronously reset scroll position before child layout effects calculate ScrollTriggers
  if (
    prevPathRef.current !== location.pathname &&
    typeof window !== "undefined"
  ) {
    window.scrollTo({ top: 0, left: 0, behavior: "instant" as ScrollBehavior });
    syncSmoothScroll();
  }

  useLayoutEffect(() => {
    const pageEl = pageRef.current;
    const maskEl = maskRef.current;
    window.scrollTo({ top: 0, left: 0, behavior: "instant" as ScrollBehavior });
    syncSmoothScroll();
    document.title =
      location.pathname === "/"
        ? "Coachify — Coaching Center Management, Even Offline"
        : `Coachify — ${location.pathname
            .split("/")
            .filter(Boolean)
            .map((x) => x.replace(/-/g, " "))
            .join(" ")
            .replace(/\b\w/g, (c) => c.toUpperCase())}`;

    if (pageEl) {
      gsap.set(pageEl, { clearProps: "all" });
    }
    ScrollTrigger.sort();
    ScrollTrigger.refresh();

    // Skip route animation on initial mount (including StrictMode double-invocation on the same path)
    if (prevPathRef.current === location.pathname) {
      return;
    }
    prevPathRef.current = location.pathname;

    let tl: gsap.core.Timeline | null = null;
    if (!prefersReduced() && pageEl && maskEl) {
      tl = gsap.timeline({
        onComplete: () => {
          gsap.set(pageEl, { clearProps: "all" });
        },
      });
      tl.set(maskEl, {
        scaleX: 0,
        autoAlpha: 1,
        transformOrigin: "left",
        force3D: true,
      })
        .fromTo(
          pageEl,
          { autoAlpha: 0 },
          { autoAlpha: 1, duration: 0.34, ease: "expo.out", clearProps: "all" },
          0,
        )
        .to(maskEl, { scaleX: 1, duration: 0.24, ease: "power2.out" }, 0)
        .to(maskEl, { autoAlpha: 0, duration: 0.18, ease: "power1.out" }, 0.2);
    }
    return () => {
      tl?.kill();
      if (pageEl) {
        gsap.set(pageEl, { clearProps: "all" });
      }
    };
  }, [location.pathname]);

  return (
    <>
      <Nav />
      <div
        ref={maskRef}
        className="pointer-events-none fixed inset-x-0 top-0 z-[70] h-0.5 origin-left scale-x-0 bg-orange-500"
      />
      <main ref={pageRef}>
        <Routes location={location}>
          <Route path="/" element={<HomePage />} />
          <Route path="/features" element={<FeaturesPage />} />
          <Route path="/how-it-works" element={<HowPage />} />
          <Route path="/screens" element={<ScreensPage />} />
          <Route path="/offline-lan" element={<OfflinePage />} />
          <Route path="/pricing" element={<PricingPage />} />
          <Route path="/download" element={<DownloadPage />} />
          <Route
            path="/downloads"
            element={<Navigate to="/download" replace />}
          />
          <Route path="/portal" element={<PortalPage />} />
          <Route path="/privacy" element={<PrivacyPage />} />
          <Route path="/help" element={<HelpPage />} />
          <Route path="/why-coachify" element={<WhyPage />} />
          <Route
            path="/about"
            element={<Navigate to="/why-coachify" replace />}
          />
          <Route path="/contact" element={<ContactPage />} />
          <Route path="/faq" element={<FaqPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <Footer />
    </>
  );
}

export function App() {
  useEffect(() => {
    initSmoothWheel();
    const refresh = () => {
      ScrollTrigger.sort();
      ScrollTrigger.refresh();
    };
    const t = window.setTimeout(refresh, 350);
    window.addEventListener("load", refresh);
    return () => {
      window.clearTimeout(t);
      window.removeEventListener("load", refresh);
    };
  }, []);
  return (
    <BrowserRouter>
      <LandingLocaleProvider>
        <CtaProvider>
          <div className="noise-overlay" aria-hidden="true" />
          <SiteRoutes />
        </CtaProvider>
      </LandingLocaleProvider>
    </BrowserRouter>
  );
}
