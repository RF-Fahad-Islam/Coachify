import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { BackupDashboard } from "./backup-dashboard";

describe("BackupDashboard Component", () => {
  it("renders loading state initially", () => {
    const html = renderToString(<BackupDashboard />);
    expect(html).toContain("Loading backup status and storage history...");
  });
});
