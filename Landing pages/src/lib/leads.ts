import { RELEASE_INFO } from "../data/appData";

export const LEADS_STORAGE_KEY = "coachify.landing.leads";

export interface LeadSubmissionInput {
  name: string;
  institution: string;
  contact: string;
  interest: string;
  studentsCount?: string;
  pcCount?: string;
  message?: string;
  source: "modal" | "contact_page";
}

export interface StoredLeadRecord extends LeadSubmissionInput {
  id: string;
  createdAt: string;
}

export function validateContactString(value: string): string | null {
  const trimmed = value.trim();
  if (!trimmed) {
    return "Please enter a phone/WhatsApp number or email address.";
  }
  const isEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
  const digits = trimmed.replace(/\D/g, "");
  const isPhone = digits.length >= 10 && digits.length <= 15;
  if (!isEmail && !isPhone) {
    return "Enter a valid Bangladesh mobile number (e.g. 01711-234567) or email address.";
  }
  return null;
}

export function readStoredLeads(): StoredLeadRecord[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = window.localStorage.getItem(LEADS_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    return Array.isArray(parsed) ? (parsed as StoredLeadRecord[]) : [];
  } catch {
    return [];
  }
}

export async function submitLeadRequest(
  input: LeadSubmissionInput,
): Promise<StoredLeadRecord> {
  const record: StoredLeadRecord = {
    ...input,
    name: input.name.trim(),
    institution: input.institution.trim(),
    contact: input.contact.trim(),
    id: `lead-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    createdAt: new Date().toISOString(),
  };

  if (typeof window !== "undefined") {
    try {
      const existing = readStoredLeads();
      window.localStorage.setItem(
        LEADS_STORAGE_KEY,
        JSON.stringify([record, ...existing]),
      );
    } catch {
      // Ignore localStorage quota errors
    }
  }

  const webhookUrl =
    typeof import.meta !== "undefined"
      ? (import.meta as { env?: Record<string, string | undefined> }).env
          ?.VITE_LEAD_WEBHOOK_URL
      : undefined;

  if (webhookUrl && typeof fetch !== "undefined") {
    try {
      await fetch(webhookUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(record),
      });
    } catch {
      // Non-blocking: local record and WhatsApp/mailto fallback remain available
    }
  }

  return record;
}

export function buildWhatsAppInquiryUrl(
  record: Pick<LeadSubmissionInput, "name" | "institution" | "interest">,
): string {
  const text = `Hello Coachify Team, I am ${record.name} from ${record.institution}. I am interested in ${record.interest}.`;
  return `https://wa.me/8801711234567?text=${encodeURIComponent(text)}`;
}

export function buildMailtoInquiryUrl(
  record: Pick<
    LeadSubmissionInput,
    "name" | "institution" | "interest" | "contact"
  >,
): string {
  const subject = `Coachify Inquiry — ${record.institution} (${record.interest})`;
  const body = `Name: ${record.name}\nInstitution: ${record.institution}\nContact: ${record.contact}\nPlan: ${record.interest}`;
  return `mailto:sales@coachify.app?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
}

export function triggerInstallerDownload(): void {
  if (typeof document === "undefined") return;
  const link = document.createElement("a");
  link.href = RELEASE_INFO.downloadUrl;
  link.download = RELEASE_INFO.fileName;
  link.rel = "noopener noreferrer";
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
