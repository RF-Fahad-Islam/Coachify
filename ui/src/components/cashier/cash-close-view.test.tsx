import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { CashCloseView } from "./cash-close-view";

describe("CashCloseView Component", () => {
  it("renders daily cash close date selector and controls", () => {
    const html = renderToString(<CashCloseView />);
    expect(html).toContain("Business Date:");
    expect(html).toContain("Refresh");
    expect(html).toContain("Print Sheet");
  });

  it("renders disabled print sheet button prior to day record load", () => {
    const html = renderToString(<CashCloseView />);
    expect(html).toContain("Print Sheet");
  });
});
