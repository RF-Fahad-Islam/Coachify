import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { CorruptionBanner } from "./corruption-banner";

describe("CorruptionBanner Component", () => {
  it("renders null by default when healthy and not checked", () => {
    const html = renderToString(<CorruptionBanner />);
    expect(html).toBe("");
  });
});
