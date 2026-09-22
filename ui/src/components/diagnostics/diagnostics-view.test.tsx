import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { DiagnosticsView } from "./diagnostics-view";

describe("DiagnosticsView Component", () => {
  it("renders loading state initially during system diagnostics collection", () => {
    const html = renderToString(<DiagnosticsView />);
    expect(html).toContain("Gathering system metrics and SQLite engine diagnostics...");
  });
});
