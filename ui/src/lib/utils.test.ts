import { describe, it, expect } from "vitest";
import { cn, formatTaka, formatAmount, formatBusinessDate, formatBytes } from "./utils";

describe("Design system utility functions", () => {
  it("merges classes and handles conflicting Tailwind classes", () => {
    expect(cn("px-2 py-1", "px-4")).toBe("py-1 px-4");
    expect(cn("text-red-500", undefined, "bg-card")).toBe("text-red-500 bg-card");
  });

  it("formats integer poisha to standard Taka currency", () => {
    // 100 poisha = 1 Taka
    expect(formatTaka(12450000)).toBe("৳ 1,24,500");
    expect(formatTaka(32800000)).toBe("৳ 3,28,000");
    expect(formatTaka(50000)).toBe("৳ 500");
    expect(formatTaka(0)).toBe("৳ 0");
  });

  it("formats amount without symbol", () => {
    expect(formatAmount(42000000)).toBe("4,20,000");
  });

  it("formats civil business date to human readable date", () => {
    expect(formatBusinessDate("2026-04-24")).toBe("Apr 24, 2026");
    expect(formatBusinessDate("2026-09-20")).toBe("Sep 20, 2026");
  });

  it("formats bytes into human readable size", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1024)).toBe("1.0 KB");
    expect(formatBytes(1048576 * 5)).toBe("5.0 MB");
    expect(formatBytes(1073741824 * 2.5)).toBe("2.5 GB");
  });
});
