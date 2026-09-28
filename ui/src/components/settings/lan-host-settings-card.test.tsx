import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { LanHostSettingsCard } from "./lan-host-settings-card";
import type { LanHostStatus, FirewallGuidance, LanUpdateMirrorStatus } from "@/gen/rpc";

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

const mockActiveMirrorStatus: LanUpdateMirrorStatus = {
  mirror_enabled: true,
  storage_dir: "C:\\Users\\User\\AppData\\Local\\Coachify\\update_mirror",
  total_cached_releases: 1,
  total_downloads_served: 4,
  active_update: {
    id: "1.1.0",
    version: "1.1.0",
    channel: "stable",
    artifact_filename: "Coachify-Setup-1.1.0.exe",
    file_size_bytes: 98124000,
    sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
    updater_signature: "deadbeef",
    updater_key_id: "u2026a",
    notes: "Bangladesh offline billing speedups",
    min_upgrade_from: null,
    staged_at: "2026-09-28T12:00:00Z",
    downloads_count: 4,
    is_active: true,
  },
};

const mockEmptyMirrorStatus: LanUpdateMirrorStatus = {
  mirror_enabled: true,
  storage_dir: "C:\\Users\\User\\AppData\\Local\\Coachify\\update_mirror",
  total_cached_releases: 0,
  total_downloads_served: 0,
  active_update: null,
};

const mockDisabledMirrorStatus: LanUpdateMirrorStatus = {
  mirror_enabled: false,
  storage_dir: "C:\\Users\\User\\AppData\\Local\\Coachify\\update_mirror",
  total_cached_releases: 0,
  total_downloads_served: 0,
  active_update: null,
};

describe("LanHostSettingsCard (Spec 0044 & 0048)", () => {
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

  it("renders LAN host update mirror card with active staged release (Spec 0048 AC-7)", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
        initialMirrorStatus={mockActiveMirrorStatus}
      />
    );

    expect(html).toContain("LAN Host Update Mirror");
    expect(html).toContain("Mirror Active");
    expect(html).toContain("Enable LAN Update Mirror");
    expect(html).toContain("Coachify v1.1.0");
    expect(html).toContain("stable");
    expect(html).toContain("4 served");
    expect(html).toContain("9f86d081884c7d65");
    expect(html).toContain("Clear Staged Update");
    expect(html).toContain("Stage Update from File or USB...");
    expect(html).toContain("Bangladesh offline billing speedups");
  });

  it("renders mirror card when enabled with no staged update", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
        initialMirrorStatus={mockEmptyMirrorStatus}
      />
    );

    expect(html).toContain("LAN Host Update Mirror");
    expect(html).toContain("Mirror Active");
    expect(html).toContain("No update installer currently staged");
    expect(html).toContain("Stage Update from File or USB...");
  });

  it("renders mirror card when mirror is disabled", () => {
    const html = renderToString(
      <LanHostSettingsCard
        initialStatus={mockRunningStatus}
        initialGuidance={mockGuidance}
        initialMirrorStatus={mockDisabledMirrorStatus}
      />
    );

    expect(html).toContain("LAN Host Update Mirror");
    expect(html).toContain("Mirror Disabled");
    expect(html).not.toContain("active-staged-update-box");
  });
});
