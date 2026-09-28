import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { UpdateSettingsCard } from "./update-settings";
import type { WorkstationClientStatus, WorkstationUpdateCheckResult } from "@/gen/rpc";

describe("UpdateSettingsCard Component (Spec 0048 AC-8)", () => {
  it("renders version and pre-update policy details in standalone mode", () => {
    const html = renderToString(<UpdateSettingsCard />);
    expect(html).toContain("Coachify v0.1.0-alpha");
    expect(html).toContain("Automatic Pre-Update Snapshot");
    expect(html).toContain("Data Preserved on Uninstall");
    expect(html).toContain("Check for Updates");
  });

  it("renders LAN update source banner and Host update controls when in client mode", () => {
    const mockClientStatus: WorkstationClientStatus = {
      is_client_mode: true,
      state: "connected",
      host_id: "host_apex",
      host_name: "Apex Main PC",
      host_ip: "192.168.1.100",
      host_port: 50443,
      device_id: "dev_ws1",
      workstation_name: "Counter 2",
      latency_ms: 2,
    };

    const html = renderToString(
      <UpdateSettingsCard initialClientStatus={mockClientStatus} />
    );

    expect(html).toContain("Update Source: Host PC (Apex Main PC) via LAN");
    expect(html).toContain("Air-Gapped LAN Delivery");
    expect(html).toContain("Air-Gapped LAN Peer");
    expect(html).toContain("Check Host for Updates");
  });

  it("renders update available from Host prompt with download button", () => {
    const mockClientStatus: WorkstationClientStatus = {
      is_client_mode: true,
      state: "connected",
      host_id: "host_apex",
      host_name: "Apex Main PC",
      host_ip: "192.168.1.100",
      host_port: 50443,
    };

    const mockCheckResult: WorkstationUpdateCheckResult = {
      update_available: true,
      current_version: "1.0.0",
      host_version: "1.1.0",
      manifest: {
        version: "1.1.0",
        channel: "stable",
        file_size_bytes: 98124000,
        sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        updater_signature: "sig123",
        updater_key_id: "u2026a",
        notes: "Major coaching center speedups",
        download_path: "/updates/download/1.1.0",
        artifact_filename: "Coachify-Setup-1.1.0.exe",
      },
    };

    const html = renderToString(
      <UpdateSettingsCard
        initialClientStatus={mockClientStatus}
        initialCheckResult={mockCheckResult}
      />
    );

    expect(html).toContain("Update v1.1.0 Available from Host");
    expect(html).toContain("Ready on LAN");
    expect(html).toContain("Download from Host");
    expect(html).toContain("Major coaching center speedups");
  });
});
