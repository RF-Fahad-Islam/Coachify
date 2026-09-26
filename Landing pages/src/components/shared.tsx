import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { cn } from "../utils/cn";
import { gsap, prefersReduced, countUp } from "../lib/anim";
import {
  CheckIcon,
  DownloadIcon,
  XIcon,
  BrandLockup,
  ArrowRightIcon,
} from "./icons";
import { RELEASE_INFO } from "../data/appData";
import {
  submitLeadRequest,
  triggerInstallerDownload,
  validateContactString,
} from "../lib/leads";

/* ------------------------------------------------------------------ */
/* CTA system — every button on the page opens a working flow          */
/* ------------------------------------------------------------------ */

export type ModalKind = "download" | "request" | "checkout";
type CtaFn = (kind: ModalKind, interest?: string) => void;
const CtaContext = createContext<CtaFn>(() => {});
export const useCta = () => useContext(CtaContext);

export function CtaProvider({ children }: { children: ReactNode }) {
  const [modal, setModal] = useState<{
    kind: ModalKind;
    interest?: string;
  } | null>(null);
  const open = useCallback<CtaFn>(
    (kind, interest) => setModal({ kind, interest }),
    [],
  );
  const close = useCallback(() => setModal(null), []);
  return (
    <CtaContext.Provider value={open}>
      {children}
      <ModalHost modal={modal} onClose={close} />
    </CtaContext.Provider>
  );
}

export function CtaButton({
  variant = "primary",
  size = "md",
  type = "button",
  className,
  children,
  onClick,
  icon,
}: {
  variant?: "primary" | "dark" | "light" | "ghost" | "lightGhost";
  size?: "sm" | "md" | "lg";
  type?: "button" | "submit" | "reset";
  className?: string;
  children: ReactNode;
  onClick?: () => void;
  icon?: ReactNode;
}) {
  return (
    <button
      type={type}
      onClick={onClick}
      className={cn(
        "group inline-flex items-center justify-center gap-2 rounded-full font-semibold transition-all duration-300 select-none cursor-pointer",
        size === "sm" && "px-4 py-2 text-[13px]",
        size === "md" && "px-5 py-2.5 text-sm",
        size === "lg" && "px-7 py-3.5 text-[15px]",
        variant === "primary" &&
          "bg-orange-500 text-white shadow-lg shadow-orange-500/30 hover:bg-orange-600 hover:shadow-xl hover:shadow-orange-500/35 hover:-translate-y-0.5 active:translate-y-0",
        variant === "dark" &&
          "bg-ink text-white shadow-md shadow-ink/20 hover:bg-ink-soft hover:-translate-y-0.5 active:translate-y-0",
        variant === "light" &&
          "bg-white text-ink shadow-lg shadow-ink/10 hover:bg-orange-50 hover:-translate-y-0.5 active:translate-y-0",
        variant === "ghost" &&
          "border border-stone-300 bg-white/70 text-stone-800 hover:border-stone-400 hover:bg-white hover:-translate-y-0.5 active:translate-y-0",
        variant === "lightGhost" &&
          "border border-white/30 bg-white/5 text-white hover:bg-white/10 hover:-translate-y-0.5 active:translate-y-0",
        className,
      )}
    >
      {children}
      {icon}
    </button>
  );
}

export function SectionTag({
  children,
  dark = false,
}: {
  children: ReactNode;
  dark?: boolean;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border px-3.5 py-1.5 text-[11px] font-semibold uppercase tracking-[0.18em]",
        dark
          ? "border-white/15 bg-white/5 text-orange-300"
          : "border-orange-200/80 bg-orange-50 text-orange-700",
      )}
    >
      <span
        className={cn(
          "h-1.5 w-1.5 rounded-full pulse-dot",
          dark ? "bg-orange-400" : "bg-orange-500",
        )}
      />
      {children}
    </span>
  );
}

export function Counter({
  to,
  suffix = "",
  prefix = "",
  decimals = 0,
  className,
}: {
  to: number;
  suffix?: string;
  prefix?: string;
  decimals?: number;
  className?: string;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    if (ref.current) countUp(ref.current, to, { suffix, decimals });
  }, [to, suffix, decimals]);
  return (
    <span className={cn("tnum", className)}>
      {prefix}
      <span ref={ref}>0{suffix}</span>
    </span>
  );
}

