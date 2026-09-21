import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Combines multiple CSS class names with tailwind-merge conflict resolution.
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/**
 * Formats integer poisha (100 poisha = 1 BDT Taka) to standard currency display.
 * Example: 12450000 poisha -> "৳ 1,24,500"
 */
export function formatTaka(poisha: number): string {
  const taka = Math.floor(poisha / 100);
  const formatted = new Intl.NumberFormat("en-IN").format(taka);
  return `৳ ${formatted}`;
}

/**
 * Formats integer poisha to plain amount without symbol.
 */
export function formatAmount(poisha: number): string {
  const taka = Math.floor(poisha / 100);
  return new Intl.NumberFormat("en-IN").format(taka);
}

/**
 * Formats standard ISO or civil business date (YYYY-MM-DD) into readable format.
 * Example: "2026-04-24" -> "Apr 24, 2026"
 */
export function formatBusinessDate(dateStr: string): string {
  if (!dateStr) return "";
  try {
    const [year, month, day] = dateStr.split("-").map(Number);
    if (!year || !month || !day) return dateStr;
    const date = new Date(year, month - 1, day);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  } catch {
    return dateStr;
  }
}

/**
 * Formats a normalized Bangladeshi phone number (e.g. "8801712345678") into readable format: "+880 1712-345678" or "01712-345678".
 */
export function formatPhone(phone: string): string {
  if (!phone) return "";
  const cleaned = phone.replace(/\D/g, "");
  if (cleaned.startsWith("880") && cleaned.length === 13) {
    return `+880 ${cleaned.slice(3, 7)}-${cleaned.slice(7)}`;
  }
  if (cleaned.startsWith("01") && cleaned.length === 11) {
    return `${cleaned.slice(0, 5)}-${cleaned.slice(5)}`;
  }
  return phone;
}
