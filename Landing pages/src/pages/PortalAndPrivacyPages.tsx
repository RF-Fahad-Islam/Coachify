import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { SectionTag, CtaButton } from "../components/shared";
import {
  ShieldIcon,
  DownloadIcon,
  CheckIcon,
  MonitorIcon,
  WifiOffIcon,
  LockIcon,
  DatabaseIcon,
  CopyIcon,
  FileTextIcon,
} from "../components/icons";
import { cn } from "../utils/cn";

// Demo mock data aligned with Spec 0042 & Slice 12
const DEMO_CUSTOMER = {
  id: "cust-atomic-01",
  org_name: "Atomic Coaching Center",
  owner_name: "Tanvir Rahman",
  phone: "01711-234567",
  email: "owner@atomic.edu.bd",
  district: "Dhaka (Farmgate)",
};

const DEMO_LICENSE = {
  license_id: "COACH-PRO-2026-ATOMIC",
  tier: "PRO",
  status: "Active",
  paid_through: "2027-02-28",
  valid_until: "2027-03-30",
  max_students: 500,
  max_staff_users: 10,
  latest_serial: 2,
  transfers_used_12m: 0,
  transfers_limit_12m: 2,
  pairing_code: "COACH-ATOMIC-2026",
  signed_blob: JSON.stringify(
    {
      license_id: "COACH-PRO-2026-ATOMIC",
      customer_id: "cust-atomic-01",
      org_name: "Atomic Coaching Center",
      tier: "PRO",
      serial: 2,
      paid_through: "2027-02-28",
      valid_until: "2027-03-30",
      signature:
        "ed25519_sig_u2026a_mock_c0a7ef20b7914e9f73ab19280d90881923fa6102",
    },
    null,
    2,
  ),
};

const DEMO_INSTALLERS = {
  stable: {
    channel: "stable",
    version: "v1.0.0",
    filename: "CoachifySetup-1.0.0.exe",
    artifact_url:
      "https://releases.coachify.app/stable/CoachifySetup-1.0.0.exe",
    sha256: "b7e6b0f3e691459424c8b2512a832fa5a73e6d860d5e16f3ea83c27e8a93e36e",
    size_mb: 68.4,
    authenticode_thumbprint: "4B8F9A1278C62334F90BDE5E3A11C90B551022FE",
    authenticode_subject:
      "CN=Coachify Software Ltd., O=Coachify Software Ltd., C=BD",
    updater_key_id: "u2026a",
    updater_signature:
      "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVOVzJ1LytXWGt3Tkd4TjRsd3c0TG5v",
    published_at: "2026-03-01",
    notes:
      "Official General Availability production release for Windows 10/11 64-bit.",
  },
  beta: {
    channel: "beta",
    version: "v1.1.0-beta.1",
    filename: "CoachifySetup-1.1.0-beta.1.exe",
    artifact_url:
      "https://releases.coachify.app/beta/CoachifySetup-1.1.0-beta.1.exe",
    sha256: "8e716df332491a56f2e867ac396b27d42cfcf4b09b5581177696131498bfe130",
    size_mb: 69.1,
    authenticode_thumbprint: "4B8F9A1278C62334F90BDE5E3A11C90B551022FE",
    authenticode_subject:
      "CN=Coachify Software Ltd., O=Coachify Software Ltd., C=BD",
    updater_key_id: "u2026a",
    updater_signature:
      "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUkVDRU5UX1NJR19CRVRBMjAyNg==",
    published_at: "2026-03-20",
    notes:
      "Preview release featuring thermal roll receipt formatting and custom exam grade curves.",
  },
};

const DEMO_INVOICES = [
  {
    id: "inv-2026-0001",
    invoice_number: "INV-2026-00001",
    date: "2026-03-01",
    period: "2026-03-01 to 2027-02-28",
    description: "Coachify Pro Annual License (500 students, 10 staff)",
    amount_bdt: 14900,
    method: "bKash",
    reference: "TRX987654321",
    status: "Paid",
  },
];

const DEMO_INSTALLATIONS = [
  {
    id: "inst-frontdesk-01",
    label: "DESKTOP-FRONTDESK (Main Reception PC)",
    app_version: "1.0.0",
    active_students: 342,
    first_seen: "2026-03-01",
    last_seen: "2026-09-25",
    status: "Active",
  },
];

/**
 * Customer Self-Service Portal (/portal)
 * Spec 0042a AC-1 & AC-2: Phone/Email + Argon2id Password + 6-digit OTP auth,
 * scoped strictly to authenticated customer ID.
 */
