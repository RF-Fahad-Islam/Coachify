import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { ThemeProvider } from "@/theme/theme-provider";
import { AuthProvider } from "@/contexts/auth-context";
import { CommunicationView } from "./communication-view";
import { OutboxMonitorView } from "./outbox-monitor-view";
import { BatchBroadcastView } from "./batch-broadcast-view";
import { DueRemindersView } from "./due-reminders-view";
import { CommunicationSettings } from "../settings/communication-settings";

describe("Communication and Messaging UI Components", () => {
  describe("CommunicationView", () => {
    it("renders page title and tabs for outbox monitor, broadcast, and due reminders", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CommunicationView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Communication &amp; Messaging");
      expect(html).toContain("Outbox Monitor");
      expect(html).toContain("Batch Broadcast");
      expect(html).toContain("Due Reminders");
    });
  });

  describe("OutboxMonitorView", () => {
    it("renders metric cards, filter controls, and queue flush trigger", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <OutboxMonitorView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Queued");
      expect(html).toContain("Delivered");
      expect(html).toContain("Uncertain");
      expect(html).toContain("Failed");
      expect(html).toContain("Cancelled");
      expect(html).toContain("All Channels");
      expect(html).toContain("All Statuses");
      expect(html).toContain("Send All Pending");
      expect(html).toContain("Outbox Queue Records");
    });
  });

  describe("BatchBroadcastView", () => {
    it("renders channel selector, target batch picker, template selector, and segment math card", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <BatchBroadcastView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Target Batch:");
      expect(html).toContain("SMS");
      expect(html).toContain("WhatsApp");
      expect(html).toContain("Email");
      expect(html).toContain("Compose Broadcast Message");
      expect(html).toContain("Live Segment &amp; Encoding Math");
      expect(html).toContain("Available Placeholders:");
      expect(html).toContain("Ready to Dispatch");
      expect(html).toContain("Missing Contact Info");
    });
  });

  describe("DueRemindersView", () => {
    it("renders candidate filters, deduplication guard banner, and dispatch panel", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <DueRemindersView />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Batch:");
      expect(html).toContain("Period:");
      expect(html).toContain("Min Due (৳):");
      expect(html).toContain("Period Deduplication Guard:");
      expect(html).toContain("Dispatch Channel:");
      expect(html).toContain("Enqueue Due Reminders");
      expect(html).toContain("Candidate Review List");
    });
  });

  describe("CommunicationSettings", () => {
    it("renders gateway providers, template catalog, and automation rule tabs", () => {
      const html = renderToString(
        <ThemeProvider>
          <AuthProvider>
            <CommunicationSettings />
          </AuthProvider>
        </ThemeProvider>
      );

      expect(html).toContain("Messaging &amp; Gateway Settings");
      expect(html).toContain("Gateway Providers");
      expect(html).toContain("Templates Catalog");
      expect(html).toContain("Automation Rules");
      expect(html).toContain("Cellular SMS Gateway");
      expect(html).toContain("WhatsApp OpenWA Companion");
      expect(html).toContain("SMTP Transactional Email Provider");
    });
  });
});
