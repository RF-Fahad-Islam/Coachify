import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { RestoreWizard } from "./restore-wizard";

describe("RestoreWizard Component", () => {
  it("renders closed dialog as empty or unmounted markup", () => {
    const html = renderToString(<RestoreWizard open={false} onOpenChange={() => {}} />);
    expect(html).toBe("");
  });

  it("renders initial inspection step without crashing when opened", () => {
    const html = renderToString(<RestoreWizard open={true} onOpenChange={() => {}} />);
    expect(html).toBeDefined();
  });
});
