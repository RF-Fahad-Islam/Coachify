import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { LanHostSettingsCard } from "./lan-host-settings-card";
import type { LanHostStatus, FirewallGuidance } from "@/gen/rpc";

const mockRunningStatus: LanHostStatus = {
  is_running: true,
  port: 50443,
  bind_address: "0.0.0.0",
  bound_addresses: ["192.168.1.55", "10.0.0.2"],
  fingerprint:
    "SHA256:4A:8B:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB:CD:EF:12:34:56:78:90:AB",
  active_workstations: 0,
  network_profile: "Private",
  firewall_status: "configured",
  mdns_broadcasting: true,
  license_entitled: true,
  prevent_sleep: true,
  auto_start_on_boot: false,
  uptime_seconds: 3600,
  host_device_id: "host-dev-001",
  hostname: "FRONT-DESK-PC",
};

const mockStoppedStatus: LanHostStatus = {
  ...mockRunningStatus,
  is_running: false,
  uptime_seconds: 0,
  mdns_broadcasting: false,
};

const mockUnentitledStatus: LanHostStatus = {
  ...mockStoppedStatus,
  license_entitled: false,
};

const mockGuidance: FirewallGuidance = {
  profile: "Private",
  is_private: true,
  rule_exists: true,
  powershell_command:
    "Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } | Set-NetConnectionProfile -NetworkCategory Private",
  netsh_command:
    'netsh advfirewall firewall add rule name="Coachify LAN Host" dir=in action=allow protocol=TCP localport=50443 profile=private',
};

const mockPublicGuidance: FirewallGuidance = {
  ...mockGuidance,
  profile: "Public",
  is_private: false,
  rule_exists: false,
};

describe("LanHostSettingsCard (Spec 0044)", () => {
  it("renders running host status with active badges, port, and fingerprint", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
      />
    );

    expect(html).toContain("Authoritative LAN Host Service");
    expect(html).toContain("Running");
    expect(html).toContain("FRONT-DESK-PC");
    expect(html).toContain("TCP :50443");
    expect(html).toContain("192.168.1.55");
    expect(html).toContain("10.0.0.2");
    expect(html).toContain("SHA256:4A:8B:12:34");
    expect(html).toContain("Stop Host");
  });

  it("renders stopped host status with Start button", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockStoppedStatus}
        initialGuidance={mockGuidance}
      />
    );

    expect(html).toContain("Stopped");
    expect(html).toContain("Start Host");
  });

  it("renders license upgrade warning banner when not entitled", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockUnentitledStatus}
        initialGuidance={mockGuidance}
      />
    );

    expect(html).toContain("Multi-PC LAN Hosting Requires License Upgrade");
    expect(html).toContain("Coachify Local Pro");
    expect(html).toContain("Upgrade License");
  });

  it("renders Windows firewall and private network guidance", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
      />
    );

    expect(html).toContain("Profile: Private (Recommended)");
    expect(html).toContain("Firewall: Allowed");
    expect(html).toContain("Windows Command: Allow Port 50443 Through Firewall");
    expect(html).toContain("netsh advfirewall firewall add rule");
  });

  it("renders public network warning and powershell switch command when on Public network", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockPublicGuidance}
      />
    );

    expect(html).toContain("Profile: Public (Blocked by Windows)");
    expect(html).toContain("Firewall: Rule Missing");
    expect(html).toContain("PowerShell Command: Switch Network Category to Private");
    expect(html).toContain("Set-NetConnectionProfile -NetworkCategory Private");
  });

  it("renders discovery scanner and manual ping test cards", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
      />
    );

    expect(html).toContain("LAN Discovery Scanner");
    expect(html).toContain("Scan Network");
    expect(html).toContain("Manual IP Connectivity Ping");
    expect(html).toContain("Test Connection (Ping)");
  });
});
