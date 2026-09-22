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
});