/* ------------------------------------------------------------------ */
/* Modal host                                                          */
/* ------------------------------------------------------------------ */

function ModalHost({
  modal,
  onClose,
}: {
  modal: { kind: ModalKind; interest?: string } | null;
  onClose: () => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!modal) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    document.body.style.overflow = "hidden";
    if (panelRef.current && !prefersReduced()) {
      gsap.fromTo(
        panelRef.current,
        { autoAlpha: 0, y: 26, scale: 0.97 },
        { autoAlpha: 1, y: 0, scale: 1, duration: 0.45, ease: "power3.out" },
      );
    }
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = "";
    };
  }, [modal, onClose]);

  if (!modal) return null;

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center p-4"
      role="dialog"
      aria-modal="true"
    >
      <div
        className="absolute inset-0 bg-ink/55 backdrop-blur-[3px]"
        onClick={onClose}
      />
      <div
        ref={panelRef}
        className={cn(
          "relative w-full overflow-hidden rounded-3xl bg-white shadow-2xl max-h-[92vh] overflow-y-auto",
          modal.kind === "checkout" ? "max-w-xl" : "max-w-md",
        )}
      >
        <div className="pointer-events-none absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-orange-400 via-orange-500 to-amber-400" />
        <button
          onClick={onClose}
          aria-label="Close"
          className="absolute right-4 top-4 z-10 rounded-full p-2 text-stone-400 transition hover:bg-stone-100 hover:text-stone-700 cursor-pointer"
        >
          <XIcon className="h-4 w-4" />
        </button>
        {modal.kind === "download" ? (
          <DownloadPanel />
        ) : modal.kind === "checkout" ? (
          <CheckoutPanel interest={modal.interest} />
        ) : (
          <RequestPanel interest={modal.interest} />
        )}
      </div>
    </div>
  );
}

function DownloadPanel() {
  const [phase, setPhase] = useState<"idle" | "downloading" | "done">("idle");
  const barRef = useRef<HTMLDivElement>(null);
  const [pct, setPct] = useState(0);

  useEffect(() => {
    if (phase !== "downloading" || !barRef.current) return;
    if (prefersReduced()) {
      setPct(100);
      const t = setTimeout(() => setPhase("done"), 300);
      return () => clearTimeout(t);
    }
    gsap.to(barRef.current, {
      width: "100%",
      duration: 2.1,
      ease: "power2.inOut",
      onUpdate: function () {
        setPct(Math.round((this.progress as number) * 100));
      },
      onComplete: () => setPhase("done"),
    });
  }, [phase]);

  return (
    <div className="p-7">
      <div className="flex items-center gap-3">
        <BrandLockup markClassName="h-11 w-11" textClassName="text-lg" />
        <div>
          <p className="font-display text-sm font-semibold text-stone-500">
            Demo installer ({RELEASE_INFO.version})
          </p>
          <p
            className="text-xs font-medium text-stone-500"
            data-testid="installer-sha256"
            title={`SHA-256: ${RELEASE_INFO.sha256}`}
          >
            {RELEASE_INFO.osRequirements} · {RELEASE_INFO.sizeLabel} · SHA-256:{" "}
            {RELEASE_INFO.sha256.slice(0, 12)}…
          </p>
        </div>
      </div>

      <ul className="mt-5 space-y-2.5 text-sm text-stone-600">
        {[
          "Every module enabled for 30 days",
          "Up to 100 students and 10 batches",
          "Runs on a single PC — no internet needed",
          "No card, no trial limits after the date",
        ].map((t) => (
          <li key={t} className="flex items-start gap-2.5">
            <span className="mt-0.5 flex h-4.5 w-4.5 shrink-0 items-center justify-center rounded-full bg-orange-100 text-orange-600">
              <CheckIcon className="h-2.5 w-2.5" />
            </span>
            {t}
          </li>
        ))}
      </ul>

      <div className="mt-6">
        {phase === "done" ? (
          <div className="rounded-2xl border border-emerald-200 bg-emerald-50 p-4">
            <div className="flex items-center gap-3">
              <span className="flex h-9 w-9 items-center justify-center rounded-full bg-emerald-500 text-white">
                <CheckIcon className="h-4 w-4" />
              </span>
              <div className="min-w-0">
                <p className="truncate font-mono text-sm font-semibold text-emerald-800">
                  {RELEASE_INFO.fileName}
                </p>
                <p className="text-xs text-emerald-700">
                  Download prepared — run the installer to get started.
                </p>
              </div>
            </div>
            <p className="mt-3 text-xs text-emerald-700">
              Prefer a guided walkthrough?{" "}
              <button
                className="font-semibold underline underline-offset-2 cursor-pointer"
                onClick={() => setPhase("idle")}
              >
                We&rsquo;ll show you how
              </button>
            </p>
          </div>
        ) : (
          <>
            <button
              onClick={() => {
                setPct(0);
                setPhase("downloading");
                triggerInstallerDownload();
              }}
              disabled={phase === "downloading"}
              className="flex w-full items-center justify-center gap-2 rounded-full bg-orange-500 px-6 py-3.5 text-sm font-semibold text-white shadow-lg shadow-orange-500/30 transition hover:bg-orange-600 disabled:cursor-wait disabled:opacity-90 cursor-pointer"
            >
              <DownloadIcon className="h-4.5 w-4.5" />
              {phase === "downloading"
                ? `Downloading… ${pct}%`
                : "Download demo installer"}
            </button>
            {phase === "downloading" && (
              <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-stone-100">
                <div
                  ref={barRef}
                  className="h-full w-0 rounded-full bg-gradient-to-r from-orange-400 to-orange-600"
                />
              </div>
            )}
            <p className="mt-3 text-center text-xs text-stone-400">
              Straight to your machine. Nothing is uploaded, ever.
            </p>
          </>
        )}
      </div>
    </div>
  );
}

