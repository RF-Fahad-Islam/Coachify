import { describe, it, expect } from "vitest";
import { renderToString } from "react-dom/server";
import { VendorAdminView } from "./vendor-admin-view";
import type { VendorDashboardOverview } from "@/gen/rpc";

const mockOverview: VendorDashboardOverview = {
  active_admin: {
    id: "admin_1",
    email: "ops@coachify.bd",
    name: "Fahad Islam (Ops)",
    role: "owner",
    totp_enabled: true,
    status: "active",
    last_login_at: "2026-09-25T09:00:00Z",
  },
  demo_totp_code: "123456",
  total_customers: 1,
  active_subscriptions: 1,
  in_grace_subscriptions: 0,
  total_revenue_poisha: 1350000,
  audit_chain_valid: true,
  signer_total_signatures: 2,
  customers: [
    {
      id: "cust_mirpur",
      org_name: "Mirpur Academic Care",
      owner_name: "Tariqul Islam",
      phone: "01711001122",
      email: "tariq@mirpurcare.bd",
      district: "Dhaka",
      status: "active",
      subscription_id: "sub_mirpur",
      plan_code: "local_pro",
      plan_name: "Coachify Local Pro (LAN)",
      subscription_status: "active",
      paid_through: "2026-10-15T23:59:59Z",
      grace_days: 14,
      license_id: "LIC-2026-MIRP-0001",
      current_serial: 2,
      bound_installation_id: "INST-MIRPUR-DESK-01",
      pairing_code: "482910",
      transfer_count_12m: 1,
      last_issued_blob: null,
    },
  ],
  payments: [
    {
      id: "pay_seed_1",
      customer_id: "cust_mirpur",
      org_name: "Mirpur Academic Care",
      subscription_id: "sub_mirpur",
      amount_poisha: 1350000,
      method: "bkash",
      reference: "BKASH-8812031",
      proof_url: null,
      period_months: 6,
      invoice_number: "INV-2026-00001",
      recorded_by: "ops@coachify.bd",
      status: "confirmed",
      recorded_at: "2026-04-15T10:00:00Z",
    },
  ],
  installations: [
    {
      id: "inst_row_1",
      installation_id: "INST-MIRPUR-DESK-01",
      customer_id: "cust_mirpur",
      org_name: "Mirpur Academic Care",
      license_id: "LIC-2026-MIRP-0001",
      label: "Front Desk Windows 11 PC",
      app_version: "0.4.2",
      active_students: 280,
      first_seen_at: "2026-04-15T10:05:00Z",
      last_seen_at: "2026-09-24T09:15:00Z",
      status: "bound",
      transfer_count_12m: 1,
    },
  ],
  revocations: [],
  releases: [
    {
      id: "rel_stable_042",
      version: "0.4.2",
      channel: "stable",
      notes: "Vendor admin & activation service",
      artifact_url: "https://downloads.coachify.bd/v0.4.2/Coachify_x64.msi",
      sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      updater_signature: "ed25519:sig042",
      min_upgrade_from: "0.3.0",
      status: "active",
      published_by: "ops@coachify.bd",
      published_at: "2026-09-20T00:00:00Z",
    },
  ],
  admin_audit_log: [
    {
      seq: 1,
      at: "2026-04-15T10:00:00Z",
      admin_user_id: "admin_1",
      admin_email: "ops@coachify.bd",
      action: "payment.recorded",
      entity_type: "license",
      entity_id: "LIC-2026-MIRP-0001",
      after_json: '{"invoice":"INV-2026-00001"}',
      prev_hash: "GENESIS",
      hash: "8a9c42e189f4e2a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5",
    },
  ],
  signer_audit_log: [
    {
      seq: 1,
      signed_at: "2026-04-15T10:00:01Z",
      caller_service_id: "cloud_api",
      operation: "sign_license",
      key_id: "k2026a",
      license_id: "LIC-2026-MIRP-0001",
      installation_id: "INST-MIRPUR-DESK-01",
      serial: 2,
      payload_sha256: "abc123def456",
      signature_prefix: "ed25519:99aa",
    },
  ],
};

describe("VendorAdminView (Spec 0041 Features 63 & 64)", () => {
  it("renders Customers & Payments tab with bKash payment form and invoice ledger (AC-1)", () => {
    const html = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="customers" />
    );

    expect(html).toContain("Vendor Admin &amp; License Issuing Studio");
    expect(html).toContain("Mirpur Academic Care");
    expect(html).toContain("LIC-2026-MIRP-0001");
    expect(html).toContain("INV-2026-00001");
    expect(html).toContain("SHA256 Verified");
  });

  it("renders Offline License Studio tab for REQ-XXXX-XXXX-XXXX-XXXX validation (AC-3, AC-4)", () => {
    const html = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="studio" />
    );

    expect(html).toContain("Air-Gapped &amp; Offline Issuing");
    expect(html).toContain("REQ-XXXX-XXXX-XXXX-XXXX");
    expect(html).toContain("Validate Checksum &amp; Sign .coachifylicense");
  });

  it("renders Device Bindings & Transfers tab with 3/12m cap and 2FA reset button (AC-8)", () => {
    const html = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="installations" />
    );

    expect(html).toContain("INST-MIRPUR-DESK-01");
    expect(html).toContain("Max 3 / 12m");
    expect(html).toContain("Reset Cap (2FA)");
  });

  it("renders Revocations & 2FA tab, Update Channels tab, and Audit & Signer Log tab (AC-5, AC-9, AC-10)", () => {
    const revHtml = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="revocations" />
    );
    expect(revHtml).toContain("Sign Cryptographic Revocation Certificate");

    const relHtml = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="releases" />
    );
    expect(relHtml).toContain("0.4.2");
    expect(relHtml).toContain("stable");

    const auditHtml = renderToString(
      <VendorAdminView initialOverview={mockOverview} defaultTab="audit" />
    );
    expect(auditHtml).toContain("payment.recorded");
    expect(auditHtml).toContain("GENESIS");
    expect(auditHtml).toContain("sign_license");
  });
});
