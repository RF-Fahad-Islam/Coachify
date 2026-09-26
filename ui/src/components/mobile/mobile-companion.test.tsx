import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { parsePairingUri, MobilePairingScreen } from "./mobile-pairing-screen";
import { MobileAttendanceScanner } from "./mobile-attendance-scanner";
import { MobileCashierView } from "./mobile-cashier-view";
import { MobileShell } from "./mobile-shell";

describe("Mobile Companion Subsystem (Spec 0046)", () => {
  describe("QR Pairing URI Parser", () => {
    it("parses valid coachify://pair URI correctly", () => {
      const uri =
        "coachify://pair?host=192.168.1.100,10.0.0.5&port=50443&fp=abc123sha256fingerprint&sec=random256bitsecret&inst=Apex+Academy&exp=1790424000";
      const parsed = parsePairingUri(uri);

      expect(parsed).not.toBeNull();
      expect(parsed?.hosts).toEqual(["192.168.1.100", "10.0.0.5"]);
      expect(parsed?.port).toBe(50443);
      expect(parsed?.fingerprint).toBe("abc123sha256fingerprint");
      expect(parsed?.secret).toBe("random256bitsecret");
      expect(parsed?.institution).toBe("Apex Academy");
      expect(parsed?.expiresEpoch).toBe(1790424000);
    });

    it("rejects non-coachify scheme or malformed URI", () => {
      expect(parsePairingUri("https://google.com")).toBeNull();
      expect(parsePairingUri("coachify://invalid")).toBeNull();
      expect(parsePairingUri("coachify://pair?port=invalid")).toBeNull();
      expect(parsePairingUri("coachify://pair?host=&port=50443&fp=&sec=")).toBeNull();
    });
  });

  describe("MobilePairingScreen Component", () => {
    it("renders camera viewfinder and pairing connect input", () => {
      const html = renderToString(
        <MobilePairingScreen initialDeviceName="Field Tablet 1" />
      );

      expect(html).toContain("scanner-viewfinder");
      expect(html).toContain("TLS 1.3 Pinning Enforced");
      expect(html).toContain("qr-uri-input");
      expect(html).toContain("Field Tablet 1");
    });
  });

  describe("MobileAttendanceScanner Component (AC-6)", () => {
    it("renders high speed optical viewfinder with SLA badge", () => {
      const html = renderToString(<MobileAttendanceScanner />);

      expect(html).toContain("attendance-viewfinder");
      expect(html).toContain("Scanning Student Badge");
      expect(html).toContain("scan-sla-badge");
      expect(html).toContain("Recent Scans");
    });
  });

  describe("MobileCashierView Component (AC-7)", () => {
    it("renders fee collection search and payment channel options", () => {
      const html = renderToString(
        <MobileCashierView designatedPrinter="Epson TM-T88VI" />
      );

      expect(html).toContain("Mobile Fee Collection");
      expect(html).toContain("student-search-input");
    });
  });

  describe("MobileShell Component (AC-8, AC-9)", () => {
    it("renders mobile navigation bar with Attendance, Cashier, and Pairing tabs", () => {
      const html = renderToString(
        <MobileShell
          initialHostIp="192.168.1.100"
          initialHostPort={50443}
          initialDeviceToken="dt_test_token"
        />
      );

      expect(html).toContain("Coachify Mobile");
      expect(html).toContain("192.168.1.100:50443");
      expect(html).toContain("tab-scanner-btn");
      expect(html).toContain("tab-cashier-btn");
      expect(html).toContain("tab-pairing-btn");
    });
  });
});
