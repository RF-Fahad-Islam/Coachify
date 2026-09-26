import { Link } from "react-router-dom";
import {
  BrandLockup,
  MailIcon,
  PhoneIcon,
  MapPinIcon,
  WifiOffIcon,
  MonitorIcon,
} from "./icons";
import { useCta } from "./shared";

const COLS: { title: string; links: { label: string; href: string }[] }[] = [
  {
    title: "Product",
    links: [
      { label: "Features", href: "/features" },
      { label: "How It Works", href: "/how-it-works" },
      { label: "Screens", href: "/screens" },
      { label: "Pricing", href: "/pricing" },
    ],
  },
  {
    title: "Resources",
    links: [
      { label: "FAQ", href: "/faq" },
      { label: "Download Demo", href: "/download" },
      { label: "Offline & LAN", href: "/offline-lan" },
      { label: "Documentation", href: "/help" },
      { label: "Customer Portal", href: "/portal" },
    ],
  },
];

export function Footer() {
  const openCta = useCta();

  return (
    <footer className="bg-ink text-stone-400">
      <div className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:px-8">
        <div className="grid gap-12 lg:grid-cols-[1.5fr_1fr_1fr_1.3fr]">
          <div>
            <Link to="/" className="flex items-center gap-2.5">
              <BrandLockup
                markClassName="h-9 w-9"
                textClassName="text-xl"
                dark
              />
            </Link>
            <p className="mt-4 max-w-xs text-sm leading-relaxed text-stone-500">
              The offline-first desktop system for coaching centers, academies
              and institutes. Admissions to results — on your own machines, on
              your terms.
            </p>
            <div className="mt-5 flex flex-wrap gap-2">
              <span className="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-[11px] font-semibold text-stone-300">
                <WifiOffIcon className="h-3.5 w-3.5 text-orange-400" /> Works
                offline
              </span>
              <span className="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-3 py-1.5 text-[11px] font-semibold text-stone-300">
                <MonitorIcon className="h-3.5 w-3.5 text-orange-400" /> Windows
                10 / 11
              </span>
            </div>
          </div>

          {COLS.map((c) => (
            <div key={c.title}>
              <p className="text-[11px] font-bold uppercase tracking-[0.2em] text-stone-500">
                {c.title}
              </p>
              <ul className="mt-4 space-y-2.5">
                {c.links.map((l) => (
                  <li key={l.label}>
                    <Link
                      to={l.href}
                      className="text-sm font-medium text-stone-400 transition-colors hover:text-orange-400"
                    >
                      {l.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}

          <div>
            <p className="text-[11px] font-bold uppercase tracking-[0.2em] text-stone-500">
              Contact & Support
            </p>
            <ul className="mt-4 space-y-2.5 text-sm font-medium">
              <li className="flex items-center gap-2.5">
                <MailIcon className="h-4 w-4 text-orange-400" />{" "}
                sales@coachify.app
              </li>
              <li className="flex items-center gap-2.5">
                <PhoneIcon className="h-4 w-4 text-orange-400" /> +91 98765
                04321
              </li>
              <li className="flex items-center gap-2.5">
                <MapPinIcon className="h-4 w-4 text-orange-400" /> Supporting
                centers across India
              </li>
            </ul>
            <button
              onClick={() => openCta("request")}
              className="mt-5 cursor-pointer rounded-full border border-white/15 bg-white/5 px-4.5 py-2.5 text-[13px] font-semibold text-stone-200 transition-all duration-300 hover:border-orange-400/50 hover:bg-orange-500/10 hover:text-orange-300"
            >
              Talk to our team →
            </button>
          </div>
        </div>

        <div className="mt-14 flex flex-col items-center justify-between gap-4 border-t border-white/10 pt-7 text-[13px] font-medium text-stone-500 sm:flex-row">
          <p>
            © 2026 Coachify. Built for coaching centers, academies &amp;
            institutes.
          </p>
          <div className="flex items-center gap-5">
            <Link
              to="/privacy"
              className="cursor-pointer transition-colors hover:text-stone-300"
            >
              Privacy
            </Link>
            <span className="cursor-pointer transition-colors hover:text-stone-300">
              Terms
            </span>
            <span className="cursor-pointer transition-colors hover:text-stone-300">
              License
            </span>
            <span className="tnum rounded-full border border-white/10 px-3 py-1 text-[11px] font-semibold text-stone-400">
              Windows desktop · x64
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
}
