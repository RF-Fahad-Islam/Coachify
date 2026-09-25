import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import {
  LicenseSettingsCard,
  ReadOnlyBanner,
  MovedLicenseModal,
} from "./license-settings-card";
import type { LicenseStatusResponse, RequestCodeResponse } from "@/gen/rpc";

const mockActiveStatus: LicenseStatusResponse = {
  status: "active",
  tier: "local_pro",
  customer_name: "Sunrise Coaching Center",
  license_id: "LIC-2026-9001",
  installation_id: "inst-host-001",
  paid_through: "2027-09-25",
  valid_until: "2027-10-09",
  days_remaining: 365,
  grace_days: 14,
  is_read_only: false,
  is_moved: false,
  moved_days_remaining: null,
  request_code: "REQ-ABCD-EFGH-JKLM-NPQR",
  active_students: 145,
  student_limit: 500,
  workstation_limit: 5,
  capabilities: ["core", "lan", "sms"],
};

const mockRequestCode: RequestCodeResponse = {
  installation_id: "inst-host-001",
  request_code: "REQ-ABCD-EFGH-JKLM-NPQR",
  qr_payload: "COACHIFY-REQ|v1|inst-host-001|0123456789abcdef",
};

describe("LicenseSettingsCard & ReadOnlyBanner (Spec 0037)", () => {
  it("renders license status, plan tier, offline request code, and QR preview", () => {
    const html = renderToString(
      <LicenseSettingsCard
        initialStatus={mockActiveStatus}
        initialRequestInfo={mockRequestCode}
        defaultShowQr={true}
      />
    );

    expect(html).toContain("Sunrise Coaching Center");
    expect(html).toContain("Coachify Local Pro (LAN)");
    expect(html).toContain("REQ-ABCD-EFGH-JKLM-NPQR");
    expect(html).toContain("COACHIFY-REQ|v1|inst-host-001|0123456789abcdef");
    expect(html).toContain("Import Offline License File (.coachifylicense)");
  });

  it("renders ReadOnlyBanner when license is expired_past_grace", () => {
    const expiredStatus: LicenseStatusResponse = {
      ...mockActiveStatus,
      status: "expired_past_grace",
      is_read_only: true,
    };

    const html = renderToString(<ReadOnlyBanner status={expiredStatus} />);
    expect(html).toContain("Read-Only Mode:");
    expect(html).toContain(
      "All student searches, reports, receipt reprints, data exports, and backups remain open"
    );
  });

  it("renders ReadOnlyBanner in 14-day grace period and moved_transitional states", () => {
    const graceStatus: LicenseStatusResponse = {
      ...mockActiveStatus,
      status: "in_grace",
      days_remaining: -4,
    };
    const graceHtml = renderToString(<ReadOnlyBanner status={graceStatus} />);
    expect(graceHtml).toContain("14-Day Grace Period Active:");

    const movedStatus: LicenseStatusResponse = {
      ...mockActiveStatus,
      status: "moved_transitional",
      is_moved: true,
      moved_days_remaining: 14,
    };
    const movedHtml = renderToString(<ReadOnlyBanner status={movedStatus} />);
    expect(movedHtml).toContain("Activate this PC:");
    expect(movedHtml).toContain("14 days");
    expect(movedHtml).toContain("REQ-ABCD-EFGH-JKLM-NPQR");
  });

  it("renders MovedLicenseModal details for replacement PC activation", () => {
    const movedStatus: LicenseStatusResponse = {
      ...mockActiveStatus,
      status: "moved_transitional",
      is_moved: true,
      moved_days_remaining: 12,
    };

    const html = renderToString(
      <MovedLicenseModal open={true} onOpenChange={() => {}} status={movedStatus} />
    );
    expect(html).toContain("12-day full-access transitional window");
    expect(html).toContain("REQ-ABCD-EFGH-JKLM-NPQR");
  });

  it("renders Online Activation & 6-Digit Pairing section and revoked ReadOnlyBanner (Spec 0041)", () => {
    const cardHtml = renderToString(
      <LicenseSettingsCard
        initialStatus={mockActiveStatus}
        initialRequestInfo={mockRequestCode}
      />
    );
    expect(cardHtml).toContain("Online Activation, 6-Digit Pairing &amp; Cloud Refresh");
    expect(cardHtml).toContain("Activate Online (/v1/activate)");
    expect(cardHtml).toContain("Check Now (/v1/refresh)");

    const revokedStatus: LicenseStatusResponse = {
      ...mockActiveStatus,
      status: "revoked",
      is_read_only: true,
    };
    const bannerHtml = renderToString(<ReadOnlyBanner status={revokedStatus} />);
    expect(bannerHtml).toContain("Read-Only Mode:");
    expect(bannerHtml).toContain("revoked");
  });
});
