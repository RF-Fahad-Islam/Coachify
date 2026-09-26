import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { DeviceManagement } from "./device-management";
import type { WorkstationSummary, WorkstationQuota } from "@/gen/rpc";

const mockDevices: WorkstationSummary[] = [
  {
    id: "dev-host-001",
    name: "Server Host",
    windows_name: "DESKTOP-HOST",
    ip_address: "192.168.1.100",
    os_version: "Windows 11 Pro",
    app_version: "0.1.0",
    status: "active",
    is_host: true,
    paired_at: "2026-09-26T10:00:00Z",
    last_seen_at: "2026-09-26T10:00:00Z",
    blocked_at: null,
    revoked_at: null,
    revocation_reason: null,
    is_online: true,
    active_sessions_count: 1,
    paired_via: "code",
    platform: "windows",
    device_type: "workstation",
  },
  {
    id: "dev-client-002",
    name: "Front Desk PC",
    windows_name: "WORKSTATION-01",
    ip_address: "192.168.1.105",
    os_version: "Windows 10 Enterprise",
    app_version: "0.1.0",
    status: "active",
    is_host: false,
    paired_at: "2026-09-26T10:15:00Z",
    last_seen_at: "2026-09-26T10:19:00Z",
    blocked_at: null,
    revoked_at: null,
    revocation_reason: null,
    is_online: true,
    active_sessions_count: 2,
    paired_via: "code",
    platform: "windows",
    device_type: "workstation",
  },
  {
    id: "dev-client-003",
    name: "Accounts Desk PC",
    windows_name: "WORKSTATION-02",
    ip_address: "192.168.1.106",
    os_version: "Windows 11 Home",
    app_version: "0.1.0",
    status: "blocked",
    is_host: false,
    paired_at: "2026-09-26T10:20:00Z",
    last_seen_at: "2026-09-26T10:22:00Z",
    blocked_at: "2026-09-26T10:25:00Z",
    revoked_at: null,
    revocation_reason: null,
    is_online: false,
    active_sessions_count: 0,
    paired_via: "code",
    platform: "windows",
    device_type: "workstation",
  },
  {
    id: "dev-mobile-004",
    name: "Teacher Tablet",
    windows_name: "IPAD-PRO-TEACHER",
    ip_address: "192.168.1.150",
    os_version: "iOS 18.2",
    app_version: "0.1.0",
    status: "active",
    is_host: false,
    paired_at: "2026-09-26T11:00:00Z",
    last_seen_at: "2026-09-26T11:05:00Z",
    blocked_at: null,
    revoked_at: null,
    revocation_reason: null,
    is_online: true,
    active_sessions_count: 1,
    paired_via: "qr",
    platform: "ios",
    device_type: "tablet",
  },
];

const mockQuota: WorkstationQuota = {
  active_workstations: 2,
  workstation_limit: 5,
  seats_remaining: 3,
  license_tier: "Professional",
};

const mockFullQuota: WorkstationQuota = {
  active_workstations: 5,
  workstation_limit: 5,
  seats_remaining: 0,
  license_tier: "Professional",
};

describe("DeviceManagement Component (Spec 0045)", () => {
  it("renders workstation inventory list and seat allocation quota", () => {
    const html = renderToString(
      <DeviceManagement initialDevices={mockDevices} initialQuota={mockQuota} />
    );

    // Header & Titles
    expect(html).toContain("Workstations &amp; Devices");
    expect(html).toContain("Pair New Workstation");

    // Quota Progress Meter
    expect(html).toContain("Seats Allocated");
    expect(html).toContain("Professional");
    expect(html).toContain("2 of 5 seats used");

    // Device Rows
    expect(html).toContain("Server Host");
    expect(html).toContain("DESKTOP-HOST");
    expect(html).toContain("192.168.1.100");
    expect(html).toContain("Host Server");

    expect(html).toContain("Front Desk PC");
    expect(html).toContain("WORKSTATION-01");
    expect(html).toContain("192.168.1.105");

    expect(html).toContain("Accounts Desk PC");
    expect(html).toContain("WORKSTATION-02");
    expect(html).toContain("blocked");
  });

  it("disables pairing button and warns when license quota is reached", () => {
    const html = renderToString(
      <DeviceManagement initialDevices={mockDevices} initialQuota={mockFullQuota} />
    );

    expect(html).toContain("5 of 5 seats used");
    expect(html).toContain("Workstation limit reached for your license tier");
    expect(html).toContain("disabled");
  });

  it("renders mobile companion pairing button and mobile device badges (Spec 0046)", () => {
    const html = renderToString(
      <DeviceManagement initialDevices={mockDevices} initialQuota={mockQuota} />
    );

    // Pair Mobile Companion button
    expect(html).toContain("Pair Mobile Companion");

    // Mobile device row with QR badge and platform badge
    expect(html).toContain("Teacher Tablet");
    expect(html).toContain("QR");
    expect(html).toContain("ios");
  });
});
