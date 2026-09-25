import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { PageCta, PageHero } from "../components/PagePrimitives";
import {
  CheckIcon,
  MailIcon,
  PhoneIcon,
  SearchIcon,
  ArrowRightIcon,
} from "../components/icons";
import { submitLeadRequest, validateContactString } from "../lib/leads";

const HELP = [
  [
    "Getting Started",
    "Install the demo, open the demo institution and understand the main navigation.",
    ["Installation", "First sign-in", "Demo institution"],
  ],
  [
    "Institution Setup",
    "Configure identity, sessions, courses, batches, receipts and grading.",
    ["First setup", "Batches", "Student import"],
  ],
  [
    "Daily Operations",
    "Practical walkthroughs for attendance, fees, printing and communication.",
    ["Attendance", "Fee collection", "Printing"],
  ],
  [
    "Data Safety",
    "Create, schedule, verify and restore local backups.",
    ["Automatic backup", "External drive", "Restore"],
  ],
  [
    "Office Network",
    "Connect reception, accounts and administration over a local LAN.",
    ["Host PC", "LAN setup", "Permissions"],
  ],
  [
    "Services",
    "Configure optional SMS, email and software updates.",
    ["SMS setup", "Email setup", "Updates"],
  ],
  [
    "Troubleshooting",
    "Resolve common installation, printing and network issues.",
    ["Installer", "Printer", "LAN diagnostics"],
  ],
] as const;

