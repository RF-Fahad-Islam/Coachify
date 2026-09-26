import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { WorkstationSettingsCard } from "./workstation-settings-card";
import type {
  WorkstationClientStatus,
  SystemPrinterInfo,
  WorkstationPrinterSettings,
} from "@/gen/rpc";

const mockConnectedStatus: WorkstationClientStatus = {
  is_client_mode: true,
  device_id: "ws-device-042",
  workstation_name: "Cashier Desk 2",
  host_name: "MAIN-SERVER-PC",
  host_ip: "192.168.1.100",
  host_port: 50443,
  state: "connected",
  rtt_ms: 8,
  last_heartbeat_at: "2026-09-26T12:00:00Z",
  paired_at: "2026-09-26T10:00:00Z",
};

const mockServerModeStatus: WorkstationClientStatus = {
  ...mockConnectedStatus,
  is_client_mode: false,
};

const mockPrinters: SystemPrinterInfo[] = [
  {
    name: "EPSON TM-T88VI Thermal",
    is_default: true,
    status: "ready",
  },
  {
    name: "HP LaserJet Pro M404n",
    is_default: false,
    status: "ready",
  },
];

const mockPrinterSettings: WorkstationPrinterSettings = {
  printer_name: "EPSON TM-T88VI Thermal",
  page_size: "thermal_80mm",
  paper_source: null,
  copies: 1,
  auto_print_on_payment: true,
  open_cash_drawer: true,
  header_title: "Apex Academy",
  footer_note: "Thank you",
  updated_at: "2026-09-26T12:00:00Z",
};

describe("WorkstationSettingsCard (Spec 0047)", () => {
  it("returns null when not in workstation client mode", () => {
    const html = renderToString(
      <WorkstationSettingsCard initialStatus={mockServerModeStatus} />
    );
    expect(html).toBe("");
  });

  it("renders workstation card with host identity and connection details", () => {
    const html = renderToString(
      <WorkstationSettingsCard
        initialStatus={mockConnectedStatus}
        initialPrinters={mockPrinters}
        initialPrinterSettings={mockPrinterSettings}
      />
    );

    expect(html).toContain("Workstation Client Mode");
    expect(html).toContain("Cashier Desk 2");
    expect(html).toContain("MAIN-SERVER-PC");
    expect(html).toContain("192.168.1.100");
    expect(html).toContain("50443");
    expect(html).toContain("8 ms");
    expect(html).toContain("connected");
  });

  it("renders local physical printer selectors strictly isolated to this PC", () => {
    const html = renderToString(
      <WorkstationSettingsCard
        initialStatus={mockConnectedStatus}
        initialPrinters={mockPrinters}
        initialPrinterSettings={mockPrinterSettings}
      />
    );

    expect(html).toContain("Local Workstation Printer Setup");
    expect(html).toContain("Default Receipt Printer (POS Thermal)");
    expect(html).toContain("EPSON TM-T88VI Thermal");
    expect(html).toContain("HP LaserJet Pro M404n");
    expect(html).toContain("Save Printer Settings");
  });

  it("renders diagnostic test connection and disconnect actions", () => {
    const html = renderToString(
      <WorkstationSettingsCard
        initialStatus={mockConnectedStatus}
        initialPrinters={mockPrinters}
        initialPrinterSettings={mockPrinterSettings}
      />
    );

    expect(html).toContain("Ping &amp; Test Host");
    expect(html).toContain("Unpair Workstation");
  });
});
