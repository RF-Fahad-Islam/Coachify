import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { WorkstationPill } from "./workstation-pill";
import { ReconnectionBanner } from "./reconnection-banner";
import { WorkstationLockoutOverlay } from "@/components/auth/workstation-lockout-overlay";
import type { WorkstationClientStatus } from "@/gen/rpc";

const mockConnectedStatus: WorkstationClientStatus = {
  is_client_mode: true,
  device_id: "ws-001",
  workstation_name: "Front Desk Cashier 1",
  host_name: "MAIN-SERVER-PC",
  host_ip: "192.168.1.100",
  host_port: 50443,
  state: "connected",
  rtt_ms: 12,
  last_heartbeat_at: "2026-09-26T12:00:00Z",
  paired_at: "2026-09-26T10:00:00Z",
};

const mockReconnectingStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  state: "reconnecting",
  rtt_ms: null,
};

const mockConnectingStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  state: "connecting",
  rtt_ms: null,
};

const mockDisconnectedStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  state: "disconnected",
  rtt_ms: null,
};

const mockRevokedStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  state: "revoked",
  rtt_ms: null,
};

const mockBlockedStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  state: "blocked",
  rtt_ms: null,
};

const mockServerModeStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  is_client_mode: false,
};

describe("WorkstationPill and Workstation Network States (Spec 0047)", () => {
  describe("WorkstationPill", () => {
    it("returns null when not in client mode", () => {
      const html = renderToString(
        <WorkstationPill initialStatus={mockServerModeStatus} />
      );
      expect(html).toBe("");
    });

    it("returns null when status is null", () => {
      const html = renderToString(<WorkstationPill initialStatus={null} />);
      expect(html).toBe("");
    });

    it("renders connected status with RTT latency", () => {
      const html = renderToString(
        <WorkstationPill initialStatus={mockConnectedStatus} />
      );
      expect(html).toContain("Workstation • 12ms");
      expect(html).toContain("bg-emerald-500");
    });

    it("renders reconnecting status with ping animation", () => {
      const html = renderToString(
        <WorkstationPill initialStatus={mockReconnectingStatus} />
      );
      expect(html).toContain("Reconnecting...");
      expect(html).toContain("animate-ping");
      expect(html).toContain("bg-amber-500");
    });

    it("renders connecting status with ping animation", () => {
      const html = renderToString(
        <WorkstationPill initialStatus={mockConnectingStatus} />
      );
      expect(html).toContain("Connecting...");
      expect(html).toContain("animate-ping");
    });

    it("renders offline disconnected status", () => {
      const html = renderToString(
        <WorkstationPill initialStatus={mockDisconnectedStatus} />
      );
      expect(html).toContain("Offline");
      expect(html).toContain("bg-slate-400");
    });

    it("renders revoked status", () => {
      const html = renderToString(<WorkstationPill initialStatus={mockRevokedStatus} />);
      expect(html).toContain("Revoked");
      expect(html).toContain("bg-red-500");
    });

    it("renders blocked status", () => {
      const html = renderToString(<WorkstationPill initialStatus={mockBlockedStatus} />);
      expect(html).toContain("Blocked");
      expect(html).toContain("bg-red-500");
    });
  });

  describe("ReconnectionBanner", () => {
    it("returns null when connected or not in client mode", () => {
      const htmlConnected = renderToString(
        <ReconnectionBanner initialStatus={mockConnectedStatus} />
      );
      expect(htmlConnected).toBe("");

      const htmlServer = renderToString(
        <ReconnectionBanner initialStatus={mockServerModeStatus} />
      );
      expect(htmlServer).toBe("");
    });

    it("renders warning banner with host IP and Retry button during reconnection", () => {
      const html = renderToString(
        <ReconnectionBanner initialStatus={mockReconnectingStatus} />
      );
      expect(html).toContain("Connection to host server lost (192.168.1.100:50443)");
      expect(html).toContain("Retry Now");
    });
  });

  describe("WorkstationLockoutOverlay", () => {
    it("returns null when connection is active and healthy", () => {
      const html = renderToString(
        <WorkstationLockoutOverlay initialStatus={mockConnectedStatus} />
      );
      expect(html).toBe("");
    });

    it("renders full-screen lockout modal when revoked by host administrator", () => {
      const html = renderToString(
        <WorkstationLockoutOverlay initialStatus={mockRevokedStatus} />
      );
      expect(html).toContain("Workstation Decommissioned by Administrator");
      expect(html).toContain("Unpair &amp; Reset Workstation");
      expect(html).toContain('role="alertdialog"');
    });

    it("renders lockout modal when blocked by host administrator", () => {
      const html = renderToString(
        <WorkstationLockoutOverlay initialStatus={mockBlockedStatus} />
      );
      expect(html).toContain("Workstation Access Temporarily Suspended");
      expect(html).toContain("Unpair &amp; Reset Workstation");
    });
  });
});