export function PortalPage() {
  const [identifier, setIdentifier] = useState("owner@atomic.edu.bd");
  const [password, setPassword] = useState("Coachify@2026");
  const [step, setStep] = useState<"login" | "otp" | "dashboard">("dashboard");
  const [otpCode, setOtpCode] = useState("");
  const [maskedDest, setMaskedDest] = useState("o***@atomic.edu.bd");
  const [authError, setAuthError] = useState<string | null>(null);
  const [failedAttempts, setFailedAttempts] = useState(0);
  const [copiedSha, setCopiedSha] = useState(false);
  const [activeTab, setActiveTab] = useState<
    "license" | "installer" | "invoices" | "installations"
  >("license");
  const [channel, setChannel] = useState<"stable" | "beta">("stable");
  const [selectedInvoice, setSelectedInvoice] = useState<
    (typeof DEMO_INVOICES)[0] | null
  >(null);

  const handleLoginSubmit = (e: FormEvent) => {
    e.preventDefault();
    setAuthError(null);
    if (!identifier.trim() || !password.trim()) {
      setAuthError("Please enter your phone or email and password.");
      return;
    }
    // Simulate OTP challenge
    setMaskedDest(
      identifier.includes("@")
        ? identifier.replace(/(?<=^.).+(?=@)/, "***")
        : identifier.slice(0, 3) + "***" + identifier.slice(-4),
    );
    setStep("otp");
  };

  const handleOtpSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (otpCode.trim() === "123456" || otpCode.trim().length === 6) {
      setStep("dashboard");
      setAuthError(null);
      setFailedAttempts(0);
    } else {
      const next = failedAttempts + 1;
      setFailedAttempts(next);
      if (next >= 5) {
        setAuthError(
          "PORTAL_ACCOUNT_LOCKED: Account locked for 15 minutes due to 5 consecutive failed attempts.",
        );
      } else {
        setAuthError(
          `Invalid verification code. ${5 - next} attempts remaining. (Demo code: 123456)`,
        );
      }
    }
  };

  const downloadLicenseFile = () => {
    const element = document.createElement("a");
    const file = new Blob([DEMO_LICENSE.signed_blob], {
      type: "application/json",
    });
    element.href = URL.createObjectURL(file);
    element.download = `${DEMO_LICENSE.license_id}.coachifylicense`;
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
  };

  const copyChecksum = (text: string) => {
    navigator.clipboard?.writeText(text);
    setCopiedSha(true);
    setTimeout(() => setCopiedSha(false), 2000);
  };

  return (
    <div className="min-h-screen bg-sand pb-24 pt-32 sm:pt-40">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        {step !== "dashboard" ? (
          <div className="mx-auto max-w-md">
            <div className="overflow-hidden rounded-3xl border border-stone-200 bg-white p-8 shadow-lift sm:p-10">
              <div className="text-center">
                <span className="inline-flex items-center gap-1.5 rounded-full bg-orange-100 px-3 py-1 text-xs font-bold text-orange-800">
                  <ShieldIcon className="h-3.5 w-3.5" /> Customer Portal
                </span>
                <h1 className="mt-4 font-display text-3xl font-extrabold text-ink">
                  {step === "login"
                    ? "Sign in to portal"
                    : "Enter 6-digit code"}
                </h1>
                <p className="mt-2 text-sm text-stone-500">
                  {step === "login"
                    ? "Access your Coachify license, verified Windows installers, and paid invoices."
                    : `We sent a temporary verification code to ${maskedDest}. Valid for 10 minutes.`}
                </p>
              </div>

              {authError && (
                <div className="mt-6 rounded-xl border border-red-200 bg-red-50 p-3.5 text-xs font-semibold text-red-700">
                  {authError}
                </div>
              )}

              {step === "login" ? (
                <form onSubmit={handleLoginSubmit} className="mt-6 space-y-4">
                  <div>
                    <label className="block text-xs font-bold uppercase tracking-wider text-stone-600">
                      Mobile number or email
                    </label>
                    <input
                      type="text"
                      value={identifier}
                      onChange={(e) => setIdentifier(e.target.value)}
                      placeholder="01711-234567 or director@coaching.edu.bd"
                      className="mt-1.5 w-full rounded-xl border border-stone-300 bg-cream/40 px-3.5 py-2.5 text-sm font-medium text-ink focus:border-orange-500 focus:outline-none focus:ring-2 focus:ring-orange-200"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-bold uppercase tracking-wider text-stone-600">
                      Password
                    </label>
                    <input
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="••••••••••••"
                      className="mt-1.5 w-full rounded-xl border border-stone-300 bg-cream/40 px-3.5 py-2.5 text-sm font-medium text-ink focus:border-orange-500 focus:outline-none focus:ring-2 focus:ring-orange-200"
                    />
                  </div>
                  <div className="rounded-xl border border-amber-200 bg-amber-50/70 p-3 text-xs text-amber-900">
                    💡 <strong>Demo Portal Credentials:</strong> identifier{" "}
                    <code>owner@atomic.edu.bd</code>, password{" "}
                    <code>Coachify@2026</code>
                  </div>
                  <CtaButton
                    type="submit"
                    className="w-full justify-center"
                    size="md"
                  >
                    Continue with OTP →
                  </CtaButton>
                </form>
              ) : (
                <form onSubmit={handleOtpSubmit} className="mt-6 space-y-4">
                  <div>
                    <label className="block text-xs font-bold uppercase tracking-wider text-stone-600">
                      6-Digit Security OTP
                    </label>
                    <input
                      type="text"
                      maxLength={6}
                      value={otpCode}
                      onChange={(e) => setOtpCode(e.target.value)}
                      placeholder="123456"
                      className="mt-1.5 w-full text-center tracking-[0.4em] font-mono text-2xl font-bold rounded-xl border border-stone-300 bg-cream/40 px-3.5 py-3 text-ink focus:border-orange-500 focus:outline-none focus:ring-2 focus:ring-orange-200"
                    />
                  </div>
                  <div className="rounded-xl border border-emerald-200 bg-emerald-50/70 p-3 text-xs text-emerald-900">
                    Demo OTP code: <strong>123456</strong> (expires in 10
                    minutes)
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => setStep("login")}
                      className="flex-1 rounded-xl border border-stone-300 py-2.5 text-sm font-bold text-stone-600 hover:bg-stone-50"
                    >
                      Back
                    </button>
                    <CtaButton
                      type="submit"
                      className="flex-2 justify-center"
                      size="md"
                    >
                      Verify &amp; Sign In
                    </CtaButton>
                  </div>
                </form>
              )}
            </div>
          </div>
        ) : (
          <div>
            {/* Customer Portal Header */}
            <div className="flex flex-col justify-between gap-4 border-b border-stone-200 pb-6 sm:flex-row sm:items-center">
              <div>
                <div className="flex items-center gap-2">
                  <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-extrabold uppercase tracking-wide text-emerald-800">
                    Authenticated Customer
                  </span>
                  <span className="text-xs font-semibold text-stone-400">
                    Tenant ID: {DEMO_CUSTOMER.id}
                  </span>
                </div>
                <h1 className="mt-1 font-display text-3xl font-extrabold text-ink sm:text-4xl">
                  {DEMO_CUSTOMER.org_name}
                </h1>
                <p className="mt-1 text-sm text-stone-500">
                  Owner: {DEMO_CUSTOMER.owner_name} · {DEMO_CUSTOMER.phone} ·{" "}
                  {DEMO_CUSTOMER.district}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setStep("login")}
                  className="rounded-full border border-stone-300 bg-white px-4 py-2 text-xs font-bold text-stone-700 hover:bg-stone-100"
                >
                  Sign Out
                </button>
              </div>
            </div>

            {/* Navigation Tabs */}
            <div className="mt-6 flex flex-wrap gap-2 border-b border-stone-200 pb-3">
              {[
                ["license", "License & Key File"],
                ["installer", "Verified Installers"],
                ["invoices", "Invoices & Receipts"],
                ["installations", "Linked PCs"],
              ].map(([key, label]) => (
                <button
                  key={key}
                  onClick={() => setActiveTab(key as typeof activeTab)}
                  className={cn(
                    "rounded-xl px-4 py-2 text-sm font-bold transition",
                    activeTab === key
                      ? "bg-orange-500 text-white shadow-md shadow-orange-500/20"
                      : "bg-white text-stone-600 hover:bg-orange-50 hover:text-orange-700",
                  )}
                >
                  {label}
                </button>
              ))}
            </div>

            {/* Tab 1: License Status & File Download */}
            {activeTab === "license" && (
              <div className="mt-6 grid gap-6 lg:grid-cols-3">
                <div className="lg:col-span-2 space-y-6">
                  <div className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift sm:p-8">
                    <div className="flex items-center justify-between border-b border-stone-100 pb-4">
                      <div>
                        <span className="text-xs font-bold uppercase tracking-wider text-stone-400">
                          Active License Key
                        </span>
                        <p className="font-mono text-xl font-extrabold text-ink sm:text-2xl">
                          {DEMO_LICENSE.license_id}
                        </p>
                      </div>
                      <span className="rounded-full bg-emerald-100 px-3 py-1 text-xs font-extrabold text-emerald-800">
                        {DEMO_LICENSE.status}
                      </span>
                    </div>

                    <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                      <div className="rounded-2xl border border-stone-200 bg-cream/30 p-4">
                        <span className="text-[11px] font-bold uppercase tracking-wider text-stone-400">
                          Edition Tier
                        </span>
                        <p className="mt-1 font-display text-xl font-bold text-orange-600">
                          {DEMO_LICENSE.tier}
                        </p>
                        <p className="text-xs text-stone-500">
                          Capacity: {DEMO_LICENSE.max_students} students
                        </p>
                      </div>
                      <div className="rounded-2xl border border-stone-200 bg-cream/30 p-4">
                        <span className="text-[11px] font-bold uppercase tracking-wider text-stone-400">
                          Paid Through
                        </span>
                        <p className="mt-1 font-mono text-lg font-bold text-ink">
                          {DEMO_LICENSE.paid_through}
                        </p>
                        <p className="text-xs text-stone-500">
                          Valid grace until {DEMO_LICENSE.valid_until}
                        </p>
                      </div>
                      <div className="rounded-2xl border border-stone-200 bg-cream/30 p-4">
                        <span className="text-[11px] font-bold uppercase tracking-wider text-stone-400">
                          Hardware Transfers
                        </span>
                        <p className="mt-1 font-display text-lg font-bold text-ink">
                          {DEMO_LICENSE.transfers_used_12m} /{" "}
                          {DEMO_LICENSE.transfers_limit_12m}{" "}
                          <span className="text-xs font-normal text-stone-500">
                            (12-mo)
                          </span>
                        </p>
                        <p className="text-xs text-stone-500">
                          Serial #{DEMO_LICENSE.latest_serial}
                        </p>
                      </div>
                    </div>

                    <div className="mt-6 flex flex-col gap-3 rounded-2xl border border-orange-200 bg-orange-50/50 p-5 sm:flex-row sm:items-center sm:justify-between">
                      <div>
                        <h4 className="font-bold text-ink">
                          Offline License File (.coachifylicense)
                        </h4>
                        <p className="text-xs text-stone-600">
                          Download this cryptographically signed file and load
                          it into Coachify Desktop without an active internet
                          connection.
                        </p>
                      </div>
                      <CtaButton
                        size="sm"
                        onClick={downloadLicenseFile}
                        icon={<DownloadIcon className="h-4 w-4" />}
                      >
                        Download .coachifylicense
                      </CtaButton>
                    </div>
                  </div>
                </div>

                {/* Pairing Code Card */}
                <div className="space-y-6">
                  <div className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift sm:p-8">
                    <span className="text-xs font-bold uppercase tracking-wider text-stone-400">
                      Pairing Code
                    </span>
                    <h3 className="mt-1 font-mono text-2xl font-black text-orange-600">
                      {DEMO_LICENSE.pairing_code}
                    </h3>
                    <p className="mt-2 text-xs leading-relaxed text-stone-500">
                      Enter this pairing code on your desktop installation to
                      link it directly to this license without transferring
                      files.
                    </p>
                    <div className="mt-6 space-y-2 border-t border-stone-100 pt-4 text-xs text-stone-600">
                      <div className="flex items-center gap-2">
                        <CheckIcon className="h-4 w-4 text-emerald-600" />
                        <span>Max staff accounts: 10 operators</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <CheckIcon className="h-4 w-4 text-emerald-600" />
                        <span>Local LAN Multi-User Hub enabled</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <CheckIcon className="h-4 w-4 text-emerald-600" />
                        <span>Thermal &amp; Laser receipt printing</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Tab 2: Verified Windows Installers */}
            {activeTab === "installer" && (
              <div className="mt-6 space-y-6">
                <div className="flex items-center gap-3">
                  <span className="text-xs font-bold uppercase tracking-wider text-stone-500">
                    Release Channel:
                  </span>
                  <div className="inline-flex rounded-xl border border-stone-300 bg-white p-1">
                    <button
                      onClick={() => setChannel("stable")}
                      className={cn(
                        "rounded-lg px-3 py-1.5 text-xs font-bold transition",
                        channel === "stable"
                          ? "bg-ink text-white"
                          : "text-stone-600 hover:text-ink",
                      )}
                    >
                      Stable (General Availability)
                    </button>
                    <button
                      onClick={() => setChannel("beta")}
                      className={cn(
                        "rounded-lg px-3 py-1.5 text-xs font-bold transition",
                        channel === "beta"
                          ? "bg-orange-500 text-white"
                          : "text-stone-600 hover:text-ink",
                      )}
                    >
                      Beta Preview (Early Access)
                    </button>
                  </div>
                </div>

                {(() => {
                  const inst = DEMO_INSTALLERS[channel];
                  return (
                    <div className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift sm:p-8">
                      <div className="flex flex-col justify-between gap-4 border-b border-stone-100 pb-5 sm:flex-row sm:items-center">
                        <div>
                          <div className="flex items-center gap-2.5">
                            <span className="font-display text-2xl font-black text-ink">
                              Coachify {inst.version}
                            </span>
                            <span
                              className={cn(
                                "rounded-full px-2.5 py-0.5 text-xs font-extrabold uppercase",
                                channel === "stable"
                                  ? "bg-emerald-100 text-emerald-800"
                                  : "bg-amber-100 text-amber-800",
                              )}
                            >
                              {channel} channel
                            </span>
                          </div>
                          <p className="mt-1 text-xs text-stone-500">
                            Published on {inst.published_at} · Size:{" "}
                            {inst.size_mb} MB · Windows 10 / 11 64-bit
                          </p>
                        </div>
                        <a
                          href={inst.artifact_url}
                          download={inst.filename}
                          className="inline-flex items-center justify-center gap-2 rounded-full bg-orange-500 px-6 py-3 text-sm font-bold text-white shadow-lg shadow-orange-500/25 transition hover:bg-orange-600"
                        >
                          <DownloadIcon className="h-4 w-4" /> Download{" "}
                          {inst.filename}
                        </a>
                      </div>

                      <div className="mt-6 grid gap-4 lg:grid-cols-2">
                        {/* SHA256 */}
                        <div className="rounded-2xl border border-stone-200 bg-cream/40 p-4">
                          <div className="flex items-center justify-between">
                            <span className="text-xs font-bold uppercase tracking-wider text-stone-500">
                              SHA256 Checksum
                            </span>
                            <button
                              onClick={() => copyChecksum(inst.sha256)}
                              className="inline-flex items-center gap-1 rounded-lg border border-stone-300 bg-white px-2 py-1 text-xs font-semibold text-stone-700 hover:bg-stone-50"
                            >
                              <CopyIcon className="h-3.5 w-3.5" />
                              {copiedSha ? "Copied!" : "Copy"}
                            </button>
                          </div>
                          <p className="mt-2 break-all font-mono text-xs text-stone-700">
                            {inst.sha256}
                          </p>
                        </div>

                        {/* Authenticode */}
                        <div className="rounded-2xl border border-emerald-200 bg-emerald-50/40 p-4">
                          <div className="flex items-center gap-2">
                            <ShieldIcon className="h-4 w-4 text-emerald-700" />
                            <span className="text-xs font-bold uppercase tracking-wider text-emerald-800">
                              Verified Microsoft Authenticode Signer
                            </span>
                          </div>
                          <p className="mt-2 text-xs font-semibold text-stone-800">
                            Subject: {inst.authenticode_subject}
                          </p>
                          <p className="mt-1 font-mono text-[11px] text-stone-500">
                            Thumbprint: {inst.authenticode_thumbprint}
                          </p>
                        </div>
                      </div>

                      <div className="mt-4 rounded-2xl border border-stone-200 bg-cream/30 p-4 text-xs">
                        <span className="font-bold text-ink">
                          Tauri Updater Signature (Key: {inst.updater_key_id}):
                        </span>
                        <p className="mt-1 truncate font-mono text-stone-500">
                          {inst.updater_signature}
                        </p>
                        <p className="mt-2 text-stone-600">
                          <strong>Release notes:</strong> {inst.notes}
                        </p>
                      </div>
                    </div>
                  );
                })()}
              </div>
            )}

            {/* Tab 3: Official Paid Invoices */}
            {activeTab === "invoices" && (
              <div className="mt-6 space-y-6">
                <div className="overflow-hidden rounded-3xl border border-stone-200 bg-white shadow-lift">
                  <div className="border-b border-stone-100 p-6 sm:p-8">
                    <h3 className="font-display text-xl font-bold text-ink">
                      Official Vendor Invoices (BDT ৳)
                    </h3>
                    <p className="mt-1 text-xs text-stone-500">
                      Paid official invoices issued by Coachify Software Ltd.
                      for license purchases and renewals.
                    </p>
                  </div>
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm">
                      <thead className="border-b border-stone-100 bg-cream/40 text-xs font-bold uppercase tracking-wider text-stone-500">
                        <tr>
                          <th className="px-6 py-3.5">Invoice #</th>
                          <th className="px-6 py-3.5">Date</th>
                          <th className="px-6 py-3.5">Coverage Period</th>
                          <th className="px-6 py-3.5">Amount</th>
                          <th className="px-6 py-3.5">Payment</th>
                          <th className="px-6 py-3.5">Status</th>
                          <th className="px-6 py-3.5 text-right">Receipt</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-stone-100">
                        {DEMO_INVOICES.map((inv) => (
                          <tr key={inv.id} className="hover:bg-cream/20">
                            <td className="px-6 py-4 font-mono font-bold text-ink">
                              {inv.invoice_number}
                            </td>
                            <td className="px-6 py-4 text-stone-600">
                              {inv.date}
                            </td>
                            <td className="px-6 py-4 text-xs text-stone-500">
                              {inv.period}
                            </td>
                            <td className="px-6 py-4 font-bold text-ink">
                              ৳{inv.amount_bdt.toLocaleString("en-US")}
                            </td>
                            <td className="px-6 py-4 text-xs text-stone-600">
                              <span className="font-bold">{inv.method}</span> (
                              {inv.reference})
                            </td>
                            <td className="px-6 py-4">
                              <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 text-xs font-extrabold text-emerald-800">
                                {inv.status}
                              </span>
                            </td>
                            <td className="px-6 py-4 text-right">
                              <button
                                onClick={() => setSelectedInvoice(inv)}
                                className="inline-flex items-center gap-1 rounded-lg border border-stone-300 bg-white px-2.5 py-1 text-xs font-bold text-stone-700 hover:border-orange-400 hover:text-orange-600"
                              >
                                <FileTextIcon className="h-3.5 w-3.5" /> View
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* Modal preview for selected invoice */}
                {selectedInvoice && (
                  <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/60 p-4 backdrop-blur-sm">
                    <div className="w-full max-w-lg rounded-3xl bg-white p-6 shadow-2xl sm:p-8">
                      <div className="flex items-center justify-between border-b border-stone-100 pb-4">
                        <div>
                          <span className="text-xs font-bold uppercase tracking-wider text-orange-600">
                            Paid Receipt
                          </span>
                          <h3 className="font-mono text-xl font-bold text-ink">
                            {selectedInvoice.invoice_number}
                          </h3>
                        </div>
                        <button
                          onClick={() => setSelectedInvoice(null)}
                          className="rounded-full border border-stone-300 p-1.5 text-stone-500 hover:bg-stone-100"
                        >
                          ✕
                        </button>
                      </div>
                      <div className="mt-4 space-y-3 text-xs text-stone-600">
                        <div className="flex justify-between py-1 border-b border-stone-50">
                          <span className="text-stone-400">Billed To:</span>
                          <span className="font-bold text-ink">
                            {DEMO_CUSTOMER.org_name} ({DEMO_CUSTOMER.owner_name}
                            )
                          </span>
                        </div>
                        <div className="flex justify-between py-1 border-b border-stone-50">
                          <span className="text-stone-400">Description:</span>
                          <span className="font-medium text-ink">
                            {selectedInvoice.description}
                          </span>
                        </div>
                        <div className="flex justify-between py-1 border-b border-stone-50">
                          <span className="text-stone-400">Period:</span>
                          <span className="font-medium text-ink">
                            {selectedInvoice.period}
                          </span>
                        </div>
                        <div className="flex justify-between py-1 border-b border-stone-50">
                          <span className="text-stone-400">Payment:</span>
                          <span className="font-bold text-ink">
                            {selectedInvoice.method} (Ref:{" "}
                            {selectedInvoice.reference})
                          </span>
                        </div>
                        <div className="flex justify-between py-2 border-t border-stone-200 text-sm font-bold text-ink">
                          <span>Total Paid:</span>
                          <span className="font-mono text-orange-600">
                            ৳
                            {selectedInvoice.amount_bdt.toLocaleString("en-US")}
                          </span>
                        </div>
                      </div>
                      <div className="mt-6 flex justify-end gap-2">
                        <button
                          onClick={() => window.print()}
                          className="rounded-xl bg-orange-500 px-4 py-2 text-xs font-bold text-white hover:bg-orange-600"
                        >
                          Print Invoice
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Tab 4: Bound PC Installations */}
            {activeTab === "installations" && (
              <div className="mt-6 space-y-6">
                <div className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift sm:p-8">
                  <h3 className="font-display text-xl font-bold text-ink">
                    Bound PC Installations
                  </h3>
                  <p className="mt-1 text-xs text-stone-500">
                    Machines currently authorized to execute Coachify with this
                    license.
                  </p>
                  <div className="mt-6 divide-y divide-stone-100">
                    {DEMO_INSTALLATIONS.map((inst) => (
                      <div
                        key={inst.id}
                        className="flex flex-col justify-between gap-4 py-4 sm:flex-row sm:items-center"
                      >
                        <div className="flex items-center gap-3">
                          <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-orange-100 text-orange-600">
                            <MonitorIcon className="h-5 w-5" />
                          </div>
                          <div>
                            <h4 className="font-bold text-ink">{inst.label}</h4>
                            <p className="font-mono text-xs text-stone-400">
                              Installation ID: {inst.id} · v{inst.app_version}
                            </p>
                          </div>
                        </div>
                        <div className="text-right text-xs">
                          <span className="rounded-full bg-emerald-100 px-2.5 py-0.5 font-bold text-emerald-800">
                            {inst.status}
                          </span>
                          <p className="mt-1 text-stone-400">
                            Last synced: {inst.last_seen}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>

                  <div className="mt-8 rounded-2xl border border-amber-200 bg-amber-50/60 p-4 text-xs text-amber-900 leading-relaxed">
                    <strong>Hardware Transfer Policy:</strong> Up to 2 PC
                    transfers are permitted per 12-month period for motherboard
                    upgrades or replacement computers. To transfer to a new PC,
                    install Coachify and enter your license pairing code.
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Privacy & Telemetry Transparency Page (/privacy)
 * Spec 0042a AC-3: Discloses the exact 5 activation/refresh fields and
 * 2 update query fields, certifying zero student PII leaves the PC.
 */
export function PrivacyPage() {
  return (
    <div className="min-h-screen bg-sand pb-28 pt-32 sm:pt-40">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="max-w-3xl">
          <SectionTag>Data privacy &amp; transparency</SectionTag>
          <h1 className="mt-5 font-display text-4xl font-extrabold tracking-tight text-ink sm:text-5xl lg:text-6xl">
            Your students' data{" "}
            <span className="text-orange-500">never leaves your computer.</span>
          </h1>
          <p className="mt-6 text-lg leading-relaxed text-stone-600">
            Coachify is architected from the first line of code as an
            offline-first desktop system. Unlike cloud apps that stream your
            students' names, phone numbers, marks and fees onto third-party
            servers, Coachify stores all records in encrypted local SQLite
            databases on your own PC.
          </p>
        </div>

        {/* 4 Core Commitments */}
        <div className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {[
            [
              <WifiOffIcon className="h-6 w-6 text-orange-500" />,
              "Zero Student PII Transmitted",
              "Names, phone numbers, guardians, marks, transactions, and documents never touch any network.",
            ],
            [
              <LockIcon className="h-6 w-6 text-orange-500" />,
              "Hardware-Bound Encryption",
              "Encrypted at rest using AES-256-GCM via Windows DPAPI and master paper recovery keys.",
            ],
            [
              <DatabaseIcon className="h-6 w-6 text-orange-500" />,
              "Pure Local SQLite",
              "You own the raw database files. You can back up to any external USB drive or local LAN folder.",
            ],
            [
              <ShieldIcon className="h-6 w-6 text-orange-500" />,
              "Full Telemetry Transparency",
              "Every single byte sent for license activation or updates is cataloged in the tables below.",
            ],
          ].map(([icon, title, desc], i) => (
            <div
              key={i}
              className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift"
            >
              <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-orange-50">
                {icon}
              </div>
              <h3 className="mt-4 font-display text-lg font-bold text-ink">
                {title as string}
              </h3>
              <p className="mt-2 text-xs leading-relaxed text-stone-500">
                {desc as string}
              </p>
            </div>
          ))}
        </div>

        {/* Telemetry Disclosures */}
        <div className="mt-16 space-y-12">
          {/* Table 1: Activation / Refresh fields */}
          <div className="overflow-hidden rounded-3xl border border-stone-200 bg-white shadow-lift">
            <div className="border-b border-stone-100 p-6 sm:p-8">
              <span className="text-xs font-bold uppercase tracking-wider text-orange-600">
                Specification 0042a · AC-3
              </span>
              <h2 className="mt-1 font-display text-2xl font-bold text-ink">
                Exact Fields Sent During License Activation &amp; Refresh (5
                Fields)
              </h2>
              <p className="mt-2 text-sm text-stone-500">
                Endpoint: <code>POST /v1/licenses/activate</code> and{" "}
                <code>POST /v1/licenses/refresh</code> (Only runs once every 30
                days or on manual license import).
              </p>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-sm">
                <thead className="border-b border-stone-100 bg-cream/40 text-xs font-bold uppercase tracking-wider text-stone-500">
                  <tr>
                    <th className="px-6 py-3.5">Field Name</th>
                    <th className="px-6 py-3.5">Data Type</th>
                    <th className="px-6 py-3.5">Example Payload</th>
                    <th className="px-6 py-3.5">Technical Purpose</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-stone-100 font-mono text-xs">
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">
                      installation_id
                    </td>
                    <td className="px-6 py-4 text-stone-600">
                      String (UUIDv4)
                    </td>
                    <td className="px-6 py-4 text-stone-500">
                      "f47ac10b-58cc-4372-a567-0e02b2c3d479"
                    </td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Binds license to this PC to enforce the 2 hardware
                      transfer annual limit.
                    </td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">license_id</td>
                    <td className="px-6 py-4 text-stone-600">String</td>
                    <td className="px-6 py-4 text-stone-500">
                      "COACH-PRO-2026-ATOMIC"
                    </td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Identifies the purchased coaching center license.
                    </td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">
                      current_serial
                    </td>
                    <td className="px-6 py-4 text-stone-600">Integer</td>
                    <td className="px-6 py-4 text-stone-500">2</td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Monotonic counter preventing license replay or rollbacks.
                    </td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">
                      app_version
                    </td>
                    <td className="px-6 py-4 text-stone-600">String</td>
                    <td className="px-6 py-4 text-stone-500">"1.0.0"</td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Checks client compatibility and minimum upgrade
                      migrations.
                    </td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">
                      active_students
                    </td>
                    <td className="px-6 py-4 text-stone-600">Integer</td>
                    <td className="px-6 py-4 text-stone-500">342</td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Aggregate student count bucketed for tier validation (e.g.
                      tier cap of 500).
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          {/* Table 2: Auto-Update Check fields */}
          <div className="overflow-hidden rounded-3xl border border-stone-200 bg-white shadow-lift">
            <div className="border-b border-stone-100 p-6 sm:p-8">
              <span className="text-xs font-bold uppercase tracking-wider text-orange-600">
                Specification 0042b · AC-7
              </span>
              <h2 className="mt-1 font-display text-2xl font-bold text-ink">
                Exact Fields Sent During Auto-Update Queries (2 Fields)
              </h2>
              <p className="mt-2 text-sm text-stone-500">
                Endpoint:{" "}
                <code>
                  GET /v1/updates/{"{channel}"}/{"{current_version}"}
                </code>
              </p>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-sm">
                <thead className="border-b border-stone-100 bg-cream/40 text-xs font-bold uppercase tracking-wider text-stone-500">
                  <tr>
                    <th className="px-6 py-3.5">Field Name</th>
                    <th className="px-6 py-3.5">Data Type</th>
                    <th className="px-6 py-3.5">Example Payload</th>
                    <th className="px-6 py-3.5">Technical Purpose</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-stone-100 font-mono text-xs">
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">channel</td>
                    <td className="px-6 py-4 text-stone-600">String</td>
                    <td className="px-6 py-4 text-stone-500">"stable"</td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Returns updates only for the user-selected channel (stable
                      or beta).
                    </td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 font-bold text-ink">
                      current_version
                    </td>
                    <td className="px-6 py-4 text-stone-600">String</td>
                    <td className="px-6 py-4 text-stone-500">"1.0.0"</td>
                    <td className="px-6 py-4 font-sans text-xs text-stone-600">
                      Allows server to check if an update is newer and safe to
                      apply.
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        {/* Audit Certification Box */}
        <div className="mt-16 rounded-3xl border border-stone-300 bg-white p-8 shadow-lift sm:p-10">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <div className="flex items-center gap-2">
                <ShieldIcon className="h-5 w-5 text-emerald-600" />
                <span className="text-xs font-extrabold uppercase tracking-wide text-emerald-800">
                  Data Sovereignty Certification
                </span>
              </div>
              <h3 className="mt-2 font-display text-2xl font-bold text-ink">
                Bangla / English Transparency Guarantee
              </h3>
              <p className="mt-2 max-w-2xl text-sm leading-relaxed text-stone-600">
                কোচিফাই সফটওয়্যারে আপনার ছাত্র-ছাত্রীদের নাম, মোবাইল নম্বর,
                অভিভাবকের তথ্য, ফি কিংবা পরীক্ষার ফলাফল কখনো কোনো ক্লাউড
                সার্ভারে আপলোড করা হয় না। সফটওয়্যারটি সম্পূর্ণ অফলাইনে কাজ করে
                এবং ডেটা আপনার পিসিতে এনক্রিপ্ট করা অবস্থায় সংরক্ষিত থাকে।
              </p>
            </div>
            <Link
              to="/portal"
              className="inline-flex items-center justify-center rounded-full bg-ink px-6 py-3 text-sm font-bold text-white shadow-lift transition hover:bg-orange-600"
            >
              Go to Customer Portal →
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