function RequestPanel({ interest }: { interest?: string }) {
  const [sent, setSent] = useState(false);
  const [name, setName] = useState("");
  const [institution, setInstitution] = useState("");
  const [contact, setContact] = useState("");
  const [selectedInterest, setSelectedInterest] = useState(
    interest ?? "Full Version",
  );
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="p-7">
      {sent ? (
        <div className="py-4 text-center">
          <span className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-orange-500 text-white shadow-lg shadow-orange-500/30">
            <CheckIcon className="h-6 w-6" />
          </span>
          <h3 className="mt-4 font-display text-xl font-bold text-ink">
            Request received
          </h3>
          <p className="mx-auto mt-2 max-w-xs text-sm leading-relaxed text-stone-500">
            Our team will reach out within one business day with your license
            details or a call slot.
          </p>
          <button
            onClick={() => {
              setSent(false);
              setName("");
              setInstitution("");
              setContact("");
            }}
            className="mt-5 text-xs font-semibold text-orange-600 hover:text-orange-700 cursor-pointer"
          >
            Send another request
          </button>
        </div>
      ) : (
        <form
          onSubmit={async (e) => {
            e.preventDefault();
            const validationError = validateContactString(contact);
            if (validationError) {
              setError(validationError);
              return;
            }
            setError(null);
            await submitLeadRequest({
              name,
              institution,
              contact,
              interest: selectedInterest,
              source: "modal",
            });
            setSent(true);
          }}
        >
          <h3 className="font-display text-xl font-bold text-ink">
            Let&rsquo;s set up your center
          </h3>
          <p className="mt-1 text-sm text-stone-500">
            Tell us a little about your institute — we&rsquo;ll take it from
            there.
          </p>
          <div className="mt-5 space-y-3.5">
            <Field
              label="Your name"
              placeholder="e.g. Tanvir Rahman"
              value={name}
              onChange={setName}
              required
            />
            <Field
              label="Coaching center / institute"
              placeholder="e.g. Atomic Coaching Center"
              value={institution}
              onChange={setInstitution}
              required
            />
            <Field
              label="Email or WhatsApp"
              placeholder="01711-234567 or you@center.com"
              type="text"
              value={contact}
              onChange={(v) => {
                setContact(v);
                if (error) setError(null);
              }}
              required
            />
            {error && (
              <p role="alert" className="text-xs font-semibold text-rose-600">
                {error}
              </p>
            )}
            <div>
              <label className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-stone-500">
                I&rsquo;m interested in
              </label>
              <select
                value={selectedInterest}
                onChange={(e) => setSelectedInterest(e.target.value)}
                className="w-full rounded-xl border border-stone-200 bg-white px-3.5 py-2.5 text-sm text-stone-800 outline-none transition focus:border-orange-400 focus:ring-2 focus:ring-orange-100"
              >
                <option value="Full Version">Full Version (৳14,900)</option>
                <option value="Pro Multi-PC (LAN)">
                  Pro Multi-PC (LAN · ৳29,900)
                </option>
                <option value="Book a live demo">Book a live demo</option>
                <option value="Something else">Something else</option>
              </select>
            </div>
          </div>
          <button
            type="submit"
            className="mt-5 flex w-full items-center justify-center gap-2 rounded-full bg-ink px-6 py-3.5 text-sm font-semibold text-white shadow-lg shadow-ink/20 transition hover:bg-ink-soft cursor-pointer"
          >
            Send request
            <ArrowRightIcon className="h-4 w-4" />
          </button>
          <p className="mt-3 text-center text-xs text-stone-400">
            No spam. One real person replies.
          </p>
        </form>
      )}
    </div>
  );
}

