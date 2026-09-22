import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { DuesCenterView } from "./dues-center-view";

describe("DuesCenterView Component", () => {
  it("renders dues directory filter controls and search box", () => {
    const html = renderToString(<DuesCenterView />);
    expect(html).toContain("Search name, code, phone...");
    expect(html).toContain("All Academic Programs");
    expect(html).toContain("All Batches");
  });

  it("renders summary metric cards for dues breakdown", () => {
    const html = renderToString(<DuesCenterView />);
    expect(html).toContain("Total Outstanding Dues");
    expect(html).toContain("Seriously Overdue");
    expect(html).toContain("Due Students Count");
  });

  it("renders directory table headers and initial empty state", () => {
    const html = renderToString(<DuesCenterView />);
    expect(html).toContain("Student");
    expect(html).toContain("Guardian / Contact");
    expect(html).toContain("Current Month Due");
    expect(html).toContain("Total Outstanding");
    expect(html).toContain("No outstanding dues found matching current filters");
  });
});
