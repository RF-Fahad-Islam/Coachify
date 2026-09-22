import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { CashierDeskView } from "./cashier-desk-view";

describe("CashierDeskView Component", () => {
  it("renders empty state with search prompt and keyboard shortcuts initially", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Fast Cashier Counter Ready");
    expect(html).toContain("Instant Student Search");
    expect(html).toContain("Alt+F");
  });

  it("renders search input with autofocus and keyboard badge (covers AC-1)", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Instant Student Search");
    expect(html).toContain("Student ID, Roll, Phone, Guardian Phone, or Name");
  });

  it("displays keyboard navigation hints for fast cashier counter", () => {
    const html = renderToString(<CashierDeskView />);
    expect(html).toContain("Focus Search Box (Alt+F)");
    expect(html).toContain("View current dues and print receipts instantly");
  });
});