export function HelpPage() {
  const [query, setQuery] = useState("");
  const filtered = HELP.filter((x) =>
    `${x[0]} ${x[1]} ${x[2].join(" ")}`
      .toLowerCase()
      .includes(query.toLowerCase()),
  );
  return (
    <div>
      <PageHero
        tag="Documentation & help"
        title="Answers that get your team"
        accent="back to work."
        copy="Search practical setup guides, daily workflow tutorials and troubleshooting steps without wading through technical language."
        primary="Download Coachify Demo"
      >
        <div className="relative max-w-2xl">
          <SearchIcon className="absolute left-5 top-1/2 h-5 w-5 -translate-y-1/2 text-stone-400" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search installation, fees, LAN, backups..."
            className="w-full rounded-2xl border border-stone-200 bg-white py-4 pl-13 pr-5 text-base text-ink shadow-lift outline-none focus:border-orange-400 focus:ring-4 focus:ring-orange-100"
          />
        </div>
      </PageHero>
      <section className="pb-28">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {filtered.map((x, i) => (
              <article
                key={x[0]}
                className="group border-t border-stone-200 py-7"
              >
                <span className="tnum text-xs font-bold text-orange-500">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <h2 className="mt-3 font-display text-xl font-bold text-ink">
                  {x[0]}
                </h2>
                <p className="mt-2 text-sm leading-relaxed text-stone-500">
                  {x[1]}
                </p>
                <div className="mt-4 space-y-2">
                  {x[2].map((t) => (
                    <button
                      key={t}
                      className="flex w-full cursor-pointer items-center justify-between rounded-lg bg-white px-3 py-2 text-left text-sm font-semibold text-stone-600 transition hover:bg-orange-50 hover:text-orange-700"
                    >
                      {t}
                      <ArrowRightIcon className="h-3.5 w-3.5" />
                    </button>
                  ))}
                </div>
              </article>
            ))}
          </div>
          {!filtered.length && (
            <p className="py-20 text-center text-stone-500">
              No guide matched “{query}”. Try a broader term or contact support.
            </p>
          )}
        </div>
      </section>
      <section className="bg-white py-20">
        <div className="mx-auto flex max-w-7xl flex-col justify-between gap-8 px-4 sm:px-6 lg:flex-row lg:items-center lg:px-8">
          <div>
            <p className="text-xs font-bold uppercase tracking-[.2em] text-orange-600">
              Human support
            </p>
            <h2 className="mt-3 font-display text-3xl font-extrabold text-ink">
              Still need help?
            </h2>
            <p className="mt-2 text-stone-500">
              Send the issue and your Coachify version. A real person will
              reply.
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <a
              href="mailto:support@coachify.app"
              className="inline-flex items-center gap-2 rounded-full bg-ink px-5 py-3 text-sm font-bold text-white"
            >
              <MailIcon className="h-4 w-4" />
              Contact Support
            </a>
            <Link
              to="/contact"
              className="inline-flex items-center gap-2 rounded-full border border-stone-300 bg-white px-5 py-3 text-sm font-bold text-ink"
            >
              <PhoneIcon className="h-4 w-4" />
              Book a call
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}

export function ContactPage() {
  const [sent, setSent] = useState(false);
  const [institution, setInstitution] = useState("");
  const [name, setName] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [studentsCount, setStudentsCount] = useState("");
  const [pcCount, setPcCount] = useState("1 PC");
  const [message, setMessage] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const validationError = validateContactString(phone.trim() || email.trim());
    if (validationError) {
      setError(validationError);
      return;
    }
    setError(null);
    await submitLeadRequest({
      name,
      institution,
      contact: `${phone.trim()} (${email.trim()})`,
      interest: pcCount,
      studentsCount,
      pcCount,
      message,
      source: "contact_page",
    });
    setSent(true);
  };

  return (
    <div>
      <PageHero
        tag="Sales & product demo"
        title="Want to see Coachify with"
        accent="your own workflow?"
        copy="Tell us how your institute operates. We will focus the conversation on your staff, student volume and office setup, not a generic sales script."
        primary="Download Coachify Demo"
      />
      <section className="pb-28">
        <div className="mx-auto grid max-w-7xl gap-12 px-4 sm:px-6 lg:grid-cols-[0.7fr_1.3fr] lg:px-8">
          <div>
            <h2 className="font-display text-3xl font-extrabold text-ink">
              Choose the conversation.
            </h2>
            <div className="mt-8 space-y-4">
              {[
                ["Book Demo", "See the relevant workflows live."],
                [
                  "Request Full Version",
                  "Discuss the right license and setup.",
                ],
                ["Ask a Question", "Get a direct answer before deciding."],
              ].map((x) => (
                <div
                  key={x[0]}
                  className="rounded-2xl border border-stone-200 bg-white p-5"
                >
                  <h3 className="font-bold text-ink">{x[0]}</h3>
                  <p className="mt-1 text-sm text-stone-500">{x[1]}</p>
                </div>
              ))}
            </div>
            <div className="mt-8 space-y-3 text-sm font-medium text-stone-500">
              <p className="flex items-center gap-2">
                <MailIcon className="h-4 w-4 text-orange-500" />
                sales@coachify.app
              </p>
              <p className="flex items-center gap-2">
                <PhoneIcon className="h-4 w-4 text-orange-500" />
                +880 1711-234567 · Sat–Thu, 10–7
              </p>
            </div>
          </div>
          <div className="rounded-3xl border border-stone-200 bg-white p-6 shadow-lift sm:p-9">
            {sent ? (
              <div className="flex min-h-[460px] flex-col items-center justify-center text-center">
                <span className="flex h-14 w-14 items-center justify-center rounded-full bg-orange-500 text-white">
                  <CheckIcon className="h-6 w-6" />
                </span>
                <h2 className="mt-5 font-display text-2xl font-bold text-ink">
                  Thanks. We have your request.
                </h2>
                <p className="mt-2 max-w-sm text-sm leading-relaxed text-stone-500">
                  A member of the Coachify team will respond using the contact
                  details you provided.
                </p>
                <button
                  onClick={() => setSent(false)}
                  className="mt-6 cursor-pointer text-sm font-bold text-orange-600"
                >
                  Send another request
                </button>
              </div>
            ) : (
              <form onSubmit={submit}>
                <div className="grid gap-5 sm:grid-cols-2">
                  <FormField
                    label="Institution name"
                    placeholder="Atomic Coaching Center"
                    value={institution}
                    onChange={setInstitution}
                  />
                  <FormField
                    label="Contact person"
                    placeholder="Your name"
                    value={name}
                    onChange={setName}
                  />
                  <FormField
                    label="Phone"
                    placeholder="Phone / WhatsApp"
                    value={phone}
                    onChange={(v) => {
                      setPhone(v);
                      if (error) setError(null);
                    }}
                  />
                  <FormField
                    label="Email"
                    placeholder="you@institute.com"
                    type="email"
                    value={email}
                    onChange={setEmail}
                  />
                  <FormField
                    label="Approximate students"
                    placeholder="e.g. 450"
                    value={studentsCount}
                    onChange={setStudentsCount}
                  />
                  <div>
                    <label className="mb-2 block text-xs font-bold uppercase tracking-wider text-stone-500">
                      Number of PCs
                    </label>
                    <select
                      value={pcCount}
                      onChange={(e) => setPcCount(e.target.value)}
                      className="w-full rounded-xl border border-stone-200 bg-white px-4 py-3 text-sm outline-none focus:border-orange-400"
                    >
                      <option>1 PC</option>
                      <option>2–3 PCs</option>
                      <option>4–5 PCs</option>
                      <option>More than 5</option>
                    </select>
                  </div>
                </div>
                {error && (
                  <p
                    role="alert"
                    className="mt-3 text-xs font-semibold text-rose-600"
                  >
                    {error}
                  </p>
                )}
                <div className="mt-5">
                  <label className="mb-2 block text-xs font-bold uppercase tracking-wider text-stone-500">
                    How can we help?
                  </label>
                  <textarea
                    required
                    rows={5}
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    placeholder="Tell us what you want to improve..."
                    className="w-full resize-none rounded-xl border border-stone-200 px-4 py-3 text-sm outline-none focus:border-orange-400 focus:ring-4 focus:ring-orange-100"
                  />
                </div>
                <button className="mt-6 w-full cursor-pointer rounded-full bg-orange-500 px-6 py-3.5 text-sm font-bold text-white shadow-lg shadow-orange-500/25 hover:bg-orange-600">
                  Request a conversation
                </button>
                <p className="mt-3 text-center text-xs text-stone-400">
                  Short form, no mailing list, one useful reply.
                </p>
              </form>
            )}
          </div>
        </div>
      </section>
      <PageCta
        title="Prefer to evaluate it yourself first?"
        copy="Download the full demo workflow and bring any questions to the conversation."
      />
    </div>
  );
}

function FormField({
  label,
  placeholder,
  type = "text",
  value,
  onChange,
}: {
  label: string;
  placeholder: string;
  type?: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div>
      <label className="mb-2 block text-xs font-bold uppercase tracking-wider text-stone-500">
        {label}
      </label>
      <input
        required
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-xl border border-stone-200 px-4 py-3 text-sm outline-none focus:border-orange-400 focus:ring-4 focus:ring-orange-100"
      />
    </div>
  );
}
