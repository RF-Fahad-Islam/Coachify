import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { DateRangeFilter, DATE_RANGE_PRESETS } from "./date-range-filter";

describe("DateRangeFilter component", () => {
  it("renders default preset correctly", () => {
    const html = renderToString(<DateRangeFilter />);
    expect(html).toContain("Apr 21, 2026 – Apr 27, 2026");
    expect(html).toContain("Filter date range");
  });

  it("renders specified preset when value is provided", () => {
    const html = renderToString(<DateRangeFilter value="today" />);
    expect(html).toContain("Apr 21, 2026");
  });

  it("renders monthly preset when value is month", () => {
    const html = renderToString(<DateRangeFilter value="month" />);
    expect(html).toContain("Apr 1, 2026 – Apr 30, 2026");
  });

  it("has all expected preset identifiers", () => {
    const ids = DATE_RANGE_PRESETS.map((p) => p.id);
    expect(ids).toContain("today");
    expect(ids).toContain("week");
    expect(ids).toContain("month");
    expect(ids).toContain("last30");
    expect(ids).toContain("term");
  });
});
