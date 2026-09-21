import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { MigrationRecoveryScreen } from "./migration-recovery-screen";
import type { RecoveryInfoResponse } from "@/gen/rpc";

describe("MigrationRecoveryScreen Component", () => {
  const mockRecoveryInfo: RecoveryInfoResponse = {
    is_in_recovery: true,
    failed_version: 9,
    error_message:
      "Validation gate failed: Foreign key violation detected in table enrollments",
    active_db_path: "C:\\Users\\User\\AppData\\Local\\Coachify\\dev\\dev.db",
    archive_paths: [
      "C:\\Users\\User\\AppData\\Local\\Coachify\\dev\\pre-migrate-v8-20260920120000.db",
      "C:\\Users\\User\\AppData\\Local\\Coachify\\dev\\pre-migrate-v7-20260919120000.db",
    ],
  };

  it("renders recovery title, failed version, and diagnostic error message", () => {
    const html = renderToString(
      <MigrationRecoveryScreen recoveryInfo={mockRecoveryInfo} />
    );

    expect(html).toContain("Database Migration Recovery");
    expect(html).toContain("Recovery Mode");
    expect(html).toContain("Failed at Migration v9");
    expect(html).toContain("Foreign key violation detected in table enrollments");
    expect(html).toContain("Your live database was not touched by this upgrade");
  });

  it("renders active database path and safety archives list", () => {
    const html = renderToString(
      <MigrationRecoveryScreen recoveryInfo={mockRecoveryInfo} />
    );

    expect(html).toContain("Active Database Location");
    expect(html).toContain("dev.db");
    expect(html).toContain("2 historical backup(s) preserved");
    expect(html).toContain("pre-migrate-v8-20260920120000.db");
    expect(html).toContain("pre-migrate-v7-20260919120000.db");
  });

  it("renders interactive recovery action buttons", () => {
    const html = renderToString(
      <MigrationRecoveryScreen recoveryInfo={mockRecoveryInfo} />
    );

    expect(html).toContain("Export Diagnostics");
    expect(html).toContain("Open Database Folder");
    expect(html).toContain("Retry Upgrade");
    expect(html).toContain("export-diagnostics-button");
    expect(html).toContain("open-db-folder-button");
  });

  it("handles empty archive list gracefully", () => {
    const infoNoArchives: RecoveryInfoResponse = {
      is_in_recovery: true,
      failed_version: null,
      error_message: "Low disk storage space",
      active_db_path: "C:\\data\\dev.db",
      archive_paths: [],
    };

    const html = renderToString(
      <MigrationRecoveryScreen recoveryInfo={infoNoArchives} />
    );

    expect(html).toContain("No prior archives found in folder");
    expect(html).toContain("Low disk storage space");
  });
});