function Field({
  label,
  placeholder,
  type = "text",
  value,
  onChange,
  required,
}: {
  label: string;
  placeholder: string;
  type?: string;
  value: string;
  onChange: (value: string) => void;
  required?: boolean;
}) {
  return (
    <div>
      <label className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-stone-500">
        {label}
      </label>
      <input
        type={type}
        required={required}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-xl border border-stone-200 bg-white px-3.5 py-2.5 text-sm text-stone-800 outline-none transition placeholder:text-stone-300 focus:border-orange-400 focus:ring-2 focus:ring-orange-100"
      />
    </div>
  );
}

/* ------------------------------------------------------------------ */
/* Checkout modal panel (Feature 67, Spec 0043a)                       */
/* ------------------------------------------------------------------ */

export type CheckoutTierKey =
  "3_months" | "6_months" | "12_months" | "lifetime";

interface CheckoutTierMeta {
  key: CheckoutTierKey;
  label: string;
  monthlyRateBdt: number;
  months: number;
  totalBdt: number;
  badge: string;
}

const CHECKOUT_TIERS: Record<CheckoutTierKey, CheckoutTierMeta> = {
  "3_months": {
    key: "3_months",
    label: "3 Months Plan",
    monthlyRateBdt: 1500,
    months: 3,
    totalBdt: 4500,
    badge: "Quarterly",
  },
  "6_months": {
    key: "6_months",
    label: "6 Months Plan",
    monthlyRateBdt: 1200,
    months: 6,
    totalBdt: 7200,
    badge: "Popular",
  },
  "12_months": {
    key: "12_months",
    label: "12 Months Annual",
    monthlyRateBdt: 1000,
    months: 12,
    totalBdt: 12000,
    badge: "Best Value · Save 33%",
  },
  lifetime: {
    key: "lifetime",
    label: "Full Lifetime License",
    monthlyRateBdt: 0,
    months: 0,
    totalBdt: 50000,
    badge: "Zero Subscriptions Forever",
  },
};

export function CheckoutPanel({ interest }: { interest?: string }) {
  const initialTierKey = ((): CheckoutTierKey => {
    if (interest?.includes("3_months") || interest?.includes("3 Months"))
      return "3_months";
    if (interest?.includes("6_months") || interest?.includes("6 Months"))
      return "6_months";
    if (interest?.includes("lifetime") || interest?.includes("Lifetime"))
      return "lifetime";
    return "12_months";
  })();

  const [tierKey, setTierKey] = useState<CheckoutTierKey>(initialTierKey);
  const [orgName, setOrgName] = useState("");
  const [ownerName, setOwnerName] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [targetRequestCode, setTargetRequestCode] = useState("");
  const [method, setMethod] = useState<"bkash" | "nagad" | "bank_transfer">(
    "bkash",
  );
  const [bankRef, setBankRef] = useState("");

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [orderResult, setOrderResult] = useState<{
    orderNumber: string;
    invoiceNumber: string;
    pairingCode: string;
    licenseBlob: string;
    method: string;
    isBankReview: boolean;
  } | null>(null);

  const tier = CHECKOUT_TIERS[tierKey];

  const handleDownloadBlob = (filename: string, content: string) => {
    const blob = new Blob([content], {
      type: "application/json;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!orgName.trim() || !ownerName.trim() || !phone.trim()) {
      setError(
        "Please fill in institution name, owner name, and phone number.",
      );
      return;
    }
    const phoneErr = validateContactString(phone);
    if (phoneErr) {
      setError(phoneErr);
      return;
    }

    setLoading(true);
    setError(null);

    // Simulate online gateway checkout transaction and license generation
    if (
      typeof window !== "undefined" &&
      !(window as { __VITEST__?: boolean }).__VITEST__
    ) {
      await new Promise((resolve) => setTimeout(resolve, 600));
    }

    const randSeq = Math.floor(10000 + Math.random() * 90000);
    const orderNumber = `CHK-2026-${randSeq}`;
    const invoiceNumber = `INV-2026-${randSeq}`;
    const pairingCode = Math.floor(100000 + Math.random() * 900000).toString();
    const paidThrough = tier.key === "lifetime" ? "9999-12-31" : "2027-09-25";

    const mockLicenseBlob = JSON.stringify(
      {
        license_id: `LIC-2026-ONLN-${randSeq}`,
        customer: {
          org_name: orgName.trim(),
          owner_name: ownerName.trim(),
          phone: phone.trim(),
        },
        plan: tier.key === "lifetime" ? "local_pro" : "local",
        billing_cycle: tier.key,
        paid_through: paidThrough,
        serial: 1,
        pairing_code: pairingCode,
        invoice_number: invoiceNumber,
        key_id: "k2026a",
        signature: `ed25519:simulated_${randSeq}`,
      },
      null,
      2,
    );

    setOrderResult({
      orderNumber,
      invoiceNumber,
      pairingCode,
      licenseBlob: mockLicenseBlob,
      method,
      isBankReview: method === "bank_transfer",
    });
    setLoading(false);
  };

  if (orderResult) {
    return (
      <div className="p-6 sm:p-8" data-testid="checkout-success-view">
        <div className="flex items-center gap-3">
          <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-emerald-100 text-emerald-600">
            <CheckIcon className="h-6 w-6" />
          </div>
          <div>
            <h3 className="font-display text-lg font-bold text-ink">
              {orderResult.isBankReview
                ? "Bank Transfer Order Submitted"
                : "Payment Completed & License Issued"}
            </h3>
            <p className="text-xs text-stone-500 font-mono">
              Order: {orderResult.orderNumber} · Invoice:{" "}
              {orderResult.invoiceNumber}
            </p>
          </div>
        </div>

        {orderResult.isBankReview ? (
          <div className="mt-6 space-y-4">
            <div className="rounded-2xl border border-amber-200 bg-amber-50/70 p-4 text-xs text-amber-900 leading-relaxed">
              <strong>Order in Review:</strong> Your bank deposit reference{" "}
              <code className="font-bold font-mono bg-amber-100 px-1.5 py-0.5 rounded">
                {bankRef || "PENDING"}
              </code>{" "}
              has been recorded. As soon as our Dhaka finance team confirms
              settlement at Eastern Bank PLC, your signed license will unlock
              automatically in the portal and activate your office PC.
            </div>

            <div className="rounded-2xl border border-stone-200 bg-stone-50 p-4">
              <h4 className="text-xs font-bold uppercase tracking-wider text-stone-500">
                Deposit Account Details
              </h4>
              <div className="mt-2 space-y-1 text-xs text-stone-700">
                <p>
                  <strong>Bank:</strong> Eastern Bank PLC (EBL)
                </p>
                <p>
                  <strong>Account Name:</strong> Coachify Technologies Ltd
                </p>
                <p>
                  <strong>Account Number:</strong> 1041060389201
                </p>
                <p>
                  <strong>Branch:</strong> Mirpur-10 Branch, Dhaka (Routing:
                  095261728)
                </p>
                <p>
                  <strong>Deposit Reference:</strong> REF-
                  {orderResult.orderNumber}
                </p>
              </div>
            </div>
          </div>
        ) : (
          <div className="mt-6 space-y-4">
            {/* 6-Digit Desktop Pairing Code Card */}
            <div className="rounded-2xl border border-orange-200 bg-orange-50/80 p-5 text-center">
              <span className="text-[11px] font-bold uppercase tracking-wider text-orange-700">
                Your 6-Digit Desktop Pairing Code
              </span>
              <div className="mt-2 font-mono text-3xl font-extrabold tracking-widest text-ink">
                {orderResult.pairingCode}
              </div>
              <p className="mt-2 text-xs text-stone-600">
                Open Coachify on your office PC &rarr;{" "}
                <strong>Settings &rarr; License</strong> &rarr; Enter this
                pairing code to activate online immediately.
              </p>
            </div>

            {/* Offline License File Download */}
            <div className="rounded-2xl border border-stone-200 bg-stone-50 p-4 flex items-center justify-between gap-3">
              <div>
                <h5 className="font-display text-xs font-bold text-ink">
                  Offline License Envelope
                </h5>
                <p className="text-[11px] text-stone-500">
                  Signed Ed25519 file for air-gapped or offline computers.
                </p>
              </div>
              <button
                type="button"
                onClick={() =>
                  handleDownloadBlob(
                    `${orderResult.invoiceNumber}_license.coachifylicense`,
                    orderResult.licenseBlob,
                  )
                }
                className="inline-flex items-center gap-1.5 rounded-xl bg-ink px-3 py-2 text-xs font-semibold text-white shadow-sm hover:bg-stone-800 transition"
              >
                <DownloadIcon className="h-3.5 w-3.5" />
                Download License
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="p-6 sm:p-8" data-testid="checkout-modal-panel">
      <div className="flex items-center gap-3">
        <BrandLockup markClassName="h-10 w-10" textClassName="text-base" />
        <div>
          <h3 className="font-display text-lg font-bold text-ink">
            Online Plan Purchase &amp; Renewal
          </h3>
          <p className="text-xs text-stone-500">
            Instant activation with sequential Bangladesh invoice.
          </p>
        </div>
      </div>

      {/* Tier Switcher Tabs */}
      <div className="mt-5 grid grid-cols-2 gap-2 sm:grid-cols-4">
        {(Object.keys(CHECKOUT_TIERS) as CheckoutTierKey[]).map((k) => {
          const t = CHECKOUT_TIERS[k];
          const active = tierKey === k;
          return (
            <button
              key={k}
              type="button"
              onClick={() => setTierKey(k)}
              className={cn(
                "rounded-xl border p-2 text-left transition cursor-pointer select-none",
                active
                  ? "border-orange-500 bg-orange-50/70 shadow-sm ring-1 ring-orange-400"
                  : "border-stone-200 bg-white hover:bg-stone-50",
              )}
            >
              <div className="text-[11px] font-bold text-stone-800">
                {t.label}
              </div>
              <div className="mt-0.5 text-xs font-extrabold text-orange-600">
                ৳{t.totalBdt.toLocaleString()}
              </div>
            </button>
          );
        })}
      </div>

      {error && (
        <div className="mt-4 rounded-xl border border-rose-200 bg-rose-50 p-3 text-xs text-rose-700">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="mt-5 space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <Field
            label="Coaching Center Name"
            placeholder="e.g. Dhanmondi Science Academy"
            required
            value={orgName}
            onChange={setOrgName}
          />
          <Field
            label="Director / Owner Name"
            placeholder="e.g. Rafiqul Islam"
            required
            value={ownerName}
            onChange={setOwnerName}
          />
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <Field
            label="Mobile Number (bKash / Nagad)"
            placeholder="01711-000000"
            required
            value={phone}
            onChange={setPhone}
          />
          <Field
            label="Email Address (Optional)"
            placeholder="director@coaching.bd"
            type="email"
            value={email}
            onChange={setEmail}
          />
        </div>

        <Field
          label="Offline PC Request Code (Optional)"
          placeholder="e.g. REQ-XXXX-XXXX-XXXX-XXXX (leave blank for online pairing)"
          value={targetRequestCode}
          onChange={setTargetRequestCode}
        />

        {/* Payment Method Selector */}
        <div>
          <label className="mb-1.5 block text-xs font-semibold uppercase tracking-wider text-stone-500">
            Payment Method
          </label>
          <div className="grid grid-cols-3 gap-2">
            {[
              { id: "bkash", label: "bKash Direct", badge: "Instant" },
              { id: "nagad", label: "Nagad Direct", badge: "Instant" },
              { id: "bank_transfer", label: "Bank Transfer", badge: "EBL PLC" },
            ].map((m) => (
              <button
                key={m.id}
                type="button"
                onClick={() =>
                  setMethod(m.id as "bkash" | "nagad" | "bank_transfer")
                }
                className={cn(
                  "flex flex-col items-center justify-center rounded-xl border py-2.5 px-2 text-center transition cursor-pointer",
                  method === m.id
                    ? "border-orange-500 bg-orange-50/70 ring-1 ring-orange-400 font-bold text-ink"
                    : "border-stone-200 bg-white hover:bg-stone-50 text-stone-600",
                )}
              >
                <span className="text-xs">{m.label}</span>
                <span className="text-[10px] text-stone-400 font-normal">
                  {m.badge}
                </span>
              </button>
            ))}
          </div>
        </div>

        {method === "bank_transfer" && (
          <div className="rounded-xl border border-stone-200 bg-stone-50 p-3 space-y-2 text-xs">
            <p className="text-stone-600">
              Transfer to Eastern Bank PLC, A/C: <strong>1041060389201</strong>{" "}
              (Coachify Technologies Ltd). Enter transaction ID below:
            </p>
            <input
              type="text"
              placeholder="e.g. EBL-TRX-948102 or Bank Deposit Slip Reference"
              value={bankRef}
              onChange={(e) => setBankRef(e.target.value)}
              className="w-full rounded-lg border border-stone-300 bg-white px-3 py-2 text-xs text-stone-800"
            />
          </div>
        )}

        {/* Live Receipt Breakdown */}
        <div className="rounded-2xl border border-stone-200 bg-stone-50/80 p-3.5">
          <div className="flex items-center justify-between text-xs text-stone-500">
            <span>Selected Plan</span>
            <span className="font-semibold text-stone-800">{tier.label}</span>
          </div>
          <div className="mt-1 flex items-center justify-between text-xs text-stone-500">
            <span>Effective Rate</span>
            <span>
              {tier.monthlyRateBdt > 0
                ? `৳${tier.monthlyRateBdt.toLocaleString()}/month`
                : "One-time perpetual"}
            </span>
          </div>
          <div className="mt-1 flex items-center justify-between text-xs text-stone-500">
            <span>VAT / Taxes (0%)</span>
            <span>৳0</span>
          </div>
          <div className="mt-2 pt-2 border-t border-stone-200 flex items-center justify-between font-display text-sm font-bold text-ink">
            <span>Total Payable</span>
            <span className="text-orange-600 font-extrabold text-base">
              ৳{tier.totalBdt.toLocaleString()} BDT
            </span>
          </div>
        </div>

        <button
          type="submit"
          disabled={loading}
          className="w-full rounded-2xl bg-orange-500 hover:bg-orange-600 text-white font-semibold py-3 text-sm shadow-lift hover:shadow-glow transition cursor-pointer disabled:opacity-50"
        >
          {loading
            ? "Processing Order..."
            : method === "bank_transfer"
              ? "Submit Bank Transfer Order"
              : `Pay ৳${tier.totalBdt.toLocaleString()} with ${method === "bkash" ? "bKash" : "Nagad"}`}
        </button>
      </form>
    </div>
  );
}
